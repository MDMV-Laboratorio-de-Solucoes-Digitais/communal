//! Tier 2 LFR Benchmark Testing with Ground Truth
//!
//! Reads graph files and ground truth from disk, runs Leiden algorithm,
//! and computes NMI/ARI against the planted partition.

#![expect(
    clippy::cast_precision_loss,
    clippy::manual_midpoint,
    clippy::float_cmp,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    reason = "Example code has casts, float comparisons, and format strings"
)]

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use std::collections::HashMap;
use std::fs;

const BENCHMARK_DIR: &str = "benchmarks/lfr_graphs";

/// Compute Normalized Mutual Information between two partitions
fn compute_nmi(partition1: &[u32], partition2: &[u32]) -> f64 {
    if partition1.len() != partition2.len() {
        return 0.0;
    }
    let n = partition1.len() as f64;
    
    // Build contingency table
    let mut contingency: HashMap<(u32, u32), f64> = HashMap::new();
    for (&c1, &c2) in partition1.iter().zip(partition2.iter()) {
        *contingency.entry((c1, c2)).or_insert(0.0) += 1.0;
    }
    
    // Row and column sums
    let mut row_sums: HashMap<u32, f64> = HashMap::new();
    let mut col_sums: HashMap<u32, f64> = HashMap::new();
    for (&(c1, c2), &count) in &contingency {
        *row_sums.entry(c1).or_insert(0.0) += count;
        *col_sums.entry(c2).or_insert(0.0) += count;
    }
    
    // Mutual information
    let mut mi = 0.0;
    for (&(c1, c2), &count) in &contingency {
        let row_sum = row_sums[&c1];
        let col_sum = col_sums[&c2];
        if count > 0.0 {
            mi += (count / n) * ((n * count) / (row_sum * col_sum)).ln();
        }
    }
    
    // Entropies
    let h1: f64 = row_sums.values().map(|&a| {
        if a > 0.0 { -(a / n) * (a / n).ln() } else { 0.0 }
    }).sum();
    let h2: f64 = col_sums.values().map(|&b| {
        if b > 0.0 { -(b / n) * (b / n).ln() } else { 0.0 }
    }).sum();
    
    if h1 + h2 == 0.0 {
        1.0
    } else {
        2.0 * mi / (h1 + h2)
    }
}

/// Compute Adjusted Rand Index
fn compute_ari(partition1: &[u32], partition2: &[u32]) -> f64 {
    if partition1.len() != partition2.len() {
        return 0.0;
    }
    let n = partition1.len() as f64;
    
    // Build contingency table
    let mut contingency: HashMap<(u32, u32), f64> = HashMap::new();
    for (&c1, &c2) in partition1.iter().zip(partition2.iter()) {
        *contingency.entry((c1, c2)).or_insert(0.0) += 1.0;
    }
    
    // Row and column sums
    let mut row_sums: HashMap<u32, f64> = HashMap::new();
    let mut col_sums: HashMap<u32, f64> = HashMap::new();
    for (&(c1, c2), &count) in &contingency {
        *row_sums.entry(c1).or_insert(0.0) += count;
        *col_sums.entry(c2).or_insert(0.0) += count;
    }
    
    // ARI components
    let mut sum_comb = 0.0;
    for &count in contingency.values() {
        if count >= 2.0 {
            sum_comb += count * (count - 1.0) / 2.0;
        }
    }
    
    let mut sum_rows = 0.0;
    for &a in row_sums.values() {
        if a >= 2.0 {
            sum_rows += a * (a - 1.0) / 2.0;
        }
    }
    
    let mut sum_cols = 0.0;
    for &b in col_sums.values() {
        if b >= 2.0 {
            sum_cols += b * (b - 1.0) / 2.0;
        }
    }
    
    let expected = sum_rows * sum_cols / (n * (n - 1.0) / 2.0);
    let max_index = (sum_rows + sum_cols) / 2.0;
    
    if max_index == expected {
        1.0
    } else {
        (sum_comb - expected) / (max_index - expected)
    }
}

/// Parse edge list file
fn parse_edge_file(path: &str) -> Result<(Vec<(u32, u32)>, usize), GraphError> {
    let content = fs::read_to_string(path).map_err(|e| GraphError::InvalidGraph {
        reason: format!("Failed to read {}: {}", path, e),
    })?;
    
    let mut edges = Vec::new();
    let mut max_node = 0u32;
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(GraphError::InvalidGraph {
                reason: format!("{} line {}: expected at least 2 values", path, line_num + 1),
            });
        }
        let from: u32 = parts[0].parse().map_err(|_| GraphError::InvalidGraph {
            reason: format!("{} line {}: invalid source node", path, line_num + 1),
        })?;
        let to: u32 = parts[1].parse().map_err(|_| GraphError::InvalidGraph {
            reason: format!("{} line {}: invalid target node", path, line_num + 1),
        })?;
        max_node = max_node.max(from).max(to);
        edges.push((from, to));
    }
    
    Ok((edges, (max_node + 1) as usize))
}

/// Parse ground truth file
fn parse_ground_truth(path: &str) -> Result<Vec<u32>, GraphError> {
    let content = fs::read_to_string(path).map_err(|e| GraphError::InvalidGraph {
        reason: format!("Failed to read {}: {}", path, e),
    })?;
    
    let mut membership = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(GraphError::InvalidGraph {
                reason: format!("{} line {}: expected node_id community_id", path, line_num + 1),
            });
        }
        let node: u32 = parts[0].parse().map_err(|_| GraphError::InvalidGraph {
            reason: format!("{} line {}: invalid node id", path, line_num + 1),
        })?;
        let community: u32 = parts[1].parse().map_err(|_| GraphError::InvalidGraph {
            reason: format!("{} line {}: invalid community id", path, line_num + 1),
        })?;
        
        // Ensure membership vector is large enough
        while membership.len() <= node as usize {
            membership.push(0);
        }
        membership[node as usize] = community;
    }
    
    Ok(membership)
}

/// Run a single benchmark
fn run_benchmark(name: &str, edge_file: &str, gt_file: &str, expected_nmi: f64) {
    println!("=== {} ===", name);
    
    // Parse edge list
    let (edges, node_count) = match parse_edge_file(edge_file) {
        Ok(result) => result,
        Err(e) => {
            println!("  ERROR parsing edges: {}", e);
            return;
        }
    };
    
    // Parse ground truth
    let ground_truth = match parse_ground_truth(gt_file) {
        Ok(result) => result,
        Err(e) => {
            println!("  ERROR parsing ground truth: {}", e);
            return;
        }
    };
    
    // Build graph
    let weighted_edges: Vec<(u32, u32, f64)> = edges.iter().map(|&(u, v)| (u, v, 1.0)).collect();
    let graph = CsrGraph::from_edges(&weighted_edges, node_count);
    
    println!("  Nodes: {}, Edges: {}", node_count, edges.len());
    println!("  Ground truth communities: {}", {
        let mut unique = ground_truth.clone();
        unique.sort_unstable();
        unique.dedup();
        unique.len()
    });
    
    // Run Leiden
    let config = LeidenConfig {
        seed: Some(42),
        ..LeidenConfig::default()
    };
    let detector = Leiden::new(config);
    
    match detector.detect(&graph) {
        Ok(partition) => {
            let membership = partition.membership_vec();
            
            // Compute NMI and ARI
            let nmi = compute_nmi(membership, &ground_truth);
            let ari = compute_ari(membership, &ground_truth);
            
            println!("  Detected communities: {}", partition.community_count());
            println!("  Quality Q: {:.4}", partition.quality_score());
            println!("  NMI: {:.4} (expected > {:.2})", nmi, expected_nmi);
            println!("  ARI: {:.4}", ari);
            
            if nmi >= expected_nmi {
                println!("  ✓ PASS: NMI >= {:.2}", expected_nmi);
            } else {
                println!("  ✗ FAIL: NMI < {:.2}", expected_nmi);
            }
        }
        Err(e) => {
            println!("  ERROR: {}", e);
        }
    }
    println!();
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  TIER 2 LFR BENCHMARK TESTING WITH GROUND TRUTH            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    let benchmarks = vec![
        ("LFR N=1000 μ=0.1 (easy)", "LFR_N1000_mu0.1", 0.90),
        ("LFR N=1000 μ=0.3 (medium)", "LFR_N1000_mu0.3", 0.70),
        ("LFR N=1000 μ=0.5 (hard)", "LFR_N1000_mu0.5", 0.50),
        ("LFR N=1000 μ=0.7 (very hard)", "LFR_N1000_mu0.7", 0.30),
        ("LFR N=5000 μ=0.3 (scaling)", "LFR_N5000_mu0.3", 0.70),
        ("LFR N=10000 μ=0.5 (large)", "LFR_N10000_mu0.5", 0.50),
    ];
    
    for (desc, name, expected_nmi) in benchmarks {
        let edge_file = format!("{}/{}.edges", BENCHMARK_DIR, name);
        let gt_file = format!("{}/{}.ground_truth", BENCHMARK_DIR, name);
        run_benchmark(desc, &edge_file, &gt_file, expected_nmi);
    }
    
    println!("══════════════════════════════════════════════════════════════");
    println!("Tier 2 benchmark testing complete.");
    println!("══════════════════════════════════════════════════════════════");
}

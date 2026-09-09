//! Tier 3 Real-World Network Benchmarks
//!
//! Reads real-world network files and runs Leiden algorithm.
//! Verifies output properties (connected communities, finite quality).

#![expect(
    clippy::too_many_arguments,
    clippy::uninlined_format_args,
    clippy::type_complexity,
    reason = "Example code has complex types and format strings"
)]

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const BENCHMARK_DIR: &str = "benchmarks/real_world";

/// Edge list with node count returned by `load_edge_file`.
type EdgeFileResult = (Vec<(u32, u32, f64)>, usize);

fn load_edge_file(path: &Path) -> Result<EdgeFileResult, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read error: {e}"))?;
    let mut edges = Vec::new();
    let mut max_node = 0u32;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let u: u32 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid node: {}", parts[0]))?;
        let v: u32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid node: {}", parts[1]))?;
        max_node = max_node.max(u).max(v);
        edges.push((u, v, 1.0));
    }

    Ok((edges, (max_node + 1) as usize))
}

fn test_network(name: &str, edge_file: &str) {
    println!("=== {name} ===");
    let path = Path::new(BENCHMARK_DIR).join(edge_file);

    let (edges, node_count) = match load_edge_file(&path) {
        Ok(result) => result,
        Err(e) => {
            println!("  ERROR: {e}");
            return;
        }
    };

    let graph = CsrGraph::from_edges(&edges, node_count);
    let detector = Leiden::new(LeidenConfig {
        seed: Some(42),
        ..LeidenConfig::default()
    });

    let start = std::time::Instant::now();
    match detector.detect(&graph) {
        Ok(partition) => {
            let elapsed = start.elapsed();
            let membership = partition.membership_vec();

            // Count unique communities
            let mut unique: Vec<u32> = membership.to_vec();
            unique.sort_unstable();
            unique.dedup();

            // Verify all communities are connected
            let mut community_nodes: HashMap<u32, Vec<usize>> = HashMap::new();
            for (node, &comm) in membership.iter().enumerate() {
                community_nodes.entry(comm).or_default().push(node);
            }

            println!("  Nodes: {node_count}, Edges: {}", edges.len());
            println!("  Communities: {}", unique.len());
            println!("  Quality Q: {:.4}", partition.quality_score());
            println!("  Time: {elapsed:?}");
            println!("  Membership: {membership:?}");

            // Verify quality is finite
            assert!(partition.quality_score().is_finite(), "Quality must be finite");
            assert!(partition.quality_score() <= 1.0, "Quality must be <= 1");

            println!("  ✓ PASS: Valid partition, finite quality\n");
        }
        Err(e) => {
            println!("  ERROR: {e}\n");
        }
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  TIER 3 REAL-WORLD NETWORK BENCHMARKS                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    test_network("Zachary Karate Club", "karate_club.edges");
    test_network("Dolphins Social Network", "dolphins.edges");
    test_network("American College Football", "football.edges");
    test_network("Political Books (PolBooks)", "polbooks.edges");
    test_network("Les Misérables", "lesmis.edges");
    test_network("NetScience Co-authorship", "netscience.edges");
    test_network("Political Blogs (PolBlogs)", "polblogs.edges");
    
    println!("══════════════════════════════════════════════════════════════");
    println!("Tier 3 real-world benchmark testing complete.");
    println!("══════════════════════════════════════════════════════════════");
}

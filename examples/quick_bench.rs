//! Quick benchmark for a single LFR graph.

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

fn compute_nmi(p1: &[u32], p2: &[u32]) -> f64 {
    let n = p1.len() as f64;
    let mut cont: HashMap<(u32, u32), f64> = HashMap::new();
    for (&c1, &c2) in p1.iter().zip(p2.iter()) {
        *cont.entry((c1, c2)).or_insert(0.0) += 1.0;
    }
    let mut rows: HashMap<u32, f64> = HashMap::new();
    let mut cols: HashMap<u32, f64> = HashMap::new();
    for (&(c1, c2), &cnt) in &cont {
        *rows.entry(c1).or_insert(0.0) += cnt;
        *cols.entry(c2).or_insert(0.0) += cnt;
    }
    let mut mi = 0.0;
    for (&(c1, c2), &cnt) in &cont {
        if cnt > 0.0 { mi += (cnt / n) * ((n * cnt) / (rows[&c1] * cols[&c2])).ln(); }
    }
    let h1: f64 = rows.values().map(|&a| if a > 0.0 { -(a/n)*(a/n).ln() } else { 0.0 }).sum();
    let h2: f64 = cols.values().map(|&b| if b > 0.0 { -(b/n)*(b/n).ln() } else { 0.0 }).sum();
    if h1 + h2 == 0.0 { 1.0 } else { 2.0 * mi / (h1 + h2) }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let name = args.get(1).map(|s| s.as_str()).unwrap_or("LFR_N1000_mu0.1");
    
    let edge_file = format!("benchmarks/lfr_graphs/{}.edges", name);
    let gt_file = format!("benchmarks/lfr_graphs/{}.ground_truth", name);
    
    let edge_content = fs::read_to_string(&edge_file).expect("read edges");
    let mut edges = Vec::new();
    let mut max_node = 0u32;
    for line in edge_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let u: u32 = parts[0].parse().unwrap();
        let v: u32 = parts[1].parse().unwrap();
        max_node = max_node.max(u).max(v);
        edges.push((u, v, 1.0));
    }
    
    let gt_content = fs::read_to_string(&gt_file).expect("read gt");
    let mut ground_truth = Vec::new();
    for line in gt_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let node: u32 = parts[0].parse().unwrap();
        let comm: u32 = parts[1].parse().unwrap();
        while ground_truth.len() <= node as usize { ground_truth.push(0); }
        ground_truth[node as usize] = comm;
    }
    
    let graph = CsrGraph::from_edges(&edges, (max_node + 1) as usize);
    let detector = Leiden::new(LeidenConfig { seed: Some(42), ..LeidenConfig::default() });
    
    let start = Instant::now();
    let partition = detector.detect(&graph).expect("detect");
    let elapsed = start.elapsed();
    
    let nmi = compute_nmi(partition.membership_vec(), &ground_truth);
    
    println!("{}: nodes={}, edges={}, comms={}, Q={:.4}, NMI={:.4}, time={:?}",
        name, max_node + 1, edges.len(), partition.community_count(),
        partition.quality_score(), nmi, elapsed);
}

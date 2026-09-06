//! Quick Tier 3 test - run one network at a time.

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let name = args.get(1).map(|s| s.as_str()).unwrap_or("karate_club");
    
    let path = Path::new("benchmarks/real_world").join(format!("{}.edges", name));
    let content = fs::read_to_string(&path).expect("read edges");
    
    let mut edges = Vec::new();
    let mut max_node = 0u32;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        let u: u32 = parts[0].parse().unwrap();
        let v: u32 = parts[1].parse().unwrap();
        max_node = max_node.max(u).max(v);
        edges.push((u, v, 1.0));
    }
    
    let graph = CsrGraph::from_edges(&edges, (max_node + 1) as usize);
    let detector = Leiden::new(LeidenConfig { seed: Some(42), ..LeidenConfig::default() });
    
    let start = Instant::now();
    let partition = detector.detect(&graph).expect("detect");
    let elapsed = start.elapsed();
    
    let mut unique = partition.membership_vec().iter().copied().collect::<Vec<_>>();
    unique.sort_unstable();
    unique.dedup();
    
    println!("{}: nodes={}, edges={}, comms={}, Q={:.4}, time={:?}",
        name, max_node + 1, edges.len(), unique.len(),
        partition.quality_score(), elapsed);
}

//! Quick Tier 3 test - run one network at a time.

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let name = args.get(1).map_or("karate_club", String::as_str);

    let path = Path::new("benchmarks/real_world").join(format!("{name}.edges"));
    let content = fs::read_to_string(&path).map_err(|e| format!("read edges: {e}"))?;

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
        let u: u32 = parts[0].parse().map_err(|e| format!("parse u: {e}"))?;
        let v: u32 = parts[1].parse().map_err(|e| format!("parse v: {e}"))?;
        max_node = max_node.max(u).max(v);
        edges.push((u, v, 1.0));
    }

    let graph = CsrGraph::from_edges(&edges, (max_node + 1) as usize);
    let detector = Leiden::new(LeidenConfig {
        seed: Some(42),
        ..LeidenConfig::default()
    });

    let start = Instant::now();
    let partition = detector
        .detect(&graph)
        .map_err(|e| format!("detect: {e}"))?;
    let elapsed = start.elapsed();

    let mut unique: Vec<u32> = partition.membership_vec().to_vec();
    unique.sort_unstable();
    unique.dedup();

    println!(
        "{name}: nodes={}, edges={}, comms={}, Q={:.4}, time={:?}",
        max_node + 1,
        edges.len(),
        unique.len(),
        partition.quality_score(),
        elapsed
    );

    Ok(())
}

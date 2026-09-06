# Communal

High-performance community detection framework in pure Rust.

## Status

v0.1.0 — Core infrastructure complete. Leiden algorithm (local moving phase) working on all Tier 1 deterministic reference graphs.

**Note:** Performance optimization needed for graphs >100 nodes. See [ROADMAP.md](ROADMAP.md) for details.

## Quick Start

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::io::edgelist::parse_edgelist_to_csr;
use communal_core::detector::CommunityDetector;

// Load graph from edge list file
let input = std::fs::read_to_string("graph.edges")?;
let graph = parse_edgelist_to_csr(&input)?;

// Run Leiden algorithm
let detector = Leiden::new(LeidenConfig::default());
let partition = detector.detect(&graph)?;

println!("Communities: {}", partition.community_count());
println!("Quality Q: {}", partition.quality_score());
println!("Membership: {:?}", partition.membership_vec());
```

## Input Format

Edge list (whitespace-delimited):
```text
# Comments start with #
0 1 1.0
0 2 1.0
1 2 1.0
```

Planned formats: GML, GraphML, Pajek, Matrix Market, GEXF. See [ROADMAP.md](ROADMAP.md#graph-format-support).

## Project Structure

```
communal/
├── crates/
│   ├── communal-core/      # Traits, CSR graph, builder, I/O
│   ├── communal-algo/      # Leiden algorithm, quality functions
│   ├── communal-petgraph/  # petgraph adapter
│   └── ...
├── benchmarks/             # LFR and real-world test graphs
├── research/               # Design research and analysis
├── ROADMAP.md              # Feature status and release plan
└── AGENTS.md               # Code map and conventions
```

## Testing

```bash
# All tests
cargo test --workspace

# Run examples
cargo run --example demo_leiden
cargo run --example tier1_tests
```

## License

MIT OR Apache-2.0

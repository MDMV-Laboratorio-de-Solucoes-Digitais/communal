# Quickstart: Leiden Algorithm Validation Guide

**Branch**: `002-leiden-completion` | **Date**: 2026-09-05**

## Overview

This guide provides step-by-step validation scenarios to verify the Leiden algorithm implementation works correctly end-to-end. Run these scenarios after implementation to confirm all requirements are met.

## Prerequisites

- Rust 1.85+ installed
- Project cloned and dependencies resolved
- All workspace crates compile: `cargo build --all-features`

## Validation Scenarios

### Scenario 1: Basic Community Detection

**Goal**: Verify basic Leiden detection produces valid communities.

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::detector::CommunityDetector;
use communal_core::graph::CsrGraph;

fn main() {
    // Create a simple graph with two clear communities
    // Community 1: nodes 0, 1, 2 (fully connected)
    // Community 2: nodes 3, 4, 5 (fully connected)
    // One bridge edge: 2-3
    let edges = vec![
        (0, 1, 1.0), (0, 2, 1.0), (1, 2, 1.0),  // Triangle 1
        (3, 4, 1.0), (3, 5, 1.0), (4, 5, 1.0),  // Triangle 2
        (2, 3, 0.1),                              // Bridge
    ];

    let graph = CsrGraph::from_edges(6, &edges);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    println!("Membership: {:?}", partition.membership_vec());
    println!("Quality: {}", partition.quality_score());
    println!("Communities: {}", partition.community_count());

    // Assertions
    assert!(partition.quality_score().is_finite());
    assert!(partition.community_count() >= 2);
}
```

**Expected Output**:
- Quality score is finite (not NaN/Inf)
- At least 2 communities detected
- Nodes 0,1,2 likely in same community; nodes 3,4,5 likely in same community

### Scenario 2: Determinism Verification

**Goal**: Verify same seed produces identical results.

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::detector::CommunityDetector;
use communal_core::graph::CsrGraph;

fn main() {
    let graph = build_test_graph();
    let config = LeidenConfig {
        seed: Some(42),
        ..Default::default()
    };

    let detector1 = Leiden::new(config.clone());
    let detector2 = Leiden::new(config);

    let partition1 = detector1.detect(&graph).unwrap();
    let partition2 = detector2.detect(&graph).unwrap();

    assert_eq!(
        partition1.membership_vec(),
        partition2.membership_vec(),
        "Results must be identical with same seed"
    );

    println!("Determinism verified: both runs produced identical results");
}
```

**Expected**: Assertion passes, confirming deterministic execution.

### Scenario 3: Connected Communities Invariant

**Goal**: Verify all detected communities are internally connected.

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::detector::CommunityDetector;
use communal_core::graph::CsrGraph;
use std::collections::HashSet;

fn main() {
    let graph = build_large_random_graph(100, 300);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    for community_id in 0..partition.community_count() {
        let nodes = partition.nodes_in_community(community_id);
        assert!(
            is_connected(&graph, &nodes),
            "Community {} is not connected!",
            community_id
        );
    }

    println!("All {} communities are internally connected", partition.community_count());
}

fn is_connected(graph: &CsrGraph, nodes: &[u32]) -> bool {
    if nodes.len() <= 1 {
        return true;
    }
    let node_set: HashSet<u32> = nodes.iter().copied().collect();
    let mut visited = HashSet::new();
    let mut stack = vec![nodes[0]];

    while let Some(node) = stack.pop() {
        if visited.insert(node) {
            for neighbor in graph.neighbors(node.into()) {
                let n = u32::from(neighbor);
                if node_set.contains(&n) {
                    stack.push(n);
                }
            }
        }
    }
    visited.len() == nodes.len()
}
```

**Expected**: All communities pass the connectedness check.

### Scenario 4: Edge Case Handling

**Goal**: Verify algorithm handles edge cases without panicking.

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::detector::CommunityDetector;
use communal_core::graph::CsrGraph;

fn main() {
    let detector = Leiden::new(LeidenConfig::default());

    // Empty graph
    let empty = CsrGraph::new(0);
    let result = detector.detect(&empty);
    assert!(result.is_ok(), "Empty graph should not panic");
    assert_eq!(result.unwrap().membership_vec().len(), 0);

    // Single node
    let single = CsrGraph::new(1);
    let result = detector.detect(&single);
    assert!(result.is_ok());
    assert!(result.unwrap().quality_score().is_finite());

    // Self-loops
    let edges = vec![(0, 0, 1.0), (0, 1, 1.0), (1, 1, 1.0)];
    let self_loops = CsrGraph::from_edges(2, &edges);
    let result = detector.detect(&self_loops);
    assert!(result.is_ok(), "Self-loops should be handled");

    // Zero weights
    let edges = vec![(0, 1, 0.0), (1, 2, 1.0)];
    let zero_weights = CsrGraph::from_edges(3, &edges);
    let result = detector.detect(&zero_weights);
    assert!(result.is_ok());
    assert!(!result.unwrap().quality_score().is_nan());

    println!("All edge cases handled successfully");
}
```

**Expected**: All assertions pass, no panics.

### Scenario 5: Quality Function Selection

**Goal**: Verify Modularity Q and CPM produce different results.

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::detector::CommunityDetector;
use communal_core::graph::CsrGraph;
use communal_algo::quality::QualityFunction;

fn main() {
    let graph = build_test_graph();

    let mod_detector = Leiden::new(LeidenConfig::default())
        .with_quality_function(QualityFunction::Modularity);
    let cpm_detector = Leiden::new(LeidenConfig::default())
        .with_quality_function(QualityFunction::Cpm);

    let mod_partition = mod_detector.detect(&graph).unwrap();
    let cpm_partition = cpm_detector.detect(&graph).unwrap();

    println!("Modularity Q: {}", mod_partition.quality_score());
    println!("CPM: {}", cpm_partition.quality_score());

    // Both should produce valid partitions
    assert!(mod_partition.quality_score().is_finite());
    assert!(cpm_partition.quality_score().is_finite());
}
```

**Expected**: Both quality functions produce finite scores.

### Scenario 6: Configuration Parameters

**Goal**: Verify configuration parameters affect results.

```rust
use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::detector::CommunityDetector;
use communal_core::graph::CsrGraph;

fn main() {
    let graph = build_test_graph();

    // Default config
    let default = Leiden::new(LeidenConfig::default());
    let p1 = default.detect(&graph).unwrap();

    // High resolution (more communities)
    let high_res = Leiden::new(LeidenConfig {
        gamma: 2.0,
        ..Default::default()
    });
    let p2 = high_res.detect(&graph).unwrap();

    // Low resolution (fewer communities)
    let low_res = Leiden::new(LeidenConfig {
        gamma: 0.5,
        ..Default::default()
    });
    let p3 = low_res.detect(&graph).unwrap();

    println!("Default (γ=1.0): {} communities", p1.community_count());
    println!("High (γ=2.0): {} communities", p2.community_count());
    println!("Low (γ=0.5): {} communities", p3.community_count());

    // Higher gamma should generally produce more communities
    assert!(p2.community_count() >= p3.community_count());
}
```

**Expected**: Higher gamma produces more (or equal) communities.

## Running the Validation

```bash
# Build the project
cargo build --all-features

# Run all tests
cargo test -p communal-algo --all-features

# Run property-based tests (may take longer)
cargo test -p communal-algo --test leiden_connected --test leiden_determinism -- --nocapture

# Run edge case tests
cargo test -p communal-algo --test leiden_edge_cases

# Run benchmark validation (requires LFR data)
cargo test -p communal-algo --test synthetic_ground_truth -- --ignored
```

## Expected Outcomes

| Scenario | Expected Result |
|----------|-----------------|
| Basic Detection | Valid partition with finite quality |
| Determinism | Identical results with same seed |
| Connected Communities | All communities internally connected |
| Edge Cases | No panics, valid partitions |
| Quality Functions | Both Modularity and CPM work |
| Configuration | Parameters affect results as expected |

## Troubleshooting

| Issue | Possible Cause | Solution |
|-------|----------------|----------|
| Test panics | Unhandled edge case | Check edge case tests for specific scenario |
| Non-deterministic results | RNG not seeded properly | Verify seed is passed to StdRng |
| Disconnected communities | Bug in refinement phase | Run Scenario 3 to identify failing graph |
| NaN/Inf quality | Numerical overflow | Check for division by zero in quality computation |
| Slow convergence | Threshold too low | Increase convergence_threshold or max_iterations |

## Next Steps

After all validation scenarios pass:
1. Run `cargo clippy --all-targets --all-features` — zero warnings
2. Run `cargo fmt --check` — properly formatted
3. Run `cargo doc --no-deps` — documentation builds
4. Commit changes to `002-leiden-completion` branch

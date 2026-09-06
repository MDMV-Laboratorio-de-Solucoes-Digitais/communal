# Contract: Leiden Test Specifications

**Branch**: `002-leiden-completion` | **Date**: 2026-09-05**

## Overview

This contract defines the test specifications for the Leiden algorithm implementation. All tests must pass before the feature is considered complete.

## Property-Based Tests (proptest)

### TC-001: Connected Communities Invariant

**File**: `tests/leiden_connected.rs`

**Property**: Every detected community is internally connected.

```rust
proptest! {
    #[test]
    fn all_communities_are_connected(
        edges in vec![(0u32..100, 0u32..100, 0.0f64..1.0), 0..200],
    ) {
        let graph = build_graph(&edges);
        let detector = Leiden::new(LeidenConfig::default());
        let partition = detector.detect(&graph).unwrap();

        for community_id in 0..partition.community_count() {
            let nodes: Vec<u32> = partition.nodes_in_community(community_id);
            prop_assert!(is_connected(&graph, &nodes),
                "Community {} is not connected: {:?}", community_id, nodes);
        }
    }
}
```

**Configuration**:
- Cases: 1000+ random graphs
- Node range: 0..100
- Edge range: 0..200
- Weight range: 0.0..1.0

### TC-002: Determinism

**File**: `tests/leiden_determinism.rs`

**Property**: Same seed produces identical results.

```rust
proptest! {
    #[test]
    fn deterministic_with_same_seed(
        edges in vec![(0u32..50, 0u32..50, 0.0f64..1.0), 0..100],
        seed in any::<u64>(),
    ) {
        let graph = build_graph(&edges);
        let config = LeidenConfig {
            seed: Some(seed),
            ..Default::default()
        };

        let detector1 = Leiden::new(config.clone());
        let detector2 = Leiden::new(config);

        let partition1 = detector1.detect(&graph).unwrap();
        let partition2 = detector2.detect(&graph).unwrap();

        prop_assert_eq!(
            partition1.membership_vec(),
            partition2.membership_vec()
        );
    }
}
```

**Configuration**:
- Cases: 100 random graphs × multiple seeds
- Runs per case: 100

### TC-003: Quality Monotonicity

**File**: `tests/leiden_connected.rs`

**Property**: Quality never decreases between iterations.

```rust
// Internal test: verify quality increases or plateaus
#[test]
fn quality_monotonicity() {
    let graph = build_test_graph();
    let mut prev_quality = f64::NEG_INFINITY;

    for iteration in 0..100 {
        let quality = run_single_iteration(&graph, iteration);
        assert!(quality >= prev_quality || (quality - prev_quality).abs() < 1e-10,
            "Quality decreased at iteration {}: {} < {}", iteration, quality, prev_quality);
        prev_quality = quality;
    }
}
```

## Edge Case Tests

### TC-004: Empty Graph

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn empty_graph_returns_empty_partition() {
    let graph = CsrGraph::new(0);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    assert_eq!(partition.membership_vec().len(), 0);
    assert_eq!(partition.quality_score(), 0.0);
}
```

### TC-005: Single Node

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn single_node_single_community() {
    let graph = CsrGraph::new(1);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    assert_eq!(partition.membership_vec(), &[0]);
    assert!(partition.quality_score().is_finite());
}
```

### TC-006: Self-Loops

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn self_loops_handled_correctly() {
    let edges = vec![(0, 0, 1.0), (0, 1, 1.0), (1, 1, 1.0)];
    let graph = CsrGraph::from_edges(2, &edges);
    let detector = Leiden::new(LeidenConfig::default());

    let result = detector.detect(&graph);
    assert!(result.is_ok());

    let partition = result.unwrap();
    assert!(partition.quality_score().is_finite());
}
```

### TC-007: Zero Weight Edges

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn zero_weight_edges_valid_partition() {
    let edges = vec![(0, 1, 0.0), (1, 2, 1.0), (2, 0, 0.0)];
    let graph = CsrGraph::from_edges(3, &edges);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    assert!(partition.quality_score().is_finite());
    assert!(!partition.quality_score().is_nan());
}
```

### TC-008: Disconnected Components

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn disconnected_components_independent_communities() {
    // Two disconnected triangles
    let edges = vec![
        (0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0),  // Component 1
        (3, 4, 1.0), (4, 5, 1.0), (5, 3, 1.0),  // Component 2
    ];
    let graph = CsrGraph::from_edges(6, &edges);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    // Each component's communities should be internally connected
    assert_all_communities_connected(&graph, &partition);
}
```

### TC-009: Negative Weights (Valid)

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn negative_weights_valid_when_total_positive() {
    // Total weight = 1.0 (positive), but individual edges can be negative
    let edges = vec![(0, 1, -0.5), (1, 2, 1.0), (2, 0, 0.5)];
    let graph = CsrGraph::from_edges(3, &edges);
    let detector = Leiden::new(LeidenConfig::default());

    let result = detector.detect(&graph);
    assert!(result.is_ok());
}
```

### TC-010: Negative Weights (Invalid)

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn negative_total_weight_rejected() {
    let edges = vec![(0, 1, -1.0), (1, 2, -1.0)];
    let graph = CsrGraph::from_edges(3, &edges);
    let detector = Leiden::new(LeidenConfig::default());

    let result = detector.detect(&graph);
    assert!(result.is_err());
}
```

## Benchmark Validation Tests

### TC-011: LFR Ground Truth (NMI)

**File**: `tests/synthetic_ground_truth.rs`

```rust
#[test]
fn lfr_nmi_above_threshold() {
    // LFR benchmark: N=10000, mu=0.3
    let (graph, ground_truth) = generate_lfr_benchmark(10000, 0.3);
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(&graph).unwrap();

    let nmi = compute_nmi(&partition, &ground_truth);
    assert!(nmi >= 0.95, "NMI {} below threshold 0.95", nmi);
}
```

**Configuration**:
- Graph size: N = 10,000 nodes
- Mixing parameter: μ ≤ 0.3
- Target NMI: ≥ 0.95

### TC-012: Convergence Within Max Iterations

**File**: `tests/leiden_edge_cases.rs`

```rust
#[test]
fn converges_within_max_iterations() {
    let graph = build_large_test_graph(1000);
    let config = LeidenConfig {
        max_iterations: 1000,
        ..Default::default()
    };
    let detector = Leiden::new(config);

    let start = Instant::now();
    let partition = detector.detect(&graph).unwrap();
    let elapsed = start.elapsed();

    assert!(partition.quality_score().is_finite());
    // Should complete in reasonable time (not a hard assertion)
    println!("Converged in {:?}", elapsed);
}
```

## Test Helper Functions

```rust
/// Checks if a set of nodes forms a connected subgraph.
fn is_connected<G: GraphView>(graph: &G, nodes: &[u32]) -> bool {
    if nodes.len() <= 1 {
        return true;
    }

    let node_set: HashSet<u32> = nodes.iter().copied().collect();
    let mut visited = HashSet::new();
    let mut stack = vec![nodes[0]];

    while let Some(node) = stack.pop() {
        if visited.insert(node) {
            for neighbor in graph.neighbors(NodeId::from(node)) {
                let neighbor_id = u32::from(neighbor);
                if node_set.contains(&neighbor_id) {
                    stack.push(neighbor_id);
                }
            }
        }
    }

    visited.len() == nodes.len()
}

/// Asserts all communities in a partition are connected.
fn assert_all_communities_connected<G: GraphView>(graph: &G, partition: &Partition) {
    for community_id in 0..partition.community_count() {
        let nodes = partition.nodes_in_community(community_id);
        assert!(is_connected(graph, &nodes),
            "Community {} is not connected", community_id);
    }
}
```

## Test Execution

```bash
# Run all tests
cargo test -p communal-algo --all-features

# Run property-based tests only
cargo test -p communal-algo --test leiden_connected --test leiden_determinism

# Run edge case tests
cargo test -p communal-algo --test leiden_edge_cases

# Run benchmark validation
cargo test -p communal-algo --test synthetic_ground_truth -- --ignored
```

## Pass Criteria

| Test Category | Minimum Pass Rate |
|---------------|-------------------|
| Property-based (proptest) | 100% of cases |
| Edge cases | 100% of tests |
| Benchmark validation | NMI ≥ 0.95 |

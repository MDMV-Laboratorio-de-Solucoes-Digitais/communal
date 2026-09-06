# Contract: Leiden Public API

**Branch**: `002-leiden-completion` | **Date**: 2026-09-05**

## Overview

This contract defines the public API for the Leiden algorithm implementation. All public interfaces must adhere to this specification.

## Public Types

### Leiden

```rust
pub struct Leiden {
    // private fields
}
```

**Constructor**:
```rust
impl Leiden {
    /// Creates a new Leiden detector with the given configuration.
    ///
    /// # Arguments
    /// * `config` — Algorithm configuration parameters
    ///
    /// # Examples
    /// ```
    /// use communal_algo::leiden::{Leiden, LeidenConfig};
    ///
    /// let config = LeidenConfig::default();
    /// let detector = Leiden::new(config);
    /// ```
    pub fn new(config: LeidenConfig) -> Self;

    /// Sets the quality function to use.
    ///
    /// # Arguments
    /// * `qf` — Quality function (Modularity or Cpm)
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use communal_algo::leiden::{Leiden, LeidenConfig};
    /// use communal_algo::quality::QualityFunction;
    ///
    /// let detector = Leiden::new(LeidenConfig::default())
    ///     .with_quality_function(QualityFunction::Cpm);
    /// ```
    pub fn with_quality_function(self, qf: QualityFunction) -> Self;
}
```

### LeidenConfig

```rust
pub struct LeidenConfig {
    pub gamma: f64,
    pub beta: f64,
    pub convergence_threshold: f64,
    pub convergence_mode: ConvergenceMode,
    pub max_iterations: usize,
    pub seed: Option<u64>,
}
```

**Default Implementation**:
```rust
impl Default for LeidenConfig {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            beta: 0.01,
            convergence_threshold: 1e-6,
            convergence_mode: ConvergenceMode::Absolute,
            max_iterations: 1000,
            seed: Some(42),
        }
    }
}
```

**Builder Methods** (optional convenience):
```rust
impl LeidenConfig {
    pub fn with_gamma(mut self, gamma: f64) -> Self;
    pub fn with_beta(mut self, beta: f64) -> Self;
    pub fn with_convergence_threshold(mut self, threshold: f64) -> Self;
    pub fn with_max_iterations(mut self, max: usize) -> Self;
    pub fn with_seed(mut self, seed: u64) -> Self;
    pub fn with_seed_option(mut self, seed: Option<u64>) -> Self;
}
```

### QualityFunction

```rust
pub enum QualityFunction {
    Modularity,
    Cpm,
    MapEquation,  // Not used by Leiden
}
```

## Trait Implementations

### CommunityDetector

```rust
impl<G: GraphView> CommunityDetector<G> for Leiden {
    /// Detects communities in the graph.
    ///
    /// # Arguments
    /// * `graph` — Input graph implementing GraphView
    ///
    /// # Returns
    /// * `Ok(Partition)` — Community assignment and quality score
    /// * `Err(GraphError)` — If graph is invalid
    ///
    /// # Errors
    /// * `GraphError::EmptyGraph` — If graph has no nodes
    /// * `GraphError::InvalidGraph` — If total edge weight ≤ 0
    ///
    /// # Examples
    /// ```
    /// use communal_algo::leiden::{Leiden, LeidenConfig};
    /// use communal_core::detector::CommunityDetector;
    /// use communal_core::graph::CsrGraph;
    ///
    /// let graph = CsrGraph::from_edges(4, &[(0,1,1.0),(1,2,1.0),(2,3,1.0),(3,0,1.0)]);
    /// let detector = Leiden::new(LeidenConfig::default());
    /// let partition = detector.detect(&graph).unwrap();
    /// assert!(partition.quality_score().is_finite());
    /// ```
    fn detect(&self, graph: &G) -> Result<Partition, GraphError>;
}
```

## Behavior Contracts

### BC-001: Determinism

**Given**: Same graph, same seed
**When**: `detect()` called multiple times
**Then**: Returns identical membership vectors

```rust
// Pseudocode for test
let partition1 = detector.detect(&graph).unwrap();
let partition2 = detector.detect(&graph).unwrap();
assert_eq!(
    partition1.membership_vec(),
    partition2.membership_vec()
);
```

### BC-002: Connected Communities

**Given**: Any valid graph
**When**: `detect()` completes
**Then**: Every community is internally connected

```rust
// Pseudocode for test
let partition = detector.detect(&graph).unwrap();
for community in partition.communities() {
    assert!(is_connected(&graph, community));
}
```

### BC-003: Empty Graph Handling

**Given**: Graph with 0 nodes
**When**: `detect()` called
**Then**: Returns `Ok(Partition::new(vec![], 0.0))`

### BC-004: Edge Case Robustness

**Given**: Graph with self-loops, zero weights, or disconnected components
**When**: `detect()` called
**Then**: Returns valid partition with finite quality (no panic)

### BC-005: Convergence

**Given**: Any valid graph
**When**: `detect()` called
**Then**: Returns within max_iterations (no infinite loop)

### BC-006: Quality Function Selection

**Given**: Leiden with QualityFunction::Modularity
**When**: `detect()` completes
**Then**: Quality computed using Modularity Q formula

**Given**: Leiden with QualityFunction::Cpm
**When**: `detect()` completes
**Then**: Quality computed using CPM formula

## Error Contracts

| Condition | Error Type | Message Pattern |
|-----------|------------|-----------------|
| Empty graph | `GraphError::EmptyGraph` | "empty graph provided" |
| Total weight ≤ 0 | `GraphError::InvalidGraph` | "invalid graph: total edge weight must be positive" |
| Invalid config | `AlgorithmError::InvalidConfiguration` | "invalid configuration: {reason}" |

## Event Contracts

The algorithm emits `StepEvent` variants during execution:

| Event | When Emitted | Payload |
|-------|--------------|---------|
| `LocalMovingStart` | Before local moving phase | `{ iteration }` |
| `LocalMovingEnd` | After local moving phase | `{ iteration }` |
| `NodeRelocation` | When node moves community | `{ node, from, to }` |
| `RefinementSplit` | When community splits | `{ community, into }` |
| `AggregationContraction` | After aggregation | `{ from_communities, to_communities }` |
| `ConvergencePlateau` | Quality below plateau threshold | `{ iteration, improvement, current_quality }` |
| `ConvergenceDetected` | Algorithm converges | `{ total_iterations, final_quality }` |

## Performance Contracts

| Metric | Target | Notes |
|--------|--------|-------|
| Time complexity | O(V + E) per iteration | Local moving + refinement |
| Space complexity | O(V + E) | Membership + community mappings |
| Termination | ≤ max_iterations | Hard limit |

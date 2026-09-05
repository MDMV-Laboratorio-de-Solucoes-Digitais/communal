# Contract: Core Traits

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

Defines the fundamental traits that form the backbone of the Communal framework. These traits provide the abstraction layer enabling algorithm interchangeability and graph source flexibility.

---

## GraphView Trait

The primary graph abstraction for read-only access during algorithm execution.

```rust
/// Provides read-only access to graph topology for community detection algorithms.
///
/// All compute-intensive algorithms operate over this trait object or its
/// concrete CSR implementation. Implementations must guarantee O(1) neighbor
/// iteration via CSR offsets.
///
/// # Invariants
/// - `node_count() >= 0`
/// - `edge_count() >= 0`
/// - For undirected graphs: `degree(node) == weighted_degree(node).count_neighbors()`
///
/// # Complexity
/// - `node_count`: O(1)
/// - `degree`: O(1) amortized
/// - `neighbors`: O(degree(node))
pub trait GraphView {
    type NodeId: Copy + Eq + Hash;
    type EdgeWeight: Copy + num_traits::Float;

    /// Returns the number of nodes in the graph.
    fn node_count(&self) -> u32;

    /// Returns the number of edges in the graph.
    fn edge_count(&self) -> u32;

    /// Returns true if the graph is treated as directed.
    fn is_directed(&self) -> bool;

    /// Returns the degree (number of neighbors) of a node.
    ///
    /// # Panics
    /// Never panics; returns 0 for invalid node IDs.
    fn degree(&self, node: Self::NodeId) -> u32;

    /// Returns an iterator over (neighbor, weight) pairs for a node.
    fn neighbors(&self, node: Self::NodeId) -> impl Iterator<Item = (Self::NodeId, Self::EdgeWeight)>;

    /// Returns the weight of a specific edge, or None if not present.
    fn edge_weight(&self, from: Self::NodeId, to: Self::NodeId) -> Option<Self::EdgeWeight>;

    /// Returns true if the edge exists in the graph.
    fn has_edge(&self, from: Self::NodeId, to: Self::NodeId) -> bool;
}
```

---

## CommunityDetector Trait

The unified interface for all community detection algorithms.

```rust
/// Core trait for community detection algorithms.
///
/// All algorithms (Leiden, Louvain, Infomap, LPA, Fluid) implement this trait,
/// enabling polymorphic usage through dynamic dispatch at the outer layer
/// (config/wrapping), while inner hot loops operate over concrete types.
///
/// # Type Parameters
/// - `G`: The graph view type the algorithm operates on
///
/// # Example
/// ```
/// use communal_algo::{Leiden, CommunityDetector, LeidenConfig};
///
/// let graph: CsrGraph<NodeId, f64> = ...;
/// let config = LeidenConfig::default();
/// let partition = Leiden::detect(&graph, &config)?;
/// ```
pub trait CommunityDetector<G: GraphView> {
    type Config: AlgorithmConfig;
    type Error: std::error::Error;

    /// Executes the community detection algorithm.
    ///
    /// # Arguments
    /// - `graph`: The input graph (immutable reference)
    /// - `config`: Algorithm-specific configuration
    ///
    /// # Returns
    /// `Result<Partition, Self::Error>` - The detected partition or error
    ///
    /// # Errors
    /// - `AlgorithmError::NonConvergence`: If max iterations reached without convergence
    /// - `AlgorithmError::InvalidGraph`: If graph structure violates algorithm assumptions
    fn detect(graph: &G, config: &Self::Config) -> Result<Partition, Self::Error>;

    /// Executes with step event emission for observability.
    ///
    /// When `subscriber` is Some, emits events for each algorithmic step.
    /// When None, runs identically to `detect` with zero overhead.
    ///
    /// # Performance
    /// Overhead when subscriber enabled: <= 20% relative to `detect`.
    fn detect_with_steps<F>(
        graph: &G,
        config: &Self::Config,
        subscriber: Option<F>,
    ) -> Result<Partition, Self::Error>
    where
        F: FnMut(StepEvent);
}
```

---

## PartitionResult Trait

Abstraction for querying partition results.

```rust
/// Provides queryable access to community detection results.
///
/// Enables algorithms to return different internal representations
/// while exposing a unified query interface.
pub trait PartitionResult {
    /// Returns the community ID for a given node.
    fn community_of(&self, node: NodeId) -> Option<CommunityId>;

    /// Returns all member nodes of a community.
    fn members(&self, community: CommunityId) -> impl Iterator<Item = NodeId>;

    /// Returns the total number of communities.
    fn community_count(&self) -> u32;

    /// Returns the size (member count) of a community.
    fn community_size(&self, community: CommunityId) -> u32;

    /// Returns the overall quality score of the partition.
    fn quality_score(&self) -> f64;

    /// Returns an iterator over all (node, community) pairs.
    fn assignments(&self) -> impl Iterator<Item = (NodeId, CommunityId)>;
}
```

---

## DynamicGraph Trait

For algorithms supporting incremental updates.

```rust
/// Trait for graphs supporting dynamic mutations without full recomputation.
///
/// Only edge insertions and deletions are supported in v1.
/// Node mutations are deferred to v1.1.
///
/// # Implementation Notes
/// - Edge insertion: O(k) local update
/// - Edge deletion: O(k) local update + possible community split
pub trait DynamicGraph: GraphView {
    /// Inserts an edge into the graph.
    ///
    /// # Arguments
    /// - `from`: Source node
    /// - `to`: Target node
    /// - `weight`: Edge weight
    ///
    /// # Returns
    /// Ok(()) on success, Err if weight is negative or nodes invalid
    fn insert_edge(
        &mut self,
        from: Self::NodeId,
        to: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Result<(), GraphError>;

    /// Removes an edge from the graph.
    ///
    /// If the removal disconnects a community, the community is immediately
    /// split into connected components.
    ///
    /// # Returns
    /// Ok(()) on success, Err if edge doesn't exist
    fn remove_edge(
        &mut self,
        from: Self::NodeId,
        to: Self::NodeId,
    ) -> Result<(), GraphError>;

    /// Returns true if the graph supports incremental updates.
    fn supports_incremental(&self) -> bool {
        true
    }
}
```

---

## AlgorithmConfig Trait

Base trait for all algorithm configurations.

```rust
/// Base trait for algorithm-specific configuration structs.
///
/// Each algorithm provides its own config type implementing this trait,
/// ensuring type-safe parameter passing and validation.
pub trait AlgorithmConfig: Clone + std::fmt::Debug {
    /// Validates all configuration parameters.
    ///
    /// # Returns
    /// Ok(()) if valid, Err with descriptive message otherwise
    fn validate(&self) -> Result<(), AlgorithmError>;

    /// Returns the convergence threshold for quality improvement.
    fn convergence_threshold(&self) -> f64;

    /// Returns the convergence mode (absolute or relative change).
    fn convergence_mode(&self) -> ConvergenceMode;

    /// Returns the optional seed for deterministic execution.
    fn seed(&self) -> Option<u64>;

    /// Returns the maximum number of iterations before forced termination.
    fn max_iterations(&self) -> u32;
}

/// Convergence criterion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceMode {
    /// Absolute change: |Q_current - Q_previous| < threshold
    Absolute,
    /// Relative change: |Q_current - Q_previous| / |Q_current| < threshold
    Relative,
}
```

---

## StepEvent Enum

Events emitted during algorithm execution for observability.

```rust
/// Represents a discrete algorithmic action during community detection.
///
/// Events are emitted when a subscriber is registered via `detect_with_steps`.
/// When no subscriber is present, event emission has zero overhead.
#[derive(Debug, Clone)]
pub enum StepEvent {
    /// A new phase has started.
    PhaseStart {
        phase: PhaseKind,
        iteration: u32,
    },

    /// A phase has completed.
    PhaseEnd {
        phase: PhaseKind,
        iteration: u32,
    },

    /// A node was relocated between communities.
    NodeRelocation {
        node: NodeId,
        from: CommunityId,
        to: CommunityId,
        /// Quality improvement from this move
        delta: f64,
    },

    /// A community was split during refinement.
    RefinementSplit {
        community: CommunityId,
        into: Vec<CommunityId>,
    },

    /// Communities were merged during aggregation.
    AggregationContract {
        communities: Vec<CommunityId>,
        into: CommunityId,
    },

    /// Algorithm reached convergence plateau.
    ConvergencePlateau {
        iteration: u32,
        delta: f64,
    },
}

/// Types of algorithm phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseKind {
    /// Local moving phase (Leiden, Louvain)
    LocalMoving,
    /// Refinement phase (Leiden only)
    Refinement,
    /// Aggregation phase (Leiden, Louvain)
    Aggregation,
    /// Label diffusion phase (LPA)
    Diffusion,
    /// Fluid expansion phase (Fluid)
    FluidExpansion,
    /// Map equation optimization (Infomap)
    FlowOptimization,
}
```

---

## Implementor's Notes

1. **Zero-cost observability**: Use Rust's type system to eliminate event emission overhead when no subscriber is present. Consider enum dispatch or const generics.

2. **CSR layout**: All concrete graph types must store data in CSR format for cache locality. The `GraphView` trait abstracts over this.

3. **Determinism**: All stochastic operations must use the seed from `AlgorithmConfig`. Use `rand::SeedableRng` with a reproducible PRNG.

4. **Error handling**: All fallible operations return `Result` with domain-specific errors via `thiserror`. No panics in production code.

5. **Documentation**: All public items must include doc comments with mathematical formulas, complexity analysis, and examples.

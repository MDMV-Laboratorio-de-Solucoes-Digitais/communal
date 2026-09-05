# Contract: Dynamic Graph (HIT Architecture)

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

Defines the interface for incremental community detection on dynamic graphs using the Hierarchical Incremental Tree (HIT) architecture. Supports edge insertions and deletions without full recomputation.

---

## DynamicDetector Trait

```rust
/// Trait for algorithms supporting incremental community updates.
///
/// Algorithms implementing this trait can process graph mutations
/// (edge insertions/deletions) without recomputing the entire partition.
///
/// # Implementation Notes
/// - Edge insertion: O(k) local boundary recalculation
/// - Edge deletion: O(k) local update + possible immediate community split
/// - Subtree stability: Unaffected communities remain unchanged
pub trait DynamicDetector<G: DynamicGraph> {
    type Config: AlgorithmConfig;

    /// Creates initial partition from a static graph.
    fn initial_detect(graph: &G, config: &Self::Config) -> Result<Partition, AlgorithmError>;

    /// Processes an edge insertion incrementally.
    ///
    /// Only recalculates affected local boundaries within O(k) neighborhood.
    ///
    /// # Arguments
    /// - `graph`: The graph (already mutated)
    /// - `partition`: Current partition (mutated in place)
    /// - `from`, `to`, `weight`: The inserted edge
    ///
    /// # Returns
    /// Ok(()) on success
    fn insert_edge(
        &self,
        graph: &G,
        partition: &mut Partition,
        from: NodeId,
        to: NodeId,
        weight: G::EdgeWeight,
    ) -> Result<(), AlgorithmError>;

    /// Processes an edge deletion incrementally.
    ///
    /// If the deletion disconnects a community, immediately splits
    /// the community into connected components to preserve the
    /// connectedness guarantee.
    ///
    /// # Arguments
    /// - `graph`: The graph (already mutated)
    /// - `partition`: Current partition (mutated in place)
    /// - `from`, `to`: The deleted edge
    ///
    /// # Returns
    /// Ok(()) on success. Communities may be split as a side effect.
    fn remove_edge(
        &self,
        graph: &G,
        partition: &mut Partition,
        from: NodeId,
        to: NodeId,
    ) -> Result<(), AlgorithmError>;

    /// Returns the current hierarchical tree.
    fn hierarchy(&self) -> &HierarchicalTree;

    /// Returns a partition slice at the specified granularity level.
    ///
    /// Level 0 = coarsest (one community), increasing levels = finer partitions.
    fn slice_at_level(&self, level: u32) -> Result<&Partition, DynamicError>;
}
```

---

## HierarchicalTree

```rust
/// Multi-level community hierarchy for coarse-to-fine analysis.
///
/// Represents community structure at multiple resolution levels,
/// supporting incremental updates and GraphRAG chunking.
///
/// # Structure
/// - Level 0: All nodes in single community (coarsest)
/// - Level N: Each node in own community (finest)
/// - Intermediate levels: Varying granularity
pub struct HierarchicalTree {
    levels: Vec<Partition>,
    current_level: u32,
}

impl HierarchicalTree {
    /// Returns the number of hierarchy levels.
    pub fn level_count(&self) -> u32;

    /// Returns the partition at the specified level.
    ///
    /// # Errors
    /// `DynamicError::InvalidLevel` if level >= level_count
    pub fn level(&self, level: u32) -> Result<&Partition, DynamicError>;

    /// Returns the current active level.
    pub fn current_level(&self) -> u32;

    /// Sets the current active level.
    pub fn set_level(&mut self, level: u32) -> Result<(), DynamicError>;

    /// Returns the coarsest partition (level 0).
    pub fn coarsest(&self) -> &Partition;

    /// Returns the finest partition (highest level).
    pub fn finest(&self) -> &Partition;

    /// Adds a new level after a hierarchy change.
    pub fn push_level(&mut self, partition: Partition);

    /// Removes levels above the specified level.
    pub fn truncate_above(&mut self, level: u32);
}
```

---

## IncrementalUpdate Result

```rust
/// Result of an incremental update operation.
#[derive(Debug, Clone)]
pub struct IncrementalUpdate {
    /// Whether the partition changed.
    pub changed: bool,

    /// Communities affected by the update.
    pub affected_communities: Vec<CommunityId>,

    /// Communities that were split (edge deletion only).
    pub split_communities: Vec<(CommunityId, Vec<CommunityId>)>,

    /// Quality delta from this update.
    pub quality_delta: f64,

    /// Nodes that changed community assignment.
    pub relocated_nodes: Vec<(NodeId, CommunityId, CommunityId)>,
}

impl IncrementalUpdate {
    /// Returns true if any structural change occurred.
    pub fn has_changes(&self) -> bool;

    /// Returns the number of affected communities.
    pub fn affected_count(&self) -> usize;
}
```

---

## StreamingDetector

```rust
/// High-level interface for streaming community detection.
///
/// Wraps a DynamicDetector and provides event-driven updates
/// suitable for GraphRAG integration.
pub struct StreamingDetector<D: DynamicDetector<CsrGraph<NodeId, f64>>> {
    detector: D,
    graph: CsrGraph<NodeId, f64>,
    partition: Partition,
    config: D::Config,
}

impl<D: DynamicDetector<CsrGraph<NodeId, f64>>> StreamingDetector<D> {
    /// Creates a new streaming detector with initial graph.
    pub fn new(
        graph: CsrGraph<NodeId, f64>,
        config: D::Config,
    ) -> Result<Self, AlgorithmError>;

    /// Processes a stream of edge mutations.
    ///
    /// # Arguments
    /// - `mutations`: Iterator of edge mutations
    ///
    /// # Returns
    /// Iterator of IncrementalUpdate results
    fn process_mutations(
        &mut self,
        mutations: impl Iterator<Item = EdgeMutation>,
    ) -> impl Iterator<Item = IncrementalUpdate>;

    /// Returns the current partition.
    pub fn partition(&self) -> &Partition;

    /// Returns the current hierarchy.
    pub fn hierarchy(&self) -> &HierarchicalTree;

    /// Returns the current graph.
    pub fn graph(&self) -> &CsrGraph<NodeId, f64>;
}
```

---

## EdgeMutation

```rust
/// Represents a single edge mutation in a streaming context.
#[derive(Debug, Clone)]
pub enum EdgeMutation {
    Insert { from: NodeId, to: NodeId, weight: f64 },
    Delete { from: NodeId, to: NodeId },
}

impl EdgeMutation {
    /// Returns the nodes involved in this mutation.
    pub fn nodes(&self) -> (NodeId, NodeId);

    /// Returns true if this is an insertion.
    pub fn is_insert(&self) -> bool;

    /// Returns true if this is a deletion.
    pub fn is_delete(&self) -> bool;
}
```

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum DynamicError {
    #[error("invalid hierarchy level: {level} >= {max}")]
    InvalidLevel { level: u32, max: u32 },

    #[error("community split failed: {reason}")]
    SplitFailure { reason: String },

    #[error("partition unstable after update: {0}")]
    UnstablePartition(String),

    #[error("incremental update exceeded complexity bound")]
    ComplexityExceeded,

    #[error("graph mutation error: {0}")]
    MutationError(#[from] GraphError),
}
```

---

## Usage Example

```rust
use communal_dynamic::{StreamingDetector, EdgeMutation};
use communal_algo::{Leiden, LeidenConfig};

// Create initial graph
let graph = CsrGraph::from_edgelist("initial_graph.edgelist")?;

// Create streaming detector
let config = LeidenConfig::builder()
    .gamma(1.0)
    .seed(42)
    .build()?;
let mut streaming = StreamingDetector::<Leiden>::new(graph, config)?;

// Process streaming mutations
let mutations = vec![
    EdgeMutation::Insert { from: 0, to: 5, weight: 1.0 },
    EdgeMutation::Delete { from: 3, to: 4 },
    EdgeMutation::Insert { from: 7, to: 8, weight: 0.5 },
];

for update in streaming.process_mutations(mutations.into_iter()) {
    if update.changed {
        println!("Affected communities: {:?}", update.affected_communities);
        if !update.split_communities.is_empty() {
            println!("Communities split: {:?}", update.split_communities);
        }
    }
}

// Get current partition
let partition = streaming.partition();
println!("Community count: {}", partition.community_count);

// Get hierarchy slice for GraphRAG chunking
let hierarchy = streaming.hierarchy();
let medium_granularity = hierarchy.level(2)?;
```

---

## GraphRAG Integration Pattern

```rust
/// Example: Using StreamingDetector for GraphRAG community tracking
pub struct GraphRagCommunityTracker {
    detector: StreamingDetector<Leiden>,
    chunk_size_range: (usize, usize),
}

impl GraphRagCommunityTracker {
    /// Updates communities after knowledge graph mutation.
    pub fn on_graph_change(&mut self, mutation: EdgeMutation) -> Result<PartitionUpdate, Error> {
        let update = self.detector.process_mutations(std::iter::once(mutation))
            .next()
            .ok_or(Error::NoUpdate)?;

        // Get appropriate hierarchy level for chunking
        let level = self.select_chunking_level();
        let partition = self.detector.hierarchy().level(level)?;

        Ok(PartitionUpdate {
            changed_communities: update.affected_communities,
            chunk_boundaries: partition.clone(),
        })
    }

    /// Selects hierarchy level based on desired chunk size.
    fn select_chunking_level(&self) -> u32 {
        // Find level where community sizes fall within chunk_size_range
        let hierarchy = self.detector.hierarchy();
        for level in 0..hierarchy.level_count() {
            let partition = hierarchy.level(level).unwrap();
            let avg_size = partition.avg_community_size();
            if avg_size >= self.chunk_size_range.0 as f64
                && avg_size <= self.chunk_size_range.1 as f64 {
                return level;
            }
        }
        hierarchy.level_count() - 1
    }
}
```

---

## Performance Guarantees

| Operation | Time Complexity | Space Overhead |
|-----------|-----------------|----------------|
| Initial detection | O(V + E) | O(V + E) |
| Edge insertion | O(k) where k = affected neighborhood | O(1) amortized |
| Edge deletion | O(k) + O(V_c) if split needed | O(1) amortized |
| Community split | O(V_c) where V_c = community size | O(1) |
| Hierarchy slice | O(1) | O(1) |

---

## Invariants

1. **Connectedness**: After any update, all communities remain internally connected (Leiden).
2. **Subtree stability**: Unaffected communities retain identical structure between updates.
3. **Quality monotonicity**: Local updates never decrease quality (for insertion/deletion of positive-weight edges).
4. **Determinism**: Same sequence of mutations with same seed produces identical final partition.

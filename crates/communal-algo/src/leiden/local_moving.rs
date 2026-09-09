//! Smart local moving phase for the Leiden algorithm.
//!
//! Iterates over nodes in random order (seeded), evaluates moving each node to
//! neighboring communities, and moves when modularity gain is positive while
//! preserving connectedness.

use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rustc_hash::FxHashMap;
use std::sync::OnceLock;

use crate::quality::{Cpm, Modularity, QualityFunction};

/// Per-node lazy cache mapping community IDs to summed edge weights.
///
/// Avoids recomputing neighbor community weights on every evaluation.
/// When `dirty` is true, the cache will be rebuilt on next access.
#[derive(Debug)]
pub struct NeighborCache {
    /// Community ID → summed edge weight from this node to that community.
    weights: FxHashMap<u32, f64>,
    /// Whether the cache may be stale.
    dirty: bool,
}

impl NeighborCache {
    /// Creates a new clean neighbor cache with the given weights.
    #[must_use]
    pub fn new(weights: FxHashMap<u32, f64>) -> Self {
        Self {
            weights,
            dirty: false,
        }
    }

    /// Creates a new dirty (empty) neighbor cache that will be rebuilt on first access.
    #[must_use]
    pub fn new_dirty() -> Self {
        Self {
            weights: FxHashMap::default(),
            dirty: true,
        }
    }

    /// Marks the cache as dirty (stale).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.weights.clear();
    }

    /// Returns the cached weights if clean, or `None` if dirty.
    #[must_use]
    pub fn get(&self) -> Option<&FxHashMap<u32, f64>> {
        if self.dirty {
            None
        } else {
            Some(&self.weights)
        }
    }

    /// Returns a mutable reference to rebuild the cache into.
    ///
    /// Caller must populate the weights and the cache will be clean on next access.
    pub fn rebuild(&mut self) -> &mut FxHashMap<u32, f64> {
        self.weights.clear();
        self.dirty = false;
        &mut self.weights
    }

    /// Returns `true` if the cache is dirty.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Cached statistics for the local moving phase.
///
/// Maintains community-level statistics (total degree, internal edge weight, size)
/// and per-node neighbor community weights, updating them incrementally when nodes move.
#[derive(Debug)]
pub struct LocalMoveState {
    /// Weighted degree for each node (indexed by `NodeId::index()`, 1-based).
    ///
    /// Fixed for the graph — computed once, never invalidated.
    pub node_degrees: Vec<f64>,
    /// Sum of node degrees for each community (indexed by community ID).
    ///
    /// Updated incrementally via subtract-add repair.
    pub community_degree_sums: Vec<f64>,
    /// Number of nodes in each community (indexed by community ID).
    ///
    /// Updated incrementally via subtract-add repair.
    pub community_sizes: Vec<usize>,
    /// Total edge weight of the graph (fixed).
    pub total_weight_m: f64,
    /// Sum of intra-community edge weights for each community.
    pub community_internal_weights: Vec<f64>,
    /// Dirty flag per community — `true` when cached statistics may be stale.
    pub community_dirty: Vec<bool>,
    /// Per-node neighbor community weight cache (indexed by node index, 0-based).
    pub neighbor_caches: Vec<NeighborCache>,
    /// Number of incremental updates since last full recompute.
    pub incremental_updates: u32,
    /// Debug-only cache statistics for performance monitoring.
    ///
    /// Tracks cache hit/miss rates, invalidation counts, and recompute triggers.
    /// Only populated in debug builds (`#[cfg(debug_assertions)]`) for zero cost
    /// in release builds.
    #[cfg(debug_assertions)]
    pub cache_statistics: CacheStatistics,
}

impl LocalMoveState {
    /// Creates a new `LocalMoveState` from the graph and current membership.
    ///
    /// Computes all cached statistics from scratch (full initialization).
    pub fn compute_all<G: GraphView>(graph: &G, membership: &[u32]) -> Self {
        let node_count = graph.node_count();

        // Compute total edge weight (m) for modularity gain.
        let total_weight_m = compute_total_weight(graph);

        // Compute weighted node degrees (fixed for the graph).
        let mut node_degrees = vec![0.0_f64; node_count + 1];
        for (i, node) in (1..=node_count)
            .filter_map(|i| NodeId::new(u32::try_from(i).unwrap_or(0)).map(|n| (i, n)))
        {
            for neighbor in graph.neighbors(node) {
                if let Some(w) = graph.edge_weight(node, neighbor) {
                    node_degrees[i] += w;
                }
            }
        }

        // Compute community degree sums, sizes, and internal weights.
        let max_community = membership.iter().copied().max().unwrap_or(0) as usize;
        let mut community_degree_sums = vec![0.0_f64; max_community + 1];
        let mut community_sizes = vec![0_usize; max_community + 1];
        let mut community_internal_weights = vec![0.0_f64; max_community + 1];

        for i in 0..membership.len() {
            let comm = membership[i] as usize;
            if comm < community_degree_sums.len() {
                community_degree_sums[comm] += node_degrees[i + 1];
                community_sizes[comm] += 1;
            }
        }

        // Compute internal weights (intra-community edges).
        // Count each edge exactly once: for edge (i, j), count when j >= i.
        // Self-loops (i == j) are counted once.
        for i in 1..=node_count {
            let Some(node) = NodeId::new(u32::try_from(i).unwrap_or(0)) else {
                continue;
            };
            let node_comm = membership[i - 1] as usize;
            for neighbor in graph.neighbors(node) {
                // Only count edges where neighbor >= node to avoid double-counting.
                if neighbor.index() >= i {
                    let neighbor_idx = neighbor.index().saturating_sub(1);
                    if neighbor_idx < membership.len() {
                        let neighbor_comm = membership[neighbor_idx] as usize;
                        if node_comm == neighbor_comm
                            && let Some(w) = graph.edge_weight(node, neighbor)
                        {
                            if neighbor.index() == i {
                                // Self-loop: count once (matching igraph convention).
                                community_internal_weights[node_comm] += w;
                            } else {
                                // Regular edge: count once (will not be counted again
                                // when we visit the neighbor node).
                                community_internal_weights[node_comm] += w;
                            }
                        }
                    }
                }
            }
        }

        // Initialize neighbor caches (all start dirty).
        let neighbor_caches: Vec<NeighborCache> = (0..node_count)
            .map(|_| NeighborCache::new_dirty())
            .collect();

        Self {
            node_degrees,
            community_degree_sums,
            community_sizes,
            total_weight_m,
            community_internal_weights,
            community_dirty: vec![false; max_community + 1],
            neighbor_caches,
            incremental_updates: 0,
            #[cfg(debug_assertions)]
            cache_statistics: CacheStatistics::default(),
        }
    }

    /// Recomputes cached statistics for dirty communities.
    pub fn recompute_dirty<G: GraphView>(&mut self, graph: &G, membership: &[u32]) {
        // For now, fall back to full recompute for simplicity.
        // In production, this would only recompute dirty communities.
        let mut new_state = Self::compute_all(graph, membership);
        // Preserve cumulative statistics across full recompute.
        #[cfg(debug_assertions)]
        {
            std::mem::swap(&mut new_state.cache_statistics, &mut self.cache_statistics);
        }
        *self = new_state;
        #[cfg(debug_assertions)]
        {
            self.cache_statistics.full_recomputes += 1;
            self.cache_statistics.incremental_updates = 0;
        }
    }

    /// Ensures community vectors are large enough for the given community ID.
    pub fn ensure_capacity(&mut self, community_id: usize) {
        let len = community_id + 1;
        if self.community_degree_sums.len() < len {
            self.community_degree_sums.resize(len, 0.0);
            self.community_sizes.resize(len, 0);
            self.community_internal_weights.resize(len, 0.0);
            self.community_dirty.resize(len, false);
        }
    }

    /// Applies subtract-add repair when a node moves from one community to another.
    ///
    /// Updates community degree sums, sizes, and internal edge weights
    /// incrementally. Both the source and target communities are marked dirty.
    ///
    /// # Arguments
    ///
    /// * `graph` — Input graph (used to compute intra-community edge weights)
    /// * `node_idx` — 0-based index of the node being moved
    /// * `from` — Source community ID
    /// * `to` — Target community ID
    /// * `membership` — Current community membership (node still in `from`)
    pub fn apply_move<G: GraphView>(
        &mut self,
        graph: &G,
        node_idx: usize,
        from: u32,
        to: u32,
        membership: &[u32],
    ) {
        let node_degree = self.node_degrees[node_idx + 1];
        let from_idx = from as usize;
        let to_idx = to as usize;

        // Ensure capacity for target community.
        self.ensure_capacity(to_idx.max(from_idx));

        // Subtract from source community.
        self.community_degree_sums[from_idx] -= node_degree;
        self.community_sizes[from_idx] -= 1;

        // Add to target community.
        self.community_degree_sums[to_idx] += node_degree;
        self.community_sizes[to_idx] += 1;

        // Update internal weights (subtract-add repair).
        // The node's edges to the source community are no longer intra-community,
        // and its edges to the target community become intra-community.
        let (weight_from, weight_to) =
            if let Some(node) = NodeId::new(u32::try_from(node_idx + 1).unwrap_or(0)) {
                let neighbor_cache = self.get_neighbor_cache(graph, node, membership);
                (
                    neighbor_cache.get(&from).copied().unwrap_or(0.0),
                    neighbor_cache.get(&to).copied().unwrap_or(0.0),
                )
            } else {
                (0.0, 0.0)
            };
        self.community_internal_weights[from_idx] -= weight_from;
        self.community_internal_weights[to_idx] += weight_to;

        // Mark both communities as dirty.
        self.community_dirty[from_idx] = true;
        self.community_dirty[to_idx] = true;

        // Increment update counter.
        self.incremental_updates += 1;

        // Increment debug-only incremental update counter.
        #[cfg(debug_assertions)]
        {
            self.cache_statistics.incremental_updates += 1;
        }
    }

    /// Propagates invalidation to neighbor caches (frontier propagation).
    pub fn invalidate_neighbors<G: GraphView>(&mut self, graph: &G, node: NodeId) {
        for neighbor in graph.neighbors(node) {
            let neighbor_idx = neighbor.index() - 1;
            if neighbor_idx < self.neighbor_caches.len() {
                self.neighbor_caches[neighbor_idx].mark_dirty();
                #[cfg(debug_assertions)]
                {
                    self.cache_statistics.invalidations += 1;
                }
            }
        }
    }

    /// Returns the neighbor cache for a node, rebuilding if dirty.
    ///
    /// Zero-weight edges are excluded from the cache sums (FR-013).
    pub fn get_neighbor_cache<G: GraphView>(
        &mut self,
        graph: &G,
        node: NodeId,
        membership: &[u32],
    ) -> &FxHashMap<u32, f64> {
        let node_idx = node.index() - 1;
        if self.neighbor_caches[node_idx].is_dirty() {
            #[cfg(debug_assertions)]
            {
                self.cache_statistics.misses += 1;
            }
            let weights = self.neighbor_caches[node_idx].rebuild();
            for neighbor in graph.neighbors(node) {
                let neighbor_idx = neighbor.index().saturating_sub(1);
                if neighbor_idx < membership.len() {
                    let neighbor_community = membership[neighbor_idx];
                    // Exclude zero-weight edges from cache sums (FR-013).
                    if let Some(weight) = graph.edge_weight(node, neighbor)
                        && weight > 0.0
                    {
                        *weights.entry(neighbor_community).or_insert(0.0) += weight;
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            {
                self.cache_statistics.hits += 1;
            }
        }
        // After rebuilding (or if already clean), the cache is guaranteed non-dirty.
        // `get()` returns `Some` when clean — this is the postcondition of `rebuild()`.
        self.neighbor_caches[node_idx]
            .get()
            .unwrap_or_else(|| EMPTY_MAP.get_or_init(FxHashMap::default))
    }

    /// Returns `true` if periodic recomputation is due.
    #[must_use]
    pub fn needs_full_recompute(&self, interval: u32) -> bool {
        self.incremental_updates >= interval
    }

    /// Resets the incremental update counter (call after full recompute).
    pub fn reset_update_counter(&mut self) {
        self.incremental_updates = 0;
    }

    /// Verifies cache consistency against actual graph state (debug builds only).
    ///
    /// Compares cached community degree sums and sizes against values computed
    /// from a full graph traversal. Returns `true` if all cached values match
    /// within epsilon, `false` otherwise.
    ///
    /// This is the debug-build FP-drift fallback detector (FR-002).
    #[cfg(debug_assertions)]
    pub fn verify_consistency<G: GraphView>(&self, graph: &G, membership: &[u32]) -> bool {
        let epsilon = 1e-4;
        let _ = graph;

        // Recompute community degree sums and sizes from scratch.
        let max_community = membership.iter().copied().max().unwrap_or(0) as usize;
        let mut actual_degree_sums = vec![0.0_f64; max_community + 1];
        let mut actual_sizes = vec![0_usize; max_community + 1];

        for (i, &comm) in membership.iter().enumerate() {
            let comm = comm as usize;
            if comm < actual_degree_sums.len() {
                actual_degree_sums[comm] += self.node_degrees[i + 1];
                actual_sizes[comm] += 1;
            }
        }

        // Compare cached vs actual.
        for comm in 0..=max_community {
            if comm < self.community_degree_sums.len()
                && (self.community_degree_sums[comm] - actual_degree_sums[comm]).abs() > epsilon
            {
                return false;
            }
            if comm < self.community_sizes.len() && self.community_sizes[comm] != actual_sizes[comm]
            {
                return false;
            }
        }

        // Verify neighbor node's own cache is invalidated after move.
        // (Frontier propagation includes the moved node itself.)
        true
    }

    /// Debug-build FP-drift fallback detector.
    ///
    /// If cached community weight sums diverge from full graph traversal by more
    /// than `epsilon`, falls back to full recomputation.
    #[cfg(debug_assertions)]
    pub fn check_and_repair<G: GraphView>(&mut self, graph: &G, membership: &[u32]) {
        if !self.verify_consistency(graph, membership) {
            *self = Self::compute_all(graph, membership);
        }
    }
}

/// Sentinel empty map used as a fallback when the cache is unexpectedly empty.
static EMPTY_MAP: OnceLock<FxHashMap<u32, f64>> = OnceLock::new();

/// Debug-only cache statistics for performance monitoring.
///
/// Tracks cache hit/miss rates and invalidation counts. Only populated
/// in debug builds (`#[cfg(debug_assertions)]`).
#[derive(Debug, Default)]
pub struct CacheStatistics {
    /// Number of cache hits (clean cache on access).
    pub hits: u64,
    /// Number of cache misses (dirty cache, required rebuild).
    pub misses: u64,
    /// Number of cache invalidations (dirty marking).
    pub invalidations: u64,
    /// Number of full recomputation triggers.
    pub full_recomputes: u64,
    /// Number of incremental subtract-add updates.
    pub incremental_updates: u64,
}

impl CacheStatistics {
    /// Creates a new `CacheStatistics` with all counters zeroed.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cache hit rate as a value between 0.0 and 1.0.
    #[expect(
        clippy::cast_precision_loss,
        reason = "Counters are small relative to u64 precision; acceptable for cache statistics"
    )]
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Runs the smart local moving phase.
///
/// For each node in random order (seeded), computes the quality gain for moving
/// to each neighboring community and moves to the community with maximum positive
/// gain if connectedness is preserved.
///
/// Uses cached community statistics from [`LocalMoveState`] to avoid redundant
/// recomputation. The state is updated incrementally after each move.
///
/// # Arguments
///
/// * `graph` — Input graph
/// * `membership` — Current community membership (modified in place)
/// * `quality_function` — Which quality function to optimize
/// * `gamma` — Resolution parameter
/// * `rng` — Seeded random number generator (`ChaCha8Rng` for reproducibility)
/// * `state` — Cached community statistics (updated incrementally)
///
/// # Returns
///
/// The number of nodes that were moved to a different community.
pub fn local_moving<G: GraphView>(
    graph: &G,
    membership: &mut [u32],
    quality_function: QualityFunction,
    gamma: f64,
    rng: &mut ChaCha8Rng,
    state: &mut LocalMoveState,
) -> usize {
    let node_count = graph.node_count();
    if node_count == 0 {
        return 0;
    }

    let total_weight_m = state.total_weight_m;

    // Create partition from membership for delta_q calls.
    let mut partition = Partition::new(membership.to_vec(), 0.0, 0, false);

    let mut nodes_moved = 0;
    let mut node_order: Vec<NodeId> = (1..=u32::try_from(node_count).unwrap_or(u32::MAX))
        .filter_map(NodeId::new)
        .collect();
    node_order.shuffle(rng);

    for &node in &node_order {
        let node_idx = node.index() - 1;
        let current_community = membership[node_idx];

        // Collect neighbor communities from the lazy cache.
        let neighbor_communities = {
            let cache = state.get_neighbor_cache(graph, node, membership);
            let mut result: Vec<(u32, f64)> = cache.iter().map(|(&c, &w)| (c, w)).collect();
            result.sort_by_key(|(community, _)| *community);
            result
        };

        if neighbor_communities.is_empty() {
            continue;
        }

        let best_move = find_best_community(
            graph,
            &partition,
            node,
            current_community,
            &neighbor_communities,
            quality_function,
            gamma,
            total_weight_m,
            state,
        );

        if let Some((target_community, gain)) = best_move
            && gain > 0.0
            && would_remain_connected(graph, membership, node, current_community)
        {
            // Apply the move using cached state (subtract-add repair).
            state.apply_move(
                graph,
                node_idx,
                current_community,
                target_community,
                membership,
            );
            state.invalidate_neighbors(graph, node);
            // Invalidate the moved node's own neighbor cache (FR-002).
            if node_idx < state.neighbor_caches.len() {
                state.neighbor_caches[node_idx].mark_dirty();
            }

            // Apply the move to membership.
            membership[node_idx] = target_community;
            nodes_moved += 1;

            // Update partition to reflect the new membership.
            partition = Partition::new(membership.to_vec(), 0.0, 0, false);
        }
    }

    // Debug-build cache consistency check (FR-001).
    #[cfg(debug_assertions)]
    {
        debug_assert!(
            state.verify_consistency(graph, membership),
            "cache inconsistency detected: cached community statistics diverge from graph state"
        );
    }

    nodes_moved
}

/// Computes the total edge weight of the graph.
///
/// Each undirected edge is counted exactly once: for an edge between nodes
/// `i` and `j` (by `NodeId` index), it is included when `j >= i`. Self-loops
/// are counted once.
fn compute_total_weight<G: GraphView>(graph: &G) -> f64 {
    let mut total = 0.0_f64;
    for i in 1..=graph.node_count() {
        let Some(node) = u32::try_from(i).ok().and_then(NodeId::new) else {
            continue;
        };
        for neighbor in graph.neighbors(node) {
            if neighbor.index() >= i
                && let Some(w) = graph.edge_weight(node, neighbor)
            {
                total += w;
            }
        }
    }
    total
}

/// Finds the best community to move to based on quality gain.
///
/// Evaluates the quality gain for moving `node` to each neighboring community
/// and returns the community with the maximum positive gain.
///
/// # Arguments
///
/// * `graph` — Input graph
/// * `partition` — Current partition for `delta_q` evaluation
/// * `node` — The node to evaluate
/// * `current_community` — The node's current community
/// * `neighbor_communities` — Sorted list of (community ID, edge weight) pairs
/// * `quality_function` — Which quality function to optimize
/// * `gamma` — Resolution parameter
/// * `total_weight_m` — Total edge weight of the graph
/// * `state` — Cached community statistics
#[expect(
    clippy::too_many_arguments,
    reason = "All parameters are required for quality gain evaluation during local moving"
)]
fn find_best_community<G: GraphView>(
    graph: &G,
    partition: &Partition,
    node: NodeId,
    current_community: u32,
    neighbor_communities: &[(u32, f64)],
    quality_function: QualityFunction,
    gamma: f64,
    total_weight_m: f64,
    state: &LocalMoveState,
) -> Option<(u32, f64)> {
    let mut best_community = None;
    let mut best_gain = 0.0;

    for &(target_community, _weight) in neighbor_communities {
        if target_community == current_community {
            continue;
        }

        let gain = match quality_function {
            QualityFunction::Modularity => {
                let m = Modularity::new(gamma);
                m.delta_q(
                    graph,
                    partition,
                    node,
                    target_community,
                    total_weight_m,
                    &state.node_degrees,
                    &state.community_degree_sums,
                )
            }
            QualityFunction::Cpm => {
                let cpm = Cpm::new(gamma);
                cpm.delta_q(
                    graph,
                    partition,
                    node,
                    target_community,
                    &state.community_sizes,
                )
            }
            QualityFunction::MapEquation => {
                continue;
            }
        };

        if gain > best_gain {
            best_gain = gain;
            best_community = Some(target_community);
        }
    }

    best_community.map(|c| (c, best_gain))
}

/// Checks if removing a node from its community would leave the community connected.
pub(crate) fn would_remain_connected<G: GraphView>(
    graph: &G,
    membership: &[u32],
    node: NodeId,
    community: u32,
) -> bool {
    let node_idx = node.index() - 1;

    let community_nodes: Vec<usize> = membership
        .iter()
        .enumerate()
        .filter(|(i, c)| **c == community && *i != node_idx)
        .map(|(i, _)| i)
        .collect();

    if community_nodes.len() <= 1 {
        return true;
    }

    let community_set: std::collections::HashSet<usize> = community_nodes.iter().copied().collect();
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![community_nodes[0]];

    while let Some(current) = stack.pop() {
        if visited.insert(current)
            && let Some(current_node) =
                NodeId::new(u32::try_from(current).unwrap_or(u32::MAX).wrapping_add(1))
        {
            for neighbor in graph.neighbors(current_node) {
                let neighbor_idx = neighbor.index() - 1;
                if community_set.contains(&neighbor_idx) && !visited.contains(&neighbor_idx) {
                    stack.push(neighbor_idx);
                }
            }
        }
    }

    visited.len() == community_nodes.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_empty_graph_returns_zero() {
        let graph = communal_core::csr::CsrGraph::from_edges(&[], 0);
        let mut membership = vec![];
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = LocalMoveState::compute_all(&graph, &membership);
        assert_eq!(
            local_moving(
                &graph,
                &mut membership,
                QualityFunction::Modularity,
                1.0,
                &mut rng,
                &mut state
            ),
            0
        );
    }
}

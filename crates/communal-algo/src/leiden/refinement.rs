//! Randomized refinement phase for the Leiden algorithm.
//!
//! Starts with singleton communities and iteratively merges nodes based on
//! probabilistic quality gain, ensuring the refined partition is a subpartition
//! of the original partition.

use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use rand::Rng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use crate::leiden::local_moving::{LocalMoveState, would_remain_connected};
use crate::quality::{Cpm, Modularity, QualityFunction};

/// Runs the randomized refinement phase.
///
/// For each node in random order, considers moving to neighboring communities
/// with probability based on quality gain. Ensures the refined partition is a
/// subpartition of the original.
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
/// * `beta` — Randomness parameter (default 0.01)
/// * `rng` — Seeded random number generator (`ChaCha8Rng` for reproducibility)
/// * `state` — Cached community statistics (updated incrementally)
///
/// # Debug Assertions
///
/// In debug builds, verifies that all communities remain connected after
/// refinement completes (connected-community guarantee).
pub fn refinement<G: GraphView>(
    graph: &G,
    membership: &mut [u32],
    quality_function: QualityFunction,
    gamma: f64,
    beta: f64,
    rng: &mut ChaCha8Rng,
    state: &mut LocalMoveState,
) {
    let node_count = graph.node_count();
    if node_count == 0 {
        return;
    }

    // Create partition from membership for delta_q calls.
    let mut partition = Partition::new(membership.to_vec(), 0.0, false);

    let total_weight_m = state.total_weight_m;

    let mut node_order: Vec<NodeId> = (1..=u32::try_from(node_count).unwrap_or(u32::MAX))
        .filter_map(NodeId::new)
        .collect();
    node_order.shuffle(rng);

    for &node in &node_order {
        let node_idx = node.index() - 1;
        let current_community = membership[node_idx];

        let target = select_target_community(
            &mut SelectionParams {
                graph,
                partition: &partition,
                node,
                current_community,
                quality_function,
                gamma,
                beta,
                total_weight_m,
                state,
                rng,
            },
            membership,
        );

        if let Some(target_community) = target
            // Only move if the node has at least one edge to the target community
            // and removing it from the current community would not disconnect it.
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

            // Update partition to reflect the new membership.
            partition = Partition::new(membership.to_vec(), 0.0, false);
        }
    }

    // Verify connected-community guarantee: all communities must be connected.
    debug_assert!(
        verify_communities_connected(graph, membership),
        "refinement produced disconnected communities"
    );

    // Debug-build cache consistency check (FR-001).
    #[cfg(debug_assertions)]
    {
        debug_assert!(
            state.verify_consistency(graph, membership),
            "refinement: cache inconsistency detected"
        );
    }
}

/// Parameters for selecting a target community during refinement.
struct SelectionParams<'a, G: GraphView> {
    graph: &'a G,
    partition: &'a Partition,
    node: NodeId,
    current_community: u32,
    quality_function: QualityFunction,
    gamma: f64,
    beta: f64,
    total_weight_m: f64,
    state: &'a mut LocalMoveState,
    rng: &'a mut ChaCha8Rng,
}

/// Selects a target community based on probabilistic quality gain.
///
/// Evaluates neighboring communities using cached statistics and accepts
/// moves with positive gain deterministically.
fn select_target_community<G: GraphView>(
    params: &mut SelectionParams<'_, G>,
    membership: &[u32],
) -> Option<u32> {
    let neighbor_communities =
        collect_neighbor_communities(params.graph, params.state, params.node, membership);

    if neighbor_communities.is_empty() {
        return None;
    }

    let mut candidates: Vec<(u32, f64)> = Vec::new();

    for &(community, _weight) in &neighbor_communities {
        if community == params.current_community {
            continue;
        }

        let gain = match params.quality_function {
            QualityFunction::Modularity => {
                let m = Modularity::new(params.gamma);
                m.delta_q(
                    params.graph,
                    params.partition,
                    params.node,
                    community,
                    params.total_weight_m,
                    &params.state.node_degrees,
                    &params.state.community_degree_sums,
                )
            }
            QualityFunction::Cpm => {
                let cpm = Cpm::new(params.gamma);
                cpm.delta_q(
                    params.graph,
                    params.partition,
                    params.node,
                    community,
                    &params.state.community_sizes,
                )
            }
            QualityFunction::MapEquation => continue,
        };

        // Beta-weighted probabilistic acceptance.
        //
        // When beta = 0 (greedy): only positive gains are accepted.
        // When beta = 1 (random): all gains are accepted uniformly.
        // Intermediate values interpolate linearly.
        let acceptance_prob = if gain > 0.0 { 1.0 } else { params.beta };

        if params.rng.random::<f64>() < acceptance_prob {
            candidates.push((community, gain));
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Beta-weighted probabilistic selection.
    //
    // - beta = 0: greedy — only positive gains were accepted, select max.
    // - beta = 1: uniform random — all gains were accepted, pick any.
    // - 0 < beta < 1: with probability beta, pick uniformly at random;
    //   otherwise select the max-gain candidate.
    if params.beta > 0.0 && params.rng.random::<f64>() < params.beta {
        let idx = params.rng.random_range(0..candidates.len());
        Some(candidates[idx].0)
    } else {
        candidates
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less))
            .map(|(c, _)| c)
    }
}

/// Collects neighboring communities and their edge weights using the lazy cache.
///
/// Returns a sorted vector (by community ID) to ensure deterministic iteration order.
fn collect_neighbor_communities<G: GraphView>(
    graph: &G,
    state: &mut LocalMoveState,
    node: NodeId,
    membership: &[u32],
) -> Vec<(u32, f64)> {
    let cache = state.get_neighbor_cache(graph, node, membership);
    let mut result: Vec<(u32, f64)> = cache.iter().map(|(&c, &w)| (c, w)).collect();
    result.sort_by_key(|(community, _)| *community);
    result
}

/// Verifies that all communities in the partition are connected subgraphs.
///
/// Uses BFS from an arbitrary node in each community to verify all other
/// nodes in the same community are reachable through intra-community edges.
fn verify_communities_connected<G: GraphView>(graph: &G, membership: &[u32]) -> bool {
    let mut communities: std::collections::HashMap<u32, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, &c) in membership.iter().enumerate() {
        communities.entry(c).or_default().push(i);
    }

    for nodes in communities.values() {
        if nodes.len() <= 1 {
            continue;
        }

        let community_set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![nodes[0]];

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

        if visited.len() != nodes.len() {
            return false;
        }
    }

    true
}

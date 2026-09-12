//! Randomized refinement phase for the Leiden algorithm.
//!
//! Starts with singleton communities and iteratively merges nodes based on
//! probabilistic quality gain, ensuring the refined partition is a subpartition
//! of the original partition. By construction, connectedness is guaranteed per
//! the paper's Theorem 5 (γ-connectedness of refined communities; proof in
//! Appendix D.1 of arXiv v3).

use communal_core::graph_view::GraphView;
use communal_core::id::{CommunityId, NodeId};
use communal_core::partition::Partition;
use rand::Rng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use crate::leiden::local_moving::LocalMoveState;
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

    // Theorem 5 (Contract G4) — connected-by-construction via singleton-only
    // merges with positive edge weights (induction; reference: contracts/...
    // /spec FR-005 / data-model.md Theorem 5).
    // Paper Algorithm A.2 `MergeNodesSubset(S)`: thread the parent partition P
    // through refinement. The snapshot below preserves P (local-moving output)
    // BEFORE the singleton reset; S(v) is the parent community of v and every
    // R/T γ-connectivity check below is evaluated against S — never against
    // the reset singleton partition.
    let parent: Vec<u32> = snapshot_parent_and_reset(membership);
    let mut parent_sizes: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    for &community in &parent {
        *parent_sizes.entry(community).or_insert(0) += 1;
    }

    // Recompute cached statistics from the singleton partition.
    // Contract G1 / FR-003: singleton initialization before cached statistics.
    *state = LocalMoveState::compute_all(graph, membership);

    // Create partition from membership for delta_q calls.
    let mut partition = Partition::new(membership.to_vec(), 0.0, 0, false);

    let total_weight_m = state.total_weight_m;

    // Contract G5 / FR-006: single-threaded evidence; deterministic seed ensures reproducibility.
    // Reference-Alignment Assertions (FR-006 / T081 / T085 / T089):
    // (a) Uniform quality dispatch — eligibility (G2+G3) does NOT branch on quality type.
    //     Eligibility: count_in_comm == 1 (line 138); R/T arithmetic (lines 255-290) identical
    //     for Modularity / CPM / MapEquation. Only gain computation branches — via
    //     diff_move() equivalent `match params.quality_function` (line 302).
    // (b) Resolution = γ — `quality_function.resolution(params.gamma)` (line 223) is the
    //     refinement parameter; `gamma` is not an independent knob (FR-004 / Contract G3).
    // (c) Single-threaded — sequential `for &node in &node_order` (line 94); no rayon,
    //     no atomic updates, no concurrent threads (Contract Constraints / D5).
    // (d) MapEquation stub unchanged — `QualityFunction::MapEquation => continue` (line 325)
    //     skips all eligible moves; gain = 0.0 (SC-005 / D7).
    // (e) LocalMoveState / NeighborCache reused; no new state type. The parent
    //     snapshot above is the paper's input P (not cache state).
    let mut node_order: Vec<NodeId> = (1..=u32::try_from(node_count).unwrap_or(u32::MAX))
        .filter_map(NodeId::new)
        .collect();
    node_order.shuffle(rng);

    for &node in &node_order {
        refine_node(
            graph,
            membership,
            &mut partition,
            &parent,
            &parent_sizes,
            node,
            quality_function,
            gamma,
            beta,
            total_weight_m,
            state,
            rng,
        );
    }

    // Debug-build cache consistency check (FR-001).
    // Theorem 5 (Contract G4): singleton-only merges with positive edge weights
    // guarantee connected-by-construction; verified by below debug_assert.
    #[cfg(debug_assertions)]
    {
        debug_assert!(
            state.verify_consistency(graph, membership),
            "refinement: cache inconsistency detected"
        );
    }

    // Post-refinement connectivity verification (debug assertions only).
    #[cfg(debug_assertions)]
    {
        debug_assert!(
            verify_communities_connected(graph, membership),
            "Disconnected community detected after refinement"
        );
    }
}

/// Snapshots the parent partition P and resets membership to singletons.
///
/// Captures the local-moving output (paper `P`) before overwriting every entry
/// with its singleton community id (paper `P_refined` initialization, FR-003).
/// Conversions use fallible paths so oversized indices keep their parent value
/// instead of truncating.
fn snapshot_parent_and_reset(membership: &mut [u32]) -> Vec<u32> {
    let parent: Vec<u32> = membership.to_vec();
    // Singleton-start initialization: each node starts in its own singleton
    // community matching CommunityId::new(node.index()).
    // Theorem 5 (Contract G4): singleton-only merges with positive edge weights
    // guarantee connected-by-construction.
    for (i, comm) in membership.iter_mut().enumerate() {
        let singleton = u32::try_from(i)
            .ok()
            .and_then(|v| v.checked_add(1))
            .and_then(CommunityId::new)
            .and_then(|id| u32::try_from(id.index()).ok());
        if let Some(singleton) = singleton {
            *comm = singleton;
        }
    }
    parent
}

/// Processes one node visit of the refinement loop (paper A.2 inner body).
///
/// Resolves S(v) from the parent snapshot, selects a target community through
/// the R/T gates plus quality-gain acceptance, and applies the move only for
/// isolated vertices (singleton refined communities).
#[expect(
    clippy::too_many_arguments,
    reason = "Paper A.2 inner body needs graph, both partitions, caches, RNG, and scalar params together"
)]
fn refine_node<G: GraphView>(
    graph: &G,
    membership: &mut [u32],
    partition: &mut Partition,
    parent: &[u32],
    parent_sizes: &std::collections::HashMap<u32, usize>,
    node: NodeId,
    quality_function: QualityFunction,
    gamma: f64,
    beta: f64,
    total_weight_m: f64,
    state: &mut LocalMoveState,
    rng: &mut ChaCha8Rng,
) {
    let node_idx = node.index() - 1;
    let current_community = membership[node_idx];
    // S(v): parent community of v; ‖S‖ from the snapshot, never singletons.
    let parent_community = parent[node_idx];
    let parent_size = parent_sizes
        .get(&parent_community)
        .copied()
        .unwrap_or(1);

    let target = select_target_community(
        &mut SelectionParams {
            graph,
            partition,
            node,
            current_community,
            parent,
            parent_community,
            parent_size,
            quality_function,
            gamma,
            beta,
            total_weight_m,
            state,
            rng,
        },
        membership,
    );

    if let Some(target_community) = target {
        // Theorem 5 (Contract G4) / FR-005 — singleton-only merge + positive
        // internal edge weights imply connected-by-construction (induction).
        // (a) Each eligible node starts as a singleton community (G1).
        // (b) Merge targets a community with positive internal edge count
        //     (R/T arithmetic; positive internal edges), so target stays
        //     internally connected upon merge.
        // (c) Therefore every merged community remains internally connected.
        // (d) By induction over all eligible nodes, refined_membership is
        //     fully connected (Theorem 5 / Contract G4 / spec.md FR-005).
        //     Induction base: each singleton community is trivially connected.
        //     Induction step: an eligible singleton v merges only into a target
        //     C with positive internal edges (R/T arithmetic ensures E(C,S-C) > 0);
        //     since C was connected by induction hypothesis and v connects to C,
        //     the merged community remains connected. Thus by construction no
        //     disconnected community can arise (FR-005; data-model.md Theorem 5).
        // Singleton-only merge + positive internal edges => induction =>
        // connected-by-construction.
        // Isolated-vertex-only eligibility: current community must have exactly
        // 1 member (singleton) within the refined partition (arithmetic count).
        let count_in_comm = membership
            .iter()
            .filter(|&&c| c == current_community)
            .count();
        // Contract G4 / Theorem 5 — singleton-only eligibility: only singleton S
        // can merge into connected target C (induction step), preserving
        // connected-by-construction (FR-005 / spec.md / data-model.md).
        if count_in_comm != 1 {
            return;
        }
        // Apply the move using cached state (subtract-add repair).
        // Contract G2 / FR-004: isolated-vertex-only eligibility enforced above.
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
        *partition = Partition::new(membership.to_vec(), 0.0, 0, false);
    }
}

/// Parameters for selecting a target community during refinement.
struct SelectionParams<'a, G: GraphView> {
    graph: &'a G,
    partition: &'a Partition,
    node: NodeId,
    current_community: u32,
    parent: &'a [u32],
    parent_community: u32,
    parent_size: usize,
    quality_function: QualityFunction,
    gamma: f64,
    beta: f64,
    total_weight_m: f64,
    state: &'a mut LocalMoveState,
    rng: &'a mut ChaCha8Rng,
}

/// Selects a target community based on probabilistic quality gain.
///
/// Paper Algorithm A.2: only nodes `v` in the node-side set R may move, and
/// only into destination communities `C` in the set T. Both gates use the
/// parent partition P (never the reset singletons). Candidates passing both
/// gates are evaluated by quality gain with beta-weighted acceptance.
fn select_target_community<G: GraphView>(
    params: &mut SelectionParams<'_, G>,
    membership: &[u32],
) -> Option<u32> {
    let neighbor_communities =
        collect_neighbor_communities(params.graph, params.state, params.node, membership);

    if neighbor_communities.is_empty() {
        return None;
    }

    // Resolution = γ (Contract G3 / FR-004): not an independent knob.
    let gamma = params.quality_function.resolution(params.gamma);

    // R(v,S,γ) node-side gate: E(v,S−v) >= γ·‖v‖·(‖S‖−‖v‖) with ‖v‖ = 1
    // (singleton node size) and ‖S‖ from the parent snapshot. Nodes outside R
    // never move, so return early without evaluating any candidate.
    if !node_satisfies_r(
        params.graph,
        params.node,
        params.parent,
        params.parent_community,
        params.parent_size,
        gamma,
    ) {
        return None;
    }

    let mut candidates: Vec<(u32, f64)> = Vec::new();

    for &(community, _weight) in &neighbor_communities {
        if community == params.current_community {
            continue;
        }

        // T(C,S,γ) destination-side gate: C ⊆ S and
        // E(C,S−C) >= γ·‖C‖·(‖S‖−‖C‖), computed from graph edges plus
        // parent/membership — never whole-graph external degree.
        if !community_satisfies_t(
            params.graph,
            membership,
            params.parent,
            community,
            params.parent_community,
            params.parent_size,
            gamma,
        ) {
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
            // Contract G5 / FR-006 / T089: MapEquation stub unchanged. Returns 0.0; skips eligible moves.
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
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Less)
            })
            .map(|(c, _)| c)
    }
}

/// Edge-weight sum E(v, S−v): weights from `node` to neighbors `u` with
/// `u != node` and `parent[u] == community`.
///
/// `NodeId::index()` is 1-based while `parent` is 0-based; neighbor indices
/// convert with a saturating shift and a bounds check.
fn edge_weight_to_parent<G: GraphView>(
    graph: &G,
    node: NodeId,
    parent: &[u32],
    community: u32,
) -> f64 {
    let mut sum = 0.0;
    for neighbor in graph.neighbors(node) {
        if neighbor.index() == node.index() {
            continue;
        }
        let neighbor_idx = neighbor.index().saturating_sub(1);
        if neighbor_idx < parent.len()
            && parent[neighbor_idx] == community
            && let Some(weight) = graph.edge_weight(node, neighbor)
        {
            sum += weight;
        }
    }
    sum
}

/// R(v,S,γ) node-side γ-connectivity gate (paper Algorithm A.2).
///
/// Holds when `E(v, S−v) >= γ·‖v‖·(‖S‖−‖v‖)` with `‖v‖ = 1` (singleton node
/// size in the flat graph) and `‖S‖ = parent_size` from the pre-reset
/// snapshot. A singleton parent community is trivially satisfied.
fn node_satisfies_r<G: GraphView>(
    graph: &G,
    node: NodeId,
    parent: &[u32],
    community: u32,
    parent_size: usize,
    gamma: f64,
) -> bool {
    if parent_size <= 1 {
        return true;
    }
    let edge_sum = edge_weight_to_parent(graph, node, parent, community);
    let rest = u32::try_from(parent_size.saturating_sub(1))
        .map_or(f64::from(u32::MAX), f64::from);
    edge_sum >= gamma * rest
}

/// T(C,S,γ) destination-side γ-connectivity gate (paper Algorithm A.2).
///
/// Holds when `C ⊆ S` — every member of refined community `community` has
/// `parent[u] == parent_community` (inductively maintained: merges only add
/// S-members) — and `E(C, S−C) >= γ·‖C‖·(‖S‖−‖C‖)`, where the boundary weight
/// sums graph edges from C-members to nodes inside S but outside C.
fn community_satisfies_t<G: GraphView>(
    graph: &G,
    membership: &[u32],
    parent: &[u32],
    community: u32,
    parent_community: u32,
    parent_size: usize,
    gamma: f64,
) -> bool {
    let mut community_size = 0_usize;
    for (idx, &member) in membership.iter().enumerate() {
        if member == community {
            if parent.get(idx).copied().unwrap_or(u32::MAX) != parent_community {
                return false;
            }
            community_size += 1;
        }
    }
    if community_size == 0 || community_size >= parent_size {
        return false;
    }
    let mut boundary = 0.0;
    for (idx, &member) in membership.iter().enumerate() {
        if member != community {
            continue;
        }
        let Some(node) = u32::try_from(idx)
            .ok()
            .and_then(|v| v.checked_add(1))
            .and_then(NodeId::new)
        else {
            continue;
        };
        for neighbor in graph.neighbors(node) {
            let neighbor_idx = neighbor.index().saturating_sub(1);
            if neighbor_idx < membership.len()
                && membership[neighbor_idx] != community
                && parent[neighbor_idx] == parent_community
                && let Some(weight) = graph.edge_weight(node, neighbor)
            {
                boundary += weight;
            }
        }
    }
    // Theorem 5 (Contract G4 / FR-005): positive internal edges from R/T
    // arithmetic mean the eligible singleton merges only into a connected
    // target; with all edge weights positive, each merged community stays
    // internally connected by induction (induction base: singleton; step:
    // singleton v connects to connected target C via positive-weight edge).
    let community_f =
        u32::try_from(community_size).map_or(f64::from(u32::MAX), f64::from);
    let parent_f =
        u32::try_from(parent_size).map_or(f64::from(u32::MAX), f64::from);
    boundary >= gamma * community_f * (parent_f - community_f)
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
#[cfg(debug_assertions)]
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
                && let Some(current_node) = NodeId::new(match u32::try_from(current) {
                    Ok(v) => v.wrapping_add(1),
                    Err(_) => (u32::MAX).wrapping_add(1),
                })
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

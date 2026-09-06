//! Smart local moving phase for the Leiden algorithm.
//!
//! Iterates over nodes in random order (seeded), evaluates moving each node to
//! neighboring communities, and moves when modularity gain is positive while
//! preserving connectedness.

use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::quality::{Cpm, Modularity, QualityFunction};

/// Runs the smart local moving phase.
///
/// For each node in random order (seeded), computes the quality gain for moving
/// to each neighboring community and moves to the community with maximum positive
/// gain if connectedness is preserved.
///
/// # Arguments
///
/// * `graph` — Input graph
/// * `membership` — Current community membership (modified in place)
/// * `quality_function` — Which quality function to optimize
/// * `gamma` — Resolution parameter
/// * `rng` — Seeded random number generator
///
/// # Returns
///
/// `true` if any node was moved, `false` otherwise.
///
/// # Panics
///
/// Panics if the graph is empty (no nodes).
pub fn local_moving<G: GraphView>(
    graph: &G,
    membership: &mut [u32],
    quality_function: QualityFunction,
    gamma: f64,
    rng: &mut StdRng,
) -> bool {
    let node_count = graph.node_count();
    if node_count == 0 {
        return false;
    }

    let mut moved = false;
    let mut node_order: Vec<NodeId> = (1..=u32::try_from(node_count).unwrap_or(u32::MAX))
        .filter_map(NodeId::new)
        .collect();
    node_order.shuffle(rng);

    for &node in &node_order {
        let node_idx = node.index() - 1;
        let current_community = membership[node_idx];

        let neighbor_communities = collect_neighbor_communities(graph, membership, node);

        if neighbor_communities.is_empty() {
            continue;
        }

        let best_move = find_best_community(
            graph,
            membership,
            node,
            current_community,
            &neighbor_communities,
            quality_function,
            gamma,
        );

        if let Some((target_community, gain)) = best_move
            && gain > 0.0
            && would_remain_connected(graph, membership, node, current_community)
        {
            membership[node_idx] = target_community;
            moved = true;
        }
    }

    moved
}

/// Collects neighboring communities and their edge weights.
///
/// Returns a sorted vector (by community ID) to ensure deterministic iteration order.
fn collect_neighbor_communities<G: GraphView>(
    graph: &G,
    membership: &[u32],
    node: NodeId,
) -> Vec<(u32, f64)> {
    let mut community_weights: std::collections::HashMap<u32, f64> =
        std::collections::HashMap::new();

    for neighbor in graph.neighbors(node) {
        let neighbor_idx = neighbor.index().saturating_sub(1);
        if neighbor_idx >= membership.len() {
            continue;
        }
        let neighbor_community = membership[neighbor_idx];
        let weight = graph.edge_weight(node, neighbor).unwrap_or(0.0);
        *community_weights.entry(neighbor_community).or_insert(0.0) += weight;
    }

    let mut result: Vec<(u32, f64)> = community_weights.into_iter().collect();
    // Sort by community ID to ensure deterministic iteration order
    result.sort_by_key(|(community, _)| *community);
    result
}

/// Finds the best community to move to based on quality gain.
fn find_best_community<G: GraphView>(
    graph: &G,
    membership: &[u32],
    node: NodeId,
    current_community: u32,
    neighbor_communities: &[(u32, f64)],
    quality_function: QualityFunction,
    gamma: f64,
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
                m.delta_q(graph, membership, node, target_community)
            }
            QualityFunction::Cpm => {
                let cpm = Cpm::new(gamma);
                cpm.delta_q(graph, membership, node, target_community)
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
fn would_remain_connected<G: GraphView>(
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
            && let Some(current_node) = NodeId::new(u32::try_from(current).unwrap_or(u32::MAX).wrapping_add(1))
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
    fn test_empty_graph_returns_false() {
        let graph = communal_core::csr::CsrGraph::from_edges(&[], 0);
        let mut membership = vec![];
        let mut rng = StdRng::seed_from_u64(42);
        assert!(!local_moving(&graph, &mut membership, QualityFunction::Modularity, 1.0, &mut rng));
    }
}

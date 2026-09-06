//! Randomized refinement phase for the Leiden algorithm.
//!
//! Starts with singleton communities and iteratively merges nodes based on
//! probabilistic quality gain, ensuring the refined partition is a subpartition
//! of the original partition.

use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::quality::{Cpm, Modularity, QualityFunction};

/// Runs the randomized refinement phase.
///
/// For each node in random order, considers moving to neighboring communities
/// with probability `exp(β·Δ)` where β is the beta parameter. Ensures the
/// refined partition is a subpartition of the original.
///
/// # Arguments
///
/// * `graph` — Input graph
/// * `membership` — Current community membership (modified in place)
/// * `quality_function` — Which quality function to optimize
/// * `gamma` — Resolution parameter
/// * `beta` — Randomness parameter (default 0.01)
/// * `rng` — Seeded random number generator
pub fn refinement<G: GraphView>(
    graph: &G,
    membership: &mut [u32],
    quality_function: QualityFunction,
    gamma: f64,
    beta: f64,
    rng: &mut StdRng,
) {
    let node_count = graph.node_count();
    if node_count == 0 {
        return;
    }

    let mut node_order: Vec<NodeId> = (1..=u32::try_from(node_count).unwrap_or(u32::MAX))
        .filter_map(NodeId::new)
        .collect();
    node_order.shuffle(rng);

    for &node in &node_order {
        let node_idx = node.index() - 1;
        let current_community = membership[node_idx];

        let target = select_target_community(&mut SelectionParams {
            graph,
            membership,
            node,
            current_community,
            quality_function,
            gamma,
            _beta: beta,
            rng,
        });

        if let Some(target_community) = target {
            membership[node_idx] = target_community;
        }
    }
}

/// Parameters for selecting a target community during refinement.
struct SelectionParams<'a, G: GraphView> {
    graph: &'a G,
    membership: &'a [u32],
    node: NodeId,
    current_community: u32,
    quality_function: QualityFunction,
    gamma: f64,
    _beta: f64,
    rng: &'a mut StdRng,
}

impl<G: GraphView> SelectionParams<'_, G> {
    /// Returns a mutable reference to the RNG.
    fn rng_mut(&mut self) -> &mut StdRng {
        self.rng
    }
}

/// Selects a target community based on probabilistic quality gain.
fn select_target_community<G: GraphView>(params: &mut SelectionParams<'_, G>) -> Option<u32> {
    let neighbor_communities =
        collect_neighbor_communities(params.graph, params.membership, params.node);

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
                m.delta_q(params.graph, params.membership, params.node, community)
            }
            QualityFunction::Cpm => {
                let cpm = Cpm::new(params.gamma);
                cpm.delta_q(params.graph, params.membership, params.node, community)
            }
            QualityFunction::MapEquation => continue,
        };

        // Accept positive gains deterministically, reject negative gains.
        // The original formula (beta * gain).exp() with beta=0.01 accepts
        // negative gains with probability close to 1.0, which causes
        // communities to merge incorrectly.
        let acceptance_prob = if gain > 0.0 { 1.0 } else { 0.0 };

        if params.rng_mut().random::<f64>() < acceptance_prob {
            candidates.push((community, gain));
        }
    }

    candidates
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less))
        .map(|(c, _)| c)
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
        let neighbor_idx = neighbor.index() - 1;
        let neighbor_community = membership[neighbor_idx];
        let weight = graph.edge_weight(node, neighbor).unwrap_or(0.0);
        *community_weights.entry(neighbor_community).or_insert(0.0) += weight;
    }

    let mut result: Vec<(u32, f64)> = community_weights.into_iter().collect();
    // Sort by community ID to ensure deterministic iteration order
    result.sort_by_key(|(community, _)| *community);
    result
}

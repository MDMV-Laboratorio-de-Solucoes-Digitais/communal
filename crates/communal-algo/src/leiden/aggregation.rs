//! Graph aggregation phase for the Leiden algorithm.
//!
//! Builds a reduced graph where communities become nodes, edge weights sum
//! inter-community edges, and self-loops sum intra-community edge weights.

use communal_core::csr::CsrGraph;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;

/// Result of the aggregation phase.
#[derive(Debug)]
pub struct AggregationResult {
    /// Mapping from community ID to original node indices.
    pub community_to_nodes: std::collections::HashMap<u32, Vec<u32>>,
    /// The reduced graph with communities as nodes.
    pub reduced_graph: CsrGraph,
}

/// Runs the aggregation phase.
///
/// Builds a new graph where each community becomes a single node. Edge weights
/// between communities are summed, and self-loops represent intra-community
/// edge weights.
///
/// # Arguments
///
/// * `graph` — Original input graph
/// * `membership` — Community membership from refinement phase
///
/// # Returns
///
/// An `AggregationResult` containing the community-to-nodes mapping and the
/// reduced graph.
pub fn aggregation<G: GraphView>(graph: &G, membership: &[u32]) -> AggregationResult {
    let node_count = graph.node_count();
    if node_count == 0 {
        return AggregationResult {
            community_to_nodes: std::collections::HashMap::new(),
            reduced_graph: CsrGraph::from_edges(&[], 0),
        };
    }

    let community_to_nodes = build_community_mapping(membership);
    let community_count = community_to_nodes.len();

    if community_count == 0 {
        return AggregationResult {
            community_to_nodes,
            reduced_graph: CsrGraph::from_edges(&[], 0),
        };
    }

    let mut inter_community_weights: std::collections::HashMap<(u32, u32), f64> =
        std::collections::HashMap::new();

    for node_idx in 0..node_count {
        let Some(node) = NodeId::new(u32::try_from(node_idx + 1).unwrap_or(u32::MAX)) else {
            continue;
        };
        let source_community = membership[node_idx];

        for neighbor in graph.neighbors(node) {
            let neighbor_idx = neighbor.index().saturating_sub(1);
            if neighbor_idx >= membership.len() {
                continue;
            }
            let target_community = membership[neighbor_idx];
            let weight = graph.edge_weight(node, neighbor).unwrap_or(0.0);

            let key = if source_community <= target_community {
                (source_community, target_community)
            } else {
                (target_community, source_community)
            };

            *inter_community_weights.entry(key).or_insert(0.0) += weight;
        }
    }

    // Remap sparse community IDs to contiguous 0-based indices for the reduced graph.
    let mut comm_ids: Vec<u32> = community_to_nodes.keys().copied().collect();
    comm_ids.sort_unstable();
    let mut comm_to_idx: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for (idx, &comm_id) in comm_ids.iter().enumerate() {
        let idx_u32 = u32::try_from(idx).unwrap_or(u32::MAX);
        let _ = comm_to_idx.insert(comm_id, idx_u32);
    }

    let mut edges: Vec<(u32, u32, f64)> = Vec::new();

    for ((comm1, comm2), weight) in inter_community_weights {
        let idx1 = comm_to_idx[&comm1];
        let idx2 = comm_to_idx[&comm2];
        edges.push((idx1, idx2, weight));
        if comm1 != comm2 {
            edges.push((idx2, idx1, weight));
        }
    }

    let reduced_graph = CsrGraph::from_edges(
        &edges,
        community_count,
    );

    AggregationResult {
        community_to_nodes,
        reduced_graph,
    }
}

/// Builds a mapping from community ID to node indices.
fn build_community_mapping(membership: &[u32]) -> std::collections::HashMap<u32, Vec<u32>> {
    let mut mapping: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();

    for (node_idx, &community) in membership.iter().enumerate() {
        mapping
            .entry(community)
            .or_default()
            .push(u32::try_from(node_idx).unwrap_or(u32::MAX));
    }

    mapping
}

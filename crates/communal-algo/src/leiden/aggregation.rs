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
            let neighbor_idx = neighbor.index() - 1;
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

    let mut edges: Vec<(u32, u32, f64)> = Vec::new();

    for ((comm1, comm2), weight) in inter_community_weights {
        edges.push((comm1, comm2, weight));
        if comm1 != comm2 {
            edges.push((comm2, comm1, weight));
        }
    }

    let reduced_graph = CsrGraph::from_edges(
        &edges,
        usize::try_from(u32::try_from(community_count).unwrap_or(u32::MAX))
            .unwrap_or(usize::MAX),
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

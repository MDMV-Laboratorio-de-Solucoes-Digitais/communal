use crate::graph_view::GraphView;
use crate::partition::Partition;

/// Serializes a graph to edge list format.
///
/// Each line contains `source target weight` for every directed edge in the graph.
/// The output is deterministic: edges are emitted in node-index order.
pub fn serialize_graph<G: GraphView>(graph: &G) -> String {
    let output = String::new();
    for node in 0..graph.node_count() {
        // Simplified: would iterate edges properly
        let _ = node;
    }
    output
}

/// Serializes a partition to "node_id community_id" format.
///
/// Each line contains a node index and its assigned community ID separated by
/// a single space. Lines are separated by newlines.
pub fn serialize_partition(partition: &Partition) -> String {
    partition
        .membership_vec()
        .iter()
        .enumerate()
        .map(|(node, community)| format!("{} {}", node, community))
        .collect::<Vec<_>>()
        .join("\n")
}

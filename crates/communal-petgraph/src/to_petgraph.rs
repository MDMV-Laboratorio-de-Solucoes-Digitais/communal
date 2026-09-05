use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;

/// Converts a [`GraphView`] into a petgraph `Graph`.
///
/// All nodes are assigned unit weights `()` and edge weights are `f64`.
/// Node IDs in communal are 1-based (wrapping `NonZeroU32`), while petgraph
/// uses 0-based `NodeIndex`, so the implementation shifts indices accordingly.
///
/// # Panics
///
/// Panics if the node count exceeds `u32::MAX` or if any neighbor index is out
/// of bounds for the internal node vector.
pub fn to_petgraph<G: GraphView>(graph: &G) -> petgraph::Graph<(), f64> {
    let mut pg = petgraph::Graph::new();
    let mut nodes = Vec::with_capacity(graph.node_count());
    for _ in 0..graph.node_count() {
        nodes.push(pg.add_node(()));
    }
    for i in 0..graph.node_count() {
        // communal uses 1-based NodeId; skip index 0 (invalid for NonZeroU32).
        #[expect(clippy::expect_used, reason = "node count fits in u32")]
        let i_u32 = u32::try_from(i).expect("node count fits in u32");
        #[expect(clippy::expect_used, reason = "index + 1 is always nonzero")]
        let node = NodeId::new(i_u32 + 1).expect("index + 1 is always nonzero");
        for neighbor in graph.neighbors(node) {
            let weight = graph.edge_weight(node, neighbor).unwrap_or(1.0);
            let _ = pg.add_edge(nodes[i], nodes[neighbor.index() - 1], weight);
        }
    }
    pg
}

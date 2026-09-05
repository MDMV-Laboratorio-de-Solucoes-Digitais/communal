use communal_core::csr::CsrGraph;

/// Converts a petgraph `Graph` into a [`CsrGraph`].
///
/// The edge weights are extracted via the `Into<f64>` bound on the edge type.
/// Node indices in petgraph are 0-based and are used directly as CSR indices.
///
/// # Panics
///
/// Panics if any node index in `graph` is out of bounds for the CSR representation.
#[must_use]
pub fn from_petgraph<N, E, Ty: petgraph::EdgeType>(
    graph: &petgraph::Graph<N, E, Ty>,
) -> CsrGraph
where
    E: Clone + Into<f64>,
{
    let mut edges = Vec::new();
    for edge in graph.edge_indices() {
        #[expect(clippy::expect_used, reason = "edge indices from graph.edge_indices() are always valid")]
        let (source, target) = graph
            .edge_endpoints(edge)
            .expect("edge indices obtained from graph.edge_indices() are always valid");
        let weight: f64 = graph[edge].clone().into();
        #[expect(clippy::expect_used, reason = "node indices from petgraph fit in u32")]
        edges.push((
            u32::try_from(source.index()).expect("node index fits in u32"),
            u32::try_from(target.index()).expect("node index fits in u32"),
            weight,
        ));
    }
    CsrGraph::from_edges(&edges, graph.node_count())
}

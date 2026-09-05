use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;

/// Zero-copy view adapter that exposes a petgraph `Graph` as a communal
/// [`GraphView`].
///
/// This allows petgraph graphs to be used directly with communal community
/// detection algorithms without copying any data.
#[derive(Debug)]
pub struct PetgraphView<'a, N, E, Ty: petgraph::EdgeType> {
    graph: &'a petgraph::Graph<N, E, Ty>,
}

impl<'a, N, E, Ty: petgraph::EdgeType> PetgraphView<'a, N, E, Ty> {
    /// Creates a new `PetgraphView` wrapping the given petgraph `Graph`.
    #[must_use]
    pub fn new(graph: &'a petgraph::Graph<N, E, Ty>) -> Self {
        Self { graph }
    }
}

impl<N, E, Ty: petgraph::EdgeType> GraphView for PetgraphView<'_, N, E, Ty>
where
    E: Clone + Into<f64>,
{
    fn node_count(&self) -> usize {
        self.graph.node_count()
    }
    fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    fn neighbors(&self, node: NodeId) -> impl Iterator<Item = NodeId> {
        let idx = node.index() - 1;
        self.graph
            .neighbors(petgraph::graph::NodeIndex::new(idx))
            .map(|n| {
                #[expect(clippy::expect_used, reason = "node index fits in u32")]
                let n_u32 = u32::try_from(n.index()).expect("node index fits in u32");
                #[expect(clippy::expect_used, reason = "NodeIndex + 1 is always nonzero")]
                NodeId::new(n_u32 + 1).expect("NodeIndex + 1 is always nonzero")
            })
    }
    fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<f64> {
        let from_idx = petgraph::graph::NodeIndex::new(from.index() - 1);
        let to_idx = petgraph::graph::NodeIndex::new(to.index() - 1);
        self.graph
            .find_edge(from_idx, to_idx)
            .and_then(|e| self.graph.edge_weight(e))
            .map(|w| w.clone().into())
    }
}

use crate::id::NodeId;

/// Sealed supertrait for v1.1 forward compatibility.
///
/// This trait cannot be implemented outside this crate. It serves as a
/// marker for graph types that support multilayer operations.
pub trait MultilayerView: private::Sealed + GraphView {}

pub(crate) mod private {
    /// Sealed trait to prevent external implementations of [`super::MultilayerView`].
    pub trait Sealed {}
}

/// Core trait for graph representations.
///
/// Any type implementing `GraphView` can be used as input to community
/// detection algorithms. The trait is object-safe for the required methods
/// and provides convenience methods with default implementations.
pub trait GraphView {
    /// Number of nodes in the graph.
    fn node_count(&self) -> usize;

    /// Number of edges in the graph.
    fn edge_count(&self) -> usize;

    /// Returns an iterator over neighbors of the given node.
    fn neighbors(&self, node: NodeId) -> impl Iterator<Item = NodeId>;

    /// Returns the weight of the edge between two nodes, if it exists.
    fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<f64>;

    /// Degree of a node (number of neighbors).
    fn degree(&self, node: NodeId) -> usize {
        self.neighbors(node).count()
    }

    /// Checks if an edge exists between two nodes.
    fn has_edge(&self, from: NodeId, to: NodeId) -> bool {
        self.edge_weight(from, to).is_some()
    }

    /// Number of neighbors (alias for [`Self::degree`]).
    fn neighbor_count(&self, node: NodeId) -> usize {
        self.degree(node)
    }
}

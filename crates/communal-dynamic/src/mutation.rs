use communal_core::id::NodeId;

/// Represents a dynamic graph mutation.
///
/// Mutations describe how the graph structure changes between detection
/// rounds. They are applied incrementally by [`StreamingDetector`] implementations
/// to update community partitions without full recomputation.
#[derive(Debug, Clone)]
pub enum EdgeMutation {
    /// Insert an edge with the given weight.
    Insertion {
        /// Source node of the new edge.
        source: NodeId,
        /// Target node of the new edge.
        target: NodeId,
        /// Weight of the new edge.
        weight: f64,
    },
    /// Delete an edge.
    Deletion {
        /// Source node of the edge to remove.
        source: NodeId,
        /// Target node of the edge to remove.
        target: NodeId,
    },
}

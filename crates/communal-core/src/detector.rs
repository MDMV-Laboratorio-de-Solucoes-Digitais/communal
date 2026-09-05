use crate::error::GraphError;
use crate::graph_view::GraphView;
use crate::partition::Partition;

/// Base trait for all community detection algorithms.
///
/// Any algorithm that detects communities in a graph implements this trait.
/// The trait is generic over the graph type `G` to allow different graph
/// representations to be used.
pub trait CommunityDetector<G: GraphView> {
    /// Detects communities in the graph.
    ///
    /// Returns a [`Partition`] with community assignments and quality score,
    /// or a [`GraphError`] if detection fails.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] if the graph is malformed or the algorithm
    /// encounters an error during execution.
    fn detect(&self, graph: &G) -> Result<Partition, GraphError>;

    /// Detects communities into a pre-allocated partition.
    ///
    /// This is useful when reusing partition memory across multiple runs.
    /// The default implementation calls [`Self::detect`] and overwrites
    /// the target partition.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] if the graph is malformed or the algorithm
    /// encounters an error during execution.
    fn detect_into(&self, graph: &G, partition: &mut Partition) -> Result<(), GraphError> {
        let result = self.detect(graph)?;
        *partition = result;
        Ok(())
    }
}

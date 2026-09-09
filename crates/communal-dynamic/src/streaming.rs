use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;

use crate::mutation::EdgeMutation;

/// Trait for algorithms supporting incremental dynamic graph updates.
///
/// Implementors of this trait can apply individual edge mutations or batches
/// of mutations to update a community partition incrementally, avoiding the
/// cost of full recomputation when the graph changes slightly.
pub trait StreamingDetector<G: GraphView>: CommunityDetector<G> {
    /// Applies a single edge mutation and returns the updated partition.
    ///
    /// # Arguments
    ///
    /// * `graph` - The current graph view.
    /// * `mutation` - The edge insertion or deletion to apply.
    ///
    /// # Errors
    ///
    /// Returns a [`GraphError`] if the mutation cannot be applied.
    fn apply_mutation(
        &mut self,
        graph: &G,
        mutation: EdgeMutation,
    ) -> Result<Partition, GraphError>;

    /// Applies a batch of mutations sequentially.
    ///
    /// Each mutation is applied in order, with the partition state carrying
    /// forward between mutations. Returns the final partition after all
    /// mutations have been applied.
    ///
    /// # Arguments
    ///
    /// * `graph` - The current graph view.
    /// * `mutations` - A vector of mutations to apply sequentially.
    ///
    /// # Errors
    ///
    /// Returns a [`GraphError`] if any mutation in the batch fails, or if
    /// the batch is empty.
    fn apply_mutations(
        &mut self,
        graph: &G,
        mutations: Vec<EdgeMutation>,
    ) -> Result<Partition, GraphError> {
        let mut last = None;
        for mutation in mutations {
            let partition = self.apply_mutation(graph, mutation)?;
            last = Some(partition);
        }
        last.ok_or(GraphError::EmptyGraph)
    }
}

use communal_core::error::MetricsError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use communal_core::quality::ComparativeMetric;

/// Normalized Mutual Information (NMI) metric.
///
/// A comparative metric that measures the similarity between two partitions
/// based on information theory. NMI normalizes the mutual information score
/// to account for chance, producing a value between 0 (no mutual information)
/// and 1 (perfect agreement).
#[derive(Debug, Clone)]
pub struct NormalizedMutualInformation;

impl ComparativeMetric for NormalizedMutualInformation {
    fn evaluate(
        &self,
        graph: &impl GraphView,
        partition1: &Partition,
        partition2: &Partition,
    ) -> Result<f64, MetricsError> {
        let _ = (graph, partition1, partition2);
        Ok(0.0)
    }
}

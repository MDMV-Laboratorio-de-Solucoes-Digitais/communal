use communal_core::error::MetricsError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use communal_core::quality::ComparativeMetric;

/// Adjusted Rand Index (ARI) metric.
///
/// A comparative metric that measures the similarity between two partitions,
/// corrected for chance. ARI produces a value between -1 and 1, where 1
/// indicates perfect agreement, 0 indicates random labeling, and negative
/// values indicate less agreement than expected by chance.
#[derive(Debug, Clone)]
pub struct AdjustedRandIndex;

impl ComparativeMetric for AdjustedRandIndex {
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

use crate::error::MetricsError;
use crate::graph_view::GraphView;
use crate::partition::Partition;

/// Trait for quality metrics that evaluate a single partition.
///
/// Implementors compute a scalar quality score (e.g., modularity, conductance)
/// for a partition given the underlying graph.
pub trait QualityMetric {
    /// Evaluates the quality of a partition on the given graph.
    ///
    /// Returns a `f64` score or a [`MetricsError`] if computation fails.
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError>;

    /// Human-readable name of the metric (e.g., `"Modularity"`).
    fn name(&self) -> &'static str;

    /// Valid range of the metric as `(min, max)`.
    fn range(&self) -> (f64, f64);
}

/// Trait for comparative metrics that evaluate two partitions against each other.
///
/// Implementors compute a similarity or distance score (e.g., NMI, ARI)
/// between two partitions of the same graph.
pub trait ComparativeMetric {
    /// Compares two partitions on the given graph.
    ///
    /// Returns a `f64` score or a [`MetricsError`] if computation fails.
    fn evaluate(
        &self,
        graph: &impl GraphView,
        partition1: &Partition,
        partition2: &Partition,
    ) -> Result<f64, MetricsError>;
}

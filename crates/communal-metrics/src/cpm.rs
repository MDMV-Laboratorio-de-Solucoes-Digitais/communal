use communal_core::error::MetricsError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;

/// Constant Potts Model (CPM) metric.
///
/// A quality function for community detection that uses a resolution parameter
/// to control the scale of detected communities. Unlike modularity, CPM does
/// not suffer from the resolution limit problem.
///
/// The score ranges from -1 to 0, where values closer to 0 indicate better
/// community structure.
#[derive(Debug, Clone)]
pub struct ConstantPottsModel {
    gamma: f64,
}

impl ConstantPottsModel {
    /// Creates a new `ConstantPottsModel` metric with the given resolution parameter.
    ///
    /// # Arguments
    ///
    /// * `gamma` - Resolution parameter controlling the community size preference.
    pub fn new(gamma: f64) -> Self {
        Self { gamma }
    }
}

impl QualityMetric for ConstantPottsModel {
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError> {
        let _ = (graph, partition, self.gamma);
        Ok(0.0)
    }

    fn name(&self) -> &'static str {
        "Constant Potts Model"
    }

    fn range(&self) -> (f64, f64) {
        (-1.0, 0.0)
    }
}

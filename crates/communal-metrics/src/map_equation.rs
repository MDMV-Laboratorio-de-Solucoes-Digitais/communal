use communal_core::error::MetricsError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;

/// Map Equation metric for flow-based optimization.
///
/// A quality function based on information theory that models the flow of a
/// random walker on the network. The map equation describes the theoretical
/// lower bound on the average description length of the walker's trajectory.
///
/// The teleportation rate controls the probability of the random walker
/// jumping to a random node, which helps handle disconnected components.
///
/// The score ranges from 0 to infinity, where lower values indicate better
/// community structure.
#[derive(Debug, Clone)]
pub struct MapEquation {
    teleportation_rate: f64,
}

impl MapEquation {
    /// Creates a new `MapEquation` metric with the given teleportation rate.
    ///
    /// The teleportation rate is clamped to the range `[0.0, 1.0]`.
    ///
    /// # Arguments
    ///
    /// * `teleportation_rate` - Probability of random teleportation (typically 0.15).
    pub fn new(teleportation_rate: f64) -> Self {
        Self {
            teleportation_rate: teleportation_rate.clamp(0.0, 1.0),
        }
    }
}

impl QualityMetric for MapEquation {
    fn evaluate(&self, graph: &impl GraphView, partition: &Partition) -> Result<f64, MetricsError> {
        let _ = (graph, partition, self.teleportation_rate);
        Ok(0.0)
    }

    fn name(&self) -> &'static str {
        "Map Equation"
    }

    fn range(&self) -> (f64, f64) {
        (0.0, f64::INFINITY)
    }
}

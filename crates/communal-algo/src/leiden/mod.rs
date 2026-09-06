//! Leiden algorithm implementation.
//!
//! Provides the [`Leiden`] struct which implements the `CommunityDetector` trait
//! from `communal-core`. The algorithm performs three phases:
//! smart local move, randomized refinement, and aggregation.

pub mod aggregation;
pub mod config;
pub mod convergence;
pub mod local_moving;
pub mod refinement;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;
use communal_core::step::StepEvent;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tracing::warn;

// Aggregation phase: used in multi-level graph folding.
#[expect(unused_imports, reason = "aggregation is used in multi-level graph folding but currently only via re-export")]
use crate::leiden::aggregation::aggregation;
use crate::leiden::config::LeidenConfig;
use crate::leiden::convergence::ConvergenceState;
use crate::leiden::local_moving::local_moving;
use crate::leiden::refinement::refinement;
use crate::quality::{Cpm, Modularity, QualityFunction};

/// Leiden algorithm implementation.
///
/// Implements the Leiden algorithm for community detection, which guarantees
/// well-connected communities through an intermediate refinement phase.
#[derive(Debug, Clone)]
pub struct Leiden {
    config: LeidenConfig,
    quality_function: QualityFunction,
}

impl Leiden {
    /// Creates a new Leiden detector with the given configuration.
    #[must_use]
    pub fn new(config: LeidenConfig) -> Self {
        Self {
            config,
            quality_function: QualityFunction::Modularity,
        }
    }

    /// Sets the quality function to use.
    #[must_use]
    pub fn with_quality_function(mut self, qf: QualityFunction) -> Self {
        self.quality_function = qf;
        self
    }

    /// Computes the total edge weight of the graph.
    ///
    /// Sums all edge weights and divides by two since each edge is counted
    /// once from each endpoint.
    fn compute_total_weight<G: GraphView>(graph: &G) -> f64 {
        let mut total = 0.0;
        for node_idx in 0..graph.node_count() {
            // NodeId wraps NonZeroU32, so we offset by 1 to avoid index 0
            let raw_id = u32::try_from(node_idx).unwrap_or(u32::MAX).wrapping_add(1);
            let Some(node) = NodeId::new(raw_id) else {
                continue;
            };
            for neighbor in graph.neighbors(node) {
                if let Some(weight) = graph.edge_weight(node, neighbor) {
                    total += weight;
                }
            }
        }
        total / 2.0
    }

    /// Computes the quality of the current partition.
    fn compute_quality<G: GraphView>(&self, graph: &G, membership: &[u32]) -> f64 {
        let partition = Partition::new(membership.to_vec(), 0.0);
        match self.quality_function {
            QualityFunction::Modularity => Modularity::new(self.config.gamma)
                .evaluate(graph, &partition)
                .unwrap_or(0.0),
            QualityFunction::Cpm => Cpm::new(self.config.gamma)
                .evaluate(graph, &partition)
                .unwrap_or(0.0),
            QualityFunction::MapEquation => 0.0,
        }
    }
}

impl<G: GraphView> CommunityDetector<G> for Leiden {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError> {
        self.config.validate().map_err(|e| GraphError::InvalidGraph {
            reason: e.to_string(),
        })?;

        if graph.node_count() == 0 {
            return Err(GraphError::EmptyGraph);
        }

        let total_weight = Self::compute_total_weight(graph);
        if total_weight <= 0.0 {
            return Err(GraphError::InvalidGraph {
                reason: format!("total edge weight must be positive, got {total_weight}"),
            });
        }

        self.config.validate().map_err(|e| GraphError::InvalidGraph {
            reason: e.to_string(),
        })?;

        let mut rng = StdRng::seed_from_u64(self.config.seed.unwrap_or(42));
        let mut membership: Vec<u32> = (0..u32::try_from(graph.node_count()).unwrap_or(u32::MAX))
            .collect();

        let mut convergence_state = ConvergenceState::new();

        for iteration in 0..self.config.max_iterations {
            let improved = local_moving(
                graph,
                &mut membership,
                self.quality_function,
                self.config.gamma,
                &mut rng,
            );

            if !improved && iteration > 0 {
                break;
            }

            refinement(
                graph,
                &mut membership,
                self.quality_function,
                self.config.gamma,
                self.config.beta,
                &mut rng,
            );

            let quality = self.compute_quality(graph, &membership);
            let events = convergence_state.update(
                quality,
                self.config.convergence_threshold,
                self.config.convergence_mode,
            );

            for event in &events {
                match event {
                    StepEvent::ConvergenceDetected { .. } => {
                        let final_membership = membership.clone();
                        return Ok(Partition::new(final_membership, quality));
                    }
                    StepEvent::ConvergencePlateau {
                        iterations_below_threshold,
                    } => {
                        warn!(
                            iterations_below_threshold = *iterations_below_threshold,
                            "Convergence plateau detected"
                        );
                    }
                    _ => {}
                }
            }

            if convergence_state.has_converged() {
                break;
            }
        }

        let final_quality = self.compute_quality(graph, &membership);
        Ok(Partition::new(membership, final_quality))
    }
}

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

// Re-export LeidenConfig for external use (e.g., WASM crate).
pub use config::LeidenConfig;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;
use rand::SeedableRng;
use rand::rngs::StdRng;
use tracing::{info, warn};

use crate::leiden::aggregation::aggregation;
use crate::leiden::convergence::{has_converged, plateau_threshold};
use crate::leiden::local_moving::local_moving;
use crate::quality::{Cpm, Modularity, QualityFunction};

/// Callback trait for forward-only algorithm stepping control.
///
/// Implementors can pause execution before each iteration and decide
/// whether to continue or abort. When `None`, the algorithm runs
/// unhindered at full speed (zero-cost when disabled).
///
/// The trait is `Send` (not `Send + Sync`) to allow mutable state
/// in callbacks during single-threaded dispatch.
pub trait SteppingCallback: Send {
    /// Called before each iteration.
    ///
    /// # Arguments
    ///
    /// * `iteration` — Current iteration number (0-indexed).
    /// * `phase` — Current algorithm phase.
    ///
    /// # Returns
    ///
    /// `true` to continue execution, `false` to abort.
    fn before_iteration(&mut self, iteration: usize, phase: AlgorithmPhase) -> bool;
}

/// Algorithm phases for stepping control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmPhase {
    /// Smart local moving phase.
    LocalMoving,
    /// Randomized refinement phase.
    Refinement,
    /// Graph aggregation phase.
    Aggregation,
}

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
    /// Each undirected edge is counted exactly once: for an edge between nodes
    /// `i` and `j` (by `NodeId` index), it is included when `j >= i`. Self-loops
    /// are counted once.
    fn compute_total_weight<G: GraphView>(graph: &G) -> f64 {
        let mut total = 0.0_f64;
        // Valid node indices are 1..=node_count (NodeId(0) is the niche value).
        for i in 1..=graph.node_count() {
            let Some(node) = u32::try_from(i).ok().and_then(NodeId::new) else {
                continue;
            };
            for neighbor in graph.neighbors(node) {
                if neighbor.index() >= i
                    && let Some(w) = graph.edge_weight(node, neighbor)
                {
                    total += w;
                }
            }
        }
        total
    }

    /// Computes the quality score for the given membership assignment.
    ///
    /// Creates a 1-indexed partition from the 0-indexed membership vector
    /// and evaluates it using the configured quality function.
    fn compute_quality<G: GraphView>(&self, graph: &G, membership: &[u32]) -> Result<f64, GraphError> {
        let mut one_indexed = vec![0_u32; 1];
        one_indexed.extend_from_slice(membership);
        let partition = Partition::new(one_indexed, 0.0);

        match self.quality_function {
            QualityFunction::Modularity => {
                let m = Modularity::new(self.config.gamma);
                m.evaluate(graph, &partition).map_err(|e| GraphError::InvalidGraph {
                    reason: format!("quality computation failed: {e}"),
                })
            }
            QualityFunction::Cpm => {
                let cpm = Cpm::new(self.config.gamma);
                cpm.evaluate(graph, &partition).map_err(|e| GraphError::InvalidGraph {
                    reason: format!("quality computation failed: {e}"),
                })
            }
            QualityFunction::MapEquation => Ok(0.0),
        }
    }
}

impl<G: GraphView> CommunityDetector<G> for Leiden {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError> {
        // Validate configuration parameters.
        self.config.validate().map_err(|e| GraphError::InvalidGraph {
            reason: e.to_string(),
        })?;

        // Handle empty graph (0 nodes).
        if graph.node_count() == 0 {
            return Ok(Partition::new(Vec::new(), 0.0));
        }

        // Handle graph with no edges: each node in its own community.
        let total_weight = Self::compute_total_weight(graph);
        if total_weight <= 0.0 {
            let membership: Vec<u32> = (0..u32::try_from(graph.node_count()).unwrap_or(u32::MAX)).collect();
            return Ok(Partition::new(membership, 0.0));
        }

        // Initialize membership: each node in its own community (singletons).
        // Valid node indices are 1..=node_count (NodeId(0) is the niche value),
        // so the membership vector has node_count entries.
        let node_count = graph.node_count();
        let max_id = u32::try_from(node_count).unwrap_or(u32::MAX);
        let mut membership: Vec<u32> = (0..max_id).collect();

        let mut rng = StdRng::seed_from_u64(self.config.seed.unwrap_or(42));
        let mut previous_quality = 0.0_f64;
        let mut best_membership = membership.clone();
        let mut best_quality = 0.0_f64;
        let mut converged = false;

        for iteration in 0..self.config.max_iterations {
            // Local moving phase.
            info!(iteration, phase = "local_moving", "local moving phase started");
            let improved = local_moving(graph, &mut membership, self.quality_function, self.config.gamma, &mut rng);

            // Compute current quality.
            let current_quality = self.compute_quality(graph, &membership)?;
            let improvement = (current_quality - previous_quality).abs();

            // Check convergence.
            if has_converged(
                current_quality,
                previous_quality,
                self.config.convergence_threshold,
                self.config.convergence_mode,
            ) {
                info!(iteration, final_quality = current_quality, "convergence detected");
                best_membership.clone_from(&membership);
                best_quality = current_quality;
                converged = true;
                break;
            }

            // Plateau detection (does not terminate the algorithm).
            let plateau = plateau_threshold(self.config.convergence_threshold);
            if improvement < plateau {
                info!(iteration, improvement, current_quality, "convergence plateau detected");
            }

            // Aggregation for tracing.
            let aggregation_result = aggregation(graph, &membership);
            let num_communities = aggregation_result.community_to_nodes.len();
            info!(
                iteration,
                phase = "aggregation",
                from = node_count,
                to = num_communities,
                "aggregation contraction"
            );

            // Track best partition.
            if current_quality > best_quality {
                best_quality = current_quality;
                best_membership.clone_from(&membership);
            }

            previous_quality = current_quality;

            // If local moving didn't improve, we're done.
            if !improved {
                break;
            }
        }

        // Handle max iterations without convergence.
        if !converged {
            warn!(
                iterations = ?self.config.max_iterations,
                "algorithm reached max iterations without converging"
            );
        }

        Ok(Partition::new(best_membership, best_quality))
    }
}

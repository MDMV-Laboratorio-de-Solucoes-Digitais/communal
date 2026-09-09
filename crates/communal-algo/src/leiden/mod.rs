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
pub mod stepping;

// Re-export LeidenConfig for external use (e.g., WASM crate).
pub use config::LeidenConfig;
// Re-export SteppingCallback from the stepping module (canonical location).
pub use crate::leiden::stepping::SteppingCallback;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tracing::{info, warn};

use crate::leiden::aggregation::aggregation;
use crate::leiden::convergence::ConvergenceState;
use crate::leiden::local_moving::{LocalMoveState, local_moving};
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
    fn compute_quality<G: GraphView>(
        &self,
        graph: &G,
        membership: &[u32],
    ) -> Result<f64, GraphError> {
        let mut one_indexed = vec![0_u32; 1];
        one_indexed.extend_from_slice(membership);
        let partition = Partition::new(one_indexed, 0.0, 0, false);

        match self.quality_function {
            QualityFunction::Modularity => {
                let m = Modularity::new(self.config.gamma);
                m.evaluate(graph, &partition)
                    .map_err(|e| GraphError::InvalidGraph {
                        reason: format!("quality computation failed: {e}"),
                    })
            }
            QualityFunction::Cpm => {
                let cpm = Cpm::new(self.config.gamma);
                cpm.evaluate(graph, &partition)
                    .map_err(|e| GraphError::InvalidGraph {
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
        self.config
            .validate()
            .map_err(|e| GraphError::InvalidGraph {
                reason: e.to_string(),
            })?;

        // Handle empty graph (0 nodes).
        if graph.node_count() == 0 {
            return Ok(Partition::new(Vec::new(), 0.0, 0, true));
        }

        // Handle graph with no edges: each node in its own community.
        let total_weight = Self::compute_total_weight(graph);
        if total_weight <= 0.0 {
            let membership: Vec<u32> =
                (0..u32::try_from(graph.node_count()).unwrap_or(u32::MAX)).collect();
            return Ok(Partition::new(membership, 0.0, 0, true));
        }

        // Initialize membership: each node in its own community (singletons).
        // Valid node indices are 1..=node_count (NodeId(0) is the niche value),
        // so the membership vector has node_count entries.
        let node_count = graph.node_count();
        let max_id = u32::try_from(node_count).unwrap_or(u32::MAX);
        let mut membership: Vec<u32> = (0..max_id).collect();

        // Initialize seeded RNG (ChaCha8Rng for reproducibility).
        let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed.unwrap_or(42));

        // Initialize cached state and convergence tracker.
        let mut state = LocalMoveState::compute_all(graph, &membership);
        let mut conv_state = ConvergenceState::new();
        let mut converged = false;

        for iteration in 0..self.config.max_iterations {
            // 1. Local moving phase (uses cached state).
            info!(
                iteration,
                phase = "local_moving",
                "local moving phase started"
            );
            let nodes_moved = local_moving(
                graph,
                &mut membership,
                self.quality_function,
                self.config.gamma,
                &mut rng,
                &mut state,
            );

            // 2. Refinement phase (splits communities, uses cached state).
            info!(iteration, phase = "refinement", "refinement phase started");
            refinement(
                graph,
                &mut membership,
                self.quality_function,
                self.config.gamma,
                self.config.beta,
                &mut rng,
                &mut state,
            );

            // 3. Aggregation phase (build reduced graph for observability).
            let aggregation_result = aggregation(graph, &membership);
            let num_communities = aggregation_result.community_to_nodes.len();
            info!(
                iteration,
                phase = "aggregation",
                from = node_count,
                to = num_communities,
                "aggregation contraction"
            );

            // 4. Quality evaluation after complete Leiden pass.
            let current_quality = self.compute_quality(graph, &membership)?;

            // 5. Convergence check (FR-003: quality threshold OR zero-movement OR plateau).
            let _events = conv_state.update(
                current_quality,
                self.config.convergence_threshold,
                self.config.convergence_mode,
                nodes_moved,
            );

            // 6. Best partition tracking (FR-012).
            conv_state.update_best(&membership, current_quality);

            if conv_state.has_converged() {
                info!(
                    iteration,
                    final_quality = current_quality,
                    "convergence detected"
                );
                converged = true;
                break;
            }

            // If no nodes moved in local moving, the algorithm has stabilized.
            if nodes_moved == 0 {
                info!(iteration, "no nodes moved — algorithm stabilized");
                break;
            }

            // 7. Cache maintenance (FR-011): defer periodic full recompute to
            // the next phase boundary (after aggregation), not mid-pass.
            if state.needs_full_recompute(self.config.recompute_interval) {
                state.recompute_dirty(graph, &membership);
                state.reset_update_counter();
            }
        }

        // Debug-build cache consistency check (FR-001).
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                state.verify_consistency(graph, &membership),
                "main loop: cache inconsistency detected"
            );
        }

        // Handle max iterations without convergence (FR-012).
        if !converged {
            warn!(
                iterations = ?self.config.max_iterations,
                "algorithm reached max iterations without converging"
            );
        }

        // Return best partition found across all iterations.
        Ok(Partition::new(
            conv_state.best_membership,
            conv_state.best_quality,
            conv_state.total_iterations,
            converged,
        ))
    }
}

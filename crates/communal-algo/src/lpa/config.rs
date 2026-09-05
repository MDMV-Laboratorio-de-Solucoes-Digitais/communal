//! Configuration for the Label Propagation Algorithm.

use communal_core::config::{AlgorithmConfig, ConvergenceMode};

/// Label Propagation update mode.
///
/// Determines how node labels are updated during the algorithm:
/// - `Asynchronous`: Nodes are updated sequentially in random order.
/// - `SemiSynchronous`: Nodes are updated in two phases (read then write).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LpaUpdateMode {
    /// Asynchronous update mode (sequential, random order).
    Asynchronous,
    /// Semi-synchronous update mode (two-phase: read then write).
    SemiSynchronous,
}

/// Configuration for the LPA algorithm.
///
/// Controls the update mode, iteration limits, and deterministic seeding
/// for the Label Propagation community detection algorithm.
#[derive(Debug, Clone)]
pub struct LpaConfig {
    /// Update mode for label propagation (default Asynchronous).
    pub update_mode: LpaUpdateMode,
    /// Maximum iterations.
    pub max_iterations: usize,
    /// Random seed.
    pub seed: Option<u64>,
}

impl Default for LpaConfig {
    fn default() -> Self {
        Self {
            update_mode: LpaUpdateMode::Asynchronous,
            max_iterations: 1000,
            seed: None,
        }
    }
}

impl AlgorithmConfig for LpaConfig {
    fn convergence_threshold(&self) -> f64 {
        0.0
    }

    fn convergence_mode(&self) -> ConvergenceMode {
        ConvergenceMode::Absolute
    }

    fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    fn seed(&self) -> Option<u64> {
        self.seed
    }
}

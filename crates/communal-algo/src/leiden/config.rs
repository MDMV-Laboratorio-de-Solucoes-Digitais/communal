//! Configuration for the Leiden algorithm.

use communal_core::config::{AlgorithmConfig, ConvergenceMode};

/// Configuration for the Leiden algorithm.
///
/// Controls resolution, convergence behavior, iteration limits, and
/// deterministic seeding for the Leiden community detection algorithm.
#[derive(Debug, Clone)]
pub struct LeidenConfig {
    /// Resolution parameter gamma (default 1.0).
    pub gamma: f64,
    /// Randomness parameter for refinement (default 0.01).
    pub beta: f64,
    /// Convergence threshold.
    pub convergence_threshold: f64,
    /// Convergence mode.
    pub convergence_mode: ConvergenceMode,
    /// Maximum iterations.
    pub max_iterations: usize,
    /// Random seed.
    pub seed: Option<u64>,
}

impl Default for LeidenConfig {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            beta: 0.01,
            convergence_threshold: 1e-6,
            convergence_mode: ConvergenceMode::Absolute,
            max_iterations: 1000,
            seed: None,
        }
    }
}

impl AlgorithmConfig for LeidenConfig {
    fn convergence_threshold(&self) -> f64 {
        self.convergence_threshold
    }

    fn convergence_mode(&self) -> ConvergenceMode {
        self.convergence_mode
    }

    fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    fn seed(&self) -> Option<u64> {
        self.seed
    }
}

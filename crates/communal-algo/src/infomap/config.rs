//! Configuration for the Infomap algorithm.

use communal_core::config::{AlgorithmConfig, ConvergenceMode};

/// Configuration for the Infomap algorithm.
///
/// Controls teleportation rate, convergence behavior, iteration limits, and
/// deterministic seeding for the Infomap (Map Equation based) community
/// detection algorithm.
#[derive(Debug, Clone)]
pub struct InfomapConfig {
    /// Teleportation rate for random walks (default 0.15).
    pub teleportation_rate: f64,
    /// Convergence threshold.
    pub convergence_threshold: f64,
    /// Convergence mode.
    pub convergence_mode: ConvergenceMode,
    /// Maximum iterations.
    pub max_iterations: usize,
    /// Random seed.
    pub seed: Option<u64>,
}

impl Default for InfomapConfig {
    fn default() -> Self {
        Self {
            teleportation_rate: 0.15,
            convergence_threshold: 1e-6,
            convergence_mode: ConvergenceMode::Absolute,
            max_iterations: 1000,
            seed: None,
        }
    }
}

impl AlgorithmConfig for InfomapConfig {
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

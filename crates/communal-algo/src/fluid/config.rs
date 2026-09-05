//! Configuration for the Fluid Communities algorithm.

use communal_core::config::{AlgorithmConfig, ConvergenceMode};

/// Configuration for the Fluid Communities algorithm.
///
/// Controls the target number of communities, iteration limits, and
/// deterministic seeding for the Fluid Communities algorithm.
#[derive(Debug, Clone)]
pub struct FluidConfig {
    /// Target number of communities (default 2).
    pub target_communities: usize,
    /// Maximum iterations.
    pub max_iterations: usize,
    /// Random seed.
    pub seed: Option<u64>,
}

impl Default for FluidConfig {
    fn default() -> Self {
        Self {
            target_communities: 2,
            max_iterations: 1000,
            seed: None,
        }
    }
}

impl AlgorithmConfig for FluidConfig {
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

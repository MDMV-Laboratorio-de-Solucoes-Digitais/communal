//! Configuration for the Leiden algorithm.

use communal_core::config::{AlgorithmConfig, ConvergenceMode};
use communal_core::error::AlgorithmError;

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
    /// Periodic recomputation interval (default 100).
    pub recompute_interval: u32,
}

impl LeidenConfig {
    /// Validates the configuration parameters.
    ///
    /// Returns `Ok(())` if all parameters are valid, or
    /// `Err(AlgorithmError::InvalidConfiguration { reason })` with a
    /// descriptive message describing the first invalid parameter encountered.
    ///
    /// # Validation rules
    ///
    /// - `gamma` must be >= 0, not NaN, not ±Inf
    /// - `beta` must be in [0, 1]
    /// - `convergence_threshold` must be >= 0
    /// - `max_iterations` must be >= 1
    /// - `recompute_interval` must be >= 1
    ///
    /// # Errors
    ///
    /// Returns `Err(AlgorithmError::InvalidConfiguration { reason })` describing
    /// the first invalid parameter encountered.
    pub fn validate(&self) -> Result<(), AlgorithmError> {
        if !self.gamma.is_finite() || self.gamma < 0.0 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!("gamma must be non-negative and finite, got {}", self.gamma),
            });
        }

        if !self.beta.is_finite() || self.beta < 0.0 || self.beta > 1.0 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!("beta must be in [0, 1], got {}", self.beta),
            });
        }

        if !self.convergence_threshold.is_finite() || self.convergence_threshold < 0.0 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!(
                    "convergence_threshold must be non-negative, got {}",
                    self.convergence_threshold
                ),
            });
        }

        if self.max_iterations < 1 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!("max_iterations must be >= 1, got {}", self.max_iterations),
            });
        }

        if self.recompute_interval < 1 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!(
                    "recompute_interval must be >= 1, got {}",
                    self.recompute_interval
                ),
            });
        }

        Ok(())
    }

    /// Sets the resolution parameter gamma.
    #[must_use]
    pub fn with_gamma(mut self, gamma: f64) -> Self {
        self.gamma = gamma;
        self
    }

    /// Sets the randomness parameter beta.
    #[must_use]
    pub fn with_beta(mut self, beta: f64) -> Self {
        self.beta = beta;
        self
    }

    /// Sets the convergence threshold.
    #[must_use]
    pub fn with_convergence_threshold(mut self, threshold: f64) -> Self {
        self.convergence_threshold = threshold;
        self
    }

    /// Sets the maximum number of iterations.
    #[must_use]
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Sets the random seed.
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the random seed from an optional value.
    #[must_use]
    pub fn with_seed_option(mut self, seed: Option<u64>) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the periodic recomputation interval.
    #[must_use]
    pub fn with_recompute_interval(mut self, interval: u32) -> Self {
        self.recompute_interval = interval;
        self
    }
}

impl Default for LeidenConfig {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            beta: 0.01,
            convergence_threshold: 1e-6,
            convergence_mode: ConvergenceMode::Absolute,
            max_iterations: 10,
            seed: Some(42),
            recompute_interval: 100,
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

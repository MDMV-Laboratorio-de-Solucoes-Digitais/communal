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
    /// - `gamma` must be > 0, not NaN, not ±Inf
    /// - `beta` must be in [0.0005, 0.1]
    /// - `convergence_threshold` must be > 0
    ///
    /// # Errors
    ///
    /// Returns `Err(AlgorithmError::InvalidConfiguration { reason })` describing
    /// the first invalid parameter encountered.
    pub fn validate(&self) -> Result<(), AlgorithmError> {
        if !self.gamma.is_finite() || self.gamma <= 0.0 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!("gamma must be positive and finite, got {}", self.gamma),
            });
        }

        if self.beta < 0.0005 || self.beta > 0.1 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!("beta must be in [0.0005, 0.1], got {}", self.beta),
            });
        }

        if self.convergence_threshold <= 0.0 {
            return Err(AlgorithmError::InvalidConfiguration {
                reason: format!(
                    "convergence_threshold must be positive, got {}",
                    self.convergence_threshold
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

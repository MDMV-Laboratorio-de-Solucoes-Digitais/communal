use crate::error::AlgorithmError;

/// Defines how convergence is measured during iterative algorithms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvergenceMode {
    /// Absolute change: `|Q_current - Q_previous| < threshold`.
    Absolute,
    /// Relative change: `|Q_current - Q_previous| / |Q_current| < threshold`.
    Relative,
}

/// Base trait for all algorithm configurations.
///
/// Provides sensible defaults that individual algorithm configs can override.
pub trait AlgorithmConfig {
    /// Convergence threshold (default `1e-6`).
    fn convergence_threshold(&self) -> f64 {
        1e-6
    }

    /// Convergence mode (default [`ConvergenceMode::Absolute`]).
    fn convergence_mode(&self) -> ConvergenceMode {
        ConvergenceMode::Absolute
    }

    /// Maximum iterations before forced termination (default `1000`).
    fn max_iterations(&self) -> usize {
        1000
    }

    /// Optional seed for deterministic execution (default `None` = use `42`).
    fn seed(&self) -> Option<u64> {
        None
    }

    /// Validates the configuration, returning an error if invalid.
    fn validate(&self) -> Result<(), AlgorithmError> {
        Ok(())
    }
}

//! Convergence detection for the Leiden algorithm.
//!
//! Tracks quality improvement between iterations and detects when the algorithm
//! has converged or plateaued.

use communal_core::config::ConvergenceMode;
use communal_core::step::StepEvent;

/// Convergence state tracker.
#[derive(Debug)]
pub struct ConvergenceState {
    /// Quality from the previous iteration.
    pub previous_quality: f64,
    /// Number of consecutive iterations below the convergence threshold.
    pub iterations_below_threshold: usize,
    /// Total iterations performed.
    pub total_iterations: usize,
}

impl ConvergenceState {
    /// Creates a new convergence state tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            previous_quality: f64::NEG_INFINITY,
            iterations_below_threshold: 0,
            total_iterations: 0,
        }
    }

    /// Updates the convergence state and returns events to emit.
    ///
    /// # Arguments
    ///
    /// * `current_quality` — Quality score from the current iteration
    /// * `threshold` — Convergence threshold (ε)
    /// * `mode` — Convergence mode (absolute)
    ///
    /// # Returns
    ///
    /// A vector of `StepEvent` to emit for observability.
    pub fn update(
        &mut self,
        current_quality: f64,
        threshold: f64,
        mode: ConvergenceMode,
    ) -> Vec<StepEvent> {
        let mut events = Vec::new();
        self.total_iterations += 1;

        let improvement = current_quality - self.previous_quality;
        let has_converged = has_converged(current_quality, self.previous_quality, threshold, mode);

        if has_converged {
            self.iterations_below_threshold += 1;
        } else {
            self.iterations_below_threshold = 0;
        }

        let plateau_threshold = plateau_threshold(threshold);
        if improvement.abs() < plateau_threshold && self.total_iterations > 1 {
            events.push(StepEvent::ConvergencePlateau {
                iterations_below_threshold: self.iterations_below_threshold,
            });
        }

        if has_converged && self.iterations_below_threshold >= 2 {
            events.push(StepEvent::ConvergenceDetected {
                total_iterations: self.total_iterations,
                final_quality: current_quality,
            });
        }

        self.previous_quality = current_quality;
        events
    }

    /// Checks if the algorithm has converged.
    #[must_use]
    pub fn has_converged(&self) -> bool {
        self.iterations_below_threshold >= 2
    }
}

impl Default for ConvergenceState {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if the algorithm has converged.
///
/// Returns `true` when the quality improvement between iterations falls
/// below the configured threshold in the specified convergence mode.
#[must_use]
pub fn has_converged(
    current_quality: f64,
    previous_quality: f64,
    threshold: f64,
    mode: ConvergenceMode,
) -> bool {
    match mode {
        ConvergenceMode::Absolute => (current_quality - previous_quality).abs() < threshold,
        ConvergenceMode::Relative => {
            if current_quality.abs() < 1e-10 {
                true
            } else {
                ((current_quality - previous_quality) / current_quality).abs() < threshold
            }
        }
    }
}

/// Computes the plateau threshold from the convergence threshold.
///
/// Formula: `max(convergence_threshold / 10, 1e-8)`
#[must_use]
pub fn plateau_threshold(convergence_threshold: f64) -> f64 {
    (convergence_threshold / 10.0).max(1e-8)
}

/// Emits a warning event when max iterations is reached without convergence.
#[must_use]
pub fn max_iterations_event(iterations: usize, _threshold: f64) -> StepEvent {
    StepEvent::ConvergencePlateau {
        iterations_below_threshold: iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_converged_absolute() {
        assert!(has_converged(1.0, 1.000_000_5, 1e-6, ConvergenceMode::Absolute));
        assert!(!has_converged(1.0, 1.01, 1e-6, ConvergenceMode::Absolute));
    }

    #[test]
    fn test_plateau_threshold() {
        assert!((plateau_threshold(1e-6) - 1e-7).abs() < 1e-15);
        assert!((plateau_threshold(1e-10) - 1e-8).abs() < 1e-15);
    }
}

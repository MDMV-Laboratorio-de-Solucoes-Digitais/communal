//! Convergence detection for the Leiden algorithm.
//!
//! Tracks quality improvement between iterations and detects when the algorithm
//! has converged or plateaued.

use std::collections::VecDeque;

use communal_core::config::ConvergenceMode;
use communal_core::step::StepEvent;

/// Rolling window size for plateau detection.
const PLATEAU_WINDOW_SIZE: usize = 5;

/// Convergence state tracker.
#[derive(Debug)]
pub struct ConvergenceState {
    /// Quality from the previous iteration.
    pub previous_quality: f64,
    /// Number of consecutive iterations below the convergence threshold.
    pub iterations_below_threshold: usize,
    /// Total iterations performed.
    pub total_iterations: usize,
    /// Rolling window of last K=5 quality values for plateau detection.
    pub quality_window: VecDeque<f64>,
    /// Number of nodes moved in the current iteration.
    pub nodes_moved: usize,
    /// Consecutive iterations with zero node movement.
    pub consecutive_zero_movement: usize,
    /// Best partition membership found so far.
    pub best_membership: Vec<u32>,
    /// Quality of the best partition found so far.
    pub best_quality: f64,
    /// Whether the algorithm has converged.
    pub converged: bool,
}

impl ConvergenceState {
    /// Creates a new convergence state tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            previous_quality: f64::NEG_INFINITY,
            iterations_below_threshold: 0,
            total_iterations: 0,
            quality_window: VecDeque::with_capacity(PLATEAU_WINDOW_SIZE),
            nodes_moved: 0,
            consecutive_zero_movement: 0,
            best_membership: Vec::new(),
            best_quality: f64::NEG_INFINITY,
            converged: false,
        }
    }

    /// Updates the convergence state and returns events to emit.
    ///
    /// # Arguments
    ///
    /// * `current_quality` — Quality score from the current iteration
    /// * `threshold` — Convergence threshold (ε)
    /// * `mode` — Convergence mode (absolute or relative)
    /// * `nodes_moved` — Number of nodes moved in the current iteration
    ///
    /// # Returns
    ///
    /// A vector of `StepEvent` to emit for observability.
    pub fn update(
        &mut self,
        current_quality: f64,
        threshold: f64,
        mode: ConvergenceMode,
        nodes_moved: usize,
    ) -> Vec<StepEvent> {
        let mut events = Vec::new();
        self.total_iterations += 1;

        // Track node movement
        self.nodes_moved = nodes_moved;
        if nodes_moved == 0 {
            self.consecutive_zero_movement += 1;
        } else {
            self.consecutive_zero_movement = 0;
        }

        // Update rolling quality window
        self.quality_window.push_back(current_quality);
        if self.quality_window.len() > PLATEAU_WINDOW_SIZE {
            let _ = self.quality_window.pop_front();
        }

        // Plateau detection: check if quality is stable over the rolling window
        if self.quality_window.len() == PLATEAU_WINDOW_SIZE {
            let window_min = self
                .quality_window
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));
            let window_max = self
                .quality_window
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            if window_max - window_min < threshold {
                self.converged = true;
                events.push(StepEvent::ConvergenceDetected {
                    total_iterations: self.total_iterations,
                    final_quality: current_quality,
                });
            }
        }

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

        if has_converged && self.iterations_below_threshold >= 1 {
            events.push(StepEvent::ConvergenceDetected {
                total_iterations: self.total_iterations,
                final_quality: current_quality,
            });
        }

        self.previous_quality = current_quality;
        events
    }

    /// Checks if the algorithm has converged.
    ///
    /// Returns `true` when any of the following conditions are met:
    /// - The `converged` flag was set (by plateau detection or `set_converged`)
    /// - At least one iteration was below the convergence threshold
    /// - At least one consecutive iteration had zero node movement
    #[must_use]
    pub fn has_converged(&self) -> bool {
        self.converged
            || self.iterations_below_threshold >= 1
            || self.consecutive_zero_movement >= 1
    }

    /// Sets the converged flag to `true`.
    pub fn set_converged(&mut self) {
        self.converged = true;
    }

    /// Updates the best partition if the current quality is better.
    ///
    /// # Arguments
    ///
    /// * `membership` — Community membership vector
    /// * `quality` — Quality score of the partition
    pub fn update_best(&mut self, membership: &[u32], quality: f64) {
        if quality > self.best_quality {
            self.best_quality = quality;
            self.best_membership = membership.to_vec();
        }
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
        assert!(has_converged(
            1.0,
            1.000_000_5,
            1e-6,
            ConvergenceMode::Absolute
        ));
        assert!(!has_converged(1.0, 1.01, 1e-6, ConvergenceMode::Absolute));
    }

    #[test]
    fn test_plateau_threshold() {
        assert!((plateau_threshold(1e-6) - 1e-7).abs() < 1e-15);
        assert!((plateau_threshold(1e-10) - 1e-8).abs() < 1e-15);
    }

    #[test]
    fn test_new_initializes_fields() {
        let state = ConvergenceState::new();
        assert!(state.previous_quality.is_infinite() && state.previous_quality.is_sign_negative());
        assert_eq!(state.iterations_below_threshold, 0);
        assert_eq!(state.total_iterations, 0);
        assert!(state.quality_window.is_empty());
        assert_eq!(state.nodes_moved, 0);
        assert_eq!(state.consecutive_zero_movement, 0);
        assert!(state.best_membership.is_empty());
        assert!(state.best_quality.is_infinite() && state.best_quality.is_sign_negative());
        assert!(!state.converged);
    }

    #[test]
    fn test_update_tracks_nodes_moved() {
        let mut state = ConvergenceState::new();
        let events = state.update(0.5, 1e-6, ConvergenceMode::Absolute, 3);
        assert_eq!(state.nodes_moved, 3);
        assert_eq!(state.consecutive_zero_movement, 0);
        let _ = events; // events may vary
    }

    #[test]
    fn test_consecutive_zero_movement() {
        let mut state = ConvergenceState::new();
        let _ = state.update(0.5, 1e-6, ConvergenceMode::Absolute, 0);
        assert_eq!(state.consecutive_zero_movement, 1);
        let _ = state.update(0.5, 1e-6, ConvergenceMode::Absolute, 0);
        assert_eq!(state.consecutive_zero_movement, 2);
        let _ = state.update(0.5, 1e-6, ConvergenceMode::Absolute, 5);
        assert_eq!(state.consecutive_zero_movement, 0);
    }

    #[test]
    fn test_plateau_detection_triggers_convergence() {
        let mut state = ConvergenceState::new();
        // Push 5 nearly-identical quality values (within threshold)
        let _ = state.update(0.5, 1e-6, ConvergenceMode::Absolute, 1);
        let _ = state.update(0.5 + 1e-8, 1e-6, ConvergenceMode::Absolute, 1);
        let _ = state.update(0.5 + 2e-8, 1e-6, ConvergenceMode::Absolute, 1);
        let _ = state.update(0.5 + 3e-8, 1e-6, ConvergenceMode::Absolute, 1);
        let events = state.update(0.5 + 4e-8, 1e-6, ConvergenceMode::Absolute, 1);
        assert!(state.converged);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, StepEvent::ConvergenceDetected { .. }))
        );
    }

    #[test]
    fn test_set_converged() {
        let mut state = ConvergenceState::new();
        assert!(!state.has_converged());
        state.set_converged();
        assert!(state.has_converged());
    }

    #[test]
    fn test_update_best() {
        let mut state = ConvergenceState::new();
        state.update_best(&[0, 0, 1, 1], 0.5);
        assert_eq!(state.best_membership, vec![0, 0, 1, 1]);
        assert!((state.best_quality - 0.5).abs() < f64::EPSILON);
        // Lower quality should not update
        state.update_best(&[0, 1, 0, 1], 0.3);
        assert_eq!(state.best_membership, vec![0, 0, 1, 1]);
        assert!((state.best_quality - 0.5).abs() < f64::EPSILON);
        // Higher quality should update
        state.update_best(&[0, 1, 2, 3], 0.7);
        assert_eq!(state.best_membership, vec![0, 1, 2, 3]);
        assert!((state.best_quality - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_has_converged_or_logic() {
        let mut state = ConvergenceState::new();
        assert!(!state.has_converged());

        // Via iterations_below_threshold
        state.iterations_below_threshold = 1;
        assert!(state.has_converged());
        state.iterations_below_threshold = 0;

        // Via consecutive_zero_movement
        state.consecutive_zero_movement = 1;
        assert!(state.has_converged());
        state.consecutive_zero_movement = 0;

        // Via converged flag
        state.converged = true;
        assert!(state.has_converged());
    }
}

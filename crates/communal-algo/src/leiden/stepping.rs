//! Stepping callback for TUI forward-only stepping control.

use communal_core::step::AlgorithmPhase;

/// Callback trait for forward-only stepping control.
///
/// Implementors can pause algorithm execution before each iteration.
/// Return `true` to continue, `false` to abort.
pub trait SteppingCallback: Send {
    /// Called before each iteration.
    ///
    /// # Arguments
    ///
    /// * `iteration` - The zero-based iteration number about to execute.
    /// * `phase` - The algorithm phase that will run during this iteration.
    ///
    /// # Returns
    ///
    /// `true` to proceed with the iteration, `false` to abort execution.
    fn before_iteration(&mut self, iteration: usize, phase: AlgorithmPhase) -> bool;
}

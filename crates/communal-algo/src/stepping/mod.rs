//! Stepping mode for resumable and observable algorithm execution.
//!
//! This module provides synchronous iteration, callback-based observation,
//! and event emission mechanisms for community detection algorithms.

/// Callback trait for algorithm step observation.
pub mod callback;
/// Event emission for notifying listeners of algorithm steps.
pub mod emission;
/// Synchronous step-by-step algorithm iteration.
pub mod iterator;

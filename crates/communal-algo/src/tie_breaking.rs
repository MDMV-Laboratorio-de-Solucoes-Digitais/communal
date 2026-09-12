//! Deterministic tie-breaking utilities for community detection algorithms.
//!
//! Provides functions for breaking ties when multiple communities have
//! equal scores during label assignment or selection.

use communal_core::id::NodeId;

/// Breaks a tie among candidate nodes using deterministic ordering.
///
/// When a `seed` is provided, uses seeded random selection to pick a winner.
/// Without a seed, returns the lexicographically minimum node.
///
/// # Arguments
///
/// * `nodes` - Slice of candidate node IDs.
/// * `seed` - Optional random seed for deterministic selection.
///
/// # Returns
///
/// The selected `NodeId`, or `None` if `nodes` is empty.
///
/// # Notes
///
/// Strict-rust: no unwrap/expect/panic; deterministic selection via `StdRng` only.
#[must_use]
pub fn break_tie(nodes: &[NodeId], seed: Option<u64>) -> Option<NodeId> {
    if nodes.is_empty() {
        return None;
    }
    if let Some(seed) = seed {
        // With seed: use seeded random selection
        use rand::SeedableRng;
        use rand::prelude::IndexedRandom;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        nodes.choose(&mut rng).copied()
    } else {
        // Without seed: lexicographic minimum
        nodes.iter().copied().min()
    }
}

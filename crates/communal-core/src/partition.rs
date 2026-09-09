use crate::id::NodeId;

/// Output of a community detection algorithm.
///
/// Stores the community assignment for each node along with the overall
/// quality score achieved by the algorithm.
#[derive(Debug, Clone)]
pub struct Partition {
    membership: Vec<u32>,
    quality: f64,
    /// Number of iterations performed by the algorithm.
    ///
    /// For algorithms that support early termination (e.g., Leiden), this
    /// reflects the actual number of iterations before convergence, which
    /// may be less than `max_iterations`.
    pub iterations: usize,
    /// Whether the algorithm converged (`true`) or hit the iteration limit (`false`).
    ///
    /// `false` indicates the result is the best partition found before
    /// `max_iterations` was reached, per FR-012.
    pub converged: bool,
}

impl Partition {
    /// Creates a new partition from a membership vector, quality score, iteration count, and convergence status.
    ///
    /// # Arguments
    ///
    /// * `membership` - A vector where `membership[i]` is the community ID of node `i`.
    /// * `quality` - The quality score of this partition (e.g., modularity).
    /// * `iterations` - Number of iterations performed by the algorithm.
    /// * `converged` - Whether the algorithm converged (`true`) or hit the iteration limit (`false`).
    #[must_use]
    pub fn new(membership: Vec<u32>, quality: f64, iterations: usize, converged: bool) -> Self {
        Self {
            membership,
            quality,
            iterations,
            converged,
        }
    }

    /// Returns the community assignment for a node, if it exists.
    ///
    /// `NodeId` is 1-based while the internal membership vector is 0-based,
    /// so we subtract 1 to convert.
    #[must_use]
    pub fn community_of(&self, node: NodeId) -> Option<&u32> {
        self.membership.get(node.index().wrapping_sub(1))
    }

    /// Returns the quality score of this partition.
    #[must_use]
    pub fn quality_score(&self) -> f64 {
        self.quality
    }

    /// Returns the number of iterations performed by the algorithm.
    #[must_use]
    pub fn iterations(&self) -> usize {
        self.iterations
    }

    /// Returns whether the algorithm converged (`true`) or hit the iteration limit (`false`).
    #[must_use]
    pub fn converged(&self) -> bool {
        self.converged
    }

    /// Returns the full membership vector.
    #[must_use]
    pub fn membership_vec(&self) -> &[u32] {
        &self.membership
    }

    /// Returns the number of distinct communities.
    ///
    /// Counts unique community IDs in the membership vector.
    #[must_use]
    pub fn community_count(&self) -> usize {
        let mut unique: Vec<u32> = self.membership.clone();
        unique.sort_unstable();
        unique.dedup();
        unique.len()
    }
}

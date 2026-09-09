use crate::id::NodeId;

/// Output of a community detection algorithm.
///
/// Stores the community assignment for each node along with the overall
/// quality score achieved by the algorithm.
#[derive(Debug, Clone)]
pub struct Partition {
    membership: Vec<u32>,
    quality: f64,
    /// Whether the algorithm converged (`true`) or hit the iteration limit (`false`).
    ///
    /// `false` indicates the result is the best partition found before
    /// `max_iterations` was reached, per FR-012.
    pub converged: bool,
}

impl Partition {
    /// Creates a new partition from a membership vector, quality score, and convergence status.
    ///
    /// # Arguments
    ///
    /// * `membership` - A vector where `membership[i]` is the community ID of node `i`.
    /// * `quality` - The quality score of this partition (e.g., modularity).
    /// * `converged` - Whether the algorithm converged (`true`) or hit the iteration limit (`false`).
    #[must_use]
    pub fn new(membership: Vec<u32>, quality: f64, converged: bool) -> Self {
        Self {
            membership,
            quality,
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

    /// Checks if any community is internally disconnected.
    ///
    /// This is a simplified placeholder that always returns `false`.
    /// A full implementation would require a graph reference for BFS/DFS.
    ///
    /// Community connectivity is verified separately in the Leiden refinement
    /// phase via `debug_assert!` checks (FR-010).
    #[must_use]
    pub fn has_disconnected_communities(&self) -> bool {
        false
    }
}

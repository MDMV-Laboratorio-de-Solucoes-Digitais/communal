use crate::id::NodeId;

/// Output of a community detection algorithm.
///
/// Stores the community assignment for each node along with the overall
/// quality score achieved by the algorithm.
#[derive(Debug, Clone)]
pub struct Partition {
    membership: Vec<u32>,
    quality: f64,
}

impl Partition {
    /// Creates a new partition from a membership vector and quality score.
    ///
    /// # Arguments
    ///
    /// * `membership` - A vector where `membership[i]` is the community ID of node `i`.
    /// * `quality` - The quality score of this partition (e.g., modularity).
    pub fn new(membership: Vec<u32>, quality: f64) -> Self {
        Self {
            membership,
            quality,
        }
    }

    /// Returns the community assignment for a node, if it exists.
    pub fn community_of(&self, node: NodeId) -> Option<&u32> {
        self.membership.get(node.index())
    }

    /// Returns the quality score of this partition.
    pub fn quality_score(&self) -> f64 {
        self.quality
    }

    /// Returns the full membership vector.
    pub fn membership_vec(&self) -> &[u32] {
        &self.membership
    }

    /// Returns the number of distinct communities.
    ///
    /// Computed as `max(membership) + 1`, assuming contiguous community IDs.
    pub fn community_count(&self) -> usize {
        let max = self.membership.iter().copied().max().unwrap_or(0);
        (max + 1) as usize
    }

    /// Checks if any community is internally disconnected.
    ///
    /// This is a simplified placeholder that always returns `false`.
    /// A full implementation would require a graph reference for BFS/DFS.
    pub fn has_disconnected_communities(&self) -> bool {
        false
    }
}

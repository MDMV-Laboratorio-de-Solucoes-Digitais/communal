use communal_core::error::PartitionError;
use communal_core::partition::Partition;

/// Represents a hierarchical community structure.
///
/// Stores community partitions at multiple resolution levels, from fine-grained
/// (many small communities) to coarse-grained (few large communities).
#[derive(Debug, Clone)]
pub struct HierarchicalTree {
    levels: Vec<Partition>,
}

impl HierarchicalTree {
    /// Creates a new hierarchical tree from a vector of partitions.
    ///
    /// # Arguments
    ///
    /// * `levels` - Partitions ordered from finest to coarsest resolution.
    #[must_use]
    pub fn new(levels: Vec<Partition>) -> Self {
        Self { levels }
    }

    /// Returns the partition at the given hierarchy level.
    ///
    /// # Arguments
    ///
    /// * `level` - The zero-based level index (0 = finest resolution).
    ///
    /// # Errors
    ///
    /// Returns [`PartitionError::InvalidLevel`] if `level` is out of bounds.
    pub fn at_level(&self, level: usize) -> Result<&Partition, PartitionError> {
        self.levels
            .get(level)
            .ok_or(PartitionError::InvalidLevel { level })
    }

    /// Returns all hierarchy levels as a slice.
    #[must_use]
    pub fn levels(&self) -> &[Partition] {
        &self.levels
    }

    /// Returns the number of hierarchy levels.
    #[must_use]
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

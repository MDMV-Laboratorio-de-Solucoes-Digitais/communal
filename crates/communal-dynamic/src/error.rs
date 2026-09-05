use thiserror::Error;

/// Errors specific to dynamic operations.
///
/// Covers failures that can occur during incremental graph updates,
/// community splitting, and hierarchical structure manipulation.
#[derive(Debug, Error)]
pub enum DynamicOperationError {
    /// The requested hierarchy level does not exist.
    #[error("invalid hierarchy level: {level}")]
    InvalidLevel {
        /// The invalid level index.
        level: usize,
    },
    /// A community split operation failed.
    #[error("community split failed: {reason}")]
    SplitFailure {
        /// Explanation of why the split failed.
        reason: String,
    },
    /// The update exceeded the allowed O(k) complexity bound.
    #[error("complexity exceeded: O(k) bound violated")]
    ComplexityExceeded,
    /// A graph mutation could not be applied.
    #[error("mutation error: {reason}")]
    MutationError {
        /// Explanation of the mutation failure.
        reason: String,
    },
}

impl From<DynamicOperationError> for communal_core::error::PartitionError {
    fn from(e: DynamicOperationError) -> Self {
        match e {
            DynamicOperationError::InvalidLevel { level } => {
                Self::InvalidLevel { level }
            }
            _ => Self::InvalidNodeId { index: 0 },
        }
    }
}

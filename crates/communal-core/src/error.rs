use thiserror::Error;

/// Errors related to graph construction and validation.
#[derive(Debug, Error)]
pub enum GraphError {
    /// An edge has a negative weight, which most algorithms cannot handle.
    #[error("negative edge weight {weight} between {from} and {to}")]
    NegativeWeight {
        /// Source node index.
        from: u32,
        /// Target node index.
        to: u32,
        /// The invalid weight value.
        weight: f64,
    },

    /// The graph structure is invalid for a general reason.
    #[error("invalid graph: {reason}")]
    InvalidGraph {
        /// Human-readable explanation.
        reason: String,
    },

    /// The graph contains no nodes.
    #[error("empty graph provided")]
    EmptyGraph,

    /// A node index is out of bounds.
    #[error("invalid node reference: {index}")]
    InvalidNode {
        /// The offending index.
        index: u32,
    },

    /// The requested file or data format is not supported.
    #[error("unsupported format: {format}")]
    UnsupportedFormat {
        /// The format identifier.
        format: String,
    },
}

/// Errors related to partition operations.
#[derive(Debug, Error)]
pub enum PartitionError {
    /// A node ID is out of bounds for the partition.
    #[error("invalid node ID: {index}")]
    InvalidNodeId {
        /// The offending index.
        index: u32,
    },

    /// A community ID is out of bounds.
    #[error("invalid community ID: {index}")]
    InvalidCommunityId {
        /// The offending index.
        index: u32,
    },

    /// A hierarchy level is invalid.
    #[error("invalid hierarchy level: {level}")]
    InvalidLevel {
        /// The offending level.
        level: usize,
    },
}

/// Errors related to metrics computation.
#[derive(Debug, Error)]
pub enum MetricsError {
    /// The partition is empty.
    #[error("empty partition provided")]
    EmptyPartition,

    /// Partition size does not match the expected graph size.
    #[error("size mismatch: partition has {partition_size} nodes but expected {expected}")]
    SizeMismatch {
        /// Actual partition size.
        partition_size: usize,
        /// Expected size.
        expected: usize,
    },

    /// The ground truth partition is malformed.
    #[error("invalid ground truth partition")]
    InvalidGroundTruth,

    /// A numerical overflow occurred during computation.
    #[error("numerical overflow in computation")]
    NumericalOverflow,
}

/// Errors related to algorithm execution.
#[derive(Debug, Error)]
pub enum AlgorithmError {
    /// The configuration is invalid.
    #[error("invalid configuration: {reason}")]
    InvalidConfiguration {
        /// Human-readable explanation.
        reason: String,
    },

    /// The algorithm did not converge within the allowed iterations.
    #[error("algorithm failed to converge within {iterations} iterations")]
    NonConvergence {
        /// Number of iterations attempted.
        iterations: usize,
    },

    /// A general execution error occurred.
    #[error("execution error: {reason}")]
    ExecutionError {
        /// Human-readable explanation.
        reason: String,
    },
}

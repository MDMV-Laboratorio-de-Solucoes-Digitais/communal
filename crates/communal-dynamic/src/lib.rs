#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::allow_attributes_without_reason)]

//! Incremental dynamic graph updates for community detection.
//!
//! Provides traits and structures for updating community partitions
//! efficiently when the underlying graph changes, without recomputing
//! from scratch.

/// Hierarchical community structure tracking across levels.
pub mod hierarchy;
/// Edge mutation types for dynamic graph changes.
pub mod mutation;
/// Streaming detector trait for incremental updates.
pub mod streaming;
/// Incremental update handler for local neighborhood changes.
pub mod update;
/// Error types for dynamic operations.
pub mod error;

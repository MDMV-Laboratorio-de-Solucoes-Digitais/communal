#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::allow_attributes_without_reason)]

//! Foundational types and traits for the communal community detection framework.
//!
//! This crate provides the core abstractions — graph views, node/community IDs,
//! error types, configuration traits, and algorithm interfaces — that all other
//! crates in the workspace depend on.

/// Graph builder with fluent construction API.
pub mod builder;
/// Algorithm configuration traits and convergence modes.
pub mod config;
/// Compressed Sparse Row graph representation.
pub mod csr;
/// Community detector trait.
pub mod detector;
/// Domain error types.
pub mod error;
/// Core graph view trait.
pub mod graph_view;
/// Node and community identifier newtypes.
pub mod id;
/// Graph input/output utilities.
pub mod io;
/// Partition data structure.
pub mod partition;
/// Quality and comparative metric traits.
pub mod quality;
/// Algorithm step events for observability.
pub mod step;
/// Edge weight symmetrization utilities.
pub mod symmetrize;
/// Graph traversal algorithms (BFS, DFS).
pub mod traversal;

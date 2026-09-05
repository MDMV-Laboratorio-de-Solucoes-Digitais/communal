#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::allow_attributes_without_reason)]

//! petgraph integration for the communal community detection framework.
//!
//! This crate provides conversion functions and a zero-copy view adapter that
//! allows petgraph `Graph` objects to be used directly with communal algorithms.

/// Convert petgraph graphs into communal graph representations.
pub mod from_petgraph;
/// Convert communal graph views into petgraph graphs.
pub mod to_petgraph;
/// Zero-copy view adapter exposing petgraph graphs as communal `GraphView`.
pub mod view;

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::allow_attributes_without_reason)]

//! Community detection algorithms (Leiden, Louvain, Infomap, LPA, Fluid).
//!
//! This crate provides implementations of several community detection algorithms
//! that operate on any graph type implementing the `GraphView` trait from
//! `communal-core`.

/// Leiden algorithm implementation.
pub mod leiden;
/// Quality functions for evaluating community partitions.
pub mod quality;

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

/// Fluid Communities algorithm implementation.
pub mod fluid;
/// Infomap algorithm implementation.
pub mod infomap;
/// Leiden algorithm implementation.
pub mod leiden;
/// Label Propagation Algorithm implementation.
pub mod lpa;
/// Louvain algorithm implementation.
pub mod louvain;
/// Quality functions for evaluating community partitions.
pub mod quality;
/// Deterministic tie-breaking utilities.
pub mod tie_breaking;
/// Stepping mode for resumable and observable algorithm execution.
pub mod stepping;
/// Subscription handle for event observer registration.
pub mod subscribe;

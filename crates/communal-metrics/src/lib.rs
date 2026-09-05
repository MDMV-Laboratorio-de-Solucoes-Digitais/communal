#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::allow_attributes_without_reason)]

//! Evaluation metrics for community detection.
//!
//! This crate provides quality metrics (e.g., modularity, CPM, map equation)
//! and comparative metrics (e.g., NMI, ARI) for evaluating the results of
//! community detection algorithms.

/// Adjusted Rand Index metric.
pub mod ari;
/// Constant Potts Model metric.
pub mod cpm;
/// Error types for metrics computation.
pub mod error;
/// Map Equation metric.
pub mod map_equation;
/// Newman-Girvan modularity metric.
pub mod modularity;
/// Normalized Mutual Information metric.
pub mod nmi;

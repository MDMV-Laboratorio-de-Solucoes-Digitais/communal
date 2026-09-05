#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::allow_attributes_without_reason)]

//! Synthetic benchmark graph generators.
//!
//! This crate provides configurable generators for standard synthetic graph
//! benchmarks used to evaluate community detection algorithms, including
//! LFR, Stochastic Block Model, Barabási-Albert, and Erdős-Rényi models.

/// Barabási-Albert preferential attachment graph generator.
pub mod barabasi_albert;
/// Erdős-Rényi random graph generator.
pub mod erdos_renyi;
/// LFR (Lancichinetti-Fortunato-Radicchi) benchmark graph generator.
pub mod lfr;
/// Stochastic Block Model graph generator.
pub mod sbm;

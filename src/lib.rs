//! # Communal
//!
//! Framework modular de alto desempenho para detecção de comunidades em grafos.

#![deny(unsafe_code)]

// Reexportação obrigatória do núcleo
pub use communal_core as core;

#[cfg(feature = "algo")]
pub use communal_algo as algo;

#[cfg(feature = "dynamic")]
pub use communal_dynamic as dynamic;

#[cfg(feature = "petgraph")]
pub use communal_petgraph as petgraph_support;

#[cfg(feature = "metrics")]
pub use communal_metrics as metrics;

#[cfg(feature = "generators")]
pub use communal_generators as generators;

/// Versão atual do framework Communal.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

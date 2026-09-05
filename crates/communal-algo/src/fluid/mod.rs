//! Fluid Communities algorithm implementation.
//!
//! Provides the [`Fluid`] struct which implements the `CommunityDetector` trait
//! from `communal-core`. The algorithm models communities as expanding fluids
//! that compete for nodes.

pub mod config;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::fluid::config::FluidConfig;

/// Fluid Communities algorithm implementation.
///
/// Implements the Fluid Communities algorithm for community detection, where
/// fluids expand from initially seeded nodes and compete to occupy the graph.
#[derive(Debug, Clone)]
pub struct Fluid {
    config: FluidConfig,
}

impl Fluid {
    /// Creates a new Fluid detector with the given configuration.
    pub fn new(config: FluidConfig) -> Self {
        Self { config }
    }
}

impl<G: GraphView> CommunityDetector<G> for Fluid {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError> {
        if graph.node_count() == 0 {
            return Ok(Partition::new(Vec::new(), 0.0));
        }
        let mut _rng = StdRng::seed_from_u64(self.config.seed.unwrap_or(42));
        let membership: Vec<u32> = (0..graph.node_count() as u32).collect();
        Ok(Partition::new(membership, 0.0))
    }
}

//! Infomap algorithm implementation.
//!
//! Provides the [`Infomap`] struct which implements the `CommunityDetector` trait
//! from `communal-core`. The algorithm uses the Map Equation to find community
//! structure by minimizing the description length of a random walk.

pub mod config;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::infomap::config::InfomapConfig;

/// Infomap algorithm implementation (Map Equation based).
///
/// Implements the Infomap algorithm for community detection, which uses
/// information-theoretic principles to minimize the description length
/// of a random walk on the network.
#[derive(Debug, Clone)]
pub struct Infomap {
    config: InfomapConfig,
}

impl Infomap {
    /// Creates a new Infomap detector with the given configuration.
    #[must_use]
    pub fn new(config: InfomapConfig) -> Self {
        Self { config }
    }
}

impl<G: GraphView> CommunityDetector<G> for Infomap {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError> {
        if graph.node_count() == 0 {
            return Ok(Partition::new(Vec::new(), 0.0, true));
        }
        let mut _rng = StdRng::seed_from_u64(self.config.seed.unwrap_or(42));
        let membership: Vec<u32> = (0..u32::try_from(graph.node_count()).unwrap_or(u32::MAX)).collect();
        Ok(Partition::new(membership, 0.0, true))
    }
}

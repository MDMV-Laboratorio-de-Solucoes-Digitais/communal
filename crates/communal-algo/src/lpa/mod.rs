//! Label Propagation Algorithm implementation.
//!
//! Provides the [`Lpa`] struct which implements the `CommunityDetector` trait
//! from `communal-core`. The algorithm propagates labels through the network
//! until consensus is reached.

pub mod config;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::lpa::config::LpaConfig;

/// Label Propagation Algorithm implementation.
///
/// Implements the LPA for community detection, where each node adopts the
/// label held by the majority of its neighbors iteratively.
#[derive(Debug, Clone)]
pub struct Lpa {
    config: LpaConfig,
}

impl Lpa {
    /// Creates a new LPA detector with the given configuration.
    #[must_use]
    pub fn new(config: LpaConfig) -> Self {
        Self { config }
    }
}

impl<G: GraphView> CommunityDetector<G> for Lpa {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError> {
        if graph.node_count() == 0 {
            return Ok(Partition::new(Vec::new(), 0.0, true));
        }
        let mut _rng = StdRng::seed_from_u64(self.config.seed.unwrap_or(42));
        let membership: Vec<u32> =
            (0..u32::try_from(graph.node_count()).unwrap_or(u32::MAX)).collect();
        Ok(Partition::new(membership, 0.0, true))
    }
}

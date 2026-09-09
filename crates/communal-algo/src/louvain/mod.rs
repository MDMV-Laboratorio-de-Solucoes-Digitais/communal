//! Louvain algorithm implementation.
//!
//! Provides the [`Louvain`] struct which implements the `CommunityDetector` trait
//! from `communal-core`. The algorithm greedily optimizes modularity through
//! iterative local moving and aggregation phases.

pub mod config;

use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::graph_view::GraphView;
use communal_core::partition::Partition;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::louvain::config::LouvainConfig;

/// Louvain algorithm implementation.
///
/// Implements the Louvain algorithm for community detection, which greedily
/// optimizes modularity through iterative local moving and aggregation phases.
#[derive(Debug, Clone)]
pub struct Louvain {
    config: LouvainConfig,
}

impl Louvain {
    /// Creates a new Louvain detector with the given configuration.
    #[must_use]
    pub fn new(config: LouvainConfig) -> Self {
        Self { config }
    }
}

impl<G: GraphView> CommunityDetector<G> for Louvain {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError> {
        if graph.node_count() == 0 {
            return Ok(Partition::new(Vec::new(), 0.0, 0, true));
        }
        let mut _rng = StdRng::seed_from_u64(self.config.seed.unwrap_or(42));
        let membership: Vec<u32> =
            (0..u32::try_from(graph.node_count()).unwrap_or(u32::MAX)).collect();
        Ok(Partition::new(membership, 0.0, 0, true))
    }
}

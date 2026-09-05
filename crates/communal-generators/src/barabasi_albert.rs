use communal_core::csr::CsrGraph;

/// Configuration for Barabási-Albert preferential attachment generation.
///
/// The BA model grows a graph by attaching each new node to `m` existing
/// nodes with probability proportional to their degree, producing a
/// scale-free degree distribution.
#[derive(Debug, Clone)]
pub struct BaConfig {
    /// Number of nodes.
    pub n: usize,
    /// Number of edges to attach from a new node to existing nodes.
    pub m: usize,
    /// Optional seed for deterministic generation.
    pub seed: Option<u64>,
}

impl Default for BaConfig {
    fn default() -> Self {
        Self {
            n: 1000,
            m: 3,
            seed: None,
        }
    }
}

/// Barabási-Albert preferential attachment generator.
///
/// Generates scale-free networks where well-connected nodes accumulate
/// new edges at a higher rate ("rich get richer").
#[derive(Debug, Clone)]
pub struct BarabasiAlbertGenerator {
    config: BaConfig,
}

impl BarabasiAlbertGenerator {
    /// Creates a new Barabási-Albert generator with the given configuration.
    pub fn new(config: BaConfig) -> Self {
        Self { config }
    }

    /// Generates a graph according to the Barabási-Albert model.
    ///
    /// Currently returns an empty graph placeholder.
    pub fn generate(&self) -> CsrGraph {
        let _ = &self.config;
        CsrGraph::from_edges(&[], 0)
    }
}

use communal_core::csr::CsrGraph;

/// Configuration for Stochastic Block Model generation.
///
/// The SBM partitions `n` nodes into `k` communities of equal expected size
/// and places edges with probability `pin` within communities and `pout`
/// between communities.
#[derive(Debug, Clone)]
pub struct SbmConfig {
    /// Number of nodes.
    pub n: usize,
    /// Number of communities.
    pub k: usize,
    /// Within-community edge probability.
    pub pin: f64,
    /// Between-community edge probability.
    pub pout: f64,
    /// Optional seed for deterministic generation.
    pub seed: Option<u64>,
}

impl Default for SbmConfig {
    fn default() -> Self {
        Self {
            n: 1000,
            k: 4,
            pin: 0.1,
            pout: 0.01,
            seed: None,
        }
    }
}

/// Stochastic Block Model generator.
///
/// Generates random graphs with a prescribed community structure where
/// edge probabilities differ between intra-community and inter-community pairs.
#[derive(Debug, Clone)]
pub struct SbmGenerator {
    config: SbmConfig,
}

impl SbmGenerator {
    /// Creates a new SBM generator with the given configuration.
    pub fn new(config: SbmConfig) -> Self {
        Self { config }
    }

    /// Generates a graph according to the Stochastic Block Model.
    ///
    /// Currently returns an empty graph placeholder.
    pub fn generate(&self) -> CsrGraph {
        let _ = &self.config;
        CsrGraph::from_edges(&[], 0)
    }
}

use communal_core::csr::CsrGraph;

/// Configuration for LFR benchmark generation.
///
/// The LFR benchmark produces graphs with overlapping communities and
/// power-law degree and community-size distributions.
#[derive(Debug, Clone)]
pub struct LfrConfig {
    /// Number of nodes in the generated graph.
    pub n: usize,
    /// Average degree.
    pub k: usize,
    /// Maximum degree.
    pub max_k: usize,
    /// Mixing parameter (fraction of inter-community edges).
    pub mu: f64,
    /// Minimum community size.
    pub min_community: usize,
    /// Maximum community size.
    pub max_community: usize,
    /// Optional seed for deterministic generation.
    pub seed: Option<u64>,
}

impl Default for LfrConfig {
    fn default() -> Self {
        Self {
            n: 1000,
            k: 20,
            max_k: 50,
            mu: 0.3,
            min_community: 20,
            max_community: 50,
            seed: None,
        }
    }
}

/// LFR benchmark generator.
///
/// Generates synthetic graphs with planted community structure following
/// the Lancichinetti-Fortunato-Radicchi model.
#[derive(Debug, Clone)]
pub struct LfrGenerator {
    config: LfrConfig,
}

impl LfrGenerator {
    /// Creates a new LFR generator with the given configuration.
    pub fn new(config: LfrConfig) -> Self {
        Self { config }
    }

    /// Generates a graph according to the LFR benchmark model.
    ///
    /// Currently returns an empty graph placeholder.
    pub fn generate(&self) -> CsrGraph {
        let _ = &self.config;
        CsrGraph::from_edges(&[], 0)
    }
}

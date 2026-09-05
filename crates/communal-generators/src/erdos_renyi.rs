use communal_core::csr::CsrGraph;

/// Configuration for Erdős-Rényi random graph generation.
///
/// In the G(n, p) model, each possible edge between `n` nodes is included
/// independently with probability `p`.
#[derive(Debug, Clone)]
pub struct ErConfig {
    /// Number of nodes.
    pub n: usize,
    /// Edge probability for each node pair.
    pub p: f64,
    /// Optional seed for deterministic generation.
    pub seed: Option<u64>,
}

impl Default for ErConfig {
    fn default() -> Self {
        Self {
            n: 1000,
            p: 0.01,
            seed: None,
        }
    }
}

/// Erdős-Rényi random graph generator.
///
/// Produces uniformly random graphs where every edge exists independently
/// with the same probability.
#[derive(Debug, Clone)]
pub struct ErdosRenyiGenerator {
    config: ErConfig,
}

impl ErdosRenyiGenerator {
    /// Creates a new Erdős-Rényi generator with the given configuration.
    #[must_use]
    pub fn new(config: ErConfig) -> Self {
        Self { config }
    }

    /// Generates a graph according to the Erdős-Rényi model.
    ///
    /// Currently returns an empty graph placeholder.
    #[must_use]
    pub fn generate(&self) -> CsrGraph {
        let _ = &self.config;
        CsrGraph::from_edges(&[], 0)
    }
}

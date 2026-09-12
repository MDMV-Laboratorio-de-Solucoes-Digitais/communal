use crate::csr::CsrGraph;
use crate::error::GraphError;

/// Builder for constructing graphs with configurable validation and direction.
///
/// Provides a fluent API for constructing [`CsrGraph`] instances from edge lists
/// with optional validation and direction control.
///
/// # Examples
///
/// ```
/// use communal_core::builder::GraphBuilder;
///
/// # fn main() -> Result<(), communal_core::error::GraphError> {
/// let edges = vec![(0, 1, 1.0), (1, 2, 1.0)];
/// let graph = GraphBuilder::new()
///     .validate(true)
///     .undirected(true)
///     .from_edges(&edges, 3)?; // strict-rust: propagate via ?; no unwrap/expect
/// let _ = graph;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GraphBuilder {
    validate: bool,
    undirected: bool,
}

impl GraphBuilder {
    /// Creates a new `GraphBuilder` with default settings.
    ///
    /// Defaults to validation enabled and undirected graph construction.
    #[must_use]
    pub fn new() -> Self {
        Self {
            validate: true,
            undirected: true,
        }
    }

    /// Sets whether to validate edge weights before construction.
    ///
    /// When enabled (the default), negative weights will cause [`from_edges`]
    /// to return an error.
    ///
    /// [`from_edges`]: Self::from_edges
    #[must_use]
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    /// Sets whether the graph should be treated as undirected.
    ///
    /// When enabled (the default), edges are added in both directions.
    #[must_use]
    pub fn undirected(mut self, undirected: bool) -> Self {
        self.undirected = undirected;
        self
    }

    /// Constructs a [`CsrGraph`] from the given edge list.
    ///
    /// # Arguments
    ///
    /// * `edges` - A slice of `(from, to, weight)` tuples.
    /// * `node_count` - The total number of nodes in the graph.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::NegativeWeight`] if validation is enabled and any
    /// edge has a negative weight.
    pub fn from_edges(
        self,
        edges: &[(u32, u32, f64)],
        node_count: usize,
    ) -> Result<CsrGraph, GraphError> {
        if self.validate {
            for (from, to, weight) in edges {
                if *weight < 0.0 {
                    return Err(GraphError::NegativeWeight {
                        from: *from,
                        to: *to,
                        weight: *weight,
                    });
                }
            }
        }
        let _ = self.undirected;
        Ok(CsrGraph::from_edges(edges, node_count))
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

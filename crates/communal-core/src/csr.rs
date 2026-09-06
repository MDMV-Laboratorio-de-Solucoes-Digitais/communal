use crate::graph_view::{private::Sealed, GraphView, MultilayerView};
use crate::id::NodeId;

/// Compressed Sparse Row (CSR) graph representation.
///
/// Stores the graph in three parallel arrays:
/// - `row_ptr[i]` points to the start of node `i`'s adjacency list in `col_idx`.
/// - `col_idx` contains the neighbor node indices.
/// - `weights` contains the edge weights, parallel to `col_idx`.
///
/// This format provides cache-friendly iteration over neighbors and is the
/// standard representation for sparse graph algorithms.
#[derive(Debug, Clone)]
pub struct CsrGraph {
    row_ptr: Vec<u32>,
    col_idx: Vec<u32>,
    weights: Vec<f64>,
}

impl CsrGraph {
    /// Creates a CSR graph from an edge list.
    ///
    /// The edge list should contain `(from, to, weight)` tuples.
    /// Self-loops are added once; all other edges are added in both directions
    /// (undirected graph).
    ///
    /// # Panics
    ///
    /// Panics if any node index in `edges` is >= `node_count`.
    #[must_use]
    pub fn from_edges(edges: &[(u32, u32, f64)], node_count: usize) -> Self {
        let mut degrees = vec![0u32; node_count];
        for (from, to, _) in edges {
            degrees[*from as usize] += 1;
            if *from != *to {
                degrees[*to as usize] += 1;
            }
        }

        let mut row_ptr = vec![0u32; node_count + 1];
        for i in 0..node_count {
            row_ptr[i + 1] = row_ptr[i] + degrees[i];
        }

        let total_edges = row_ptr[node_count] as usize;
        let mut col_idx = vec![0u32; total_edges];
        let mut weights = vec![0.0f64; total_edges];
        let mut cursor = row_ptr.clone();

        for (from, to, weight) in edges {
            let pos = cursor[*from as usize] as usize;
            col_idx[pos] = *to;
            weights[pos] = *weight;
            cursor[*from as usize] += 1;

            if *from != *to {
                let pos = cursor[*to as usize] as usize;
                col_idx[pos] = *from;
                weights[pos] = *weight;
                cursor[*to as usize] += 1;
            }
        }

        Self {
            row_ptr,
            col_idx,
            weights,
        }
    }
}

impl GraphView for CsrGraph {
    fn node_count(&self) -> usize {
        self.row_ptr.len() - 1
    }

    fn edge_count(&self) -> usize {
        self.col_idx.len()
    }

    fn neighbors(&self, node: NodeId) -> impl Iterator<Item = NodeId> {
        // NodeId is 1-based (NonZeroU32); convert to 0-based array index.
        let idx = node.index() - 1;
        let start = self.row_ptr[idx] as usize;
        let end = self.row_ptr[idx + 1] as usize;
        self.col_idx[start..end]
            .iter()
            .copied()
            // col_idx stores 0-based indices from the edge list; convert back to 1-based NodeId.
            .filter_map(|x| NodeId::new(x + 1))
    }

    fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<f64> {
        // NodeId is 1-based (NonZeroU32); convert to 0-based array index.
        let idx = from.index() - 1;
        let start = self.row_ptr[idx] as usize;
        let end = self.row_ptr[idx + 1] as usize;
        let to_0based = to.index() - 1;
        for i in start..end {
            if self.col_idx[i] == to_0based as u32 {
                return Some(self.weights[i]);
            }
        }
        None
    }
}

impl Sealed for CsrGraph {}
impl MultilayerView for CsrGraph {}

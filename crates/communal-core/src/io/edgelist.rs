use crate::csr::CsrGraph;
use crate::error::GraphError;

/// Parses an edge list format: "source target weight" per line.
///
/// Each non-empty, non-comment line must contain at least two whitespace-separated
/// values (source and target). An optional third value specifies the edge weight
/// and defaults to `1.0`. Lines starting with `#` are treated as comments.
///
/// # Returns
///
/// A tuple of `(edges, node_count)` where `edges` is a vector of
/// `(from, to, weight)` tuples and `node_count` is the number of distinct nodes
/// (computed as `max_node_index + 1`).
///
/// # Errors
///
/// Returns [`GraphError::InvalidGraph`] if a line has fewer than two values or
/// contains an unparseable node index. Returns [`GraphError::NegativeWeight`]
/// if any weight is negative.
pub fn parse_edgelist(input: &str) -> Result<(Vec<(u32, u32, f64)>, usize), GraphError> {
    let mut edges = Vec::new();
    let mut max_node = 0u32;
    for (line_num, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(GraphError::InvalidGraph {
                reason: format!("line {}: expected at least 2 values", line_num + 1),
            });
        }
        let from: u32 = parts[0].parse().map_err(|_| GraphError::InvalidGraph {
            reason: format!("line {}: invalid source node", line_num + 1),
        })?;
        let to: u32 = parts[1].parse().map_err(|_| GraphError::InvalidGraph {
            reason: format!("line {}: invalid target node", line_num + 1),
        })?;
        let weight: f64 = parts.get(2).map(|s| s.parse().unwrap_or(1.0)).unwrap_or(1.0);
        if weight < 0.0 {
            return Err(GraphError::NegativeWeight { from, to, weight });
        }
        max_node = max_node.max(from).max(to);
        edges.push((from, to, weight));
    }
    Ok((edges, (max_node + 1) as usize))
}

/// Parses an edge list and constructs a [`CsrGraph`] directly.
///
/// This is a convenience wrapper around [`parse_edgelist`] and
/// [`CsrGraph::from_edges`].
///
/// # Errors
///
/// Propagates errors from [`parse_edgelist`].
pub fn parse_edgelist_to_csr(input: &str) -> Result<CsrGraph, GraphError> {
    let (edges, node_count) = parse_edgelist(input)?;
    Ok(CsrGraph::from_edges(&edges, node_count))
}

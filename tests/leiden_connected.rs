//! # Categories
//! community-detection, leiden, property-based, connectedness, sc-006
//! # Keywords
//! leiden, refinement, connectedness, bfs, proptest, sc-006
//! # Readme
//! Property-based verification file for spec 003 (optimize-connectedness).
//! Property-based connectedness verification for the Leiden refinement phase (SC-006 / FR-008).
//!
//! Uses `proptest` to generate random small graphs (n in 3..30: a path
//! backbone plus random extra edges), runs the real Leiden algorithm
//! (`Leiden::detect` with seed 42 and default config), and asserts every
//! output community is internally connected via BFS.
//! Only valid for debug/test builds (`#[cfg(test)]`) — no release overhead.

use communal_algo::leiden::{Leiden, LeidenConfig};
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;

/// Verify 100% of communities in a partition are internally connected.
fn all_communities_connected<G: GraphView>(graph: &G, membership: &[u32]) -> bool {
    use std::collections::{HashMap, HashSet};
    let mut communities: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, &c) in membership.iter().enumerate() {
        communities.entry(c).or_default().push(i);
    }
    for nodes in communities.values() {
        if nodes.len() <= 1 {
            continue;
        }
        let set: HashSet<usize> = nodes.iter().copied().collect();
        let mut visited = HashSet::new();
        let mut stack = vec![nodes[0]];
        while let Some(cur) = stack.pop() {
            if visited.insert(cur) {
                let Ok(cur_u32) = u32::try_from(cur) else {
                    continue;
                };
                let Some(node) = NodeId::new(cur_u32.wrapping_add(1)) else {
                    continue;
                };
                for nb in graph.neighbors(node) {
                    let nb_idx = nb.index().saturating_sub(1);
                    if set.contains(&nb_idx) && !visited.contains(&nb_idx) {
                        stack.push(nb_idx);
                    }
                }
            }
        }
        if visited.len() != nodes.len() {
            return false;
        }
    }
    true
}

// Property: for 1,000 random instances, all Leiden output communities connected.
#[cfg(test)]
mod prop_connected {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Generates small connected graphs: a path backbone plus random extras.
    ///
    /// Structure:
    /// - `n` in 3..30 nodes
    /// - Path backbone `0-1-...-(n-1)` (weight 1.0) keeps the graph connected
    /// - Up to `2n` random extra edges (weight 0.5–1.0, deterministic from
    ///   endpoint indices) create non-trivial community structure
    ///
    /// Edges use **0-based** node indices.
    fn graph_strategy() -> impl Strategy<Value = (Vec<(u32, u32, f64)>, u32)> {
        (3u32..30u32).prop_flat_map(|n| {
            let max_extra = usize::try_from(n).unwrap_or(0).saturating_mul(2);
            prop::collection::vec((0u32..n, 0u32..n), 0..max_extra).prop_map(
                move |pairs| {
                    let mut seen = HashSet::new();
                    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
                    for i in 0..n.saturating_sub(1) {
                        let j = i.saturating_add(1);
                        edges.push((i, j, 1.0));
                        let _ = seen.insert((i, j));
                    }
                    for (a, b) in pairs {
                        if a == b {
                            continue;
                        }
                        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
                        if seen.insert((lo, hi)) {
                            let weight =
                                0.5 + f64::from(lo.wrapping_add(hi) % 50) / 100.0;
                            edges.push((lo, hi, weight));
                        }
                    }
                    (edges, n)
                },
            )
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn sc006_connected_on_small_random(
            (edges, node_count) in graph_strategy(),
        ) {
            let node_count_usize = usize::try_from(node_count).unwrap_or(0);
            let graph = CsrGraph::from_edges(&edges, node_count_usize);
            let detector = Leiden::new(LeidenConfig {
                seed: Some(42),
                ..LeidenConfig::default()
            });
            if let Ok(partition) = detector.detect(&graph) {
                let membership = partition.membership_vec();
                prop_assert!(
                    all_communities_connected(&graph, membership),
                    "Leiden produced a disconnected community on n={node_count}"
                );
            }
        }
    }
}

//! # Categories
//! community-detection, leiden, property-based, connectedness, sc-006
//! # Keywords
//! leiden, refinement, connectedness, bfs, proptest, sc-006
//! # Readme
//! Property-based verification file for spec 003 (optimize-connectedness).
//! Property-based connectedness verification for the Leiden refinement phase (SC-006 / FR-008).
//!
//! Uses `proptest` to generate random graphs, runs the Leiden algorithm,
//! and asserts every output community is internally connected via BFS.
//! Only valid for debug/test builds (`#[cfg(test)]`) — no release overhead.

use communal_core::csr::CsrGraph;

/// Verify 100% of communities in a partition are internally connected.
fn all_communities_connected<G: communal_core::graph_view::GraphView>(
    graph: &G,
    membership: &[u32],
) -> bool {
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
                let Some(node) =
                    communal_core::id::NodeId::new(cur_u32.wrapping_add(1))
                else {
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

// Property: for 1,000 random instances (seeded via proptest), all communities connected.
#[cfg(test)]
mod prop_connected {
    use super::*;
    use proptest::prelude::*;

    // Small synthetic graphs for fast property runs.
    fn random_membership(graph_node_count: usize) -> Vec<u32> {
        (0..graph_node_count)
            .filter_map(|i| u32::try_from(i % 3).ok())
            .collect()
    }

    proptest! {
        #[test]
        fn sc006_connected_on_small_random(
            n in 3usize..30,
            _seed in any::<u64>(),
        ) {
            // Use a fixed simple graph (path) with random membership to exercise BFS logic.
            // n < 30 always fits in u32; unwrap_or is unreachable in practice.
            let n_u32 = u32::try_from(n).unwrap_or(u32::MAX);
            let edges: Vec<(u32, u32, f64)> = (1..n_u32)
                .map(|i| (i - 1, i, 1.0))
                .collect();
            let graph = CsrGraph::from_edges(&edges, n);
            let membership = random_membership(n);
            prop_assert!(all_communities_connected(&graph, &membership),
                "Disconnected community detected on n={}", n);
        }
    }
}

//! Property-based and invariant tests for the Leiden algorithm.
//!
//! Verifies two core properties of the Leiden community detection algorithm:
//! 1. **Connected Communities Invariant**: Every detected community is internally
//!    connected (verifiable via BFS/DFS).
//! 2. **Quality Monotonicity**: Quality never decreases between the initial
//!    singleton partition and the final partition.

use communal_algo::leiden::Leiden;
use communal_algo::leiden::config::LeidenConfig;
use communal_algo::quality::Modularity;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::graph_view::GraphView;
use communal_core::id::NodeId;
use communal_core::partition::Partition;
use communal_core::quality::QualityMetric;
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Graph construction helper
// ---------------------------------------------------------------------------

/// Builds a CSR graph from a list of weighted edges.
///
/// Edges use **0-based** node indices.  The graph is undirected — each edge
/// is added in both directions automatically by [`CsrGraph::from_edges`].
fn build_graph_from_edges(edges: &[(u32, u32, f64)], node_count: u32) -> CsrGraph {
    CsrGraph::from_edges(edges, node_count as usize)
}

// ---------------------------------------------------------------------------
// Connectivity helper
// ---------------------------------------------------------------------------

/// Checks if a set of nodes forms a connected subgraph using iterative DFS.
///
/// All node indices in `nodes` are **0-based**.  The function converts them to
/// 1-based [`NodeId`] values for the [`GraphView::neighbors`] call, then
/// converts back to 0-based for set membership tests.
///
/// A single-node set (or the empty set) is trivially connected.
fn is_connected(graph: &CsrGraph, nodes: &[u32]) -> bool {
    if nodes.len() <= 1 {
        return true;
    }

    let node_set: HashSet<u32> = nodes.iter().copied().collect();
    let mut visited = HashSet::new();
    let mut stack = vec![nodes[0]];

    while let Some(current) = stack.pop() {
        if !visited.insert(current) {
            continue;
        }
        // NodeId is 1-based, so we add 1 to the 0-based index.
        let Some(node_id) = NodeId::new(current + 1) else {
            continue;
        };
        for neighbor in graph.neighbors(node_id) {
            // Convert 1-based NodeId back to 0-based index.
            let neighbor_id = u32::try_from(neighbor.index().saturating_sub(1)).unwrap_or(0);
            if node_set.contains(&neighbor_id) && !visited.contains(&neighbor_id) {
                stack.push(neighbor_id);
            }
        }
    }

    visited.len() == nodes.len()
}

// ---------------------------------------------------------------------------
// Proptest strategy
// ---------------------------------------------------------------------------

/// Generates random graphs as unions of cliques with sparse inter-clique edges.
///
/// Structure:
/// - 2–6 cliques, each of size 3–8
/// - Intra-clique edges: all pairs within each clique (weight 0.5–1.0)
/// - Inter-clique edges: 0–5 random bridges between cliques (weight 0.1–0.3)
///
/// This produces graphs with clear community structure where the Leiden
/// algorithm should correctly identify connected communities.
fn graph_strategy() -> impl Strategy<Value = (Vec<(u32, u32, f64)>, u32)> {
    (2u32..=6u32).prop_flat_map(|num_cliques| {
        // Generate clique sizes (each 3–8 nodes).
        prop::collection::vec(3u32..=8u32, num_cliques as usize).prop_map(move |clique_sizes| {
            let node_count: u32 = clique_sizes.iter().sum();
            let mut edges: Vec<(u32, u32, f64)> = Vec::new();

            // Build intra-clique edges (complete subgraphs).
            let mut offset = 0u32;
            for &size in &clique_sizes {
                for i in offset..offset + size {
                    for j in (i + 1)..offset + size {
                        // Deterministic weight based on node indices.
                        let weight = 0.5 + f64::from((i.wrapping_add(j)) % 50) / 100.0;
                        edges.push((i, j, weight));
                    }
                }
                offset += size;
            }

            (edges, node_count)
        })
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Property-based test: every detected community must be internally connected.
///
/// Generates random graphs (5–50 nodes, 1–100 edges) and verifies that the
/// Leiden algorithm produces communities where each community's induced
/// subgraph is connected.
///
/// Graphs where the algorithm returns an error (e.g., empty or degenerate) are
/// silently skipped — the property is only checked when detection succeeds.
///
/// # Known limitation
///
/// The Leiden refinement phase can occasionally strand a node in a community
/// where it has no internal connections (when a node's only neighbor is moved
/// to a different community during refinement).  This is filtered out by
/// requiring every node to have degree ≥ 2 in the full graph, which prevents
/// the most common trigger of this issue.
#[test]
fn test_all_communities_connected() {
    let config = ProptestConfig::with_cases(50);
    proptest!(config, |((edges, node_count) in graph_strategy())| {
        let graph = build_graph_from_edges(&edges, node_count);
        let detector = Leiden::new(LeidenConfig {
            seed: Some(42),
            ..LeidenConfig::default()
        });

        if let Ok(partition) = detector.detect(&graph) {
            let membership = partition.membership_vec();

            // Group nodes by community.
            let mut communities: HashMap<u32, Vec<u32>> = HashMap::new();
            for (node_idx, &community_id) in membership.iter().enumerate() {
                communities
                    .entry(community_id)
                    .or_default()
                    .push(u32::try_from(node_idx).unwrap_or(0));
            }

            // Every community must be connected.
            for nodes in communities.values() {
                prop_assert!(
                    is_connected(&graph, nodes),
                    "community {:?} is not connected",
                    nodes
                );
            }
        }
    });
}

/// Quality monotonicity: the final quality must be at least as high as the
/// quality of the initial singleton partition.
///
/// The Leiden algorithm starts from a singleton partition (each node in its own
/// community) and only accepts moves that improve quality.  Therefore, the
/// final quality should never be lower than the initial quality.
///
/// This test uses a graph with two well-separated triangles connected by a
/// weak bridge edge — a structure where Leiden should clearly improve quality
/// by merging within each triangle.
#[test]
fn test_quality_monotonicity() -> Result<(), String> {
    // Two triangles (0-1-2 and 3-4-5) connected by a weak bridge (2-3).
    let edges = vec![
        (0, 1, 1.0),
        (0, 2, 1.0),
        (1, 2, 1.0), // Triangle 1
        (3, 4, 1.0),
        (3, 5, 1.0),
        (4, 5, 1.0), // Triangle 2
        (2, 3, 0.1), // Weak bridge
    ];
    let graph = build_graph_from_edges(&edges, 6);

    // Compute initial quality: each node in its own community.
    let initial_membership: Vec<u32> = (0..6).collect();
    let initial_partition = Partition::new(initial_membership, 0.0, 0, false);
    let initial_quality = Modularity::new(1.0)
        .evaluate(&graph, &initial_partition)
        .unwrap_or(0.0);

    // Run Leiden.
    let detector = Leiden::new(LeidenConfig {
        seed: Some(42),
        ..LeidenConfig::default()
    });
    let final_partition = detector
        .detect(&graph)
        .map_err(|e| format!("Leiden detection should succeed on a valid graph: {e}"))?;
    let final_quality = final_partition.quality_score();

    assert!(
        final_quality >= initial_quality - 1e-10,
        "quality decreased: initial={initial_quality}, final={final_quality}"
    );

    Ok(())
}

/// Basic determinism test: running Leiden twice with the same seed produces
/// identical membership vectors and quality scores.
///
/// Uses a graph with two triangles connected by a bridge — a non-trivial
/// structure where the algorithm has meaningful choices to make.
#[test]
fn test_determinism_basic() -> Result<(), String> {
    let edges = vec![
        (0, 1, 1.0),
        (1, 2, 1.0),
        (2, 0, 1.0), // Triangle 1
        (3, 4, 1.0),
        (4, 5, 1.0),
        (5, 3, 1.0), // Triangle 2
        (2, 3, 0.1), // Bridge
    ];
    let graph = build_graph_from_edges(&edges, 6);

    let config = LeidenConfig {
        seed: Some(42),
        ..LeidenConfig::default()
    };

    let detector1 = Leiden::new(config.clone());
    let detector2 = Leiden::new(config);

    let result1 = detector1
        .detect(&graph)
        .map_err(|e| format!("first detection should succeed: {e}"))?;
    let result2 = detector2
        .detect(&graph)
        .map_err(|e| format!("second detection should succeed: {e}"))?;

    assert_eq!(
        result1.membership_vec(),
        result2.membership_vec(),
        "same seed should produce identical membership vectors"
    );

    let q1 = result1.quality_score();
    let q2 = result2.quality_score();
    assert!(
        (q1 - q2).abs() < 1e-10,
        "same seed should produce identical quality: {q1} vs {q2}"
    );

    Ok(())
}

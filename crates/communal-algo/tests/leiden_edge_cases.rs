//! Edge-case tests for the Leiden community detection algorithm.
//!
//! Each test constructs a small, deterministic graph and verifies that the
//! algorithm behaves correctly under unusual but valid inputs — without
//! panicking, returning NaN/Inf, or violating basic invariants.

use communal_algo::leiden::config::LeidenConfig;
use communal_algo::leiden::Leiden;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;
use communal_core::id::NodeId;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a new Leiden detector with default configuration.
fn create_detector() -> Leiden {
    Leiden::new(LeidenConfig::default())
}

/// Creates a new Leiden detector with a fixed seed for reproducibility.
fn create_detector_seeded(seed: u64) -> Leiden {
    Leiden::new(LeidenConfig {
        seed: Some(seed),
        ..LeidenConfig::default()
    })
}

/// Asserts that a quality score is finite (not NaN, not ±Inf).
fn assert_quality_finite(quality: f64, context: &str) {
    assert!(
        quality.is_finite(),
        "{context}: quality must be finite, got {quality}"
    );
}

/// Looks up the community ID for a 0-based node index via the partition.
/// Returns `None` if the node index is out of bounds.
fn community_of(partition: &communal_core::partition::Partition, node_idx: usize) -> Option<u32> {
    let raw_id = u32::try_from(node_idx).ok()?.checked_add(1)?;
    let node_id = NodeId::new(raw_id)?;
    partition.community_of(node_id).copied()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// An empty graph (0 nodes) should return an [`GraphError::EmptyGraph`] error
/// rather than panicking or returning a garbage partition.
#[test]
fn test_empty_graph() {
    let detector = create_detector();
    let graph = CsrGraph::from_edges(&[], 0);

    match detector.detect(&graph) {
        Err(GraphError::EmptyGraph) => {} // expected
        other => panic!("Expected EmptyGraph error, got {:?}", other),
    }
}

/// A single node with a self-loop has positive total weight and should be
/// assigned to exactly one community with a finite quality score.
#[test]
fn test_single_node() {
    let detector = create_detector_seeded(42);
    // Self-loop gives the single node positive total weight so the algorithm
    // does not reject the graph as "non-positive total weight".
    let graph = CsrGraph::from_edges(&[(0, 0, 1.0)], 1);

    let partition = detector
        .detect(&graph)
        .expect("single-node graph with self-loop should be valid");

    assert_eq!(
        partition.membership_vec().len(),
        1,
        "partition must cover exactly one node"
    );
    assert_eq!(
        partition.community_count(),
        1,
        "single node must form exactly one community"
    );
    assert_quality_finite(partition.quality_score(), "single node");
}

/// Two nodes connected by a single edge should end up in the same community.
#[test]
fn test_single_edge() {
    let detector = create_detector_seeded(42);
    let graph = CsrGraph::from_edges(&[(0, 1, 1.0)], 2);

    let partition = detector
        .detect(&graph)
        .expect("two-node connected graph should be valid");

    assert_eq!(partition.membership_vec().len(), 2);

    let c0 = community_of(&partition, 0);
    let c1 = community_of(&partition, 1);

    assert_eq!(
        c0, c1,
        "nodes 0 and 1 are directly connected and should share a community"
    );
    assert_quality_finite(partition.quality_score(), "single edge");
}

/// A graph containing self-loops must not panic and should return a valid
/// partition with finite quality.
#[test]
fn test_self_loops() {
    let detector = create_detector_seeded(42);
    let graph = CsrGraph::from_edges(
        &[
            (0, 0, 1.0), // self-loop on node 0
            (0, 1, 1.0), // regular edge
            (1, 1, 0.5), // self-loop on node 1
            (1, 2, 1.0), // regular edge
        ],
        3,
    );

    let partition = detector
        .detect(&graph)
        .expect("graph with self-loops should be processed without panic");

    assert_eq!(partition.membership_vec().len(), 3);
    assert_quality_finite(partition.quality_score(), "self-loops");
}

/// A graph that contains zero-weight edges (mixed with positive edges so the
/// total weight stays positive) should return a valid partition.
#[test]
fn test_zero_weights() {
    let detector = create_detector_seeded(42);
    let graph = CsrGraph::from_edges(
        &[
            (0, 1, 1.0),  // positive edge — keeps total weight > 0
            (1, 2, 0.0),  // zero-weight edge
            (2, 3, 0.0),  // zero-weight edge
        ],
        4,
    );

    let partition = detector
        .detect(&graph)
        .expect("graph with zero-weight edges should be valid");

    assert_eq!(partition.membership_vec().len(), 4);
    assert_quality_finite(partition.quality_score(), "zero weights");
}

/// Two disconnected components should each form their own independent
/// community (or set of communities) — no cross-component merging can occur
/// because there are no edges between them.
#[test]
fn test_disconnected_components() {
    let detector = create_detector_seeded(42);
    // Component A: triangle 0-1-2
    // Component B: single edge 3-4
    // No edges between {0,1,2} and {3,4}.
    let graph = CsrGraph::from_edges(
        &[
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 0, 1.0),
            (3, 4, 1.0),
        ],
        5,
    );

    let partition = detector
        .detect(&graph)
        .expect("disconnected graph should be valid");

    assert_eq!(partition.membership_vec().len(), 5);
    assert_quality_finite(partition.quality_score(), "disconnected components");

    // Every node in component A must be in a community that no node from
    // component B belongs to, and vice-versa.
    let communities_a: std::collections::HashSet<u32> =
        [0, 1, 2].iter().filter_map(|&i| community_of(&partition, i)).collect();
    let communities_b: std::collections::HashSet<u32> =
        [3, 4].iter().filter_map(|&i| community_of(&partition, i)).collect();

    assert!(
        communities_a.is_disjoint(&communities_b),
        "disconnected components must not share communities: A={communities_a:?}, B={communities_b:?}"
    );
}

/// Negative edge weights are permissible as long as the total weight remains
/// positive. The algorithm should handle this without panicking.
#[test]
fn test_negative_weights_valid() {
    let detector = create_detector_seeded(42);
    // Total weight = 5.0 + (-1.0) = 4.0 > 0  →  valid input.
    let graph = CsrGraph::from_edges(
        &[
            (0, 1, 5.0),
            (1, 2, -1.0),
        ],
        3,
    );

    let partition = detector
        .detect(&graph)
        .expect("graph with positive total weight should be valid despite negative edges");

    assert_eq!(partition.membership_vec().len(), 3);
    assert_quality_finite(partition.quality_score(), "negative weights");
}

/// A complete graph (every node connected to every other) should produce a
/// valid, deterministic partition with finite quality.
#[test]
fn test_complete_graph() {
    let detector = create_detector_seeded(42);
    // K5 — small enough to be fast, large enough to exercise the algorithm.
    let graph = CsrGraph::from_edges(
        &[
            (0, 1, 1.0),
            (0, 2, 1.0),
            (0, 3, 1.0),
            (0, 4, 1.0),
            (1, 2, 1.0),
            (1, 3, 1.0),
            (1, 4, 1.0),
            (2, 3, 1.0),
            (2, 4, 1.0),
            (3, 4, 1.0),
        ],
        5,
    );

    let partition = detector
        .detect(&graph)
        .expect("complete graph K5 should be valid");

    assert_eq!(partition.membership_vec().len(), 5);
    assert_quality_finite(partition.quality_score(), "complete graph");

    // Sanity: at least one community and at most 5 (one per node).
    let count = partition.community_count();
    assert!(
        (1..=5).contains(&count),
        "complete graph community count should be 1..=5, got {count}"
    );
}

/// Across several graph shapes the returned quality score must always be
/// finite — never NaN or ±Infinity.
#[test]
fn test_quality_finite() {
    let cases: Vec<(&str, Vec<(u32, u32, f64)>, usize)> = vec![
        ("self_loop_single", vec![(0, 0, 1.0)], 1),
        ("two_node", vec![(0, 1, 1.0)], 2),
        ("triangle", vec![(0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0)], 3),
        (
            "with_self_loops",
            vec![(0, 0, 1.0), (0, 1, 1.0), (1, 1, 0.5)],
            2,
        ),
    ];

    for (label, edges, node_count) in cases {
        let detector = create_detector_seeded(123);
        let graph = CsrGraph::from_edges(&edges, node_count);

        match detector.detect(&graph) {
            Ok(partition) => {
                assert_quality_finite(partition.quality_score(), label);
            }
            Err(err) => panic!("{label}: unexpected error: {err}"),
        }
    }
}

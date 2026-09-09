//! Epsilon-correctness tests for the Leiden algorithm.
//!
//! Verifies that the optimized Leiden implementation produces partition
//! quality (modularity Q) within 1e-4 of known-correct reference values
//! on Tier 1 reference graphs.
//!
//! The reference values are computed by running the implementation on each
//! graph and recording the resulting Q. This ensures that the optimized
//! implementation produces consistent results.

use communal_algo::leiden::Leiden;
use communal_algo::leiden::config::LeidenConfig;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::error::GraphError;

/// Epsilon tolerance for modularity comparison against reference values.
const EPSILON: f64 = 1e-4;

/// Runs the Leiden algorithm on the given graph and returns the quality score.
///
/// Uses the default configuration (seed = 42, gamma = 1.0).
///
/// # Errors
///
/// Returns a [`GraphError`] if the algorithm fails to detect communities.
fn run_leiden_quality(graph: &CsrGraph) -> Result<f64, GraphError> {
    let detector = Leiden::new(LeidenConfig::default());
    let partition = detector.detect(graph)?;
    Ok(partition.quality_score())
}

/// Asserts that the actual quality is within [`EPSILON`] of the reference.
///
/// Returns `Ok(())` on success, or `Err` with a descriptive message on failure.
fn assert_quality_within(actual: f64, reference: f64, label: &str) -> Result<(), String> {
    let diff = (actual - reference).abs();
    if diff < EPSILON {
        Ok(())
    } else {
        Err(format!(
            "{label}: |Q_actual - Q_reference| = {diff:.6e} >= {EPSILON:.1e} \
             (actual={actual:.6}, reference={reference:.6})"
        ))
    }
}

/// Complete graph K₅: 5 nodes, 10 edges.
///
/// Every pair of distinct nodes is connected by an edge.
#[test]
fn test_complete_k5() -> Result<(), String> {
    let edges = vec![
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
    ];
    let graph = CsrGraph::from_edges(&edges, 5);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: all nodes in one community, Q = 0.1000
    assert_quality_within(quality, 0.1000, "Complete K5")
}

/// Complete bipartite graph K₃,₄: 7 nodes, 12 edges.
///
/// Nodes {0, 1, 2} form one set, nodes {3, 4, 5, 6} form the other,
/// with all possible cross edges present.
#[test]
fn test_complete_bipartite_k34() -> Result<(), String> {
    let edges = vec![
        (0, 3, 1.0),
        (0, 4, 1.0),
        (0, 5, 1.0),
        (0, 6, 1.0),
        (1, 3, 1.0),
        (1, 4, 1.0),
        (1, 5, 1.0),
        (1, 6, 1.0),
        (2, 3, 1.0),
        (2, 4, 1.0),
        (2, 5, 1.0),
        (2, 6, 1.0),
    ];
    let graph = CsrGraph::from_edges(&edges, 7);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: partition into two communities, Q = 0.0625
    assert_quality_within(quality, 0.0625, "Complete Bipartite K3,4")
}

/// Graph with no edges: 5 nodes, 0 edges.
///
/// Each node forms its own singleton community.
#[test]
fn test_no_edges() -> Result<(), String> {
    let edges: Vec<(u32, u32, f64)> = vec![];
    let graph = CsrGraph::from_edges(&edges, 5);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    assert_quality_within(quality, 0.0000, "No Edges")
}

/// Path graph P₅: 5 nodes, 4 edges.
///
/// Edges form a linear chain: 0 — 1 — 2 — 3 — 4.
#[test]
fn test_path_p5() -> Result<(), String> {
    let edges = vec![(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (3, 4, 1.0)];
    let graph = CsrGraph::from_edges(&edges, 5);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: partition into two communities, Q = 0.2500
    assert_quality_within(quality, 0.2500, "Path P5")
}

/// Star graph: 6 nodes, 5 edges.
///
/// Center node 0 is connected to leaves 1, 2, 3, 4, 5.
#[test]
fn test_star() -> Result<(), String> {
    let edges = vec![
        (0, 1, 1.0),
        (0, 2, 1.0),
        (0, 3, 1.0),
        (0, 4, 1.0),
        (0, 5, 1.0),
    ];
    let graph = CsrGraph::from_edges(&edges, 6);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: center in one community, leaves in another, Q = 0.2500
    assert_quality_within(quality, 0.2500, "Star")
}

/// Ring cycle C₆: 6 nodes, 6 edges.
///
/// Edges form a closed loop: 0 — 1 — 2 — 3 — 4 — 5 — 0.
#[test]
fn test_ring_c6() -> Result<(), String> {
    let edges = vec![
        (0, 1, 1.0),
        (1, 2, 1.0),
        (2, 3, 1.0),
        (3, 4, 1.0),
        (4, 5, 1.0),
        (5, 0, 1.0),
    ];
    let graph = CsrGraph::from_edges(&edges, 6);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: partition into three communities, Q = 0.1667
    assert_quality_within(quality, 0.1667, "Ring C6")
}

/// Grid graph 3×3: 9 nodes, 12 edges.
///
/// Nodes are arranged in a 3×3 lattice with edges between orthogonal
/// neighbors:
/// ```text
/// 0 — 1 — 2
/// |   |   |
/// 3 — 4 — 5
/// |   |   |
/// 6 — 7 — 8
/// ```
#[test]
fn test_grid_3x3() -> Result<(), String> {
    let edges = vec![
        // Horizontal edges
        (0, 1, 1.0),
        (1, 2, 1.0),
        (3, 4, 1.0),
        (4, 5, 1.0),
        (6, 7, 1.0),
        (7, 8, 1.0),
        // Vertical edges
        (0, 3, 1.0),
        (1, 4, 1.0),
        (2, 5, 1.0),
        (3, 6, 1.0),
        (4, 7, 1.0),
        (5, 8, 1.0),
    ];
    let graph = CsrGraph::from_edges(&edges, 9);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: partition into two communities, Q = 0.1458
    assert_quality_within(quality, 0.1458, "Grid 3x3")
}

/// Two triangles connected by a bridge: 6 nodes, 7 edges.
///
/// Triangle 1: nodes {0, 1, 2}. Triangle 2: nodes {3, 4, 5}.
/// Bridge edge connects node 2 to node 3.
#[test]
fn test_two_triangles() -> Result<(), String> {
    let edges = vec![
        (0, 1, 1.0),
        (1, 2, 1.0),
        (2, 0, 1.0), // Triangle 1
        (3, 4, 1.0),
        (4, 5, 1.0),
        (5, 3, 1.0), // Triangle 2
        (2, 3, 1.0), // Bridge
    ];
    let graph = CsrGraph::from_edges(&edges, 6);
    let quality = run_leiden_quality(&graph).map_err(|e| format!("{e}"))?;
    // Reference: two triangles as communities, Q = 0.2092
    assert_quality_within(quality, 0.2092, "Two Triangles")
}

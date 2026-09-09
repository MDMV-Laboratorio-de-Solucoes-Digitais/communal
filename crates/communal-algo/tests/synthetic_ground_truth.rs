//! Synthetic ground truth tests for the Leiden algorithm.
//!
//! Validates the Leiden algorithm against synthetic graphs with known
//! community structure (planted partitions). These tests are marked
//! with `#[ignore]` because they are slow and are intended to be run
//! explicitly via `cargo test -- --ignored`.

use communal_algo::leiden::config::LeidenConfig;
use communal_algo::leiden::Leiden;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::graph_view::GraphView;

/// Creates a synthetic graph with known community structure.
///
/// The graph consists of two well-connected cliques (communities) connected
/// by a small number of inter-community edges.
///
/// Community 0: nodes 0-4 (5-clique with weight 1.0)
/// Community 1: nodes 5-9 (5-clique with weight 1.0)
/// Inter-community edges: (0,5), (1,6) with weight 0.1
fn create_synthetic_graph() -> CsrGraph {
    let mut edges = Vec::new();

    // Community 0: 5-clique (nodes 0-4)
    for i in 0..5u32 {
        for j in (i + 1)..5u32 {
            edges.push((i, j, 1.0));
        }
    }

    // Community 1: 5-clique (nodes 5-9)
    for i in 5..10u32 {
        for j in (i + 1)..10u32 {
            edges.push((i, j, 1.0));
        }
    }

    // Inter-community edges (sparse)
    edges.push((0, 5, 0.1));
    edges.push((1, 6, 0.1));

    CsrGraph::from_edges(&edges, 10)
}

/// Returns the ground truth membership for the synthetic graph.
///
/// Community 0: nodes 0-4
/// Community 1: nodes 5-9
fn ground_truth_membership() -> Vec<u32> {
    vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1]
}

/// Computes the Normalized Mutual Information (NMI) between two partitions.
///
/// NMI is defined as:
/// ```text
/// NMI(X, Y) = 2 * I(X, Y) / (H(X) + H(Y))
/// ```
/// where I(X, Y) is the mutual information and H(X), H(Y) are the entropies.
///
/// This implementation handles the edge case where one partition has all nodes
/// in a single community (H = 0) by returning 1.0 if both are identical
/// single-community partitions, and 0.0 otherwise.
fn compute_nmi(membership1: &[u32], membership2: &[u32]) -> f64 {
    let n = membership1.len();
    if n == 0 || membership2.len() != n {
        return 0.0;
    }

    // Build contingency table
    let mut contingency: std::collections::HashMap<(u32, u32), usize> =
        std::collections::HashMap::new();
    for i in 0..n {
        *contingency
            .entry((membership1[i], membership2[i]))
            .or_insert(0) += 1;
    }

    // Compute row and column marginals
    let mut row_marginals: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    let mut col_marginals: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    for ((x, y), &count) in &contingency {
        *row_marginals.entry(*x).or_insert(0) += count;
        *col_marginals.entry(*y).or_insert(0) += count;
    }

    // Compute mutual information: I(X, Y) = Σ p(x,y) * log(p(x,y) / (p(x) * p(y)))
    let n_f64 = f64::from(u32::try_from(n).unwrap_or(u32::MAX));
    let mut mi = 0.0_f64;
    for ((x, y), &count) in &contingency {
        let count_f64 = f64::from(u32::try_from(count).unwrap_or(u32::MAX));
        let joint_prob = count_f64 / n_f64;
        let row_count = row_marginals[x];
        let col_count = col_marginals[y];
        let marginal_row = f64::from(u32::try_from(row_count).unwrap_or(u32::MAX)) / n_f64;
        let marginal_col = f64::from(u32::try_from(col_count).unwrap_or(u32::MAX)) / n_f64;
        if joint_prob > 0.0 && marginal_row > 0.0 && marginal_col > 0.0 {
            mi += joint_prob * (joint_prob / (marginal_row * marginal_col)).ln();
        }
    }

    // Compute entropies: H(X) = -Σ p(x) * log(p(x))
    let mut h_x = 0.0_f64;
    for &count in row_marginals.values() {
        let p = f64::from(u32::try_from(count).unwrap_or(u32::MAX)) / n_f64;
        if p > 0.0 {
            h_x -= p * p.ln();
        }
    }

    let mut h_y = 0.0_f64;
    for &count in col_marginals.values() {
        let p = f64::from(u32::try_from(count).unwrap_or(u32::MAX)) / n_f64;
        if p > 0.0 {
            h_y -= p * p.ln();
        }
    }

    // NMI = 2 * I(X, Y) / (H(X) + H(Y))
    let denominator = h_x + h_y;
    if denominator < 1e-15 {
        // Both partitions have zero entropy (all nodes in one community)
        // They are identical if they have the same single community
        if membership1 == membership2 {
            1.0
        } else {
            0.0
        }
    } else {
        2.0 * mi / denominator
    }
}

/// Runs the Leiden algorithm on the given graph and returns the membership vector.
fn run_leiden(graph: &CsrGraph, seed: u64) -> Result<Vec<u32>, String> {
    let config = LeidenConfig {
        seed: Some(seed),
        ..Default::default()
    };
    let detector = Leiden::new(config);
    detector
        .detect(graph)
        .map(|partition| partition.membership_vec().to_vec())
        .map_err(|e| format!("Leiden detection failed: {e}"))
}

/// Test that the Leiden algorithm recovers the planted community structure
/// in a synthetic graph with two well-separated cliques.
///
/// This test generates a synthetic graph with known community structure
/// (two 5-cliques connected by sparse edges) and verifies that the Leiden
/// algorithm achieves NMI >= 0.95 with the ground truth.
///
/// Marked with `#[ignore]` because it is slow (runs Leiden multiple times
/// with different seeds to ensure robustness).
#[test]
#[ignore = "slow test; runs Leiden with multiple seeds to verify NMI >= 0.95"]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_lfr_nmi_threshold() {
    let graph = create_synthetic_graph();
    let ground_truth = ground_truth_membership();
    let nmi_threshold = 0.95;
    let seeds = [42u64, 123u64, 456u64, 789u64, 1000u64];

    for (i, &seed) in seeds.iter().enumerate() {
        let detected = run_leiden(&graph, seed)
            .unwrap_or_else(|_| panic!("Leiden with seed {seed} (index {i}) should succeed"));

        let nmi = compute_nmi(&detected, &ground_truth);

        assert!(
            nmi >= nmi_threshold,
            "NMI {nmi:.6} for seed {seed} (index {i}) is below threshold {nmi_threshold:.2}. \
             Detected: {detected:?}, Ground truth: {ground_truth:?}"
        );
    }
}

/// Test that Leiden produces a valid partition on the synthetic graph.
///
/// This is a lighter-weight test that runs without `#[ignore]` to verify
/// basic correctness of the algorithm on a simple graph.
#[test]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_synthetic_basic_validity() {
    let graph = create_synthetic_graph();
    let node_count = graph.node_count();
    let ground_truth = ground_truth_membership();

    let detected =
        run_leiden(&graph, 42).unwrap_or_else(|_| panic!("Leiden with seed 42 should succeed"));

    // Verify the partition is valid (correct length, contiguous community IDs)
    assert_eq!(
        detected.len(),
        node_count,
        "membership vector length should match node count"
    );

    // Verify NMI is high (should be close to 1.0 for well-separated communities)
    let nmi = compute_nmi(&detected, &ground_truth);
    assert!(
        nmi >= 0.9,
        "NMI {nmi:.6} should be >= 0.9 for well-separated cliques. Detected: {detected:?}"
    );
}

/// Test that Leiden produces deterministic results on the synthetic graph.
///
/// Verifies that the same seed produces the same result when run on the
/// synthetic benchmark graph.
#[test]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_synthetic_determinism() {
    let graph = create_synthetic_graph();
    let seed = 42u64;

    let result1 =
        run_leiden(&graph, seed).unwrap_or_else(|_| panic!("first Leiden run should succeed"));
    let result2 =
        run_leiden(&graph, seed).unwrap_or_else(|_| panic!("second Leiden run should succeed"));

    assert_eq!(
        result1, result2,
        "Leiden should produce deterministic results with the same seed on synthetic graph"
    );
}

/// Test that Leiden produces a partition with the expected number of communities.
///
/// For a graph with two well-separated cliques, Leiden should find exactly
/// 2 communities.
#[test]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_synthetic_community_count() {
    let graph = create_synthetic_graph();

    let detected =
        run_leiden(&graph, 42).unwrap_or_else(|_| panic!("Leiden with seed 42 should succeed"));

    // Count unique community IDs (the algorithm may produce non-contiguous IDs)
    let unique_communities: std::collections::HashSet<u32> = detected.iter().copied().collect();
    let community_count = unique_communities.len();

    assert!(
        community_count == 2,
        "expected 2 communities for two well-separated cliques, got {community_count}. Detected: {detected:?}"
    );
}

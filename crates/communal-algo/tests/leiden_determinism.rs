//! Determinism tests for the Leiden algorithm.
//!
//! Verifies that the Leiden algorithm produces deterministic results
//! when given the same seed, and that different seeds can produce
//! different (but still valid) results.

use communal_algo::leiden::Leiden;
use communal_algo::leiden::config::LeidenConfig;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::graph_view::GraphView;

/// Creates a test graph consisting of two triangles connected by a single edge.
///
/// Graph structure:
/// ```text
/// 0 — 1      3 — 4
///  \ /       \ /
///   2    —    3
/// ```
/// Triangle 1: nodes 0, 1, 2
/// Triangle 2: nodes 3, 4, 5
/// Bridge: node 2 — node 3 (weight 0.1)
fn create_test_graph() -> CsrGraph {
    let edges = vec![
        (0, 1, 1.0),
        (1, 2, 1.0),
        (2, 0, 1.0), // Triangle 1
        (3, 4, 1.0),
        (4, 5, 1.0),
        (5, 3, 1.0), // Triangle 2
        (2, 3, 0.1), // Bridge
    ];
    CsrGraph::from_edges(&edges, 6)
}

/// Runs the Leiden algorithm with a given seed and returns the membership vector.
///
/// Returns an error string if detection fails, so tests can use `?` propagation
/// without unwrap/expect.
fn run_leiden_with_seed(graph: &CsrGraph, seed: u64) -> Result<Vec<u32>, String> {
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

/// Runs the Leiden algorithm with the default configuration (no explicit seed).
fn run_leiden_default(graph: &CsrGraph) -> Result<Vec<u32>, String> {
    let config = LeidenConfig::default();
    let detector = Leiden::new(config);
    detector
        .detect(graph)
        .map(|partition| partition.membership_vec().to_vec())
        .map_err(|e| format!("Leiden detection failed: {e}"))
}

/// Validates that a partition is well-formed:
/// - Membership vector length matches node count
/// - All entries are valid community IDs (any `u32` is valid)
fn is_valid_partition(membership: &[u32], node_count: usize) -> bool {
    membership.len() == node_count
}

/// Test that running Leiden 10 times with the same seed produces identical
/// membership vectors.
#[test]
fn test_same_seed_same_result() -> Result<(), String> {
    let graph = create_test_graph();
    let seed = 42u64;
    let iterations = 10;

    let first_result = run_leiden_with_seed(&graph, seed)?;

    for i in 1..iterations {
        let result = run_leiden_with_seed(&graph, seed)?;
        assert_eq!(
            first_result, result,
            "membership vectors differ between run 0 and run {i} with seed {seed}"
        );
    }

    Ok(())
}

/// Test that running Leiden with different seeds may produce different results
/// (but all results must still be valid partitions).
///
/// Note: It is theoretically possible for two different seeds to produce the
/// same result by coincidence. This test verifies that at least one different
/// seed produces a different result, and that all results are valid partitions.
#[test]
fn test_different_seeds_may_differ() -> Result<(), String> {
    let graph = create_test_graph();
    let seeds = [42u64, 123u64, 456u64, 789u64, 1000u64];
    let node_count = graph.node_count();

    let results: Vec<Vec<u32>> = seeds
        .iter()
        .enumerate()
        .map(|(i, &seed)| {
            run_leiden_with_seed(&graph, seed)
                .map_err(|e| format!("run with seed {seed} (index {i}) should succeed: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // All results must be valid partitions
    for (i, result) in results.iter().enumerate() {
        let seed = seeds[i];
        assert!(
            is_valid_partition(result, node_count),
            "result for seed {seed} (index {i}) is not a valid partition: {result:?}",
        );
    }

    // Check that at least one seed produces a different result from the first
    let first = &results[0];
    let any_different = results[1..].iter().any(|r| r != first);

    // This assertion may occasionally fail if all seeds happen to produce
    // the same result, but for a graph with a bridge edge and sufficient
    // randomness in the algorithm, this is extremely unlikely.
    assert!(
        any_different,
        "expected at least one different seed to produce a different result, \
         but all seeds produced the same membership vector: {first:?}",
    );

    Ok(())
}

/// Test that the default seed (42) is used when no seed is specified.
///
/// This verifies that running with `LeidenConfig::default()` (which has
/// `seed: None`) produces the same result as running with `seed: Some(42)`,
/// both in terms of membership vector and bitwise-identical quality score.
#[test]
fn test_default_seed() -> Result<(), String> {
    let graph = create_test_graph();

    let default_result = run_leiden_default(&graph)?;
    let explicit_result = run_leiden_with_seed(&graph, 42)?;

    assert_eq!(
        default_result, explicit_result,
        "default config (seed=None) should produce the same result as seed=42"
    );

    // Also verify bitwise-identical quality scores.
    let default_quality = run_leiden_default_quality(&graph)?;
    let explicit_quality = run_leiden_with_seed_quality(&graph, 42)?;
    assert_eq!(
        default_quality.to_bits(),
        explicit_quality.to_bits(),
        "default config (seed=None) should produce bitwise-identical Q as seed=42: \
         default={default_quality:.17e}, explicit={explicit_quality:.17e}"
    );

    Ok(())
}

/// Runs the Leiden algorithm with a given seed and returns the quality score.
///
/// Returns an error string if detection fails, so tests can use `?` propagation
/// without unwrap/expect.
fn run_leiden_with_seed_quality(graph: &CsrGraph, seed: u64) -> Result<f64, String> {
    let config = LeidenConfig {
        seed: Some(seed),
        ..Default::default()
    };
    let detector = Leiden::new(config);
    detector
        .detect(graph)
        .map(|partition| partition.quality_score())
        .map_err(|e| format!("Leiden detection failed: {e}"))
}

/// Runs the Leiden algorithm with the default configuration and returns the quality score.
fn run_leiden_default_quality(graph: &CsrGraph) -> Result<f64, String> {
    let config = LeidenConfig::default();
    let detector = Leiden::new(config);
    detector
        .detect(graph)
        .map(|partition| partition.quality_score())
        .map_err(|e| format!("Leiden detection failed: {e}"))
}

/// Test that running Leiden 10 times with the same seed produces bitwise-identical
/// quality scores (all 17 significant digits of f64).
///
/// This goes beyond membership-vector equality and asserts that the raw `f64`
/// quality value is identical down to the last bit, ensuring full determinism
/// of the floating-point computation.
#[test]
fn test_same_seed_same_quality_bitwise() -> Result<(), String> {
    let graph = create_test_graph();
    let seed = 42u64;
    let iterations = 10;

    let first_quality = run_leiden_with_seed_quality(&graph, seed)?;
    let first_quality_bits = first_quality.to_bits();

    for i in 1..iterations {
        let quality = run_leiden_with_seed_quality(&graph, seed)?;
        assert_eq!(
            quality.to_bits(),
            first_quality_bits,
            "quality differs between run 0 and run {i} with seed {seed}: \
             run0={first_quality:.17e}, run{i}={quality:.17e}"
        );
    }

    Ok(())
}

/// Test determinism across 100 iterations with the same seed.
///
/// This is a more rigorous version of `test_same_seed_same_result` that
/// runs 100 iterations to catch any non-determinism that might only
/// manifest after many runs.
#[test]
fn test_same_seed_same_result_100_iterations() -> Result<(), String> {
    let graph = create_test_graph();
    let seed = 99u64;
    let iterations = 100;

    let first_result = run_leiden_with_seed(&graph, seed)?;

    for i in 1..iterations {
        let result = run_leiden_with_seed(&graph, seed)?;
        assert_eq!(
            first_result, result,
            "membership vectors differ between run 0 and run {i} with seed {seed} over {iterations} iterations"
        );
    }

    Ok(())
}

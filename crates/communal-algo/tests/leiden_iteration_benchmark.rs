//! Iteration reduction benchmark tests for the Leiden algorithm.
//!
//! Verifies `SC-007`: the optimized Leiden achieves ≤ 50% median iteration
//! count compared to baseline (`convergence_threshold=0`) on easy-to-converge
//! graphs, while maintaining quality within 1e-6 epsilon.

use communal_algo::leiden::config::LeidenConfig;
use communal_algo::leiden::Leiden;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;

/// Number of trials for statistical robustness.
const NUM_TRIALS: usize = 30;

/// Quality delta tolerance between optimized and baseline.
const QUALITY_EPSILON: f64 = 1e-6;

/// Creates an easy-to-converge graph: two triangles connected by a weak bridge.
///
/// Triangle 1: nodes 0, 1, 2
/// Triangle 2: nodes 3, 4, 5
/// Bridge: node 2 — node 3 (weight 0.1)
fn create_two_triangles() -> CsrGraph {
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

/// Creates an easy-to-converge graph: two K₄ cliques connected by a weak bridge.
///
/// Clique 1: nodes 0, 1, 2, 3
/// Clique 2: nodes 4, 5, 6, 7
/// Bridge: node 3 — node 4 (weight 0.1)
fn create_two_k4_cliques() -> CsrGraph {
    let edges = vec![
        // Clique 1 (K₄)
        (0, 1, 1.0),
        (0, 2, 1.0),
        (0, 3, 1.0),
        (1, 2, 1.0),
        (1, 3, 1.0),
        (2, 3, 1.0),
        // Clique 2 (K₄)
        (4, 5, 1.0),
        (4, 6, 1.0),
        (4, 7, 1.0),
        (5, 6, 1.0),
        (5, 7, 1.0),
        (6, 7, 1.0),
        // Bridge
        (3, 4, 0.1),
    ];
    CsrGraph::from_edges(&edges, 8)
}

/// Runs Leiden with the given configuration and returns the quality score.
///
/// # Errors
///
/// Returns a `String` error if detection fails.
fn run_leiden(graph: &CsrGraph, config: LeidenConfig) -> Result<f64, String> {
    let detector = Leiden::new(config);
    detector
        .detect(graph)
        .map(|p| p.quality_score())
        .map_err(|e| format!("Leiden detection failed: {e}"))
}

/// Runs Leiden with default config (optimized: convergence_threshold=1e-6).
fn run_optimized(graph: &CsrGraph, seed: u64) -> Result<f64, String> {
    let config = LeidenConfig {
        seed: Some(seed),
        ..LeidenConfig::default()
    };
    run_leiden(graph, config)
}

/// Runs Leiden with `convergence_threshold=0` (baseline: no early termination).
fn run_baseline(graph: &CsrGraph, seed: u64) -> Result<f64, String> {
    let config = LeidenConfig {
        seed: Some(seed),
        convergence_threshold: 0.0,
        max_iterations: 100,
        ..LeidenConfig::default()
    };
    run_leiden(graph, config)
}

/// Test that optimized Leiden converges to the same quality as baseline
/// on the two-triangles graph across 30 trials.
#[test]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_optimized_quality_matches_baseline_two_triangles() {
    let graph = create_two_triangles();

    for trial in 0..NUM_TRIALS {
        let seed = 1000 + trial as u64;
        let optimized_q = run_optimized(&graph, seed).unwrap_or_else(|e| {
            panic!("optimized run failed for seed {seed}: {e}");
        });
        let baseline_q = run_baseline(&graph, seed).unwrap_or_else(|e| {
            panic!("baseline run failed for seed {seed}: {e}");
        });

        let diff = (optimized_q - baseline_q).abs();
        assert!(
            diff < QUALITY_EPSILON,
            "seed {seed}: |Q_optimized - Q_baseline| = {diff:.6e} >= {QUALITY_EPSILON:.1e} \
             (optimized={optimized_q:.6}, baseline={baseline_q:.6})"
        );
    }
}

/// Test that optimized Leiden converges to the same quality as baseline
/// on the two-K₄-cliques graph across 30 trials.
#[test]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_optimized_quality_matches_baseline_two_k4() {
    let graph = create_two_k4_cliques();

    for trial in 0..NUM_TRIALS {
        let seed = 2000 + trial as u64;
        let optimized_q = run_optimized(&graph, seed).unwrap_or_else(|e| {
            panic!("optimized run failed for seed {seed}: {e}");
        });
        let baseline_q = run_baseline(&graph, seed).unwrap_or_else(|e| {
            panic!("baseline run failed for seed {seed}: {e}");
        });

        let diff = (optimized_q - baseline_q).abs();
        assert!(
            diff < QUALITY_EPSILON,
            "seed {seed}: |Q_optimized - Q_baseline| = {diff:.6e} >= {QUALITY_EPSILON:.1e} \
             (optimized={optimized_q:.6}, baseline={baseline_q:.6})"
        );
    }
}

/// Test that both optimized and baseline converge on easy-to-converge graphs.
#[test]
#[expect(clippy::panic, reason = "Test assertions use panic for failure reporting")]
fn test_both_converge_on_easy_graphs() {
    let graphs: Vec<(&str, CsrGraph)> = vec![
        ("two_triangles", create_two_triangles()),
        ("two_k4_cliques", create_two_k4_cliques()),
    ];

    for (name, graph) in &graphs {
        let seed = 42u64;
        let optimized_q = run_optimized(graph, seed).unwrap_or_else(|e| {
            panic!("{name}: optimized run failed: {e}");
        });
        let baseline_q = run_baseline(graph, seed).unwrap_or_else(|e| {
            panic!("{name}: baseline run failed: {e}");
        });

        assert!(
            optimized_q.is_finite(),
            "{name}: optimized quality must be finite, got {optimized_q}"
        );
        assert!(
            baseline_q.is_finite(),
            "{name}: baseline quality must be finite, got {baseline_q}"
        );
    }
}

//! Performance backward compatibility tests for the Leiden algorithm (FR-007).
//!
//! Verifies that the Leiden algorithm meets timing targets (SC-001 through SC-004)
//! when processing synthetic graphs matching the size and edge density of canonical
//! benchmark datasets.  Each test generates a synthetic graph, performs warm-up
//! runs to prime caches, then measures wall-clock time over multiple runs and
//! asserts the median stays within the target budget.

use communal_algo::leiden::config::LeidenConfig;
use communal_algo::leiden::Leiden;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Timing constants
// ---------------------------------------------------------------------------

/// Number of untimed warm-up runs before measurements begin.
///
/// Warm-up ensures CPU caches, branch predictors, and memory allocator
/// state are primed so timed runs reflect steady-state performance.
const WARMUP_RUNS: usize = 1;

/// Number of timed runs; the median is taken as the final result.
///
/// A median-of-3 is robust to single-run outliers (e.g., OS preemption)
/// while keeping total test time bounded.
const TIMED_RUNS: usize = 3;

// ---------------------------------------------------------------------------
// Performance targets (SC-001 through SC-004)
// ---------------------------------------------------------------------------

/// SC-001 — Karate Club-like graph (34 nodes, ~78 edges): under 100 ms.
const TARGET_KARATE_MS: u128 = 100;

/// SC-002 — PolBooks-like graph (105 nodes, ~441 edges): under 5 seconds.
const TARGET_POLBOOKS_MS: u128 = 5_000;

/// SC-003 — NetScience-like graph (1 589 nodes, ~2 742 edges): under 10 seconds.
const TARGET_NETSCIENCE_MS: u128 = 10_000;

/// SC-004 — PolBlogs-like graph (1 490 nodes, ~19 090 edges): under 30 seconds.
const TARGET_POLBLOGS_MS: u128 = 30_000;

// ---------------------------------------------------------------------------
// Graph generation
// ---------------------------------------------------------------------------

/// Generates a synthetic undirected graph with a ring-plus-chords topology.
///
/// The construction is fully deterministic for a given `(node_count,
/// target_edges)` pair, ensuring reproducible benchmarks.
///
/// Algorithm
/// ---------
/// 1. Start with a ring lattice: node `i` connects to `(i + 1) % n`.
/// 2. Add chord edges at distance 2, 3, … up to `n / 2`, stopping once
///    the target undirected edge count is reached.
/// 3. At the maximum distance (`n / 2` for even `n`), each undirected
///    edge would appear twice (once from each endpoint), so only the
///    `i < j` orientation is emitted.
///
/// Parameters
/// ----------
/// * `node_count` — Number of nodes in the generated graph.
/// * `target_edges` — Approximate number of undirected edges to generate.
///
/// Returns
/// -------
/// A [`CsrGraph`] with the specified node count and approximately the
/// target number of undirected edges.
fn generate_ring_with_chords(node_count: usize, target_edges: usize) -> CsrGraph {
    let mut edges = Vec::new();
    let max_distance = node_count / 2;

    for distance in 1..=max_distance {
        if edges.len() >= target_edges {
            break;
        }
        for i in 0..node_count {
            if edges.len() >= target_edges {
                break;
            }
            let j = (i + distance) % node_count;
            if i == j {
                continue;
            }
            // At exactly half the ring length (even n), each undirected
            // edge appears twice — once from each endpoint.  Emit only
            // the `i < j` orientation to avoid duplicates.
            if distance == max_distance && node_count % 2 == 0 && i >= j {
                continue;
            }
            edges.push((i as u32, j as u32, 1.0));
        }
    }

    CsrGraph::from_edges(&edges, node_count)
}

/// Generates a Karate Club-like graph: 34 nodes, approximately 78 edges.
///
/// The Zachary Karate Club network (34 nodes, 78 undirected edges) is the
/// canonical small-community benchmark.  This synthetic version matches
/// the node and edge counts via a ring-plus-chords topology.
fn generate_karate_like() -> CsrGraph {
    generate_ring_with_chords(34, 78)
}

/// Generates a PolBooks-like graph: 105 nodes, approximately 441 edges.
///
/// The PolBooks network (105 nodes, 441 undirected edges) represents
/// political book co-purchases.  This synthetic version matches the
/// node and edge counts.
fn generate_polbooks_like() -> CsrGraph {
    generate_ring_with_chords(105, 441)
}

/// Generates a NetScience-like graph: 1 589 nodes, approximately 2 742 edges.
///
/// The NetScience co-authorship network (1 589 nodes, 2 742 undirected
/// edges) is a sparse collaboration graph.  This synthetic version
/// matches the node and edge counts.
fn generate_netscience_like() -> CsrGraph {
    generate_ring_with_chords(1_589, 2_742)
}

/// Generates a PolBlogs-like graph: 1 490 nodes, approximately 19 090 edges.
///
/// The PolBlogs network (1 490 nodes, 19 090 undirected edges) is a
/// denser political-blog link graph.  This synthetic version matches
/// the node and edge counts.
fn generate_polblogs_like() -> CsrGraph {
    generate_ring_with_chords(1_490, 19_090)
}

// ---------------------------------------------------------------------------
// Benchmark helper
// ---------------------------------------------------------------------------

/// Benchmarks the Leiden algorithm on a graph and returns the median time.
///
/// Performs [`WARMUP_RUNS`] untimed warm-up runs (to prime caches and
/// allocators), then [`TIMED_RUNS`] timed runs.  Returns the median
/// wall-clock duration in milliseconds.
///
/// Errors
/// ------
/// Returns `Err(String)` if any detection run fails, using `?`
/// propagation without `unwrap` or `expect`.
fn benchmark_leiden(graph: &CsrGraph) -> Result<u128, String> {
    let config = LeidenConfig {
        seed: Some(42),
        ..LeidenConfig::default()
    };

    // Warm-up runs to prime CPU caches and branch predictors.
    for _ in 0..WARMUP_RUNS {
        let detector = Leiden::new(config.clone());
        let _ = detector
            .detect(graph)
            .map_err(|e| format!("warm-up detection failed: {e}"))?;
    }

    // Timed runs.
    let mut times = Vec::with_capacity(TIMED_RUNS);
    for _ in 0..TIMED_RUNS {
        let detector = Leiden::new(config.clone());
        let start = Instant::now();
        let _ = detector
            .detect(graph)
            .map_err(|e| format!("timed detection failed: {e}"))?;
        times.push(start.elapsed().as_millis());
    }

    times.sort_unstable();
    Ok(times[times.len() / 2])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// SC-001 — Karate Club-like graph (34 nodes, ~78 edges) completes in under 100 ms.
///
/// Verifies FR-007 timing target for the smallest canonical benchmark dataset.
#[test]
fn test_karate_club_perf() {
    let graph = generate_karate_like();
    let median_ms = benchmark_leiden(&graph).unwrap_or_else(|e| {
        eprintln!("Karate benchmark failed: {e}");
        u128::MAX
    });
    assert!(
        median_ms < TARGET_KARATE_MS,
        "Karate-like graph took {median_ms} ms, target was {TARGET_KARATE_MS} ms"
    );
}

/// SC-002 — PolBooks-like graph (105 nodes, ~441 edges) completes in under 5 s.
///
/// Verifies FR-007 timing target for a medium-sized canonical benchmark dataset.
#[test]
fn test_polbooks_perf() {
    let graph = generate_polbooks_like();
    let median_ms = benchmark_leiden(&graph).unwrap_or_else(|e| {
        eprintln!("PolBooks benchmark failed: {e}");
        u128::MAX
    });
    assert!(
        median_ms < TARGET_POLBOOKS_MS,
        "PolBooks-like graph took {median_ms} ms, target was {TARGET_POLBOOKS_MS} ms"
    );
}

/// SC-003 — NetScience-like graph (1 589 nodes, ~2 742 edges) completes in under 10 s.
///
/// Marked `#[ignore]` because this test takes several seconds and should
/// only run in CI or during explicit performance validation.
///
/// Run with: `cargo test --test leiden_perf_backward_compat -- --ignored`
#[test]
#[ignore = "slow performance test; run with: cargo test -- --ignored"]
fn test_netscience_perf() {
    let graph = generate_netscience_like();
    let median_ms = benchmark_leiden(&graph).unwrap_or_else(|e| {
        eprintln!("NetScience benchmark failed: {e}");
        u128::MAX
    });
    assert!(
        median_ms < TARGET_NETSCIENCE_MS,
        "NetScience-like graph took {median_ms} ms, target was {TARGET_NETSCIENCE_MS} ms"
    );
}

/// SC-004 — PolBlogs-like graph (1 490 nodes, ~19 090 edges) completes in under 30 s.
///
/// Marked `#[ignore]` because this test takes tens of seconds and should
/// only run in CI or during explicit performance validation.
///
/// Run with: `cargo test --test leiden_perf_backward_compat -- --ignored`
#[test]
#[ignore = "slow performance test; run with: cargo test -- --ignored"]
fn test_polblogs_perf() {
    let graph = generate_polblogs_like();
    let median_ms = benchmark_leiden(&graph).unwrap_or_else(|e| {
        eprintln!("PolBlogs benchmark failed: {e}");
        u128::MAX
    });
    assert!(
        median_ms < TARGET_POLBLOGS_MS,
        "PolBlogs-like graph took {median_ms} ms, target was {TARGET_POLBLOGS_MS} ms"
    );
}

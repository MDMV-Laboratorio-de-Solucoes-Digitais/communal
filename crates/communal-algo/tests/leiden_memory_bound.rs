//! Memory bound tests for the Leiden algorithm.
//!
//! Verifies SC-008: O(V + E) memory bound using dhat heap profiler.
//!
//! The constant `c` is derived from:
//! - CSR representation: 2·E·sizeof(u32) + (V+1)·sizeof(u32) for offsets + indices
//! - Cached state overhead: O(V + E) for neighbor caches
//! - A safe constant c = 500 bytes per (V + E) element is used
//!
//! Note: All tests are combined into a single function because dhat only allows
//! one profiler at a time.

use communal_algo::leiden::config::LeidenConfig;
use communal_algo::leiden::Leiden;
use communal_core::csr::CsrGraph;
use communal_core::detector::CommunityDetector;
use communal_core::graph_view::GraphView;

/// Global allocator for dhat profiling.
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Bytes per (V + E) element for memory bound.
///
/// Derived from:
/// - CSR: 2·E·4 + (V+1)·4 ≈ 12·E + 4·V bytes
/// - Cached state: O(V + E) with FxHashMap overhead
/// - Conservative constant: 500 bytes per (V + E)
const MEMORY_BOUND_BYTES_PER_ELEMENT: usize = 500;

/// Creates a ring graph with additional edges.
///
/// Ring: each node i connected to (i+1) % n
/// Additional: edges between nodes at distance 2-5
fn create_ring_with_chords(n: usize) -> CsrGraph {
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();

    // Ring edges
    for i in 0..n {
        let j = (i + 1) % n;
        edges.push((i as u32, j as u32, 1.0));
    }

    // Additional edges (deterministic pattern)
    for i in 0..n {
        for dist in 2..=5 {
            let j = (i + dist) % n;
            if i < j {
                edges.push((i as u32, j as u32, 0.5));
            }
        }
    }

    CsrGraph::from_edges(&edges, n)
}

/// Runs Leiden with dhat profiling and returns peak heap bytes.
///
/// # Errors
///
/// Returns a `String` error if detection fails.
fn profile_memory(graph: &CsrGraph) -> Result<usize, String> {
    let profiler = dhat::Profiler::builder().testing().build();

    let detector = Leiden::new(LeidenConfig::default());
    let _partition = detector
        .detect(graph)
        .map_err(|e| format!("Leiden detection failed: {e}"))?;

    let stats = dhat::HeapStats::get();
    drop(profiler);
    Ok(stats.max_bytes)
}

/// Test memory bound and scaling in a single function (dhat limitation).
#[test]
fn test_memory_bound_and_scaling() {
    // Test memory bound on graphs of increasing size.
    let sizes: Vec<usize> = vec![100, 500, 1000];

    for n in sizes {
        let graph = create_ring_with_chords(n);
        let e = graph.edge_count();

        let max_bytes = profile_memory(&graph).unwrap_or_else(|err| {
            panic!("memory profiling failed for n={n}: {err}");
        });

        let bound = MEMORY_BOUND_BYTES_PER_ELEMENT * (n + e);
        assert!(
            max_bytes <= bound,
            "n={n}, e={e}: max_bytes={max_bytes} > bound={bound} \
             (per-element={MEMORY_BOUND_BYTES_PER_ELEMENT})"
        );
    }

    // Test memory scaling is linear (not quadratic).
    let n1 = 100;
    let n2 = 200;

    let graph1 = create_ring_with_chords(n1);
    let graph2 = create_ring_with_chords(n2);

    let bytes1 = profile_memory(&graph1).unwrap_or_else(|e| {
        panic!("memory profiling failed for n={n1}: {e}");
    });
    let bytes2 = profile_memory(&graph2).unwrap_or_else(|e| {
        panic!("memory profiling failed for n={n2}: {e}");
    });

    // Linear scaling: bytes2 should be < 4x bytes1 (quadratic would be ~4x)
    let ratio = bytes2 as f64 / bytes1.max(1) as f64;
    assert!(
        ratio < 4.0,
        "memory scaling ratio {ratio:.2}x suggests non-linear growth: \
         n={n1} -> {bytes1} bytes, n={n2} -> {bytes2} bytes"
    );
}

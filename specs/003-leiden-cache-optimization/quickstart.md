# Quickstart Validation Guide: Leiden Cache Optimization

**Feature**: 003-leiden-cache-optimization
**Date**: 2026-09-08

---

## Purpose

This guide provides runnable validation scenarios that prove the cache optimization feature works end-to-end. Execute these commands after implementation to verify correctness, performance, and edge case handling.

---

## Prerequisites

```bash
# Build the project
cargo build --release -p communal-algo

# Run existing tests (must all pass)
cargo test -p communal-algo
cargo test -p communal-core
```

---

## Scenario 1: Correctness — Partition Quality Preservation

**Verifies**: FR-006, SC-005 (partition quality within 1e-4 epsilon)

```bash
# Run Tier 1 reference graph tests
cargo test -p communal-algo --test synthetic_ground_truth

# Run connected-community invariant tests
cargo test -p communal-algo --test leiden_connected
```

**Expected**: All tests pass. Partition quality (modularity Q) for each reference graph matches the expected value within 1e-4 absolute epsilon.

**Reference Values** (from AGENTS.md):
| Graph | Nodes | Edges | Expected Q |
|-------|-------|-------|------------|
| Complete K₅ | 5 | 10 | 0.2000 |
| Complete Bipartite K₃,₄ | 7 | 12 | 0.3333 |
| No Edges | 5 | 0 | 0.0000 |
| Path P₅ | 5 | 4 | 0.3125 |
| Star | 6 | 5 | 0.0000 |
| Ring C₆ | 6 | 6 | 0.2222 |
| Grid 3×3 | 9 | 12 | 0.4583 |
| Two Triangles | 6 | 7 | 0.1748 |

---

## Scenario 2: Performance — Wall-Clock Time Targets

**Verifies**: SC-001 through SC-004 (performance targets)

```bash
# Run with release mode and single-threaded pinning
cargo build --release -p communal-cli

# PolBooks (105 nodes) — target < 5s
taskset -c 0 cargo run --release --bin communal-cli -- \
  --input benchmarks/lfr_graphs/polbooks.edgelist \
  --algorithm leiden

# PolBlogs (1,490 nodes) — target < 30s
taskset -c 0 cargo run --release --bin communal-cli -- \
  --input benchmarks/lfr_graphs/polblogs.edgelist \
  --algorithm leiden

# Karate Club (34 nodes) — target < 100ms
taskset -c 0 cargo run --release --bin communal-cli -- \
  --input benchmarks/lfr_graphs/karate.edgelist \
  --algorithm leiden

# NetScience (1,589 nodes) — target < 10s
taskset -c 0 cargo run --release --bin communal-cli -- \
  --input benchmarks/lfr_graphs/netscience.edgelist \
  --algorithm leiden
```

**Expected**: Each dataset completes within the target time. Measure with `Instant::now()` or `time` command.

---

## Scenario 3: Early Termination — Convergence Detection

**Verifies**: FR-003, SC-007 (50% iteration reduction)

```bash
# Run determinism + convergence tests
cargo test -p communal-algo --test leiden_determinism
```

**Expected**: On easy-to-converge graphs (e.g., two cliques connected by few edges), the algorithm terminates before reaching `max_iterations` (10). Median iteration count over 30 runs is ≤ 50% of baseline (early-termination-disabled) median.

**Manual verification**:
```rust
// Pseudocode for manual test
let graph = two_cliques_with_bridge(50, 50, 3); // 100 nodes
let config = LeidenConfig::default(); // convergence_threshold=1e-6, max_iterations=10
let leiden = Leiden::new(config);
let partition = leiden.detect(&graph).unwrap();
// Assert: partition converged before iteration 10
```

---

## Scenario 4: Determinism — Cross-Platform Bitwise Identical

**Verifies**: FR-009 (ChaCha8Rng, bitwise-identical results)

```bash
cargo test -p communal-algo --test leiden_determinism
```

**Expected**: Same graph + same config + same seed → identical membership vector and identical Q value (all 17 significant digits of f64) across multiple runs.

**Test approach**:
1. Run Leiden on Karate Club with seed=42 → record membership + Q
2. Run again with same seed → assert exact equality (not just epsilon)
3. Run with different seed → assert different result (sanity check)

---

## Scenario 5: Edge Cases — Self-Loops, Isolated Nodes, Zero-Weight Edges

**Verifies**: FR-008, FR-013 (edge case semantics)

```bash
cargo test -p communal-algo --test leiden_edge_cases
```

**Expected**:
- **Self-loops**: Counted exactly once in community internal weight (matching igraph convention)
- **Isolated nodes (degree 0)**: Empty neighbor cache, skipped during local moving, contribute 0 to Q
- **Zero-weight edges**: Contribute 0.0 to neighbor cache weight sums, excluded from community weight aggregation
- **Empty graph**: Returns Q=0, empty membership
- **No-edge graph**: Each node in its own community, Q=0

---

## Scenario 6: Connected-Community Guarantee

**Verifies**: FR-010, SC-006 (all communities internally connected)

```bash
# Run in debug mode to enable debug_assert! checks
cargo test -p communal-algo --test leiden_connected
```

**Expected**: All detected communities are internally connected (verified via BFS traversal). Debug builds panic if a disconnected community is detected.

---

## Scenario 7: Memory Bound — O(V + E)

**Verifies**: SC-008 (linear memory scaling)

```bash
# Run with dhat heap tracking (if available)
# Or verify via code inspection: no allocations proportional to V×E
```

**Expected**: Peak heap usage ≤ c·(V + E) for documented constant c. Memory grows linearly with graph size.

---

## Scenario 8: Configuration Validation

**Verifies**: LeidenConfig::validate() enforces spec ranges

```bash
cargo test -p communal-algo config::tests
```

**Expected**:
- `gamma = -1.0` → `AlgorithmError::InvalidConfiguration`
- `beta = 0.0` → `AlgorithmError::InvalidConfiguration`
- `beta = 2.0` → `AlgorithmError::InvalidConfiguration`
- `max_iterations = 0` → `AlgorithmError::InvalidConfiguration`
- `recompute_interval = 0` → `AlgorithmError::InvalidConfiguration`
- `gamma = 0.0` → OK (valid per spec)
- `beta = 0.5` → OK (valid per spec)
- `gamma = 1.0, beta = 0.01, convergence_threshold = 1e-6, max_iterations = 10, seed = Some(42), recompute_interval = 100` → OK (default)

---

## Scenario 9: Backward Compatibility — API Preservation

**Verifies**: FR-007 (public API unchanged)

```bash
# Build dependent crates
cargo build -p communal-cli
cargo build -p communal-wasm
cargo build --features full
```

**Expected**: All dependent crates compile without modification. The `Leiden::new()`, `Leiden::with_quality_function()`, and `CommunityDetector::detect()` signatures are preserved.

---

## Scenario 10: Performance Backward Compatibility

**Verifies**: FR-007 (optimized ≥ unoptimized on all inputs)

**Expected**: On Tier 1 reference graphs, the optimized implementation completes in less time than the unoptimized baseline (same hardware, same seed, same graph).

---

## Regression Checklist

Before marking implementation complete:

- [ ] All Tier 1 reference graph tests pass (Scenario 1)
- [ ] All performance targets met (Scenario 2)
- [ ] Early termination reduces iterations by ≥ 50% (Scenario 3)
- [ ] Deterministic results across runs (Scenario 4)
- [ ] Edge cases handled correctly (Scenario 5)
- [ ] All communities connected (Scenario 6)
- [ ] Memory bound O(V + E) verified (Scenario 7)
- [ ] Config validation enforces spec ranges (Scenario 8)
- [ ] Dependent crates compile (Scenario 9)
- [ ] No performance regression vs baseline (Scenario 10)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo test --workspace` passes

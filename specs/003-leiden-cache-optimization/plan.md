# Implementation Plan: Leiden Cache Optimization

**Branch**: `003-leiden-cache-optimization` | **Date**: 2026-09-08** | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-leiden-cache-optimization/spec.md`

## Summary

Optimize the Leiden community detection algorithm by caching community-level statistics (total degree, internal edge weight, size) and per-node neighbor community weights, updating them incrementally when nodes move. Add early termination with plateau detection (rolling K=5 window), zero-nodes-moved convergence, and OR logic between conditions. Replace `StdRng` with `ChaCha8Rng` for cross-platform bitwise-identical determinism. Restructure the main loop to evaluate convergence only after a complete Leiden pass (local-moving → refinement → aggregation). Expected speedup: 5-10× on graphs ≥100 nodes, plus 1.5-3× from early termination.

## Technical Context

**Language/Version**: Rust 2024 edition (latest stable toolchain)

**Primary Dependencies**:
- `communal-core` (workspace) — `GraphView`, `Partition`, `NodeId`, `CommunityId`, `AlgorithmError`, `ConvergenceMode`, `StepEvent`
- `rand 0.9` — PRNG (already declared in `communal-algo/Cargo.toml`)
- `rand_chacha` — `ChaCha8Rng` for cross-platform deterministic seeding (FR-009). Already present in `Cargo.lock` as transitive dep of `rand 0.9`.
- `rustc-hash` — `FxHashMap` for neighbor cache (FR-001/FR-002). Already in `Cargo.lock`.
- `dhat` — heap usage testing for memory bound validation (SC-008). Added to workspace `[workspace.dev-dependencies]` and `communal-algo` dev-dependencies.
- `tracing` (workspace) — structured logging (already declared)

**Storage**: N/A (in-memory algorithm only)

**Testing**: `cargo test` with `proptest` (property-based), `criterion` (benchmarks, declared but unused). Existing tests in `crates/communal-algo/tests/`.

**Target Platform**: Linux (kernel 5.15+), single-threaded (pinned to core 0 for benchmarks). Cross-platform determinism required for PRNG (FR-009).

**Project Type**: Library crate (`communal-algo`) within workspace. No binary or external API surface changes.

**Performance Goals**:
- PolBooks (105 nodes) < 5s (currently >30s timeout)
- PolBlogs (1,490 nodes) < 30s (currently >30s timeout)
- Karate (34 nodes) < 100ms
- NetScience (1,589 nodes) < 10s
- Partition quality within 1e-4 epsilon of non-optimized
- O(V + E) memory bound
- 50% median iteration reduction on easy-to-converge inputs

**Constraints**:
- `#![deny(unsafe_code)]` — zero unsafe
- `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented)]`
- `#![deny(missing_docs)]` — all public items documented
- No `dyn GraphView` in hot loops (constitution Principle II — monoculture in critical loops)
- Single-threaded (parallelism deferred per spec CHK015)
- f64 precision for all statistical accumulations (FR-011)
- `ConvergenceMode::Absolute` for early termination decisions (FR-003)

**Scale/Scope**: Graphs from 34 to 1,589+ nodes (benchmark targets). O(V + E) memory. No concurrency. ~8 files modified in `crates/communal-algo/src/leiden/`, plus Cargo.toml updates.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Mathematical Rigor | ✅ PASS | Caching preserves exact mathematical results (subtract-add repair in FR-002). Debug invariant checks (FR-001) verify correctness. Connected-community guarantee preserved (FR-010). Periodic full recomputation prevents FP drift (FR-011). |
| II. Performance & Cache Locality | ✅ PASS | Caching community statistics reduces redundant O(V + E) traversals to O(1) incremental updates. `FxHashMap` for neighbor cache preserves cache-friendly flat iteration. No `dyn GraphView` in hot loops — dispatch via `QualityFunction` enum (zero-cost). |
| III. Modular Workspace Architecture | ✅ PASS | Changes isolated to `communal-algo` crate plus one sanctioned `communal-core` addition (`Partition::converged` flag, FR-012; all in-workspace call sites migrated in T003b). No new crate dependencies leak to facade. |
| IV. Zero Technical Debt | ✅ PASS | No unsafe code. All errors via `AlgorithmError`/`GraphError` with `thiserror`. No `unwrap`/`expect`/`panic` in production paths. All public items documented. `tracing` for logging. |
| V. Dynamic Graph Support | ✅ N/A | Not in scope — spec is for batch Leiden optimization only. |
| VI. Verification & Invariant Testing | ✅ PASS | Debug invariant checks (FR-001) provide zero-cost-in-release verification. BFS connectedness checks (FR-010). Tier 1 reference graph tests validate correctness. `proptest` invariants. |
| VII. Licensing & Attribution | ✅ PASS | `rand_chacha` and `rustc-hash` are MIT/Apache-2.0 licensed (compatible). |

**No violations.** No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/003-leiden-cache-optimization/
├── spec.md                      # Feature specification
├── plan.md                      # This file
├── research.md                  # Phase 0: consolidated research findings
├── data-model.md                # Phase 1: entity definitions
├── quickstart.md                # Phase 1: validation guide
├── contracts/                   # Phase 1: interface contracts
│   ├── leiden-config.md         # LeidenConfig field contract
│   ├── leiden-detector.md       # CommunityDetector impl contract
│   └── quickstart.md            # Runnable validation scenarios
└── tasks.md                     # Phase 2: generated by /speckit-tasks
```

### Source Code (repository root)

```text
crates/
├── communal-core/               # MODIFIED — Partition gains converged flag (FR-012)
│   └── src/
│       ├── partition.rs         # MODIFIED: converged field + getter (T003b)
│       ├── graph_view.rs        # GraphView trait (preserved)
│       ├── csr.rs               # CsrGraph (preserved)
│       ├── partition.rs         # Partition (preserved)
│       ├── id.rs                # NodeId, CommunityId (preserved)
│       ├── error.rs             # AlgorithmError, GraphError (preserved)
│       ├── step.rs              # StepEvent enum (preserved)
│       ├── quality.rs           # QualityMetric trait (preserved)
│       ├── config.rs            # AlgorithmConfig, ConvergenceMode (preserved)
│       └── detector.rs          # CommunityDetector trait (preserved)
│
└── communal-algo/               # MODIFIED — all changes here
    ├── Cargo.toml               # ADD: rand_chacha, rustc-hash deps
    └── src/
        ├── quality.rs           # UNCHANGED — Modularity, Cpm, QualityFunction
        └── leiden/
            ├── mod.rs           # MODIFIED: detect() restructure, ChaCha8Rng
            ├── config.rs        # MODIFIED: add recompute_interval, fix seed default, fix validate()
            ├── local_moving.rs  # MODIFIED: cache integration, FxHashMap neighbor cache
            ├── refinement.rs    # MODIFIED: cache integration, ChaCha8Rng
            ├── aggregation.rs   # UNCHANGED — already produces reduced graph correctly
            ├── convergence.rs   # MODIFIED: rolling K=5 window, OR logic, ConvergenceState update
            └── stepping.rs      # UNCHANGED — SteppingCallback trait (infrastructure)

tests/                           # MODIFIED — add performance + edge case tests
    ├── leiden_connected.rs      # ADD: connected-community invariant tests
    ├── leiden_determinism.rs    # ADD: cross-platform bitwise-identical tests
    ├── leiden_edge_cases.rs     # ADD: self-loops, isolated nodes, zero-weight edges
    ├── leiden_correctness_epsilon.rs # ADD: FR-006 partition quality within 1e-4 epsilon
    ├── leiden_iteration_benchmark.rs # ADD: SC-007 50% median iteration reduction
    ├── leiden_memory_bound.rs   # ADD: SC-008 O(V+E) memory bound via dhat
    ├── leiden_perf_backward_compat.rs # ADD: FR-007 performance backward compatibility
    └── synthetic_ground_truth.rs # UNCHANGED
```

**Structure Decision**: All algorithm changes live in `communal-algo`. The sole core touch is the FR-012 `converged` flag on `Partition` (T003b migrates all in-workspace call sites in the same change). No new crates or directories. The optimization is otherwise an implementation-level concern.

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

N/A — no violations.

---

## Phase 0: Research & Decision Log

See [research.md](research.md) for consolidated findings. All technical decisions resolved:

| Decision | Choice | Alternatives Rejected |
|----------|--------|----------------------|
| Neighbor cache hash map | `FxHashMap` (rustc-hash) | `HashMap` (slower), `BTreeMap` (wrong access pattern) |
| Deterministic PRNG | `ChaCha8Rng` (rand_chacha) | `StdRng` (version-dependent), `ChaCha20` (slower) |
| Cache invalidation strategy | Dirty marking + frontier propagation (LD-Leiden style) | Immediate recompute (slow), full rebuild per iteration (defeats purpose) |
| FP drift mitigation | Periodic full recomputation every N=100 updates | Kahan summation (4× overhead), exact rational arithmetic (complex) |
| Convergence evaluation timing | After complete Leiden pass (local-moving → refinement → aggregation) | After each sub-phase (premature termination risk) |
| Early termination logic | OR between quality threshold and zero-nodes-moved | AND (too restrictive), sequential (complex) |
| Oscillation detection | Rolling K=5 window with max-min < ε | Separate oscillation pass (overhead), no detection (premature termination risk) |
| Memory ordering | Not specified (single-threaded) | Explicit ordering (premature — parallelism deferred) |

## Phase 1: Design Artifacts

- [data-model.md](data-model.md) — `LocalMoveState`, `NeighborCache`, `ConvergenceState`, `CacheStatistics` entities
- [contracts/](contracts/) — Interface contracts for `LeidenConfig`, `Leiden` detector, and validation scenarios
- [quickstart.md](quickstart.md) — Runnable end-to-end validation guide

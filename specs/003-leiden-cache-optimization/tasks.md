# Implementation Tasks: Leiden Cache Optimization

**Branch**: `dev` | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Phase 0: Dependencies & Configuration

### Task 1: Add workspace dependencies
- **ID**: T001
- **Files**: `Cargo.toml` (workspace)
- **Action**: Add `rand_chacha = "0.9"` and `rustc-hash = "2.0"` to workspace dependencies
- **Verify**: `cargo check -p communal-algo` compiles

### Task 2: Add communal-algo dependencies
- **ID**: T002
- **Files**: `crates/communal-algo/Cargo.toml`
- **Action**: Add `rand_chacha = "0.9"` and `rustc-hash = "2.0"` as dependencies
- **Verify**: `cargo check -p communal-algo` compiles

### Task 3: Update LeidenConfig
- **ID**: T003
- **Files**: `crates/communal-algo/src/leiden/config.rs`
- **Action**:
  - Add `recompute_interval: u32` field
  - Change `seed` default from `None` to `Some(42)`
  - Relax `gamma` validation from `> 0` to `>= 0`
  - Relax `beta` validation from `[0.0005, 0.1]` to `[0, 1]`
  - Add `max_iterations >= 1` validation
  - Add `recompute_interval >= 1` validation
  - Add `with_recompute_interval()` builder method
- **Verify**: `cargo test -p communal-algo config::tests` passes

## Phase 1: Core Data Structures

### Task 3b: Add converged field to Partition
- **ID**: T003b
- **Files**: `crates/communal-core/src/partition.rs`
- **Action**:
  - Add `converged: bool` field to `Partition` struct
  - Update `Partition::new()` to accept `converged: bool` parameter
  - Add `converged()` getter method
  - Update all `Partition::new()` call sites across workspace (leiden, louvain, lpa, infomap, fluid, tests)
- **Verify**: `cargo test --workspace` passes

### Task 4: Implement ConvergenceState extensions
- **ID**: T004
- **Files**: `crates/communal-algo/src/leiden/convergence.rs`
- **Action**:
  - Add `quality_window: VecDeque<f64>` (K=5 rolling window)
  - Add `nodes_moved: usize`
  - Add `consecutive_zero_movement: usize`
  - Add `best_membership: Vec<u32>`
  - Add `best_quality: f64`
  - Add `converged: bool`
  - Implement `update()` with plateau detection (max-min < threshold)
  - Implement `has_converged()` with OR logic (quality OR zero-movement, per consolidated FR-003)
  - Add `use std::collections::VecDeque`
- **Verify**: `cargo test -p communal-algo convergence::tests` passes

### Task 5: Implement LocalMoveState and NeighborCache
- **ID**: T005
- **Files**: `crates/communal-algo/src/leiden/local_moving.rs`
- **Action**:
  - Add `use rustc_hash::FxHashMap`
  - Create `NeighborCache` struct (`weights: FxHashMap<u32, f64>`, `dirty: bool`)
  - Create `LocalMoveState` struct with all cached fields
  - Implement `LocalMoveState::compute_all()` for full initialization
  - Implement `LocalMoveState::recompute_dirty()` for selective recompute
  - Implement subtract-add repair for community_degree_sums, community_sizes, community_internal_weights
  - Implement frontier propagation for neighbor cache invalidation
  - Implement `CacheStatistics` (debug-only tracking)
- **Verify**: `cargo check -p communal-algo` compiles

## Phase 2: Algorithm Phase Integration

### Task 6: Update local_moving to use LocalMoveState
- **ID**: T006
- **Files**: `crates/communal-algo/src/leiden/local_moving.rs`
- **Action**:
  - Change signature to accept `&mut LocalMoveState` instead of recomputing
  - Use cached `node_degrees`, `community_degree_sums`, `community_sizes`
  - Use neighbor cache with lazy rebuild
  - Return `usize` (nodes moved count) instead of `bool`
  - Implement subtract-add repair on node moves
  - Implement frontier propagation on node moves
  - Add debug_assert! for cache consistency
- **Verify**: `cargo check -p communal-algo` compiles

### Task 7: Update refinement with cache + ChaCha8Rng
- **ID**: T007
- **Files**: `crates/communal-algo/src/leiden/refinement.rs`
- **Action**:
  - Replace `StdRng` with `ChaCha8Rng`
  - Change signature to accept `&mut LocalMoveState`
  - Use cached statistics
  - Add debug_assert! for connected-community guarantee (FR-010)
  - Simplify `SelectionParams` struct: the `partition` field is retained for delta_q evaluation but the struct no longer holds a separate membership reference (membership is accessed via the `partition` field). This avoids dual-source-of-truth between partition and membership slice.
- **Verify**: `cargo check -p communal-algo` compiles

### Task 8: Update mod.rs main loop
- **ID**: T008
- **Files**: `crates/communal-algo/src/leiden/mod.rs`
- **Action**:
  - Replace `StdRng` with `ChaCha8Rng`
  - Restructure main loop: local_moving -> refinement -> aggregation -> convergence check
  - Initialize `LocalMoveState` from graph
  - Track `ConvergenceState` across iterations
  - Evaluate convergence only after complete Leiden pass (FR-003)
  - Implement best-partition tracking (FR-012)
  - Implement periodic recomputation counter (FR-011)
  - Emit warning log on non-convergence (FR-012)
  - Update `detect()` to use new phase signatures
- **Verify**: `cargo check -p communal-algo` compiles

## Phase 3: Integration & Testing

### Task 9: Verify aggregation correctness
- **ID**: T009
- **Files**: `crates/communal-algo/src/leiden/aggregation.rs`
- **Action**:
  - Verify reduced graph node count equals unique community count from membership
  - Verify edge weights in reduced graph equal sum of inter-community edge weights from original graph
  - Verify community ID remapping is bijective (each sparse community ID maps to exactly one contiguous 0-based index)
  - Verify self-loops in reduced graph equal intra-community edge weight sums
  - Confirm no structural changes needed (already works correctly)
- **Verify**: `cargo check -p communal-algo` compiles

### Task 10: Add determinism tests
- **ID**: T010
- **Files**: `crates/communal-algo/tests/leiden_determinism.rs`
- **Action**:
  - Verify ChaCha8Rng produces bitwise-identical results across runs
  - Test same seed -> exact same membership + Q
  - Test different seed -> different result (sanity)
- **Verify**: `cargo test -p communal-algo --test leiden_determinism` passes

### Task 11: Add edge case tests
- **ID**: T011
- **Files**: `crates/communal-algo/tests/leiden_edge_cases.rs`
- **Action**:
  - Test self-loops (counted once, FR-013)
  - Test isolated nodes (empty cache, skipped, Q=0 contribution)
  - Test zero-weight edges (excluded from sums)
  - Test empty graph (Q=0)
  - Test no-edge graph (singletons, Q=0)
  - Test fully connected graph (single community, FR-008)
  - Test all nodes in same initial community (FR-008)
- **Verify**: `cargo test -p communal-algo --test leiden_edge_cases` passes

### Task 12: Add connected-community tests
- **ID**: T012
- **Files**: `crates/communal-algo/tests/leiden_connected.rs`
- **Action**:
  - Verify all communities are internally connected (BFS)
  - Run in debug mode to trigger debug_assert!
- **Verify**: `cargo test -p communal-algo --test leiden_connected` passes

## Phase 4: Validation

### Task 13: Run full workspace tests
- **ID**: T013
- **Action**: `cargo test --workspace` passes
- **Verify**: All existing + new tests pass

### Task 14: Run clippy and fmt
- **ID**: T014
- **Action**:
  - `cargo clippy --workspace --all-targets -- -D warnings` passes
  - `cargo fmt -- --check` passes
- **Verify**: Zero warnings, zero formatting issues

### Task 15: Performance validation
- **ID**: T015
- **Action**: Run Tier 1 reference graphs, verify correctness + timing
- **Verify**: All Tier 1 tests pass with correct Q values

### Task 16: Add explicit FR-006 correctness epsilon test
- **ID**: T016
- **Files**: `crates/communal-algo/tests/leiden_correctness_epsilon.rs`
- **Action**:
  - Run the optimized Leiden implementation on all Tier 1 reference graphs (K₅, K₃,₄, No Edges, P₅, Star, Ring C₆, Grid 3×3, Two Triangles)
  - Compare resulting modularity Q against known-correct reference values from the spec's Tier 1 table
  - Assert |Q_optimized - Q_reference| < 1e-4 for each graph
  - Also compare against the non-optimized implementation (if available via feature flag) to assert |Q_optimized - Q_baseline| < 1e-4
- **Verify**: `cargo test -p communal-algo --test leiden_correctness_epsilon` passes

### Task 17: Add SC-007 iteration reduction benchmark
- **ID**: T017
- **Files**: `crates/communal-algo/tests/leiden_iteration_benchmark.rs`
- **Action**:
  - Add `communal-generators` to `communal-algo` dev-dependencies in `crates/communal-algo/Cargo.toml`
  - Implement baseline measurement: run Leiden with `convergence_threshold=0` (disables quality-based termination), same seed, until zero nodes moved or max_iterations reached
  - Implement optimized measurement: run Leiden with default config (convergence_threshold=1e-6)
  - Use LFR benchmark graphs with μ ≤ 0.3 (easy-to-converge) — generate via `communal_generators::Lfr`
  - Run 30 trials per graph, report median iteration count for baseline and optimized
  - Assert median_optimized ≤ 0.5 × median_baseline
  - Assert |Q_optimized - Q_baseline| < 1e-6 (early termination quality bound, US2/AC2)
- **Verify**: `cargo test -p communal-algo --test leiden_iteration_benchmark` passes

### Task 18: Add SC-008 memory bound test
- **ID**: T018
- **Files**: `crates/communal-algo/tests/leiden_memory_bound.rs`
- **Action**:
  - Add `dhat` to workspace `[dev-dependencies]` in root `Cargo.toml`
  - Set `#[global_allocator] static ALLOC: dhat::Alloc = dhat::Alloc;` in test file
  - Use `dhat::Profiler::builder().testing().build()` to profile heap usage
  - Run Leiden on graphs of increasing size (100, 500, 1000, 2000 nodes)
  - Assert `dhat::HeapStats::get().max_bytes ≤ c · (V + E)` where `c` is a documented constant derived from CSR representation (2·E·sizeof(u32) + (V+1)·sizeof(u32) for offsets + indices + cached state overhead)
  - Document the constant `c` in the test file header
- **Verify**: `cargo test -p communal-algo --test leiden_memory_bound` passes

### Task 19: Add FR-007 performance backward compatibility test
- **ID**: T019
- **Files**: `crates/communal-algo/tests/leiden_perf_backward_compat.rs`
- **Action**:
  - Acquire canonical benchmark datasets per spec Dataset Sources (Karate Club, PolBooks, NetScience, PolBlogs) — commit edgelist fixtures to `benchmarks/` or generate via `communal_generators::Lfr`
  - Run the optimized Leiden implementation on each dataset
  - Measure wall-clock time for each graph under release mode, single-threaded, median-of-30 runs after warm-up
  - Assert each graph completes within the SC-001 through SC-004 targets (Karate < 100ms, PolBooks < 5s, NetScience < 10s, PolBlogs < 30s)
  - Additionally, if a baseline (pre-optimization) implementation is available via feature flag, assert optimized is at least as fast
- **Verify**: `cargo test -p communal-algo --test leiden_perf_backward_compat` passes

## Phase 5: Convergence

> Appended by `/speckit-converge` (2026-09-09). Each item traces to its source requirement/decision and carries its gap type. Existing tasks were NOT modified or renumbered.

- [X] T020 Implement beta-weighted probabilistic community selection in refinement per FR-002, FR-003 (contradicts)
- [X] T021 Incrementally maintain community_internal_weights via subtract-add repair and count self-loops exactly once per FR-001, FR-013 (contradicts)
- [X] T022 Add cache-consistency debug_assert verifying cached weight sums against graph state per FR-001 (partial)
- [X] T023 Add FR-006/SC-005 epsilon test asserting |Q_optimized - Q_reference| < 1e-4 on all Tier 1 graphs per FR-006 (missing)
- [X] T024 Add SC-007 iteration benchmark with median_optimized <= 0.5 x median_baseline plus paired 1e-6 quality-delta assertion per SC-007, US2/AC2 (missing)
- [X] T025 Add SC-008 dhat memory-bound test asserting max_bytes <= c.(V + E) with documented constant c per SC-008 (missing)
- [X] T026 Add FR-007 performance backward-compatibility test with canonical dataset acquisition and SC-001 through SC-004 timing targets per FR-007 (missing)
- [X] T027 Relax beta validation to [0, 1] (`beta = 0` = greedy deterministic) and align error message and docs per spec Clarifications (contradicts)
- [X] T028 Align contracts and data-model to keep `max_iterations: usize` (no type change) and add `convergence_mode` row to spec Key Entities per plan: config contract (contradicts)
- [X] T029 Implement debug-only CacheStatistics tracking with hit/miss/invalidation counters per plan: data-model (partial)
- [X] T030 Forward ConvergenceState StepEvents to observers and deduplicate the SteppingCallback trait per plan: stepping contract (partial)
- [X] T031 Invalidate the moved node's own neighbor cache and exclude zero-weight edges from cache sums per FR-002, FR-013 (partial)
- [X] T032 Pin ascending-NodeId accumulation order with a test, implement the debug-build FP-drift fallback detector (FR-002 secondary safety net), and defer periodic recomputation to the next phase boundary (FR-011) per FR-011, FR-002 (partial)
- [X] T036 In the main loop (mod.rs), defer periodic recomputation to the next phase boundary (after aggregation) per FR-011, rather than executing it mid-pass per FR-011 (partial)
- [X] T033 Harden determinism tests to avoid expect/panic and assert bitwise-identical Q values per FR-009, Constitution IV (contradicts)
- [X] T034 Add isolated-node, no-edge-singleton, and same-initial-community edge-case tests per FR-008 (partial)
- [X] T035 Review or remove the always-false Partition::has_disconnected_communities placeholder per FR-010 (unrequested)

## Phase 6: Convergence

> Appended by `/speckit-converge` (2026-09-09). Each item traces to its source requirement/decision and carries its gap type. Existing tasks were NOT modified or renumbered.

- [X] T037 Implement beta-weighted probabilistic community selection in refinement (`refinement.rs`) — replace deterministic `acceptance_prob = if gain > 0.0 { 1.0 }` with beta-weighted probability per FR-002, FR-003 (partial)
- [X] T038 Invalidate the moved node's own neighbor cache after a node move in both `local_moving` and `refinement` (`local_moving.rs`, `refinement.rs`) per FR-002 (partial)
- [X] T039 Add cache-consistency `debug_assert!` in `local_moving` and `refinement` that verifies cached community weight sums match actual graph state (computed via full traversal) per FR-001 (partial)
- [X] T040 Add FR-006/SC-005 epsilon correctness test asserting |Q_optimized - Q_reference| < 1e-4 on all Tier 1 reference graphs (K₅, K₃,₄, No Edges, P₅, Star, Ring C₆, Grid 3×3, Two Triangles) per FR-006, SC-005 (missing)
- [X] T041 Pin ascending-NodeId accumulation order in `compute_all` and fix self-loop counting to be exact (count once, not /2.0 approximation) per FR-011, FR-013 (partial)
- [X] T042 Exclude zero-weight edges from neighbor cache weight sums in `get_neighbor_cache` per FR-013 (partial)
- [X] T043 Implement debug-build FP-drift fallback detector comparing cached community weight sums against full graph traversal, falling back to full recompute if mismatch exceeds 1e-4 per FR-002 (partial)
- [X] T044 Add SC-007 iteration reduction benchmark test: median_optimized ≤ 0.5 × median_baseline on easy-to-converge graphs (LFR μ ≤ 0.3, 30 runs) plus paired 1e-6 quality-delta assertion per SC-007, US2/AC2 (missing)
- [X] T045 Add SC-008 dhat memory-bound test asserting max_bytes ≤ c·(V + E) with documented constant c per SC-008 (missing)
- [X] T046 Add FR-007 performance backward-compatibility test with canonical datasets and SC-001–SC-004 timing targets per FR-007 (missing)
- [X] T047 Add all-nodes-same-initial-community edge-case test per FR-008 (partial)
- [X] T048 Review or remove the always-false `Partition::has_disconnected_communities` placeholder (`partition.rs:78-80`) per FR-010 (unrequested)
- [X] T049 Implement debug-only `CacheStatistics` tracking with hit/miss/invalidation counters per plan: data-model (partial)
- [X] T050 Forward `ConvergenceState` `StepEvent`s to observers and deduplicate the `SteppingCallback` trait (currently duplicated in `leiden/stepping.rs` and `mod.rs`) per plan: stepping contract (partial)
- [X] T051 Harden determinism tests to assert bitwise-identical Q values (all 17 significant digits of f64) per FR-009, Constitution IV (partial)

## Phase 7: Convergence

> Appended by `/speckit-converge` (2026-09-09). Each item traces to its source requirement/decision and carries its gap type. Existing tasks were NOT modified or renumbered.

- [X] T052 Implement beta-weighted uniform random community selection in `refinement.rs` — when `beta = 1`, select uniformly at random among eligible candidate communities instead of always selecting max-gain, matching spec Field Interactions per FR-002, FR-003 (partial)
- [X] T053 Add subtract-add repair for `community_internal_weights` in `apply_move()` (`local_moving.rs:210-232`) — update intra-community edge weight sums when a node moves between communities, per FR-002 (partial)
- [X] T054 Integrate `CacheStatistics` into `LocalMoveState` — wire hit/miss/invalidation/full-recompute counters into `get_neighbor_cache`, `mark_dirty`, `recompute_dirty`, and `apply_move` per plan: data-model (partial)
- [X] T055 Deduplicate `SteppingCallback` trait (currently defined in both `leiden/mod.rs:40-52` and `leiden/stepping.rs:9-21`) and forward `ConvergenceState` `StepEvent`s from `mod.rs:216` to registered observers per plan: stepping contract (partial)
- [X] T056 Add test verifying ascending-NodeId accumulation order produces deterministic cross-platform results for cached community statistics per FR-011 (partial)
- [X] T057 Assert bitwise-identical Q values (all 17 significant digits of f64) in determinism tests (`leiden_determinism.rs`) per FR-009, Constitution IV (partial)
- [X] T058 Add edge-case test where all nodes start in the same initial community per FR-008 (partial)
- [X] T059 Review or remove the always-false `Partition::has_disconnected_communities` placeholder (`partition.rs:81-83`) per FR-010 (unrequested)

## Phase 8: Convergence

> Appended by `/speckit-converge` (2026-09-09). Each item traces to its source requirement/decision and carries its gap type. Existing tasks were NOT modified or renumbered.

- [X] T060 Add subtract-add repair for `community_internal_weights` in `apply_move()` (`local_moving.rs:209-231`) — update intra-community edge weight sums when a node moves between communities, per FR-002 (partial)
- [X] T061 Call `check_and_repair()` in the main loop (`mod.rs`) to trigger the FP-drift fallback detector when cached community weight sums diverge from full graph traversal by more than 1e-4, per FR-002 (partial)
- [X] T062 Assert bitwise-identical Q values (all 17 significant digits of f64) in determinism tests (`leiden_determinism.rs`) per FR-009, Constitution IV (partial)
- [X] T063 Integrate `CacheStatistics` into `LocalMoveState` — add backing field and wire hit/miss/invalidation/full-recompute counters into `get_neighbor_cache`, `mark_dirty`, `recompute_dirty`, and `apply_move` per plan: data-model (partial)
- [X] T064 Deduplicate `SteppingCallback` trait (currently defined in both `leiden/mod.rs:40-52` and `leiden/stepping.rs:9-21`) — keep one definition and re-export, per plan: stepping contract (partial)
- [X] T065 Add test verifying ascending-NodeId accumulation order produces deterministic cross-platform results for cached community statistics per FR-011 (partial)
- [X] T066 Add edge-case test where all nodes start in the same initial community per FR-008 (missing)
- [X] T067 Review or remove the always-false `Partition::has_disconnected_communities` placeholder (`partition.rs:81-83`) per FR-010 (unrequested)

## Phase 9: Convergence

> Appended by `/speckit-converge` (2026-09-09). Each item traces to its source requirement/decision and carries its gap type. Existing tasks were NOT modified or renumbered.

- [X] T068 Expose iteration count from `Partition` (add `iterations: usize` field + getter) and assert SC-007 median iteration reduction (`median_optimized ≤ 0.5 × median_baseline`) in `leiden_iteration_benchmark.rs` per SC-007, US2/AC1 (partial)
- [X] T069 Call `LocalMoveState::check_and_repair()` in the main loop (`mod.rs`) in debug builds so the FP-drift fallback detector repairs cached community weight sums when they diverge from full graph traversal by more than 1e-4, per FR-002 (partial)
- [X] T070 Add test verifying ascending-NodeId accumulation order in `compute_all` produces deterministic cross-platform results for cached community statistics per FR-011 (partial)

## Phase 10: Convergence

> Appended by `/speckit-converge` (2026-09-09). Each item traces to its source requirement/decision and carries its gap type. Existing tasks were NOT modified or renumbered.

- [X] T071 Expose iteration count from `Partition` (add `iterations: usize` field + getter) and assert SC-007 median iteration reduction (`median_optimized ≤ 0.5 × median_baseline`) in `leiden_iteration_benchmark.rs` per SC-007 (partial)

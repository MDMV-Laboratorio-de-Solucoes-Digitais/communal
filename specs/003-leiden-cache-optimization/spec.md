# Feature Specification: Leiden Cache Optimization

**Feature Branch**: `003-leiden-cache-optimization`

**Created**: 2026-09-07

**Status**: Draft

**Input**: User description: "Implementation Spec: Caching Community Statistics + Early Termination"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Fast Community Detection on Medium Graphs (Priority: P1)

**As a** researcher or data scientist running community detection,
**I want** the Leiden algorithm to complete on graphs with hundreds to thousands of nodes within seconds,
**So that** I can iterate on my analysis without waiting minutes for results.

**Why this priority**: The current implementation times out on graphs as small as 105 nodes (PolBooks), making the framework unusable for real-world research. This is the primary blocker for adoption.

**Independent Test**: Run the Leiden algorithm on the PolBooks dataset (105 nodes, 441 edges) and verify it completes within 5 seconds while producing the same community partition as the current implementation.

**Acceptance Scenarios**:

1. **Given** a graph with 100-2000 nodes, **When** the user runs Leiden community detection, **Then** the algorithm completes within 5 seconds
2. **Given** the same input graph, **When** running the optimized algorithm, **Then** the resulting partition quality (modularity Q) is identical to the current implementation within 1e-4 absolute epsilon
3. **Given** a graph with disconnected components, **When** running the optimized algorithm, **Then** all detected communities remain internally connected

---

### User Story 2 - Early Termination on Convergence (Priority: P2)

**As a** user running community detection on large graphs,
**I want** the algorithm to stop early when further iterations produce negligible improvement,
**So that** I don't waste compute time on iterations that don't meaningfully change the result.

**Why this priority**: Even with caching, unnecessary iterations waste time. Early termination provides 1.5-3× additional speedup with no quality loss.

**Independent Test**: Run the algorithm on a graph that converges quickly (e.g., two cliques connected by a few edges) and verify it terminates before reaching the maximum iteration count.

**Acceptance Scenarios**:

1. **Given** a graph where community structure stabilizes within 10 iterations, **When** running the algorithm, **Then** it terminates before reaching the maximum iteration limit
2. **Given** early termination triggers, **When** the algorithm stops, **Then** the final partition quality is within 1e-6 of what would be achieved at maximum iterations
3. **Given** a graph with no clear community structure, **When** running the algorithm, **Then** it runs until the maximum iteration limit is reached

---

### User Story 3 - Scalability to Large Networks (Priority: P3)

**As a** researcher analyzing social networks or web graphs,
**I want** the Leiden algorithm to handle graphs with thousands of nodes and edges within documented time bounds (e.g., 5000+ nodes within 2 minutes),
**So that** I can analyze real-world network datasets without timeouts.

**Why this priority**: The framework's value increases with the size of graphs it can handle. This story validates the optimization scales beyond small test cases.

**Independent Test**: Run the Leiden algorithm on the PolBlogs dataset (1,490 nodes, 19,090 edges) and verify it completes within 30 seconds.

**Acceptance Scenarios**:

1. **Given** a graph with 1000-2000 nodes and 10000+ edges, **When** running the algorithm, **Then** it completes within 30 seconds
2. **Given** a graph with 5000+ nodes, **When** running the algorithm, **Then** it completes within 2 minutes
3. **Given** memory constraints, **When** processing large graphs, **Then** memory usage grows linearly with graph size (O(V + E))

---

### Edge Cases

- What happens when the graph has no edges (all singleton communities)?
- What happens when the graph is fully connected (single community)?
- What happens when all nodes belong to the same community initially?
- How does the system handle graphs with self-loops?
- How does the system handle graphs with zero-weight edges?
- What happens when the graph has isolated nodes (degree 0)?

---

## Clarifications

### Session 2026-09-07

- **Q:** What valid ranges should the spec document for all user-configurable `LeidenConfig` parameters? → **A:** Option B — Document ranges matching codebase + add missing: gamma ≥ 0, beta ∈ [0, 1], convergence_threshold ≥ 0, max_iterations ≥ 1, seed: any u64. `LeidenConfig::validate()` MUST enforce these and return `AlgorithmError::InvalidConfiguration` on violation.

- **Q:** What reference hardware should the spec name for reproducible performance targets? → **A:** Tiered approach. Reference class: ≥4 CPU threads, ≥16 GB RAM. Measured on: AMD Ryzen 7 3700U (4C/8T), 13 GB RAM. Targets scale linearly for faster hardware: pass = median ≤ target × (reference_threads / actual_threads). This accommodates the current development machine (laptop APU) and GitHub Actions (4 vCPU shared) while remaining reproducible.

- **Q:** What numerical stability guarantees should the system provide for accumulated floating-point error across cached incremental updates? → **A:** Periodic full recompute of cached statistics after every N incremental updates (user-configurable, default 100) using f64 precision for all accumulations

- **Q:** During which algorithm phases should early termination (FR-003/FR-004) be evaluated? → **A:** Evaluate only after a complete Leiden pass (local-moving → refinement → aggregation) completes, not after individual sub-phases

- **Q:** What should happen when the algorithm reaches the maximum iteration limit without converging? → **A:** Return the best partition found so far with a warning log AND include convergence status flag in Partition metadata

- **Q:** What edge case semantics should the caching system enforce for self-loops, isolated nodes, and zero-weight edges? → **A:** Explicit per-case semantics: self-loops counted once in community weight, isolated nodes have empty neighbor cache, zero-weight edges excluded from cache weight sums

- **Q:** Should the early termination parameters be user-configurable or fixed internal constants? → **A:** User-configurable via `LeidenConfig` fields (`convergence_threshold`, `movement_threshold`, `beta`)
- **Q:** What should be the default convergence threshold for early termination (FR-003)? → **A:** 1e-6 (0.000001), matching the existing `convergence_threshold` default in `LeidenConfig`
- **Q:** How should the two early termination conditions interact — convergence threshold (FR-003) and node movement threshold (FR-004)? → **A:** OR logic — algorithm terminates when EITHER condition is satisfied for the configured number of consecutive iterations
- **Q:** What scope of reproducibility should the deterministic seed behavior guarantee (FR-009)? → **A:** Cross-platform bitwise-identical — same seed produces identical results across compiler versions, OSes, and architectures
- **Q:** What hardware baseline and measurement methodology should apply to the performance targets (SC-001 through SC-004)? → **A:** Release mode, single-threaded, median-of-30 runs after warm-up, on specified reference hardware


- **Q:** What should the node movement threshold be for early termination (FR-004), given that 1% causes premature termination on small graphs? → **A:** Option B — Use `max(1, floor(0.01 * |V|))` as a secondary early-exit heuristic combined with quality-based convergence. Primary convergence: zero nodes move (standard Leiden). Secondary: fewer than `max(1, floor(0.01 * |V|))` nodes move. Either condition triggers termination after the configured number of consecutive iterations.

- **Q:** What cache consistency and fallback approach should the caching system use for maintaining community statistics? → **A:** Option B — Dirty cache marking + frontier-based invalidation (LD-Leiden style) with debug-only invariant checks. Cached entries are marked dirty when invalidated; frontier propagation handles transitive invalidation. Debug builds assert community weight sums match actual graph state. Fallback to full recompute at phase boundaries and every N incremental updates (FR-011).

- **Q:** What baseline should the 50% iteration reduction target (SC-007) be measured against? → **A:** Option A — Same algorithm, same seed, early termination disabled (`convergence_threshold=0` so |ΔQ| < 0 is never satisfied, disabling quality-based termination), running until zero nodes moved or `max_iterations` limit. Matches leidenalg `n_iterations=-1` behavior. "Easy-to-converge" graphs are defined as those where the baseline converges in ≤20 iterations (e.g., LFR benchmarks with μ ≤ 0.3).

- **Q:** Should the spec name canonical dataset sources for benchmark graphs? → **A:** Option A — Add a "Dataset Sources" section with canonical URLs, formats, and citations for each benchmark dataset. This ensures reproducibility and aligns with the constitution's verification principle.

- **Q:** What concrete measurement method should verify the O(V + E) memory bound (SC-008)? → **A:** Option A — Use the `dhat` crate's heap usage testing mode to assert `max_bytes ≤ c·(V + E)` for a documented constant `c`. This is purpose-built for asserting allocation bounds in tests and provides deterministic results.

- **Q:** How should the 1% node movement threshold (FR-004) be justified? → **A:** Option A — Remove the 1% threshold entirely. Use only zero nodes moved (standard Leiden convergence criterion), matching all reference implementations (leidenalg, igraph, NetworkX) and the original Leiden paper (Traag et al. 2019). The `movement_threshold` field is removed from `LeidenConfig`; `beta` (refinement randomness) is now a separate parameter.

- **Q:** Should the spec document interactions between `LeidenConfig` fields? → **A:** Option A — Add a "Field Interactions" subsection to the Key Entities section documenting boundary behavior for each config field. This is standard practice in reference implementations (leidenalg, igraph) and prevents user confusion.

- **Q:** What oscillation detection mechanism should the system use to prevent premature termination on graphs with oscillating quality? → **A:** Option B — Rolling quality history window (K=5 iterations) with plateau detection. The system tracks the last K quality values. If `max(Q_window) - min(Q_window) < ε` (where ε = `convergence_threshold`) for K consecutive iterations, the algorithm terminates. This catches both true convergence and floating-point oscillation without requiring a separate detection pass.

- **Q:** What performance testing and backward compatibility requirements should be added beyond API signature stability? → **A:** Option B — Add performance backward compat to FR-007 ("optimized implementation MUST be at least as fast as the unoptimized implementation on all input graphs, measured under the same conditions") and add a performance test tier that validates timing bounds on Tier 1 reference graphs (correctness + wall-clock time).

- **Q:** What numerical edge case semantics should the caching system enforce for community statistics? → **A:** Option B — Explicit per-case semantics: (1) single-node communities without self-loops contribute `−(k_i/2m)²` to modularity Q, (2) isolated nodes (k=0) contribute exactly 0 to Q, (3) empty graph (m=0) returns Q=0 (avoids division by zero), (4) self-loops in singleton communities count as internal weight (`Σ_in += w_ii`), potentially offsetting the degree tax.

### Session 2026-09-08

- **Q:** Which specific PRNG algorithm should be used to guarantee cross-platform bitwise-identical results (FR-009)? → **A:** `ChaCha8Rng` (from `rand_chacha` crate) — fixed specification (RFC 8439), recommended by `rand` docs for reproducibility, faster than ChaCha20 while deterministic

- **Q:** What is the valid range for the periodic recomputation interval N (FR-011), and can N=0 disable it? → **A:** N ∈ [1, u32::MAX], default 100. N=0 rejected by `validate()` — disabling the safety net would allow unbounded FP drift. Field named `recompute_interval` in `LeidenConfig`.

- **Q:** Should the 50% iteration reduction target (SC-007) be measured using mean or median? → **A:** Median iteration count over 30 runs, consistent with Measurement Methodology. Pass criterion: median_optimized ≤ 0.5 × median_baseline.

- **Q:** Should the obsolete `movement_threshold` field reference in FR-004 be removed? → **A:** Yes — removed. FR-004 now only mentions `beta` as the refinement randomness parameter, matching the Clarifications and Key Entities.

- **Q:** What should the default maximum iteration count be and how should FR-005 justify it, given that Traag et al. 2019 does NOT claim "5-80 iterations" and the spec (100) contradicts the codebase (1000)? → **A:** Default 10 (match reference implementations leidenalg/igraph which default to n_iterations=2-10), justify with leidenalg citation. Remove the unsupported "5-80 iterations" claim.

- **Q:** How should defaults and units be consolidated in Key Entities (CHK029)? → **A:** Option A — Expand Key Entities §ConvergenceState into a full "LeidenConfig Fields" table with all 6 fields, their defaults, types, units, and descriptions.

- **Q:** How should the SC-007 baseline disable early termination, given that "convergence_threshold set to max_iterations" is trivially satisfied when max_iterations=10? → **A:** `convergence_threshold=0` (disables quality-based termination since |ΔQ| < 0 is never true); algorithm runs until zero nodes moved (standard Leiden convergence) or max_iterations reached. Matches leidenalg `n_iterations=-1` behavior and benchmarking literature.

### Session 2026-09-08 (2)

- **Q:** The spec documents `recompute_interval` (default 100) in LeidenConfig Fields and FR-011, but this field does NOT exist in the codebase's `LeidenConfig` struct. Should the field be added or is the spec premature? → **A:** Option A — Add `recompute_interval` field to `LeidenConfig` struct. The spec (FR-011) explicitly requires periodic full recomputation to reset FP drift, and the field is already documented with valid range [1, u32::MAX]. The struct is missing this field and MUST be updated during implementation.

- **Q:** The spec documents `seed` default as 42, but the codebase's `LeidenConfig::Default` trait produces `None` with a runtime fallback of 42. Should the `Default` trait produce `Some(42)` or should the spec say `None (runtime fallback: 42)`? → **A:** Option A — Change `LeidenConfig::Default` to produce `Some(42)` for the `seed` field, matching the spec. This eliminates the runtime fallback and makes the default explicit in the type system.

- **Q:** FR-005 cites Traag et al. 2019 Lemma 8 for "1-2 iterations achieve majority of quality improvement," but the paper reports DBLP requires ~80 iterations. Should FR-005 cite leidenalg/igraph default (n_iterations=2-10) as primary justification instead? → **A:** Option A — Cite leidenalg/igraph default n_iterations=2-10 as primary justification. The 10-iteration cap is a practical default matching reference implementations, not a theoretical guarantee. Removed the misleading Lemma 8 citation.

---

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: The system MUST cache community-level statistics (total degree, internal edge weight, size) and update them incrementally when nodes move between communities. Cache consistency is maintained via **dirty cache marking** — when a cached entry is invalidated, it is marked dirty rather than immediately recomputed. Dirty entries are recomputed on next access or at phase boundaries. Debug builds MUST include invariant checks asserting that cached community weight sums match the actual graph state (computed via full traversal), panicking if a mismatch is detected. This provides zero-cost-in-release correctness verification. **Note:** `debug_assert!` is NOT flagged by `clippy::panic` (verified with clippy 0.1.98), so no `#[allow]` attribute is needed.

- **FR-002**: The system MUST cache per-node neighbor community weights and reuse them across candidate evaluations within a single node's move decision. Cache invalidation MUST handle all of the following triggers: (1) direct node movement (when a node changes community), (2) community merge/split events during refinement, (3) aggregation phase rebuild when the reduced graph is constructed, and (4) transitive invalidation via frontier propagation. The system uses **subtract-add repair** for exact aggregate weight updates: when a node moves from community X to Y, subtract its contribution from X's statistics and add to Y's statistics. **FP-drift fallback**: FR-011's periodic full recomputation (every N=100 incremental updates) is the primary defense against accumulated floating-point error. As a secondary safety net, debug builds MUST detect drift by comparing cached community weight sums against a full graph traversal (see FR-001 invariant check); if a mismatch exceeding the partition quality tolerance of 1e-4 (per SC-005) is detected, the system MUST fall back to full recomputation of affected community statistics. **Beta/cache independence**: The refinement randomness parameter `beta` affects which moves are considered (random neighbor community selection per Traag et al. 2019) but does NOT affect cache update semantics. All accepted moves use subtract-add repair regardless of how the move was selected. This is because cache updates are deterministic given a move — the selection mechanism (greedy or probabilistic) only determines WHICH moves happen, not HOW caches are updated.

  **Frontier propagation semantics** (LD-Leiden style, per Bokov et al. 2026): When a node `v` moves from community `X` to community `Y`, the following cache entries MUST be marked dirty: (a) the neighbor cache of every graph neighbor of `v` (their community weight sums are stale because `v` changed community), (b) the community statistics of `X` and `Y` (degree sums, internal weights, and sizes changed), and (c) the neighbor cache of `v` itself (its community membership changed). The dirty entries are recomputed lazily on next access or at phase boundaries (end of the current Leiden pass, after aggregation). This ensures that subsequent move decisions for neighbors of `v` use up-to-date community weight statistics without requiring a full cache rebuild.

- **FR-003**: The system MUST detect convergence and terminate early when EITHER of the following conditions is satisfied (OR logic): (a) the absolute quality change (|Q_new - Q_old|) falls below the `convergence_threshold` field of `LeidenConfig` (default 1e-6) for a user-configurable number of consecutive iterations, OR (b) zero nodes moved during a complete local-moving pass (standard Leiden convergence criterion, matching leidenalg, igraph, NetworkX, and Traag et al. 2019). Additionally, the system MUST maintain a rolling quality history window of the last K=5 quality values. If `max(Q_window) - min(Q_window) < convergence_threshold` for K consecutive iterations, the algorithm MUST terminate (plateau detection). This catches both true convergence and floating-point oscillation, preventing premature termination on cycling quality values. Early termination MUST be evaluated only after a complete Leiden pass (local-moving → refinement → aggregation) completes, not after individual sub-phases, to prevent premature termination when refinement creates new community boundaries. The refinement randomness parameter `beta` (default 0.01) controls exploration during the refinement phase and does NOT affect early termination decisions.

- **FR-004**: RESERVED — merged into FR-003. Historical references to "FR-004"/"FR-004a" in the Clarifications log, checklists, `research.md`, and code comments denote the zero-nodes-moved convergence condition now specified in FR-003.

- **FR-005**: The system MUST use a default maximum iteration count of 10, matching reference implementations (leidenalg, igraph) which default to n_iterations=2-10. This is a practical cap based on empirical observation: the Leiden algorithm converges rapidly on most real-world graphs, with reference implementations converging within 2-10 iterations on standard benchmarks. A maximum of 10 prevents excessive runtime on pathological inputs while matching the de facto standard. FR-012 handles the rare non-convergent case. **Note:** The codebase currently defaults to 1000; this MUST be updated to 10 to match the spec. **Future variant (deferred):** An adaptive iteration mode that starts with N=5, checks convergence, and increments by 5 until plateau or convergence. This may be added in a future iteration if real-world profiling shows >10 iterations are needed on target-scale graphs.

- **FR-006**: The system MUST produce identical community partitions (within 1e-4 absolute epsilon for modularity Q) compared to the non-optimized implementation. See SC-005 for the measurable criterion and justification.

- **FR-007**: The system MUST maintain the existing public API for the Leiden algorithm without breaking changes. Sanctioned exception: `Partition` gains the `converged` status flag (FR-012) with an updated constructor; all in-workspace call sites are migrated in the same change (T003b). No other public API changes are permitted. Additionally, the optimized implementation MUST be at least as fast as the unoptimized implementation on all input graphs, measured under the same conditions (same hardware, same seed, same graph). Performance is a backward-incompatible change.

- **FR-008**: The system MUST handle edge cases: empty graphs, no-edge graphs, fully connected graphs, and graphs with isolated nodes

- **FR-009**: The system MUST produce bitwise-identical community partitions (same membership vector and same Q value to all 17 significant digits of f64) when given the same graph, same config, and same seed, across different compiler versions, operating systems, and CPU architectures. This requires using `ChaCha8Rng` (from the `rand_chacha` crate) as the PRNG, which has a fixed specification (RFC 8439) and is explicitly recommended by the `rand` crate documentation for reproducible, cross-platform deterministic randomness. `StdRng` MUST NOT be used because its algorithm is version-dependent. The `seed` field in `LeidenConfig` is of type `Option<u64>` with default `Some(42)`. The `AlgorithmConfig::seed()` trait method returns `Some(42)` for the default config.

- **FR-010**: The system MUST maintain the connected-community guarantee (all detected communities remain internally connected). Caching optimizations MUST NOT violate this guarantee. A `debug_assert!` in the refinement phase MUST verify connectedness after each refinement pass in debug builds, panicking if a disconnected community is detected. This provides zero-cost-in-release regression detection. **Note:** `debug_assert!` is NOT flagged by `clippy::panic` (verified with clippy 0.1.98), so no `#[allow]` attribute is needed.

- **FR-011**: The system MUST perform periodic full recomputation of cached community statistics after every N incremental updates (where N is a user-configurable parameter of type `u32`, defaulting to 100, valid range N ∈ [1, u32::MAX]) to reset accumulated floating-point error. `LeidenConfig::validate()` MUST reject N=0, as disabling periodic recomputation would allow unbounded floating-point drift. All statistical accumulations MUST use f64 precision. This ensures cached statistics do not drift beyond the partition quality tolerance (SC-005) over thousands of node movements. **Accumulation order**: All cached community statistics (community_degree_sums, community_internal_weights, neighbor cache weights) MUST be accumulated in ascending `NodeId` value order (i.e., sorted by `NodeId::value()` ascending) to ensure deterministic cross-platform results. This is required because floating-point addition is non-associative (Goldberg 1991), and different accumulation orders produce different results across platforms (arXiv:2411.00442). **Recomputation timing**: When the recomputation interval is reached during a local-moving pass, the full recomputation MUST be deferred to the next phase boundary (end of the current Leiden pass, after aggregation) to avoid disrupting the current pass's cache state. This matches the standard practice in incremental graph algorithms (LD-Leiden, leidenalg) where batches are processed atomically.

- **FR-012**: When the algorithm reaches the maximum iteration limit without converging, the system MUST return the best partition found so far (highest quality Q) with a warning log indicating non-convergence. The Partition result MUST include a convergence status flag indicating whether the algorithm converged or hit the iteration limit

- **FR-013**: The caching system MUST enforce explicit edge case semantics: (1) self-loops are counted exactly once in community internal weight and delta-Q computation, matching igraph convention; (2) isolated nodes (degree 0) maintain an empty neighbor cache and are skipped during local moving; (3) zero-weight edges contribute 0.0 to neighbor cache weight sums and are excluded from community weight aggregation; (4) single-node communities without self-loops contribute `−(k_i/2m)²` to modularity Q (degree tax); (5) isolated nodes (k=0) contribute exactly 0 to Q; (6) empty graph (m=0) returns Q=0 (avoids division by zero); (7) self-loops in singleton communities count as internal weight (`Σ_in += w_ii`), potentially offsetting the degree tax.

### Key Entities *(include if feature involves data)*

- **LocalMoveState**: Cached statistics for the local moving phase, including node degrees, community degrees, community internal weights, community sizes, and per-node neighbor weight cache

- **NeighborCache**: Per-node lazy cache mapping community IDs to edge weights, invalidated when the node being processed changes

- **ConvergenceState**: Tracks iteration count, consecutive iterations below `convergence_threshold`, and nodes moved in current iteration for early termination decisions. Convergence is detected when zero nodes moved during a complete local-moving pass.

#### LeidenConfig Fields

| Field | Type | Default | Unit | Description | Valid Range |
|-------|------|---------|------|-------------|-------------|
| `gamma` | f64 | 1.0 | dimensionless | Resolution parameter controlling community size scale | ≥ 0 |
| `beta` | f64 | 0.01 | dimensionless | Refinement randomness (θ parameter for randomized merging) | [0, 1] |
| `convergence_threshold` | f64 | 1e-6 | absolute quality change (\|ΔQ\|) | Minimum quality improvement to continue iterating | ≥ 0 |
| `convergence_mode` | `ConvergenceMode` | `Absolute` | — | Convergence comparison mode (Absolute = \|ΔQ\|, Relative = \|ΔQ\|/\|Q\|) | `Absolute` or `Relative` |
| `max_iterations` | `usize` | 10 | iterations | Maximum number of Leiden passes before forced termination | ≥ 1 |
| `recompute_interval` | u32 | 100 | incremental updates | Periodic full recomputation interval to reset FP drift | [1, u32::MAX] |
| `seed` | `Option<u64>` | `Some(42)` | PRNG seed | Deterministic seed for cross-platform bitwise-identical results (FR-009) | any u64 or `None` |

`LeidenConfig::validate()` MUST enforce these ranges and return `AlgorithmError::InvalidConfiguration` on violation.

#### Field Interactions

The following boundary behaviors MUST be documented and tested:

| Field | Boundary | Behavior |
|-------|----------|----------|
| `convergence_threshold` | = 0 | Effectively disables quality-based early termination; only zero-nodes-moved detection (FR-003) and `max_iterations` limit apply |
| `max_iterations` | = 1 | Exactly one Leiden pass (local-moving → refinement → aggregation); convergence checks irrelevant for termination |
| `gamma` | = 0 | Q = Σe_c only (no null model); all-in-one-community becomes optimal |
| `beta` | = 0 | Greedy maximum-improvement selection during refinement (deterministic) |
| `beta` | = 1 | Maximum randomness: uniform random selection among eligible communities |
| `seed` | any u64 or `None` | Determines node processing order; same seed produces bitwise-identical results across platforms (FR-009) |
| `convergence_threshold` × `max_iterations` | threshold very small, max_iterations reached first | Plateau event, not convergence; FR-012 returns best partition with warning |
| `gamma` × `convergence_threshold` | large gamma → large Q values | Absolute convergence mode may trigger too early; consider relative mode |

---

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: PolBooks dataset (105 nodes) completes in under 5 seconds (currently >30s timeout)

- **SC-002**: PolBlogs dataset (1,490 nodes) completes in under 30 seconds (currently >30s timeout)

- **SC-003**: Karate Club dataset (34 nodes) completes in under 100 milliseconds

- **SC-004**: NetScience dataset (1,589 nodes) completes in under 10 seconds

- **SC-005**: Partition quality (modularity Q) remains within 1e-4 absolute epsilon of the non-optimized implementation on all reference graphs. This threshold is 100× larger than the convergence threshold (1e-6), providing margin for accumulated floating-point divergence across cached incremental updates, while remaining statistically insignificant for community structure interpretation (typically insensitive to Q differences below 0.001).

- **SC-006**: All detected communities remain internally connected (verified via BFS traversal)

- **SC-007**: Early termination triggers correctly on converged graphs, reducing median iteration count by at least 50% on easy-to-converge inputs (measured over 30 runs, consistent with the Measurement Methodology). Pass criterion: median_optimized ≤ 0.5 × median_baseline. **Baseline**: same algorithm, same seed, early termination disabled (`convergence_threshold=0` so |ΔQ| < 0 is never satisfied, disabling quality-based termination), running until zero nodes moved (standard Leiden convergence) or `max_iterations` limit. This matches leidenalg's `n_iterations=-1` behavior and the standard unoptimized baseline in benchmarking literature. **"Easy-to-converge"** graphs are defined as those where the baseline converges in ≤20 iterations (e.g., LFR benchmarks with μ ≤ 0.3).

- **SC-008**: Memory usage does not exceed O(V + E) where V is node count and E is edge count. Peak heap usage is measured using the `dhat` crate's heap usage testing mode (`dhat::HeapStats::max_bytes`), with the assertion that `max_bytes ≤ c·(V + E)` for a documented constant `c` determined by the CSR graph representation (2·E·sizeof(u32) + (V+1)·sizeof(u32) for offsets + indices).

---

## Measurement Methodology

All performance targets (SC-001 through SC-004) MUST be measured under the following conditions:

- **Build mode**: `cargo build --release` with workspace `profile.release` settings
- **Threading**: Single-threaded measurement (pin to core 0 via `taskset -c 0`)
- **Warm-up**: Discard first 5 runs; measure runs 6-35
- **Statistic**: Report median wall-clock time; pass criterion is median ≤ target
- **Reference hardware**: Reference class: ≥4 CPU threads, ≥16 GB RAM. Measured on: AMD Ryzen 7 3700U (4C/8T, 2.0 GHz base), 13 GB RAM. Targets scale linearly for faster hardware: pass = median ≤ target × (reference_threads / actual_threads)
- **Memory**: ≥ 16 GB DDR4-3200
- **OS**: Linux (kernel 5.15+)
- **Rust**: Latest stable toolchain at time of measurement

### Dataset Sources

The following canonical dataset sources MUST be used for performance benchmarking to ensure reproducibility:

| Dataset | Nodes | Edges | Canonical Source | Format | Citation |
|---------|-------|-------|------------------|--------|----------|
| PolBooks | 105 | 441 | [Network Repository](https://networkrepository.com/polbooks.php) | MTX | Krebs, V. (unpublished); Newman, M.E.J. (2006). *Phys Rev E*, 74(3), 036104 |
| PolBlogs | 1,490 | 19,090 | [KONECT](http://konect.cc/networks/dimacs10-polblogs/) (undirected version) | TSV | Adamic, L.A. & Glance, N. (2005). *Proc. 3rd Int. Workshop on Link Discovery*, 36–43. ACM |
| Karate Club | 34 | 78 | [KONECT](http://konect.cc/networks/ucidata-zachary/) | TSV | Zachary, W.W. (1977). *J Anthropol Res*, 33, 452–473 |
| NetScience | 1,589 | 2,742 | [Newman's Data](https://websites.umich.edu/~mejn/netdata/) (full graph) | GML | Newman, M.E.J. (2006). *Phys Rev E*, 74(3), 036104 |

**⚠️ Version Conflicts:**
- **PolBlogs**: The 1,490-node undirected version is standard. KONECT hosts a 1,224-node directed version (33,430 edges). Network Repository hosts a 643-node sampled subgraph. Use the undirected version for benchmarks.
- **NetScience**: The original GML from Newman's page has 1,589 nodes (full graph). Network Repository hosts a 379-node LCC-only version. Use the full graph for benchmarks.
- **LFR Benchmarks**: Pre-generated datasets available on [Zenodo](https://zenodo.org/records/4450167) (Lancichinetti et al., 2008. *Phys Rev E*, 78(4), 046110).

---

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
-->

- The existing `delta_q` function signature and mathematical formulation will be preserved; only the caching mechanism changes
- The existing refinement phase (CPM/modularity optimization) will remain structurally unchanged
- The aggregation phase (graph reduction) will remain structurally unchanged
- The quality function dispatch (`QualityFunction` enum) will remain unchanged
- The `GraphView` trait interface will remain unchanged
- The `Partition` struct gains a `converged: bool` field (FR-012); existing fields and behavior are unchanged
- The CLI and TUI interfaces will not require changes (performance improvement is transparent)
- The existing test suite (Tier 1 reference graphs) will continue to pass without modification. A performance test tier MUST validate both correctness (partition quality within 1e-4 epsilon) and performance (wall-clock time within SC-001 through SC-004 targets) on Tier 1 reference graphs.
- The `FxHashMap` (rustc-hash) is acceptable for the neighbor cache (already used elsewhere in the codebase)
- The queue-based fast local move (Part C) is optional and may be deferred to a future iteration if Part A + B achieve sufficient speedup

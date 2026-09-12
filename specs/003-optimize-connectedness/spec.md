# Feature Specification: Optimize Leiden Connectedness Check

**Feature Branch**: `003-optimize-connectedness`

**Created**: 2026-09-09

**Status**: Ready (Clarifications CLOSED 2026-09-10; checklist review complete; 2 clarification answers integrated — performance gate binding + protocol split; 3 minor normative-text edits applied: SC-005 epsilon 1e-10, SC-006 framework = `proptest`, SC-008 pinning clarified)

**Input**: User description: "@research/leiden-connectedness-optimization.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Faster Community Detection on Dense Graphs (Priority: P1)

As a user running community detection on dense real-world networks, I want the Leiden algorithm to complete in reasonable time (see SC-001, SC-002, SC-003) on graphs with high average degree, so that I can analyze networks like PolBlogs, NetScience, and similar dense graphs without performance degradation.

**Why this priority**: The current BFS-based connectedness check causes super-linear scaling on dense graphs, making the framework impractical for real-world use cases it was designed for.

**Independent Test**: Can be fully tested by running the Leiden algorithm on LFR benchmark graphs with mixing parameter μ ≥ 0.5 and average degree ≥ 20, and verifying completion time scales sub-quadratically (target near-linear) with graph size per SC-001 / SC-002 / SC-003 (SC-003's two-point validation).

**Acceptance Scenarios**:

1. **Given** a graph with 10,000 nodes and average degree 50, **When** running Leiden community detection, **Then** the algorithm completes in under 5 seconds
2. **Given** a graph with 50,000 nodes and average degree 10, **When** running Leiden community detection, **Then** the algorithm completes in under 5 seconds
3. **Given** the PolBlogs network (1,490 nodes, 19,090 edges), **When** running Leiden community detection, **Then** the algorithm completes in under 500 milliseconds

---

### User Story 2 - Maintain Connected Community Guarantee (Priority: P1)

As a researcher relying on the framework's correctness guarantees, I want every detected community to remain internally connected, so that my analysis results are mathematically valid per the Leiden algorithm's proven properties.

**Why this priority**: The constitution (Principle I) mandates that "The Leiden engine MUST produce internally connected communities." Any optimization must preserve this guarantee.

**Independent Test**: Can be fully tested by running property-based tests that verify every community in the output partition is internally connected via BFS/DFS traversal.

**Acceptance Scenarios**:

1. **Given** any valid input graph, **When** the Leiden algorithm completes, **Then** every community in the resulting partition is internally connected
2. **Given** a graph with known community structure, **When** running Leiden detection, **Then** no community contains disconnected subgraphs
3. **Given** edge cases (empty graphs, singletons, disconnected cliques), **When** running Leiden detection, **Then** all communities are trivially connected

---

### User Story 3 - Align with Reference Implementation Design (Priority: P2)

As a developer maintaining the codebase, I want the connectedness check to follow the Leiden paper's design rather than an ad-hoc BFS, so that the implementation is architecturally sound and matches the reference implementations (libleidenalg, igraph).

**Why this priority**: The current BFS approach is not used by any reference implementation. Aligning with the standard design reduces maintenance burden and ensures correctness by construction.

**Independent Test**: Can be fully tested by verifying that the refinement phase starts from singleton communities and only allows well-connected isolated vertices to move — matching the paper's `MergeNodesSubset` procedure including its γ-connectivity precondition.

**Acceptance Scenarios**:

1. **Given** the refinement phase starts, **When** the algorithm initializes, **Then** every node begins in its own singleton community
2. **Given** a node is not in a singleton community during refinement, **When** the algorithm considers moves, **Then** the node is not eligible to move
3. **Given** the refinement phase completes, **When** the algorithm proceeds to aggregation, **Then** all communities are internally connected by construction

---

### Edge Cases

- What happens when a community has only one node? (Trivially connected, no check needed)
- What happens when a node has no edges to its current community? (Should not be in that community; refinement handles this)
- What happens when the graph has zero edges? (All singletons, trivially connected)
- What happens when a community is a complete graph? (Removing any node keeps it connected)
- What happens when a node is an articulation point? (The refinement phase's isolated-vertex-only design prevents disconnection)

## Clarifications

### Session 2026-09-09

- Q: Should the optimization include parallelizing the refinement phase using atomic operations? → A: No — single-threaded refinement only. Parallel refinement is an active research problem (first provably correct solution published August 2026). The reference implementation (libleidenalg) is sequential. GVE-Leiden's parallel refinement has known race conditions that violate guarantees. Parallelization can be a follow-up once the single-threaded version is verified.
- Q: When the debug-build connectedness assertion detects a disconnected community, what should happen? → A: Panic with descriptive message using `debug_assert!`. Matches existing codebase convention (cache consistency checks use same pattern) and constitution's zero-panic-in-production principle (debug_assert is debug-only). (Refined by Session 2026-09-09 continued: confirmed fail-fast over recovery via research — see `research/leiden-disconnected-handling.md`.)
- Q: Should the "isolated vertices only" refinement approach apply uniformly to all quality functions? → A: Yes — uniform behavior. Reference implementation's `move_nodes_constrained()` uses identical refinement flow for all quality functions, dispatching only on `diff_move()`. Quality function affects which moves are accepted, not which nodes are eligible.
- Q: The spec states "currently ~10+ seconds" for d=50, n=10,000 but benchmarks show ~2 seconds. Should the baseline be updated? → A: Yes — update SC-001 to reflect actual baseline of ~2 seconds. Optimization target should be meaningful improvement (e.g., <500ms for same graph).
- Q: Should this optimization apply only to Leiden or also to other algorithms? → A: Leiden only — scope is naturally bounded. Codebase search confirms `would_remain_connected` and `verify_communities_connected` are only used in Leiden (local_moving.rs, refinement.rs). No other algorithm has similar BFS checks.

### Session 2026-09-09 (b): Performance Targets and Test Infrastructure

- Q: US1 Acceptance Scenario 1 says "completes in under 5 seconds" while SC-001 says "under 500 milliseconds" for the same graph profile — which is the binding target? → A: B — Keep Scenario 1 at ≤ 5s as the minimum acceptance gate; treat SC-001's ≤ 500ms as the optimization stretch target for dense graphs. Two different purposes: pass/fail baseline vs meaningful improvement measure.
- Q: Which NMI threshold should Tier 2 LFR benchmark tests enforce against ground truth? → A: B — NMI ≥ 0.95 as the governing default per constitution Principle VI; for high-mixing graphs (μ ≥ 0.5) a relaxed threshold of ≥ 0.90 is explicitly permitted with documented rationale. Primary sources confirm: no reference implementation (Traag et al. 2019, Lancichinetti 2008) specifies a rigid NMI bar across all μ levels. At μ = 0.5, half of each node's edges go outside its true community, making NMI ≥ 0.95 unsatisfiable on reasonable graphs regardless of algorithm quality. See `research/leiden-nmi-thresholds.md` for full evidence.

### Session 2026-09-09 (c): Debug Check Granularity

- Q: How often should debug-build connectedness checks (`debug_assert!`) fire during algorithm execution? → A: B — After the refinement phase in each iteration only. The refinement phase contains the new logic introduced by this feature and is the highest-risk phase to verify. Local moving and aggregation remain structurally unchanged from existing code, so checking them repeatedly adds cost without proportional risk reduction.

### Session 2026-09-11 (measurement verification — T086–T087 / T088–T089)

- Q: SC-002 (50k/d=10) and SC-003 (scaling ratio n=10k→50k, both d=50) measurement records are blocked — should the spec document them as deferred with block reasons rather than mask them? → A: A — Document both as deferred/blocking with explicit reasons. SC-002 BLOCKED: edgelist `benchmarks/lfr_50k_d10.edges` missing (parameter JSON `lfr_50k_d10.json` exists; measurement `sc-002-50k-d10.json` records BLOCKED with `median_time_ms` null, `binding_met` false, `block_reason` documented per Measurement Protocol T078). SC-003 BLOCKED: 50k/d=50 release-build execution exceeded 300s timeout so no 5-run median available; ratio `<12.5` cannot be computed (`sc-003-scaling.json` records ratio `null`, `status` BLOCKED). No false-completion claim made; measurement protocol (release, 5-run median, pinned `communal-ref-01`, JSON sidecars with `mu`/`d`/`n`/`seed`/`compiler_version`/`rustc_version`/`float_rounding_mode`) satisfied for SC-001 (3471.2ms, binding met, stretch not met) and SC-009 (85.3ms, binding met). `rustc` discrepancy (measured 1.98.1 vs pinned spec 1.83.0) noted across all records. T088 (inline Theorem 5 / induction comment in `refinement.rs`) and T089 (contract assertions `(f)` in `contracts/leiden-refinement-contract.md`, assertions `a–e` verified in `refinement.rs` lines 79–85 / `local_moving.rs` 778) completed without source contradiction.

### Session 2026-09-09 (d): Scaling Validation Methodology (SC-003)

- Q: How should SC-003's "linear scaling" claim be validated operationally? → A: Modified Option A — Two-point test at n=10k and n=50k; pass if time(50k)/time(10k) < 12.5 (strictly sub-quadratic: quadratic would yield 25×). This matches how Traag et al. (2019), igraph leidenalg, and NetworKit validate scaling (timing comparison without formal regression). Full multi-point regression per `research/leiden-scaling-validation.md` can be added as a future enhancement once communal-benches supports on-the-fly generation. See also `research/leiden-nmi-thresholds.md` Appendix on linear scaling methods.

### Session 2026-09-09 (continued)

- Q: Should the release build also verify community connectedness, or is debug-only verification sufficient? → A: B — Debug-only verification is sufficient. Research confirms no reference implementation (libleidenalg, igraph, GVE-Leiden, leiden_rs) performs runtime connectedness checks in release builds. The Leiden algorithm's connectedness guarantee is proven by construction (Theorem 5; proof in Appendix D.1 of arXiv v3 — the published Scientific Reports HTML contains no appendix content). Adding release-mode checks would reintroduce the performance overhead this optimization removes. Debug builds + property-based tests (SC-006) provide adequate safety.
- Q: Should the optimized algorithm produce identical community assignments to the current implementation for all graphs, or is equivalent quality sufficient? → A: B — Identical assignments for Tier 1 only; equivalent quality for Tier 2/3. Research confirms this is the community standard: the Leiden algorithm is inherently stochastic (randomized node ordering), and no reference implementation tests for exact partition identity on stochastic graphs. Tier 1 graphs have deterministic, well-separated communities where output should be stable. For Tier 2/3, quality equivalence (within floating-point epsilon) plus structural invariants (community count, sizes) and optimality conditions (quality-gain magnitude below the assertion threshold — plan anchor: ≤1e-10) is the correct assertion pattern.
- Q: Should the optimization work with all quality functions or only Modularity and CPM? → A: A — All quality functions must work. Research confirms no reference implementation (libleidenalg, igraph, leiden-rs, leidenalg) has quality-function-specific refinement behavior. The paper's connectedness proof relies only on structural design (singleton start + isolated-vertex-only merges), not on quality function form. Restricting to Modularity/CPM would create a maintenance burden (two code paths) and a confusing API.
- Q: When the debug-build assertion detects a disconnected community, should the algorithm panic or attempt to recover by splitting into connected components? → A: A — Panic with descriptive message (debug_assert). Research confirms no reference implementation (libleidenalg, igraph, GVE-Leiden, leiden_rs, Networkit) has recovery/splitting logic triggered by a debug assertion. The refinement phase already splits disconnected communities as part of its normal operation (singleton-start + isolated-vertex-only design). If a debug_assert detects a disconnected community AFTER refinement, it indicates an implementation bug that must be fixed — recovery logic would mask the bug. Networkit #1244 was resolved by fixing the implementation, not by adding recovery. Research: `research/leiden-disconnected-handling.md`.

### Session 2026-09-10 (e): SC-003 Benchmark Data Persistence

- Q: SC-003's two-point scaling test needs n=10k/d=50 and n=50k/d=50 LFR graphs — should these generated benchmark files be permanently committed? → A: A — Commit to benchmarks/ with recorded LFR parameters per Measurement Protocol. Permanent storage enables reproducible CI/regression testing and aligns with Tier 2 benchmark practice (existing LFR graphs in benchmarks/lfr_graphs/). Generation parameters must be recorded in the file metadata or adjacent spec note.

### Session 2026-09-10

- Q: Which measurement environment and protocol should the performance targets in US1's acceptance scenarios and SC-001/002/003 be evaluated against? → A: A — Pin the reference setup in the spec: release build, LFR benchmark graphs with fixed recorded parameters (μ and average degree per criterion; remaining LFR parameters per `research/leiden-test-parameters.md`), median of at least 5 runs, single pinned reference machine with recorded specs. SC-001 is annotated as a stretch target; the binding acceptance gate for the 10k/d=50 profile is US1 Scenario 1's ≤5 s (per Session 2026-09-09 (b)).

- Q: During refinement, should a singleton node be allowed to merge into a neighboring community only when it also satisfies the Leiden paper's γ-connectivity precondition, or is the singleton restriction alone sufficient? → A: A — Paper/igraph-faithful: singleton-only eligibility plus the γ-connectivity precondition E(v,S−v) ≥ γ·‖v‖·(‖S‖−‖v‖) on every merge (paper Algorithm A.2 `MergeNodesSubset`; igraph implements both conditions; libleidenalg implements only the singleton rule). Corrects the former Assumption that γ-connectivity was implicitly satisfied by the quality function's resolution parameter. Verified against primary sources: `research/leiden-refinement-design-verification.md`.

- Q: How should the spec close the acceptance-criteria gaps for FR-006 (per-quality-function regression), FR-007 (same-seed determinism), and FR-008's release clause? → A: A — Add SC-008 (same-seed-twice determinism), extend SC-005 to name covered quality functions (Modularity + CPM; MapEquation keeps current stub behavior until implemented), and add SC-009 anchoring US1 Scenario 3 (PolBlogs ≤500 ms, Tier 3). FR-008's release clause is accepted via the release-build performance gates: a release build performing runtime connectedness verification cannot meet the Measurement Protocol targets.

- Q: How should the spec reconcile FR-008's debug-only connectedness verification with constitution Principle I's "verifiably connected via BFS/DFS traversal" clause? → A: B — Reconcile + genericize: the spec documents that Principle I's verifiability is satisfied by FR-008's debug-build assertions together with Principle VI's property-based BFS/DFS verification (SC-006), and MUST-level requirements are worded technology-agnostically; concrete implementation anchors (`debug_assert!`, `LocalMoveState`, `diff_move`) become non-normative plan hints. Reference-implementation API citations (e.g., libleidenalg's refinement routine) remain as external evidence.

- Q: Should US1's Independent Test and SC-003 keep saying "scales linearly" when the clarified bar accepts strictly sub-quadratic scaling (ratio < 12.5)? → A: A — Reworded to "sub-quadratic (target near-linear)" in both places; the two-point ratio < 12.5 bar is unchanged (per Session 2026-09-09 (d)).

### Session 2026-09-09 (e): Post-Review Clarifications

- Q: Should FR-004's merge-eligibility rule also include the paper's destination-community filter — a refined community C may only receive a merged node if E(C, S−C) ≥ γ·‖C‖·(‖S‖−‖C‖) — in addition to the node-side singleton + γ-connectivity conditions it already mandates? → A: A — Both filters. Primary-source verification of arXiv v3 Algorithm A.2 confirms the paper defines both R (node-side: singleton + E(v, S−v) ≥ γ·‖v‖·(‖S‖−‖v‖)) and T (destination-side: C ⊆ S with E(C, S−C) ≥ γ·‖C‖·(‖S‖−‖C‖)); igraph implements both conditions using the quality function's resolution parameter as γ; libleidenalg's singleton-only rule is a documented deviation from the paper, not the standard. γ's provenance is settled by the paper: "γ refers to the resolution parameter in the quality function that is optimised" — not a separate refinement parameter. See `research/leiden-merge-nodes-subset-verification.md`.
- Q: Which graph profiles should SC-003's two-point scaling comparison use at n=10k and n=50k, given the committed corpus's only 10k-node graph has average degree ≈19 and no 50k-node graph exists? → A: Both points at d=50 — regenerate LFR benchmark graphs at n=10k/d=50 and n=50k/d=50 with recorded parameters per the Measurement Protocol. This matches SC-003's "dense graphs (d=50)" wording and US1's Independent Test premise ("average degree ≥ 20"); SC-002's 50k/d=10 profile remains a separate no-regression gate, never inputs to the SC-003 ratio.
- Q: How should the spec handle the baseline timing claims "currently ~2 seconds" (SC-001) and "currently ~2.7 seconds" (SC-002), given the provenance investigation found both numbers unsourced? → A: A — Strike the unsourced numbers; state the baseline qualitatively (measured >30s timeouts on graphs as small as 105 nodes; super-linear scaling) as documented in AGENTS.md and `research/leiden-connectedness-optimization.md`; the actual pre-optimization timings will be recorded at the Measurement-Protocol run during implementation. The ≤500ms, ≤5s, and <12.5 targets themselves remain unchanged.
- Q: Should the MapEquation stub clause (FR-006) have a pinning criterion to make it enforceable, or should it be downgraded to a note? → A: A — Pin it. Add a concrete acceptance criterion: MapEquation stub quality returns 0.0 and refinement skips it; this is tested via the general quality-function test suite (SC-005, which explicitly covers Modularity and CPM; MapEquation stub must match that exact behavior until MapEquation is implemented). This makes FR-006 enforceable.
- Q: Should the specification be rewritten for non-technical stakeholders, or is the technical audience appropriate? → A: Technical stakeholders (recommended). The domain (graph algorithms, μ/NMI/γ terminology, BFS/DFS, R/T filter conditions) presupposes technical fluency; the user stories carry the plain-language value narrative for broader context. The checklist item "Written for non-technical stakeholders" remains intentionally unchecked per the spec's intended audience.

- Q: Should SC-009 (PolBlogs timing) include quality/structure invariants to encode the "equivalent quality" decision from Session 2026-09-09 (continued), or keep timing-only? → A: A — Extend SC-009 with quality/structure invariants: PolBlogs must also produce modularity Q within 1e-10 of a one-time pre-optimization baseline (measured during implementation) and a consistent community count. This encodes the "equivalent quality" decision with an enforceable criterion, while still permitting stochastic partition differences per the Leiden algorithm's design (no requirement on exact partition equality). The epsilon value 1e-10 is now normative.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST remove the BFS-based `would_remain_connected` check from the local moving phase entirely
- **FR-002**: System MUST remove the BFS-based `would_remain_connected` check from the refinement phase
- **FR-003**: System MUST initialize the refinement phase with each node in its own singleton community (matching the paper's `P_refined` initialization)
- **FR-004**: System MUST restrict refinement-phase moves to only isolated vertices (nodes in singleton communities within the refined partition), and a node MUST additionally satisfy the paper's γ-connectivity precondition before merging: within the community S being refined, only nodes v with E(v, S−v) ≥ γ·‖v‖·(‖S‖−‖v‖) are eligible to merge, and only destination communities C with C ⊆ S and E(C, S−C) ≥ γ·‖C‖·(‖S‖−‖C‖) are eligible to receive a merge — the paper's R and T sets, both enforced (paper Algorithm A.2; igraph implements both conditions; libleidenalg's singleton-only rule is a documented deviation). γ is the resolution parameter of the quality function being optimized (modularity or CPM), not a separate refinement parameter. Refinement MUST remain single-threaded (no parallel atomic operations).
- **FR-005**: System MUST guarantee that all communities in the final partition are internally connected
- **FR-006**: System MUST maintain the existing quality function behavior for all quality functions without regression. The optimization MUST apply uniformly to all quality functions — no quality-function-specific code paths. Regression acceptance covers Modularity and CPM (SC-005); MapEquation stub behavior is pinned to exact current behavior (quality returns 0.0 and refinement skips it; tested via SC-005's general test suite until MapEquation is implemented).
- **FR-007**: System MUST preserve the deterministic behavior of the algorithm (same seed produces same result)
- **FR-008**: System MUST verify community connectedness via debug-build-only assertions (a BFS/DFS check after the refinement phase in each iteration, using the implementation language's standard debug-only assertion facility — plan anchor: `debug_assert!`). Verification procedure embedded: `grep verify_communities_connected` must find assertions only inside `#[cfg(debug_assertions)]`; `cargo build --release` must contain zero runtime connectivity traversal; frequency = after refinement per iteration. Release builds MUST NOT perform runtime connectedness verification — this aligns with all reference implementations (libleidenalg, igraph, GVE-Leiden, leiden_rs) which rely solely on the by-construction guarantee. Constitution reconciliation (Session 2026-09-10): Principle I's "verifiably connected via BFS/DFS traversal" is satisfied by FR-008's debug-build assertions together with Principle VI's property-based BFS/DFS verification (SC-006); conflict-resolution rule 1 (mathematical correctness over performance) is honored because verification effort is invested at development/test time, not in release hot paths.

### Key Entities

- **Refinement Partition**: A partition where each node starts in its own community and only isolated vertices can move. Guarantees connectedness by construction.
- **Community Bound**: The community assignment from the local moving phase that constrains which communities a node can join during refinement.
- **Isolated Vertex**: A node that is in a singleton community within the refined partition. Only these vertices are eligible to move during refinement.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Leiden algorithm on a 10,000-node graph with average degree 50 completes in under 500 milliseconds (stretch target) / under 5 seconds (binding acceptance gate; see US1 Scenario 1). Measurement record must label `binding_target_ms` (5000) and `stretch_target_ms` (500) with `which_met` flag. Measured per the Measurement Protocol below.
- **SC-002**: Leiden algorithm on a 50,000-node graph with average degree 10 completes in under 5 seconds.
- **SC-003**: Scaling behavior for dense graphs (d=50) improves from super-linear to sub-quadratic (target near-linear aspirational; enforceable bar: ratio < 12.5) with respect to number of nodes. Validated by two-point runtime comparison between LFR benchmark graphs at n=10k and n=50k (both d=50, committed to benchmarks/ with recorded parameters per Session 2026-09-10 (e)), both at average degree 50; pass if time(50k)/time(10k) < 12.5 (strictly sub-quadratic). - Multi-point regression per `research/leiden-scaling-validation.md` may supersede this in a future enhancement once communal-benches supports on-the-fly graph generation. (Acknowledged; not blocking.)
> Note: SC-001 / SC-003 use a two-value clarification (binding gate + stretch target) settled in Session 2026-09-09 / 2026-09-10 — duplication/non-blocking; see SC-001 L148 for the dual-value pattern reference.
- **SC-004**: All existing Tier 1 reference graph tests continue to pass with identical community assignments and quality scores (deterministic graphs with well-separated communities — output is stable)
- **SC-005**: Quality regression: All test graphs (Tier 1 deterministic, Tier 2 LFR, Tier 3 real-world) produce modularity Q within floating-point epsilon 1e-10 (normative) of pre-optimization baseline. For LFR benchmarks against ground truth: Normalized Mutual Information ≥ 0.95 per constitution Principle VI; for high-mixing graphs (μ ≥ 0.5) a relaxed threshold of ≥ 0.90 applies with documented rationale (`research/leiden-nmi-thresholds.md`). The lower bar for μ ≥ 0.5 reflects empirical reality: at half-inter-community edge density, no reference implementation achieves NMI ≥ 0.95 across all seeds. Quality-function coverage: Modularity and CPM. Measured per the Measurement Protocol below.
- **SC-006**: Property-based tests (framework: `proptest`; seed-recorded random instance generation) verify 100% of communities are internally connected via BFS/DFS traversal across 1,000 random graph instances (Principle VI verification mechanism, supporting FR-008's constitution reconciliation; verification mode: debug/test builds only, after refinement per iteration, not release)
- **SC-007**: Merged into SC-005 — quality regression unified.
- **SC-008**: Running the Leiden algorithm twice on the same graph with the same seed, same compiler version, and same platform/float-rounding mode produces byte-for-byte identical partitions (membership and quality) — determinism per FR-007 bounded to reproducible environment
- **SC-009**: Real-world Tier 3 networks complete without regression: PolBlogs (1,490 nodes, 19,090 edges) completes in under 500 milliseconds and produces modularity Q within 1e-10 of a one-time pre-optimization baseline (measured during implementation) with consistent community count — the "equivalent quality" decision from Session 2026-09-09 (continued), with no requirement on exact partition equality due to Leiden's stochasticity; measured per the Measurement Protocol below.
- **Measurement Protocol** (two-mode):
  - **Timing** (release build; governs SC-001/SC-002/SC-003/SC-009 and US1 scenarios 1–3): release build; synthetic gates use LFR benchmark graphs with fixed, recorded parameters (mixing parameter μ and average degree as stated in each criterion; remaining LFR parameters per `research/leiden-test-parameters.md`), generated benchmark files (e.g., SC-003's n=10k/d=50 and n=50k/d=50) committed to benchmarks/ with JSON sidecar metadata (`graph.json` recording `mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode` — per Session 2026-09-10 (f); measurement record must also include `machine_specs`, `date`, `runs` (≥5), `median_ms`, real-world gates use the committed benchmark corpus files; result = median of at least 5 runs on a single pinned reference machine whose specifications are recorded alongside the results.
  - **Verification** (debug/test build; governs FR-008 and SC-006): after refinement phase in each iteration; property-based framework (`proptest`; equivalent only if `proptest` unavailable) over 1,000 random instances; BFS/DFS traversal verification; frequency: per-iteration after refinement; release builds MUST NOT run verification (`grep verify_communities_connected` outside `#[cfg(debug_assertions)]` passes; `--release` build must contain no runtime connectedness check).

## Assumptions

- The refinement phase's "isolated vertices only" design with the γ-connectivity precondition (FR-004) correctly prevents disconnected communities (proven by the paper's Theorem 5, which establishes γ-connectedness — a stronger property than plain connectivity)
- The local moving phase does not require explicit connectedness checks (the refinement phase handles this)
- The existing local-moving cache infrastructure (`communal-algo/src/leiden/local_moving.rs`, `LocalMoveState`) can be reused for the optimized implementation (plan anchor: reuse the current `LocalMoveState` cache state type rather than introducing a new one)
- The γ-connectivity precondition — node-side R and destination-side T — is enforced explicitly during refinement (FR-004); the numerical value of γ is the resolution parameter of the quality function being optimized, per the paper ("γ refers to the resolution parameter in the quality function that is optimised"). It is not satisfied implicitly and not a separate refinement knob (corrected by Session 2026-09-10; destination filter and γ provenance settled by Session 2026-09-09 (e); primary-source verification in `research/leiden-refinement-design-verification.md` and `research/leiden-merge-nodes-subset-verification.md`)
- Debug-build assertions (BFS/DFS verification) are sufficient for development-time correctness checking. Research confirms this is the standard approach: no reference implementation performs runtime connectedness checks in release builds.
- The existing test suite covers all correctness requirements and will catch any regressions

### Session 2026-09-10 (clarified — checklist review)

- Q: Performance binding/gate for SC-001 (≤500 ms vs ≤5 s)? → A: B — ≤5 s binding (US1 Scenario 1); ≤500 ms stretch target.
- Q: Measurement Protocol split (Timing vs Verification) to close CHK008/CHK027/CHK011? → A: B — Two-mode header: Timing (release, 5-run median, pinned machine); Verification (debug/test, after-refinement per Session 2026-09-09 (c), property-based 1,000 instances, framework named Proptest/equivalent).

### Session 2026-09-10 (f): Committed Benchmark Metadata Format

- Q: Which metadata format should accompany committed SC-003 LFR benchmark graphs (n=10k/d=50, n=50k/d=50) in benchmarks/? → A: A — JSON sidecar (`graph.json`) recording μ, average degree d, node count n, seed, and generator version. Enables reproducible CI/regression without parsing binary edgelist headers; aligns with existing benchmarks/lfr_graphs/ practice.

## Resolution

Clarification session complete (1 question asked, 1 answer integrated). Benchmark file format settled: JSON sidecar per `A` (e.g., `benchmarks/lfr_10k_d50.json` alongside edgelist). Measurement Protocol updated to specify sidecar contents (`mu`, `avg_degree`, `n_nodes`, `seed`, `generator_version`). Specification status: Clarifications CLOSED; ready to proceed to `/speckit-plan`.

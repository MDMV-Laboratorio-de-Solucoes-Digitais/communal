# Phase 0 Research: Leiden Connectedness Optimization

**Date:** 2026-09-10 | **Branch:** `003-optimize-connectedness` | **Status:** COMPLETE (all NEEDS CLARIFICATION resolved)

---

## Decisions

### D1 — Remove BFS from local-moving (architecturally required)
- **Decision:** Delete the `would_remain_connected` call at `local_moving.rs:75`. No connectivity guard during local moves.
- **Rationale:** The Leiden paper (Traag et al. 2019, Appendix A.2) defines local moving with only `diff_move` quality improvement; no traversal check. All reference implementations (`libleidenalg`, `igraph` `leiden_fastmove_vertices()`, `leiden_rs`) confirm zero connectivity checks here. The guarantee is refinement's responsibility, not local-moving's.
- **Alternatives:** Early-exit heuristics (degree ≤1, size ≤2) — rejected because they don't address the architectural error and add branching overhead.

### D2 — Remove BFS from refinement; replace with singleton-start + γ-filter
- **Decision:** Refine only isolated vertices (nodes in singleton communities within the refined partition) and enforce both R (node-side: singleton + E(v,S−v) ≥ γ·‖v‖·‖S−v‖) and T (destination-side: C ⊆ S + E(C,S−C) ≥ γ·‖C‖·‖S−C‖) filters per Algorithm A.2.
- **Rationale:** Paper proves (Theorem 5, Appendix C.1 / D.1) that singleton-only merges with the well-connectedness threshold guarantee connected communities by construction. `igraph`'s `leiden_merge_vertices()` uses the arithmetic γ-condition, not BFS. `libleidenalg`'s `merge_nodes_constrained()` uses singleton-only (documented deviation from full R+T; we implement both per Session 2026-09-09 (e)).
- **Alternatives:** Articulation-point caching, DSU — both O(k+ec) rebuild cost; exceed benefit for this optimization.

### D3 — γ provenance = quality function resolution parameter
- **Decision:** γ is `quality_function`'s resolution parameter (modularity γ or CPM γ), not a separate refinement knob.
- **Rationale:** Paper: "γ refers to the resolution parameter in the quality function that is optimised". `igraph` uses the same `resolution` value for both `diff_move` and merge-filter arithmetic. No separate parameter exists in any reference.

### D4 — Debug-only verification (`debug_assert!` after refinement per iteration)
- **Decision:** One `debug_assert!(verify_communities_connected(...))` after the refinement phase in each iteration; none in release.
- **Rationale:** No reference implementation checks at runtime in release (`leiden-rs`, `libleidenalg`, `igraph`, GVE-Leiden, Networkit #1244 resolved by fixing implementation, not adding recovery). Constitution reconciliation (Session 2026-09-10): Principle I's verifiability satisfied by FR-008 + Principle VI property tests (SC-006). `debug_assert!` matches existing cache-consistency convention.
- **Note:** Fail-fast (panic with descriptive message) — recovery/splitting logic masked bugs in past (Networkit #1244).

### D5 — Refinement remains single-threaded
- **Decision:** No parallel atomic operations in refinement; sequential `select_target_community` only.
- **Rationale:** First provably-correct parallel refinement published August 2026; GVE-Leiden has documented race conditions violating guarantees. `libleidenalg` is sequential. Spec clarification (Session 2026-09-09) confirms.

### D6 — Uniform across all quality functions (no special-casing)
- **Decision:** R+T filters and singleton-only eligibility apply identically to Modularity, CPM, MapEquation stub.
- **Rationale:** Reference implementations (`libleidenalg` `move_nodes_constrained`, `igraph`) dispatch only on `diff_move`; structural eligibility is quality-agnostic. Restricting to Modularity/CPM would create two code paths and confusing API.

### D7 — MapEquation stub pinned (quality 0.0, refinement skips)
- **Decision:** SC-005/SC-007 cover Modularity + CPM only; MapEquation behaves as stub (`return 0.0`, skip refinement moves) — verified by general test suite.
- **Rationale:** MapEquation is unimplemented; pinning prevents accidental regression. Added concrete criterion per Session 2026-09-10.

### D8 — Scaling validation = two-point ratio, not regression
- **Decision:** SC-003 passes if time(50k,d=50)/time(10k,d=50) < 12.5 (strictly sub-quadratic vs quadratic 25×).
- **Rationale:** Matches Traag et al. 2019, `igraph`, NetworKit validation methodology. Full regression requires `communal-benches` on-the-fly generation (future enhancement per `research/leiden-scaling-validation.md`). Benchmark files committed with recorded μ/avg-degree metadata.

### D9 — Baseline claims struck (unsourced)
- **Decision:** Remove "~2 seconds" / "~2.7 seconds" from spec; state baseline qualitatively (AGENTS.md shows >30s timeouts at 105 nodes / 441 edges; super-linear scaling proven by current BFS cost).
- **Rationale:** Provenance investigation found both numbers unsourced (Session 2026-09-09). Actual pre-opt timing recorded at Measurement-Protocol run.

### D10 — Equivalence testing: identical Tier 1; equivalent quality Tier 2/3
- **Decision:** Tier 1 (deterministic, well-separated) requires byte-for-byte partitions; Tier 2/3 require Q within 1e-10 + consistent community count + structural invariants (not exact partition, due to stochastic node ordering).
- **Rationale:** Standard across all reference implementations; Leiden is stochastic. 1e-10 epsilon is normative per Session 2026-09-09 (continued) / 2026-09-10.

---

## Resolved Clarifications (All Spec Sessions)

| Session | Q | Answer (recorded) |
|---|---|---|
| 2026-09-09 | Parallel refinement? | No — sequential only; parallel has race conditions |
| 2026-09-09 | Debug assertion failure → recover or panic? | Panic (`debug_assert!`) — fail-fast; recovery masks bugs |
| 2026-09-09 | Quality-function-specific refinement? | No — uniform; only `diff_move` dispatches |
| 2026-09-09 | Baseline timing update? | Strike unsourced; qualitative baseline; opt targets unchanged |
| 2026-09-09 (b) | SC-001 vs Scenario 1 target? | Scenario 1 ≤5s = binding gate; SC-001 ≤500ms = stretch |
| 2026-09-09 (b) | NMI threshold? | ≥0.95 default; ≥0.90 permitted for μ ≥ 0.5 with rationale |
| 2026-09-09 (c) | Debug check frequency? | After refinement phase only (highest-risk new logic) |
| 2026-09-09 (d) | SC-003 linear claim? | Reworded sub-quadratic; ratio < 12.5 unchanged |
| 2026-09-09 (continued) | Release-mode connectedness check? | No — debug only; by-construction guarantee |
| 2026-09-09 (continued) | Identical assignments? | Tier 1 identical; Tier 2/3 equivalent (within epsilon) |
| 2026-09-09 (continued) | Quality-function scope? | All quality functions (uniform) |
| 2026-09-09 (continued) | Panic vs split on disconnect? | Panic (`debug_assert!`) |
| 2026-09-10 (e) | Benchmark persistence? | Commit to `benchmarks/` with recorded LFR params |
| 2026-09-10 | Measurement protocol? | Release; LFR fixed params; 5-run median; pinned machine; committed files |
| 2026-09-10 | R + T destination filter? | Both — paper Algorithm A.2 defines both sets |
| 2026-09-10 | Scaling profiles (10k/50k d=50)? | Both d=50; regenerate; SC-002 d=10 separate |
| 2026-09-10 | Unsource baseline numbers? | Strike; record at run |
| 2026-09-10 (e) | MapEquation stub pin? | Add concrete criterion (stub behavior pinned) |
| 2026-09-10 | SC-009 quality invariants? | Add Q within 1e-10 + consistent count + structural |

---

## Sources Verified (Primary)

- Traag, Waltman, van Eck (2019) *Scientific Reports 9:5233* — Theorem 5, Algorithm A.2 (MergeNodesSubset), Appendix C.1 / D.1
- `libleidenalg` (`Optimiser.cpp`: `move_nodes()`, `merge_nodes_constrained()`) — zero connectivity checks
- `igraph` (`src/community/leiden.c`: `leiden_fastmove_vertices()`, `leiden_merge_vertices()`) — arithmetic γ-filter
- `leiden-rs` (docs.rs) — sequential, no per-move DFS
- `research/leiden-refinement-design-verification.md`, `leiden-merge-nodes-subset-verification.md` — R+T filter verification
- `research/leiden-disconnected-handling.md` — fail-fast over recovery (Networkit #1244)

No remaining NEEDS CLARIFICATION.

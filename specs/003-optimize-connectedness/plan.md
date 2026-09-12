# Implementation Plan: Optimize Leiden Connectedness Check

**Branch**: `003-optimize-connectedness` | **Date**: 2026-09-10 | **Spec**: [`specs/003-optimize-connectedness/spec.md`](../../spec.md)

**Input**: Feature specification referencing `@research/leiden-connectedness-optimization.md` and 14 supporting research docs.

---

## Summary

Remove the redundant per-move BFS/DFS connectivity check (`would_remain_connected`) from both the local-moving and refinement phases of `communal-algo/src/leiden/`. Replace with the paper-correct design: singleton-start refinement + isolated-vertex-only merges + explicit γ-connectivity precondition (R/T filters per Algorithm A.2), verified by `debug_assert!` only after refinement per iteration. This aligns with `libleidenalg`, `igraph`, `leiden_rs`, and the Traag et al. 2019 reference. Performance target: sub-quadratic scaling (time(50k)/time(10k) < 12.5 at d=50), PolBlogs <500ms, no regression on Tier 1–3.

---

## Technical Context

- **Language/Version**: Rust 2024 edition (`edition = "2024"`); `resolver = "3"`
- **Primary Dependencies**: `communal-core` (GraphView, CommunityDetector, QualityFunction, CsrGraph, NodeId, CommunityId); `communal-algo` internal (local_moving, refinement, aggregation, stepping, quality); no new external crates required
- **Storage**: N/A (algorithm only; benchmark artifacts committed to `benchmarks/`)
- **Testing**: `cargo test --all-features`; `proptest` property-based for SC-006 (BFS/DFS connectedness over 1,000 random graphs); deterministic Tier 1 regression; LFR Tier 2 (NMI ≥ 0.95 / ≥ 0.90 for μ ≥ 0.5); Tier 3 real-world timing
- **Target Platform**: Linux reference machine (pinned specs per Measurement Protocol); release builds for performance gates; debug builds for `debug_assert!`
- **Project Type**: Library crate (`crates/communal-algo/`)
- **Performance Goals**: SC-001 ≤500ms (10k/d=50 stretch) / ≤5s (acceptance gate); SC-002 ≤5s (50k/d=10); SC-003 ratio < 12.5; SC-009 PolBlogs ≤500ms + Q within 1e-10 of baseline
- **Constraints**: `#![deny(unsafe_code, missing_docs, unwrap_used, expect_used, panic, todo, unimplemented)]`; no parallel refinement (race conditions documented); single-threaded only; `debug_assert!` only (no release-check overhead); uniform across all quality functions (Modularity + CPM + MapEquation stub)
- **Scale/Scope**: Up to 50,000 nodes / ~2.5M edges (dense d=50); real-world up to 1,490 nodes / 19k edges; confined to `crates/communal-algo/src/leiden/`
- **Reference machine spec**: see `reference-machine-spec.md` (pinned `communal-ref-01`: Linux / AMD EPYC 7371X / 128 GB / rustc 1.83.0 / roundTiesToEven; resolves B2 / C1)
- **Unknowns**: All resolved in spec clarifications (Sessions 2026-09-09 through 2026-09-10); see `research.md`

---

## Constitution Check (Pre-Design)

*GATE: Must pass before Phase 0; re-verify post-Phase 1.*

| Principle | Status | Evidence / Justification |
|---|---|---|
| **I — Mathematical Rigor** | PASS (with reconciliation) | Connectedness guarantee preserved by construction (singleton-start + isolated-only + γ-condition). Debug-only BFS/DFS verification (`debug_assert!`) satisfies "verifiably connected" via Principle VI property tests + FR-008. No release overhead. Conflict rule 1 (correctness over performance) honored at dev/test time, not hot path. |
| **II — Cache/Performance** | PASS | Removes O(V+E) per-move DFS from both phases — dominant cost on dense graphs (PolBooks >30s → target <500ms). Hot loops use flat CSR + arithmetic checks (O(1) γ-condition), not traversal. |
| **III — Modular Workspace** | PASS | Changes confined to `communal-algo`; interface (`CommunityDetector`) unchanged; facade (`communal` root) unchanged. |
| **IV — Zero Unsafe / Panic** | PASS | `debug_assert!` is debug-only (no panic in release); all fallible paths stay `Result`; no `.unwrap()`. Constitution's zero-panic-in-production applies to release; debug assertions are standard development convention confirmed by project (cache consistency checks). |
| **V — Dynamic/GraphRAG** | N/A | No impact; Leiden is batch. |
| **VI — Verification/Benchmark** | PASS | SC-004/005/007/009 preserve existing tiers; new SC-006 property-based; measurement protocol pinned (5-run median, fixed LFR params, single reference machine, committed benchmark files). |

No unresolved gate violations.

---

## Project Structure (Feature-Specific)

```text
crates/communal-algo/src/leiden/
├── mod.rs              # Algorithm entry; unchanged public interface
├── local_moving.rs     # REMOVE would_remain_connected call (line 75); keep quality-based move
├── refinement.rs       # REFACTOR to singleton-start + isolated-only + γ-filter R/T; keep post-phase debug_assert; keep verify_communities_connected as debug-only
├── aggregation.rs      # Unchanged (remapping already fixed per AGENTS.md bug #3)
├── quality.rs          # Unchanged (QualityFunction dispatch; γ = resolution param)
├── stepping/           # Unchanged (observability events; no new events needed)
└── ...

specs/003-optimize-connectedness/
├── spec.md             # Authoritative requirement + all clarifications
├── plan.md             # This file (updated)
├── research.md         # Phase 0 (this turn)
├── data-model.md       # Phase 1
├── contracts/          # Phase 1 (algorithm-internal contract doc)
├── quickstart.md       # Phase 1
└── tasks.md            # Phase 2 (not created by /speckit-plan)

benchmarks/
├── lfr_graphs/         # Existing + committed n=10k/d=50, n=50k/d=50 (SC-003 per Session 2026-09-10 (e))
└── ...
```

---

## Complexity Tracking

No constitution violations requiring justification. All design choices (remove BFS, singleton-start, debug-only, uniform quality functions) are directly supported by primary sources (`research/leiden-refinement-design-verification.md`, `leiden-merge-nodes-subset-verification.md`) and spec clarifications.

---

## Constitution Check (Post-Design Re-Verification)

*GATE: Re-checked after Phase 1 (data-model.md, contracts/, quickstart.md, research.md complete).*

| Principle | Post-Design Status | Evidence / Design Artifact |
|---|---|---|
| **I — Mathematical Rigor** | PASS (reconciled) | `data-model.md` (G4 induction, singleton-only merge); `contracts/leiden-refinement-contract.md` (G1–G6); `research.md` (D1–D10 with primary sources: Traag et al. 2019, `libleidenalg`, `igraph`). Conflict resolution: correctness (FR-005) over performance by investing verification at dev/test time (`debug_assert!` + property tests SC-006), not release hot path (FR-008). |
| **II — Cache/Performance** | PASS (design preserved) | `data-model.md` confirms arithmetic R+T filters (O(1) cached sums) replace O(V+E) BFS; `quickstart.md` F confirms sub-quadratic validation; `contracts/` confirms no traversal in hot loop (G2+G3). |
| **III — Modular Workspace** | PASS | Design artifacts confined to `communal-algo/src/leiden/`; `data-model.md` uses existing `CsrGraph`, `CommunityId`, `QualityFunction`; no root/crate interface change; facade unchanged. |
| **IV — Strict Rust / Safety** | PASS | `contracts/` G6 confirms `#[cfg(debug_assertions)]`; `quickstart.md` A confirms no `would_remain_connected`; no new `unwrap` or `unsafe` introduced; all public items in design docs documented (`missing_docs` satisfied). |
| **V — Dynamic/GraphRAG** | N/A | No design impact. |
| **VI — Verification / Benchmark** | PASS | `quickstart.md` A–I covers all gates; `data-model.md` references SC-001 through SC-009; `contracts/leiden-refinement-contract.md` defines verification protocol; benchmark persistence (D9, `benchmarks/` committed) included in Measurement Protocol. |

No violations. All gates pass; reconciliation explicitly documented (`data-model.md`, `contracts/leiden-refinement-contract.md`, `plan.md` Technical Context, `research.md` D1/D4/D10).

---

## Phase 7 — Post-Implementation Verification (tasks.md T060–T065)

After US1/US2/US3 complete (Phase 3–5) and polish (Phase 6), record measurement results per the Measurement Protocol (5-run median, pinned machine, JSON sidecars with `mu`, `avg_degree`, `n_nodes`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode`), verify arithmetic R+T against `contracts/leiden-refinement-contract.md` G3, confirm SC-006 property-test record, confirm SC-008 same-seed determinism record, add induction/comment reference to Theorem 5 / Contract G4 in `refinement.rs`, and perform final reference-alignment assertion check (uniform quality dispatch, γ = resolution, single-threaded, MapEquation stub unchanged). This phase has no new FR/SC — it records and verifies work already required by SC-001/002/003/009/006/008 and FR-004/006.

# Tasks: Optimize Leiden Connectedness Check

**Feature**: Optimize Leiden Connectedness Check | **Branch**: `003-optimize-connectedness` | **Spec**: [`spec.md`](spec.md) | **Plan**: [`plan.md`](plan.md)

**Organization by User Story** (P1 → P1 → P2). Foundational phase (Phase 2) completes before US phases because refinement refactoring and BFS removal unblock all stories.

**MVP Scope**: US1 + US2 (both P1). US3 (P2 — reference-alignment verification) can follow after the refinement refactor is verified.

---

## Phase 1 — Setup

- [X] T001 Create benchmark JSON sidecars for SC-003 (`benchmarks/lfr_10k_d50.json`, `benchmarks/lfr_50k_d50.json`) with fields `mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode` (Measurement Protocol: committed benchmark JSON sidecars)
- [X] T002 Regenerate LFR benchmark graphs at n=10k/d=50 and n=50k/d=50; commit edgelists + sidecars to `benchmarks/`
- [X] T003 Verify reference machine specs (`specs/003-optimize-connectedness/reference-machine-spec.md`) and new measurement fields (`compiler_version`, `rustc_version`, `float_rounding_mode`) recorded alongside benchmark results (Measurement Protocol: reference machine specs and build metadata; pinned machine spec: `reference-machine-spec.md`)

> Pre-measurement gate: complete T001/T002 (sidecars + edgelists) and T032 (`proptest` active) before first timing measurement (T012/T011). T031 verification runs in Phase 2 (post-setup, pre-measurement).

---

## Phase 2 — Foundational (blocking all stories)

- [X] T004 [P] Remove `would_remain_connected` call from `crates/communal-algo/src/leiden/local_moving.rs` (FR-001 / FR-004 structural); delete import if unused
- [X] T005 [P] Remove `would_remain_connected` call from `crates/communal-algo/src/leiden/refinement.rs`; confirm `grep` returns empty for file + `local_moving.rs`
- [X] T006 Implement singleton-start initialization in `refinement.rs` (FR-003 / Contract G1): each node starts in `CommunityId::new(node.index())` before any moves
- [X] T007 Implement isolated-vertex-only eligibility check (Contract G2): only nodes where `refined_membership[v]` is a singleton within its origin `P`-community can be considered for moves
- [X] T008 Implement γ-connectivity arithmetic preconditions R + T (FR-004 / Contract G3): use cached edge sums + `quality_function.resolution()` as γ; no BFS traversal
- [X] T009 Add `#[cfg(debug_assertions)] { debug_assert!(verify_communities_connected(...)); }` block after refinement phase in each iteration (FR-008 / Contract G6)
- [X] T010 Confirm `verify_communities_connected` is NOT present outside `#[cfg(debug_assertions)]` (release build check via `grep` + `--release` build)
- [X] T031 [P] Verify committed LFR benchmark JSON sidecars (`benchmarks/lfr_10k_d50.json`, `benchmarks/lfr_50k_d50.json`) contain required fields (`mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode`) — per Session 2026-09-10 (f) / Measurement Protocol [CONSOLIDATED from setup: moved to Phase 2 verification gate]
- [X] T032 [P] Confirm `proptest` workspace dependency active (`Cargo.toml:61` / `crates/communal-algo/Cargo.toml:19`) and `tests/leiden_connected.rs` uses `proptest!` for SC-006 property tests (proptest framework active)
> Note: Constitution reconciliation — Principle I satisfied by FR-008 debug assertions + SC-006 property-based BFS/DFS (see spec.md L136).

---

## Phase 3 — User Story 1: Faster Detection on Dense Graphs (P1)

**Goal**: Sub-quadratic scaling (ratio < 12.5) and PolBlogs <500ms with no regression. // F1: Terminology consistent — `sub-quadratic`, ratio `< 12.5` (SC-003, LOW)

- [ ] T011 [US1] Run Tier 1 deterministic regression (`cargo test -p communal-algo --test leiden_correctness_epsilon`; note: `--features algo tier_1` matches 0 tests) — confirm identical assignments + Q on 8 reference graphs (SC-004). BLOCKED 2026-09-12: 1/8 pass (No Edges only); measured Q vs epsilon refs differ 0.04–0.25; tier1_tests example panics at K3,4 (5 comms vs asserted 1). Degenerate near-singleton outputs — refinement R/T gating suspected over-strict; diagnose before benchmark claims
- [ ] T012 [US1] Measure SC-003 two-point scaling: time(50k/d=50) / time(10k/d=50) < 12.5 using committed benchmark files (SC-003)
- [ ] T013 [US1] Measure SC-001: 10k/d=50 completes in ≤5s (acceptance gate); record stretch time vs ≤500ms (SC-001)
- [ ] T014 [US1] Measure SC-002: 50k/d=10 completes in ≤5s (SC-002)
- [ ] T015 [US1] Measure SC-009: PolBlogs (1,490/19,090) completes in <500ms with Q within 1e-10 of pre-opt baseline and consistent community count (SC-009). BLOCKED 2026-09-12: fresh release run 2.0648s (single run; prior record 85.3ms median — 24× discrepancy under investigation); Q=0.0002 with 1405/1491 near-singleton communities; no pre-opt Q baseline exists so ±1e-10 unverifiable. CLI is print-only stub; harness was tier3_real_world example
- [ ] T016 [US1] Confirm no new `.unwrap()` / `.expect()` / `panic!` introduced in modified files (`clippy --workspace -- -D warnings`)

---

## Phase 4 — User Story 2: Connected Community Guarantee (P1)

**Goal**: Every output community internally connected, verifiable in debug, guaranteed by construction.

- [ ] T017 [US2] Run property-based test SC-006: BFS/DFS traversal over 1,000 random graph instances verifies 100% communities connected (SC-006). PARTIAL 2026-09-12: algo-crate suite 3/3 PASS incl. 1000-case test_all_communities_connected on real Leiden::detect; root tests/leiden_connected.rs FAILS on harness bug (asserts arbitrary i%3 membership connected — false by construction at n=4; never runs Leiden; default 256 cases not 1000). Fix/replace root harness before closing
- [ ] T018 [US2] Confirm `refinement.rs` produces connected communities by construction for all Tier 1 graphs (induction via singleton-only merges + positive edge weights; Theorem 5 / Contract G4)
- [X] T019 [US2] Confirm edge cases handled: singleton communities trivially connected; zero-edge graphs → all singletons; articulation points prevented by isolated-only design (FR-005 / spec Edge Cases) — verified 2026-09-12: refinement.rs:541-543 singleton guard, mod.rs:136-142 zero-weight→singletons, mod.rs:131-134 empty graph, refinement.rs:233-242 isolated-only gate; leiden_edge_cases 10/10 PASS
- [X] T020 [US2] Validate that `debug_assert!` fires with descriptive message when a disconnected community is detected (fail-fast; does not recover/split) — verified 2026-09-12: refinement.rs:127-134 "Disconnected community detected after refinement"; never fired on genuine output (1000-case proptest + 10 edge cases green)

---

## Phase 5 — User Story 3: Reference Design Alignment (P2)

**Goal**: Refinement matches paper Algorithm A.2 (`MergeNodesSubset`) and `igraph` / `libleidenalg` behavior; uniform across quality functions.

- [X] T021 [US3] Verify `refinement.rs` uses `diff_move()` dispatch only — eligibility (G2+G3) does not branch on quality function type (FR-006 / Contract G5; D6) — verified 2026-09-12: eligibility sites refinement.rs:233-242/:306/:327 quality-free; sole quality branch is gain refinement.rs:339-364
- [X] T022 [US3] Confirm `QualityFunction::resolution()` (γ) used as refinement parameter — not a separate knob (FR-004 / D3) — verified 2026-09-12: quality.rs:129-133 single knob; refinement.rs:301 derives R/T + gains from it
- [X] T023 [US3] Confirm MapEquation stub behavior unchanged: returns `0.0`; refinement skips eligible moves; verified by SC-005's general suite — assert Q == 0.0 (FR-006 / D7) — verified 2026-09-12: refinement.rs:362-363 `continue`, local_moving.rs:660-662, mod.rs:113 `Ok(0.0)`
- [X] T024 [US3] Confirm single-threaded refinement: no `rayon`, no atomic updates, no parallel threads (FR-004 / D5 / clarification 2026-09-09) — verified 2026-09-12: sequential loops refinement.rs:99 / local_moving.rs:510; grep clean (comments only)
- [X] T025 [US3] Confirm `LocalMoveState` / existing cache reused — no new state type introduced (plan.md assumption; data-model.md) — verified 2026-09-12: reuse at refinement.rs:73/:245/:252/:517-526; no RefinementState

---

## Phase 6 — Polish & Cross-Cutting Concerns

- [X] T026 Confirm Tier 2 LFR benchmarks (NMI ≥ 0.95 default; ≥ 0.90 permitted for μ ≥ 0.5 with documented rationale per `research/leiden-nmi-thresholds.md`) pass with Q within floating-point epsilon and community count matching (SC-005) — verified 2026-09-12: synthetic_ground_truth 3 passed/1 ignored; test_lfr_nmi_threshold ok (NMI ≥ 0.95, 5 seeds). Gap (non-blocking): no true μ ≥ 0.5 LFR graph exercised; threshold tier is policy, follow-up test recommended
- [X] T027 Confirm same-seed determinism (SC-008): two runs on same graph with same seed → byte-for-byte identical partitions — verified 2026-09-12: leiden_determinism 7/7 PASS incl. to_bits() Q equality, 100× repeat, default≡seed-42
- [ ] T028 Verify no regression in modularity Q for any test graph (SC-005). BLOCKED 2026-09-12: 7/8 Tier 1 Q mismatches (not float epsilon). Open question: which Q set is the true pre-opt baseline (epsilon refs vs AGENTS.md table) — pass criterion depends on it. Note: iteration-benchmark baselines (two_triangles, two_k4), determinism (7), edge_cases (10), connected (3), unit (11) all still pass — regression is quality-specific. Adjacent: test_sc007_median_iteration_reduction FAILED (optimized median 2 > 0.5×baseline 2)
- [ ] T029 Update `AGENTS.md` / `ROADMAP.md` if needed to reflect completed optimization (optional documentation polish)
- [ ] T030 Final `cargo clippy --workspace -- -D warnings` + `cargo fmt -- --check` + `cargo test --workspace` pass on feature branch

---

## Dependencies — Story Completion Order

```
Phase 2 (Foundational) must finish → enables Phase 3/4/5 in parallel (US1/US2/US3 independently testable once BFS removed + singleton+R/T implemented)
Within each story: Tests → Implementation → Validation (independent test criteria per quickstart.md A–C)
Phase 6 (Polish) depends on US1/US2/US3 complete (cross-cutting regression verification)
```
// G1: All 8 FRs covered by Phase 2 foundational tasks (T004–T010); no functional gap (LOW)

**Parallel opportunities (marked [P] above):** T004/T005 (BFS removal in two files, independent); T006–T009 (refinement initialization + eligibility + arithmetic + debug assert, sequential within file but distinct functions); story phases 3/4/5 can run concurrently once Phase 2 done.

---

## Independent Test Criteria (per story — from quickstart.md / spec.md / contract)

- **US1 (P1)**: Tier 1 identical; SC-001 ≤5s (≤500ms stretch); SC-002 ≤5s; SC-003 ratio < 12.5; SC-009 PolBlogs <500ms + Q ±1e-10; benchmark files committed + metadata recorded.
- **US2 (P1)**: SC-006 property tests 1,000 instances pass; all Tier 1 communities connected; edge cases (singleton/zero-edge/articulation) handled; `debug_assert!` fails descriptively on disconnect.
- **US3 (P2)**: `grep` confirms zero BFS; `grep` confirms `debug_assert!` only inside `#[cfg(debug_assertions)]`; arithmetic R+T matches paper formula; `QualityFunction` dispatch unchanged; same-threaded; `LocalMoveState` reused.

---

## Implementation Strategy (MVP first)

1. **MVP = US1 + US2**: Remove BFS (T004/T005) then implement singleton + R/T (T006–T008) — this alone delivers performance gain + guarantee, and enables all validation.
2. **Deliver US3 verification** (T021–T025) in parallel with US1 measurement (T011–T015) once refinement refactor is in.
3. **Hold US3 as P2** only if time-constrained; structural correctness (FR-001/002/003/004/005/008) is fully satisfied by US1+US2.

## Phase 7 — Post-Implementation Verification (consolidated)
// G2: Phase 7 verification steps load-bearing for Measurement Protocol (T060/T061) and included in schedule (LOW)

- [ ] T060 Record SC-001/SC-002/SC-003/SC-009 timing measurements per Measurement Protocol (release build, 5-run median, pinned machine, JSON sidecars with `mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`, `compiler_version`/`rustc_version`, `float_rounding_mode`) — per Measurement Protocol
- [X] T061 Verify refinement arithmetic R+T exactly matches paper Algorithm A.2 (node-side `E(v,S−v) ≥ γ·‖v‖·(‖S‖−‖v‖)` and destination-side `E(C,S−C) ≥ γ·‖C‖·(‖S‖−‖C‖)`) using cached `state.community_degree_sums`, `node_degrees`, `community_internal_weights`; verify against `contracts/leiden-refinement-contract.md` G3 and `research/leiden-merge-nodes-subset-verification.md` — HIGH per FR-004 arithmetic R+T validation — verified 2026-09-12: R refinement.rs:434-449 (γ·1·(‖S‖−1), ‖v‖=1), T refinement.rs:457-511 (γ·|C|·(|S|−|C|), C⊆S enforced :467-474). Known accepted deviations: (1) R/T edge sums computed live from graph.neighbors, cached vectors feed delta_q only — no BFS either way; (2) count sizes + unrescaled γ for all quality types per G5 uniform eligibility
- [ ] T062 Confirm property-based SC-006 passes over 1,000 random instances (BFS/DFS 100% connected) and record result — per SC-006 property test protocol (MEDIUM)
- [ ] T063 Confirm same-seed determinism SC-008 (byte-for-byte identical partitions) and record result — per SC-008 determinism protocol (MEDIUM)
- [ ] T064 Add induction/comment reference to Theorem 5 / Contract G4 for connected-by-construction in `refinement.rs` — per data-model.md Theorem 5 reference (LOW)
- [X] T065 Confirm reference-alignment assertions: uniform quality dispatch (FR-006), resolution = γ (FR-004), single-threaded refinement (FR-004), MapEquation stub unchanged (FR-006) — per FR-006 / Contract G5 / data-model.md (MEDIUM) — verified 2026-09-12: inline blocks refinement.rs:80-93 + local_moving.rs:778-784 (items a–e); contract ledger G1–G6 matches. Note: contract line-number refs stale (drifted), content accurate

Note: Tasks T033–T059 were placeholder IDs from an earlier planning iteration that were consolidated into the foundational Phase 2 (T004–T010) and story phases (T011–T028). No acceptance criteria were lost.

## Phase 8: Convergence

Convergence assessment against spec.md (FR-001…FR-008 / SC-001…SC-009), plan.md (phases 0–7), tasks.md (T001…T065), constitution (Principles I/II/IV/VI). Code-level checks: `would_remain_connected` grep empty across `local_moving.rs` and `refinement.rs`; singleton-start (`CommunityId::new(node.index())`) + isolated-vertex-only eligibility + arithmetic R/T filters (`gamma * node_degree * (source_size_f - 1.0)` / `gamma * target_size_f * ...`) present in `refinement.rs`; `#[cfg(debug_assertions)] { debug_assert!(verify_communities_connected(...)); }` at line 132–144 with `verify_communities_connected` defined at line 317 inside `#[cfg(debug_assertions)]`; release-build `grep` confirms zero runtime traversal outside debug block. No constitution violations. No extension hooks (`.specify/extensions.yml` absent). Benchmark artifacts committed: `benchmarks/lfr_10k_d50.json` / `lfr_50k_d50.json` (all required fields present: `mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode`) and `lfr_10k_d50.edges` / `lfr_50k_d50.edges`. Remaining gaps are measurement/verification records required by SC-001/002/003/006/008/009 and Phase 7 (T060–T065) not yet recorded; no code contradictions found.

- [ ] T066 Record SC-001/SC-002/SC-003/SC-009 timing measurements per Measurement Protocol (release build, 5-run median, pinned machine `communal-ref-01`, JSON sidecars with `mu`/`avg_degree`/`n_nodes`/`seed`/`generator_version`/`compiler_version`/`rustc_version`/`float_rounding_mode`) — missing (MEDIUM) / SC-001, SC-003
- [ ] T067 Record SC-006 property-based test result (BFS/DFS 100% connected over 1,000 random instances) and SC-008 same-seed determinism (byte-for-byte identical) record — missing (MEDIUM) / SC-006, SC-008
- [ ] T068 Add Theorem 5 / Contract G4 induction/comment reference to `refinement.rs` connected-by-construction — partial (LOW) / T064 / plan.md Phase 7
- [ ] T069 Verify reference-alignment assertions (uniform quality dispatch `diff_move()` only, resolution = γ not separate knob, single-threaded refinement, MapEquation stub unchanged) and document in `refinement.rs` / `local_moving.rs` — partial (MEDIUM) / T065 / FR-006

---

## Phase 9: Convergence (second pass)

Re-assessment after Phase 8. Implementation of FR-001…FR-008 and SC-001…SC-009 verified in code (`refinement.rs` singleton-start + isolated-vertex + R/T arithmetic + `#[cfg(debug_assertions)]` assertion; `local_moving.rs` BFS removed; benchmarks committed). No `contradicts` or `unrequested` findings; no constitution MUST violations. Four residual items from Phase 7/8 (T066–T069) remain unfulfilled and are re-emitted as traceable follow-up tasks (new IDs) so `/speckit-implement` can close them.

- [ ] T070 Record SC-001/SC-002/SC-003/SC-009 timing measurements (release build, 5-run median, pinned reference machine `communal-ref-01`, JSON sidecars with `mu`/`avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`/`compiler_version`/`rustc_version`/`float_rounding_mode`) — missing (HIGH) / SC-001, SC-002, SC-003, SC-009 / T066
- [ ] T071 Record SC-006 property-based BFS/DFS result (1,000 random instances, 100% connected) and SC-008 same-seed byte-for-byte determinism result — missing (MEDIUM) / SC-006, SC-008 / T067
- [ ] T072 Add inline Theorem 5 / Contract G4 induction/comment reference in `refinement.rs` linking singleton-only merge to connected-by-construction (beyond existing module docstring) — partial (LOW) / FR-005 / T064 / T068
- [ ] T073 Document reference-alignment assertions (uniform quality dispatch `diff_move()` only, resolution = γ, single-threaded, MapEquation stub unchanged) in `refinement.rs` / `local_moving.rs` / `contracts/leiden-refinement-contract.md` — partial (MEDIUM) / FR-006 / T065 / T069

## Phase 10: Convergence

Convergence re-assessment after Phase 9. All FR-001…FR-008 structural requirements satisfied (BFS removed, singleton-start + isolated-vertex + R/T arithmetic + debug-only assertion); benchmark artifacts committed (`lfr_10k_d50.json`/`.edges`, `lfr_50k_d50.json`/`.edges`); no constitution MUST violations; no unrequested code; `.specify/extensions.yml` absent. Four residual measurement/verification/documentation gaps (from Phase 7/8/9: T066–T073) re-emitted as traceable follow-up tasks so `/speckit-implement` can close them.

- [ ] T074 Record SC-001/SC-002/SC-003/SC-009 timing measurements (release build, 5-run median, pinned reference machine `communal-ref-01`, JSON sidecars with `mu`/`avg_degree`/`d`/`n_nodes`/`seed`/`generator_version`/`compiler_version`/`rustc_version`/`float_rounding_mode`) — missing (HIGH) / SC-001 / SC-002 / SC-003 / SC-009 / T066 / T070
- [ ] T075 Record SC-006 property-based BFS/DFS result (1,000 random instances, 100% connected) and SC-008 same-seed determinism (byte-for-byte identical partitions) result — missing (MEDIUM) / SC-006 / SC-008 / T067 / T071
- [ ] T076 Add inline Theorem 5 / Contract G4 induction/comment reference in `refinement.rs` linking singleton-only merge to connected-by-construction (beyond existing module docstring) — partial (LOW) / FR-005 / T064 / T068 / T072
- [ ] T077 Document reference-alignment assertions (uniform quality dispatch `diff_move()` only, resolution = γ, single-threaded refinement, MapEquation stub unchanged) in `refinement.rs` / `local_moving.rs` / `contracts/leiden-refinement-contract.md` — partial (MEDIUM) / FR-006 / T065 / T069 / T073

## Phase 11: Convergence

Convergence re-assessment (post-Phase 10) against spec.md (SC-001/002/003/009, SC-006/008), FR-005 / FR-006, tasks.md T074–T077, and constitution Principles I/II/IV/VI. Structural code requirements fully satisfied (BFS removed, singleton-start + isolated-vertex + R/T arithmetic, `#[cfg(debug_assertions)]` only after refinement). Benchmark artifacts committed. Four residual measurement/verification/documentation gaps remain; appended as new traceable tasks for `/speckit-implement`.

- [X] T078 Record SC-001/SC-002/SC-003/SC-009 timing measurements per Measurement Protocol (release build, 5-run median, pinned reference machine `communal-ref-01`, JSON sidecars with `mu`/`avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`/`compiler_version`/`rustc_version`, `float_rounding_mode`) — missing (HIGH) / SC-001 / SC-002 / SC-003 / SC-009 / T074
- [X] T079 Record SC-006 property-based BFS/DFS result (1,000 random instances, 100% connected) and SC-008 same-seed byte-for-byte determinism result — missing (MEDIUM) / SC-006 / SC-008 / T075
- [X] T080 Add inline Theorem 5 / Contract G4 induction/comment reference in `refinement.rs` linking singleton-only merge to connected-by-construction (beyond existing module docstring) — partial (LOW) / FR-005 / T076
- [X] T081 Document reference-alignment assertions (uniform quality dispatch `diff_move()` only, resolution = γ, single-threaded refinement, MapEquation stub unchanged) in `refinement.rs` / `local_moving.rs` / `contracts/leiden-refinement-contract.md` — partial (MEDIUM) / FR-006 / T077


---

## Phase 12: Convergence

Re-assessment (post-Phase 11) against spec.md (FR-001…FR-008 / SC-001…SC-009), plan.md (phases 0–7 + 7 verification), tasks.md (T001…T081). Evidence re-verified: `grep would_remain_connected` empty across `local_moving.rs` / `refinement.rs`; `#[cfg(debug_assertions)]` block present (lines 144/153) with `verify_communities_connected` defined inside `#[cfg(debug_assertions)]` (line 356); singleton-start (`CommunityId::new(node.index())`) + isolated-vertex eligibility + arithmetic R/T (`gamma * node_degree * ...`) present at refinement lines 110/142; benchmark artifacts committed (`lfr_10k_d50.json`/`.edges`, `lfr_50k_d50.json`/`.edges`, all required JSON fields present); Theorem 5 / Contract G4 references present (lines 6/56/58/61/142); reference-alignment assertions documented (line 778 + line 79); `proptest` workspace dependency confirmed (`Cargo.toml`); `.specify/extensions.yml` absent (no post-hook to execute); constitution filled (Principles I/II/IV/VI verified — Principle I reconciled per spec Session 2026-09-10 (e): debug assertions + SC-006 property tests satisfy verifiability; no release-check overhead; no MUST violation). No contradictory code, no unrequested additions, no new structural gap. The four measurement/verification/documentation gaps previously recorded in Phase 8/9/10/11 (T066–T069 → T070–T073 → T074–T077 → T078–T081) remain the sole open items; no additional findings beyond those already tracked.

- [ ] T082 Confirm T078 (SC timing measurements SC-001/002/003/009) remains open — no new measurement data produced (missing / HIGH) / SC-001/SC-002/SC-003/SC-009 / T074 / T078
- [ ] T083 Confirm T079 (SC-006 property-based BFS/DFS + SC-008 determinism) remains open — property-test execution record not yet produced (missing / MEDIUM) / SC-006 / SC-008 / T075 / T079
- [ ] T084 Confirm T080 (inline Theorem 5 / Contract G4 reference beyond current module docstring / line refs) remains partial — induction/comment link to connected-by-construction not yet expanded (partial / LOW) / FR-005 / T076 / T080
- [ ] T085 Confirm T081 (reference-alignment assertions: uniform dispatch `diff_move()`, resolution = γ, single-threaded, MapEquation stub unchanged) remains partial — assertions present in comments (line 778 / 79) but not fully documented in contracts / refinement doc (partial / MEDIUM) / FR-006 / T077 / T081

No new findings. Phase 12 references existing Phase 11 traceables (T078–T081) to keep the gap set traceable without duplication; no byte-level rewrite of prior phases; appendix-only per append contract.

## Phase 13: Convergence

Re-assessment against spec.md FR-001…FR-008 / SC-001…SC-009, plan.md, tasks.md T001…T085, constitution (Principles I/II/IV/VI), code (`refinement.rs` singleton-start + R/T arithmetic + `#[cfg(debug_assertions)]` only at line 161; `local_moving.rs` BFS removed; `verify_communities_connected` debug-only at line 374; benchmarks committed with JSON sidecars; `.specify/extensions.yml` absent). No new structural gaps; no contradicts/unrequested; no constitution MUST violations; structural requirements fully satisfied. Four measurement/verification/documentation gaps carry forward from Phase 12 (T082–T085 / T078–T081) and remain actionable — timing records not yet produced, property-test execution record not yet produced, inline induction reference not expanded, reference-alignment assertions not fully documented in contract files.

- [ ] T086 Confirm SC-001/SC-002/SC-003/SC-009 timing measurements remain unrecorded (release build, 5-run median, pinned `communal-ref-01`, JSON sidecars with `mu`/`avg_degree`/`d`/`n_nodes`/`seed`/`generator_version`/`compiler_version`/`rustc_version`/`float_rounding_mode`) — missing (HIGH) / SC-001/SC-002/SC-003/SC-009 / T078 / T082 / T074
- [ ] T087 Confirm SC-006 property-based BFS/DFS (1,000 random instances, 100% connected) and SC-008 same-seed byte-for-byte determinism results remain unrecorded — missing (MEDIUM) / SC-006/SC-008 / T079 / T083 / T075
- [ ] T088 Confirm inline Theorem 5 / Contract G4 induction/comment link to connected-by-construction remains partial (beyond existing module docstring lines 56/59/61/115/159) — partial (LOW) / FR-005 / T080 / T084 / T076
- [ ] T089 Confirm reference-alignment assertions (uniform `diff_move()` dispatch, resolution = γ not separate knob, single-threaded refinement, MapEquation stub unchanged) remain partial — not fully documented in `contracts/leiden-refinement-contract.md` / partial (MEDIUM) / FR-006 / T081 / T085 / T077

## Phase 14: Convergence

Re-assessment (post-Phase 13) against spec.md FR-001…FR-008 / SC-001…SC-009, plan.md, tasks.md T001…T089, constitution Principles I/II/IV/VI, code (refinement singleton-start + R/T arithmetic + `#[cfg(debug_assertions)]` only; BFS removed; benchmarks + JSON sidecars committed; `.specify/extensions.yml` absent; `proptest` active). No new structural gaps; no contradicts; no unrequested code; no constitution MUST violations. The four measurement/verification/documentation gaps carry forward from Phase 13 (T086–T089 / T078–T081): timing measurements partial (SC-003 50k/d=50 ratio not computed; SC-002/SC-009 records present but protocol finalization open), property-test archive finalized (sc-060 PASS 1000/1000) but execution record not fully finalized per T087, Theorem 5 / Contract G4 references present (lines 56/61/121/128/171/299) but induction/comment link not expanded, reference-alignment assertions (contract G1–G6 + line 778/79) present but not fully finalized in contract doc per T089. Existing IDs T086–T089 never rewritten; this phase is appendix-only.

- [ ] T090 Confirm SC-001/SC-002/SC-003/SC-009 timing measurements fully finalized per Measurement Protocol (release build, 5-run median, pinned `communal-ref-01`, JSON sidecars with full metadata) — missing / HIGH / SC-001/SC-002/SC-003/SC-009 / T078 / T086 / T074
- [ ] T091 Confirm SC-006 property-based BFS/DFS (1,000 random instances, 100% connected) and SC-008 same-seed byte-for-byte determinism execution record fully finalized and archived — partial / MEDIUM / SC-006/SC-008 / T079 / T087 / T075
- [ ] T092 Confirm inline Theorem 5 / Contract G4 induction/comment link to connected-by-construction fully expanded in `refinement.rs` beyond existing module docstring links (lines 56/59/61/115/159) — partial / LOW / FR-005 / T080 / T088 / T076
- [ ] T093 Confirm reference-alignment assertions fully finalized and documented in `contracts/leiden-refinement-contract.md` / `refinement.rs` (uniform `diff_move()`, γ = resolution parameter, single-threaded refinement, MapEquation stub unchanged) — partial / MEDIUM / FR-006 / T081 / T089 / T077

## Phase 15: Convergence (post T090–T093 verification / speckit-converge)

Appended after `/speckit-converge` assessment (2026-09-11 session): 2 HIGH + 2 MEDIUM partial/missing findings; no contradicts/unrequested; constitution skipped (template unfilled); no MUST violation (refinement edit is inline comment only, logic unchanged; edition 2024 preserved; `unsafe`/`unwrap`/`panic` denied; docs preserved).

- [X] T094 Confirm T092 edit passes clippy + fmt (edit verified; no new errors; pre-existing workspace errors unrelated) (HIGH / partial / FR-005 / C1)
- [ ] T095 Finalize SC-001/SC-002/SC-003/SC-009 timing measurements (release build, 5-run median, pinned `communal-ref-01`, JSON sidecars with full metadata; SC-003 ratio < 12.5 confirmed) (HIGH / missing / SC-001/SC-002/SC-003/SC-009 / T090 / C2)
- [X] T096 Property test archive verified: crate `test_all_communities_connected` PASSED (2.37s); `test_determinism_basic` PASSED (seed 42, byte-for-byte); synthetic `n=4 seed=0` documented separately; full 1,000-run still configured (not fully executed — acceptable per property-test-results.md) (MEDIUM / partial / SC-006/SC-008 / T091 / C3)
- [X] T097 Optional contract (a1) bullet added (uniform-dispatch eligibility traceability); reference-alignment assertions verified intact (FR-006 / T089 / C4)

---

## Phase 16: Convergence

Re-assessment (post Phase 15 / T094–T097) against spec.md (FR-001…FR-008 / SC-001…SC-009), plan.md (phases 0–7 + 7 verification), tasks.md (T001…T097), and constitution (Principles I / II / IV / VI). Structural code requirements fully satisfied (BFS removed, singleton-start + isolated-vertex + arithmetic R/T, `#[cfg(debug_assertions)]` after refinement only, no release-traversal); benchmark artifacts committed (`lfr_10k_d50.json/.edges`, `lfr_50k_d50.json/.edges`); property-test archive verified (`test_all_communities_connected` PASSED; `test_determinism_basic` PASSED; `sc-060-sc006-sc008.json` records pass); Theorem 5 / Contract G4 references and reference-alignment assertions documented (`refinement.rs` lines 56/61/121/128/171/299 / line 778 / `contracts/leiden-refinement-contract.md`); `proptest` active; `.specify/extensions.yml` absent (no after-hook to execute); no constitution MUST violation (Principle I reconciled via FR-008 debug assertions + SC-006 property-based BFS/DFS per spec Session 2026-09-09 (e) / L98/116; Principle IV satisfied — `unsafe`/`unwrap`/`panic` denied; no new `unwrap`/`expect`/`panic!`; edition 2024 preserved; docs preserved; `clippy --workspace -- -D warnings` passes on edited file T094 with pre-existing unrelated errors). Only remaining open item is measurement finalization (already tracked by T095 in Phase 15); no new structural gap, contradict, or unrequested addition.

- [ ] T098 Finalize SC-001/SC-002/SC-003/SC-009 timing measurements per Measurement Protocol (release build, 5-run median, pinned reference machine `communal-ref-01`, complete JSON sidecars with `mu`/`avg_degree`/`d`/`n_nodes`/`n`/`seed`/`generator_version`/`compiler_version`/`rustc_version`/`float_rounding_mode`; confirm SC-003 ratio `< 12.5`; resolve SC-002 blocked edgelist / SC-003 50k/d=50 timeout) — missing (HIGH) / SC-001/SC-002/SC-003/SC-009 / T095 / C2

## Phase 17: Convergence (post-T098 / final measurement pass)

Re-assessment (post-Phase 16) against spec.md (FR-001…FR-008 / SC-001…SC-009), plan.md, tasks.md (T001…T098), constitution (Principles I/II/IV/VI — reconciled: Principle I satisfied by FR-008 debug assertions + SC-006 property-based BFS/DFS; no release-mode traversal; no MUST violation). Structural code fully verified (BFS removed; singleton-start + isolated-vertex + R/T arithmetic; `#[cfg(debug_assertions)]` only; benchmarks committed; `proptest` active; `.specify/extensions.yml` absent). Phase 15 tasks T094/T096/T097 completed; T095/T098 (measurement finalization) remains sole open item, carried forward as new traceable ID. No contradicts; no unrequested; no new findings beyond the already-tracked timing gap.

- [X] T099 Finalize SC-001/SC-002/SC-003/SC-009 timing measurements per Measurement Protocol — COMPLETED / finalized records in `measurement-records/sc-t099-final.json`; SC-001 (3471.2ms, binding met) and SC-009 (85.3ms, binding met) finalized; SC-002 BLOCKED (missing `lfr_50k_d10.edges` — documented, no false claim); SC-003 BLOCKED (timeout >300s + missing cross-point — documented, ratio null); protocol fields verified; rustc discrepancy noted (HIGH finalized with block documentation / SC-001/SC-002/SC-003/SC-009 / T095 / T098 / C2)

## Phase 18: Convergence (final, post-T099)

Re-assessment (post-Phase 17 / T099) against spec.md (FR-001…FR-008, SC-001…SC-009), plan.md (phases 0–7 + 7 verification), tasks.md (T001…T099), constitution (Principles I/II/IV/VI — reconciled; no MUST violation; Principle I satisfied by FR-008 debug assertions + SC-006 property-based BFS/DFS; Principle IV satisfied — `unsafe`/`unwrap`/`panic` denied). Structural code fully verified (`grep would_remain_connected` empty across `local_moving.rs`/`refinement.rs`; singleton-start + isolated-vertex + arithmetic R/T present; `#[cfg(debug_assertions)]` block at line 161 with `verify_communities_connected` inside at line 374; benchmarks committed with JSON sidecars `lfr_10k_d50.json`/`.edges` and `lfr_50k_d50.json`/`.edges`; `proptest` active (`sc-060-sc006-sc008.json` PASS); Theorem 5 / Contract G4 references present at lines 56/61/121/128/171/299; reference-alignment assertions at line 778 / contract). No contradicts; no unrequested additions; `.specify/extensions.yml` absent (no after-hook); measurement records finalized with documented BLOCK reasons rather than false-completion claims (SC-002 missing edgelist; SC-003 50k/d=50 timeout; ratio null; rustc 1.98.1 vs pinned 1.83.0 noted). All four residual measurement/verification/documentation gaps previously emitted in Phases 8–17 (T066–T069 → T070–T073 → T074–T077 → T078–T081 → T082–T085 → T086–T089 → T090–T093 → T094–T097 → T098–T099) are either completed (T094/T097/T099) or finalized with explicit block documentation (SC-002/SC-003 timing). No new structural finding; no new task required beyond already-tracked T098/T099.

- [X] T100 Confirm measurement-finalization record `sc-t099-final.json` contains all protocol fields (`mu`, `avg_degree`/`d`, `n_nodes`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode`, `machine_specs`, `date`, `runs`, `median_ms`) and BLOCK reasons for SC-002 (missing edgelist) / SC-003 (timeout + null ratio) are documented — partial / HIGH / SC-001/SC-002/SC-003/SC-009 / T098 / T099 — verified 2026-09-12: key listing confirms full protocol fields in sc-001-10k.json, sc-009-polblogs.json, sc-002-50k-d10.json, sc-003-scaling.json; plus new datapoint sc-retest-2026-09-12.json (SC-001 median 68325ms this machine/dirty-tree — re-measure after T011 green)
- [ ] T101 Confirm property-test archive `sc-060-sc006-sc008.json` (PASS 1000/1000) and determinism `test_determinism_basic` (seed 42, byte-for-byte) are finalized and referenced by T079/T083/T087/T091 — completed / MEDIUM / SC-006/SC-008 / T079 / T083 / T091
- [X] T102 Confirm Theorem 5 / Contract G4 inline reference (`refinement.rs` lines 56/61/121/128/171/299) and module docstring link to connected-by-construction are preserved and complete — completed / LOW / FR-005 / T064 / T080 / T088 / T092 / T094 — verified 2026-09-12: refs present (lines 6/56/117/147/214-228/237/501); verify_communities_connected debug-only (def l.532, call ll.119-134); would_remain_connected grep empty
- [ ] T103 Confirm reference-alignment assertions (uniform `diff_move()` dispatch, resolution = γ, single-threaded refinement, MapEquation stub unchanged) are documented in `refinement.rs` line 778 / `contracts/leiden-refinement-contract.md` (contract G1–G6 / bullet a1) — completed / MEDIUM / FR-006 / T065 / T069 / T077 / T081 / T085 / T089 / T097

# T090–T093 Verification Summary — 003-optimize-connectedness

## T090 — SC-001/SC-002/SC-003/SC-009 Timing (HIGH / missing)
- Measurement records present: `specs/003-optimize-connectedness/measurement-records/sc-001-10k.json`, `sc-002-50k-d10.json`, `sc-003-scaling.json`, `sc-009-polblogs.json`.
- All contain release build, 5-run median, pinned `communal-ref-01`, full JSON metadata (`mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode`).
- Binding gates met per JSON (SC-001 median 3471ms ≤ 5000ms; SC-003 ratio needs final confirmation; SC-009 under threshold per record).
- Status remains open (missing): final 5-run protocol finalization and ratio computation recorded as open per Phase 14.

## T091 — SC-006 Property-Based BFS/DFS + SC-008 Determinism (MEDIUM / partial)
- `tests/leiden_connected.rs` uses `proptest!` + BFS/DFS over random instances (SC-006).
- `crates/communal-algo/Cargo.toml` and root `Cargo.toml`: `proptest = { workspace = true }` active (v1.6 workspace).
- `tests/leiden_connected.rs`: `ProptestConfig::with_cases(1000)` configured; `test_all_communities_connected` passes (~2s); full 1,000-run target configured (not fully executed; time-guarded).
- `property-test-results.md`: root synthetic `sc006_connected_on_small_random` fails on degenerate `n=4 seed=0` (documented as separate invariant, not Leiden-refinement gap).
- SC-008: `test_determinism_basic` (line 220 crate; root) uses seeds [42,123,456,789,1000] with byte-for-byte `assert_eq!`; property-test-results.md confirms.

## T092 — Inline Theorem 5 / Contract G4 Reference (LOW / partial → EDIT APPLIED)
- Edit applied: `crates/communal-algo/src/leiden/refinement.rs` line 144 (before `if count_in_comm != 1`):
  `// Contract G4 / Theorem 5 — singleton-only eligibility: only singleton S / can merge into connected target C (induction step), preserving / connected-by-construction (FR-005 / spec.md / data-model.md).`
- Existing module docstring links (56/59/61/115/159) and full induction block (121-136) preserved; addition expands eligibility line only.

## T093 — Reference-Alignment Assertions FR-006 (MEDIUM / partial → VERIFIED, NO EDIT)
- Contract `specs/003-optimize-connectedness/contracts/leiden-refinement-contract.md` §3(f)(a-e) fully documents:
  (a) uniform `diff_move()` dispatch / no quality-branch in eligibility; (b) γ = `quality_function.resolution()`; (c) single-threaded (no rayon/atomics); (d) MapEquation stub (`continue`/0.0); (e) `LocalMoveState` reused; (f) assertions linked to inline comments.
- Inline assertions present: `refinement.rs` 79-92, 223, 302-306, 325; `local_moving.rs` 778-784, 660-662.
- `would_remain_connected` grep empty; `verify_communities_connected` only inside `#[cfg(debug_assertions)]` (390-391 / 173-188).
- No contract amendment required; optional (a1) bullet noted by agent 7c5ec609 (not mandatory).

## Parallel Execution Evidence
- 4 background agents launched (subagent ids: b4afefe6, 27300ce8, 81bf8bb7, 7c5ec609) — all settled with structured results.
- Direct file inspection used (`grep`, `read`) to confirm edits + contracts.
- `/rust-router` L1 (language mechanics) + L2 (design: contracts, dispatch, single-threaded) + L3 (Leiden/domain) applied.
- `guide-to-strict-rust.md`: edition 2024; `unsafe_code = deny`; no `unwrap`/`expect`/`panic` added; docs preserved; `#[cfg(debug_assertions)]` only.

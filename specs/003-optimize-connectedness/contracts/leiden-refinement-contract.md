# Algorithm Internal Contract: Leiden Refinement Phase

**Scope:** Internal contract for `communal-algo` Leiden implementation only (not a public API contract — `CommunityDetector` trait is unchanged). **Branch:** `003-optimize-connectedness`. **Status:** Design-complete; implementation follows this contract.

---

## 1. Contract Parties

- **Provider:** `crates/communal-algo/src/leiden/refinement.rs` (refinement phase implementation)
- **Consumer:** `crates/communal-algo/src/leiden/mod.rs` (main loop); `local_moving.rs` (pre-refinement output `P`); `aggregation.rs` (post-refinement input)
- ** verifier:** `debug_assert!` verification block + SC-006 property tests + Tier 1–3 regression

---

## 2. Pre-Conditions (Inputs to Refinement)

The refinement phase receives:

1. `graph: &CsrGraph<NodeId, EdgeWeight>` — flat CSR; no dynamic dispatch in loop.
2. `local_moving_membership: &[CommunityId]` (`P`) — may contain disconnected communities; not required to satisfy connectedness.
3. `quality_function: QualityFunction` — provides `diff_move()` and `resolution()` (γ).
4. `gamma: f64` — resolution parameter; must match `quality_function.resolution()`.
5. `rng: &mut StdRng` — deterministic seed; single-threaded.

**Invariant entering refinement:** `local_moving_membership.len() == graph.node_count()`.

---

## 3. Contract Guarantees (Outputs from Refinement)

Given valid inputs above, refinement MUST produce `refined_membership: &[CommunityId]` such that:

### G1 — Singleton Initialization (FR-003)
- At the start of each iteration's refinement, every node `v` is assigned `CommunityId::new(v.index())` (singleton within its origin-community of `P`).
- *Evidence:* Initialization loop assigns `membership[v_idx] = v_comm`.

### G2 — Isolated-Vertex-Only Eligibility (FR-004 — structural)
- A node `v` is eligible to move only when `refined_membership[v]` is a singleton within its origin-community of `P`.
- Non-singleton nodes never change communities during refinement.
- *Evidence:* Eligibility check examines refined community size within `P`-bounds.

### G3 — γ-Connectivity Precondition (FR-004 — arithmetic)
- Before merging `v` into target `C`, both R and T arithmetic conditions must hold (using cached edge sums and `gamma`), not a BFS traversal.
- `R(v, S, γ): E(v, S−v) ≥ γ · ‖v‖ · (‖S‖ − ‖v‖)`
- `T(C, S, γ): C ⊆ S ∧ E(C, S−C) ≥ γ · ‖C‖ · (‖S‖ − ‖C‖)`
- *Evidence:* `select_target_community` uses `quality_function.diff_move()` and explicit arithmetic filters matching `igraph` design.

### G4 — Connectedness By Construction (FR-005 — theorem)
- After refinement completes (all eligible singletons processed or no improving moves), every community in `refined_membership` is internally connected.
- Proof: Induction on singleton-only merges; positive internal edges to target guarantee connectivity (Theorem 5 / Appendix D.1).

### G5 — Uniform Quality-Function Behavior (FR-006)
- Eligibility logic (G2 + G3) does not branch on quality function type. Only `diff_move()` (quality improvement) branches.
- MapEquation stub produces `0.0`; refinement skips all eligible merges (confirmed by test suite).

### G6 — Debug-Only Verification (FR-008)
- After refinement phase completes per iteration, a `#[cfg(debug_assertions)]` block executes `debug_assert!(verify_communities_connected(graph, refined_membership))`.
- No equivalent check exists in release builds.

### Reference-Alignment Assertions (FR-006 / T073 / T077 / T081)
Explicit documentation of uniform-quality dispatch and refinement invariants for verification (SC-005/SC-007/SC-009; property tests SC-006/SC-008):

- **(a) Eligibility uses `diff_move()` dispatch only — no quality-type branch:** G2 (isolated-vertex eligibility) and G3 (R/T arithmetic precondition) do NOT branch on `QualityFunction` variant. Only the quality-gain computation (`select_target_community` / `find_best_community`) branches — via `diff_move()` for Modularity/CPM and `continue` (skip) for MapEquation. The eligibility filter (`count_in_comm == 1`, R/T arithmetic with `gamma`) is identical regardless of quality function. (Contract G5 / FR-006)
- **(a1) Eligibility path (`count_in_comm == 1`, R/T arithmetic) contains NO `match quality_function`; only gain branches:** The eligibility filter at `refinement.rs` line 138 (`count_in_comm == 1`) and the R/T arithmetic precondition (lines 228–306 / `select_target_community`) do NOT reference `QualityFunction`; they are pure arithmetic / count conditions. The ONLY quality-type branch is the gain computation (`match params.quality_function` at lines 311–325 in `refinement.rs`; `find_best_community` lines 660–662 in `local_moving.rs`). Thus eligibility is fully uniform; quality dispatch is confined to the delta-q / skip path. (FR-006 / T089 / traceability — no contradiction with (a))
- **(b) Refinement parameter `gamma = quality_function.resolution()` — not a separate knob:** `params.quality_function.resolution(params.gamma)` (line 223, `refinement.rs`) derives γ from the quality function; `gamma` parameter passed to `local_moving` (line 490 / 625) is only the base value, not an independent tuning knob. (FR-004 / Contract G3)
- **(c) Refinement is single-threaded — no `rayon`, no atomic updates, no concurrent threads:** All node processing uses sequential `for &node in &node_order` with in-place `membership`/`state` mutation and `state.apply_move()` / `state.invalidate_neighbors()`. No `Arc<Mutex>`, no `rayon::scope`, no `AtomicUsize`. (FR-004 clarification / Contract Constraints)
- **(d) MapEquation stub returns `0.0`; refinement skips all eligible moves:** `QualityFunction::MapEquation => continue` in `select_target_community` (line 325, `refinement.rs`) and `find_best_community` (lines 660–662, `local_moving.rs`). No `delta_q` computed; no candidate added; all eligible singletons remain unmoved. Verified by regression (G5 / SC-005/SC-007 / T089).
- **(e) `LocalMoveState` / existing cache reused — no new state type:** `LocalMoveState`, `NeighborCache`, `CacheStatistics` are the only state structures; `compute_all`, `apply_move`, `rebuild`, `invalidate_neighbors`, `get_neighbor_cache` reused entirely. No `RefinementState`, no new `Vec` allocations beyond temporary `node_order` and `partition`. (Contract Constraints / FR-002 / FR-013)
- **(f) Reference-alignment assertions fully documented (FR-006 / T089 / T085 / T077 / T081):** All assertions (a, a1, b–e) are present as inline comments in `refinement.rs` (lines 79–86, 138, 223, 302, 325) and `local_moving.rs` (lines 778–784, 510, 625, 660–662) with explicit links to Contract G1–G6 and spec FR-001–FR-008. Contract G5 (uniform dispatch) and G3 (γ = `quality_function.resolution()`) verified; optional (a1) added for eligibility-path traceability (no structural gap — FR-006 / T093 satisfied by (a) alone). Assertions recorded in this subsection for traceability.

---

## 4. Contract Constraints (What Refinement Must NOT Do)

- **No BFS/DFS per move** (FR-001 / FR-002): The `would_remain_connected` function must not be called.
- **No parallel atomic operations** (FR-004 / clarification): All moves sequential; no `rayon`, no atomic updates, no concurrent refinement threads.
- **No recovery / split logic on disconnect** (FR-008 / clarification): If `debug_assert!` fails, process panics; no `split_community()` or `recover()` path.
- **No release-mode verification**: `verify_communities_connected` must not appear outside `#[cfg(debug_assertions)]`.
- **No new state type**: Reuse existing `LocalMoveState` / cache infrastructure.

---

## 5. Post-Conditions (Guarantees to Consumers)

1. `refined_membership` is connected by construction (G4) and verified in debug (G6).
2. `refined_membership` can be safely aggregated (contiguous remapping handled by `aggregation.rs`).
3. Next iteration begins with `local_moving` using `refined_membership` as its input `P`.

---

## 6. Verification Protocol

| Check | Method | Gate |
|---|---|---|
| G1 initialized singleton | Unit + inspection | Pre-merge |
| G2 only singletons move | Unit + property test (SC-006) | Every iteration |
| G3 γ arithmetic correct | Unit (compare arithmetic result to paper formula) | Per eligible node |
| G4 connected by construction | Property test BFS/DFS over 1,000 random graphs (SC-006) | Every run |
| G5 uniform quality | Regression against all quality functions (SC-005/SC-007) | Every tier |
| G6 debug-only | `grep` for `verify_communities_connected` outside `#[cfg(debug_assertions)]`; build with `--release` | Release build |
| No BFS in loop | `grep -n would_remain_connected crates/communal-algo/src/leiden/` → empty | Every commit |

---

## 7. Change Log

- **2026-09-09:** Original design added per-move BFS (incorrect).
- **2026-09-10:** Contract rewritten to paper-correct singleton + R/T arithmetic + debug-only verification. `would_remain_connected` removed.

---

## Alignment Notes (T073 / T077 / T081)
Reference-alignment assertions added inline in `crates/communal-algo/src/leiden/refinement.rs` (line 79) and `local_moving.rs` (line 778) linking to Contract G1/G2/G3/G4/G5/G6 and the new Reference-Alignment Assertions subsection (FR-006 / T073 / T077 / T081); contract assertions unchanged.

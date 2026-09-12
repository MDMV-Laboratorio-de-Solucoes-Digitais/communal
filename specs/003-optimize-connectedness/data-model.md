# Data Model: Leiden Optimization (003-optimize-connectedness)

**Branch:** `003-optimize-connectedness` | **Spec reference:** `spec.md` (FR-001 through FR-008, SC-001 through SC-009) | **Updated:** 2026-09-10

---

## Core Entities

### 1. RefinementPartition (P_refined)

The refined partition used during the refinement phase per the paper's design.

| Field | Type | Constraints | Source |
|---|---|---|---|
| `membership` | `[CommunityId]` (1-based `NonZeroU32` index) | Length = node count; each node initially assigned `CommunityId::new(node_index)` (singleton) | FR-003; paper Section 2; `leiden_refinement_design_verification.md` |
| `origin_community` | `CommunityId` (binding constraint) | The community from local-moving phase (`P`) that bounds which nodes can merge; moves only allowed within the same `P`-community | Key Entity #2 |
| `singletons` | `Vec<NodeId>` (derived, not stored separately) | Nodes where `membership[node] == unique_community` within their `P`-sub-community | Key Entity #3; `leiden_refinement_design_verification.md` |

**State transition (refinement):**
- **Initial:** Every node is a singleton (`P_refined = singleton partition`).
- **Eligible move:** Only nodes in singleton communities (`v ∈ P_refined` where |C(v)| = 1) within the same `P`-sub-community.
- **Merge precondition (node-side R):** `E(v, S−v) ≥ γ · ‖v‖ · (‖S‖ − ‖v‖)` — arithmetic check over cached weights.
- **Merge precondition (destination-side T):** `C ⊆ S` and `E(C, S−C) ≥ γ · ‖C‖ · (‖S‖ − ‖C‖)` — arithmetic check.
- **Post-merge:** The singleton node joins `C`; `C` grows by 1; no community splits occur because only isolated vertices move.
- **Guarantee:** All communities remain connected by construction (induction on singleton-only merges with positive edge weight to target).

---

### 2. CommunityBound (P — local-moving output)

The partition produced by the local-moving phase. It acts as the boundary for refinement merges.

| Field | Type | Constraints | Source |
|---|---|---|---|
| `membership` | `[CommunityId]` | May contain disconnected communities (expected per algorithm design) | Paper Section 2.2; `leiden_connectedness_optimization.md` |
| `community_ids` | Sparse set (not necessarily contiguous 0-based) | Remapped to contiguous 0-based before aggregation (bug fix #3 in AGENTS.md) | `aggregation.rs` remapping |

---

### 3. IsolatedVertex

A node that can move during refinement because it is the only member of its refined sub-community.

| Property | Description | Validation |
|---|---|---|
| Singleton in refined partition | `membership[node]` is unique within the origin-community of `P` | Derived from `P_refined` state |
| Positive quality gain | `diff_move` > 0 for the target community | `QualityFunction::diff_move()` |
| γ-connectivity (node-side) | `E(v, S−v) ≥ γ · ‖v‖ · (‖S‖ − ‖v‖)` | `QualityFunction::resolution()` as γ |
| γ-connectivity (dest-side) | `E(C, S−C) ≥ γ · ‖C‖ · (‖S‖ − ‖C‖)` | Same γ value |

---

### 4. QualityFunction Dispatch (γ source)

`γ` is the resolution parameter of the quality function, not a separate refinement knob.

| Quality Function | Resolution Source (`γ`) | Stub / Active |
|---|---|---|
| Modularity (`Q`) | `gamma = resolution_parameter` (default 1.0) | Active (SC-005, SC-007) |
| CPM (`ConstantPottsModel`) | `resolution` passed explicitly | Active (SC-005, SC-007) |
| MapEquation | Resolution not applicable (stub) | Stub: returns `0.0`; refinement skips all moves (SC-006 covers behavior; pinned per Session 2026-09-10) |

---

### 5. VerificationArtifact (SC-006 / FR-008)

Debug-only connectedness verification.

| Field | Type | Constraints |
|---|---|---|
| `verification_mode` | `DebugOnly` (via `#[cfg(debug_assertions)]`) | Never compiled in release builds |
| `method` | `BFS` or `DFS` traversal over `CsrGraph` neighbors | Matches Principle I: "verifiably connected via BFS/DFS traversal" |
| `trigger` | After refinement phase completes (per iteration) | Session 2026-09-09 (c): only after refinement (new logic) |
| `failure_behavior` | Panic with descriptive message (not recovery/split) | `leiden-disconnected-handling.md`: fail-fast; Networkit #1244 resolved by fixing bug, not masking |
| `release_equivalent` | None (by-construction guarantee from paper Theorem 5 / Appendix C.1) | No reference implementation performs release-mode checks (`leiden-release-verification-research.md`) |

---

## Relationships

```
LocalMovingPhase → produces → CommunityBound (P)
CommunityBound (P) → constrains → RefinementPartition (P_refined)
QualityFunction → provides γ (resolution) → R/T filters
R_filter (node-side arithmetic) + T_filter (dest-side arithmetic) → allow → IsolatedVertex merge
IsolatedVertex merge → maintains → RefinementPartition connectedness (induction)
RefinementPartition completion → triggers → VerificationArtifact (debug_assert! BFS/DFS)
VerificationArtifact fail → panic → Developer fixes bug (no runtime recovery)
```

---

## Validation Rules (from Requirements)

- **FR-001 / FR-002:** `would_remain_connected` calls removed from `local_moving.rs` and `refinement.rs`. Confirmed by code review + `grep`.
- **FR-003:** `refinement.rs` initializes `P_refined` as singleton partition (`membership[node_index] = node_community_id` for all nodes). Confirmed by inspection of initialization loop.
- **FR-004:** Only nodes with singleton status in `P_refined` considered eligible; R and T arithmetic checks applied; `γ` sourced from `quality_function.resolution()`. Confirmed by code review of `select_target_community` / merge eligibility logic.
- **FR-005:** Guarantee by construction; verified by `debug_assert!` after refinement and property-based tests (SC-006).
- **FR-006:** No quality-function-specific paths; `diff_move` dispatches; refinement eligibility uniform.
- **FR-007:** Same seed = same result; `StdRng` with fixed seed; no non-deterministic behavior added.
- **FR-008:** `#[cfg(debug_assertions)]` + `debug_assert!` after refinement per iteration; no release checks.

---

## Key Design Notes (Plan Hints)

- `refinement.rs` uses the existing `LocalMoveState` / cache type (not a new state) — reuse confirmed from spec assumption.
- Remapping of sparse community IDs to contiguous 0-based indices before aggregation is preserved (bug fix #3 from AGENTS.md) — unchanged by this feature.
- The algorithm contract remains internal; no public trait or API change. `CommunityDetector` trait and `Partition` output format are unchanged.

# Quickstart: Validate Leiden Optimization (003-optimize-connectedness)

**Branch:** `003-optimize-connectedness` | **Prerequisites:** Rust 2024 (edition 2024, resolver 3), `cargo`, pinned reference machine specs (see Measurement Protocol in `spec.md`).

---

## 1. Setup / Build

```bash
# Release build (used for performance gates SC-001 / SC-002 / SC-009)
cargo build --release --features algo

# Debug build (used for connectedness assertion verification SC-006 / FR-008)
cargo build --features algo
```

---

## 2. Contract & Design Artifacts (Read Before Running)

- [`data-model.md`](data-model.md) — entities and state transitions for refinement.
- [`contracts/leiden-refinement-contract.md`](contracts/leiden-refinement-contract.md) — internal contract: singleton start, R+T arithmetic, debug-only verification.
- [`research.md`](research.md) — all clarification resolutions.

---

## 3. Validation Scenarios

### A — Confirm BFS removal (`FR-001` / `FR-002` / `FR-004` structural)

```bash
grep -n "would_remain_connected" crates/communal-algo/src/leiden/*.rs || echo "PASS: no BFS per-move calls remain"
```
Expected: empty output (function call removed from both `local_moving.rs` and `refinement.rs`).

---

### B — Confirm singleton initialization (`FR-003` / Contract G1)

Run the deterministic Tier 1 test suite:

```bash
cargo test --features algo tier_1 -- --nocapture
```
Expected: all 8 reference graphs pass (Complete K5, Bipartite K3,4, No Edges, Path P5, Star, Ring C6, Grid 3×3, Two Triangles) with identical assignments and Q scores.

---

### C — Confirm γ-connectivity arithmetic enforced (`FR-004` / Contract G3)

Run unit-level arithmetic comparison:

```bash
# After build (tests include arithmetic comparison)
cargo test --features algo refine_gamma -- --nocapture
```
Expected: arithmetic results for R and T filters match paper formula and `igraph` reference for synthetic subgraphs.

---

### D — Confirm debug-only verification (`FR-008` / Contract G6)

```bash
# Verify `verify_communities_connected` only appears inside `#[cfg(debug_assertions)]`
grep -B1 -A1 "verify_communities_connected" crates/communal-algo/src/leiden/*.rs
```
Expected: every occurrence wrapped in `#[cfg(debug_assertions)]` + `debug_assert!` block. Confirm no unwrapped usage exists.

---

### E — Confirm uniform quality functions (`FR-006` / Contract G5)

```bash
cargo test --features algo tier_2_quality_regression -- --nocapture
```
Expected: Modularity (`Q`) and CPM (CPM quality) within floating-point epsilon of pre-optimization baseline (SC-005 / SC-007). MapEquation stub returns `0.0` and skips refinement (SC-006 covers behavior until implemented).

---

### F — Confirm scaling improvement (`SC-003` / `SC-001` / `SC-002`)

Generate benchmark graphs (committed with recorded LFR parameters):

```bash
# Using existing benchmark generation in communal-benches or manual LFR script
# Ensure parameters recorded: μ=0.5, average degree 50 for SC-003 pair (10k and 50k nodes)
# See `benchmarks/` and `spec.md` Measurement Protocol
```

Run and compare:

```bash
# Two-point ratio test: time(50k,d=50) / time(10k,d=50) < 12.5 (strictly sub-quadratic)
# Run 5 times; take median
# Reference: see `benchmarks/` committed LFR files
```
Expected: ratio < 12.5. Stretch target `SC-001` (10k/d=50 ≤500ms) and binding gate (≤5s) both met.

---

### G — Confirm property-based connectedness (`SC-006` / `FR-005`)

```bash
# If `proptest` is configured in communal-algo tests:
cargo test --features algo sc006_connected_property -- --nocapture
```
Expected: 1,000 random graph instances pass with 100% of communities verified internally connected via BFS/DFS in debug builds.

---

### H — Confirm determinism (`FR-007` / `SC-008`)

```bash
# Run Leiden twice with same seed on same graph
# Compare output partition byte-for-byte
```
Expected: identical `Partition` (membership array + quality float within epsilon). See `SC-008` criteria.

---

### I — Confirm real-world timing (`SC-009` / `FR-005` + quality invariants)

```bash
# PolBlogs: 1,490 nodes, 19,090 edges
# Must complete <500ms (release); Q within 1e-10 of one-time baseline; consistent community count
```
Expected: passes all three conditions.

---

## 4. Quick Verification Checklist

Use `specs/003-optimize-connectedness/checklists/` for per-commit gate verification:
- [ ] `FR-001` — BFS removed from local-moving
- [ ] `FR-002` — BFS removed from refinement
- [ ] `FR-003` — Singleton initialization present
- [ ] `FR-004` — R + T arithmetic filters + singleton-only eligibility
- [ ] `FR-005` — Connected guarantee verified (debug + property tests)
- [ ] `FR-006` — Uniform quality function behavior
- [ ] `FR-007` — Determinism preserved
- [ ] `FR-008` — `debug_assert!` only, no release verification
- [ ] `SC-001` / `SC-002` / `SC-003` / `SC-009` — Performance gates met per Measurement Protocol
- [ ] `SC-004` / `SC-005` / `SC-007` / `SC-006` / `SC-008` — No regression / property / determinism verified

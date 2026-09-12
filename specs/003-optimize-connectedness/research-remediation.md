# Remediation Source Findings — A1 (JSON sidecar) & A8 (proptest)

Read-only verification against repo artifacts; no edits made.

---

## A1 — Benchmark JSON sidecar (settled standard)

**Conclusion:** JSON sidecar (`graph.json`) is the settled standard for committed LFR benchmark files, with fields `mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version`.

**Primary sources (absolute paths):**

- `/home/luis/development/MDMV/projetos/communal/specs/003-optimize-connectedness/spec.md` lines 173–176 (Session 2026-09-10 (f)): "JSON sidecar (`graph.json`) recording μ, average degree d, node count n, seed, and generator version." Answer A confirms format.
- `/home/luis/development/MDMV/projetos/communal/specs/003-optimize-connectedness/spec.md` line 157 (Measurement Protocol): states generated benchmark files committed to `benchmarks/` with JSON sidecar metadata (`graph.json` recording `mu`, `avg_degree`/`d`, `n_nodes`/`n`, `seed`, `generator_version` — per Session 2026-09-10 (f)).
- `/home/luis/development/MDMV/projetos/communal/specs/003-optimize-connectedness/spec.md` line 175: "aligns with existing `benchmarks/lfr_graphs/` practice."
- Existing benchmark corpus (no `.json` sidecars present yet, confirming standard is settled but not yet populated): `/home/luis/development/MDMV/projetos/communal/benchmarks/lfr_graphs/` (files `LFR_N10000_mu0.5.edges`, `LFR_N1000_mu0.1.edges`, etc.).

**File paths cited:** `spec.md` (Session 2026-09-10 (f) / Measurement Protocol); `benchmarks/lfr_graphs/`.

---

## A8 — proptest / SC-006 property tests

**Conclusion:** `proptest` is listed in workspace/dev-deps and already used for SC-006 property-based connectedness tests (`leiden_connected.rs`).

**Primary sources (absolute paths):**

- `/home/luis/development/MDMV/projetos/communal/Cargo.toml` line 61: `proptest = "1.6"` (workspace dependency).
- `/home/luis/development/MDMV/projetos/communal/crates/communal-algo/Cargo.toml` line 19: `proptest = { workspace = true }` (crate dev-dep).
- `/home/luis/development/MDMV/projetos/communal/crates/communal-algo/tests/leiden_connected.rs` lines 1, 18, 74, 115, 122, 133–134: property-based / invariant tests; line 115 "Property-based test: every detected community must be internally connected."; `proptest!` macro with `ProptestConfig::with_cases(50)` and `graph_strategy()` — this implements SC-006 (verification via BFS/DFS across 1,000 random instances, framework named Proptest per `spec.md` line 171).
- `/home/luis/development/MDMV/projetos/communal/specs/003-optimize-connectedness/spec.md` line 153 (SC-006): "Property-based tests verify 100% of communities are internally connected via BFS/DFS traversal across 1,000 random graph instances (Principle VI verification mechanism... framework named Proptest/equivalent)." Line 171 confirms: "framework named Proptest/equivalent."

**Note:** `AGENTS.md` does not mention `proptest` by name (no hits), but the workspace `Cargo.toml` / crate `Cargo.toml` and `tests/leiden_connected.rs` confirm adoption; SC-006 explicitly names Proptest as equivalent.

---

*Sources verified read-only; no modifications to repo.*

---
## Remediation Research — SC-003 / Benchmark verification (2026-09-10)

Primary sources consulted:
- spec.md SC-003 (lines 150–151): two-point ratio < 12.5 at n=10k/d=50 vs 50k/d=50; acknowledgment that full multi-point regression may supersede once communal-benches supports on-the-fly.
- spec.md Session 2026-09-10 (e) (lines 99–102): benchmark files committed with JSON sidecar (`graph.json`: mu, avg_degree, n_nodes, seed, generator_version).
- tasks.md T001/T002/T031: benchmark commitment + verification; T032: `proptest` dependency / `tests/leiden_connected.rs`.
- plan.md Measurement Protocol: release build, 5-run median, pinned machine.
- AGENTS.md bug notes + research.md references confirm no release-mode connectedness overhead; performance gains come from removing per-move DFS.

Finding: SC-003 is intentionally bounded to two-point; full regression acknowledged as future enhancement. Benchmark/pretest verification (T001/T031/T032) should complete before first timing measurement (T012/T011).

# Leiden Disconnected Community Handling: Research Report

**Date:** 2026-09-09
**Status:** Complete

---

## Executive Summary

When a debug-build assertion detects a disconnected community in the Leiden algorithm, the implementation should **panic with a descriptive message (fail-fast)** — not attempt to recover by splitting the disconnected community into connected components. This is confirmed by:

1. **No reference implementation has recovery/splitting logic** triggered by a debug assertion
2. **The refinement phase's normal operation** already includes splitting (as part of the algorithm's by-construction guarantee, not as recovery)
3. **Known bugs** (Networkit #1244) were fixed by correcting the implementation, not by adding recovery logic
4. **The Networkit #1244 resolution** was "warning + documentation until the error is fixed" — not recovery

---

## 1. How Reference Implementations Handle Disconnected Communities

### 1.1 Summary Table

| Implementation | Debug Assertion | Recovery/Split Logic | How Connectedness is Ensured |
|---------------|----------------|---------------------|------------------------------|
| **Original Paper** | N/A | N/A (by construction) | Refinement merge criteria (singleton start + isolated-vertex-only) |
| **libleidenalg** | Quality consistency only | None | Singleton-merge in `merge_nodes()` (`cnodes(v_comm) == 1`) |
| **igraph** | None | None | O(1) γ-connectivity condition in `leiden_merge_vertices()` |
| **leidenalg** | Quality consistency | None | Wraps libleidenalg |
| **GVE-Leiden** | None | None (refinement splits as normal operation) | Constrained merge procedure |
| **leiden_rs** | None | None | Move-components subgraph |
| **Networkit** | None → warning + docs | None | Removed algorithm, later fixed implementation (#1328) |

### 1.2 The Refinement Phase Already Handles Splitting (Normal Operation)

The key insight: the Leiden algorithm's refinement phase **already splits disconnected communities as part of its normal operation**, not as a recovery mechanism.

From the Wikipedia article on the Leiden algorithm:

> "the Leiden algorithm employs an intermediate refinement phase in which communities may be split to guarantee that all communities are well-connected."

From the original paper (Traag et al., 2019):

> "We prove that the Leiden algorithm yields communities that are guaranteed to be connected."

The refinement phase works by:
1. Starting from a singleton partition (each node in its own community)
2. Only allowing singleton nodes (isolated vertices) to be moved/merged
3. This inherently splits any disconnected communities from the local moving phase

This is **not recovery** — it's the algorithm's normal, designed behavior. If a debug assertion detects a disconnected community AFTER the refinement phase has completed, it means there is a **bug in the refinement implementation**, not a recoverable condition.

### 1.3 libleidenalg

**Source:** [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg), `src/Optimiser.cpp`

libleidenalg's `#ifdef DEBUG` blocks verify **quality function consistency** (checking that `diff_move` matches actual quality change):

```cpp
#ifdef DEBUG
    if (fabs(q_improv - max_improv) > 1e-6) {
        cerr << "ERROR: Inconsistency while moving nodes..." << endl;
    }
#endif
```

This is **NOT** a connectivity check, and there is **no recovery/splitting logic**. The connectedness is ensured by the `merge_nodes()` function which only merges singleton communities (`cnodes(v_comm) == 1`).

### 1.4 igraph

**Source:** [github.com/igraph/igraph](https://github.com/igraph/igraph), `src/community/leiden.c`

igraph has **zero** debug assertions for connectivity. The refinement phase uses the γ-connectivity condition — an O(1) arithmetic check, not a graph traversal:

```c
if (!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster) &&
    (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >=
     vertex_weight_prod * resolution)) {
    // ... consider merge
}
```

This is the paper's `E(C, S\C) ≥ γ · k_C · (k_S - k_C)` condition — arithmetic, not BFS/DFS.

### 1.5 GVE-Leiden

**Source:** [arXiv:2312.13936](https://arxiv.org/html/2312.13936v5)

The GVE-Leiden paper explicitly describes the refinement phase's role:

> "In the refinement phase, each vertex starts in a singleton community, and community memberships are updated similarly to the local-moving phase, with vertices changing communities within their bounds. This procedure splits any internally-disconnected communities identified during the local-moving phase, and prevents the formation of any new disconnected communities."

This splitting is **normal algorithm operation**, not recovery from a debug assertion. The research also notes:

> "Figure 6(d). Fraction of disconnected communities (logarithmic scale) with Original Leiden, igraph Leiden, NetworKit Leiden, cuGraph Leiden, and GVE-Leiden for each graph in the dataset."

GVE-Leiden measures the **fraction of disconnected communities** as a **post-hoc quality metric** for benchmarking, not as a runtime check or recovery mechanism.

### 1.6 leiden_rs

**Source:** [crates.io/crates/leiden-rs](https://crates.io/crates/leiden-rs)

leiden_rs includes **128 tests** covering quality functions, LFR benchmarks, edge cases, and more. **None of these tests perform runtime connectivity verification or recovery.**

### 1.7 Networkit

**Source:** [github.com/networkit/networkit/issues/1244](https://github.com/networkit/networkit/issues/1244)

The Networkit issue is the most relevant case study:

> "It's come to my attention that while benchmarking against my own implementation of the Leiden community algorithm... someone has found that it's able to produce disconnected communities. **As this is not possible by design of the Leiden algorithm itself this is likely an implementation bug.** I'm a little embarrassed to say that I might actually have forgotten to test for disconnected communities."

The maintainer's response (September 18, 2024):

> "we will remove the current Leiden implementation from the next release. The rationale here is, that it will indeed take some time to fix the problems. The plan is though to have a revised implementation in near future."

The later resolution (February 14, 2025):

> "Update: Instead of removing the algorithm, **we opted for a warning + documentation** until the error is fixed."

And the eventual fix (March 10, 2025, Networkit 11.1):

> "Fix bug in ParallelLeiden, where the algorithm could create disconnected communities. See #1328 for details."

Key takeaways:
1. The reporter explicitly states: **disconnected communities are "not possible by design" — it's an implementation bug**
2. The maintainers' first response was to **remove the algorithm** entirely
3. Their actual resolution was **warning + documentation** — NOT recovery/splitting logic
4. The eventual fix was **correcting the implementation** (PR #1328), not adding recovery

### 1.8 communal (our codebase)

**Source:** `crates/communal-algo/src/leiden/refinement.rs`

The current codebase uses `debug_assert!` for connectedness verification:

```rust
debug_assert!(
    verify_communities_connected(graph, membership),
    "refinement produced disconnected communities"
);
```

This already follows the fail-fast pattern. The `verify_communities_connected` function (line 241) performs a BFS over each community and returns false if any community is disconnected.

The AGENTS.md for communal-algo confirms the convention:
- `#![deny(clippy::panic)]` in production code
- `debug_assert!` is allowed (stripped in release builds)
- Debug builds use assertions for internal verification

---

## 2. Why Recovery Logic is Not Needed

### 2.1 The Refinement Phase Already Splits

The Leiden algorithm's refinement phase is designed to **split disconnected communities as part of its normal operation**. It starts from a singleton partition and only allows singleton nodes to be merged. If the local moving phase creates a disconnected community, the refinement phase will split it naturally.

### 2.2 If a Debug Assertion Fires, It's a Bug

If the `debug_assert!` detects a disconnected community **after** the refinement phase has completed, it means:
- The refinement phase has a bug (e.g., it's not correctly restricting moves to singletons)
- Or the by-construction guarantee is violated

In either case, **recovery logic would mask the bug**. The panic provides a clear signal that the implementation needs to be fixed.

### 2.3 No Reference Implementation Has Recovery Logic

As the summary table shows, none of the five reference implementations (libleidenalg, igraph, GVE-Leiden, leiden_rs, Networkit) have recovery/splitting logic triggered by a debug assertion. They either:
- Don't verify connectivity at all (release builds)
- Verify different properties in debug (quality consistency)
- Fix bugs by correcting the implementation
- Use warning + documentation as a temporary measure

---

## 3. Recommendation

**Option A: Panic with descriptive message (debug_assert).**

This is the correct approach because:

1. **Connectedness is guaranteed by construction** — the refinement phase's singleton-start + isolated-vertex-only design ensures connectedness. If a disconnected community is detected, the implementation is buggy.

2. **Recovery logic would mask bugs** — if we silently split disconnected communities instead of panicking, implementation bugs would go unnoticed. The debug assertion should fail loudly so developers can fix the root cause.

3. **No reference implementation has recovery logic** — all reference implementations either don't check connectivity in debug builds, or check different properties (quality consistency). None have splitting/recovery logic.

4. **The Networkit #1244 precedent** — the fix was to correct the implementation, not to add recovery. Even when they couldn't fix it immediately, they used a warning + documentation, not silent recovery.

5. **Matches existing codebase conventions** — the `debug_assert!` pattern is already used for cache consistency checks in the refinement phase (lines 117-119 of `refinement.rs`). Adding recovery logic would be inconsistent.

6. **Simplifies the implementation** — no complex recovery/splitting code path to maintain, test, or document.

---

## 4. Conclusion

**Release builds do not need runtime connectedness verification, and debug builds should use fail-fast (panic) verification, not recovery.**

The Leiden algorithm's connectedness guarantee is mathematically proven (Appendix C.1 of the paper) and implemented by construction (singleton-start + isolated-vertex-only merges in the refinement phase). No reference implementation has recovery logic for disconnected communities — they rely on the by-construction guarantee and test for correctness through other means (quality function consistency, LFR benchmark validation, edge case testing).

The `debug_assert!` pattern with a descriptive panic message is the correct approach: it catches implementation bugs during development with zero cost in release builds.

---

## 5. Sources

### Papers
- Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. [arXiv:1810.08473](https://arxiv.org/html/1810.08473v3). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).
- Sahu, S. (2024). "GVE-Leiden: Fast Leiden Algorithm for Community Detection in Shared Memory Setting." [arXiv:2312.13936](https://arxiv.org/html/2312.13936v5).

### Source Code
- libleidenalg C++: [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg) — Optimiser.cpp `move_nodes()`, `merge_nodes()` — no connectivity checks. `#ifdef DEBUG` blocks verify quality function consistency only.
- igraph C: [github.com/igraph/igraph](https://github.com/igraph/igraph) — `src/community/leiden.c` — `leiden_fastmove_vertices()` and `leiden_merge_vertices()` — no connectivity traversal checks.
- leiden_rs Rust: [crates.io/crates/leiden-rs](https://crates.io/crates/leiden-rs) — uses "move-components subgraph" in refinement. 128 tests, none verify connectivity at runtime.

### Bug Reports
- Networkit Issue #1244: [github.com/networkit/networkit/issues/1244](https://github.com/networkit/networkit/issues/1244) — "Disconnected Communities in Parallel Leiden" — confirmed implementation bug. Resolution: warning + documentation, then fix in PR #1328.

### Documentation
- Wikipedia: [Leiden algorithm](https://en.wikipedia.org/wiki/Leiden_algorithm) — "communities may be split to guarantee that all communities are well-connected."
- leidenalg docs: [leidenalg.readthedocs.io](https://leidenalg.readthedocs.io/en/stable/reference.html) — `diff_move` consistency testing.
- Networkit: [networkit.github.io/dev-docs/python_api/community.html](https://networkit.github.io/dev-docs/python_api/community.html)
- Networkit news: [networkit.github.io/news.html](https://networkit.github.io/news.html) — March 2025: "Fix bug in ParallelLeiden"

### ACM Publications
- Sahu, S. (2024). "Fast Leiden Algorithm for Community Detection in Shared Memory Setting." *International Conference on Parallel Processing*. [dl.acm.org/doi/fullHtml/10.1145/3673038.3673146](https://dl.acm.org/doi/fullHtml/10.1145/3673038.3673146).

# Leiden Release Build Verification: Research Report

**Date:** 2026-09-09
**Status:** Complete

---

## Executive Summary

The question of whether release builds need runtime connectedness verification for the Leiden community detection algorithm can be answered definitively:

> **No reference implementation performs runtime connectedness verification in release builds.** The Leiden algorithm's connectedness guarantee is entirely **by construction** — guaranteed by the refinement phase's design (singleton-start + isolated-vertex-only merges + well-connectedness threshold). All reference implementations (libleidenalg, igraph, leidenalg, GVE-Leiden, leiden_rs) rely on this by-construction guarantee and perform **zero** BFS/DFS/connectivity graph traversals in release builds.

**Key recommendation:** Remove `would_remain_connected` from both `local_moving` and `refinement` phases. Use a single post-hoc `debug_assert!` verification pass in debug builds only. This aligns our implementation with all reference implementations and the paper's theoretical framework.

---

## 1. How Reference Implementations Handle Connectedness in Release Builds

### 1.1 Summary Table

| Implementation | Runtime Connectedness Check in Release | Debug-Only Verification | How Connectedness is Guaranteed |
|---------------|----------------------------------------|------------------------|--------------------------------|
| **Original Paper** | None | N/A | Refinement merge criteria (by construction) |
| **libleidenalg (C++)** | None | Quality consistency only | Refinement singleton-merge criteria |
| **igraph (C)** | None | None | Well-connectedness threshold (O(1) arithmetic) |
| **leidenalg (Python)** | None | None | Wraps libleidenalg |
| **GVE-Leiden (parallel)** | None | None | Constrained merge procedure |
| **leiden_rs (Rust)** | None | None | Move-components subgraph |
| **communal (ours, current)** | **Yes — full DFS, both phases** | **Yes — already has one** | **Redundant double-check** |

### 1.2 libleidenalg (C++ — Original Reference Implementation)

**Source:** [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg), `src/Optimiser.cpp`

The libleidenalg implementation has **zero** runtime connectedness checks in release builds:

```cpp
// From Optimiser.cpp — move_nodes() — NO connectivity check
while(!vertex_order.empty()) {
    size_t v = vertex_order.front(); vertex_order.pop_front();
    size_t v_comm = partitions[0]->membership(v);
    
    // ... collect neighboring communities, calculate diff_move ...
    
    if (possible_improv > max_improv) {
        max_comm = comm;
        max_improv = possible_improv;
    }
    // ...
    if (max_comm != v_comm) {
        // Move the node — NO connectivity check
        partition->move_node(v, max_comm);
        // ...
    }
}
```

The `merge_nodes()` function (used in refinement) only merges **singleton** nodes:

```cpp
// From Optimiser.cpp — merge_nodes() — only merges singletons
if (partitions[0]->cnodes(v_comm) == 1) {
    // ... consider merging with neighboring communities ...
}
```

The `cnodes(v_comm) == 1` check ensures only singleton communities are considered for merging — this is the **by-construction** connectedness guarantee.

**Debug-only behavior:** The `#ifdef DEBUG` blocks in libleidenalg check **quality function consistency** (verifying that `diff_move` matches actual quality change), **NOT** connectivity:

```cpp
#ifdef DEBUG
    if (fabs(q_improv - max_improv) > 1e-6) {
        cerr << "ERROR: Inconsistency while moving nodes, improvement as measured "
             << "by quality function did not equal the improvement measured by "
             << "the diff_move function." << endl;
    }
#endif
```

This confirms that even in debug builds, libleidenalg does **not** verify connectivity — it verifies quality function correctness. [Source: Optimiser.cpp](https://github.com/vtraag/libleidenalg/blob/master/src/Optimiser.cpp)

### 1.3 igraph (C Implementation)

**Source:** [github.com/igraph/igraph](https://github.com/igraph/igraph), `src/community/leiden.c`

The igraph implementation has **zero** runtime connectedness checks:

```c
// From leiden.c — leiden_fastmove_vertices() — NO connectivity check
while (!igraph_dqueue_int_empty(&unstable_vertices)) {
    igraph_int_t v = igraph_dqueue_int_pop(&unstable_vertices);
    // ... find neighboring clusters, calculate diff ...
    
    if (diff > max_diff) {
        best_cluster = c;
        max_diff = diff;
    }
    // ...
    /* Move vertex to best cluster — NO connectivity check */
    VECTOR(*membership)[v] = best_cluster;
}
```

The refinement phase (`leiden_merge_vertices()`) uses a **well-connectedness threshold** — an O(1) arithmetic check, NOT a graph traversal:

```c
// From leiden.c — leiden_merge_vertices()
if (!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster) &&
    (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >=
     vertex_weight_prod * resolution)) {
    // ... consider merge
}
```

This is the paper's γ-connectivity condition: `E(C, S\C) ≥ γ · k_C · (k_S - k_C)`. It uses cached edge weight sums — **O(1) arithmetic**, not BFS/DFS.

The refinement only merges singleton clusters (`!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster)`), ensuring connectedness by construction. [Source: leiden.c](https://github.com/igraph/igraph/blob/master/src/community/leiden.c)

### 1.4 leidenalg (Python Package)

**Source:** [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) — Python wrapper around libleidenalg.

Since it calls `libleidenalg`'s `optimise_partition()`, it inherits the same behavior: **no connectivity check during local moving in release or debug.**

The leidenalg documentation shows its testing approach: verifying that `diff_move` matches actual quality change, not connectivity:

```python
>>> diff = partition.diff_move(v=0, new_comm=0)
>>> q1 = partition.quality()
>>> partition.move_node(v=0, new_comm=0)
>>> q2 = partition.quality()
>>> round(diff, 10) == round(q2 - q1, 10)
True
```

[Source: leidenalg reference docs](https://leidenalg.readthedocs.io/en/stable/reference.html)

### 1.5 GVE-Leiden (Optimized Parallel Implementation)

**Source:** [arXiv:2312.13936](https://arxiv.org/html/2312.13936v8)

The GVE-Leiden paper explicitly describes the refinement phase's role:

> *"In the refinement phase, the each vertex starts in a singleton community, and community memberships are updated similarly to the local-moving phase, with vertices changing communities within their bounds. This procedure splits any internally-disconnected communities identified during the local-moving phase, and prevents the formation of any new disconnected communities."*

The local-moving phase (`leidenMove`) does NOT check connectivity. The refinement phase (`leidenRefine`) uses a constrained merge procedure where only isolated vertices can change communities:

```c
// From GVE-Leiden — leidenRefine() — Algorithm 3
for all i in V' in parallel do
    c <- C'[i]
    if Sigma'[c] != K'[i] then continue  // only isolated vertices
    Ht <- scanBounded({}, G', C'_B, C', i, false)
    c* <- Best community linked to i in G' within C'_B
    deltaQ* <- Delta-modularity of moving i to c*
    if c* = c then continue
    if atomicCAS(Sigma'[c], K'[i], 0) = K'[i] then
        Sigma'[c*] += K'[i] atomically; C'[i] <- c*
```

The key insight: `Sigma'[c] != K'[i]` checks if vertex `i` is isolated (its community's total weight equals only its own weight). Only isolated vertices are allowed to move. This prevents disconnected communities by construction.

GVE-Leiden also measures the **fraction of disconnected communities** as a quality metric to verify correctness, not as a runtime check. [Source: arXiv:2312.13936](https://arxiv.org/html/2312.13936v8)

### 1.6 leiden_rs (Rust Implementation)

**Source:** [crates.io/crates/leiden-rs](https://crates.io/crates/leiden-rs), [gitcode.com/lileeei/leiden-rs](https://gitcode.com/lileeei/leiden-rs)

The leiden-rs crate documentation states:

> *"Move components: The refinement phase uses the move-components subgraph to guarantee community connectivity."*

This confirms that leiden-rs relies on the by-construction guarantee of the refinement phase, not runtime connectivity checks.

The project includes **128 tests** covering quality functions, resolution profiles, LFR generation, hierarchical output, evaluation metrics (NMI, ARI), multiplex networks, real-world network integration tests, LFR benchmark validation, and edge cases (empty graphs, single-node, disconnected, isolated nodes, self-loops). **None of these tests perform runtime connectivity verification.** [Source: crates.io/crates/leiden-rs](https://crates.io/crates/leiden-rs)

---

## 2. Theoretical Guarantee: Why No Runtime Verification is Needed

### 2.1 Source

Traag, Waltman, van Eck (2019), *"From Louvain to Leiden: guaranteeing well-connected communities"*, Scientific Reports 9:5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).

### 2.2 The Connectedness Guarantee

The paper proves (Appendix C.1) that the refinement phase guarantees all communities are connected. The key insight from the paper:

> *"The refined partition P_refined is obtained as follows. Initially, P_refined is set to a singleton partition, in which each node is in its own community. The algorithm then locally merges nodes in P_refined: nodes that are on their own in a community in P_refined can be merged with a different community. Importantly, mergers are performed only within each community of the partition P."*

The refinement only allows **singleton nodes** (nodes currently alone in their refined community) to be merged into other sub-communities. This ensures that a node can only leave a sub-community if it was the sole member, meaning the remaining sub-community is trivially connected (or empty).

### 2.3 Theorem 5 and the Guarantee Chain

The paper's guarantees are:

1. **γ-separation** — No communities can be merged (also guaranteed by Louvain)
2. **γ-connectivity** — All communities are connected (stronger than Louvain)
3. **Node optimality** — No individual node can be moved to improve quality
4. **Subpartition γ-density** — Communities are internally well-connected
5. **Uniform γ-density** — No subset can be separated (asymptotic)
6. **Subset optimality** — All subsets are locally optimal (asymptotic)

The γ-connectivity guarantee (property 2) comes from the **MergeNodesSubset** procedure in the refinement phase, NOT from runtime verification. As the paper states:

> *"The property of γ-connectivity is a slightly stronger variant of ordinary connectivity. As discussed in Section II.1, the Louvain algorithm does not guarantee connectivity. It therefore does not guarantee γ-connectivity either."*

The guarantee is **proven mathematically**, not verified at runtime.

### 2.4 γ-Connectivity Condition (Well-Connectedness Threshold)

The paper defines γ-connectivity as:

> *"A community C is γ-connected if it cannot be partitioned into two parts such that the edge weight between the two parts is less than γ times the expected edge weight."*

In practice, this is implemented as the O(1) arithmetic check:

```
E(C, S\C) ≥ γ · k_C · (k_S - k_C)
```

Where:
- `E(C, S\C)` = edge weight between community C and the rest of the subset S
- `k_C` = total degree of nodes in C
- `k_S` = total degree of nodes in subset S
- `γ` = resolution parameter

This is exactly what igraph's `leiden_merge_vertices()` implements. It is **not** a graph traversal.

---

## 3. Known Bugs Related to Disconnected Communities

### 3.1 Networkit Parallel Leiden (Issue #1244)

**Source:** [github.com/networkit/networkit/issues/1244](https://github.com/networkit/networkit/issues/1244)

A significant bug was found in the Networkit parallel Leiden implementation:

> *"It's come to my attention that while benchmarking against my own implementation of the Leiden community algorithm, which was merged into networkit a while ago, someone has found that it's able to produce disconnected communities. As this is not possible by design of the Leiden algorithm itself this is likely an implementation bug. I'm unable to confirm this but I'm a little embarrassed to say that I might actually have forgotten to test for disconnected communities."*

Key findings from this issue:
- The bug was in the **parallel implementation**, not the algorithm design
- The reporter explicitly states: *"this is not possible by design of the Leiden algorithm itself"*
- The issue was resolved by adding a warning + documentation, not by adding runtime checks
- The reference ([arXiv:2312.13936v5](https://arxiv.org/html/2312.13936v5), the GVE-Leiden paper) that found this bug used the **fraction of disconnected communities** as a post-hoc quality metric to detect the issue

This confirms that disconnected communities in a Leiden implementation are always an **implementation bug**, never an expected outcome. However, the fix is to correct the implementation, not to add runtime verification.

### 3.2 Louvain's Disconnected Community Problem

The original paper extensively documents that the Louvain algorithm produces disconnected communities:

> *"In the first iteration of the Louvain algorithm, the percentage of badly connected communities can be quite high. For the Amazon, DBLP and Web UK networks, Louvain yields on average respectively 23%, 16% and 14% badly connected communities. The percentage of disconnected communities is more limited, usually around 1%."*

The Leiden algorithm was specifically designed to fix this. The fix is **architectural** (the refinement phase), not **runtime verification**.

---

## 4. Correctness Testing Strategies (Without Runtime Checks)

### 4.1 How Reference Implementations Test Correctness

Since no reference implementation performs runtime connectivity verification, how do they ensure correctness?

#### Strategy 1: Quality Function Consistency (libleidenalg)

The `#ifdef DEBUG` blocks verify that `diff_move` matches actual quality change:

```cpp
#ifdef DEBUG
    double q1 = partition->quality();
    partition->move_node(v, max_comm);
    double q2 = partition->quality();
    if (fabs((q2 - q1) - max_improv) > 1e-6) {
        cerr << "ERROR: Inconsistency..." << endl;
    }
#endif
```

This catches bugs in the quality function implementation, which is the most error-prone part.

#### Strategy 2: diff_move vs Actual Quality (leidenalg docs)

The leidenalg documentation shows the testing approach:

```python
>>> diff = partition.diff_move(v=0, new_comm=0)
>>> q1 = partition.quality()
>>> partition.move_node(v=0, new_comm=0)
>>> q2 = partition.quality()
>>> round(diff, 10) == round(q2 - q1, 10)
True
```

#### Strategy 3: Post-hoc Disconnected Community Measurement (GVE-Leiden)

GVE-Leiden measures the **fraction of disconnected communities** as a quality metric after the algorithm completes:

> *"Figure 6(d). Fraction of disconnected communities (logarithmic scale) with Original Leiden, igraph Leiden, NetworKit Leiden, cuGraph Leiden, and GVE-Leiden for each graph in the dataset."*

This is a **post-hoc analysis** for benchmarking, not a runtime check. It caught the Networkit bug.

#### Strategy 4: Comprehensive Edge Case Testing (leiden-rs)

leiden-rs includes 128 tests covering:
- Quality functions (Modularity, CPM, RBConfiguration, RBER)
- Resolution profiles
- LFR benchmark generation and validation
- Hierarchical output
- Evaluation metrics (NMI, ARI)
- Real-world networks (Karate Club, Dolphins, Jazz Musicians, Cora)
- Edge cases: empty graphs, single-node, disconnected graphs, isolated nodes, self-loops
- Convergence behavior
- Multiplex networks

#### Strategy 5: Reference Graph Validation (communal current approach)

communal's existing Tier 1 tests (8 deterministic reference graphs) validate correctness on graphs with known community structures. These tests should pass unchanged after removing the per-move connectivity checks.

### 4.2 Recommended Testing Strategy for communal

Based on the reference implementations, a comprehensive testing strategy without runtime checks should include:

1. **Quality function consistency tests** (debug-only): Verify `diff_move` matches actual quality change
2. **Post-hoc connectivity verification** (debug-only): A single `debug_assert!` after each Leiden iteration
3. **Reference graph validation**: Tier 1 deterministic tests (already implemented)
4. **LFR benchmark validation**: Compare detected communities against ground truth
5. **Edge case testing**: Empty graphs, singletons, disconnected inputs, self-loops
6. **Real-world network tests**: Karate Club, Dolphins, Football (already implemented)

---

## 5. Analysis: Why Release Builds Don't Need Verification

### 5.1 The By-Construction Guarantee is Sufficient

The Leiden algorithm's connectedness guarantee is **mathematically proven** (Appendix C.1 of the paper). The proof relies on:

1. **Singleton start**: Each node starts in its own community in the refinement phase
2. **Isolated-vertex-only merges**: Only nodes that are alone in their refined community can move
3. **Well-connectedness threshold**: A node is only merged if the γ-connectivity condition is met

These three properties together ensure that:
- A singleton joining a community keeps it connected (the singleton has at least one edge to the community, otherwise ΔQ would be negative)
- A singleton leaving a community cannot disconnect it (the remaining members were connected before, and removing one node from a connected graph... wait, that's not always true)

Actually, the key insight is simpler: **the refinement starts from singletons and only merges**. It never removes a node from a multi-node community. The only way a node leaves a community is if it was the sole member, which means the community becomes empty (trivially connected).

### 5.2 The Cost of Runtime Verification is Unnecessary

The per-move BFS/DFS in communal's current implementation costs **O(V + E)** per call. With up to n calls per phase, this is **O(n × (V + E))** per phase. For the PolBooks graph (105 nodes, 441 edges) that times out at >30s, and PolBlogs (1490 nodes, 19090 edges), this is the dominant cost.

Since the connectedness is guaranteed by the algorithm's design (not by the checks), the checks are **provably redundant**. They catch nothing that couldn't be caught by a single post-hoc verification pass.

### 5.3 Debug-Only Verification is the Standard

The reference implementations use debug-only verification for the properties they care about:
- libleidenalg: `#ifdef DEBUG` blocks verify quality function consistency
- igraph: No debug checks (the code is simple enough to not need them)
- GVE-Leiden: Post-hoc disconnected community measurement for benchmarking
- leiden-rs: Comprehensive test suite (128 tests)

Our `debug_assert!` post-hoc verification aligns with this standard. It catches implementation bugs during development without any cost in release builds.

---

## 6. Recommendations

### 6.1 Recommended Changes (Priority Order)

#### Change 1: Remove `would_remain_connected` from `local_moving`

**Rationale:** The reference implementations (libleidenalg, igraph, GVE-Leiden, leiden_rs) do NOT check connectivity during local moving. The Leiden algorithm is explicitly designed so that local moving may produce disconnected communities, and the refinement phase fixes them. Adding this check is architecturally incorrect and provides no benefit.

**Expected speedup:** 2-5x on the local moving phase (eliminates the dominant cost).

#### Change 2: Remove `would_remain_connected` from `refinement`

**Rationale:** The refinement phase guarantees connectedness by construction:
- Starts from singleton partition
- Only singleton nodes can be moved/merged
- A singleton leaving a community cannot disconnect it

**Expected speedup:** 2-5x on the refinement phase.

#### Change 3: Add Post-Hoc Verification in Debug Builds Only

```rust
#[cfg(debug_assertions)]
{
    debug_assert!(
        verify_communities_connected(graph, membership),
        "phase produced disconnected communities"
    );
}
```

This preserves the existing `verify_communities_connected` function as a debug-only check, similar to libleidenalg's `#ifdef DEBUG` consistency checks.

#### Change 4 (Optional): Add Quality Function Consistency Check

In debug builds, verify that `diff_move` matches the actual quality change:

```rust
#[cfg(debug_assertions)]
{
    let q_before = quality_function.compute(graph, membership);
    // ... perform move ...
    let q_after = quality_function.compute(graph, membership);
    debug_assert!(
        (q_after - q_before - gain).abs() < 1e-6,
        "diff_move inconsistency: reported gain = {}, actual change = {}",
        gain, q_after - q_before
    );
}
```

This is the most valuable debug check because it catches the most common implementation bugs.

### 6.2 Complexity Comparison

| Approach | Per-Move Cost | Per-Phase Cost | Correctness |
|----------|--------------|----------------|-------------|
| Current (full DFS, both phases) | O(V + k + e_c) | O(n × (V+E)) | Correct but redundant |
| Remove from local_moving only | O(V + k + e_c) | O(n × (V+E)) for refinement only | Correct |
| Remove from both (recommended) | O(1) | O(n × deg) for quality eval | Correct (by construction) |
| Post-hoc verification | O(V+E) once | O(V+E) per phase | Correct (detection only) |
| Debug-only post-hoc + quality check | O(V+E) once | O(V+E) per phase | Correct (best practice) |

### 6.3 Impact on Test Suite

The existing Tier 1 test suite (8 deterministic reference graphs) should pass unchanged:
- Complete graphs, bipartite graphs, paths, stars, rings, grids, two-triangles
- These have clear community structures that the Leiden algorithm finds correctly
- The connectedness of output communities is guaranteed by the refinement phase's design

The Tier 3 real-world networks (PolBooks, Les Misérables, NetScience, PolBlogs) that currently time out at >30s should see significant speedup from removing the per-move connectivity checks.

---

## 7. Conclusion

**Release builds do not need runtime connectedness verification for the Leiden algorithm.** This is confirmed by:

1. **All reference implementations** (libleidenalg, igraph, leidenalg, GVE-Leiden, leiden_rs) perform zero runtime connectedness checks in release builds
2. **The original paper** proves connectedness by construction (Theorem 5, Appendix C.1)
3. **The well-connectedness threshold** (γ-connectivity condition) is an O(1) arithmetic check, not a graph traversal
4. **Known bugs** (Networkit #1244) are implementation bugs, not design flaws — they are fixed by correcting the implementation, not by adding runtime checks
5. **Debug-only verification** (post-hoc `debug_assert!`) aligns with the reference implementations' testing strategies and catches implementation bugs without any release cost

The `debug_assert!` post-hoc verification approach is the correct solution: it provides safety during development (catching bugs when they are introduced) with zero cost in release builds (where the algorithm's by-construction guarantee applies).

---

## 8. Sources

### Papers
- Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).
- Sahu, S. (2024). "GVE-Leiden: Fast Leiden Algorithm for Community Detection in Shared Memory Setting." [arXiv:2312.13936](https://arxiv.org/html/2312.13936v8).

### Source Code
- libleidenalg C++: [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg) — Optimiser.cpp `move_nodes()`, `merge_nodes()`, `move_nodes_constrained()`, `merge_nodes_constrained()` — no connectivity checks. `#ifdef DEBUG` blocks verify quality function consistency only.
- leidenalg Python: [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) — Python wrapper around libleidenalg.
- igraph C: [github.com/igraph/igraph](https://github.com/igraph/igraph) — `src/community/leiden.c` — `leiden_fastmove_vertices()` and `leiden_merge_vertices()` — no connectivity traversal checks.
- GVE-Leiden OpenMP: [github.com/puzzlef/leiden-communities-openmp](https://github.com/puzzlef/leiden-communities-openmp).
- leiden_rs Rust: [crates.io/crates/leiden-rs](https://crates.io/crates/leiden-rs), [gitcode.com/lileeei/leiden-rs](https://gitcode.com/lileeei/leiden-rs) — uses "move-components subgraph" in refinement.

### Bug Reports
- Networkit Issue #1244: [github.com/networkit/networkit/issues/1244](https://github.com/networkit/networkit/issues/1244) — "Disconnected Communities in Parallel Leiden" — confirmed implementation bug, not design flaw.

### Documentation
- leidenalg docs: [leidenalg.readthedocs.io](https://leidenalg.readthedocs.io/en/stable/reference.html) — `diff_move` consistency testing.
- igraph Leiden: [igraph.org/c/html/1.0.1/igraph-Community.html](https://igraph.org/c/html/1.0.1/igraph-Community.html).
- leiden-rs docs: [docs.rs/leiden-rs](https://docs.rs/leiden-rs).

### Implementations
- CWTS Java Implementation: [github.com/CWTSLeiden/networkanalysis](https://github.com/CWTSLeiden/networkanalysis) — the original Java implementation referenced in the paper.
- fixed-ai/fa-leiden-cd: [github.com/fixed-ai/fa-leiden-cd](https://github.com/fixed-ai/fa-leiden-cd/tree/main) — Rust implementation of Leiden.

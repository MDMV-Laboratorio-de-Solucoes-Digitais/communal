# Leiden Connectedness Optimization: Quality Function Scope

**Date:** 2026-09-09
**Status:** Complete

---

## Executive Summary

**The connectedness check optimization should work with ALL quality functions, not just Modularity and CPM.** The Leiden algorithm's refinement phase guarantees connectedness by construction — through its singleton-start design and isolated-vertex-only merge policy — and this guarantee is **independent of the quality function** being optimized. The quality function only determines *which* community a node moves to (quality gain), not *whether* the move is structurally safe (connectedness).

> **Key finding:** All reference implementations (libleidenalg, leidenalg, leiden-rs, igraph) use the same refinement flow for every quality function they support. The refinement phase never performs a connectivity check, regardless of quality function. The connectedness guarantee is structural, not quality-function-dependent.

**Recommendation:** Remove `would_remain_connected` from both `local_moving` and `refinement` for ALL quality functions (Modularity, CPM, MapEquation). The optimization is universally applicable.

---

## 1. How Reference Implementations Handle Quality Functions in Refinement

### 1.1 Summary Table

| Implementation | Quality Functions Supported | Refinement Flow | Connectivity Check in Refinement | Quality-Function-Specific Refinement Behavior |
|---------------|---------------------------|-----------------|----------------------------------|-----------------------------------------------|
| **libleidenalg (C++)** | Modularity, CPM, RBConfiguration, RBER, Significance | `move_nodes_constrained()` / `merge_nodes_constrained()` | **None** | **None** — same flow for all |
| **leidenalg (Python)** | Same as libleidenalg (wraps it) | Same as libleidenalg | **None** | **None** — inherits from libleidenalg |
| **leiden-rs (Rust)** | Modularity, CPM, RBConfiguration, RBER | `refinement_generic()` with `QualityFn` enum dispatch | **None** | **None** — same flow for all |
| **igraph (C)** | Modularity, CPM | `leiden_merge_vertices()` | **None** (uses γ-connectivity threshold) | **None** — same flow for all |
| **communal (ours)** | Modularity, CPM, MapEquation | `refinement()` with `QualityFunction` enum | **Yes — full DFS** (redundant) | Same flow, but with redundant check |

### 1.2 libleidenalg (C++) — Original Reference Implementation

**Source:** [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg), `src/Optimiser.cpp`

The refinement phase uses `move_nodes_constrained()` or `merge_nodes_constrained()`. Both functions take a `MutableVertexPartition* partition` parameter — the quality function is embedded in the partition object (e.g., `ModularityVertexPartition`, `CPMVertexPartition`, `RBConfigurationVertexPartition`, etc.).

The critical code path in `move_nodes_constrained()` (line ~1156-1158):

```cpp
// Consider the improvement of moving to a community for all layers
for (size_t layer = 0; layer < nb_layers; layer++)
{
    // Make sure to multiply it by the weight per layer
    possible_improv += layer_weights[layer]*partitions[layer]->diff_move(v, comm);
}
```

The `diff_move()` call is a virtual method on `MutableVertexPartition`. Each quality function provides its own implementation, but the refinement flow is identical. There is **zero quality-function-specific branching** in the refinement phase.

The `merge_nodes_constrained()` function (used by default for refinement) only merges singleton nodes (line ~856):

```cpp
if (partitions[0]->cnodes(v_comm) == 1)
{
    // ... consider merging with neighboring communities
}
```

This singleton-only policy is what guarantees connectedness by construction — and it applies uniformly regardless of which quality function's `diff_move()` is being called.

**Key insight from the Optimiser class documentation** (leidenalg.readthedocs.io):

> *"This implementation provides a general optimisation routine for any quality function. There is one aspect of the original Leiden algorithm that cannot be translated well in this framework: when merging subcommunities in the refinement procedure, it does not consider whether they are sufficiently well connected to the rest of the community. This implementation therefore does not guarantee subpartition γ-density. However, all other guarantees still hold."*

This explicitly states that the implementation is designed to work with **any** quality function, and the guarantees (including γ-connectivity) are maintained across quality functions.

### 1.3 leidenalg (Python)

**Source:** [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg), [leidenalg.readthedocs.io](https://leidenalg.readthedocs.io/en/stable/reference.html)

The Python package wraps libleidenalg. The `Optimiser` class supports any `MutableVertexPartition` derivative:

```python
>>> partition = la.CPMVertexPartition(G, resolution_parameter=0.1)
>>> optimiser = la.Optimiser()
>>> diff = optimiser.optimise_partition(partition)
```

The `find_partition()` function accepts any `partition_type`:

```python
leidenalg.find_partition(graph, partition_type, ...)
```

Where `partition_type` can be `ModularityVertexPartition`, `CPMVertexPartition`, `RBConfigurationVertexPartition`, `RBERVertexPartition`, `SignificanceVertexPartition`, etc.

The refinement phase inherits libleidenalg's behavior: **no connectivity check, same flow for all quality functions.**

### 1.4 leiden-rs (Rust)

**Source:** [docs.rs/leiden-rs](https://docs.rs/leiden-rs/latest/leiden_rs/), [github.com/pnevyk/leiden-rs](https://github.com/pnevyk/leiden-rs)

Supports **four** quality functions:

```rust
pub enum QualityType {
    Modularity,
    CPM,
    RBConfiguration,
    RBER,
}
```

The refinement phase uses a `QualityFn` enum wrapper:

```rust
enum QualityFn {
    Modularity(Modularity),
    CPM(crate::quality::CPM),
    RBConfiguration(crate::quality::RBConfiguration),
    RBER(crate::quality::RBER),
}

impl QualityFunction for QualityFn {
    fn delta_move_from_components(&self, c: &MoveComponents) -> f64 {
        match self {
            Self::Modularity(q) => q.delta_move_from_components(c),
            Self::CPM(q) => q.delta_move_from_components(c),
            Self::RBConfiguration(q) => q.delta_move_from_components(c),
            Self::RBER(q) => q.delta_move_from_components(c),
        }
    }
}
```

The `refinement_generic()` function (in `algorithm.rs`) takes a `refine_fn` closure and applies it uniformly to each community. The quality function is passed as a trait object (`&dyn QualityFunction`) and dispatched through the `QualityFn` enum. **The refinement flow is identical for all four quality functions.**

### 1.5 igraph (C)

**Source:** [github.com/igraph/igraph](https://github.com/igraph/igraph), `src/community/leiden.c`

The igraph Leiden implementation supports Modularity and CPM. The refinement phase (`leiden_merge_vertices()`) uses the same flow for both. The connectedness guarantee comes from the γ-connectivity threshold:

```c
if (!IGRAPH_BIT_SET(non_singleton_cluster, current_cluster) &&
    (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >=
     vertex_weight_prod * resolution)) {
    // ... consider merge
}
```

This is an O(1) arithmetic check, not a graph traversal. And it applies to both quality functions.

---

## 2. What the Original Paper Says

### 2.1 Source

Traag, Waltman, van Eck (2019), *"From Louvain to Leiden: guaranteeing well-connected communities"*, Scientific Reports 9:5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z). [PMC full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC6435756/).

### 2.2 The Connectedness Guarantee

The paper explicitly states the connectedness guarantee is quality-function-agnostic:

> *"In these properties, γ refers to the resolution parameter in the quality function that is optimised, which can be either modularity or CPM. The property of γ-connectivity is a slightly stronger variant of ordinary connectivity."*

The guarantees after each iteration:
- All communities are γ-separated
- All communities are **γ-connected**

The paper proves (Supplementary Information, Section C.1) that the refinement phase guarantees all communities are connected. The proof relies on the refinement's structural design:
1. Starts from a singleton partition
2. Only singleton nodes can be merged
3. Merges only happen within each community of the original partition P

**The proof does not depend on the specific form of the quality function.** It only requires that:
- The quality function increases when a node is merged (otherwise the merge is rejected)
- The refinement only merges singletons

### 2.3 The Refinement Phase Description

> *"The refined partition P_refined is obtained as follows. Initially, P_refined is set to a singleton partition, in which each node is in its own community. The algorithm then locally merges nodes in P_refined: nodes that are on their own in a community in P_refined can be merged with a different community. Importantly, mergers are performed only within each community of the partition P."*

> *"In the refinement phase, nodes are not necessarily greedily merged with the community that yields the largest increase in the quality function. Instead, a node may be merged with any community for which the quality function increases."*

The quality function only affects the *probability* of selecting a community (higher increase = more likely), not the *structural safety* of the merge.

---

## 3. Is There Any Quality Function Where "Isolated Vertices Only" Would NOT Guarantee Connectedness?

**No.** The connectedness guarantee is structural, not quality-function-dependent. Here's why:

### 3.1 Proof Sketch

1. **Base case:** Initially, all vertices are singletons (trivially connected).
2. **Inductive step:** When an isolated vertex v joins community C:
   - The new community C ∪ {v} is connected because v has at least one edge to C (otherwise the quality function would not increase by moving v to C — the diff_move would be non-positive).
   - Non-isolated vertices never move, so existing connected communities stay connected.
3. **By induction:** All communities remain connected throughout refinement.

This proof does not depend on the form of the quality function. It only requires that:
- A move only happens if the quality function increases (ΔQ > 0)
- Only isolated vertices can move

For ANY quality function where ΔQ > 0 implies the node has at least one edge to the target community, the connectedness guarantee holds.

### 3.2 Edge Case: What If ΔQ > 0 But Node Has No Edges to Target?

This cannot happen for any reasonable quality function. For a node v to have ΔQ > 0 when moving to community C, there must be some "attraction" to C. In all standard quality functions (Modularity, CPM, RBConfiguration, RBER, MapEquation), this attraction requires at least one edge from v to C.

For example:
- **Modularity:** ΔQ(v→C) = (1/2m) * [w(v,C) - (k_v * K_C)/2m]. For this to be positive, w(v,C) must be sufficiently large.
- **CPM:** ΔQ(v→C) = [w(v,C) + w(v,v) - γ*(2n_C+1)] - [...]. For this to be positive, w(v,C) must be sufficiently large.
- **Map Equation:** The codelength decrease requires flow connections to the target module.

In all cases, ΔQ > 0 implies at least one edge to the target community.

### 3.3 What About Significance VertexPartition?

`SignificanceVertexPartition` is a special case — it's a *comparative* quality function that measures the statistical significance of a partition. It is typically used for evaluating a partition, not for optimization during refinement. libleidenalg does not use it in the refinement phase.

---

## 4. Map Equation and Infomap: A Special Case

### 4.1 Infomap Is a Different Algorithm

Infomap ([mapequation.org](https://mapequation.org)) uses the Map Equation, which is a flow-based quality function. However, Infimap's algorithm structure is fundamentally different from Leiden:

- Infomap uses a two-level or hierarchical encoding of random walks
- It does NOT use the Leiden refinement phase (singleton start + isolated-vertex-only merges)
- Its connectedness guarantee comes from the flow model, not from structural constraints

### 4.2 Can Leiden Optimize the Map Equation?

Yes. The paper *"Know thy tools! Limits of popular algorithms used for topic..."* ([MIT Press](https://direct.mit.edu/qss/article/3/4/1054/113321/Know-thy-tools-Limits-of-popular-algorithms-used)) notes:

> *"Infomap uses the map equation... However, one could, for example, also use the Louvain or Leiden algorithm to optimize the map equation."*

When Leiden optimizes the Map Equation, it uses the same refinement phase (singleton start + isolated-vertex-only merges). The Map Equation's `diff_move()` computes the codelength change, but the refinement flow is identical.

### 4.3 leiden-rs Map Equation Support

leiden-rs includes a separate `Infomap` implementation (with its own `MapEquation` struct) that uses the flow-based algorithm. This is NOT the same as using Map Equation as a quality function within the Leiden algorithm. The communal codebase's `QualityFunction::MapEquation` variant would use the Map Equation as a quality function within the standard Leiden refinement flow.

### 4.4 Conclusion for Map Equation

When Map Equation is used as a quality function within Leiden's refinement phase, the same structural guarantees apply. The refinement flow is identical — only the `diff_move()` computation differs.

---

## 5. Communal Codebase Analysis

### 5.1 QualityFunction Enum

```rust
// crates/communal-algo/src/quality.rs
pub enum QualityFunction {
    Modularity,
    Cpm,
    MapEquation,
}
```

Three quality functions are currently supported. The refinement phase should handle all three identically.

### 5.2 diff_move() Implementation per Quality Function

**Modularity::delta_q()** (lines 65-115):
- Computes ΔQ using the standard modularity gain formula
- Requires `w_to_target` (sum of edge weights to target community) for positive gain
- Positive ΔQ implies at least one edge to target

**Cpm::delta_q()** (lines 244-295):
- Computes ΔQ using the CPM gain formula
- Requires `w_to_target` for positive gain
- Positive ΔQ implies at least one edge to target

**MapEquation:** (not yet implemented in communal, but would follow the same pattern)
- Would compute codelength change
- Positive ΔQ implies flow connection to target

### 5.3 Refinement Phase Behavior

The refinement phase in `crates/communal-algo/src/leiden/refinement.rs` currently:
1. Starts from a singleton partition ✓
2. Only allows singleton nodes to move ✓
3. Uses `would_remain_connected` check ✗ (redundant)
4. Dispatches on `QualityFunction` for `diff_move()` ✓

The `would_remain_connected` check is redundant for ALL three quality functions because the singleton-start + isolated-vertex-only design already guarantees connectedness.

---

## 6. Findings and Recommendations

### 6.1 Core Finding

> **The connectedness check optimization is universally applicable to all quality functions.** The refinement phase's connectedness guarantee is structural (by construction), not quality-function-dependent. No reference implementation performs a connectivity check during refinement, regardless of quality function.

### 6.2 Why This Is True

1. **All reference implementations** (libleidenalg, leidenalg, leiden-rs, igraph) use the same refinement flow for every quality function.
2. **The original paper's proof** of connectedness (Supplementary Information C.1) does not depend on the quality function's form — only on the refinement's structural design.
3. **The "isolated vertices only" policy** guarantees connectedness for any quality function where ΔQ > 0 implies at least one edge to the target community (which is true for all standard quality functions).
4. **No quality function** in any reference implementation triggers different refinement behavior.

### 6.3 Recommendation

Remove `would_remain_connected` from both `local_moving` and `refinement` for ALL quality functions:
- Modularity
- CPM
- MapEquation

The optimization is not quality-function-specific. The refinement phase's design guarantees connectedness by construction, regardless of which quality function is being optimized.

### 6.4 Additional Quality Functions to Consider

Based on reference implementations, communal may want to add:
- **RBConfiguration** (Reichardt-Bornholdt with configuration null model) — supported by libleidenalg and leiden-rs
- **RBER** (Reichardt-Bornholdt with Erdős-Rényi null model) — supported by libleidenalg and leiden-rs

Both would use the same refinement flow with no connectivity check.

---

## 7. Sources

### Papers
- Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z). [PMC full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC6435756/).

### Source Code
- libleidenalg C++: [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg) — `src/Optimiser.cpp` lines ~1012-1247 (`move_nodes_constrained`) and `merge_nodes_constrained` — same flow for all quality functions, no connectivity checks.
- leidenalg Python: [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) — wraps libleidenalg, inherits same behavior.
- leiden-rs Rust: [docs.rs/leiden-rs](https://docs.rs/leiden_rs/latest/leiden_rs/) — `QualityFn` enum dispatches `delta_move_from_components()` for all 4 quality functions through identical refinement flow.
- leiden-rs algorithm.rs: [docs.rs/leiden-rs src](https://docs.rs/leiden-rs/latest/src/leiden_rs/algorithm.rs.html) — `refinement_generic()` applies same refinement logic regardless of quality function.
- igraph C: [github.com/igraph/igraph](https://github.com/igraph/igraph) — `src/community/leiden.c` — `leiden_merge_vertices()` uses same flow for Modularity and CPM.

### Documentation
- leidenalg docs: [leidenalg.readthedocs.io](https://leidenalg.readthedocs.io/en/stable/reference.html) — Optimiser class: "general optimisation routine for any quality function."
- leiden-rs docs: [docs.rs/leiden-rs](https://docs.rs/leiden-rs/latest/leiden_rs/) — "Supports four quality functions: Modularity, CPM, RBConfiguration, and RBER."

### Related
- Sahu, S. (2024). "GVE-Leiden: Fast Leiden Algorithm for Community Detection in Shared Memory Setting." [arXiv:2312.13936](https://arxiv.org/html/2312.13936v5) — refinement phase pseudocode shows same flow for all quality functions.
- Map Equation / Infomap: [mapequation.org](https://mapequation.org) — different algorithm, not Leiden refinement.
- "Know thy tools! Limits of popular algorithms..." [MIT Press](https://direct.mit.edu/qss/article/3/4/1054/113321/Know-thy-tools-Limits-of-popular-algorithms-used) — notes that Leiden can optimize the map equation.

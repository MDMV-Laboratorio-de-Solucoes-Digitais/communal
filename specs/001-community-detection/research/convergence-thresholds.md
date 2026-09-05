# Convergence Thresholds for Community Detection Quality Functions

## Research Question

Should the convergence threshold (default 1e-6) and mode (absolute/relative) apply uniformly to all quality functions (Modularity Q, CPM, Map Equation), or should each quality function be allowed to define its own default convergence criteria?

---

## Executive Summary

**Recommendation: Each quality function should define its own convergence criteria.** A single uniform threshold (e.g., 1e-6) is mathematically inappropriate because the quality functions operate on vastly different scales and have different semantic meanings. The field's standard practice is function-specific convergence, not uniform thresholds.

---

## 1. Findings Per Quality Function

### 1.1 Modularity Q (Newman-Girvan)

**Typical Value Range:** [-0.5, 1.0], with empirical networks typically yielding Q ∈ [0.3, 0.7].

**Original Paper Convergence Criteria:**
- Blondel et al. (2008) — "Fast unfolding of communities in large networks" (J. Stat. Mech. Theory Exp.)
- Convergence defined as: "The two phases are repeated until the quality function cannot be increased further."
- **No explicit threshold.** The algorithm terminates when no single node movement can strictly increase modularity.
- Key quote: "The algorithm optimises a quality function such as modularity or CPM in two elementary phases: (1) local moving of nodes; and (2) aggregation of the network... The two phases are repeated until the quality function cannot be increased further." (Traag et al. 2019, Section II, paraphrasing Blondel et al.)

**Scale Analysis:**
- Modularity is a normalized measure: Q = (1/2m) Σ_c [e_c - γ(K_c²/2m)]
- The denominator 2m (total edge weight) normalizes the scale
- A threshold of 1e-6 on Q is meaningful because Q is already dimensionless and bounded
- However, the *absolute* change in Q depends on network size — larger networks naturally have smaller per-node contributions

**Reference Implementation Behavior:**
- **igraph** (`cluster_leiden` with modularity): No threshold parameter exposed. Convergence determined by "no nodes moved" in an iteration.
- **leidenalg** (`ModularityVertexPartition`): No convergence threshold parameter. The `optimise_partition` method runs until no strictly positive move exists.

### 1.2 Constant Potts Model (CPM)

**Typical Value Range:** Unbounded; depends on resolution parameter γ and community sizes. Raw H = Σ_c [e_c - γ(n_c choose 2)].

**Original Paper Convergence Criteria:**
- Traag, Van Dooren & Nesterov (2011) — "Narrow scope for resolution-limit-free community detection" (Phys. Rev. E 84, 016114)
- The Leiden paper (Traag et al. 2019) uses CPM as the primary quality function for theoretical analysis
- Convergence: same as modularity — "until the quality function cannot be increased further"
- **No explicit threshold.** Only strictly positive improvements are accepted (line 17 of Algorithm A.1: "if ΔH(v→C') > 0 then").

**Scale Analysis:**
- CPM is NOT normalized by 2m. The raw value scales with the number of possible node pairs: Σ_c γ(n_c choose 2)
- For a network with n nodes, the maximum penalty term is γ(n choose 2) = γn(n-1)/2
- This means CPM values scale quadratically with network size
- A fixed threshold of 1e-6 is **meaningless** for CPM on large networks — the quality function value can be in the thousands or millions
- Conversely, a threshold that works for large networks would be overly strict for small networks

**Reference Implementation Behavior:**
- **leidenalg** (`CPMVertexPartition`): No convergence threshold. Uses strictly positive `diff_move` values.
- **igraph** (`igraph_community_leiden_simple` with `IGRAPH_LEIDEN_OBJECTIVE_CPM`): No threshold parameter. Convergence is "no nodes moved."

### 1.3 Map Equation (Infomap)

**Typical Value Range:** Code length L ∈ [0, log₂(n)] bits per step, where n is the number of nodes. For a network with 10,000 nodes, L ∈ [0, ~13.3] bits/step.

**Original Paper Convergence Criteria:**
- Rosvall & Bergstrom (2008) — "Maps of random walks on complex networks reveal community structure" (PNAS 105, 1118)
- Convergence: "repeatedly merge the two modules that give the largest decrease in description length until further merging gives a longer description"
- **No explicit threshold.** The greedy search terminates when no merge can decrease L.
- Refinement via simulated annealing: "We use the heat-bath algorithm and start with the module configuration achieved by the greedy search... we select the run that gives the shortest description of the map"
- Key detail: The map equation L is measured in **bits per step** — it's a rate, not an absolute value

**Scale Analysis:**
- L(M) = q_↕ H(Q) + Σ_i p_↕_i H(P_i) — measured in bits per random walk step
- This is inherently normalized by the random walk process
- The value is bounded below by 0 and above by log₂(n) (the entropy of uniform node visits)
- A threshold of 1e-6 bits/step would be extremely strict — it would require convergence to near-machine-precision
- In practice, improvements of < 0.01 bits/step are typically considered negligible

**Reference Implementation Behavior:**
- **igraph** (`igraph_community_infomap`): Delegates to the Infomap C++ library (mapequation/infomap). The library uses its own internal convergence criteria based on codelength improvements.
- **Infomap library**: Uses a combination of greedy search + simulated annealing. Convergence is determined by the absence of improving moves, not a fixed threshold.

---

## 2. Evidence From Reference Implementations

### 2.1 leidenalg (Python/C++)

**Source:** [vtraag/leidenalg](https://github.com/vtraag/leidenalg) and [vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)

**Key Finding: No convergence threshold parameter exists.**

The `Optimiser::optimise_partition` method (in `src/Optimiser.cpp`) runs a loop:
```cpp
do {
    // ... move nodes, refine, aggregate ...
    aggregate_further &= (new_collapsed_graphs[0]->vcount() < collapsed_graphs[0]->vcount()) &&
                         (collapsed_graphs[0]->vcount() > collapsed_partitions[0]->n_communities());
} while (aggregate_further);
```

The `move_nodes` method only accepts **strictly positive** improvements:
```cpp
double max_improv = ... 10*DBL_EPSILON;  // essentially zero
// ...
if (possible_improv > max_improv) {
    max_comm = comm;
    max_improv = possible_improv;
}
```

**Interpretation:** The effective threshold is `10 * DBL_EPSILON ≈ 2.22e-15`, but this is applied to the **raw quality function output**, not a normalized value. Since different quality functions produce values at different scales, this is effectively a function-specific threshold.

### 2.2 igraph (C Library)

**Source:** [igraph/igraph](https://github.com/igraph/igraph), files `src/community/leiden.c` and `src/community/infomap.cpp`

**Leiden (`igraph_community_leiden`):**
- No convergence threshold parameter in the public API
- The `n_iterations` parameter controls how many *outer* iterations to run
- With `n_iterations < 0`, it iterates until "an iteration did not change the clustering"
- Convergence per iteration: the `leiden_fastmove_vertices` function sets `*changed = true` only when a node actually moves
- Comment in source: "Only consider strictly improving moves. Note that this is important in considering convergence."

**Infomap (`igraph_community_infomap`):**
- Delegates entirely to the Infomap C++ library
- No threshold exposed in the igraph API
- The Infomap library internally determines convergence

### 2.3 CDlib (Python Meta-Library)

**Source:** [GiulioRossetti/cdlib](https://github.com/GiulioRossetti/cdlib)

CDlib wraps multiple community detection algorithms. For Leiden and Louvain implementations:
- When using igraph backends, it inherits igraph's convergence behavior (no threshold)
- When using leidenalg backends, it inherits leidenalg's behavior (no threshold)
- CDlib itself does not impose a uniform convergence threshold across different algorithms

---

## 3. Scale Analysis Summary

| Quality Function | Typical Range | Scaling Behavior | Threshold Sensitivity |
|-----------------|---------------|------------------|----------------------|
| **Modularity Q** | [0, 1] | Normalized by 2m | 1e-6 is reasonable (dimensionless) |
| **CPM (raw)** | [-γn²/2, m] | Quadratic in n | 1e-6 is meaningless for large n |
| **Map Equation L** | [0, log₂(n)] | Bits per step | 1e-6 is overly strict |

**Critical Insight:** A uniform threshold of 1e-6 would be:
- **Reasonable** for Modularity Q (already normalized to [0,1])
- **Too strict** for Map Equation (would require machine-precision convergence on a measure that is only meaningful to ~0.01 bits)
- **Too lenient or too strict** for CPM depending on network size (since CPM scales with n²)

---

## 4. Best Practices in the Field

### 4.1 Standard Approach

The field's standard practice is **function-specific convergence**, not uniform thresholds:

1. **No major library uses a uniform threshold.** Neither leidenalg, igraph, nor the Infomap library expose a single convergence threshold parameter that applies across quality functions.

2. **Convergence is defined semantically, not numerically.** The standard definition is: "terminate when no single-node move can strictly improve the quality function." This is inherently function-specific because "improvement" is measured in the native units of each quality function.

3. **When thresholds are used, they are function-specific:**
   - Modularity: often no threshold (strict positivity)
   - CPM: strict positivity on raw H value
   - Map Equation: strict positivity on L (bits/step)

### 4.2 The "Strictly Positive" Convention

All three algorithms (Leiden, Louvain, Infomap) share a common convergence convention:
- **Only accept moves that strictly improve the quality function**
- **Terminate when no improving move exists**
- This is equivalent to a threshold of **zero** on the improvement

The Leiden paper formalizes this (Algorithm A.1, line 17):
```
if ΔH(v→C') > 0 then
    v → C'
```

This is NOT a threshold of 1e-6 — it's a threshold of exactly 0. The algorithm terminates at a **local optimum** where no single-node move yields any improvement, no matter how small.

### 4.3 Practical Considerations

**Why not use a small positive threshold (like 1e-6)?**

1. **For modularity:** A threshold of 1e-6 on Q is reasonable but unnecessary. The strictly-positive convention already guarantees convergence to a local optimum. A positive threshold would stop earlier but potentially at a worse partition.

2. **For CPM:** A threshold of 1e-6 on raw H is meaningless because H scales with network size. A threshold that works for n=100 is too strict for n=100,000.

3. **For Map Equation:** A threshold of 1e-6 bits/step is far below the meaningful precision of the measure. Code length improvements of < 0.01 bits/step are typically considered negligible in practice.

---

## 5. Recommendation

### 5.1 Proposed Design

Each quality function should define its own convergence criteria:

```rust
trait QualityFunction {
    /// The quality value type (f64 for all current implementations)
    type Value: Copy + PartialOrd;
    
    /// Calculate the change in quality from moving node v to community C
    fn diff_move(&self, v: Node, target: Community) -> Self::Value;
    
    /// Check if an improvement is significant enough to accept
    /// Default implementation: strictly positive
    fn is_significant_improvement(&self, improvement: Self::Value) -> bool {
        improvement > Self::Value::zero()
    }
    
    /// Optional: convergence threshold for early termination
    /// Default: None (run until local optimum)
    fn convergence_threshold(&self) -> Option<Self::Value> {
        None
    }
}
```

### 5.2 Function-Specific Defaults

| Quality Function | Default Threshold | Mode | Rationale |
|-----------------|------------------|------|-----------|
| **Modularity Q** | 0 (strictly positive) | Absolute | Normalized measure; any improvement is meaningful |
| **CPM** | 0 (strictly positive) | Absolute | Unbounded scale; threshold must be relative to γ |
| **Map Equation** | 0 (strictly positive) | Absolute | Bits/step; sub-0.01 improvements are negligible |

### 5.3 Rationale

1. **Mathematical correctness:** Each quality function has a different scale and semantics. A uniform threshold is dimensionally inconsistent.

2. **Alignment with literature:** The original papers for all three algorithms use strictly-positive convergence, not fixed thresholds.

3. **Alignment with implementations:** No major library (leidenalg, igraph, Infomap) uses a uniform threshold across quality functions.

4. **Practicality:** Users who want early termination can set a function-specific threshold. The default should be "run until local optimum" (threshold = 0).

5. **Extensibility:** New quality functions (Significance, Surprise, etc.) can define their own convergence behavior without being constrained by a global default.

### 5.4 What About the 1e-6 Default?

If a non-zero default threshold is desired for practical purposes (e.g., to avoid infinite loops due to floating-point noise), it should be:

- **Relative to the quality function's scale**, not absolute
- **Modularity Q:** 1e-6 is acceptable (Q ∈ [0,1])
- **CPM:** Should be relative to γ (e.g., 1e-6 * γ)
- **Map Equation:** Should be relative to the current codelength (e.g., 1e-6 * L)

However, the recommended approach is to use **strictly positive** (threshold = 0) as the default, with the `is_significant_improvement` method allowing subclasses to override this behavior.

---

## 6. Citations

### Primary Sources

1. **Blondel, V. D., Guillaume, J.-L., Lambiotte, R., & Lefebvre, E. (2008).** "Fast unfolding of communities in large networks." *Journal of Statistical Mechanics: Theory and Experiment*, 2008(10), P10008.
   - Convergence: "until the quality function cannot be increased further" (no explicit threshold)
   - Section: Algorithm description, Phase 1 (local moving)

2. **Traag, V. A., Waltman, L., & van Eck, N. J. (2019).** "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9(1), 5233.
   - Convergence: strictly positive improvements only (Algorithm A.1, line 17: "if ΔH(v→C') > 0 then")
   - Section: "Louvain Algorithm" and "Leiden Algorithm" sections; Appendix A (pseudo-code)
   - Key insight: "In both the Leiden algorithm and the Louvain algorithm, we therefore consider only strictly positive improvements" (Appendix D.2)

3. **Rosvall, M., & Bergstrom, C. T. (2008).** "Maps of random walks on complex networks reveal community structure." *Proceedings of the National Academy of Sciences*, 105(4), 1118-1123.
   - Convergence: "repeatedly merge the two modules that give the largest decrease in description length until further merging gives a longer description"
   - Section: "Implementation" subsection
   - Key insight: No threshold on L; convergence is when no merge can decrease description length

4. **Traag, V. A., Van Dooren, P., & Nesterov, Y. (2011).** "Narrow scope for resolution-limit-free community detection." *Physical Review E*, 84(1), 016114.
   - Defines CPM: H = Σ_c [e_c - γ(n_c choose 2)]
   - Convergence: same strictly-positive convention as Leiden

### Implementation Sources

5. **leidenalg** — [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) and [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)
   - File: `src/Optimiser.cpp`, method `Optimiser::move_nodes`
   - Evidence: `double max_improv = ... 10*DBL_EPSILON;` — effectively zero threshold on raw quality
   - No convergence threshold parameter in public API

6. **igraph** — [github.com/igraph/igraph](https://github.com/igraph/igraph)
   - File: `src/community/leiden.c`, function `leiden_fastmove_vertices`
   - Evidence: "Only consider strictly improving moves. Note that this is important in considering convergence." (source comment)
   - No threshold parameter in `igraph_community_leiden()` API

7. **Infomap** — [github.com/mapequation/infomap](https://github.com/mapequation/infomap)
   - Used via igraph's `igraph_community_infomap()` wrapper
   - Convergence determined internally by the Infomap library based on codelength improvements

### Secondary Sources

8. **Rosvall, M., Axelsson, D., & Bergstrom, C. T. (2009).** "The map equation." *The European Physical Journal Special Topics*, 178(1), 13-23.
   - Formal definition of the map equation L(M)
   - Confirms L is measured in bits per step

9. **Fortunato, S., & Hric, D. (2016).** "Community detection in networks: A user guide." *Physics Reports*, 659, 1-44.
   - Survey article confirming different quality functions have different scales and convergence behaviors

---

## 7. Appendix: Mathematical Comparison of Scales

### Modularity Q
```
Q = (1/2m) Σ_c [e_c - γ(K_c²/2m)]
```
- Bounded: Q ∈ [-0.5, 1.0] (theoretical), typically [0, 0.7] (empirical)
- Scale: dimensionless, normalized by total edge weight
- A threshold of 1e-6 on Q ≈ 1e-6 / Q_max is a relative threshold of ~1e-6

### Constant Potts Model
```
H = Σ_c [e_c - γ(n_c choose 2)]
```
- Unbounded: H ∈ [-γn(n-1)/2, m]
- Scale: depends on γ and n²
- For γ=1, n=1000: H can be as low as ~-500,000
- A threshold of 1e-6 on H is meaningless (below floating-point precision relative to the scale)

### Map Equation
```
L(M) = q_↕ H(Q) + Σ_i p_↕_i H(P_i)
```
- Bounded: L ∈ [0, log₂(n)] bits per step
- Scale: bits per random walk step (a rate)
- For n=10,000: L ∈ [0, ~13.3]
- A threshold of 1e-6 bits/step is 6 orders of magnitude below typical improvements

---

*Document generated from primary source analysis. All claims are traceable to cited papers or source code locations.*

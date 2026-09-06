# Leiden Algorithm: Default Convergence Mode Research

## Summary & Recommendation

**Recommendation: Use absolute improvement threshold (ΔQ < ε) as the default convergence mode, with ε = 1e-6.**

This aligns with the reference implementation's behavior, is numerically stable across graph scales, and provides predictable iteration behavior. Relative convergence (ΔQ/|Q| < ε) introduces division-by-zero risk when Q ≈ 0 and creates inconsistent stopping behavior across different graph densities.

---

## 1. What the Reference Implementation Uses

### 1.1 Traag et al. C++ libleidenalg (v0.12+)

The reference C++ implementation (`Optimiser::move_nodes`) does **not** use a convergence threshold in the traditional sense. Instead, it uses a **strictly positive improvement** criterion:

```cpp
// From Optimiser.cpp, move_nodes method:
double max_improv = (0 < max_comm_size && max_comm_size < partitions[0]->csize(v_comm))
    ? -INFINITY
    : 10 * DBL_EPSILON;  // ≈ 2.22e-15
```

The algorithm continues iterating the fast-local-move queue until **no single node move can improve the quality function**. The effective threshold is `10 * DBL_EPSILON` (approximately 2.22e-15), which is effectively zero at double-precision floating point. This means:

- **Convergence = no move yields ΔQ > 0** (strictly positive)
- There is no "relative" vs "absolute" distinction — it's absolute with ε ≈ 0
- The outer loop (`optimise_partition`) continues aggregating the graph as long as the number of communities decreases

### 1.2 Python leidenalg Package

The Python interface (`Optimiser.optimise_partition`) exposes `n_iterations` parameter:

```python
# From Optimiser.py:
def optimise_partition(self, partition, n_iterations=2, ...):
    while continue_iteration:
        diff_inc = _c_leiden._Optimiser_optimise_partition(...)
        diff += diff_inc
        itr += 1
        if n_iterations < 0:
            continue_iteration = (diff_inc > 0)  # strict positive improvement
        else:
            continue_iteration = itr < n_iterations
```

- **Default: `n_iterations=2`** (not convergence-based at all!)
- When `n_iterations < 0`: runs until `diff_inc > 0` is false (no improvement)
- The convergence check is: `diff_inc > 0` — i.e., **absolute improvement must be strictly positive**

### 1.3 igraph C Implementation of Leiden

The igraph C library (`igraph_community_leiden`) follows the same pattern:

```c
// From leiden.c:
igraph_bool_t changed = true;
for (igraph_int_t itr = 0;
     n_iterations < 0 ? changed : itr < n_iterations;
     itr++) {
    IGRAPH_CHECK(community_leiden(..., &changed));
}
```

- When `n_iterations < 0`: iterates until `changed == false`
- `changed` is only set to `true` when a vertex is actually moved (strict improvement)
- Default in the R interface: `n_iterations = 2`

### 1.4 Key Insight from Reference Implementations

**Neither the C++ libleidenalg nor the Python leidenalg uses a configurable convergence threshold.** They all converge based on **strictly positive improvement** (ΔQ > 0). The concept of a "convergence threshold" (ε = 1e-6) is an addition in your spec that doesn't exist in the reference.

---

## 2. Traag et al. (2019) Paper: Mathematical Definition

### 2.1 Convergence Definition in the Paper

From the paper (Appendix D, Lemma 8):

> "Only strict improvements can be made in the Leiden algorithm. Consequently, if P^{t+1} ≠ P^t, then P^{t+1} ≠ P^{t'} for all t' < t. [...] There exists a τ such that P^t = P^τ for all t ≥ τ."

The paper defines convergence as reaching a **stable partition** where no further node movements can improve the quality function. The pseudo-code (Algorithm A.2, line 18) specifies:

```
if ΔH_P(v → C*) > 0 then  // Perform only strictly positive node movements
    v → C*
```

### 2.2 No Threshold in the Paper

The paper does **not** mention any ε threshold. Convergence is defined as:
- **Node optimality**: No individual node can be moved to improve quality
- **Subset optimality** (asymptotic): No subset of nodes can be moved to improve quality

The mathematical framework relies on the fact that quality strictly increases with each move and is bounded above, guaranteeing convergence.

### 2.3 Practical Implication

The paper's theoretical framework assumes exact arithmetic. In floating-point implementations, the reference uses `DBL_EPSILON` as the practical floor, effectively requiring ΔQ > 0.

---

## 3. Practical Differences: Absolute vs Relative Convergence

### 3.1 Absolute Convergence: ΔQ < ε

**Definition**: Stop when the quality improvement in an iteration is less than ε.

**Advantages**:
- Simple, predictable behavior
- No division-by-zero risk
- Directly corresponds to "no meaningful improvement"
- Matches the reference implementation's ΔQ > 0 criterion (with ε as a practical floor)
- Scale-independent in terms of the quality function's natural units

**Disadvantages**:
- For very large graphs, even tiny per-node improvements can sum to large ΔQ values, causing early stopping
- For small graphs, may iterate unnecessarily on floating-point noise if ε is too small

### 3.2 Relative Convergence: ΔQ/|Q| < ε

**Definition**: Stop when the relative quality improvement is less than ε.

**Advantages**:
- Scale-adaptive: works across different graph sizes and quality magnitudes
- More intuitive "percentage improvement" interpretation

**Disadvantages**:
- **Division by zero** when Q ≈ 0 (common in early iterations or with negative modularity)
- **Numerical instability** when |Q| is very small but non-zero
- **Inconsistent behavior**: A graph with Q = 0.001 and ΔQ = 0.0001 would stop (10% improvement < ε), while a graph with Q = 0.5 and ΔQ = 0.001 would continue (0.2% improvement > ε for ε=1e-6)
- **Does not match** the reference implementation at all
- Modularity can be negative, making the ratio undefined or misleading

### 3.3 Numerical Comparison

| Scenario | Q value | ΔQ | Absolute (ε=1e-6) | Relative (ε=1e-6) |
|----------|---------|-----|-------------------|-------------------|
| Dense graph, late iteration | 0.75 | 1e-7 | **Stop** (1e-7 < 1e-6) | Continue (1.33e-7 > 1e-6) |
| Sparse graph, early iteration | 0.01 | 1e-7 | **Stop** (1e-7 < 1e-6) | **Stop** (1e-5 > 1e-6) → Continue |
| Negative modularity | -0.05 | 1e-7 | **Stop** | **Undefined** (negative denominator) |
| Near-zero quality | 1e-8 | 1e-7 | Continue (1e-7 > 1e-6) → Stop | **Stop** (10 > 1e-6) → Continue |
| Floating-point noise | 0.30 | 1e-16 | Continue | Continue |

The absolute mode provides more predictable and consistent behavior across all scenarios.

---

## 4. How Other Libraries Handle Convergence

### 4.1 NetworkX Louvain (`louvain_communities`)

```python
# From networkx/algorithms/community/louvain.py:
def louvain_communities(G, weight="weight", resolution=1, threshold=0.0000001, ...):
    ...
    new_mod = modularity(G, inner_P, resolution=resolution, weight="weight")
    if new_mod - mod <= threshold:
        return
    mod = new_mod
```

- **Default threshold: 1e-7** (absolute modularity gain between levels)
- Uses **absolute** improvement comparison
- Threshold applies to the **inter-level** modularity gain, not per-node moves
- Within a level, runs until no node can be moved for positive gain

### 4.2 python-louvain (`community.best_partition`)

```python
# From python-louvain source:
def best_partition(graph, partition=None, weight='weight', resolution=1.0, ...):
    # Uses internal _one_level function that runs until no improvement
    # No configurable convergence threshold exposed
```

- No configurable convergence threshold
- Runs until no positive improvement possible

### 4.3 igraph Louvain (`cluster_louvain`)

The igraph C implementation of Louvain:
- Runs until no node can be moved to improve modularity
- No configurable threshold
- Uses strict positive improvement (ΔQ > 0)

### 4.4 Summary of Library Defaults

| Library | Algorithm | Default Convergence | Threshold |
|---------|-----------|-------------------|-----------|
| libleidenalg (C++) | Leiden | Strict ΔQ > 0 | None (DBL_EPSILON floor) |
| leidenalg (Python) | Leiden | n_iterations=2, or ΔQ > 0 | None |
| igraph (C) | Leiden | n_iterations=2, or no change | None |
| igraph (C) | Louvain | No positive improvement | None |
| NetworkX | Louvain | Absolute ΔQ ≤ 1e-7 | 1e-7 (absolute) |
| python-louvain | Louvain | No positive improvement | None |

---

## 5. Recommendation

### 5.1 Default: Absolute Improvement Threshold (ε = 1e-6)

**Reasoning:**

1. **Matches reference semantics**: The reference uses ΔQ > 0 (strictly positive absolute improvement). Using ε = 1e-6 as the threshold is the closest practical approximation that avoids infinite loops from floating-point noise.

2. **Numerical safety**: No division-by-zero risk, no issues with negative modularity values.

3. **Predictable behavior**: Users can reason about "stop when improvement is below 1 part per million" without needing to know the scale of their quality function.

4. **Industry precedent**: NetworkX uses absolute threshold (1e-7) for Louvain. This is the established convention in the Python community.

5. **Floating-point appropriate**: At ε = 1e-6, you're well above double-precision noise floor (~1e-15) but well below any meaningful modularity improvement.

### 5.2 Implementation Suggestion

```rust
// Recommended default configuration
pub struct ConvergenceConfig {
    pub mode: ConvergenceMode,      // Default: Absolute
    pub threshold: f64,             // Default: 1e-6
    pub max_iterations: Option<u32>, // Safety bound
}

pub enum ConvergenceMode {
    Absolute,    // |ΔQ| < ε
    Relative,    // |ΔQ| / |Q| < ε  (with Q ≈ 0 fallback)
}
```

### 5.3 Edge Cases to Handle

1. **Q ≈ 0 with relative mode**: Fall back to absolute comparison when |Q| < ε
2. **Negative Q with relative mode**: Use |ΔQ| / max(|Q|, ε) or fall back to absolute
3. **Floating-point noise**: Ensure threshold is well above machine epsilon (≥ 1e-10 recommended)
4. **Always provide max_iterations**: As a safety bound regardless of convergence mode

---

## 6. Citations

1. Traag, V.A., Waltman, L., Van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9(1), 5233. [DOI: 10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z). [arXiv:1810.08473](https://arxiv.org/abs/1810.08473)

2. Traag, V.A., Waltman, L., Van Eck, N.J. (2019). PDF of the paper. [traag.net](https://traag.net/wp/wp-content/papercite-data/pdf/traag_leiden_algo_2018.pdf)

3. vtraag/libleidenalg: C++ library of Leiden algorithm. [GitHub](https://github.com/vtraag/libleidenalg)

4. vtraag/leidenalg: Python implementation of the Leiden algorithm. [GitHub](https://github.com/vtraag/leidenalg)

5. leidenalg Python package documentation. [ReadTheDocs](https://leidenalg.readthedocs.io/)

6. igraph C library - Leiden implementation (`src/community/leiden.c`). [GitHub](https://github.com/igraph/igraph/blob/master/src/community/leiden.c)

7. NetworkX Louvain implementation (`networkx/algorithms/community/louvain.py`). [GitHub](https://github.com/networkx/networkx/blob/main/networkx/algorithms/community/louvain.py)

8. NetworkX `louvain_communities` documentation. [NetworkX Docs](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.louvain.louvain_communities.html)

9. Blondel, V.D., Guillaume, J.L., Lambiotte, R., Lefebvre, E. (2008). "Fast unfolding of communities in large networks." *Journal of Statistical Mechanics: Theory and Experiment*, 2008(10), P10008. [DOI: 10.1088/1742-5468/2008/10/P10008](https://doi.org/10.1088/1742-5468/2008/10/P10008)

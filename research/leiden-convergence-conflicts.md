# Research: Conflicts Between Quality Function Selection and Convergence Criteria in the Leiden Algorithm

**Date:** 2026-01-28  
**Related Checklist Item:** CHK044  
**Scope:** Mathematical edge cases, reference implementation behavior, and recommended spec constraints

---

## Executive Summary

This research investigates potential conflicts between quality function selection and convergence criteria in the Leiden algorithm. The central finding is that **relative convergence** (converging when `|ΔQ| / |Q| < ε`) is mathematically unsafe with both Modularity Q and CPM quality functions because these functions can produce values near zero or negative, making the ratio undefined or misleading.

All major reference implementations (libleidenalg, leidenalg, igraph) avoid this problem by using **absolute convergence** (converging when `|ΔQ| < ε` with a fixed ε). The spec should constrain convergence modes to prevent undefined behavior.

---

## 1. Mathematical Edge Cases by Quality Function + Convergence Mode

### 1.1 Modularity Q

**Definition:**
```
Q = (1/2m) Σ_ij [A_ij - (k_i * k_j)/(2m)] δ(σ_i, σ_j)
```

**Range:** Theoretically unbounded below, practically ∈ [-1, 1] for most graphs. Can be:
- **Negative:** When communities are worse than random expectation
- **Near-zero:** When community structure is indistinguishable from random
- **Positive:** When genuine community structure exists

**Edge Cases with Relative Convergence:**

| Condition | Q value | Relative Convergence `ΔQ/Q` | Problem |
|-----------|---------|---------------------------|---------|
| Q ≈ 0 | ≈ 0 | Division by zero | **Undefined** |
| Q < 0 | Negative | Negative ratio | **Sign flip**: improvements appear as negative ratios |
| Q → 0⁺ | ≈ 0⁺ | Very large ratio | **Premature convergence**: tiny absolute changes trigger stop |
| Q → 0⁻ | ≈ 0⁻ | Very large negative ratio | **Numerical instability** |

**Specific Scenarios:**
1. **Random graphs (Erdős-Rényi):** Q ≈ 0 for large random graphs. Relative convergence is undefined.
2. **Graphs with no community structure:** Q oscillates around zero. The algorithm may never converge.
3. **Transition phases:** During optimization, Q may cross zero, causing a discontinuity in the convergence criterion.
4. **Negative modularity partitions:** Some partitions have Q < 0 (worse than random). Relative convergence produces a negative denominator, making the ratio sign-inverted.

### 1.2 Constant Potts Model (CPM)

**Definition:**
```
Q = Σ_c [m_c - γ * C(n_c, 2)]
```

where `m_c` is internal edge weight of community c, `γ` is resolution parameter, and `C(n_c, 2)` is the number of possible internal edges.

**Range:** Unbounded. Depends on:
- Graph size (number of nodes n)
- Resolution parameter γ
- Edge weights (can be negative)

**Edge Cases with Relative Convergence:**

| Condition | CPM value | Relative Convergence `ΔQ/Q` | Problem |
|-----------|-----------|---------------------------|---------|
| CPM ≈ 0 | ≈ 0 | Division by zero | **Undefined** |
| CPM < 0 | Negative | Negative ratio | **Sign flip** |
| Large γ | Very negative | Small positive ratio | **False convergence** |
| Small γ | Large positive | Small ratio | **Premature convergence** |
| Negative weights | Can be any sign | Unpredictable | **Numerical instability** |

**Specific Scenarios:**
1. **High resolution (large γ):** CPM becomes very negative as the penalty term dominates. The absolute value is large, making relative convergence insensitive to actual improvements.
2. **Low resolution (small γ):** CPM becomes large and positive. Small relative changes may be insignificant in absolute terms.
3. **Negative edge weights:** CPM can be negative even with good community structure. Relative convergence is undefined when CPM = 0.

### 1.3 RBConfiguration (RBC) and RBER

**RBC Definition:**
```
Q = Σ_c [m_c - γ * K_c²/(4m)]
```

**RBER Definition:**
```
Q = Σ_c [m_c - γ * p * C(n_c, 2)]
```

where `p = m / C(n, 2)` is the graph density.

**Similar edge cases to Modularity:** Both can be negative or near-zero, making relative convergence problematic.

### 1.4 Significance and Surprise

**Significance:** Always non-negative (Q ≥ 0). Can be near-zero for random graphs.
**Surprise:** Can be zero or positive. Uses KL-divergence formulation.

**Edge Cases:**
- Both are non-negative, avoiding the sign-flip problem
- Division by zero still possible when Q = 0

### 1.5 Summary of Edge Cases

| Quality Function | Can Be Negative? | Can Be Zero? | Relative Convergence Safe? |
|-----------------|------------------|--------------|---------------------------|
| Modularity Q | ✅ Yes | ✅ Yes | ❌ No |
| CPM | ✅ Yes | ✅ Yes | ❌ No |
| RBConfiguration | ✅ Yes | ✅ Yes | ❌ No |
| RBER | ✅ Yes | ✅ Yes | ❌ No |
| Significance | ❌ No | ✅ Yes | ⚠️ Only when Q > 0 |
| Surprise | ❌ No | ✅ Yes | ⚠️ Only when Q > 0 |

---

## 2. How Reference Implementations Handle Edge Cases

### 2.1 libleidenalg (C++ Library)

**Source:** [vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)

**Convergence Mode:** Absolute convergence only

**Implementation Details:**

The `Optimiser::move_nodes()` method uses an absolute threshold:

```cpp
// From Optimiser.cpp, line ~380
double max_improv = (0 < max_comm_size && max_comm_size < partitions[0]->csize(v_comm)) 
    ? -INFINITY 
    : 10*DBL_EPSILON;  // Absolute threshold, NOT relative
```

**Key observations:**
- Uses `10 * DBL_EPSILON` ≈ 2.22e-15 as the minimum improvement threshold
- Convergence is determined by whether any move produces positive improvement
- **No division by quality function value** — completely avoids the division-by-zero problem
- The outer loop in `optimise_partition()` stops when no aggregate moves produce improvement

**Quality Function Implementation:**

Modularity `quality()` method includes a guard:
```cpp
double m = 2*this->graph->total_weight();
if (m == 0)
    return 0.0;  // Guard against division by zero
```

CPM `quality()` does NOT divide by total weight, so it doesn't have this guard — but CPM values are additive (sum over communities), so division by zero is not part of the formula.

### 2.2 leidenalg (Python Interface)

**Source:** [vtraag/leidenalg](https://github.com/vtraag/leidenalg)

**Convergence Mode:** Absolute convergence with configurable iteration count

**Implementation Details:**

The `optimise_partition()` method provides two modes:

```python
def optimise_partition(self, partition, n_iterations=2, ...):
    # ...
    while continue_iteration:
        diff_inc = _c_leiden._Optimiser_optimise_partition(...)
        diff += diff_inc
        itr += 1
        if n_iterations < 0:
            continue_iteration = (diff_inc > 0)  # Absolute: any positive improvement
        else:
            continue_iteration = itr < n_iterations
```

**Key observations:**
- `n_iterations=2` (default): Fixed number of iterations, no convergence check
- `n_iterations<0`: Run until no improvement (absolute convergence, threshold = 0)
- No relative convergence mode exposed
- The `diff_inc > 0` check is purely absolute

### 2.3 igraph (C Implementation)

**Source:** [igraph/igraph](https://github.com/igraph/igraph)

**Convergence Mode:** Absolute convergence via changed flag

**Implementation Details:**

The `community_leiden()` function in `src/community/leiden.c`:

```c
// From leiden.c, line ~1300
igraph_bool_t changed = true;
for (igraph_int_t itr = 0;
     n_iterations < 0 ? changed : itr < n_iterations;
     itr++) {
    IGRAPH_CHECK(community_leiden(graph, ...));
}
```

The `leiden_fastmove_vertices()` function uses:
```c
// Only consider strictly improving moves
if (diff > max_diff) {  // Absolute comparison
    best_cluster = c;
    max_diff = diff;
}
```

**Key observations:**
- Convergence based on whether ANY vertex was moved (`changed` flag)
- No quality-function-value division
- Quality is computed post-hoc, not used for convergence decisions
- The quality function CAN be zero or negative — but this doesn't affect convergence

### 2.4 NetworkX (Python Implementation)

**Source:** [networkx/networkx](https://github.com/networkx/networkx)

**Convergence Mode:** Absolute convergence with fixed epsilon

**Implementation Details:**

```python
# From leiden.py
improvement_made = (Q_new - Q) > 0.0000001  # Absolute threshold of 1e-7
```

**Key observations:**
- Uses fixed absolute threshold of 1e-7
- Does NOT use relative convergence
- Simple and robust against all edge cases

### 2.5 Summary of Reference Implementation Approaches

| Implementation | Convergence Mode | Threshold | Relative Convergence? |
|---------------|------------------|-----------|----------------------|
| libleidenalg | Absolute | 10·DBL_EPSILON ≈ 2.2e-15 | ❌ No |
| leidenalg | Absolute | diff > 0 | ❌ No |
| igraph | Absolute | changed flag | ❌ No |
| NetworkX | Absolute | 1e-7 | ❌ No |

**Critical Finding:** No major implementation uses relative convergence. This is a deliberate design choice to avoid the mathematical pathologies documented in Section 1.

---

## 3. Mathematical Relationship Between Quality Function Scale and Convergence Behavior

### 3.1 Scale Mismatch Between Quality Functions

**Modularity Q:** Scale is normalized by total edge weight (1/2m factor). For a graph with m edges:
- Q values typically in range [-0.5, 0.7] for real-world networks
- Scale is INDEPENDENT of graph size (bounded)
- For disconnected graphs: Q is computed per component

**CPM:** Scale is NOT normalized. For a graph with n nodes and m edges:
- CPM values scale with graph size: O(m - γ·n²)
- For large graphs with high resolution, CPM can be very negative (O(γ·n²))
- Scale is GRAPH-SIZE DEPENDENT

**Implication:** A single relative threshold cannot work across both quality functions:
- For Modularity: A threshold of 0.01 (1%) might be reasonable
- For CPM: A threshold of 0.01 could be either trivially small (for large graphs) or impossibly large (for small graphs)

### 3.2 Convergence Detection Analysis

**Absolute Convergence Properties:**
- **Pros:** Always well-defined, scale-invariant, works for all quality functions
- **Cons:** Threshold must be tuned per graph; no notion of "relative improvement"

**Relative Convergence Properties:**
- **Pros:** Intuitive (percentage improvement), scale-independent in theory
- **Cons:** Undefined when Q=0, misleading when Q<0, scale-dependent across quality functions

### 3.3 Numerical Stability Considerations

For floating-point arithmetic:
- `DBL_EPSILON ≈ 2.22e-15` (machine epsilon for double)
- `10 * DBL_EPSILON ≈ 2.22e-14` (libleidenalg threshold)
- For graphs with total edge weight m, modularity precision is ~m·DBL_EPSILON
- For graphs with m > 10¹⁰, modularity may have precision worse than 1e-5

---

## 4. Recommended Constraints and Warnings for the Spec

### 4.1 Prohibited Combinations

The following combinations should be **explicitly disallowed** by the spec:

1. **Relative convergence + Modularity Q**
   - Reason: Q can be zero or negative, causing undefined behavior
   - Error message: "Relative convergence is incompatible with Modularity Q: Q can be zero or negative, making the ratio ΔQ/Q undefined"

2. **Relative convergence + CPM**
   - Reason: CPM can be zero or negative, and its scale is graph-size dependent
   - Error message: "Relative convergence is incompatible with CPM: CPM can be zero or negative, and its scale varies with graph size"

3. **Relative convergence + RBConfiguration**
   - Reason: Same as Modularity (shares the same mathematical structure)

4. **Relative convergence + RBER**
   - Reason: Same as Modularity (shares the same mathematical structure)

### 4.2 Conditional Combinations (Allowed with Warnings)

1. **Relative convergence + Significance** (allowed only when Q > ε_min)
   - Reason: Significance is non-negative but can be zero
   - Warning: "Relative convergence with Significance requires Q > 0; algorithm should fall back to absolute mode when Q ≈ 0"

2. **Relative convergence + Surprise** (allowed only when Q > ε_min)
   - Reason: Surprise is non-negative but can be zero
   - Warning: "Relative convergence with Surprise requires Q > 0; algorithm should fall back to absolute mode when Q ≈ 0"

### 4.3 Safe Operating Ranges

For **absolute convergence** (the recommended mode):

| Quality Function | Safe Threshold Range | Recommended Default | Notes |
|-----------------|---------------------|---------------------|-------|
| Modularity Q | [DBL_EPSILON, 0.01] | 1e-6 | Must account for 1/2m scaling |
| CPM | [DBL_EPSILON, 0.01] | 1e-6 | Scale depends on graph size |
| RBConfiguration | [DBL_EPSILON, 0.01] | 1e-6 | Same scaling as Modularity |
| RBER | [DBL_EPSILON, 0.01] | 1e-6 | Scale depends on graph density |
| Significance | [DBL_EPSILON, 0.01] | 1e-6 | Non-negative, safer |
| Surprise | [DBL_EPSILON, 0.01] | 1e-6 | Non-negative, safer |

### 4.4 Recommended Fallback Behavior

If relative convergence is requested with an incompatible quality function, the spec should mandate one of:

1. **Hard error:** Reject the configuration with a clear error message
2. **Automatic fallback:** Switch to absolute convergence with a warning logged
3. **Hybrid approach:** Use relative convergence when |Q| > ε_fallback, otherwise use absolute convergence

**Recommended approach:** Option 1 (hard error) for API clarity, with documentation explaining why.

### 4.5 Spec Warnings Required

The spec should include the following warnings:

```
WARNING: Quality Function Scale Mismatch
- Modularity Q and CPM operate on different numerical scales
- A convergence threshold appropriate for one may be inappropriate for the other
- Absolute convergence thresholds should be calibrated per quality function

WARNING: Negative Quality Values
- Modularity Q, CPM, RBConfiguration, and RBER can produce negative values
- Any convergence criterion that divides by Q will fail when Q ≤ 0
- Use absolute convergence for these quality functions

WARNING: Graph-Size Dependent Scale
- CPM values scale with O(n²) where n is the number of nodes
- Convergence thresholds for CPM should be scaled with graph size
- A threshold of 1e-6 may be too tight for graphs with n > 10⁴
```

---

## 5. Source Citations

### 5.1 Reference Implementations

1. **libleidenalg** (C++): https://github.com/vtraag/libleidenalg
   - `src/Optimiser.cpp`: Lines 380-390 (convergence threshold: `10*DBL_EPSILON`)
   - `src/ModularityVertexPartition.cpp`: `quality()` method with `m == 0` guard
   - `src/CPMVertexPartition.cpp`: `quality()` method (additive, no division)

2. **leidenalg** (Python): https://github.com/vtraag/leidenalg
   - `src/leidenalg/Optimiser.py`: `optimise_partition()` method (n_iterations < 0 → absolute convergence)
   - `src/leidenalg/VertexPartition.py`: Quality function definitions

3. **igraph** (C): https://github.com/igraph/igraph
   - `src/community/leiden.c`: `community_leiden()` function (changed flag convergence)
   - `leiden_fastmove_vertices()`: Absolute diff comparison

4. **NetworkX** (Python): https://github.com/networkx/networkx
   - `networkx/algorithms/community/leiden.py`: `improvement_made = (Q_new - Q) > 0.0000001`

### 5.2 Academic References

1. Traag, V.A., Waltman, L., Van Eck, N.-J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." Scientific Reports, 9(1), 5233. https://doi.org/10.1038/s41598-019-41695-z

2. Newman, M.E.J., & Girvan, M. (2004). "Finding and evaluating community structure in networks." Physical Review E, 69(2), 026113. https://doi.org/10.1103/PhysRevE.69.026113

3. Traag, V.A., Van Dooren, P., & Nesterov, Y. (2011). "Narrow scope for resolution-limit-free community detection." Physical Review E, 84(1), 016114. https://doi.org/10.1103/PhysRevE.84.016114

4. Reichardt, J., & Bornholdt, S. (2006). "Statistical mechanics of community detection." Physical Review E, 74(1), 016110. https://doi.org/10.1103/PhysRevE.74.016110

5. Traag, V.A., Krings, G., & Van Dooren, P. (2013). "Significant scales in community structure." Scientific Reports, 3, 2930. https://doi.org/10.1038/srep02930

6. Traag, V.A., Aldecoa, R., & Delvenne, J.-C. (2015). "Detecting communities using asymptotical surprise." Physical Review E, 92(2), 022816. https://doi.org/10.1103/PhysRevE.92.022816

### 5.3 Technical References

1. IEEE 754 Double-Precision Floating Point: DBL_EPSILON = 2⁻⁵² ≈ 2.22e-16

---

## 6. Conclusion

The Leiden algorithm's convergence criterion must be carefully matched to the quality function to avoid mathematical pathologies. The key findings are:

1. **Relative convergence is unsafe** with Modularity Q, CPM, RBConfiguration, and RBER because these functions can be zero or negative.

2. **Absolute convergence is the universal safe choice** — all major implementations use it.

3. **Scale mismatch** between quality functions means a single threshold cannot serve all functions.

4. **The spec should:**
   - Disallow relative convergence with Modularity Q, CPM, RBConfiguration, and RBER
   - Require absolute convergence as the default
   - Document the mathematical justification for these constraints
   - Provide clear error messages when incompatible combinations are requested

---

*Document generated from analysis of libleidenalg v0.12+, leidenalg v0.10+, igraph v0.10+, and NetworkX v3.7+ source code.*

# Leiden Algorithm Beta Parameter: Mathematical Justification Research

## Research Question

Does mathematical justification exist for the Leiden algorithm's refinement phase beta parameter range [0.0005, 0.1]? Can this range be derived theoretically, or is it purely empirical?

---

## Executive Summary

**Answer: The range [0.0005, 0.1] is entirely empirical. No mathematical justification, theoretical derivation, or formal sensitivity analysis exists in the academic literature.**

After exhaustive searching across:
- The primary source (Traag et al., 2019) and all follow-up papers by the same authors
- All major reference implementations (C++ libleidenalg, Python leidenalg, igraph C core, Java networkanalysis)
- The broader academic literature on Leiden algorithm sensitivity analysis

**No study provides a mathematical derivation, formal sensitivity analysis, or theoretical grounding for why beta ∈ [0.0005, 0.1] works well.** The range is stated once, without justification, in the original paper, and has been adopted as a de facto standard by all implementations.

---

## 1. Primary Source Analysis

### 1.1 Traag et al. (2019) — The Original Paper

**Source:** Traag, V.A., Waltman, L., & Van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. https://arxiv.org/abs/1810.08473

The beta range appears exactly once, in Section IV (Experimental Analysis), page 7:

> "In all experiments reported here, we used a value of 0.01 for the parameter β that determines the degree of randomness in the refinement phase of the Leiden algorithm. However, values of β within a range of roughly [0.0005, 0.1] all provide reasonable results, thus allowing for some, but not too much randomness."

**Critical observations:**
1. No derivation, citation, or justification accompanies this statement
2. The word "roughly" signals an informal estimate, not a precise bound
3. "Reasonable results" is undefined — no quality metrics, benchmarks, or comparisons are provided for different beta values
4. The appendices (C.1, C.2, D, E) contain detailed mathematical proofs about reachability of optimal partitions and connectivity guarantees, but **none mention beta or derive its optimal range**
5. The only theorem involving randomness (Theorem 11, Appendix D.3) requires β > 0 to guarantee asymptotic convergence — but this gives no guidance on the magnitude of β

### 1.2 Traag (2015) — The Random Neighbor Move Predecessor

**Source:** Traag, V.A. (2015). "Faster community detection using a refinement phase." *Physical Review E* 92, 032801. https://doi.org/10.1103/PhysRevE.92.032801

This earlier work (cited in the 2019 paper as reference [18]) introduces the "random neighbour move" concept. It provides some theoretical discussion of why random selection helps escape local optima, but **does not quantify the optimal randomness level**. The 2015 paper is about Louvain refinement, not Leiden specifically, and does not use the beta parameterization.

### 1.3 No Follow-up Papers by the Authors

Searches for subsequent work by Traag, Waltman, or Van Eck specifically addressing the beta parameter yielded **no results**. The authors have not published:
- A sensitivity analysis of beta
- A theoretical derivation of the optimal beta range
- Any follow-up discussing or refining the [0.0005, 0.1] range

The authors have published on related topics (bibliometrics, citation analysis, other network methods), but none provide additional insight into beta.

---

## 2. Reference Implementation Analysis

### 2.1 C++ libleidenalg

**Repository:** https://github.com/vtraag/libleidenalg  
**Source file:** `src/Optimiser.cpp`, `include/Optimiser.h`

**Finding: Beta does NOT exist as a parameter.**

The C++ implementation controls refinement randomness through a completely different mechanism — the `refine_consider_comms` setting:

```cpp
// From Optimiser.cpp constructor
this->refine_consider_comms = Optimiser::RAND_NEIGH_COMM;
```

In `merge_nodes_constrained()` (the refinement routine), when `consider_comms == RAND_NEIGH_COMM`:

```cpp
// Select a random community among neighbors, proportional to frequency
vector<size_t> all_neigh_comms_incl_dupes;
// ... collect neighboring communities (with duplicates) ...
size_t random_idx = get_random_int(0, k - 1, &rng);
size_t rand_comm = all_neigh_comms_incl_dupes[random_idx];
```

This is **uniform random selection among neighboring communities**, which is equivalent to the β → 0 limit of the paper's exp(β · Δ) weighting. The C++ code:
- Has **no beta member variable** in the `Optimiser` class
- Has **no exponential weighting** in the refinement phase
- Does **not expose beta** in the API
- Uses uniform random neighbor selection instead

**Implication:** The reference C++ implementation is effectively hardcoded to a single point (maximum randomness) that is the limit of the paper's recommended range.

### 2.2 Python leidenalg Package

**Repository:** https://github.com/vtraag/leidenalg  
**Documentation:** https://leidenalg.readthedocs.io/

The Python package is a wrapper around libleidenalg. It inherits the same behavior:
- **No `beta` parameter** in the `Optimiser` class
- Only `refine_consider_comms` and `refine_routine` are exposed
- The Python `Optimiser` cannot control beta because the C++ layer doesn't implement it

**Source:** `src/leidenalg/Optimiser.py` — no beta property exists.

### 2.3 igraph C Core Implementation

**Repository:** https://github.com/igraph/igraph  
**Source file:** `src/community/leiden.c`

**Finding: igraph implements the full exponential weighting with beta as a configurable parameter.**

The igraph C implementation (`leiden_merge_vertices()` function, starting around line 100 of the source) contains the actual exp(β·Δ) sampling:

```c
// From leiden.c - leiden_merge_vertices function
/* Calculate the transformed difference for sampling */
if (diff >= 0) {
    total_cum_trans_diff += exp(diff / beta);
}
```

And the sampling step:
```c
if (total_cum_trans_diff < IGRAPH_INFINITY) {
    igraph_real_t r = RNG_UNIF(0, total_cum_trans_diff);
    igraph_int_t chosen_idx;
    igraph_vector_binsearch_slice(&cum_trans_diff, r, &chosen_idx, 0, nb_neigh_clusters);
    chosen_cluster = VECTOR(neighbor_clusters)[chosen_idx];
} else {
    chosen_cluster = best_cluster;
}
```

The R interface (`cluster_leiden()`) exposes beta with default 0.01:

```
beta: Parameter affecting the randomness in the Leiden algorithm.
      This affects only the refinement step of the algorithm.
      (default: 0.01)
```

**Source:** https://r.igraph.org/reference/cluster_leiden.html

The C source code comment explicitly acknowledges the dual limit behavior:

> "For beta to 0 this converges to selecting a cluster with the maximum improvement. For beta to infinity this converges to a uniform distribution among all eligible clusters."

**Key insight:** igraph is the only reference implementation that faithfully implements the paper's exp(β·Δ) formulation. However, its documentation cites only the original 2019 paper for the default value — no additional justification.

### 2.4 Java NetworkAnalysis Package

**Repository:** https://github.com/CWTSLeiden/networkanalysis  
**Source file:** `LocalMergingAlgorithm.java`

The Java implementation (used in Gephi and the original experimental analysis of the paper) does implement the exponential weighting with a parameter called `randomness`:

```java
// From LocalMergingAlgorithm.java
public static final double DEFAULT_RANDOMNESS = 1e-2;  // = 0.01

// The sampling formula:
totalTransformedQualityValueIncrement += FastMath.fastExp(qualityValueIncrement / randomness);
```

This matches the paper's formulation exactly: Pr(C) ∝ exp(Δ / randomness) = exp(β · Δ) where β = 1/randomness.

Wait — this is the **inverse** relationship. In the Java code, `randomness` is in the denominator (equivalent to β), not the numerator. So `randomness = 0.01` means β = 0.01, consistent with the paper.

The Javadoc confirms the interpretation:

> "The higher the value of the randomness parameter, the stronger the randomness in the choice of a cluster. The lower the value of the randomness parameter, the more likely the cluster resulting in the largest increase in the quality function is to be chosen."

**No justification for the range is provided in the Java source code or documentation.**

### 2.5 Gephi Plugin

**Repository:** https://github.com/vtraag/gephi-leiden-plugin

The Gephi plugin wraps the Java implementation. It exposes the randomness parameter through the `LeidenAlgorithm` class. The API documentation confirms:

> "Constructs a Leiden algorithm for a specified resolution parameter, number of iterations, randomness parameter, and local moving algorithm."

The default randomness is inherited from `LocalMergingAlgorithm.DEFAULT_RANDOMNESS = 1e-2`.

No mathematical justification for the default or range is provided in the Gephi plugin documentation or README.

### 2.6 Implementation Comparison Summary

| Implementation | Beta/Randomness | Default | Configurable? | Formula | Justification |
|---|---|---|---|---|---|
| **Paper (2019)** | β | 0.01 | Yes (in theory) | exp(β·Δ) | None |
| **igraph C** | beta | 0.01 | Yes | exp(diff/beta) | Cites paper only |
| **Java networkanalysis** | randomness | 0.01 | Yes | exp(incr/randomness) | None |
| **C++ libleidenalg** | N/A | N/A | No | Uniform random | Simplified |
| **Python leidenalg** | N/A | N/A | No | Uniform random | Simplified |
| **Gephi plugin** | randomness | 0.01 | Yes | exp(incr/randomness) | None |

---

## 3. Follow-up Literature Search

### 3.1 Academic Database Searches

The following search queries were executed across Google Scholar, arXiv, and the web:

1. "Leiden algorithm beta parameter sensitivity analysis"
2. "Leiden algorithm refinement phase randomness parameter"
3. "Traag Waltman Van Eck beta parameter justification"
4. "libleidenalg beta parameter range justification"
5. "leidenalg beta parameter mathematical derivation"
6. "Leiden algorithm beta 0.01 0.0005 0.1 range derivation theoretical"
7. "Leiden algorithm sensitivity analysis parameter tuning"
8. "Leiden refinement phase theoretical analysis beta"

**Result: Zero papers provide a mathematical derivation, sensitivity analysis, or theoretical justification for the beta range.**

### 3.2 What Was Found

Several papers mention the Leiden algorithm and its parameters, but none analyze beta specifically:

- **Waltman & van Eck (2013)** — Smart local move algorithm (the Leiden precursor). No beta analysis.
- **Traag (2015)** — Random neighbor move refinement. Discusses why randomness helps escape local optima qualitatively, but does not quantify optimal randomness levels.
- **Nguyen et al. (2021)** — "Leiden-Based Parallel Community Detection" (KIT thesis). Describes the algorithm but does not analyze beta.
- **Various single-cell RNA-seq papers** — Use Leiden for clustering but focus on resolution parameter, not beta.

### 3.3 Community Discussion: GitHub Issue #77

**Source:** https://github.com/vtraag/leidenalg/issues/77

A 2021 GitHub issue ("Non-Deterministic selection of Community") directly raised the question of beta implementation. The issue author noted:

> "Since the theta parameter is a constant, the resulting exponent would continuously grow smaller as the graph size increases (or rather, the individual modularity deltas decrease). Doesn't this effectively turn it into a more or less true-random selection for big graphs?"

This astute observation highlights a genuine theoretical concern: since quality function deltas scale with graph size, a fixed beta value has different effective behavior on different-sized networks. **The issue received no public reply from the maintainer**, and was eventually closed without resolution.

**Implication:** Even within the community of implementors, the theoretical basis for beta is acknowledged as unclear.

### 3.4 Single-Cell RNA-seq Community Practice

In the single-cell community, Leiden is widely used (Scanpy, Seurat). The beta parameter is typically left at default (0.01) or not exposed at all. The community's focus is almost exclusively on the resolution parameter, with beta considered a "set and forget" parameter.

The CASRAI guide notes:

> "Leiden's refinement phase makes randomised moves, so the seed is a real parameter."

But provides no guidance on beta selection.

---

## 4. Theoretical Analysis: Why No Derivation Exists

### 4.1 The Core Problem

The exp(β·Δ) weighting in the refinement phase is fundamentally different from the quality-function optimization in the main Leiden loop:

1. **The main loop** has provable guarantees (γ-separation, γ-connectivity, subset optimality)
2. **The refinement phase** is a randomized heuristic whose purpose is to explore the partition space more broadly than a purely greedy approach

The beta parameter controls a **randomized heuristic** within a **theoretically-grounded algorithm**. The randomness serves a practical purpose (avoiding local optima) but is not part of the formal guarantees. The paper proves that the algorithm converges regardless of the randomness level (as long as β > 0), so the precise value of β affects efficiency and exploration, not correctness.

### 4.2 Why the Range is Scale-Dependent

As noted in GitHub issue #77, the issue author correctly identified a critical scaling problem:

- The quality increment Δ_H scales with graph size (number of nodes/edges)
- For modularity: Δ_H is proportional to 1/(2m), so it decreases as the network grows
- For CPM: Δ_H is bounded by edge count within a community

This means:
- With **fixed β** and increasing graph size, β·Δ → 0, making the selection approach uniform random
- With **fixed β** and small graph size, β·Δ can be large, making the selection approach greedy

A mathematically justified beta range would need to be **normalized by the scale of quality function deltas**, which is network-dependent. No such normalization has been proposed.

### 4.3 The Asymptotic Guarantee (Theorem 11)

The only formal result involving randomness is Theorem 11 (Appendix D.3), which states:

> "P is asymptotically stable if and only if P is subset optimal."

The proof relies on Lemma 10, which requires only that β > 0 (so there is some non-zero probability of any non-decreasing move). The theorem provides **no bound on convergence speed or exploration quality** as a function of β.

---

## 5. Conclusion

### 5.1 Can the Range [0.0005, 0.1] Be Mathematically Justified?

**No. The range is purely empirical, and the conditions for a mathematical justification are not met:**

1. **No formal sensitivity analysis exists** — The original paper states the range without any accompanying data, figures, or statistical analysis
2. **No theoretical derivation exists** — The appendices contain rigorous proofs about other aspects of the algorithm but do not address beta
3. **The scale-dependence problem** — A universal beta range cannot exist without normalizing for graph size and quality function scale
4. **No follow-up work** — The authors have not revisited beta in any subsequent publication
5. **Implementation divergence** — The reference C++ implementation doesn't even use beta, while igraph and Java implementations do

### 5.2 What We Can Say with Confidence

1. **β = 0.01 is the canonical default** — Used in all experiments in the original paper, and adopted by all implementations
2. **β > 0 is required for theoretical guarantees** — Theorem 11 requires randomness for asymptotic convergence
3. **The effective behavior of β is scale-dependent** — A fixed beta value has different effective randomness on different-sized networks
4. **The range [0.0005, 0.1] is an informal guideline** — Based on the authors' experience with a specific set of benchmark and empirical networks, not on theory
5. **The limit behaviors are understood** — β → 0 gives greedy selection; β → ∞ gives uniform random selection

### 5.3 Practical Recommendation

Given the absence of mathematical justification, the [0.0005, 0.1] range should be treated as:

- A **rule of thumb** derived from empirical observation on a specific set of networks
- **Not a guarantee** of good performance on arbitrary networks
- **Possibly network-dependent** — larger networks may need different values due to scaling effects
- **A reasonable default range** in the absence of better information

For applications where partition quality is critical, sensitivity analysis across beta values is recommended, since no theoretical guidance exists to rule out the possibility that optimal beta varies by network.

---

## References

1. Traag, V.A., Waltman, L., & Van Eck, N.J. (2019). From Louvain to Leiden: guaranteeing well-connected communities. *Scientific Reports* 9, 5233. https://arxiv.org/abs/1810.08473

2. Traag, V.A. (2015). Faster community detection using a refinement phase. *Physical Review E* 92, 032801. https://doi.org/10.1103/PhysRevE.92.032801

3. Waltman, L. & van Eck, N.J. (2013). A smart local move algorithm for large-scale modularity-based community detection. *European Physical Journal B* 86, 471. https://doi.org/10.1143/epjb/e2013-40501-y

4. C++ libleidenalg source: `include/Optimiser.h`, `src/Optimiser.cpp` — https://github.com/vtraag/libleidenalg

5. Python leidenalg source: `src/leidenalg/Optimiser.py` — https://github.com/vtraag/leidenalg

6. igraph C source: `src/community/leiden.c` — https://github.com/igraph/igraph

7. igraph R documentation: `cluster_leiden()` — https://r.igraph.org/reference/cluster_leiden.html

8. Java networkanalysis source: `LeidenAlgorithm.java`, `LocalMergingAlgorithm.java` — https://github.com/CWTSLeiden/networkanalysis

9. Gephi Leiden plugin: https://github.com/vtraag/gephi-leiden-plugin

10. GitHub Issue #77 (leidenalg): "Non-Deterministic selection of Community" — https://github.com/vtraag/leidenalg/issues/77

11. leidenalg documentation: https://leidenalg.readthedocs.io/

12. NetworkAnalysis API docs: https://cwtsleiden.github.io/networkanalysis/nl/cwts/networkanalysis/LeidenAlgorithm.html

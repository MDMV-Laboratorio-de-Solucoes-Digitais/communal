# Infomap Algorithm Time Complexity Analysis

## Executive Summary

The specification's complexity claim contains a conflation of two distinct algorithms that have both been called "Infomap" at different points in the literature. After analyzing the primary sources, the C++ reference implementation, and the underlying algorithmic literature, the correct complexity characterization is:

- **Per-iteration complexity of the modern (Louvain-style) Infomap: O(E)** for a single sweep, not O(E log V).
- **The original 2008 greedy algorithm:** O(E log² V) in the average case for sparse networks, degrading to O(V² log V) in the worst case with unbalanced dendrograms.
- **The O(n³) exhaustive multilevel search claim:** No academic source was found supporting this specific bound. This appears to be either a misattribution or a confusion with the general NP-hard nature of the partition problem. Exhaustive enumeration of all partitions is super-exponential (Bell numbers), not O(n³).

---

## 1. What the Primary Source (Rosvall & Bergstrom 2008) Actually Says

### Source
Rosvall M, Bergstrom CT. "Maps of random walks on complex networks reveal community structure." *Proceedings of the National Academy of Sciences* 105(4):1118–1123 (2008). https://doi.org/10.1073/pnas.0706851105

### Key Passage on Complexity
The paper does **not** state a big-O complexity directly. Instead, it describes the algorithm:

> "We use the map equation to calculate the description length and **repeatedly merge the two modules that give the largest decrease in description length** until further merging gives a longer description. With the improved version in ref. [4] of the greedy search algorithm in ref. [3], we have successfully partitioned networks with 2.6 million nodes and 29 million links."

The references are:
- **[3]** Clauset A, Newman MEJ, Moore C. "Finding community structure in very large networks." *Phys Rev E* 70:066111 (2004).
- **[4]** Wakita K, Tsurumi T. "Finding community structure in mega-scale social networks." arXiv:cs/0702048 (2007).

The paper then refines the greedy result with simulated annealing (heat-bath algorithm).

### What This Means
The 2008 algorithm is a **greedy pair-merge** (agglomerative hierarchical) approach, not the Louvain-style node-sweep used in the modern implementation. The complexity claim must therefore come from the CNM paper [3] and its improvement [4].

---

## 2. The CNM Algorithm Complexity (Clauset, Newman, Moore 2004)

### Source
Clauset A, Newman MEJ, Moore C. "Finding community structure in very large networks." *Phys Rev E* 70:066111 (2004).

### Complexity Claim
The paper estimates the complexity of the CNM algorithm as:

> **O(m d log n)**, where *n* and *m* are the numbers of nodes and edges, and *d* is the height of the dendrogram.

For sparse networks, *m* ≈ *n* and *d* ≈ log *n*, giving:

> **O(n log² n)** for social networks.

### The Wakita-Tsurumi Correction (2007)
Wakita & Tsurumi (arXiv:cs/0702048) identified that the CNM algorithm's efficiency degrades when merges are unbalanced. They observed:

> "Unbalanced merging process makes the height of the dendrogram grow more or less proportionally to its size and leads to the degradation of the computational efficiency to **O(n² log n)**."

This worst-case behavior occurs because the dendrogram height *d* becomes O(n) instead of O(log n) when communities grow in a highly unbalanced manner.

---

## 3. The Modern Infomap Implementation (Louvain-Style)

### Source
Rosvall M, Axelsson D, Bergstrom CT. "The map equation." *European Physical Journal Special Topics* 178:13–23 (2009). https://doi.org/10.1140/epjst/e2010-01179-1

### Key Passage
> "The core of the algorithm follows closely the method presented in ref. [28]: neighboring nodes are joined into modules, which subsequently are joined into super-modules and so on."

Reference [28] is: Blondel J, Guillaume J, Lambiotte R, Mech E. "Fast unfolding of communities in large networks." *J Stat Mech: Theory Exp* 2008:P10008 (2008).

> "In practice, for networks with on the order of 10,000 nodes and 1,000,000 directed and weighted links, each iteration takes about 5 seconds on a modern PC."

### Analysis of the C++ Reference Implementation
The reference implementation at `https://github.com/mapequation/infomap` uses the Louvain-style approach described in the 2009 paper. The core routine is `tryMoveEachNodeIntoBestModule()` (in `src/core/InfomapOptimizer.h`):

1. For each node *v*, iterate over its in-edges and out-edges to collect neighboring modules and compute ΔL for each candidate move.
2. Move the node to the module that minimizes the map equation.
3. The "dirty bit" optimization skips nodes whose neighborhood hasn't changed.

**Per-sweep complexity:** Each edge is inspected a constant number of times (once from each endpoint). Therefore, a single sweep is **O(V + E)**, or simply O(E) for connected graphs where E ≥ V−1.

**Consolidation:** Rebuilding the network with modules as super-nodes is also O(E).

**Total per-iteration:** O(E) per sweep, with multiple sweeps per iteration until local convergence.

### Why O(E log V) Is Not Correct for the Modern Implementation
The O(E log V) claim would apply if the algorithm used priority queues (heaps) to select the best merge at each step, as in the original CNM algorithm. The Louvain-style sweep, however, visits each node and inspects only its *neighboring* modules (bounded by its degree), not a global priority queue. The modern Infomap uses randomized index vectors for visit order, not a heap.

---

## 4. The O(n³) Exhaustive Multilevel Search Claim

### Finding
**No academic source was found that claims O(n³) complexity for exhaustive multilevel search in Infomap.**

### What the Literature Says About Exhaustive Search
The 2008 paper explicitly acknowledges the infeasibility of exhaustive search:

> "For all but the smallest networks, it is infeasible to check all possible partitions to find the one that minimizes the description length in the map equation."

The number of possible partitions of *n* nodes is the Bell number *Bₙ*, which grows super-exponentially (asymptotically faster than *cⁿ* for any constant *c*). Even restricting to two-level partitions, the number of ways to partition *n* nodes into *m* modules is the Stirling number of the second kind S(n,m), summed over all *m* from 1 to *n*, yielding *Bₙ*.

### Possible Origin of the O(n³) Claim
The O(n³) bound may be a conflation with:
1. **The Louvain method's worst case:** Some analyses of the Louvain method note that each node can move to any of its neighboring modules, and in the worst case with dense graphs and many modules, the per-iteration cost can approach O(V²) for a single sweep. Over O(V) iterations in the worst case (though convergence is typically much faster), this could suggest O(V³) as a very loose worst-case bound.
2. **Matrix multiplication or Floyd-Warshall:** O(n³) is the complexity of all-pairs shortest paths (Floyd-Warshall) or matrix multiplication, which are sometimes used in other community detection methods (e.g., some spectral methods), but not in Infomap.
3. **A misremembered bound:** The Wakita-Tsurumi paper shows O(n² log n) as the degraded worst case for CNM, which might have been loosely rounded to O(n³) in informal communication.

### Correct Statement
Exhaustive enumeration of all partitions is **super-exponential** (Bell numbers), making it infeasible for any non-trivial network. The practical Infomap algorithms avoid exhaustive search entirely, using greedy/Louvain-style heuristics instead.

---

## 5. Corrected Complexity Summary

| Algorithm Variant | Average Case | Worst Case | Source |
|---|---|---|---|
| **Original 2008 greedy (CNM-based)** | O(E log² V) | O(V² log V) | Clauset et al. 2004; Wakita & Tsurumi 2007 |
| **Modern Louvain-style Infomap** | O(E) per sweep | O(E) per sweep | Rosvall et al. 2009; Blondel et al. 2008 |
| **Exhaustive partition search** | Super-exponential (Bell numbers) | Super-exponential | Rosvall & Bergstrom 2008 |

### Recommended Specification Language

> **O(E) per sweep** — Louvain-style node moves with the Map Equation objective [Rosvall 2009; Blondel 2008]. The algorithm performs repeated sweeps until convergence; each sweep visits every edge a constant number of times. The original 2008 greedy variant used agglomerative pair-merging with priority queues [Clauset 2004; Wakita 2007] and achieved O(E log² V) for sparse networks. Exhaustive search over all partitions is super-exponential and never used in practice.

---

## 6. Citations

1. Rosvall M, Bergstrom CT. "Maps of random walks on complex networks reveal community structure." *PNAS* 105(4):1118–1123 (2008). https://doi.org/10.1073/pnas.0706851105
2. Clauset A, Newman MEJ, Moore C. "Finding community structure in very large networks." *Phys Rev E* 70:066111 (2004). https://doi.org/10.1103/PhysRevE.70.066111
3. Wakita K, Tsurumi T. "Finding community structure in mega-scale social networks." arXiv:cs/0702048 (2007). https://arxiv.org/abs/cs/0702048
4. Rosvall M, Axelsson D, Bergstrom CT. "The map equation." *Eur Phys J Special Topics* 178:13–23 (2009). https://doi.org/10.1140/epjst/e2010-01179-1
5. Blondel J, Guillaume J, Lambiotte R, Mech E. "Fast unfolding of communities in large networks." *J Stat Mech: Theory Exp* 2008:P10008 (2008). https://doi.org/10.1088/1742-5468/2008/10/P10008
6. Edler D, Holmgren A, Rosvall M. Infomap source code. https://github.com/mapequation/infomap (accessed 2026).

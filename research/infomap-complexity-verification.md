# Infomap Per-Iteration Complexity Verification

## Summary of Findings

**FR-028 ("O(E) per sweep") is correct** for the modern Louvain-style Infomap implementation. **T071 ("O(V + E·log V) per-iteration") is incorrect** as a characterization of the core node-move sweep. The latter appears to conflate the original 2008 greedy-merge variant with the modern node-moving approach, or to borrow complexity from unrelated algorithms.

---

## 1. Primary Source: Rosvall & Bergstrom (2008)

**Paper:** M. Rosvall and C. T. Bergstrom, "Maps of random walks on complex networks reveal community structure," *PNAS* 105(4), 1118–1123 (2008).
**DOI:** https://doi.org/10.1073/pnas.0706851105
**arXiv:** https://arxiv.org/abs/0707.0609

### What the Original Algorithm Did

The paper describes a **greedy merge** algorithm (not node-moving):

> "We first calculate the ergodic node visit frequencies and then assign every node to a unique module and derive the exit probabilities as described above. We use the map equation to calculate the description length and **repeatedly merge the two modules that give the largest decrease in description length** until further merging gives a longer description." (Supporting Material, Implementation section)

The paper explicitly cites the Clauset-Newman-Moore (CNM) fast greedy algorithm and its improved variant:

> "With the improved version in ref. (4) of the greedy search algorithm in ref. (3), we have successfully partitioned networks with 2.6 million nodes and 29 million links."

Where ref. (3) is Clauset et al. (2004) and ref. (4) is Wakita & Tsurumi (2007) — both are **modularity optimization** algorithms using **priority queues (heaps)**.

### Complexity of the Original Approach

The original Infomap used a CNM-style greedy merge with a heap. The CNM algorithm has worst-case complexity **O(|E|·|V|·log|V|)** due to heap operations. However, the paper does **not** explicitly state a complexity bound for Infomap itself. It only notes that the improved greedy approach scales to millions of nodes and links.

**Critical point:** The 2008 paper does NOT claim O(E) per sweep, nor does it claim O(V + E·log V). The O(V + E·log V) figure does not appear anywhere in the original paper.

### The 2009 Map Equation Paper

The follow-up paper (Rosvall, Axelsson & Bergstrom, *Eur. Phys. J. Special Topics* 178, 13 (2009)) describes the map equation framework more formally but also does not specify per-iteration complexity for the search algorithm.

---

## 2. Reference Implementation: mapequation/infomap (C++)

**Repository:** https://github.com/mapequation/infomap
**Key file:** `src/core/InfomapOptimizer.h` — `tryMoveEachNodeIntoBestModule()`

### How the Modern Algorithm Works

The modern Infomap uses a **Louvain-style node-moving** strategy (not greedy merge). The core sweep function iterates over all active nodes in random order:

```cpp
template <typename Objective>
INFOMAP_HOT unsigned int InfomapOptimizer<Objective>::tryMoveEachNodeIntoBestModule()
{
  auto& network = m_infomap->activeNetwork();
  // ... get random enumeration of nodes ...
  unsigned int numNodes = networkEnumeration.size();

  for (unsigned int i = 0; i < numNodes; ++i) {
    InfoNode& current = *network[nodeEnumeration[i]];

    if (!current.dirty)
      continue;

    deltaFlow.startRound();

    // For all outlinks
    for (auto& e : current.outEdges()) {
      deltaFlow.add(neighbour->index, DeltaFlowDataType(neighbour->index, edge.data.flow, 0.0));
    }
    // For all inlinks
    for (auto& e : current.inEdges()) {
      deltaFlow.add(neighbour->index, DeltaFlowDataType(neighbour->index, 0.0, edge.data.flow));
    }

    // ... add self, empty module, teleportation ...

    // Find the move that minimizes the description length
    for (unsigned int k = 0; k < numModuleLinks; ++k) {
      double deltaCodelength = m_objective.getDeltaCodelengthOnMovingNodeHoisted(...);
      // track best move
    }

    // Make best possible move
    if (bestDeltaModule.module != current.index) {
      m_objective.updateCodelengthOnMovingNode(...);
      // mark neighbors dirty
    }
  }
  return numMoved;
}
```

### Complexity Analysis of the Modern Implementation

**Per-node work:**

1. **Collect neighbor modules:** Iterate over all outlinks + inlinks → O(degree(node))
2. **Evaluate candidate modules:** The `deltaFlow` VectorMap contains at most O(degree(node)) entries (one per neighbor module, plus self and possibly an empty module). For each candidate, `getDeltaCodelengthOnMovingNodeHoisted()` performs O(1) arithmetic (a fixed-size `plogp_batch` of 7 values and some additions).
3. **Commit best move:** O(1) for updating module flow data and codelength terms.

**Total per sweep:** O(Σ degree(node)) = **O(E)**

### The VectorMap Data Structure

The `VectorMap<DeltaFlowDataType>` is a custom flat hash table with O(1) amortized `add` and lookup, using a redirect array indexed by module ID. It does NOT use a tree or heap. This is critical — there is no log V factor in candidate module lookup.

### Supporting Evidence from Comments

The code comment near the parallel threshold confirms the linear scaling expectation:

```cpp
// Below this active network size the fixed costs of a parallel move sweep
// (team fork/join, proposal buffers, serial commit pass) outweigh the work.
constexpr unsigned int minNetworkSizeForInnerParallelization = 10000;
```

This confirms the sweep is treated as work-proportional to network size (nodes + edges), not superlinear.

---

## 3. igraph C Implementation

**File:** `src/community/infomap.cpp` (in igraph)
**Documentation:** https://igraph.org/c/html/latest/igraph-Community.html#igraph_community_infomap

The igraph documentation for `igraph_community_infomap` states:

> **Time complexity: TODO.**

The igraph C code is a thin wrapper around the mapequation/infomap library (since igraph 1.0). It delegates all computation to the same C++ core described above. igraph does not provide its own complexity analysis for Infomap.

---

## 4. Other Reference Points

### python-leidenalg

The python-leidenalg library does not implement Infomap — it implements the Leiden algorithm (which optimizes modularity or the Constant Potts Model, not the map equation). Leiden's per-iteration complexity is O(E) for the local moving phase, analogous to Louvain.

The mapequation/infomap Python package is a binding to the same C++ core described above.

### Elixir Reference (yog/community/infomap)

The Hex docs state:

> **Complexity Time: O(V + E) per iteration**, typically converges quickly Space: O(V + E)

This aligns with O(E) per iteration for connected graphs (since E ≥ V−1 for connected graphs).

---

## 5. Analysis: O(E) vs O(V + E·log V)

### Why O(E) Is Correct for Modern Infomap

| Operation | Cost per node | Cost per sweep |
|-----------|---------------|----------------|
| Scan outlinks | O(out-degree) | O(E) |
| Scan inlinks | O(in-degree) | O(E) |
| Build candidate module set | O(degree) amortized (VectorMap) | O(E) |
| Evaluate candidates | O(degree) × O(1) per candidate | O(E) |
| Commit moves | O(1) per moved node | O(V) worst case |
| Mark neighbors dirty | O(degree) | O(E) |
| **Total** | | **O(E)** |

The key insight: each node only examines its **neighboring modules** (bounded by its degree), not a global priority queue. The delta codelength evaluation is O(1) per candidate (fixed-size arithmetic on pre-computed flow data).

### Why O(V + E·log V) Is Incorrect

The O(V + E·log V) complexity does not match any standard Infomap variant:

1. **The original 2008 greedy-merge** used a CNM-style heap, giving O(|E|·|V|·log|V|) in the worst case — much worse than O(V + E·log V).

2. **The modern node-moving** approach has no heap and no log V factor in the sweep. The VectorMap is O(1) amortized.

3. **O(V + E·log V)** is closer to Dijkstra's algorithm (with a binary heap) or to the Fast Clauset-Newman-Moore modularity algorithm's typical case O(|E| + |V|·log²|V|) — but neither applies to Infomap's map equation optimization.

4. The T071 claim may stem from conflating Infomap with modularity-based methods (Louvain/CNM) or from misreading the original paper's citation of the CNM heap-based approach.

### Edge Cases and Caveats

- **First sweep from singleton partition:** When every node starts in its own module, each node's candidate set is at most its degree + 2, so still O(E) total.
- **Convergence:** The outer loop runs until no improvement, with a configurable `coreLoopLimit`. The number of outer iterations is typically small (the paper notes convergence in few iterations), but worst-case is bounded by the loop limit.
- **Multilevel/hierarchical:** The full algorithm includes recursive partitioning of modules, but the per-sweep complexity at each level remains O(E_level) where E_level is the number of edges in the active network at that level.
- **Teleportation:** The PageRank-style teleportation adds a constant-time operation per node (pre-computed teleport weights), not a log V factor.

---

## 6. Recommendation

**Use O(E) per sweep (FR-028).** The T071 claim of O(V + E·log V) per-iteration is not supported by:

- The original 2008 paper (which describes a greedy-merge algorithm with heap-based O(|E|·|V|·log|V|) worst case)
- The modern mapequation/infomap C++ reference implementation (which uses Louvain-style node-moving with O(E) per sweep via O(1) VectorMap operations)
- The igraph documentation (which lists "Time complexity: TODO")
- Any published academic source claiming O(V + E·log V) for Infomap

**Suggested spec language:**

> Infomap's core optimization sweep runs in **O(E)** time per iteration, where E is the number of edges in the active network. Each node examines only its directly connected neighboring modules and evaluates the map equation delta in O(1) time per candidate using pre-computed flow data and the VectorMap structure. The algorithm converges in typically few outer iterations.

---

## Sources Consulted

1. Rosvall, M. & Bergstrom, C. T. (2008). "Maps of random walks on complex networks reveal community structure." *PNAS* 105(4), 1118–1123. https://doi.org/10.1073/pnas.0706851105 / https://arxiv.org/abs/0707.0609
2. Rosvall, M., Axelsson, D. & Bergstrom, C. T. (2009). "The map equation." *Eur. Phys. J. Special Topics* 178, 13–23. https://doi.org/10.1140/epjst/e2010-01179-1
3. mapequation/infomap C++ source: `src/core/InfomapOptimizer.h`, `src/core/MapEquation.h`, `src/utils/VectorMap.h` — https://github.com/mapequation/infomap
4. igraph C library: `src/community/infomap.cpp` — https://github.com/igraph/igraph
5. igraph documentation: https://igraph.org/c/html/latest/igraph-Community.html
6. Clauset, A., Newman, M. E. J. & Moore, C. (2004). "Finding community structure in very large networks." *Phys. Rev. E* 70, 066111.
7. Wakita, K. & Tsurumi, T. (2007). "Finding community structure in mega-scale social networks." arXiv:cs/0702048.

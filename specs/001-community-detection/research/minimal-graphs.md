# Minimal Graphs: Expected Behavior for Community Detection

## Research Date: 2026-01-28

## 1. Summary of Findings

This document defines the expected behavior when running community detection algorithms on three minimal graph cases:

| Case | Nodes | Edges | Description |
|------|-------|-------|-------------|
| A | 1 | 0 | Single isolated node |
| B | 2 | 1 | Single edge (two connected nodes) |
| C | 2 | 0 | Two disconnected nodes |

**Core finding:** All well-established algorithms return each node in its own community for these trivial cases, EXCEPT for the single-edge case where Leiden/Louvain/Infomap correctly identify one community of two nodes. Quality metrics require explicit guards for the m=0 (zero edges) case to avoid division by zero.

---

## 2. Algorithm-Specific Behavior

### 2.1 Leiden Algorithm (Traag et al. 2019)

**Paper:** "From Louvain to Leiden: guaranteeing well-connected communities" (Nature Scientific Reports, 2019) [1]

**Edge case handling in paper:** The paper does NOT explicitly address trivial graphs. It focuses on guaranteeing well-connected communities and fixing the Louvain algorithm's defect of producing disconnected communities. The algorithm is defined in terms of modularity optimization, which requires m > 0.

**Reference implementation (leidenalg v0.10.x):**

From the C++ source (`Optimiser.cpp`), the algorithm initializes each node in its own community (singleton partition), then iterates moving nodes to neighboring communities if it improves quality. For trivial graphs:

- **Case A (1 node, 0 edges):** The node has no neighbors. The `move_nodes` loop processes the node but finds no neighboring communities. Result: **1 community with 1 node**.
- **Case B (2 nodes, 1 edge):** Both nodes start in separate communities. Moving one node to the other's community improves modularity (since the edge is internal). Result: **1 community with 2 nodes**.
- **Case C (2 nodes, 0 edges):** No edges means no neighboring communities to consider. Result: **2 communities, each with 1 node**.

**Evidence from test suite:** The leidenalg test suite (`test_VertexPartition.py`, `test_Optimiser.py`) does NOT contain explicit tests for graphs with fewer than 3 nodes. The smallest test graphs are the Zachary karate club (34 nodes) and ER graphs with 100 nodes. This is a gap in the reference implementation's test coverage.

**Key implementation detail:** The `optimise_partition` loop in `Optimiser.cpp` has a termination condition based on aggregation: `aggregate_further = (new_collapsed_graphs[0]->vcount() < collapsed_graphs[0]->vcount()) && (collapsed_graphs[0]->vcount() > collapsed_partitions[0]->n_communities())`. For Case A, the graph cannot aggregate further (1 node, 1 community), so the algorithm terminates immediately.

### 2.2 Louvain Algorithm (Blondel et al. 2008)

**Paper:** "Fast unfolding of communities in large networks" (J. Stat. Mech., 2008) [2]

**Edge case handling in paper:** The paper does NOT address trivial graphs. It describes a two-phase approach: (1) local moving of nodes to maximize modularity gain, (2) aggregation of the network. The algorithm assumes a non-empty graph.

**Expected behavior (from algorithm description):**

- **Case A (1 node, 0 edges):** No neighbors to move to. Result: **1 community with 1 node**.
- **Case B (2 nodes, 1 edge):** Moving one node to the other's community gives positive modularity gain. Result: **1 community with 2 nodes**.
- **Case C (2 nodes, 0 edges):** No edges, no modularity gain possible. Result: **2 communities, each with 1 node**.

**igraph implementation:** The igraph C library's `cluster_louvain` follows the same logic. The documentation states: "The process stops when there is only a single vertex left or when the modularity cannot be increased any more in a step." For 2-node cases, the algorithm processes until no improvement is possible.

### 2.3 Infomap (Rosvall & Bergstrom 2008)

**Paper:** "Maps of random walks on complex networks reveal community structure" (PNAS, 2008) [3]

**Edge case handling in paper:** The paper does NOT address trivial graphs. Infomap uses random walks on the network to compress the description of information flow. The map equation requires a flow distribution.

**Reference implementation (mapequation/infomap):**

From the C++ source (`InfomapBase.cpp`), there is explicit handling for edge cases:

```cpp
void InfomapBase::initNetwork(Network& network)
{
  if (network.numNodes() == 0)
    throw std::domain_error("No nodes in network");
  // ...
}
```

And for flow validation:
```cpp
void checkFlowPostCondition() const
{
  const double sumNodeFlow = m_infomap.root().data.flow;
  const bool zeroFlowIsExpected = m_network.numLinks() == 0;
  if (!std::isfinite(sumNodeFlow) || (sumNodeFlow <= 0 && !zeroFlowIsExpected))
    throw InfomapError(ExitCode::InputError, ...);
  if (zeroFlowIsExpected)
    return;
  // ...
}
```

**Expected behavior:**

- **Case A (1 node, 0 edges):** The network has 1 node and 0 links. Flow is 0, which is expected. The algorithm returns **1 module with 1 node**. The codelength is 0 (trivial partition).
- **Case B (2 nodes, 1 edge):** Random walk flows along the edge. Both nodes are in the same module. Result: **1 module with 2 nodes**.
- **Case C (2 nodes, 0 edges):** No links means no flow. The algorithm returns **2 modules, each with 1 node** (each node is its own trivial module).

**Key insight:** Infomap explicitly handles the zero-flow case: "A link-free network has no flow to distribute and no structure to find; the trivial partition at zero bits is its correct answer."

### 2.4 LPA - Label Propagation Algorithm (Raghavan et al. 2007)

**Paper:** "Near linear time algorithm to detect community structures in large-scale networks" (Phys. Rev. E, 2007) [4]

**Edge case handling in paper:** The paper does NOT address trivial graphs. LPA initializes each node with a unique label and iteratively updates each node's label to the most frequent label among its neighbors.

**Reference implementation (NetworkX `asyn_lpa_communities`):**

From the source code (`label_propagation.py`):

```python
def asyn_lpa_communities(G, weight=None, seed=None):
    labels = {n: i for i, n in enumerate(G)}
    cont = True
    while cont:
        cont = False
        nodes = list(G)
        seed.shuffle(nodes)
        for node in nodes:
            if not G[node]:  # <-- KEY: skip isolated nodes
                continue
            # ... label propagation logic
```

The critical line is `if not G[node]: continue` — isolated nodes retain their initial unique label.

**Expected behavior:**

- **Case A (1 node, 0 edges):** The node has no neighbors (`G[node]` is empty). It keeps its unique label. Result: **1 community with 1 node**.
- **Case B (2 nodes, 1 edge):** Each node has one neighbor. Labels propagate. Result: **1 community with 2 nodes** (both get the same label, though which label depends on random order).
- **Case C (2 nodes, 0 edges):** Both nodes are isolated. Both keep their unique labels. Result: **2 communities, each with 1 node**.

**Note on randomness:** LPA is probabilistic. For Case B, the result is always 1 community, but the specific label assigned depends on the random processing order.

### 2.5 Fluid Communities (Parasidis et al. 2019)

**Paper:** "Fluid Communities: A Competitive, Scalable and Diverse Community Detection Algorithm" (arXiv:1703.09307) [5]

**Edge case handling in paper:** The paper does NOT address trivial graphs. FluidC requires specifying k (number of communities).

**Reference implementation (HPAI-BSC/Fluid-Communities):**

From the source code (`asyn_fluid_communities.py`):

```python
def asyn_fluidc(G, k, max_iter=100):
    max_density = 1.0
    vertices = list(G)
    random.shuffle(vertices)
    communities = {n: i for i, n in enumerate(vertices[:k])}
    # ...
```

**Expected behavior:**

- **Case A (1 node, 0 edges):** 
  - If k=1: The single node is assigned to community 0. Result: **1 community with 1 node**.
  - If k>1: Only 1 node available, so only 1 community is populated. Result: **1 non-empty community, k-1 empty communities** (implementation-dependent).
- **Case B (2 nodes, 1 edge):**
  - If k=1: Both nodes in community 0. Result: **1 community with 2 nodes**.
  - If k=2: Each node starts in its own community. The density update rule may or may not merge them depending on iteration. Result: **2 communities, each with 1 node** (for k=2).
  - If k>2: Only 2 nodes available. Result: **2 non-empty communities, k-2 empty communities**.
- **Case C (2 nodes, 0 edges):**
  - If k=1: Both nodes forced into community 0. Result: **1 community with 2 nodes**.
  - If k=2: Each node in its own community. No edges means no density-based merging. Result: **2 communities, each with 1 node**.

**Critical issue:** FluidC requires k <= number of nodes. If k > number of nodes, the initialization `communities = {n: i for i, n in enumerate(vertices[:k])}` will only assign communities to available nodes, and the remaining communities will be empty. The implementation does NOT guard against k > n.

---

## 3. Quality Metric Edge Cases

### 3.1 Modularity (Newman & Girvan 2004)

**Formula:** Q = (1/2m) * Σ_ij [A_ij - (k_i * k_j / 2m)] * δ(c_i, c_j)

**Edge cases:**

- **Case A (1 node, 0 edges):** m = 0. The formula has 1/(2m) = 1/0 → **DIVISION BY ZERO**. Modularity is undefined.
- **Case B (2 nodes, 1 edge):** m = 1. If both nodes in same community: Q = (1/2) * [1 - (1*1/2)] = (1/2) * (1/2) = 0.25. If separate: Q = 0.
- **Case C (2 nodes, 0 edges):** m = 0. **DIVISION BY ZERO**. Modularity is undefined.

**How implementations handle this:**

- **NetworkX** (`quality.py`): Explicitly checks `if m == 0: return 0`. Returns 0 for any graph with no edges.
- **igraph** (`modularity.igraph`): Returns NaN or 0 for empty graphs (version-dependent).
- **leidenalg:** The C++ implementation computes quality via `diff_move` which also divides by m. For m=0, the behavior is undefined at the C++ level (floating point division by zero → NaN or inf).

**Recommendation:** The spec MUST define that modularity returns 0.0 for any graph with m=0 edges, matching NetworkX behavior.

### 3.2 Constant Potts Model (CPM) (Traag et al. 2011)

**Formula:** Q = Σ_c [m_c - γ * C(n_c, 2)]

Where m_c is internal edges, n_c is community size, γ is resolution parameter, and C(n,2) = n*(n-1)/2.

**Edge cases:**

- **Case A (1 node, 0 edges):** m_c = 0, n_c = 1, C(1,2) = 0. Q = 0 - γ * 0 = **0**.
- **Case B (2 nodes, 1 edge):** 
  - Same community: m_c = 1, n_c = 2, C(2,2) = 1. Q = 1 - γ * 1 = **1 - γ**.
  - Separate: m_c = 0 for both, Q = 0 - 0 + 0 - 0 = **0**.
- **Case C (2 nodes, 0 edges):** m_c = 0 for any partition. Q = 0 - γ * C(n_c, 2). For separate: Q = 0. For together: Q = -γ.

**CPM is well-defined for all cases** — no division by zero risk.

### 3.3 Surprise (Traag et al. 2015)

**Formula:** Q = m * D(q || ⟨q⟩)

Where q is observed internal edge fraction, ⟨q⟩ is expected fraction, D is KL divergence.

**Edge cases:**

- **Case A (1 node, 0 edges):** m = 0. Q = 0 * D(...) = **0** (by convention, 0 * undefined = 0).
- **Case B (2 nodes, 1 edge):** m = 1. q = 1 (all edges internal), ⟨q⟩ = C(2,2)/C(2,2) = 1. D(1||1) = 0. Q = **0**.
- **Case C (2 nodes, 0 edges):** m = 0. Q = **0**.

**Surprise is well-defined** when m=0 by multiplying first.

### 3.4 Significance (Traag et al. 2013)

**Formula:** Q = Σ_c C(n_c, 2) * D(p_c || p)

Where p_c = m_c / C(n_c, 2), p = m / C(n, 2).

**Edge cases:**

- **Case A (1 node, 0 edges):** C(1,2) = 0. Q = 0 * D(...) = **0**.
- **Case B (2 nodes, 1 edge):** C(2,2) = 1. p_c = 1/1 = 1, p = 1/1 = 1. D(1||1) = 0. Q = **0**.
- **Case C (2 nodes, 0 edges):** C(2,2) = 1. p = 0/1 = 0. D(p_c || 0) involves log(p_c/0) → **undefined** (division by zero in KL divergence).

**Significance has a division-by-zero risk** when m=0 and nodes are in the same community.

### 3.5 Map Equation (Infomap's quality metric)

**Edge cases:**

- **Case A (1 node, 0 edges):** Codelength = 0 (trivial partition).
- **Case B (2 nodes, 1 edge):** Codelength > 0 (one module describes both nodes).
- **Case C (2 nodes, 0 edges):** Codelength = 0 (each node is its own module, no flow to encode).

The map equation is well-defined for all cases because it operates on flow distributions, and zero flow is explicitly handled.

---

## 4. Algorithm Comparison for Minimal Graphs

| Algorithm | Case A (1 node, 0 edges) | Case B (2 nodes, 1 edge) | Case C (2 nodes, 0 edges) |
|-----------|--------------------------|--------------------------|--------------------------|
| Leiden | 1 community (1 node) | 1 community (2 nodes) | 2 communities (1 each) |
| Louvain | 1 community (1 node) | 1 community (2 nodes) | 2 communities (1 each) |
| Infomap | 1 module (1 node) | 1 module (2 nodes) | 2 modules (1 each) |
| LPA | 1 community (1 node) | 1 community (2 nodes) | 2 communities (1 each) |
| FluidC (k=1) | 1 community (1 node) | 1 community (2 nodes) | 1 community (2 nodes) |
| FluidC (k=2) | 1 community (1 node) + 1 empty | 2 communities (1 each) | 2 communities (1 each) |

**Key observation:** All algorithms agree on Cases A and C. For Case B, Leiden/Louvain/Infomap/LPA correctly identify the edge as a single community. FluidC with k=2 keeps them separate (by design, since k is fixed).

---

## 5. Recommendations for Spec Requirements

### 5.1 Required Behaviors

1. **Single node (no edges):** MUST return 1 community containing the single node. Membership vector: `[0]`.

2. **Single edge (2 connected nodes):** MUST return 1 community containing both nodes. Membership vector: `[0, 0]`.

3. **Two disconnected nodes:** MUST return 2 communities, each with 1 node. Membership vector: `[0, 1]`.

### 5.2 Quality Metric Guards

4. **Modularity:** MUST return 0.0 when the graph has 0 edges (m=0). This avoids division by zero and matches NetworkX behavior.

5. **CPM:** No special guard needed. Well-defined for all cases.

6. **Surprise:** No special guard needed. Well-defined for all cases (0 * undefined = 0).

7. **Significance:** MUST return 0.0 or NaN when m=0. The spec should define this explicitly (recommend: return 0.0 with a warning).

8. **Map Equation:** No special guard needed. Well-defined for all cases.

### 5.3 Input Validation

9. **Empty graph (0 nodes):** The spec SHOULD define behavior. Recommendation: raise `ValueError("Graph has no nodes")` to match Infomap's behavior.

10. **k > n for FluidC:** The spec SHOULD define behavior when the requested number of communities exceeds the number of nodes. Recommendation: raise `ValueError("k cannot exceed number of nodes")` or clamp k to n.

### 5.4 Determinism

11. **LPA randomness:** The spec SHOULD note that LPA results for Case B are deterministic in structure (always 1 community) but the specific label values may vary across runs due to random processing order.

12. **FluidC randomness:** The spec SHOULD note that FluidC uses random initialization and may produce different results across runs for small graphs.

---

## 6. Citations

[1] Traag, V.A., Waltman, L., & Van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z

[2] Blondel, V.D., Guillaume, J.L., Lambiotte, R., & Lefebvre, E. (2008). "Fast unfolding of communities in large networks." *J. Stat. Mech.*, P10008. https://doi.org/10.1088/1742-5468/2008/10/P10008

[3] Rosvall, M. & Bergstrom, C.T. (2008). "Maps of random walks on complex networks reveal community structure." *PNAS*, 105(4), 1118-1123. https://doi.org/10.1073/pnas.0706851105

[4] Raghavan, U.N., Albert, R., & Kumara, S. (2007). "Near linear time algorithm to detect community structures in large-scale networks." *Phys. Rev. E*, 76(3), 036106. https://doi.org/10.1103/PhysRevE.76.036106

[5] Parasidis, F., García-Gasulla, D., et al. (2019). "Fluid Communities: A Competitive, Scalable and Diverse Community Detection Algorithm." arXiv:1703.09307. https://arxiv.org/abs/1703.09307

[6] Newman, M.E.J. & Girvan, M. (2004). "Finding and evaluating community structure in networks." *Phys. Rev. E*, 69(2), 026113. https://doi.org/10.1103/PhysRevE.69.026113

[7] Traag, V.A., Van Dooren, P., & Nesterov, Y. (2011). "Narrow scope for resolution-limit-free community detection." *Phys. Rev. E*, 84(1), 016114. https://doi.org/10.1103/PhysRevE.84.016114

[8] Traag, V.A., Aldecoa, R., & Delvenne, J.C. (2015). "Detecting communities using asymptotical surprise." *Phys. Rev. E*, 92(2), 022816. https://doi.org/10.1103/PhysRevE.92.022816

[9] Traag, V.A., Krings, G., & Van Dooren, P. (2013). "Significant scales in community structure." *Scientific Reports*, 3, 2930. https://doi.org/10.1038/srep02930

---

## Appendix: Source Code References

### leidenalg (Python Leiden)
- Repository: https://github.com/vtraag/leidenalg
- C++ core: https://github.com/vtraag/libleidenalg
- Optimiser.cpp: `move_nodes` loop at line ~230, aggregation termination at line ~180
- Tests: `tests/test_VertexPartition.py`, `tests/test_Optimiser.py` (no minimal graph tests found)

### NetworkX (LPA implementation)
- Repository: https://github.com/networkx/networkx
- File: `networkx/algorithms/community/label_propagation.py`
- Key line: `if not G[node]: continue` (line ~138) — skips isolated nodes

### Infomap
- Repository: https://github.com/mapequation/infomap
- File: `src/core/InfomapBase.cpp`
- Key: `validateNetwork()` throws on empty network, `checkFlowPostCondition()` handles zero-flow case

### Fluid Communities
- Repository: https://github.com/HPAI-BSC/Fluid-Communities
- File: `asyn_fluid_communities.py`
- Key: Initialization `communities = {n: i for i, n in enumerate(vertices[:k])}` — no guard for k > n

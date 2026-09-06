# Leiden Algorithm: Quality Functions Research

**Date:** 2026-01-28
**Context:** Determining scope for FR-008 quality function implementation in the Communal framework's Leiden algorithm (Rust).

---

## 1. What Quality Functions Does Leiden Actually Support?

### Per the Reference Paper (Traag et al. 2019)

The Leiden paper explicitly states that the algorithm optimizes **either Modularity Q or the Constant Potts Model (CPM)**. At no point in the paper is Map Equation mentioned as a Leiden-compatible quality function.

Key quotation from the paper (Section III.A):

> "In these properties, γ refers to the resolution parameter in the quality function that is optimised, which can be either modularity or CPM."

The paper's mathematical framework defines two quality functions:

| Quality Function | Formula | Null Model |
|---|---|---|
| **Modularity Q** | H = (1/2m) Σ_c [e_c − γ · (K_c²/2m)] | Configuration model (degree-preserving) |
| **CPM** | H = Σ_c [e_c − γ · (n_c choose 2)] | Constant Potts (size-based) |

Where:
- e_c = number of edges inside community c
- K_c = sum of degrees of nodes in community c
- n_c = number of nodes in community c
- γ = resolution parameter
- m = total number of edges

### Per the Reference C++ Implementation (libleidenalg)

The `libleidenalg` library exposes 6 `MutableVertexPartition` subclasses for use with the Leiden `Optimiser` class:

1. **`ModularityVertexPartition`** — Newman-Girvan modularity
2. **`CPMVertexPartition`** — Constant Potts Model (with resolution parameter)
3. **`RBConfigurationVertexPartition`** — Reichardt-Bornholdt with configuration null model
4. **`RBERVertexPartition`** — Reichardt-Bornholdt with Erdős-Rényi null model
5. **`SignificanceVertexPartition`** — Traag et al. 2013 significance measure
6. **`SurpriseVertexPartition`** — Traag et al. 2015 surprise measure

**Map Equation is NOT among them.** There is no `MapEquationVertexPartition` or equivalent.

### Per the Python Implementation (leidenalg)

The Python `leidenalg` package mirrors the C++ library exactly. The `find_partition()` function accepts these partition types:

- `la.ModularityVertexPartition`
- `la.CPMVertexPartition`
- `la.RBConfigurationVertexPartition`
- `la.RBERVertexPartition`
- `la.SignificanceVertexPartition`
- `la.SurpriseVertexPartition`

Again, **no Map Equation support**. The package documentation states:

> "This implementation provides a general optimisation routine for any quality function."

This generality is about the framework accepting *any* custom partition type that implements `diff_move()` and `quality()` — not about supporting Map Equation specifically.

---

## 2. Is Map Equation Used by Leiden?

**No. Map Equation is exclusive to Infomap.**

### Evidence

The Map Equation is an information-theoretic quality function that measures the codelength (in bits) of a random walk on the network. It is fundamentally different from modularity-style quality functions:

- **Map Equation** compresses a *description of information flow* (random walk). It is flow-based.
- **Modularity/CPM** measure *edge density against a null model*. They are density-based.

The Infomap documentation explicitly contrasts the two families:

> "Louvain and Leiden score a partition by **modularity**: whether more edges fall inside groups than a random graph with the same degrees would predict. Infomap scores it by the **map equation**: how few bits describe a random walk on the network under that partition."

> "Leiden is a general optimiser: it maximises whichever quality function you give it, adds a refinement step that guarantees internally connected communities, and reaches better optima than Louvain's greedy moves."

This means Leiden *can* optimize various quality functions (Modularity, CPM, Significance, Surprise), but Map Equation operates under a fundamentally different paradigm (flow compression via random walks) that is not compatible with the Leiden refinement/aggregation framework as described in the Traag et al. paper.

### Practical confirmation

The Infimap and Leiden communities are separate:
- **Infomap** source: `github.com/mapequation/infomap` — implements Map Equation exclusively
- **Leiden** source: `github.com/vtraag/leidenalg` + `github.com/vtraag/libleidenalg` — implements density-based quality functions only

There is no cross-pollination: no Map Equation in leidenalg, no Modularity/CPM in Infomap.

---

## 3. Resolution Parameter γ: Modularity vs. CPM

Both quality functions use a resolution parameter γ but interpret it differently:

| Property | Modularity Q | CPM |
|---|---|---|
| **Formula** | H = (1/2m) Σ [e_c − γ · (K_c²/2m)] | H = Σ [e_c − γ · (n_c choose 2)] |
| **γ interpretation** | Scales expected edges under degree-preserving null model | Community density threshold |
| **Null model** | Configuration model (keeps degrees) | Constant Potts (simple size-based) |
| **Resolution limit** | Yes (suffers from it) | No (resolution-limit-free) |
| **γ range** | Typically [0, 1] for normalized; >1 for finer | Depends on edge density |

Key insight from the Traag 2011 paper: CPM is **resolution-limit-free**, meaning it can detect communities of any size regardless of network scale. Modularity suffers from the resolution limit (Fortunato & Barthélemy 2007), which CPM overcomes.

The RBConfiguration model generalizes both: it uses Reichardt-Bornholdt's statistical mechanics framework with a configuration null model, which reduces to Modularity when using the appropriate null model and to CPM-like behavior with the Erdős-Rényi variant.

---

## 4. Recommendation for FR-008 Scope

### Core (implement now)

| Priority | Quality Function | Rationale |
|---|---|---|
| **P0** | **Modularity Q** | The most widely used; paper's primary example; baseline for all comparisons |
| **P0** | **CPM** | Paper's second named function; resolution-limit-free; uses same γ parameter |

### Extended (implement if scope allows)

| Priority | Quality Function | Rationale |
|---|---|---|
| **P1** | **RBConfiguration** | Generalization of Modularity; Reichardt-Bornholdt framework |
| **P1** | **RBER** | Erdős-Rényi variant of RB; useful for specific null model assumptions |
| **P2** | **Significance** | Traag et al. 2013; detects statistically significant communities |
| **P2** | **Surprise** | Traag et al. 2015; alternative statistical measure |

### Out of scope for Leiden

| Quality Function | Reason |
|---|---|
| **Map Equation** | Exclusive to Infomap; fundamentally different paradigm (flow compression vs. density optimization); not supported by any reference Leiden implementation |

### Suggested approach

1. **Implement Modularity Q and CPM first** — these are the two quality functions the Leiden paper explicitly names and for which the paper's theoretical guarantees (γ-separation, γ-connectivity, subset optimality) are proven.
2. **Defer Map Equation** — it belongs to Infomap, not Leiden. If the Communal framework needs Map Equation support, it should be implemented as a separate Infomap algorithm module, not as a Leiden quality function.
3. **Consider RBConfiguration as a generalization** — if you want to support the Reichardt-Bornholdt framework, RBConfiguration subsumes Modularity as a special case, potentially reducing the number of distinct implementations needed.

---

## 5. Citations

1. Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z — arXiv:1810.08473

2. Traag, V.A., Van Dooren, P., & Nesterov, Y. (2011). "Narrow scope for resolution-limit-free community detection." *Physical Review E*, 84(1), 016114. https://doi.org/10.1103/PhysRevE.84.016114 — arXiv:1104.3083

3. Reichardt, J., & Bornholdt, S. (2006). "Statistical mechanics of community detection." *Physical Review E*, 74(1), 016110. https://doi.org/10.1103/PhysRevE.74.016110

4. Traag, V.A., Krings, G., & Van Dooren, P. (2013). "Significant scales in community structure." *Scientific Reports*, 3, 2930. https://doi.org/10.1038/srep02930

5. Traag, V.A., Aldecoa, R., & Delvenne, J.-C. (2015). "Detecting communities using asymptotical surprise." *Physical Review E*, 92(2), 022816. https://doi.org/10.1103/PhysRevE.92.022816

6. Fortunato, S., & Barthélemy, M. (2007). "Resolution limit in community detection." *Proceedings of the National Academy of Sciences*, 104(1), 36-41. https://doi.org/10.1073/pnas.0605965104

7. Newman, M.E.J., & Girvan, M. (2004). "Finding and evaluating community structure in networks." *Physical Review E*, 69(2), 026113. https://doi.org/10.1103/PhysRevE.69.026113

8. Blondel, V.D., Guillaume, J.-L., Lambiotte, R., & Lefebvre, E. (2008). "Fast unfolding of communities in large networks." *Journal of Statistical Mechanics: Theory and Experiment*, 2008(10), P10008. https://doi.org/10.1088/1742-5468/2008/10/P10008

9. Rosvall, M., & Bergstrom, C.T. (2008). "Maps of random walks on complex networks reveal community structure." *Proceedings of the National Academy of Sciences*, 105(4), 1118-1123. https://doi.org/10.1073/pnas.0706851105

10. vtraag/libleidenalg — C++ reference implementation. https://github.com/vtraag/libleidenalg

11. vtraag/leidenalg — Python reference implementation. https://github.com/vtraag/leidenalg

12. Infomap documentation — "Reading Infomap through Louvain and Leiden." https://mapequation.org/infomap-python-docs/concepts/choosing-a-method.html

13. leidenalg documentation. https://leidenalg.readthedocs.io/

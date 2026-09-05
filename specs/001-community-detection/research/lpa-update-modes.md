# LPA Update Modes: Synchronous vs Asynchronous vs Semi-Synchronous

## Summary of Findings

| Mode | Convergence | Parallelizability | Stability | Oscillation Risk |
|------|-------------|-------------------|-----------|------------------|
| **Synchronous** | Not guaranteed | High (embarrassingly parallel) | High (deterministic) | Yes (bipartite graphs) |
| **Asynchronous** | Guaranteed | Low (sequential) | Low (order-dependent) | No |
| **Semi-synchronous** | Guaranteed | Medium-High | Medium-High | No |

**Recommendation:** Support **asynchronous** (canonical, always converges) and **semi-synchronous** (better parallelism + stability). Default to **asynchronous** for fidelity to the original paper. The semi-synchronous mode is a valuable alternative for parallel execution contexts.

---

## 1. Behavioral Difference Between Synchronous and Asynchronous Updates

### Synchronous Updates

In the synchronous model, at iteration `t`, every node computes its new label based **exclusively** on the labels of its neighbors at iteration `t-1`. All updates happen simultaneously:

```
C_x(t) = f(C_{x1}(t-1), C_{x2}(t-1), ..., C_{xk}(t-1))
```

where `C_x(t)` is the label of node `x` at time `t` and `f` returns the most frequent label among neighbors (ties broken randomly).

**Key property:** The update of any node at step `t` does not depend on any other node's update at step `t`. This makes synchronous updates embarrassingly parallel.

### Asynchronous Updates

In the asynchronous model, nodes are updated one at a time in a random order. When node `x` is updated, some of its neighbors may have already been updated in the current iteration (carrying labels from iteration `t`), while others have not (still carrying labels from iteration `t-1`):

```
C_x(t) = f(C_{xi1}(t), ..., C_{xim}(t), C_{xi(m+1)}(t-1), ..., C_{xik}(t-1))
```

where `xi_1, ..., xi_m` are neighbors already updated in the current iteration and `xi_{m+1}, ..., xi_k` are neighbors not yet updated.

**Key property:** The update order matters and is randomized before each iteration. This breaks symmetry and prevents oscillations.

### Semi-Synchronous Updates

The semi-synchronous model (Cordasco & Gargano, 2010/2012) partitions nodes into color classes such that no two adjacent nodes share the same color. Updates proceed color class by color class (synchronously within each class, sequentially across classes). Since adjacent nodes never update simultaneously, oscillations are prevented while retaining parallelism within each color class.

---

## 2. Canonical/Default Mode in the Original LPA Paper (Raghavan et al. 2007)

**The original paper explicitly adopts asynchronous updating as the canonical mode.**

From the paper (Section III, "Community Detection using Label Propagation", p. 4):

> "The updating process can either be synchronous or asynchronous. In synchronous updating, node x at the t-th iteration updates its label based on the labels of its neighbors at iteration t − 1. Hence, `C_x(t) = f(C_{x1}(t − 1), ..., C_{xk}(t − 1))`, where `c_x(t)` is the label of node x at time t. The problem however is that subgraphs in the network that are bi-partite or nearly bi-partite in structure lead to oscillations of labels (see Figure 3). This is especially true in cases where communities take the form of a star graph. **Hence we use asynchronous updating** where `C_x(t) = f(C_{xi1}(t), ..., C_{xim}(t), C_{xi(m+1)}(t − 1), ..., C_{xik}(t − 1))` and `xi_1, ..., xi_m` are neighbors of x that have already been updated in the current iteration while `xi_{m+1}, ..., xi_k` are neighbors that are not yet updated in the current iteration. The order in which all the n nodes in the network are updated at each iteration is chosen randomly."

**Source:** Raghavan, U.N., Albert, R., and Kumara, S. (2007). "Near linear time algorithm to detect community structures in large-scale networks." *Physical Review E*, 76(3), 036106. DOI: [10.1103/PhysRevE.76.036106](https://doi.org/10.1103/PhysRevE.76.036106). arXiv: [0709.2938](https://arxiv.org/abs/0709.2938).

The paper's Algorithm (steps 1–5, p. 5) formalizes the asynchronous approach:
1. Initialize: `C_x(0) = x` for all nodes
2. Set `t = 1`
3. Arrange nodes in random order `X`
4. For each `x ∈ X` in that order: `C_x(t) = f(...)` (using already-updated neighbors)
5. Stop when every node has a label that the maximum number of its neighbors have

---

## 3. How Existing Implementations Expose This Choice

### 3.1 `python-igraph` (C library + Python bindings)

**C-level API** exposes the choice via an **enum** (`igraph_lpa_variant_t`):

```c
// From: https://github.com/igraph/igraph/blob/master/include/igraph_constants.h
typedef enum {
    IGRAPH_LPA_DOMINANCE = 0, /* Sample from dominant labels, check for dominance after each iteration. */
    IGRAPH_LPA_RETENTION,     /* Keep current label if among dominant labels, only check if labels changed. */
    IGRAPH_LPA_FAST           /* Sample from dominant labels, only check neighbors. */
} igraph_lpa_variant_t;
```

The C function signature:
```c
igraph_error_t igraph_community_label_propagation(
    const igraph_t *graph,
    igraph_vector_int_t *membership,
    igraph_neimode_t mode,
    const igraph_vector_t *weights,
    const igraph_vector_int_t *initial,
    const igraph_vector_bool_t *fixed,
    igraph_lpa_variant_t lpa_variant   // <-- enum parameter
);
```

**Variant descriptions from the C source** ([label_propagation.c](https://github.com/igraph/igraph/blob/master/src/community/label_propagation.c)):

- **`IGRAPH_LPA_DOMINANCE`**: After updating all labels, an additional iteration checks whether all nodes have a dominant label. Alternates between update iterations and control iterations.
- **`IGRAPH_LPA_RETENTION`**: Nodes retain their current label if it is already among the dominant labels. Only updates when the current label is not dominant. Terminates when no labels change.
- **`IGRAPH_LPA_FAST`**: Uses a queue-based approach (from Traag & Šubelj, 2023). Only neighbors of changed nodes are re-checked. Terminates when the queue is empty.

**Python-level API** (`python-igraph`): The Python wrapper `_community_label_propagation` in `community.py` does **NOT** expose the `lpa_variant` parameter. It only accepts `graph`, `weights`, `initial`, and `fixed`. The variant is not configurable from Python.

**Source:** [igraph/igraph/include/igraph_constants.h](https://github.com/igraph/igraph/blob/master/include/igraph_constants.h) (lines with `igraph_lpa_variant_t`); [igraph/igraph/src/community/label_propagation.c](https://github.com/igraph/igraph/blob/master/src/community/label_propagation.c); [igraph/python-igraph/src/igraph/community.py](https://github.com/igraph/python-igraph/blob/main/src/igraph/community.py).

### 3.2 `networkx`

NetworkX exposes the choice via **three separate functions** (no enum, no boolean flag):

| Function | Mode | Reference |
|----------|------|-----------|
| `asyn_lpa_communities(G, weight, seed)` | **Asynchronous** (Raghavan et al. 2007) | [NetworkX Docs](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.asyn_lpa_communities.html) |
| `label_propagation_communities(G)` | **Semi-synchronous** (Cordasco & Gargano 2010) | [NetworkX Docs](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.label_propagation_communities.html) |
| `fast_label_propagation_communities(G, weight, seed)` | **Fast LPA** (Traag & Šubelj 2023) | [NetworkX Docs](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.fast_label_propagation_communities.html) |

**Asynchronous implementation** (`asyn_lpa_communities`): Shuffles nodes randomly each iteration, updates each node in-place using `Counter` of neighbor labels. Continues until no node changes its label.

**Semi-synchronous implementation** (`label_propagation_communities`): First colors the graph with `nx.coloring.greedy_color(G)`, then updates all nodes of the same color simultaneously in each stage. Uses Prec-Max tie-breaking.

**Source:** [networkx/algorithms/community/label_propagation.html](https://networkx.org/documentation/stable/_modules/networkx/algorithms/community/label_propagation.html).

### 3.3 `leidenalg` / `louvain-igraph`

The `leidenalg` Python package does **NOT** include a Label Propagation implementation. It focuses exclusively on the Leiden algorithm (Traag, van Eck & Waltman, 2019), which is a modularity-based method, not LPA.

The `leidenAlg` R package similarly implements only the Leiden algorithm.

**Source:** [vtraag/leidenalg](https://github.com/vtraag/leidenalg); [CRAN leidenAlg](https://cran.r-project.org/package=leidenAlg).

### 3.4 Rust Implementations

**`label-propagation` crate** ([docs.rs/label-propagation](https://docs.rs/label-propagation)): Implements Label Propagation for **semi-supervised learning** (SSL), not community detection. Provides `LGC` (Local and Global Consistency) and `CAMLP` (Confidence-Aware Modulated Label Propagation). Not relevant to the Raghavan et al. community detection variant.

**`Jerrykl/LPA`** ([GitHub](https://github.com/Jerrykl/LPA)): A high-performance parallel LPA in Rust. The README describes a simple iterative approach: "Traverse over each node and update its own label with the largest number of labels in the labels of its neighbor nodes." The implementation appears to use a synchronous-style parallel update (all nodes updated simultaneously based on previous iteration), but the source code was not fully inspectable. No explicit mode choice is exposed.

**`rust-igraph`** ([totoro-jam/rust-igraph](https://totoro-jam.github.io/rust-igraph)): Provides `LpaVariant` enum that mirrors `igraph_lpa_variant_t`:
```rust
enum LpaVariant {
    Dominance,  // IGRAPH_LPA_DOMINANCE
    Retention,  // IGRAPH_LPA_RETENTION
    Fast,       // IGRAPH_LPA_FAST
}
```

---

## 4. Convergence and Quality Implications

### 4.1 Convergence

| Mode | Guaranteed Convergence? | Notes |
|------|------------------------|-------|
| Synchronous | **No** | Oscillations in bipartite/near-bipartite graphs and star graphs (Raghavan et al. 2007, p. 4) |
| Asynchronous | **Yes** | Random update order breaks symmetry; stop criterion (c1) ensures termination (Raghavan et al. 2007, p. 5) |
| Semi-synchronous | **Yes** | With LPA-Prec, LPA-Max, or LPA-Prec-Max tie-breaking, converges to period 1 (Cordasco & Gargano 2012, Theorem 5.1) |

**Synchronous oscillation example** (from Raghavan et al. 2007, Figure 3): In a bipartite graph, labels oscillate between two sets because each side copies the other side's previous label. This also occurs in star graphs and other non-bipartite structures (Cordasco & Gargano 2012, Figure 1).

**Asynchronous stop criterion** (Raghavan et al. 2007, p. 5): The algorithm stops when every node has a label to which the maximum number of its neighbors belong. Formally: if node `i` has label `C_m`, then `d_{C_m}^i ≥ d_{C_j}^i` for all `j`. This is a local optimum condition, not a global objective maximization.

**Semi-synchronous convergence proof** (Cordasco & Gargano 2012, Theorem 5.1): For LPA-Prec, LPA-Max, and LPA-Prec-Max tie-breaking strategies, the semi-synchronous algorithm converges to a stable labeling (period 1). The proof uses a potential function `f(i)` that strictly increases with each step.

### 4.2 Quality

**Empirical findings from Cordasco & Gargano (2012):**

- Semi-synchronous and asynchronous produce **comparable quality** partitions (measured by modularity, NMI, and accuracy).
- Semi-synchronous slightly outperforms asynchronous on Karate, Dolphins, Football, and E-mail networks.
- Asynchronous slightly outperforms semi-synchronous on NetScience, Power, HepTh, and Cond-Mat networks.
- The authors stress: "our goal here is not to improve the quality of the results obtained by the asynchronous approach; rather we want to show that the speed-up obtained with the semi-synchronous approach does not degrade any quality."

**Source:** Cordasco, G. and Gargano, L. (2012). "Label propagation algorithm: a semi-synchronous approach." *International Journal of Social Network Mining*, 1(1), 3–26. DOI: [10.1504/IJSNM.2012.045103](https://doi.org/10.1504/IJSNM.2012.045103). Zenodo: [14423559](https://zenodo.org/records/14423559).

### 4.3 Stability

- **Asynchronous**: Low stability. Different random seeds produce different partitions due to random update order and random tie-breaking. Multiple runs are needed to aggregate results (Raghavan et al. 2007, Section IV.A).
- **Semi-synchronous**: Higher stability. Less randomization (only initial label assignment and tie-breaking within color classes). Cordasco & Gargano (2012) show lower standard deviation in modularity across runs.
- **Synchronous**: High stability (deterministic given fixed tie-breaking), but convergence is not guaranteed.

### 4.4 Efficiency and Parallelizability

- **Synchronous**: O(d) time per iteration where d is maximum degree (fully parallelizable). But may not terminate.
- **Asynchronous**: O(m) time per iteration (sequential). Each step processes all n nodes.
- **Semi-synchronous**: O(m) time per iteration, but each stage is parallelizable. Number of stages bounded by Δ(G) + 1 (max degree + 1). Wall-clock time with unbounded CPUs: O(stages × log n).

**Source:** Cordasco & Gargano (2012), Section 3.2 and Section 6.4.2.

### 4.5 "Monster Community" Problem

Leung et al. (2009) observed that asynchronous LPA tends to produce a "monster" community (one very large community plus many small ones). This is because the random update order in early iterations favors the spread of certain labels. The semi-synchronous approach slightly reduces this phenomenon by slowing down label propagation.

**Source:** Leung, I.X.Y., Hui, P., Liò, P., and Crowcroft, J. (2009). "Towards real-time community detection in large networks." *Physical Review E*, 79(6). DOI: [10.1103/PhysRevE.79.066103](https://doi.org/10.1103/PhysRevE.79.066103).

---

## 5. Common API Patterns Across Libraries

| Library | Mechanism | Modes Exposed |
|---------|-----------|---------------|
| **igraph (C)** | Enum parameter (`igraph_lpa_variant_t`) | DOMINANCE, RETENTION, FAST |
| **python-igraph** | Not exposed (hardcoded) | Only one mode available |
| **networkx** | Separate functions | `asyn_lpa_communities`, `label_propagation_communities`, `fast_label_propagation_communities` |
| **rust-igraph** | Enum parameter (`LpaVariant`) | Dominance, Retention, Fast |
| **R igraph** | Enum parameter (wraps C) | DOMINANCE, RETENTION, FAST |

**Two dominant patterns:**
1. **Enum parameter** (igraph family): A single function with a variant enum. Clean, extensible.
2. **Separate functions** (networkx): Each mode is a standalone function. Discoverable, no need to understand enum values.

---

## 6. Recommendations

### Modes to Support

1. **Asynchronous** (default): Canonical mode from Raghavan et al. (2007). Always converges. Use this as the default for fidelity to the original algorithm and guaranteed termination.

2. **Semi-synchronous**: Based on Cordasco & Gargano (2010/2012). Better parallelism and stability with comparable quality. Valuable for multi-threaded execution.

3. **Fast LPA** (optional): Based on Traag & Šubelj (2023). Queue-based approach that only re-checks neighbors of changed nodes. Most efficient for large sparse graphs.

### Default

**Asynchronous** should be the default. It is the canonical mode from the original paper, always converges, and is the most widely implemented and validated.

### API Pattern Recommendation

For a Rust implementation, the **enum parameter** pattern (as used by igraph) is idiomatic:

```rust
pub enum LpaVariant {
    /// Asynchronous updates (Raghavan et al. 2007) - default, always converges
    Asynchronous,
    /// Semi-synchronous updates (Cordasco & Gargano 2010) - parallelizable, stable
    SemiSynchronous,
    /// Fast LPA (Traag & Šubelj 2023) - queue-based, most efficient
    Fast,
}
```

This is more extensible than separate functions and more discoverable than a boolean flag.

---

## References

1. Raghavan, U.N., Albert, R., and Kumara, S. (2007). "Near linear time algorithm to detect community structures in large-scale networks." *Physical Review E*, 76(3), 036106. DOI: [10.1103/PhysRevE.76.036106](https://doi.org/10.1103/PhysRevE.76.036106). arXiv: [0709.2938](https://arxiv.org/abs/0709.2938).

2. Cordasco, G. and Gargano, L. (2012). "Label propagation algorithm: a semi-synchronous approach." *International Journal of Social Network Mining*, 1(1), 3–26. DOI: [10.1504/IJSNM.2012.045103](https://doi.org/10.1504/IJSNM.2012.045103). Zenodo: [14423559](https://zenodo.org/records/14423559).

3. Cordasco, G. and Gargano, L. (2010). "Community detection via semi-synchronous label propagation algorithms." *Proceedings of the IEEE International Workshop on Business Applications of Social Network Analysis (BASNA 2010)*, Bangalore, India, 15 December 2010.

4. Traag, V.A. and Šubelj, L. (2023). "Large network community detection by fast label propagation." *Scientific Reports*, 13, 2701. DOI: [10.1038/s41598-023-29610-z](https://doi.org/10.1038/s41598-023-29610-z). arXiv: [2209.13338](https://arxiv.org/abs/2209.13338).

5. Leung, I.X.Y., Hui, P., Liò, P., and Crowcroft, J. (2009). "Towards real-time community detection in large networks." *Physical Review E*, 79(6). DOI: [10.1103/PhysRevE.79.066103](https://doi.org/10.1103/PhysRevE.79.066103).

6. igraph C library — `igraph_lpa_variant_t` enum: [github.com/igraph/igraph/blob/master/include/igraph_constants.h](https://github.com/igraph/igraph/blob/master/include/igraph_constants.h)

7. igraph C library — label propagation implementation: [github.com/igraph/igraph/blob/master/src/community/label_propagation.c](https://github.com/igraph/igraph/blob/master/src/community/label_propagation.c)

8. python-igraph Python bindings: [github.com/igraph/python-igraph/blob/main/src/igraph/community.py](https://github.com/igraph/python-igraph/blob/main/src/igraph/community.py)

9. NetworkX label propagation module: [networkx.org/documentation/stable/_modules/networkx/algorithms/community/label_propagation.html](https://networkx.org/documentation/stable/_modules/networkx/algorithms/community/label_propagation.html)

10. NetworkX `asyn_lpa_communities` docs: [networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.asyn_lpa_communities.html](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.asyn_lpa_communities.html)

11. NetworkX `label_propagation_communities` docs: [networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.label_propagation_communities.html](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.label_propagation.label_propagation_communities.html)

12. rust-igraph `LpaVariant` enum: [totoro-jam.github.io/rust-igraph/rust_igraph/enum.LpaVariant.html](https://totoro-jam.github.io/rust-igraph/rust_igraph/enum.LpaVariant.html)

13. label-propagation Rust crate (SSL, not community detection): [docs.rs/label-propagation](https://docs.rs/label-propagation)

14. Jerrykl/LPA (Rust parallel LPA): [github.com/Jerrykl/LPA](https://github.com/Jerrykl/LPA)

15. Sobieczky, F. (survey on LPA variants, citation): Garza, S.E. and Schaeffer, S.E. (2019). "Community detection with the Label Propagation Algorithm: A survey." *Physica A: Statistical Mechanics and its Applications*, 534, 122058. DOI: [10.1016/j.physa.2019.122058](https://doi.org/10.1016/j.physa.2019.122058).

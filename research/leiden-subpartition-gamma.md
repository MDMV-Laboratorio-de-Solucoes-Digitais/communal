# Leiden Algorithm: Subpartition Guarantee and Gamma Parameter Behavior

**Research findings for CHK043, CHK004, CHK045**
**Source:** Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z

---

## 1. The Subpartition Guarantee (CHK043)

### 1.1 Problem Statement

The refinement phase in the Leiden algorithm produces a partition $\mathscr{P}_{\text{refined}}$ from the original partition $\mathscr{P}$ obtained in the local-moving phase. The paper states that $\mathscr{P}_{\text{refined}}$ is "a refinement of $\mathscr{P}$" — but this relationship is used declaratively without a formal definition in the main text.

### 1.2 Formal Definition of "Subpartition of the Original"

The standard meaning (consistent with the paper's intent and the mathematical structure of partitions) is:

$$\forall C' \in \mathscr{P}_{\text{refined}},\; \exists C \in \mathscr{P} : C' \subseteq C$$

In words: **every community in the refined partition is a subset of some community in the original partition.** Equivalently, the refined partition is *finer* than the original: $\mathscr{P}_{\text{refined}} \preceq \mathscr{P}$ in the partition lattice.

More formally, let $\mathcal{P}(V)$ denote the set of all partitions of vertex set $V$. For partitions $\pi, \sigma \in \mathcal{P}(V)$, we say $\pi$ is a **refinement** of $\sigma$ (written $\pi \preceq \sigma$) if and only if:

$$\forall B \in \pi,\; \exists A \in \sigma : B \subseteq A$$

The refinement guarantee of Leiden is: $\mathscr{P}_{\text{refined}} \preceq \mathscr{P}$.

### 1.3 The Mechanism that Enforces the Guarantee

The subpartition property is **structurally enforced by design** — not by post-hoc verification. The mechanism operates at two levels:

#### 1.3.1 Isolated Per-Community Processing (Algorithm-Level Guarantee)

From Algorithm A.2 (Appendix A of Traag et al. 2019):

```
P_refined ← SingletonPartition(G)
for C ∈ P do
    P_refined ← MergeNodesSubset(G, P_refined, C)
end for
```

**Key insight:** The `MergeNodesSubset` function is called *once per community* $C \in \mathscr{P}$, and operates exclusively on the nodes within $C$. Since:
- Each community $C$ is processed independently
- Nodes are only merged with other nodes *within the same original community*
- No cross-community merges ever occur during refinement

The resulting refined communities are necessarily subsets of a single original community.

**Source:** Traag et al. 2019, Algorithm A.2, lines 27–31 (Appendix A, p. 15).

#### 1.3.2 The `MergeNodesSubset` Function (Fine-Grained Guarantee)

From Algorithm A.2, the `MergeNodesSubset(G, P, S)` function:

1. **Input:** A subset $S \subseteq V$ (the nodes of a single community $C \in \mathscr{P}$)
2. **Initialization:** `P ← SingletonPartition(G)` — each node in $S$ starts in its own singleton community
3. **Well-connectedness filter (line 34):**

$$R = \left\{ v \;\middle|\; v \in S,\; E(v, S \setminus v) \geq \kappa_v \cdot \gamma (|S| - \kappa_v) \right\}$$

   Where:
   - $E(v, S \setminus v)$ = total edge weight from node $v$ to other nodes in $S$
   - $\kappa_v$ = degree (or node weight) of node $v$
   - $\gamma$ = resolution parameter
   - $|S|$ = total weight of subset $S$

   Only nodes in $R$ are eligible to be merged.

4. **Community eligibility filter (line 37):**

$$T = \left\{ C \;\middle|\; C \in P,\; C \subseteq S,\; E(C, S \setminus C) \geq \kappa_C \cdot \gamma (|S| - \kappa_C) \right\}$$

   Only well-connected communities (relative to the subset) are eligible targets.

5. **Random merge (line 38):** A community $C' \in T$ is selected with probability:

$$\Pr(C' = C) \propto \begin{cases} \exp\left(\frac{1}{\beta} \Delta_H(v \to C)\right) & \text{if } \Delta_H(v \to C) \geq 0 \\ 0 & \text{otherwise} \end{cases}$$   Where $\beta > 0$ is the randomness parameter (default 0.01) and $\Delta_H(v \to C)$ is the quality improvement from moving node $v$ to community $C$.

**Since merges only occur between nodes already within $S \subseteq C \in \mathscr{P}$, and the initial state has each node separate, all resulting merged communities are subsets of $S$, hence subsets of $C$.**

**Source:** Traag et al. 2019, Algorithm A.2, lines 33–43 (Appendix A, pp. 15–16).

### 1.4 Reference Implementation Evidence

#### 1.4.1 `libleidenalg` (C++)

**File:** `src/Optimiser.cpp`

The `optimise_partition` method (lines ~125–200) implements refinement via:

```cpp
if (this->refine_partition)
{
    // Create sub-partitions (one per community in collapsed graph)
    for (size_t layer = 0; layer < nb_layers; layer++)
        sub_collapsed_partitions[layer] = collapsed_partitions[layer]->create(collapsed_graphs[layer]);

    // Refine: movement constrained to within original communities
    if (this->refine_routine == Optimiser::MERGE_NODES)
        this->merge_nodes_constrained(sub_collapsed_partitions, ...);
```

The `merge_nodes_constrained` function (line ~530) enforces the constraint by:
1. Getting the constrained communities: `constrained_comms = constrained_partition->get_communities()`
2. Only considering communities within the same constrained community (original community):

```cpp
size_t v_constrained_comm = constrained_partition->membership(v);
for (size_t u : constrained_comms[v_constrained_comm])
{
    size_t u_comm = partitions[0]->membership(u);
    // ... only adds communities within the constrained community
}
```

**Source:** `vtraag/libleidenalg`, `src/Optimiser.cpp`, `merge_nodes_constrained()` method (lines ~530–650).

#### 1.4.2 `igraph` (C)

**File:** `src/community/leiden.c`

The refinement loop in `community_leiden()`:

```c
/* Refine each cluster */
nb_refined_clusters = 0;
for (c = 0; c < *nb_clusters; c++) {
    igraph_vector_int_t* cluster = igraph_vector_int_list_get_ptr(&clusters, c);
    IGRAPH_CHECK(leiden_merge_vertices(i_graph,
                 &edges_per_vertex,
                 i_edge_weights, ...,
                 cluster, i_membership, c,
                 resolution, beta,
                 &nb_refined_clusters, &refined_membership));
    igraph_vector_int_clear(cluster);
}
```

The `leiden_merge_vertices()` function processes each cluster independently. The constraint enforcement is visible in the neighbor iteration:

```c
if (u != v && VECTOR(*membership)[u] == cluster_subset) {
    // Only consider edges within the same original cluster
    VECTOR(external_edge_weight_per_cluster_in_subset)[i] += VECTOR(*edge_weights)[e];
}
```

Where `cluster_subset` is the original community index passed as parameter `c`.

**Source:** `igraph/igraph`, `src/community/leiden.c`, `leiden_merge_vertices()` function (lines ~150–280).

### 1.5 Mathematical Proof of the Guarantee

The subpartition property follows directly from the algorithm structure:

**Theorem (Subpartition Guarantee):** After refinement, $\mathscr{P}_{\text{refined}}$ satisfies $\forall C' \in \mathscr{P}_{\text{refined}} : \exists C \in \mathscr{P} \text{ s.t. } C' \subseteq C$.

**Proof sketch:**
1. Refinement initializes each node to its own singleton community
2. `MergeNodesSubset` is called independently for each $C \in \mathscr{P}$
3. Within `MergeNodesSubset(G, P, S)` where $S = C \in \mathscr{P}$:
   - Only nodes $v \in S$ are considered for merging (line 34)
   - Only communities $C' \subseteq S$ are eligible targets (line 37)
   - Therefore, any resulting community satisfies $C' \subseteq S = C \in \mathscr{P}$
4. Since each $C \in \mathscr{P}$ is processed independently and no cross-community operations occur, the guarantee holds for all refined communities.

---

## 2. The Gamma (γ) Parameter: Constraints and Behavioral Effects (CHK004/CHK045)

### 2.1 Formal Definition in Quality Functions

The resolution parameter $\gamma$ appears in two standard quality functions:

#### 2.1.1 Constant Potts Model (CPM)

$$H(G, \mathscr{P}) = \sum_{C \in \mathscr{P}} \left[ E(C, C) - \gamma \binom{n_C}{2} \right]$$

Where:
- $E(C, C)$ = total edge weight internal to community $C$
- $n_C$ = number of nodes in community $C$ (or total weight for weighted version)
- $\gamma$ = resolution parameter
- $\binom{n_C}{2} = \frac{n_C(n_C - 1)}{2}$ = number of possible edges

**Source:** Traag et al. 2019, Eq. (2), p. 2 (main text).

#### 2.1.2 Modularity (with resolution)

$$H(G, \mathscr{P}) = \frac{1}{2m} \sum_{C \in \mathscr{P}} \left[ E(C, C) - \gamma \frac{K_C^2}{2m} \right]$$

Where:
- $m$ = total number of edges (or total edge weight)
- $K_C$ = sum of degrees of nodes in community $C$
- $\gamma$ = resolution parameter

**Source:** Traag et al. 2019, Eq. (1), p. 2 (main text).

### 2.2 Valid Range for Gamma

#### 2.2.1 Mathematical Constraint

**From the paper (main text, p. 2):**

> "where $\gamma > 0$ is a resolution parameter"

The strict positivity constraint $\gamma > 0$ is stated explicitly.

**Behavioral boundary analysis:**
- **$\gamma \to 0^+$:** The quality function reduces to $H \approx \sum_C E(C, C)$, which counts only internal edges. Every node in its own community maximizes this trivially. However, since merges never *decrease* quality when $\gamma = 0$ (the penalty term vanishes), the algorithm tends to merge aggressively. In the limit, all nodes collapse into a single community (the partition $\{V\}$).
- **$\gamma \to \infty$:** The penalty term dominates. Any non-singleton community has a large negative contribution. The optimal partition is the singleton partition $\{\{v\} : v \in V\}$ — each node in its own community.

**Source:** Traag et al. 2019, p. 2, and Appendix E (Bounds on Optimality).

#### 2.2.2 Implementation-Level Validation

**In `igraph` (`src/community/leiden.c`):**

The `igraph_community_leiden_simple()` function does **NOT** explicitly validate that $\gamma > 0$. However, the behavior is implicitly constrained:

- For **Modularity objective**: `resolution /= igraph_vector_sum(&vertex_out_weights)` — the resolution is normalized by total edge weight, so the *effective* $\gamma$ for modularity is $\gamma_{\text{input}} / (2m)$.
- For **CPM objective**: No transformation — $\gamma$ is used directly.
- For **ER objective**: `resolution *= p` — scaled by graph density.

The lack of explicit $\gamma > 0$ validation means negative values would be accepted but produce degenerate/undefined behavior (e.g., encouraging nodes to be in the same community when they have no edges).

**In `libleidenalg` (`src/ResolutionParameterVertexPartition.h`):**

```cpp
class ResolutionParameterVertexPartition : public MutableVertexPartition {
  public:
    double resolution_parameter;
    // ...
};
```

No validation constraints are present in the header. The `resolution_parameter` is stored as a plain `double` and used directly in `diff_move()` and `quality()` calculations. No range checking is performed.

**In `leidenalg` (Python):**

The Python package wraps `libleidenalg` and similarly does not validate $\gamma > 0$ at the interface level.

### 2.3 Behavioral Effects on Community Count

#### 2.3.1 Monotonic Relationship

**From the paper (main text, p. 2):**

> "Higher resolutions lead to more communities, while lower resolutions lead to fewer communities."

This monotonic relationship holds for both Modularity and CPM.

**Mathematical justification:**
The quality contribution of a community $C$ is $E(C,C) - \gamma \cdot f(C)$ where $f(C)$ is the null model term. For a merge of two communities $A$ and $B$ into $C = A \cup B$ to be favorable:

$$\Delta_H(A \cup B \to C) = \underbrace{E(A,B)}_{\text{actual edges between}} - \gamma \cdot \underbrace{\Delta f(A,B)}_{\text{change in null model}} > 0$$

Rearranging:

$$E(A,B) > \gamma \cdot \Delta f(A,B)$$

Since $\Delta f(A,B) > 0$ (combining communities always increases the null model term), higher $\gamma$ requires more actual edges $E(A,B)$ to justify a merge. Therefore:
- **Higher $\gamma$** → fewer merges justified → **more, smaller communities**
- **Lower $\gamma$** → more merges justified → **fewer, larger communities**

**Source:** Traag et al. 2019, p. 2, and the discussion of resolution parameter interpretation in Reichardt & Bornholdt (2006).

#### 2.3.2 CPM Interpretation (Threshold Behavior)

**From the paper (main text, p. 2):**

> "The interpretation of the resolution parameter γ is quite straightforward. The parameter functions as a sort of threshold: communities should have a density of at least γ, while the density between communities should be lower than γ."

More precisely for CPM:
- Internal density of any community $C$ is at least $\gamma$: $\frac{E(C,C)}{\binom{n_C}{2}} \geq \gamma$
- External density between any two communities is less than $\gamma$

This threshold interpretation directly explains why higher $\gamma$ → more communities (stricter density requirement means fewer nodes qualify to be grouped together).

### 2.4 Gamma's Effect on the Refinement Phase

During refinement, $\gamma$ directly controls which subcommunities are formed:

**Well-connectedness condition (from Algorithm A.2, line 34):**

$$E(v, S \setminus v) \geq \gamma \cdot \kappa_v \cdot (|S| - \kappa_v)$$

- **Higher $\gamma$:** Stricter well-connectedness requirement → fewer nodes eligible for merging → more refined subcommunities
- **Lower $\gamma$:** Looser requirement → more nodes eligible → fewer, larger refined subcommunities

This is the mechanism by which $\gamma$ influences the granularity of the refinement phase specifically, beyond its effect in the local-moving phase.

### 2.5 Summary of Gamma Constraints

| Aspect | Value/Range | Source |
|--------|-------------|--------|
| **Mathematical constraint** | $\gamma > 0$ (strictly positive) | Traag et al. 2019, p. 2 |
| **Effect on community count** | Higher $\gamma$ → more communities | Traag et al. 2019, p. 2 |
| **CPM interpretation** | Minimum internal community density threshold | Traag et al. 2019, p. 2 |
| **Modularity scaling** | $\gamma$ is scaled by $1/(2m)$ internally | `igraph` source code |
| **Default value in experiments** | $\gamma = 1$ | Traag et al. 2019, experimental setup |
| **Validation in `libleidenalg`** | None (any `double` accepted) | `ResolutionParameterVertexPartition.h` |
| **Validation in `igraph`** | None explicit for $\gamma > 0$ | `igraph_community_leiden_simple()` |
| **Behavior as $\gamma \to 0^+$** | Trivial partition (one community) | Derived from quality function |
| **Behavior as $\gamma \to \infty$** | Singleton partition | Derived from quality function |

### 2.6 Recommended Validation Constraints

Based on this research, the following validation should be enforced in any implementation:

1. **Strict positivity:** $\gamma > 0$ (mathematical requirement for resolution parameter)
2. **Finite value:** $\gamma \in \mathbb{R}^+$ (no NaN, no infinity)
3. **Practical upper bound:** While mathematically any $\gamma > 0$ is valid, values much larger than $\max_{C} \frac{E(C,C)}{\binom{n_C|2}}$ will always yield singleton communities. No hard upper bound is needed, but documentation should note this behavioral limit.
4. **Practical lower bound:** Values approaching 0 will yield a single community. No hard lower bound other than $\gamma > 0$ is needed.

---

## 3. Mathematical Notation Summary

| Symbol | Definition | First Appearance |
|--------|------------|-----------------|
| $G = (V, E)$ | Graph with vertex set $V$ and edge set $E$ | Appendix A, p. 12 |
| $n = \|V\|$ | Number of nodes | Appendix A, p. 12 |
| $m = \|E\|$ | Number of edges | Appendix A, p. 12 |
| $\mathscr{P} = \{C_1, \ldots, C_r\}$ | Partition of $V$ into $r$ communities | Appendix A, p. 12 |
| $n_c$ | Number of nodes in community $c$ | Eq. (2), p. 2 |
| $K_c$ | Sum of degrees of nodes in community $c$ | Eq. (1), p. 2 |
| $e_c = E(C,C)$ | Total edge weight internal to community $c$ | Eq. (1), p. 2 |
| $\gamma > 0$ | Resolution parameter | Eq. (1), p. 2 |
| $\beta > 0$ | Randomness parameter in refinement | Algorithm A.2, line 38 |
| $\kappa_v$ | Node weight (degree for modularity, 1 for CPM) | Eq. (A1), Appendix A |
| $E(C, D)$ | Total edge weight between communities $C$ and $D$ | Eq. (A4), Appendix A |
| $\Delta_H(v \to C)$ | Quality change from moving node $v$ to community $C$ | Appendix A, p. 12 |
| $H(G, \mathscr{P})$ | Quality function value | Appendix A, p. 12 |
| $\mathscr{P}_{\text{refined}}$ | Refined partition (subpartition of $\mathscr{P}$) | Algorithm A.2, p. 4 |
| $\text{flat}(S)$ | Flattening operation for a set of sets | Eq. (A2), Appendix A |

---

## 4. Source Citations

### Primary Source
- Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z

### Reference Implementations
- **libleidenalg (C++):** https://github.com/vtraag/libleidenalg — `src/Optimiser.cpp` (refinement via `merge_nodes_constrained`), `include/ResolutionParameterVertexPartition.h` (gamma storage without validation)
- **igraph (C):** https://github.com/igraph/igraph — `src/community/leiden.c` (`leiden_merge_vertices()` for refinement, `igraph_community_leiden_simple()` for parameter handling)
- **leidenalg (Python):** https://github.com/vtraag/leidenalg — Python wrapper around libleidenalg

### Supporting References
- Reichardt, J., & Bornholdt, S. (2006). "Statistical mechanics of community detection." *Phys. Rev. E*, 74, 016110. (Resolution parameter introduction)
- Traag, V.A., Van Dooren, P., & Nesterov, Y. (2011). "Narrow scope for resolution-limit-free community detection." *Phys. Rev. E*, 84, 016114. (CPM quality function)
- Fortunato, S., & Barthélemy, M. (2007). "Resolution Limit in Community Detection." *PNAS*, 104, 36. (Modularity resolution limit)

---

## 5. Resolution of Checklist Items

### CHK043: "Subpartition of the original" not formally defined

**RESOLVED:** The formal definition is $\forall C' \in \mathscr{P}_{\text{refined}},\; \exists C \in \mathscr{P} : C' \subseteq C$, equivalently $\mathscr{P}_{\text{refined}} \preceq \mathscr{P}$. This is structurally enforced by processing each original community independently in `MergeNodesSubset`. The guarantee is not stated as a separate theorem in the paper because it follows trivially from the algorithm design (line-by-line independence of community processing).

### CHK004/CHK045: Gamma parameter constraints not stated

**RESOLVED:** The paper states $\gamma > 0$ (p. 2, after Eq. 1). Behavioral effects ("Higher resolutions lead to more communities") are documented. However, **no explicit validation is performed in any reference implementation** (`libleidenalg`, `igraph`, or `leidenalg`). The resolution parameter is stored and used as a raw `double` with no range checking. Implementations should add explicit $\gamma > 0$ validation.

---

*Document generated: 2026-01-14*

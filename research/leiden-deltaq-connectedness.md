# Leiden Algorithm: ΔQ Formula and Connectedness Preservation

## Research Summary

This document details the exact mathematical formulation of the modularity gain (ΔQ) for node moves in the Leiden algorithm's local moving phase, and the precise algorithmic definition of "connectedness preservation" during that phase.

**Primary Source:** Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. https://doi.org/10.1038/s41598-019-41695-z

**Reference Implementations:**
- libleidenalg C++: https://github.com/vtraag/libleidenalg
- leidenalg Python: https://github.com/vtraag/leidenalg
- igraph C: https://github.com/igraph/igraph/blob/main/src/community/leiden.c

---

## 1. Quality Functions

The Leiden algorithm can optimize multiple quality functions. The two most common are:

### 1.1 Constant Potts Model (CPM)

$$H(G, P) = \sum_{c \in P} \left[ E(c,c) - \gamma \binom{n_c}{2} \right]$$

where:
- $G = (V, E)$ — the graph (undirected or directed, weighted or unweighted)
- $P$ — a partition of $V$ into communities
- $E(c, c)$ — total weight of edges with both endpoints in community $c$ (for undirected; for directed, this counts edges from $c$ to $c$)
- $n_c$ — number of nodes in community $c$ (or sum of node sizes for weighted)
- $\gamma > 0$ — resolution parameter

In the igraph C implementation (Eq. A4 in paper), this is rewritten using the identity $\binom{n_c}{2} = \frac{1}{2}(n_c^2 - n_c)$:

$$H(G, P) = \sum_{c \in P} \left[ E(c,c) - \gamma \frac{n_c(n_c - 1)}{2} \right]$$

For **directed graphs**: $E(c,c)$ counts directed edges within $c$, and the possible edges term is $n_c^2$ (not $n_c(n_c-1)/2$).

### 1.2 Modularity

$$Q(G, P) = \frac{1}{2m} \sum_{c \in P} \left[ E(c,c) - \frac{K_c^2}{2m} \right]$$

where:
- $m$ — total number of edges (or total weight of edges) in the graph
- $K_c$ — sum of degrees of nodes in community $c$

In the libleidenalg source, modularity is reformulated (Eq. A5 in paper) as:

$$H(G, P) = \sum_{c \in P} \left[ E(c,c) - \frac{1}{2m} K_c^{\text{out}} K_c^{\text{in}} \right]$$

where $K_c^{\text{out}}$ is the sum of out-degrees and $K_c^{\text{in}}$ is the sum of in-degrees. For undirected graphs, $K_c^{\text{out}} = K_c^{\text{in}} = K_c$, recovering the standard form.

---

## 2. ΔQ Formula for Node Moves (CPM)

### 2.1 Mathematical Definition

The change in quality from moving node $v$ to community $C$ is:

$$\Delta Q(v \to C) = Q(P(v \to C)) - Q(P)$$

where $P(v \to C)$ is the partition obtained by moving $v$ to community $C$.

For the **CPM quality function**, expanding the telescoping sum gives:

$$\Delta Q(v \to C) = \underbrace{\left[w(v, C) - \gamma \cdot n_v \cdot n_C\right]}_{\text{gain from joining } C} - \underbrace{\left[w(v, \sigma_v) - w(v,v) - \gamma \cdot n_v \cdot (n_{\sigma_v} - n_v)\right]}_{\text{loss from leaving } \sigma_v}$$

where:
- $w(v, C)$ — total weight of edges from node $v$ to nodes in community $C$
- $w(v, \sigma_v)$ — total weight of edges from node $v$ to nodes in its current community $\sigma_v$
- $w(v,v)$ — weight of self-loop on node $v$ (may be zero)
- $n_v$ — size of node $v$ (1 for unweighted, node weight for weighted)
- $n_C$ — size of target community $C$ (before move)
- $n_{\sigma_v}$ — size of source community $\sigma_v$ (before move)
- $\gamma$ — resolution parameter

**Simplified for undirected graphs** (where $w(v,C) = w(C,v)$):

$$\Delta Q(v \to C) = \left[w(v, C) + w(v,v) - \gamma \cdot n_v(n_C + n_v)\right] - \left[w(v, \sigma_v) - w(v,v) - \gamma \cdot n_v(n_{\sigma_v} - n_v)\right]$$

**Further simplified** (grouping terms):

$$\Delta Q(v \to C) = w(v,C) - w(v,\sigma_v) + 2w(v,v) - \gamma \cdot n_v(n_C + n_v - n_{\sigma_v} + n_v)$$

$$\Delta Q(v \to C) = w(v,C) - w(v,\sigma_v) + 2w(v,v) - \gamma \cdot n_v(n_C - n_{\sigma_v} + 2n_v)$$

### 2.2 Algorithmic Form (libleidenalg C++ source)

File: `src/CPMVertexPartition.cpp`, function `diff_move(size_t v, size_t new_comm)`:

```
diff_move(v, new_comm):
    old_comm = membership(v)
    if new_comm == old_comm: return 0

    w_to_old   = weight_to_comm(v, old_comm)     // edges v → old_comm
    w_to_new   = weight_to_comm(v, new_comm)     // edges v → new_comm
    w_from_old = weight_from_comm(v, old_comm)   // edges old_comm → v
    w_from_new = weight_from_comm(v, new_comm)   // edges new_comm → v
    nsize      = node_size(v)                    // n_v
    csize_old  = csize(old_comm)                 // n_{old} BEFORE move
    csize_new  = csize(new_comm)                 // n_{new} BEFORE move
    self_weight = node_self_weight(v)            // w(v,v)

    // "possible edge difference" = change in the n_c²/2 term
    if correct_self_loops:
        possible_edge_difference_old = nsize * (2*csize_old - nsize)
    else:
        possible_edge_difference_old = nsize * (2*csize_old - nsize - 1)

    if correct_self_loops:
        possible_edge_difference_new = nsize * (2*csize_new + nsize)
    else:
        possible_edge_difference_new = nsize * (2*csize_new + nsize - 1)

    diff_old = w_to_old + w_from_old - self_weight - γ * possible_edge_difference_old
    diff_new = w_to_new + w_from_new + self_weight - γ * possible_edge_difference_new

    return diff_new - diff_old
```

**Variable definitions for CPM diff_move:**

| Variable | Definition |
|----------|-----------|
| `v` | Node being moved |
| `new_comm` | Target community index |
| `old_comm` | Current community of `v` before the move |
| `w_to_comm(v, c)` | Sum of edge weights from `v` to nodes in community `c` |
| `weight_from_comm(v, c)` | Sum of edge weights from nodes in community `c` to `v` (equals `weight_to_comm` for undirected) |
| `node_size(v)` | Size of node `v` (1.0 for unweighted graphs) |
| `csize(c)` | Total size of community `c` (sum of node sizes), computed BEFORE the move |
| `node_self_weight(v)` | Weight of self-loops on `v` |
| `γ` | `resolution_parameter` |
| `correct_self_loops` | Whether the graph has self-loops that should be treated as internal edges |

**Key detail:** `csize_old` and `csize_new` are the community sizes BEFORE the move. The `possible_edge_difference` terms encode the change in $\binom{n_c}{2}$:
- Removing `v` from old community: $\binom{n_{\text{old}}}{2} \to \binom{n_{\text{old}} - n_v}{2}$, difference = $n_v(n_{\text{old}} - n_v) + \binom{n_v}{2}$
- Adding `v` to new community: $\binom{n_{\text{new}}}{2} \to \binom{n_{\text{new}} + n_v}{2}$, difference = $n_v \cdot n_{\text{new}} + \binom{n_v}{2}$

The code uses `2*csize - nsize` and `2*csize + nsize` rather than the binomial form because the factor of 2 is deferred to the quality function's $(2 - \text{is\_directed})$ multiplier.

### 2.3 Igraph C Implementation

File: `src/community/leiden.c`, function `leiden_fastmove_vertices()`:

For **undirected graphs** (CPM/modularity with resolution):

```c
// For each neighboring cluster c (including the node's own cluster):
diff = edge_weights_per_cluster[c]
     - vertex_out_weights[v] * cluster_out_weights[c] * resolution;

// For directed graphs:
diff = edge_weights_per_cluster[c]
     - (vertex_out_weights[v] * cluster_in_weights[c]
        + vertex_in_weights[v] * cluster_out_weights[c]) * resolution;
```

**Igraph variable definitions:**

| Variable | Definition |
|----------|-----------|
| `edge_weights_per_cluster[c]` | Total weight of edges from node `v` to community $c$ (accumulated during neighbor scan) |
| `vertex_out_weights[v]` | Total out-weight (degree/strength) of node $v$ |
| `vertex_in_weights[v]` | Total in-weight (degree/strength) of node $v$ |
| `cluster_out_weights[c]` | Sum of out-weights of all nodes currently in cluster $c$ (BEFORE the move, with $v$ removed) |
| `cluster_in_weights[c]` | Sum of in-weights of all nodes currently in cluster $c$ (BEFORE the move, with $v$ removed) |
| `resolution` | Resolution parameter $\gamma$ |

**Critical algorithm detail in igraph:** The node $v$ is **removed from its current cluster before** computing diffs for candidate clusters. This means `cluster_out_weights` and `cluster_in_weights` reflect the state AFTER removing $v$ from its old community but BEFORE adding it to a new one.

### 2.4 Modularity ΔQ Formula (libleidenalg)

File: `src/ModularityVertexPartition.cpp`, function `diff_move(size_t v, size_t new_comm)`:

```
diff_move(v, new_comm):
    old_comm = membership(v)
    if new_comm == old_comm: return 0
    if total_weight == 0: return 0

    w_to_old    = weight_to_comm(v, old_comm)
    w_from_old  = weight_from_comm(v, old_comm)
    w_to_new    = weight_to_comm(v, new_comm)
    w_from_new  = weight_from_comm(v, new_comm)
    k_out       = strength(v, OUT)          // out-degree of v
    k_in        = strength(v, IN)           // in-degree of v
    self_weight = node_self_weight(v)
    K_out_old   = total_weight_from_comm(old_comm)  // sum of out-degrees in old_comm
    K_in_old    = total_weight_to_comm(old_comm)    // sum of in-degrees in old_comm
    K_out_new   = total_weight_from_comm(new_comm) + k_out  // sum of out-degrees in new_comm + v's contribution
    K_in_new    = total_weight_to_comm(new_comm) + k_in     // sum of in-degrees in new_comm + v's contribution
    total_weight = graph.total_weight() * (2 - is_directed)  // 2m for undirected, m for directed

    diff_old = (w_to_old - k_out * K_in_old / total_weight)
             + (w_from_old - k_in * K_out_old / total_weight)

    diff_new = (w_to_new + self_weight - k_out * K_in_new / total_weight)
             + (w_from_new + self_weight - k_in * K_out_new / total_weight)

    diff = diff_new - diff_old

    if directed:
        m = graph.total_weight()
    else:
        m = 2 * graph.total_weight()

    return diff / m
```

**Modularity-specific variable definitions:**

| Variable | Definition |
|----------|-----------|
| `k_out` / `k_in` | Out-degree / in-degree (strength) of node $v$ |
| `K_out_old` / `K_in_old` | Sum of out/in-degrees of nodes in old community (BEFORE move) |
| `K_out_new` / `K_in_new` | Sum of out/in-degrees of nodes in new community PLUS $v$'s contribution (i.e., after hypothetical move) |
| `total_weight` | $2m$ for undirected, $m$ for directed |

**For undirected graphs** ($k_{\text{out}} = k_{\text{in}} = k_v$, $K_{\text{out}} = K_{\text{in}} = K_c$, $\text{total\_weight} = 2m$):

$$\Delta Q(v \to C) = \frac{1}{m} \left[ (w(v,C) + w(v,v)) - w(v,\sigma_v) - \frac{k_v (K_C^{\text{after}} - K_{\sigma_v}^{\text{before}})}{2m} \right]$$

where $K_C^{\text{after}} = K_C + k_v$ (new community after adding $v$) and $K_{\sigma_v}^{\text{before}} = K_{\sigma_v}$ (old community before removing $v$).

Simplifying:

$$\Delta Q(v \to C) = \frac{1}{2m} \left[ 2w(v,C) + 2w(v,v) - 2w(v,\sigma_v) - \frac{k_v(K_C + k_v)}{m} + \frac{k_v K_{\sigma_v}}{m} \right]$$

$$\Delta Q(v \to C) = \frac{1}{2m} \left[ 2(w(v,C) - w(v,\sigma_v) + w(v,v)) - \frac{k_v}{m}(K_C + k_v - K_{\sigma_v}) \right]$$

---

## 3. Connectedness Preservation Condition

### 3.1 The Problem

The Louvain algorithm may produce disconnected communities because:
1. A node acting as a "bridge" between two components of its community can be moved away
2. The remaining nodes are still "locally optimal" (no individual node wants to move)
3. The Louvain algorithm has no mechanism to split or repair such communities

The Leiden algorithm addresses this through a **refinement phase** that runs between local moving and aggregation.

### 3.2 Formal Definition (from Traag et al. 2019, Appendix D)

**Definition 6 ($\gamma$-connectivity):** A set of nodes $S \subseteq C$ is $\gamma$-connected if:
- $|S| = 1$, OR
- $S$ can be partitioned into two non-empty sets $R$ and $T$ such that:
  - $E(R, T) \geq \gamma \cdot k_R \cdot k_T$ (where $E(R,T)$ is the total edge weight between $R$ and $T$, and $k_R, k_T$ are the total sizes/weights of $R$ and $T$)
  - $R$ is $\gamma$-connected
  - $T$ is $\gamma$-connected

A community $C$ is $\gamma$-connected if $S = C$ is $\gamma$-connected.

**Note:** $\gamma$-connectivity is a *stronger* condition than ordinary graph connectivity. Ordinary connectivity only requires $E(R,T) > 0$, while $\gamma$-connectivity requires the connection to be proportionally dense relative to the sizes of the subsets.

### 3.3 Connectedness Check During Refinement (MergeNodesSubset)

In Algorithm A.2, lines 34 and 37 (paper) / `merge_nodes_constrained` (code):

```
MergeNodesSubset(Graph G, Partition P_refined, Subset S):
    R = { v | v ∈ S,  E(v, S \ {v}) ≥ γ · k_v · (k_S - k_v) }
    for v ∈ R (in random order):
        if v in singleton community:
            T = { C | C ∈ P_refined, C ⊆ S,  E(C, S \ C) ≥ γ · k_C · (k_S - k_C) }
            Choose random C' ∈ T with probability ∝ exp(ΔQ(v→C')/β) if ΔQ ≥ 0
            Move v to C'
```

**The connectedness condition has TWO parts:**

**Pre-condition for a node to be eligible (line 34):**
$$E(v, S \setminus \{v\}) \geq \gamma \cdot k_v \cdot (k_S - k_v)$$

A node $v$ can only be considered for merging if it is sufficiently well-connected to the rest of the subset $S$.

**Pre-condition for a target community to be eligible (line 37):**
$$E(C, S \setminus C) \geq \gamma \cdot k_C \cdot (k_S - k_C)$$

A community $C$ can only receive a merge if it is sufficiently well-connected to the rest of the subset $S$.

**Variable definitions for connectedness checks:**

| Symbol | Definition |
|--------|-----------|
| $S$ | The subset being refined (a community from the local-moving partition $P$) |
| $v$ | A node in $S$ being considered for merging |
| $E(v, S \setminus \{v\})$ | Total weight of edges from $v$ to other nodes in $S$ |
| $k_v$ | Size/weight of node $v$ |
| $k_S$ | Total size/weight of subset $S$ (sum of $k_u$ for all $u \in S$) |
| $E(C, S \setminus C)$ | Total weight of edges between community $C$ and the rest of $S$ |
| $k_C$ | Total size/weight of community $C$ |
| $\gamma$ | Resolution parameter |
| $\beta$ | Randomness parameter (temperature), controls exploration vs. exploitation |

### 3.4 Guarantees and Proof Structure

**Theorem 5 (from paper):** After each iteration of the Leiden algorithm, all communities are $\gamma$-connected.

The proof relies on the refinement phase (MergeNodesSubset). When building a merged community $S$ by adding nodes one at a time ($S_1 \subset S_2 \subset \dots \subset S_k = S$), each addition satisfies:

$$E(u_{i+1}, S_i) \geq \gamma \cdot k_{u_{i+1}} \cdot k_{S_i}$$

This recursive construction, combined with the inductive hypothesis that each added node is itself $\gamma$-connected (which holds trivially for singletons), proves that the final merged community is $\gamma$-connected.

**Lemma 2 (from paper):** During the MoveNodesFast procedure, for any node $v$ at any time:
- $v$ is in the queue $Q$ (scheduled for re-evaluation), OR
- $|C_v| = 1$ (v is a singleton community), OR
- $E(v, C_v \setminus \{v\}) > 0$ (v has at least one edge to its community)

This lemma ensures that if a node becomes disconnected from its community, it will be re-queued for evaluation. However, note that this only guarantees *ordinary* connectivity, not $\gamma$-connectivity. The $\gamma$-connectivity guarantee comes from the refinement phase.

### 3.5 How Reference Implementations Handle This

#### libleidenalg (`merge_nodes_constrained` in `src/Optimiser.cpp`):

The constrained merge phase:
1. Starts each node in its own singleton community
2. Iterates over nodes in random order
3. For each node $v$ that is still a singleton:
   - Considers merging with neighboring communities $C$ within the same constrained partition
   - The connectedness check is implicit: a community $C$ is only considered if `diff_move(v, C) >= 0`, which for CPM means $w(v,C) - \gamma \cdot k_v \cdot k_C + \dots \geq 0$
   - Additionally, `get_neigh_comms(v, IGRAPH_ALL, constrained_partition->membership())` restricts candidates to communities of neighbors that are in the same constrained community

The actual connectedness threshold is enforced by the **quality function itself**: the resolution parameter $\gamma$ penalizes merging when the edge weight is insufficient relative to community sizes.

#### igraph (`leiden_merge_vertices` in `src/community/leiden.c`):

```c
// The key connectedness check before considering a merge:
if (!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster) &&
    (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >=
     vertex_weight_prod * resolution)) {
```

Where `vertex_weight_prod` is:
- Undirected: `cluster_out_weights[current] * (total_out_weight - cluster_out_weights[current])`
- Directed: `cluster_out_weights[current] * (total_in_weight - cluster_in_weights[current]) + cluster_in_weights[current] * (total_out_weight - cluster_out_weights[current])`

This encodes the condition that the community must be sufficiently connected to the rest of the subset.

Then for each candidate cluster:
```c
if (VECTOR(external_edge_weight_per_cluster_in_subset)[c] >= vertex_weight_prod * resolution) {
    diff = edge_weights_per_cluster[c]
         - vertex_out_weights[v] * cluster_out_weights[c] * resolution;
    ...
}
```

The merge is only considered if both:
1. The current cluster is well-connected to the rest of the subset (pre-condition)
2. The candidate cluster is well-connected to the rest of the subset (pre-condition)
3. The move doesn't decrease quality (diff >= 0)

---

## 4. Algorithmic Measurement Steps

### 4.1 Measuring ΔQ for a Node Move

**Input:** Graph $G$, partition $P$, node $v$, target community $C$
**Output:** Quality change $\Delta Q$

**Steps (CPM, undirected):**
1. Let $\sigma_v$ = current community of $v$
2. If $C = \sigma_v$, return 0
3. Compute $w(v, C)$ = sum of edge weights from $v$ to nodes in $C$
4. Compute $w(v, \sigma_v)$ = sum of edge weights from $v$ to nodes in $\sigma_v$ (excluding self-loops)
5. Let $w(v, v)$ = self-loop weight of $v$
6. Let $n_v$ = size of $v$, $n_C$ = size of $C$, $n_{\sigma_v}$ = size of $\sigma_v$ (all before move)
7. Compute loss from leaving $\sigma_v$:
   $$\text{loss} = w(v, \sigma_v) - w(v,v) - \gamma \cdot n_v(n_{\sigma_v} - n_v + \mathbb{1}_{\text{self-loops}} \cdot n_v)$$
8. Compute gain from joining $C$:
   $$\text{gain} = w(v, C) + w(v,v) - \gamma \cdot n_v(n_C + n_v - \mathbb{1}_{\text{self-loops}} \cdot n_v)$$
9. Return $\text{gain} - \text{loss}$

### 4.2 Checking Connectedness Preservation

**Pre-conditions for refinement merge:**
1. $v$ is in a singleton community in $P_{\text{refined}}$
2. $v \in S$ (the community being refined from $P$)
3. $E(v, S \setminus \{v\}) \geq \gamma \cdot k_v \cdot (k_S - k_v)$

**Pre-conditions for target community:**
4. $C \in P_{\text{refined}}$ and $C \subseteq S$
5. $E(C, S \setminus C) \geq \gamma \cdot k_C \cdot (k_S - k_C)$

**Post-conditions after merge:**
6. $\Delta Q(v \to C) \geq 0$ (quality doesn't decrease)
7. The merged community $C \cup \{v\}$ is still $\gamma$-connected (guaranteed by conditions 3-5)

**Rollback semantics:**
- There is no explicit rollback in the Leiden algorithm
- Moves are only made if they don't decrease quality (greedy within refinement)
- The queue-based fast local move ensures that any disconnection triggers re-evaluation
- If a node is moved and later becomes disconnected, it will be re-queued (Lemma 2)

---

## 5. Key Differences: Louvain vs. Leiden

| Aspect | Louvain | Leiden |
|--------|---------|--------|
| **Disconnected communities** | Possible (up to 16% of communities) | Guaranteed connected (γ-connected) |
| **Local moving** | Re-scans all nodes until convergence | Queue-based fast local move (only re-evaluates affected nodes) |
| **Refinement phase** | None | Merges within communities with connectedness check |
| **Quality function** | Any | Any (CPM, Modularity, RB Configuration, RBER, Significance, Surprise) |
| **Convergence** | Single pass to stable point | Iterative with asymptotic guarantees |

---

## 6. Source Citations

### Paper Sections
- **Eq. (1)-(2):** Quality function definitions (CPM and Modularity)
- **Eq. (A4)-(A5):** CPM and Modularity formulations used in proofs
- **Section III.A:** Guarantees overview (Table I)
- **Definition 6:** $\gamma$-connectivity
- **Theorem 5:** $\gamma$-connectivity guaranteed after each iteration
- **Lemma 2:** Queue invariant during MoveNodesFast
- **Algorithm A.2:** Full Leiden pseudo-code with refinement phase
- **Lines 34, 37:** Connectedness preconditions for refinement

### Source Code Files
- `libleidenalg/src/CPMVertexPartition.cpp` — CPM diff_move implementation
- `libleidenalg/src/ModularityVertexPartition.cpp` — Modularity diff_move implementation
- `libleidenalg/src/RBERVertexPartition.cpp` — RBER diff_move (same pattern as CPM)
- `libleidenalg/src/Optimiser.cpp` — move_nodes, merge_nodes, move_nodes_constrained, merge_nodes_constrained
- `libleidenalg/include/MutableVertexPartition.h` — Base class with weight_to_comm, weight_from_comm, csize
- `igraph/src/community/leiden.c` — leiden_fastmove_vertices, leiden_merge_vertices, leiden_move_vertices

---

## 7. Summary for Implementation

For an implementation satisfying FR-001 (local moving phase):

1. **ΔQ computation** must use the CPM or Modularity formula from Section 2 above, with all variables computed relative to the **current state** (before the move).

2. **The fast local move procedure** uses a queue:
   - Initialize queue with all nodes (random order)
   - Pop node $v$, compute best move via $\Delta Q$
   - If move improves quality, move $v$ and enqueue all neighbors not in $v$'s new community
   - Continue until queue is empty

3. **Connectedness is NOT explicitly checked during local moving.** Instead:
   - The refinement phase (after local moving) guarantees $\gamma$-connected communities
   - The queue invariant (Lemma 2) ensures disconnected nodes are re-evaluated
   - The overall algorithm converges to a partition where all communities are well-connected

4. **The refinement phase** uses the connectedness checks $E(X, S \setminus X) \geq \gamma \cdot k_X \cdot (k_S - k_X)$ for both nodes and communities as preconditions for merging.

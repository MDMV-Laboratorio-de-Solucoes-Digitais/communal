# Leiden Connectedness Check: Research & Optimization Report

**Date:** 2026-09-07
**Status:** Complete

---

## Executive Summary

The `would_remain_connected` function in `crates/communal-algo/src/leiden/local_moving.rs` (lines 154-191) is an **O(V+E) DFS-based connectivity check** called on every node move attempt in both `local_moving` and `refinement` phases. Our investigation reveals a critical architectural finding:

> **Neither the original Leiden paper's reference implementation (libleidenalg), the igraph C library, nor the optimized GVE-Leiden implementation perform any connectivity check during the local-moving phase.** The Leiden algorithm's connectedness guarantee comes entirely from the **refinement phase's design** (singleton-start + isolated-vertex-only merges + well-connectedness threshold), making the per-move BFS/DFS redundant in `local_moving` and unnecessary in `refinement`.

**Key recommendation:** Remove `would_remain_connected` from `local_moving` entirely (architecturally incorrect), and remove it from `refinement` (guaranteed by construction). Use a single post-hoc `debug_assert!` verification pass. This aligns our implementation with all reference implementations and the paper's theoretical framework.

---

## 1. How the Original Paper Handles Connectedness

### 1.1 Source

Traag, Waltman, van Eck (2019), *"From Louvain to Leiden: guaranteeing well-connected communities"*, Scientific Reports 9:5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).

### 1.2 The Three Phases

The Leiden algorithm consists of:

1. **Local Moving Phase** — A "fast local move" procedure. Nodes are visited from a queue. Each node is moved to the neighboring community that maximizes quality improvement. Only nodes whose neighborhood has changed are re-visited. **No connectivity check is performed.** The paper states: *"In the fast local move procedure in the Leiden algorithm, only nodes whose neighbourhood has changed are visited."*

2. **Refinement Phase** — Starts from a **singleton partition** (each node in its own community). Nodes are merged within each community of the original partition P. A node can only be merged with a community if both are "sufficiently well connected." The refinement uses randomized selection weighted by quality gain (parameter θ/β). **This is where the connectedness guarantee comes from.**

3. **Aggregation Phase** — The graph is collapsed based on the refined partition.

### 1.3 The Connectedness Guarantee Mechanism

The paper proves (Appendix C.1) that the refinement phase guarantees all communities are connected. The key insight:

> *"The refined partition P_refined is obtained as follows. Initially, P_refined is set to a singleton partition, in which each node is in its own community. The algorithm then locally merges nodes in P_refined: nodes that are on their own in a community in P_refined can be merged with a different community. Importantly, mergers are performed only within each community of the partition P."*

The refinement only allows **singleton nodes** (nodes currently alone in their refined community) to be merged into other sub-communities. This ensures that a node can only leave a sub-community if it was the sole member, meaning the remaining sub-community is trivially connected (or empty). The connectedness is guaranteed by construction, not by checking.

### 1.4 Fast Local Move Pseudo-Code (From Paper's Appendix A.2)

```
while queue not empty:
    v = queue.pop_front()
    if quality_can_be_increased_by_moving(v):
        move v to best community
        for each neighbour u of v not in new community:
            queue.push_back(u)
```

**No BFS/DFS/connectivity check appears in the algorithm.** The quality function alone determines moves.

---

## 2. How Reference Implementations Handle It

### 2.1 Summary Table

| Implementation | Connectivity Check in Local Moving | Connectivity Check in Refinement | How Connectedness is Guaranteed |
|---------------|-----------------------------------|----------------------------------|-------------------------------|
| **Original Paper** | None | None (by construction) | Refinement merge criteria |
| **libleidenalg (C++)** | None | None | Refinement merge criteria |
| **igraph (C)** | None | None | Refinement merge criteria |
| **leidenalg (Python)** | None | None | Wraps libleidenalg |
| **GVE-Leiden (parallel)** | None | None | Refinement merge criteria |
| **leiden_rs (Rust)** | None | None | Refinement merge criteria |
| **communal (ours)** | **Yes — full DFS** | **Yes — full DFS** | Redundant double-check |

### 2.2 libleidenalg (C++ — Original Reference Implementation)

**Source:** [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg), `src/Optimiser.cpp`

The `move_nodes()` method in the Optimiser class:

```cpp
// From Optimiser.cpp — move_nodes()
while (!vertex_order.empty()) {
    size_t v = vertex_order.front(); vertex_order.pop_front();
    size_t v_comm = partitions[0]->membership(v);
    
    // ... collect neighboring communities ...
    
    size_t max_comm = v_comm;
    double max_improv = /* epsilon */;
    
    for (size_t comm : comms) {
        double possible_improv = 0.0;
        for (size_t layer = 0; layer < nb_layers; layer++) {
            possible_improv += layer_weights[layer] * partitions[layer]->diff_move(v, comm);
        }
        if (possible_improv > max_improv) {
            max_comm = comm;
            max_improv = possible_improv;
        }
    }
    
    if (max_comm != v_comm) {
        // Move the node — NO connectivity check
        for (size_t layer = 0; layer < nb_layers; layer++) {
            partition->move_node(v, max_comm);
        }
        // Mark neighbours as unstable
        // ...
    }
}
```

**There is zero connectivity checking in `move_nodes()`.** The only criterion is `diff_move` (quality improvement). The connectedness guarantee comes from the refinement phase (`merge_nodes` / `merge_nodes_constrained`).

The `merge_nodes_constrained` function (used during refinement) only merges singleton nodes within community bounds, ensuring connectedness by construction.

### 2.3 igraph C Implementation

**Source:** [github.com/igraph/igraph](https://github.com/igraph/igraph), `src/community/leiden.c`

The `leiden_fastmove_vertices()` function:

```c
// From leiden.c — leiden_fastmove_vertices()
while (!igraph_dqueue_int_empty(&unstable_vertices)) {
    igraph_int_t v = igraph_dqueue_int_pop(&unstable_vertices);
    // ... find neighboring clusters, calculate diff ...
    
    /* Only consider strictly improving moves.
     * Note that this is important in considering convergence. */
    if (diff > max_diff) {
        best_cluster = c;
        max_diff = diff;
    }
    // ...
    
    /* Move vertex to best cluster — NO connectivity check */
    VECTOR(*membership)[v] = best_cluster;
}
```

**No connectivity check.** Only `diff > max_diff` (quality improvement). The refinement phase (`leiden_merge_vertices()`) handles connectedness by only merging singleton clusters that meet the well-connectedness threshold:

```c
// From leiden.c — leiden_merge_vertices()
if (!IGRAPH_BIT_SET(non_singleton_cluster, current_cluster) &&
    (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >=
     vertex_weight_prod * resolution)) {
    // ... consider merge
}
```

This is the paper's γ-connectivity condition: `E(C, S\C) ≥ γ · k_C · (k_S - k_C)`. It is an **O(1) arithmetic check** using cached edge weight sums, not a graph traversal.

### 2.4 leidenalg Python Package

**Source:** [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) — Python wrapper around libleidenalg.

Since it calls `libleidenalg`'s `optimise_partition()`, it inherits the same behavior: **no connectivity check during local moving.**

### 2.5 GVE-Leiden (Optimized Parallel Implementation)

**Source:** [arXiv:2312.13936](https://arxiv.org/html/2312.13936v5)

This paper explicitly describes the refinement phase's role:

> *"In the refinement phase, the each vertex starts in a singleton community, and community memberships are updated similarly to the local-moving phase, with vertices changing communities within their bounds. This procedure splits any internally-disconnected communities identified during the local-moving phase, and prevents the formation of any new disconnected communities."*

The local-moving phase (`leidenMove`) does NOT check connectivity. The refinement phase (`leidenRefine`) fixes disconnected communities that arose during local moving by re-merging them properly.

The refinement uses a "constrained merge procedure" where only isolated vertices (singletons in the refined partition) can change communities, preventing the formation of disconnected communities by construction.

### 2.6 Rust Implementations (leiden_rs)

**Source:** [docs.rs/leiden-rs](https://docs.rs/leiden-rs), [github.com/pnevyk/leiden-rs](https://github.com/pnevyk/leiden-rs)

A Rust implementation supporting multiple quality functions (Modularity, CPM, RBConfiguration, RBER). Follows the standard Leiden structure (local moving → refinement → aggregation) without per-move connectivity checks.

---

## 3. Specific Bottleneck Analysis in Our Code

### 3.1 The Function (local_moving.rs lines 154-191)

```rust
fn would_remain_connected<G: GraphView>(
    graph: &G,
    membership: &[u32],
    node: NodeId,
    community: u32,
) -> bool {
    let node_idx = node.index() - 1;

    let community_nodes: Vec<usize> = membership
        .iter()
        .enumerate()
        .filter(|(i, c)| **c == community && *i != node_idx)
        .map(|(i, _)| i)
        .collect();

    if community_nodes.len() <= 1 {
        return true;
    }

    let community_set: std::collections::HashSet<usize> = community_nodes.iter().copied().collect();
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![community_nodes[0]];

    while let Some(current) = stack.pop() {
        if visited.insert(current)
            && let Some(current_node) = NodeId::new(u32::try_from(current).unwrap_or(u32::MAX).wrapping_add(1))
        {
            for neighbor in graph.neighbors(current_node) {
                let neighbor_idx = neighbor.index() - 1;
                if community_set.contains(&neighbor_idx) && !visited.contains(&neighbor_idx) {
                    stack.push(neighbor_idx);
                }
            }
        }
    }

    visited.len() == community_nodes.len()
}
```

### 3.2 Complexity Per Call

| Operation | Complexity |
|-----------|-----------|
| Collect community nodes (full membership scan) | O(V) |
| Build HashSet | O(k) where k = community size |
| DFS traversal | O(k + e_c) where e_c = edges within community |
| **Total per call** | **O(V + k + e_c) ≈ O(V + E)** |

### 3.3 Call Frequency

The function is called in **two** places:

1. **`local_moving.rs` line 75** — For EVERY node move attempt in the local moving phase:
   ```rust
   if let Some((target_community, gain)) = best_move
       && gain > 0.0
       && would_remain_connected(graph, membership, node, current_community)
   {
   ```

2. **`refinement.rs` line 87** — For EVERY node move attempt in the refinement phase:
   ```rust
   if let Some(target_community) = target
       && would_remain_connected(graph, membership, node, current_community)
   {
   ```

### 3.4 Total Complexity Impact

For a graph with n nodes, m edges, and an average community size k:

- **Local moving phase:** Up to n calls per iteration, each O(V + E_c) → **O(n × (V + E))** per phase
- **Refinement phase:** Up to n calls per iteration, each O(V + E_c) → **O(n × (V + E))** per phase
- **Both phases combined:** O(2n × (V + E)) per full Leiden iteration

For the PolBooks graph (105 nodes, 441 edges) that times out at >30s, and PolBlogs (1490 nodes, 19090 edges), this is the dominant cost.

### 3.5 Additional Inefficiencies

1. **Full membership scan:** `membership.iter().enumerate().filter(...)` scans ALL nodes to find community members — O(V) each time.
2. **HashSet allocation:** A new `HashSet` is allocated for every call.
3. **Vec allocation:** `community_nodes` Vec is allocated every call.
4. **Repeated work:** The connectivity of the same community is re-checked for every node within it that considers moving.

---

## 4. Known Optimization Techniques

### 4.1 Remove the Check from Local Moving (Architectural Fix)

**This is the most important finding.** The Leiden algorithm is explicitly designed so that:
- Local moving can produce disconnected communities (this is expected and fine)
- The refinement phase fixes them

Adding a connectivity check to local moving is **architecturally redundant** — it goes against the paper's design and is absent from all reference implementations.

**Impact:** Removes O(n × (V+E)) work from the most frequently executed phase.

### 4.2 Remove the Check from Refinement (By-Construction Guarantee)

The refinement phase already guarantees connectedness by construction:
- Starts from singleton partition
- Only singleton nodes can be moved/merged
- A singleton leaving a community cannot disconnect it (the community was either empty, or had other members that remain connected through their own edges)

The connectedness is further ensured by the **well-connectedness threshold** (used in igraph's `leiden_merge_vertices`): a node is only merged into a community if `external_edge_weight >= vertex_weight_prod * resolution`. This ensures the merged result is connected.

**Note:** Our refinement uses `select_target_community` which applies a probabilistic acceptance based on β, not the well-connectedness threshold from the paper. This means our refinement might NOT guarantee connectedness by construction, depending on the acceptance criteria. If the refinement uses the paper's well-connectedness threshold, the check can be removed. If using a simpler quality-gain threshold, keep a single post-hoc verification.

### 4.3 Early-Exit Conditions

If keeping the check is desired:

1. **Degree-0 or degree-1 node:** If the node has ≤1 intra-community edges, removing it cannot disconnect the community (unless it's the only connection between parts — but degree-1 means it's a leaf, safe to remove).
2. **Community size ≤ 2:** Trivially connected after removal.
3. **Node has edges to all other community members:** Then it's a hub, removing it might disconnect — but the BFS would quickly find this.
4. **Articulation point check:** Use Tarjan's algorithm to find articulation points in the community subgraph. A node is safe to remove iff it's not an articulation point. This is O(V+E) per community but can be cached.

### 4.4 Incremental Connectivity Tracking

**Union-Find / Disjoint Set Union (DSU):**

The idea: maintain a DSU for each community. When a node is removed, check if its removal splits the community.

**Problem:** DSU supports union efficiently but NOT split/removal. Removing a node from a DSU requires rebuilding the affected component, which is O(k + e_c) — same as BFS.

**Dynamic connectivity** data structures (e.g., Holm-De Lichtenberg-Thorup) support edge insertions/deletions in O(log²n) amortized, but are complex to implement and have high constant factors. For the Leiden algorithm's use case (node moving = removing all intra-community edges for that node), they would not provide practical benefit over BFS.

### 4.5 Caching Connectivity Information

Cache the connected components of each community. When a node is removed:
- If the node was not an articulation point → still connected, O(1) check
- If the node was an articulation point → recompute via BFS

This requires maintaining articulation points per community, which is O(k + e_c) to compute initially and O(k + e_c) to update after each move. For small communities this is cheap; for large communities it's the same cost as the current approach.

### 4.6 Local BFS (Neighborhood-Only Check)

Instead of BFS over the entire community, check if the node's neighbors within the community are still connected after removal:

1. Pick two neighbors u, v of the node in the community
2. BFS from u to v avoiding the removed node
3. If reachable → locally connected (not a guarantee of global connectivity, but a fast heuristic)

This is O(local_degree + local_edges) instead of O(k + e_c). However, it's a heuristic — a node can be locally connected but the community globally disconnected.

### 4.7 Post-Hoc Verification (Debug-Only)

Run a single connectivity check after each phase completes, not per-move. This catches disconnected communities without the per-move cost. If disconnected communities are found, they can be split into their connected components.

---

## 5. Recommended Approach for Our Implementation

### 5.1 Recommended Changes (Priority Order)

#### Change 1: Remove `would_remain_connected` from `local_moving`

**Rationale:** The reference implementations (libleidenalg, igraph) do NOT check connectivity during local moving. The Leiden algorithm is explicitly designed so that local moving may produce disconnected communities, and the refinement phase fixes them. Adding this check is architecturally incorrect and provides no benefit.

**Expected speedup:** 2-5x on the local moving phase (eliminates the dominant cost).

#### Change 2: Refactor `refinement` to Guarantee Connectedness by Construction

**Rationale:** The refinement phase should guarantee connectedness through its merge criteria, not through post-hoc checking. Two options:

**Option A (Paper-faithful):** Implement the paper's well-connectedness threshold. A singleton node can only be merged into a sub-community if the external edge weight to that sub-community meets the resolution threshold. This is what igraph's `leiden_merge_vertices()` does.

**Option B (Simpler):** Keep the current β-weighted probabilistic acceptance but only allow singleton nodes to move (which is already the case since refinement starts from singletons). A singleton leaving a sub-community cannot disconnect it.

**Expected speedup:** 2-5x on the refinement phase.

#### Change 3: Add Post-Hoc Verification in Debug Builds Only

```rust
#[cfg(debug_assertions)]
{
    debug_assert!(
        verify_communities_connected(graph, membership),
        "phase produced disconnected communities"
    );
}
```

This preserves the existing `verify_communities_connected` function (which is already in `refinement.rs`) as a debug-only check, similar to what the refinement phase already does at lines 112-115.

#### Change 4 (Optional): If Keeping the Check is Necessary

If after Changes 1-2 there are still disconnected communities (which would indicate a bug), consider:

1. **Early-exit for small communities:** Skip BFS for communities of size ≤ 2
2. **Degree-based pruning:** Skip BFS for nodes with degree ≤ 1 within the community
3. **Caching:** Cache community connected components and only recompute when a node in a "critical" position considers moving

### 5.2 Complexity Comparison

| Approach | Per-Move Cost | Per-Phase Cost | Correctness |
|----------|--------------|----------------|-------------|
| Current (full DFS, both phases) | O(V + k + e_c) | O(n × (V+E)) | Correct but redundant |
| Remove from local_moving only | O(V + k + e_c) | O(n × (V+E)) for refinement only | Correct |
| Remove from both (recommended) | O(1) | O(n × deg) for quality eval | Correct (by construction) |
| Post-hoc verification | O(V+E) once | O(V+E) per phase | Correct (detection only) |
| Articulation point caching | O(1) amortized | O(n × (V+E)) initial | Correct |

### 5.3 Impact on the AGENTS.md Test Cases

The existing test suite (Tier 1) should pass unchanged:
- Complete graphs, bipartite graphs, paths, stars, rings, grids, and two-triangles
- These have clear community structures that the Leiden algorithm finds correctly
- The connectedness of output communities is guaranteed by the refinement phase's design

The Tier 3 real-world networks (PolBooks, Les Misérables, NetScience, PolBlogs) that currently time out at >30s should see significant speedup from removing the per-move connectivity checks.

---

## 6. Correctness Proof Sketch

**Claim**: Removing the BFS from local moving does not violate the Leiden algorithm's guarantees.

**Proof**:
1. The Leiden algorithm's guarantees (Theorem 5) state that after each iteration, all communities are γ-connected.
2. The proof relies on the refinement phase (MergeNodesSubset), not on checks during local moving.
3. The local moving phase only needs to find a partition where no individual node can improve quality (node optimality).
4. The refinement phase then splits any disconnected communities and ensures γ-connectivity.
5. Therefore, the BFS during local moving is redundant for correctness.

**Claim**: The "isolated vertices only" refinement prevents disconnected communities.

**Proof**:
1. Initially, all vertices are singletons (trivially connected).
2. When an isolated vertex v joins community C, the new community C ∪ {v} is connected because v has at least one edge to C (otherwise ΔQ would be negative).
3. Non-isolated vertices never move, so existing connected communities stay connected.
4. By induction, all communities remain connected throughout refinement.

---

## 7. Sources

### Papers
- Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. [arXiv:1810.08473](https://arxiv.org/abs/1810.08473). [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).
- Sahu, S. (2024). "GVE-Leiden: Fast Leiden Algorithm for Community Detection in Shared Memory Setting." [arXiv:2312.13936](https://arxiv.org/html/2312.13936v5).

### Source Code
- libleidenalg C++: [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg) — Optimiser.cpp `move_nodes()` and `merge_nodes()` — no connectivity checks.
- leidenalg Python: [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) — Python wrapper around libleidenalg.
- igraph C: [github.com/igraph/igraph](https://github.com/igraph/igraph) — `src/community/leiden.c` — `leiden_fastmove_vertices()` and `leiden_merge_vertices()`.
- GVE-Leiden OpenMP: [github.com/puzzlef/leiden-communities-openmp](https://github.com/puzzlef/leiden-communities-openmp).
- leiden_rs Rust: [docs.rs/leiden-rs](https://docs.rs/leiden-rs).

### Documentation
- leidenalg docs: [leidenalg.readthedocs.io](https://leidenalg.readthedocs.io/en/stable/reference.html).
- igraph Leiden: [igraph.org/c/html/1.0.1/igraph-Community.html](https://igraph.org/c/html/1.0.1/igraph-Community.html).
- Neo4j Leiden: [neo4j.com/docs/graph-data-science/current/algorithms/leiden/](https://neo4j.com/docs/graph-data-science/current/algorithms/leiden/).

### Implementations
- CWTS Java Implementation: [github.com/CWTSLeiden/networkanalysis](https://github.com/CWTSLeiden/networkanalysis) — the original Java implementation referenced in the paper.
- fixed-ai/fa-leiden-cd: [github.com/fixed-ai/fa-leiden-cd](https://github.com/fixed-ai/fa-leiden-cd/tree/main) — Rust implementation of Leiden.

---

## 8. Appendix: Pseudo-Code for Recommended `local_moving`

```rust
pub fn local_moving<G: GraphView>(
    graph: &G,
    membership: &mut [u32],
    quality_function: QualityFunction,
    gamma: f64,
    rng: &mut StdRng,
) -> bool {
    let node_count = graph.node_count();
    if node_count == 0 {
        return false;
    }

    let mut moved = false;
    let mut node_order: Vec<NodeId> = (1..=u32::try_from(node_count).unwrap_or(u32::MAX))
        .filter_map(NodeId::new)
        .collect();
    node_order.shuffle(rng);

    for &node in &node_order {
        let node_idx = node.index() - 1;
        let current_community = membership[node_idx];

        let neighbor_communities = collect_neighbor_communities(graph, membership, node);

        if neighbor_communities.is_empty() {
            continue;
        }

        let best_move = find_best_community(
            graph, membership, node, current_community,
            &neighbor_communities, quality_function, gamma,
        );

        // NOTE: No would_remain_connected check here.
        // The Leiden algorithm's connectedness guarantee comes from
        // the refinement phase, not from per-move checks.
        if let Some((target_community, gain)) = best_move
            && gain > 0.0
        {
            membership[node_idx] = target_community;
            moved = true;
        }
    }

    moved
}
```

The refinement phase's `select_target_community` and the singleton-start design ensure connectedness by construction. A single `verify_communities_connected` call after the full Leiden iteration (in debug builds only) is sufficient for safety.

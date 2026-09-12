# Leiden Output Equivalence Testing: Identical vs. Quality-Equivalent

**Date:** 2026-09-09
**Status:** Complete

---

## Executive Summary

When optimizing the Leiden algorithm's connectedness check (or any internal subroutine), the question arises: **should the optimized version produce identical community assignments to the original, or is equivalent quality sufficient?**

The answer from the research community and reference implementations is unambiguous:

> **Equivalent quality is the standard. Identical output is neither expected nor testable for Leiden.**

The Leiden algorithm is **inherently stochastic** — it uses randomized node ordering during the local-moving phase. Different runs with different random seeds produce different (but equally valid) partitions. No reference implementation tests for exact partition identity across runs. Instead, they test for:

1. **Quality equivalence** — modularity/CPM scores match within tolerance
2. **Structural invariants** — community sizes, number of communities, aggregate properties
3. **Optimality conditions** — no single-node move can improve the quality (node optimality)
4. **Deterministic reproducibility with fixed seed** — same seed → same output (bit-exact)

For optimization regression testing, the correct approach is: **fix the seed, assert quality equivalence (within floating-point tolerance), and assert structural invariants (community count, sizes). Do NOT assert exact membership vectors unless the seed is fixed and the implementation is bit-for-bit identical.**

---

## 1. The Inherent Stochasticity of Leiden

### 1.1 Source

Traag, Waltman, van Eck (2019), *"From Louvain to Leiden: guaranteeing well-connected communities"*, Scientific Reports 9:5233. [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).

### 1.2 Why Leiden is Non-Deterministic

The Leiden algorithm's local-moving phase processes nodes in **random order**. The paper's pseudo-code (Appendix A.2) specifies:

```
for each node v in random order:
    if quality_can_be_increased_by_moving(v):
        move v to best community
```

This random ordering means:
- Different runs visit nodes in different sequences
- Tie-breaking when multiple communities yield equal quality improvement is order-dependent
- The final partition depends on the random seed

### 1.3 Community Consensus

From [StackOverflow: "Leiden Clustering results are not always the same"](https://stackoverflow.com/questions/73155343/leiden-clustering-results-are-not-always-the-same-given-the-same-resolution-para) (answered by Szabolcs, 2022):

> *"This is expected, as the algorithm is randomized, as discussed [in the original paper](https://doi.org/10.1038/s41598-019-41695-z). It is not a unique feature of this community detection algorithm. Several others, if not most, use stochastic methods to try to optimize some quality measure such as modularity."*

From [Memgraph documentation](https://memgraph.com/docs/advanced-algorithms/available-algorithms/leiden_community_detection):

> *"This implementation of Leiden is non-deterministic, meaning it can generate different communities in subsequent runs."*

From [Articsledge guide](https://www.articsledge.com/post/leiden-algorithm):

> *"Different random seeds can produce slightly different results, though stable networks usually show minimal variation. Setting random_state..."*

---

## 2. How Reference Implementations Handle Testing

### 2.1 leidenalg (Python) — The Reference Test Suite

**Source:** [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg), `tests/test_Optimiser.py` and `tests/test_VertexPartition.py`

The leidenalg test suite reveals the community's testing philosophy:

#### Test Strategy A: Deterministic Graphs + Exact Assertions

For **deterministic graphs** (complete graphs, trees, famous karate club), tests assert **exact community sizes**:

```python
# From test_Optimiser.py — test_move_nodes()
G = ig.Graph.Full(100)
partition = leidenalg.CPMVertexPartition(G, resolution_parameter=0.5)
self.optimiser.move_nodes(partition, consider_comms=leidenalg.ALL_NEIGH_COMMS)
self.assertListEqual(
    partition.sizes(), [100],
    msg="CPMVertexPartition(resolution_parameter=0.5) of complete graph after move nodes incorrect.")
```

This works because on a complete graph, the optimal partition is trivially a single community — no randomness affects the outcome.

#### Test Strategy B: Quality Equivalence (assertAlmostEqual)

For operations involving floating-point quality calculations, tests use tolerance-based assertions:

```python
# From test_VertexPartition.py — test_move_nodes()
self.assertAlmostEqual(
    q2 - q1,
    diff,
    places=5,
    msg="Difference in quality ({0}) not equal to calculated difference ({1})".format(
    q2 - q1, diff))
```

The `places=5` tolerance acknowledges floating-point non-determinism.

#### Test Strategy C: Structural Invariants

Tests verify aggregate properties rather than exact membership:

```python
# From test_Optimiser.py — test_optimiser()
G = reduce(ig.Graph.disjoint_union, (ig.Graph.Tree(10, 3, mode=ig.TREE_UNDIRECTED) for i in range(10)))
partition = leidenalg.CPMVertexPartition(G, resolution_parameter=0)
self.optimiser.optimise_partition(partition)
self.assertListEqual(
    partition.sizes(), 10*[10],
    msg="After optimising partition failed to find different components...")
```

This asserts that 10 disjoint trees produce 10 communities of size 10 — a structural invariant independent of node ordering.

#### Test Strategy D: Optimality Conditions (Not Output Identity)

The most rigorous test verifies that no further improvement is possible:

```python
# From test_Optimiser.py — test_diff_move_node_optimality()
G = ig.Graph.Erdos_Renyi(100, p=5./100, directed=False, loops=False)
partition = leidenalg.CPMVertexPartition(G, resolution_parameter=0.1)
while 0 < self.optimiser.move_nodes(partition, consider_comms=leidenalg.ALL_NEIGH_COMMS):
    pass
for v in G.vs:
    neigh_comms = set(partition.membership[u.index] for u in v.neighbors())
    for c in neigh_comms:
        self.assertLessEqual(
            partition.diff_move(v.index, c), 1e-10,
            msg="Was able to move a node to a better community, violating node optimality.")
```

This test uses a **random graph** (Erdos-Renyi) and does NOT assert a specific partition. Instead, it asserts the **optimality invariant**: no single-node move can improve quality beyond a 1e-10 tolerance.

#### Test Strategy E: Seed Parameter for Reproducibility

The `find_partition()` function accepts a `seed` parameter:

```python
# From leidenalg reference documentation
leidenalg.find_partition(graph, partition_type, initial_membership=None, weights=None,
                         n_iterations=2, max_comm_size=0, seed=None, **kwargs)
# seed (int) – Seed for the random number generator. By default uses a random seed
# if nothing is specified.
```

The `Optimiser` class also has a `set_seed()` method. This enables deterministic reproduction when needed.

**Key insight:** The leidenalg tests NEVER assert exact membership on random/stochastic graphs. They either:
1. Use deterministic graphs where output is seed-independent
2. Assert quality within tolerance
3. Assert structural invariants (sizes, counts)
4. Assert optimality conditions

### 2.2 igraph (C Implementation)

**Source:** [igraph.org/c](https://igraph.org/c/html/latest/igraph-Community.html#igraph_community_leiden), [github.com/igraph/igraph](https://github.com/igraph/igraph)

#### Random Number Generation

igraph has a full RNG framework (`igraph_rng`). The Leiden implementation uses `igraph_rng` for node ordering randomization. From the [igraph discourse](https://igraph.discourse.group/t/set-up-seed-for-community-leiden/2193):

> *"You can simply set a seed using random.seed() directly before the community_leiden() call."*

#### Testing Approach

igraph's test suite (in `tests/`) for community detection focuses on:
- Modularity score comparison against known values
- Membership vector properties (correct length, valid range)
- Edge cases (empty graphs, singletons, disconnected components)

The igraph documentation for `cluster_leiden` explicitly documents the `seed` parameter for reproducibility, acknowledging the algorithm's stochastic nature.

### 2.3 leiden-rs (Rust — fa-leiden-cd)

**Source:** [github.com/fixed-ai/fa-leiden-cd](https://github.com/fixed-ai/fa-leiden-cd), `src/lib.rs`

The leiden-rs test suite (visible in the source) demonstrates the quality-equivalence approach:

```rust
// From src/lib.rs — test_simplest()
let hierarchy = g.leiden(Some(100), &mut optimizer);
// ... collect assignments ...
assert!(assignments.borrow().values().collect::<HashSet<_>>().len() == 3);
```

This asserts the **number of communities** (3), NOT the exact partition assignment. The test creates three disjoint triangles and expects three communities — a structural invariant.

The `test_example` test runs the algorithm but has no assertions at all — it only prints the result, serving as a smoke test.

### 2.4 leidenAlg (R Package)

**Source:** [CRAN: leidenAlg](https://cran.r-project.org/package=leidenAlg)

The R package explicitly handles non-determinism through replication:

> *"This function performs Leiden algorithm nrep times and returns the result from the run with the maximum quality. Since Leiden algorithm has stochastic process,..."*

This is the **quality-maximization** approach: run multiple times, pick the best quality. It acknowledges that different runs produce different partitions and the "best" one is the one with highest quality.

---

## 3. Standard Practices in Community Detection Literature

### 3.1 Quality Score Equivalence Testing

The community detection literature consistently uses quality metrics (modularity, CPM, NMI, ARI) as the ground truth for comparison, not partition identity.

From [Fortunato & Hric (2016), "Community Detection in Networks: A User Guide"](https://doi.org/10.1016/j.physrep.2016.09.002):

> *"The quality of a partition is assessed by the value of the quality function. Different partitions of the same graph are compared based on their quality scores."*

### 3.2 Normalized Mutual Information (NMI) / Adjusted Rand Index (ARI)

When comparing partitions (e.g., against ground truth or between implementations), the standard approach is to use comparative metrics:

- **Normalized Mutual Information (NMI)** — measures information overlap between partitions
- **Adjusted Rand Index (ARI)** — measures agreement corrected for chance
- **Variation of Information (VI)** — measures distance between partitions

igraph provides `igraph_compare_communities()` with all these methods. A partition with NMI=1.0 to the reference is "equivalent"; ARI=1.0 means "identical up to label permutation."

### 3.3 The "Best of N Runs" Pattern

For stochastic algorithms, the standard validation pattern is:

1. Run N times with different seeds
2. Select the run with the highest quality score
3. Compare that quality score against the expected/reference quality

This is what `leidenAlg::leidenAlg()` does with `n_iter` parameter, and what `scanpy.tl.leiden` does with `n_iterations`.

---

## 4. Recommendations for Testing the Connectedness Optimization

### 4.1 What to Assert

| Assertion | Rationale | Tolerance |
|-----------|-----------|-----------|
| **Quality score (Q)** | Primary correctness criterion | `assertAlmostEqual(places=5)` or `|Q_opt - Q_orig| < 1e-6` |
| **Number of communities** | Structural invariant | Exact equality |
| **Community sizes (sorted)** | Structural invariant | Exact equality (as a multiset) |
| **Node optimality** | No single-node move improves Q | `diff_move <= 1e-10` for all nodes |
| **Connectedness** | All communities are connected | Binary (must hold) |

### 4.2 What NOT to Assert

| Anti-Pattern | Why |
|--------------|-----|
| **Exact membership vector match** | Fails due to random node ordering |
| **Exact membership across different seeds** | Fundamentally impossible by design |
| **Exact membership after optimization** | Optimization may change tie-breaking order |
| **Bit-exact quality across platforms** | Floating-point non-determinism |

### 4.3 Recommended Test Design

```rust
// Pseudocode for optimization regression test

#[test]
fn test_connectedness_optimization_preserves_quality() {
    let graph = load_reference_graph(); // e.g., Karate Club
    let seed = 42; // Fixed seed for reproducibility
    
    // Run original (unoptimized) version
    let result_original = leiden_original(&graph, seed);
    
    // Run optimized version
    let result_optimized = leiden_optimized(&graph, seed);
    
    // 1. Quality equivalence (primary assertion)
    assert!(
        (result_original.quality - result_optimized.quality).abs() < 1e-6,
        "Quality diverged: orig={}, opt={}",
        result_original.quality, result_optimized.quality
    );
    
    // 2. Structural invariants
    assert_eq!(
        result_original.community_count,
        result_optimized.community_count,
        "Community count diverged"
    );
    
    // 3. Connectedness guarantee (must hold for both)
    assert!(all_communities_connected(&graph, &result_original));
    assert!(all_communities_connected(&graph, &result_optimized));
    
    // 4. Optimality condition (no improving move exists)
    assert!(no_improving_move(&graph, &result_optimized, 1e-10));
}

#[test]
fn test_connectedness_optimization_quality_equivalence_multi_seed() {
    let graph = load_reference_graph();
    
    for seed in 0..20 {
        let orig = leiden_original(&graph, seed);
        let opt = leiden_optimized(&graph, seed);
        
        // With same seed, quality should match exactly (or within FP tolerance)
        assert!(
            (orig.quality - opt.quality).abs() < 1e-10,
            "Seed {}: quality mismatch {} vs {}", seed, orig.quality, opt.quality
        );
    }
}
```

### 4.4 Handling the Seed

The critical insight for testing an optimization:

- **Same seed + same algorithm → bit-exact same output** (if the optimization is purely computational, not algorithmic)
- **Different seeds → different but equally valid outputs**
- **Optimization should not change the algorithm's logic**, only its efficiency

If the connectedness optimization changes the algorithm's behavior (e.g., skipping a check that affects which moves are considered), then even with the same seed, the output may differ. In that case:

1. **Assert quality is equivalent or better** (optimization should not degrade quality)
2. **Assert structural invariants hold** (community count, connectedness)
3. **Assert optimality conditions** (no improving move exists)
4. **Do NOT assert exact membership match** across algorithmic changes

### 4.5 The leidenalg Precedent

The leidenalg test suite's approach to the `test_diff_move_node_optimality` test is the gold standard for stochastic algorithm testing:

```python
# This is the pattern to follow:
# 1. Run to convergence
# 2. Assert optimality condition (no improving move)
# 3. Use tolerance for floating-point comparisons
# 4. Do NOT assert specific partition identity

for v in G.vs:
    neigh_comms = set(partition.membership[u.index] for u in v.neighbors())
    for c in neigh_comms:
        self.assertLessEqual(
            partition.diff_move(v.index, c), 1e-10,
            msg="Was able to move a node to a better community, violating node optimality.")
```

---

## 5. Summary: Decision Framework

```
Is the optimization purely computational (same algorithm, faster)?
├── YES → Same seed should produce bit-exact output
│         → Assert: exact membership match (with fixed seed)
│         → Assert: quality match (within FP tolerance)
│         → This is a "performance optimization"
│
└── NO → Does the optimization change which moves are considered?
         ├── YES → Different output is expected and acceptable
         │         → Assert: quality equivalence (within tolerance)
         │         → Assert: structural invariants (count, sizes)
         │         → Assert: optimality conditions
         │         → Assert: connectedness guarantee
         │         → This is an "algorithmic optimization"
         │
         └── NO → The optimization is neutral to the algorithm
                   → Assert: same as YES case (bit-exact with fixed seed)
```

### For the Connectedness Check Optimization

The connectedness check optimization (removing the per-move DFS from `local_moving` and `refinement`) is an **algorithmic change** — it changes which moves are considered. Therefore:

> **Equivalent quality is the correct assertion standard. Identical output should NOT be expected.**

The optimization should be validated by:
1. Quality score equivalence (within floating-point tolerance)
2. Community count and size distribution matching
3. All communities being connected (the correctness guarantee)
4. Node optimality (no single-node move improves quality)
5. Multiple seeds tested (not just one)

---

## 6. References

1. Traag, Waltman, van Eck (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9:5233. [DOI:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z). [arXiv:1810.08473](https://arxiv.org/abs/1810.08473).

2. leidenalg Python test suite. [github.com/vtraag/leidenalg/blob/main/tests/test_Optimiser.py](https://github.com/vtraag/leidenalg/blob/main/tests/test_Optimiser.py) and [test_VertexPartition.py](https://github.com/vtraag/leidenalg/blob/main/tests/test_VertexPartition.py).

3. leidenalg documentation — `find_partition()` seed parameter. [leidenalg.readthedocs.io/en/stable/reference.html](https://leidenalg.readthedocs.io/en/stable/reference.html).

4. igraph C library — Leiden community detection. [igraph.org/c/html/latest/igraph-Community.html](https://igraph.org/c/html/latest/igraph-Community.html#igraph_community_leiden).

5. igraph discourse — "set up seed for community_leiden." [igraph.discourse.group/t/set-up-seed-for-community-leiden/2193](https://igraph.discourse.group/t/set-up-seed-for-community-leiden/2193).

6. StackOverflow — "Leiden Clustering results are not always the same." [stackoverflow.com/questions/73155343](https://stackoverflow.com/questions/73155343/leiden-clustering-results-are-not-always-the-same-given-the-same-resolution-para).

7. Memgraph — "leiden_community_detection." [memgraph.com/docs](https://memgraph.com/docs/advanced-algorithms/available-algorithms/leiden_community_detection).

8. leiden-rs (fa-leiden-cd) Rust implementation. [github.com/fixed-ai/fa-leiden-cd](https://github.com/fixed-ai/fa-leiden-cd), `src/lib.rs` lines 350-450 (test module).

9. leidenAlg R package — `nrep` parameter for stochastic replication. [CRAN: leidenAlg](https://cran.r-project.org/package=leidenAlg).

10. Fortunato & Hric (2016). "Community Detection in Networks: A User Guide." *Physics Reports* 659:1-44. [DOI:10.1016/j.physrep.2016.09.002](https://doi.org/10.1016/j.physrep.2016.09.002).

11. GVE-Leiden — "Fast Leiden Algorithm for Community Detection in Shared Memory Setting." [arXiv:2312.13936](https://arxiv.org/html/2312.13936v5).

12. scanpy — `tl.leiden()` with `n_iterations` parameter. [scanpy.readthedocs.io](https://scanpy.readthedocs.io/en/stable/generated/scanpy.tl.leiden.html).

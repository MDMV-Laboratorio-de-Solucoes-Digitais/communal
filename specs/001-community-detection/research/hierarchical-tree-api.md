# Hierarchical Community Tree API Design — Research Notes

## Overview

This document surveys how existing community detection libraries expose hierarchical
community structure, focusing on method signatures for accessing resolution levels,
return types, and common API patterns.

---

## 1. `leidenalg` (Python) — Reference Leiden Implementation

**Source:** https://github.com/vtraag/leidenalg — https://leidenalg.readthedocs.io/

### Core API

```python
# Single-shot: returns ONE partition (NOT hierarchical by default)
partition = la.find_partition(
    graph,                           # igraph Graph
    la.CPMVertexPartition,           # partition type
    resolution_parameter=0.1,        # kwargs forwarded to partition constructor
    n_iterations=2,
    max_comm_size=0,
)
# Returns: CPMVertexPartition (subclass of MutableVertexPartition)
# Access membership: partition.membership  -> list[int]
# Access quality:     partition.quality()   -> float
```

### Resolution Profile (threshold-based access)

```python
optimiser = la.Optimiser()
profile = optimiser.resolution_profile(
    G,                               # igraph Graph
    la.CPMVertexPartition,           # partition type
    resolution_range=(0, 1),         # scan range
)
# Returns: list[VertexPartition]
# profile[i] is optimal for resolution_parameter in
#   [profile[i].resolution_parameter, profile[i+1].resolution_parameter]
```

**Key insight:** The resolution profile returns partitions only at *change points*
— values of γ where the optimal partition actually changes. Between change points,
the partition is stable. This is an efficient threshold-based access pattern.

Source: https://leidenalg.readthedocs.io/en/stable/advanced.html#resolution-profile

### Partition Types (all subclass `MutableVertexPartition`)

| Class | Resolution param | Notes |
|-------|-----------------|-------|
| `ModularityVertexPartition` | No | Classic Newman-Girvan modularity |
| `CPMVertexPartition` | Yes (γ) | Constant Potts Model |
| `RBConfigurationVertexPartition` | Yes (γ) | Reichardt-Bornholdt configuration null |
| `RBERVertexPartition` | Yes (γ) | Reichardt-Bornholdt Erdős-Rényi null |
| `SignificanceVertexPartition` | No | Statistical significance |
| `SurpriseVertexPartition` | No | Statistical surprise |

### Temporal / Multiplex

```python
# Temporal community detection across time slices
partition = la.find_partition_temporal(
    graph, la.CPMVertexPartition,
    resolution_parameter=0.1,
)
# Returns: list[VertexPartition] — one per time slice
```

### Return Type Hierarchy

```
MutableVertexPartition (base)
  ├── membership: list[int]         # community ID per node
  ├── quality(): float
  ├── diff_move(v, comm): float     # quality delta for moving node v to comm
  ├── move_node(v, comm): None
  ├── aggregate_partition(): VertexPartition
  └── from_coarse_partition(coarse, membership=None): None
```

**Source:** https://deepwiki.com/vtraag/leidenalg/5-api-reference — https://github.com/vtraag/leidenalg/blob/main/src/leidenalg/__init__.py

---

## 2. `igraph` (C library + Python bindings)

**Source:** https://igraph.org/c/html/1.0.1/igraph-Community.html — https://python.igraph.org/

### Hierarchical Algorithms Return `VertexDendrogram`

The hierarchical community detection functions in igraph return a **dendrogram**
object that encodes the full merge history:

```python
# Fast greedy (agglomerative)
dendrogram = g.community_fastgreedy(weights=None)
# Returns: VertexDendrogram

# Walktrap
dendrogram = g.community_walktrap(weights=None, steps=4)
# Returns: VertexDendrogram

# Edge betweenness (divisive)
clustering = g.community_edge_betweenness(directed=False, weights=None)
# Returns: VertexDendrogram (initially cut at max modularity)
```

### `VertexDendrogram` — Index-based Access

```python
# Cut dendrogram to get a flat clustering at desired number of clusters
clustering = dendrogram.as_clustering(n=5)
# Returns: VertexClustering

# Let algorithm pick optimal cut (max modularity)
clustering = dendrogram.as_clustering()
# Returns: VertexClustering

# Access optimal count
n_clusters = dendrogram.optimal_count  # property, int

# Access merge matrix
merges = dendrogram.merges  # list of tuples
```

**Source:** https://github.com/igraph/python-igraph/blob/main/src/igraph/clustering.py (lines 1030–1130)

### `community_to_membership` — Low-level Index-based Cutting

```c
// C API: cut dendrogram after given number of merges
igraph_community_to_membership(
    const igraph_matrix_int_t *merges,  // merge matrix
    igraph_int_t nodes,                 // number of leaf nodes
    igraph_int_t steps,                 // number of merge steps to perform
    igraph_vector_int_t *membership,    // output: membership vector
    igraph_vector_int_t *csize          // output: community sizes (optional)
);
// Result: n - steps communities
```

**Source:** https://igraph.org/c/html/1.0.1/igraph-Community.html (section 1.4)

### `community_multilevel` (Louvain) — Level-based Access

```python
# Python: returns flat OR per-level partitions
clustering = g.community_multilevel(weights=None, resolution=1.0)
# Returns: VertexClustering (best modularity)

clusterings = g.community_multilevel(weights=None, resolution=1.0, return_levels=True)
# Returns: list[VertexClustering] — one per aggregation level
```

```c
// C API: returns membership matrix for ALL levels
igraph_community_multilevel(
    const igraph_t *graph,
    const igraph_vector_t *weights,
    const igraph_real_t resolution,
    igraph_vector_int_t *membership,    // best membership
    igraph_matrix_int_t *memberships,   // membership at each level (optional)
    igraph_vector_t *modularity         // modularity at each level (optional)
);
```

**Source:** https://igraph.org/c/html/1.0.1/igraph-Community.html (section 6.2) — https://github.com/igraph/python-igraph/blob/main/src/igraph/community.py (lines 225–260)

### `VertexClustering` — Flat Partition Return Type

```python
# Properties
clustering.membership    # list[int] — community ID per vertex
clustering.modularity    # float — modularity score (alias: .q)
clustering.graph         # Graph — associated graph

# Index-based access
clustering[0]            # list[int] — members of cluster 0
clustering.size(0)       # int — size of cluster 0
clustering.sizes()       # list[int] — sizes of all clusters
clustering.subgraph(0)   # Graph — subgraph of cluster 0
clustering.subgraphs()   # list[Graph] — all cluster subgraphs
clustering.giant()       # VertexClustering — largest cluster
clustering.crossing()    # list[bool] — edges between clusters
```

**Source:** https://python.igraph.org/en/main/api/igraph.VertexClustering.html — https://github.com/igraph/python-igraph/blob/main/src/igraph/clustering.py (lines 199–520)

### Return Type Summary for igraph

| Algorithm | Return type | Hierarchical? |
|-----------|-------------|---------------|
| `community_fastgreedy` | `VertexDendrogram` | Yes |
| `community_walktrap` | `VertexDendrogram` | Yes |
| `community_edge_betweenness` | `VertexDendrogram` | Yes |
| `community_leading_eigenvector` | `VertexClustering` | No (flat) |
| `community_multilevel` | `VertexClustering` or `list[VertexClustering]` | Optional |
| `community_leiden` | `VertexClustering` | No (flat) |
| `community_label_propagation` | `VertexClustering` | No (flat) |
| `community_spinglass` | `VertexClustering` | No (flat) |
| `community_infomap` | `VertexClustering` | No (flat) |

---

## 3. `louvain-igraph` (Python) — Predecessor to leidenalg

**Source:** https://github.com/vtraag/louvain-igraph — https://louvain-igraph.readthedocs.io/

### API Structure

Nearly identical to leidenalg (same author, same architecture):

```python
partition = la.find_partition(graph, la.ModularityVertexPartition)
# Returns: MutableVertexPartition

optimiser = la.Optimiser()
diff = optimiser.optimise_partition(partition)
```

### Resolution Profile

```python
profile = optimiser.resolution_profile(
    G, la.CPMVertexPartition,
    resolution_range=(0, 1),
)
# Returns: list[VertexPartition] at change points
```

**Source:** https://louvain-igraph.readthedocs.io/en/latest/advanced.html — https://deepwiki.com/vtraag/louvain-igraph/2.1-optimiser-and-louvain-algorithm

---

## 4. `python-louvain` / `community` (NetworkX-compatible)

**Source:** https://github.com/taynaud/python-louvain — https://python-louvain.readthedocs.io/

### Dendrogram as List of Dicts

```python
import community as community_louvain

# Generate full dendrogram
dendrogram = community_louvain.generate_dendrogram(
    graph,              # networkx.Graph
    part_init=None,     # initial partition (dict)
    weight='weight',
    resolution=1.0,
)
# Returns: list[dict]
#   dendrogram[0] = finest partition (smallest communities)
#   dendrogram[-1] = coarsest partition (largest communities)
#   Each dict: {node_id: community_id, ...}
#   Keys of level i+1 are values of level i (hierarchical nesting)
```

### Index-based Access

```python
# Access partition at specific level
partition = community_louvain.partition_at_level(dendrogram, level=2)
# Returns: dict {node_id: community_id}
```

### Best Partition (convenience)

```python
# Directly get highest-modularity partition
partition = community_louvain.best_partition(
    graph,
    partition=None,
    weight='weight',
    resolution=1.0,
)
# Returns: dict {node_id: community_id}
```

**Source:** https://python-louvain.readthedocs.io/en/latest/api.html

---

## 5. NetworkX Native (`leiden_partitions`)

**Source:** https://github.com/networkx/networkx/blob/main/networkx/algorithms/community/leiden.py

### Generator-based Level Access

```python
# Yields partitions at each aggregation level (generator)
partitions = nx.community.leiden_partitions(
    G,
    weight='weight',
    metric='cpm',          # or 'modularity'
    resolution=1.0,
    seed=None,
    theta=0.01,
)
# Yields: list[set] — each element is a set of nodes forming a community
# Level 0 = smallest communities (finest)
# Top level = largest communities (coarsest)
# Quality metric is nondecreasing across yielded partitions
```

### Best Partition (convenience wrapper)

```python
communities = nx.community.leiden_communities(
    G,
    weight='weight',
    resolution=1.0,
    max_level=None,        # limit number of aggregation levels
    metric='cpm',
    seed=None,
)
# Returns: list[set] — the final/best partition
```

**Source:** https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.leiden.leiden_partitions.html — https://github.com/networkx/networkx/blob/main/networkx/algorithms/community/leiden.py

---

## 6. Rust: `leiden-rs` Crate

**Source:** https://lib.rs/crates/leiden-rs — https://github.com/lfgranja/leiden-rs

### Hierarchical Output

```rust
use leiden_rs::{Leiden, LeidenConfig};

let leiden = Leiden::new(LeidenConfig { seed: Some(42), ..Default::default() });

// Single-level (flat) result
let result = leiden.run(&graph)?;
// result.partition: Partition
// result.quality: f64

// Hierarchical result
let h = leiden.run_hierarchical(&graph)?;
// h.num_levels() -> usize
// h.levels.iter() -> Vec<HierarchicalLevel>
//   level.node_count: usize
//   level.num_communities: usize
//   level.quality: f64

// Query community of a specific node at any level
let comm = h.community_of_at_level(node_index, level);
```

### Resolution Profile (threshold-based)

```rust
use leiden_rs::{resolution_scan, resolution_profile, QualityType};

// Linear scan: evenly spaced gamma values
let entries = resolution_scan(&graph, QualityType::CPM, (0.0, 1.0), 20, Some(42))?;
// entries: Vec<ResolutionEntry>
//   entry.resolution: f64
//   entry.num_communities: usize
//   entry.quality: f64

// Bisection profile: only change points (like leidenalg)
let profile = resolution_profile(&graph, QualityType::CPM, (0.0, 1.0), Some(42), 1e-3, 1.0)?;
```

### Return Types

```rust
pub struct Partition {
    // membership: Vec<usize>,  (community ID per node)
    // num_communities: usize,
}

pub struct LeidenOutput {
    pub partition: Partition,
    pub quality: f64,
}

pub struct HierarchicalOutput {
    pub levels: Vec<HierarchicalLevel>,
    // num_levels(): usize
    // community_of_at_level(node: usize, level: usize) -> usize
}
```

**Source:** https://lib.rs/crates/leiden-rs (README documentation)

---

## 7. Common Patterns Across Libraries

### Pattern A: Dendrogram Object (igraph)

The most structured approach — a dedicated `Dendrogram` / `VertexDendrogram` class
that encapsulates the merge history and supports cutting at arbitrary levels.

```
VertexDendrogram
  ├── .as_clustering(n=None) -> VertexClustering    # index-based cut
  ├── .optimal_count -> int                          # optimal cut hint
  ├── .merges -> list[tuple]                         # raw merge matrix
  └── .names -> list[str]                            # leaf labels
```

**Pros:** Full history preserved; can cut at any level post-hoc; optimal count cached.
**Cons:** Extra object to manage; not all algorithms produce dendrograms.

### Pattern B: List of Partitions (python-louvain, NetworkX)

The simplest approach — a plain list/array where each element is a complete
partition at one level.

```
dendrogram: list[dict]    # python-louvain
dendrogram: list[set]     # NetworkX (generator)
```

**Pros:** Dead simple; easy to iterate; no custom types.
**Cons:** No metadata (modularity per level, merge history); memory-heavy for large graphs.

### Pattern C: Resolution Profile (leidenalg, louvain-igraph, leiden-rs)

Returns partitions only at *change points* — values of the resolution parameter
where the optimal partition actually changes.

```
profile: list[VertexPartition]   # leidenalg
profile: Vec<ResolutionEntry>    # leiden-rs
```

**Pros:** Efficient; directly gives stable regions; no redundant partitions.
**Cons:** Only works for resolution-parameter methods; not a full dendrogram.

### Pattern D: Membership Matrix (igraph C API)

The `community_multilevel` C function returns a matrix where each row is the
membership vector at one aggregation level.

```
memberships: igraph_matrix_int_t   // n_levels × n_vertices
modularity:  igraph_vector_t       // modularity per level
```

**Pros:** Compact; all levels at once; easy to index.
**Cons:** Only for agglomerative methods; fixed at computation time.

### Pattern E: Hierarchical Output Object (leiden-rs)

A dedicated object that stores per-level partition data with query methods.

```
HierarchicalOutput
  ├── .num_levels() -> usize
  ├── .levels -> Vec<HierarchicalLevel>
  └── .community_of_at_level(node, level) -> usize
```

**Pros:** Type-safe; queryable; encapsulates level metadata.
**Cons:** Rust-specific; more API surface.

---

## 8. Recommended API Design

Based on the survey, a well-designed hierarchical community tree API should:

### Method Signatures

```rust
/// Run community detection, returning the best flat partition.
fn run(&self, graph: &Graph) -> Result<Partition, Error>;

/// Run community detection, returning the full hierarchical structure.
fn run_hierarchical(&self, graph: &Graph) -> Result<HierarchicalTree, Error>;

/// Scan resolution parameter, returning partitions at change points.
fn resolution_profile(
    &self,
    graph: &Graph,
    range: (f64, f64),
) -> Result<Vec<ResolutionLevel>, Error>;
```

### Return Types

```rust
/// A flat partition: maps each node to a community.
struct Partition {
    membership: Vec<usize>,      // community_id per node
    num_communities: usize,
    quality: f64,
}

/// Full hierarchical tree with index-based level access.
struct HierarchicalTree {
    levels: Vec<Level>,          // level[0] = finest, level[-1] = coarsest
}

impl HierarchicalTree {
    fn num_levels(&self) -> usize;
    fn level(&self, index: usize) -> Option<&Level>;        // index-based
    fn partition_at(&self, level: usize) -> Option<&Partition>;
    fn community_of_at_level(&self, node: usize, level: usize) -> Option<usize>;
    fn quality_at(&self, level: usize) -> Option<f64>;
}

/// A single level in the resolution profile.
struct ResolutionLevel {
    resolution: f64,             // gamma value
    partition: Partition,
    num_communities: usize,
}
```

### Key Design Decisions

1. **Index-based level access** (`tree.level(i)`) — the most universal pattern,
   supported by igraph (`as_clustering(n)`), python-louvain (`partition_at_level`),
   NetworkX (`leiden_partitions` generator), and leiden-rs (`community_of_at_level`).

2. **Separate `run()` vs `run_hierarchical()`** — follows leiden-rs pattern;
   avoids paying for hierarchical storage when only the flat result is needed.

3. **Resolution profile as change-point list** — follows leidenalg's efficient
   approach; returns only partitions at γ values where the assignment changes.

4. **Return `Partition` struct with `membership: Vec<usize>`** — the de facto
   standard across all libraries (igraph's `membership`, leidenalg's
   `partition.membership`, python-louvain's dict, NetworkX's list of sets).

5. **Quality score per level** — consistently provided by all libraries;
   essential for selecting the optimal cut.

---

## 9. Edge Cases and Constraints

| Edge case | Handling in libraries |
|-----------|----------------------|
| **Disconnected graphs** | igraph: each connected component starts as its own cluster; dendrogram merges within components only. leidenalg: handles natively. |
| **Single-node graphs** | igraph `community_to_membership`: returns trivial membership. leiden-rs: tested. |
| **Empty graphs** | igraph modularity returns NaN. leidenalg: partition with 0 quality. |
| **Self-loops** | igraph: counted twice in modularity. leiden-rs: supported. |
| **Negative edge weights** | igraph spinglass: supported. leiden: supported via CPM. |
| **Resolution parameter = 0** | leidenalg: degenerates to single community. |
| **Max community size** | leidenalg: `max_comm_size` parameter on Optimiser. leiden-rs: `max_comm_size` in config. |
| **Determinism** | leidenalg: `n_iterations` and random seed. leiden-rs: `seed` field. igraph: `set_rng_seed()`. |
| **Incomplete dendrograms** | igraph `le_community_to_membership`: handles divisive algorithms that stop early. |

---

## 10. Source Citations

| Claim | Source |
|-------|--------|
| leidenalg `find_partition` signature | https://github.com/vtraag/leidenalg/blob/main/src/leidenalg/__init__.py |
| leidenalg `resolution_profile` | https://leidenalg.readthedocs.io/en/stable/advanced.html#resolution-profile |
| leidenalg partition types | https://deepwiki.com/vtraag/leidenalg/2.1-vertexpartition-classes |
| igraph `VertexDendrogram.as_clustering` | https://github.com/igraph/python-igraph/blob/main/src/igraph/clustering.py |
| igraph `community_to_membership` | https://igraph.org/c/html/1.0.1/igraph-Community.html |
| igraph `community_multilevel` return | https://igraph.org/c/html/1.0.1/igraph-Community.html (section 6.2) |
| igraph `VertexClustering` API | https://python.igraph.org/en/main/api/igraph.VertexClustering.html |
| igraph community detection overview | https://deepwiki.com/igraph/igraph/3.2-community-detection |
| louvain-igraph Optimiser | https://deepwiki.com/vtraag/louvain-igraph/2.1-optimiser-and-louvain-algorithm |
| python-louvain `generate_dendrogram` | https://python-louvain.readthedocs.io/en/latest/api.html |
| python-louvain `partition_at_level` | https://python-louvain.readthedocs.io/en/latest/api.html |
| NetworkX `leiden_partitions` | https://github.com/networkx/networkx/blob/main/networkx/algorithms/community/leiden.py |
| NetworkX `leiden_communities` | https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.leiden.leiden_communities.html |
| leiden-rs hierarchical output | https://lib.rs/crates/leiden-rs |
| leiden-rs `resolution_profile` | https://lib.rs/crates/leiden-rs |
| leiden-rs `community_of_at_level` | https://lib.rs/crates/leiden-rs |
| Leiden algorithm paper | Traag, V.A., Waltman, L. & van Eck, N.J. (2019). Sci Rep 9, 5233. https://doi.org/10.1038/s41598-019-41695-z |
| Louvain algorithm paper | Blondel, V.D. et al. (2008). J Stat Mech P10008. https://doi.org/10.1088/1742-5468/2008/10/P10008 |
| Resolution profile insight | Traag, V.A., Krings, G., & Van Dooren, P. (2013). Sci Rep 3, 2930. https://doi.org/10.1038/srep02930 |

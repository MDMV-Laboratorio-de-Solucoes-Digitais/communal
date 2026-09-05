# Research: Community Detection Framework

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

This document consolidates technology decisions and best practices for implementing the Communal community detection framework. The feature specification is comprehensive with no unresolved clarifications; this research focuses on implementation patterns and algorithmic references.

---

## 1. Core Graph Representation

### Decision: Compressed Sparse Row (CSR) with typed identifiers

**Rationale**: CSR provides optimal cache locality for the neighbor iteration patterns dominant in community detection algorithms. The contiguous memory layout enables SIMD vectorization and minimizes cache misses during the local moving phase.

**Implementation**:
```rust
pub struct CsrGraph<N, E> {
    offsets: Vec<u32>,      // Length: node_count + 1
    targets: Vec<N>,        // Length: edge_count
    weights: Vec<E>,        // Length: edge_count (1.0 for unweighted)
    node_count: u32,
    edge_count: u32,
    directed: bool,
}
```

**Alternatives considered**:
- Adjacency list (Vec<Vec<(NodeId, EdgeWeight)>>): Higher memory overhead, poor cache locality
- Adjacency matrix: O(V²) memory, unsuitable for sparse graphs
- petgraph::Graph: Used only as external interchange format; converted to CSR for computation

**References**:
- Traag et al. (2019) - "From Louvain to Leiden: guaranteeing well-connected communities"
- CSR format: Standard sparse matrix representation (Bell & Garland, 2009)

---

## 2. Algorithm Implementations

### 2.1 Leiden Algorithm

**Decision**: Smart local move + randomized refinement + aggregation phases

**Key implementation details**:
- Smart local move: Only visit nodes whose neighborhood changed (queue-based)
- Randomized refinement: Random node ordering with optional seed for determinism
- Quality function: CPM or modularity with configurable resolution parameter gamma
- Convergence: Absolute or relative change threshold (default 1e-6)

**References**:
- Traag, Waltman, & van Eck (2019) - Nature Scientific Reports
- Implementations: leidenalg (Python), igraph (C)

### 2.2 Louvain Algorithm

**Decision**: Classical local moving + aggregation (no refinement phase)

**Key implementation details**:
- Local moving: Greedy modularity optimization via node moves
- Aggregation: Build reduced graph from community assignments
- No refinement phase (distinguishing feature from Leiden)

**References**:
- Blondel et al. (2008) - J. Stat. Mech.
- Implementations: louvain-igraph, python-louvain

### 2.3 Infomap

**Decision**: Map Equation optimization via random walks

**Key implementation details**:
- Teleportation rate (tau) configurable (default 0.15)
- Two-level Huffman coding of module exit/entry
- Flow-based clustering for directed graphs

**References**:
- Rosvall & Bergstrom (2008) - PNAS
- Rosvall, Axelsson, & Bergstrom (2009) - European Physical Journal ST

### 2.4 Label Propagation Algorithm (LBA)

**Decision**: Diffusion-based with sync/async modes

**Key implementation details**:
- Synchronous: All nodes update simultaneously (parallelizable)
- Asynchronous: Random node ordering with immediate updates
- Tie-breaking: Random with optional seed

**References**:
- Raghavan, Albert, & Kumara (2007) - Physical Review E

### 2.5 Fluid Communities

** Decision**: Fluid density-based expansion

**Key implementation details**:
- Target community count k (user-specified)
- Linear-time complexity O(V + E)
- Iterative density diffusion until convergence

**References**:
- Parés et al. (2018) - "Fluid Communities: A Competitive, Scalable and Highly Quality Community Detection Algorithm"

---

## 3. Quality Metrics

### Decision: Modularity Q, CPM, Map Equation, NMI, ARI

**Implementation**:
- **Modularity Q**: Newman-Girvan with resolution parameter
- **CPM**: Constant Potts Model with explicit gamma
- **Map Equation**: Infomap's flow-based quality
- **NMI**: Normalized Mutual Information (for ground-truth comparison)
- **ARI**: Adjusted Rand Index (for ground-truth comparison)

**References**:
- Newman (2006) - Modularity
- Traag, Van Dooren, & Nesterov (2011) - CPM
- Vinh, Epps, & Bailey (2010) - NMI/ARI information-theoretic variants

---

## 4. Dynamic Graph Updates (HIT Architecture)

### Decision: Hierarchical Incremental Tree for edge insertions/deletions

**Key implementation details**:
- Edge insertion: Local boundary recalculation within O(k) neighborhood
- Edge deletion: Immediate split into connected components if community disconnects
- Subtree stability: Unaffected communities remain unchanged
- Hierarchical slices: Coarse-to-fine levels for GraphRAG chunking

**References**:
- Şen et al. (2018) - "Incremental Community Detection in Social Networks"
- Zakrzewska & Bader (2015) - "Tracking Local Community Changes in Dynamic Networks"

---

## 5. Observability & Stepping

### Decision: Hybrid iterator + callback interface

**Implementation**:
- Iterator: `StepIterator` yields `StepEvent` on each `.next()` call
- Callback: `StepCallback` trait invoked per step
- Zero-cost when disabled: Event emission only triggers when subscriber registered
- Overhead budget: <= 20% when enabled

**Event types**:
- `PhaseStart(PhaseKind)`
- `NodeRelocation { node, from, to }`
- `RefinementSplit { community, into }`
- `AggregationContract { from, into }`
- `ConvergencePlateau { iteration, delta }`

---

## 6. Error Handling

### Decision: thiserror-based domain error types

**Error categories**:
- `GraphError`: Invalid topology, negative weights, self-loop issues
- `AlgorithmError`: Non-convergence, invalid parameters
- `PartitionError`: Invalid community assignments
- `IoError`: File I/O failures (CLI)

**Pattern**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("negative edge weight {weight} between nodes {from} and {to}")]
    NegativeWeight { from: NodeId, to: NodeId, weight: EdgeWeight },
    // ...
}
```

---

## 7. WASM Bindings

### Decision: wasm-bindgen with serde-wasm-bindgen

**Scope**: Leiden + Louvain only (single-threaded)
**Interface**: JSON input/output for graph data
**Zero-copy**: Shared memory buffers where possible

---

## 8. CLI Design

### Decision: Full pipeline capabilities

**Commands**:
- `communal run <file> --algorithm leiden` - Single algorithm execution
- `communal compare <file> --algorithms leiden,louvain,infomap` - Algorithm comparison
- `communal batch --config batch.toml` - Batch processing (file/algorithm/parameter sweep)
- `communal convert <input> <output> --from edgelist --to json` - Format conversion
- `communal metrics <file> --partition partition.json` - Quality metrics computation

**Input formats**: EdgeList, JSON, GML
**Output formats**: JSON, CSV, GML

---

## 9. TUI Design

### Decision: ratatui-based with pedagogical explanations

**Components**:
- Force-directed graph layout (2D canvas)
- Event log panel
- Statistics panel (quality, iteration, community count)
- Pedagogical explanation panel (hybrid static templates + interpolated state)

**Pedagogical template example**:
```
"Iteration {iteration}: Moving node {node} from community {from} to {to}.
 This improves modularity by {delta:.6f} because the node has
 {neighbor_count} neighbors in the target community."
```

---

## 10. Testing Strategy

### Decision: Multi-layer testing approach

1. **Unit tests**: Per-algorithm correctness on small graphs
2. **Property tests (proptest)**: Invariants (connected communities, quality monotonicity)
3. **Integration tests**: End-to-end pipeline validation
4. **Benchmark tests (criterion)**: Performance regression detection
5. **Ground-truth validation**: LFR/SBM benchmarks with NMI >= 0.95
6. **SNAP regression**: Zachary Karate Club, Dolphins, Cora, Enron

---

## Summary of Key Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Graph layout | CSR with u32 indexing | Cache locality, SIMD-friendly |
| Algorithms | Leiden, Louvain, Infomap, LPA, Fluid | Multi-paradigm coverage |
| Dynamic updates | HIT architecture | O(k) incremental, GraphRAG-ready |
| Observability | Hybrid iterator/callback | Zero-cost when disabled |
| Error handling | thiserror domain types | Type-safe, descriptive errors |
| WASM | Leiden + Louvain only | Lightweight, single-threaded |
| CLI | Full pipeline | Research + production workflows |
| TUI | ratatui + pedagogical | Education + debugging |
| Testing | Multi-layer | Mathematical confidence |

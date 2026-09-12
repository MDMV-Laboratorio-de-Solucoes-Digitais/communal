# Communal Roadmap

**Last updated:** 2026-03-09

## Current Status

Communal is a high-performance community detection framework in pure Rust. The core infrastructure is complete with the full Leiden algorithm (local moving + refinement + aggregation) working correctly on all Tier 1 deterministic reference graphs and real-world networks up to ~1,500 nodes.

---

## Optimization / Connectedness (003-optimize-connectedness)

Status: structural requirements satisfied (FR-001..FR-008 / SC-001..SC-009 verified in code; BFS removed, singleton-start + isolated-vertex + R/T arithmetic, debug-only assertion; benchmarks committed). Residual measurement/verification gaps T078–T081 remain — timing measurements per Measurement Protocol (SC-001/SC-002/SC-003/SC-009), property-based BFS/DFS result (SC-006/SC-008), inline Theorem 5 / Contract G4 reference (FR-005), and reference-alignment assertions (FR-006). See specs/003-optimize-connectedness/tasks.md.

---

## Graph Format Support

### Currently Supported

| Format | Extension | Status | Notes |
|--------|-----------|--------|-------|
| **Edge list** | `.edges`, `.txt`, `.el` | ✅ Implemented | Whitespace-delimited `source target [weight]`. Comments with `#`. |

**Example edge list:**
```text
# This is a comment
0 1 1.0
0 2 1.0
1 2 1.0
2 3 0.5
```

**Usage:**
```rust
use communal_core::io::edgelist::parse_edgelist_to_csr;

let input = std::fs::read_to_string("graph.edges")?;
let graph = parse_edgelist_to_csr(&input)?;
```

### Planned Format Support

| Format | Extension | Priority | Description |
|--------|-----------|----------|-------------|
| **GML** | `.gml` | HIGH | Graph Modularity Language. Used by Newman's data (Karate, Dolphins, Football, PolBooks, etc.) |
| **GraphML** | `.graphml` | HIGH | XML-based interchange format. Supported by 11+ tools (NetworkX, igraph, Gephi, yEd, Cytoscape, Neo4j) |
| **Pajek** | `.net` | MEDIUM | Social network analysis format. Used by Pajek and Visone |
| **Matrix Market** | `.mtx` | MEDIUM | Sparse matrix format. Used by SuiteSparse collection |
| **GEXF** | `.gexf` | LOW | Graph Exchange XML Format. Used by Gephi for dynamic graphs |
| **SNAP** | `.txt` | LOW | SNAP dataset format (edge list with `#` headers) |

### Format Details

**GML (Graph Modularity Language):**
```
graph [
  node [ id 0 label "0" ]
  node [ id 1 label "1" ]
  edge [ source 0 target 1 weight 1.0 ]
]
```

**GraphML:**
```xml
<graphml>
  <graph id="G" edgedefault="undirected">
    <node id="0"/>
    <node id="1"/>
    <edge source="0" target="1" weight="1.0"/>
  </graph>
</graphml>
```

**Pajek (.net):**
```
*Vertices 3
1 "one"
2 "two"
3 "three"
*Edges
1 2 1.0
2 3 1.0
```

**Matrix Market:**
```
%%MatrixMarket matrix coordinate real general
% Comments
3 3 4
1 2 1.0
1 3 1.0
2 3 1.0
3 1 0.5
```

---

## Algorithm Implementation Status

### Community Detection Algorithms

| Algorithm | Status | Notes |
|-----------|--------|-------|
| **Leiden** (local moving) | ✅ Working | Core optimization phase complete |
| **Leiden** (refinement) | ✅ Integrated | Subpartition guarantee enforced; connected-community invariant holds |
| **Louvain** | 📋 Planned | Similar to Leiden but without refinement |
| **Infomap** | 📋 Planned | Information-theoretic flow-based |
| **LPA** | 📋 Planned | Label Propagation Algorithm |
| **Fluid Communities** | 📋 Planned | Diffusion-based, requires specifying k |
| **Walktrap** | 📋 Planned | Random walk-based |
| **Fast-Greedy (CNM)** | 📋 Planned | Hierarchical modularity optimization |
| **Spinglass** | 📋 Planned | Statistical physics approach |
| **Leading Eigenvector** | 📋 Planned | Spectral method |
| **Girvan-Newman** | 📋 Planned | Edge betweenness (slow: O(VE²)) |

### Quality Metrics

| Metric | Status | Notes |
|--------|--------|-------|
| **Modularity Q** | ✅ Implemented | With resolution parameter γ |
| **CPM** | ✅ Implemented | Constant Potts Model, resolution-limit-free |
| **Map Equation** | 📋 Planned | For Infomap |
| **NMI** | 📋 Planned | Normalized Mutual Information (for benchmarks) |
| **ARI** | 📋 Planned | Adjusted Rand Index (for benchmarks) |

### Graph Generators

| Generator | Status | Notes |
|-----------|--------|-------|
| **LFR Benchmark** | 📋 Planned | Standard benchmark with ground truth |
| **SBM** | 📋 Planned | Stochastic Block Model |
| **Erdős-Rényi** | 📋 Planned | Random graphs |
| **Barabási-Albert** | 📋 Planned | Preferential attachment |
| **Watts-Strogatz** | 📋 Planned | Small-world networks |

---

## Infrastructure & Tooling

### Current Architecture

| Component | Status | Crate |
|-----------|--------|-------|
| Core traits | ✅ | `communal-core` |
| CSR graph | ✅ | `communal-core` |
| Graph builder | ✅ | `communal-core` |
| Edge list I/O | ✅ | `communal-core` |
| Error types | ✅ | `communal-core` |
| Leiden algorithm | ✅ | `communal-algo` |
| Quality functions | ✅ | `communal-algo` |
| petgraph adapter | ✅ | `communal-petgraph` |

### Planned Infrastructure

| Component | Priority | Notes |
|-----------|----------|-------|
| CLI binary | HIGH | `communal-cli` with file I/O |
| TUI binary | MEDIUM | `communal-tui` with algorithm stepping |
| WASM bindings | MEDIUM | `communal-wasm` for browser |
| Python bindings | LOW | PyO3 wrapper |
| Benchmark suite | HIGH | Criterion benchmarks |
| LFR generator | HIGH | For ground-truth testing |

---

## Testing & Validation

### Current Test Coverage

- ✅ Property-based: connected communities invariant
- ✅ Property-based: determinism (same seed → same result)
- ✅ Property-based: quality monotonicity
- ✅ Edge cases: empty graph, single node, single edge, disconnected components, negative weights
- ✅ Tier 1 deterministic graphs: all 8 pass
- ✅ Tier 2 LFR benchmarks: N=1000, μ=0.1 runs in ~1.2s with NMI=0.977
- ✅ Tier 3 real-world networks: all pass

| Network | Nodes | Edges | Time | Q |
|---------|-------|-------|------|---|
| Karate Club | 34 | 78 | 9ms | 0.0355 |
| Dolphins | 62 | 159 | 51ms | 0.0578 |
| Football | 115 | 613 | 170ms | 0.0369 |
| PolBooks | 105 | 441 | 83ms | 0.1522 |
| Les Misérables | 77 | 254 | 33ms | 0.1642 |
| NetScience | 1,589 | 2,742 | 926ms | 0.1838 |
| PolBlogs | 1,491 | 19,090 | 60s | 0.0441 |

### Planned Testing

| Test Type | Priority | Notes |
|-----------|----------|-------|
| LFR benchmark validation | HIGH | NMI ≥ 0.95 at μ ≤ 0.3 |
| Real-world benchmarks | HIGH | DBLP, Amazon, LiveJournal modularity targets |
| Performance benchmarks | MEDIUM | Compare against igraph/leidenalg |
| Stress tests | MEDIUM | Large graphs (10⁵+ nodes) |
| Fuzz testing | LOW | Random graph generation |

---

## Performance Goals

| Metric | Target | Current |
|--------|--------|---------|
| Leiden on N=1,589, E=2,742 (NetScience) | < 5 seconds | ✅ 926ms |
| Leiden on N=1,491, E=19,090 (PolBlogs) | < 120 seconds | ✅ 60s |
| Leiden on N=10⁴, E=10⁵ | < 1 second | Not benchmarked |
| Leiden on N=10⁶, E=10⁷ | < 10 seconds | Not benchmarked |
| Memory usage | O(V + E) | ✅ Achieved (CSR format) |
| Parallel speedup | 4-8x on 8 cores | Not implemented |

## Next Steps for Performance

- ✅ Cache community degree sums and sizes (avoid recomputation)
- ✅ Early termination when no nodes move
- Parallel local moving (optional, future)
- Sparse vector representation for community membership (optional, future)

---

## Release Plan

### v0.1.0 (Current — Publishable)
- Core infrastructure (traits, CSR graph, builder)
- Full Leiden algorithm (local moving + refinement + aggregation)
- Modularity Q and CPM quality functions
- Edge list I/O
- Property-based and edge case tests
- Performance: real-world networks up to ~1,500 nodes supported

### v0.2.0 (Next)
- GML and GraphML format support
- LFR benchmark generator
- CLI binary
- NMI/ARI metrics for benchmarking

### v0.3.0
- Additional algorithms (Louvain, Infomap, LPA)
- TUI binary
- Performance optimizations
- Python bindings

### v1.0.0
- All planned algorithms
- All planned formats
- Production-ready CLI/TUI/WASM
- Comprehensive benchmark suite
- Documentation and tutorials

---

## Contributing

See [AGENTS.md](AGENTS.md) for project conventions and code map.

Research notes:
- [Community Detection Implementations](research/community-detection-implementations.md)
- [Benchmark Graphs](research/benchmark-graphs-leiden.md)
- [Graph Formats](research/graph-formats.md)

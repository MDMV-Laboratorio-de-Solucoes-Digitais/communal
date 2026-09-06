# Community Detection Algorithm Implementations: Open-Source Landscape

**Date:** 2026-02-10
**Context:** Research for the Communal Rust-based community detection framework. This document catalogs the primary open-source implementations of community detection algorithms, organized by library type, language ecosystem, and maintenance status.

---

## Table of Contents

1. [Multi-Algorithm Libraries](#1-multi-algorithm-libraries)
2. [Single-Algorithm Libraries](#2-single-algorithm-libraries)
3. [Language-Specific Ecosystems](#3-language-specific-ecosystems)
4. [Benchmark / Reference Implementations](#4-benchmark--reference-implementations)
5. [Algorithm Complexity Reference](#5-algorithm-complexity-reference)
6. [Summary Matrix](#6-summary-matrix)

---

## 1. Multi-Algorithm Libraries

These libraries implement many algorithms under a unified interface and are the most relevant for Communal's design.

### 1.1 igraph

| Field | Detail |
|---|---|
| **URL** | [github.com/igraph/igraph](https://github.com/igraph/igraph) |
| **Language** | C (core), with bindings for Python, R, Julia |
| **License** | GPL-2.0 |
| **Algorithms** | Louvain (multilevel), Leiden, Walktrap, Fast-Greedy (CNM), Spinglass, Leading Eigenvector, Edge Betweenness (Girvan-Newman), Fluid Communities, Label Propagation, Infomap, Optimal Modularity |
| **Maintenance** | Actively maintained. Version 1.0.0 released September 2025; 1.0.1 bugfix release December 2025. |
| **Notable features** | The most widely-used C network analysis library. Highly optimized. All algorithms accessible from Python (`python-igraph`), R (`igraph`), and Julia (`Graphs.jl`). The C core is the reference implementation for many algorithms. |
| **Performance** | C core with hand-tuned data structures. Scales to millions of nodes and billions of edges. Louvain/Leiden implementations are among the fastest available. |

**Algorithms provided by igraph (C core):**

| Algorithm | igraph Function | Complexity |
|---|---|---|
| Louvain (Multilevel) | `igraph_community_multilevel()` | O(E) per iteration |
| Leiden | `igraph_community_leiden()` | O(E) per iteration |
| Walktrap | `igraph_community_walktrap()` | O(V² log V) |
| Fast-Greedy (CNM) | `igraph_community_fastgreedy()` | O(V log² V) sparse |
| Spinglass | `igraph_community_spinglass()` | O(V³) worst case |
| Leading Eigenvector | `igraph_community_leading_eigenvector()` | O(E) per split |
| Edge Betweenness | `igraph_community_edge_betweenness()` | O(V·E²) |
| Fluid Communities | `igraph_community_fluid_communities()` | O(E) per iteration |
| Label Propagation | `igraph_community_label_propagation()` | O(E) per iteration |
| Infomap | `igraph_community_infomap()` | O(E) per sweep |
| Optimal Modularity | `igraph_community_optimal_modularity()` | Exponential (exact) |

**Sources:** [igraph C Manual](https://igraph.org/c/html/1.0.1/igraph-Community.html), [igraph GitHub](https://github.com/igraph/igraph), [igraph 1.0.0 announcement](https://igraph.org/2025/09/20/igraph-1.0.0-c.html)

---

### 1.2 CDlib (Community Discovery Library)

| Field | Detail |
|---|---|
| **URL** | [github.com/GiulioRossetti/cdlib](https://github.com/GiulioRossetti/cdlib) |
| **Language** | Python (meta-library wrapping many backends) |
| **License** | BSD-2-Clause |
| **Algorithms** | 70+ algorithms across crisp, overlapping, fuzzy, attributed, bipartite, dynamic, and edge clustering categories. Includes: Louvain, Leiden, Infomap, Walktrap, Spinglass, Girvan-Newman, Fast-Greedy, Fluid Communities, LPA, SLPA, COPRA, DEMON, OSLOM, BigClam, GCE, WalkSCAN, Link Communities, SBM (via graph-tool), and many more. |
| **Maintenance** | Actively maintained. 1,090 commits. Latest release 0.4.0. |
| **Notable features** | The most comprehensive meta-library. Standardized input/output across all algorithms. Wraps original implementations from igraph, networkx, graph-tool, and standalone repos. Includes community evaluation metrics and visualization. |
| **Performance** | Varies by backend. The meta-layer adds overhead but delegates computation to optimized C/C++ backends where available. |

**Algorithm categories in CDlib 0.4.0:**

| Category | Count | Examples |
|---|---|---|
| Crisp Communities | ~30 | Louvain, Leiden, Infomap, Walktrap, Spinglass, LPA, Fluid, Girvan-Newman, Fast-Greedy, Leading Eigenvector, MCL, SCAN, PARIS, PyCombo, etc. |
| Overlapping Communities | ~30 | DEMON, OSLOM, BigClam, COPRA, SLPA, WalkSCAN, GCE, CONGA, CONGO, Ego Networks, LFM, LPAM, LPANNI, ANGEL, etc. |
| Fuzzy Communities | 2 | FRC-FGSN, Principled Clustering |
| Attributed Communities | 2 | EVA, iLouvain |
| Bipartite Communities | 4 | BiMLPA, CONDOR, CPM_Bipartite, Infomap_Bipartite |
| Dynamic Communities | 1 | Tiles |
| Edge Clustering | 3 | Hierarchical Link Community (3 variants) |

**Sources:** [CDlib GitHub](https://github.com/GiulioRossetti/cdlib), [CDlib Algorithms Reference Table](https://cdlib.readthedocs.io/en/latest/reference/algorithms_table.html), [CDlib PyPI](https://pypi.org/project/cdlib/)

---

### 1.3 NetworKit

| Field | Detail |
|---|---|
| **URL** | [github.com/networkit/networkit](https://github.com/networkit/networkit) |
| **Language** | C++ (core), Python (via Cython) |
| **License** | MIT |
| **Algorithms** | PLM (Parallel Louvain), PLP (Parallel Label Propagation), LCPM, and additional graph analysis algorithms. Primarily focused on Louvain and Label Propagation variants. |
| **Maintenance** | Actively maintained. 9,588 commits. 873 stars. |
| **Notable features** | Designed for massive-scale networks (billions of edges). Parallel implementations using OpenMP. Research testbed for algorithm engineering. |
| **Performance** | PLM is one of the fastest parallel Louvain implementations. Scales to billions of edges on multicore systems. |

**Sources:** [NetworKit GitHub](https://github.com/networkit/networkit), [NetworKit Community Detection](https://networkit.github.io/dev-docs/notebooks/Community.html), [NetworKit PyPI](https://pypi.org/project/networkit/)

---

### 1.4 graph-tool

| Field | Detail |
|---|---|
| **URL** | [graph-tool.skewed.de](https://graph-tool.skewed.de/) / [github.com/antmd/graph-tool](https://github.com/antmd/graph-tool) |
| **Language** | C++ (core with Boost Graph Library), Python |
| **License** | GPL-3.0 |
| **Algorithms** | Stochastic Block Model (SBM) inference: `minimize_blockmodel_dl()`, `minimize_nested_blockmodel_dl()`. Also supports overlapping SBM. |
| **Maintenance** | Actively maintained. |
| **Notable features** | SBM-based approach is fundamentally different from modularity optimization. Uses Bayesian inference. Log-linear complexity on network size. |
| **Performance** | C++ with template metaprogramming. Log-linear complexity for SBM inference. |

**Sources:** [graph-tool SBM documentation](https://graph-tool.skewed.de/static/doc/demos/inference/inference.html), [graph-tool GitHub](https://github.com/antmd/graph-tool)

---

### 1.5 NetworkX

| Field | Detail |
|---|---|
| **URL** | [github.com/networkx/networkx](https://github.com/networkx/networkx) |
| **Language** | Pure Python |
| **License** | BSD-3-Clause |
| **Algorithms** | Louvain (via `python-louvain` or `nx-cugraph` backend), Leiden (via `leidenalg` or `nx-cugraph`), Girvan-Newman, Label Propagation, Fluid Communities (via igraph bridge), Modularity-based (greedy/CPM via `greedy_modularity_communities`), Asyn LPA, LPA, Semi-synchronous LPA |
| **Maintenance** | Actively maintained. Version 3.6+. |
| **Notable features** | The most widely-used Python graph library. Many algorithms delegate to optimized backends. GPU acceleration available via `nx-cugraph` (RAPIDS). |
| **Performance** | Pure Python implementations are slow for large graphs. GPU backend (`nx-cugraph`) provides 315x speedup for Leiden on genomics graphs. |

**Sources:** [NetworkX Community Detection](https://networkx.org/documentation/stable/reference/algorithms/community.html), [NetworkX GitHub](https://github.com/networkx/networkx), [NVIDIA GPU Leiden blog](https://developer.nvidia.com/blog/how-to-accelerate-community-detection-in-python-using-gpu-powered-leiden/)

---

## 2. Single-Algorithm Libraries

### 2.1 leidenalg (Leiden Algorithm)

| Field | Detail |
|---|---|
| **URL** | [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg) |
| **Language** | C++ (core `libleidenalg`), Python bindings |
| **License** | GPL-3.0 |
| **Algorithms** | Leiden algorithm with 6 quality functions: Modularity, CPM, RBConfiguration, RBER, Significance, Surprise |
| **Maintenance** | Actively maintained. 1,030 commits. 797 stars. Latest version 0.10.3.dev. |
| **Notable features** | The reference implementation of the Leiden algorithm (Traag et al. 2019). Supports multiplex partitions, bipartite graphs, partial optimization. |
| **Performance** | Scales to millions of nodes. Faster than Louvain on large networks. |

**Sources:** [leidenalg GitHub](https://github.com/vtraag/leidenalg), [leidenalg documentation](https://leidenalg.readthedocs.io/), [libleidenalg GitHub](https://github.com/vtraag/libleidenalg)

---

### 2.2 louvain-igraph (Louvain Algorithm)

| Field | Detail |
|---|---|
| **URL** | [github.com/vtraag/louvain-igraph](https://github.com/vtraag/louvain-igraph) |
| **Language** | C++ (core), Python bindings |
| **License** | GPL-3.0 |
| **Algorithms** | Louvain algorithm with 5 quality functions: Modularity, RBConfiguration, RBER, Significance, Surprise |
| **Maintenance** | **Superseded by leidenalg.** No longer maintained. |
| **Notable features** | Was the reference Louvain implementation for igraph. Now users are directed to leidenalg. |
| **Performance** | Scales to millions of nodes. |

**Sources:** [louvain-igraph GitHub](https://github.com/vtraag/louvain-igraph), [louvain-igraph documentation](https://louvain-igraph.readthedocs.io/)

---

### 2.3 Infomap (mapequation/infomap)

| Field | Detail |
|---|---|
| **URL** | [github.com/mapequation/infomap](https://github.com/mapequation/infomap) |
| **Language** | C++ (core), Python, R, JavaScript (Node.js/browser) |
| **License** | GPL-3.0 (dual-licensing available) |
| **Algorithms** | Infomap (Map Equation optimization) |
| **Maintenance** | Actively maintained. 2,002 commits. 493 stars. Latest release 2.15.0 (July 2026). |
| **Notable features** | The reference implementation. Multi-level network clustering based on information theory. Supports two-level and hierarchical solutions. CLI, Python, R, JavaScript, and Docker interfaces. |
| **Performance** | O(E) per sweep. Scales to millions of nodes and billions of edges. |

**Sources:** [Infomap GitHub](https://github.com/mapequation/infomap), [Infomap releases](https://github.com/mapequation/infomap/releases), [mapequation.org](https://mapequation.org/infomap/)

---

### 2.4 python-louvain (taynaud/python-louvain)

| Field | Detail |
|---|---|
| **URL** | [github.com/taynaud/python-louvain](https://github.com/taynaud/python-louvain) |
| **Language** | Pure Python (with NumPy) |
| **License** | BSD (see repo) |
| **Algorithms** | Louvain algorithm |
| **Maintenance** | Last significant activity ~2018. Largely unmaintained. |
| **Notable features** | Simple pure-Python implementation. Used as the Louvain backend for NetworkX. |
| **Performance** | Slower than C++ implementations. Suitable for small to medium graphs. |

**Sources:** [python-louvain GitHub](https://github.com/taynaud/python-louvain), [python-louvain PyPI](https://pypi.org/project/python-louvain/)

---

### 2.5 jlguillaume/louvain (C Implementation)

| Field | Detail |
|---|---|
| **URL** | [github.com/jlguillaume/louvain](https://github.com/jlguillaume/louvain) |
| **Language** | C |
| **License** | Free (see author's website) |
| **Algorithms** | Louvain algorithm |
| **Maintenance** | Minimal activity. Historical reference. |
| **Notable features** | One of the earliest C implementations of Louvain. By Jean-Loup Guillaume, co-author of the original Louvain paper. |
| **Performance** | C implementation, efficient for its era. |

**Sources:** [jlguillaume/louvain GitHub](https://github.com/jlguillaume/louvain), [Jean-Loup Guillaume website](http://jlguillaume.free.fr/www/programs.php)

---

### 2.6 Fluid Communities (HPAI-BSC/Fluid-Communities)

| Field | Detail |
|---|---|
| **URL** | [github.com/HPAI-BSC/Fluid-Communities](https://github.com/HPAI-BSC/Fluid-Communities) |
| **Language** | Python |
| **License** | See repo |
| **Algorithms** | Fluid Communities (Parés et al. 2018) |
| **Maintenance** | Research code. |
| **Notable features** | Reference implementation of the Fluid Communities algorithm. Allows specifying the number of communities k. |
| **Performance** | O(E) per iteration. |

**Sources:** [Fluid-Communities GitHub](https://github.com/HPAI-BSC/Fluid-Communities), [igraph Fluid Communities](https://python.igraph.org/en/latest/api/igraph.community.html)

---

### 2.7 WalkSCAN (ahollocou/walkscan)

| Field | Detail |
|---|---|
| **URL** | [github.com/ahollocou/walkscan](https://github.com/ahollocou/walkscan) |
| **Language** | C++ (core), Python |
| **License** | See repo |
| **Algorithms** | WalkSCAN (random walk + DBSCAN for local community detection) |
| **Maintenance** | Research code. 26 stars. |
| **Notable features** | Reference implementation. Combines PageRank-style random walks with DBSCAN clustering. |
| **Performance** | Near-linear per random walk. |

**Sources:** [walkscan GitHub](https://github.com/ahollocou/walkscan), [CDlib WalkSCAN](https://cdlib.readthedocs.io/en/latest/reference/generated/cdlib.algorithms.walkscan.html)

---

### 2.8 DEMON (GiulioRossetti/DEMON)

| Field | Detail |
|---|---|
| **URL** | [github.com/GiulioRossetti/DEMON](https://github.com/GiulioRossetti/DEMON) |
| **Language** | Python |
| **License** | See repo |
| **Algorithms** | DEMON (Democratic Overlapping Method) |
| **Maintenance** | Integrated into CDlib. Standalone repo may be less active. |
| **Notable features** | Local-first overlapping community detection. Democratic label propagation on ego networks. |
| **Performance** | Near-linear on ego networks. |

**Sources:** [DEMON GitHub](https://github.com/GiulioRossetti/DEMON), [CDlib DEMON](https://cdlib.readthedocs.io/en/0.4.1/reference/generated/cdlib.algorithms.demon.html)

---

### 2.9 Link Communities (ntamas/hlc)

| Field | Detail |
|---|---|
| **URL** | [github.com/ntamas/hlc](https://github.com/ntamas/hlc) |
| **Language** | Python (using igraph) |
| **License** | See repo |
| **Algorithms** | Hierarchical Link Clustering (Ahn et al. 2010) |
| **Maintenance** | Research code. |
| **Notable features** | Python implementation of link communities. Uses Jaccard similarity between links. |
| **Performance** | Super-linear, hierarchical. |

**Sources:** [hlc GitHub](https://github.com/ntamas/hlc), [linkcomm R package](https://rdrr.io/github/alextkalinka/linkcomm/man/getLinkCommunities.html)

---

### 2.10 COPRA (puzzlef/copra-communities)

| Field | Detail |
|---|---|
| **URL** | [github.com/puzzlef/copra-communities](https://github.com/puzzlef/copra-communities), [github.com/puzzlef/copra-communities-openmp](https://github.com/puzzlef/copra-communities-openmp) |
| **Language** | C++ |
| **License** | See repo |
| **Algorithms** | COPRA (Community OVerlap PRopagation Algorithm) |
| **Maintenance** | Research code. |
| **Notable features** | Both single-threaded and OpenMP-based multi-threaded implementations available. |
| **Performance** | Near-linear per iteration. OpenMP version scales with cores. |

**Sources:** [copra-communities GitHub](https://github.com/puzzlef/copra-communities), [copra-communities-openmp GitHub](https://github.com/puzzlef/copra-communities-openmp)

---

### 2.11 OSLOM (idekerlab/cdoslom)

| Field | Detail |
|---|---|
| **URL** | [github.com/idekerlab/cdoslom](https://github.com/idekerlab/cdoslom) |
| **Language** | C++ |
| **License** | See repo |
| **Algorithms** | OSLOM (Order Statistics Local Optimization Method) |
| **Maintenance** | Packaged for Cytoscape. Original code from Lancichinetti et al. |
| **Notable features** | Detects statistically significant overlapping communities. Accounts for edge directions, weights, hierarchies. |
| **Performance** | Super-linear, iterative. |

**Sources:** [cdoslom GitHub](https://github.com/idekerlab/cdoslom), [OSLOM paper](https://arxiv.org/abs/1012.2363)

---

### 2.12 BigClam (snap-stanford/snap)

| Field | Detail |
|---|---|
| **URL** | [github.com/snap-stanford/snap](https://github.com/snap-stanford/snap) (in `examples/bigclam/`) |
| **Language** | C++ |
| **License** | BSD (SNAP license) |
| **Algorithms** | BigClam (Cluster Affiliation Model for Big Networks) |
| **Maintenance** | SNAP is maintained. BigClam example is part of SNAP. |
| **Notable features** | Non-negative matrix factorization approach. Detects overlapping communities. |
| **Performance** | Scales to large networks. |

**Sources:** [SNAP BigClam](https://github.com/snap-stanford/snap/tree/master/examples/bigclam), [SNAP GitHub](https://github.com/snap-stanford/snap)

---

## 3. Language-Specific Ecosystems

### 3.1 Python Ecosystem

| Library | Algorithms | License | Maintenance |
|---|---|---|---|
| **igraph (python-igraph)** | Louvain, Leiden, Walktrap, Fast-Greedy, Spinglass, Leading Eigenvector, Edge Betweenness, Fluid, Label Propagation, Infomap, Optimal | GPL-2.0 | Active |
| **NetworkX** | Louvain (via backend), Leiden (via backend), Girvan-Newman, LPA, Asyn LPA, Semi-sync LPA, Greedy Modularity | BSD-3-Clause | Active |
| **CDlib** | 70+ algorithms (meta-library) | BSD-2-Clause | Active |
| **leidenalg** | Leiden (6 quality functions) | GPL-3.0 | Active |
| **louvain-igraph** | Louvain (5 quality functions) | GPL-3.0 | Superseded |
| **python-louvain** | Louvain | BSD | Unmaintained |
| **infomap (PyPI)** | Infomap | GPL-3.0 | Active |
| **graph-tool** | SBM inference | GPL-3.0 | Active |
| **NetworKit** | PLM, PLP, LCPM | MIT | Active |
| **DEMON** | DEMON | See repo | Integrated into CDlib |
| **communities (PyPI)** | Louvain | See repo | Minimal |

**Sources:** [python-igraph](https://python.igraph.org/), [NetworkX](https://networkx.org/), [CDlib](https://cdlib.readthedocs.io/), [leidenalg PyPI](https://pypi.org/project/leidenalg/), [infomap PyPI](https://pypi.org/project/infomap/)

---

### 3.2 R Ecosystem

| Package | Algorithms | License | Maintenance |
|---|---|---|---|
| **igraph (R)** | Same as C core: Louvain, Leiden, Walktrap, Fast-Greedy, Spinglass, Leading Eigenvector, Edge Betweenness, Fluid, Label Propagation, Infomap | GPL-2.0 | Active |
| **linkcomm** | Link Communities | GPL-2.0 | Moderate |
| **leidenbase** | Leiden (R port of libleidenalg) | GPL-3.0 | Active |
| **leiden (TomKellyGenetics)** | Leiden (R port) | GPL-3.0 | Active |
| **leidenAlg (kharchenkolab)** | Leiden (R port) | GPL-3.0 | Active |
| **infomap (R)** | Infomap | GPL-3.0 | Active |

**Sources:** [igraph R](https://igraph.org/r/), [linkcomm R](https://rdrr.io/github/alextkalinka/linkcomm/), [leidenbase GitHub](https://github.com/cole-trapnell-lab/leidenbase)

---

### 3.3 C/C++ Ecosystem

| Library | Algorithms | License | Maintenance |
|---|---|---|---|
| **igraph (C)** | 11+ community detection algorithms | GPL-2.0 | Active (v1.0.0, Sept 2025) |
| **libleidenalg** | Leiden (6 quality functions) | GPL-3.0 | Active |
| **louvain-igraph (C++)** | Louvain (5 quality functions) | GPL-3.0 | Superseded |
| **Infomap (C++)** | Infomap | GPL-3.0 | Active |
| **NetworKit (C++)** | PLM, PLP, LCPM | MIT | Active |
| **graph-tool (C++)** | SBM inference | GPL-3.0 | Active |
| **SNAP (C++)** | BigClam, other graph algorithms | BSD | Active |
| **jlguillaume/louvain (C)** | Louvain | Free | Minimal |
| **COPRA (C++)** | COPRA | See repo | Research code |
| **OSLOM (C++)** | OSLOM | See repo | Research code |
| **WalkSCAN (C++)** | WalkSCAN | See repo | Research code |
| **GCE (C++)** | Greedy Clique Expansion | GPL-3.0 (QOCE fork) | Research code |

**Sources:** [igraph C](https://igraph.org/c/), [libleidenalg](https://github.com/vtraag/libleidenalg), [NetworKit](https://networkit.github.io/), [SNAP](https://snap.stanford.edu/)

---

### 3.4 Rust Ecosystem

| Library/Crate | Algorithms | License | Maintenance |
|---|---|---|---|
| **petgraph** | Graph data structures only (no community detection algorithms) | MIT/Apache-2.0 | Active |
| **rustworkx (Qiskit)** | Graph algorithms (no community detection yet; LPA requested in issues) | Apache-2.0 | Active |
| **rustworkx-core** | Same as rustworkx (Rust API) | Apache-2.0 | Active |
| **fast-louvain (Splines)** | Louvain (work in progress) | See repo | WIP |
| **louvain-rs (graphext)** | Louvain | See repo | Moderate |
| **graphify-cluster** | Leiden | MIT | Active |

**Key observation:** The Rust ecosystem currently lacks comprehensive community detection implementations. This is the gap Communal aims to fill.

**Sources:** [petgraph GitHub](https://github.com/petgraph/petgraph), [rustworkx GitHub](https://github.com/Qiskit/rustworkx), [fast-louvain GitHub](https://github.com/Splines/fast-louvain), [louvain-rs GitHub](https://github.com/graphext/louvain-rs), [graphify-cluster](https://lib.rs/crates/graphify-cluster)

---

### 3.5 Julia Ecosystem

| Package | Algorithms | License | Maintenance |
|---|---|---|---|
| **CommunityDetection.jl** | Nonbacktracking, Bethe Hessian | MIT | Active |
| **GraphCommunities.jl** | Fast LPA, Graph K-means, experimental algorithms | MIT | Active |
| **Graphs.jl** | Graph data structures (delegates to CommunityDetection.jl) | MIT | Active |

**Sources:** [CommunityDetection.jl GitHub](https://github.com/JuliaGraphs/CommunityDetection.jl), [GraphCommunities.jl](https://randyrdavila.github.io/GraphCommunities.jl/), [Graphs.jl](https://juliagraphs.org/Graphs.jl/)

---

### 3.6 Java Ecosystem

| Library | Algorithms | License | Maintenance |
|---|---|---|---|
| **JGraphT** | Graph algorithms (limited community detection) | EPL-2.0 / LGPL-2.1 | Active |
| **SNAP (Java via JNI)** | Via C++ core | BSD | Active |

**Sources:** [JGraphT GitHub](https://github.com/jgrapht/jgrapht), [JGraphT website](https://jgrapht.org/)

---

## 4. Benchmark / Reference Implementations

### 4.1 CommunityDetectionCodes (RapidsAtHKUST)

| Field | Detail |
|---|---|
| **URL** | [github.com/RapidsAtHKUST/CommunityDetectionCodes](https://github.com/RapidsAtHKUST/CommunityDetectionCodes) |
| **Language** | C++, Python, Java |
| **License** | GPL-2.0 |
| **Algorithms** | 20+ overlapping community detection algorithms: CPM, CIS, EAGLE, LinkComm, iLCD, CONGA, TopGC, GCE, OSLOM, MOSES, SLPA, FastCPM, ParCPM, DEMON, SVINET, SeedExpansion, HRGrow, LEMON, and more. |
| **Maintenance** | Research code (PhD survey, ~2016). 240 commits. |
| **Notable features** | Collects and refactors overlapping community detection algorithms from the literature. Includes LFR benchmark generator and evaluation metrics. |
| **Performance** | Research-grade code, not production-optimized. |

**Sources:** [CommunityDetectionCodes GitHub](https://github.com/RapidsAtHKUST/CommunityDetectionCodes)

---

### 4.2 Neo4j Graph Data Science Library

| Field | Detail |
|---|---|
| **URL** | [neo4j.com/docs/graph-data-science](https://neo4j.com/docs/graph-data-science/current/algorithms/community/) |
| **Language** | Java (with native C++ kernels) |
| **License** | Neo4j Community License (GPL-3.0 for community edition) |
| **Algorithms** | Louvain, Leiden, Label Propagation, Local Clustering Coefficient, K-Core, K-1 Coloring, SCC, WCC, and more. |
| **Maintenance** | Actively maintained by Neo4j. |
| **Notable features** | Production-grade graph analytics on top of Neo4j. Stream/mutate/write modes. Native GPU acceleration for some algorithms. |
| **Performance** | Optimized for production use. Scales to billions of edges with Neo4j's native graph engine. |

**Sources:** [Neo4j GDS Community Detection](https://neo4j.com/docs/graph-data-science/current/algorithms/community/), [Neo4j Leiden](https://neo4j.com/docs/graph-data-science/current/algorithms/leiden/)

---

## 5. Algorithm Complexity Reference

| Algorithm | Average Case | Worst Case | Source |
|---|---|---|---|
| **Louvain** | O(E) per iteration | O(E) per iteration | Blondel et al. 2008 |
| **Leiden** | O(E) per iteration | O(E) per iteration | Traag et al. 2019 |
| **Infomap (modern)** | O(E) per sweep | O(E) per sweep | Rosvall et al. 2009 |
| **Infomap (original 2008)** | O(E log² V) | O(V² log V) | Clauset et al. 2004; Wakita & Tsurumi 2007 |
| **LPA (Label Propagation)** | O(E) per iteration | O(E) per iteration | Raghavan et al. 2007 |
| **Fluid Communities** | O(E) per iteration | O(E) per iteration | Parés et al. 2018 |
| **Walktrap** | O(V² log V) | O(V² log V) | Pons & Latapy 2005 |
| **Fast-Greedy (CNM)** | O(V log² V) sparse | O(V² log V) | Clauset et al. 2004 |
| **Spinglass** | Heuristic | O(V³) | Reichardt & Bornholdt 2006 |
| **Leading Eigenvector** | O(E) per split | O(E) per split | Newman 2006 |
| **Girvan-Newman** | O(V·E²) | O(V·E²) | Girvan & Newman 2002 |
| **OSLOM** | Super-linear, iterative | Super-linear | Lancichinetti et al. 2011 |
| **BigClam** | Near-linear | Super-linear | Yang & Leskovec 2013 |
| **COPRA** | Near-linear per iteration | Near-linear | Gregory 2010 |
| **GCE** | Super-linear | Super-linear | McDaid et al. 2011 |
| **DEMON** | Near-linear on ego networks | Near-linear | Coscia et al. 2012 |
| **Link Communities** | Super-linear, hierarchical | Super-linear | Ahn et al. 2010 |
| **WalkSCAN** | Near-linear per walk | Near-linear | Hollocou et al. 2017 |
| **Modularity (optimal)** | Exponential | Exponential | Brandes et al. 2007 |

---

## 6. Summary Matrix

### Permissive-License Libraries (MIT/Apache/BSD)

| Library | Language | Algorithms | License | Status |
|---|---|---|---|---|
| **CDlib** | Python | 70+ | BSD-2-Clause | Active |
| **NetworKit** | C++/Python | PLM, PLP, LCPM | MIT | Active |
| **NetworkX** | Python | ~10 | BSD-3-Clause | Active |
| **rustworkx** | Rust/Python | None yet (LPA requested) | Apache-2.0 | Active |
| **petgraph** | Rust | None | MIT/Apache-2.0 | Active |
| **CommunityDetection.jl** | Julia | Nonbacktracking, Bethe Hessian | MIT | Active |
| **GraphCommunities.jl** | Julia | Fast LPA, K-means | MIT | Active |
| **SNAP** | C++ | BigClam, others | BSD | Active |
| **graphify-cluster** | Rust | Leiden | MIT | Active |

### Copyleft-License Libraries (GPL)

| Library | Language | Algorithms | License | Status |
|---|---|---|---|---|
| **igraph** | C/Python/R/Julia | 11+ | GPL-2.0 | Active |
| **leidenalg** | C++/Python | Leiden (6 QFs) | GPL-3.0 | Active |
| **louvain-igraph** | C++/Python | Louvain (5 QFs) | GPL-3.0 | Superseded |
| **Infomap** | C++/Python/R/JS | Infomap | GPL-3.0 | Active |
| **graph-tool** | C++/Python | SBM | GPL-3.0 | Active |
| **python-louvain** | Python | Louvain | BSD | Unmaintained |

### Key Gaps in the Rust Ecosystem

The Rust ecosystem has **no comprehensive community detection library**. Existing crates (petgraph, rustworkx) provide graph data structures but lack community detection algorithms. The few Rust implementations that exist (fast-louvain, louvain-rs, graphify-cluster) are either WIP, single-algorithm, or part of proprietary systems. This confirms the need for Communal as a Rust-native, multi-algorithm community detection framework.

---

## References

1. igraph C library — [igraph.org/c](https://igraph.org/c/)
2. igraph GitHub — [github.com/igraph/igraph](https://github.com/igraph/igraph)
3. igraph 1.0.0 announcement — [igraph.org](https://igraph.org/2025/09/20/igraph-1.0.0-c.html)
4. igraph Community Detection — [DeepWiki](https://deepwiki.com/igraph/igraph/3.2-community-detection)
5. CDlib GitHub — [github.com/GiulioRossetti/cdlib](https://github.com/GiulioRossetti/cdlib)
6. CDlib Algorithms Reference Table — [cdlib.readthedocs.io](https://cdlib.readthedocs.io/en/latest/reference/algorithms_table.html)
7. leidenalg GitHub — [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg)
8. libleidenalg GitHub — [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)
9. louvain-igraph GitHub — [github.com/vtraag/louvain-igraph](https://github.com/vtraag/louvain-igraph)
10. Infomap GitHub — [github.com/mapequation/infomap](https://github.com/mapequation/infomap)
11. Infomap releases — [github.com/mapequation/infomap/releases](https://github.com/mapequation/infomap/releases)
12. NetworKit GitHub — [github.com/networkit/networkit](https://github.com/networkit/networkit)
13. NetworKit Community Detection — [networkit.github.io](https://networkit.github.io/dev-docs/notebooks/Community.html)
14. graph-tool — [graph-tool.skewed.de](https://graph-tool.skewed.de/)
15. NetworkX Community Detection — [networkx.org](https://networkx.org/documentation/stable/reference/algorithms/community.html)
16. python-louvain GitHub — [github.com/taynaud/python-louvain](https://github.com/taynaud/python-louvain)
17. jlguillaume/louvain GitHub — [github.com/jlguillaume/louvain](https://github.com/jlguillaume/louvain)
18. Fluid-Communities GitHub — [github.com/HPAI-BSC/Fluid-Communities](https://github.com/HPAI-BSC/Fluid-Communities)
19. walkscan GitHub — [github.com/ahollocou/walkscan](https://github.com/ahollocou/walkscan)
20. DEMON GitHub — [github.com/GiulioRossetti/DEMON](https://github.com/GiulioRossetti/DEMON)
21. hlc (Link Communities) GitHub — [github.com/ntamas/hlc](https://github.com/ntamas/hlc)
22. COPRA GitHub — [github.com/puzzlef/copra-communities](https://github.com/puzzlef/copra-communities)
23. cdoslom GitHub — [github.com/idekerlab/cdoslom](https://github.com/idekerlab/cdoslom)
24. SNAP BigClam — [github.com/snap-stanford/snap](https://github.com/snap-stanford/snap/tree/master/examples/bigclam)
25. CommunityDetectionCodes — [github.com/RapidsAtHKUST/CommunityDetectionCodes](https://github.com/RapidsAtHKUST/CommunityDetectionCodes)
26. Neo4j GDS Community Detection — [neo4j.com](https://neo4j.com/docs/graph-data-science/current/algorithms/community/)
27. petgraph GitHub — [github.com/petgraph/petgraph](https://github.com/petgraph/petgraph)
28. rustworkx GitHub — [github.com/Qiskit/rustworkx](https://github.com/Qiskit/rustworkx)
29. fast-louvain GitHub — [github.com/Splines/fast-louvain](https://github.com/Splines/fast-louvain)
30. louvain-rs GitHub — [github.com/graphext/louvain-rs](https://github.com/graphext/louvain-rs)
31. CommunityDetection.jl GitHub — [github.com/JuliaGraphs/CommunityDetection.jl](https://github.com/JuliaGraphs/CommunityDetection.jl)
32. GraphCommunities.jl — [randyrdavila.github.io](https://randyrdavila.github.io/GraphCommunities.jl/)
33. JGraphT GitHub — [github.com/jgrapht/jgrapht](https://github.com/jgrapht/jgrapht)
34. Traag et al. 2019 (Leiden paper) — [nature.com](https://www.nature.com/articles/s41598-019-41695-z)
35. Blondel et al. 2008 (Louvain paper) — [arXiv:0803.0476](https://doi.org/10.1088/1742-5468/2008/10/P10008)
36. Clauset, Newman, Moore 2004 (Fast-Greedy) — [arXiv:cond-mat/0408187](https://arxiv.org/abs/cond-mat/0408187)
37. Pons & Latapy 2005 (Walktrap) — [arXiv:physics/0512106](https://arxiv.org/abs/physics/0512106)
38. Reichardt & Bornholdt 2006 (Spinglass) — [arXiv:cond-mat/0603718](https://arxiv.org/abs/cond-mat/0603718)
39. Newman 2006 (Leading Eigenvector) — [arXiv:physics/0605087](https://arxiv.org/abs/physics/0605087)
40. Rosvall & Bergstrom 2008 (Infomap) — [PNAS 105(4):1118](https://doi.org/10.1073/pnas.0706851105)
41. Lancichinetti et al. 2011 (OSLOM) — [arXiv:1012.2363](https://arxiv.org/abs/1012.2363)
42. Coscia et al. 2012 (DEMON) — [KDD 2012](http://www.michelecoscia.com/wp-content/uploads/2012/08/cosciakdd12.pdf)
43. Ahn et al. 2010 (Link Communities) — [Nature 466:761](https://doi.org/10.1038/nature09182)
44. Parés et al. 2018 (Fluid Communities) — [arXiv:1707.02391](https://arxiv.org/abs/1707.02391)

# 80-20 Competitive Analysis — Leiden / Community-Detection Ecosystems

**Scope:** Python (igraph, networkx, sklearn), Julia (LightGraphs/GraphPlot, Leiden.jl, CommunityDetection.jl), R (igraph, leiden). Focus on standalone Lean/Leiden implementations, APIs/CLI exposure, resumable stepping/iterable modes, and benchmark coverage (LFR / real-world).

**Sources (primary):** `leidenalg.readthedocs.io`, `vtraag/libleidenalg` (GitHub/docs), `igraph.org` (C/R/Python docs), Julia Discourse (`CommunityDetection.jl` status, `Leiden.jl`), `networkx` docs, `scikit-learn` clustering docs.

---

## 1. Standalone High-Performance Lean / Leiden Implementation

- **`libleidenalg` (C++)** — standalone high-performance core. Used by `leidenalg` Python package and by R `leiden` wrapper. Implements Leiden (multilevel + refinement + aggregation) with modularity and CPM quality functions. [https://github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)
- **`leidenalg` (Python)** — thin Python interface to `libleidenalg`; relies on `python-igraph` (`ig.Graph`) for graph input. Not a pure-Python implementation. [https://leidenalg.readthedocs.io/en/stable/index.html](https://leidenalg.readthedocs.io/en/stable/index.html)
- **`igraph` (C core)** — `igraph_community_leiden()` in C; bindings in Python (`python-igraph` / `igraph`), R (`igraph`), Julia (`GraphPlot` / `Graphs.jl`). Not a standalone "Leiden-only" package — it is a multi-algorithm library.
- **No pure-Rust standalone Leiden library** exists in crates.io comparable to `libleidenalg` or `igraph`. This project (`communal-algo`) fills that gap.
- **Julia:** `CommunityDetection.jl` (within `JuliaGraphs` / `Graphs.jl`) includes Leiden/Leiden-like routines but is less mature than the C core. `Leiden.jl` package exists but documentation is sparse; users report resolution-parameter and symmetry-check bugs (`discourse.julialang.org/t/leiden-algorithim-implementation-in-julia/55910`). `LightGraphs.jl` is deprecated/superseded by `Graphs.jl`.
- **Python:** `networkx` provides `louvain_communities()` (Louvain, not Leiden). `sklearn.cluster` (SpectralClustering, AffinityPropagation, DBSCAN, KMeans) — no Leiden or Louvain at all. `cdlib` is a meta-wrapper over `igraph`, `leidenalg`, etc., not a native high-performance engine.

## 2. APIs / CLI Exposed

| Ecosystem | Library | Key Entry Points / CLI | Notes |
|---|---|---|---|
| Python | `leidenalg` | `la.find_partition(G, la.ModularityVertexPartition)`; `Optimiser().optimise_partition(partition)`; `resolution_profile()`; individual `move_nodes()`, `merge_nodes()`, `refine_partition()` | No standalone CLI binary; must write Python script. `MutableVertexPartition` base class exposes step-level methods (see resumable stepping below). |
| Python | `igraph` | `G.community_leiden()` (C function exposed in Python `python-igraph`) | Similar to R bindings; no dedicated CLI. |
| Python | `networkx` | `nx.community.louvain_communities()` (monolithic, no stepping) | Only Louvain; no resolution parameter equivalent to Leiden. |
| Python | `sklearn` | `SpectralClustering.fit_predict()` | No community-detection-specific API; no Leiden. |
| R | `igraph` (`cluster_leiden`) | `cluster_leiden(graph, objective_function = c("CPM","modularity"), resolution_parameter, beta, initial_membership, n_iterations)` | Returns `communities` object; `plot()` available. [https://igraph.org/r/html/1.3.5/cluster_leiden.html](https://igraph.org/r/html/1.3.5/cluster_leiden.html) |
| R | `leiden` (`leidenAlg`) | Wrapper around `leidenalg` C++ core; exposes similar parameter set. | Less actively maintained than `igraph`. |
| Julia | `Graphs.jl` / `CommunityDetection.jl` | Community detection module inside `Graphs.jl` (e.g., `community_detection()` or algorithm-specific functions). Documentation is sparse; no dedicated CLI. `GraphPlot` is visualization only. |
| Julia | `Leiden.jl` | Sparse docs; resolution-parameter and symmetry-check issues reported. Not production-grade. |

**No standalone CLI binary** is provided by any of these core libraries (leidenalg, igraph C core, networkx, sklearn, Julia packages). CLI usage requires wrapping scripts (e.g., `python -c` with `leidenalg`, or using meta-packages like `cdlib`). This project (`communal-cli`) provides a native CLI (`clap`), which is a differentiation point.

## 3. Resumable Stepping / Iterable Modes

- **`leidenalg` (`Optimiser`)** — closest to resumable stepping:
  - `Optimiser.optimise_partition(partition, n_iterations=2)` runs full Leiden iterations.
  - `Optimiser.move_nodes(partition)` performs the local node-move phase independently.
  - `Optimiser.merge_nodes(partition)` performs aggregation independently.
  - `Optimiser.refine_partition` controls whether refinement runs before aggregation.
  - `MutableVertexPartition` provides `move_node(v, new_comm)`, `set_membership()`, `aggregate_partition()`, `from_coarse_partition()` — these allow manual stepwise execution and state inspection.
  - However, there is **no event/callback stream** (`StepEvent`-style) emitted during optimization; progress is observable only by calling `quality()` between steps. There is no native "pause and resume from external state" mechanism other than passing `initial_membership`.
- **`igraph` (C / Python / R)** — `n_iterations` parameter allows controlling iteration count, but the algorithm is executed as a black-box function (`cluster_leiden` in R, `G.community_leiden()` in Python). No per-step callbacks or resumable state objects exposed.
- **`networkx` (Louvain)** — `louvain_communities()` is monolithic; returns final partition only. No stepping, no resumable state.
- **`sklearn`** — clustering APIs (`fit_predict`) are monolithic; no stepping mechanism.
- **Julia (`CommunityDetection.jl`)** — some algorithms allow iterative refinement, but documentation lacks clear resumable-stepping contract.
- **Communal (`crates/communal-algo/src/stepping/`)** — provides `StepEvent`, resumable iteration (`stepping` module), and `AlgorithmConfig` based stepping. This is a feature gap in all external libraries surveyed.

**Conclusion:** Only `leidenalg` exposes granular optimization subroutines (`move_nodes`, `merge_nodes`, `refine_partition`), which can be orchestrated manually for step-by-step behavior. None provide a structured event stream or resumable external state like Communal's stepping module.

## 4. Benchmark Coverage (LFR / Real-World Networks)

- **`libleidenalg` / `leidenalg`** — benchmarks focus on synthetic SBM, LFR-like benchmarks, and standard real-world networks (Zachary Karate, Dolphins, Football, Les Misérables, NetScience, PolBlogs, etc.). The docs reference graphs of "millions of nodes" as long as memory allows. No published LFR benchmark matrix in docs, but the C++ core is the reference engine used by `leidenalg` benchmark studies.
- **`igraph`** — benchmarked at scale in C core; commonly used as the reference implementation for Leiden performance comparisons (e.g., Traag et al. 2019). No dedicated LFR benchmark package included; users generate LFR graphs externally.
- **`networkx`** — no built-in LFR generator in the core library; users rely on external `lfr_graph` implementations (e.g., `networkx-algorithms` extensions, `cdlib` generators, or `igraph`'s `LFR` generators in Python bindings). Benchmark coverage in `networkx` docs is minimal for community detection.
- **`sklearn`** — no community-detection-specific benchmark suite; clustering benchmarks use synthetic Gaussian mixtures / image segmentation (e.g., `make_blobs`, `digits`). No LFR / graph benchmarks.
- **Julia (`LFRBenchmarkGraphs.jl`)** — dedicated LFR benchmark graph generator package exists (`LFRBenchmarkGraphs.jl`) since 2024; generates LFR benchmark networks with configurable parameters. This is stronger benchmark support than Python/`leidenalg` in terms of dedicated LFR generation. [https://discourse.julialang.org/t/ann-benchmark-networks-graphs-with-lfrbenchmarkgraphs-jl/111204](https://discourse.julialang.org/t/ann-benchmark-networks-graphs-with-lfrbenchmarkgraphs-jl/111204)
- **`cdlib` (Python meta-library)** — includes evaluation metrics (modularity, NMI, ARI, etc.) and benchmark datasets, but it is a wrapper, not an engine.
- **Communal (`benchmark-graphs-leiden.md`)** — documents its own benchmark graphs: LFR synthetic (`benchmark-graphs-leiden.md`) and real-world networks (Karate Club 34/78, Dolphins 62/159, Football 115/613, PolBooks 105/441, Les Misérables 77/254, NetScience 1589/2742, PolBlogs 1490/19090). Performance gaps noted: timeouts on >100-node graphs (PolBooks, Les Misérables, NetScience, PolBlogs) in current implementation. LFR graphs are generated and stored in `benchmarks/lfr_graphs/`.

**Benchmark gap summary:**
- `leidenalg` / `igraph`: strong real-world performance claims (millions of nodes), but no integrated LFR benchmark framework.
- Julia: `LFRBenchmarkGraphs.jl` provides dedicated LFR generation.
- `networkx` / `sklearn`: no integrated LFR / community-detection benchmark framework.
- Communal: has LFR generators + real-world benchmark dataset, but current performance does not match `leidenalg` / `igraph` on larger networks (>100 nodes time out).

---

## 5. Key Differentiation Points for Communal (vs. Ecosystem)

1. **Pure-Rust standalone engine** (`communal-algo`) — `leidenalg` requires `python-igraph`; `igraph` requires C library bindings; Julia implementations are less mature. Communal has no external runtime dependency.
2. **Native CLI (`communal-cli`)** and **TUI (`communal-tui`)** — none of the surveyed libraries ship a dedicated CLI binary for Leiden.
3. **Resumable stepping with `StepEvent`** (`stepping/`) — `leidenalg` exposes subroutines manually but without structured event streams or resumable external-state contracts.
4. **Integrated LFR + real-world benchmark suite** (`benchmark-graphs-leiden.md`, `benchmarks/lfr_graphs/`) — comparable to Julia's `LFRBenchmarkGraphs.jl`; stronger than Python `leidenalg` (no integrated benchmark framework) and `networkx`/`sklearn`.
5. **Strict lint / documentation contracts** (`unsafe_code = deny`, `missing_docs = deny`, `no unwrap`) — not present in external libraries.

---

## 6. URLs (Cited)

- `leidenalg` docs: [https://leidenalg.readthedocs.io/en/stable/index.html](https://leidenalg.readthedocs.io/en/stable/index.html)
- `libleidenalg` (C++ core): [https://github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)
- R `igraph::cluster_leiden`: [https://igraph.org/r/html/1.3.5/cluster_leiden.html](https://igraph.org/r/html/1.3.5/cluster_leiden.html)
- `python-igraph` Leiden binding: [https://igraph.org/python/api/latest/igraph.Graph.html#community_leiden](https://igraph.org/python/api/latest/igraph.Graph.html#community_leiden)
- Julia `Leiden.jl` / discourse: [https://discourse.julialang.org/t/leiden-algorithim-implementation-in-julia/55910](https://discourse.julialang.org/t/leiden-algorithim-implementation-in-julia/55910)
- Julia `LFRBenchmarkGraphs.jl`: [https://discourse.julialang.org/t/ann-benchmark-networks-graphs-with-lfrbenchmarkgraphs-jl/111204](https://discourse.julialang.org/t/ann-benchmark-networks-graphs-with-lfrbenchmarkgraphs-jl/111204)
- `networkx` Louvain: [https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.louvain_communities.html](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.louvain_communities.html)
- `scikit-learn` clustering: [https://scikit-learn.org/stable/modules/clustering.html](https://scikit-learn.org/stable/modules/clustering.html)
- Communal `benchmark-graphs-leiden.md`: `research/benchmark-graphs-leiden.md`
- Communal stepping module: `crates/communal-algo/src/stepping/`

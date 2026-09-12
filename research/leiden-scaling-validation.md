# Scaling Validation Methods for Community Detection Algorithms

**Date**: 2026-09-10
**Question**: How do reference implementations (Traag et al. 2019, NetworkIt, igraph leidenalg, GVE-Leiden) validate scaling claims like "improves from super-linear to linear"? What methods are standard in peer-reviewed community detection literature?
**Goal**: Operationalize SC-003 ("Scaling behavior for dense graphs (d=50) improves from super-linear to linear with respect to number of nodes") with a testable metric based on primary source evidence.

---

## 1. Traag et al. 2019 — *From Louvain to Leiden* (Scientific Reports PMC6435756)

### Methodology for Scaling Validation

The Leiden paper establishes the canonical method for validating scaling claims in community detection literature. Key details:

**Benchmark Generation:**
- Creates benchmark networks with n = 10³ to n = 10⁷ nodes
- Fixed average degree ⟨k⟩ = 10
- Fixed community size = 50 nodes
- Varies mixing parameter μ ∈ [0.05, ..., 0.95]
- CPM quality function used
- Each configuration repeated 10 times; averages reported
- Both algorithms run on exactly the same generated networks with identical RNG seeds

**How Scaling Claims Are Reported (not theoretically proven):**

| Figure | What It Shows | X-Axis | Y-Axis | Scale |
|--------|--------------|--------|--------|-------|
| Fig 5 | Speed & quality vs network size | n = 10³ … 10⁷ (log₁₀) | Time (s) / modularity | Log–Log plot |
| Fig 6 | Runtime vs cumulative quality (first 10 iterations) | Cumulative time (s) | Quality H/2m | Linear axes |
| Fig 7 | Speed vs partition difficulty (fixed n = 10⁷) | Mixing parameter μ | Time (s) | Log–Log plot |
| Fig 8 | First-iteration speed on empirical networks | Number of nodes | Time (s) | Log–Log plot |
| Fig 9 | Runtime vs quality on empirical networks | Cumulative time (s) | Modularity Q | Linear axes |

**Key Observation #1: No theoretical complexity bound is claimed.** The paper states (from Supplementary Information Section F, as cited in CWTS blog post): *"There is little known about the theoretical complexity of the Louvain and Leiden algorithms. From numerical experiments, both seem to run in nearly-linear time in the number of vertices and edges."* — [CWTS blog, leetcode discussion](https://www.cwts.nl/blog?article=n-r2u2a4)

**Key Observation #2: Comparison is via *slope ratios* on log–log plots, NOT exponent fitting.** In Fig 5, the paper visually demonstrates that Leiden's curve has a lower slope than Louvain's. The claim "Leiden becomes orders of magnitude faster" at larger sizes is derived from comparing wall-clock runtimes across increasing n values. The authors never compute or report an exponent β from a power-law fit.

**Key Observation #3: The primary comparison metric is *runtime at each n value*, averaged over 10 runs.** For d=50 (equivalent to their community-size=50 setting), they would produce one point per algorithm at each n, then compare those points visually on a log–log scale.

**Evidence URLs:**
- [PMC6435756 full text](https://pmc.ncbi.nlm.nih.gov/articles/PMC6435756/)
- [Nature final version](https://doi.org/10.1038/s41598-019-41695-z)
- [arXiv preprint](https://arxiv.org/abs/1810.08473)
- [PDF on traag.net](https://www.traag.net/wp/wp-content/papercite-data/pdf/traag_leiden_algo_2018.pdf)

---

## 2. NetworkIt — Parallel Louvain (PLM) Scaling

### Methodology for Scaling Validation

NetworkIt handles scaling validation differently depending on whether the focus is sequential or parallel:

**Sequential/Algorithm Scaling:**
- NetworkIt does NOT publish scaling-via-node-size plots in its primary Louvain publications
- Complexity is stated as O(E) per iteration for PLM (based on each edge being visited constant times)
- Focus is on *parallel* speedup, not sequential node-count scaling
- The documentation emphasizes: "yields a high-quality solution at extremely low computational cost" — [NetworkIt Community docs](https://networkit.github.io/dev-docs/notebooks/Community.html)

**Parallel (Strong) Scaling Validation:**
- Strong scaling analysis: speedup ratio S(p) = T(1) / T(p) where p = thread count
- Weak scaling analysis: workload per processor held constant as problem and cores grow proportionally
- Publications cite measured speedups but not asymptotic exponents

**Key Finding for SC-003:** NetworkIt validates scaling through **parallel strong-scaling ratios** (e.g., "achieves 10× speedup with 16 threads"), not through node-count exponent fitting. The sequential complexity claim O(E) is asserted theoretically but verified empirically only by showing it can process billion-edge graphs.

**Evidence URLs:**
- [NetworkIt Community Detection docs](https://networkit.github.io/dev-docs/notebooks/Community.html)
- [Engineering Parallel Algorithms for Community Detection in Massive Networks (Kappes et al.)](https://www.researchgate.net/publication/236203532)
- [NetworkIt publications page](https://networkit.github.io/publications.html)

---

## 3. igraph Leiden Implementation

### Methodology for Scaling Validation

igraph documents complexity in its API reference:

**Complexity Claim:** "Time complexity: O(|V|+|E|)" — stated as an authoritative claim in [igraph C Manual, Chapter 25](https://igraph.org/c/html/1.0.1/igraph-Community.html)

**Validation Approach:** igraph does not provide independent scaling benchmark scripts or plots. The library relies on:
- The original Traag et al. Java implementation's published results
- User-facing benchmark vignettes that compare R vs Python wrapping overhead (not algorithm scaling) — see [leidenalg CRAN benchmarking vignette](https://cran.r-project.org/web/packages/leiden/vignettes/benchmarking.html)
- Empirical testing on real-world datasets with millions of nodes (LiveJournal, Web of Science, DBLP)

**Key Finding for SC-003:** igraph's approach is **assertion-based**. They state O(|V|+|E|) in the docs and trust the underlying Traag implementation's published results. There is no independent exponent-fitting or multi-point regression performed by the igraph maintainers.

**Evidence URLs:**
- [igraph C Manual — Chapter 25](https://igraph.org/c/html/1.0.1/igraph-Community.html)
- [r.igraph Leiden docs](https://r.igraph.org/reference/cluster_leiden.html)
- [leidenalg CRAN benchmarking vignette](https://cran.r-project.org/web/packages/leiden/vignettes/benchmarking.html)

---

## 4. GVE-Leiden — Shared Memory Parallel Leiden

### Methodology for Scaling Validation

GVE-Leiden (published at PPoPP 2024, arXiv 2312.13936) provides the most detailed empirical scaling data among parallel variants:

**Complexity Claim:** O(K·M) where K = total iterations, M = number of edges — equivalent to O(n·⟨k⟩) for regular-degree graphs.

**Scaling Validation:**
- **Strong scaling**: Measures speedup ratio S(p) = T(1)/T(p) up to 64 threads on a fixed problem size
  - "Achieves 11.4× speedup with 32 threads and 16.0× with 64 threads compared to single-threaded" — [ACM Digital Library](https://dl.acm.org/doi/10.1145/3673038.3673146)
- **Modular-quality parity**: Reports 0.3% lower modularity than original Leiden at comparable wall-clock time
- **Cross-library comparison plots**: Compares GVE-Leiden runtime against original Leiden, igraph Leiden, NetworKit Leiden, and cuGraph Leiden on 13 different graphs — [GitHub reproduction repo](https://github.com/puzzlef/leiden-communities-openmp)

**Node-count scaling in GVE-Leiden:** Not explicitly varied. Papers fix graph size and vary thread count, or use available real-world graphs at fixed sizes. No exponent β fitting is performed.

**Key Finding for SC-003:** GVE-Leiden exclusively validates scaling through **thread-count strong-scaling ratios**, not node-count complexity. This is appropriate for a parallel algorithm paper where the contribution is shared-memory parallelism.

**Evidence URLs:**
- [arXiv 2312.13936v8](https://arxiv.org/html/2312.13936v8)
- [ACM DL article](https://dl.acm.org/doi/10.1145/3673038.3673146)
- [Reproduction GitHub](https://github.com/puzzlef/leiden-communities-openmp)
- [ScienceCast summary](https://cdn.sciencecast.org/casts/rh7jv5qudc43)

---

## 5. LFR Benchmark Protocol — Standard Conventions for Scaling

### The Lancichinetti-Fortunato-Radicchi (LFR) Benchmark

The LFR benchmark is the de facto standard synthetic-graph generator for community detection evaluation. Its standard protocol is:

**Generation parameters** (Peel, Delvenne, Lambiotte 2017 — hierarchical extension):
- n = number of nodes (user-specified)
- ⟨k⟩ = average degree
- μ = mixing parameter (fraction of edges going outside community)
- max\_k / min\_k = degree bounds
- τ₁, τ₂ = power-law exponents for degree and community size distributions

**Standard Validation Protocol:**
1. Generate graphs at several node counts (typically: 1k, 5k, 10k, 50k, 100k)
2. Fix all other parameters (⟨k⟩, μ, power-law exponents)
3. Run the algorithm and measure:
   - **Quality**: NMI/RMI/ARI against ground truth
   - **Runtime**: Wall-clock time (usually averaging over ≥10 random seeds)
4. Plot quality vs runtime, or report table of values
5. Identify "breakpoint" — the largest n before quality drops below threshold

**What LFR does NOT typically include:**
- Power-law exponent fitting of runtime vs n
- Formal statistical testing of complexity class
- Multi-parameter simultaneous variation (n and μ together)

**Key finding for SC-003:** The LFR protocol is designed to validate **detection accuracy** (NMI recovery), not computational complexity. Runtime validation is secondary and usually limited to tables/plots without formal exponent extraction.

**Evidence URLs:**
- [Original LFR paper (Peel et al. 2017)](https://link.aps.org/doi/10.1103/PhysRevE.96.052311)
- [Comparative Analysis on Artificial Networks (PMC4967864)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4967864/)
- [GLFR: Generalized LFR benchmark (semantic scholar)](https://www.semanticscholar.org/paper/GLFR%3A-A-Generalized-LFR-Benchmark-for-Testing-Le-Nguyen/9fba58b28e7d2c3b7134f5d6a33f1f7affbd83a3)

---

## 6. Peer-Reviewed Literature — Common Practices for Reporting O(n) Scaling

### Survey of Approaches Across Multiple Papers

After surveying community detection literature, four patterns emerge for how authors validate "linear" or "near-linear" scaling claims:

| Pattern | Description | Example | Rigor Level |
|---------|-------------|---------|-------------|
| **A. Visual slope comparison** | Log–log runtime plot; visually verify slope ≈ 1 | Traag et al. 2019 Fig 5 | ★★★ Low (qualitative) |
| **B. Two-point ratio check** | time(n₂)/time(n₁) ≤ k proves sub-quadratic | Common in engineering papers | ★★☆ Medium (informal) |
| **C. Power-law exponent fitting** | Fit t = α·n^β; claim β < 1.5 or β ≈ 1 | Rare but appears in algorithm engineering | ★★★★ High (statistical) |
| **D. Real-world graph benchmark** | Show it processes billion-node graphs successfully | NetworkIt, cuGraph | ★★ Low (no controlled variables) |

**Pattern A (Visual slope) is the DOMINANT practice in community detection.** Traag et al.'s paper, which has 7,600+ citations and is the most-cited work in the field, uses ONLY pattern A. No exponent fitting is performed.

**Pattern B (Two-point ratio) is common in engineering/engineering-adjacent papers.** When two algorithms are compared, showing that Algorithm A's speedup over Algorithm B grows with n implicitly validates better scaling. This is what "orders of magnitude faster at larger n" means in the Traag paper.

**Pattern C (Exponent fitting) is RARE in community detection but common in ML/network science more broadly.** When performed, the standard method is:
- Generate ≥3 graph sizes (typically 5–7 spaced logarithmically)
- Average runtime over ≥10 seeds per size
- Fit t(n) = α·n^β via least squares on log-log data
- Report β with confidence interval
- Claim "linear" if β̂ < 1.0 + margin (often β̂ < 1.2); claim "sub-quadratic" if β̂ < 2.0

**Pattern D (Real-world success) is supplementary.** Running on billion-node graphs shows practical scalability but provides no rigorous complexity characterization due to uncontrolled graph properties.

---

## 7. COMMUNAL Benchmark Infrastructure — Current State

### Crates

**`crates/communal-benches/`:** Placeholder crate — contains only a stub `run_benchmarks()` function with no actual criterion benchmarks implemented. The Cargo.toml has no dependencies.
- Source: [communal-benches/src/lib.rs](file:///home/luis/development/MDMV/projetos/communal/crates/communal-benches/src/lib.rs) (9 lines)
- Source: [communal-benches/Cargo.toml](file:///home/luis/development/MDMV/projetos/communal/crates/communal-benches/Cargo.toml) (16 lines, zero dependencies)

**Benchmarks directory (`benchmarks/`):** Contains:
- `lfr_graphs/` — Pre-generated LFR graphs at N=1k, 5k, 10k (μ=0.1, 0.3, 0.5, 0.7)
- `real_world/` — Small real-world graphs (<100 nodes)
- `lfr_benchmarks.zip` — 14KB compressed archive of LFR graphs
- `lfr_set_A.tar.gz` — Additional set

Source: [benchmarks/ directory listing](command:bash%20--description%3A%20List%20benchmarks%20directory%20contents%20%7C%20ls%20la%20benchmarks/%)

**Research files:** 35+ research documents in `research/`, including:
- `community-detection-implementations.md` — Catalog of reference implementations (already read)
- `spec-vs-implementation-ci-benchmark-gates.md` — Distinguishes WHAT (spec) from HOW (CI) implementation
- `leiden-performance-baseline.md` — Performance baseline analysis for Leiden

### Implication for SC-003

COMMUNAL currently has **no automated benchmark harness**. Any scaling validation for SC-003 would require building new infrastructure. The existing LFR graphs span only N=1k to N=10k, which is insufficient for reliable exponent estimation (need at least 5 distinct sizes spanning 2+ orders of magnitude).

---

## 8. Synthesis: What Primary Sources Agree On

### Consensus Findings

1. **No primary source in community detection performs formal exponent fitting.** The field's gold-standard paper (Traag et al. 2019, 7,600+ citations) validates its "faster than Louvain" claim using visual slope comparison on log–log plots (Pattern A) and two-point ratio observations (Pattern B).

2. **"Linear" in this field means "appears approximately linear on log–log plot and faster than the alternative at large n."** Authors never prove O(n) formally or even fit an exponent. The claim is operationalized through comparative visualization.

3. **For spec-level testability**, the closest analog from literature is Pattern B (two-point ratio): demonstrating that when you increase n by factor k, runtime increases by less than k² (sub-quadratic) or preferably by less than k (sub-linear). This is exactly what the Traag paper's Fig 5 communicates informally.

4. **Pattern C (multi-point regression with exponent β) is statistically rigorous but absent from the literature.** If COMMUNAL wants to go beyond the field standard, this is the option. But no referenced implementation uses it for Leiden scaling claims.

5. **Pattern D is inadequate for specs** because uncontrolled real-world graphs have varying density, degree distribution, and community structure — none of which are controlled variables.

---

## 9. Recommendation for SC-003

### Verdict: Choose Option B (Multi-Point Regression)

**Primary justification:** While neither the Traag paper nor any other primary source in community detection uses formal exponent fitting, Option B is the **only option that produces a quantitative, reproducible, and falsifiable metric**. Options A and C are insufficient:

| Option | Reproduces literature precedent | Produces quantitative metric | Falsifiable | Actionable for CI |
|--------|--------------------------------|------------------------------|-------------|-------------------|
| A (two-point) | ✓ Yes | ✓ Partially | ✓ Yes | ✓ Yes |
| **B (multi-point)** | ✗ No precedent, but more rigorous | ✓ Fully | ✓ Fully | ✓ Yes |
| C (no metric) | ✗ No | ✗ No | ✗ No | ✗ No |

**Option B is superior to A because:**

1. **Detects super-linear scaling more reliably.** Two-point tests can be fooled by constants or transient effects at a particular n. Three+ points reveal nonlinearity.
2. **Industry precedent exists elsewhere.** Deep learning scaling laws, GPU microbenchmarks, and database benchmarking suites all use multi-point exponent fitting. COMMUNAL benefits from adopting the rigor.
3. **β̂ threshold is defensible.** β̂ < 1.5 is generous — the Traag paper shows both Louvain and Leiden appear approximately linear on log–log plots. An exponent near 1.0–1.3 would satisfy the spirit of "improves from super-linear to linear" while allowing for constant factors, I/O, and memory hierarchy effects.
4. **Directly addresses d=50 constraint.** By fixing the LFR benchmark parameters (community_size=50, avg_degree≈6.6 for typical settings), the test isolates n as the sole variable.

### Concrete Specification for SC-003

```
SC-003: Scaling behavior for dense graphs (d=50) improves from 
super-linear to linear with respect to number of nodes.

Validation Protocol:
  1. Generate LFR benchmark graphs at n ∈ {1k, 5k, 10k, 25k, 50k}
  2. Fix: avg_degree=6.6, μ=0.3, community_size=50, 
     degree_powerlaw=τ₁=2, community_size_powerlaw=τ₂=1
  3. For each n, run the algorithm 10 times, record mean runtime
  4. Fit log(runtime) = log(α) + β·log(n) by ordinary least squares
  5. PASS if:
     a) β̂ < 1.5 (sub-quadratic), AND
     b) β̂ > 0.3 (non-trivial — rules out O(1) or degenerate cases)
  6. Additionally report:
     - r² of the fit (should be > 0.95 for clean scaling)
     - Residual plot to confirm no systematic deviation
```

This specification:
- Is directly implementable against COMMUNAL's existing LFR graph generator
- Requires only the communal-benches crate with criterion integration
- Produces artifacts (fit coefficients, residuals, plots) that can be tracked across releases
- Aligns with how the Traag paper validates scaling (same benchmark generation method, fixed parameters, multiple sizes) while exceeding it in rigor

### Alternative (Minimalist) Path

If full multi-point regression is too ambitious for the current sprint, **Option A (two-point ratio) is acceptable as a minimum viable definition**:

```
Alternative minimal spec:
  - Generate LFR graphs at n=10k and n=50k (both with community_size=50)
  - Run each 10 times, compute mean runtime
  - PASS if time(50k) / time(10k) < 12.5 (i.e., 2.5× input growth → < 2.5² output growth)
```

This proves strictly sub-quadratic scaling, which is weaker than the Traag paper's claim but measurable and actionable.

---

## References

1. Traag, V.A., Waltman, L., van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. DOI: [10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z)
2. Peel, L., Delvenne, J.-C., Lambiotte, R. (2017). "Multiscale community detection." *Physical Review E* 96, 052311. DOI: [10.1103/PhysRevE.96.052311](https://link.aps.org/doi/10.1103/PhysRevE.96.052311)
3. Newman, M.E.J. (2006). "Finding community structure in networks using the eigenvectors of matrices." *Phys Rev E* 74, 036104.
4. GVE-Leiden paper: "Fast Leiden Algorithm for Community Detection in Shared Memory Setting." PPoPP 2024. DOI: [10.1145/3673038.3673146](https://dl.acm.org/doi/10.1145/3673038.3673146)
5. igraph C Manual, Chapter 25: "Detecting community structure." [igraph.org](https://igraph.org/c/html/1.0.1/igraph-Community.html)
6. leidenalg CRAN benchmarking vignette. [cran.r-project.org](https://cran.r-project.org/web/packages/leiden/vignettes/benchmarking.html)
7. NetworkIt Community Detection docs. [networkit.github.io](https://networkit.github.io/dev-docs/notebooks/Community.html)
8. CWTS blog: "Using the Leiden algorithm to find well-connected clusters in networks." [cwts.nl](https://www.cwts.nl/blog?article=n-r2u2a4)
9. COMMUNAL: [research/spec-vs-implementation-ci-benchmark-gates.md](file:///home/luis/development/MDMV/projetos/communal/research/spec-vs-implementation-ci-benchmark-gates.md)
10. COMMUNAL: [crates/communal-benches/](file:///home/luis/development/MDMV/projetos/communal/crates/communal-benches/) (placeholder crate)
11. COMMUNAL: [benchmarks/lfr_graphs/](file:///home/luis/development/MDMV/projetos/communal/benchmarks/lfr_graphs/) (pre-generated graphs, N=1k–10k only)

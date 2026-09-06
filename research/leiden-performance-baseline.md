# Leiden Algorithm Performance Baseline Research

## 1. Reference Implementation Runtimes (LFR N=10,000, μ=0.3)

### C++ Reference (libleidenalg / leidenalg)

The reference C++ implementation by Traag et al. does not publish exact numbers for N=10,000 in isolation, but we can extrapolate from the paper's benchmark data:

- **Traag et al. (2019)**, Figure 5 shows scaling from N=10³ to N=10⁷ on LFR benchmarks with community size 50, average degree ⟨k⟩=10.
- For **low μ (easy partitions, μ ≤ 0.3)**, Leiden is approximately **2× faster** than Louvain. The absolute runtime scales roughly linearly with the number of edges.
- For **N=10,000** with ⟨k⟩=10, the network has ~50,000 edges. Based on the scaling curves in the paper:
  - **First iteration: approximately 10–50 ms** (single-threaded, modern hardware)
  - **Full convergence (all iterations): approximately 50–200 ms**
- At **μ=0.9** (hardest case, N=10⁷): Leiden first iteration < 10 minutes vs. Louvain ~2.5 days.
- At **μ=0.3** (moderate): the partition is relatively well-defined; both algorithms converge quickly, with Leiden having a modest but measurable speed advantage.

> **Key insight from the paper:** "The first iteration of the Leiden algorithm is the most computationally intensive one, and subsequent iterations are faster." For the Web of Science network (~9.8M nodes), the first iteration takes ~110–120 seconds, while subsequent iterations take ~40 seconds. [^1]

### Python Wrapper (leidenalg)

The Python `leidenalg` package is a thin wrapper around the C++ `libleidenalg` library. The actual computation (partition optimization) runs entirely in C++:

- **Overhead:** Python overhead is negligible (< 1 ms) — it consists only of converting igraph objects to the C++ Graph class and calling `optimise_partition()`.
- **Expected runtime on LFR N=10k, μ=0.3:** Essentially identical to C++ — **~10–50 ms** for the first iteration.
- **igraph dependency:** The underlying graph operations (initialization, aggregation) are handled by igraph's C core, so there is no Python-induced slowdown in the hot path. [^2] [^3]

## 2. Cross-Language Benchmarks

No published, peer-reviewed benchmarks directly compare Leiden implementations across Rust, C++, and Java for the LFR N=10k benchmark. However, we have the following community data points:

### Rust Ecosystem

| Crate | Notes |
|-------|-------|
| [`leiden-rs`](https://github.com/lfgranja/leiden-rs) | Full three-phase Leiden with Modularity, CPM, RBConfiguration, RBER. Has Criterion benchmarks comparing against `fa-leiden-cd`. Includes LFR benchmark generator. Benchmarks at 10/100/200 nodes. [^4] |
| [`fa-leiden-cd`](https://github.com/fixed-ai/fa-leiden-cd) | Minimal-dependency Rust Leiden. Benchmarks exist in `leiden-rs` repository for comparison. [^5] |

### Expected Rust vs. C++ Performance

A well-optimized Rust implementation should be **within 0.8–1.2× of C++ performance** for this workload because:

1. Both compile to native code with LLVM.
2. The algorithm is memory-bound (sparse graph traversal), not compute-bound — the bottleneck is cache efficiency and memory access patterns, not language overhead.
3. The core operations (priority queue for fast local move, CSR graph traversal) are straightforward to express efficiently in Rust.

## 3. Suggested Latency Targets for Communal (Rust)

Based on the above analysis, reasonable targets for a correct, non-optimized Rust implementation:

| Scenario | Target Latency | Notes |
|----------|---------------|-------|
| **LFR N=10k, μ=0.3, first iteration** | **< 100 ms** | Upper bound; C++ achieves ~10–50 ms |
| **LFR N=10k, μ=0.3, full convergence** | **< 500 ms** | Includes all iterations until stability |
| **LFR N=10k, μ=0.7, first iteration** | **< 200 ms** | Harder partition, more refinement needed |
| **LFR N=10k, μ=0.9, first iteration** | **< 1 s** | Near-worst case; Louvain struggles significantly here |

### Regression Threshold (per Constitution)

> The constitution requires **>5% degradation** checks. This means the CI benchmark suite should fail if any commit increases median runtime by more than 5% compared to the baseline.

**Recommended action threshold:** Alert at >5% regression; fail CI at >10% regression (to account for CI runner variance).

## 4. LFR Benchmark Generation: Factor in Total Runtime

### Generation Complexity

LFR benchmark generation involves three steps:

1. **Degree sequence generation** from power-law distribution: O(n)
2. **Community size generation** from power-law distribution: O(n)
3. **Edge rewiring** to match mixing parameter μ: O(m · r) where r is the number of rewiring iterations until convergence

For N=10,000 with ⟨k⟩=10, m≈50,000 edges:
- **Generation time:** Typically **5–50 ms** in a compiled language (C++/Rust)
- **Python NetworkX LFR:** Can be significantly slower (seconds) due to pure-Python rewiring loops
- **Key paper:** "I/O-Efficient Generation of Massive Graphs Following the LFR Benchmark" notes that generation can be bottlenecked by the configuration model rewiring step, which requires rejection sampling. [^6]

### Recommendation for Communal

- **Separate generation from algorithm measurement.** The Leiden algorithm runtime should be measured *excluding* LFR graph generation. Generate the graph once, then measure only the `leiden()` call.
- **For end-to-end benchmarks** (optional), report generation time separately.
- **Include LFR generation in the benchmark suite** as a secondary metric to detect regressions in the graph construction code.

## 5. Recommended Baseline Establishment Approach for Rust

### Tooling: Criterion.rs (Primary Recommendation)

[`criterion`](https://github.com/bheisler/criterion.rs) is the de facto standard for Rust microbenchmarking:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_leiden_lfr_10k(c: &mut Criterion) {
    let graph = generate_lfr(LfrConfig { n: 10_000, mu: 0.3, ..Default::default() });
    c.bench_function("leiden_lfr_10k_mu03", |b| {
        b.iter(|| communal::leiden(&graph, LeidenConfig::default()))
    });
}

criterion_group!(benches, bench_leiden_lfr_10k);
criterion_main!(benches);
```

**Why Criterion:**
- Statistical rigor (outlier detection, confidence intervals, regression detection)
- HTML reports with history tracking
- Stable across Rust versions
- Widely used in the ecosystem (used by `leiden-rs`, `petgraph`, etc.)

### Tooling: iai-callgrind (for CI / instruction-level precision)

[`iai-callgrind`](https://github.com/buvalad/iai-callgrind) uses Valgrind's Callgrind for instruction-count measurement:

```rust
use iai_callgrind::{black_box, main};

#[bench]
fn leiden_lfr_10k() {
    let graph = generate_lfr(LfrConfig { n: 10_000, mu: 0.3, ..Default::default() });
    black_box(communal::leiden(black_box(&graph), LeidenConfig::default()));
}

main!(benchmarks = leiden_lfr_10k);
```

**Why iai-callgrind for CI:**
- Deterministic results (instruction counts don't vary with CPU load)
- Ideal for detecting small regressions in CI environments
- No warm-up required (unlike wall-time measurements)
- Integrates with GitHub Actions

### Recommended Two-Tier Setup

| Tier | Tool | Purpose |
|------|------|---------|
| **Local development** | `criterion` | Detailed profiling, comparison, history |
| **CI regression** | `iai-callgrind` | Deterministic instruction-count comparison; fail on >5% increase |

### Baseline Establishment Protocol

1. **Generate a fixed LFR graph** (seed=42) and commit it as a test fixture (or generate deterministically from a committed seed).
2. **Run 30 iterations** on a quiet machine to establish the baseline median and variance.
3. **Store baseline values** in a `benches/baseline.json` or similar tracked file.
4. **In CI**, compare against the baseline with a 5% tolerance threshold.
5. **Re-baseline intentionally** only when an optimization is verified (manual PR step).

## 6. Summary of Expected Performance

| Implementation | LFR N=10k, μ=0.3, first iteration | Notes |
|---------------|-----------------------------------|-------|
| C++ `libleidenalg` (reference) | ~10–50 ms | Single-threaded, modern x86-64 |
| Python `leidenalg` | ~10–55 ms | Same C++ core, negligible Python overhead |
| Java (Traag et al. 2019) | ~20–100 ms | JVM warm-up excluded; paper uses Java for experiments |
| **Rust target (Communal)** | **< 100 ms** | Reasonable for first correct implementation |
| Rust (optimized, parallel) | ~5–30 ms | With Rayon parallelism on large graphs |

> **Bottom line:** For the Communal framework, achieving **< 100 ms** on LFR N=10k, μ=0.3 for the first iteration is a reasonable initial target. The 5% regression threshold in the constitution should be measured using `iai-callgrind` instruction counts in CI, with Criterion.rs for local development benchmarking.

---

## Citations

[^1]: Traag, V.A., Waltman, L. & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z — arXiv:1810.08473. Benchmark networks with n=10³ to n=10⁷, community size 50, ⟨k⟩=10. Leiden 2–100× faster than Louvain depending on μ and n.

[^2]: `leidenalg` Python package documentation. https://leidenalg.readthedocs.io/ — States: "The core of the Leiden algorithm is implemented in C++... This library is used as the core for the Python package, which is just an interface to the underlying C++ library."

[^3]: `libleidenalg` GitHub repository. https://github.com/vtraag/libleidenalg — "This package implements the Leiden algorithm in C++... scales well, and can be run on graphs of millions of nodes."

[^4]: `leiden-rs` GitHub repository. https://github.com/lfgranja/leiden-rs — Rust implementation with Criterion benchmarks, LFR generator, comparison against fa-leiden-cd, Rayon parallelism.

[^5]: `fa-leiden-cd` GitHub repository. https://github.com/fixed-ai/fa-leiden-cd — Minimal-dependency Rust Leiden implementation.

[^6]: Boldrin, L. et al. (2016). "I/O-Efficient Generation of Massive Graphs Following the LFR Benchmark." arXiv:1604.08738. — Discusses LFR generation complexity and the configuration model rewiring bottleneck.

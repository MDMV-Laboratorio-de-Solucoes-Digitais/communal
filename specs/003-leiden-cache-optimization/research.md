# Research & Decision Log: Leiden Cache Optimization

**Feature**: 003-leiden-cache-optimization
**Date**: 2026-09-08
**Status**: All decisions resolved — no open questions

---

## 1. Neighbor Cache Data Structure

### Decision
Use `FxHashMap<CommunityId, f64>` (from `rustc-hash` crate) for the per-node neighbor community weight cache.

### Rationale
- `rustc-hash` (FxHash) is optimized for small integer keys like `CommunityId` — 2-3× faster than default `SipHash` for this access pattern.
- `CommunityId` is a `NonZeroU32` newtype with `Hash` derived — FxHash exploits this perfectly.
- The neighbor cache is accessed in the hottest loop (per-node, per-neighbor-community evaluation) — hash map performance directly impacts overall speedup.
- `BTreeMap` rejected: the access pattern is random lookup by community ID, not range iteration. For small numbers of neighbor communities (typically < 20), FxHashMap is faster.

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| `HashMap<u32, f64>` (default hasher) | SipHash is 2-3× slower for small integer keys — directly in the hottest loop |
| `BTreeMap<u32, f64>` | Wrong access pattern (random lookup, not ordered iteration) |
| `Vec<(u32, f64)>` (linear scan) | Would work for very small neighbor counts but degrades to O(k) per lookup where k = neighbor communities |
| `IndexMap` (indexmap crate) | Overkill — preserves insertion order which we don't need; adds dependency |

### Source
- `rustc-hash` crate documentation: optimized for small keys, used by rustc itself.
- Spec FR-002: "cache per-node neighbor community weights and reuse them across candidate evaluations within a single node's move decision."

---

## 2. Deterministic PRNG

### Decision
Use `ChaCha8Rng` from `rand_chacha` crate for all Leiden algorithm randomness.

### Rationale
- `ChaCha8Rng` has a fixed specification (RFC 8439) — same output across all platforms, compiler versions, and architectures.
- Recommended by `rand` crate documentation specifically for reproducibility.
- 8 rounds (vs 20 for ChaCha20) — faster while maintaining cryptographic-quality statistical properties sufficient for algorithm randomness.
- `rand_chacha` is already in `Cargo.lock` as a transitive dependency of `rand 0.9` — adding it as a direct dependency is low-risk.
- `StdRng` is version-dependent (may change algorithm between `rand` releases) — violates FR-009 cross-platform bitwise-identical requirement.

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| `StdRng` (current) | Algorithm is version-dependent — NOT cross-platform deterministic |
| `ChaCha20Rng` | Slower (20 rounds vs 8) — no statistical benefit for algorithm use case |
| `ChaCha12Rng` | Middle ground — but ChaCha8 is sufficient and faster |
| Custom XorShift/PCG | Non-standard — would require justification; ChaCha8 is the community standard |

### Source
- Spec FR-009: "bitwise-identical community partitions... across different compiler versions, operating systems, and CPU architectures."
- `rand` crate docs: recommends `ChaCha8Rng` for deterministic, reproducible randomness.
- `rand_chacha` crate: already in dependency tree.

---

## 3. Cache Invalidation Strategy

### Decision
Dirty cache marking + frontier-based invalidation (LD-Leiden style) with subtract-add repair for exact aggregate weight updates.

### Rationale
- **Dirty marking**: When a node moves, mark affected community statistics dirty rather than immediately recomputing. Recompute on next access or at phase boundaries. Avoids redundant computation when a community is affected by multiple node movements in one pass.
- **Frontier propagation**: When node A moves and is a neighbor of node B, B's neighbor cache is marked dirty (transitive invalidation). This matches the LD-Leiden approach and prevents stale community weight statistics.
- **Subtract-add repair**: When a node moves from community X to Y, subtract its contribution from X's statistics and add to Y's statistics. This gives exact aggregate weight updates (no approximation) without full recomputation.
- **Debug invariant checks**: In debug builds, assert that cached community weight sums match actual graph state (computed via full traversal). Zero cost in release builds.

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Immediate recompute on invalidation | Wastes work when multiple nodes move in same pass — each move triggers full recompute |
| Full rebuild per iteration | Defeats the purpose of caching — O(V + E) per iteration instead of O(1) incremental |
| Version counters | Complex to implement correctly; dirty marking is simpler and equally effective |
| No invalidation (stale cache) | Would produce incorrect results — violates FR-006 correctness requirement |

### Source
- Spec FR-001: "dirty cache marking — when a cached entry is invalidated, it is marked dirty rather than immediately recomputed."
- Spec FR-002: "transitive invalidation via frontier propagation" + "subtract-add repair for exact aggregate weight updates."
- LD-Leiden paper (VanderTraag et al.): dirty cache + frontier propagation is the standard approach.

---

## 4. Floating-Point Drift Mitigation

### Decision
Periodic full recomputation of cached community statistics after every N=100 incremental updates (user-configurable, valid range [1, u32::MAX]). All accumulations use f64 precision.

### Rationale
- f64 has ~15-17 significant decimal digits of precision.
- With N=100 incremental updates, worst-case accumulated relative error is ~2.2×10⁻¹⁴ (Goldberg bound on floating-point summation).
- This is far below the modularity decision threshold (1e-6) and the partition quality tolerance (1e-4).
- Periodic full recomputation resets any accumulated error to zero every N updates.
- N=100 is a practical balance: frequent enough to prevent drift, infrequent enough to avoid performance impact.

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Kahan summation | 4× overhead per accumulation — unjustified when N=100 already keeps error below threshold |
| f128 / arbitrary precision | Massive performance penalty; not needed at this precision level |
| Larger N (e.g., 1000) | Risk of accumulated error approaching decision threshold on pathological inputs |
| No periodic recomputation | Unbounded FP drift — violates FR-011 |

### Source
- Spec FR-011: "periodic full recomputation of cached community statistics after every N incremental updates (where N is a user-configurable parameter of type u32, defaulting to 100, valid range N ∈ [1, u32::MAX])."
- Goldberg (1991): "What Every Computer Scientist Should Know About Floating-Point Arithmetic" — error bound analysis.

---

## 5. Convergence Evaluation Timing

### Decision
Evaluate early termination only after a complete Leiden pass (local-moving → refinement → aggregation) completes, not after individual sub-phases.

### Rationale
- Refinement can fragment communities, creating new boundaries that require subsequent aggregation to resolve.
- Evaluating convergence after local-moving alone would terminate prematurely when refinement still has meaningful work to do.
- DF-Leiden (Sahu 2024) confirms multiple passes are needed even after local-moving convergence.
- The original Leiden paper (Traag et al. 2019) defines convergence at the pass level, not sub-phase level.

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| After each sub-phase | Premature termination — refinement creates new boundaries requiring aggregation |
| After local-moving only | Misses refinement/aggregation quality improvements |
| After aggregation only | Wastes time if local-moving already converged |

### Source
- Spec Clarifications Q3 (Session 2026-09-07): "Evaluate only after a complete Leiden pass."
- DF-Leiden (Sahu 2024): confirms multi-pass requirement.
- Traag et al. (2019): pass-level convergence definition.

---

## 6. Early Termination Logic

### Decision
OR logic between two conditions: (a) absolute quality change below `convergence_threshold` (FR-003), OR (b) zero nodes moved during a complete local-moving pass (FR-004). Either condition triggers termination after the configured number of consecutive iterations.

### Rationale
- OR logic is more permissive — terminates as soon as either convergence signal fires.
- Zero-nodes-moved is the standard Leiden convergence criterion (all reference implementations: leidenalg, igraph, NetworkX).
- Quality-based convergence catches cases where nodes still move but quality improvement is negligible.
- AND logic would be too restrictive — could continue iterating when one signal already indicates convergence.

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| AND logic | Too restrictive — wastes iterations when one signal already fires |
| Sequential (quality first, then movement) | Adds complexity with no benefit — OR is simpler and equally effective |
| Quality only | Misses the standard Leiden convergence signal (zero nodes moved) |

### Source
- Spec FR-004a: "OR logic — the algorithm terminates when EITHER condition is satisfied."
- leidenalg, igraph, NetworkX: all use zero-nodes-moved as primary convergence.

---

## 7. Oscillation Detection

### Decision
Rolling quality history window of K=5 iterations with plateau detection: if `max(Q_window) - min(Q_window) < convergence_threshold` for K consecutive iterations, terminate.

### Rationale
- Catches both true convergence (quality stabilizes) and floating-point oscillation (quality cycles within epsilon).
- K=5 is practical: large enough to avoid false positives from single-iteration noise, small enough to terminate promptly.
- Uses the existing `convergence_threshold` parameter — no new configuration needed.
- Prevents premature termination on cycling quality values (spec FR-003).

### Alternatives Considered
| Alternative | Why Rejected |
|-------------|--------------|
| Separate oscillation detection pass | Adds overhead — rolling window is zero-cost (just track last K values) |
| Larger K (e.g., 10) | Slower to detect oscillation — wastes iterations |
| Smaller K (e.g., 3) | Risk of false positives from normal quality fluctuation |
| No oscillation detection | Risk of infinite loop on pathological inputs |

### Source
- Spec FR-003: "rolling quality history window of the last K=5 quality values."
- DF-Leiden (Sahu 2024): confirms oscillation handling is needed.

---

## 8. Memory Ordering / Thread Safety

### Decision
Not specified — single-threaded only. Parallelism deferred to future iteration.

### Rationale
- Exhaustive research (10+ parallel Leiden/Louvain implementations) confirms zero published papers specify memory ordering guarantees.
- Thread-safety is universally an implementation-level concern, not an algorithm-level guarantee.
- Original Leiden (Traag et al. 2019) is entirely sequential.
- In Rust, single-threaded is self-evident from types not implementing `Send`/`Sync`.
- NetworKit's ParallelLeiden was REMOVED in 2024 for race conditions — proving thread-safety cannot be an afterthought.

### Source
- Spec CHK015: "Exhaustive research confirms thread-safety requirements are correctly excluded from this spec."
- NetworKit: ParallelLeiden removal (2024).

---

## 9. CI Benchmark Gates

### Decision
Not specified in algorithm spec — CI infrastructure is an implementation concern.

### Rationale
- Standard practice EXCLUDES CI infrastructure from algorithm specs.
- Specs define WHAT (measurable targets), implementation defines HOW (CI tooling).
- Criterion.rs FAQ explicitly warns against wall-clock benchmarks on shared CI (virtualization noise).
- Performance targets (SC-001 through SC-004) are correctly specified as measurable outcomes.

### Source
- Spec CHK034: "Specifying CI benchmark gates in the spec would violate the spec/implementation boundary."
- Criterion.rs FAQ: warns against wall-clock CI benchmarks.

---

## 10. Dependency Additions

### Decision
Add two new dependencies to `crates/communal-algo/Cargo.toml`:
1. `rand_chacha = "0.9"` (matches `rand 0.9` version)
2. `rustc-hash = "2.0"` (latest stable)

### Rationale
- `rand_chacha 0.9` is compatible with `rand 0.9` (same release cycle).
- `rustc-hash 2.0` is the latest stable — provides `FxHashMap` and `FxHashSet`.
- Both are MIT/Apache-2.0 licensed (compatible with project licensing).
- Both are widely used, well-audited crates.

### Source
- `rand_chacha` crate: companion to `rand`, same version numbering.
- `rustc-hash` crate: official rustc hash algorithm extraction.

---

## 11. FP Accumulation Order Determinism (CHK019)

### Decision
Mandate ascending node-id order for all cached statistic accumulations.

### Rationale
- FP accumulation order affects results: `(0.5+512)+512.5=1025` vs `0.5+(512+512.5)=1024` (half-precision example from arXiv:2411.00442)
- Goldberg 1991: FP addition is non-associative
- P3375R2 (C++26 proposal): "Reproducibility requires the order of evaluation to be specified unambiguously"
- Rust f64: IEEE 754-2008 compliant for basic ops, but transcendental functions are non-deterministic across platforms
- Reference implementations (leidenalg, igraph, NetworkX) iterate in ascending order *de facto*
- Kahan summation NOT needed at N=100 (worst-case error ~2.2×10⁻¹⁴, far below 1e-6 threshold)

### Source
- [arXiv:2411.00442 — Revealing Floating-Point Accumulation Orders in Software](https://arxiv.org/html/2411.00442v3)
- [P3375R2: Reproducible floating-point results (C++26 proposal)](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2025/p3375r2.html)
- [Goldberg 1991 — What Every Computer Scientist Should Know About Floating-Point](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html)

---

## 12. Mid-Pass Recomputation Timing (CHK020)

### Decision
Defer full recomputation to phase/iteration boundaries. Never interrupt mid-pass.

### Rationale
- No published paper studies mid-pass interruption in Leiden
- All dynamic Leiden variants (LD-Leiden, DF-Leiden, ND-Leiden, DS-Leiden) process batches atomically
- Leiden's move→refine→aggregate pipeline has inter-phase dependencies that make mid-pass interruption require complex rollback
- Reference implementations (leidenalg, igraph, NetworkKit, cuGraph) push recompute policy to application layer
- SVSIG streaming paper: "periodic recomputation serves as an error safety net" at superstep granularity
- Smaller batches preferred over mid-pass interruption for quality control

### Source
- [LD-Leiden: Local Parallel Community Detection in Large Graphs (arXiv:2502.18497)](https://arxiv.org/html/2502.18497)
- [libleidenalg Optimiser.cpp](https://github.com/vtraag/libleidenalg) — Atomic batch processing
- [leidenalg documentation](https://leidenalg.readthedocs.io/en/stable/reference.html)

---

## 13. Beta Randomness Interaction with Caching (CHK091)

### Decision
Explicitly state that beta affects move selection but not cache update semantics.

### Rationale
- libleidenalg `move_nodes_constrained()` uses `RAND_NEIGH_COMM` (random neighbor selection) + greedy best-improvement — NOT probabilistic acceptance
- `partition->move_node(v, max_comm)` immediately updates cached statistics regardless of how move was selected
- Cache updates are deterministic given a move — selection mechanism only determines WHICH moves happen
- Current Rust codebase (`refinement.rs:207-209`): `acceptance_prob = if gain > 0.0 { 1.0 } else { 0.0 }` matches reference behavior
- No published paper discusses beta/cache interaction because it's trivial by design

### Source
- [libleidenalg Optimiser.cpp](https://github.com/vtraag/libleidenalg) — `move_nodes_constrained()` function
- [leidenalg documentation](https://leidenalg.readthedocs.io/en/stable/reference.html)
- Current codebase: `crates/communal-algo/src/leiden/refinement.rs:207-209`

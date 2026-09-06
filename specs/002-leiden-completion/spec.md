# Feature Specification: Leiden Algorithm Completion

**Feature Branch**: `002-leiden-completion`

**Created**: 2026-09-05

**Status**: Draft

**Input**: Complete the Leiden algorithm implementation to deliver mathematically verifiable connected communities — the framework's core differentiator.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Trustworthy Community Detection (Priority: P1)

As a network researcher, I want to run community detection on any graph and receive internally connected communities, so that I can trust the results for downstream analysis without manual validation.

**Why this priority**: This is the framework's sole differentiator and value proposition. Without mathematically verified connected communities, Communal offers no advantage over existing crates.

**Independent Test**: Can be fully tested by running Leiden on any valid graph input and verifying via BFS/DFS that every detected community is internally connected. Delivers the core guarantee that distinguishes this framework.

**Acceptance Scenarios**:

1. **Given** a valid graph with known community structure, **When** Leiden detection runs, **Then** every output community is internally connected (BFS from any member reaches all members)
2. **Given** an empty graph, **When** Leiden detection runs, **Then** no panic occurs and a valid empty partition is returned
3. **Given** a single-node graph, **When** Leiden detection runs, **Then** the node is assigned to a single community with finite quality score

---

### User Story 2 - Reproducible Research Results (Priority: P2)

As an AI engineer building GraphRAG pipelines, I want community detection results to be deterministic and reproducible across runs, so that my experiments and production systems produce consistent outputs.

**Why this priority**: Reproducibility is essential for scientific validity and production debugging. Non-deterministic results invalidate experiments and make bug isolation impossible.

**Independent Test**: Can be fully tested by running Leiden 100 times with the same seed on identical input and verifying bit-for-bit identical membership vectors across all runs.

**Acceptance Scenarios**:

1. **Given** a graph and a fixed seed value, **When** Leiden detection runs multiple times, **Then** every run produces identical membership vectors
2. **Given** the same graph with different seed values, **When** Leiden detection runs, **Then** results may differ but all still satisfy connected-community invariant
3. **Given** no seed is provided, **When** Leiden detection runs, **Then** default seed (42) is used and results are reproducible

---

### User Story 3 - Edge Case Robustness (Priority: P3)

As a data scientist working with messy real-world graphs, I want the algorithm to handle edge cases gracefully without crashing or producing garbage output, so that I can trust it in automated pipelines.

**Why this priority**: Production pipelines encounter diverse graph structures. Crashes or NaN/Inf outputs break downstream processing and erode trust.

**Independent Test**: Can be fully tested by running Leiden on edge case graphs (self-loops, zero weights, disconnected components) and verifying valid partitions with finite quality scores.

**Acceptance Scenarios**:

1. **Given** a graph with self-loops, **When** Leiden detection runs, **Then** no panic occurs and self-loops are handled correctly in quality computation
2. **Given** a graph with zero-weight edges, **When** Leiden detection runs, **Then** valid partition is returned with finite quality (no NaN/Inf)
3. **Given** a graph with disconnected components, **When** Leiden detection runs, **Then** each component's communities are internally connected

---

### Edge Cases

- What happens when the graph has no edges (all isolated nodes)?
- How does the algorithm handle self-loops during community aggregation?
- What happens with zero-weight edges in quality computation?
- How does the algorithm behave with negative weight inputs (mathematically valid per Traag et al. 2019; only rejected when total edge weight m ≤ 0)?
- What happens when all nodes belong to a single community (complete graph)?
- How does the algorithm terminate when quality improvement stalls (plateau detection emits observability event but does NOT terminate)?

## Clarifications

### Session 2026-09-05

- Q: What formula should be used to compute the quality gain (ΔQ) when evaluating whether to move a node to a different community during Leiden's local moving phase? → A: Add explicit ΔQ formulas for both CPM and Modularity Q to spec §FR-001, with all variables defined (matches libleidenalg source).

- Q: How should "connectedness preservation" during local moving and "subpartition guarantee" for refinement be formally defined? → A: Define connectedness via refinement's γ-connectivity conditions and subpartition via subset relation (∀C' ∈ P', ∃C ∈ P : C' ⊆ C), matching paper's Lemma 2 and Algorithm A.2.

- Q: Should the spec constrain which convergence modes are valid for each quality function, given that relative convergence causes division-by-zero when Modularity Q or CPM approaches zero? → A: Disallow relative convergence with Modularity Q and CPM; restrict to absolute mode for these quality functions (matches all reference implementations).

- Q: What validation constraints and behavioral documentation should apply to the gamma (resolution) parameter? → A: Add γ > 0 validation constraint and document behavioral effect (higher γ → more communities, acts as density threshold in CPM).

- Q: What graph corpus and test parameters should the spec define for property-based testing and LFR benchmark validation? → A: Define three-tier corpus (deterministic reference graphs, real-world benchmarks, synthetic generators) and explicit property-based test parameter ranges (node counts, edge densities, weight ranges).

- Q: What should the default value be for Leiden's refinement phase beta parameter, and should it be configurable by the caller? → A: Default beta=0.01 (matching Traag et al. 2019), configurable via LeidenConfig within range [0.0005, 0.1]. Implements `exp(β·Δ)` weighted selection as described in the paper (reference implementations simplify to uniform random, but the paper's formulation is authoritative).
- Q: Which quality functions must be implemented for this feature — Modularity Q, CPM, and Map Equation all together, or only Modularity Q with others deferred? → A: Modularity Q and CPM only. Map Equation is exclusive to Infomap — no reference Leiden implementation supports it, and the paper's theoretical guarantees only apply to Modularity and CPM. Map Equation deferred to Infomap feature.
- Q: What mechanism should plateau events use — callback trait, tracing crate, or hybrid? → A: Hybrid — `tracing` crate for structured events (constitution-compliant, zero-cost when no subscriber) PLUS a minimal `SteppingCallback` trait for TUI bidirectional stepping control (optional, `Option<Box<dyn SteppingCallback>>`).
- Q: Should negative weight edges be rejected at the algorithm level with a domain error, or is rejection solely at graph construction sufficient? → A: Negative weights are mathematically valid for Leiden (paper makes no positivity assumption; igraph supports them). Algorithm checks total edge weight `m > 0` at entry (single O(E) pass), returns `GraphError::InvalidGraph { reason }` if `m ≤ 0`. No per-edge runtime check in hot loops.
- Q: What convergence mode should be default — absolute quality improvement threshold or relative percentage improvement? → A: Absolute mode only (ΔQ < ε, ε=1e-6). All reference implementations use absolute improvement as stopping condition. Relative mode was considered but rejected — it does not exist in any reference implementation (igraph, libleidenalg, leidenalg) and provides no benefit for Modularity Q or CPM.
- Q: When the Leiden algorithm reaches max_iterations without converging, what should it return to the caller? → A: Return current best partition with a `tracing` warning event (best-effort, always valid). Matches reference implementations (igraph, leidenalg) which never fail on non-convergence. Keeps API ergonomic (no Result wrapping for non-error condition) and aligns with zero-panic principle.
- Q: How should the caller specify which quality function (Modularity Q or CPM) the Leiden algorithm should optimize? → A: Add `quality_function: QualityFunction` as a field in `LeidenConfig` (default `QualityFunction::Modularity`). All configuration lives in one place, matches existing config pattern, avoids complicating trait bounds or method signatures.
- Q: How should community IDs be assigned when communities are detected, to ensure deterministic and reproducible results? → A: Contiguous IDs assigned by first-node-encountered order during local moving phase. Node 0's community = 0, next new community encountered = 1, etc. Deterministic (depends only on seed), simple to implement, matches reference implementations.
- Q: What does "bidirectional stepping control" mean for the SteppingCallback trait? → A: Forward-only stepping: pause before each iteration, callback returns whether to continue or abort. Sufficient for TUI observation/debugging, avoids checkpointing complexity for backward navigation, matches zero-cost-when-disabled principle.
- Q: When emitting plateau events during convergence, what specific data fields should the event carry to support debugging and TUI display? → A: iteration (u64), improvement (f64), current_quality (f64) — full context for convergence visualization without requiring consumers to maintain separate state.
- Q: What domain error types should Leiden use for error conditions? → A: Reuse existing `communal-core` error types: `GraphError::InvalidGraph { reason }` (total weight m ≤ 0), `GraphError::EmptyGraph`, `AlgorithmError::InvalidConfiguration { reason }` (invalid parameters), `AlgorithmError::NonConvergence { iterations }` (max iterations reached, informational). No new error enums needed.
- Q: How should self-loops contribute to the quality function computation (Modularity Q and CPM)? → A: Self-loop counted once as intra-community edge weight (matches Traag et al. 2019 and igraph treatment).
- Q: Should Leiden validate that the input graph is connected before running, or process each connected component independently? → A: Process each component independently without pre-validation (matches igraph, leidenalg). No O(V+E) validation pass. Communities never span components because there are no edges between them.
- Q: During the aggregation phase, how should self-loops be created in the reduced graph? → A: Self-loop weight for community C = sum of all edge weights between nodes within community C (including original self-loops). Matches Leiden paper definition and igraph implementation.

- Q: Does mathematical justification exist for the beta parameter range [0.0005, 0.1], and how should the spec document this? → A: No mathematical justification exists in any primary source (confirmed via exhaustive search). Traag et al. 2019 states the range once without derivation — it is purely empirical. A fundamental scale-dependence problem prevents theoretical derivation: a fixed beta has different effective randomness on different-sized networks because quality deltas scale with graph size (igraph GitHub issue #77 acknowledges this). Spec documents the range as "empirical" with explicit note that no mathematical justification exists. Beta range should be treated as a rule of thumb, not a guarantee of good performance on arbitrary networks.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST implement Leiden's local moving procedure: iterate over nodes in random order (seeded), evaluate moving each node to neighboring communities using the quality gain formula ΔQ, move when ΔQ is positive, repeat until no node moves. The ΔQ formulas are:

  **CPM quality gain** for moving node $v$ to community $C$:
  $$\Delta Q(v \to C) = [w(v,C) + w(v,v) - \gamma \cdot n_v(2n_C + n_v)] - [w(v,\sigma_v) - w(v,v) - \gamma \cdot n_v(2n_{\sigma_v} - n_v)]$$
  Where: $w(v,C)$ = total edge weight from $v$ to nodes in $C$; $w(v,v)$ = self-loop weight of $v$; $n_v$ = 1 (single node); $n_C$ = size of community $C$; $\sigma_v$ = current community of $v$; $\gamma$ = resolution parameter.

  **Modularity quality gain** for moving node $v$ to community $C$:
  $$\Delta Q(v \to C) = \frac{1}{2m}\left[\Delta E - \frac{k_v \cdot \Delta K}{2m}\right]$$
  Where: $m$ = total edge weight; $k_v$ = weighted degree of $v$; $\Delta E$ = change in internal edge weight; $\Delta K$ = change in community degree sum. All variables computed from graph state before the move.

- **FR-002**: System MUST implement Leiden's randomized refinement: start with singleton communities, consider moves with probability `exp(β·Δ)` (weighted selection per Traag et al. 2019), beta parameter (default 0.01, empirical range [0.0005, 0.1] — no mathematical justification exists in literature; range is stated without derivation in Traag et al. 2019 and is purely empirical; a fixed beta has different effective randomness on different-sized networks because quality deltas scale with graph size) configurable via LeidenConfig. The refined partition MUST be a **subpartition** of the original: formally, ∀C' ∈ P_refined, ∃C ∈ P : C' ⊆ C. This is structurally enforced by processing each original community independently (Algorithm A.2, lines 27-31) — MergeNodesSubset only merges nodes from the same original community. The refinement phase enforces **γ-connectivity**: node eligibility requires E(v, S\{v}) ≥ γ·k_v·(k_S - k_v), ensuring every merged community is recursively γ-connected.

- **FR-003**: System MUST implement aggregation: build reduced graph where communities become nodes, edge weights sum inter-community edges, self-loops sum all intra-community edge weights (including original self-loops); self-loop weight for community C = sum of all edge weights between nodes within C

- **FR-004**: System MUST detect convergence using absolute quality improvement threshold (ΔQ < ε, default ε=1e-6). All reference implementations (igraph, libleidenalg, leidenalg) use absolute convergence exclusively. Hard stop at max_iterations (default 1000). When max_iterations is reached without convergence, system MUST return the current best partition and emit a `tracing` warning event (iteration, threshold) — never return an error or panic for non-convergence. **NOTE**: Relative convergence mode was considered but rejected — it does not exist in any reference implementation and is unnecessary for Modularity Q or CPM.

- **FR-005**: System MUST execute deterministically: accept optional seed parameter (u64), default seed = 42, same seed + same input = bit-for-bit identical membership vector

- **FR-006**: System MUST guarantee connected communities: every detected community forms a single connected component verifiable via BFS/DFS traversal from any member reaching all members

- **FR-007**: System MUST accept configuration parameters: resolution parameter gamma (default 1.0, MUST be > 0, NaN and ±Inf rejected) for CPM and modularity, convergence threshold (default 1e-6), maximum iteration bound (default 1000). **Gamma behavior**: higher γ → more communities (monotonic relationship); γ acts as a density threshold in CPM (communities must have internal density ≥ γ); γ → 0⁺ yields single community; γ → ∞ yields singleton partition. Invalid γ (≤ 0, NaN, ±Inf) MUST return `AlgorithmError::InvalidConfiguration { reason }` error.

- **FR-008**: System MUST compute quality scores using specified functions: Modularity Q or Constant Potts Model (CPM). Map Equation is NOT supported (exclusive to Infomap algorithm, deferred to Infomap feature)

- **FR-009**: System MUST derive plateau threshold as `max(convergence_threshold / 10, 1e-8)` and emit plateau events via `tracing` with structured fields: iteration (u64), improvement (f64), current_quality (f64). Plateau events do NOT terminate execution. Plateau detection is a novel observability enhancement (not present in reference implementations) for debugging convergence behavior. Optional `SteppingCallback` trait for TUI forward-only stepping control (pauses before each iteration, returns continue/abort); when None, algorithm runs unhindered at full speed

- **FR-010**: System MUST handle edge cases gracefully: empty graphs, single nodes, single edges, disconnected nodes/components (each component processed independently without pre-validation, matching igraph/leidenalg), self-loops (counted once as intra-community edge weight, matching Traag et al. 2019 and igraph), zero weights, negative weights (mathematically valid per Traag et al. 2019) — all without panics and returning valid partitions with finite quality. Leiden MUST use domain-rich error types from `communal-core`: `GraphError::InvalidGraph { reason }` (returned when total edge weight m ≤ 0), `GraphError::EmptyGraph`, `AlgorithmError::InvalidConfiguration { reason }` (for invalid gamma, beta, threshold), `AlgorithmError::NonConvergence { iterations }` (informational, returned alongside best partition)

### Key Entities

- **LeidenConfig**: Configuration struct holding resolution parameter (gamma, default 1.0), random seed (default 42), convergence threshold (default 1e-6), max iterations (default 1000), convergence mode (default absolute), beta parameter (default 0.01, range [0.0005, 0.1]), quality function (default QualityFunction::Modularity)

- **Partition**: Assignment of nodes to communities, with contiguous community IDs (assigned by first-node-encountered order during local moving phase) and quality score

- **QualityFunction**: Enum specifying which quality metric to optimize (Modularity Q, CPM). Map Equation NOT included (Infomap-only)

- **ConvergenceMode**: Enum for convergence detection. Only `Absolute` variant exists (ΔQ < ε). Relative mode was considered but rejected — no reference implementation supports it.

- **SteppingCallback**: Optional trait for TUI forward-only stepping control (pauses before each iteration, returns continue/abort); when None, algorithm runs unhindered at full speed

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of communities across the defined test corpus are internally connected (verified via BFS/DFS traversal)

- **SC-002**: 100% deterministic across 100 runs with identical seed on LFR benchmark (N=10k, μ=0.3)

- **SC-003**: NMI >= 0.95 against planted partition for LFR benchmarks (N=10k, ⟨k⟩=20, μ ≤ 0.3, τ1=2, τ2=1, c_min=10, c_max=50)

- **SC-004**: All edge cases (empty, single node, single edge, disconnected, self-loops, zero weights) handled without panics and return valid partitions

- **SC-005**: Algorithm terminates within max_iterations for all valid inputs

- **SC-006**: Quality scores are finite (no NaN, no Inf) for all valid inputs

## Test Corpus & Benchmarks

### Three-Tier Test Corpus

**Tier 1 — Deterministic Reference Graphs:**
- Empty graph, single node, single edge
- Complete graph (Kₙ), complete bipartite (Kₙ,ₙ)
- Path graph, cycle graph, star graph
- Grid/graph lattice (2D)
- Zachary Karate Club

**Tier 2 — Standard Real-World Benchmarks:**
- Dolphins social network
- American College Football
- Political Books (PolBooks)
- Political Blogs (PolBlogs)
- Les Miserables character co-occurrence
- NetScience co-authorship

**Tier 3 — Synthetic Generators (property-based):**
- Erdős-Rényi (G(n,p))
- Barabási-Albert (preferential attachment)
- Watts-Strogatz (small-world)
- LFR benchmark (ground truth)
- Trees and forests

### Property-Based Test Parameters

| Dimension | Range |
|-----------|-------|
| **Node counts** | Tiny (1-10), Small (10-100), Medium (100-1,000), Large (1,000-10,000) |
| **Edge density** | Very Sparse (p=1/N), Sparse (p=5/N), Medium (p=10/N), Dense (p=0.5) |
| **Weight types** | Unweighted, Positive [0,1], Positive Integer, Negative (m > 0), Mixed Signs, Zero |
| **Test cases** | 50-200 per property (fast), 20-50 (slow) |

### LFR Benchmark Configuration

| Parameter | Value | Description |
|-----------|-------|-------------|
| N | 10,000 | Number of nodes |
| ⟨k⟩ | 20 | Average degree |
| k_max | 50 | Maximum degree |
| τ1 | 2 | Degree distribution exponent |
| τ2 | 1 | Community size distribution exponent |
| c_min | 10 | Minimum community size |
| c_max | 50 | Maximum community size |
| μ | 0.0 → 0.6 | Mixing parameter (fraction of inter-community edges) |

**Constraints**: c_min > k_min, c_max > k_max (critical for LFR validity per Lancichinetti et al. 2008).

## Assumptions

- Users provide valid graph structures; negative edge weights are mathematically valid for Leiden (per Traag et al. 2019, igraph supports them); algorithm only rejects when total edge weight m ≤ 0
- Default resolution parameter gamma=1.0 produces reasonable community counts for benchmark graphs
- Convergence threshold 1e-6 (absolute mode) produces modularity within acceptable epsilon of best-known for LFR N=10k
- Leiden is the only algorithm completed in this feature; other algorithms remain stubs
- Performance optimization is deferred until correctness is verified through benchmarks
- Existing infrastructure (CsrGraph, GraphView, Partition, AlgorithmConfig) is not modified
- Quality score computation (Modularity Q, CPM) is implemented to support Leiden's optimization; Map Equation deferred to Infomap feature
- Beta parameter (refinement phase) defaults to 0.01 per Traag et al. 2019, configurable within [0.0005, 0.1] (empirical range — no mathematical derivation exists in literature; scale-dependent on graph size)
- Plateau events emitted via `tracing` with optional `SteppingCallback` for TUI stepping control

## Dependencies

- **Internal**: `communal-core` (CsrGraph, GraphView, Partition, AlgorithmConfig), `communal-algo` (LeidenConfig struct)
- **External**: petgraph (graph interoperability for testing), proptest (property-based testing), rand (seeded RNG)
- **Test Data**: LFR synthetic benchmarks for ground-truth validation

## Out of Scope

- Other algorithms (Louvain, Infomap, LPA, Fluid) — deferred to v1.1
- Dynamic updates for GraphRAG — depends on Leiden working first
- CLI/TUI/WASM integration — delivery mechanisms, not core product
- Performance optimization — correctness first, then benchmark-driven optimization
- Parallel execution — deferred until sequential correctness verified

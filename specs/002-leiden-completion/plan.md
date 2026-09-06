# Implementation Plan: Leiden Algorithm Completion

**Branch**: `002-leiden-completion` | **Date**: 2026-09-05** | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-leiden-completion/spec.md`

## Summary

Complete the Leiden algorithm implementation by implementing the three phase stubs (local move, refinement, aggregation) and the quality function computations (Modularity Q, CPM). Add property-based tests verifying the connected communities invariant and determinism. All infrastructure (types, traits, config) already exists — this feature fills in algorithmic logic only.

## Technical Context

**Language/Version**: Rust 1.85+ (edition 2024)

**Primary Dependencies**:
- `rand 0.9` — Seeded RNG for deterministic execution
- `proptest 1.6` — Property-based testing (workspace)
- `petgraph 0.8.3` — Graph interoperability for test helpers (workspace)
- `thiserror 2.0` — Error types (workspace)
- `tracing 0.1.44` — Structured event emission (workspace)
- `num-traits 0.2.19` — Numeric trait abstractions (workspace)

**Storage**: N/A (in-memory graph processing only)

**Testing**: `cargo test` with proptest for property-based invariants, criterion for benchmark validation

**Target Platform**: Linux/macOS/WASM (cross-platform Rust)

**Project Type**: Library (algorithm implementation within communal-algo crate)

**Performance Goals**:
- Correctness first: 100% connected communities guarantee
- Deterministic: bit-for-bit identical results with same seed
- NMI >= 0.95 on LFR benchmarks (N=10k, μ<=0.3)

**Constraints**:
- `#![deny(unsafe_code, missing_docs, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::allow_attributes_without_reason)]` (matches existing crate-level config)
- No new dependencies (all already in workspace)
- No changes to communal-core types/traits
- Zero panic in production code
- Reuse existing `GraphError` and `AlgorithmError` from `communal-core` (no new error enums)

**Scale/Scope**: Single algorithm (Leiden) completion, ~6 files modified/created, ~1500 lines of algorithmic code + tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Mathematical Rigor | PASS | Connected communities verifiable via BFS/DFS; proptest invariant validates FR-006 |
| II. Performance & Zero-Cost | PASS | Static dispatch over `impl GraphView`; no `dyn` in hot loops; CsrGraph for cache locality |
| III. Modular Workspace | PASS | All changes in communal-algo crate; no cross-crate modifications |
| IV. Strict Rust Engineering | PASS | Existing deny attributes enforced; no new dependencies; domain-rich error types |
| V. Dynamic Graph Support | N/A | Out of scope for this feature |
| VI. Verification & Testing | PASS | proptest invariants (connected communities, determinism) + LFR benchmark validation |
| VII. Licensing | PASS | MIT OR Apache-2.0; no new dependencies |

## Project Structure

### Documentation (this feature)

```text
specs/002-leiden-completion/
├── plan.md              # This file ($speckit-plan command output)
├── research.md          # Phase 0 output ($speckit-plan command)
├── data-model.md        # Phase 1 output ($speckit-plan command)
├── quickstart.md        # Phase 1 output ($speckit-plan command)
├── contracts/           # Phase 1 output ($speckit-plan command)
└── tasks.md             # Phase 2 output ($speckit-tasks command - NOT created by $speckit-plan)
```

### Source Code (repository root)

```text
crates/communal-algo/src/
├── leiden/
│   ├── mod.rs          # Leiden struct + CommunityDetector impl (UPDATE: implement phases)
│   ├── config.rs       # LeidenConfig (exists — add validate() per T009)
│   ├── local_moving.rs # NEW: smart local move phase
│   ├── refinement.rs   # NEW: randomized refinement phase
│   ├── aggregation.rs  # NEW: graph aggregation phase
│   └── convergence.rs  # NEW: convergence + plateau detection (extract from mod.rs)
├── quality.rs          # QualityFunction, Modularity, Cpm (UPDATE: implement evaluate())
└── lib.rs              # Module exports (UPDATE: add submodules)

crates/communal-algo/tests/
├── leiden_connected.rs      # Property-based: connected communities invariant
├── leiden_determinism.rs    # Property-based: deterministic execution
├── leiden_edge_cases.rs     # Edge case handling
└── synthetic_ground_truth.rs # LFR benchmark validation

contracts/
└── reference-partitions/    # Reference data for benchmark validation
```

**Structure Decision**: Single-crate modification within communal-algo. New files for algorithm phases (local_moving.rs, refinement.rs, aggregation.rs, convergence.rs) to keep mod.rs focused on the public API and CommunityDetector implementation. Quality function implementations updated in-place in quality.rs.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*No violations — all constitution principles pass without justification needed.*

## Phase 0: Research & Design Decisions

### Unknowns Resolved

| Question | Decision | Rationale |
|----------|----------|-----------|
| Modularity Q formula for directed vs undirected | Undirected: `Q = (1/2m) * Σ_ij [A_ij - γ·k_i·k_j/2m]·δ(c_i,c_j)` | Matches Traag et al. 2019; GraphView provides undirected semantics |
| CPM formula | `Q = Σ_c [e_c - γ·(n_c choose 2)]` where e_c = intra-community edges, n_c = community size | Standard CPM formulation; resolution-limit-free |
| Self-loop handling in quality | Counted once as intra-community weight | Matches Traag et al. 2019 and igraph |
| Negative weight handling | Valid per paper; reject only when total weight m ≤ 0 | igraph supports negative weights; algorithm checks m > 0 at entry |
| Convergence plateau detection | `max(threshold/10, 1e-8)` as plateau threshold; emits tracing event (iteration, improvement, current_quality) | Novel observability enhancement (not in reference implementations); does NOT terminate execution |
| Community ID assignment | Contiguous by first-node-encountered order | Deterministic, simple, matches reference implementations |

### Algorithm Implementation Details

**Local Moving Phase** (`local_moving.rs`):
- Input: graph, current membership, seeded RNG
- For each node in random order:
  - Compute neighbor communities and edge weights to each
  - Compute modularity gain ΔQ for moving to each neighbor community
  - Move to community with max positive ΔQ (if connectedness preserved)
- Return true if any node moved

**Refinement Phase** (`refinement.rs`):
- Input: graph, current membership, seeded RNG, beta parameter
- Start with singleton communities for unmerged nodes
- Random order traversal
- Probabilistic moves: `P(accept) = exp(β·ΔQ)` weighted selection
- Track community-to-nodes mapping
- Guarantee: refined partition is subpartition of input

**Aggregation Phase** (`aggregation.rs`):
- Input: graph, refined membership
- Build community-to-nodes mapping
- For each pair of communities (C, D):
  - Edge weight = sum of all edges between nodes in C and nodes in D
- For each community C:
  - Self-loop weight = sum of all edges within C (including original self-loops)
- Return new CsrGraph

**Convergence** (`convergence.rs`):
- Track quality between iterations
- Absolute mode only: `|Q_new - Q_old| < ε` (matches all reference implementations)
- Plateau threshold: `max(ε/10, 1e-8)` — emits tracing event when improvement drops below (does NOT terminate)
- Emit tracing events for plateau detection (iteration, improvement, current_quality)
- Max iterations hard stop: return best partition with tracing warning (never error/panic)

## Phase 1: Design Artifacts

*See generated files: `data-model.md`, `contracts/`, `quickstart.md`*

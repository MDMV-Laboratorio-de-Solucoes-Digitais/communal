# Remediation Summary: API Design Specifications

**Date**: 2026-09-05
**Scope**: CHK044–CHK049 (API & Interface Design Quality Checklist)
**Files modified**:
- `specs/001-community-detection/spec.md`
- `specs/001-community-detection/checklists/api-design.md`

## What was added to spec.md

A new `### API Design Specifications` section was inserted after FR-046 and before `### Key Entities`, containing six `####` subsections:

### 1. Petgraph Integration API (CHK044)

- **Supported types**: `petgraph::Graph<N, E, Ty>` and `petgraph::StableGraph<N, E, Ty>`, with `GraphMap`/`MatrixGraph`/`petgraph::csr::Csr` explicitly excluded with rationale
- **Conversion functions**: `From<petgraph::Graph> for CsrGraph` (owned, O(V+E)), `From<&petgraph::Graph> for CsrGraph` (borrowed view, O(V+E) batch / O(1) when density permits), plus `StableGraph` equivalents
- **Direction handling**: `Directed`/`Undirected` maps to CsrGraph's undirected-by-default flag; directed weights preserved for flow-based algorithms (FR-031)
- **Complexity annotations**: O(V+E) for owned conversions, O(1) view for borrowed when density permits
- **Feature gating**: behind `petgraph` feature flag

### 2. Generator API Contract (CHK045)

- **Common trait**: `Generator<G: GraphView>` with `fn generate(self) -> G`
- **LfrConfig**: `n`, `k`, `max_k`, `mu`, `min_community`, `max_community`, `seed` — all with types, valid ranges, defaults
- **SbmConfig**: `n`, `k`, `pin`, `pout`, `seed`
- **BaConfig**: `n`, `m`, `seed`
- **ErConfig**: `n`, `p`, `seed`
- **Validation**: each config has `validate()` checking ranges and constraints
- **Feature gating**: behind `generators` feature flag

### 3. Facade Re-export Pattern (CHK046)

- **Top-level re-exports**: `GraphView`, `CommunityDetector`, `Partition`, `AlgorithmConfig`, `ConvergenceMode`, `CommunalError`
- **Feature-gated re-exports**: full matrix mapping feature flags (`core`, `algo`, `dynamic`, `petgraph`, `metrics`, `generators`, `wasm`, `cli`, `tui`, `full-observability`, `u64-idx`) to source crates and types, matching the plan.md feature flag table
- **Naming convention**: canonical names at `communal::TypeName` (tokio/serde style)
- **Default feature set**: `["core", "algo", "metrics"]`
- **Feature unification**: Cargo's unified feature set behavior noted

### 4. Newtype Pattern Contracts (CHK047)

- **NodeId**: `#[repr(transparent)] pub struct NodeId(NonZeroU32)` (NonZeroU64 with u64-idx feature)
- **Construction**: `new() -> Option<Self>` (rejects 0), `from_index(usize) -> Self` (infallible), `unsafe_new_unchecked() -> Self` (doc hidden)
- **Validation**: zero reserved as sentinel; niche optimization for `Option<NodeId>`
- **Conversion**: `From<NodeId> for usize`, `From<NodeId> for u32`, `TryFrom<u32> for NodeId`
- **Traits**: Copy, Clone, Eq, Ord, Hash, Debug, Display; no Deref (per Rust API Guidelines C-DEREF)
- **CommunityId**: identical pattern
- **Layout guarantee**: `#[repr(transparent)]` ensures same size/ABI as inner type

### 5. CsrGraph Type Specification (CHK048)

- **Type definition**: `CsrGraph<N = u32, E = f64>` with `IndexType` and `EdgeWeight` bounds
- **CSR storage layout**: `row_ptr`, `col_idx`, `weights`, `directed`, `node_count`, `edge_count`
- **Construction**: `from_edges()` (O(V + E log E)), `from_adjacency()` (O(V + E)), `from_builder()` (with validation)
- **Access methods**: inherits GraphView (`node_count`, `edge_count`, `neighbors`, `edge_weight`) plus `is_directed()`
- **Memory**: O(V + E), cache-friendly, SIMD-suitable

### 6. Builder Pattern Consistency (CHK049)

- **Common trait**: `Builder` with `build() -> Result<Self::Output, Self::Error>`
- **Consuming pattern**: all setters consume `self` return `Self` (not `&mut self`)
- **GraphBuilder**: `new()`, `with_edges()`, `validate_on_construction()`, `symmetrize()`, `build()`
- **AlgorithmConfig builders**: each has `new()` with defaults + consuming setters + `build()`
- **MetricsBuilder**: `new()`, `with_metric()`, `with_metrics()`, `build()`
- **Consistency rules**: 5 rules documented (fluent API, terminal build(), validation in build(), defaults in new(), no &mut self setters)

## Checklist updates

| Item | Status | Annotation |
|------|--------|------------|
| CHK044 | `[x]` | spec.md API Design Specifications > Petgraph Integration API |
| CHK045 | `[x]` | spec.md API Design Specifications > Generator API Contract |
| CHK046 | `[x]` | spec.md API Design Specifications > Facade Re-export Pattern |
| CHK047 | `[x]` | spec.md API Design Specifications > Newtype Pattern Contracts |
| CHK048 | `[x]` | spec.md API Design Specifications > CsrGraph Type Specification |
| CHK049 | `[x]` | spec.md API Design Specifications > Builder Pattern Consistency |

A notes entry was added documenting the remediation date, items covered, and content summary.

## Research sources consulted

- petgraph 0.8.3 docs: Graph, StableGraph, GraphMap type definitions and traits (https://docs.rs/petgraph/0.8)
- Rust API Guidelines: newtype pattern (C-NEWTYPE, C-DEREF, C-CONV), builder pattern recommendations (https://rust-lang.github.io/api-guidelines/)
- Rust standard library: `std::num::Wrapping` as newtype reference pattern
- NetworkX LFR benchmark: parameter definitions (n, tau1, tau2, mu, average_degree, min_degree, max_degree, min_community, max_community)
- SBM, Barabási-Albert, Erdős-Rényi: canonical parameter definitions from graph theory literature
- tokio/serde/axum: facade crate re-export patterns via `pub use` and feature flags
- derive_builder / Essential Rust: consuming vs &mut self builder tradeoffs
- CSR (Compressed Sparse Row) format: standard sparse matrix representation (row_ptr, col_idx, weights layout)

## Design decisions

1. **Zero-reserved sentinel for NodeId**: Using `NonZeroU32` with 1-based internal representation (0 reserved) enables both niche optimization (`Option<NodeId>` is 4 bytes) and a clean "unassigned" sentinel for partition algorithms.

2. **Exclusion of GraphMap from petgraph integration**: Hash-table-backed keys don't map efficiently to dense contiguous CSR indices; documented explicitly to prevent user confusion.

3. **Consuming builders over &mut self**: Prevents reuse of partially-configured builders and enables clean fluent one-liners. Validation deferred to `build()` to keep setters infallible.

4. **Default feature set**: `["core", "algo", "metrics"]` provides the essential surface without bloating compile times for users who don't need dynamic updates, petgraph, generators, WASM, CLI, or TUI.

# Implementation Plan: Community Detection Framework

**Branch**: `001-community-detection` | **Date**: 2026-09-03** | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-community-detection/spec.md`

## Summary

Implement a high-performance, modular community detection framework in Rust. The framework provides a unified interface for five core algorithms (Leiden, Louvain, Infomap, LPA, Fluid Communities), supports incremental dynamic graph updates for GraphRAG use cases, and exposes both synchronous (iterator) and asynchronous (callback) stepping modes for algorithmic observability. The architecture is a modular Cargo workspace with strict separation of concerns, featuring a facade crate with granular feature flags. The core data structure is a Compressed Sparse Row (CSR) graph layout optimized for cache locality and SIMD vectorization.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024)

**Primary Dependencies**:
- `petgraph` 0.8.3 - External graph interoperability
- `rayon` 1.10 - Data parallelism for algorithm phases
- `thiserror` 2.0 - Domain-rich error types
- `serde` + `serde_json` - Serialization for I/O and WASM
- `ratatui` 0.30.2 - TUI rendering
- `crossterm` 0.29.0 - Terminal control for TUI
- `wasm-bindgen` 0.2 - WebAssembly bindings
- `proptest` 1.6 - Property-based testing
- `criterion` 0.8.2 - Benchmarking
- `num-traits` 0.2.19 - Numeric trait abstractions
- `tracing` 0.1.44 - Structured observability

**Storage**: N/A - In-memory graph processing (graphs fit in system memory; out-of-core deferred)

**Testing**: cargo test (unit/integration/doctests), proptest (property-based), criterion (benchmarks)

**Target Platform**: Linux, macOS, Windows (native); WebAssembly (single-threaded)

**Project Type**: Library + CLI + TUI (modular Cargo workspace)

**Performance Goals**:
- Observability overhead <= 20% when enabled
- Incremental updates within O(k) neighborhood
- Core algorithms run over flat CSR buffers with no dynamic dispatch in hot loops

**Constraints**:
- Zero unsafe code (#![deny(unsafe_code)])
- Zero missing documentation (#![deny(missing_docs)])
- Zero panics in production (#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)])
- No todo/unimplemented markers (#![deny(clippy::todo, clippy::unimplemented)])
- No allow attributes without reason (#![deny(clippy::allow_attributes_without_reason)])
- No standard printing in libraries (#![warn(clippy::print_stdout)])
- No graph data/topology in logs by default (zero-trust logging)
- Deterministic execution with fixed seed (u64)
- Force-directed layout uses Fruchterman-Reingold algorithm for TUI graph visualization

**Scale/Scope**: Scale-agnostic; performs well from small graphs to large-scale networks without scale-specific tuning

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Justification |
|-----------|--------|---------------|
| I. Mathematical Rigor | PASS | Leiden guarantees connected communities; deterministic seed support; validation via BFS/DFS |
| II. Performance & Zero-Cost | PASS | CSR layout in communal-core; no dyn dispatch in hot loops; u32 default indexing |
| III. Modular Workspace | PASS | 10 subcrates with facade pattern; strict dependency hierarchy |
| IV. Strict Rust Engineering | PASS | #![deny(unsafe_code)]; #![deny(missing_docs)]; clippy::all deny; thiserror for all errors |
| V. Dynamic Graph Support | PASS | HIT architecture in communal-dynamic; O(k) localized updates |
| VI. Verification & Testing | PASS | proptest invariants; SNAP benchmarks; NMI >= 0.95 on LFR/SBM |
| VII. Licensing | PASS | MIT OR Apache-2.0; THIRD_PARTY_LICENSES.md for upstream attribution |

**No violations.**

## Project Structure

### Documentation (this feature)

```text
specs/001-community-detection/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── reference-partitions/  # SC-003 reference partition data
└── tasks.md             # Phase 2 output (NOT created by plan)
```

Implementation phases are defined in tasks.md: Setup, Foundational, User Story 1-5, and Polish.

### Source Code (repository root)

```text
crates/
├── communal-core/       # Primitives, NodeId, CommunityId, CsrGraph, GraphView trait
├── communal-algo/       # Leiden, Louvain, Infomap, LPA, Fluid implementations; internal QualityFunction enum for algorithm optimization
├── communal-dynamic/    # HIT architecture for incremental updates
├── communal-petgraph/   # Zero-copy petgraph adapters
├── communal-metrics/    # Public evaluation metrics: Modularity Q, CPM, Map Equation, NMI, ARI (external API for partition quality assessment)
├── communal-generators/ # LFR, SBM, Barabási-Albert, Erdős-Rényi
├── communal-wasm/       # WASM bindings (wasm-bindgen)
├── communal-cli/        # Headless CLI (file I/O, batch processing, comparison)
├── communal-tui/        # TUI (ratatui, stepping engine, pedagogical explanations)
├── communal-benches/    # Benchmark harness (SNAP datasets)

communal (facade crate)
├── Cargo.toml           # Re-exports subcrates via feature flags
└── src/lib.rs           # Public API re-exports
```

**Structure Decision**: Modular Cargo workspace with facade crate as specified in Constitution Section III. The root `communal` crate re-exports functionality via feature flags (`core`, `algo`, `dynamic`, `petgraph`, `metrics`, `generators`, `wasm`, `cli`, `tui`, `full-observability`, `u64-idx`).

### CLI Argument Contracts

| Command | Arguments | Description |
|---------|-----------|-------------|
| `run` | `--algorithm <name>`, `--input <path>`, `--output <path>`, `--gamma <f64>`, `--seed <u64>` | Execute community detection |
| `compare` | `--input <path>`, `--algorithms <list>`, `--metrics <list>` | Multi-algorithm comparison |
| `batch` | `--config <path>`, `--output-dir <path>` | Batch processing |
| `convert` | `--input <path>`, `--output <path>`, `--format <format>` | Format conversion |
| `metrics` | `--input <path>`, `--ground-truth <path>` | Compute quality metrics |
| `generate` | `--type <generator>`, `--output <path>`, `--nodes <usize>` | Generate synthetic graphs |
| `validate` | `--input <path>` | Validate graph file structure |

**Quality Metrics Separation**: Internal quality functions (used by algorithms during optimization) reside in `communal-algo/src/quality/` as a `QualityFunction` enum for zero-cost dispatch (no dynamic dispatch per Constitution Principle II). Public evaluation metrics (for external partition quality assessment) reside in `communal-metrics/` as a separate crate. Both implement the `QualityMetric` trait defined in `communal-core` (per FR-021). This separation ensures algorithm hot loops remain uncluttered while providing a rich public API for evaluation. **Note**: Map Equation is computed in both locations — internally in `communal-algo` for Infomap optimization, and in `communal-metrics` for external evaluation — since Infomap requires the metric during its optimization phase. The `QualityFunction` enum is marked `#[doc(hidden)]` per FR-021's internal quality function requirement.

**Terminology Clarification**: The canonical type name is `Partition` (the core struct with query API). This is consistent across spec, plan, and tasks.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations to track. All constitution principles are satisfied by design.

---

## Phase 0: Research Output

See [research.md](research.md) for technology decisions and best practices.

## Phase 1: Design Output

- [data-model.md](data-model.md) - Entity definitions, validation rules, state transitions
- [contracts/](contracts/) - Interface contracts for public APIs
- [quickstart.md](quickstart.md) - Runnable validation scenarios

# COMMUNAL — Community Detection Framework

**Generated:** 2026-09-05
**Commit:** d40f37e
**Branch:** dev

## OVERVIEW

High-performance community detection framework in pure Rust. Workspace of 11 crates with strict lint config (`unsafe_code = "deny"`, `missing_docs = "deny"`, no `unwrap`/`expect`/`panic`). All public APIs require docstrings. Edition 2024.

## STRUCTURE

```
communal/
├── src/lib.rs              # Feature-gated re-exports (core, algo, dynamic, petgraph, metrics, generators)
├── crates/
│   ├── communal-core/      # Foundation: traits, errors, CSR graph, builder, IO
│   ├── communal-algo/      # Algorithms: Leiden, Louvain, Infomap, LPA, Fluid
│   ├── communal-dynamic/   # Incremental updates: streaming, hierarchy, mutations
│   ├── communal-metrics/   # Evaluation: modularity, CPM, NMI, ARI, map equation
│   ├── communal-generators/# Synthetic: LFR, SBM, Barabási-Albert, Erdős-Rényi
│   ├── communal-petgraph/  # Zero-copy petgraph adapter
│   ├── communal-cli/       # CLI binary (clap)
│   ├── communal-tui/       # TUI binary (ratatui) — unpublished
│   ├── communal-wasm/      # WASM bindings
│   ├── communal-frontend/  # Frontend lib (placeholder)
│   └── communal-benches/   # Criterion benchmarks
├── specs/                  # Feature specs with research docs
└── research/               # Design research and analysis docs
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new algorithm | `crates/communal-algo/src/` | Create module, implement `CommunityDetector` |
| Modify core trait | `crates/communal-core/src/graph_view.rs` | `GraphView` is the main trait |
| Add error variant | `crates/communal-core/src/error.rs` | Enum-based with `thiserror` |
| Add quality metric | `crates/communal-metrics/src/` | Implement `QualityMetric` or `ComparativeMetric` |
| Add graph generator | `crates/communal-generators/src/` | Follow existing pattern (LFR, SBM, etc.) |
| Parse graph format | `crates/communal-core/src/io/` | edgelist, CSV, GML, JSON supported |
| Build a graph | `GraphBuilder` in `communal-core` | Fluent API, validates by default |
| Algorithm stepping | `crates/communal-algo/src/stepping/` | Resumable iteration, callbacks, events |
| Configuring algorithms | `*Config` structs in each algo | Implements `AlgorithmConfig` trait |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `GraphView` | trait | `core/src/graph_view.rs` | Core graph abstraction; all algorithms accept this |
| `CommunityDetector` | trait | `core/src/detector.rs` | Base trait for all algorithms |
| `QualityMetric` | trait | `core/src/quality.rs` | Single-partition quality evaluation |
| `ComparativeMetric` | trait | `core/src/quality.rs` | Two-partition comparison |
| `AlgorithmConfig` | trait | `core/src/config.rs` | Configuration base trait |
| `CsrGraph` | struct | `core/src/csr.rs` | Compressed Sparse Row graph |
| `Partition` | struct | `core/src/partition.rs` | Algorithm output (membership + quality) |
| `NodeId` | struct | `core/src/id.rs` | NonZeroU32 newtype, niche optimization |
| `CommunityId` | struct | `core/src/id.rs` | NonZeroU32 newtype |
| `StepEvent` | enum | `core/src/step.rs` | Algorithm observability events |
| `GraphError` | enum | `core/src/error.rs` | Construction/validation errors |
| `AlgorithmError` | enum | `core/src/error.rs` | Execution errors |
| `MetricsError` | enum | `core/src/error.rs` | Metrics computation errors |
| `Leiden` | struct | `algo/src/leiden/` | Leiden algorithm (refinement + aggregation) |
| `Louvain` | struct | `algo/src/louvain/` | Louvain algorithm (greedy modularity) |
| `QualityFunction` | enum | `algo/src/quality.rs` | Dispatch: Modularity, CPM, MapEquation |
| `Modularity` | struct | `algo/src/quality.rs` | Modularity Q implementation |
| `Cpm` | struct | `algo/src/quality.rs` | Constant Potts Model implementation |

## CONVENTIONS

- **No unsafe**: `#![deny(unsafe_code)]` in every crate
- **No unwrap/expect**: Must use `?`, pattern matching, or `if let`
- **No panic/todo/unimplemented**: Denied at workspace level
- **All public items documented**: `#![deny(missing_docs)]`
- **Debug on all public types**: `#![deny(missing_debug_implementations)]`
- **No wildcard dependencies**: `#![deny(wildcard_dependencies)]`
- **Sealed trait pattern**: Used for `MultilayerView` (forward compatibility)
- **Newtype with NonZeroU32**: `NodeId`, `CommunityId` for niche optimization
- **Workspace lints**: Defined once in root `Cargo.toml`, inherited via `[lints] workspace = true`
- **Error handling**: `thiserror` for libraries; error enums with `#[derive(Error)]`
- **Builder pattern**: `GraphBuilder` with fluent API
- **Feature gating**: Optional deps via features (algo, dynamic, petgraph, metrics, generators)

## ANTI-PATTERNS (THIS PROJECT)

- `unsafe` blocks — forbidden without explicit justification
- `.unwrap()` / `.expect()` — denied by clippy
- `panic!()` / `todo!()` / `unimplemented!()` — denied at workspace level
- `#[allow(...)]` without `reason = "..."` — denied
- `dbg!()` left in code — denied
- Missing docstrings on public items — denied
- Multiple versions of same crate — denied
- `print!/println!` — warned (use `tracing` or `log`)

## UNIQUE STYLES

- **Sealed supertrait**: `MultilayerView` cannot be implemented externally
- **Step event system**: Algorithms emit `StepEvent` for observability
- **Stepping mode**: `stepping/` module for resumable/iterable algorithm execution
- **Quality dispatch enum**: `QualityFunction` avoids dynamic dispatch
- **ConvergenceMode**: Absolute vs Relative quality change
- **Plateau detection**: `convergence::plateau_threshold()` formula

## COMMANDS

```bash
# Build
cargo build
cargo build --features full

# Test
cargo test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt -- --check

# Bench
cargo bench

# CLI
cargo run --bin communal-cli -- --help
```

## NOTES

- `communal-tui` is `publish = false` — internal only
- `crates/communal-frontend/src/lib.rs` is a placeholder (no frontend code yet)
- Root crate re-exports `communal_core as core` unconditionally; others feature-gated
- `resolver = "3"` in workspace (required for edition 2024)
- `specs/` and `research/` are extensive — contains design rationale and analysis
- `graphify-out/` is gitignored (generated dependency graph snapshots)

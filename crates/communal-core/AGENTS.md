# communal-core — Foundational Types & Traits

**Generated:** 2026-09-05

## OVERVIEW

Foundation crate: core abstractions (graph views, detector traits, errors, IDs) that all other crates depend on.

## STRUCTURE

```
src/
├── builder.rs      # GraphBuilder — fluent API, validates by default
├── config.rs       # AlgorithmConfig trait, ConvergenceMode enum
├── csr.rs          # CsrGraph — Compressed Sparse Row representation
├── detector.rs     # CommunityDetector<G> trait
├── error.rs        # GraphError, PartitionError, MetricsError, AlgorithmError
├── graph_view.rs   # GraphView trait, MultilayerView sealed supertrait
├── id.rs           # NodeId, CommunityId — NonZeroU32 newtypes
├── io/             # edgelist, CSV, GML, JSON, serialize
├── lib.rs          # Crate root with lint config
├── partition.rs    # Partition struct (membership + quality)
├── quality.rs      # QualityMetric, ComparativeMetric traits
├── step.rs         # StepEvent enum, AlgorithmPhase enum
├── symmetrize.rs   # Edge weight symmetrization
└── traversal.rs    # BFS, DFS graph traversal
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new error variant | `error.rs` | Enum variants with `#[error("...")]` |
| Add new graph trait method | `graph_view.rs` | `GraphView` is the core abstraction |
| Add IO format | `io/` | Follow existing pattern (edgelist, CSV, etc.) |
| Build a graph | `builder.rs` | `GraphBuilder::new().from_edges(...)` |
| Understand algorithm stepping | `step.rs` | Events for observability/debugging |

## DEPENDENCIES

| Crate | Purpose |
|-------|---------|
| `num-traits` | Numeric trait abstractions |
| `thiserror` | Derive-based error enum macros |
| `serde` | Serialization/deserialization |

No `petgraph` or `rayon` — this crate is fully self-contained with zero optional features.

## STRICT LINTS

Inherits workspace lints via `[lints] workspace = true`. Crate-level denies in `lib.rs`:

- `unsafe_code = "deny"` — no unsafe blocks
- `missing_docs = "deny"` — all public items must be documented
- `unwrap_used = "deny"`, `expect_used = "deny"` — no panicking unwraps
- `panic = "deny"`, `todo = "deny"`, `unimplemented = "deny"` — no panicking macros
- `allow_attributes_without_reason = "deny"` — any `#[allow]` must have `reason = "..."`

## UNIQUE STYLES

- **Sealed supertrait**: `MultilayerView` uses `private::Sealed` to prevent external implementations
- **Newtype with NonZeroU32**: `NodeId`, `CommunityId` enable `Option<T>` niche optimization
- **ConvergenceMode dispatch**: Absolute vs Relative quality change comparison
- **Plateau detection**: `convergence::plateau_threshold()` = `max(threshold/10, 1e-8)`

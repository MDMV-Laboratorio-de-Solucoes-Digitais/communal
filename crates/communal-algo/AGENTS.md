# communal-algo — Community Detection Algorithms

**Generated:** 2026-09-05

## OVERVIEW

Five algorithm implementations: Leiden, Louvain, Infomap, LPA, Fluid. All implement `CommunityDetector<G: GraphView>`.

## STRUCTURE

```
src/
├── fluid/          # Fluid Communities algorithm
├── infomap/        # Infomap algorithm
├── leiden/            # Leiden algorithm (refinement + aggregation)
│   ├── mod.rs          # Leiden module root
│   ├── config.rs       # LeidenConfig { gamma, beta, convergence_threshold, ... }
│   ├── convergence.rs  # ConvergenceMode (Absolute, Relative), plateau detection
│   ├── local_moving.rs # Fast local moving phase
│   ├── refinement.rs   # Refinement phase (splits communities)
│   ├── aggregation.rs  # Community aggregation into reduced graph
│   └── stepping.rs     # Leiden-specific stepping logic
├── louvain/           # Louvain algorithm (greedy modularity)
│   └── config.rs       # LouvainConfig
├── lpa/               # Label Propagation Algorithm
├── lib.rs          # Crate root
├── quality.rs      # QualityFunction enum, Modularity, Cpm structs
├── stepping/          # Resumable/observable execution
│   ├── mod.rs          # Stepping module root
│   ├── callback.rs     # Step observation trait
│   ├── emission.rs     # Event emission to listeners
│   └── iterator.rs     # Synchronous step-by-step iteration
├── subscribe.rs    # Event observer registration
└── tie_breaking.rs # Deterministic tie-breaking utilities
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new algorithm | `src/` | Create module + config, implement `CommunityDetector` |
| Configure algorithm | `*/config.rs` | Struct implementing `AlgorithmConfig` |
| Quality functions | `quality.rs` | `QualityFunction` enum for dispatch |
| Stepping/observability | `stepping/` | Callback, emission, iterator submodules |
| Tie-breaking | `tie_breaking.rs` | Deterministic behavior utilities |

## UNIQUE STYLES

- **QualityFunction dispatch**: Enum avoids dynamic dispatch in hot loops
- **Stepping mode**: Algorithms support resumable iteration with callbacks
- **Convergence detection**: `convergence::has_converged()` + `plateau_threshold()`
- **Event emission**: `StepEvent` enum for real-time observability

## DEPENDENCIES

- `communal-core` — GraphView, CommunityDetector, AlgorithmConfig traits
- `rand` — Random number generation (shuffling, tie-breaking)
- `rayon` — Data parallelism (parallel local moving, aggregation)
- `thiserror` — Error types (`AlgoError`)
- `serde` — Serialization support (optional via feature)
- `tracing` — Structured logging and observability
- `hashbrown` — Faster hash maps for community statistics

## FEATURES

- `default` — Empty (no default features)
- `async-tokio` — Async algorithm execution via Tokio
- `serde` — Derive `Serialize`/`Deserialize` for config structs

## LINTS

Strict lints inherited from workspace (`[lints] workspace = true`):

- `#![deny(unsafe_code)]`
- `#![deny(missing_docs)]`
- `#![deny(missing_debug_implementations)]`
- `#![deny(wildcard_dependencies)]`
- No `unwrap`/`expect`/`panic` — use `?`, pattern matching, or `if let`

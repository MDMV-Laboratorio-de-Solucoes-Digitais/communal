# communal-algo — Community Detection Algorithms

**Generated:** 2026-09-05

## OVERVIEW

Five algorithm implementations: Leiden, Louvain, Infomap, LPA, Fluid. All implement `CommunityDetector<G: GraphView>`.

## STRUCTURE

```
src/
├── fluid/          # Fluid Communities algorithm
├── infomap/        # Infomap algorithm
├── leiden/         # Leiden algorithm (refinement + aggregation)
│   └── config.rs   # LeidenConfig { gamma, beta, convergence_threshold, ... }
├── louvain/        # Louvain algorithm (greedy modularity)
│   └── config.rs   # LouvainConfig
├── lpa/            # Label Propagation Algorithm
├── lib.rs          # Crate root
├── quality.rs      # QualityFunction enum, Modularity, Cpm structs
├── stepping/       # Resumable/observable execution
│   ├── callback.rs # Step observation trait
│   ├── emission.rs # Event emission to listeners
│   └── iterator.rs # Synchronous step-by-step iteration
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

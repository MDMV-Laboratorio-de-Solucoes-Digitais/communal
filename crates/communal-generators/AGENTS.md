# communal-generators — Synthetic Benchmark Graphs

**Generated:** 2026-09-05

## OVERVIEW

Configurable generators for standard synthetic graph benchmarks used to evaluate community detection algorithms.

## STRUCTURE

```
src/
├── barabasi_albert.rs  # Preferential attachment (scale-free networks)
├── erdos_renyi.rs      # Random graph G(n,p)
├── lib.rs              # Crate root
├── lfr.rs              # LFR benchmark (overlapping communities)
└── sbm.rs              # Stochastic Block Model
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add new generator | Create module | Follow existing pattern with config struct |
| Scale-free graph | `barabasi_albert.rs` | Power-law degree distribution |
| Random graph | `erdos_renyi.rs` | Baseline comparison |
| Overlapping communities | `lfr.rs` | Ground-truth with overlap |
| Block structure | `sbm.rs` | Defined community structure |

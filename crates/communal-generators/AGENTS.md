# communal-generators — Synthetic Benchmark Graphs

**Generated:** 2026-09-07

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

## DEPENDENCIES

| Crate | Version | Notes |
|-------|---------|-------|
| `communal-core` | path | Core types (`CsrGraph`) |
| `rand` | 0.9 | Random number generation |

## NOTES

- All generators currently return **empty graph placeholders** (`CsrGraph::from_edges(&[], 0)`) — implementation is pending.
- Each generator follows the same pattern: `*Config` struct with `Default` impl + `*Generator` struct with `new()` and `generate()` methods.
- All public items require docstrings (`#![deny(missing_docs)]`).
- Strict lints inherited from workspace (`unsafe_code = "deny"`, no `unwrap`/`expect`/`panic`).

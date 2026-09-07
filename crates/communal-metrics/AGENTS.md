# communal-metrics — Evaluation Metrics

**Generated:** 2026-09-07

## OVERVIEW

Quality and comparative metrics for evaluating community detection results.

## STRUCTURE

```
src/
├── ari.rs          # Adjusted Rand Index (comparative)
├── cpm.rs          # Constant Potts Model (quality)
├── error.rs        # Metrics computation errors
├── lib.rs          # Crate root
├── map_equation.rs # Map Equation (quality)
├── modularity.rs   # Newman-Girvan modularity (quality)
└── nmi.rs          # Normalized Mutual Information (comparative)
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add quality metric | Create module | Implement `QualityMetric` trait |
| Add comparative metric | Create module | Implement `ComparativeMetric` trait |
| Compute modularity | `modularity.rs` | Newman-Girvan Q score |

## TRAITS

All metrics implement traits from `communal-core`:
- `QualityMetric` — single-partition evaluation (returns `f64`)
- `ComparativeMetric` — two-partition comparison (e.g., NMI, ARI)

## DEPENDENCIES

- `communal-core` — Core traits (`QualityMetric`, `ComparativeMetric`)
- `thiserror` — Error derive macro
- `ndarray` — N-dimensional arrays for matrix operations
- `num-traits` — Numeric traits
- `rayon` — Data parallelism

## FEATURES

- `default = []` — No default features
- `serde` — Serialization support for metric results

## LINTS

Strict lints inherited from workspace (`[lints] workspace = true`):
- `#![deny(unsafe_code)]`
- `#![deny(missing_docs)]`
- `#![deny(missing_debug_implementations)]`
- No `unwrap`/`expect`/`panic` allowed

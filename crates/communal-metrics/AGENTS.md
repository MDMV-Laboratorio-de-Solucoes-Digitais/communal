# communal-metrics — Evaluation Metrics

**Generated:** 2026-09-05

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

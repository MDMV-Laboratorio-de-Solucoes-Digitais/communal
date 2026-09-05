# Contract: Quality Metrics

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

Defines the interface for computing quality metrics on community detection results. Metrics are used both internally (for algorithm convergence) and externally (for partition comparison).

---

## QualityMetric Trait

```rust
/// Trait for partition quality metrics.
///
/// Quality metrics evaluate how "good" a community partition is.
/// Different metrics capture different notions of quality:
/// - Internal metrics: Modularity, CPM (no ground truth needed)
/// - External metrics: NMI, ARI (require ground truth)
pub trait QualityMetric {
    /// Computes the quality score for a partition.
    ///
    /// # Arguments
    /// - `graph`: The input graph
    /// - `partition`: The community partition to evaluate
    ///
    /// # Returns
    /// Quality score as f64. Higher is better for internal metrics;
    /// range varies by metric type.
    fn compute<G: GraphView>(
        &self,
        graph: &G,
        partition: &Partition,
    ) -> Result<f64, MetricsError>;

    /// Returns the metric name for display/logging.
    fn name(&self) -> &'static str;

    /// Returns the valid range of the metric.
    fn range(&self) -> (f64, f64);
}
```

---

## Modularity Q

```rust
/// Newman-Girvan modularity with resolution parameter.
///
/// Formula: Q = (1/2m) * Σ_ij [A_ij - γ * (k_i * k_j / 2m)] * δ(c_i, c_j)
///
/// # Parameters
/// - `gamma`: Resolution parameter (default 1.0)
///   - γ < 1: favors larger communities
///   - γ = 1: standard modularity
///   - γ > 1: favors smaller communities
///
/// # Range
/// [-1, 1], higher is better
///
/// # Complexity
/// O(V + E)
pub struct Modularity {
    gamma: f64,
}

impl Modularity {
    pub fn new(gamma: f64) -> Result<Self, MetricsError>;
    pub fn default() -> Self; // gamma = 1.0
}

impl QualityMetric for Modularity { /* ... */ }
```

---

## Constant Potts Model (CPM)

```rust
/// Constant Potts Model quality function.
///
/// Formula: H = -Σ_ij [A_ij - γ * (k_i * k_j / 2m)] * δ(c_i, c_j)
///
/// Unlike modularity, CPM does not suffer from the resolution limit.
/// The resolution parameter γ directly controls community size.
///
/// # Parameters
/// - `gamma`: Resolution parameter (no default; must be specified)
///
/// # Range
/// (-∞, 0], higher (closer to 0) is better
///
/// # Complexity
/// O(V + E)
pub struct ConstantPottsModel {
    gamma: f64,
}

impl ConstantPottsModel {
    pub fn new(gamma: f64) -> Result<Self, MetricsError>;
}

impl QualityMetric for ConstantPottsModel { /* ... */ }
```

---

## Map Equation

```rust
/// Infomap's Map Equation quality function.
///
/// Describes the theoretical limit of how well a random walker's path
/// can be compressed given a partition. Lower values indicate better
/// compression (better partition).
///
/// # Range
/// [0, ∞), lower is better
///
/// # Complexity
/// O(V + E)
pub struct MapEquation {
    teleportation_rate: f64,
}

impl MapEquation {
    pub fn new(teleportation_rate: f64) -> Result<Self, MetricsError>;
    pub fn default() -> Self; // teleportation_rate = 0.15
}

impl QualityMetric for MapEquation { /* ... */ }
```

---

## Normalized Mutual Information (NMI)

```rust
/// Normalized Mutual Information for partition comparison.
///
/// Measures the mutual information between two partitions, normalized
/// to [0, 1] range. Requires ground truth labels.
///
/// # Formula
/// NMI(X, Y) = 2 * I(X, Y) / (H(X) + H(Y))
///
/// Where I is mutual information and H is entropy.
///
/// # Range
/// [0, 1], where 1 = identical partitions
///
/// # Complexity
/// O(V)
pub struct NormalizedMutualInformation;

impl NormalizedMutualInformation {
    /// Computes NMI between detected partition and ground truth.
    ///
    /// # Arguments
    /// - `detected`: Community assignments from algorithm
    /// - `ground_truth`: Known community assignments
    pub fn compute(
        detected: &Partition,
        ground_truth: &Partition,
    ) -> Result<f64, MetricsError>;
}
```

---

## Adjusted Rand Index (ARI)

```rust
/// Adjusted Rand Index for partition comparison.
///
/// Measures similarity between two partitions, corrected for chance.
/// More robust than raw Rand Index for imbalanced partitions.
///
/// # Range
/// [-1, 1], where 1 = identical, 0 = random agreement
///
/// # Complexity
/// O(V)
pub struct AdjustedRandIndex;

impl AdjustedRandIndex {
    /// Computes ARI between detected partition and ground truth.
    pub fn compute(
        detected: &Partition,
        ground_truth: &Partition,
    ) -> Result<f64, MetricsError>;
}
```

---

## Metrics Calculator

```rust
/// Convenience struct for computing multiple metrics at once.
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Computes all applicable metrics for a partition.
    pub fn compute_all<G: GraphView>(
        &self,
        graph: &G,
        partition: &Partition,
    ) -> Result<MetricsReport, MetricsError>;

    /// Computes comparison metrics against ground truth.
    pub fn compare_to_ground_truth(
        &self,
        detected: &Partition,
        ground_truth: &Partition,
    ) -> Result<ComparisonReport, MetricsError>;
}

/// Report containing all computed metrics.
#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub modularity_q: f64,
    pub cpm: Option<f64>,
    pub map_equation: Option<f64>,
}

/// Report containing comparison metrics.
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub nmi: f64,
    pub ari: f64,
}
```

---

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("empty partition: cannot compute metrics")]
    EmptyPartition,

    #[error("partition size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u32, actual: u32 },

    #[error("invalid ground truth: {reason}")]
    InvalidGroundTruth { reason: String },

    #[error("numerical error: {0}")]
    NumericalError(String),
}
```

---

## Usage Example

```rust
use communal_metrics::{Modularity, QualityMetric, MetricsCalculator};

// Compute modularity
let modularity = Modularity::new(1.0)?;
let q = modularity.compute(graph, &partition)?;
println!("Modularity Q = {:.4}", q);

// Compute all metrics
let calculator = MetricsCalculator;
let report = calculator.compute_all(graph, &partition)?;
println!("{:?}", report);

// Compare with ground truth
let comparison = calculator.compare_to_ground_truth(&detected, &truth)?;
println!("NMI = {:.4}, ARI = {:.4}", comparison.nmi, comparison.ari);
```

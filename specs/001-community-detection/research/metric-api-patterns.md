# Metric API Design Patterns in Rust Scientific/Numerical Libraries

Research compiled for: Community Detection crate (communal)
Date: 2026-01-27
Depth: Thorough — primary-source evidence from crate docs, source links, and API guidelines.

---

## 1. Quality Metric Trait Design

### Finding: Two dominant trait patterns exist — the "method-on-trait" pattern (linfa) and the "scoring-function" pattern (smartcore).

#### Pattern A: Method-on-Trait (linfa)

In `linfa`, quality metrics are traits with a method that takes `&self` and returns a `Result<F>`. The trait is implemented directly on the dataset type.

**Source: `linfa::metrics::SilhouetteScore` (linfa v0.7.x)**
```rust
pub trait SilhouetteScore<F> {
    fn silhouette_score(&self) -> Result<F>;
}
```
- Docs: https://docs.rs/linfa/latest/linfa/metrics/trait.SilhouetteScore.html
- Source: `src/linfa/metrics_clustering.rs`

The trait is implemented for `DatasetBase<ArrayBase<D, Ix2>, T>` where `T: AsSingleTargets + Labels`. The metric consumes the dataset (which already contains cluster labels from a prior `Predict` step) and computes the score.

Other linfa metric traits follow the same shape:
- `BinaryClassification` — classification for binary-labels
- `SingleTargetRegression` — regression metrics trait for single targets
- `MultiTargetRegression` — regression metrics trait for multiple targets
- `ToConfusionMatrix` — classification for multi-label evaluation

All documented at: https://docs.rs/linfa/latest/linfa/metrics/index.html

#### Pattern B: Scoring-Function Trait (smartcore)

In `smartcore`, all metrics implement a unified `Metrics<T>` trait with a `get_score` method that takes true and predicted values as trait objects.

**Source: `smartcore::metrics::Metrics` (smartcore v0.4.x)**
```rust
pub trait Metrics<T> {
    fn new() -> Self where Self: Sized;
    fn new_with(_parameter: f64) -> Self where Self: Sized;
    fn get_score(
        &self,
        y_true: &dyn ArrayView1<T>,
        y_pred: &dyn ArrayView1<T>,
    ) -> f64;
}
```
- Docs: https://docs.rs/smartcore/latest/smartcore/metrics/trait.Metrics.html
- Source: `src/smartcore/metrics/mod.rs` lines 87–99

Implementors include: `Accuracy<T>`, `AUC<T>`, `F1<T>`, `MeanAbsoluteError<T>`, `MeanSquareError<T>`, `Precision<T>`, `R2<T>`, `Recall<T>`, `HCVScore<T>`.

#### Pattern C: Objective Function Trait (perpetual)

The `perpetual` gradient boosting library separates the *internal optimization objective* into its own trait with loss + gradient methods:

**Source: `perpetual::objective::core::ObjectiveFunction`**
```rust
pub trait ObjectiveFunction: Send + Sync {
    fn loss(&self, y: &[f64], yhat: &[f64], sample_weight: Option<&[f64]>, group: Option<&[u64]>) -> Vec<f32>;
    fn gradient(&self, y: &[f64], yhat: &[f64], sample_weight: Option<&[f64]>, group: Option<&[u64]>) -> (Vec<f32>, Option<Vec<f32>>);
    fn default_metric(&self) -> Metric;  // provided: links to public eval metric
    // ... other provided methods
}
```
- Docs: https://docs.rs/perpetual/latest/perpetual/objective/core/trait.ObjectiveFunction.html

This is the clearest example of a trait designed for *internal* optimization (loss + gradient) that also links to a public evaluation metric via `default_metric()`.

#### Recommendation for `evaluate(&self, graph, partition) -> f64`

Neither linfa nor smartcore uses a signature exactly like `evaluate(&self, graph, partition) -> f64`. The closest patterns are:

1. **linfa-style**: `fn score(&self) -> Result<F>` — the data is already in `self` (the dataset/model). Cleanest for internal use.
2. **smartcore-style**: `fn get_score(&self, y_true: &dyn ArrayView1<T>, y_pred: &dyn ArrayView1<T>) -> f64` — explicit inputs, no Result.

For a community detection crate, the linfa pattern is more idiomatic when the partition is already attached to the graph/dataset, while the smartcore pattern is better when the metric is a standalone scorer that takes inputs.

---

## 2. Internal vs Public Metric Separation

### Finding: Rust libraries use module visibility, sealed traits, and distinct trait/objective types to separate internal optimization from public evaluation.

#### Approach 1: Separate trait hierarchies (perpetual)

`perpetual` has two distinct concepts:
- `ObjectiveFunction` trait — for internal optimization (loss + gradient for training)
- `Metric` enum — for public evaluation (e.g., `Metric::RootMeanSquaredError`)

The `default_metric()` method on `ObjectiveFunction` links the two: each objective declares which public metric is most appropriate for evaluating models trained with that objective.

Source: https://docs.rs/perpetual/latest/perpetual/objective/core/trait.ObjectiveFunction.html

#### Approach 2: Module-level visibility

Rust's module system is the primary mechanism. Internal metrics go in `pub(crate)` or non-`pub` modules; public metrics are re-exported at the crate root or in a `pub mod metrics`.

The Rust API Guidelines recommend "minimize visibility" (Item 22, Effective Rust):
- Source: https://www.effective-rust.com/visibility.html
- Reference: https://doc.rust-lang.org/reference/visibility-and-privacy.html

#### Approach 3: Sealed traits for internal-only metrics

The Rust API Guidelines (C-SEALED) describe sealing traits so they can only be implemented within the defining crate. This is used when a trait is public (for internal use across modules) but must not be implementable by downstream crates.

- Source: https://rust-lang.github.io/api-guidelines/future_proofing.html
- DeepWiki: https://deepwiki.com/rust-lang/api-guidelines/7.1-sealed-traits-and-encapsulation
- Helper crate: https://docs.rs/sealed/latest/sealed/

#### Approach 4: linfa's dataset-embedded approach

In `linfa`, the `PredictInplace` trait writes predictions into the dataset, and then `SilhouetteScore` is implemented *on* the dataset. The metric is computed as a method of the already-predicted dataset. This means the "internal" state (predictions) is managed by the dataset, not by the metric.

Source: https://docs.rs/linfa/latest/linfa/traits/index.html

#### Practical pattern for community detection

For a community detection library, the recommended separation is:

| Concern | Visibility | Trait/Type | Example |
|---------|-----------|------------|---------|
| Internal optimization quality fn | `pub(crate)` or sealed trait | `ObjectiveFunction` / `QualityFn` | Modularity delta for Louvain moves |
| Public evaluation metric | `pub` trait in `metrics` mod | `CommunityMetric` | Modularity, NMI, ARI, Conductance |
| Linking the two | Method on objective | `default_metric()` | `LouvainObjective::default_metric() -> Metric::Modularity` |

---

## 3. MetricsCalculator Aggregation Pattern

### Finding: Rust ML libraries use "aggregator" structs that group related metrics and provide factory methods, but there is no single standardized `MetricsCalculator` trait.

#### Pattern: Metric Category Structs (smartcore)

`smartcore` defines empty structs that serve as factories/namespaces for related metrics:

```rust
pub struct ClassificationMetrics<T> { /* private fields */ }
impl<T: Number> ClassificationMetrics<T> {
    pub fn recall() -> Recall<T> { ... }
    pub fn precision() -> Precision<T> { ... }
    pub fn f1(beta: f64) -> F1<T> { ... }
}

pub struct RegressionMetrics<T> { /* private fields */ }
pub struct ClusterMetrics<T> { /* private fields */ }
impl<T: Number + Ord> ClusterMetrics<T> {
    pub fn hcv_score() -> HCVScore<T> { ... }  // Homogeneity + Completeness + V-Measure at once
}
```

- Docs: https://docs.rs/smartcore/latest/smartcore/metrics/struct.ClassificationMetrics.html
- Docs: https://docs.rs/smartcore/latest/smartcore/metrics/struct.ClusterMetrics.html
- Source: `src/smartcore/metrics/mod.rs`

These are *not* traits — they are unit structs with associated functions that return metric instances. The `<T>` parameter is carried via `PhantomData`.

#### Pattern: Free Functions (smartcore)

Smartcore also provides free functions for direct computation:
```rust
pub fn accuracy(y_true: &dyn ArrayView1<T>, y_pred: &dyn ArrayView1<T>) -> f64 { ... }
pub fn mean_absolute_error(y_true: &dyn ArrayView1<T>, y_pred: &dyn ArrayView1<T>) -> f64 { ... }
pub fn roc_auc_score(y_true: &dyn ArrayView1<T>, y_pred: &dyn ArrayView1<T>) -> f64 { ... }
```

These coexist with the trait-based `Metrics<T>` API, giving users a choice between ergonomic one-liners and structured trait objects.

#### Pattern: Metrics Ecosystem (metrics-rs)

The `metrics` crate (unrelated to ML metrics — it's for application/infrastructure telemetry) provides a different kind of aggregation via `metrique-aggregation`:

- Docs: https://docs.rs/metrique-aggregation
- Purpose: Aggregates multiple observations into a single metric entry (distributions, sums)

This is not directly applicable to ML quality metrics but shows the naming convention in the Rust ecosystem.

#### Recommended pattern for community detection

A `MetricsCalculator` or `CommunityMetrics` struct should:

1. Be a struct (not a trait) that holds references to the graph and partition
2. Provide methods for each metric: `modularity()`, `conductance()`, `nmi()`, `ari()`
3. Optionally provide an `all()` method returning a `HashMap<String, f64>` or a `MetricsResult` struct
4. Use the newtype pattern for the return values (see Section 4)

```rust
pub struct CommunityMetrics<'a, G: Graph> {
    graph: &'a G,
    partition: &'a Partition,
}

impl<'a, G: Graph> CommunityMetrics<'a, G> {
    pub fn new(graph: &'a G, partition: &'a Partition) -> Self { ... }
    pub fn modularity(&self) -> Modularity { ... }
    pub fn conductance(&self) -> Conductance { ... }
    pub fn all(&self) -> MetricsResult { ... }
}
```

---

## 4. Newtype Pattern for Domain Types

### Finding: The newtype pattern is the standard Rust idiom for domain-specific wrappers. `petgraph::NodeIndex` is the canonical graph-library example.

#### Canonical Example: `petgraph::NodeIndex`

```rust
pub struct NodeIndex<Ix = DefaultIx>(/* private fields */);

impl<Ix: IndexType> NodeIndex<Ix> {
    pub fn new(x: usize) -> Self { ... }
    pub fn index(self) -> usize { ... }     // explicit unwrap
    pub fn end() -> Self { ... }
}
```

Key design decisions:
1. **Private field** — the inner index is not `pub`, preventing construction bypass
2. **`new(usize) -> Self`** — explicit constructor from raw usize
3. **`index(self) -> usize`** — explicit conversion back to raw (consumes self, preventing accidental reuse)
4. **Implements `Copy`, `Clone`, `Debug`, `Hash`, `Eq`, `Ord`, `From<Ix>`, `IndexType`**
5. **Implements `Serialize`/`Deserialize`** (with serde feature)
6. **Used as index** for `Graph` and `StableGraph` via `impl Index<NodeIndex<Ix>>`

- Docs: https://docs.rs/petgraph/latest/petgraph/graph/struct.NodeIndex.html
- Source: `src/petgraph/graph_impl/mod.rs` lines 105–137

#### Idiomatic Newtype Implementation Pattern

From the Rust Design Patterns guide and community conventions:

```rust
/// Newtype for type-safe domain IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// Construct from a raw usize (panics or returns None if out of range).
    pub fn new(id: u32) -> Self { Self(id) }
    /// Convert back to usize.
    pub fn index(self) -> usize { self.0 as usize }
    /// Get the raw u32 value.
    pub fn raw(self) -> u32 { self.0 }
}

// Conversions
impl From<u32> for NodeId {
    fn from(id: u32) -> Self { Self(id) }
}

impl From<NodeId> for usize {
    fn from(id: NodeId) -> Self { id.0 as usize }
}

// Display
impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

Sources:
- Rust Design Patterns: https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html
- Idiomatic Rust Snippets: https://idiomatic-rust-snippets.org/patterns/rust-idioms/newtype.html
- DeepWiki type safety: https://deepwiki.com/rust-lang/api-guidelines/4.2-type-safety-patterns

#### Validation Pattern

For fallible construction (e.g., validating that an ID is within bounds):

```rust
impl NodeId {
    pub fn try_new(id: u32, max: u32) -> Result<Self, IdError> {
        if id < max {
            Ok(Self(id))
        } else {
            Err(IdError::OutOfBounds { id, max })
        }
    }
}
```

Or using `NonZeroU32` for IDs that must not be zero:

```rust
pub struct NodeId(NonZeroU32);

impl NodeId {
    pub fn new(id: u32) -> Option<Self> {
        NonZeroU32::new(id).map(Self)
    }
}
```

#### Recommendation for community detection

Use newtypes for:
- `NodeId(u32)` — wraps a node identifier
- `CommunityId(u32)` — wraps a community/cluster identifier
- `EdgeId(u64)` — wraps an edge identifier (if needed)
- `Modularity(f64)` — wraps a modularity score (with `Display` showing 4 decimal places)
- `PartitionId(u32)` — wraps a partition identifier (for multi-resolution)

All should derive `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash` and implement `Display`.

---

## 5. Builder Pattern Consistency

### Finding: Consistent builder patterns across Rust crates follow a well-established convention: `new()` -> chain of setters -> `build()`. The key consistency rules are naming, return types, and error handling.

#### Canonical Example: `reqwest`

`reqwest` has two builders that follow identical conventions:

**`ClientBuilder`** — builds a `Client`:
```rust
impl ClientBuilder {
    pub fn new() -> Self { ... }                    // or Client::builder()
    pub fn user_agent<V>(self, value: V) -> Self { ... }  // setter returns Self
    pub fn timeout(self, timeout: Duration) -> Self { ... }
    pub fn build(self) -> Result<Client> { ... }    // finalizing method
}
```

**`RequestBuilder`** — builds a `Request`:
```rust
impl RequestBuilder {
    pub fn header<K, V>(self, key: K, value: V) -> Self { ... }
    pub fn body<T: Into<Body>>(self, body: T) -> Self { ... }
    pub fn query<T: Serialize>(self, query: &T) -> Self { ... }
    pub fn build(self) -> Result<Request> { ... }
    pub fn send(self) -> impl Future<Output = Result<Response>> { ... }  // convenience
}
```

Docs:
- https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
- https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html

#### Consistency Rules Observed Across Crates

1. **Construction entry point**: `Type::new()` or `Type::builder()` — both are acceptable. `reqwest` uses both: `Client::new()` for defaults, `Client::builder()` for customization.

2. **Setter signature**: `fn setter(self, value: T) -> Self` — consumes and returns `Self` for chaining. Some setters accept `impl Into<T>` or `V: TryInto<T>` for ergonomics.

3. **Build method**: `fn build(self) -> Result<T>` — consumes the builder and returns `Result`. This is the standard. Some crates also provide a convenience method that calls `build()` and then does something (like `RequestBuilder::send()`).

4. **Error type**: `Result<T, Error>` where `Error` is a crate-specific error enum. Builders validate at `build()` time, not at setter time.

5. **Default trait**: Most builders implement `Default` as an alternative to `new()`.

6. **Private fields**: Builder fields are always private — users can only set them through setters.

7. **Optional parameters via `Option<T>` or `Into<Option<T>>`**: For parameters that may or may not be set, setters accept `impl Into<Option<Duration>>` or similar.

#### Example from `linfa-clustering`

`linfa-clustering` uses the builder pattern for algorithm hyperparameters:

```rust
pub struct KMeansParams { /* private fields */ }
pub struct KMeansValidParams { /* private fields */ }

// KMeansParams is the builder; KMeansValidParams is the validated result
```

- Docs: https://docs.rs/linfa-clustering/latest/linfa_clustering/struct.KMeansParams.html

#### Example from `tokio`

`tokio` uses builders for `Runtime` configuration:

```rust
pub struct RuntimeBuilder { /* ... */ }
impl RuntimeBuilder {
    pub fn new_current_thread() -> Self { ... }
    pub fn new_multi_thread() -> Self { ... }
    pub fn worker_threads(self, val: usize) -> Self { ... }
    pub fn enable_all(self) -> Self { ... }
    pub fn build(self) -> Result<Runtime> { ... }
}
```

#### Example from `serde`

While `serde` itself doesn't use builders heavily, the ecosystem convention it reinforces is:
- `Serialize`/`Deserialize` traits are the "interface"
- `Serializer`/`Deserializer` are the "builder-like" types that accumulate state
- `derive` macros generate the implementation

The `serde_json::Serializer` and `serde_json::Deserializer` have builder-like configuration:
```rust
pub struct Serializer<W, F = CompactFormatter> { /* ... */ }
pub struct Deserializer<R> { /* ... */ }
```

#### Recommended Builder Pattern for Community Detection

```rust
/// Builder for configuring community detection algorithms.
#[derive(Debug, Clone)]
pub struct LouvainBuilder {
    resolution: f64,
    max_iterations: usize,
    tolerance: f64,
    seed: Option<u64>,
}

impl LouvainBuilder {
    pub fn new() -> Self {
        Self {
            resolution: 1.0,
            max_iterations: 100,
            tolerance: 1e-6,
            seed: None,
        }
    }

    pub fn resolution(mut self, value: f64) -> Self {
        self.resolution = value;
        self
    }

    pub fn max_iterations(mut self, value: usize) -> Self {
        self.max_iterations = value;
        self
    }

    pub fn tolerance(mut self, value: f64) -> Self {
        self.tolerance = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn build(self) -> Louvain {
        Louvain { params: self }
    }
}

impl Default for LouvainBuilder {
    fn default() -> Self { Self::new() }
}
```

Usage:
```rust
let louvain = LouvainBuilder::new()
    .resolution(0.8)
    .max_iterations(50)
    .seed(42)
    .build();
```

---

## Summary of Recommendations for `communal`

| Pattern | Recommended Approach | Primary Source |
|---------|---------------------|----------------|
| Quality metric trait | `fn score(&self) -> Result<F>` (linfa-style) or `fn evaluate(&self, g: &G, p: &P) -> MetricValue` | linfa `SilhouetteScore`, smartcore `Metrics` |
| Internal vs public separation | Sealed trait for internal quality fns; `pub trait CommunityMetric` for public; `default_metric()` link | `perpetual::ObjectiveFunction`, Rust visibility |
| Aggregator pattern | `CommunityMetrics` struct with `modularity()`, `conductance()`, `all()` methods | smartcore `ClusterMetrics` |
| Newtype pattern | `NodeId(u32)`, `CommunityId(u32)`, `Modularity(f64)` with `Debug+Copy+Eq+Ord+Hash+Display` | `petgraph::NodeIndex` |
| Builder pattern | `LouvainBuilder::new()...build()` with `Default` impl, private fields, `Result` from `build()` | `reqwest::ClientBuilder`, `linfa-clustering` |

---

## Sources

1. [linfa::metrics - Docs.rs](https://docs.rs/linfa/latest/linfa/metrics/index.html)
2. [linfa::metrics::SilhouetteScore - Docs.rs](https://docs.rs/linfa/latest/linfa/metrics/trait.SilhouetteScore.html)
3. [linfa::traits - Docs.rs](https://docs.rs/linfa/latest/linfa/traits/index.html)
4. [linfa_clustering - Docs.rs](https://docs.rs/linfa-clustering/latest/linfa_clustering/index.html)
5. [smartcore::metrics - Docs.rs](https://docs.rs/smartcore/latest/smartcore/metrics/index.html)
6. [smartcore::metrics::Metrics trait - Docs.rs](https://docs.rs/smartcore/latest/smartcore/metrics/trait.Metrics.html)
7. [smartcore::metrics::ClassificationMetrics - Docs.rs](https://docs.rs/smartcore/latest/smartcore/metrics/struct.ClassificationMetrics.html)
8. [smartcore::metrics::ClusterMetrics - Docs.rs](https://docs.rs/smartcore/latest/smartcore/metrics/struct.ClusterMetrics.html)
9. [perpetual::objective::core::ObjectiveFunction - Docs.rs](https://docs.rs/perpetual/latest/perpetual/objective/core/trait.ObjectiveFunction.html)
10. [petgraph::graph::NodeIndex - Docs.rs](https://docs.rs/petgraph/latest/petgraph/graph/struct.NodeIndex.html)
11. [reqwest::ClientBuilder - Docs.rs](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html)
12. [reqwest::RequestBuilder - Docs.rs](https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html)
13. [Rust API Guidelines - Future Proofing (C-SEALED)](https://rust-lang.github.io/api-guidelines/future_proofing.html)
14. [DeepWiki - Sealed Traits and Encapsulation](https://deepwiki.com/rust-lang/api-guidelines/7.1-sealed-traits-and-encapsulation)
15. [Effective Rust - Item 22: Minimize visibility](https://www.effective-rust.com/visibility.html)
16. [Rust Reference - Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
17. [Idiomatic Rust Snippets - Newtype Pattern](https://idiomatic-rust-snippets.org/patterns/rust-idioms/newtype.html)
18. [DeepWiki - Type Safety Patterns](https://deepwiki.com/rust-lang/api-guidelines/4.2-type-safety-patterns)
19. [sealed crate - Docs.rs](https://docs.rs/sealed/latest/sealed/)
20. [metrique-aggregation - Docs.rs](https://docs.rs/metrique-aggregation)
21. [metrics-rs/metrics - GitHub](https://github.com/metrics-rs/metrics)
22. [petgraph - GitHub](https://github.com/petgraph/petgraph)
23. [tch-rs - GitHub](https://github.com/LaurentMazare/tch-rs)
24. [rust-ml/linfa - GitHub](https://github.com/rust-ml/linfa)
25. [smartcorelib/smartcore - GitHub](https://github.com/smartcorelib/smartcore)

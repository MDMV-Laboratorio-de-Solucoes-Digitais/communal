# Remediation of Constitution Violations

**Date**: 2026-05-24  
**Scope**: 5 constitution violations in specs/001-community-detection/tasks.md  
**References**: `.specify/memory/constitution.md`, `specs/001-community-detection/spec.md`, `specs/001-community-detection/tasks.md`

---

## Violation 1: QualityMetric Trait in Wrong Crate

### Problem Statement

**Spec says (FR-021):**
> The system MUST define a `QualityMetric` trait in `communal-core` with the following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`.

**Tasks.md T073 says:**
> Implement `QualityMetric` trait in `crates/communal-metrics/src/lib.rs`

This is a constitution violation. The spec explicitly mandates the trait live in `communal-core`, but the task places it in `communal-metrics`.

### Root Cause Analysis

The dependency hierarchy in the constitution is:

```
communal-core (no internal dependencies)
├── communal-algo (depends on communal-core)
├── communal-metrics (depends on communal-core)
```

If `QualityMetric` lives in `communal-metrics`, then `communal-algo` (which implements `QualityMetric` for internal optimization per FR-021) would need to depend on `communal-metrics`. This creates either:
1. A circular dependency (communal-metrics depends on communal-core for types, communal-algo depends on communal-metrics for the trait, communal-metrics might need communal-algo for implementations), OR
2. An inversion where the algorithm crate depends on the metrics crate — architecturally wrong since metrics are a *consumer* of algorithms, not a dependency.

### Idiomatic Rust Pattern: Trait Placement in Workspace Crates

The rule for trait placement in Rust workspaces is:

> **Define traits in the lowest crate in the dependency graph that all implementors depend on.**

This follows from Rust's **orphan rule** — to implement a trait for a type, either the trait or the type must be local to the crate. Since multiple crates (`communal-algo`, `communal-metrics`) need to implement `QualityMetric`, and they only share `communal-core` as a common dependency, the trait MUST live in `communal-core`.

**Sources:**
- [Rust API Guidelines — C-TRAIT](https://rust-lang.github.io/api-guidelines/interoperability.html): "Traits should be defined in the crate that contains the primary types they operate on, or in a foundational crate."
- [The Rust Programming Language, Traits chapter](https://doc.rust-lang.org/book/ch10-02-traits.html): Implementing a trait requires the trait or type to be local.
- Constitution Section III, Dependency Flow: `communal-core` is the foundational crate with "no internal dependencies."

### Recommended Fix

1. **Move `QualityMetric` trait definition** from `crates/communal-metrics/src/lib.rs` to `crates/communal-core/src/quality_metric.rs` (or `crates/communal-core/src/lib.rs` if minimal).
2. **Update T073** to: *Implement `QualityMetric` trait in `crates/communal-core/src/quality_metric.rs`*.
3. **T074-T078 remain in communal-metrics**: Each metric struct (`Modularity`, `ConstantPottsModel`, `MapEquation`, `Nmi`, `Ari`) implements the `QualityMetric` trait. Since `communal-metrics` depends on `communal-core`, it can see and implement the trait.
4. **Internal implementations in communal-algo**: The internal `QualityFunction` enum (T034) and quality function implementations (T035-T036a) can also implement `QualityMetric` since `communal-algo` depends on `communal-core`.

### Concrete Code Change

```rust
// crates/communal-core/src/quality_metric.rs

/// Trait for quality metrics that evaluate partition quality.
///
/// This trait is defined in communal-core so that both communal-algo
/// (for internal optimization) and communal-metrics (for public evaluation)
/// can implement it without circular dependencies.
pub trait QualityMetric {
    /// Evaluate the quality of a single partition.
    fn evaluate<G: GraphView>(
        &self,
        graph: &G,
        partition: &Partition,
    ) -> Result<f64, MetricsError>;
}
```

```rust
// crates/communal-metrics/src/lib.rs
use communal_core::QualityMetric;  // Import, don't redefine

pub struct Modularity;
impl QualityMetric for Modularity { /* ... */ }
```

### Impact on Other Files/Tasks

| Task | Change Required |
|------|----------------|
| T073 | File path: `communal-metrics/src/lib.rs` → `communal-core/src/quality_metric.rs` |
| T074-T078 | Add `use communal_core::QualityMetric;` at top of each file |
| T034-T036a | Add `use communal_core::QualityMetric;` for internal implementations |
| communal-core/src/lib.rs | Add `pub mod quality_metric;` |
| communal-metrics/Cargo.toml | Already depends on communal-core (no change needed) |

---

## Violation 2: AlgorithmError in Wrong Crate

### Problem Statement

**Spec says (FR-044):**
> Domain error types MUST be split into per-module error enums: `GraphError` (graph construction, validation, parsing), `AlgorithmError` (configuration, convergence, execution), `PartitionError` (query, invalid access), `MetricsError` (computation, comparison).

**Tasks.md T017 says:**
> Implement domain error types (GraphError, AlgorithmError, PartitionError) in `crates/communal-core/src/error.rs`

This is a constitution violation. `AlgorithmError` covers algorithm-specific failures (configuration, convergence, execution) and belongs in `communal-algo`, not `communal-core`.

### Root Cause Analysis

The spec mandates **per-module error types**. The modules map to crates:

| Error Type | Module/Crate | Domain |
|-----------|-------------|--------|
| `GraphError` | communal-core | Graph construction, validation, parsing |
| `PartitionError` | communal-core | Query, invalid access to partitions |
| `AlgorithmError` | communal-algo | Configuration, convergence, execution |
| `MetricsError` | communal-metrics | Computation, comparison |

`communal-core` should only own errors that arise from its own operations (graph construction, validation, partition queries). Algorithm-specific errors naturally belong in `communal-algo`.

### Idiomatic Rust Pattern: Per-Module Error Types

The idiomatic Rust pattern for workspaces is:

> **Errors live in the crate that produces them, with `#[from]` conversions at crate boundaries.**

This is the pattern recommended by `thiserror`'s documentation and used by major Rust ecosystem crates:

- `serde_json::Error` lives in `serde_json`, not `serde`
- `tokio::io::Error` wraps `std::io::Error` via `From` conversions
- `sqlx::Error` encompasses database-specific errors

**Sources:**
- [thiserror documentation](https://docs.rs/thiserror/2.0/thiserror/): "The error type should be an enum whose variants represent the different errors that can occur."
- [Rust API Guidelines — C-ERROR](https://rust-lang.github.io/api-guidelines/interoperability.html): "Error types should be meaningful and well-structured."
- Constitution FR-044: "Per-module error types with unified re-export" — the unified re-export lives in the facade crate.

### Recommended Fix

1. **Split T017 into three tasks** — one per crate:
   - T017a: Implement `GraphError` and `PartitionError` in `crates/communal-core/src/error.rs`
   - T017b: Implement `AlgorithmError` in `crates/communal-algo/src/error.rs`
   - T017c: Implement `MetricsError` in `crates/communal-metrics/src/error.rs` (already T080)
2. **Facade re-export** (per FR-044): The `communal` facade crate provides a `CommunalError` enum with `From` conversions:
   ```rust
   // communal/src/lib.rs
   pub enum CommunalError {
       Graph(communal_core::GraphError),
       Partition(communal_core::PartitionError),
       Algorithm(communal_algo::AlgorithmError),
       Metrics(communal_metrics::MetricsError),
       Dynamic(communal_dynamic::DynamicError),
   }
   ```

### Concrete Code Change

```rust
// crates/communal-core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("negative edge weight {weight} between {source} and {target}")]
    NegativeWeight { source: u32, target: u32, weight: f64 },
    // ... other variants
}

#[derive(Debug, Error)]
pub enum PartitionError {
    #[error("invalid node ID: {0}")]
    InvalidNodeId(u32),
    #[error("invalid hierarchy level: {0}")]
    InvalidLevel(usize),
    // ... other variants
}
```

```rust
// crates/communal-algo/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlgorithmError {
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("failed to converge within {0} iterations")]
    NonConvergence(usize),
    // ... other variants
}
```

### Impact on Other Files/Tasks

| Task | Change Required |
|------|----------------|
| T017 | Remove `AlgorithmError` from `communal-core/src/error.rs` |
| T080 | Add `AlgorithmError` to `communal-algo/src/error.rs` (already there — just ensure it's complete) |
| T017a (error tests) | Split tests: communal-core tests for GraphError/PartitionError, communal-algo tests for AlgorithmError |
| communal/src/lib.rs | Add `CommunalError` re-export with `From` conversions |

---

## Violation 3: DynamicError Not in Spec

### Problem Statement

**Spec says (FR-044):**
> Domain error types MUST be split into per-module error enums: `GraphError`, `AlgorithmError`, `PartitionError`, `MetricsError`.

**Tasks.md T097 creates:**
> Implement `DynamicError` type in `crates/communal-dynamic/src/error.rs`

And T017a references:
> DynamicError (4: invalid level, split failure, complexity exceeded, mutation error)

The spec defines exactly 4 error types. The tasks create a 5th type (`DynamicError`) not in the spec.

### Root Cause Analysis

The question is whether dynamic graph errors deserve their own error type or should be folded into existing types.

Arguments for **adding as 5th type**:
- `communal-dynamic` is a separate crate with its own error domain
- Dynamic graph operations have unique failure modes (split failures, complexity violations, mutation errors)
- Consistency with the "per-module error types" pattern — each crate owns its errors
- `DynamicError` provides type safety and enables programmatic handling of dynamic-specific failures

Arguments for **folding into AlgorithmError**:
- Dynamic detection is still algorithmically-driven; errors like "split failure" or "mutation error" could be seen as algorithmic failures
- The spec only defines 4 types; adding a 5th is a spec change

### Idiomatic Rust Pattern: Extending Error Types

The idiomatic approach is:

> **Each crate in a workspace owns its error type. When a new crate is added with distinct error modes, it gets its own error enum.**

This is the pattern used by:
- `tokio` has `tokio::io::Error` distinct from `std::io::Error`
- `reqwest` has its own error type wrapping underlying HTTP errors
- `serde` crates each have their own error type

The spec's "at minimum" wording in FR-044 ("covering at minimum: ...") implies the list is extensible. Dynamic graph operations have genuinely distinct error modes that don't fit neatly into the other four categories.

### Recommended Fix

1. **Update spec FR-044** to add `DynamicError` as a 5th per-module error type:
   > Domain error types MUST be split into per-module error enums: `GraphError` (graph construction, validation, parsing), `AlgorithmError` (configuration, convergence, execution), `PartitionError` (query, invalid access), `MetricsError` (computation, comparison), `DynamicError` (mutation processing, split complexity, hierarchy management).

2. **Keep T097 as-is** — it's correct given the spec update.

3. **T017a remains valid** — the 4 DynamicError scenarios are appropriate test cases.

### Concrete Spec Edit

```markdown
// In spec.md, FR-044:
- **FR-044**: Domain error types MUST be split into per-module error enums:
  - `GraphError` (graph construction, validation, parsing)
  - `AlgorithmError` (configuration, convergence, execution)
  - `PartitionError` (query, invalid access)
  - `MetricsError` (computation, comparison)
  - `DynamicError` (mutation processing, split complexity, hierarchy management)
  Each error type uses `thiserror` with descriptive messages. A unified `CommunalError`
  re-export MAY be provided at the facade crate level for users who want a single error
  type, with `From` conversions for each per-module error.
```

### Impact on Other Files/Tasks

| Item | Change Required |
|------|----------------|
| spec.md FR-044 | Add `DynamicError` as 5th error type |
| spec.md Key Entities | Add `DynamicError` to Domain Errors description |
| T097 | No change needed (correct as-is) |
| T017a | No change needed |
| communal/src/lib.rs | Add `Dynamic(communal_dynamic::DynamicError)` variant to `CommunalError` |

---

## Violation 4: DynamicDetector Trait Not in Spec

### Problem Statement

**Spec says (FR-045):**
> The system MUST define a `StreamingDetector: CommunityDetector` trait for algorithms supporting incremental dynamic graph updates. This trait extends `CommunityDetector` with mutation methods.

**Tasks.md T088 creates:**
> Implement `DynamicDetector` trait in `crates/communal-dynamic/src/detector.rs`

The spec defines `StreamingDetector`, but the tasks create `DynamicDetector`. These appear to be distinct traits.

### Root Cause Analysis

There are two possible interpretations:

1. **They are the same concept**: The task author used `DynamicDetector` as an internal name, but the spec mandates `StreamingDetector` as the public API. This is a naming inconsistency, not a design conflict.

2. **They are different concepts**: `DynamicDetector` could be an internal trait for dynamic graph operations (graph mutations, hierarchy management), while `StreamingDetector` is the spec-mandated public trait that extends `CommunityDetector`.

Looking at T087 vs T088:
- T087: *Implement `DynamicGraph` trait* — for graph-level dynamic operations
- T088: *Implement `DynamicDetector` trait* — for detector-level dynamic operations

And T096: *Implement `StreamingDetector`* — the spec-mandated trait

This suggests a likely relationship: `StreamingDetector` (public API) might be a supertrait or wrapper around `DynamicDetector` (internal abstraction).

### Idiomatic Rust Pattern: Internal vs Public Traits

The idiomatic Rust pattern is:

> **Define internal traits for implementation flexibility, and public traits (possibly sealed) for the external API.**

This is used by:
- `std::io::Read` (public) vs internal `read_buf` mechanisms
- `serde::Serialize` (public) vs internal `Serializer` trait
- `tokio::io::AsyncRead` (public) vs internal reactor traits

The relationship should be:
- `DynamicDetector` (internal, in communal-dynamic) — handles the mechanics of incremental updates, subtree management, and community splitting
- `StreamingDetector` (public, specified in spec) — extends `CommunityDetector` with `apply_mutation`/`apply_mutations`, implemented by wrapping `DynamicDetector` logic

Alternatively, `DynamicDetector` could be the internal trait name for what is exposed as `StreamingDetector`. In that case, the task should use the spec-mandated name.

### Recommended Fix

1. **Rename T088 to use `DynamicDetector` as an internal trait** that is separate from `StreamingDetector`. Define their relationship:

```rust
// crates/communal-dynamic/src/detector.rs

/// Internal trait for dynamic graph detection operations.
/// This is the implementation detail that powers StreamingDetector.
pub(crate) trait DynamicDetector {
    fn apply_mutation_internal(&mut self, mutation: EdgeMutation) -> Result<Partition, DynamicError>;
    fn apply_mutations_internal(&mut self, mutations: Vec<EdgeMutation>) -> Result<Partition, DynamicError>;
    fn split_community(&mut self, community: CommunityId) -> Result<Vec<CommunityId>, DynamicError>;
}
```

2. **T096 implements `StreamingDetector`** as the spec-mandated public trait, which delegates to `DynamicDetector`:

```rust
// crates/communal-dynamic/src/streaming.rs
use communal_core::CommunityDetector;

/// Public API trait extending CommunityDetector with mutation support (per FR-045).
pub trait StreamingDetector: CommunityDetector {
    fn apply_mutation(&mut self, mutation: EdgeMutation) -> Result<Partition, CommunalError>;
    fn apply_mutations(&mut self, mutations: Vec<EdgeMutation>) -> Result<Partition, CommunalError>;
}
```

3. **Update spec FR-045** to mention the internal `DynamicDetector` trait as an implementation detail:

```markdown
// spec.md, after FR-045:
**Implementation note**: The `StreamingDetector` trait may be backed by an internal
`DynamicDetector` trait that handles the mechanics of incremental updates, subtree
management, and community splitting. The internal trait is not part of the public API.
```

### Concrete Code Change

```rust
// crates/communal-dynamic/src/detector.rs

/// Internal trait for dynamic community detection.
///
/// This trait provides the low-level machinery for incremental graph updates.
/// It is used internally by `StreamingDetector` (the public API) but is not
/// exposed in the public API.
pub(crate) trait DynamicDetector {
    /// Process a single mutation, returning the updated partition.
    fn apply_single(&mut self, mutation: &EdgeMutation) -> Result<Partition, DynamicError>;

    /// Process multiple mutations sequentially.
    fn apply_batch(&mut self, mutations: &[EdgeMutation]) -> Result<Partition, DynamicError>;

    /// Split a community into its connected components.
    fn decompose(&mut self, community: CommunityId) -> Result<Vec<CommunityId>, DynamicError>;
}
```

```rust
// crates/communal-dynamic/src/streaming.rs

use communal_core::{CommunityDetector, Partition, EdgeMutation};

/// Public trait for streaming community detection (per FR-045).
///
/// Extends `CommunityDetector` with methods for incremental graph updates.
/// This is the spec-mandated public API for dynamic graph support.
pub trait StreamingDetector: CommunityDetector {
    /// Apply a single edge mutation and return the updated partition.
    fn apply_mutation(&mut self, mutation: EdgeMutation) -> Result<Partition, CommunalError>;

    /// Apply a batch of edge mutations and return the final partition.
    fn apply_mutations(&mut self, mutations: Vec<EdgeMutation>) -> Result<Partition, CommunalError>;
}
```

### Impact on Other Files/Tasks

| Task | Change Required |
|------|----------------|
| T088 | Clarify that `DynamicDetector` is internal (not public API). Add `pub(crate)` visibility. |
| T096 | Ensure `StreamingDetector` is the public trait, delegating to `DynamicDetector` |
| spec.md FR-045 | Add implementation note about internal `DynamicDetector` |
| spec.md Key Entities | Optionally add `DynamicDetector` as an internal entity |
| communal/src/lib.rs | Export `StreamingDetector` (not `DynamicDetector`) |

---

## Violation 5: QualityMetric::evaluate() Signature Conflict

### Problem Statement

**FR-019 says:**
> Comparative metrics follow the `QualityMetric` trait pattern: each metric is a struct implementing `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`.

**FR-021 says:**
> The system MUST define a `QualityMetric` trait with the following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`.

**Conflict**: FR-019 describes comparative metrics (NMI, ARI) that take **2 partitions**, while FR-021 describes a trait that takes **1 partition**. A single trait cannot have both signatures.

### Root Cause Analysis

There are two categories of quality metrics:
1. **Absolute metrics** (Modularity Q, CPM, Map Equation) — evaluate a single partition against a graph
2. **Comparative metrics** (NMI, ARI) — compare two partitions to each other

The current spec tries to unify both under one `QualityMetric` trait with one `evaluate()` method, which is mathematically incoherent — they take different numbers of arguments.

### Idiomatic Rust Patterns for This Situation

Three idiomatic options exist:

#### Option A: Two Separate Traits (Recommended)

```rust
/// For metrics that evaluate a single partition's quality.
pub trait QualityMetric {
    fn evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>;
}

/// For metrics that compare two partitions (NMI, ARI).
pub trait ComparativeMetric {
    fn evaluate<G: GraphView>(
        &self,
        graph: &G,
        partition1: &Partition,
        partition2: &Partition,
    ) -> Result<f64, MetricsError>;
}
```

**Pros**: Clear separation of concerns, follows Single Responsibility Principle, each trait has one method.  
**Cons**: Two traits to learn, slight duplication of `evaluate` method name.

#### Option B: Single Trait with Enum Parameter

```rust
pub enum QualityInput<'a> {
    Single(&'a Partition),
    Comparative(&'a Partition, &'a Partition),
}

pub trait QualityMetric {
    fn evaluate<G: GraphView>(&self, graph: &G, input: QualityInput<'_>) -> Result<f64, MetricsError>;
}
```

**Pros**: One trait.  
**Cons**: Implementors must match on `input`, runtime error if comparative metric receives single partition. Violates Rust's preference for compile-time safety.

#### Option C: One Trait with Two Methods

```rust
pub trait QualityMetric {
    fn evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>;
    fn compare<G: GraphView>(
        &self,
        graph: &G,
        partition1: &Partition,
        partition2: &Partition,
    ) -> Result<f64, MetricsError> {
        unimplemented!("Not all metrics support comparison")
    }
}
```

**Pros**: One trait, discoverable API.  
**Cons**: Violates "Zero Panic" constitution principle (default impl calls `unimplemented!()`). Metrics that don't support comparison must return `Err` or panic.

### Recommended Fix: Option A (Two Separate Traits)

**Sources:**
- [Rust API Guidelines — C-COHESION](https://rust-lang.github.io/api-guidelines/cohesion.html): "Type and trait names should be cohesive — a trait should represent a single concept."
- [The Rust Programming Language, Trait Objects chapter](https://doc.rust-lang.org/book/ch17-02-trait-objects.html): Traits should have clear, focused contracts.
- Constitution Principle IV ("Zero Technical Debt"): `unimplemented!()` in default impls is forbidden by `#![deny(clippy::unimplemented)]`.

Two traits is the cleanest solution because:
1. Absolute and comparative metrics are mathematically different operations
2. Each trait has exactly one responsibility
3. No `unimplemented!()` or runtime errors
4. Type system enforces correct usage at compile time
5. Follows the principle from FR-019: "No separate `MetricsCalculator` type is introduced" — we don't need a calculator service, just the right trait boundaries

### Concrete Spec Edit

```markdown
// Replace FR-019 and FR-021 with:

- **FR-019**: The system MUST compute comparative metrics: Normalized Mutual Information (NMI)
  and Adjusted Rand Index (ARI). Comparative metrics implement the `ComparativeMetric` trait
  (FR-021b) with `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`.

- **FR-021**: The system MUST define a `QualityMetric` trait in `communal-core` with the
  following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`.
  This trait MUST be implemented by all absolute quality metrics (Modularity Q, CPM, Map Equation)
  used for algorithm optimization. Internal quality functions MUST be placed in an `internal`
  module and marked with `#[doc(hidden)]`.

- **FR-021b**: The system MUST define a `ComparativeMetric` trait in `communal-core` with the
  following contract: `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`.
  This trait MUST be implemented by all comparative metrics (NMI, ARI) used for partition comparison.
```

### Concrete Code Change

```rust
// crates/communal-core/src/quality_metric.rs

/// For metrics that evaluate a single partition's quality against a graph.
///
/// Implemented by: Modularity Q, CPM, Map Equation (absolute metrics).
pub trait QualityMetric {
    /// Evaluate partition quality.
    fn evaluate<G: GraphView>(
        &self,
        graph: &G,
        partition: &Partition,
    ) -> Result<f64, MetricsError>;
}

/// For metrics that compare two partitions.
///
/// Implemented by: Normalized Mutual Information (NMI), Adjusted Rand Index (ARI).
pub trait ComparativeMetric {
    /// Compare two partitions and return a similarity score.
    fn evaluate<G: GraphView>(
        &self,
        graph: &G,
        partition1: &Partition,
        partition2: &Partition,
    ) -> Result<f64, MetricsError>;
}
```

```rust
// crates/communal-metrics/src/nmi.rs
use communal_core::{ComparativeMetric, GraphView, Partition, MetricsError};

pub struct NormalizedMutualInformation;

impl ComparativeMetric for NormalizedMutualInformation {
    fn evaluate<G: GraphView>(
        &self,
        graph: &G,
        partition1: &Partition,
        partition2: &Partition,
    ) -> Result<f64, MetricsError> {
        // NMI computation
    }
}
```

```rust
// crates/communal-metrics/src/modularity.rs
use communal_core::{QualityMetric, GraphView, Partition, MetricsError};

pub struct Modularity;

impl QualityMetric for Modularity {
    fn evaluate<G: GraphView>(
        &self,
        graph: &G,
        partition: &Partition,
    ) -> Result<f64, MetricsError> {
        // Modularity Q computation
    }
}
```

### Impact on Other Files/Tasks

| Task | Change Required |
|------|----------------|
| T073 | Define both `QualityMetric` and `ComparativeMetric` traits |
| T074-T076 | Implement `QualityMetric` (absolute metrics) |
| T077-T078 | Implement `ComparativeMetric` (NMI, ARI) |
| T079 (report types) | Update to use both traits |
| spec.md FR-019, FR-021 | Split into FR-021 (QualityMetric) and FR-021b (ComparativeMetric) |
| spec.md Key Entities | Add `ComparativeMetric` trait entity |
| communal/src/lib.rs | Export both traits |

---

## Summary of All Changes

### Tasks.md Updates

| Original Task | Change |
|--------------|--------|
| T017 | Remove `AlgorithmError` — move to T080 (communal-algo) |
| T073 | Move `QualityMetric` to `communal-core/src/quality_metric.rs`; add `ComparativeMetric` trait |
| T074-T076 | Implement `QualityMetric` (already correct crate) |
| T077-T078 | Implement `ComparativeMetric` instead of `QualityMetric` |
| T088 | Clarify `DynamicDetector` is internal (`pub(crate)`); add relationship doc to `StreamingDetector` |
| T097 | Keep `DynamicError` — requires spec FR-004 update |
| T096 | Ensure `StreamingDetector` is public API wrapping `DynamicDetector` |

### Spec.md Updates

| FR | Change |
|----|--------|
| FR-019 | Add reference to `ComparativeMetric` trait |
| FR-021 | Clarify signature is for absolute metrics only |
| FR-021b (new) | Add new FR for `ComparativeMetric` trait |
| FR-044 | Add `DynamicError` as 5th per-module error type |
| FR-045 | Add implementation note about internal `DynamicDetector` |

### Cargo.toml Updates

| Crate | Change |
|-------|--------|
| communal-core/Cargo.toml | Add `thiserror`, `num-traits` dependencies |
| communal-algo/Cargo.toml | Already depends on communal-core; no change |
| communal-metrics/Cargo.toml | Already depends on communal-core; no change |

### Dependency Verification

After fixes, the dependency graph remains acyclic:

```
communal-core (depends on: nothing)
├── communal-algo (depends on: communal-core) — owns AlgorithmError
├── communal-metrics (depends on: communal-core) — owns MetricsError
├── communal-dynamic (depends on: communal-core, communal-algo) — owns DynamicError
└── communal (facade, depends on all) — re-exports CommunalError
```

No circular dependencies introduced. All traits in `communal-core` can be implemented by any downstream crate.

---

## Sources & References

1. **Rust API Guidelines** — [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/)
   - C-TRAIT: Trait placement and design
   - C-ERROR: Error type design
   - C-COHESION: Cohesion of type and trait names

2. **The Rust Programming Language** — [https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)
   - Chapter 10: Traits
   - Chapter 17: Trait Objects
   - Chapter 19: Advanced Traits

3. **thiserror crate documentation** — [https://docs.rs/thiserror/](https://docs.rs/thiserror/)
   - Error enum patterns in workspace crates

4. **Constitution Section III** — Workspace Architecture & Crate Organization
   - Dependency flow hierarchy
   - Feature flag matrix

5. **Spec FR-019, FR-021, FR-044, FR-045** — Functional requirements driving the fixes

# Remediation of Underspecified Items — Community Detection Framework

**Date**: 2026-03-09
**Spec**: `specs/001-community-detection/spec.md`
**Purpose**: Resolve 8 underspecified types/APIs referenced in the spec by providing concrete type definitions, signatures, rationale with sources, usage examples, and impact analysis.

---

## Item 1: Resolution Parameter Gamma (FR-008)

### Research Findings

**Academic sources.** In the foundational Leiden paper (Traag, Waltman & van Eck, 2019, *Scientific Reports* "From Louvain to Leiden"), the resolution parameter is denoted **γ (gamma)** and appears in the Constant Potts Model (CPM) and modularity quality functions. The paper states: *"γ refers to the resolution parameter in the quality function that is optimised, which can be either modularity or CPM"* and *"Higher resolutions lead to more communities and lower resolutions lead to fewer communities"* [Nature 2019, s41598-019-41695-z].

**Reference implementations.**
- **scanpy.tl.leiden**: `resolution: float (default: 1)` — "A parameter value controlling the coarseness of the clustering. Higher values lead to more clusters." [scanpy docs]
- **networkx.louvain_communities**: `resolution: float, optional (default=1)` — "If resolution is less than 1, the algorithm favors larger communities. Greater than 1 favors smaller communities." [NetworkX 3.6.1]
- **leidenAlg (R interface to leidenalg)**: `resolution: Numeric scalar (default=1.0)` — "Higher resolutions lead to more [communities]." [RDocumentation / CRAN]
- **cdlib.algorithms.leiden**: `resolution_parameter: double > 0` — explicitly documents the constraint as **strictly positive** [CDlib 0.4.0]

**Valid range.** All authoritative implementations agree gamma must be **strictly positive** (γ > 0). There is no canonical upper bound in the literature; it is graph-dependent. CPM with γ → 0 collapses all nodes into a single community; γ → ∞ puts each node in its own community. The value γ = 1.0 recovers standard Newman-Girvan modularity (the "natural" scale).

**Default.** The universal default across all reference implementations is **1.0**.

### Proposed Type Definition

```rust
/// Resolution parameter γ for quality functions (CPM, modularity).
///
/// Must be strictly positive. The value 1.0 corresponds to the natural
/// resolution scale (standard modularity). Higher values yield more,
/// smaller communities; lower values (0 < γ < 1) yield fewer, larger communities.
///
/// # Validation
/// `Gamma::new(0.0)` and `Gamma::new(f64::NAN)` return `Err(AlgorithmError::InvalidConfiguration)`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Gamma(f64);

impl Gamma {
    /// Natural resolution scale (standard Newman-Girvan modularity).
    pub const NATURAL: Gamma = Gamma(1.0);

    /// Creates a new Gamma resolution parameter.
    ///
    /// # Errors
    /// Returns `Err(AlgorithmError::InvalidConfiguration)` if `value` is
    /// not strictly positive, is NaN, or is infinite.
    pub fn new(value: f64) -> Result<Self, AlgorithmError> {
        if value.is_finite() && value > 0.0 {
            Ok(Gamma(value))
        } else {
            Err(AlgorithmError::InvalidConfiguration {
                parameter: "gamma",
                message: format!(
                    "resolution parameter gamma must be strictly positive and finite, got {value}"
                ),
            })
        }
    }

    /// Returns the underlying f64 value.
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for Gamma {
    fn default() -> Self {
        Self::NATURAL
    }
}
```

### Rationale

Using a **newtype wrapper** (`Gamma(f64)`) rather than a raw `f64` provides:
1. **Type safety** — callers cannot pass an unvalidated `f64` where a `Gamma` is expected.
2. **Validation at construction** — the invariant γ > 0 is enforced once, at the boundary, not scattered across algorithm internals.
3. **Self-documenting signatures** — `gamma: Gamma` is clearer than `gamma: f64`.

The newtype pattern is idiomatic Rust (e.g., `Meters(f64)`, `Ecdf(f64)` in the ecosystem). Since the spec mandates runtime validation with typed errors (FR-029), construction-time validation fits the existing `thiserror` convention.

### Impact on Spec

- **FR-008** text update: "The system MUST support a resolution parameter **gamma: Gamma** for both CPM and modularity optimization, where Gamma is a strictly-positive f64 newtype (γ > 0.0), default 1.0."
- **AlgorithmConfig trait**: Add `fn resolution(&self) -> Gamma` as a required method.
- **Key Entities** section: Update "Resolution Parameter (gamma)" entry to reference the `Gamma` newtype, valid range (0, ∞), default 1.0.

---

## Item 2: StepCallback Trait

### Research Findings

Rust callback idioms fall into three categories [Stack Overflow "Idiomatic callbacks in Rust"; CodeTrail.io]:
1. **Function pointers** (`fn(...)`) — zero overhead, no state capture.
2. **Generic `F: FnMut(...)`** — monomorphized, captures state, no allocation.
3. **Trait objects `Box<dyn FnMut(...)>`** — type-erased, dynamic dispatch, allocation cost.

For an **observer registration** API (which stores heterogeneous callbacks), the idiomatic pattern is a **trait object with `Send` bound**, stored as `Box<dyn StepCallback>`. The spec already determined the bound: `Send` only (single-threaded dispatch, allows mutable callback state) [Session 2026-09-04 (3)].

The event should be **borrowed** (`&StepEvent`) because:
- Callers may want to handle an event without consuming it.
- Multiple observers may receive the same event (Fan-out dispatch per FR-046: "events are dispatched to all registered observers").
- A callback taking ownership would prevent further dispatch.

### Proposed Trait Definition

```rust
/// Callback trait for observing algorithmic step events.
///
/// Implementors receive borrowed `StepEvent` references during algorithm
/// execution. The trait bound is `Send` only (not `Send + Sync`), allowing
/// callbacks to hold mutable state that is not shared across threads.
///
/// Callbacks are dispatched in registration order (FIFO). A panic in one
/// callback MUST NOT crash the algorithm (isolated via `std::panic::catch_unwind`).
pub trait StepCallback: Send {
    /// Called when a discrete algorithmic event occurs.
    ///
    /// `event` is borrowed to allow dispatch to multiple observers.
    fn on_event(&mut self, event: &StepEvent);
}

// Type-erased storage alias for internal use
type BoxedCallback = Box<dyn StepCallback + Send + 'static>;
```

**Concrete implementations for common cases:**

```rust
/// A no-op callback for the "disabled" case (zero-cost when compiled out).
pub struct NoOpCallback;
impl StepCallback for NoOpCallback {
    fn on_event(&mut self, _event: &StepEvent) {}
}

/// Collects all events into a `Vec` for post-hoc analysis.
pub struct EventCollector {
    pub events: Vec<StepEvent>,  // StepEvent must be Clone
}
impl StepCallback for EventCollector {
    fn on_event(&mut self, event: &StepEvent) {
        self.events.push(event.clone());
    }
}
```

### Rationale

**`fn on_event(&mut self, event: &StepEvent)`** was chosen over alternatives:

| Alternative | Reason for rejection |
|---|---|
| `fn handle(&mut self, event: StepEvent)` | Takes ownership — prevents multi-observer fan-out (FR-046 requires FIFO dispatch to all observers). |
| `fn on_event(&mut self, event: StepEvent) -> CallbackResult` | Return value adds complexity; spec says callbacks allow "external synchronization" not flow control. Cancellation can be a separate `CancellationCallback` trait if needed later. |
| `fn on_event(&self, event: &StepEvent)` | `&self` forbids mutable state, contradicting spec Session 2026-09-04 (3) which explicitly chose `Send` to allow mutable state. |
| Function pointer `fn(&StepEvent)` | Cannot capture state (e.g., a TUI handle or collector buffer), which defeats the use case. |

### Impact on Spec

- **FR-018**: "The callback interface (`StepCallback`) — trait method `fn on_event(&mut self, event: &StepEvent)`."
- **FR-046**: `subscribe(&mut self, observer: impl StepCallback + Send + 'static) -> Subscription` — already consistent; no change needed.
- **Key Entities** "Step Callback" entry: replace "Trait bound: Send only" with the full signature.
- Requires `StepEvent: Clone` (for collector patterns and multi-observer dispatch). Add `#[derive(Clone)]` to `StepEvent`.

---

## Item 3: ConvergenceMode Enum

### Research Findings

The spec (FR-032) states: *"The convergence mode (absolute change |Q_current - Q_previous| or relative change |Q_current - Q_previous| / |Q_current|) MUST be configurable via algorithm-specific configuration, defaulting to absolute change."*

This maps directly to the standard mathematical definitions used in iterative optimization:
- **Absolute change**: `|Q_{i} - Q_{i-1}| < ε` — measures raw improvement magnitude.
- **Relative change**: `|Q_{i} - Q_{i-1}| / |Q_{i}| < ε` — normalizes by current quality magnitude, making the threshold scale-invariant to the quality function's absolute scale.

These are the two canonical convergence modes documented in the leidenalg/igraph literature. The `igraph::cluster_leiden` and `leidenalg::Optimiser` expose an `improve` function that checks quality delta, and both support configuring absolute vs. relative tolerance [leidenAlg R docs; igraph docs].

### Proposed Enum Definition

```rust
/// Convergence criterion for terminating iterative optimization.
///
/// The algorithm terminates when the quality improvement between consecutive
/// iterations falls below the configured convergence threshold (see
/// `AlgorithmConfig::convergence_threshold`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ConvergenceMode {
    /// Absolute change: terminate when `|Q_current - Q_previous| < threshold`.
    ///
    /// This is the default. It measures raw improvement magnitude and matches
    /// the convention of canonical Leiden/Louvain implementations.
    #[default]
    Absolute,

    /// Relative change: terminate when `|Q_current - Q_previous| / |Q_current| < threshold`.
    ///
    /// Normalizes improvement by the current quality magnitude, making the
    /// threshold scale-invariant. Useful when quality function values span
    /// several orders of magnitude across different graphs.
    ///
    /// Falls back to absolute comparison when `|Q_current| < f64::EPSILON`
    /// to avoid division by near-zero.
    Relative,
}

impl ConvergenceMode {
    /// Returns `true` if convergence is reached given current and previous quality.
    pub fn is_converged(self, current: f64, previous: f64, threshold: f64) -> bool {
        let absolute_change = (current - previous).abs();
        match self {
            ConvergenceMode::Absolute => absolute_change < threshold,
            ConvergenceMode::Relative => {
                if current.abs() < f64::EPSILON {
                    // Fall back to absolute when denominator is near zero
                    absolute_change < threshold
                } else {
                    absolute_change / current.abs() < threshold
                }
            }
        }
    }
}
```

### Rationale

**Two variants (Absolute, Relative)** is the complete set required by the spec and the literature. A third variant (e.g., `NoIteration` for strict convergence with threshold 0) is unnecessary because threshold = 0 already achieves "run until no improvement" with `Absolute` mode.

The `is_converged` method encapsulates the math on the type itself — idiomatic Rust (placing behavior with data via `impl`). The fallback for near-zero denominators in `Relative` mode prevents division-by-zero panics (important given the `#![deny(unsafe_code)]` and no-panic policies).

### Impact on Spec

- **FR-032**: Add: "ConvergenceMode is an enum with `Absolute` (default) and `Relative` variants. `Relative` mode falls back to absolute comparison when `|Q_current| < f64::EPSILON`."
- **AlgorithmConfig trait**: `fn convergence_mode(&self) -> ConvergenceMode` — already referenced; now fully defined.
- **Key Entities** "Algorithm Configuration" entry: ConvergenceMode enum defined here.
- Add a `## Types` subsection under the spec's Key Entities to hold `ConvergenceMode`, `Gamma`, `StepCallback`, etc.

---

## Item 4: Serialization API (FR-041)

### Research Findings

**Serde** is Rust's de facto serialization framework (700M+ downloads, the most-downloaded crate) [rustify.rs 2026 guide]. The idiomatic pattern is:
- `#[derive(Serialize, Deserialize)]` on domain types.
- `serde_json::to_string(&value)` / `from_str(&string)` for JSON.
- Round-trip property: `deserialize(serialize(x)) ≈ x` — serde guarantees this for well-formed types but it is **not automatic** for types with non-derived fields (e.g., the internal CSR index mapping). Explicit round-trip tests are required.

For a library API, exposing `serde` traits directly couples consumers to serde. The common pattern (used by `petgraph`, `nalgebra`, etc.) is:
1. Derive `Serialize`/`Deserialize` on all domain types (required).
2. Provide **free functions** in a `ser` module for ergonomic one-shot serialization.
3. Provide a **typed error** wrapping serde errors via `thiserror`.

**Round-trip guarantees.** FR-041 requires *"serialized output MUST be re-parseable by the corresponding parser (FR-040) without data loss or corruption."* This is stronger than serde's structural guarantee — it requires **semantic equivalence** after deserialization+re-serialization. The internal index mapping (opaque per FR-010) means the round-trip must preserve node-identity semantics, not just raw bytes.

### Proposed API Definition

```rust
/// Serialization formats supported by the framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphFormat {
    #[default]
    Json,
    Csv,
    Gml,
}

/// Serializes a graph to the specified format.
///
/// # Errors
/// Returns `GraphError::Serialization` on failure, wrapping the underlying
/// format error with a descriptive message.
pub fn serialize_graph<G: GraphView>(
    graph: &G,
    format: GraphFormat,
) -> Result<String, GraphError> {
    match format {
        GraphFormat::Json => serde_json::to_string(&SerializableGraph::from(graph))
            .map_err(|e| GraphError::Serialization {
                format: "JSON",
                message: e.to_string(),
            }),
        GraphFormat::Csv => write_edge_list_csv(graph).map_err(|e| GraphError::Serialization {
            format: "CSV",
            message: e.to_string(),
        }),
        GraphFormat::Gml => write_gml(graph).map_err(|e| GraphError::Serialization {
            format: "GML",
            message: e.to_string(),
        }),
    }
}

/// Serializes a partition (membership vector + quality + metadata) to the specified format.
///
/// # Errors
/// Returns `PartitionError::Serialization` on failure.
pub fn serialize_partition(
    partition: &Partition,
    format: GraphFormat,
) -> Result<String, PartitionError> { /* ... */ }

/// Round-trip test helper: serialize then deserialize, asserting semantic equivalence.
///
/// Used internally to satisfy FR-041 round-trip fidelity guarantee.
/// Public for integration testing.
pub fn round_trip_graph<G>(original: &G, format: GraphFormat) -> Result<bool, CommunalError>
where G: GraphView + PartialEq,
{
    let serialized = serialize_graph(original, format)?;
    let deserialized = crate::parser::parse_graph_str(&serialized, format)?;
    Ok(&deserialized == original)
}
```

**Trait implementations (the core requirement):**

```rust
// On domain types in communal-core:
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Graph { /* ... */ }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Partition { /* ... */ }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MembershipVector(pub Vec<CommunityId>);
```

### Rationale

- **Serde derives** on all public types is the idiomatic baseline. Consumers get `serde_json`, `serde_yaml`, `bincode`, etc. compatibility for free.
- **Free functions** (`serialize_graph`) in a `ser` module provide the "without data loss" contract at the framework boundary — they ensure the opaque index mapping is materialized in a stable, parseable form.
- **No custom `Serializable` trait** — serde's trait is the standard. Introducing a parallel `Serializable` trait would be non-idiomatic and confuse consumers.
- **Round-trip tests** are required in the test suite (not just relying on serde) because the internal→external index mapping must be proven invertible.

### Impact on Spec

- **FR-041**: "Domain types (`Graph`, `Partition`, `MembershipVector`) MUST derive `serde::Serialize` and `serde::Deserialize`. The framework MUST provide free functions `serialize_graph`, `serialize_partition` in a `ser` module, returning typed errors per FR-035. Round-trip fidelity is guaranteed by property-based tests asserting `deserialize(serialize(x)) ≈ x`."
- **FR-035** error variants: Add `GraphError::Serialization { format: &str, message: String }` and `PartitionError::Serialization { ... }`.
- **External Dependencies** table: serde and serde_json already listed (Cargo.toml).
- New `## Types` section: `GraphFormat` enum.

---

## Item 5: GraphBuilder API (FR-028)

### Research Findings

The spec requires (FR-028):
- Accept adjacency list representations in-memory.
- `validate_on_construction: bool` flag.
- Eager edge weight symmetrization for undirected graphs.

Idiomatic Rust builder patterns use **consuming `self`** for the terminal `build()` call and **`&mut self`** for intermediate setters, OR a fully consuming pattern where each setter takes and returns `self`. The consuming `build(self)` pattern is preferred when:
1. The builder is a one-shot construction tool (not reused).
2. The built type is expensive to clone (a graph with CSR storage).
3. It prevents use-after-build bugs.

This is the pattern used by `std::process::Command`, `tokio::runtime::Builder`, and `petgraph`'s graph construction APIs.

### Proposed Builder Definition

```rust
/// Builder for constructing graphs from adjacency list representations.
///
/// # Example
/// ```
/// let graph = GraphBuilder::new(Directionality::Undirected)
///     .validate_on_construction(true)
///     .add_edge(0, 1, 1.0)
///     .add_edge(1, 2, 0.5)
///     .add_node(3)  // isolated node
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct GraphBuilder {
    directionality: Directionality,
    validate: bool,
    edges: Vec<(NodeId, NodeId, f64)>,
    node_count: usize,
}

impl GraphBuilder {
    /// Creates a new builder for a graph with the given directionality.
    pub fn new(directionality: Directionality) -> Self {
        Self {
            directionality,
            validate: true,  // default: validate eagerly per FR-033
            edges: Vec::new(),
            node_count: 0,
        }
    }

    /// Sets whether validation runs at construction time (default: `true`).
    ///
    /// When `true`, negative weights are rejected at build time (FR-033).
    /// When `false`, validation is deferred to algorithm execution time.
    pub fn validate_on_construction(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    /// Adds an undirected edge (or directed, per builder directionality).
    pub fn add_edge(mut self, source: NodeId, target: NodeId, weight: f64) -> Self {
        self.edges.push((source, target, weight));
        self.node_count = self.node_count.max(source.max(target) + 1);
        self
    }

    /// Adds an isolated node (no edges).
    pub fn add_node(mut self, node: NodeId) -> Self {
        self.node_count = self.node_count.max(node + 1);
        self
    }

    /// Adds multiple edges from an iterator.
    pub fn add_edges(mut self, edges: impl IntoIterator<Item = (NodeId, NodeId, f64)>) -> Self {
        for (s, t, w) in edges {
            self = self.add_edge(s, t, w);
        }
        self
    }

    /// Builds the graph, consuming the builder.
    ///
    /// Applies eager symmetrization for undirected graphs (FR-030).
    /// Runs validation if `validate_on_construction` is `true` (FR-033).
    ///
    /// # Errors
    /// - `GraphError::NegativeWeight` if validation is enabled and a negative weight is found.
    /// - `GraphError::EmptyGraph` if no nodes are present.
    pub fn build(self) -> Result<Graph, GraphError> {
        let mut graph = Graph::with_capacity(self.node_count, self.directionality);

        for (source, target, weight) in self.edges {
            if self.validate && weight < 0.0 {
                return Err(GraphError::NegativeWeight {
                    source,
                    target,
                    weight,
                });
            }
            graph.add_edge(source, target, weight);
        }

        if self.directionality == Directionality::Undirected {
            graph.symmetrize_eagerly();  // (w_ij + w_ji) / 2 per FR-030
        }

        Ok(graph)
    }
}
```

### Rationale

- **Consuming `build(self)`** — prevents accidental reuse after build; matches `std::process::Command` and `petgraph` idioms.
- **`&mut self` setters returning `self`** — the `mut self` pattern (consuming + returning) is equivalent to `&mut self` but more ergonomic for chaining. It avoids borrow-checker issues when collecting from iterators.
- **Eager symmetrization in `build()`** — FR-030 requires it at construction time; the builder is the natural place.
- **`validate_on_construction` defaults to `true`** — matches FR-033's "validate at input/construction time by default."

### Impact on Spec

- **FR-028**: "The system MUST provide a `GraphBuilder` with consuming `build(self) -> Result<Graph, GraphError>`, `add_edge`, `add_node`, `add_edges`, and `validate_on_construction` flag. Eager symmetrization applied in `build()` for undirected graphs."
- **Key Entities**: Add `GraphBuilder` entry.
- **User Story 2** alternate flows: "loaded via the graph builder" — now concretely `GraphBuilder::new(...).add_edges(...).build()`.

---

## Item 6: Error Variant Specifications (FR-035)

### Research Findings

The spec (FR-044) mandates per-module error enums using `thiserror`:
- `GraphError` — construction, validation, parsing
- `AlgorithmError` — configuration, convergence, execution
- `PartitionError` — query, invalid access
- `MetricsError` — computation, comparison

The `thiserror` crate's idiomatic pattern [docs.rs thiserror; codesnips.io] is:
- `#[error("...")]` with field interpolation: `#[error("negative weight {weight} on edge {source}->{target}")]`
- `#[from]` for automatic `?` conversion from underlying errors (e.g., `io::Error`).
- Each variant carries **structured data** (not just strings) for programmatic handling.

### Proposed Error Definitions

```rust
/// Errors from graph construction, validation, and parsing.
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("negative weight {weight} on edge {source}->{target}")]
    NegativeWeight { source: NodeId, target: NodeId, weight: f64 },

    #[error("graph is empty (zero nodes)")]
    EmptyGraph,

    #[error("node index {index} out of bounds (node count: {node_count})")]
    NodeOutOfBounds { index: usize, node_count: usize },

    #[error("duplicate edge {source}->{target}")]
    DuplicateEdge { source: NodeId, target: NodeId },

    #[error("malformed input at line {line}: {message} (expected: {expected}")]
    ParseError {
        line: usize,
        message: String,
        expected: String,
    },

    #[error("unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("serialization failed ({format}): {message}")]
    Serialization { format: &'static str, message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors from algorithm configuration, convergence, and execution.
#[derive(Debug, Error)]
pub enum AlgorithmError {
    #[error("invalid configuration for parameter '{parameter}': {message}")]
    InvalidConfiguration {
        parameter: &'static str,
        message: String,
    },

    #[error("algorithm failed to converge within {max_iterations} iterations (final quality: {final_quality})")]
    ConvergenceFailure {
        max_iterations: usize,
        final_quality: f64,
    },

    #[error("algorithm execution error: {0}")]
    ExecutionError(String),

    #[error("k ({k}) exceeds node count ({node_count})")]
    KExceedsNodeCount { k: usize, node_count: usize },
}

/// Errors from partition queries and access.
#[derive(Debug, Error)]
pub enum PartitionError {
    #[error("node {node_id} not found in partition")]
    NodeNotFound { node_id: NodeId },

    #[error("level {level} out of range (valid: 0..{max_level})")]
    InvalidLevel { level: usize, max_level: usize },

    #[error("partition is empty (no communities)")]
    EmptyPartition,

    #[error("serialization failed for partition: {message}")]
    Serialization { message: String },
}

/// Errors from metric computation and comparison.
#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("partition size mismatch: {size1} vs {size2}")]
    SizeMismatch { size1: usize, size2: usize },

    #[error("invalid metric input: {message}")]
    InvalidInput { message: String },

    #[error("computation error: {0}")]
    ComputationError(String),
}

/// Unified error type for the facade crate (optional per FR-044).
#[derive(Debug, Error)]
pub enum CommunalError {
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),

    #[error("algorithm error: {0}")]
    Algorithm(#[from] AlgorithmError),

    #[error("partition error: {0}")]
    Partition(#[from] PartitionError),

    #[error("metrics error: {0}")]
    Metrics(#[from] MetricsError),
}
```

### Rationale

- **Structured variants** (e.g., `NegativeWeight { source, target, weight }`) enable programmatic handling — callers can match on specific fields rather than parsing strings. This satisfies FR-035's "descriptive and enable programmatic handling."
- **`#[from] std::io::Error`** on `GraphError::Io` allows `?` on file operations — idiomatic `thiserror`.
- **`KExceedsNodeCount`** in `AlgorithmError` directly supports the Fluid Communities k > n case (Item 7).
- **`ConvergenceFailure`** carries `max_iterations` and `final_quality` — supports FR-027's "returns the best partition found with a convergence warning."
- **`CommunalError`** as a unified re-export with `#[from]` conversions satisfies FR-044's "MAY be provided at the facade crate level."

### Impact on Spec

- **FR-035**: "All fallible operations MUST return typed domain errors per the `GraphError`, `AlgorithmError`, `PartitionError`, `MetricsError` enums defined above."
- **FR-044**: "Per-module error types as defined above, with `CommunalError` unified re-export at facade level."
- **Key Entities** "Domain Errors" entry: replace prose with the enum definitions.
- **User Story 3** exception flows: `AlgorithmError::InvalidConfiguration` — now concretely `{ parameter: "teleportation_rate", message: "..." }`.

---

## Item 7: Fluid Communities k > n Behavior (FR-036 vs FR-029)

### Research Findings

**The contradiction:**
- **FR-029** (Fluid Communities config): "valid range [1, n] where n is node count; default behavior clamps k to n if k > n."
- **FR-036** (minimal graph handling): "Fluid Communities MUST guard against k > n by raising an error or clamping k to n."

**Reference implementation behavior.** The canonical NetworkX `asyn_fluidc` implementation (the most widely-used reference) **raises an error** when k > n:

```python
# From networkx/algorithms/community/asyn_fluid.py (lines 72-73):
if len(G) < k:
    raise nx.NetworkXError("k cannot be bigger than the number of nodes.")
```

This is a **hard error**, not a clamp. The HPAI-BSC/Fluid-Communities (the original authors' implementation) also requires k ≤ n because the algorithm initializes k fluids on k distinct random vertices — it is mathematically impossible to place k fluids on n vertices when k > n.

**Theoretical justification.** The Fluid Communities algorithm (Parés et al. 2011, arXiv:1703.09307) initializes by placing each of the k communities on a distinct random vertex. When k > n, there are not enough vertices for unique initialization. The algorithm's density model (total density = 1.0 distributed equally among members of each community) also assumes at least one member per community.

### Resolution

**Raise a typed error.** Clamping (FR-029's "default behavior clamps k to n") is misleading to users — silently producing fewer communities than requested hides a configuration mistake. The reference implementation raises an error, and the spec's FR-036 already lists "raising an error" as the first option.

**FR-029 should be amended** to remove the clamp language and align with FR-036's error-raising option.

### Proposed Definition

```rust
impl FluidCommunitiesConfig {
    /// Validates the configuration.
    ///
    /// # Errors
    /// Returns `AlgorithmError::KExceedsNodeCount` if `target_communities > node_count`.
    pub fn validate(&self, node_count: usize) -> Result<(), AlgorithmError> {
        if self.target_communities > node_count {
            return Err(AlgorithmError::KExceedsNodeCount {
                k: self.target_communities,
                node_count,
            });
        }
        Ok(())
    }
}

// In the algorithm's detect() method:
fn detect(&self, graph: &G) -> Result<Partition, AlgorithmError> {
    self.config.validate(graph.node_count())?;  // Raises KExceedsNodeCount if k > n
    // ... proceed with algorithm
}
```

### Rationale

- **Error over clamp**: Clamping silently produces k=n communities (all singletons), which is almost never the user's intent. An error forces explicit user action.
- **Matches reference**: NetworkX raises `NetworkXError("k cannot be bigger than the number of nodes.")`.
- **Mathematically necessary**: The algorithm cannot initialize k > n fluids on n vertices.
- **Typed error enables recovery**: `AlgorithmError::KExceedsNodeCount { k, node_count }` lets callers programmatically detect and handle the condition (e.g., prompt user for a smaller k).

### Impact on Spec

- **FR-029** (Fluid Communities bullet): Change from "valid range [1, n] where n is node count; default behavior clamps k to n if k > n" to **"valid range [1, n] where n is node count; raises `AlgorithmError::KExceedsNodeCount` if k > n at algorithm initialization."**
- **FR-036**: "Fluid Communities MUST guard against k > n by raising `AlgorithmError::KExceedsNodeCount`." (Remove "or clamp" option.)
- **User Story 3** alternate flow: Change "it either raises a typed error or clamps k to n per FR-036" to **"it raises `AlgorithmError::KExceedsNodeCount` per FR-036."**
- **Property-Based Testing** (Fluid Communities): "k > n is handled by raising `AlgorithmError::KExceedsNodeCount`" (remove "error or clamp").

---

## Item 8: StepEvent::IterationBoundary Phase Field

### Research Findings

The spec's Key Entities section defines `StepEvent::IterationBoundary { phase: String, iteration: usize }`. The question is whether `phase` should be a `String` or a type-safe enum.

**Rust idiom.** The Rust Reference and Book emphasize that enums are the type-safe choice for a fixed set of variants. Using `String` for a field that can only take specific values is an anti-pattern — it pushes compile-time guarantees to runtime, requires string matching (error-prone, non-exhaustive), and prevents the compiler from catching typos or missing cases.

The spec already chose **phase-encoded variant names** for all other events (e.g., `LocalMovingStart`, `RefinementSplit`, `AggregationContraction`) [Session 2026-09-04 (3)]. The `IterationBoundary` variant is the exception — it uses a `phase: String` field instead of being split into phase-specific variants.

**The algorithm phases are a closed set** (per FR-017): local moving, refinement, aggregation, convergence. These map cleanly to enum variants.

### Proposed Definition

Replace the single `IterationBoundary { phase: String, iteration: usize }` variant with **phase-specific variants** (consistent with the rest of the enum):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StepEvent {
    // Phase boundaries (replaces IterationBoundary { phase: String, ... })
    LocalMovingStart { iteration: usize },
    LocalMovingEnd { iteration: usize },
    RefinementStart { iteration: usize },
    RefinementEnd { iteration: usize },
    AggregationStart { iteration: usize },
    AggregationEnd { iteration: usize },

    // Discrete events
    NodeRelocation { node: NodeId, from: CommunityId, to: CommunityId },
    RefinementSplit { community: CommunityId, into: usize },
    AggregationContraction { from_communities: usize, to_communities: usize },
    CommunityMerge { into: CommunityId, merged: CommunityId },
    CommunitySplit { from: CommunityId, into: Vec<CommunityId> },

    // Convergence signals
    ConvergencePlateau { iterations_below_threshold: usize },
    ConvergenceDetected { total_iterations: usize, final_quality: f64 },
}
```

**If a generic iteration boundary is still desired** (e.g., for algorithms with custom phases), a type-safe enum is preferred over `String`:

```rust
/// Identifies which algorithm phase an event belongs to.
///
/// Each algorithm phase emits Start/End events. This enum is closed — new
/// phases require adding a variant, ensuring exhaustive matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseKind {
    LocalMoving,
    Refinement,
    Aggregation,
    Convergence,
}

// Alternative (if generic boundary needed):
IterationBoundary { phase: PhaseKind, iteration: usize },
```

### Rationale

- **Type safety**: `PhaseKind::LocalMoving` is checked at compile time; `"local_moving"` (a string) is not. Typos like `"local_mving"` compile fine but fail at runtime.
- **Exhaustive matching**: `match event { PhaseKind::LocalMoving => ..., PhaseKind::Refinement => ..., ... }` forces handling of all phases. String matching is non-exhaustive.
- **Consistency**: The spec already chose phase-encoded names for 9 other variants. `IterationBoundary` is the only outlier. Splitting it into `LocalMovingStart/End`, `RefinementStart/End`, `AggregationStart/End` makes the enum uniform.
- **Zero-cost**: `PhaseKind` is a single byte (enum discriminant). No allocation, no runtime cost.
- **Serde compatibility**: `#[derive(Serialize, Deserialize)]` on `PhaseKind` produces stable string representations (`"LocalMoving"`, `"Refification"`, etc.) for JSON serialization.

### Impact on Spec

- **Key Entities** "StepEvent" entry: Replace `IterationBoundary { phase: String, iteration: usize }` with `LocalMovingStart { iteration: usize }`, `LocalMovingEnd { iteration: usize }`, `RefinementStart { iteration: usize }`, `RefinementEnd { iteration: usize }`, `AggregationStart { iteration: usize }`, `AggregationEnd { iteration: usize }`.
- **FR-017**: "iteration boundaries (start/end of each local moving pass)" — now concretely `LocalMovingStart`/`LocalMovingEnd`.
- **FR-018**: `StepIterator` yields `Option<StepEvent>` — no change to signature, but the concrete variants are now type-safe.
- **User Story 5** acceptance scenario 1: "phase starts, node relocations, refinement splits, aggregation contractions" — now all phase starts are explicit variants.
- **StepEvent** must derive `Clone` (for multi-observer dispatch, Item 2) and `PartialEq` (for testing). Add `Serialize, Deserialize` if events are serialized.

---

## Summary of Changes by Spec Section

| Spec Section | Change |
|---|---|
| **FR-008** | Add `Gamma` newtype, valid range (0, ∞), default 1.0 |
| **FR-017** | Replace `IterationBoundary { phase: String }` with phase-specific variants |
| **FR-018** | Define `StepCallback::on_event(&mut self, event: &StepEvent)` |
| **FR-028** | Define `GraphBuilder` with consuming `build(self)` |
| **FR-029** | Remove clamp language; raise `KExceedsNodeCount` when k > n |
| **FR-032** | Define `ConvergenceMode` enum (Absolute, Relative) |
| **FR-035** | Define per-module error enums with structured variants |
| **FR-036** | Resolve contradiction: raise error, do not clamp |
| **FR-041** | Define serde derives + free functions + `GraphFormat` enum |
| **FR-044** | Define `CommunalError` unified re-export |
| **Key Entities** | Add `Gamma`, `ConvergenceMode`, `GraphBuilder`, `PhaseKind`, `StepCallback` signatures, error enum definitions |
| **User Story 3** | Update alternate/exception flows to reference concrete error variants |
| **Property-Based Testing** | Update Fluid Communities invariant to "raises error" |

---

## Sources

- [From Louvain to Leiden: guaranteeing well-connected communities (Nature 2019)](https://www.nature.com/articles/s41598-019-41695-z) — Traag, Waltman & van Eck
- [scanpy.tl.leiden documentation](https://scanpy.readthedocs.io/en/stable/generated/scanpy.tl.leiden.html) — resolution parameter default 1.0
- [NetworkX louvain_communities](https://networkx.org/documentation/stable/reference/algorithms/generated/networkx.algorithms.community.louvain.louvain_communities.html) — resolution default 1.0
- [NetworkX asyn_fluid source code](https://raw.githubusercontent.com/networkx/networkx/main/networkx/algorithms/community/asyn_fluid.py) — k > n raises NetworkXError
- [leidenAlg R documentation](https://www.rdocumentation.org/packages/leidenAlg/versions/1.1.5/topics/find_partition) — resolution default 1.0
- [CDlib leiden](https://cdlib.readthedocs.io/en/latest/reference/generated/cdlib.algorithms.leiden.html) — resolution_parameter double > 0
- [Idiomatic callbacks in Rust (Stack Overflow)](https://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust)
- [thiserror documentation](https://docs.rs/thiserror/latest/thiserror/)
- [Serde documentation](https://docs.rs/serde/latest/serde/)
- [The Rust Programming Language — Defining an Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
- [Fluid Communities paper (arXiv:1703.09307)](https://arxiv.org/pdf/1703.09307v2) — Parés et al. 2011
- [Constant Potts Model — NetworkX](https://networkx.org/documentation/latest/reference/algorithms/generated/networkx.algorithms.community.quality.constant_potts_model.html)

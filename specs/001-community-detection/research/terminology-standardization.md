# Terminology Standardization: Issues I2 & I3

Research compiled for: Communal project — community detection framework in Rust
Date: 2026-03-08
Scope: Cross-artifact terminology drift between spec.md, plan.md, tasks.md, and contracts/metrics.md

---

## Issue I2: Internal Dispatch Enum Naming

### Current Usage Analysis

| Artifact | Term Used | Location | Context |
|----------|-----------|----------|---------|
| **spec.md** | `InternalQualityMetric` | Line 311 (Key Entities) | "The internal dispatch enum in `communal-algo` ... is named `InternalQualityMetric` to distinguish it from the public trait." |
| **spec.md** | `QualityMetric` (trait) | Lines 19, 28, 32, 274, 276, 311 | Formal trait name per FR-021 |
| **plan.md** | `QualityFunction` | Line 91 (crate description) | "internal `QualityFunction` enum for algorithm optimization" |
| **plan.md** | `QualityFunction` (implied) | Line 120 | "Internal quality functions (used by algorithms during optimization) reside in `communal-algo/src/quality/` as an enum" |
| **tasks.md** | `QualityFunction` | Line 92 (T034) | "Implement `QualityFunction` enum (Modularity, CPM, MapEquation)" |
| **tasks.md** | `QualityFunction` | Line 96 (T036b) | "applied in the quality function formula" |
| **tasks.md** | `QualityMetric` (trait) | Line 169 (T073) | "Implement `QualityMetric` trait in `communal-metrics`" |
| **tasks.md** | `quality function` (generic) | Lines 322, 331, 362 | Informal references |
| **contracts/metrics.md** | `QualityMetric` (trait) | Lines 11, 20 | Formal trait definition |

### Naming Pattern Summary

- **`QualityMetric`** — used consistently for the *trait* across ALL artifacts (spec, plan, tasks, contracts). No drift here.
- **`InternalQualityMetric`** — used ONLY in spec.md line 311 for the internal dispatch enum.
- **`QualityFunction`** — used in plan.md (lines 91, 120) and tasks.md (line 92) for the same internal dispatch enum.

### Rust Naming Convention Research

**Observation from ecosystem:** Rust libraries distinguish internal dispatch enums from public traits by:
1. **Separate naming** — `perpetual` uses `ObjectiveFunction` (trait) vs `Metric` (enum) for public eval. The internal type has a distinct name.
2. **Visibility, not prefix** — Rust idiom favors `pub(crate)` or module-level visibility over `Internal` prefixes. The prefix is somewhat un-idiomatic in Rust where visibility modifiers already convey this intent.
3. **Enum-of-functions naming** — When an enum exists to dispatch between function implementations (zero-cost dispatch), naming it `XxxFunction` or `XxxFn` is conventional (e.g., `PredicateFn`, `CompareFn` in various crates).

### Recommendation for I2

**Canonical term: `QualityFunction`** for the internal dispatch enum.

**Rationale:**
1. **Used in 2 of 3 planning artifacts** (plan.md and tasks.md) — majority adoption.
2. **More idiomatic Rust** — "Function" describes what the enum IS (a closed set of function implementations); "Internal" is redundant with `pub(crate)` visibility.
3. **Avoids name redundancy** — `InternalQualityMetric` contains "Metric" which is already the trait name, making it easy to confuse the enum with the trait in prose.
4. **Spec already uses "quality function" generically** — FR-032 (line 291), FR-008, and line 43 all use "quality function" as a generic term for Modularity Q, CPM, Map Equation. Formalizing this as `QualityFunction` aligns with existing spec language.

**Counter-argument considered:** The spec is authoritative and explicitly names the enum `InternalQualityMetric` at line 311. However, the spec's Key Entities section is descriptive (naming entities for reference) while the plan/tasks are prescriptive (driving implementation). Standardizing on `QualityFunction` requires only a minor spec edit and brings all three artifacts into alignment.

**Exception note:** If the project prefers `InternalQualityMetric` as canonical (to preserve spec authority), the plan and tasks should be updated to match. Both options are documented below.

---

## Issue I3: QualityMetric Trait Signature — Fallible vs Infallible

### Current Usage Analysis

| Artifact | Signature | Location | Fallibility |
|----------|-----------|----------|-------------|
| **spec.md** | `evaluate(&self, graph: &G, partition: &Partition) -> f64` | Line 276 (FR-021) | **Infallible** |
| **spec.md** | `evaluate(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> f64` | Line 274 (FR-019) | **Infallible** |
| **spec.md** | `evaluate(&self, graph: &G, partition: &Partition) -> f64` | Line 311 (Key Entities) | **Infallible** |
| **spec.md** | `MetricsError` defined | Line 301 (FR-044) | Error type EXISTS |
| **contracts/metrics.md** | `fn compute(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>` | Lines 30-34 | **Fallible** |
| **contracts/metrics.md** | `Modularity::new(gamma) -> Result<Self, MetricsError>` | Line 69 | Fallible construction |
| **tasks.md** | T080: "Implement `MetricsError` type" | Line 176 | Error type EXISTS |
| **tasks.md** | T055: "zero-weight edge handling (no division-by-zero)" | Line 132 | Infallible guard |

### The Contradiction

The spec says `-> f64` (infallible) but:
- Contracts define `-> Result<f64, MetricsError>` (fallible)
- FR-044 defines `MetricsError` with "computation, comparison" variants
- Tasks T080 explicitly implements `MetricsError` in communal-metrics

### Can Quality Metrics Fail? Analysis by Scenario

| Scenario | Spec Stance | Can Fail? | Recommendation |
|----------|-------------|-----------|----------------|
| **Empty graph (0 nodes)** | FR-022: returns 0.0 | No | Infallible — defined result |
| **Graph with no edges** | FR-036: Modularity returns 0.0 when m=0 | No | Infallible — guard against div-by-zero |
| **Isolated nodes** | FR-023: each gets own community | No | Infallible — defined result |
| **Invalid partition** (node IDs out of range) | Not explicitly addressed | **Yes** | Partition validation should happen at construction, not during evaluation |
| **NaN in edge weights** | Not explicitly addressed | **Yes** | NaN propagation makes result meaningless — should be caught at graph validation |
| **Incompatible partitions** (NMI/ARI with different node counts) | Not explicitly addressed | **Yes** | Comparative metrics require same node set |
| **Infinity in computation** | Not explicitly addressed | Edge case | Unlikely with valid finite weights |

### Rust Ecosystem Convention

From the existing `metric-api-patterns.md` research (already in this repo):

| Library | Metric Signature | Fallibility |
|---------|-----------------|-------------|
| **linfa** | `fn silhouette_score(&self) -> Result<F>` | **Fallible** |
| **smartcore** | `fn get_score(&self, y_true, y_pred) -> f64` | **Infallible** |
| **perpetual** | `fn loss(&self, ...) -> Vec<f32>` | **Infallible** (internal) |

**linfa** (the most comparable Rust ML library) uses `Result<F>` for all metric computations, establishing a precedent for fallible metrics in Rust.

### Recommendation for I3

**Canonical signature: `fn evaluate(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`**

**Rationale:**

1. **`MetricsError` already exists in the type system.** FR-044 defines it; tasks T080 implements it; contracts use it. If metrics are infallible, `MetricsError` has no purpose for the trait method, creating dead code.

2. **Comparative metrics (NMI, ARI) can genuinely fail.** They take two partitions — if partitions cover different node sets or have incompatible structures, the computation is undefined. Returning `Result` is the correct way to express this.

3. **linfa precedent.** The most analogous Rust ML library uses `Result<F>` for metric evaluation.

4. **Defense in depth.** Even if "valid inputs produce valid outputs" (FR-022, FR-036), Rust's type system should enforce this. If evaluation can fail for ANY reason (future metrics, edge cases), the trait should express it.

5. **Alignment with contracts.** The contracts/metrics.md already defines the fallible signature. The spec should be updated to match.

6. **Internal metrics remain effectively infallible.** For internal use (Modularity Q, CPM, Map Equation on validated graphs), the `Result` is always `Ok(...)`. The `?` operator makes this ergonomic:
   ```rust
   let quality = metric.evaluate(graph, partition)?;
   ```

**Impact on infallible cases:** FR-022, FR-023, FR-036 all define concrete return values for edge cases. These remain unchanged — they produce `Ok(0.0)` etc. The `Result` wrapper adds zero runtime cost for the happy path (it's a discriminant, and LLVM optimizes it away in monomorphic code).

**Addressing the concern:** "But the spec says `-> f64` and metrics SHOULD be infallible for valid inputs."

Resolution: The spec's intent (metrics don't fail on valid inputs) is preserved. `Result` handles the case where inputs might not be valid. Graph validation (FR-033) catches most issues at construction time, but the trait signature should be robust to any computation failure.

---

## Standardization Summary

### Canonical Terms

| Concept | Canonical Term | Type | Visibility |
|---------|---------------|------|------------|
| Public evaluation trait | `QualityMetric` | trait | `pub` in `communal-core` |
| Evaluation method | `evaluate` | method | `pub` |
| Evaluation return | `Result<f64, MetricsError>` | return type | — |
| Internal dispatch enum | `QualityFunction` | enum | `pub(crate)` in `communal-algo` |
| Error type | `MetricsError` | error enum | `pub` in `communal-metrics` |

### Method Name Note

The spec uses `evaluate` (FR-021, lines 274, 276, 311) while contracts/metrics.md uses `compute` (line 30). Standardize on **`evaluate`** as the canonical method name because:
1. Spec is authoritative (FR-021 explicitly defines it)
2. `evaluate` is more semantically precise for "compute a quality score"
3. `compute` is too generic and could conflict with other computation methods

---

## Exact Edits Required

### Option A: Recommended — Standardize on `QualityFunction` + `Result`

#### spec.md edits

**Line 311** (Key Entities — Quality Metric):
```
- **Quality Metric**: Measures partition quality. Includes Modularity Q, CPM, Map Equation, NMI, and ARI. The `QualityMetric` trait (FR-021) is defined in `communal-core` and provides the evaluation contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> f64`. Both `communal-algo` (internal optimization) and `communal-metrics` (public evaluation) implement this trait. All optimization quality metrics (Modularity Q, CPM, Map Equation) implement this trait. The internal dispatch enum in `communal-algo` (used for zero-cost algorithm optimization) is named `InternalQualityMetric` to distinguish it from the public trait.
```
→
```
- **Quality Metric**: Measures partition quality. Includes Modularity Q, CPM, Map Equation, NMI, and ARI. The `QualityMetric` trait (FR-021) is defined in `communal-core` and provides the evaluation contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`. Both `communal-algo` (internal optimization) and `communal-metrics` (public evaluation) implement this trait. All optimization quality metrics (Modularity Q, CPM, Map Equation) implement this trait. The internal dispatch enum in `communal-algo` (used for zero-cost algorithm optimization) is named `QualityFunction` to distinguish it from the public trait.
```

**Line 274** (FR-019):
```
 Comparative metrics follow the `QualityMetric` trait pattern (FR-021): each metric is a struct implementing `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> f64`.
```
→
```
 Comparative metrics follow the `QualityMetric` trait pattern (FR-021): each metric is a struct implementing `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`.
```

**Line 276** (FR-021):
```
- **FR-021**: The system MUST define a `QualityMetric` trait in `communal-core` with the following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> f64`.
```
→
```
- **FR-021**: The system MUST define a `QualityMetric` trait in `communal-core` with the following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`.
```

#### plan.md edits

**Line 91** (crate description):
```
├── communal-algo/       # Leiden, Louvain, Infomap, LPA, Fluid implementations; internal QualityFunction enum for algorithm optimization
```
→ No change needed (already uses `QualityFunction`).

**Line 120** (Quality Metrics Separation):
```
**Quality Metrics Separation**: Internal quality functions (used by algorithms during optimization) reside in `communal-algo/src/quality/` as an enum for zero-cost dispatch.
```
→ No change needed (already uses "quality functions" informally and `QualityFunction` conceptually).

#### tasks.md edits

**Line 92** (T034):
```
- [ ] T034 [P] [US1] Implement `QualityFunction` enum (Modularity, CPM, MapEquation) in `crates/communal-algo/src/quality.rs`
```
→ No change needed (already uses `QualityFunction`).

**Line 96** (T036b):
```
The gamma field MUST be read from `AlgorithmConfig` and applied in the quality function formula per FR-008.
```
→ No change needed ("quality function" is a generic term).

#### contracts/metrics.md edits

**Lines 30-34** (trait method):
```rust
fn compute<G: GraphView>(
    &self,
    graph: &G,
    partition: &Partition,
) -> Result<f64, MetricsError>;
```
→
```rust
fn evaluate<G: GraphView>(
    &self,
    graph: &G,
    partition: &Partition,
) -> Result<f64, MetricsError>;
```

Also update the method name throughout contracts/metrics.md where `compute` is used in doc comments or examples.

---

### Option B: Alternative — Keep `InternalQualityMetric` + Infallible

If the project prefers to preserve spec authority (keeping `InternalQualityMetric`) and keep metrics infallible:

#### spec.md edits
- Line 311: No change (already `InternalQualityMetric`)
- Lines 274, 276: No change (already `-> f64`)

#### plan.md edits
- Line 91: Change `QualityFunction` → `InternalQualityMetric`
- Line 120: Change "quality functions" → "InternalQualityMetric variants"

#### tasks.md edits
- Line 92: Change `QualityFunction` → `InternalQualityMetric`
- Line 96: Change "quality function" → "quality metric"
- Line 169: Change `QualityMetric` trait → keep as-is (this is the trait, not enum)

#### contracts/metrics.md edits
- Line 30: Change `Result<f64, MetricsError>` → `f64`
- Line 69: Change `Result<Self, MetricsError>` → `Self` (or keep fallible construction)
- Remove `MetricsError` from trait method; potentially remove `MetricsError` type entirely

---

## Decision Matrix

| Criterion | Option A (QualityFunction + Result) | Option B (InternalQualityMetric + f64) |
|-----------|-------------------------------------|----------------------------------------|
| Spec edit needed | Yes (3 lines) | No |
| Plan edit needed | No | Yes (2 lines) |
| Tasks edit needed | No | Yes (2 lines) |
| Contracts edit needed | Yes (method name only) | Yes (remove Result) |
| Rust idiom | ✅ Strong (linfa precedent) | ⚠️ Acceptable |
| Error robustness | ✅ Comparative metrics can fail | ⚠️ Panics or silent corruption |
| `MetricsError` purpose | ✅ Used by trait | ❌ Dead code for trait |
| Spec authority | ⚠️ Requires spec edit | ✅ Preserves spec |
| Simplicity | ⚠️ Result adds ? in hot loop | ✅ Direct f64 return |

**Recommendation: Option A** — the ergonomic cost of `Result` in hot loops is negligible (LLVM optimizes the `Ok` branch), and the type safety benefit for comparative metrics is substantial.

---

## Sources

1. [linfa::metrics::SilhouetteScore — Docs.rs](https://docs.rs/linfa/latest/linfa/metrics/trait.SilhouetteScore.html) — Fallible metric trait precedent (`Result<F>`)
2. [smartcore::metrics::Metrics — Docs.rs](https://docs.rs/smartcore/latest/smartcore/metrics/trait.Metrics.html) — Infallible metric trait (`-> f64`)
3. [perpetual::objective::core::ObjectiveFunction — Docs.rs](https://docs.rs/perpetual/latest/perpetual/objective/core/trait.ObjectiveFunction.html) — Internal vs public separation pattern
4. [enum_dispatch — Docs.rs](https://docs.rs/enum_dispatch/latest/enum_dispatch/) — Enum dispatch pattern for zero-cost optimization
5. [Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html) — UpperCamelCase for type-level constructs
6. [Rust API Guidelines — Future Proofing (C-SEALED)](https://rust-lang.github.io/api-guidelines/future_proofing.html) — Sealed trait pattern for internal-only traits
7. [Effective Rust — Item 22: Minimize visibility](https://www.effective-rust.com/visibility.html) — Visibility over naming prefixes

# Remediation: QualityMetric Trait Placement

**Date**: 2026-09-05  
**Spec**: 001-community-detection  
**Issue**: T073 (QualityMetric trait) incorrectly placed in Phase 5 (User Story 3 - Multi-Algorithm) when it's required by T034 (QualityFunction enum) in Phase 3 (User Story 1 - Leiden)

---

## 1. Problem Statement

Task T073 was originally located in Phase 5 (User Story 3 - Multi-Algorithm Clustering Spectrum), but the `QualityMetric` trait it defines is a prerequisite for T034 (QualityFunction enum) in Phase 3 (User Story 1 - Leiden). This creates a circular phase dependency: Phase 3 cannot complete without a task from Phase 5.

### Original Placement

| Task | Phase | Description |
|------|-------|-------------|
| T073 | Phase 5 (US3) | Implement `QualityMetric` trait in `communal-core` |
| T034 | Phase 3 (US1) | Implement `QualityFunction` enum in `communal-algo` |

**Conflict**: T034 requires the `QualityMetric` trait to be defined first (the `QualityFunction` enum dispatches to metric implementations that implement `QualityMetric`).

---

## 2. Evidence from Specification

### FR-021 (QualityMetric Trait Location)

> "The system MUST define a `QualityMetric` trait in `communal-core` with the following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`. [...] Placing the trait in `communal-core` ensures both `communal-algo` (internal optimization) and `communal-metrics` (public evaluation) can implement it without circular dependencies, consistent with the constitution's dependency hierarchy."

**Key requirements from FR-021**:
1. Trait MUST be in `communal-core`
2. Both `communal-algo` and `communal-metrics` implement it
3. Avoids circular dependencies (communal-algo → communal-core ← communal-metrics)

### FR-019 (ComparativeMetric Trait)

> "The `ComparativeMetric` trait is defined in `communal-core` alongside `QualityMetric` (FR-021) to avoid circular dependencies."

This trait was entirely missing from the task list and should also be in Phase 2.

### Constitution Principle III (Modular Cargo Workspace)

> `crates/communal-core`: Primitives, typed identifiers (`NodeId`, `CommunityId`), canonical CSR structures, **fundamental traits** (`GraphView`, `CommunityDetector`, `PartitionResult`).

The `QualityMetric` and `ComparativeMetric` traits are fundamental framework traits per FR-021/FR-019, placing them squarely in `communal-core`'s domain.

### Plan.md (Quality Metrics Separation)

> "Both implement the `QualityMetric` trait defined in `communal-core` (per FR-021). This separation ensures algorithm hot loops remain uncluttered while providing a rich public API for evaluation."

---

## 3. Dependency Analysis

### Tasks Requiring QualityMetric Trait

| Task | Phase | Why It Needs QualityMetric |
|------|-------|---------------------------|
| T034 | Phase 3 (US1) | `QualityFunction` enum dispatches to metric implementations |
| T035 | Phase 3 (US1) | Modularity Q computation implements QualityMetric |
| T036 | Phase 3 (US1) | CPM computation implements QualityMetric |
| T036a | Phase 3 (US1) | Map Equation computation implements QualityMetric |
| T074 | Phase 5 (US3) | Modularity metric struct implements QualityMetric |
| T075 | Phase 5 (US3) | CPM metric struct implements QualityMetric |
| T076 | Phase 5 (US3) | Map Equation metric struct implements QualityMetric |

### Tasks Requiring ComparativeMetric Trait

| Task | Phase | Why It Needs ComparativeMetric |
|------|-------|-------------------------------|
| T077 | Phase 5 (US3) | NMI implements ComparativeMetric |
| T078 | Phase 5 (US3) | ARI implements ComparativeMetric |

### Dependency Graph

```
Phase 2 (Foundational)
  └── T028a: QualityMetric trait (communal-core) ← MOVED HERE
  └── T028b: ComparativeMetric trait (communal-core) ← NEW
        │
        ├── Phase 3 (US1 - Leiden)
        │   ├── T034: QualityFunction enum (depends on T028a)
        │   ├── T035: Modularity Q (depends on T028a)
        │   ├── T036: CPM (depends on T028a)
        │   └── T036a: Map Equation (depends on T028a)
        │
        └── Phase 5 (US3 - Multi-Algorithm)
            ├── T074: Modularity metric (depends on T028a)
            ├── T075: CPM metric (depends on T028a)
            ├── T076: Map Equation metric (depends on T028a)
            ├── T077: NMI (depends on T028b)
            └── T078: ARI (depends on T028b)
```

---

## 4. Rust Trait Design Pattern Research

### Convention: Core Traits in Core Crate

The Rust ecosystem consistently places core traits in the core crate of a workspace:

| Project | Core Crate | Core Traits | Satellite Crates |
|---------|-----------|-------------|------------------|
| `linfa` | `linfa` | `Predict`, `Fit`, `DatasetBase` | `linfa-clustering`, `linfa-linear` |
| `petgraph` | `petgraph` | `GraphBase`, `IntoNeighbors`, `Visitable` | (extensions via traits) |
| `ndarray` | `ndarray` | `Ndarray`, `Data`, `RawData` | `ndarray-linalg`, `ndarray-stats` |
| `nalgebra` | `nalgebra` | `Dim`, `Scalar`, `RealField` | `nalgebra-sparse` |
| `communal` | `communal-core` | `GraphView`, `CommunityDetector`, `AlgorithmConfig`, **`QualityMetric`**, **`ComparativeMetric`** | `communal-algo`, `communal-metrics` |

### Why This Pattern?

1. **Avoids circular dependencies**: Satellite crates depend on core, never the reverse
2. **Enables trait objects**: `Box<dyn QualityMetric>` works across crates
3. **Single source of truth**: Trait definition lives in one place
4. **Orangutans principle**: "When you're an orangutan, you can't see the forest for the trees" — core traits define the forest (the abstraction framework), satellite crates are the trees (concrete implementations)

### Communal's Dependency Hierarchy (Constitution III)

```
communal-core (no internal deps)
    ↑
communal-algo (depends on communal-core)
communal-metrics (depends on communal-core)
communal-dynamic (depends on communal-core, communal-algo)
    ↑
communal-cli / communal-tui (depend on multiple)
```

**Key insight**: Traits shared by multiple satellite crates MUST be in `communal-core`. If `QualityMetric` were in `communal-algo`, then `communal-metrics` would need to depend on `communal-algo` — breaking the dependency hierarchy.

---

## 5. Remediation Applied

### Change 1: Move T073 → T028a in Phase 2

**Before** (Phase 5):
```markdown
- [ ] T073 [P] [US3] Implement `QualityMetric` trait in `crates/communal-core/src/lib.rs` per FR-021
```

**After** (Phase 2, communal-core section):
```markdown
- [ ] T028a [P] Implement `QualityMetric` trait in `crates/communal-core/src/quality.rs` per FR-021: 
      `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`, 
      `name(&self) -> &'static str`, `range(&self) -> (f64, f64)`. 
      Trait MUST be in communal-core to avoid circular dependencies; communal-metrics and communal-algo both implement it. 
      This is a foundational trait required by T034 (QualityFunction enum) in Phase 3 and T074-T076 (metrics implementations) in Phase 5.
```

### Change 2: Add T028b (ComparativeMetric trait) in Phase 2

FR-019 explicitly requires a `ComparativeMetric` trait in `communal-core` alongside `QualityMetric`. This was missing from the task list entirely.

```markdown
- [ ] T028b [P] Implement `ComparativeMetric` trait in `crates/communal-core/src/quality.rs` per FR-019: 
      `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`. 
      This trait is for comparative metrics (NMI, ARI) that require two partition inputs. 
      Also placed in communal-core alongside QualityMetric to avoid circular dependencies. 
      Required by T077-T078 in Phase 5.
```

### Change 3: Update Dependency Notes

Added explicit cross-references in the "User Story Dependencies" section:
- US1: T034 and T035-T036b depend on T028a/T028b from Phase 2
- US3: T074-T076 depend on T028a; T077-T078 depend on T028b

### Change 4: File Path Correction

Changed `crates/communal-core/src/lib.rs` → `crates/communal-core/src/quality.rs` for the trait definition. This is more idiomatic (dedicated module for quality-related traits) and matches the pattern used by other modules in the crate (e.g., `config.rs`, `step.rs`, `partition.rs`).

---

## 6. Impact Assessment

### What Changed

| Aspect | Before | After |
|--------|--------|-------|
| QualityMetric location | Phase 5 (T073) | Phase 2 (T028a) |
| ComparativeMetric | Missing from tasks | Phase 2 (T028b) |
| Phase 3 → Phase 5 dependency | Broken (circular) | Resolved |
| Trait file path | `lib.rs` | `quality.rs` (dedicated module) |

### What Stayed the Same

- T074-T076 (metric implementations in communal-metrics) remain in Phase 5 — they implement the trait, they don't define it
- T034 (QualityFunction enum) remains in Phase 3 — it uses the trait, correctly after it's defined
- All other task IDs, ordering, and structure preserved

### Verification

After the fix:
1. Phase 2 completes → QualityMetric and ComparativeMetric traits exist in communal-core
2. Phase 3 (US1) can proceed → T034 QualityFunction enum can reference the trait
3. Phase 5 (US3) can proceed → T074-T078 metric structs can implement the traits
4. No circular dependencies: communal-core → (communal-algo, communal-metrics)

---

## 7. Sources

- **FR-021**: QualityMetric trait must be in communal-core (spec.md, line 273)
- **FR-019**: ComparativeMetric trait must be in communal-core alongside QualityMetric (spec.md, line 271)
- **Constitution III**: communal-core contains fundamental traits (constitution.md, line 52)
- **Plan.md**: Quality Metrics Separation section confirms trait in communal-core (plan.md, line 121)
- **contracts/core-traits.md**: QualityMetric trait contract definition
- **contracts/metrics.md**: Full metric API contract including QualityMetric trait
- **Rust ecosystem convention**: Core traits in core crate (linfa, petgraph, ndarray patterns)

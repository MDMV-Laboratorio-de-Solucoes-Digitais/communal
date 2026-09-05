# Remediation: Doctest Coverage for All Crates

**Date**: 2026-09-05
**Spec**: 001-community-detection
**Issue**: T024 mentions doctests for 7 crates but only enumerates communal-core doctests

---

## Problem Statement

Task T024 (tasks.md line 65) states:

> Write doctests for all public APIs in `crates/communal-core/src/lib.rs`. NOTE: Per Constitution (Exhaustive Documentation), all public APIs in ALL crates require doctests. Add doctest tasks for communal-algo, communal-metrics, communal-dynamic, communal-generators, communal-cli, communal-tui, and communal-wasm.

The NOTE acknowledges that doctests are required for all crates, but no actual tasks exist for the 7 enumerated crates. This creates a documentation gap that violates Constitution Principle IV (Exhaustive Documentation).

---

## Constitution Requirements

### Principle IV: Strict Rust Engineering & Zero Technical Debt

> **Exhaustive Documentation**: `#![deny(missing_docs)]` on all public APIs. Every public struct, trait, method, and error enum variant MUST feature clear documentation including mathematical formulas, time/space complexity, and executable doctests.

### Documentation Standards (Constitution §Engineering Standards)

Every public API MUST include:

1. **Summary**: One-line description of purpose.
2. **Mathematical Foundation**: Relevant formulas and complexity analysis.
3. **Parameters**: Description of each parameter with constraints.
4. **Returns**: Description of return value and potential errors.
5. **Examples**: Executable doctest demonstrating usage.
6. **Complexity**: Time and space complexity in Big-O notation.

### Pre-Commit Requirements

> **Testing**: `cargo test --all-features` passes all unit, integration, and doctests.

This means doctests are not optional—they are a gated requirement for all commits.

---

## Rust Doctest Conventions

### What Should Doctests Cover?

Per Rust documentation best practices and the Communal Constitution:

1. **Every public API item** must have at least one doctest demonstrating basic usage.
2. **Doctests should be executable** — they run as part of `cargo test` and must pass.
3. **Edge cases** should be demonstrated (empty inputs, single elements, error paths).
4. **Mathematical formulas** should be shown in documentation with examples verifying correctness.
5. **Complexity annotations** should be included in doc comments.

### Multi-Crate Workspace Doctest Structure

In a Cargo workspace with multiple crates:

- Each crate's `lib.rs` serves as the primary documentation entry point.
- Doctests in `lib.rs` demonstrate the crate's primary API surface.
- Module-level doctests can supplement `lib.rs` for complex modules.
- Integration tests in `tests/` complement doctests but do not replace them.

### Recommended Doctest Patterns for Communal

```rust
/// Detects communities in a graph using the Leiden algorithm.
///
/// # Algorithm
///
/// The Leiden algorithm guarantees internally connected communities through three phases:
/// 1. **Local Moving**: Optimize modularity by moving nodes between communities.
/// 2. **Refinement**: Randomized splitting to escape local optima.
/// 3. **Aggregation**: Contract communities into super-nodes and repeat.
///
/// # Parameters
///
/// * `graph` — Input graph with non-negative edge weights
/// * `config` — Algorithm configuration (convergence threshold, max iterations, seed)
///
/// # Returns
///
/// `Ok(Partition)` on success, `Err(AlgorithmError)` on invalid input or non-convergence.
///
/// # Errors
///
/// * `AlgorithmError::InvalidConfiguration` — Invalid hyperparameters
/// * `AlgorithmError::NonConvergence` — Failed to converge within max iterations
/// * `AlgorithmError::InvalidInput` — Graph contains negative weights (when validation enabled)
///
/// # Complexity
///
/// Time: O(V + E) per iteration, Space: O(V + E)
///
/// # Example
///
/// ```
/// use communal_algo::{Leiden, LeidenConfig};
/// use communal_core::Graph;
///
/// // Create a simple graph
/// let graph = Graph::from_edge_list(&[(0, 1, 1.0), (1, 2, 1.0), (2, 0, 1.0)])?;
///
/// // Configure Leiden with deterministic seed
/// let config = LeidenConfig::default().with_seed(42);
///
/// // Detect communities
/// let partition = Leiden::new(config).detect(&graph)?;
///
/// // Verify all communities are connected
/// assert!(!partition.has_disconnected_communities());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn detect(&self, graph: &G) -> Result<Partition, AlgorithmError> { ... }
```

---

## Gap Analysis

### Current Task Coverage

| Task | Crate | Doctest Coverage |
|------|-------|-----------------|
| T024 | communal-core | Explicit task exists |
| — | communal-algo | No task exists (Leiden, Louvain, Infomap, LPA, Fluid) |
| — | communal-metrics | No task exists (Modularity, CPM, Map Equation, NMI, ARI) |
| — | communal-dynamic | No task exists (StreamingDetector, HierarchicalTree) |
| — | communal-generators | No task exists (LFR, SBM, Barabasi-Albert, Erdos-Renyi) |
| — | communal-cli | No task exists (CLI commands, output formatters) |
| — | communal-tui | No task exists (TUI app, panels, pedagogy) |
| — | communal-wasm | No task exists (WASM bindings, memory cleanup) |
| — | communal-petgraph | No task exists (petgraph conversions, view adapters) |

### Crates Requiring Doctest Tasks

All 10 workspace crates (excluding communal-benches and communal-frontend which are binary/publish=false crates) require explicit doctest tasks:

1. **communal-core** — Covered by T024
2. **communal-algo** — 5 algorithms, configs, stepping, quality functions
3. **communal-metrics** — 5 metrics, report types, error types
4. **communal-dynamic** — Streaming detector, hierarchical tree, mutations
5. **communal-generators** — 4 synthetic graph generators
6. **communal-cli** — CLI commands, output formatters, batch config
7. **communal-tui** — TUI app, panels, pedagogy renderer
8. **communal-wasm** — WASM bindings, memory cleanup
9. **communal-petgraph** — petgraph conversions, zero-copy views

---

## Proposed Solution

### Approach: Add Explicit Doctest Tasks

Add new tasks T151-T159 at the end of Phase 8 (Polish) for all crates requiring doctests. This approach:

1. **Centralizes documentation tasks** — All doctest tasks are in one place, making them easy to find and track.
2. **Follows existing patterns** — T024 already establishes the pattern for communal-core doctests.
3. **Aligns with Constitution** — Explicitly addresses Principle IV's exhaustive documentation requirement.
4. **Enables parallel execution** — All doctest tasks are marked [P] since they target different files.
5. **Completes the documentation phase** — Phase 8 already contains documentation tasks (T145, T146, T147).

### Task Additions

```markdown
### Doctest Coverage (Constitution Principle IV Compliance)

- [ ] T151 [P] Write doctests for communal-algo public APIs in `crates/communal-algo/src/lib.rs`. Cover: Leiden, Louvain, Infomap, LPA, Fluid detectors; QualityFunction enum; StepIterator; StepCallback trait; AlgorithmError variants. Include mathematical formulas for quality functions and algorithmic complexity annotations.
- [ ] T152 [P] Write doctests for communal-metrics public APIs in `crates/communal-metrics/src/lib.rs`. Cover: Modularity, ConstantPottsModel, MapEquation, Nmi, Ari metric structs; QualityMetric trait impls; report types; MetricsError variants. Include mathematical formulas (with brief summaries and external references) for each metric.
- [ ] T153 [P] Write doctests for communal-dynamic public APIs in `crates/communal-dynamic/src/lib.rs`. Cover: StreamingDetector trait, HierarchicalTree (at_level, at_resolution, levels), EdgeMutation enum, IncrementalUpdate struct, DynamicOperationError variants. Include O(k) complexity annotations.
- [ ] T154 [P] Write doctests for communal-generators public APIs in `crates/communal-generators/src/lib.rs`. Cover: LfrGenerator, SbmGenerator, BarabasiAlbertGenerator, ErdosRenyiGenerator. Include parameter constraints and generation complexity.
- [ ] T155 [P] Write doctests for communal-cli public APIs in `crates/communal-cli/src/lib.rs`. Cover: CLI argument parsing, command implementations (run, compare, batch, convert, metrics, generate, validate), output formatters, batch config parser.
- [ ] T156 [P] Write doctests for communal-tui public APIs in `crates/communal-tui/src/lib.rs`. Cover: TUI app structure, graph layout, panel rendering, pedagogy templates, stepping controller.
- [ ] T157 [P] Write doctests for communal-wasm public APIs in `crates/communal-wasm/src/lib.rs`. Cover: WASM bindings for Leiden/Louvain, graph I/O, explicit memory cleanup function.
- [ ] T158 [P] Write doctests for communal-petgraph public APIs in `crates/communal-petgraph/src/lib.rs`. Cover: petgraph to CSR conversion, CSR to petgraph conversion, zero-copy view adapters.
```

### T024 Modification

Remove the NOTE from T024 since the tasks are now explicit:

**Before**:
```markdown
- [ ] T024 Write doctests for all public APIs in `crates/communal-core/src/lib.rs`. NOTE: Per Constitution (Exhaustive Documentation), all public APIs in ALL crates require doctests. Add doctest tasks for communal-algo, communal-metrics, communal-dynamic, communal-generators, communal-cli, communal-tui, and communal-wasm.
```

**After**:
```markdown
- [ ] T024 Write doctests for all public APIs in `crates/communal-core/src/lib.rs`. Cover: NodeId, CommunityId newtypes; CsrGraph construction and query methods; GraphView trait; Partition query API; AlgorithmConfig trait; AlgorithmPhase and StepEvent enums; GraphError, PartitionError error types. Include complexity annotations for all graph operations.
```

---

## Rationale

### Why Add Tasks at End of Phase 8?

1. **Phase 8 is the documentation phase** — It already contains T145 (README), T146 (CONTRIBUTING), T147 (rust docs generation).
2. **Doctests require stable APIs** — Adding them at the end ensures the public API surface is finalized.
3. **Avoids mid-phase disruptions** — Inserting tasks in earlier phases would require renumbering.
4. **Follows MVP strategy** — Documentation is polish, not core functionality.

### Why Not Expand T024?

T024 is in Phase 2 (Foundational), but the other crates are implemented in Phases 3-7. Adding all doctest tasks to Phase 2 would:
- Create dependencies on not-yet-implemented crates
- Break the phase-based organization
- Make the task list harder to follow

### Why Include communal-petgraph?

While not mentioned in T024's NOTE, communal-petgraph has public APIs (T137-T139) that require doctests per Constitution Principle IV. Adding it ensures complete coverage.

---

## Verification

After implementing all doctest tasks:

1. `cargo test --all-features` passes all doctests
2. `cargo doc --no-deps` generates documentation without errors
3. `#![deny(missing_docs)]` passes for all crates
4. All public APIs have executable examples

---

## Sources

- Constitution Principle IV: Exhaustive Documentation (constitution.md, line 79)
- Constitution Documentation Standards (constitution.md, lines 200-209)
- Rust Documentation Book: https://doc.rust-lang.org/rustdoc/write-documentation/doc-comments.html
- Cargo Book: https://doc.rust-lang.org/cargo/reference/manifest.html#the-documentation-field

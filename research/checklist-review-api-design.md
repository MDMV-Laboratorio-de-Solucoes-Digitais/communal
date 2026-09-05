# Checklist Review: API Design (001-community-detection)

**Spec**: `/home/luis/development/MDMV/projetos/communal/specs/001-community-detection/spec.md`
**Review Date**: 2026-09-05
**Reviewer**: Autonomous Agent

---

## CHK044: Petgraph Integration API

**Verdict: NO**

**Evidence**: NOT FOUND

The spec mentions petgraph only as an external dependency:
- Line 522: `petgraph | 0.8.3 (https://docs.rs/petgraph/0.8) | Graph data structure interoperability (communal-petgraph crate)`
- Line 36: "External dependencies (rayon, petgraph) are trusted as well-audited crates"
- Line 311: "external dependencies (e.g., rayon, petgraph) do NOT trigger this lint"

The spec does NOT specify:
- Conversion functions (to/from CSR)
- Zero-copy view constraints
- Supported petgraph types (e.g., `Graph`, `StableGraph`, `Csr`)
- A `GraphView` trait implementation for petgraph types
- The `communal-petgraph` crate's API surface

The mention of "communal-petgraph crate" at line 522 implies such a crate exists, but its API contract is not detailed in the spec.

---

## CHK045: Generator API Contract

**Verdict: NO**

**Evidence**: NOT FOUND

The spec references generators only indirectly:
- Line 536: `LFR Benchmark | Lancichinetti & Fortunato (2009) | Synthetic benchmark generator with known ground truth`
- Line 391: "SBM (N=10k, 4 communities, pin=0.1, pout=0.01)"
- Line 412: "LFR benchmark (N=10k, μ=0.3)"
- FR-042 (line 314): CLI has `generate` command for "synthetic benchmark graphs"

The spec does NOT specify:
- Which generators are implemented (LFR, SBM, Barabási-Albert, Erdős-Rényi)
- Generator parameters (e.g., LFR's μ, mixing parameter; SBM's pin/pout; BA's m parameter)
- Output format for generated graphs
- A generator trait or API contract
- Whether generators are in a separate crate (e.g., `communal-generate`)

The CLI `generate` command is listed but "Detailed argument contracts (flags, options, defaults) are deferred to planning" (line 314).

---

## CHK046: Facade Crate Re-export Pattern

**Verdict: NO**

**Evidence**: NOT FOUND

The spec mentions the facade crate only twice, both times regarding error re-exports:
- Line 316: "A unified `CommunalError` re-export MAY be provided at the facade crate level for users who want a single error type, with `From` conversions for each per-module error."
- Line 346: "A unified `CommunalError` re-export MAY be provided at the facade crate level (FR-044)."

The spec does NOT specify:
- Which types are re-exported at the root (e.g., `CommunityDetector`, `GraphView`, `Partition`, algorithm configs)
- How feature flags control availability of re-exports
- The facade crate's name (e.g., `communal` vs `communal-facade`)
- Module organization at the facade level
- Whether the facade is a thin re-export layer or provides additional functionality

The spec mentions internal crates (`communal-core`, `communal-algo`, `communal-metrics`, `communal-petgraph`) but does not define the facade crate's re-export pattern.

---

## CHK047: Newtype Pattern (NodeId, CommunityId)

**Verdict: NO**

**Evidence**: NOT FOUND

The spec uses `NodeId` and `CommunityId` as type names but does not specify their newtype contract:
- Line 260: `community_of(node_id) -> Option<&CommunityId>`
- Line 330: `community_of(node_id) -> Option<&CommunityId>`, `membership_vec() -> Vec<CommunityId>`, `communities() -> Vec<&[NodeId]>`
- Line 337: `EdgeMutation` variants use `NodeId`
- Line 343: `StepEvent` variants use `NodeId` and `CommunityId`

The spec does NOT specify:
- Newtype wrapper definition (e.g., `struct NodeId(u32)` or `struct NodeId(u64)`)
- Construction methods (e.g., `NodeId::new()`, `NodeId::from_raw()`)
- Validation contracts (e.g., bounds checking, invalid value handling)
- Conversion contracts (e.g., `From<u32>`, `TryFrom<usize>`)
- Whether they are type aliases (`type NodeId = u32`) or newtypes
- Internal representation (u32 vs u64, feature-gated)

Line 260 mentions "opaque dense contiguous index (u32 default, u64 feature-gated)" for internal indexing, but this is about the internal indexing strategy, not the `NodeId`/`CommunityId` newtype contracts.

---

## CHK048: CSR Graph Type `CsrGraph<N, E>`

**Verdict: NO**

**Evidence**: NOT FOUND

The spec mentions CSR only as an internal representation:
- Line 126: "the system parses it into the internal CSR representation and Leiden executes successfully"
- Line 298: "Graph builders MUST support a `validate_on_construction: bool` flag controlling whether validation (non-negative weights, CSR structure) runs at build time or is deferred"

The spec does NOT specify:
- The `CsrGraph<N, E>` type name or existence
- Type parameters (`N` for node weight, `E` for edge weight)
- Construction methods (e.g., `CsrGraph::new()`, `CsrGraph::from_edges()`)
- Access patterns (e.g., `neighbors()`, `edge_weight()`)
- Whether `CsrGraph` implements `GraphView`
- Memory layout details (compressed sparse row format)

The spec treats CSR as an implementation detail ("internal CSR representation") rather than a specified public type.

---

## CHK049: Builder Pattern Consistency

**Verdict: NO**

**Evidence**: NOT FOUND

The spec mentions builders and configs but does not specify a consistent builder pattern:
- Line 298 (FR-029): "Graph builders MUST support a `validate_on_construction: bool` flag"
- Line 335: `AlgorithmConfig` trait with required methods (`convergence_threshold`, `convergence_mode`, `max_iterations`, `seed`, `validate`)
- Line 56: "Base trait with common fields (convergence_threshold, convergence_mode, max_iterations, seed) as required methods plus a `validate() -> Result<(), Error>` method"

The spec does NOT specify:
- A consistent builder pattern across graph builders, algorithm configs, and metrics calculators
- Whether builders use the type-state pattern, consuming builders, or setters
- Method naming conventions (e.g., `with_convergence_threshold()` vs `set_convergence_threshold()`)
- Whether `AlgorithmConfig` has a builder or is constructed directly
- A `MetricsCalculator` builder (line 19 explicitly states "No separate `MetricsCalculator` type is introduced")
- Consistency in validation timing (build-time vs runtime)

The spec explicitly rejects a unified `MetricsCalculator` type (line 19), which suggests inconsistency rather than a consistent builder pattern.

---

## Summary Table

| Checklist Item | Verdict | Spec Coverage |
|----------------|---------|---------------|
| CHK044: Petgraph Integration API | NO | Only mentions petgraph as external dependency; no API contract |
| CHK045: Generator API Contract | NO | References LFR/SBM in benchmarks; no generator API specified |
| CHK046: Facade Crate Re-export Pattern | NO | Only mentions error re-export; no comprehensive re-export spec |
| CHK047: Newtype Pattern (NodeId, CommunityId) | NO | Uses types but no newtype contract specified |
| CHK048: CSR Graph Type `CsrGraph<N, E>` | NO | Mentions CSR as internal representation; no public type spec |
| CHK049: Builder Pattern Consistency | NO | No consistent builder pattern across components |

---

## Observations

The spec focuses heavily on algorithm behavior, trait definitions, and success criteria but leaves several API design details unspecified:

1. **Internal vs Public Types**: CSR is treated as internal (lines 126, 298), but the spec does not specify what public graph types exist or how they relate to CSR.

2. **Crate Architecture**: The spec mentions multiple crates (`communal-core`, `communal-algo`, `communal-metrics`, `communal-petgraph`, `communal-wasm`) but does not fully specify their public APIs or the facade crate's role.

3. **Type Contracts**: Types like `NodeId`, `CommunityId`, and `CsrGraph` are used in signatures but their construction, validation, and conversion contracts are not specified.

4. **Generator API**: The CLI has a `generate` command (FR-042) but the generator API is deferred to planning.

These gaps suggest the spec is at a requirements level rather than a full API design specification. The missing details may be appropriate for a planning phase but would need to be resolved before implementation.

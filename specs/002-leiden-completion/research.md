# Research: Leiden Algorithm Completion

**Branch**: `002-leiden-completion` | **Date**: 2026-09-05**

## Overview

This document consolidates research findings and design decisions for implementing the Leiden algorithm. All technical unknowns from the specification have been resolved through analysis of the Traag et al. 2019 paper and reference implementations (igraph, leidenalg).

## Research Findings

### 1. Modularity Q Formula

**Decision**: Implement undirected modularity per Newman-Girvan with resolution parameter.

**Formula**:
```
Q = (1/2m) · Σ_ij [A_ij - γ · k_i · k_j / 2m] · δ(c_i, c_j)
```

Where:
- `m` = total edge weight (sum of all edge weights)
- `A_ij` = edge weight between nodes i and j
- `k_i` = weighted degree of node i
- `γ` = resolution parameter (gamma)
- `δ(c_i, c_j)` = 1 if nodes i and j are in same community, 0 otherwise

**Rationale**: Matches Traag et al. 2019 formulation. The `GraphView` trait provides undirected neighbor iteration, so directed modularity is not applicable here.

**Alternatives Considered**:
- Directed modularity (Leicht & Newman 2008) — rejected: GraphView provides undirected semantics
- Signed modularity — rejected: out of scope for this feature

### 2. Constant Potts Model (CPM) Formula

**Decision**: Implement standard CPM formulation.

**Formula**:
```
Q = Σ_c [e_c - γ · (n_c choose 2)]
```

Where:
- `e_c` = total weight of intra-community edges in community c
- `n_c` = number of nodes in community c
- `γ` = resolution parameter

**Rationale**: CPM is resolution-limit-free, meaning it can detect communities at any scale. Standard formulation used by all reference Leiden implementations.

**Alternatives Considered**:
- Normalized CPM — rejected: non-standard, not used in reference implementations

### 3. Self-Loop Handling

**Decision**: Self-loops counted once as intra-community edge weight.

**Rationale**: Matches Traag et al. 2019 and igraph treatment. Self-loop weight contributes to:
- Total edge weight `m` (counted once)
- Intra-community edge weight `e_c` (counted once)
- Self-loop in aggregated graph (sum of all intra-community edges including original self-loops)

**Alternatives Considered**:
- Counting self-loops twice — rejected: inconsistent with paper
- Ignoring self-loops — rejected: loses information, inconsistent with igraph

### 4. Negative Weight Handling

**Decision**: Negative weights are mathematically valid; algorithm rejects only when total weight `m ≤ 0`.

**Rationale**: Traag et al. 2019 makes no positivity assumption. igraph supports negative weights. Single O(E) pass at algorithm entry checks `m > 0`.

**Implementation**:
```rust
fn validate_graph<G: GraphView>(graph: &G) -> Result<f64, GraphError> {
    let total_weight: f64 = /* sum all edge weights */;
    if total_weight <= 0.0 {
        return Err(GraphError::InvalidGraph {
            reason: "total edge weight must be positive".into(),
        });
    }
    Ok(total_weight)
}
```

**Alternatives Considered**:
- Reject all negative edges — rejected: mathematically unnecessary, limits use cases
- Per-edge runtime check — rejected: adds overhead to hot loops

### 5. Convergence and Plateau Detection

**Decision**: Absolute mode default (ΔQ < ε), relative mode opt-in. Plateau threshold = `max(ε/10, 1e-8)`.

**Rationale**: All reference implementations use absolute improvement as stopping condition. Plateau detection enables observability without affecting convergence semantics.

**Implementation**:
```rust
pub fn has_converged(current: f64, previous: f64, threshold: f64, mode: ConvergenceMode) -> bool {
    match mode {
        ConvergenceMode::Absolute => (current - previous).abs() < threshold,
        ConvergenceMode::Relative => {
            if current.abs() < 1e-10 { true }
            else { ((current - previous) / current).abs() < threshold }
        }
    }
}

pub fn plateau_threshold(convergence_threshold: f64) -> f64 {
    (convergence_threshold / 10.0).max(1e-8)
}
```

**Alternatives Considered**:
- Relative mode as default — rejected: non-standard, can cause premature termination
- Separate plateau counter — rejected: unnecessary complexity

### 6. Community ID Assignment

**Decision**: Contiguous IDs assigned by first-node-encountered order during local moving.

**Rationale**: Deterministic (depends only on seed), simple to implement, matches reference implementations. Node 0's community = 0, next new community encountered = 1, etc.

**Alternatives Considered**:
- Random community IDs — rejected: non-deterministic, harder to debug
- Sorted by community size — rejected: adds overhead, non-deterministic ordering

### 7. Refinement Phase Beta Parameter

**Decision**: Default β = 0.01, configurable within [0.0005, 0.1]. Implements `exp(β·Δ)` weighted selection.

**Rationale**: Matches Traag et al. 2019 paper formulation. Reference implementations simplify to uniform random, but the paper's formulation is authoritative.

**Implementation**:
```rust
fn accept_move(delta_q: f64, beta: f64, rng: &mut StdRng) -> bool {
    if delta_q > 0.0 {
        true
    } else {
        let probability = (beta * delta_q).exp();
        rng.gen::<f64>() < probability
    }
}
```

**Alternatives Considered**:
- Uniform random (simpler) — rejected: paper's formulation is authoritative
- Fixed probability — rejected: doesn't scale with gain magnitude

### 8. SteppingCallback Trait Design

**Decision**: Forward-only stepping with `Send` bound (not `Send + Sync`).

**Rationale**: Single-threaded dispatch allows mutable state in callbacks. Sufficient for TUI observation/debugging. Matches zero-cost-when-disabled principle.

**Implementation**:
```rust
pub trait SteppingCallback: Send {
    /// Called before each iteration. Returns true to continue, false to abort.
    fn before_iteration(&mut self, iteration: usize, phase: AlgorithmPhase) -> bool;
}
```

**Alternatives Considered**:
- `Send + Sync` — rejected: unnecessary constraint, prevents mutable state
- Backward navigation — rejected: requires checkpointing, adds complexity

## Dependencies Confirmed

All required dependencies are already in the workspace:

| Crate | Version | Location | Status |
|-------|---------|----------|--------|
| rand | 0.9 | communal-algo/Cargo.toml | Present |
| proptest | 1.6 | workspace/Cargo.toml | Present |
| petgraph | 0.8.3 | workspace/Cargo.toml | Present |
| thiserror | 2.0 | workspace/Cargo.toml | Present |
| tracing | 0.1.44 | workspace/Cargo.toml | Present |
| num-traits | 0.2.19 | workspace/Cargo.toml | Present |

**No new dependencies required.**

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Numerical instability in modularity computation | Medium | High | Use f64 throughout; validate finite results in tests |
| Non-termination on pathological graphs | Low | Medium | Hard max_iterations limit; return best partition |
| Performance regression vs reference | Medium | Low | Correctness first; benchmark-driven optimization later |
| Edge case panics | Low | High | Comprehensive edge case tests; zero-panic policy |

## Open Questions (Resolved During Implementation)

None — all design decisions are documented above. Implementation will follow these specifications.

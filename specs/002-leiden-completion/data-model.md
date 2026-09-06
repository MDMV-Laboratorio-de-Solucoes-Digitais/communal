# Data Model: Leiden Algorithm Completion

**Branch**: `002-leiden-completion` | **Date**: 2026-09-05**

## Overview

This document describes the data entities and their relationships for the Leiden algorithm implementation. The data model is derived from the existing infrastructure in `communal-core` and `communal-algo`, with new internal structures for algorithm state.

## Entities

### 1. LeidenConfig (existing, complete)

Configuration struct for the Leiden algorithm.

```rust
pub struct LeidenConfig {
    pub gamma: f64,                  // Resolution parameter (default: 1.0)
    pub beta: f64,                   // Refinement randomness (default: 0.01, range: [0.0005, 0.1])
    pub convergence_threshold: f64,  // Convergence epsilon (default: 1e-6)
    pub convergence_mode: ConvergenceMode,  // Absolute only (default: Absolute)
    pub max_iterations: usize,       // Hard iteration limit (default: 1000)
    pub seed: Option<u64>,           // RNG seed (default: Some(42))
}
```

**Validation Rules**:
- `gamma` > 0.0
- `beta` ∈ [0.0005, 0.1]
- `convergence_threshold` > 0.0
- `max_iterations` > 0

**Relationships**:
- Implements `AlgorithmConfig` trait
- Used by `Leiden` struct

### 2. Leiden (existing, to be completed)

Main algorithm struct implementing `CommunityDetector`.

```rust
pub struct Leiden {
    config: LeidenConfig,
    quality_function: QualityFunction,
}
```

**State**: Stateless between calls (configuration only). Internal state during `detect()`:
- `membership: Vec<u32>` — current community assignment
- `rng: StdRng` — seeded random number generator
- `iteration: usize` — current iteration count
- `previous_quality: f64` — quality from previous iteration

**Relationships**:
- Holds `LeidenConfig`
- Uses `QualityFunction` for dispatch
- Implements `CommunityDetector<G: GraphView>`

### 3. QualityFunction (existing, complete)

Enum for quality function dispatch.

```rust
pub enum QualityFunction {
    Modularity,
    Cpm,
    MapEquation,  // Not used by Leiden (Infomap-only)
}
```

**Relationships**:
- Used by `Leiden` for quality computation
- Maps to `Modularity` or `Cpm` structs

### 4. Modularity (existing, to be completed)

Modularity Q quality metric.

```rust
pub struct Modularity {
    gamma: f64,
}
```

**Computation State** (during `evaluate()`):
- `m: f64` — total edge weight
- `k: Vec<f64>` — weighted degree for each node
- `intra_community_weights: HashMap<u32, f64>` — sum of intra-community edge weights

**Relationships**:
- Implements `QualityMetric` trait
- Used via `QualityFunction::Modularity`

### 5. Cpm (existing, to be completed)

Constant Potts Model quality metric.

```rust
pub struct Cpm {
    gamma: f64,
}
```

**Computation State** (during `evaluate()`):
- `n_c: HashMap<u32, usize>` — node count per community
- `e_c: HashMap<u32, f64>` — intra-community edge weight per community

**Relationships**:
- Implements `QualityMetric` trait
- Used via `QualityFunction::Cpm`

### 6. Partition (existing, complete)

Output of community detection.

```rust
pub struct Partition {
    membership: Vec<u32>,  // community_id for each node
    quality: f64,          // quality score
}
```

**Invariants**:
- `membership.len()` == number of nodes
- Community IDs are contiguous (0..community_count)
- `quality` is finite (no NaN, no Inf)

**Relationships**:
- Returned by `CommunityDetector::detect()`
- Input to `QualityMetric::evaluate()`

### 7. GraphView (existing, complete)

Trait for graph access.

```rust
pub trait GraphView {
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn neighbors(&self, node: NodeId) -> impl Iterator<Item = NodeId>;
    fn edge_weight(&self, from: NodeId, to: NodeId) -> Option<f64>;
    fn degree(&self, node: NodeId) -> usize;
    fn has_edge(&self, from: NodeId, to: NodeId) -> bool;
    fn neighbor_count(&self, node: NodeId) -> usize;
}
```

**Relationships**:
- Implemented by `CsrGraph` and other graph types
- Used by all algorithm phases

### 8. StepEvent (existing, complete)

Events emitted during algorithm execution.

```rust
pub enum StepEvent {
    LocalMovingStart { iteration: usize },
    LocalMovingEnd { iteration: usize },
    NodeRelocation { node: NodeId, from: CommunityId, to: CommunityId },
    RefinementSplit { community: CommunityId, into: usize },
    AggregationContraction { from_communities: usize, to_communities: usize },
    IterationBoundary { phase: AlgorithmPhase, iteration: usize },
    ConvergencePlateau { iteration: u64, improvement: f64, current_quality: f64 },
    ConvergenceDetected { total_iterations: usize, final_quality: f64 },
}
```

**Relationships**:
- Emitted by algorithm phases
- Consumed by `SteppingCallback` trait

### 9. AlgorithmPhase (existing, complete)

Identifies which phase emitted an event.

```rust
pub enum AlgorithmPhase {
    LocalMoving,
    Refinement,
    Aggregation,
    Convergence,
}
```

### 10. ConvergenceMode (existing, complete)

Convergence detection mode. Only absolute mode is supported (matches all reference implementations: igraph, libleidenalg, leidenalg).

```rust
pub enum ConvergenceMode {
    Absolute,  // |Q_new - Q_old| < ε
}
```

**Note**: Relative mode was considered but rejected — no reference implementation supports it, and it provides no benefit for Modularity Q or CPM.

## New Internal Structures

### 11. LocalMoveState (new)

Internal state for the local moving phase.

```rust
struct LocalMoveState {
    neighbor_communities: HashMap<u32, f64>,  // community -> edge weight sum
    community_degrees: HashMap<u32, f64>,     // community -> total degree
    node_degree: f64,
}
```

**Lifetime**: Created fresh for each node evaluation in local moving.

### 12. RefinementState (new)

Internal state for the refinement phase.

```rust
struct RefinementState {
    community_to_nodes: HashMap<u32, Vec<NodeId>>,
    unmerged_nodes: Vec<NodeId>,
}
```

**Lifetime**: Created at start of refinement, updated during phase.

### 13. AggregationResult (new)

Output of the aggregation phase.

```rust
struct AggregationResult {
    community_to_nodes: HashMap<u32, Vec<NodeId>>,
    new_graph: CsrGraph,  // Reduced graph with communities as nodes
}
```

**Lifetime**: Created during aggregation, used to build next iteration's graph.

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Leiden::detect()                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐     ┌──────────────────────────────────────┐ │
│  │  Input Graph │────▶│  Validate: m > 0, node_count > 0    │ │
│  └──────────────┘     └──────────────┬───────────────────────┘ │
│                                      │                          │
│                                      ▼                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Initialize membership (singletons)           │  │
│  └────────────────────────────┬─────────────────────────────┘  │
│                               │                                 │
│                               ▼                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Main Iteration Loop                    │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │  Local Moving Phase                                │  │  │
│  │  │  - Random node order (seeded)                      │  │  │
│  │  │  - Compute ΔQ for each neighbor community          │  │  │
│  │  │  - Move if ΔQ > 0 and connectedness preserved     │  │  │
│  │  │  - Emit NodeRelocation events                      │  │  │
│  │  └─────────────────────┬──────────────────────────────┘  │  │
│  │                        │                                  │  │
│  │                        ▼                                  │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │  Refinement Phase                                  │  │  │
│  │  │  - Start with singleton communities                │  │  │
│  │  │  - Random order, probabilistic moves (exp(β·Δ))   │  │  │
│  │  │  - Guarantee: subpartition of input                │  │  │
│  │  │  - Emit RefinementSplit events                     │  │  │
│  │  └─────────────────────┬──────────────────────────────┘  │  │
│  │                        │                                  │  │
│  │                        ▼                                  │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │  Aggregation Phase                                 │  │  │
│  │  │  - Build community-to-nodes mapping                │  │  │
│  │  │  - Sum inter-community edge weights                │  │  │
│  │  │  - Create self-loops from intra-community edges    │  │  │
│  │  │  - Emit AggregationContraction events              │  │  │
│  │  └─────────────────────┬──────────────────────────────┘  │  │
│  │                        │                                  │  │
│  │                        ▼                                  │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │  Convergence Check                                 │  │  │
│  │  │  - Compute quality (Modularity Q or CPM)           │  │  │
│  │  │  - Check |Q_new - Q_old| < ε                     │  │  │
│  │  │  - Emit ConvergencePlateau / ConvergenceDetected   │  │  │
│  │  └─────────────────────┬──────────────────────────────┘  │  │
│  │                        │                                  │  │
│  │              Converged? ──No──▶ Loop back                 │  │
│  │                        │                                  │  │
│  │                       Yes                                 │  │
│  └────────────────────────┼──────────────────────────────────┘  │
│                           │                                     │
│                           ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Return Partition(membership, quality)        │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## State Transitions

### Membership Vector

```
Initial:  [0, 1, 2, 3, 4, 5]  (singleton communities)
              │
              ▼
Local:    [0, 0, 1, 1, 2, 2]  (nodes merged into communities)
              │
              ▼
Refine:   [0, 0, 1, 1, 2, 2]  (subpartition guarantee)
              │
              ▼
Aggregate: [0, 1, 2]          (communities become nodes)
              │
              ▼
Repeat until convergence
```

### Algorithm State

```
┌─────────────┐
│  Initialized │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│ LocalMoving │────▶│  Refinement │
└──────┬──────┘     └──────┬──────┘
       │                   │
       │                   ▼
       │            ┌─────────────┐
       │            │ Aggregation │
       │            └──────┬──────┘
       │                   │
       ▼                   ▼
┌─────────────────────────────────┐
│         Convergence Check        │
└──────────────┬──────────────────┘
               │
       ┌───────┴───────┐
       │               │
   Converged      Not Converged
       │               │
       ▼               ▼
   ┌──────┐      ┌──────────┐
   │ Done │      │ LocalMoving (loop) │
   └──────┘      └──────────┘
```

## Validation Rules

| Entity | Rule | Error |
|--------|------|-------|
| LeidenConfig | gamma > 0.0 (reject NaN, ±Inf) | AlgorithmError::InvalidConfiguration { reason } |
| LeidenConfig | beta ∈ [0.0005, 0.1] | AlgorithmError::InvalidConfiguration { reason } |
| LeidenConfig | convergence_threshold > 0.0 | AlgorithmError::InvalidConfiguration { reason } |
| Graph | node_count > 0 | GraphError::EmptyGraph |
| Graph | total_weight > 0.0 | GraphError::InvalidGraph { reason } |
| Partition | membership.len() == node_count | (internal invariant, never user-facing) |
| Partition | quality.is_finite() | (test assertion) |
| Communities | each community is connected | (test assertion) |

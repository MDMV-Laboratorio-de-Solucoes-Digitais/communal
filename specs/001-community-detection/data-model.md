# Data Model: Community Detection Framework

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

This document defines the core entities, their fields, relationships, validation rules, and state transitions for the Communal community detection framework.

---

## Entity Definitions

### 1. Graph

Represents a network topology with nodes and weighted edges.

**Fields**:
| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `node_count` | `u32` | Number of nodes | >= 0 |
| `edge_count` | `u32` | Number of edges | >= 0 |
| `offsets` | `Vec<u32>` | CSR row offsets | Length: node_count + 1 |
| `targets` | `Vec<NodeId>` | CSR column indices | Length: edge_count |
| `weights` | `Vec<EdgeWeight>` | Edge weights | Length: edge_count; must be non-negative (default validation) |
| `directed` | `bool` | Directionality flag | Default: false |

**Validation rules**:
- All edge weights must be non-negative (validated at construction by default; deferrable)
- Zero-weight edges allowed (must not cause division-by-zero)
- Self-loops allowed (factored into degree calculations)
- Isolated nodes (degree 0) assigned to own community

**Relationships**:
- Contains many `Node` (implicit via indices)
- Contains many `Edge` (implicit via CSR structure)
- Produces one `Partition` per algorithm execution

---

### 2. NodeId / CommunityId

Typed identifiers for nodes and communities.

**Definition**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommunityId(pub u32);
```

**Constraints**:
- `u32` default for cache efficiency
- `u64` support feature-gated for large graphs
- Non-contiguous external identifiers mapped to contiguous internal indices

---

### 3. Partition

Represents the output of a community detection algorithm.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `node_communities` | `Vec<CommunityId>` | Community assignment per node |
| `community_sizes` | `HashMap<CommunityId, u32>` | Size of each community |
| `quality_score` | `f64` | Overall partition quality |
| `community_count` | `u32` | Total number of communities |
| `iterations` | `u32` | Number of iterations executed |
| `convergence_delta` | `f64` | Final convergence delta |

**Validation rules**:
- Every node must have exactly one community assignment
- Communities must be internally connected (for Leiden)
- Isolated nodes have unique single-member communities
- Empty graph returns empty partition with quality 0

---

### 4. Community

Represents a cluster of nodes.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `id` | `CommunityId` | Unique identifier |
| `members` | `Vec<NodeId>` | Member node identifiers |
| `size` | `u32` | Number of members |
| `internal_edges` | `u32` | Edges within community |
| `total_weight` | `f64` | Sum of internal edge weights |

**Invariants**:
- `members.len() == size`
- For Leiden: members form connected subgraph
- Single-member communities allowed (isolated nodes)

---

### 5. AlgorithmConfig (trait)

Base trait for typed algorithm-specific configurations.

**Definition**:
```rust
pub trait AlgorithmConfig {
    fn validate(&self) -> Result<(), AlgorithmError>;
    fn convergence_threshold(&self) -> f64;
    fn convergence_mode(&self) -> ConvergenceMode;
    fn seed(&self) -> Option<u64>;
}
```

**Concrete implementations**:

#### LeidenConfig
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `gamma` | `f64` | 1.0 | Resolution parameter |
| `quality_function` | `QualityFunction` | `Modularity` | CPM or Modularity |
| `seed` | `Option<u64>` | None | Deterministic seed |
| `convergence_threshold` | `f64` | 1e-6 | Convergence threshold |
| `convergence_mode` | `ConvergenceMode` | Absolute | Absolute or relative |
| `max_iterations` | `u32` | 1000 | Maximum iterations |

#### LouvainConfig
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `gamma` | `f64` | 1.0 | Resolution parameter |
| `seed` | `Option<u64>` | None | Deterministic seed |
| `convergence_threshold` | `f64` | 1e-6 | Convergence threshold |
| `convergence_mode` | `ConvergenceMode` | Absolute | Absolute or relative |
| `max_iterations` | `u32` | 1000 | Maximum iterations |

#### InfomapConfig
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `teleportation_rate` | `f64` | 0.15 | Random walk teleportation rate |
| `seed` | `Option<u64>` | None | Deterministic seed |
| `convergence_threshold` | `f64` | 1e-6 | Convergence threshold |
| `max_iterations` | `u32` | 1000 | Maximum iterations |

#### LpaConfig
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sync_mode` | `bool` | false | Synchronous vs asynchronous |
| `seed` | `Option<u64>` | None | Deterministic seed |
| `convergence_threshold` | `f64` | 1e-6 | Convergence threshold |
| `max_iterations` | `u32` | 1000 | Maximum iterations |

#### FluidConfig
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `target_k` | `u32` | (required) | Target community count |
| `seed` | `Option<u64>` | None | Deterministic seed |
| `max_iterations` | `u32` | 1000 | Maximum iterations |

---

### 6. HierarchicalTree

Represents multi-level community structure from coarse to fine granularity.

**Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `levels` | `Vec<Partition>` | Partitions at each hierarchy level |
| `level_count` | `u32` | Number of hierarchy levels |
| `current_level` | `u32` | Current active level |

**Operations**:
- `slice_at(level: u32) -> &Partition` - Get partition at specific granularity
- `coarsen() -> Partition` - Merge communities for coarser view
- `refine() -> Partition` - Split communities for finer view

---

### 7. StepEvent

Represents a discrete algorithmic action for inspection and stepping.

**Variants**:
```rust
pub enum StepEvent {
    PhaseStart { phase: PhaseKind, iteration: u32 },
    PhaseEnd { phase: PhaseKind, iteration: u32 },
    NodeRelocation { node: NodeId, from: CommunityId, to: CommunityId, delta: f64 },
    RefinementSplit { community: CommunityId, into: Vec<CommunityId> },
    AggregationContract { communities: Vec<CommunityId>, into: CommunityId },
    ConvergencePlateau { iteration: u32, delta: f64 },
}
```

**PhaseKind variants**:
- `LocalMoving`
- `Refinement`
- `Aggregation`
- `Diffusion` (LPA)
- `FluidExpansion` (Fluid)

---

### 8. Quality Metrics

**Modularity Q**:
- Formula: Q = (1/2m) * Σ_ij [A_ij - γ * (k_i * k_j / 2m)] * δ(c_i, c_j)
- Range: [-1, 1]; higher is better

**Constant Potts Model (CPM)**:
- Formula: H = -Σ_ij [A_ij - γ * (k_i * k_j / 2m)] * δ(c_i, c_j)
- Resolution parameter γ directly controls community size

**Map Equation**:
- Describes flow compression via random walks
- Lower values indicate better compression (better partition)

**Normalized Mutual Information (NMI)**:
- Range: [0, 1]; 1 = identical partitions
- For ground-truth comparison

**Adjusted Rand Index (ARI)**:
- Range: [-1, 1]; 1 = identical, 0 = random
- For ground-tright comparison

---

## State Transitions

### Algorithm Execution State Machine

```
┌─────────────┐
│   Initial   │
└──────┬──────┘
       │
       ▼
┌─────────────┐     convergence     ┌────────────┐
│  Iterating  │ ─────────────────► │  Converged │
└──────┬──────┘                     └────────────┘
       │ max_iterations
       ▼
┌─────────────┐
│    MaxIter  │
└─────────────┘
```

### Partition Evolution (per iteration)

```
┌──────────────┐
│ LocalMoving  │ - Node moves to maximize quality
└──────┬───────┘
       │
       ▼ (Leiden only)
┌──────────────┐
│  Refinement  │ - Split communities, reassign
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Aggregation  │ - Build reduced graph
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Converge?  │ - Check delta < threshold
└──────────────┘
```

### Dynamic Graph State (HIT)

```
┌──────────────┐
│ StableState  │ - Partition stable
└──────┬───────┘
       │ edge insertion/deletion
       ▼
┌──────────────┐
│  LocalUpdate │ - Recalculate O(k) neighborhood
└──────┬───────┘
       │
       ├──► disconnect? ──► ImmediateSplit
       │
       ▼
┌──────────────┐
│ StableState  │ - Updated partition
└──────────────┘
```

---

## Relationships Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Graph                                      │
│  (CSR layout, node_count, edge_count, directed)                     │
└───────────────────────────┬─────────────────────────────────────────┘
                            │ 1
                            │ executes
                            ▼ *
┌─────────────────────────────────────────────────────────────────────┐
│                    AlgorithmConfig (trait)                           │
│  ┌─────────────┬─────────────┬─────────────┬──────────┬───────────┐│
│  │ LeidenConfig│LouvainConfig│InfomapConfig│ LpaConfig│FluidConfig││
│  └─────────────┴─────────────┴─────────────┴──────────┴───────────┘│
└───────────────────────────┬─────────────────────────────────────────┘
                            │ 1
                            │ produces
                            ▼ 1
┌─────────────────────────────────────────────────────────────────────┐
│                         Partition                                    │
│  (node_communities, quality_score, community_count)                  │
└────────────┬────────────────────────────────────┬───────────────────┘
             │ 1                                  │ 1
             │ contains                           │ produces
             ▼ *                                  ▼ 1
┌────────────────────────┐          ┌─────────────────────────────────┐
│      Community          │          │        HierarchicalTree          │
│  (id, members, size)   │          │  (levels, current_level)         │
└────────────────────────┘          └─────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                       StepEvent                                      │
│  (NodeRelocation, RefinementSplit, AggregationContract, ...)        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Validation Summary

| Entity | Validation | Error Type |
|--------|------------|------------|
| Graph construction | Non-negative weights (default) | `GraphError::NegativeWeight` |
| Graph construction | Valid CSR structure | `GraphError::InvalidTopology` |
| AlgorithmConfig | Parameter ranges | `AlgorithmConfig::InvalidParameter` |
| Partition | Connected communities (Leiden) | `PartitionError::DisconnectedCommunity` |
| Partition | All nodes assigned | `PartitionError::UnassignedNode` |
| HierarchicalTree | Valid level transitions | `TreeError::InvalidLevel` |

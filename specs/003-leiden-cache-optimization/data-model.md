# Data Model: Leiden Cache Optimization

**Feature**: 003-leiden-cache-optimization
**Date**: 2026-09-08

---

## Overview

This document defines the entity structures introduced by the cache optimization feature. All entities live within `crates/communal-algo/src/leiden/` and are internal to the algorithm implementation (not part of the public API).

---

## Entity: `LocalMoveState`

**Location**: `crates/communal-algo/src/leiden/local_moving.rs` (new struct)

**Purpose**: Persistent cached statistics across iterations of the local-moving phase. Currently, `local_moving()` recomputes `node_degrees`, `community_degree_sums`, and `community_sizes` from scratch every call. This struct caches them.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `node_degrees` | `Vec<f64>` | Weighted degree for each node (indexed by `NodeId::index()`, 1-based). Fixed for the graph — computed once, never invalidated. |
| `community_degree_sums` | `Vec<f64>` | Sum of node degrees for each community (indexed by community ID). Updated incrementally via subtract-add repair. |
| `community_sizes` | `Vec<usize>` | Number of nodes in each community (indexed by community ID). Updated incrementally via subtract-add repair. |
| `total_weight_m` | `f64` | Total edge weight of the graph (fixed). Computed once. |
| `community_internal_weights` | `Vec<f64>` | Sum of intra-community edge weights for each community. Updated incrementally. |
| `community_dirty` | `Vec<bool>` | Dirty flag per community — `true` when cached statistics may be stale. |
| `neighbor_caches` | `Vec<NeighborCache>` | Per-node neighbor community weight cache (indexed by node index, 0-based). |

### Validation Rules
- `node_degrees.len() == node_count + 1` (1-based indexing, index 0 unused)
- `community_degree_sums.len() == max_community_id + 1`
- `community_sizes.len() == max_community_id + 1`
- `neighbor_caches.len() == node_count`

### State Transitions
```
[Graph Input] → compute_all() → [Fully Populated]
[Node Moves X→Y] → mark_dirty(X), mark_dirty(Y) → [Partially Dirty]
[Phase Boundary] → recompute_dirty() → [Fully Populated]
[After N updates] → compute_all() → [Full Reset]  (FR-011)
```

---

## Entity: `NeighborCache`

**Location**: `crates/communal-algo/src/leiden/local_moving.rs` (new struct)

**Purpose**: Per-node lazy cache mapping community IDs to summed edge weights. Avoids recomputing `collect_neighbor_communities()` (which builds a fresh `HashMap` per node per iteration in the current code).

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `weights` | `FxHashMap<u32, f64>` | Community ID → summed edge weight from this node to that community. |
| `dirty` | `bool` | `true` when the cache may be stale (node's neighborhood changed). |

### Validation Rules
- `weights` is empty when `dirty == true` (lazy evaluation — recompute on access)
- All weights must be non-negative (edge weights are non-negative per `GraphError::NegativeWeight`)

### State Transitions
```
[Node processed] → build() → [Clean, Populated]
[Neighbor moves] → mark_dirty() → [Dirty, Empty]
[Next access] → rebuild_if_dirty() → [Clean, Populated]
```

### Frontier Propagation
When node A moves from community X to Y:
1. For each neighbor B of A: mark `B.neighbor_cache.dirty = true`
2. This is the "frontier" — the set of nodes whose neighbor caches are affected by A's move.

---

## Entity: `ConvergenceState`

**Location**: `crates/communal-algo/src/leiden/convergence.rs` (extends existing struct)

**Purpose**: Tracks convergence signals across iterations. Extends the existing `ConvergenceState` with rolling quality window and node movement tracking.

### Fields (existing + new)

| Field | Type | Description |
|-------|------|-------------|
| `previous_quality` | `f64` | Quality from the previous iteration (existing). |
| `iterations_below_threshold` | `usize` | Consecutive iterations below convergence threshold (existing). |
| `total_iterations` | `usize` | Total iterations performed (existing). |
| `quality_window` | `VecDeque<f64>` | **NEW**: Rolling window of last K=5 quality values for plateau detection. |
| `nodes_moved` | `usize` | **NEW**: Number of nodes moved in the current iteration (for FR-003 zero-movement detection). |
| `consecutive_zero_movement` | `usize` | **NEW**: Consecutive iterations with zero node movement. |
| `best_membership` | `Vec<u32>` | **NEW**: Best partition membership found so far (for FR-012). |
| `best_quality` | `f64` | **NEW**: Quality of the best partition found so far. |
| `converged` | `bool` | **NEW**: Whether the algorithm has converged. |

### Validation Rules
- `quality_window.len() ≤ 5` (K=5 window size)
- `best_quality` is monotonically non-decreasing (we only update when quality improves)
- `converged == true` implies `iterations_below_threshold ≥ 1` OR `consecutive_zero_movement ≥ 1`

### State Transitions
```
[Init] → previous_quality = NEG_INFINITY, window empty
[After pass] → update(quality, nodes_moved) → check convergence
[Quality ↓ or =] → window tracks, plateau detection active
[Zero moved] → consecutive_zero_movement++
[Moved > 0] → consecutive_zero_movement = 0
[Converged] → set converged = true, break loop
```

---

## Entity: `CacheStatistics`

**Location**: `crates/communal-algo/src/leiden/local_moving.rs` (new struct, debug-only)

**Purpose**: Tracks cache hit/miss rates for performance monitoring and debugging. Only populated in debug builds.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `hits` | `u64` | Number of cache hits (dirty=false on access). |
| `misses` | `u64` | Number of cache misses (dirty=true, required rebuild). |
| `invalidations` | `u64` | Number of cache invalidations (dirty marking). |
| `full_recomputes` | `u64` | Number of full recomputation triggers (FR-011). |
| `incremental_updates` | `u64` | Number of incremental subtract-add updates. |

### Validation Rules
- `hit_rate = hits / (hits + misses)` should be > 0.8 for effective caching
- `full_recomputes` should be approximately `total_iterations * nodes_per_iteration / N`

---

## Entity: `LeidenConfig` (Modified)

**Location**: `crates/communal-algo/src/leiden/config.rs`

**Purpose**: User-configurable parameters for the Leiden algorithm. Extended with new fields per spec.

### Field Changes

| Field | Type | Default | Change | Spec Ref |
|-------|------|---------|--------|----------|
| `gamma` | `f64` | `1.0` | Unchanged | FR-006 |
| `beta` | `f64` | `0.01` | Unchanged | FR-003 |
| `convergence_threshold` | `f64` | `1e-6` | Unchanged | FR-003 |
| `convergence_mode` | `ConvergenceMode` | `Absolute` | Unchanged | — |
| `max_iterations` | `usize` | `10` | Unchanged | FR-005 |
| `seed` | `Option<u64>` | `Some(42)` | **Changed from `None` to `Some(42)`** | FR-009 |
| `recompute_interval` | `u32` | `100` | **NEW field** | FR-011 |

### Validation Rule Changes (per spec Clarifications)

| Field | Current Validation | Spec-Required Validation |
|-------|-------------------|-------------------------|
| `gamma` | `> 0`, finite | `≥ 0` (allow 0 for all-in-one-community) |
| `beta` | `[0.0005, 0.1]` | `[0, 1]` (broader range per spec; `beta = 0` = greedy deterministic) |
| `convergence_threshold` | `≥ 0` | `≥ 0` (unchanged) |
| `max_iterations` | (not validated) | `≥ 1` |
| `recompute_interval` | (not present) | `≥ 1` (reject 0 — would allow unbounded FP drift) |
| `seed` | (any) | `any u64` (unchanged) |

---

## Relationships

```
Leiden
  └── config: LeidenConfig
  └── quality_function: QualityFunction
  └── (runtime) state: LocalMoveState
        ├── node_degrees: Vec<f64> (fixed)
        ├── community_degree_sums: Vec<f64> (incremental)
        ├── community_sizes: Vec<usize> (incremental)
        ├── community_internal_weights: Vec<f64> (incremental)
        ├── community_dirty: Vec<bool>
        └── neighbor_caches: Vec<NeighborCache>
              ├── weights: FxHashMap<u32, f64>
              └── dirty: bool

ConvergenceState
  ├── quality_window: VecDeque<f64> (K=5)
  ├── nodes_moved: usize
  ├── consecutive_zero_movement: usize
  ├── best_membership: Vec<u32>
  └── best_quality: f64

CacheStatistics (debug only)
  ├── hits, misses, invalidations
  ├── full_recomputes
  └── incremental_updates
```

---

## Memory Layout

Per spec SC-008, total memory must be O(V + E):

| Structure | Size | Bound |
|-----------|------|-------|
| `node_degrees` | O(V) | (V+1) × 8 bytes |
| `community_degree_sums` | O(V) | ≤ V communities × 8 bytes |
| `community_sizes` | O(V) | ≤ V communities × 8 bytes |
| `community_internal_weights` | O(V) | ≤ V communities × 8 bytes |
| `community_dirty` | O(V) | ≤ V communities × 1 byte |
| `neighbor_caches` | O(V + E) | Each node caches its neighbor communities; total entries across all nodes = O(E) |
| `ConvergenceState` | O(1) | Fixed-size window (K=5) + scalars |
| `best_membership` | O(V) | V × 4 bytes |
| **Total** | **O(V + E)** | Dominated by CSR graph + neighbor caches |

The `FxHashMap` per node is small (typically < 20 entries for real-world sparse graphs), so the total neighbor cache memory is bounded by O(E) — each edge contributes to at most 2 neighbor cache entries (one per endpoint).

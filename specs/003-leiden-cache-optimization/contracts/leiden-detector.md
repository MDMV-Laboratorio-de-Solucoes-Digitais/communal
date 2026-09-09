# Contract: Leiden CommunityDetector Implementation

**Feature**: 003-leiden-cache-optimization
**Location**: `crates/communal-algo/src/leiden/mod.rs`

---

## Trait Implementation

```rust
impl<G: GraphView> CommunityDetector<G> for Leiden {
    fn detect(&self, graph: &G) -> Result<Partition, GraphError>;
}
```

## detect() Algorithm Contract

### Preconditions
1. `self.config.validate()` MUST be called first — returns `GraphError::InvalidGraph` on failure
2. Graph with 0 nodes → returns `Partition::new(Vec::new(), 0.0, true)` immediately
3. Graph with 0 total edge weight → returns singleton partition with Q=0.0

### Main Loop Structure

```
FOR iteration IN 0..max_iterations:
    1. LOCAL MOVING PHASE
       - Call local_moving() with cached LocalMoveState
       - Returns nodes_moved count
    
    2. REFINEMENT PHASE
       - Call refinement() with cached LocalMoveState
       - Uses beta parameter for randomized merging
    
    3. AGGREGATION PHASE
       - Call aggregation() to build reduced graph
       - reduced_graph becomes input for next iteration's local_moving
    
    4. QUALITY EVALUATION (after complete pass)
       - Compute current_quality via compute_quality()
       - Update ConvergenceState with (current_quality, nodes_moved)
    
    5. CONVERGENCE CHECK (FR-003)
       - Check quality threshold: |Q_new - Q_old| < convergence_threshold
       - Check zero movement: nodes_moved == 0
       - Rolling window plateau: max(Q_window) - min(Q_window) < threshold
       - OR logic: terminate if ANY condition satisfied
       - If converged: set converged=true, break
    
    6. BEST PARTITION TRACKING (FR-012)
       - If current_quality > best_quality: update best_membership, best_quality
    
    7. CACHE MAINTENANCE (FR-011)
       - Incremental update counter
       - If counter >= recompute_interval: full recompute of cached statistics

POST-LOOP:
    - If !converged: emit warning log (FR-012)
    - Return Partition::new(best_membership, best_quality, converged)
    - Partition includes convergence status flag
```

### Postconditions
- Returned `Partition.quality` is the highest quality found across all iterations
- Returned `Partition.membership` corresponds to the best quality partition
- All communities in the partition are internally connected (FR-010, verified via debug_assert!)
- If converged: algorithm terminated early (iteration < max_iterations - 1)
- If not converged: algorithm ran for max_iterations iterations

### Error Conditions
- `GraphError::InvalidGraph` — configuration validation failure
- `GraphError::InvalidGraph` — quality computation failure (wrapped from MetricsError)

---

## PRNG Contract

### Requirement (FR-009)
- MUST use `ChaCha8Rng` (NOT `StdRng`)
- MUST be seeded from `config.seed` (default 42)
- MUST produce bitwise-identical results across:
  - Different compiler versions
  - Different operating systems
  - Different CPU architectures

### Implementation
```rust
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed.unwrap_or(42));
```

---

## Cache Consistency Contract

### Invariant (FR-001, debug builds only)
```rust
debug_assert!(
    cached_community_weight_sums match actual_graph_state,
    "Cache inconsistency detected: community {} cached={} actual={}",
    community_id, cached, actual
);
```

### Fallback (FR-002)
If cache consistency cannot be detected (e.g., accumulated FP error exceeds threshold):
- Fall back to full recomputation of affected community statistics
- This is a safety net — should not trigger in normal operation with recompute_interval=100

---

## Convergence Detection Contract (FR-003)

### Quality Threshold
```
|Q_new - Q_old| < convergence_threshold  for  consecutive iterations
```

### Node Movement Threshold
```
nodes_moved == 0  for  consecutive iterations
```

### Plateau Detection (rolling K=5 window)
```
max(Q_window[K=5]) - min(Q_window[K=5]) < convergence_threshold
```

### Combined Logic
```
terminate IF (quality_converged) OR (zero_movement) OR (plateau_detected)
evaluate ONLY after complete pass (local_moving → refinement → aggregation)
```

---

## StepEvent Emission Contract

The detect() method SHOULD emit StepEvent variants at appropriate points:

| Point | Event |
|-------|-------|
| Start of local-moving phase | `StepEvent::LocalMovingStart { iteration }` |
| Node relocation | `StepEvent::NodeRelocation { node, from, to }` |
| End of local-moving phase | `StepEvent::LocalMovingEnd { iteration }` |
| Refinement split | `StepEvent::RefinementSplit { community, into }` |
| Aggregation contraction | `StepEvent::AggregationContraction { from_communities, to_communities }` |
| Iteration boundary | `StepEvent::IterationBoundary { phase, iteration }` |
| Convergence plateau | `StepEvent::ConvergencePlateau { iterations_below_threshold }` |
| Convergence detected | `StepEvent::ConvergenceDetected { total_iterations, final_quality }` |

**Note**: StepEvent emission is observability-only and MUST NOT affect algorithm correctness or performance in release builds. The `SteppingCallback` trait (existing infrastructure) provides zero-cost when no listeners are registered.

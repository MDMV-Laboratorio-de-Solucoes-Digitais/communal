# Task Remediation Proposals for Issues G2, G3, I1

**Date**: 2026-09-05  
**Spec**: 001-community-detection  
**Issues Addressed**: G2 (MEDIUM), G3 (LOW), I1 (MEDIUM)

---

## Issue G2: FIFO Dispatch Ordering Not Explicitly Tested

### Analysis

**Functional Requirement**: FR-046 states:
> "Multiple observers MAY be registered; events are dispatched to all registered observers in registration order (FIFO guarantee)."

**Current Task Coverage**:

| Task | Description | FIFO Coverage |
|------|-------------|---------------|
| T100 [P] [US5] | Write callback observer tests in `crates/communal-algo/tests/callback_tests.rs` | Tests callback invocation but does not assert ordering semantics across multiple observers |
| T104 [P] [US5] | Implement `StepCallback` trait | Defines the trait, no ordering test |
| T104a [US5] | Implement `subscribe()` method with FIFO dispatch | Implementation mentions FIFO per FR-045, but no dedicated test verifies ordering |

**Gap**: T100 tests that callbacks receive events, but does not verify that when multiple observers are registered, they are invoked in strict registration order (FIFO). The implementation (T104a) states FIFO as a requirement, but there is no corresponding test task that explicitly verifies this guarantee.

### Observer Pattern Testing Research

To verify FIFO dispatch semantics in Rust, the canonical approach is:

1. **Registration-order tracking**: Each observer appends its identity to a shared `Vec` (or an ordered collection) upon receiving an event.
2. **Assertion**: After emitting a sequence of events, verify that the invocation log matches the expected registration-order sequence for each event.
3. **Edge cases to test**:
   - Multiple observers (3+) receiving the same event
   - Observers registering at different times (after some events already emitted)
   - Observer deregistration (via `Subscription` drop) does not disrupt FIFO for remaining observers
   - Events with side effects in earlier observers don't affect later observers

**Recommended test pattern** (pseudocode):

```rust
#[test]
fn fifo_dispatch_order_preserved() {
    let mut invocation_log: Vec<(usize, usize)> = Vec::new(); // (observer_id, event_seq)
    
    let mut detector = /* ... */;
    
    // Register observers A (id=0), B (id=1), C (id=2)
    let sub_a = detector.subscribe(RecordingObserver::new(0, &mut invocation_log));
    let sub_b = detector.subscribe(RecordingObserver::new(1, &mut invocation_log));
    let sub_c = detector.subscribe(RecordingObserver::new(2, &mut invocation_log));
    
    // Emit single event
    detector.step_one_event();
    
    // Assert FIFO: A invoked first, then B, then C
    assert_eq!(invocation_log[0].0, 0); // observer A
    assert_eq!(invocation_log[1].0, 1); // observer B  
    assert_eq!(invocation_log[2].0, 2); // observer C
    
    // Drop middle observer, verify remaining order preserved
    drop(sub_b);
    invocation_log.clear();
    detector.step_one_event();
    
    assert_eq!(invocation_log[0].0, 0); // observer A still first
    assert_eq!(invocation_log[1].0, 2); // observer C now second
    assert_eq!(invocation_log.len(), 2);
}
```

### Proposed Task Modification

**Option A (Recommended): Extend T100 with explicit FIFO test scope**

**Before**:
```
- [ ] T100 [P] [US5] Write callback observer tests in `crates/communal-algo/tests/callback_tests.rs`
```

**After**:
```
- [ ] T100 [P] [US5] Write callback observer tests in `crates/communal-algo/tests/callback_tests.rs`. MUST include: (1) single observer receives all emitted events, (2) deregistration via Subscription Drop stops further callbacks, (3) FIFO ordering — when N observers are registered, assert each event dispatches in strict registration order (observer_0 before observer_1 before ... before observer_N-1), (4) dropping a middle observer preserves FIFO order for remaining observers. These tests validate FR-046's FIFO guarantee explicitly.
```

**Option B (Alternative): Add new test task T100a**

**Before**: (no task exists)

**After**:
```
- [ ] T100a [P] [US5] Write FIFO dispatch ordering tests in `crates/communal-algo/tests/fifo_dispatch_tests.rs`. Register 3+ observers that record invocation timestamps/sequences to a shared Vec. Emit multiple StepEvent variants. Assert observers are invoked in strict registration order for each event. Test dropping middle observers preserves FIFO for remaining observers. Validate deregistered observers receive no further events. These tests explicitly verify FR-046's "FIFO dispatch to all observers" guarantee.
```

### Recommendation

**Option A** is preferred because:
- T100 is the natural home for observer behavior tests; adding FIFO scope keeps related tests together
- Avoids proliferation of single-purpose test files
- FIFO is a core callback contract property, not a separate feature
- Minimal file footprint (one test file: `callback_tests.rs`)

---

## Issue G3: Log Rotation Configuration Not Explicitly Specified

### Analysis

**Functional Requirement**: FR-037 states:
> "File output MUST support configurable path and rotation policies."

**Current Task Coverage**:

| Task | Description | Rotation Coverage |
|------|-------------|-------------------|
| T026a | Implement stdout and file log output destinations | Mentions "configurable path and rotation policies" but does not specify rotation semantics, configuration parameters, or test requirements |
| T028 | Zero-trust logging verification tests | Tests content sanitization, not rotation behavior |

**Gap**: While T026a mentions "rotation policies," it is vague about:
1. What rotation policies are supported (size-based, time-based, both)
2. How rotation is configured (max file size, max age, max file count)
3. What happens when rotation triggers (new file created, old files archived/deleted)
4. Whether rotation preserves log continuity

### Log Rotation Best Practices Research

**Industry-standard rotation strategies** (from `tracing-subscriber`'s `RollingFileAppender` and `log4rs`):

| Strategy | Trigger | Use Case |
|----------|---------|----------|
| **Size-based** | File exceeds N bytes (e.g., 10MB) | Predictable disk usage, high-throughput systems |
| **Time-based** | Time period elapsed (daily, hourly) | Audit trails, log retention compliance |
| **Compound** (size + time) | Whichever triggers first | Production systems needing both guarantees |

**Rust ecosystem reference**: `tracing-subscriber` (already in workspace dependencies) provides `tracing_subscriber::fmt::writer::RollingFileAppender` with variants:
- `RollingFileAppender::new(rotation, directory, filename_prefix)` where `rotation` is:
  - `Rotation::DAILY` — rotate every day at midnight
  - `Rotation::HOURLY` — rotate every hour
  - `Rotation::MINUTELY` — rotate every minute (testing)
  - `Rotation::NEVER` — no rotation (unbounded file)

**Recommendation for Communal**: Support size-based and time-based rotation via a `RotationPolicy` enum:
- `RotationPolicy::Never` — single unbounded file
- `RotationPolicy::Size(max_bytes)` — rotate when file exceeds threshold
- `RotationPolicy::Daily` — rotate at midnight UTC
- `RotationPolicy::Hourly` — rotate at the top of each hour

Configuration should be part of a `LogConfig` struct in `crates/communal-core/src/logging.rs`.

### Proposed Task Modification

**Before**:
```
- [ ] T026a Implement stdout and file log output destinations with `tracing-subscriber` integration in `crates/communal-core/src/logging.rs`. Support stdout layer (suitable for piping/redirection without decorative formatting), file layer (configurable path and rotation policies), and sanitization preventing graph data/node identifiers/topology from appearing in logs unless explicitly enabled per FR-030 and FR-036.
```

**After**:
```
- [ ] T026a Implement stdout and file log output destinations with `tracing-subscriber` integration in `crates/communal-core/src/logging.rs`. Support stdout layer (suitable for piping/redirection without decorative formatting), file layer with configurable path and rotation policy, and sanitization preventing graph data/node identifiers/topology from appearing in logs unless explicitly enabled per FR-030 and FR-36. Rotation MUST support: `RotationPolicy::Never` (unbounded), `RotationPolicy::Size(max_bytes)` (rotate when file exceeds threshold), `RotationPolicy::Daily` (rotate at midnight UTC), `RotationPolicy::Hourly` (rotate at top of hour). When rotation triggers, a new log file is created with a timestamp suffix (e.g., `communal.2026-09-05T14-30-00.log`); the previous file is closed and retained (no automatic deletion in v1). Configuration exposed via `LogConfig { path: PathBuf, rotation: RotationPolicy, sanitize: bool }`. Per FR-037, file output MUST support configurable path and rotation policies.
```

**Additional recommended test task** (new):

**Before**: (no task exists)

**After**:
```
- [ ] T026c [P] Write log rotation behavior tests in `crates/communal-core/tests/log_rotation_tests.rs`. Create a temporary directory. Configure file logging with `RotationPolicy::Size(1KB)` and emit log output exceeding the threshold. Assert multiple rotated files exist with timestamp suffixes. Configure with `RotationPolicy::Never` and assert single unbounded file. Verify all rotated files contain valid log content (no partial lines, no dropped entries per write). Validate sanitization: no node IDs, edge weights, or topology appear in rotated files when `sanitize=true`.
```

### Recommendation

Extend T026a as shown above and add T026c for rotation testing. Size-based rotation is most practical for library users (predictable disk usage), while time-based suits long-running services. The `RotationPolicy` enum keeps the API ergonomic and matches `tracing-subscriber` conventions already in the workspace.

---

## Issue I1: T027a Cross-Phase Story Tagging

### Analysis

**Current State**:
```
- [ ] T027a [P] [US2] Write order-independence tests verifying asymmetric edge weights produce identical modularity results when edge direction input order is reversed in `crates/communal-core/tests/symmetrization_tests.rs` (validates SC-014)
```

**Location**: Phase 2 (Foundational — line 70 of tasks.md)
**Tag**: `[US2]` (User Story 2 — Universal Graph Input & Partition Extraction)

**The Problem**:
1. T027a is physically placed in Phase 2 (Foundational), which blocks ALL user stories
2. But it is tagged `[US2]`, implying it only belongs to User Story 2
3. This creates confusion: Is T027a a prerequisite for US1 as well (since it's in Foundational)? Or only for US2 (since it's tagged)?
4. The test validates SC-014 (Asymmetric Weight Symmetry), which is a graph construction/symmetrization property — not specific to any user story

**Content Analysis of T027a**:

The test verifies:
- Given a graph with asymmetric edge weights (w_ij ≠ w_ji)
- When the edge direction input order is reversed
- Then modularity results are identical (within floating-point tolerance)

This validates the **symmetrization logic** (FR-030: arithmetic mean at construction time), which is:
- A foundational graph property
- Required by ALL algorithms (Leiden, Louvain, Infomap, LPA, Fluid)
- Not specific to US2's domain (parsing, I/O, node mapping)

**Cross-Reference with Related Tasks**:

| Task | Phase | Tag | Content |
|------|-------|-----|---------|
| T025 | Phase 2 | (none) | Implement edge weight symmetrization |
| T027 | Phase 2 | (none) | Symmetrization correctness tests |
| T027a | Phase 2 | US2 | Order-independence tests for asymmetric weights |

T025 and T027 (the parent implementation and base test) have NO story tag — they are pure foundational. T027a extends T027 with additional order-independence coverage, so it should logically share the same (untagged) foundational classification.

**SC-014 Placement in Spec**:

SC-014 ("Asymmetric Weight Symmetry") is listed under Success Criteria with no story-specific scoping. It measures a framework-wide correctness property.

### Determination

**T027a should NOT be tagged with any user story.** It belongs in Phase 2 (Foundational) as a pure infrastructure validation test, alongside T027.

**Reasoning**:
1. Symmetrization order-independence is a graph construction invariant required by all algorithms
2. SC-014 is a framework-wide success criterion, not specific to US2
3. Placing it in Phase 2 without a story tag communicates correctly: "this blocks everything, belongs to nothing specific"
4. The `[US2]` tag incorrectly suggests that order-independence is only relevant to graph I/O parsing (US2's domain), when in fact it's about graph construction math

### Proposed Task Modification

**Before**:
```
- [ ] T027a [P] [US2] Write order-independence tests verifying asymmetric edge weights produce identical modularity results when edge direction input order is reversed in `crates/communal-core/tests/symmetrization_tests.rs` (validates SC-014)
```

**After**:
```
- [ ] T027a [P] Write order-independence tests verifying asymmetric edge weights produce identical modularity results when edge direction input order is reversed in `crates/communal-core/tests/symmetrization_tests.rs` (validates SC-014). NOTE: This is a foundational correctness test for graph construction symmetrization (FR-030), required by ALL algorithms. It is intentionally NOT story-tagged because SC-014 is a framework-wide invariant, not specific to any single user story.
```

### Recommendation

Remove the `[US2]` tag from T027a. Keep it in Phase 2 (Foundational) where it correctly blocks all stories until symmetrization correctness is verified. Add the clarifying note to prevent future re-tagging.

**Alternative considered and rejected**: Moving T027a to Phase 4 (US2 section) would be incorrect because:
- It tests symmetrization math, not graph I/O parsing
- Phase 4 depends on Phase 2, but this test is a Phase 2 prerequisite
- Other stories (US1, US3, US4, US5) also depend on correct symmetrization

---

## Summary of Proposed Changes

| Issue | Task | Change Type | Priority |
|-------|------|-------------|----------|
| G2 | T100 | Extend description to explicitly require FIFO ordering tests | MEDIUM |
| G3 | T026a | Extend description with specific `RotationPolicy` enum variants and semantics | LOW |
| G3 | T026c (new) | Add rotation behavior test task | LOW |
| I1 | T027a | Remove `[US2]` tag; add note clarifying foundational scope | MEDIUM |

---

## Appendix: Full Modified Task Listings

### Phase 2 (Modified Tasks Only)

```markdown
- [ ] T026a Implement stdout and file log output destinations with `tracing-subscriber` integration in `crates/communal-core/src/logging.rs`. Support stdout layer (suitable for piping/redirection without decorative formatting), file layer with configurable path and rotation policy, and sanitization preventing graph data/node identifiers/topology from appearing in logs unless explicitly enabled per FR-030 and FR-36. Rotation MUST support: `RotationPolicy::Never` (unbounded), `RotationPolicy::Size(max_bytes)` (rotate when file exceeds threshold), `RotationPolicy::Daily` (rotate at midnight UTC), `RotationPolicy::Hourly` (rotate at top of hour). When rotation triggers, a new log file is created with a timestamp suffix (e.g., `communal.2026-09-05T14-30-00.log`); the previous file is closed and retained (no automatic deletion in v1). Configuration exposed via `LogConfig { path: PathBuf, rotation: RotationPolicy, sanitize: bool }`. Per FR-037, file output MUST support configurable path and rotation policies.

- [ ] T027a [P] Write order-independence tests verifying asymmetric edge weights produce identical modularity results when edge direction input order is reversed in `crates/communal-core/tests/symmetrization_tests.rs` (validates SC-014). NOTE: This is a foundational correctness test for graph construction symmetrization (FR-030), required by ALL algorithms. It is intentionally NOT story-tagged because SC-014 is a framework-wide invariant, not specific to any single user story.

- [ ] T026c [P] Write log rotation behavior tests in `crates/communal-core/tests/log_rotation_tests.rs`. Create a temporary directory. Configure file logging with `RotationPolicy::Size(1KB)` and emit log output exceeding the threshold. Assert multiple rotated files exist with timestamp suffixes. Configure with `RotationPolicy::Never` and assert single unbounded file. Verify all rotated files contain valid log content (no partial lines, no dropped entries per write). Validate sanitization: no node IDs, edge weights, or topology appear in rotated files when `sanitize=true`.
```

### Phase 7 (Modified Tasks Only)

```markdown
- [ ] T100 [P] [US5] Write callback observer tests in `crates/communal-algo/tests/callback_tests.rs`. MUST include: (1) single observer receives all emitted events, (2) deregistration via Subscription Drop stops further callbacks, (3) FIFO ordering — when N observers are registered, assert each event dispatches in strict registration order (observer_0 before observer_1 before ... before observer_N-1), (4) dropping a middle observer preserves FIFO order for remaining observers. These tests validate FR-046's FIFO guarantee explicitly.
```

---

## Sources

- FR-037: File output path and rotation requirements (spec.md, line 295)
- FR-046: FIFO dispatch to all observers (spec.md, line 303)
- SC-014: Asymmetric Weight Symmetry verification (spec.md, lines 445-452)
- `tracing-subscriber` `RollingFileAppender`: Size/time-based rotation patterns
- `log4rs` rotation strategies: Industry-standard size + time compound policies
- Observer pattern testing: Registration-order verification via shared invocation log

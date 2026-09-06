# Tasks: Leiden Algorithm Completion

**Input**: Design documents from `/specs/002-leiden-completion/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Property-based tests included per spec requirements (SC-001, SC-002, SC-004)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Source**: `crates/communal-algo/src/`
- **Tests**: `crates/communal-algo/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Module structure and file organization

- [ ] T001 Create leiden submodule files in `crates/communal-algo/src/leiden/` (local_moving.rs, refinement.rs, aggregation.rs, convergence.rs)
- [ ] T002 Update `crates/communal-algo/src/leiden/mod.rs` to declare new submodules and export types
- [ ] T003 Update `crates/communal-algo/src/lib.rs` to export leiden submodule if not already present

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 [P] Implement `Modularity::evaluate()` in `crates/communal-algo/src/quality.rs` per research.md §1 (undirected Newman-Girvan with resolution parameter)
- [ ] T005 [P] Implement `Cpm::evaluate()` in `crates/communal-algo/src/quality.rs` per research.md §2 (standard CPM formulation)
- [ ] T006 [P] Implement `Modularity::delta_q()` in `crates/communal-algo/src/quality.rs` for local moving phase gain computation
- [ ] T007 [P] Implement `Cpm::delta_q()` in `crates/communal-algo/src/quality.rs` for local moving phase gain computation
- [ ] T008 Implement convergence detection in `crates/communal-algo/src/leiden/convergence.rs` (absolute mode only, plateau threshold `max(ε/10, 1e-8)` per research.md §5)
- [ ] T009 Implement `LeidenConfig::validate()` in `crates/communal-algo/src/leiden/config.rs` (gamma > 0 with NaN/±Inf rejection, beta ∈ [0.0005, 0.1], threshold > 0; returns `AlgorithmError::InvalidConfiguration { reason }`)
- [ ] T010 Implement graph validation in `crates/communal-algo/src/leiden/mod.rs` (check total weight m > 0 → `GraphError::InvalidGraph { reason }`, empty graph → `GraphError::EmptyGraph`)
- [ ] T011 Implement `SteppingCallback` trait in `crates/communal-algo/src/leiden/mod.rs` (forward-only, Send bound per research.md §8)

**Checkpoint**: Foundation ready — quality functions, convergence, config validation, and callback trait available

---

## Phase 3: User Story 1 - Trustworthy Community Detection (Priority: P1) 🎯 MVP

**Goal**: Implement Leiden's three-phase algorithm (local moving, refinement, aggregation) producing internally connected communities verifiable via BFS/DFS.

**Independent Test**: Run Leiden on any valid graph and verify every output community is internally connected (BFS from any member reaches all members).

### Implementation for User Story 1

- [ ] T012 [P] [US1] Implement local moving phase in `crates/communal-algo/src/leiden/local_moving.rs` (seeded RNG for random node order, ΔQ computation, positive gain moves, connectedness preservation)
- [ ] T013 [P] [US1] Implement refinement phase in `crates/communal-algo/src/leiden/refinement.rs` (singleton start, exp(β·Δ) weighted selection with beta parameter, subpartition guarantee, γ-connectivity)
- [ ] T014 [P] [US1] Implement aggregation phase in `crates/communal-algo/src/leiden/aggregation.rs` (community-to-nodes mapping, inter-community edge sums, self-loop creation)
- [ ] T015 [US1] Implement `Leiden::detect()` main loop in `crates/communal-algo/src/leiden/mod.rs` (initialize singletons, iterate phases, check convergence, return Partition)
- [ ] T016 [US1] Add tracing event emission in `crates/communal-algo/src/leiden/mod.rs` and phase files. Event schemas: LocalMovingStart/End { iteration }, NodeRelocation { node, from, to }, RefinementSplit { community, into }, AggregationContraction { from_communities, to_communities }, ConvergencePlateau { iteration, improvement, current_quality }, ConvergenceDetected { total_iterations, final_quality }

**Checkpoint**: Leiden produces connected communities — core differentiator delivered

---

## Phase 4: User Story 2 - Reproducible Research Results (Priority: P2)

**Goal**: Guarantee deterministic, bit-for-bit identical results with same seed across runs.

**Independent Test**: Run Leiden 100 times with same seed on identical input and verify identical membership vectors.

### Implementation for User Story 2

- [ ] T017 [US2] Verify seeded RNG integration in local moving and refinement phases produces deterministic output (cross-check T012/T013 implementation)
- [ ] T018 [US2] Ensure community ID assignment is deterministic (contiguous by first-node-encountered order) in `crates/communal-algo/src/leiden/local_moving.rs`

### Tests for User Story 2

- [ ] T019 [P] [US2] Add property-based determinism test in `crates/communal-algo/tests/leiden_determinism.rs` (same seed → identical membership, 100 runs)

**Checkpoint**: Determinism verified — same seed always produces identical results

---

## Phase 5: User Story 3 - Edge Case Robustness (Priority: P3)

**Goal**: Handle edge cases gracefully without panics, returning valid partitions with finite quality scores.

**Independent Test**: Run Leiden on edge case graphs (empty, single node, self-loops, zero weights, disconnected components) and verify valid partitions.

### Implementation for User Story 3

- [ ] T020 [US3] Implement empty graph handling in `crates/communal-algo/src/leiden/mod.rs` (return empty partition, no panic)
- [ ] T021 [US3] Implement self-loop handling in quality functions in `crates/communal-algo/src/quality.rs` (counted once per research.md §3)
- [ ] T022 [US3] Verify (no implementation needed): disconnected components produce naturally isolated communities — algorithm inherently separates via absent edges; add integration test confirming communities never span disconnected components
- [ ] T023 [US3] Implement negative weight validation in `crates/communal-algo/src/leiden/mod.rs` (reject only when total weight m ≤ 0, per FR-010)
- [ ] T024 [US3] Implement max_iterations handling in `crates/communal-algo/src/leiden/convergence.rs` (return best partition with `AlgorithmError::NonConvergence { iterations }` and tracing warning — never panic)

### Tests for User Story 3

- [ ] T025 [P] [US3] Add edge case tests in `crates/communal-algo/tests/leiden_edge_cases.rs` (empty graph, single node, self-loops, zero weights, disconnected components, negative weights valid/invalid; verify all returned quality scores are finite — no NaN/Inf — per SC-006)

**Checkpoint**: All edge cases handled without panics, valid partitions returned

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Testing, validation, and quality assurance

- [ ] T026 [P] Add property-based connected communities test in `crates/communal-algo/tests/leiden_connected.rs` (verify all communities internally connected via BFS)
- [ ] T027 [P] Add quality monotonicity test in `crates/communal-algo/tests/leiden_connected.rs` (quality never decreases between iterations)
- [ ] T028 [P] Add LFR benchmark validation test in `crates/communal-algo/tests/synthetic_ground_truth.rs` (NMI ≥ 0.95, marked #[ignore])
- [ ] T029 Execute quickstart.md validation scenarios; resolve any failures found
- [ ] T030 Run `cargo clippy --all-targets --all-features`; fix all warnings
- [ ] T031 Run `cargo fmt --check`; fix any formatting issues
- [ ] T032 Run `cargo doc --no-deps`; fix any documentation errors
- [ ] T033 Verify `#![deny(unsafe_code, missing_docs, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::allow_attributes_without_reason)]` passes
- [ ] T034 [P] Document all public APIs in `crates/communal-algo/src/leiden/` and `crates/communal-algo/src/quality.rs` with formulas, complexity analysis, and executable doctests per constitution Principle IV

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase — core algorithm implementation
- **User Story 2 (Phase 4)**: Depends on Foundational phase — integrates with US1 but independently testable
- **User Story 3 (Phase 5)**: Depends on Foundational phase — extends US1 with edge cases
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — no dependencies on other stories
- **US2 (P2)**: Can start after Foundational — extends US1 with determinism verification
- **US3 (P3)**: Can start after Foundational — extends US1 with edge case handling

### Within Each User Story

- Quality functions before algorithm phases
- Algorithm phases before main loop
- Core implementation before tracing events
- Implementation before tests

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational completes, US1, US2, US3 can proceed in parallel
- All tests marked [P] can run in parallel within their story

---

## Parallel Example: Foundational Phase

```bash
# Launch all quality function implementations together:
Task T004: "Implement Modularity::evaluate() in crates/communal-algo/src/quality.rs"
Task T005: "Implement Cpm::evaluate() in crates/communal-algo/src/quality.rs"
Task T006: "Implement Modularity::delta_q() in crates/communal-algo/src/quality.rs"
Task T007: "Implement Cpm::delta_q() in crates/communal-algo/src/quality.rs"
```

## Parallel Example: User Story 1

```bash
# Launch all algorithm phases together:
Task T012: "Implement local moving phase in crates/communal-algo/src/leiden/local_moving.rs"
Task T013: "Implement refinement phase in crates/communal-algo/src/leiden/refinement.rs"
Task T014: "Implement aggregation phase in crates/communal-algo/src/leiden/aggregation.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test connected communities invariant independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Polish phase → Final validation

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (core algorithm)
   - Developer B: User Story 2 (determinism tests)
   - Developer C: User Story 3 (edge case handling)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- All code must comply with `#![deny(unsafe_code, missing_docs, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::allow_attributes_without_reason)]`
- Use `tracing` crate for all event emission (no stdout/print in library code)
- Domain-rich error types via `thiserror` (GraphError, AlgorithmError)
- Commit after each task or logical group

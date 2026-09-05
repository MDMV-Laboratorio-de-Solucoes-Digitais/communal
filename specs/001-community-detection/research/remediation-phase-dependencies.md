# Remediation: Cross-Phase Dependency Issue in tasks.md

**Date**: 2026-03-10
**Issue**: Task T043a (Phase 3, US1) depended on T132-T135 (Phase 8, Polish), violating phase ordering.

## Problem Analysis

### Issue 1: Cross-Phase Dependency (Circular)

**Original state**:
- T043a was in Phase 3 (User Story 1 - Leiden algorithm)
- T043a depended on T132-T135 (synthetic benchmark generators in Phase 8 - Polish)
- Phase 8 (Polish) explicitly depends on all user stories being complete
- This created a circular dependency: US1 → Polish → US1

**Why this is wrong**:
1. Phase ordering principle states: "Polish (Phase 8): Depends on all desired user stories being complete"
2. Test infrastructure should be available before tests that use it
3. A Phase 3 task cannot depend on Phase 8 tasks without breaking the phase ordering

### Issue 2: Wrong Success Criteria Reference

**Original state**:
- T043a referenced SC-003 but used NMI >= 0.95 threshold
- SC-003 (Benchmark Graph Partitioning): Uses real-world benchmarks (Karate Club, Dolphins, Cora, Enron) with NMI >= 0.80
- SC-004 (Synthetic Benchmark Validation): Uses synthetic benchmarks (LFR, SBM) with NMI >= 0.95

**Why this is wrong**:
- T043a tests synthetic benchmarks (LFR, SBM) with NMI >= 0.95
- This aligns with SC-004, not SC-003
- SC-003 is for real-world benchmarks with noisier ground truth (NMI >= 0.80)

## Solution Applied

### Fix 1: Move Generators to Phase 2 (Foundational)

**Rationale**:
- Synthetic benchmark generators (LFR, SBM, Barabasi-Albert, Erdos-Renyi) are test infrastructure
- They produce graphs with known ground truth for validation tests
- Phase 2 is explicitly "Core infrastructure that MUST complete before ANY user story can be implemented"
- Moving generators to Phase 2 ensures they're available before any user story tests

**Changes made**:
1. Added new section "### communal-generators Crate (Test Infrastructure)" to Phase 2
2. Moved T131-T135 from Phase 8 to Phase 2
3. Added clear purpose statement explaining why these are foundational

### Fix 2: Corrected SC Reference in T043a

**Change**: Updated T043a description from referencing SC-003 to SC-004

**Before**:
```
T043a [P] [US1] Write synthetic benchmark validation tests verifying NMI >= 0.95...
```

**After**:
```
T043a [P] [US1] Write synthetic benchmark validation tests verifying NMI >= 0.95... (validates SC-004)
```

## Verification

After the fix:
- ✅ T043a (Phase 3) depends on T132-T135 (Phase 2) - correct ordering
- ✅ No circular dependencies between phases
- ✅ Test infrastructure available before tests that use it
- ✅ SC-004 correctly referenced for synthetic benchmark validation
- ✅ Phase 8 (Polish) no longer contains test infrastructure tasks

## Phase Structure After Fix

1. **Phase 1: Setup** - Workspace initialization
2. **Phase 2: Foundational** - Core infrastructure + test generators (T131-T135)
3. **Phase 3: User Story 1** - Leiden algorithm (can use generators)
4. **Phase 4: User Story 2** - Graph I/O
5. **Phase 5: User Story 3** - Multi-algorithm
6. **Phase 6: User Story 4** - Dynamic updates
7. **Phase 7: User Story 5** - Observability
8. **Phase 8: Polish** - CLI, WASM, petgraph, benchmarks (no longer has generators)

## References

- spec.md: SC-003 (line 369-386) vs SC-004 (line 388-395)
- tasks.md: Phase dependencies section (line 316-327)
- Constitution Principle VI: Test infrastructure requirements

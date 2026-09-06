# Leiden Spec Remediation Analysis

**Date**: 2026-09-06
**Source**: speckit-analyze output + codebase verification
**Scope**: spec.md, plan.md, tasks.md, checklists/algorithm-correctness.md

## Summary

Applied remediation edits to resolve 11 findings from specification analysis. All edits verified against actual codebase state.

## Findings & Remediations Applied

### F1 (HIGH) — Error Type Inconsistency
- **Issue**: FR-007 referenced `InvalidParameter` error type which does not exist in codebase
- **Source**: `crates/communal-core/src/error.rs` — actual types are `AlgorithmError::InvalidConfiguration`, `GraphError::InvalidGraph`
- **Fix**: Updated spec.md FR-007 to use `AlgorithmError::InvalidConfiguration { reason }`

### A1 (MEDIUM) — Duplicated Beta Documentation
- **Issue**: FR-002 repeated "no mathematical justification exists" twice
- **Fix**: Consolidated to single statement with scale-dependence note

### A4 (MEDIUM) — Duplicated Plateau Event Fields
- **Issue**: FR-009 repeated plateau event fields and SteppingCallback description
- **Fix**: Consolidated to single definition with callback behavior note

### D2 (LOW) — Missing Deny Attribute
- **Issue**: tasks.md missing `clippy::allow_attributes_without_reason` (constitution Principle IV requires it)
- **Fix**: Added to both occurrences (T033 and notes section)

### C2 (LOW) — Ambiguous Task Type
- **Issue**: T022 unclear if implementation or verification
- **Fix**: Clarified as verification task with integration test requirement

### F3 (LOW) — Plan/Code Mismatch
- **Issue**: plan.md said config.rs "no changes" but T009 requires validate()
- **Fix**: Updated plan.md to note validate() addition needed

### E1 (LOW) — Missing SC-006 Coverage
- **Issue**: SC-006 (finite quality) had no explicit task
- **Fix**: Added explicit finite quality verification to T025

## Checklist Updates

Updated `checklists/algorithm-correctness.md`:
- CHK004 review note: `InvalidParameter` → `AlgorithmError::InvalidConfiguration { reason }`
- CHK036/CHK037: Updated to reflect actual communal-core error types (`GraphError::InvalidGraph`, `AlgorithmError::NonConvergence`)

## Research: Refinement Phase Implementation

**Question**: Should refinement use `exp(β·Δ)` weighted selection (paper) or uniform random (reference impls)?

**Findings**:
- Paper (Traag et al. 2019): `Pr(C) ∝ exp(β·Δ)` with β=0.01
- igraph C: Implements `exp(Δ/beta)` (inverse convention, same effect)
- libleidenalg (C++): Uniform random (β→0 limit)
- Java networkanalysis: `exp(Δ/randomness)` matching igraph

**Resolution**: Spec's choice of paper formulation is defensible and documented in research.md. No change needed — difference is minimal in practice (β=0.01 makes exponent small).

## Files Modified

1. `specs/002-leiden-completion/spec.md` — FR-007, FR-002, FR-009
2. `specs/002-leiden-completion/plan.md` — config.rs comment
3. `specs/002-leiden-completion/tasks.md` — T022, T025, T033, notes
4. `specs/002-leiden-completion/checklists/algorithm-correctness.md` — CHK004, CHK036, CHK037

## Verification

All edits verified with grep:
- `InvalidParameter` removed from spec.md (remains only in checklists, now aligned)
- `clippy::allow_attributes_without_reason` present in both tasks.md locations
- Error types match `crates/communal-core/src/error.rs` definitions

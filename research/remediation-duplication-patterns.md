# Remediation Patterns for Duplicate Requirements in Specifications

**Date**: 2026-09-04
**Spec**: `specs/001-community-detection/spec.md`
**Framework**: ISO/IEC/IEEE 29148:2018 quality characteristics

---

## 1. Theoretical Foundation: Requirements Quality per ISO/IEC/IEEE 29148

ISO/IEC/IEEE 29148:2018 defines quality characteristics for requirements specifications at two levels:

### Individual Requirement Characteristics
- **Atomic**: Expresses a single need, capability, constraint, or behavior
- **Unambiguous**: Has only one possible interpretation
- **Complete**: Contains all information necessary for understanding, implementation, and verification
- **Verifiable**: Can be objectively determined to have been satisfied
- **Necessary**: Addresses a legitimate stakeholder need

### Specification-Level Characteristics
- **Non-Redundant**: Duplicate or overlapping requirements are avoided. Redundancy increases maintenance effort and leads to inconsistencies when changes occur.
- **Consistent**: Requirements within the specification do not contradict one another.
- **Well-Structured and Organized**: Uses consistent terminology, unique identifiers, and facilitates navigation, understanding, and maintenance.
- **Modifiable**: Structure supports efficient updates and change management without compromising consistency.

### Why Duplication Is Harmful

Per ISO/IEC/IEEE 29148, the **Non-Redundant** characteristic is not merely stylistic — it directly threatens specification quality:

1. **Inconsistency Risk**: When one copy is updated and the other is not, the spec becomes self-contradictory.
2. **Maintenance Burden**: Every future change must be applied in multiple places.
3. **Ambiguity**: Readers cannot determine which requirement is authoritative.
4. **Verification Conflict**: Two requirements may have overlapping but subtly different success criteria.
5. **Traceability Breakdown**: Cross-references become circular or point to the wrong authoritative source.

### Consolidation Principles

Based on ISO/IEC/IEEE 29148 and industry best practices:

1. **Single Source of Truth (SSOT)**: Each distinct requirement has one authoritative location.
2. **Cross-Reference, Don't Duplicate**: When two FRs touch the same domain, one references the other rather than restating content.
3. **Atomic Decomposition**: Split overlapping FRs by concern (policy vs. mechanism, general vs. specific, trait definition vs. usage semantics).
4. **Preserve Intent**: Never delete information during consolidation — relocate it to the appropriate FR.

---

## 2. Duplication Analysis and Remediation

### 2.1 FR-022 vs FR-036 — Empty Graph / Minimal Graph Handling

**Current State:**

| Aspect | FR-022 | FR-036 |
|--------|--------|--------|
| Scope | Empty graph only (0 nodes) | All minimal graph cases (5 cases) |
| Content | Single sentence: empty partition with quality 0 | Detailed table with input/output/membership vectors for each case |
| Quality guards | None | Modularity division-by-zero guard, CPM/Map Equation well-definedness, Fluid k>n handling |
| Cross-refs | References FR-023, FR-036 | Self-contained |
| Related | Brief mention of FR-023 isolated nodes | References FR-023 for all-isolated case |

**Problem**: FR-022 is entirely subsumed by FR-036 (the "Empty graph" row covers FR-022's content). FR-022 also creates a false separation: readers must check both FR-022 and FR-036 to understand minimal graph behavior.

**Recommended Strategy**: **Delete FR-022, keep FR-036 as authoritative.**

**Exact Text Reorganization:**

1. Delete FR-022 entirely.
2. In FR-036, add a clarifying introductory sentence before the table:
   > "The system MUST handle minimal graph edge cases with explicit expected outputs. The empty graph case (0 nodes) returns an empty partition with quality 0.0."
3. Ensure FR-036's "Empty graph" row remains as the first row of the table (already present).

**Cross-Reference Updates Needed:**

| Location | Current Reference | Updated Reference |
|----------|-------------------|-------------------|
| FR-022 text | "per FR-023 and the minimal graph behavior defined in FR-036" | N/A (FR-022 deleted) |
| User Story 1, Exception Flow 3 | "per FR-022" | "per FR-036" |
| Spec body line 132 | "Then the system returns an empty partition with quality 0.0 per FR-022" | "Then the system returns an empty partition with quality 0.0 per FR-036" |
| FR-036 text | "Empty graph (0 nodes) MUST return an empty partition per FR-022" | Remove self-reference (row already makes this clear) |

**Impact on Success Criteria:**
- SC-008 (Graceful Edge Case Handling): No change needed — already references edge cases generally.
- No task decomposition impact — the test case for empty graphs remains, just traceable to FR-036 instead of FR-022.

---

### 2.2 FR-017 vs FR-032 — Convergence Thresholds

**Current State:**

| Aspect | FR-017 | FR-032 |
|--------|--------|--------|
| Primary concern | Observable events (including ConvergencePlateau) | Convergence configuration and termination |
| Threshold mentioned | Plateau threshold (1e-7) explained in depth | Convergence threshold (1e-6) with cross-ref to FR-017 for plateau |
| Design rationale | Extensive (4 paragraphs explaining two-threshold design) | Brief note referencing FR-017 |
| Termination semantics | "Plateau events do NOT trigger algorithm termination" | "Convergence threshold is the stopping criterion" |

**Problem**: Both FRs explain the relationship between 1e-6 (convergence) and 1e-7 (plateau), creating redundancy. FR-017 contains a "Note" section that restates FR-032's content. However, the two FRs serve genuinely different primary concerns: FR-017 is about observability events, FR-032 is about algorithm termination.

**Recommended Strategy**: **Keep both, but eliminate explanatory duplication.** Each FR owns its primary concern; shared context is cross-referenced, not restated.

**Exact Text Reorganization:**

1. **In FR-017**: Keep the ConvergencePlateau event definition and the N=5 persistence requirement. Remove the "Note" section (lines 279) that restates FR-032's convergence threshold semantics. Replace with a shorter cross-reference:
   > "See FR-032 for the convergence threshold (1e-6) and termination semantics. The plateau threshold (1e-7) is one order of magnitude stricter and used solely for event emission."

2. **In FR-032**: Remove the sentence "A separate, stricter plateau threshold (1e-7, defined in FR-017) is used for observability events that do NOT trigger termination." This is already covered by FR-017's cross-reference. Replace with:
   > "The plateau threshold for observability events is defined in FR-017."

**Cross-Reference Updates Needed:**

| Location | Current Text | Updated Text |
|----------|--------------|--------------|
| FR-017 lines 272-275 | Detailed explanation of plateau vs convergence relationship | Short paragraph: "A convergence plateau event is emitted when quality improvement remains below a sub-convergence threshold (1e-7) for N consecutive iterations (default N=5). This is an observability signal only — see FR-032 for convergence threshold and termination semantics." |
| FR-017 lines 277-278 | Design rationale paragraph | Keep — this explains *why* the two-threshold design exists (unique to observability concern) |
| FR-017 lines 279 | "Note" section restating FR-032 | Delete entirely |
| FR-032 line 313 | Detailed note about plateau threshold | Replace with: "For the observability plateau threshold (1e-7), see FR-017." |

**Impact on Success Criteria:**
- SC-009 (Observable Events): Already references FR-017 — no change.
- SC-013 (Observability Overhead): References both thresholds — clarify that "base observability" includes plateau detection.
- No task impact — test cases remain unchanged.

---

### 2.3 FR-013 vs FR-045 — Streaming Mutation API

**Current State:**

| Aspect | FR-013 | FR-045 |
|--------|--------|--------|
| Primary concern | Incremental update capability and failure semantics | StreamingDetector trait definition |
| Methods listed | `apply_mutation`, `apply_mutations` (with full signatures) | `apply_mutation`, `apply_mutations` (with full signatures) |
| Failure behavior | Detailed fail-fast semantics (no rollback, partial progress) | Not mentioned |
| Trait definition | Implicit (mentions "The dynamic graph API MUST provide...") | Explicit `StreamingDetector: CommunityDetector` trait |
| Context | Algorithmic behavior (immediate split on disconnect) | Type system / trait hierarchy |

**Problem**: Both FRs define the same two methods with identical signatures. FR-013 embeds method signatures in behavioral requirements; FR-045 defines them as a trait. This creates two sources of truth for the API contract.

**Recommended Strategy**: **FR-045 owns the trait definition and method signatures. FR-013 references the trait and owns the behavioral semantics (failure modes, immediate split, fail-fast).**

**Exact Text Reorganization:**

1. **In FR-013**: Remove the method signatures and failure behavior block. Replace with:
   > "The system MUST support incremental updates for edge insertions and deletions without full recomputation, via the `StreamingDetector` trait (FR-045). Edge deletion that disconnects a community MUST trigger immediate split into connected components (see FR-006 for connectivity verification). **Failure behavior**: If any mutation fails, processing stops immediately and returns the error. Previous successful mutations in the batch are NOT rolled back — the partition reflects partial progress up to the failed mutation. This fail-fast semantics avoids complex rollback logic and gives users visibility into partial application."

2. **In FR-045**: Keep as-is (already well-structured trait definition). Add one sentence at the end:
   > "Failure semantics for mutation methods are defined in FR-013."

**Cross-Reference Updates Needed:**

| Location | Current Text | Updated Text |
|----------|--------------|--------------|
| FR-013 lines 264-265 | Method signatures and failure behavior | Replace with trait reference + behavioral semantics only |
| FR-045 line 325 | Trait definition (no mention of failure) | Add: "See FR-013 for failure semantics." |
| User Story 4, Acceptance 1 | "the incremental update is triggered" | "the StreamingDetector's apply_mutation is called" (more precise) |

**Impact on Success Criteria:**
- SC-005 (Incremental Update Complexity): No change.
- SC-006 (Subtree Stability): References FR-015, not directly affected.
- Task decomposition: When creating implementation tasks, ensure the trait definition task (FR-045) precedes the behavioral semantics task (FR-013).

---

### 2.4 FR-006 vs FR-024 — Connected Communities Invariant

**Current State:**

| Aspect | FR-006 | FR-024 |
|--------|--------|--------|
| Primary concern | Leiden guarantees connected communities; other algorithms may not | Hard invariant: disconnected components MUST never be merged |
| Key content | `has_disconnected_communities()` method contract (return semantics, BFS, O(V+E)) | Distinction: community-internal disconnection vs. component merging |
| Algorithm coverage | Leiden (must be connected), others (may be disconnected) | All algorithms (must not merge components) |
| Verification method | BFS-based `has_disconnected_communities()` | Same method, used for bug detection |

**Problem**: Both FRs discuss the same conceptual distinction (internally-disconnected community ≠ merged components) but from different angles. FR-006 focuses on Leiden's guarantee and the verification method; FR-024 focuses on the universal invariant. The distinction is clarified in Session 2026-09-03 (4) Q&A, but both FRs restate parts of it.

**Recommended Strategy**: **FR-006 owns the verification method and algorithm-specific guarantees. FR-024 owns the universal invariant. Add a cross-reference in each.**

**Exact Text Reorganization:**

1. **In FR-006**: Keep the `has_disconnected_communities()` contract and the algorithm-specific guarantees. Add at the end:
   > "Note: FR-024 defines the universal invariant that disconnected components MUST never be merged, which is a separate concern from algorithm-specific connected-community guarantees."

2. **In FR-024**: Keep the invariant statement. Replace the last sentence ("This guarantee is inherent to the Leiden algorithm's connected-community design (see FR-006)") with:
   > "This invariant applies to all algorithms. For algorithm-specific connected-community guarantees (Leiden) and the `has_disconnected_communities()` verification method, see FR-006."

**Cross-Reference Updates Needed:**

| Location | Current Text | Updated Text |
|----------|--------------|--------------|
| FR-006 line 256 | Ends with "for post-hoc partition validation" | Add cross-ref to FR-024 (see above) |
| FR-024 line 297 | "This guarantee is inherent to the Leiden algorithm's connected-community design (see FR-006). For all algorithms..." | Replace with cleaner cross-ref (see above) |
| Session 2026-09-03 (4) Q&A | Explains the distinction | No change — historical record |

**Impact on Success Criteria:**
- SC-001 (Connected Communities Guarantee): References FR-006 — no change.
- SC-006 (Subtree Stability): No direct impact.
- Property-based testing: "Leiden: Two disconnected graph components are never merged" — already references both invariants.
- No task impact.

---

### 2.5 FR-010 vs FR-012 — Node Indexing

**Current State:**

| Aspect | FR-010 | FR-012 |
|--------|--------|--------|
| Primary concern | Query API for partition results | Mapping behavior for non-contiguous IDs |
| Key content | Opaque dense contiguous index; query methods | Bidirectional mapping at construction time |
| Methods mentioned | `community_of(node_id)`, `node_at_index(index)` | `graph.node_index(original_id)`, `graph.node_id_at(index)` |
| Direction | User-facing (partition results) | Internal (graph construction) |
| Node ID types | Original node identifiers (generic) | Non-contiguous and zero-based integer IDs |

**Problem**: FR-010 and FR-012 describe two sides of the same indexing system — FR-010 from the consumer perspective (querying results by original ID) and FR-012 from the storage perspective (how mapping works at construction). The mapping cannot be understood without reading both.

**Recommended Strategy**: **Merge into a single FR that covers the complete indexing lifecycle: construction-time mapping → internal representation → query API.**

**Exact Text Reorganization:**

1. **Replace FR-010 and FR-012 with a single FR-010:**

   > **FR-010**: The system MUST use an opaque dense contiguous index (u32/u64) internally, with a bidirectional mapping between original node identifiers and internal indices established at graph construction time. The mapping is accessible only through query methods:
   > - `graph.node_index(original_id) -> Option<usize>`: Resolves original → internal (available after construction)
   > - `graph.node_id_at(index) -> Option<NodeId>`: Resolves internal → original (available after construction)
   > - `partition.community_of(node_id) -> Option<&CommunityId>`: Queries partition results by original node ID
   > 
   > Users cannot inspect or control internal index assignment directly. Non-contiguous identifiers (including zero-based integers) are mapped to a dense contiguous range (0..n-1) at construction time. The framework emits partition results queryable by original node identifier, abstracting the internal index representation.

2. **Delete the original FR-010 and FR-012.**

3. **Renumber**: Since FR-011 follows FR-010 and FR-013 follows FR-012, the merged FR-010 takes the position of the original FR-010. FR-011 remains as-is. FR-012 becomes FR-011, and all subsequent FRs are NOT renumbered (to avoid massive renumbering, the deleted FR-012 can be marked as "Reserved — merged into FR-010" or the sequence can simply skip from FR-010 to FR-011).

**Cross-Reference Updates Needed:**

| Location | Current Reference | Updated Reference |
|----------|-------------------|-------------------|
| User Story 2, Acceptance 2 | "non-contiguous integer node identifiers" (no explicit FR ref) | Reference FR-010 |
| User Story 2, Acceptance 4 | "zero-based node identifiers" (no explicit FR ref) | Reference FR-010 |
| User Story 2, Alternate Flow 1 | "string node identifiers" (no explicit FR ref) | Reference FR-010 |
| Key Entities → Graph | "Has a directionality flag" | No change needed |
| Session 2026-09-04 (1) Q&A | "Opaque dense contiguous index" answer | Update FR reference to FR-010 |

**Impact on Success Criteria:**
- SC-007 (Descriptive Typed Errors): References invalid node queries — no change needed.
- SC-014 (Asymmetric Weight Symmetry): Unaffected.
- Task decomposition: Merge any tasks split across FR-010 and FR-012 into a single "Node Indexing" task group.

---

### 2.6 FR-031 vs FR-037 — Logging

**Current State:**

| Aspect | FR-031 | FR-037 |
|--------|--------|--------|
| Primary concern | Zero-trust logging policy (what must NOT appear in logs) | Logging infrastructure (destinations, rotation, sanitization) |
| Key content | No graph data/node IDs/topology in any log tier unless explicitly enabled | File output with path/rotation, stdout without decorative formatting |
| Policy statement | "MUST adopt a zero-trust logging posture by default" | References FR-031 for sanitization requirement |
| Destinations | Not mentioned | stdout, file, tracing subscriber ecosystem |

**Problem**: FR-037 includes the statement "Each output destination MUST apply sanitization to prevent graph data, node identifiers, or topology information from appearing in logs unless explicitly enabled by the user (per FR-031)." This duplicates FR-031's core policy statement. However, FR-037's primary concern (output destinations) is distinct from FR-031's concern (what data is prohibited).

**Recommended Strategy**: **Keep both. FR-031 owns the data classification policy. FR-037 owns the output mechanism. Eliminate the restated policy in FR-037.**

**Exact Text Reorganization:**

1. **In FR-031**: Keep as-is. This is the authoritative source for zero-trust logging policy.

2. **In FR-037**: Replace the sanitization sentence:
   - Current: "Each output destination MUST apply sanitization to prevent graph data, node identifiers, or topology information from appearing in logs unless explicitly enabled by the user (per FR-031)."
   - Updated: "All output destinations MUST comply with the data sanitization policy defined in FR-031."

**Cross-Reference Updates Needed:**

| Location | Current Text | Updated Text |
|----------|--------------|--------------|
| FR-037 line 317 | Long restatement of FR-031 policy | Short cross-ref to FR-031 (see above) |
| SC-015 (Zero-Trust Logging) | Tests compliance with logging policy | No change needed |
| Key Entities | No logging entity defined | Consider adding a "Logging" entity section for completeness |

**Impact on Success Criteria:**
- SC-015 (Zero-Trust Logging): Tests that no graph data appears in logs. Still valid — references FR-031's policy.
- No task impact — implementation tasks for logging remain unchanged.

---

### 2.7 FR-035 vs FR-040 — Error Handling Hierarchy

**Current State:**

| Aspect | FR-035 | FR-040 |
|--------|--------|--------|
| Scope | All fallible operations (general principle) | File parsers only (specific application) |
| Error categories | 4 broad categories (graph input, negative weights, convergence failure, invalid config) | 3 specific fields (line number, error type, expected format) |
| Enforcement | MUST return typed domain errors using `thiserror` | MUST fail fast with descriptive errors per FR-035 |
| Relationship | General principle | Specific application of the principle |

**Problem**: The hierarchy is already clear (FR-035 = principle, FR-040 = specific case). The duplication is minimal — FR-040 references FR-035 and adds specificity. The only redundancy is the phrase "descriptive typed domain errors" which appears in both.

**Recommended Strategy**: **Keep both. The hierarchy is correct. Minor wording cleanup only.**

**Exact Text Reorganization:**

1. **In FR-035**: Keep as-is. This is the authoritative error handling principle.

2. **In FR-040**: Minor cleanup to reduce word overlap:
   - Current: "File parsers (EdgeList, JSON, GML) MUST fail fast on malformed input with descriptive typed domain errors (per FR-035)."
   - Updated: "File parsers (EdgeList, JSON, GML) MUST fail fast on malformed input, returning typed domain errors (per FR-035) with the following additional fields: line number, error type, and expected format."

**Cross-Reference Updates Needed:**

| Location | Current Text | Updated Text |
|----------|--------------|--------------|
| FR-040 line 320 | "descriptive typed domain errors (per FR-035)" | "typed domain errors (per FR-035) with additional fields..." |
| FR-041 (serialization errors) | "typed domain errors (per FR-035)" | Already correct |
| FR-044 (domain error types) | Defines per-module errors | No change needed |

**Impact on Success Criteria:**
- SC-007 (Descriptive Typed Errors): Tests all error conditions. No change needed.
- No task impact — error handling is already well-structured.

---

## 3. Summary of Consolidation Decisions

| Duplication | Strategy | Action |
|-------------|----------|--------|
| FR-022 vs FR-036 | **Delete FR-022** | FR-036 is authoritative; update cross-refs |
| FR-017 vs FR-032** | **Cross-reference** | Remove duplicated explanatory notes; each FR owns its primary concern |
| FR-013 vs FR-045 | **Split by concern** | FR-045 owns trait/method signatures; FR-013 owns behavioral semantics |
| FR-006 vs FR-024 | **Cross-reference** | Each FR owns its perspective; add mutual cross-refs |
| FR-010 vs FR-012 | **Merge** | Single FR covering full indexing lifecycle |
| FR-031 vs FR-037 | **Cross-reference** | FR-031 owns policy; FR-037 owns mechanism |
| FR-035 vs FR-040 | **Keep hierarchy** | Minor wording cleanup only; hierarchy is already correct |

---

## 4. Implementation Priority

### High Impact (eliminates substantive duplication):
1. **FR-022 deletion** — removes a fully redundant requirement
2. **FR-010/FR-012 merge** — unifies two halves of the same indexing system
3. **FR-013/FR-045 split** — prevents API contract duplication

### Medium Impact (reduces explanatory overlap):
4. **FR-017/FR-032 note removal** — eliminates restated content
5. **FR-031/FR-037 sanitization dedup** — removes policy restatement

### Low Impact (clarifies without content change):
6. **FR-006/FR-024 cross-refs** — adds navigation aids
7. **FR-035/FR-040 wording** — cosmetic cleanup

---

## 5. Verification Checklist

After consolidation, verify:

- [ ] Every FR has exactly one authoritative definition (no method signatures in two FRs)
- [ ] Cross-references are bidirectional where appropriate (FR-006 ↔ FR-024)
- [ ] No FR references a deleted FR (update all FR-022 refs to FR-036)
- [ ] Success criteria traceability is preserved (SCs reference valid FR IDs)
- [ ] User story exception flows reference valid FRs
- [ ] Key Entities section remains consistent with FR changes
- [ ] Historical Q&A sessions are not modified (clarifications are preserved as context)

---

## 6. Lessons for Future Spec Development

1. **Define API contracts in one place**: Method signatures belong in trait definition FRs, not behavioral FRs.
2. **Separate policy from mechanism**: What must be done (policy) and how it's implemented (mechanism) should be in separate FRs with cross-references.
3. **Avoid "Note" sections that restate other FRs**: Use cross-references instead.
4. **When an FR is subsumed**: Delete it entirely rather than keeping a stub. Redirect all references to the authoritative FR.
5. **Merge lifecycle-spanning requirements**: If two FRs describe different phases of the same system (construction → storage → query), they should be one FR.

---

*Sources:*
- [Requirements Quality According to ISO/IEC/IEEE 29148 — LinkedIn](https://www.linkedin.com/pulse/requirements-quality-according-isoiecieee-29148-villarrubia-guarino-sx8ce)
- [ISO/IEC/IEEE 29148:2018 — IEEE Xplore](https://ieeexplore.ieee.org/document/8559686)
- [ISO/IEC/IEEE 29148 Overview — ISO](https://www.iso.org/obp/ui#!iso:std:iso-iec-ieee:29148:ed-2:v1:en)

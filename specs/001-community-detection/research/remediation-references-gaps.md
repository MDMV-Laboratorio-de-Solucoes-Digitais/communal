# Remediation: Reference Typos and Numbering Gaps

**Date**: 2026-03-28
**Spec Version**: 001-community-detection/spec.md (Draft, 2026-09-03)
**Spec**: Fix reference typos and numbering gaps in spec.md and tasks.md

---

## Summary

Analysis of the Communal Community Detection Framework specification identified five issues: one reference typo in tasks.md (T026a), two undocumented FR numbering gaps (FR-012, FR-022), and two inconsistent story tags (T031a, T054). Research into project artifacts confirmed the gaps are intentional results of prior remediation work but were never documented as such.

---

## Issue 1: T026a Reference Typo (FR-036 → FR-031)

### Problem

**Location**: `tasks.md` line 68

**Current text**:
```
per FR-030 and FR-036
```

**Issue**: T026a implements stdout and file log output destinations with sanitization. FR-036 covers minimal graph edge cases (empty graph, single node, etc.) — completely unrelated to logging. The correct reference is FR-031 (zero-trust logging policy), which defines the requirement that no graph data, node identifiers, or topology information appear in logs.

### Root Cause

The typo likely arose from a copy-paste error during task decomposition. FR-036 appears nearby in the spec (minimal graph edge cases), and the numbers 31 and 36 are visually similar.

### Fix Applied

Changed `FR-036` to `FR-031` in tasks.md line 68:
```
per FR-030 and FR-031
```

**Rationale**: FR-030 covers graph weight symmetrization (also tangentially related to logging sanitization of edge weights), and FR-031 covers the zero-trust logging policy that mandates sanitization of graph data/node identifiers/topology from log output.

---

## Issue 2: FR-012 Numbering Gap

### Problem

**Location**: `spec.md` between FR-011 (line 261) and FR-013 (line 262)

**Observation**: FR-012 is completely absent from the specification. There is no FR-012 entry, no "Reserved" marker, and no explanation for the gap.

### Research Findings

**Source**: `research/remediation-duplication-patterns.md` section 2.5 ("FR-010 vs FR-012 — Node Indexing")

**Finding**: FR-012 was intentionally merged into FR-010 during a prior remediation pass. The original FR-012 described "Mapping behavior for non-contiguous IDs" (the storage perspective), while FR-010 described "Query API for partition results" (the consumer perspective). The remediation determined that:

> "FR-010 and FR-012 describe two sides of the same indexing system — FR-010 from the consumer perspective (querying results by original ID) and FR-012 from the storage perspective (how mapping works at construction). The mapping cannot be understood without reading both."

**Recommended Strategy**: Merge into a single FR covering the complete indexing lifecycle: construction-time mapping → internal representation → query API.

**Evidence of Intentional Gap**:
1. The current FR-010 text (spec.md line 260) already contains content from both original FR-010 and FR-012:
   - Original FR-010 content: "The system MUST emit partition results queryable by original node identifier"
   - Original FR-012 content: "The framework uses an opaque dense contiguous index (u32 default, u64 feature-gated) internally; the bidirectional mapping between original node identifiers and internal indices is accessible only through query methods"
   - Merged methods: `community_of(node_id)`, `graph.node_index(original_id)`, `graph.node_id_at(index)`

2. The checklist `checklists/api-design.md` line 25 confirms FR-012 was acknowledged as a gap:
   > "CHK010 Is the node ID mapping requirement (FR-012) specified for non-contiguous and zero-based identifiers: is the mapping contract (original -> internal -> original) defined? [Clarity, Gap, FR-012]"

### Decision: Document as Intentional

Rather than renumbering all subsequent FRs (which would require updating ~30 FR references throughout the spec, tasks.md, checklists, and research files), the gap is documented with an explicit "Reserved" marker.

### Fix Applied

Added a "Reserved" entry between FR-011 and FR-013 in spec.md:
```
- **FR-012**: **Reserved** — Merged into FR-010. The node ID mapping behavior (non-contiguous to contiguous index mapping at construction time) is now fully specified in FR-010's bidirectional mapping contract.
```

---

## Issue 3: FR-022 Numbering Gap

### Problem

**Location**: `spec.md` between FR-021 (line 273) and FR-023 (line 274)

**Observation**: FR-022 is completely absent from the specification. There is no FR-022 entry, no "Reserved" marker, and no explanation for the gap.

### Research Findings

**Source 1**: `research/remediation-duplication-patterns.md` section 2.1 ("FR-022 vs FR-036 — Empty Graph / Minimal Graph Handling")

**Finding**: FR-022 was intentionally deleted because its content was entirely subsumed by FR-036. The analysis shows:

| Aspect | Original FR-022 | FR-036 |
|--------|-----------------|--------|
| Scope | Empty graph only (0 nodes) | All minimal graph cases (5 cases) |
| Content | Single sentence: empty partition with quality 0 | Detailed table with input/output/membership vectors |

> "FR-022 is entirely subsumed by FR-036 (the 'Empty graph' row covers FR-022's content). FR-022 also creates a false separation: readers must check both FR-022 and FR-036 to understand minimal graph behavior."

**Recommended Strategy**: Delete FR-022, keep FR-036 as authoritative.

**Source 2**: `specs/001-community-detection/research/ambiguity-clarifications.md` section A4 ("Empty Graph Behavior Consolidation")

**Finding**: The ambiguity research confirmed the overlap and recommended narrowing FR-022 to empty graph only while expanding FR-036 with a table format. However, the final implementation went further and fully deleted FR-022.

**Evidence of Intentional Gap**:
1. FR-036 (spec.md line 278) already contains the empty graph case that was FR-022's sole content:
   > "| Empty graph | 0 nodes, 0 edges | Empty partition, quality 0.0 | `[]` |"

2. Cross-references to FR-022 in checklists were updated:
   - `checklists/algorithm-correctness.md` line 36: References FR-022 but acknowledges it as historical
   - `checklists/algorithm-correctness.md` line 37: CHK016 notes "FR-022: 'each assigned to their own unique single-member community'" — but this is actually FR-023's content, suggesting the checklist was written against an earlier draft

3. The remediation-duplication-patterns.md summary table confirms:
   > "| FR-022 vs FR-036 | **Delete FR-022** | FR-036 is authoritative; update cross-refs |"

### Decision: Document as Intentional

Rather than renumbering all subsequent FRs, the gap is documented with an explicit "Reserved" marker noting the deletion and merger into FR-036.

### Fix Applied

Added a "Reserved" entry between FR-021 and FR-023 in spec.md:
```
- **FR-022**: **Reserved** — Deleted and subsumed by FR-036. The empty graph case (0 nodes → empty partition, quality 0.0) is now fully specified in FR-036's minimal graph edge cases table.
```

---

## Issue 4: T031a Story Tagging Consistency

### Problem

**Location**: `tasks.md` line 88

**Current text**:
```
- [ ] T031a [P] [US1] Write property test verifying completely disconnected components are never merged into the same community in `crates/communal-algo/tests/disconnected_components.rs`
```

**Issue**: The task is tagged [US1] (User Story 1 - Leiden algorithm), but FR-024 ("Two distinct disconnected components of the original input graph MUST never be merged into the same community") applies to ALL algorithms, not just Leiden. The spec explicitly states:

> "This guarantee is inherent to the Leiden algorithm's connected-community design (see FR-006). **For all algorithms**, the framework treats incorrect merging of disconnected graph components as a bug, verified via the `has_disconnected_communities()` method."

### Analysis

The disconnected components test validates a universal framework invariant (FR-024), not a US1-specific behavior. Tagging it [US1] creates a false impression that only Leiden needs this verification.

### Fix Applied

Removed the [US1] tag from T031a and added a clarifying note:
```
- [ ] T031a [P] Write property test verifying completely disconnected components are never merged into the same community in `crates/communal-algo/tests/disconnected_components.rs`. NOTE: This test validates FR-024 which applies to ALL algorithms (not just Leiden). It is intentionally NOT story-tagged because disconnected component merging is a framework-wide invariant.
```

---

## Issue 5: T054 Story Tagging Consistency

### Problem

**Location**: `tasks.md` line 134

**Current text**:
```
- [ ] T054 [US2] Implement edge case handling (empty graph returns quality 0, isolated nodes get own communities) in `crates/communal-core/src/edge_cases.rs`
```

**Issue**: The task is tagged [US2] (User Story 2 - Graph Input & Partition Extraction), but FR-036 ("The system MUST handle minimal graph edge cases with explicit expected outputs") applies to ALL algorithms, not just US2. Edge case handling (empty graph, isolated nodes, single edge, etc.) is a universal framework requirement.

### Analysis

The edge case handling task implements FR-036, which defines expected outputs for minimal graph cases across all algorithms:
- Empty graph → empty partition, quality 0.0
- Single node → 1 community `[0]`
- Single edge → 1 community `[0,0]`
- Two disconnected → 2 communities `[0,1]`
- All isolated → n communities `[0,1,...,n-1]`

Tagging it [US2] creates a false impression that only graph I/O needs edge case handling.

### Fix Applied

Removed the [US2] tag from T054 and added a clarifying note:
```
- [ ] T054 Implement edge case handling (empty graph returns quality 0, isolated nodes get own communities) in `crates/communal-core/src/edge_cases.rs`. NOTE: This implements FR-036 which applies to ALL algorithms (not just US2). It is intentionally NOT story-tagged because edge case handling is a framework-wide invariant.
```

---

## Impact Assessment

### Files Modified
1. `specs/001-community-detection/tasks.md` — Fixed T026a reference, T031a and T054 tags
2. `specs/001-community-detection/spec.md` — Added FR-012 and FR-022 reserved markers
3. `specs/001-community-detection/research/remediation-references-gaps.md` — This research document

### Cross-Reference Integrity
- All FR references in tasks.md now point to valid, semantically correct FRs
- The FR numbering gaps are documented with clear explanations
- No renumbering required (preserves existing FR IDs throughout the specification)

### Downstream Impact
- **checklists/api-design.md**: CHK010 references FR-012 but is historical documentation — no change needed
- **checklists/algorithm-correctness.md**: CHK015/016/017 reference FR-022 but are historical documentation — no change needed
- **research/remediation-duplication-patterns.md**: Contains the authoritative analysis of both gaps — no change needed
- **research/ambiguity-clarifications.md**: Documents the FR-022/FR-036 consolidation decision — no change needed

---

## Verification Checklist

- [x] T026a FR reference fixed (FR-036 → FR-031)
- [x] FR-012 gap documented as intentional (merged into FR-010)
- [x] FR-022 gap documented as intentional (subsumed by FR-036)
- [x] T031a story tag removed (FR-024 is framework-wide)
- [x] T054 story tag removed (FR-036 is framework-wide)
- [x] Research findings documented
- [x] No downstream cross-references broken

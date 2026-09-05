# FR-041 Gap Analysis: Specification Numbering Investigation

**Date**: 2026-09-04
**Investigator**: Automated analysis
**Status**: Complete

## Executive Summary

FR-041 is a **missing functional requirement** accidentally skipped during spec reorganization. The numbering jumps from FR-040 (file parsers) directly to FR-042 (CLI commands). Evidence strongly suggests this is an accidental skip rather than an intentional deletion or merge. The gap should be filled with a new FR-041 covering **Graph and Partition Serialization/Output**, which logically complements FR-040 (parsing/input) and provides the underlying capability for FR-042's `convert` command.

---

## 1. Investigation Findings

### 1.1 Current FR Numbering (spec.md)

The functional requirements section (lines 251-303) contains the following sequence around the gap:

| Line | FR | Description |
|------|----|-------------|
| 298 | **FR-040** | File parsers (EdgeList, JSON, GML) MUST fail fast on malformed input |
| 299 | **FR-042** | CLI MUST provide: `run`, `compare`, `batch`, `convert`, `metrics`, `generate`, `validate` |
| 300 | FR-043 | Trait hierarchy: `MultiplexView: GraphView` |
| 301 | FR-044 | Per-module domain error enums |
| 302 | FR-045 | `StreamingDetector` trait for dynamic updates |
| 303 | FR-046 | Observer registration via `subscribe` method |

**FR-041 is absent.** No requirement with that identifier exists anywhere in the current spec.

### 1.2 Spec Numbering Is Non-Sequential

The spec has been through multiple reorganizations. Evidence of this:
- **FR-036 appears at line 279**, before FR-024 (line 280), indicating content was rearranged after initial drafting
- The clarification session on line 86 mentions creating FR-028 to fill a previous numbering gap (graph builders)
- The checklist file references FR numbers that don't match the current spec (see Section 2)

This confirms the spec was iteratively reorganized, making accidental skips likely.

---

## 2. Git History Evidence

### 2.1 Repository Log

```
edc7bdf chore: add .gitignore excluding target/
28115b9 feat: add initial main with welcome message
c7fac1f chore: add workspace Cargo.toml
2f417be docs: add project README
```

**Finding**: The repository has only 4 commits. The spec.md was created in the initial commit and has **never been modified** in any commit. The gap was present from the spec's creation.

### 2.2 FR-041 Search in Git History

Commands run:
```bash
git log --all --oneline -S "FR-041" -- specs/001-community-detection/spec.md
git log --all --oneline -S "FR-041"
```

**Result**: No commits reference FR-041. The string "FR-041" has never appeared in any version-controlled file.

### 2.3 Checklist Reference (Outdated)

In `checklists/api-design.md` (line 103):
```
CHK004, CHK030, CHK034, CHK038 marked satisfied based on clarifications
(FR-040 QualityMetric trait, FR-041 CLI commands, FR-042 trait hierarchy, FR-043 domain errors).
```

**Analysis**: This checklist references a **completely different numbering scheme**:
- Old FR-040 = QualityMetric trait → Now FR-021
- Old FR-041 = CLI commands → Now FR-042
- Old FR-042 = trait hierarchy → Now FR-043
- Old FR-043 = domain errors → Now FR-044

This indicates the checklist was written during an earlier drafting phase before the spec was reorganized. The "FR-041 CLI commands" reference confirms that CLI commands was always intended to have a dedicated FR, but the numbering shifted when new requirements were inserted earlier in the spec.

**Conclusion**: The checklist is outdated and should be updated to match current FR numbering.

---

## 3. Logical Gap Analysis

### 3.1 Context Around the Gap

**FR-040 (File Parsers)**:
- Covers: EdgeList, JSON, GML parsing
- Focus: Input/reading files, error handling for malformed input
- Key phrase: "Parsers MUST NOT silently skip or best-effort parse malformed entries"

**FR-042 (CLI Commands)**:
- Covers: `run`, `compare`, `batch`, `convert`, `metrics`, `generate`, `validate`
- Focus: User-facing command-line interface
- Includes: `convert` (convert between graph formats: JSON, CSV, GML)

### 3.2 What's Missing?

Between file **input** (FR-040) and CLI **interface** (FR-042), there is no requirement for:

1. **Graph serialization** - Writing graphs to files in JSON, CSV, GML formats
2. **Partition serialization** - Writing community detection results to files
3. **Output formatting** - Structuring results for consumption by other tools
4. **Round-trip fidelity** - Ensuring serialized output can be re-parsed

The `convert` command in FR-042 implies the existence of serialization capabilities, but those capabilities are not specified anywhere.

### 3.3 Evidence from tasks.md

Task T125 confirms the existence of output formatting as a distinct concern:
```
- [ ] T125 Implement output formatters (JSON, CSV, GML) in `crates/communal-cli/src/output.rs`
```

This task has no corresponding FR in the current spec, suggesting it was intended to implement FR-041.

### 3.4 Pattern from Previous Gap Fix

The clarification session (line 86) established a pattern for handling numbering gaps:
> "Q: FR-009 references FR-020 for graph builders, but FR-020 defines quality metrics. Should graph builders become a separate FR? → A: Yes, create FR-028 specifically for graph builders - This closes the numbering gap (FR-028 is currently missing)..."

The same pattern should be applied: create FR-041 to fill the gap.

---

## 4. Recommended Remediation

### 4.1 Decision: Create New FR-041

Based on the analysis, FR-041 should be created to cover **Graph and Partition Serialization**. This:
- Complements FR-040 (parsing/input) with serialization/output
- Provides the underlying capability for FR-042's `convert` command
- Matches task T125 in tasks.md
- Follows the established pattern from the FR-028 fix

### 4.2 Proposed FR-041 Text

Insert the following at **line 299** of spec.md (between FR-040 and FR-042):

```markdown
- **FR-041**: The system MUST provide graph and partition serialization capabilities for JSON, CSV, and GML formats, complementing the file parsers (FR-040). Graph serializers MUST produce well-formed output conforming to the format schemas defined in the `contracts/` directory. Partition serializers MUST output community membership vectors, quality scores, and community metadata. All serializers MUST guarantee round-trip fidelity: serialized output MUST be re-parseable by the corresponding parser (FR-040) without data loss or corruption. Serialization errors MUST return typed domain errors (per FR-035) with descriptive messages indicating the failure cause (e.g., I/O error, invalid data state).
```

### 4.3 Exact spec.md Edit

**File**: `/home/luis/development/MDMV/projetos/communal/specs/001-community-detection/spec.md`
**Line**: 299 (currently contains FR-042)

**Change**: Insert new FR-041 before FR-042:

```diff
 - **FR-040**: File parsers (EdgeList, JSON, GML) MUST fail fast on malformed input with descriptive typed domain errors (per FR-035). Each error MUST include: the line number where the error occurred, the error type (e.g., invalid format, missing field, invalid value), and the expected format or valid range. Parsers MUST NOT silently skip or best-effort parse malformed entries.
+- **FR-041**: The system MUST provide graph and partition serialization capabilities for JSON, CSV, and GML formats, complementing the file parsers (FR-040). Graph serializers MUST produce well-formed output conforming to the format schemas defined in the `contracts/` directory. Partition serializers MUST output community membership vectors, quality scores, and community metadata. All serializers MUST guarantee round-trip fidelity: serialized output MUST be re-parseable by the corresponding parser (FR-040) without data loss or corruption. Serialization errors MUST return typed domain errors (per FR-035) with descriptive messages indicating the failure cause (e.g., I/O error, invalid data state).
 - **FR-042**: The CLI MUST provide the following commands with their high-level purposes: `run` (execute community detection on a graph file), `compare` (run multiple algorithms and produce metrics table + pairwise NMI/ARI), `batch` (process multiple files/algorithms/parameters via unified batch configuration), `convert` (convert between graph formats: JSON, CSV, GML), `metrics` (compute quality/comparative metrics for existing partitions), `generate` (generate synthetic benchmark graphs), `validate` (validate graph file structure and content). Detailed argument contracts (flags, options, defaults) are deferred to planning.
```

### 4.4 Secondary Action: Update Checklist

The `checklists/api-design.md` file (line 103) contains outdated FR references and should be updated:

```diff
-CHK004, CHK030, CHK034, CHK038 marked satisfied based on clarifications (FR-040 QualityMetric trait, FR-041 CLI commands, FR-042 trait hierarchy, FR-043 domain errors).
+CHK004, CHK030, CHK034, CHK038 marked satisfied based on clarifications (FR-021 QualityMetric trait, FR-041 graph/partition serialization, FR-043 trait hierarchy, FR-044 domain errors).
```

### 4.5 Alternative: Add Intentional Gap Note

If creating a new FR is deemed unnecessary (e.g., if output formatting is considered part of FR-042 CLI commands), an alternative is to add a note:

```markdown
- **FR-040**: File parsers (EdgeList, JSON, GML) MUST fail fast...
+ **FR-040**: File parsers (EdgeList, JSON, GML) MUST fail fast...
+ 
+ > **Note**: FR-041 is intentionally reserved. Numbering skips from FR-040 (file parsers) to FR-042 (CLI commands) to maintain consistency with the original drafting sequence. No requirement was deleted or merged.
+ 
 - **FR-042**: The CLI MUST provide the following commands...
```

**Recommendation**: The new FR approach (Section 4.2) is preferred because:
1. It adds value by specifying serialization requirements
2. It provides clear guidance for T125 implementation
3. It follows the established pattern from FR-028
4. It's easier to add a requirement than to explain a gap

---

## 5. Summary

| Question | Answer |
|----------|--------|
| Was FR-041 deleted? | No evidence in git history |
| Was FR-041 merged into another FR? | No — content doesn't exist elsewhere |
| Is the gap intentional? | Unlikely — spec was reorganized multiple times |
| What should FR-041 cover? | Graph and Partition Serialization |
| What action is needed? | Insert FR-041 at line 299 of spec.md |
| What else needs updating? | checklists/api-design.md line 103 |

---

## Appendix: Full FR Numbering Audit

For reference, here is the complete FR numbering in spec.md (document order):

```
FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010,
FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020,
FR-021, FR-022, FR-023, FR-036, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029,
FR-030, FR-031, FR-032, FR-033, FR-034, FR-035, FR-037, FR-038, FR-039, FR-040,
[FR-041 MISSING], FR-042, FR-043, FR-044, FR-045, FR-046
```

Note: FR-036 appears out of sequence (before FR-024), confirming the spec was reorganized.

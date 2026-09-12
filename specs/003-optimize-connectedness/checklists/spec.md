# Checklist: Requirements Quality — Optimize Leiden Connectedness Check (spec.md)

**Purpose**: Validate the quality, completeness, clarity, consistency, measurability, and coverage of the requirements in `specs/003-optimize-connectedness/spec.md` (FR-001..FR-008, SC-001..SC-009, clarifications, measurement protocol, edge cases, and scenarios). This is a requirements-quality review, not an implementation verification.

**Created**: 2026-09-10
**Feature**: `specs/003-optimize-connectedness/spec.md`
**Status**: Draft — reviewer-owned; all items intentionally unchecked.

**Note**: Per `research.md`, all clarifications (Sessions 2026-09-09 through 2026-09-10) are resolved (`Status: COMPLETE`; `No remaining NEEDS CLARIFICATION`). Ambiguity items below note this explicitly rather than reopening them.

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. `[x]` means the reviewer has confirmed the requirement-quality criterion is satisfied; it does not mean implementation is complete. Do not mark items `[x]` until reviewed.

---

## Requirement Completeness

- [ ] CHK001 Are all eight functional requirements (FR-001 through FR-008) fully covered in the Requirements section with no omitted removal/check/invariant clauses? [Completeness, Spec §FR-001..FR-008]
- [ ] CHK002 Are all nine success criteria (SC-001 through SC-009) documented, each with an explicit timing target and a quality/invariant threshold where applicable? [Completeness, Spec §SC-001..SC-009]
- [ ] CHK003 Does the spec define the Measurement Protocol (release build, LFR fixed params, median of ≥5 runs, pinned reference machine) that governs all timing-based SC and US acceptance scenarios? [Completeness, Spec §Measurement Protocol]
- [ ] CHK004 Are the Key Entities (Refinement Partition, Community Bound, Isolated Vertex) defined for all concepts relied on by FR-003, FR-004, and the data model? [Completeness, Spec §Key Entities / data-model.md]

## Requirement Clarity

- [ ] CHK005 Is the term "isolated vertices" quantified and disambiguated — specifically that it means nodes in singleton communities within the refined partition (not just nodes with low degree)? [Clarity, Spec §FR-004 / Key Entity #3]
- [ ] CHK006 Is γ explicitly quantified and sourced (resolution parameter of the quality function being optimized, not a separate refinement parameter), with both node-side R and destination-side T arithmetic conditions specified? [Clarity, Spec §FR-004 / Clarifications Session 2026-09-09 (e)]
- [ ] CHK007 Is "debug_assert!" specified with its failure behavior (panic with descriptive message, debug-only, no recovery/splitting), matching the existing codebase convention and constitution Principle I reconciliation? [Clarity, Spec §FR-008 / Clarifications]
- [ ] CHK008 Are the acceptance scenarios for User Story 1 (US1 Scenario 1 ≤5 s binding gate vs SC-001 ≤500 ms stretch target) clearly distinguished as separate purposes (pass/fail baseline vs improvement measure)? [Clarity, Spec §US1 / Clarifications 2026-09-09 (b)]

## Requirement Consistency

- [ ] CHK009 Do FR-001 (remove BFS from local-moving), FR-002 (remove BFS from refinement), FR-003 (singleton initialization), and FR-004 (isolated-vertex-only + γ-filter refinement) align without contradiction — i.e., each phase’s design matches the paper’s Algorithm A.2 and reference implementations? [Consistency, Spec §FR-001..FR-004 / research.md D2]
- [ ] CHK010 Is FR-006’s uniform quality-function behavior consistent with FR-004 (uniform R+T filters) and SC-005/SC-007 (Modularity + CPM coverage, MapEquation stub pinned), with no quality-function-specific code paths proposed? [Consistency, Spec §FR-006 / SC-005 / Clarifications 2026-09-09 (continued)]
- [ ] CHK011 Is FR-005’s connectedness guarantee consistent with FR-008’s debug-only verification and the release-build clause — i.e., by-construction guarantee plus debug assertions suffices, and release builds do not verify? [Consistency, Spec §FR-005 / FR-008 / Clarifications 2026-09-09 (continued)]
- [ ] CHK012 Are the User Scenario acceptance criteria (US1 scenarios 1–3, US2 scenarios 1–3, US3 scenarios 1–3) consistent with the binding SC targets and the measurement protocol, with no conflicting timing/quality expectations? [Consistency, Spec §User Scenarios / SC-001..SC-009]

## Measurability (Acceptance Criteria Quality)

- [ ] CHK013 Is SC-001 measurable with a specific time threshold (≤500 ms for 10k nodes / d=50) plus a separate binding gate (US1 Scenario 1 ≤5 s), both tied to the Measurement Protocol? [Measurability, Spec §SC-001 / US1]
- [ ] CHK014 Is SC-003’s sub-quadratic claim measurable by the two-point ratio bound (time(50k)/time(10k) < 12.5, both d=50) rather than an unspecified "linear" claim, and does it reference committed benchmark files? [Measurability, Spec §SC-003 / Clarifications 2026-09-09 (d)]
- [ ] CHK015 Is SC-009 measurable with both a timing target (PolBlogs ≤500 ms), a quality threshold (modularity Q within 1e-10 of pre-optimization baseline), and a structural invariant (consistent community count), with epsilon normative? [Measurability, Spec §SC-009 / Clarifications 2026-09-10]
- [ ] CHK016 Is SC-006 measurable as 100% of communities internally connected via BFS/DFS across 1,000 random instances, explicitly tied to Principle VI’s verification mechanism? [Measurability, Spec §SC-006]

## Scenario Coverage

- [ ] CHK017 Are Primary scenario requirements complete — i.e., normal refinement initialization, isolated-vertex merge with R+T preconditions, and by-construction connectedness for typical inputs? [Coverage, Spec §FR-003 / FR-004 / FR-005]
- [ ] CHK018 Are Exception / Error scenario requirements defined — specifically the debug_assert! failure (panic, descriptive message, no recovery), and the edge cases of singleton communities, zero-edge graphs, and articulation points? [Coverage, Spec §Edge Cases / FR-008 / Clarifications]
- [ ] CHK019 Are Recovery / Resilience scenario requirements addressed — noting the explicit decision (per `research.md` D4 / Networkit #1244) that recovery/splitting logic is intentionally excluded and fail-fast is required? [Coverage, Spec §FR-008 / Clarifications / research.md]
- [ ] CHK020 Are Non-Functional / Performance scenario requirements covered — scaling (SC-003), determinism (SC-008 / FR-007), release-vs-debug behavior (FR-008 / SC-006), and benchmark persistence (SC-003 / Session 2026-09-10 (e))? [Coverage, Spec §SC-003 / SC-008 / FR-007 / FR-008]

## Edge Case Coverage

- [ ] CHK021 Does the Edge Cases section (singleton community, node with no edges to its community, zero-edge graph, complete graph, articulation point) cover the full boundary space referenced by FR-004, FR-005, and the refinement-phase design? [Edge Cases, Spec §Edge Cases]
- [ ] CHK022 Is the behavior for a disconnected community after refinement defined — i.e., that a debug assertion failure indicates an implementation bug (not a user-correctable condition), and that refinement’s singleton-start + isolated-vertex design already prevents disconnection by construction? [Edge Cases, Spec §FR-008 / Clarifications / data-model.md]

## Ambiguities & Conflicts (Noted: All Resolved Per research.md)

- [ ] CHK023 Are all specification ambiguities from the clarification sessions resolved, with no remaining open items — specifically parallel refining (No), debug failure response (panic / debug_assert!), quality-function uniformity (Yes), baseline updates (struck / qualitative), and R+T filter enforcement (Both)? [Ambiguity, Spec §Clarifications / research.md §Resolved Clarifications]
- [ ] CHK024 Is the conflict between Principle I ("verifiably connected via BFS/DFS") and FR-008 (debug-only verification / no release check) reconciled in the spec with both dimensions satisfied (debug assertions + SC-006 property tests, technology-agnostic MUST wording)? [Ambiguity / Conflict, Spec §Clarifications Session 2026-09-10 / FR-008]
- [ ] CHK025 Is the MapEquation stub clause (FR-006 / SC-005 / SC-007) pinned to an enforceable criterion (quality returns 0.0, refinement skips; verified by general test suite) rather than left vague? [Ambiguity / Gap, Spec §FR-006 / Clarifications 2026-09-10 (e)]

## Traceability

- [ ] CHK026 Does each FR (001–008) have at least one corresponding SC or scenario reference that verifies it (e.g., FR-001/FR-002 → removal verified by contract + code review; FR-003 → G1; FR-004 → G2 + SC-006; FR-005 → SC-006 + principle; FR-006 → SC-005/SC-007; FR-007 → SC-008; FR-008 → SC-006 + debug design)? [Traceability, Spec §FR-001..FR-008 / contract.md / data-model.md]
- [ ] CHK027 Does each SC (001–009) explicitly reference the governing acceptance criteria (timing, quality, structural, or benchmark requirements) and the measurement protocol where timing is involved? [Traceability, Spec §SC-001..SC-009 / Measurement Protocol]
- [ ] CHK028 Are all assumptions (refinement guarantees connectedness, local-moving needs no check, existing cache reusable, γ is quality resolution, debug assertions sufficient, existing test suite adequate) documented and validated by research / primary sources rather than left implicit? [Traceability / Assumptions, Spec §Assumptions / research.md D1..D10]

---

## Notes

- All 28 items are intentionally unchecked (`[ ]`). Mark `[x]` only after a reviewer confirms the requirement-quality criterion is met.
- Clarifications: Per `research.md` (Status: COMPLETE; 89 lines; `No remaining NEEDS CLARIFICATION`), all clarifications are resolved; ambiguity items (CHK023–CHK024) reflect that resolution rather than reopening questions.
- This checklist tests requirement quality, not implementation: no item asks whether code executes correctly (no "verify", "test", "confirm" + behavior patterns). All items ask whether a requirement is defined, quantified, consistent, measurable, covered, or traceable.
- Related artifacts: `spec.md` (source), `research.md` (resolved clarifications / decisions D1–D10), `data-model.md` (entities / validation rules), `contracts/leiden-refinement-contract.md` (G1–G4 guarantees), `.specify/templates/checklist-template.md` (structure).

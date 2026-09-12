# Verification / Correctness / Determinism — Requirement-Quality Checklist

**Purpose**: Unit-test the verification, correctness, and determinism requirements for quality of specification (NOT execution/implementation). Tests whether the spec's verification clauses, success criteria, protocol, and consistency assertions are clearly written, measurable, consistent, and fully covered.

**Created**: 2026-09-10
**Feature**: [spec.md](../spec.md) / [contracts/leiden-refinement-contract.md](../contracts/leiden-refinement-contract.md) / [research.md](../research.md)
**Focus**: Requirement-quality review for verification / correctness / determinism domain (SC-004 through SC-008, FR-008, verification protocol, property-based spec, FR-005/FR-008 consistency).

**Note**: Checklist items test the requirements themselves — completeness, clarity, consistency, measurability, coverage — per `/speckit-checklist` skill. They do NOT instruct execution (no "run BFS", no "verify Q"). `[x]` belongs to the reviewer; not applied here.

**Review Ownership**: Reviewer-owned artifact. Do NOT mark `[x]` until the criteria are satisfied.

---

## Requirement Clarity — Success Criteria

- [x] CHK001 — Is SC-004's "identical community assignments and quality scores" for Tier 1 quantified with an explicit equality definition (byte-for-byte partition + Q equality, not just "close")? [Clarity, Spec §SC-004]
- [ ] CHK002 — Is SC-005's "quality scores within floating-point epsilon of baseline" defined with a normative epsilon value (1e-10 stated in SC-009 / Session 2026-09-09 (continued)) rather than left to interpreter discretion? [Clarity, Spec §SC-005, Session 2026-09-09 (continued)]
- [x] CHK003 — Are the Tier 2 NMI thresholds (≥0.95 default; ≥0.90 permitted for μ ≥ 0.5) explicitly tied to a documented rationale source (`research/leiden-nmi-thresholds.md`) rather than stated as bare numbers? [Clarity / Traceability, Spec §SC-005, Session 2026-09-09 (b)]
- [x] CHK004 — Does SC-005 explicitly restrict quality-function regression coverage to Modularity + CPM and state that MapEquation stub behavior is pinned (not implicitly assumed)? [Completeness, Spec §SC-005, D7, Session 2026-09-10]
- [x] CHK005 — Is SC-006's property-based requirement specific about instance count (1,000 random graph instances) and the traversal method (BFS/DFS) rather than using vague "property-based tests" language? [Clarity, Spec §SC-006, FR-008 / G6]
- [x] CHK006 — Is SC-007's "within floating-point epsilon" for Q across "any test graph" paired with a naming of covered quality functions (Modularity + CPM), matching SC-005's scope, so the two criteria are aligned rather than overlapping ambiguously? [Consistency / Clarity, Spec §SC-007, D7]
- [x] CHK007 — Is SC-008's "byte-for-byte identical partitions (membership and quality)" qualified with the required precondition (same seed, same graph) so determinism is bounded to a defined scenario, not claimed unconditionally? [Clarity / Edge Case, Spec §SC-008, FR-007]

---

## Requirement Completeness — Verification Protocol & Method

- [ ] CHK008 — Is the verification protocol in the Measurement Protocol (§Measurement Protocol, lines 157–158) fully specified for verification-quality evaluation (release build, LFR fixed params, 5-run median, pinned reference machine) rather than only for timing gates (SC-001/002/003/009)? [Completeness / Gap, Spec §Measurement Protocol]
- [ ] CHK009 — Does the protocol explicitly distinguish which checks run under release build vs. debug build (SC-006 property tests + debug_assert! verification vs. release performance measurement), so a reviewer can tell whether verification is executed in each mode? [Completeness, Spec §FR-008, D4, Contract G6]
- [ ] CHK010 — Are the verification-protocol details for SC-004 / SC-005 / SC-006 / SC-007 / SC-008 listed (frequency: after refinement per iteration per Session 2026-09-09 (c); no release-mode connectedness check per D4 / G6), rather than only implied by the contract table (§6)? [Completeness, Contract §6, Session 2026-09-09 (c)]
- [ ] CHK011 — Is the property-based test framework specified (Proptest, or equivalent; random instance count = 1,000; graph profiles from LFR / benchmark corpus) rather than stated generically as "property-based tests"? [Gap / Completeness, Spec §SC-006, Principle VI; check for tool naming in spec — if absent, gap]
- [x] CHK012 — Are the graph profiles for property-based instances defined (Tier 1 deterministic reference graphs, Tier 2 LFR with μ/d, Tier 3 real-world) so 1,000 instances are reproducible, not arbitrary? [Gap / Completeness, Spec §SC-006 vs. SC-003 / SC-009 profiles]

---

## Requirement Consistency — FR-005 / FR-008 / Contract G4–G6

- [x] CHK013 — Is the consistency between FR-005 ("System MUST guarantee all communities internally connected") and FR-008 ("debug-build-only assertions; release MUST NOT verify") explicitly reconciled in the spec (Session 2026-09-09 continued / Session 2026-09-10) rather than left to the reader to infer? [Consistency / Conflict, Spec §FR-005 + §FR-008, clarifications 2026-09-09 (continued)]
- [x] CHK014 — Is the reconciliation mechanism documented (Principle I verifiability = FR-008 debug assertions + SC-006 property tests; by-construction proof via Theorem 5 / Appendix D.1; no release overhead required) so the two requirements don't read as contradictory? [Consistency / Traceability, Spec §FR-008 paragraph; Contract G4 + G6]
- [x] CHK015 — Does the contract (§3 G4, §4 Constraints, §6 Verification Protocol) align with FR-008 so that "no release-mode verification" and "debug-only after refinement per iteration" are consistent across spec, contract, and research (D4)? [Consistency, Contract G6, D4]
- [x] CHK016 — Is the fail-fast behavior (panic on disconnect via `debug_assert!`) specified consistently across FR-008, clarification 2026-09-09, contract §4 (no recovery/split logic), and D4 — with no contradiction between "panic" and "guarantee"? [Consistency, Spec §FR-008, research.md D4, Contract §4]

---

## Requirement Clarity — Debug-Only Verification (FR-008) Scope

- [x] CHK017 — Is FR-008's assertion mechanism explicitly named (`debug_assert!`, not just "debug-build-only assertions") and its placement explicitly bounded (after refinement phase in each iteration, per Session 2026-09-09 (c))? [Clarity / Traceability, Spec §FR-008, Session 2026-09-09 (c)]
- [x] CHK018 — Is the release-build prohibition unambiguous ("Release builds MUST NOT perform runtime connectedness verification") with a rationale (reference implementations don't; by-construction proof) rather than only a prohibition? [Clarity / Measurability, Spec §FR-008, D4, clarifications 2026-09-09 (continued)]
- [x] CHK019 — Is the frequency/value of verification clearly stated (only after refinement phase per iteration, not after local-moving or aggregation, because refinement is the new logic) so the cost/benefit is explicit in requirements? [Clarity, Session 2026-09-09 (c), Contract G6]
- [ ] CHK020 — Does the verification protocol (§6 table, G6 check) specify exactly how to confirm the release build does NOT contain verification (e.g., `grep` for `verify_communities_connected` outside `#[cfg(debug_assertions)]`; build with `--release`), making the requirement testable without code inspection ambiguity? [Measurability / Completeness, Contract §6 G6]

---

## Requirement Completeness — Quality-Function Scope & Equivalence

- [x] CHK021 — Is SC-005 / SC-007's coverage of quality functions stated explicitly (Modularity + CPM; MapEquation stub pinned) rather than implicitly assuming all functions? [Completeness, Spec §SC-005, §SC-007, D7, Session 2026-09-10]
- [ ] CHK022 — Is the equivalence/identical-assignment distinction clear across tiers: SC-004 = byte-for-byte identical for deterministic Tier 1; SC-005/SC-007/SC-009 = Q within epsilon + structural invariants for Tier 2/3; SC-008 = same-seed determinism — with no cross-tier ambiguity? [Clarity / Consistency, Spec §SC-004 / §SC-005 / §SC-008 / §SC-009; D10]
- [x] CHK023 — Are edge-case verification requirements defined (singleton communities trivially connected; zero-edge graph all singletons; empty graph) under SC-006 / SC-004 so property tests have defined passing conditions for boundary graphs? [Coverage / Edge Case, Spec §Edge Cases (line 61–68); Contract G1–G4]

---

## Requirement Quality — Ambiguities / Gaps / Assumptions to Clarify

- [ ] CHK024 — [Gap] Is the property-based framework (Proptest / `proptest` crate, or custom generator with `StdRng` seed recording) named in spec/plan; if absent, does the checklist flag that the instance-generation mechanism must be defined before SC-006 is enforceable? [Gap / Assumption, Spec §SC-006; check `specs/003-optimize-connectedness/` for framework reference]
- [ ] CHK025 — [Ambiguity] Is the meaning of "byte-for-byte identical" for SC-008 fully bounded (same seed + same graph + same build mode = identical `membership` array + identical `Q` float) — and is floating-point determinism assured across platforms/compiler versions? [Ambiguity, Spec §SC-008, FR-007, D10]
- [x] CHK026 — [Assumption] Does the spec document the assumption that BFS/DFS traversal is the correct connectedness verification method (per Principle VI / FR-008 / G6) rather than relying on a different structural check, so verification quality is anchored? [Assumption, Principle VI; Contract G4 + G6; D4]
- [ ] CHK027 — [Conflict / Check] Is the verification protocol's "release vs. debug" distinction consistent with the Measurement Protocol's single-mode (release) framing — i.e., does the protocol clearly say verification is evaluated in debug builds / test suites, while timing is evaluated in release? If not, is that gap noted? [Consistency / Gap, Spec §Measurement Protocol line 157 + Session 2026-09-09 (continued)]

---

## Notes

- Focus chosen per user request: verification / correctness / determinism (not execution/implementation). No item starts with "Verify", "Test", or "Confirm" in the execution sense — all ask whether the requirement is written clearly, consistently, and completely.
- Traceability references: §SC-### / §FR-### / §Contract-G# / Session 2026-09-09 (c)/(continued)/(b) / D4/D10 / Measurement Protocol / Principle VI / `research.md` D4–D7.
- All 27 items unmarked (`[ ]`); none pre-checked. Reviewer should mark `[x]` only after confirming the requirement-quality criterion.
- Related artifacts: `spec.md`, `contracts/leiden-refinement-contract.md` (G1–G6, verification protocol §6), `research.md` (D4 debug-only, D5 single-threaded, D10 equivalence), `.specify/templates/checklist-template.md`.
- No new file created in source tree; only this `checklists/verification.md` checklist.

# Consistency Checklist: Optimize Leiden Connectedness Check

**Purpose**: Test REQUIREMENT CONSISTENCY, AMBIGUITIES, ASSUMPTIONS, and DEPENDENCIES — NOT implementation correctness. This checklist validates whether the specification, clarification settlements, dependency declarations, and traceability mappings are coherent, complete, and unambiguous.
**Created**: 2026-09-10
**Feature**: specs/003-optimize-connectedness/spec.md, research.md
**Review Ownership**: Reviewer-owned requirements-quality artifact. `[x]` = reviewer confirms the consistency/quality criterion is satisfied. It does NOT mean implementation work is complete. Do not mark `[x]` in this file unless explicitly reviewing.

---

## Requirement Consistency — FR-001 to FR-008 Inter-Requirement Alignment

- [x] CHK001 Are FR-001 (remove BFS local-moving) and FR-002 (remove BFS refinement) mutually consistent — i.e., does removing BFS from both phases leave any unguarded connectedness gap that conflicts with FR-005 (guarantee connected communities)? [Consistency, Spec §FR-001/FR-002/FR-005]
- [x] CHK002 Is FR-003 (singleton init for refinement) consistent with FR-002's removal of BFS — does singleton start plus isolated-vertex-only moves (FR-004) replace the BFS guard without contradiction? [Consistency, Spec §FR-003/FR-002/FR-004]
- [x] CHK003 Is FR-004's isolated-only + R/T filter rule consistent with FR-003's singleton initialization — does the eligibility rule assume singletons already exist at refinement start? [Consistency, Spec §FR-004/FR-003]
- [x] CHK004 Does FR-005 (guarantee connected communities) remain consistent with FR-001/FR-002 after the BFS removal — is the by-construction argument (Theorem 5, paper Appendix D.1) explicitly linked to FR-005? [Consistency, Spec §FR-005, Spec §Assumptions]
- [x] CHK005 Is FR-006's uniform behavior across all quality functions consistent with FR-004's R/T filter definition — does the filter reference only structural conditions (singleton + γ-connectivity) with no quality-function-specific branching? [Consistency, Spec §FR-006/FR-004]
- [x] CHK006 Does FR-007 (same-seed determinism) conflict with the stochastic node-ordering property of the Leiden algorithm — is the scope of determinism (same seed → same partition) clearly bounded and consistent with session answers (Tier 1 identical, Tier 2/3 equivalent within epsilon)? [Consistency, Spec §FR-007, Research §D10]
- [x] CHK007 Is FR-008 (debug-only connectedness assertions) consistent with FR-005 — does the spec explicitly state that debug assertions + SC-006 property-based BFS/DFS verification satisfy Principle I's "verifiably connected" requirement? [Consistency, Spec §FR-008, Spec §Assumptions]
- [x] CHK008 Are FR-006's stub-pinning clause (MapEquation quality 0.0, refinement skips) and FR-004's R/T filter consistent — does the stub behavior avoid applying R/T filters to an unimplemented quality function in a contradictory way? [Consistency, Spec §FR-006/FR-004]

---

## Clarification Answer Consistency — Cross-Session Settlement Agreement

- [x] CHK009 Does the uniform quality-function behavior answer (Session 2026-09-09: "Yes — uniform behavior") agree across FR-006 clarifications from multiple sessions — is there any session that suggests quality-function-specific refinement? [Consistency, Research §D6, Spec §FR-006]
- [x] CHK010 Is the γ provenance answer (γ = quality function resolution parameter, not a separate refinement knob) consistent across FR-004 and all clarification sessions (2026-09-09 continued; 2026-09-10 R+T destination filter; Session 2026-09-09 (e))? [Consistency, Spec §FR-004, Research §D3, Research §D2]
- [x] CHK011 Does the R+T filter clarification (Session 2026-09-10 (e): both node-side R and destination-side T enforced) align with FR-004's specification of both conditions on the same paragraph? [Consistency, Spec §FR-004, Research §D2, Research §D7? no — D2/D3/D10 cover it]
- [x] CHK012 Is the debug-only verification settlement (Session 2026-09-09 continued: "B — Debug-only verification is sufficient") consistent with the reconciliation explanation in FR-008 (Principle I satisfied by FR-008 + SC-006)? [Consistency, Spec §FR-008, Research §D4]
- [x] CHK013 Are the scaling validation settlements (Session 2026-09-09 (d): two-point ratio <12.5; Session 2026-09-09 (e): sub-quadratic rewording) consistent with SC-003's stated measurement protocol? [Consistency, Spec §SC-003, Research §D8, Spec §Measurement Protocol]
- [x] CHK014 Does the equivalence-testing settlement (D10: Tier 1 identical; Tier 2/3 equivalent quality within epsilon) agree with SC-004, SC-005, SC-007, and SC-009 language? [Consistency, Research §D10, Spec §SC-004/SC-005/SC-007/SC-009]
- [x] CHK015 Is the stub-pinning settlement for MapEquation (Session 2026-09-10 (e): concrete criterion pinned; tested via SC-005/SC-007) consistent with FR-006 and with SC-005/SC-007's stated quality-function coverage (Modularity + CPM)? [Consistency, Spec §FR-006, Research §D7, Spec §SC-005/SC-007]

---

## Assumptions — Documented and Validated

- [x] CHK016 Is the paper's Theorem 5 (connectedness by construction) explicitly cited as the assumption supporting FR-004/FR-005, with the appendix reference (Appendix C.1 / D.1 of arXiv v3) noted? [Assumption, Spec §Assumptions, Research §D2]
- [x] CHK017 Does the assumption state clearly that γ-connectivity (Theorem 5) is a stronger property than plain connectivity — and is this distinction preserved when justifying FR-005? [Assumption, Spec §Assumptions, Spec §FR-005]
- [x] CHK018 Is the assumption about reference-implementation alignment (libleidenalg singleton-only deviation; igraph full R+T) explicitly documented so the spec's "both filters" decision is traceable? [Assumption, Spec §FR-004, Research §D2, Research §Sources Verified]
- [x] CHK019 Does the assumption about debug-only verification being sufficient cite primary-source evidence (no reference implementation uses release-mode checks) and the constitution reconciliation (Principle I + Principle VI)? [Assumption, Spec §Assumptions, Spec §FR-008, Research §D4]
- [x] CHK020 Is the assumption about existing test-suite sufficiency (SC-006 + property-based BFS/DFS) consistent with the measurement protocol and benchmark persistence requirements? [Assumption, Spec §Assumptions, Spec §SC-006, Spec §Measurement Protocol]
- [x] CHK021 Does the assumption about reuse of current cache state (plan anchor: reuse current cache type) reference a concrete dependency (`LocalMoveState`) without introducing an unverified new dependency? [Assumption, Spec §Assumptions, Plan §? reference if present]

---

## Dependencies — Documented and Unambiguous

- [x] CHK022 Are communal-core traits (`GraphView`, `CommunityDetector`, `QualityMetric`, `QualityFunction`) explicitly listed as dependencies for this feature? [Dependency, AGENTS.md, Spec §FR-006/FR-004]
- [x] CHK023 Is `CsrGraph` (compressed sparse row) referenced as the concrete graph representation the optimization operates on — is there any ambiguity about whether a different graph structure is required? [Dependency, AGENTS.md §CODE MAP, Spec §FR-001/FR-002]
- [x] CHK024 Is `QualityFunction` (dispatch enum for Modularity, CPM, MapEquation stub) documented as the dependency for uniform behavior (FR-006) and for γ provenance (FR-004)? [Dependency, AGENTS.md §CODE MAP, Spec §FR-006/FR-004]
- [x] CHK025 Are contracts / data-model dependencies (`spec.md`, `research.md`, `plan.md`, `tasks.md`, `research/leiden-refinement-design-verification.md`, `research/leiden-merge-nodes-subset-verification.md`) referenced so that clarification evidence is traceable? [Dependency, Spec §Sources Verified, Research §Sources Verified]
- [x] CHK026 Is there any missing dependency on `communal-algo/src/leiden/` (local_moving.rs, refinement.rs, aggregation.rs) that should be declared for FR-001/FR-002/FR-003/FR-004 changes? [Dependency, AGENTS.md §CODE MAP, Spec §FR-001/FR-002]

---

## Edge-Case Coverage Completeness

- [x] CHK027 Does the spec cover empty graph (zero edges) explicitly — are the requirements for all-singleton partitions and Q=0 stated? [Coverage, Edge Case, Spec §Edge Cases]
- [x] CHK028 Is the single-node (singleton) community case covered — is it explicitly noted as trivially connected with no BFS check needed? [Coverage, Edge Case, Spec §Edge Cases, Spec §FR-003/FR-004]
- [x] CHK029 Is the zero-edges graph addressed in both the user scenarios (US2 Scenario 3) and the success criteria (SC-004, SC-006)? [Coverage, Edge Case, Spec §User Story 2, Spec §SC-004/SC-006]
- [x] CHK030 Are articulation-point scenarios explicitly covered — is the explanation (isolated-vertex-only design prevents disconnection) present and linked to FR-004? [Coverage, Edge Case, Spec §Edge Cases, Spec §FR-004]
- [x] CHK031 Are complete-graph communities addressed — is the removal of any node still keeping the community connected explained? [Coverage, Edge Case, Spec §Edge Cases]
- [x] CHK032 Are disconnected cliques (multiple disconnected communities) covered — does US2 Scenario 3 and SC-006 address them? [Coverage, Edge Case, Spec §User Story 2 Scenario 3, Spec §SC-006]
- [x] CHK033 Are high-mixing graphs (μ ≥ 0.5) covered with the relaxed NMI threshold (≥0.90) and documented rationale (`research/leiden-nmi-thresholds.md`)? [Coverage, Edge Case, Spec §SC-005, Spec §Assumptions / NMI settlement]
- [x] CHK034 Is there any missing edge-case scenario (e.g., single-edge graph, star graph, ring graph, bipartite with isolated nodes) that is not mentioned in Edge Cases or Success Criteria? [Gap, Edge Case, Spec §Edge Cases, AGENTS.md §TEST RESULTS]

---

## Ambiguity & Declaration — Remaining Unresolved Questions

- [x] CHK035 Does the spec explicitly declare "no further clarifications needed" — does `research.md` state "No remaining NEEDS CLARIFICATION" and is that statement mirrored in `spec.md` (e.g., Status = Draft with a note, or a clarification-closure section)? [Ambiguity, Research §No remaining NEEDS CLARIFICATION, Spec §Status / Clarifications]
- [x] CHK036 If research.md states all resolved but spec.md does not explicitly declare closure, is there a gap between the clarification log and the specification document — should spec.md contain a "Clarification Status: CLOSED" line? [Ambiguity, Spec §Status, Spec §Clarifications]
- [x] CHK037 Are any ambiguous terms remaining in FR-004 — specifically, is "isolated vertex" defined identically to the paper's refined-partition singleton definition, and is there any ambiguity about whether a node in a multi-node community can become isolated during refinement? [Ambiguity, Spec §FR-004, Key Entities]
- [x] CHK038 Is the term "uniform behavior" (FR-006) quantified — does it explicitly exclude any quality-function-specific branching, including for the unimplemented MapEquation? [Ambiguity, Spec §FR-006, Spec §FR-006 stub clause]
- [x] CHK039 Does the measurement protocol use terms ("fixed recorded parameters", "committed benchmark files", "pinned reference machine") with sufficient specificity to avoid ambiguity during SC-001/SC-002/SC-003/SC-009 evaluation? [Ambiguity, Spec §Measurement Protocol, Spec §SC-001/SC-002/SC-003/SC-009]

---

## Traceability — FR → SC Mapping and Coverage

- [x] CHK040 Is every FR (FR-001 through FR-008) mapped to at least one SC — is the mapping documented in the spec (implicitly via SC descriptions or explicitly in a traceability table)? [Traceability, Spec §Functional Requirements / Success Criteria]
- [x] CHK041 Is FR-001 (remove BFS local-moving) mapped to a measurable SC — does SC-003 (scaling) or SC-001/SC-002 (timing) serve as the proxy, and is that linkage explicit? [Traceability, Spec §FR-001, Spec §SC-001/SC-002/SC-003]
- [x] CHK042 Is FR-002 (remove BFS refinement) mapped to a SC — is the refinement-phase change covered by SC-003, SC-006, or another criterion? [Traceability, Spec §FR-002, Spec §SC-003/SC-006]
- [x] CHK043 Is FR-003 (singleton init) traceable to SC-005 (Tier 2 LFR) or SC-006 (property-based connectedness) — is the initialization step verifiable through these criteria? [Traceability, Spec §FR-003, Spec §SC-005/SC-006]
- [x] CHK044 Is FR-004 (isolated-only + R/T) traceable to SC-006 (BFS/DFS verification) and SC-005/SC-007 (quality regression) — is there any missing criterion specifically for R/T filter enforcement? [Traceability, Spec §FR-004, Spec §SC-006/SC-005/SC-007]
- [x] CHK045 Is FR-006 mapped to SC-005 and SC-007 plus the stub-pinning criterion — does the spec explicitly reference SC-005/SC-007 for Modularity + CPM and the stub behavior for MapEquation (as clarified by Session 2026-09-10 (e))? [Traceability, Spec §FR-006, Spec §SC-005/SC-007, Research §D7]
- [x] CHK046 Is FR-007 (same-seed determinism) mapped exclusively to SC-008 — is there any additional verification mechanism (e.g., property-based determinism) that should be linked? [Traceability, Spec §FR-007, Spec §SC-008]
- [x] CHK047 Is FR-008 (debug-only assertions) mapped to SC-006 (property-based BFS/DFS) and to the constitution reconciliation — is the mapping between FR-008, SC-006, and Principle I/VI explicitly stated? [Traceability, Spec §FR-008, Spec §SC-006, Spec §Assumptions]
- [x] CHK048 Does the stub pin for MapEquation (FR-006) have a concrete acceptance criterion (SC-005/SC-007 covers it via general test suite) — is this linkage explicitly written rather than implied? [Traceability, Spec §FR-006, Spec §SC-005/SC-007, Research §D7]

---

## Notes

- This checklist is a requirements-quality artifact, not an implementation verification plan. It tests whether the spec, clarification log, dependency references, and traceability mappings are coherent, unambiguous, and complete — not whether the code executes correctly.
- Reference sections: `spec.md` (§FR-001 to FR-008; §SC-001 to SC-009; §Edge Cases; §Clarifications; §Assumptions; §Measurement Protocol), `research.md` (§D1–D10; §Resolved Clarifications; §Sources Verified; §No remaining NEEDS CLARIFICATION), `.specify/templates/checklist-template.md`, `AGENTS.md` (CODE MAP; BUGS FOUND; TEST RESULTS; ANTI-PATTERNS).
- Leave all items unchecked (`[ ]`). Mark `[x]` only when a reviewer confirms the consistency/quality criterion is satisfied. Do not use `[x]` for implementation completion.

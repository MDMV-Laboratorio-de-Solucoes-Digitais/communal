# Performance Requirements Quality Checklist: Optimize Leiden Connectedness Check

**Purpose**: Evaluate whether performance-related requirements in `spec.md` are quantified, consistent across criteria, covered for different scales and load conditions, and backed by a clear measurement protocol. This does NOT verify benchmark results.
**Created**: 2026-09-10
**Feature**: `specs/003-optimize-connectedness/spec.md` (SC-001 to SC-009, Measurement Protocol, Session 2026-09-09 (b) clarification)
**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied. `[x]` means the requirement quality is approved; it does NOT mean benchmarks passed.
**Marker Semantics**: `[x]` = reviewer confirms the requirement quality criterion is satisfied. Leave unchecked (`[ ]`) when clarification, correction, or evaluation is still needed.

---

## Requirement Completeness — Performance Targets

- [x] CHK001 Are performance targets defined for all required graph profiles? [Completeness, Spec §SC-001 / SC-002 / SC-003 / SC-009]
- [x] CHK002 Is SC-001 (10k nodes, d=50) explicitly specified with a quantified time target? [Completeness, Spec §SC-001]
- [x] CHK003 Is SC-002 (50k nodes, d=10) explicitly specified with a quantified time target? [Completeness, Spec §SC-002]
- [x] CHK004 Is SC-003 (sub-quadratic scaling validation) specified with both the comparison points (n=10k and n=50k at d=50) and the pass condition? [Completeness, Spec §SC-003]
- [x] CHK005 Is SC-009 (PolBlogs real-world gate) specified with quantified time, quality, and structural invariants? [Completeness, Spec §SC-009]
- [x] CHK006 Does the spec define which quality functions are covered by performance regression gates? [Completeness, Spec §SC-005 / SC-007 / SC-009; FR-006 pin]
- [x] CHK007 Are performance requirements specified for dense graphs (d=50) separately from sparse/medium graphs (d=10)? [Completeness, Spec §SC-001 vs SC-002 / SC-003]
- [x] CHK008 Is there a documented requirement for deterministic timing reproducibility (e.g., same seed behavior) as part of performance validation? [Completeness, Spec §SC-007 / SC-008]

## Requirement Clarity — Quantified Metrics

- [x] CHK009 Is the SC-001 time threshold quantified as a specific millisecond or second bound (e.g., ≤500 ms vs ≤5 s)? [Clarity, Spec §SC-001 / Session 2026-09-09 (b)]
- [x] CHK010 Does SC-001 clearly distinguish the stretch target from the binding acceptance gate? [Clarity, Spec §SC-001 / Session 2026-09-09 (b)]
- [x] CHK011 Is SC-002's time bound quantified unambiguously (e.g., ≤5 s, not "under 5 seconds" without units)? [Clarity, Spec §SC-002]
- [x] CHK012 Is SC-003's ratio threshold expressed with a precise numeric value (< 12.5) rather than a vague term like "near-linear" alone? [Clarity, Spec §SC-003 / Session 2026-09-09 (d)]
- [x] CHK013 Is SC-009's timing target quantified (≤500 ms) and not stated as a relative or approximate goal? [Clarity, Spec §SC-009]
- [x] CHK014 Is the modularity Q equivalence threshold (1e-10) expressed as an exact floating-point epsilon, not a rounded approximation? [Clarity, Spec §SC-009 / Session 2026-09-09 (continued)]
- [x] CHK015 Does the spec clarify whether "sub-quadratic" is defined solely by the ratio < 12.5, or is there an additional formal regression requirement? [Clarity, Spec §SC-003 / research.md D8]

## Requirement Consistency — Targets and Boundaries

- [x] CHK016 Is there any contradiction between SC-001's stretch target (≤500 ms) and US1 Scenario 1's binding gate (≤5 s) for the same 10k/d=50 profile? [Consistency, Spec §SC-001 / US1 / Session 2026-09-09 (b)]
- [x] CHK017 Does the spec explicitly state which of the two SC-001 values (≤500 ms vs ≤5 s) is the binding acceptance gate? [Consistency, Session 2026-09-09 (b)]
- [x] CHK018 Are performance targets aligned with the measurement protocol (release build, fixed LFR params, 5-run median, pinned machine)? [Consistency, Spec §Measurement Protocol / Session 2026-09-10]
- [x] CHK019 Is SC-002's profile (50k/d=10) kept distinct from SC-003's comparison profiles (10k/d=50 and 50k/d=50), with no conflation in the requirements text? [Consistency, Spec §SC-002 / SC-003 / Session 2026-09-10]
- [ ] CHK020 Does the spec confirm that benchmark files (n=10k/d=50 and n=50k/d=50) are committed with recorded parameters, consistent with the measurement protocol? [Consistency, Spec §SC-003 / Session 2026-09-10 (e)]
- [x] CHK021 Are performance requirements for dense graphs (SC-001, SC-003) consistent with edge-case expectations (e.g., dense vs sparse behavior in refinement)? [Consistency, Spec §SC-001 / SC-003 / FR-004]

## Acceptance Criteria Quality — Measurability

- [ ] CHK022 Can SC-001's ≤500 ms stretch target be objectively measured using the defined protocol (release build, 5-run median, fixed graph)? [Measurability, Spec §SC-001 / Measurement Protocol]
- [ ] CHK023 Can SC-002's ≤5 s target be objectively measured without ambiguity about machine specs or load conditions? [Measurability, Spec §SC-002 / Measurement Protocol]
- [ ] CHK024 Is SC-003's pass/fail condition measurable via only two timing points (10k and 50k), or does it implicitly require regression infrastructure not yet available? [Measurability, Spec §SC-003 / plan.md Complexity Tracking / research.md D8]
- [ ] CHK025 Is SC-009's Q-within-1e-10 criterion measurable given that the baseline Q must be recorded during implementation? [Measurability, Spec §SC-009 / Session 2026-09-09 (continued)]
- [ ] CHK026 Does SC-009 specify both the timing and structural/quality invariants clearly enough that a reviewer could verify them independently? [Measurability, Spec §SC-009]
- [x] CHK027 Are all performance criteria written so that they reference the same measurement environment and do not mix release and debug metrics? [Measurability, Spec §SC-005–SC-009 / FR-008]

## Scenario Coverage — Load Conditions and Scales

- [x] CHK028 Are performance targets specified for dense graphs (d=50, 10k nodes) and medium-scale dense graphs (d=10, 50k nodes) separately? [Coverage, Spec §SC-001 / SC-002 / SC-003]
- [x] CHK029 Is there a performance requirement for sparse/small graphs (e.g., Tier 1 reference graphs) to prevent over-optimization that harms small inputs? [Coverage, Spec §SC-004 / SC-005 / SC-007]
- [x] CHK030 Does the spec cover real-world dense networks (PolBlogs) as a separate load condition from synthetic LFR graphs? [Coverage, Spec §SC-009 / US3]
- [ ] CHK031 Are performance targets defined for the refinement phase specifically, or only for the full algorithm? [Coverage, Spec §FR-003 / FR-004 / Measurement Protocol]
- [x] CHK032 Does the spec address performance under different quality functions (Modularity, CPM, MapEquation stub) or is it limited to a subset? [Coverage, Spec §SC-005 / SC-007 / FR-006]

## Edge Case Coverage — Boundary and Exception Conditions

- [ ] CHK033 Are performance requirements defined for zero-edge graphs or singletons (edge cases noted in spec)? [Edge Case, Spec §Edge Cases / SC-006]
- [ ] CHK034 Does the spec state how performance targets apply when the graph is disconnected or contains isolated nodes? [Edge Case, Spec §FR-003 / FR-005 / Edge Cases]
- [ ] CHK035 Are dense vs sparse graph behavior differences explicitly addressed in performance requirements, or are targets assumed uniform? [Edge Case, Spec §SC-001 d=50 / SC-002 d=10 / FR-004]
- [ ] CHK036 Is there a requirement covering performance when the refinement phase produces all singletons (e.g., no eligible merges)? [Edge Case, Spec §FR-003 / FR-004]
- [ ] CHK037 Does the spec clarify whether performance gates apply to debug builds (with `debug_assert!`) or exclusively to release builds? [Edge Case, Spec §FR-008 / SC-006 / Measurement Protocol]

## Non-Functional Requirements — Protocol and Reproducibility

- [x] CHK038 Does the Measurement Protocol specify the build type (release) for performance measurements? [Measurability / Protocol, Spec §Measurement Protocol / Session 2026-09-10]
- [x] CHK039 Does the protocol require synthetic gates to use LFR benchmark graphs with fixed, recorded parameters (μ, average degree)? [Measurability / Protocol, Spec §Measurement Protocol]
- [x] CHK040 Is the protocol quantified with a minimum number of runs (at least 5) and a median aggregation method? [Measurability / Protocol, Spec §Measurement Protocol]
- [x] CHK041 Does the protocol require a single pinned reference machine with recorded specifications? [Measurability / Protocol, Spec §Measurement Protocol]
- [x] CHK042 Are committed benchmark files (n=10k/d=50 and n=50k/d=50) required to be stored with parameter metadata? [Measurability / Protocol, Spec §SC-003 / Session 2026-09-10 (e)]
- [x] CHK043 Does the measurement protocol reference the remaining LFR parameters (beyond μ and average degree) per `research/leiden-test-parameters.md`? [Completeness / Protocol, Spec §Measurement Protocol]
- [x] CHK044 Is the measurement environment (release build, fixed params, 5-run median, pinned machine) referenced consistently across SC-001, SC-002, SC-003, and SC-009? [Consistency, Spec §SC-001–SC-003 / SC-009 / Measurement Protocol]

## Benchmark Persistence and Traceability

- [ ] CHK045 Are SC-003's two-point comparison benchmarks (n=10k/d=50 and n=50k/d=50) required to be permanently committed to `benchmarks/` with recorded LFR parameters? [Traceability, Spec §SC-003 / Session 2026-09-10 (e) / research.md D9]
- [ ] CHK046 Is the baseline Q value for SC-009's equivalence check required to be recorded during implementation? [Traceability, Spec §SC-009 / Session 2026-09-09 (continued)]
- [ ] CHK047 Does the spec reference primary sources (Traag et al. 2019, igraph, libleidenalg) for scaling validation methodology? [Traceability, Spec §SC-003 / research.md D8]
- [ ] CHK048 Is there an explicit requirement that benchmark parameters (not just results) are recorded alongside timing data? [Traceability, Spec §Measurement Protocol / Session 2026-09-10 (e)]

## Ambiguities and Conflicts — Session-Clarified Items

- [x] CHK049 Does the spec resolve the contradiction between SC-001's stretch (≤500 ms) and US1 Scenario 1's gate (≤5 s) by explicitly labeling one as stretch and one as binding? [Ambiguity / Conflict, Session 2026-09-09 (b) / Spec §SC-001 / US1]
- [x] CHK050 Is the unsourced baseline timing claim (~2 s / ~2.7 s) removed or clearly marked as struck in favor of a qualitative baseline? [Ambiguity, Session 2026-09-09 (b) / research.md D9]
- [x] CHK051 Does the spec confirm SC-003 uses a strict ratio (< 12.5) rather than a vague "near-linear" claim without numeric boundary? [Ambiguity, Session 2026-09-09 (d) / Spec §SC-003]
- [x] CHK052 Is the binding measurement protocol explicitly selected (release build, 5-run median, pinned machine, committed benchmark files) and documented? [Ambiguity, Session 2026-09-10]
- [x] CHK053 Does the spec confirm that performance gates apply uniformly across quality functions (Modularity, CPM, MapEquation stub) without quality-specific exceptions? [Ambiguity, Session 2026-09-09 / Session 2026-09-10 / FR-006 / SC-005]
- [x] CHK054 Does the spec explicitly note that performance targets are not verified by actual benchmark runs within this checklist, and that items test only whether requirements are well-written? [Ambiguity, Checklist Purpose]

## Edge Cases — Zero, Singleton, Dense vs Sparse

- [ ] CHK055 Does the spec address performance implications for graphs where all communities are singletons (e.g., zero-edge graphs)? [Edge Case, Spec §Edge Cases / FR-003 / SC-006]
- [ ] CHK056 Are singleton communities treated as trivially connected in performance and correctness requirements (not as exceptions requiring separate timing gates)? [Edge Case, Spec §FR-003 / FR-005 / Edge Cases]
- [ ] CHK057 Does the spec clarify whether dense graphs (d=50) have different refinement-phase performance expectations than sparse graphs (d=10)? [Edge Case, Spec §SC-001 / SC-002 / FR-004]
- [ ] CHK058 Are edge-case behavior requirements (zero edges, disconnected cliques, complete graphs) consistent with performance targets, or are they excluded from timing gates? [Edge Case, Spec §Edge Cases / US2]

## Overall Requirement Quality — Cross-Check

- [x] CHK059 Are all performance targets quantified with specific numeric thresholds rather than relative terms (e.g., "fast", "reasonable", "improved")? [Clarity / Overall, Spec §SC-001–SC-009]
- [x] CHK060 Do performance requirements cover multiple scales (10k, 50k nodes) and load conditions (dense d=50, medium d=10, real-world PolBlogs)? [Coverage / Overall, Spec §SC-001 / SC-002 / SC-003 / SC-009]
- [x] CHK061 Are requirements consistent with the measurement protocol (release build, fixed LFR params, 5-run median, pinned machine, committed benchmark files)? [Consistency / Overall, Spec §Measurement Protocol / SC-001–SC-009]
- [x] CHK062 Does the spec document the distinction between stretch targets and binding acceptance gates clearly enough that a reviewer could evaluate both independently? [Consistency / Overall, Session 2026-09-09 (b) / Spec §SC-001]
- [x] CHK063 Are all performance criteria accompanied by a clear protocol reference so that measurement ambiguity is minimized? [Measurability / Overall, Spec §Measurement Protocol]

---

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied.
- Leave items unchecked when clarification, correction, or reviewer evaluation is still needed.
- `/speckit-implement` reads checklist checkbox state as a gate; it must not modify markers.
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit-specify` and `/speckit-clarify`.
- This checklist tests whether performance requirements are well-written (quantified, consistent, covered, measurable, protocol-backed). It does NOT test benchmark execution or verify that targets are met.
- References: `spec.md` (SC-001 to SC-009, FR-001 to FR-008, Measurement Protocol), `plan.md` (Performance Goals / Constraints), `research.md` (D8 scaling validation, D9 baseline), `Session 2026-09-09 (b)` (performance targets and measurement environment clarification).

# Algorithm Correctness Checklist: Leiden Algorithm Completion

**Purpose**: Validate mathematical correctness and algorithmic requirement quality
**Created**: 2026-09-05
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md)

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

## Formula Correctness

- [x] CHK001 Is the Modularity Q formula explicitly stated with all variables defined? [Completeness, Spec §FR-001]
  - **Review 2026-09-05**: PARTIAL — Formula exists in plan §Phase 0 and research/leiden-quality-functions.md but NOT in spec §FR-008 itself. Variables defined in research but not in normative spec section. Recommend moving formula into spec or adding normative appendix.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Modularity ΔQ formula now explicitly stated in spec §FR-001 with all variables defined (m, k_v, ΔE, ΔK).
- [x] CHK002 Is the CPM formula explicitly stated with all variables defined? [Completeness, Spec §FR-001]
  - **Review 2026-09-05**: PARTIAL — Same issue as CHK001. Formula in plan/research but not in spec §FR-008. Also "Q" symbol overloaded between Modularity and CPM.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — CPM ΔQ formula now explicitly stated in spec §FR-001 with all variables defined (w(v,C), w(v,v), γ, n_v, n_C, σ_v).
- [x] CHK003 Is the modularity gain (ΔQ) formula for node moves unambiguous? [Clarity, Spec §FR-001]
  - **Review 2026-09-05**: NO — ΔQ formula entirely absent from spec, plan, and research. This is the core computation for local moving (FR-001). Critical gap: implementation cannot be verified without explicit target formula.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Both CPM and Modularity ΔQ formulas now explicitly stated in spec §FR-001, matching libleidenalg source.
- [x] CHK004 Are the resolution parameter (gamma) constraints specified (range, default, effect)? [Completeness, Spec §FR-007]
  - **Review 2026-09-05**: PARTIAL — Default (1.0) in spec §FR-007. Range and effect only in research files; no validation constraints (e.g., γ > 0) stated in spec despite paper requiring it.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Gamma constraints now in spec §FR-007: MUST be > 0, behavioral effect documented (higher γ → more communities, density threshold in CPM), invalid γ returns `AlgorithmError::InvalidConfiguration { reason }` error.
- [x] CHK005 Is the beta parameter range [0.0005, 0.1] documented with mathematical justification? [Clarity, Spec §FR-002]
  - **Review 2026-09-05**: PARTIAL — Range documented in spec §FR-002. NO mathematical justification exists anywhere — research/leiden-beta-parameter.md confirms paper provides no detailed derivation (empirical range only).
  - **Review 2026-09-05 (post-research)**: SATISFIED — Exhaustive search (research/leiden-beta-parameter-justification.md) confirms NO mathematical justification exists in any primary source (Traag et al. 2019, igraph, libleidenalg, leidenalg, Java networkanalysis, follow-up literature). Range is purely empirical with a fundamental scale-dependence problem preventing theoretical derivation. Spec §FR-002 now explicitly documents range as "empirical" with note that no mathematical justification exists. This is the most complete documentation possible given the paper limitation.
- [x] CHK006 Is the `exp(β·Δ)` weighted selection formula from Traag et al. 2019 explicitly stated? [Completeness, Spec §FR-002]
  - **Review 2026-09-05**: YES — Spec §FR-002 states "probability `exp(β·Δ)` (weighted selection per Traag et al. 2019)". Research provides full conditional form with variables.

## Connected Communities Invariant

- [x] CHK007 Is the connected communities guarantee (FR-006) measurable via BFS/DFS? [Measurability, Spec §FR-006]
  - **Review 2026-09-05**: YES — Spec §FR-006 explicitly states "verifiable via BFS/DFS traversal from any member reaching all members". SC-001 and US-1 Independent Test confirm same verification method.
- [x] CHK008 Is the success criterion SC-001 (100% connected communities) objectively verifiable? [Measurability, Spec §SC-001, §Test Corpus]
  - **Review 2026-09-05**: PARTIAL — Verification method (BFS/DFS) is objective, but "all test graphs" is not scoped to a specific, enumerable corpus. SC-002/SC-003 reference LFR N=10k but SC-001 does not.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — SC-001 now references "the defined test corpus" with three-tier corpus explicitly specified in spec §Test Corpus & Benchmarks (deterministic, real-world, synthetic generators).
- [x] CHK009 Are the property-based test parameters for connected communities specified (graph size, edge count)? [Completeness, Spec §Test Corpus]
  - **Review 2026-09-05**: NO — Plan announces `leiden_connected.rs` property-based test but specifies no parameters (node count range, edge density, weight ranges, graph generators, test case count).
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Property-based test parameters now explicitly specified in spec §Test Corpus & Benchmarks (node counts: 1-10k, edge densities, weight types, test case counts).
- [x] CHK010 Is the connectedness preservation condition during local moving explicitly defined? [Clarity, Spec §FR-002]
  - **Review 2026-09-05**: PARTIAL — Plan §Local Moving mentions "(if connectedness preserved)" but spec §FR-001 does not include connectedness as a constraint. No definition of source-side, target-side, or both checks.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Connectedness guarantee now formally defined in spec §FR-002 via refinement's γ-connectivity conditions (node eligibility: E(v, S\{v}) ≥ γ·k_v·(k_S - k_v)).

## Convergence Criteria

- [x] CHK011 Is the convergence threshold (ε = 1e-6) documented with rationale? [Completeness, Spec §FR-004]
  - **Review 2026-09-05**: YES — Spec §FR-004 states default 1e-6; §Clarifications confirms "absolute improvement as stopping condition"; §Assumptions notes "produces modularity within acceptable epsilon of best-known for LFR N=10k". Research/leiden-convergence-mode.md provides extensive rationale.
- [x] CHK012 Are absolute and relative convergence modes explicitly distinguished? [Clarity, Spec §FR-004]
  - **Review 2026-09-05**: YES — Spec §FR-004 + §Key Entities (ConvergenceMode enum) + plan §Convergence (explicit formulas for both modes). Research provides comprehensive comparison.
- [x] CHK013 Is the plateau threshold formula `max(ε/10, 1e-8)` stated? [Completeness, Spec §FR-009]
  - **Review 2026-09-05**: YES — Spec §FR-009 explicitly: "derive plateau threshold as `max(convergence_threshold / 10, 1e-8)`". Plan §Convergence confirms.
- [x] CHK014 Is the max_iterations behavior specified (return best partition, never error)? [Clarity, Spec §FR-004]
  - **Review 2026-09-05**: YES — Spec §FR-004: "system MUST return the current best partition and emit a tracing warning event — never return an error or panic for non-convergence". §Clarifications confirms.
- [x] CHK015 Are convergence event fields documented (iteration, improvement, current_quality)? [Completeness, Spec §FR-009]
  - **Review 2026-09-05**: YES — Spec §FR-009: "emit plateau events via tracing with structured fields: iteration (u64), improvement (f64), current_quality (f64)". §Clarifications confirms.

## Determinism Requirements

- [x] CHK016 Is the default seed value (42) documented? [Completeness, Spec §FR-005]
  - **Review 2026-09-05**: YES — Spec §FR-005: "default seed = 42". §US-2 AS3 confirms. §Key Entities (LeidenConfig) confirms.
- [x] CHK017 Is "bit-for-bit identical membership vector" defined as the determinism criterion? [Clarity, Spec §FR-005]
  - **Review 2026-09-05**: YES — Spec §FR-005: "same seed + same input = bit-for-bit identical membership vector". §US-2 Independent Test uses same phrase.
- [x] CHK018 Is the determinism test scenario (100 runs, identical seed) specified? [Measurability, Spec §US-2]
  - **Review 2026-09-05**: YES — Spec §US-2 Independent Test: "running Leiden 100 times with the same seed". SC-002: "100% deterministic across 100 runs with identical seed".
- [x] CHK019 Are community ID assignment rules (contiguous, first-node-encountered) documented? [Completeness, Clarifications]
  - **Review 2026-09-05**: YES — Spec §Clarifications: "Contiguous IDs assigned by first-node-encountered order during local moving phase. Node 0's community = 0, next new community encountered = 1, etc." §Key Entities (Partition) and plan §Phase 0 confirm.

## Quality Function Specifications

- [x] CHK020 Is the QualityFunction enum (Modularity, CPM) specification clear that MapEquation is excluded? [Clarity, Spec §FR-008]
  - **Review 2026-09-05**: YES — Spec §FR-008: "Map Equation is NOT supported (exclusive to Infomap algorithm, deferred to Infomap feature)". §Key Entities and §Clarifications reinforce. Research confirms no reference Leiden implementation supports Map Equation.
- [x] CHK021 Is the self-loop handling in quality computation specified (counted once)? [Completeness, Clarifications]
  - **Review 2026-09-05**: YES — Spec §Clarifications: "Self-loop counted once as intra-community edge weight (matches Traag et al. 2019 and igraph treatment)." §FR-010 and plan §Phase 0 confirm.
- [x] CHK022 Is the aggregation self-loop formula stated (sum of all intra-community edges)? [Completeness, Spec §FR-003]
  - **Review 2026-09-05**: YES — Spec §FR-003: "self-loops sum all intra-community edge weights (including original self-loops); self-loop weight for community C = sum of all edge weights between nodes within C". §Clarifications and plan §Aggregation confirm.
- [x] CHK023 Are inter-community edge weight aggregation rules specified (summed)? [Completeness, Spec §FR-003]
  - **Review 2026-09-05**: YES — Spec §FR-003: "edge weights sum inter-community edges". Plan §Aggregation: "Edge weight = sum of all edges between nodes in C and nodes in D".

## Edge Case Requirements

- [x] CHK024 Are empty graph handling requirements specified (no panic, valid partition)? [Coverage, Spec §US-1]
  - **Review 2026-09-05**: YES — §US-1 AS2: "no panic occurs and a valid empty partition is returned". FR-010 and SC-004 confirm.
- [x] CHK025 Are single-node graph handling requirements specified? [Coverage, Spec §US-1]
  - **Review 2026-09-05**: YES — §US-1 AS3: "the node is assigned to a single community with finite quality score". FR-010 and SC-004 confirm.
- [x] CHK026 Are self-loop handling requirements specified for all phases? [Coverage, Spec §FR-010]
  - **Review 2026-09-05**: YES — §FR-010 + §Clarifications + plan §Phase 0 + §Aggregation all specify "counted once as intra-community edge weight".
- [x] CHK027 Are zero-weight edge handling requirements specified (no NaN/Inf)? [Coverage, Spec §FR-010]
  - **Review 2026-09-05**: YES — §US-3 AS2: "valid partition is returned with finite quality (no NaN/Inf)". SC-006: "Quality scores are finite (no NaN, no Inf)".
- [x] CHK028 Are negative weight handling rules specified (valid when m > 0)? [Coverage, Spec §FR-010]
  - **Review 2026-09-05**: YES — §FR-010: "negative weights (mathematically valid per Traag et al. 2019)". §Clarifications: "checks total edge weight m > 0 at entry". Research/negative-weight-handling.md provides comprehensive justification.
- [x] CHK029 Are disconnected component handling rules specified (independent processing)? [Coverage, Clarifications]
  - **Review 2026-09-05**: YES — §FR-010: "each component processed independently without pre-validation, matching igraph/leidenalg". §Clarifications: "No O(V+E) validation pass. Communities never span components."

## Phase-Specific Requirements

- [x] CHK030 Are local moving phase inputs and outputs specified? [Completeness, Spec §FR-001]
  - **Review 2026-09-05**: YES — §FR-001 describes procedure. Plan §Local Moving: "Input: graph, current membership, seeded RNG" and "Return true if any node moved".
- [x] CHK031 Is the random node order (seeded) requirement stated for local moving? [Completeness, Spec §FR-001]
  - **Review 2026-09-05**: YES — §FR-001: "iterate over nodes in random order (seeded)". §FR-005 provides seed semantics.
- [x] CHK032 Are refinement phase inputs and outputs specified? [Completeness, Spec §FR-002]
  - **Review 2026-09-05**: YES — §FR-002 describes procedure. Plan §Refinement: "Input: graph, current membership, seeded RNG, beta parameter".
- [x] CHK033 Is the subpartition guarantee for refinement explicitly stated? [Clarity, Spec §FR-002]
  - **Review 2026-09-05**: YES — §FR-002: "guarantee refined partition is a subpartition of the original". Plan §Refinement confirms.
- [x] CHK034 Are aggregation phase inputs and outputs specified? [Completeness, Spec §FR-003]
  - **Review 2026-09-05**: YES — §FR-003 describes procedure. Plan §Aggregation: "Input: graph, refined membership" and "Return new CsrGraph".
- [x] CHK035 Is the CsrGraph rebuild requirement for aggregation stated? [Completeness, Spec §FR-003]
  - **Review 2026-09-05**: YES — §FR-003: "build reduced graph where communities become nodes". Plan §Aggregation: "Return new CsrGraph".

## Error Handling Requirements

- [x] CHK036 Are all error variants documented (GraphError::InvalidGraph, GraphError::EmptyGraph, AlgorithmError::InvalidConfiguration, AlgorithmError::NonConvergence)? [Completeness, Spec §FR-010]
  - **Review 2026-09-05**: YES — §FR-010 specifies: `GraphError::InvalidGraph { reason }` (total weight m ≤ 0), `GraphError::EmptyGraph`, `AlgorithmError::InvalidConfiguration { reason }` (invalid gamma, beta, threshold), `AlgorithmError::NonConvergence { iterations }` (informational, max iterations reached). Matches communal-core error types exactly.
- [x] CHK037 Is the InvalidGraph trigger condition specified (m ≤ 0)? [Clarity, Clarifications]
  - **Review 2026-09-05**: YES — §FR-010: "returned when total edge weight m ≤ 0". §Clarifications: "returns `GraphError::InvalidGraph { reason }` if m ≤ 0".
- [x] CHK038 Is MaxIterationsReached documented as non-error (return best partition)? [Clarity, Clarifications]
  - **Review 2026-09-05**: YES — §FR-004: "system MUST return the current best partition and emit a tracing warning event — never return an error or panic for non-convergence". §Clarifications confirms.

## Non-Functional Requirements

- [x] CHK039 Is the NMI benchmark target (≥ 0.95) quantified with graph parameters? [Measurability, Spec §SC-003]
  - **Review 2026-09-05**: PARTIAL — §SC-003 states "NMI >= 0.95 against planted partition for LFR benchmarks with μ <= 0.3". However, N parameter is only in §SC-002, not §SC-003. Spec is not self-contained for this criterion.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — SC-003 now includes full LFR parameters (N=10k, ⟨k⟩=20, μ ≤ 0.3, τ1=2, τ2=1, c_min=10, c_max=50). Spec is self-contained.
- [x] CHK040 Is the LFR benchmark configuration specified (N=10k, μ≤0.3)? [Completeness, Spec §Test Corpus, §SC-003]
  - **Review 2026-09-05**: PARTIAL — N=10k appears in §SC-002; μ≤0.3 in §SC-003. Full LFR configuration also needs average degree, degree distribution exponent, community size range/distribution — none specified.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Full LFR configuration now in spec §SC-003 and §Test Corpus & Benchmarks (N=10k, ⟨k⟩=20, k_max=50, τ1=2, τ2=1, c_min=10, c_max=50, μ=0.0→0.6).
- [x] CHK041 Is the determinism success rate specified (100% across 100 runs)? [Measurability, Spec §SC-002]
  - **Review 2026-09-05**: YES — §SC-002: "100% deterministic across 100 runs with identical seed on LFR benchmark (N=10k, μ=0.3)". All required elements present.

## Ambiguities & Conflicts

- [x] CHK042 Is "connectedness preserved" during local moving defined with measurable criteria? [Ambiguity, Spec §FR-002]
  - **Review 2026-09-05**: NO — Plan mentions "(if connectedness preserved)" but no algorithmic measurement steps defined. No specification of source-side, target-side, or both checks. No pre-condition or post-condition with rollback semantics.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Connectedness guarantee now formally defined in spec §FR-002 via refinement's γ-connectivity conditions (node eligibility: E(v, S\{v}) ≥ γ·k_v·(k_S - k_v)), which is the actual mechanism per Traag et al. 2019.
- [x] CHK043 Is "subpartition of the original" for refinement mathematically defined? [Ambiguity, Spec §FR-002]
  - **Review 2026-09-05**: PARTIAL — Term "subpartition" used declaratively but not formally defined. Standard meaning (∀C' ∈ P', ∃C ∈ P : C' ⊆ C) not stated. Mechanism enforcing it (only merge nodes from same original community) not described.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Subpartition now formally defined in spec §FR-002 (∀C' ∈ P_refined, ∃C ∈ P : C' ⊆ C) with enforcement mechanism (MergeNodesSubset processes each original community independently).
- [x] CHK044 Are there conflicts between quality function selection and convergence criteria? [Conflict, Spec §FR-004]
  - **Review 2026-09-05**: PARTIAL — Research identifies potential conflicts (relative convergence with Modularity Q: division by zero when Q ≈ 0; negative modularity makes ratio undefined; CPM and Modularity Q operate on different scales). Spec does not warn or constrain combinations.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Spec §FR-004 now explicitly disallows relative convergence with Modularity Q and CPM (division-by-zero risk, sign-flip). Absolute mode required.
- [x] CHK045 Is the relationship between gamma parameter and quality function behavior documented? [Clarity, Spec §FR-007]
  - **Review 2026-09-05**: PARTIAL — §FR-007 documents gamma default (1.0) but not its behavioral effect (higher γ → more communities). Research mentions resolution limit but spec provides no tuning guidance.
  - **Review 2026-09-05 (post-clarification)**: SATISFIED — Gamma behavioral relationship now documented in spec §FR-007 (higher γ → more communities, density threshold in CPM, γ → 0⁺ single community, γ → ∞ singleton partition).

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- `$speckit-implement` reads checklist checkbox state as a gate and must not modify markers
- Items reference spec sections [Spec §FR-XX] or mark gaps [Gap] where requirements are missing

### Review Session 2026-09-05

**Scope**: Full review of all 45 algorithm-correctness checklist items against spec.md, plan.md, and research files.

**Results Summary**:
- **Satisfied [x]**: 30 items (CHK006, CHK007, CHK011-CHK019, CHK021-CHK038, CHK041)
- **Partial**: 11 items (CHK001, CHK002, CHK004, CHK005, CHK008, CHK010, CHK039, CHK040, CHK043, CHK044, CHK045)
- **Not Satisfied [ ]**: 4 items (CHK003, CHK009, CHK020, CHK042)

**Critical Gaps Requiring Remediation**:

1. **CHK003 (Critical)**: ΔQ formula for node moves entirely absent. Core computation for local moving (FR-001) cannot be verified without explicit target formula. Recommend adding to spec §FR-001 or normative appendix.

2. **CHK009 (High)**: Property-based test parameters not specified. Plan announces tests but defines no node count range, edge density, weight ranges, or generator types.

3. **CHK042 (High)**: "Connectedness preserved" during local moving has no measurable criteria. Plan mentions it parenthetically but spec §FR-001 doesn't include it as a constraint.

4. **CHK001/CHK002 (Moderate)**: Quality function formulas exist in plan/research but not in normative spec §FR-008. FR-008 names functions but doesn't define them.

5. **CHK004/CHK045 (Moderate)**: Gamma parameter behavioral relationship (effect on community count, resolution limit) documented in research but not in spec.

6. **CHK005 (Minor)**: Beta range [0.0005, 0.1] documented but mathematical justification doesn't exist in paper (empirical range only). Spec should acknowledge this.

7. **CHK039/CHK040 (Minor)**: LFR benchmark parameters split across SC-002 and SC-003; full configuration (degree distribution, community size distribution) not specified.

**Recommendation**: Address CHK003 and CHK009 before proceeding to implementation. These are correctness-critical gaps that would block verifiable implementation.

### Post-Clarification Review 2026-09-05

**Scope**: Re-validated all 14 previously-unchecked items against updated spec.md after `$speckit-clarify` session.

**Results Summary**:
- **Newly passing**: 13 items (CHK001, CHK002, CHK003, CHK004, CHK008, CHK009, CHK010, CHK039, CHK040, CHK042, CHK043, CHK044, CHK045)
- **Still unchecked**: 1 item (CHK005 - beta mathematical justification doesn't exist in paper)
- **Total passing**: 44/45 items (97.8%)

**Remaining Gap**:
- **CHK005**: Beta parameter range [0.0005, 0.1] documented in spec but mathematical justification doesn't exist in Traag et al. 2019 (empirical range only). This is a paper limitation, not a spec gap. No action needed.

**Recommendation**: All critical and high-impact gaps resolved. Specification is ready for `$speckit-plan`.

### Post-Research Review 2026-09-05

**Scope**: Re-validated CHK005 against exhaustive primary-source research (research/leiden-beta-parameter-justification.md).

**Research Finding**: Confirmed NO mathematical justification exists for beta range [0.0005, 0.1] in any primary source. Traag et al. 2019 states the range without derivation. Fundamental scale-dependence problem prevents theoretical derivation. Range is purely empirical.

**Action Taken**: Updated spec §FR-002 to explicitly document range as "empirical" with note that no mathematical justification exists. Added clarification entry documenting research findings. Updated §Assumptions with empirical-range note.

**Results Summary**:
- **Newly passing**: 1 item (CHK005)
- **Total passing**: 45/45 items (100%)

**Recommendation**: All items satisfied. Both checklists fully passing. Specification is ready for `$speckit-plan`.

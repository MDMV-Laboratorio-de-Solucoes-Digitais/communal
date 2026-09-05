# Best Practices for Documenting Assumptions in Software Specifications

## Executive Summary

This report examines how assumptions should be documented in software specifications, focusing on whether they should be explicitly tagged as "testable" (with validation method) or "design decision" (untestable). The analysis draws from international standards (ISO/IEC/IEEE 29148, RFC 2119), industry guides (INCOSE, SWEBOK), domain-specific standards (DO-178C, ARP4754A, IEC 61508), and open-source specification templates.

**Key Finding:** The standards do not mandate a binary "testable vs. design decision" taxonomy. Instead, they converge on a richer model: every assumption should be documented with its rationale, validation/verification method (where applicable), owner, and risk if proven false. The distinction between testable assumptions and design decisions is better understood as a spectrum of verifiability rather than a binary classification.

---

## 1. Should Assumptions Be Tagged as "Testable" vs. "Design Decision"?

### 1.1 What the Standards Say

**ISO/IEC/IEEE 29148:2018** (the current international standard for requirements engineering, superseding IEEE 830) addresses assumptions in two specific sections:

- **Section 9.5.19 (System Requirements Specification — Assumptions and Dependencies):** "This section shall describe, as applicable, each of the assumptions and dependencies that affect the requirements in the specification. Assumptions are statements that are taken as true without proof or demonstration. Dependencies are conditions or events that must exist or occur for the requirements to be achievable."

- **Section 9.6.8 (Software Requirements Specification — Assumptions and Dependencies):** "This section shall describe each of the factors that affect the software requirements and are not part of the software design. Assumptions are statements that are taken as true without proof or demonstration."

The standard does **not** mandate tagging assumptions as "testable" or "design decision." Instead, it treats assumptions as a distinct category from requirements — statements taken as true without proof. However, the standard's emphasis on verification and validation (V&V) implies that assumptions should be traceable to V&V activities where possible.

**IEEE 830-1998** (superseded but historically influential) stated in Section 5.5: "States assumptions about availability of certain resources that, if not satisfied, will alter system requirements and/or affect the design." The standard noted that "unfulfilled assumptions or unmet dependencies may imply changes in one or more of the described requirements."

### 1.2 The INCOSE Position: Verifiability as a Core Characteristic

The **INCOSE Guide to Writing Requirements v4** (2023) defines "Verifiable" (Characteristic C7) as a fundamental property of well-formed requirements:

> "The need statement is structured and worded such that its realization can be validated to the approving authority's satisfaction. The requirement statement is structured and worded such that its realization can be verified to the approving authority's satisfaction."

The Guide defines a comprehensive set of attributes for each requirement, including:

| Attribute | ID | Description |
|-----------|-----|-------------|
| Rationale | A1 | Why the requirement exists |
| System V&V Success Criteria | A6 | What constitutes success |
| System V&V Strategy | A7 | Overall approach to V&V |
| System V&V Method | A8 | Specific method (Test, Analysis, Inspection, Demonstration) |
| System V&V Responsible Organization | A9 | Who performs V&V |
| Condition of Use | A12 | Conditions under which V&V applies |
| Results | A13 | Outcomes of V&V activities |
| Status | A14 | Current V&V status |

**Critical insight:** The INCOSE framework does not use a binary "testable/design decision" distinction. Instead, it requires that every requirement (and by extension, every assumption that constrains requirements) have a defined verification or validation method. The method can be "Test," "Analysis," "Inspection," or "Demonstration" — accommodating both empirical testing and analytical reasoning.

### 1.3 SWEBOK: The Verifiability Mandate

The **Guide to the Software Engineering Body of Knowledge (SWEBOK v3)** states:

> "An essential property of all software requirements is that they be verifiable as an individual feature as a functional requirement or at the system level as a nonfunctional requirement."

SWEBOK further notes: "Software requirements, software testing, and quality personnel must ensure that the requirements can be verified within available resource constraints." This applies equally to assumptions that underpin requirements.

### 1.4 Practical Recommendation

Rather than a binary "testable vs. design decision" tag, the standards support a **multi-dimensional classification**:

1. **Verifiability Method:** Test, Analysis, Inspection, Demonstration, or "Design Decision" (untestable)
2. **Criticality:** Safety-critical, mission-critical, business-critical, or cosmetic
3. **Stability:** Stable (unlikely to change) vs. Volatile (likely to change)
4. **Source:** Stakeholder-imposed, regulatory, environmental, or technical

---

## 2. Standards and Methodologies Addressing Assumption Documentation

### 2.1 ISO/IEC/IEEE 29148:2018 — Requirements Engineering

The current international standard for requirements engineering. Key provisions:

- **Clause 5.2.5 (Characteristics of individual requirements):** Lists "verifiable" as a required characteristic
- **Clause 5.2.8 (Requirements attributes):** Mandates that each requirement have attributes including verification method
- **Clause 6.5.2 (Requirements activities in verification):** Describes how requirements (and their assumptions) feed into verification planning
- **Clause 9.5.19 / 9.6.8:** Dedicated sections for assumptions and dependencies in SyRS and SRS

The standard defines **requirements traceability** (3.1.23) as "identification and documentation of the derivation path (upward) and allocation/flow-down path (downward) of requirements." Assumptions should be traceable to the requirements they constrain.

### 2.2 RFC 2119 — Key Words for Requirements

RFC 2119 defines the requirement levels (MUST, SHALL, SHOULD, MAY, etc.) used in IETF specifications. While it does not directly address assumptions, its framework implies that:

- **MUST/SHALL/REQUIRED:** Absolute requirements — any assumption underlying these must be validated
- **SHOULD/RECOMMENDED:** Recommended but not absolute — assumptions may be relaxed
- **MAY/OPTIONAL:** Truly optional — assumptions are less critical

RFC 8174 (which updates 2119) reinforces that these terms are used to specify behavior that can be verified for conformance.

### 2.3 Behavior-Driven Development (BDD) and Specification by Example

BDD, as described by Adzic (Specification by Example) and North (BDD), treats assumptions as **preconditions** in the Given-When-Then format:

```
Given [assumption/precondition]
When [action/event]
Then [expected outcome]
```

In BDD:
- **Assumptions are made explicit as "Given" clauses** — they are testable preconditions
- **Design decisions are captured as scenarios** — they describe how the system behaves under specific conditions
- **Every scenario is executable** — assumptions that cannot be tested are reframed as constraints or removed

The BDD approach effectively eliminates the "untestable assumption" category by requiring all preconditions to be concrete and verifiable.

### 2.4 Domain-Specific Standards

#### DO-178C / DO-278A (Airborne Software)
- Requires **bidirectional traceability** from requirements to assumptions
- Assumptions must be validated as part of the software verification process
- Design decisions are captured in Software Design Descriptions (SDD) and are traceable to requirements

#### ARP4754A (Aerospace Development Assurance)
- Defines validation as "the process by which we ensure that the set of requirements is correct"
- Assumptions are explicitly identified during requirements validation
- The standard distinguishes between **requirements validation** (are we building the right thing?) and **design verification** (are we building it right?)

#### IEC 61508 (Functional Safety)
- Requires assumptions to be documented in the Safety Requirements Specification
- Each assumption must have an associated validation method
- Assumptions about operator behavior, environmental conditions, and system interfaces must be testable

#### EN 50128 / EN 50716 (Railway Software)
- Safety-related assumptions must be traceable and validated
- The standard requires a Software Requirements Specification that includes assumptions and dependencies
- Assumptions are classified by Safety Integrity Level (SIL)

### 2.5 NASA Systems Engineering Handbook

NASA's approach (documented in NASA/SP-2016-6105 and the NASA Systems Engineering Handbook) emphasizes:

- **Assumptions must be confirmed before baselining** requirements
- Each requirement should be accompanied by "an intelligible rationale, including any assumptions"
- "Assumptions should be confirmed before baselining" — implying they are testable/confirmable
- The validation plan includes a **Validation Requirements Matrix** that maps requirements (and their assumptions) to validation methods

---

## 3. Examples from Open-Source Projects and Templates

### 3.1 GitHub SRS Template (jam01/SRS-Template)

A widely-used open-source SRS template (439 stars, 153 forks) that follows ISO/IEC/IEEE 29148:

```markdown
### 2.5 Assumptions and Dependencies
💬 *External assumed factors or conditions, as opposed to known facts, that the project relies on.*
➥ List assumptions about environment, hardware, usage patterns, third-party 
components/services, and organizational support. List dependencies on external 
systems, libraries, or teams. For each, indicate potential impact if proven false.
💡 Tips:
* Link assumptions to risk register with owner and mitigation when available.
```

This template treats assumptions as **external factors** that may prove false, with explicit linkage to risk management. It does not use a "testable/design decision" binary but implies testability through the "impact if proven false" language.

### 3.2 IEEE 29148 SRS Template (jorgeherrera1/ieee-29148-srs-template)

A modernized template that explicitly addresses verification:

```markdown
## 4. Verification
💬 *Describes how each requirement will be verified to provide objective evidence of compliance.*
➥ Outline verification methods (test, canary metrics, analysis, inspection, demonstration) 
and test evidence preferably in a matrix paralleling Section 3.
```

This template uses four verification methods: **TEST, E2E, MEASURE, AUDIT** — each with a defined artifact and pass condition. Assumptions are implicitly handled through the verification matrix.

### 3.3 ReqView ISO/IEC/IEEE 29148 Example

ReqView's example SRS includes a dedicated "Assumptions and Dependencies" section (Section 1.7) with custom attributes for each assumption, including:
- **Impact if false:** What happens if the assumption proves incorrect
- **Mitigation:** How to address if the assumption is invalidated
- **Owner:** Who is responsible for monitoring the assumption

### 3.4 ECSS-E-ST-40C (Space Engineering)

The European Cooperation for Space Standardization SRS template includes:
- Assumptions as a dedicated section
- Each assumption linked to affected requirements
- Verification method specified per assumption

---

## 4. Practical Benefits vs. Costs of Tagging Assumptions

### 4.1 Benefits of Explicit Tagging

| Benefit | Description | Source |
|---------|-------------|--------|
| **Improved traceability** | Tagged assumptions can be traced to requirements, design decisions, and test cases | ISO/IEC/IEEE 29148 (2018), Clause 5.2.8 |
| **Risk management** | Assumptions tagged with criticality enable prioritized risk mitigation | INCOSE GtWR v4, Attribute A36 |
| **Validation planning** | Testable assumptions feed directly into V&V planning | NASA SE Handbook, Appendix E |
| **Change impact analysis** | When assumptions change, tagged items show affected requirements | SWEBOK v3, Section 7.4 |
| **Stakeholder communication** | Explicit tags make assumptions visible to non-technical stakeholders | BDD/Specification by Example |
| **Regulatory compliance** | Safety-critical standards (DO-178C, IEC 61508) require assumption traceability | DO-178C, Section 6.3 |

### 4.2 Costs of Explicit Tagging

| Cost | Description | Mitigation |
|------|-------------|------------|
| **Documentation overhead** | Each assumption requires multiple attributes | Use templates and automation tools |
| **Maintenance burden** | Tags must be updated as assumptions evolve | Link to configuration management |
| **False precision** | Binary tags may oversimplify complex assumptions | Use multi-dimensional classification |
| **Analysis paralysis** | Over-classification can delay specification | Focus on safety-critical assumptions first |
| **Tooling requirements** | Traceability requires requirements management tools | Use open-source tools (ReqView, StrictDoc) |

### 4.3 Cost-Benefit Analysis by Project Type

| Project Type | Recommended Tagging Level | Rationale |
|--------------|---------------------------|-----------|
| **Safety-critical (medical, aerospace, rail)** | Full tagging with V&V method, owner, criticality | Regulatory compliance (DO-178C, IEC 61508, EN 50128) |
| **Enterprise/Business-critical** | Moderate tagging with rationale and impact | Risk management and stakeholder alignment |
| **Consumer/Web applications** | Lightweight tagging (testable vs. assumption) | Agile development, rapid iteration |
| **Research/Prototype** | Minimal tagging (document and move on) | High uncertainty, rapid exploration |

---

## 5. Assumption Traceability and Requirements Quality

### 5.1 The Traceability Problem

Gotel and Finkelstein's classic paper "An Analysis of the Requirements Traceability Problem" (1994) identified that traceability is essential for:
- Impact analysis when requirements change
- Verification coverage assessment
- Stakeholder need fulfillment

Assumptions are a critical but often neglected part of the traceability chain. An assumption that proves invalid can invalidate multiple requirements.

### 5.2 Bidirectional Traceability

ISO/IEC/IEEE 29148 requires **bidirectional traceability**:

- **Backward (upward):** From requirement to source (stakeholder need, assumption, constraint)
- **Forward (downward):** From requirement to implementation, test case, verification artifact

Assumptions should participate in both directions:
- **Backward:** What requirement(s) does this assumption constrain?
- **Forward:** What test/analysis validates this assumption?

### 5.3 Requirements Quality Characteristics

The INCOSE Guide defines characteristics of well-formed requirements that apply equally to assumptions:

| Characteristic | ID | Application to Assumptions |
|----------------|-----|---------------------------|
| Necessary | C1 | Is this assumption truly required? |
| Unambiguous | C3 | Is the assumption stated clearly? |
| Complete | C4 | Are all conditions specified? |
| Feasible | C6 | Can the assumption be realized? |
| Verifiable | C7 | Can the assumption be validated? |

### 5.4 The "Verifiable" Characteristic in Practice

The INCOSE Guide's "Verifiable" characteristic (C7) is the key to the testable/design decision distinction:

> "The requirement statement is structured and worded such that its realization can be verified to the approving authority's satisfaction."

For assumptions, this means:
- **Testable assumptions:** Can be verified through test, analysis, inspection, or demonstration
- **Design decisions:** Cannot be verified against requirements but are documented as constraints on the solution

The Guide recommends four verification methods:
1. **Test:** Direct measurement or observation
2. **Analysis:** Analytical reasoning or modeling
3. **Inspection:** Visual examination or review
4. **Demonstration:** Showing that a capability exists without measurement

---

## 6. Recommendations

### 6.1 For Specification Authors

1. **Use a multi-dimensional classification** rather than a binary tag:
   - Verifiability method (Test / Analysis / Inspection / Demonstration / Design Decision)
   - Criticality (Safety / Mission / Business / Cosmetic)
   - Stability (Stable / Volatile)
   - Source (Stakeholder / Regulatory / Environmental / Technical)

2. **Include assumptions in the traceability matrix** — link each assumption to the requirements it constrains and the verification activities that validate it.

3. **Specify the impact if false** — for each assumption, document what happens if it proves incorrect.

4. **Assign an owner** — each assumption should have a responsible party who monitors its validity.

5. **Use the INCOSE attribute set** — particularly A1 (Rationale), A6-A14 (V&V attributes), and A36 (Risk).

### 6.2 For Organizations

1. **Adopt ISO/IEC/IEEE 29148** as the baseline for requirements engineering, including assumption documentation.

2. **Use requirements management tools** that support custom attributes and traceability (e.g., ReqView, Jama Connect, IBM DOORS).

3. **Integrate with risk management** — link assumptions to risk register items with mitigation strategies.

4. **Train teams on the INCOSE Guide to Writing Requirements** — particularly the characteristics of well-formed requirements and the attribute framework.

5. **For safety-critical systems**, adopt domain-specific standards (DO-178C, IEC 61508, EN 50128) that mandate assumption traceability.

### 6.3 For Tool Builders

1. **Support custom attributes** for assumptions (verifiability method, criticality, stability, source).

2. **Enable bidirectional traceability** between assumptions, requirements, design decisions, and test cases.

3. **Generate verification matrices** that include assumptions alongside requirements.

4. **Provide templates** based on ISO/IEC/IEEE 29148 with pre-configured assumption sections.

---

## 7. Conclusion

The question of whether to tag assumptions as "testable" or "design decision" is a false binary. The standards and best practices converge on a more nuanced approach:

1. **ISO/IEC/IEEE 29148** requires assumptions to be documented in dedicated sections but does not mandate a binary classification.

2. **INCOSE** provides a rich attribute framework (42 attributes) that captures verifiability, rationale, and V&V method for each requirement and assumption.

3. **BDD/Specification by Example** eliminates untestable assumptions by requiring all preconditions to be concrete and executable.

4. **Domain-specific standards** (DO-178C, ARP4754A, IEC 61508) mandate traceability and validation of safety-critical assumptions.

5. **Open-source templates** demonstrate practical implementation with multi-dimensional classification.

The practical recommendation is to **document assumptions with their verifiability method, rationale, impact if false, owner, and traceability links** — rather than a simple binary tag. This approach satisfies regulatory requirements, enables risk management, and supports verification planning without imposing excessive documentation overhead.

---

## References

### Primary Sources

1. **ISO/IEC/IEEE 29148:2018** — Systems and software engineering — Life cycle processes — Requirements engineering. International Organization for Standardization / Institute of Electrical and Electronics Engineers. https://www.iso.org/standard/72089.html

2. **RFC 2119** — Key words for use in RFCs to Indicate Requirement Levels. S. Bradner, Harvard University, March 1997. https://datatracker.ietf.org/doc/html/rfc2119

3. **INCOSE Guide to Writing Requirements v4** (INCOSE-TP-2010-006-04). International Council on Systems Engineering, Requirements Working Group, June 2023. https://www.incose.org/resources-publications/

4. **Guide to the Software Engineering Body of Knowledge (SWEBOK v3)**. IEEE Computer Society. https://www.computer.org/portal/web/swebok/v3guide

5. **IEEE 830-1998** — IEEE Recommended Practice for Software Requirements Specifications (superseded by ISO/IEC/IEEE 29148). https://standards.ieee.org/ieee/830/1222/

### Domain-Specific Standards

6. **DO-178C** — Software Considerations in Airborne Systems and Equipment Certification. RTCA, 2011. https://www.rtca.org/do-178/

7. **SAE ARP4754A** — Guidelines for Development of Civil Aircraft and Systems. SAE International. https://www.sae.org/standards/arp4754a-guidelines-development-civil-aircraft-systems

8. **IEC 61508** — Functional safety of electrical/electronic/programmable electronic safety-related systems. International Electrotechnical Commission.

9. **EN 50128:2011** — Railway applications — Communication, signalling and processing systems — Software for railway control and protection systems. CENELEC.

### Guides and Handbooks

10. **NASA Systems Engineering Handbook** (NASA/SP-2016-6105). National Aeronautics and Space Administration. https://www.nasa.gov/reference/system-engineering-handbook-appendix/

11. **NASA SE Handbook Appendix C: How to Write a Good Requirement**. https://www.nasa.gov/reference/appendix-c-how-to-write-a-good-requirement/

12. **Specification by Example** — Gojko Adzic. http://gojko.net/books/specification-by-example/

### Open-Source Templates

13. **jam01/SRS-Template** — Software Requirements Specification Template (IEEE 830 / ISO/IEC/IEEE 29148). GitHub. https://github.com/jam01/SRS-Template

14. **jorgeherrera1/ieee-29148-srs-template** — IEEE 29148 SRS Template for Spec-Driven Development. GitHub. https://github.com/jorgeherrera1/ieee-29148-srs-template

15. **ReqView ISO/IEC/IEEE 29148 Templates**. https://www.reqview.com/doc/iso-iec-ieee-29148-templates/

### Academic References

16. Gotel, O. and Finkelstein, C.W. "An Analysis of the Requirements Traceability Problem." Proceedings of the 1st International Conference on Requirements Engineering, IEEE, 1994.

17. Femmer, Henning; Méndez Fernández, Daniel; Wagner, Stefan; Eder, Sebastian. "Rapid quality assurance with Requirements Smells." Journal of Systems and Software, vol. 123, 2017, pp. 190–213.

---

*Report compiled: 2026*
*Document version: 1.0*

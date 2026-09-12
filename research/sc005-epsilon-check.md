# SC-005 / 1e-10 Epsilon Check — spec.md findings

Source: `specs/003-optimize-connectedness/spec.md` (181 lines)

## Where `1e-10` / epsilon appears

| Line | Section | Text (excerpt) | Value explicit? |
|------|---------|----------------|-----------------|
| 95 | Clarifications — Session 2026-09-09 (continued) | "quality-gain magnitude below the assertion threshold — plan anchor: ≤1e-10" | Yes (1e-10) |
| 123 | Clarifications — Session 2026-09-10 | "PolBlogs must also produce modularity Q within 1e-10 of a one-time pre-optimization baseline ... The epsilon value 1e-10 is now normative." | Yes (1e-10, called normative) |
| 152 | SC-005 | "quality scores within floating-point epsilon of baseline" | **No** — only phrase "floating-point epsilon"; no numeric value given |
| 154 | SC-007 | "No regression in modularity Q values for any test graph (within floating-point epsilon)." | **No** — only phrase "floating-point epsilon" |
| 156 | SC-009 | "produces modularity Q within 1e-10 of a one-time pre-optimization baseline" | Yes (1e-10) |
| 157-159 | Measurement Protocol | No epsilon/1e-10 reference in the two-mode protocol (Timing / Verification). | N/A |

## SC-005 line 152 — explicit or referred?

**Only refers to "floating-point epsilon"; does NOT state the value explicitly.**
Line 152: "All existing Tier 2 LFR benchmark tests continue to pass with quality scores within floating-point epsilon of baseline and community count matching."
No `1e-10`, no numeric literal. By contrast:
- SC-009 (line 156) explicitly says `1e-10`.
- Clarification answer at line 123 explicitly says `1e-10` is normative.
- Clarification at line 95 references `≤1e-10` as the assertion-threshold plan anchor.

SC-007 (line 154) is the same pattern — only "floating-point epsilon", no explicit value.

## Clarifications that anchor the value

- Line 95: "equivalent quality (within floating-point epsilon) ... quality-gain magnitude below the assertion threshold — plan anchor: ≤1e-10"
- Line 123: Direct clarification answer (Session 2026-09-10): extends SC-009 with invariants; explicitly states "The epsilon value 1e-10 is now normative."
- Line 109: SC-005 / SC-007 extension reference — "quality equivalence (within floating-point epsilon) plus structural invariants ... optimality conditions (quality-gain magnitude below the assertion threshold — plan anchor: ≤1e-10)" (same line 95 context, reiterated in FR-006 context).

## Measurement Protocol

Lines 157-159 (Timing / Verification). No epsilon/value specified there; SC-009's `1e-10` is the only measurement-anchor for quality equivalence, tied to the one-time pre-optimization baseline measured during implementation.

## Summary

- Explicit `1e-10`: SC-009 (line 156), clarification answers (lines 95, 123).
- Implicit / unnamed: SC-005 (line 152) and SC-007 (line 154) — both say only "floating-point epsilon".
- The normative `1e-10` value originates from clarification answers (line 123) and should be read into SC-005/SC-007 by reference, but the spec line itself does not state it.

# Checklist Review: CHK034 — License Compatibility for External Dependencies

## Verdict: **NO**

The spec does **not** specify license compatibility requirements for external dependencies (petgraph, rayon, etc.).

---

## Evidence from Spec

### 1. External Dependencies Table (Lines 518–541)

The "External Dependencies" table lists dependencies with version/reference and purpose only:

| Dependency | Version/Reference | Purpose |
|------------|-------------------|---------|
| petgraph | 0.8.3 | Graph data structure interoperability |
| rayon | 1.10 | Data parallelism for algorithm phases |
| thiserror | 2.0 | Domain-rich error types |
| ... | ... | ... |

**No license column, no compatibility notes, no MIT/Apache-2.0 verification requirements.**

### 2. Clarifications Session 2026-09-04 (4) — Line 20

A direct Q&A on this exact topic:

> **Q**: Should the spec explicitly require license compatibility verification for external dependencies (petgraph, rayon, thiserror, etc.), or is the existing dual-license statement (MIT OR Apache-2.0) sufficient?
>
> **A**: Current dual-license statement is sufficient; no explicit compatibility check needed.

This clarification **explicitly rejected** adding license compatibility verification requirements to the spec.

---

## Evidence from Constitution (Principle VII — Lines 121–131)

The constitution addresses licensing but only for the framework's own code and absorbed repositories:

- **Permissive Dual-Licensing**: All original codebase assets are licensed under `MIT OR Apache-2.0`.
- **Third-Party Attribution**: Explicitly preserve copyright notices from absorbed open-source repositories (`hit-leiden`, `leiden-rs`, `fa-leiden-cd`, `leiden-wasm`) within their respective files and a dedicated `THIRD_PARTY_LICENSES.md`.

**The constitution does NOT specify license compatibility requirements for external dependencies** (petgraph, rayon, thiserror, etc.). It only governs:
1. The framework's own dual-license (MIT OR Apache-2.0)
2. Attribution for absorbed/embedded third-party code (not external crate dependencies)

---

## Explanation

The spec and constitution together establish:

1. The framework's own license: `MIT OR Apache-2.0` (Constitution Principle VII, line 127).
2. A decision that no explicit compatibility check is needed for external dependencies (Spec clarification, line 20).
3. Attribution requirements only for absorbed repositories, not for external crate dependencies.

**What is missing**: Any requirement to verify that external dependencies (petgraph, rayon, thiserror, serde, ratatui, etc.) have licenses compatible with the framework's `MIT OR Apache-2.0` dual-license. The spec does not mandate checking dependency licenses, documenting them, or ensuring compatibility.

**Conclusion**: CHK034 is **NOT satisfied**. The spec explicitly decided against specifying license compatibility requirements for external dependencies, and the constitution's licensing governance does not cover external crate dependencies.

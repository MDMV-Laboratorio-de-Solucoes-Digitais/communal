# Unsafe Code Policy Conflict Resolution

**Date**: 2026-09-05  
**Spec**: 001-community-detection  
**Conflict**: Constitution Principle IV vs. Specification FR-039  
**Severity**: LOW (semantic alignment, not functional correctness)  
**Status**: Resolved

---

## 1. Current State Analysis

### 1.1 Constitution (Authoritative Source)

**File**: `.specify/memory/constitution.md`, Principle IV, line 73

```markdown
- **Zero Unsafe Policy**: `#![deny(unsafe_code)]` across all library crates. Any potential performance optimization requiring `unsafe` MUST be subjected to a formal justification RFC and isolated.
```

Key properties:
- **Absolute deny**: `#![deny(unsafe_code)]` is mandated with no opt-out.
- **Exception process exists**: Exceptions require "formal justification RFC" and isolation.
- **Scope**: "across all library crates" — applies to framework crates only.

### 1.2 Specification FR-039

**File**: `specs/001-community-detection/spec.md`, line 297

```markdown
- **FR-039**: The system MUST enforce `#![deny(unsafe_code)]` across all library crates. This policy applies to the framework's own code only; external dependencies (e.g., rayon, petgraph) are trusted as well-audited crates where unsafe is carefully isolated for performance. External crates that trigger `#![deny(clippy::allow_attributes_without_reason)]` or similar lints MUST be addressed with `#[expect(...)]` attributes specifying the reason. Any exception for performance optimization in framework code MUST require a formal RFC document containing: (1) safety justification explaining why safe alternatives are insufficient, (2) isolation requirements specifying how the unsafe code is contained, and (3) mandatory code review approval before merging. Approved exceptions MUST be documented in a dedicated tracking file.
```

Key properties:
- Same `#![deny(unsafe_code)]` mandate.
- Explicit "framework code only" scope (constitution implies this).
- Mentions external crate trigger behavior (technically incorrect — see §2.1).
- Adds detailed RFC requirements (constitution is more terse).

### 1.3 Nature of the Conflict

The conflict is **semantic, not functional**. Both documents agree on the core policy: `#![deny(unsafe_code)]` with RFC-gated exceptions. The tension is:

| Dimension | Constitution | Specification |
|-----------|-------------|---------------|
| Tone | Absolute ("NO exception clause" at lint level) | Elaborated (explicit exception process) |
| External deps | Not addressed (implicit: lint is crate-local) | Explicitly mentioned (with incorrect trigger description) |
| RFC details | "formal justification RFC and isolated" | 3-point RFC content requirement + tracking file |
| `#[expect(...)]` | Not mentioned | Mentioned for external crates (misleading) |

The constitution takes authoritative precedence per `.specify/memory/constitution.md` line 235: *"The `.specify/memory/constitution.md` is the authoritative source for governance questions."*

---

## 2. Rust Community Best Practices Research

### 2.1 How `#![deny(unsafe_code)]` Actually Works (rustc semantics)

**Critical finding**: The `unsafe_code` lint is **crate-local**. It only fires on unsafe code within the crate being compiled. External dependencies' unsafe code **never** triggers the lint in the consuming crate. This is a rustc guarantee documented in the [rustc lints reference](https://doc.rust-lang.org/rustc/lints/index.html).

Implications for FR-039's language:
- "External crates that trigger `#![deny(clippy::allow_attributes_without_reason)]` or similar lints" — this is **technically impossible**. External crates do not trigger local lints in the consuming crate.
- "This policy applies to the framework's own code only; external dependencies are trusted" — this is already how `#![deny(unsafe_code)]` works by construction. The clarification is helpful but the mechanism is mischaracterized.
- `#[expect(...)]` attributes are only needed for **framework-internal** exceptions, never for external dependencies.

### 2.2 How Major Projects Handle This

| Project | Policy | Exception Model |
|---------|--------|-----------------|
| **Bevy** | `#![deny(unsafe_code)]` in most crates | Uses `deny` (not `forbid`) so that `#[allow(unsafe_code)]` can be applied to specific items with justification ([bevyengine/bevy#3824](https://github.com/bevyengine/bevy/issues/3824)) |
| **Tokio** | No `#![deny(unsafe_code)]` | Uses `unsafe` extensively; does not enforce deny |
| **Rayon** | No `#![deny(unsafe_code)]` | Uses `unsafe` extensively for parallelism primitives |
| **Serde** | No `#![deny(unsafe_code)]` | Had a discussion ([serde-rs/serde#2096](https://github.com/serde-rs/serde/issues/2096)) about removing unsafe and adding `#![forbid(unsafe_code)]`, but the one `unsafe` usage (UTF-8 conversion) was justified for performance |
| **brynary/rust-style-guide** | `unsafe_code = "deny"` in workspace | "Treat project-written unsafe as an explicit crate-level exception, not a local convenience" |

**Key insight**: High-performance Rust crates (tokio, rayon) that fundamentally rely on `unsafe` for their core abstractions **do not** use `#![deny(unsafe_code)]`. Projects that do use `#![deny(unsafe_code)]` (like Bevy's non-core crates) treat exceptions as **explicit, justified, item-level allow attributes** — not as blanket policy carve-outs.

### 2.3 Trade-off Analysis

| Approach | Pros | Cons |
|----------|------|------|
| `#![deny(unsafe_code)]` (current) | Catches accidental unsafe; signals safety commitment; zero-cost to maintain | May block legitimate performance optimizations; requires RFC overhead for exceptions |
| `#![forbid(unsafe_code)]` | Absolute guarantee; cannot be overridden locally | No exceptions possible; too rigid for a high-performance framework |
| `unsafe_code = "warn"` in workspace | Allows exceptions without friction | Unsafe becomes normalized; loses signaling value |
| `#![deny(unsafe_code)]` + item-level `#[allow]` | Best of both: catches accidents, allows justified exceptions | Requires discipline to ensure every `#[allow]` has justification |

**Recommended**: Continue with `#![deny(unsafe_code)]` + item-level `#[allow(unsafe_code)]` gated by RFC. This matches Bevy's approach and the constitution's "RFC and isolated" requirement.

---

## 3. Recommended Resolution

### 3.1 Principle

1. **Constitution is authoritative** — the spec must conform to it, not vice versa.
2. **The constitution's policy is sound** — `#![deny(unsafe_code)]` with RFC-gated exceptions is the right posture for a safety-critical framework.
3. **FR-039 needs technical corrections** — the language about external crate triggers is incorrect and must be rewritten to accurately describe how rustc lints work.
4. **FR-039's RFC details are compatible** — the 3-point RFC content requirement and tracking file are elaborations that do not conflict with the constitution's "formal justification RFC and isolated" language.

### 3.2 Resolution Strategy

- **Constitution**: No edit needed. It is clear and correct as-is.
- **Spec FR-039**: Rewrite to:
  1. Correct the technically inaccurate external-crate trigger language.
  2. Clarify that `#![deny(unsafe_code)]` is crate-local by rustc semantics (no external dependency can trigger it).
  3. Keep the RFC details (they are a valid elaboration of the constitution's requirement).
  4. Use `#[allow(unsafe_code)]` at item level (not `#[expect(...)]` for external crates) as the mechanism for approved exceptions.

---

## 4. Exact Edits

### 4.1 Constitution — No Edit Required

The constitution at line 73 is correct and needs no modification:

```markdown
- **Zero Unsafe Policy**: `#![deny(unsafe_code)]` across all library crates. Any potential performance optimization requiring `unsafe` MUST be subjected to a formal justification RFC and isolated.
```

Rationale: The constitution correctly establishes the policy at the appropriate governance level. It is terse by design and does not need elaboration of rustc lint mechanics.

### 4.2 Specification FR-039 — Rewrite

**File**: `specs/001-community-detection/spec.md`  
**Line**: 297

**Current text**:
```markdown
- **FR-039**: The system MUST enforce `#![deny(unsafe_code)]` across all library crates. This policy applies to the framework's own code only; external dependencies (e.g., rayon, petgraph) are trusted as well-audited crates where unsafe is carefully isolated for performance. External crates that trigger `#![deny(clippy::allow_attributes_without_reason)]` or similar lints MUST be addressed with `#[expect(...)]` attributes specifying the reason. Any exception for performance optimization in framework code MUST require a formal RFC document containing: (1) safety justification explaining why safe alternatives are insufficient, (2) isolation requirements specifying how the unsafe code is contained, and (3) mandatory code review approval before merging. Approved exceptions MUST be documented in a dedicated tracking file.
```

**Replacement text**:
```markdown
- **FR-039**: The system MUST enforce `#![deny(unsafe_code)]` across all library crates. Per rustc semantics, this lint is crate-local: it only fires on unsafe code within each framework crate, and external dependencies (e.g., rayon, petgraph) do NOT trigger this lint in framework crates regardless of their internal unsafe usage. Any exception for performance optimization in framework code MUST require a formal RFC document containing: (1) safety justification explaining why safe alternatives are insufficient, (2) isolation requirements specifying how the unsafe code is contained (item-level `#[allow(unsafe_code)]` with mandatory `reason = "..."`), and (3) mandatory code review approval before merging. Approved exceptions MUST be documented in a dedicated tracking file at `docs/unsafe-exceptions.md` with the RFC reference, affected modules, and review date.
```

### 4.3 Diff Summary

| What | Change |
|------|--------|
| Removed | "This policy applies to the framework's own code only; external dependencies are trusted as well-audited crates where unsafe is carefully isolated for performance." — redundant given rustc semantics |
| Removed | "External crates that trigger `#![deny(clippy::allow_attributes_without_reason)]` or similar lints MUST be addressed with `#[expect(...)]` attributes specifying the reason." — technically impossible; external crates never trigger local lints |
| Added | Explicit rustc semantics clarification: "Per rustc semantics, this lint is crate-local: it only fires on unsafe code within each framework crate, and external dependencies do NOT trigger this lint" |
| Added | Mechanism specification: "item-level `#[allow(unsafe_code)]` with mandatory `reason = "..."`" |
| Added | Tracking file path: `docs/unsafe-exceptions.md` with required fields |
| Preserved | RFC 3-point content requirement |
| Preserved | Mandatory code review approval |
| Preserved | Tracking file requirement |

---

## 5. Verification

### 5.1 Constitution Alignment

The revised FR-039:
- ✅ Preserves `#![deny(unsafe_code)]` mandate
- ✅ Maintains RFC-gated exceptions (constitution line 73)
- ✅ Preserves "isolated" requirement via item-level `#[allow]`
- ✅ Does not add any provision that contradicts constitution
- ✅ Respects constitution's authoritative status (line 235)

### 5.2 Technical Correctness

The revised FR-039:
- ✅ Correctly describes rustc lint scoping (crate-local)
- ✅ Removes the impossibility of external crates triggering local lints
- ✅ Specifies the correct attribute mechanism (`#[allow(unsafe_code)]` with reason)
- ✅ Provides a concrete tracking file location

### 5.3 Workspace Lint Configuration Impact

The existing `Cargo.toml` `[workspace.lints.rust]` block already has `unsafe_code = "deny"` (line 95), which applies to all workspace members with `workspace = true` (which all library crates declare). No change needed there.

The facade crate (`src/lib.rs`) already has `#![deny(unsafe_code)]` at line 5, which is correct for the root crate.

---

## 6. Sources

1. [rustc Lints Reference - unsafe_code](https://doc.rust-lang.org/rustc/lints/index.html) — Documents that `unsafe_code` lint catches unsafe code within the crate being compiled.
2. [Cargo Lints Reference](https://doc.rust-lang.org/cargo/reference/lints.html) — Documents workspace lint inheritance and crate-local semantics.
3. [Bevy Issue #3824 — Deny unsafe code in most of the code base](https://github.com/bevyengine/bevy/issues/3824) — Real-world example of a major project adopting `#![deny(unsafe_code)]` with item-level allow exceptions.
4. [Serde Issue #2096 — remove unsafe from serde and add `#![forbid(unsafe_code)]`](https://github.com/serde-rs/serde/issues/2096) — Discussion of the trade-offs of absolute unsafe denial in a serialization framework.
5. [Rayon lib.rs (v1.12.0)](https://docs.rs/crate/rayon/latest/source/src/lib.rs) — Shows a high-performance crate that does NOT use `#![deny(unsafe_code)]` because unsafe is fundamental to its implementation.
6. [brynary/rust-style-guide: Unsafe Code and Macros](https://github.com/brynary/rust-style-guide/blob/main/guidelines/unsafe-code-and-macros.md) — Recommends `unsafe_code = "deny"` as default with exceptions treated as explicit crate-level decisions.
7. [Rust RFC 3389 — Manifest Lint Configuration](https://rust-lang.github.io/rfcs/3389-manifest-lint.html) — Documents the `[lints]` table mechanism used by Cargo for workspace-level lint configuration.

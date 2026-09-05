# Remediation Summary: CHK034 — License Compatibility Requirements

**Date**: 2026-09-04
**Checklist Item**: CHK034 (Security & Privacy Requirements Quality Checklist)
**Status**: Remediated — `[ ]` → `[x]`

## Gap Identified

The spec did not explicitly state license compatibility requirements for external dependencies (petgraph, rayon, thiserror, serde, ratatui, crossterm, wasm-bindgen, proptest, criterion, num-traits, tracing, tracing-subscriber, rolling-file). The original clarification concluded that the dual-license statement alone was sufficient, leaving this as an unchecked gap.

## Research: Rust License Compatibility Practices

Rust projects commonly handle license compatibility through:

- **`cargo-deny`**: A popular cargo subcommand that lints against license incompatibilities, checks for advisories, and validates dependency licenses against an allowlist/denylist.
- **`cargo-license`**: Generates a summary of all dependency licenses, useful for auditing.
- **`deny.toml` configuration**: Projects define allowed licenses (e.g., `["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "Unicode-DFS-2016"]`) and the tool flags any dependency outside the list.

The Rust ecosystem overwhelmingly uses permissive licenses (MIT, Apache-2.0, or dual MIT/Apache-2.0). GPL/copyleft dependencies are rare in the core ecosystem and typically avoided in dual-licensed MIT/Apache-2.0 projects because GPL terms are incompatible with Apache-2.0's patent grant.

## Dependency License Audit

| Dependency | License | MIT-Compatible |
|------------|---------|----------------|
| petgraph | MIT/Apache-2.0 | Yes |
| rayon | MIT/Apache-2.0 | Yes |
| thiserror | MIT/Apache-2.0 | Yes |
| serde | MIT/Apache-2.0 | Yes |
| ratatui | MIT | Yes |
| crossterm | MIT | Yes |
| wasm-bindgen | MIT/Apache-2.0 | Yes |
| proptest | MIT/Apache-2.0 | Yes |
| criterion | MIT/Apache-2.0 | Yes |
| num-traits | MIT/Apache-2.0 | Yes |
| tracing | MIT | Yes |
| tracing-subscriber | MIT | Yes |
| rolling-file | MIT | Yes |

**Result**: All 13 external dependencies use MIT-compatible licenses. No GPL or copyleft dependencies are present in the default feature set.

## Changes Made

### 1. spec.md — Added "License Compatibility" Section

**Location**: After the External Dependencies table (line 536), before the Assumptions section.

**Content**:
- Framework uses MIT OR Apache-2.0 per Constitution Principle VII
- All external dependencies use MIT-compatible licenses (MIT, Apache-2.0, or dual MIT/Apache-2.0)
- No GPL or copyleft dependencies in the default feature set
- CI pipeline SHOULD include a license compatibility check (e.g., `cargo-deny` or `cargo-license`)
- `THIRD_PARTY_LICENSES.md` (required by T006) will contain upstream attribution

### 2. spec.md — Updated Clarification Entry

The original clarification (Session 2026-09-04 (4)) stated "no explicit compatibility check needed." Updated with a **Remediated** note explaining the new section and its contents.

### 3. security-privacy.md — Checklist Updated

- CHK034 changed from `[ ]` to `[x]`
- Added remediation note in the Notes section documenting the change

## Verification

- [x] License compatibility statement added to spec.md
- [x] All 13 dependencies confirmed MIT-compatible
[x] No GPL/copyleft dependencies in default feature set
- [x] Constitution Principle VII referenced
- [x] CI recommendation included (cargo-deny / cargo-license)
- [x] THIRD_PARTY_LICENSES.md attribution noted
- [x] Checklist CHK034 marked satisfied
- [x] Clarification entry updated with remediation note

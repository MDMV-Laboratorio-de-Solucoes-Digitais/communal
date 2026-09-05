# Remediation Summary: CLI & TUI Interface Contracts

**Date**: 2026-09-05
**Spec**: specs/001-community-detection/spec.md
**Checklist**: specs/001-community-detection/checklists/api-design.md

## Checklist Items Addressed

| Checklist Item | Status | What Was Added |
|----------------|--------|----------------|
| CHK039 | [x] | CLI output formats per command table |
| CHK040 | [x] | Batch TOML config schema |
| CHK041 | [x] | TUI interface contract |
| CHK042 | [x] | Pedagogical template structure |

## Additions to spec.md

### 1. CLI Output Formats per Command (after FR-042)

Added a table mapping each of the 7 CLI commands (`run`, `compare`, `batch`, `convert`, `metrics`, `generate`, `validate`) to their default format, supported formats, and output content description. Also added format selection rules: file extension overrides `--format` flag, JSON default to stdout, CSV column conventions, GML schema conformance, and round-trip fidelity guarantee per FR-041.

**Pattern reference**: ripgrep's `--json` flag and output format documentation, where each format has a clear purpose (machine-readable vs. human-readable) and the default is the most pipe-friendly option.

### 2. Batch TOML Config Schema (FR-042b)

Added a TOML schema section with:
- `output_dir` field (required, string)
- `algorithms` array (required, valid identifiers)
- `[parameters]` table with `gamma_range` and `seed_range` for Cartesian product sweeps
- `[[input]]` array-of-tables with `file`, optional `algorithm` override, optional `gamma` override
- Example TOML snippet demonstrating all features
- Error contract: unknown keys produce `ConfigError::UnknownKey` with line number

**Pattern reference**: TOML v1.0 array-of-tables convention (`[[input]]`), Cargo's `[dependencies]` table structure, and pre-commit's `[[repos]]` array pattern. Follows TOML best practices: logical grouping, descriptive table names, limited nesting depth, arrays of tables for collections.

### 3. TUI Interface Contract

Added a comprehensive TUI contract section covering:
- **Panel layout**: ASCII art diagram showing 60/40 horizontal split (graph left, event log + stats right) with full-width pedagogical overlay at bottom, implemented via ratatui's `Layout` constraint system
- **Left panel**: Fruchterman-Reingold force-directed graph layout with community-based node coloring, log-scaled node sizes, animated transitions, 2-hop neighborhood highlighting during streaming
- **Right panel top**: Scrollable event log of `StepEvent` variants with timestamps, keyboard navigation, 10K entry limit
- **Right panel bottom**: Statistics panel showing algorithm name, iteration, quality, community count, convergence config
- **Bottom overlay**: Pedagogical explanation text area (3-4 lines)
- **Event loop contract**: `StepCallback` registration, terminal resize handling, keyboard input (pause/resume/scroll/quit), paused state display

**Pattern reference**: ratatui's `Layout` struct with `Direction::Horizontal`/`Vertical` and `Constraint::Percentage`/`Min`; nesting layouts for complex dashboards; the constraint solver approach for responsive terminal sizing.

### 4. Pedagogical Template Structure

Added a pedagogical template spec defining:
- **Template format**: Static `&str` const values with `{interpolation_points}`
- **Interpolation variables table**: 9 variables with types, descriptions, and example values
- **Content guidelines**: Max 2 sentences, first = WHAT happened, second = WHY it matters, avoid jargon, active voice, present tense
- **Example templates per phase type**: One `const &str` example for each major phase (LocalMovingStart, NodeRelocation, RefinementSplit, AggregationContraction, CommunityMerge, CommunitySplit, ConvergenceDetected, ConvergencePlateau)
- **Template selection**: One template per `StepEvent` variant; missing variables render as `"—"` fallback

**Pattern reference**: "Explain Like I'm 5" technical writing — concrete, analogy-driven, active voice; educational software patterns where templates provide consistent framing with dynamic context; Rust's `format!`-style interpolation adapted for static templates.

## Research Sources

- TOML v1.0 specification: array-of-tables convention (https://toml.io)
- ratatui layout documentation: `Layout` constraints, `Direction`, nesting, flex strategies (https://ratatui.rs/concepts/layout/)
- ripgrep output format documentation: per-format purpose and default selection (https://ripgrep.dev/docs/)
- TOML best practices from Cargo, pre-commit config patterns
- Educational software patterns: ELI5 technical writing, static templates with dynamic interpolation

## What Was NOT Modified

- `plan.md` — intentionally left unchanged per task instructions
- `tasks.md` — intentionally left unchanged per task instructions
- FR numbering beyond FR-042b — new requirements added as subsections rather than renumbering existing FR-043 through FR-046

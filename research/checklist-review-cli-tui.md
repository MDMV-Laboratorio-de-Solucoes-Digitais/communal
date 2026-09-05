# Checklist Review: CLI, TUI, Batch, and Pedagogical Explanations

**Spec**: `/home/luis/development/MDMV/projetos/communal/specs/001-community-detection/spec.md`
**Date**: 2026-09-05

---

## CHK039: CLI Output Format Requirement (JSON, CSV, GML per command)

### Verdict: NO

**Evidence:**

- **FR-042** (line 314) lists CLI commands but does **not** specify which output formats each command supports:
  > `run` (execute community detection on a graph file), `compare` (run multiple algorithms and produce metrics table + pairwise NMI/ARI), `batch` (process multiple files/algorithms/parameters via unified batch configuration), `convert` (convert between graph formats: JSON, CSV, GML), `metrics` (compute quality/comparative metrics for existing partitions), `generate` (generate synthetic benchmark graphs), `validate` (validate graph file structure and content).

- **FR-041** (line 313) specifies serialization formats at the library level but does not map them to CLI commands:
  > The system MUST provide graph and partition serialization capabilities for JSON, CSV, and GML formats.

- The `convert` command (line 314) mentions "convert between graph formats: JSON, CSV, GML" — the only place formats are tied to a command — but there is no specification of which formats `run`, `compare`, `batch`, or `metrics` accept as output.

**Explanation:** The spec defines *that* serialization exists (FR-041) and *that* CLI commands exist (FR-042), but never cross-references them. A reader cannot determine whether `run --format json` is valid, whether `compare` outputs CSV, or whether `batch` supports GML output. The mapping from commands to their supported output formats is absent.

---

## CHK040: Batch Configuration Format (TOML) with Schema and Parameters

### Verdict: NO

**Evidence:**

- **Line 79** (Clarification Session 2026-09-03 (3)) confirms batch modes but not the format:
  > What should CLI "batch processing runs" support? → A: All modes - File batching (multiple files), algorithm batching (multiple algorithms on one file), and parameter sweeps (varying gamma, thresholds) through unified batch configuration.

- **FR-042** (line 314) mentions the `batch` command:
  > `batch` (process multiple files/algorithms/parameters via unified batch configuration)

- **Line 556** (Assumptions) restates the capability:
  > batch processing runs (file batching, algorithm batching, and parameter sweeps via unified batch configuration).

- **No mention of TOML anywhere in the spec.** The word "TOML" does not appear. There is no schema definition, no parameter specification for the batch config file, and no `contracts/batch-config.schema.md` or equivalent reference.

**Explanation:** The spec establishes *what* batch processing must do (file batching, algorithm batching, parameter sweeps) but never specifies *how* the batch configuration is expressed. TOML is not named, no schema is provided, and no parameter list (e.g., `files`, `algorithms`, `gamma_range`, `output_format`) is defined. The "unified batch configuration" is a black box.

---

## CHK041: TUI Interface Contract (Event Log, Statistics, Graph Layout, Pedagogical Explanations)

### Verdict: NO (as a formal contract)

**Evidence:**

- **User Story 5** (line 221) describes the TUI at a high level:
  > The TUI provides force-directed graph visualization with pedagogical explanations generated from static templates with interpolated algorithm state values, providing consistent educational framing with concrete context-aware details.

- **Acceptance Scenario 5.5** (line 233) mentions the TUI behavior:
  > Given the TUI is displaying algorithm execution, When a step completes, Then the force-directed graph layout updates to reflect current community assignments with pedagogical explanations (generated from static templates with interpolated algorithm state values) describing the visualized state.

- **Line 557** (Assumptions) confirms the four elements exist:
  > The TUI provides event logging, real-time statistics, force-directed graph layout visualization, and pedagogical didactic explanations generated from static templates with interpolated algorithm state values (hybrid approach).

- **FR-017** (line 270) defines observable events but does not specify how the TUI renders them into panels.

- **No formal TUI contract exists**: There is no specification of panel layout (e.g., "event log panel occupies left 40% of screen"), no statistics panel content (e.g., "shows iteration count, current quality, community count"), no graph layout algorithm details (e.g., "force-directed using Fruchterman-Reingold"), and no rendering contract for pedagogical explanations.

**Explanation:** The spec confirms the TUI *has* these four features (event logging, statistics, graph layout, pedagogical explanations) but does not specify them as a formal *contract*. There are no panel dimensions, no layout algorithm specification, no statistics fields enumeration, and no rendering lifecycle. The mentions are descriptive acceptance scenarios, not prescriptive interface contracts.

---

## CHK042: Pedagogical Explanation Template (Structure, Interpolation Points, Content Guidelines)

### Verdict: NO

**Evidence:**

- **Line 77** (Clarification Session 2026-09-03 (3)) establishes the hybrid approach:
  > Should TUI pedagogical explanations be static, dynamic, or hybrid? → A: Hybrid - Static pedagogical templates with interpolated algorithm state values, providing consistent educational framing with concrete context-aware details.

- **Line 221** (User Story 5) repeats the concept:
  > pedagogical explanations generated from static templates with interpolated algorithm state values

- **Line 233** (Acceptance Scenario 5.5) repeats again:
  > pedagogical explanations (generated from static templates with interpolated algorithm state values) describing the visualized state

- **Line 557** (Assumptions) confirms:
  > pedagogical didactic explanations generated from static templates with interpolated algorithm state values (hybrid approach)

- **No template structure is provided**: There is no example template (e.g., "Phase {phase_name}: {node_count} nodes moved across {community_count} communities..."), no defined interpolation points (e.g., `{iteration}`, `{quality_delta}`, `{community_id}`), and no content guidelines (e.g., "explanations must be understandable by undergraduates", "must reference the current algorithm phase").

**Explanation:** The spec consistently states *that* pedagogical explanations use a hybrid static-template-with-interpolation approach, but never specifies *what* the templates look like, *which* variables are available for interpolation, or *what* content standards they must meet. The concept is affirmed; the contract is absent.

---

## Summary

| Checklist Item | Verdict | Gap |
|----------------|---------|-----|
| **CHK039** — CLI output formats per command | **NO** | FR-041 defines formats; FR-042 defines commands; no mapping between them |
| **CHK040** — Batch TOML config schema | **NO** | "Unified batch configuration" mentioned but TOML not named, no schema, no parameters |
| **CHK041** — TUI interface contract | **NO** | Four TUI features confirmed but no panel/layout/rendering contract specified |
| **CHK042** — Pedagogical template structure | **NO** | Hybrid approach affirmed but no template examples, interpolation points, or content guidelines |

All four items share a common pattern: the spec establishes *that* a capability exists at a conceptual level but does not provide the prescriptive detail (contracts, schemas, mappings, templates) needed for unambiguous implementation.

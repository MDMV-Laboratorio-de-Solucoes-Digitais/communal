# Research Findings: Specification Remediation (2026-09-05)

**Related**: [fr-041-gap-analysis.md](fr-041-gap-analysis.md) (previous gap analysis)

## 1. FR Numbering Convention

### Finding
The established convention in this spec is to preserve numbering stability — never renumber existing FRs. Gaps are handled two ways:

| Scenario | Convention | Example |
|----------|------------|---------|
| Intentionally unused slot | `~~Reserved~~` placeholder with explanation | FR-012, FR-022 |
| Accidentally skipped number | Insert new FR to fill gap | FR-028, FR-041 |

### Source
- `/specs/001-community-detection/research/fr-041-gap-analysis.md` — Documents the gap-filling convention
- `/specs/001-community-detection/spec.md` lines 262, 275 — Existing reserved placeholders

### Action Applied
Added `~~Reserved~~` placeholders for FR-012 and FR-022 matching the established convention.

---

## 2. Convergence Threshold vs. Plateau Threshold

### Finding
Reference implementations (leidenalg, igraph, Java CWTS) use **zero** as the convergence threshold (any strictly positive improvement is accepted) and stop after **one** non-improving iteration. The spec's two-threshold design (1e-6 convergence + 1e-7 plateau with N=5 persistence) is a **non-standard extension**.

| Implementation | Convergence Threshold | Plateau/Early-Stop | Stopping Criterion |
|---|---|---|---|
| Leiden paper (Traag 2019) | Zero (strictly improving) | None | Single iteration with no change |
| igraph C | Zero (`diff > max_diff`) | None | `changed == false` |
| leidenalg Python | `10*DBL_EPSILON` ≈ 0 | None | `aggregate_further == false` |
| Java (CWTS) | Zero | None | Single iteration with no change |

### Source
- [From Louvain to Leiden (Traag et al. 2019)](https://www.nature.com/articles/s41598-019-41695-z)
- [igraph C leiden.c](https://github.com/igraph/igraph/blob/master/src/community/leiden.c)
- [libleidenalg Optimiser.cpp](https://raw.githubusercontent.com/vtraag/libleidenalg/main/src/Optimiser.cpp)
- [CWTS Leiden Java LeidenAlgorithm](https://raw.githubusercontent.com/CWTSLeiden/networkanalysis/master/src/main/java/nl/cwts/networkanalysis/LeidenAlgorithm.java)

### Rationale for Spec Design
The spec's two-threshold design serves practical purposes not addressed by reference implementations:
1. **Floating-point noise tolerance**: 1e-6 threshold prevents infinite loops from floating-point precision artifacts
2. **Early stopping for performance**: Allows trading marginal quality gains for computation time
3. **Observability signal**: Plateau detection (N=5 below 1e-7) provides debugging signal without affecting correctness

### Action Applied
Added explicit plateau threshold derivation formula: `plateau_threshold = max(convergence_threshold / 10, 1e-8)`. This documents the relationship and prevents floating-point issues with very small convergence thresholds.

---

## 3. CSV as Graph Input Format

### Finding
CSV is **not** a formal graph file format standard (like GraphML, GML, Pajek). The canonical graph formats are adjacency matrix, edge list, and adjacency list. However, CSV edge-list (`source,target,weight` per row) is a **de facto standard** for data exchange with spreadsheets and tools like Gephi.

| Library | CSV Support | Mechanism |
|---------|-------------|-----------|
| NetworkX | Via `read_edgelist(delimiter=',')` | Delimiter parameter |
| igraph | None native | Parse with pandas/csv, then `Read_Ncol` |
| graph-tool | Native `load_graph_from_csv()` | Full dialect options |
| SNAP | Whitespace edge list | `\t` or space, `#` comments |

### Source
- [NetworkX read_edgelist docs](https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.edgelist.read_edgelist.html)
- [igraph Foreign Formats](https://igraph.org/c/html/0.9.5/igraph-Foreign.html)
- [graph-tool load_graph_from_csv](https://graph-tool.skewed.de/static/doc/autosummary/graph_tool.load_graph_from_csv.html)
- [SNAP LoadEdgeList](https://metromaps.stanford.edu/snappy/doc/reference/LoadEdgeList.html)

### Action Applied
Added CSV to FR-009 input formats with explicit edge-list format specification (`source,target,weight` per line). Added T051a task for CSV parser implementation.

---

## 4. Timestamp Format in Log Rotation

### Finding
The original FR-037 example `app.log.2026-005-06-14-30` contained an invalid month value ("005"). Standard timestamp formats for log rotation use `YYYY-MM-DD-HH-MM` pattern.

### Action Applied
Fixed to `app.log.YYYY-MM-DD-HH-MM` format with valid example `app.log.2026-09-05-14-30`.

---

## Summary of Remediations Applied

| Issue | Location | Remediation |
|-------|----------|-------------|
| FR-012 missing | spec.md | Added `~~Reserved~~` placeholder |
| FR-022 missing | spec.md | Added `~~Reserved~~` placeholder |
| FR-037 typo | spec.md | Fixed timestamp format example |
| FR-009 CSV scope | spec.md | Added CSV to input formats + schema |
| FR-033 plateau relationship | spec.md | Added derivation formula |
| T028b incomplete | tasks.md | Completed trait description |
| T028d CSV schema | tasks.md | Added csv-graph.schema.md |
| T051a CSV parser | tasks.md | Added CSV parser task |

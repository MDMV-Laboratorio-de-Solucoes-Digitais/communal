# Converting Vague Qualitative Terms into Measurable Quantitative Criteria in Software Specifications

## Overview

This document provides concrete, citation-backed replacements for common vague qualitative terms found in software specifications. It draws on IEEE standards (830-1998, 29148), Google SRE practices, and measurable SLAs/SLOs from well-specified open-source systems (Apache Spark, Neo4j, ScyllaDB, Apache APISIX, TigerGraph, LDBC benchmarks).

---

## 1. "High-Performance" → Specific Throughput/Latency Metrics

### The Problem
"High-performance" is unverifiable. IEEE 830-1998 explicitly calls out non-verifiable requirements like *"works well"* and *"shall usually happen"* as inadequate because no finite cost-effective process can check them (§4.3.6).

### Measurable Replacements

| Vague Term | Measurable Replacement | Source |
|---|---|---|
| "high-performance" | "The system shall process ≥ 95% of transactions in < 1 s" | IEEE 830-1998, §5.3.3 |
| "high throughput" | "The system shall sustain ≥ 20,000 requests per second per CPU core under workload A" | Apache APISIX benchmarks |
| "low latency" | "The p99 latency shall be < 10 ms for single-key lookups at 1M ops/s" | ScyllaDB SLA guidance |
| "fast response" | "Median response time ≤ 0.2 ms at 18,000 req/s/core" | Apache APISIX benchmarks |

### Canonical Example from IEEE 830-1998 (§5.3.3)

> *"All of these requirements should be stated in measurable terms. For example:*
> - **95% of the transactions shall be processed in less than 1 s.**
> - *rather than:*
> - *An operator shall not have to wait for the transaction to complete."*

### Real-World Specification Pattern

From ScyllaDB benchmark guidance for production SLAs [⁽¹⁾](https://docs.scylladb.com/manual/stable/operating-scylla/procedures/tips/benchmark-tips.html):

> *"Especially if your production depends on meeting the needs of the SLA. For example, the 99.99 percentile should have a latency less than 10ms."*

**Template for specifying performance:**

```
The system SHALL, under [workload profile X, data size Y, concurrent users Z]:
  (a) Achieve throughput ≥ [N] operations/second
  (b) Maintain p50 latency ≤ [A] ms
  (c) Maintain p95 latency ≤ [B] ms
  (d) Maintain p99 latency ≤ [C] ms
  (e) Maintain p99.9 latency ≤ [D] ms
```

---

## 2. "Scalable" / "Scale-Agnostic" → Specific Graph Size Benchmarks

### The Problem
"Scalable" without a concrete scale target is non-verifiable per IEEE 830-1998 §4.3.6. A system's scalability must be expressed against specific, testable graph dimensions.

### Measurable Replacements

| Vague Term | Measurable Replacement | Source |
|---|---|---|
| "scales to large graphs" | "The system shall process graphs with up to 10⁹ vertices and 10¹⁰ edges with query latency degradation ≤ 2× relative to the 10⁶-vertex baseline" | Derived from LDBC SNB scale factors |
| "scale-agnostic" | "Correctness guarantees hold for graphs from 10³ to 10¹⁰ edges without configuration changes" | TigerGraph benchmarks |
| "horizontal scalability" | "Throughput shall increase linearly (≥ 0.8× ideal) when cluster nodes are doubled from N to 2N, on a graph of 10⁹ edges" | ScyllaDB horizontal scaling |

### Concrete Scale Anchors from LDBC and Industry

The **LDBC Social Network Benchmark (SNB)** defines specific scale factors [⁽²⁾](https://ldbcouncil.org/benchmarks/snb/):

| Scale Factor | Vertices | Edges | Use Case |
|---|---|---|---|
| SF-1 | ~10⁴ | ~10⁵ | Unit testing, algorithm validation |
| SF-10 | ~10⁵ | ~10⁶ | Small-scale integration testing |
| SF-100 | ~10⁶ | ~10⁷ | Mid-scale performance testing |
| SF-10K | ~10⁸ | ~10¹⁰ | Large-scale production simulation |
| SF-30K | ~7.3×10¹⁰ | ~5.34×10¹¹ | Stress testing (TigerGraph benchmark) |

From TigerGraph [⁽³⁾](https://www.eastlinkcap.com/new-benchmark-shows-tigergraphs-capacity-to-handle-big-datasets/):
> *"TigerGraph's powerful graph analytics software was put to the test using the LDBC SNB Scale Factor 30k dataset, which features 36TB of raw data with 73 billion vertices and 534 billion edges."*

**Template for specifying scalability:**

```
The system SHALL:
  (a) Support graphs with V vertices and E edges where V ∈ [V_min, V_max] and E ∈ [E_min, E_max]
  (b) Demonstrate throughput scaling efficiency ≥ [X]% when dataset size grows 10×
  (c) Maintain p99 query latency ≤ [Y] ms at the maximum specified graph size
  (d) Handle graphs of [specific concrete size, e.g., 10⁹ edges] on [hardware profile]
```

---

## 3. "Near-Optimal" → Specific Optimality Gap Bounds

### The Problem
"Near-optimal" is ambiguous and non-verifiable. IEEE 830-1998 §4.3.2 requires that *"each characteristic of the final product be described using a single unique term."* The concept of approximation ratio/optimality gap provides the precise language needed.

### Measurable Replacements

| Vague Term | Measurable Replacement | Source |
|---|---|---|
| "near-optimal solution" | "The algorithm shall produce a solution within 5% of the optimal value, i.e., (ALG - OPT)/OPT ≤ 0.05" | Approximation ratio formulation |
| "close to optimal" | "The algorithm has an approximation ratio of c ≤ 1.5 (i.e., ALG ≤ 1.5 × OPT) in the worst case" | Approximation guarantee |
| "near-optimal partition" | "The partition shall achieve a modularity Q ≥ Q* − ε, where Q* is the maximum modularity found by any comparison algorithm" | Modularity-based bound |
| "approximately minimal" | "The cut size shall be ≤ α × OPT where α = [specific constant, e.g., 2.0]" | α-approximation |

### Formal Definition Pattern

From approximation algorithm theory:

> *"The factor c ≥ 1 is often referred to as the **approximation ratio** or the **approximation guarantee**. An algorithm is called a **c-approximation** if it always produces a solution whose value is within factor c of the optimum."* [⁽⁴⁾](https://ems.press/content/book-files/33324)

**Template for specifying optimality bounds:**

```
The algorithm SHALL guarantee:
  (a) An approximation ratio of c ≤ [value] for [objective function]
  (b) Formally: |f(ALG) - f(OPT)| / f(OPT) ≤ ε, where ε = [value]
  (c) The bound SHALL hold for all inputs satisfying [input constraints]
  (d) Empirically, on benchmark suite [X], the mean optimality gap shall be ≤ [Y]% with standard deviation ≤ [Z]%
```

### Granularity-Specific Formulation

When the objective involves community detection/partitioning:

```
The partition quality SHALL be measured by:
  (a) Modularity Q (Newman-Girvan), with the system achieving Q ≥ [threshold, e.g., 0.3]
  (b) OR Normalized Mutual Information (NMI) ≥ [value] relative to ground-truth communities
  (c) OR the modularity gap |Q_system - Q_benchmark| ≤ [ε] compared to [reference algorithm]
```

---

## 4. "Reasonable Community Granularity" → Measurable Granularity Metrics

### The Problem
"Reasonable" is inherently subjective and context-dependent. IEEE 830-1998 §4.3.2 notes that terms with multiple meanings must be defined in a glossary. Community granularity must be expressed through quantitative metrics with explicit measurement methods.

### Measurable Replacements

| Vague Term | Measurable Replacement | Source |
|---|---|---|
| "reasonable granularity" | "The number of detected communities k shall be in the range [k_min, k_max] with average community size in [s_min, s_max]" | Community size distribution |
| "fine-grained communities" | "The system shall detect communities with average size ≤ 50 nodes, with ≥ 80% of communities having size < 100" | Community size constraint |
| "balanced communities" | "The ratio of largest to smallest community size shall be ≤ [R], and no community shall contain > [P]% of total nodes" | Balance metric |
| "appropriate resolution" | "The resolution parameter γ shall be swept over [γ_min, γ_max], and the γ that maximizes modularity subject to k ∈ [k_min, k_max] shall be selected" | Leiden quality-resolution sweep |

### Concrete Granularity Metrics

**1. Resolution Parameter (γ) Sweep** [⁽⁵⁾](https://metricgate.com/docs/network-leiden-quality-resolution/):

> *"The Leiden algorithm sweeps the resolution parameter gamma across a user-specified range, exposing the quality-resolution tradeoff."*

**2. Community Size Distribution Constraints:**
- Number of communities: k ∈ [k_min, k_max]
- Average community size: μ_size ∈ [μ_min, μ_max]
- Community size standard deviation: σ_size ≤ [value]
- Minimum community size: |C_i| ≥ [min_size] for all communities i

**3. Modularity at a Given Resolution** [⁽⁶⁾](https://github.com/marbatlle/Optimize-Mod-Resolution):

> *"Determine an optimal modularity resolution parameter as well as the corresponding number of communities and average community size."*

**Template for specifying community granularity:**

```
The community detection SHALL satisfy:
  (a) Number of communities: k ∈ [k_min, k_max]
  (b) Average community size: ∈ [μ_min, μ_max] nodes
  (c) Modularity: Q ≥ [threshold] at the selected resolution γ*
  (d) Resolution selection: γ* = argmax Q(γ) subject to k(γ) ∈ [k_min, k_max]
  (e) Community size constraint: ≥ 90% of communities have size ∈ [s_min, s_max]
  (f) No community shall exceed [P]% of total graph nodes
```

---

## 5. "Descriptive Typed Errors" → Specific Error Message Format Requirements

### The Problem
"Descriptive" and "typed" are subjective. A specification must mandate a concrete, machine-parseable error format with defined fields, types, and content requirements.

### The Standard: RFC 9457 (Obsoletes RFC 7807)

RFC 9457 *"Problem Details for HTTP APIs"* [⁽⁷⁾](https://www.rfc-editor.org/info/rfc9457/) defines the standard, machine-readable error format:

> *"This document defines a 'problem detail' to carry machine-readable details of errors in HTTP response content to avoid the need to define new error response formats for HTTP APIs."*

### Required Fields (RFC 9457) [⁽⁸⁾](https://jsonic.io/guides/json-api-error-handling):

| Field | Type | Required | Description |
|---|---|---|---|
| `type` | URI | No (defaults to `"about:blank"`) | URI identifying the problem type; SHOULD resolve to human-readable documentation |
| `title` | string | No | Short, human-readable summary; SHOULD NOT change from occurrence to occurrence for a given type |
| `status` | integer | No | HTTP status code (repeated in the body); represents the closest human meaning |
| `detail` | string | No | Human-readable explanation specific to this occurrence (not the general type) |
| `instance` | URI | No | URI identifying the specific occurrence; may or may not yield information if dereferenced |

From [⁽⁹⁾](https://dev.to/apikumo/stop-inventing-your-own-api-error-format-use-rfc-9457-problem-details-4hma):

> *"The fields mean:*
> - *type — a URI identifying the kind of problem. It doubles as documentation.*
> - *title — a short, human-readable summary that stays constant for a given type.*
> - *status — the HTTP status code, repeated in the body for convenience.*
> - *detail — a human-readable explanation specific to this occurrence.*
> - *instance — a URI identifying the specific occurrence."*

### Specification Template

```
The system SHALL return error responses conforming to RFC 9457 (Problem Details for HTTP APIs):

  (a) Content-Type: application/problem+json
  (b) Every error response body SHALL contain:
      - "type": a URI identifying the problem type
      - "title": a short, human-readable summary constant for each type
      - "status": the HTTP status code as an integer
      - "detail": a specific explanation of this particular error occurrence
      - "instance": a URI identifying the specific occurrence (optional)
  (c) Each problem type URI SHALL resolve to documentation explaining:
      - What condition triggers this error
      - How a client can resolve it
      - What parameters or inputs are relevant
  (d) Error types SHALL be defined for at least the following conditions:
      - [list specific error conditions relevant to the system]
  (e) The "detail" field SHALL provide actionable guidance (what went wrong AND how to fix it)
  (f) Error responses SHALL be parseable by a machine client without string matching on the message text
```

### Concrete Error Type Registry (Example)

```
Error types SHALL include at minimum:
  | Type URI | Title | HTTP Status | Trigger |
  |----------|-------|-------------|---------|
  | https://api.example.com/errors/validation | Request validation failed | 400 | Input fails schema validation |
  | https://api.example.com/errors/not-found | Resource not found | 404 | Requested resource does not exist |
  | https://api.example.com/errors/rate-limited | Rate limit exceeded | 429 | Client exceeded allowed request rate |
  | https://api.example.com/errors/graph-too-large | Graph exceeds size limits | 413 | Input graph exceeds max vertices/edges |
```

---

## General Principles for Measurable Requirements

### From IEEE 830-1998

IEEE 830-1998 defines the characteristics of a good Software Requirements Specification [⁽¹⁰⁾](https://ieeexplore.ieee.org/document/720574):

1. **Unambiguous** — each requirement has only one interpretation; use single unique terms (§4.3.2)
2. **Verifiable** — every requirement must be checkable by a finite cost-effective process; non-verifiable examples: "works well," "good human interface," "shall usually happen" (§4.3.6)
3. **Complete** — full labels and references; definition of all terms and units of measure (§4.3.3)
4. **Consistent** — no subset of requirements conflicts (§4.3.4)
5. **Ranked** — requirements ranked by importance (essential/conditional/optional) and stability (§4.3.5)

### The VERIFIABILITY Test

Before finalizing any requirement, apply this test (from IEEE 830-1998 §4.3.6):

> *"A requirement is verifiable if, and only if, there exists some finite cost-effective process with which a person or machine can check that the software product meets the requirement."*

If no method can be devised to determine compliance, the requirement must be **removed or revised**.

### NASA's Approach

NASA SWE-050 [⁽¹¹⁾](https://swehb.nasa.gov/spaces/SWEHBVB/pages/32604503/SWE-050+-+Software-Requirements) formalizes this:

> *"The software technical requirements definition process is used to transform the baselined stakeholder expectations into **unique, quantitative, and measurable** technical software requirements."*

---

## Quick-Reference Conversion Table

| Vague Term | Measurable Replacement | Key Metric |
|---|---|---|
| "high-performance" | Throughput ≥ X ops/s; p99 latency ≤ Y ms | Throughput, latency percentiles |
| "scalable" | Correctness/performance at V vertices, E edges | Graph dimensions |
| "scale-agnostic" | No config changes needed across [min, max] graph sizes | Graph size range |
| "near-optimal" | Approximation ratio c ≤ [value]; optimality gap ≤ ε% | Optimality gap |
| "reasonable granularity" | Communities: k ∈ [k_min, k_max]; avg size ∈ [μ_min, μ_max] | Community size distribution |
| "descriptive errors" | RFC 9457 problem+json; type/title/status/detail/instance | Error schema compliance |
| "fast" | p50 ≤ X ms; p95 ≤ Y ms; p99 ≤ Z ms | Latency percentiles |
| "reliable" | Availability ≥ 99.9%; MTBF ≥ X hours; MTTR ≤ Y minutes | Availability metrics |
| "efficient" | Memory ≤ X GB for input size N; CPU-hours ≤ Y for workload Z | Resource bounds |
| "user-friendly" | Task completion rate ≥ X%; error rate ≤ Y%; SUS score ≥ Z | Usability metrics |

---

## References

1. ScyllaDB Benchmark Tips — [docs.scylladb.com](https://docs.scylladb.com/manual/stable/operating-scylla/procedures/tips/benchmark-tips.html)
2. LDBC Social Network Benchmark — [ldbcouncil.org/benchmarks/snb/](https://ldbcouncil.org/benchmarks/snb/)
3. TigerGraph Benchmark (73B vertices, 534B edges) — [eastlinkcap.com](https://www.eastlinkcap.com/new-benchmark-shows-tigergraphs-capacity-to-handle-big-datasets/)
4. Approximation Ratio/Guarantee Definition — [EMS Press](https://ems.press/content/book-files/33324)
5. Leiden Quality-Resolution Sweep — [metricgate.com](https://metricgate.com/docs/network-leiden-quality-resolution/)
6. Modularity Resolution Parameter Optimization — [GitHub: Optimize-Mod-Resolution](https://github.com/marbatlle/Optimize-Mod-Resolution)
7. RFC 9457: Problem Details for HTTP APIs — [rfc-editor.org](https://www.rfc-editor.org/info/rfc9457/)
8. JSON API Error Handling (RFC 9457 fields) — [jsonic.io](https://jsonic.io/guides/json-api-error-handling)
9. Stop Inventing Your Own API Error Format — [dev.to](https://dev.to/apikumo/stop-inventing-your-own-api-error-format-use-rfc-9457-problem-details-4hma)
10. IEEE Std 830-1998: Recommended Practice for Software Requirements Specifications — [ieeexplore.ieee.org](https://ieeexplore.ieee.org/document/720574)
11. NASA SWE-050: Software Requirements — [swehb.nasa.gov](https://swehb.nasa.gov/spaces/SWEHBVB/pages/32604503/SWE-050+-+Software-Requirements)

---

## Additional Sources

- **IEEE 29148-2018** / ISO/IEC/IEEE 29148:2018 — Systems and software engineering — Life cycle processes — Requirements engineering (successor to IEEE 830, maintains the verifiability principle)
- **Google SRE Book: Service Level Objectives** — [sre.google](https://sre.google/sre-book/service-level-objectives/) — Distinguishes SLI (indicator), SLO (objective), and SLA (agreement)
- **Apache APISIX Benchmarks** — [apisix.apache.org](https://apisix.apache.org/learning-center/page/2/) — 20,000+ req/s/core, 0.2ms median latency
- **Neo4j Operations Manual: Performance** — [neo4j.com/docs](https://neo4j.com/docs/operations-manual/current/performance/) — Throughput tuning factors
- **Apache Spark Structured Streaming** — Millisecond end-to-end latency (Spark 4.2+ Real-Time Mode)
- **CERN ScyllaDB Benchmark** — p95=126.9ms, p99=253.9ms, p99.9=364.6ms at scale

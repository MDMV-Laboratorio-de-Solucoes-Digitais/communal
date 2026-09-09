# Spec vs Implementation: CI Benchmark Gates Research

**Date**: 2026-09-08
**Question**: Do algorithm specifications and feature specs typically include CI infrastructure requirements like benchmark regression gates?

## Executive Summary

**NO** — it is NOT standard practice to include CI benchmark gates in algorithm specs. The boundary is clear:
- **Spec/NFRs**: Define *what* performance target must be met (e.g., "O(n log n)", "< 100ms for n=1000")
- **Implementation/CI**: Define *how* to verify it (which benchmarking framework, CI runner, regression threshold, statistical method)

---

## Finding 1: Algorithm Specifications Focus on WHAT, Not HOW

### Source: NASA Algorithm Publication Tool (APT) / ATBD Standard
- **URL**: ntrs.nasa.gov/api/citations/20200000737/downloads/20200000737.pdf
- Algorithm Theoretical Basis Documents (ATBDs) include: algorithm description, mathematical procedures, assumptions, input/output variables, scientific theory, sensitivity analysis
- **Do NOT include**: CI infrastructure, testing methodology, regression gates
- Standardized content model for ATBDs has ~30 metadata fields, zero related to CI/CD pipeline configuration

### Source: "Reproducibility by Construction" (2025)
- **URL**: researchgate.net/publication/399364989
- DOI: 10.61359/11.2206-2562
- Proposes integrating CI with algorithm specifications — but explicitly frames this as a **forward-looking proposal**, not current practice
- "implementable algorithm specifications" + "cloud/VM-backed continuous integration" = their novel contribution
- States current state is: "high-level algorithm descriptions without implementable detail"

> **LEAD**: Algorithm specs historically exclude CI concerns — emerging research proposes integration but this is not yet standard — APT/ATBD template shows what algorithm specs actually contain (math, inputs, assumptions, NOT CI)

---

## Finding 2: Non-Functional Requirements Capture TARGETS, Not CI Methods

### Source: Jama Software, Altexsoft, Perforce NFR guides
- Performance is a standard Non-Functional Requirement (NFR) category
- Good NFRs specify: response time thresholds, throughput targets, resource usage limits
- Bad NFRs specify: "use JMeter" or "run in GitHub Actions" — those are implementation decisions
- Jama: "Performance and scalability: What are the required response times, benchmark specifications, and other attributes"
- Altexsoft: "Specify the current workload for a measurement" — targets the metric, not the pipeline

### Source: Scaled Agile Framework (SAFE)
- NFRs include performance constraints but explicitly separate "constraints" from "solution architecture"

> **LEAD**: NFRs specify measurable performance targets (e.g., "95th percentile < 200ms at 1000 RPS") but NOT the CI tooling to verify them — Jama/Altexsoft/Perforce all teach that NFRs should be implementation-agnostic

---

## Finding 3: Rust Ecosystem — Benchmark Gates Are Implementation Concerns

### Source: Official Criterion.rs Documentation
- **URL**: bheisler.github.io/criterion.rs/book/user_guide/analysis_and_ci.html
- **Explicitly advises AGAINST relying on wall-clock benchmarks in virtualized CI** due to CPU frequency scaling and shared-host noise
- Recommends for CI: `cargo test --benches` (smoke test compilation/execution only)
- For regression testing: use deterministic instruction-counting (Iai/Iai-Callgrind) or bare-metal runners
- Key guidance: "Never block or fail PR builds on small performance deltas. Instead, compare branches as informational PR comments."

### Source: github-action-benchmark, Bencher, criterion-compare-action, hotpath.rs
- All treat benchmark regression as CI pipeline YAML configuration, NOT spec content
- Thresholds, alerting, and gating are configured in `.github/workflows/`, not in algorithm specifications
- Even the most aggressive setups recommend "raise alert via commit comment" not "fail the build"

> **LEAD**: Criterion.rs docs explicitly separate benchmark methodology (implementation) from performance targets (spec); recommends informational-only CI checks — Shows what the Rust community considers best practice

---

## Finding 4: leidenalg & igraph — No Spec-Level Benchmark Gates

### Source: leidenalg (vtraag/leidenalg GitHub, CRAN vignette)
- **URL**: cran.r-project.org/web/packages/leiden/vignettes/benchmarking.html
- Documentation covers: algorithm phases (move/refine/aggregate), quality functions, configuration options
- Benchmarking vignette: compares R vs Python timing on karate club graph (0.02s for 100 runs)
- **No mention of CI regression gates, automated performance testing, or benchmark thresholds in spec**
- Performance testing is ad-hoc and comparative (R vs Python), not gated

> **LEAD**: leidenalg benchmarks are comparative and informational, not part of a spec or CI gate — Direct look at how a Rust-relevant algorithm library handles performance documentation

---

## Finding 5: Research Papers Confirm the Boundary

### Source: "Algorithm-driven Development" (arxiv)
- **URL**: arxiv.org/html/2608.01533v1
- "ADD translates requirements into algorithmic flowcharts from which acceptance tests"
- Flowcharts are spec artifacts; acceptance tests derive from them
- Testing methodology is derived from spec but NOT part of spec

### Source: "Automated Benchmarking Pipeline for HPC Applications" (arxiv)
- **URL**: arxiv.org/html/2604.15919v1
- Treats benchmarking pipeline as infrastructure, not spec content
- Reproducibility is achieved through pipeline automation, not specification language

> **LEAD**: Research consistently treats benchmark infrastructure as separate from algorithm specification — Peer-reviewed evidence for the spec vs implementation boundary claim

---

## Finding 6: Requirements Engineering Best Practices

### Source: Jama Software "Characteristics of Excellent Requirements"
- Excellent requirements are: complete, correct, feasible, necessary, prioritized, unambiguous, verifiable
- They are NOT: implementation-prescriptive
- "How" decisions (tools, frameworks, pipelines) belong in design documents, not requirements

### Source: Martin Fowler "Practical Test Pyramid"
- **URL**: martinfowler.com/articles/practical-test-pyramid.html
- Unit tests have narrowest scope; different test types serve different purposes
- Performance tests are at the top of the pyramid — expensive, slow, not gating

> **LEAD**: Requirements engineering canon says specs should be verifiable but not implementation-prescriptive — Establishes the professional standard for what belongs in a spec vs design

---

## Answers to Research Questions

### 1. Do specs include CI/CD infrastructure requirements?
**NO.** Algorithm specs and feature specs define WHAT (correctness, performance targets, complexity bounds). CI/CD infrastructure (which runner, which tool, which threshold) is a design/implementation decision.

### 2. Is "benchmark regression gate" a spec concern or implementation concern?
**IMPLEMENTATION concern.** Specs may state a performance target ("O(n log n) community detection on graphs with 1M edges"). The regression gate (criterion.rs threshold, 5% regression = fail) is entirely a CI pipeline configuration.

### 3. What's the boundary between spec requirements and CI/infrastructure decisions?
**Spec = measurable targets. Implementation = how to verify them.** The spec says "what" and "how fast"; the CI config says "measure with this tool, on this hardware, alert at this threshold."

### 4. How do similar Rust algorithm libraries handle this?
- **Criterion.rs**: benchmarks in `benches/` directory, CI runs them informally (not blocking)
- **Petgraph/ndarray/nalgebra**: benchmark suites exist, CI runs them for regression visibility
- **leidenalg**: comparative benchmarking in documentation, no CI gate
- **No Rust algorithm library puts benchmark regression gates in the spec document**

---

## Conclusion

**Standard practice is to EXCLUDE CI benchmark gates from algorithm specs.**

| Include in Spec | Exclude from Spec |
|---|---|
| Performance NFRs | CI tooling |
| Complexity targets | Benchmark framework choice |
| Benchmark graphs/datasets | Regression thresholds |
| Acceptance criteria for correctness | Runner configuration |
| Measurable response time targets | Statistical methods for noise reduction |

This separation is consistent across: NASA ATBD standards, requirements engineering canon (Jama, SAFe), Rust ecosystem best practices (criterion.rs, petgraph), and academic research on reproducibility.

---

## Search Queries Executed

1. "algorithm specification CI benchmark gate examples"
2. "feature spec performance regression requirements examples"
3. "PRD performance benchmark requirements template"
4. "specification vs implementation concern CI infrastructure boundary"
5. "leidenalg python performance testing approach"
6. "igraph C performance regression testing CI"
7. "rust algorithm library benchmark CI petgraph ndarray nalgebra"
8. "performance test tier specification language best practices"
9. "criterion.rs CI integration documentation recommendations"
10. "when to specify CI requirements in spec best practices"
11. "software requirements specification performance benchmarks non-functional"
12. "github actions benchmark regression gate rust CI"
13. "algorithm specification document what to include examples"
14. "petgraph rust benchmark CI github actions regression"
15. "non-functional requirements specification performance threshold benchmark"
16. "feature specification performance requirements Rust library"
17. "algorithm specification include testing CI implementation details scope"
18. "scientific computing benchmark specification reproducibility CI pipeline"

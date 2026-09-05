# Feature Specification: Community Detection Framework

**Feature Branch**: `001-community-detection`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Specify the Communal Community Detection Framework: a modular library and runtime for graph community detection and hierarchical network partitioning. Performance targets: process LFR benchmark graphs (N=10k, μ=0.3) in < 5 seconds wall-clock on standard hardware (4-core x86_64 CPU, 16GB RAM, single-threaded hot loops; see SC-003, SC-004, SC-005)."

## Clarifications

### Session 2026-09-04 (5)

- Q: What does "stable" mean for community hierarchy subtrees between mutations (FR-015), and how should it be verified? → A: Subtrees whose members are all outside the 2-hop neighborhood of the mutated edge MUST have identical member sets, structure, and community IDs before and after the mutation (aligning with SC-006's verification criteria).

### Session 2026-09-04 (4)

- Q: How should the comparative metrics (NMI, ARI) API be structured—should it be a unified `MetricsCalculator` type with methods for each metric, or free functions following the existing `QualityMetric` trait pattern? → A: Free functions following `QualityMetric` trait pattern: `Nmi::evaluate(graph, partition1, partition2) -> f64`.
- Q: Should the spec explicitly require license compatibility verification for external dependencies (petgraph, rayon, thiserror, etc.), or is the existing dual-license statement (MIT OR Apache-2.0) sufficient? → A: Current dual-license statement is sufficient; no explicit compatibility check needed. **Remediated**: A "License Compatibility" section was added to the spec (after External Dependencies) explicitly stating the framework uses MIT OR Apache-2.0 per Constitution Principle VII, all dependencies use MIT-compatible licenses, no GPL/copyleft in default feature set, and CI SHOULD include a license check (e.g., cargo-deny).

### Session 2026-09-04 (3)

- Q: How should the streaming detector API expose mutation submission and result retrieval—should it be a dedicated `StreamingDetector` type that wraps the base `CommunityDetector`, or should streaming capabilities be added directly to the existing detector via feature flags? → A: Dedicated `StreamingDetector: CommunityDetector` trait with `apply_mutation`/`apply_mutations` methods.
- Q: What should the `StepCallback` trait's invocation contract specify for thread safety—should callbacks be required to implement `Send + Sync` or `Send` only? → A: `Send` only (single-threaded dispatch, allows mutable state in callback).
- Q: Should the `StepEvent` enum carry a `PhaseKind` variant to identify which algorithm phase emitted the event, or should phase information be encoded in the event variant names directly? → A: Phase encoded directly in variant names (e.g., `LocalMovingStart`, `RefinementSplit`, `AggregationContraction`).
- Q: How should observer registration work for the event emission system—should observers be registered via a `subscribe` method on the detector, or via a shared event bus? → A: `subscribe` method on detector returning a subscription handle (handle deregisters on Drop).
- Q: How should the spec distinguish internal quality functions from public metrics—should they share the same `QualityMetric` trait or use separate traits? → A: Same trait; internal metrics are `#[doc(hidden)]` and placed in `internal` module.

### Session 2026-09-04 (2)

- Q: Should the `QualityMetric` trait be defined as a formal trait in the spec with a standardized evaluation contract (input types, return type, and computation method), or should quality metrics remain as free functions documented per metric? → A: Define formal `QualityMetric` trait with `evaluate(&self, graph: &G, partition: &Partition) -> f64` method.
- Q: Should the user story acceptance scenarios be extended to include explicit alternate flows and exception flows (e.g., invalid input handling, algorithm failure, edge cases), or is the current primary-flow coverage sufficient for v1? → A: Extend all user stories with explicit alternate flows (e.g., different input formats) and exception flows (e.g., invalid input, algorithm failure).
- Q: How should the trait hierarchy be structured—should `MultilayerView` extend `GraphView` as a supertrait, or should it be an independent marker trait that types can implement separately? → A: `MultilayerView: GraphView` — multilayer graphs are always graphs, get all GraphView methods automatically.
- Q: Should domain error types be consolidated into a single `CommunalError` enum covering all failure modes, or split into per-module error types (GraphError, AlgorithmError, PartitionError, MetricsError) with a unified re-export? → A: Per-module error types (GraphError, AlgorithmError, PartitionError, MetricsError) with unified re-export via `#[error("...")]` conversions.
- Q: When the zero-unsafe policy is enforced, how should external dependencies that contain unsafe code be treated? → A: Zero-unsafe applies to framework code only; external dependencies (rayon, petgraph) are trusted as well-audited crates. External crates that break linting must use `#[expect()]` with reason.
- Q: For WASM deployment, are explicit security requirements needed for memory isolation and sandboxing beyond the browser's WASM sandbox? → A: Browser WASM sandbox is sufficient for isolation; WASM module must validate all graph input received from JavaScript at the boundary.
- Q: For property-based testing, should the spec define common invariants for all algorithms, algorithm-specific invariants, or both? → A: Both—common invariants for all algorithms (valid partition, no panics, determinism, finite scores, termination) plus algorithm-specific invariants per algorithm.
- Q: When the system runs in a WebAssembly deployment, how should graph data be passed between JavaScript and the WASM module? → A: Use `wasm-bindgen` with serialized data transfer (JSON/bytes) and explicit memory cleanup calls.
- Q: When the system runs in a WebAssembly deployment, should the single-threaded execution constraint be treated as a security requirement or a performance limitation? → A: Performance limitation - single-threaded reflects current WASM runtime constraints; multi-threading may be added later.
- Q: When a performance optimization requires unsafe code, what process should be followed to justify and approve it? → A: Require a formal RFC document with safety justification, isolation requirements, and mandatory code review.
- Q: When parsing graph files (EdgeList, JSON, GML), how should the system handle malformed input? → A: Fail fast with descriptive typed errors including line number, error type, and expected format.
- Q: When deterministic execution with a fixed seed yields suboptimal community quality compared to non-deterministic runs, what bound should the spec place on acceptable quality degradation? → A: Bounded degradation - Quality within 5% of best-known non-deterministic result (measured by target quality function).
- Q: Should the convergence threshold (default 1e-6) and mode (absolute/relative) apply uniformly to all quality functions (Modularity Q, CPM, Map Equation), or should each quality function be allowed to define its own default convergence criteria? → A: Function-specific defaults - Each quality function defines its own convergence criteria, defaulting to 1e-6 per canonical implementations (leidenalg, igraph). Users may set to 0 for strict convergence (theoretical guarantee: continue until no improvement) or configure function-specific thresholds for early termination.
- Q: What should be the expected behavior and output when running community detection on minimal graphs: a single node (no edges), a single edge (two nodes connected), and two disconnected nodes? → A: Explicit outputs defined - Single node → 1 community `[0]`; Single edge → 1 community `[0,0]`; Two disconnected nodes → 2 communities `[0,1]`. Modularity returns 0.0 when m=0 to avoid division by zero.
- Q: FR-017 currently enumerates these observable event types: phase starts, node relocations, refinement splits, aggregation contractions, and convergence plateaus. Are there any additional discrete algorithmic events that should be observable for debugging or TUI visualization? → A: Add high-value events - community merges (when two communities combine during aggregation), community splits (when a community divides, especially post-deletion), iteration boundaries (start/end of each local moving pass).
- Q: Looking at the success criteria (SC-001 through SC-015), which ones need more specific measurement methodologies to be objectively verifiable? Should the spec add explicit measurement methodologies to all success criteria, or only to those currently lacking? → A: Add Measurement Methodology subsections to ALL 15 SCs - follows ISO/IEC/IEEE 29148, ensures reproducibility, minimal overhead (2-4 lines per SC).
- Q: Should the typed algorithm config trait (FR-030) enforce that invalid configurations are caught at compile time through Rust's type system, or is runtime validation with descriptive errors sufficient? → A: Runtime validation with descriptive typed errors — aligns with existing `thiserror` convention; compile-time type-state enforcement would be over-engineering for dynamically-set hyperparameters.
- Q: The spec states the framework should be "scale-agnostic" and perform well "from small graphs to large-scale networks without scale-specific tuning." How should this be validated—should the spec add specific scale benchmarks as test targets, or keep it as a qualitative design goal? → A: Add specific scale benchmarks (1K, 100K, 1M nodes) as test targets with performance expectations — converts untestable assumption into measurable success criteria.
- Q: Should the spec include a traceability matrix mapping each functional requirement (FR-001 through FR-036) to at least one success criterion or acceptance scenario, or is the current cross-referencing sufficient? → A: Current cross-referencing is sufficient — checklist provides adequate FR-to-SC mapping; full traceability matrix would be documentation overhead without proportional value.
- Q: Should external dependencies (petgraph API, SNAP dataset formats, LFR benchmark specification) be documented with specific version numbers or reference URLs in the spec, or is naming them without versions sufficient? → A: Document with specific version numbers and reference URLs — prevents ambiguity, enables reproducible builds, and protects against upstream breaking changes.
- Q: Should the spec explicitly tag each assumption as "testable" (with validation method) or "design decision" (untestable)? → A: Explicitly tag each assumption — lightweight binary classification improves clarity and guides implementers on which assumptions need validation vs. acceptance.
- Q: When a user provides a graph with non-contiguous node identifiers, what should the framework expose as the internal indexing strategy? → A: Opaque dense contiguous index (u32/u64) with bidirectional mapping accessible only through query methods like `community_of(node_id)` — internal indices are hidden from users.
- Q: Should the `GraphView` trait expose a minimal core set of required methods plus provided convenience methods, or require all methods to be implemented? → A: Minimal core trait (node_count, edge_count, neighbors, edge_weight) with provided convenience methods (degree, has_edge, neighbor_count) having default implementations.
- Q: Should the `CommunityDetector` trait expose a single `detect` method or also require a `detect_into` method for pre-allocated buffers? → A: Single required `detect(&self, graph) -> Result<Partition, Error>` method; `detect_into(&self, graph, &mut Partition)` as a provided method with default implementation calling `detect`.
- Q: Should the `AlgorithmConfig` trait define common fields directly in the base trait or be a marker trait with no fields? → A: Base trait with common fields (convergence_threshold, convergence_mode, max_iterations, seed) as required methods plus a `validate() -> Result<(), Error>` method. Algorithm-specific configs extend this trait.
- Q: Should the `Partition` query API provide borrowed references to internal data or owned copies? → A: Borrowed references for individual queries (community_of, quality_score) plus owned bulk accessors (membership_vec, communities) for convenience.
- Q: For the Infomap algorithm's teleportation rate parameter, what should be the type, valid range, and default value? → A: `f64` type, valid range `[0.0, 1.0]`, default `0.15` (canonical Infomap default per Rosvall & Bergstrom 2008, corresponds to PageRank damping factor d = 0.85).
- Q: For the Label Propagation Algorithm (LPA), should the spec define a sync/async mode enum, and what is the behavioral difference between synchronous and asynchronous update modes? → A: Define `LpaUpdateMode` enum with `Asynchronous` (default, sequential node-order processing, canonical per Raghavan et al. 2007) and `SemiSynchronous` (color-class-based parallel updates with guaranteed convergence per Cordasco & Gargano). Synchronous mode excluded due to oscillation problems on bipartite graphs.
- Q: For the hierarchical community tree (FR-016), what should be the method signature and return type for requesting specific resolution levels? → A: Both index-based and threshold-based access: `at_level(level: usize) -> Partition` (level 0 = coarsest, increasing index = finer granularity) and `at_resolution(gamma: f64) -> Partition` (specifying gamma value for desired granularity).
- Q: For dynamic graph mutations (FR-013), what should be the API contract for submitting edge insertions and deletions? → A: Both single and batch mutation APIs: `apply_mutation(mutation: EdgeMutation) -> Result<Partition, Error>` for single mutations and `apply_mutations(mutations: Vec<EdgeMutation>) -> Result<Partition, Error>` for batch processing (processed sequentially, returns single updated partition).

### Session 2026-09-03 (1)

- Q: When an edge deletion partitions an existing community into disconnected subgraphs, should the system immediately split the community or flag it for deferred refinement? → A: Immediate split - Community is synchronously divided into connected components, preserving the connectedness guarantee at the moment of deletion.
- Q: How should algorithm-specific hyperparameters (Infomap teleportation rate, LPA sync/async, Fluid target k) be surfaced through the generic interface? → A: Typed algorithm configs - Each algorithm has its own config struct with required/optional params, unified by a common trait.
- Q: What is the default graph directionality assumption, and how should asymmetric edge weights be handled? → A: Undirected default - Graphs treated as undirected unless explicitly marked directed; asymmetric weights averaged for modularity, raw directed weights used for flow-based algorithms.
- Q: What is the interaction model for stepping mode—iterator or observer/callback? → A: Hybrid - Detector exposes both iterator (for sync library use) and callback (for async/TUI use) interfaces without degrading core performance.
- Q: What security posture should the framework adopt for graphs containing sensitive or proprietary data? → A: Zero-trust by default—no graph data or topology in logs/metrics. Optional, configurable, opt-in granular+tiered logging for users who need detailed algorithmic tracing.
- Q: What is the acceptable performance overhead when observability (stepping mode, event emission) is enabled? → A: Up to 20% overhead - Generous budget prioritizing observability detail and algorithmic transparency over raw speed.
- Q: When should the system reject graphs with negative edge weights? → A: Both - Validate at input time by default with option to defer validation for advanced use cases requiring flexible graph construction.

### Session 2026-09-03 (3)

- Q: When symmetrizing asymmetric edge weights for modularity optimization, which formula should be used to combine the two directed weights into a single undirected weight? → A: Arithmetic mean `(w_ij + w_ji) / 2` - Preserves original weight scale and follows Traag et al. (2019) convention.
- Q: Should the convergence threshold measure absolute or relative change in the quality function? → A: Both configurable - Algorithm config allows users to choose absolute or relative mode; defaults to absolute change (standard Leiden/Louvain convention).
- Q: Should TUI pedagogical explanations be static, dynamic, or hybrid? → A: Hybrid - Static pedagogical templates with interpolated algorithm state values, providing consistent educational framing with concrete context-aware details.
- Q: What should CLI "algorithm comparison" include? → A: Metrics table + pairwise NMI/ARI between algorithm outputs - Shows quality scores per algorithm plus similarity between detected community structures.
- Q: What should CLI "batch processing runs" support? → A: All modes - File batching (multiple files), algorithm batching (multiple algorithms on one file), and parameter sweeps (varying gamma, thresholds) through unified batch configuration.

### Session 2026-09-03 (4)

- Q: What is the distinction between FR-006 (non-Leiden algorithms MAY produce disconnected communities) and FR-024 (disconnected components MUST never be merged)? → A: A community may consist of multiple disconnected components (allowed as a structural outcome), but two distinct disconnected components of the original graph must not be placed in the same community (forbidden as a bug). This means algorithms may produce communities that are internally disconnected, but must never merge separate graph components into one community.
- Q: For incremental edge updates in FR-014, what does the O(k) complexity bound refer to? → A: Time complexity only - The O(k) bound refers to the time complexity of incremental updates, where k is the number of nodes in the 2-hop neighborhood of the mutated edge.
- Q: For SC-013 observability overhead measurement, what is the "default build" baseline? → A: Build with base event emission (stepping) but without `full-observability` feature flag - This isolates the overhead of detailed tracing (full observability) from the zero-cost base stepping mechanism.
- Q: FR-009 references FR-020 for graph builders, but FR-020 defines quality metrics. Should graph builders become a separate FR? → A: Yes, create FR-029 specifically for graph builders - This closes the numbering gap (FR-029 is currently missing) and provides clear requirements for graph builder functionality.
- Q: For SC-003 benchmark validation, where should reference partitions for SNAP datasets be sourced from? → A: Include reference partition data directly in `contracts/` directory - This ensures tests are reproducible without external dependencies and can run offline.

### Session 2026-09-03 (2)

- Q: Which algorithms should be included in the "lightweight" WASM build? → A: Leiden + Louvain for v1; future enhancement to provide a build customization tool allowing users to select their desired algorithm subset.
- Q: What pipeline capabilities should the CLI support beyond basic file I/O? → A: Full CLI - File I/O + quality metrics + algorithm comparison + format conversion + batch processing runs.

### Session 2026-09-03 (6)

- Q: What should the exact contract for `has_disconnected_communities()` specify regarding its return value semantics, traversal algorithm choice, and computational complexity? → A: Full contract: returns `true` if ANY community has disconnected subgraphs; uses BFS for traversal; complexity O(V+E) per community.
- Q: When no seed is provided to a stochastic algorithm phase, what should be the default behavior—random initialization each run, or a fixed default seed for reproducibility? → A: Fixed default seed (e.g., 42) for deterministic execution by default; users can override with custom seed.
- Q: How should self-loops affect quality metric computation (Modularity Q, CPM, Map Equation), in addition to their treatment in degree calculations? → A: Self-loops contribute 2x to degree (k_i += 2 * w_ii) and are included in quality metric self-loop term per Traag et al. (2019).
- Q: What conditions define a "convergence plateau" event that should be emitted separately from normal convergence detection? → A: Plateau = quality improvement below sub-convergence threshold (e.g., 1e-7) for N consecutive iterations; triggers event but not termination.
- Q: Which phases across the five algorithms (Leiden, Louvain, Infomap, LPA, Fluid) are stochastic and require seed control? → A: Explicit per-algorithm enumeration: Leiden (refinement sampling, tie-break), Louvain (node order, tie-break), Infomap (walk path, teleportation), LPA (update order, tie-break), Fluid (initial seeds, update order).

### Session 2026-09-03 (5)

- Q: For quality metrics (Modularity Q, CPM, Map Equation, NMI, ARI), should the mathematical formulas be included directly in the specification, or is referencing external academic papers sufficient? → A: Reference external papers + include brief formula summary in the spec — keeps spec self-contained for implementers while avoiding transcription errors.
- Q: How should the "zero-cost when disabled" requirement for observability be enforced? → A: Compile-time feature flag elimination via `full-observability` flag — guarantees zero cost for detailed tracing code by eliminating it at compile time, matching Constitution's feature flag pattern.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Guaranteed Connected Partitioning via Leiden (Priority: P1)

As a network researcher or AI engineer, I want to cluster large weighted or unweighted graphs such that every detected community is guaranteed to be internally connected, preventing disconnected sub-clusters from distorting community boundaries.

**Why this priority**: This is the core value proposition of the framework. The Leiden algorithm's primary advantage over Louvain is guaranteeing connected communities, which is essential for valid semantic clustering in downstream applications like GraphRAG.

**Independent Test**: Can be fully tested by running the Leiden algorithm on benchmark graphs and verifying via BFS/DFS traversal that every detected community forms an internally connected component. Delivers mathematically valid partitions that can be trusted for research and production use.

**Acceptance Scenarios**:

1. **Given** a weighted graph with known community structure, **When** the Leiden algorithm is executed with a fixed seed, **Then** every detected community is verified internally connected via BFS/DFS traversal.
2. **Given** the same input graph and fixed seed, **When** the algorithm is executed multiple times, **Then** the output partitions are bit-for-bit identical across all runs.
3. **Given** a graph and a resolution parameter gamma, **When** the algorithm is configured for CPM optimization, **Then** the partition quality is measured according to the Constant Potts Model metric.
4. **Given** a graph and a resolution parameter gamma, **When** the algorithm is configured for modularity optimization, **Then** the partition quality is measured according to Newman-Girvan modularity.

**Alternate Flows**:

- **Given** a graph file in EdgeList format, **When** loaded via the graph builder, **Then** the system parses it into the internal CSR representation and Leiden executes successfully.
- **Given** a graph file in JSON format, **When** loaded via the graph builder, **Then** the system parses it and Leiden executes successfully.

**Exception Flows**:

- **Given** a graph with negative edge weights and validation enabled, **When** the graph is constructed, **Then** the system returns a typed `GraphError::NegativeWeight` error without panicking.
- **Given** an empty graph (zero nodes), **When** Leiden is executed, **Then** the system returns an empty partition with quality 0.0 per FR-026.
- **Given** a malformed graph file, **When** parsing is attempted, **Then** the system returns a typed error with line number, error type, and expected format per FR-040.

---

### User Story 2 - Universal Graph Input & Partition Extraction (Priority: P1)

As a systems developer, I want to pass graph topologies into the framework and extract node-to-community assignments without managing internal array pointers or manual index conversions.

**Why this priority**: Usability is essential for adoption. Developers must be able to integrate the framework into existing pipelines without friction, regardless of their graph representation.

**Independent Test**: Can be fully tested by accepting various graph input formats (adjacency lists, edge lists with weights) and verifying that partition results are queryable by original node identifiers. Delivers a clean abstraction over graph data.

**Acceptance Scenarios**:

1. **Given** a graph represented as an adjacency list with node counts and floating-point edge weights, **When** the partition algorithm is executed, **Then** the system returns community assignments queryable by node ID.
2. **Given** a graph with non-contiguous integer node identifiers, **When** querying partition results, **Then** the system correctly maps original identifiers to community assignments without manual conversion.
3. **Given** a completed partition, **When** requesting summary statistics, **Then** the system returns community identifiers, community sizes, overall quality scores, and total community count.
4. **Given** a graph with zero-based node identifiers, **When** extracting partition results, **Then** the system handles indexing correctly without off-by-one errors.

**Alternate Flows**:

- **Given** a graph with string node identifiers, **When** the graph is constructed, **Then** the system maps them to dense contiguous indices internally and queries return correct community assignments.
- **Given** a graph with self-loops, **When** the partition algorithm executes, **Then** self-loops are handled per FR-027 (counted as 2x in degree) without infinite loops.

**Exception Flows**:

- **Given** a query for a non-existent node identifier, **When** `community_of(node_id)` is called, **Then** the system returns `None` without panicking.
- **Given** a graph with negative weights and validation disabled, **When** the algorithm executes, **Then** the system returns a typed error at algorithm execution time per FR-034.

---

### User Story 3 - Multi-Algorithm Clustering Spectrum (Priority: P2)

As a data scientist, I want to switch between different clustering paradigms (Leiden, Louvain, Infomap, Label Propagation Algorithm, and Fluid Communities) through a consistent interface so I can select the best algorithm for my domain.

**Why this priority**: Different domains require different clustering semantics. Providing multiple algorithms under a unified interface enables comparative analysis and domain-specific optimization.

**Independent Test**: Can be fully tested by running each algorithm on the same graph and verifying that results are produced according to each algorithm's theoretical properties. Delivers flexibility for diverse clustering needs.

**Acceptance Scenarios**:

1. **Given** a graph and selection of Louvain mode with its algorithm-specific configuration, **When** the algorithm executes, **Then** it optimizes modularity through local moving and aggregation without randomized refinement.
2. **Given** a graph and selection of Infomap mode with teleportation rate configuration, **When** the algorithm executes, **Then** it clusters based on flow compression (Map Equation) over random walks.
3. **Given** a graph and selection of LPA mode with sync/async tie-breaking configuration, **When** the algorithm executes, **Then** it performs diffusion-based label propagation for fast partitioning.
4. **Given** a graph and selection of Fluid Communities mode with target community count k, **When** the algorithm executes, **Then** it performs fluid density updates for linear-time partitioning.
5. **Given** partition results from two different algorithms and ground-truth labels, **When** comparing partitions, **Then** the system computes NMI and ARI similarity scores.

**Alternate Flows**:

- **Given** a graph with directed edges, **When** running Infomap, **Then** the algorithm uses raw directed weights for flow-based optimization per FR-031.
- **Given** Fluid Communities with k > n (requested communities exceed nodes), **When** the algorithm executes, **Then** it either raises a typed error or clamps k to n per FR-026.

**Exception Flows**:

- **Given** an invalid algorithm configuration (e.g., negative teleportation rate), **When** the algorithm is initialized, **Then** the system returns a typed `AlgorithmError::InvalidConfiguration` error per FR-030.
- **Given** an algorithm that fails to converge within max iterations, **When** execution completes, **Then** the system returns the best partition found with a convergence warning per FR-028.

---

### User Story 4 - Incremental Dynamic Graph Updates for GraphRAG (Priority: P2)

As an AI engineer maintaining an evolving knowledge graph, I want to insert or remove edges incrementally and receive updated community memberships without triggering a full graph recalculation. Node insertions and deletions are deferred to a future release.

**Why this priority**: Production GraphRAG systems operate on continuously evolving knowledge graphs. Batch-only algorithms are insufficient for real-time community awareness in LLM infrastructure.

**Independent Test**: Can be fully tested by mutating a graph (adding/removing edges) and verifying that only local boundaries are recalculated while unaffected community subtrees remain stable. Delivers streaming-capable community detection.

**Acceptance Scenarios**:

1. **Given** an existing partition and a new edge insertion, **When** the incremental update is triggered, **Then** only affected local boundaries within O(k) neighborhood are recalculated.
2. **Given** an existing partition and an edge deletion that partitions a community into disconnected subgraphs, **When** the incremental update is triggered, **Then** the community is immediately split into connected components to preserve the connectedness guarantee.
3. **Given** an existing partition and an edge deletion that does not disconnect any community, **When** the incremental update is triggered, **Then** unaffected subtrees in the community hierarchy remain stable.
4. **Given** a dynamic graph undergoing streaming edge mutations, **When** hierarchical tree slices are requested, **Then** the system provides coarse-to-fine community levels suitable for chunking text in LLM prompts.

**Alternate Flows**:

- **Given** a batch of edge insertions, **When** submitted as a batch, **Then** the system processes them sequentially and returns a single updated partition.
- **Given** a hierarchical tree slice request with a specific gamma threshold, **When** the system returns communities, **Then** the granularity matches the requested resolution per FR-016.

**Exception Flows**:

- **Given** an edge insertion that introduces a negative weight with validation enabled, **When** the mutation is submitted, **Then** the system returns a typed error without corrupting the existing partition.
- **Given** a mutation that would violate the connectedness guarantee, **When** the update is processed, **Then** the system rejects the mutation or applies corrective splits per FR-013.

---

### User Story 5 - Observability & Step-by-Step Algorithmic Stepping (Priority: P3)

As an educator or engineer debugging convergence, I want to inspect each phase of the algorithm (node moves, refinement splits, and community mergers) as discrete events or steps. The TUI provides force-directed graph visualization with pedagogical explanations generated from static templates with interpolated algorithm state values, providing consistent educational framing with concrete context-aware details.

**Why this priority**: Algorithmic transparency enables education, debugging, and trust. Users must understand how community frontiers evolve to diagnose convergence issues and validate results. Pedagogical explanations make the tool accessible for learning.

**Independent Test**: Can be fully tested by running the algorithm in stepping mode and verifying that discrete events are emitted for each phase transition, allowing pause-and-resume inspection. The TUI displays force-directed graph layout with didactic descriptions. Delivers algorithmic observability for education and debugging.

**Acceptance Scenarios**:

1. **Given** the algorithm is executing with observability enabled, **When** phase transitions occur, **Then** the system emits observable events denoting: phase starts, node relocations, refinement splits, aggregation contractions, and convergence plateaus.
2. **Given** the algorithm is in stepping mode via the iterator interface, **When** a discrete movement or aggregation pass completes, **Then** the iterator yields the step event and pauses until the user calls next() to resume.
3. **Given** the algorithm is executing with a registered callback observer, **When** each step completes, **Then** the callback is invoked with the step event, allowing external synchronization (e.g., TUI event loop).
4. **Given** an algorithm execution with full observability enabled, **When** the execution completes, **Then** the user can reconstruct the complete sequence of community evolution events.
5. **Given** the TUI is displaying algorithm execution, **When** a step completes, **Then** the force-directed graph layout updates to reflect current community assignments with pedagogical explanations (generated from static templates with interpolated algorithm state values) describing the visualized state.

**Alternate Flows**:

- **Given** the algorithm is executing with base observability only (no `full-observability` flag), **When** phase transitions occur, **Then** the system emits phase-level events without per-node state dumps, maintaining zero-cost stepping per FR-017.
- **Given** the iterator interface is used with a timeout, **When** the timeout expires before the next step completes, **Then** the iterator returns a timeout indicator without corrupting algorithm state.

**Exception Flows**:

- **Given** the algorithm is executing with observability disabled, **When** stepping mode is attempted, **Then** the system returns a typed error indicating observability must be enabled for stepping.
- **Given** a callback observer panics during event handling, **When** the next step completes, **Then** the system isolates the panic and continues algorithm execution without crashing.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST implement the Leiden algorithm with smart local move, randomized refinement, and aggregation phases.
- **FR-002**: The system MUST implement the Louvain algorithm with local moving and aggregation phases.
- **FR-003**: The system MUST implement the Infomap algorithm based on the Map Equation and Huffman-coded random walks.
- **FR-004**: The system MUST implement Label Propagation Algorithm (LPA) for diffusion-based clustering.
- **FR-005**: The system MUST implement Fluid Communities for fluid density-based partitioning.
- **FR-006**: The Leiden algorithm MUST produce internally connected communities verifiable via BFS/DFS traversal. Other algorithms (Louvain, Infomap, LPA, Fluid) MAY produce communities that are internally disconnected as a known characteristic of their design (a community may consist of multiple disconnected components). The `Partition` type MUST expose a `has_disconnected_communities() -> bool` method with the following contract: (1) Return value semantics: returns `true` if ANY community contains two or more disconnected subgraphs (i.e., the community is not internally connected), `false` if all communities are internally connected; (2) Traversal algorithm: uses BFS traversal to verify internal connectivity of each community; (3) Computational complexity: O(V+E) per community where V and E are the nodes and edges within that community. The method MUST be callable independently of algorithm execution for post-hoc partition validation.
- **FR-007**: All stochastic algorithm phases MUST accept an optional seed parameter (u64) for deterministic execution. When no seed is provided, algorithms MUST use a fixed default seed (42) to ensure reproducible results by default. Users MAY override with a custom seed value. Stochastic phases per algorithm: **Leiden** (refinement node sampling, tie-breaking in local moving), **Louvain** (node processing order, tie-breaking), **Infomap** (random walk path selection, teleportation events), **LPA** (asynchronous node update order, tie-breaking on equal labels), **Fluid Communities** (initial community seed selection, node update order).
- **FR-008**: The system MUST support a resolution parameter (gamma) for both Constant Potts Model (CPM) and modularity optimization. **Type**: `f64`. **Valid range**: `(0.0, +∞)`. **Default**: `1.0` (standard resolution per Traag et al. 2019). Applied in quality function formula as `Q = ... + γ * ...`.
- **FR-009**: The system MUST accept graph inputs in EdgeList, JSON, GML, and CSV formats with floating-point weights. CSV input uses edge-list format (`source,target,weight` per line). File parsers (EdgeList, JSON, GML, CSV) handle graph representations on disk; graph builders (see FR-029) accept adjacency list representations in-memory from programmatic API callers. Format schemas are defined in `contracts/` directory with the following files: `contracts/formats/edgelist.schema.md`, `contracts/formats/json-graph.schema.md`, `contracts/formats/gml.schema.md`, `contracts/formats/csv-graph.schema.md`. Each schema file MUST document: field definitions, required/optional markers, valid value ranges, and example valid/invalid inputs.
- **FR-010**: The system MUST emit partition results queryable by original node identifier. The framework uses an opaque dense contiguous index (u32 default, u64 feature-gated) internally; the bidirectional mapping between original node identifiers and internal indices is accessible only through query methods. Users cannot inspect or control internal index assignment directly. Non-contiguous and zero-based identifiers are mapped to a dense contiguous range (0..n-1) at graph construction time. Query methods: `community_of(node_id) -> Option<&CommunityId>`, `graph.node_index(original_id) -> Option<NodeIndex>`, `graph.node_id_at(index: NodeIndex) -> Option<&NodeId>`.
- **FR-011**: Partition results MUST include community identifiers, community sizes, overall quality scores, and community count.
- **FR-012**: ~~Reserved~~ — This requirement number is intentionally unused to preserve alignment with the original requirement taxonomy.
- **FR-013**: The system MUST support incremental updates for edge insertions and deletions without full recomputation. Node insertions and deletions are deferred to a future release. Edge deletion that disconnects a community MUST trigger immediate split into connected components (see FR-006 for connectivity verification). The dynamic graph API is provided via the `StreamingDetector` trait (see FR-045) which defines both single and batch mutation methods with fail-fast failure semantics (if any mutation fails, processing stops immediately; previous successful mutations are NOT rolled back).
- **FR-014**: Incremental edge updates MUST complete in O(k) time complexity, where k is the number of nodes within the 2-hop neighborhood of the mutated edge endpoint(s) (i.e., nodes reachable within two edge traversals from either endpoint). Only affected local boundaries within this neighborhood are recalculated.
- **FR-015**: The system MUST maintain stable community hierarchy subtrees between streaming mutations. **Stability contract**: Subtrees whose members are all outside the 2-hop neighborhood of the mutated edge MUST have identical member sets, structure, and community IDs before and after the mutation. This ensures unaffected communities are deterministically preserved, enabling incremental GraphRAG updates without global recomputation.
- **FR-016**: The system MUST provide hierarchical tree slices representing coarse-to-fine community levels. The API MUST support index-based access, resolution threshold-based access, and bulk access to all levels:
   - `at_level(level: usize) -> Partition`: Returns partition at the specified hierarchy level (level 0 = coarsest, increasing index = finer granularity). Invalid level index returns `PartitionError::InvalidLevel` with the valid level range.
   - `at_resolution(gamma: f64) -> Partition`: Returns partition at the specified resolution threshold (gamma value for desired granularity, used for LLM chunking use cases).
   - `levels() -> Vec<Partition>`: Returns all hierarchy levels as a vector (index 0 = coarsest, last index = finest). Useful for iterating over all granularities.
- **FR-017**: The system MUST emit observable progress events for: phase starts, node relocations, refinement splits, aggregation contractions, community merges (when two communities combine during aggregation), community splits (when a community divides, especially post-deletion per FR-013), iteration boundaries (start/end of each local moving pass), and convergence plateaus. A convergence plateau event is emitted when quality improvement remains below the plateau threshold for N consecutive iterations (default N=5). The plateau threshold is derived from the convergence threshold per FR-033's formula: `plateau_threshold = max(convergence_threshold / 10, 1e-8)` (defaulting to 1e-7 when convergence threshold is 1e-6). Plateau events are informational and do NOT trigger algorithm termination—only convergence detection (FR-033) or reaching the maximum iteration bound triggers termination. Base observability (event emission for stepping) is always available. Full observability (via `full-observability` feature flag) adds detailed tracing output including per-node state dumps and debug-level logging. Full observability code MUST be gated behind the `full-observability` feature flag and eliminated at compile time when disabled, ensuring zero cost for detailed tracing when the flag is not enabled. See FR-033 for the convergence/plateau threshold relationship.
- **FR-018**: The system MUST support stepping mode via both iterator (resumable) and callback (observer) interfaces. The `StepIterator` MUST implement `next(&mut self) -> Option<StepEvent>` matching Rust's standard `Iterator` trait, where `None` indicates normal completion. Errors are reported via a separate `status() -> Option<&Error>` method or stored in iterator state. The callback interface (`StepCallback`) handles error propagation separately.
- **FR-019**: The system MUST compute comparative metrics: Normalized Mutual Information (NMI) and Adjusted Rand Index (ARI). Comparative metrics follow a `ComparativeMetric` trait (distinct from `QualityMetric`): each metric is a struct implementing `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`. No separate `MetricsCalculator` type is introduced. Mathematical formulas for comparative metrics MUST be documented with both authoritative external references and brief formula summaries, consistent with FR-020. **Note**: The `ComparativeMetric` trait is defined in `communal-core` alongside `QualityMetric` (FR-021) to avoid circular dependencies.
- **FR-020**: The system MUST compute quality metrics: Modularity Q, Constant Potts Model, and Map Equation. Mathematical formulas for each quality metric MUST be documented with both an authoritative external reference (e.g., Traag et al. 2019 for Modularity, Rosvall & Bergstrom 2008 for Map Equation) and a brief formula summary in the spec or code documentation to ensure implementers have self-contained guidance. Metric contracts (formulas, valid ranges, default parameter values, edge case behaviors) are documented in `contracts/metrics.md`.
- **FR-021**: The system MUST define a `QualityMetric` trait in `communal-core` with the following contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`. This trait MUST be implemented by all quality metrics (Modularity Q, CPM, Map Equation) used for algorithm optimization. The trait enables generic code over metrics and provides a unified evaluation interface. Placing the trait in `communal-core` ensures both `communal-algo` (internal optimization) and `communal-metrics` (public evaluation) can implement it without circular dependencies, consistent with the constitution's dependency hierarchy. Internal quality functions (used exclusively for algorithm optimization and not intended for public evaluation) MUST be placed in an `internal` module and marked with `#[doc(hidden)]` to distinguish them from public metrics while sharing the same trait. **Note**: Comparative metrics (NMI, ARI per FR-019) use a separate `ComparativeMetric` trait (also in `communal-core`) since they require two partition inputs.
- **FR-022**: ~~Reserved~~ — This requirement number is intentionally unused to preserve alignment with the original requirement taxonomy.
- **FR-023**: Isolated nodes (degree 0) MUST each be assigned to their own unique single-member community. This is enforced at the algorithm level (communal-algo) since community assignment is an algorithmic decision; communal-core provides the structural primitives (empty graph detection) while algorithms implement the per-node behavior.
- **FR-024**: Two distinct disconnected components of the original input graph MUST never be merged into the same community. **Relationship to FR-006**: FR-006 defines the `has_disconnected_communities()` method and the Leiden connected-community guarantee; FR-024 extends this to ALL algorithms — incorrect merging of disconnected graph components is treated as a bug across the entire framework (not just Leiden). A single community MAY consist of multiple disconnected components (allowed structural outcome per FR-006), but separate graph components must not be incorrectly merged (forbidden).
- **FR-025**: Zero-weight edges MUST NOT trigger division-by-zero errors. Zero-weight edges are included in quality metric calculations as edges with weight 0.0 (no special-casing required; avoids division by zero via quality metric guards in FR-026).

- **FR-026**: The system MUST handle minimal graph edge cases with explicit expected outputs:

| Case | Input | Expected Output | Membership Vector |
|------|-------|-----------------|-------------------|
| Empty graph | 0 nodes, 0 edges | Empty partition, quality 0.0 | `[]` |
| Single node | 1 node, 0 edges | 1 community with 1 node | `[0]` |
| Single edge | 2 nodes, 1 edge | 1 community with 2 nodes | `[0, 0]` |
| Two disconnected | 2 nodes, 0 edges | 2 communities, 1 node each | `[0, 1]` |
| All isolated | n nodes, 0 edges | n communities, 1 node each (per FR-023) | `[0, 1, ..., n-1]` |

**Quality metric guards**: Modularity Q MUST return 0.0 when the graph has zero edges (m=0) to avoid division by zero; CPM and Map Equation are well-defined for all minimal cases. Fluid Communities MUST guard against k > n by clamping k to n at execution time (config validation error if k > n at construction time with validation enabled). Degenerate graph inputs MUST produce well-defined results: complete graph → 1 community (all nodes equivalent); fully disconnected → n communities (one per node); uniform weights → deterministic partition via tie-breaking (per FR-028).
- **FR-027**: Self-loops MUST be factored into degree calculations (k_i += 2 * w_ii, counting both endpoints) and MUST be included in quality metric self-loop terms per Traag et al. (2019). Self-loops MUST NOT cause infinite loops.
- **FR-028**: The system MUST avoid infinite oscillation in bipartite graphs and cyclic rings through deterministic tie-breaking logic (lexicographic node ordering by NodeId, with optional seed-based randomization when configured). Algorithm iteration complexity varies by algorithm. Per-iteration complexity is the cost of a single pass through the algorithm's main loop (one local moving pass, one diffusion pass, etc.). Total algorithm complexity is the per-iteration cost multiplied by the number of iterations until convergence (bounded by `max_iterations`, default 1000):
- **Leiden** (local moving, refinement, aggregation): `O(V + E)` per iteration — fast local move procedure visits only nodes whose neighborhood changed [Traag 2019].
- **Louvain** (local moving, aggregation): `O(V + E)` per iteration — standard local moving pass [Blondel 2008].
- **LPA** (diffusion pass): `O(V + E)` per iteration — near-linear label propagation [Raghavan 2007].
- **Fluid Communities** (density update pass): `O(V + E)` per iteration — linear fluid density diffusion between neighbors [Parșesan 2011].
- **Infomap** (flow optimization pass): `O(E)` per sweep — Louvain-style node moves with the Map Equation objective [Rosvall 2009; Blondel 2008]. Each sweep visits every edge a constant number of times; the algorithm performs repeated sweeps until convergence. The original 2008 greedy variant used agglomerative pair-merging with priority queues [Clauset 2004; Wakita 2007] achieving O(E log² V) for sparse networks, degrading to O(V² log V) with unbalanced merges. Exhaustive partition search is super-exponential (Bell numbers) and never used in practice. Algorithms MUST terminate via convergence detection (quality improvement below threshold) or maximum iteration bound (default: 1000 iterations), guaranteeing termination on any input graph. The maximum iteration bound MUST be configurable via algorithm-specific configuration.
- **FR-029**: The system MUST provide graph builder functionality accepting adjacency list representations in-memory from programmatic API callers. Graph builders MUST support a `validate_on_construction: bool` flag controlling whether validation (non-negative weights, CSR structure) runs at build time or is deferred. Graph builders MUST apply edge weight symmetrization eagerly at construction time for undirected graphs per FR-031.
- **FR-030**: The system MUST provide typed algorithm-specific configuration structs unified by a common trait, allowing algorithms to expose their unique hyperparameters. Invalid configurations MUST be detected at algorithm initialization time and reported via descriptive typed domain errors (per FR-036). Compile-time type-state enforcement is NOT required; runtime validation with `Result` returns is sufficient. Algorithm-specific hyperparameters MUST include:
  - **Infomap**: `teleportation_rate: f64` with valid range `[0.0, 1.0]` and default `0.15` (canonical default per Rosvall & Bergstrom 2008, corresponds to PageRank damping factor d = 0.85).
  - **LPA**: `update_mode: LpaUpdateMode` enum with `Asynchronous` (default, sequential node-order processing, canonical per Raghavan et al. 2007) and `SemiSynchronous` (color-class-based parallel updates with guaranteed convergence per Cordasco & Gargano). Synchronous mode is NOT supported due to oscillation problems on bipartite graphs.
  - **Fluid Communities**: `target_communities: usize` (target community count k) with valid range `[1, n]` where n is node count; default behavior clamps k to n if k > n.
- **FR-031**: Graphs MUST be treated as undirected by default; asymmetric edge weights MUST be symmetrized using arithmetic mean `(w_ij + w_ji) / 2` for modularity optimization while preserving raw directed weights for flow-based algorithms (Infomap) with ergodic flow preservation. Symmetrization MUST be applied eagerly at graph construction time when the undirected flag is set.
- **FR-032**: The system MUST adopt a zero-trust logging posture by default: no graph data, node identifiers, or topology information MAY appear in logs or metrics (applies to all output destinations including stdout and file per FR-037). Optional, configurable, opt-in logging MUST be available for users requiring detailed algorithmic tracing. Logging tiers MUST include at minimum: `error` (fatal errors only), `warn` (non-fatal issues), `info` (high-level phase transitions), `debug` (per-node moves and splits), and `trace` (full internal state dumps). Graph data, node identifiers, and topology information MUST NOT appear in any log tier unless explicitly enabled by the user.
- **FR-033**: The system MUST provide a configurable convergence threshold parameter. Each quality function (Modularity Q, CPM, Map Equation) defines its own convergence criteria, defaulting to 1e-6 per canonical implementations (leidenalg, igraph). **Two-threshold design**: The convergence threshold (1e-6) is the stopping criterion — when quality improvement falls below this value, the algorithm terminates. A separate, stricter plateau threshold is used for observability events (FR-017) that do NOT trigger termination. **Plateau threshold derivation**: `plateau_threshold = max(convergence_threshold / 10, 1e-8)`. This ensures the plateau threshold remains one order of magnitude below convergence threshold (defaulting to 1e-7 when convergence is 1e-6), with a floor of 1e-8 to prevent floating-point precision issues when users configure very small convergence thresholds. The plateau threshold requires persistence (N=5 consecutive iterations below threshold) to prevent false detection from single-iteration noise. This two-threshold design separates "stopping criterion" (convergence) from "observability signal" (plateau), following the pattern used in machine learning early stopping. Users MAY optionally set a function-specific threshold for early termination (e.g., 0 for strict convergence where the algorithm runs until no single-node move yields improvement). The convergence mode (absolute change `|Q_current - Q_previous|` or relative change `|Q_current - Q_previous| / |Q_current|`) MUST be configurable via algorithm-specific configuration, defaulting to absolute change. The default convergence behavior (1e-6 threshold) provides practical convergence to a partition with modularity within ε=0.01 of the best-known result for benchmark graphs (LFR N=10k), while avoiding floating-point noise issues.
- **FR-034**: The system MUST reject negative edge weights at input/construction time by default. An option to defer validation to algorithm execution time MUST be available for advanced use cases requiring flexible graph construction.
- **FR-035**: The system MUST design core traits (GraphView, CommunityDetector) to be forward-compatible with multilayer network support per Constitution Principle V, enabling non-breaking addition of multilayer capabilities in v1.1. A sealed `MultilayerView` marker trait MUST be added to the trait hierarchy as a placeholder for v1.1. Forward-compatibility is achieved via the sealed trait pattern (prevents downstream implementations) and supertrait design (allows future multilayer methods to be added as trait extensions without breaking existing implementors). Node insertions and deletions are explicitly deferred to v1.1; the sealed trait pattern ensures this addition is non-breaking.
- **FR-036**: All fallible operations MUST return typed domain errors using `thiserror`, covering at minimum: invalid graph input (malformed files, unsupported formats), negative edge weights (when validation enabled), algorithm convergence failure (non-convergence within iteration bound), and invalid algorithm configuration. Errors MUST be descriptive and enable programmatic handling by callers.
- **FR-037**: The system MUST support log output to stdout and file destinations, plus integration with the `tracing` subscriber ecosystem. The zero-trust prohibition (FR-032) applies to all output destinations. File output MUST support configurable path and rotation policies via a `RotationPolicy` enum with variants: `Never` (single unbounded file), `Size(max_bytes)` (rotate when file exceeds byte limit), `Daily` (rotate at midnight UTC), `Hourly` (rotate at top of hour). When rotation triggers, archived files MUST be named with timestamp suffix in the format `app.log.YYYY-MM-DD-HH-MM` (e.g., `app.log.2026-09-05-14-30`). The system MUST support `max_files: usize` to limit retained archived files (deleting oldest when exceeded). Stdout output MUST be line-oriented UTF-8 text suitable for piping and redirection without decorative formatting (no ANSI escape sequences, no progress bars, no Unicode drawing characters).
- **FR-038**: The WASM deployment (communal-wasm crate) MUST use `wasm-bindgen` for JavaScript integration. Graph data MUST be passed between JavaScript and WASM via serialized data transfer (JSON or binary bytes). The WASM module MUST expose explicit memory cleanup calls to free allocated memory after graph operations complete, preventing memory leaks in long-running web applications. The browser's WASM memory sandbox provides memory isolation and sandboxing boundaries; the spec does not require additional sandboxing. However, the WASM module MUST validate all graph input received from JavaScript at the boundary (structure, weights, node count limits) before processing to prevent malformed data from causing panics or incorrect results.
- **FR-039**: The system MUST enforce `#![deny(unsafe_code)]` across all library crates. Per rustc semantics, this lint is crate-local: it only fires on unsafe code within each framework crate, and external dependencies (e.g., rayon, petgraph) do NOT trigger this lint in framework crates regardless of their internal unsafe usage. Any exception for performance optimization in framework code MUST require a formal RFC document containing: (1) safety justification explaining why safe alternatives are insufficient, (2) isolation requirements specifying how the unsafe code is contained (item-level `#[allow(unsafe_code)]` with mandatory `reason = "..."`), and (3) mandatory code review approval before merging. Approved exceptions MUST be documented in a dedicated tracking file at `docs/unsafe-exceptions.md` with the RFC reference, affected modules, and review date.
- **FR-040**: File parsers (EdgeList, JSON, GML) MUST fail fast on malformed input with descriptive typed domain errors (per FR-036). Each error MUST include: the line number where the error occurred, the error type (e.g., invalid format, missing field, invalid value), and the expected format or valid range. Parsers MUST NOT silently skip or best-effort parse malformed entries.
- **FR-041**: The system MUST provide graph and partition serialization capabilities for JSON, CSV, and GML formats, complementing the file parsers (FR-040). Graph serializers MUST produce well-formed output conforming to the format schemas defined in the `contracts/` directory. Partition serializers MUST output community membership vectors, quality scores, and community metadata. All serializers MUST guarantee round-trip fidelity: serialized output MUST be re-parseable by the corresponding parser (FR-040) without data loss or corruption. Serialization errors MUST return typed domain errors (per FR-036) with descriptive messages indicating the failure cause (e.g., I/O error, invalid data state).
- **FR-042**: The CLI MUST provide the following commands with their high-level purposes: `run` (execute community detection on a graph file), `compare` (run multiple algorithms and produce metrics table + pairwise NMI/ARI), `batch` (process multiple files/algorithms/parameters via unified batch configuration), `convert` (convert between graph formats: JSON, CSV, GML), `metrics` (compute quality/comparative metrics for existing partitions), `generate` (generate synthetic benchmark graphs), `validate` (validate graph file structure and content). Detailed argument contracts (flags, options, defaults) are deferred to planning.

**CLI Output Formats per Command**: Each CLI command produces output in one or more serialization formats (FR-041). The default format is JSON for machine-readable piping; users MAY override via `--format <format>` flag. The following table defines the supported output formats per command:

| Command | Default Format | Supported Formats | Output Content |
|---------|----------------|-------------------|----------------|
| `run` | JSON | JSON, CSV, GML | Partition output (membership vector, quality scores, community metadata) |
| `compare` | JSON | JSON, CSV | Comparison report (per-algorithm quality metrics table + pairwise NMI/ARI matrix) |
| `batch` | JSON | JSON | Batch results (per-input summary, aggregated statistics, failed runs) |
| `convert` | JSON | JSON, CSV, GML | Converted graph in target format |
| `metrics` | JSON | JSON | Metrics report (quality scores, comparative metrics against ground truth if provided) |
| `generate` | JSON | JSON, CSV, GML | Synthetic graph output in target format |
| `validate` | JSON | JSON | Validation report (valid/invalid, error details with line numbers per FR-040) |

**Format selection rules**: When `--output <path>` is provided, the file extension (`.json`, `.csv`, `.gml`) overrides the `--format` flag if they conflict. When neither `--format` nor `--output` is specified, JSON is written to stdout. CSV output for partition data uses `node_id,community_id` columns. GML output conforms to the `contracts/formats/gml.schema.md` schema. All formats guarantee round-trip fidelity per FR-041.

- **FR-042b**: The batch command (`batch`) MUST accept a TOML configuration file via `--config <path>` (defined in plan.md CLI Argument Contracts). The TOML schema follows standard TOML v1.0 array-of-tables conventions (see https://toml.io/en/v1.0.0#array-of-tables) and supports the following structure:

**Batch TOML Config Schema**:

```toml
# Batch configuration for communal batch processing
# Follows TOML v1.0 array-of-tables convention

# Output directory for all batch results
output_dir = "./batch-results"

# Algorithms to run (applied to each input unless overridden)
algorithms = ["leiden", "louvain", "infomap"]

# Parameter sweep configuration (optional)
[parameters]
gamma_range = [0.5, 1.0, 2.0]        # Resolution parameter values to sweep
seed_range = [42, 123, 999]          # Seed values for deterministic sweep

# Input files array - each entry is a table in the array
[[input]]
file = "graphs/karate.edgelist"
algorithm = "leiden"                 # Optional: override algorithms for this input
gamma = 1.0                          # Optional: override gamma for this input

[[input]]
file = "graphs/dolphins.edgelist"
algorithm = "louvain"

[[input]]
file = "graphs/cora.json"
# Uses default algorithms and gamma from top-level config
```

**Schema contract**:
- `output_dir`: Required. String path. Directory created if it does not exist. Results are written as `<output_dir>/<input_stem>_<algorithm>.json`.
- `algorithms`: Required. Array of strings. Each value MUST be a valid algorithm identifier (`leiden`, `louvain`, `infomap`, `lpa`, `fluid`).
- `parameters`: Optional table. Defines parameter sweeps.
  - `gamma_range`: Optional array of `f64`. Resolution parameter values. Default `[1.0]`.
  - `seed_range`: Optional array of `u64`. Seed values for deterministic execution. Default `[42]`.
  - When sweep arrays contain multiple values, the batch runs the Cartesian product of all parameters × all algorithms × all inputs.
- `[[input]]`: Required. Array of tables (TOML array-of-tables syntax). Each entry specifies:
  - `file`: Required string. Path to graph file (EdgeList, JSON, GML, or CSV per FR-009).
  - `algorithm`: Optional string. Overrides `algorithms` for this specific input.
  - `gamma`: Optional `f64`. Overrides gamma for this specific input.
- Unknown keys MUST produce a typed `ConfigError::UnknownKey` error with the key name and line number (consistent with FR-040's fail-fast parsing contract).

**TUI Interface Contract**: The TUI (communal-tui crate) uses `ratatui` for rendering and `crossterm` for terminal control (per plan.md Technical Context). The interface contract defines the panel layout, event handling, and visualization behavior:

**Panel Layout**: The terminal is divided using ratatui's `Layout` constraint system (Direction::Horizontal / Direction::Vertical with Percentage/Min constraints):

```
┌────────────────────────┬───────────────────────┐
│                        │   Event Log (top)     │
│   Graph Visualization  │   scrollable list     │
│   (left, 60% width)    ├───────────────────────┤
│   Fruchterman-Reingold │   Statistics (bottom) │
│   force-directed layout│   algorithm metadata  │
│                        │                       │
├────────────────────────┴───────────────────────┤
│   Pedagogical Overlay (bottom, full width)     │
│   explanation text area                        │
└───────────────────────────────────────────────┘
```

- **Left panel (60% width, full height above overlay)**: Graph visualization using Fruchterman-Reingold force-directed layout. Nodes are colored by community assignment (distinct colors per community, using a categorical palette). Edges drawn as thin lines. The layout re-computes positions on each `NodeRelocation` or `AggregationContraction` event, with smooth interpolation between frames (animation via linear interpolation over ~10 frames at 30fps). Node size scales with degree (log-scaled). The 2-hop neighborhood of the most recently mutated edge is highlighted during streaming updates.
- **Right panel (40% width)**: Split horizontally into:
  - **Event Log (top, 50% height)**: Scrollable list of `StepEvent` variants with timestamps. Each entry shows: `[HH:MM:SS.ms] <EventType> <key_fields>`. Example: `[14:23:01.420] NodeRelocation { node: 42, from: 0, to: 1 }`. Supports keyboard scrolling (↑/↓, PageUp/PageDown) and auto-scroll toggle. Maximum 10,000 entries in memory (FIFO eviction).
  - **Statistics (bottom, 50% height)**: Algorithm metadata display showing: algorithm name, current iteration count, current quality score (6 decimal places), community count, convergence threshold, and convergence mode. Updates in real-time on each `IterationBoundary` or `ConvergenceDetected` event.
- **Pedagogical Overlay (bottom, full-width, 3-4 lines height)**: Explanation text area rendering pedagogical template output (see Pedagogical Template Structure below). Updates on each phase transition event. Text is word-wrapped and supports ANSI color for emphasis.

**TUI Event Loop Contract**: The TUI registers as a `StepCallback` observer (FR-046) on the detector. Events are dispatched via the callback interface and rendered in the ratatui event loop. The TUI MUST handle: terminal resize events (re-layout), keyboard input (pause/resume stepping, scroll event log, quit), and callback-dispatched `StepEvent` variants. When stepping mode is paused, the TUI displays "⏸ PAUSED" in the statistics panel and queues events for display on resume.

**Pedagogical Template Structure**: Per the hybrid pedagogical approach (clarification session 2026-09-03), explanations are generated from static string templates with interpolated algorithm state values. The template contract:

**Template format**: Static strings stored as const `&str` values in `communal-tui/src/pedagogy/`. Each template contains zero or more `{interpolation_points}` — curly-brace-delimited variable names that are replaced at runtime with current algorithm state.

**Interpolation variables**:
| Variable | Type | Description | Example Value |
|----------|------|-------------|---------------|
| `{algorithm_name}` | `&str` | Canonical algorithm name | `"Leiden"` |
| `{iteration}` | `usize` | Current iteration number | `7` |
| `{node_id}` | `NodeId` | Node identifier in current event | `42` |
| `{community_id}` | `CommunityId` | Community identifier | `1` |
| `{quality_delta}` | `f64` | Quality change in last step | `0.0032` |
| `{phase_description}` | `&str` | Human-readable phase name | `"local moving"` |
| `{from_community}` | `CommunityId` | Source community (for moves) | `0` |
| `{to_community}` | `CommunityId` | Target community (for moves) | `1` |
| `{community_count}` | `usize` | Current number of communities | `5` |

**Content guidelines**:
- Maximum 2 sentences per explanation.
- First sentence: explain WHAT happened (concrete event description).
- Second sentence: explain WHY it matters (algorithmic significance).
- Avoid jargon unless defined; prefer active voice.
- Use present tense for immediacy.
- Quality deltas are shown as `+0.0032` or `-0.0001` with sign.

**Example templates per phase type**:

```rust
// LocalMovingStart
"Starting {phase_description} pass {iteration}. The algorithm will try moving each node to a neighboring community to improve modularity."

// NodeRelocation
"Node {node_id} moved from community {from_community} to {to_community}. This swap improved partition quality by {quality_delta} — the node's neighbors are now mostly in the same community."

// RefinementSplit
"Community {community_id} split into sub-communities during refinement. Refinement ensures each community stays internally connected, which is Leiden's key guarantee."

// AggregationContraction
"The network contracted: {community_count} super-nodes formed from the previous level. Each super-node represents one detected community, and the algorithm repeats on this smaller graph."

// CommunityMerge
"Two communities merged into {community_id}. Merging happens when combining groups improves the overall quality score — fewer, larger communities emerge."

// CommunitySplit
"Community {community_id} split into parts. This typically happens after an edge deletion disconnects a community, preserving the connectedness guarantee."

// ConvergenceDetected
"Algorithm converged after {iteration} iterations with final quality {quality_delta}. No further node moves can improve the partition — the result is locally optimal."

// ConvergencePlateau
"Quality improvement has stalled for several iterations. The algorithm will continue until improvement drops below the convergence threshold."
```

**Template selection**: The TUI selects the template based on the `StepEvent` variant (one template per variant). If a template references an interpolation variable not present in the current event (e.g., `{node_id}` in a `ConvergenceDetected` event), the variable is rendered as `"—"` (em dash) as a fallback.

- **FR-043**: The trait hierarchy MUST follow these supertrait relationships: `MultilayerView: GraphView` (multilayer graphs are always graphs and inherit all `GraphView` methods). `CommunityDetector` is an independent trait (not a supertrait of `GraphView`). This ensures multilayer graphs automatically support all graph operations while keeping detection algorithms decoupled from graph representation.
- **FR-044**: Domain error types MUST be split into per-module error enums: `GraphError` (graph construction, validation, parsing), `AlgorithmError` (configuration, convergence, execution), `PartitionError` (query, invalid access), `MetricsError` (computation, comparison). Each error type uses `thiserror` with descriptive messages. A unified `CommunalError` re-export MAY be provided at the facade crate level for users who want a single error type, with `From` conversions for each per-module error.
- **FR-045**: The system MUST define a `StreamingDetector: CommunityDetector` trait for algorithms supporting incremental dynamic graph updates. This trait extends `CommunityDetector` with mutation methods:
   - `apply_mutation(&mut self, mutation: EdgeMutation) -> Result<Partition, Error>`: Applies a single edge insertion or deletion and returns the updated partition.
   - `apply_mutations(&mut self, mutations: Vec<EdgeMutation>) -> Result<Partition, Error>`: Applies a batch of mutations sequentially and returns a single updated partition. **Failure behavior**: If any mutation fails, processing stops immediately and returns the error. Previous successful mutations in the batch are NOT rolled back — the partition reflects partial progress up to the failed mutation. This fail-fast semantics avoids complex rollback logic and gives users visibility into partial application.
   The trait is opt-in; algorithms that do not support streaming do not implement it. The streaming detector maintains internal state (current partition, hierarchical tree) between mutations.
- **FR-046**: The system MUST provide an observer registration mechanism via a `subscribe` method on detectors supporting observability. The method signature is `fn subscribe(&mut self, observer: impl StepCallback + Send + 'static) -> Subscription`. The returned `Subscription` handle deregisters the observer when dropped (RAII pattern). Multiple observers MAY be registered; events are dispatched to all registered observers in registration order (FIFO guarantee). The `StepCallback` trait has a single bound: `Send` (not `Send + Sync`), allowing callbacks to mutate local state.

### API Design Specifications

The following subsections specify the concrete API contracts for petgraph integration, graph generators, the facade crate re-export pattern, newtype identifiers, the CSR graph type, and builder patterns. These specifications ensure consistency across crate boundaries and align with Rust ecosystem best practices (Rust API Guidelines, petgraph 0.8.3 conventions, and established facade patterns from tokio/serde/axum).

#### Petgraph Integration API

The `communal-petgraph` crate provides zero-copy and owned conversion between petgraph graph types and the internal `CsrGraph` representation. The integration targets petgraph 0.8.3 (https://docs.rs/petgraph/0.8).

**Supported petgraph types:**

| petgraph type | Module | Direction support | Notes |
|---------------|--------|-------------------|-------|
| `petgraph::Graph<N, E, Ty>` | `petgraph::graph` | `Directed` / `Undirected` | Adjacency list; compact indices; primary conversion target |
| `petgraph::StableGraph<N, E, Ty>` | `petgraph::stable_graph` | `Directed` / `Undirected` | Stable indices across removals; requires `stable_graph` feature |

`GraphMap` is excluded because its hash-table-backed node keys do not map efficiently to the dense contiguous index space required by `CsrGraph`. `MatrixGraph` and `petgraph::csr::Csr` are excluded from the v1 integration surface due to limited algorithm support in petgraph and format mismatch respectively.

**Conversion functions:**

- `impl<N, E, Ty: EdgeType, Ix: IndexType> From<petgraph::Graph<N, E, Ty, Ix>> for CsrGraph<u32, f64>` — Owned batch conversion. Consumes the petgraph `Graph`, produces a `CsrGraph`. Time complexity: **O(V + E)**. Symmetrizes edge weights per FR-031 when `Ty = Undirected`.
- `impl<'a, N, E, Ty: EdgeType, Ix: IndexType> From<&'a petgraph::Graph<N, E, Ty, Ix>> for CsrGraph<u32, f64>` — Borrowed view conversion. Borrows the petgraph `Graph`, produces a `CsrGraph` with copied edge data. Time complexity: **O(V + E)** in general; **O(1)** when the petgraph's internal storage is already CSR-contiguous (density permitting, detected via `edge_count() / node_count()²` threshold).
- `impl<N, E, Ty: EdgeType, Ix: IndexType> From<petgraph::StableGraph<N, E, Ty, Ix>> for CsrGraph<u32, f64>` — Owned conversion from `StableGraph`. Time complexity: **O(V + E)**. Gap indices from node removals are compacted to contiguous range.
- `impl<'a, N, E, Ty: EdgeType, Ix: IndexType> From<&'a petgraph::StableGraph<N, E, Ty, Ix>> for CsrGraph<u32, f64>` — Borrowed view from `StableGraph`. Time complexity: **O(V + E)**.

**Direction handling:** petgraph's `Directed` / `Undirected` type markers map to the `CsrGraph` directionality flag. `Undirected` petgraph graphs are stored as undirected (symmetric) in `CsrGraph`. `Directed` petgraph graphs set the directed flag; Infomap and other flow-based algorithms use raw directed weights (per FR-031).

**Feature gating:** All petgraph integration lives behind the `petgraph` feature flag on the `communal` facade crate. The `communal-petgraph` crate depends on `petgraph 0.8.3` and re-exports nothing at the facade level — users access conversion via `CsrGraph::from(petgraph_graph)`.

#### Generator API Contract

The `communal-generators` crate provides synthetic benchmark graph generators. All generators implement a common `Generator` trait and produce graphs conforming to `GraphView`.

**Common trait:**

```rust
pub trait Generator<G: GraphView> {
    fn generate(self) -> G;
}
```

Each generator has a dedicated config struct with typed fields, valid ranges, and sensible defaults. The config structs implement `Default` where canonical defaults exist.

**LFR Benchmark (Lancichinetti, Fortunato, Radicchi 2009):**

```rust
pub struct LfrConfig {
    pub n: usize,                // Number of nodes. Range: [1, usize::MAX]. Default: 1000.
    pub k: usize,                // Average degree. Range: [1, n-1]. Default: 15.
    pub max_k: usize,            // Maximum degree. Range: [k, n-1]. Default: 50.
    pub mu: f64,                 // Mixing parameter (fraction of inter-community edges). Range: [0.0, 1.0]. Default: 0.3.
    pub min_community: usize,    // Minimum community size. Range: [1, n]. Default: 20.
    pub max_community: usize,    // Maximum community size. Range: [min_community, n]. Default: 50.
    pub seed: Option<u64>,       // RNG seed for reproducibility. Default: None (uses fixed default 42 per FR-007).
}
```

**Stochastic Block Model (SBM):**

```rust
pub struct SbmConfig {
    pub n: usize,                // Number of nodes. Range: [1, usize::MAX]. Default: 1000.
    pub k: usize,                // Number of communities. Range: [1, n]. Default: 4.
    pub pin: f64,                // Intra-community edge probability. Range: [0.0, 1.0]. Default: 0.1.
    pub pout: f64,               // Inter-community edge probability. Range: [0.0, 1.0]. Default: 0.01.
    pub seed: Option<u64>,       // RNG seed for reproducibility. Default: None (uses fixed default 42).
}
```

**Barabási-Albert (Preferential Attachment):**

```rust
pub struct BaConfig {
    pub n: usize,                // Number of nodes. Range: [1, usize::MAX]. Default: 1000.
    pub m: usize,                // Number of edges to attach from each new node. Range: [1, n-1]. Default: 3.
    pub seed: Option<u64>,       // RNG seed for reproducibility. Default: None (uses fixed default 42).
}
```

**Erdős-Rényi (G(n, p) variant):**

```rust
pub struct ErConfig {
    pub n: usize,                // Number of nodes. Range: [0, usize::MAX]. Default: 1000.
    pub p: f64,                  // Edge probability. Range: [0.0, 1.0]. Default: 0.1.
    pub seed: Option<u64>,       // RNG seed for reproducibility. Default: None (uses fixed default 42).
}
```

**Validation:** Each config struct implements `validate(&self) -> Result<(), GraphError>` checking field ranges and constraints (e.g., `mu ∈ [0.0, 1.0]`, `min_community ≤ max_community`, `pin ≥ pout` for meaningful community structure). Validation runs at construction time when `validate_on_construction` is true (per FR-029).

**Feature gating:** Generators are gated behind the `generators` feature flag on the facade crate.

#### Facade Re-export Pattern

The top-level `communal` crate re-exports subcrate types via Cargo feature flags, following the facade pattern used by tokio, serde, and axum. This provides a single import surface for users while maintaining modular compilation.

**Top-level re-exports (always available):**

The following core types are re-exported at `communal::TypeName` regardless of feature flags:

| Type | Source crate | Re-export path |
|------|-------------|----------------|
| `GraphView` | `communal-core` | `communal::GraphView` |
| `CommunityDetector` | `communal-core` | `communal::CommunityDetector` |
| `Partition` | `communal-core` | `communal::Partition` |
| `AlgorithmConfig` | `communal-core` | `communal::AlgorithmConfig` |
| `ConvergenceMode` | `communal-core` | `communal::ConvergenceMode` |
| `CommunalError` (unified re-export) | `communal-core` | `communal::CommunalError` |

**Feature-gated re-exports:**

| Feature flag | Types re-exported | Source crate |
|-------------|-------------------|--------------|
| `core` (default) | `CsrGraph`, `NodeId`, `CommunityId`, `EdgeMutation` | `communal-core` |
| `algo` (default) | `Leiden`, `Louvain`, `Infomap`, `Lpa`, `Fluid`, `LeidenConfig`, `LouvainConfig`, `InfomapConfig`, `LpaConfig`, `FluidConfig` | `communal-algo` |
| `dynamic` | `StreamingDetector`, `HierarchicalTree` | `communal-dynamic` |
| `petgraph` | (conversion via `CsrGraph::from`) | `communal-petgraph` |
| `metrics` (default) | `Nmi`, `Ari`, `ModularityQ`, `Cpm`, `MapEquation`, `QualityMetric`, `ComparativeMetric` | `communal-metrics` |
| `generators` | `LfrConfig`, `SbmConfig`, `BaConfig`, `ErConfig`, `Generator` | `communal-generators` |
| `wasm` | WASM binding types | `communal-wasm` |
| `cli` | CLI argument types | `communal-cli` |
| `tui` | TUI stepping types | `communal-tui` |
| `full-observability` | Detailed tracing types | `communal-algo` |
| `u64-idx` | Switches `CsrGraph` default to `CsrGraph<u64, f64>` | `communal-core` |

**Naming convention:** Types are re-exported at their canonical name — `communal::CsrGraph`, `communal::Leiden`, `communal::Nmi` — not namespaced. This matches the tokio/serde convention where users write `tokio::spawn` or `serde::Serialize` without submodule prefixes. Internal modules are `#[doc(hidden)]`.

**Default feature set:** `default = ["core", "algo", "metrics"]` — provides the essential library surface without dynamic updates, petgraph integration, generators, WASM, CLI, TUI, or full observability.

**Feature unification:** When multiple dependency crates enable the same feature on `communal`, Cargo's unified feature set ensures a single compilation of each subcrate. The facade crate never enables features in subcrates directly — users opt in at the facade level.

#### Newtype Pattern Contracts

`NodeId` and `CommunityId` follow the Rust newtype pattern (per Rust API Guidelines C-NEWTYPE, C-DEREF, C-CONV) with `#[repr(transparent)]` for zero-cost layout guarantees.

**NodeId:**

```rust
#[repr(transparent)]
pub struct NodeId(NonZeroU32);  // Default; NonZeroU64 with u64-idx feature
```

- **Construction:**
  - `NodeId::new(id: u32) -> Option<Self>`: Returns `None` if `id == 0` (zero is reserved as sentinel). Valid range: `1..=u32::MAX`.
  - `NodeId::from_index(index: usize) -> Self`: Infallibly constructs from a dense contiguous index (shifts to 1-based internal representation). Panics if `index + 1 > u32::MAX`.
  - `NodeId::unsafe_new_unchecked(id: u32) -> Self`: `unsafe`, `#[doc(hidden)]` — for hot loops where the caller guarantees validity.

- **Validation:** Zero and out-of-range indices are rejected at construction. `NodeId::new(0)` returns `None` (zero is the `NonZeroU32` niche, enabling `Option<NodeId>` optimization).

- **Conversion:**
  - `From<NodeId> for usize`: Extracts the 1-based internal value as usize. Inverse of `from_index`.
  - `From<NodeId> for u32`: Extracts the raw 1-based value.
  - `TryFrom<u32> for NodeId`: Returns `Err` if input is 0.

- **Traits:** `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Display`. Does **not** implement `Deref` (per Rust API Guidelines C-DEREF: only smart pointers implement `Deref`).

**CommunityId:**

```rust
#[repr(transparent)]
pub struct CommunityId(NonZeroU32);  // Same pattern as NodeId
```

Follows the identical construction, validation, and conversion contract as `NodeId`. Community IDs are also 1-based internally (0 reserved for "unassigned" sentinel in partition algorithms).

**Layout guarantee:** `#[repr(transparent)]` ensures `sizeof::<NodeId>() == sizeof::<u32>()` and identical ABI passing. This allows direct transmutation to/from `u32` in unsafe-free code via `From` impls, and enables niche optimization (`Option<NodeId>` is 4 bytes).

#### CsrGraph Type Specification

`CsrGraph` is the internal Compressed Sparse Row graph representation used throughout the framework. It implements `GraphView` and provides efficient cache-friendly memory layout.

**Type definition:**

```rust
pub struct CsrGraph<N = u32, E = f64>
where
    N: IndexType,   // NodeId type (u32 default, u64 with u64-idx feature)
    E: EdgeWeight,  // Edge weight type (f64 default)
```

- `N`: The node index type parameter. Default `u32` maps to `CsrGraph<u32, f64>`. With the `u64-idx` feature flag, the default becomes `CsrGraph<u64, f64>` for graphs exceeding ~4 billion nodes.
- `E`: The edge weight type parameter. Default `f64` for floating-point weights. `EdgeWeight` trait bound ensures numeric operations for quality metrics.

**CSR storage layout:**
- `row_ptr: Vec<usize>` — length `n + 1`, where `row_ptr[i]` is the start index into `col_idx`/`weights` for node `i`. `row_ptr[n]` equals total edge count.
- `col_idx: Vec<usize>` — length `2m` (undirected) or `m` (directed), column indices of adjacent nodes.
- `weights: Vec<E>` — length matches `col_idx`, edge weights parallel to column indices.
- `directed: bool` — directionality flag (undirected by default per FR-031).
- `node_count: usize` — cached for O(1) access.
- `edge_count: usize` — cached for O(1) access.

**Construction:**

```rust
impl<N, E> CsrGraph<N, E> {
    pub fn from_edges(iter: impl IntoIterator<Item = (usize, usize, E)>) -> Self;
    pub fn from_adjacency(adj: &[Vec<(usize, E)>]) -> Self;
    pub fn from_builder(builder: GraphBuilder) -> Result<Self, GraphError>;
}
```

- `from_edges`: Builds CSR from an iterator of `(source, target, weight)` triples. Time: **O(V + E log E)** due to sorting for CSR construction.
- `from_adjacency`: Builds CSR from an adjacency list slice. Time: **O(V + E)**.
- `from_builder`: Builds from a `GraphBuilder` with validation and symmetrization flags. Returns `GraphError` if validation fails.

**Access methods (inherits from GraphView, plus):**
- `node_count() -> usize` — O(1), from `GraphView`.
- `edge_count() -> usize` — O(1), from `GraphView`.
- `neighbors(node: usize) -> &[usize]` — slice of adjacent node indices. O(1) to return slice, O(d) to iterate d neighbors.
- `edge_weight(from: usize, to: usize) -> Option<&E>` — O(d) binary/linear search within neighbor slice.
- `is_directed() -> bool` — O(1) directionality check.

**Memory:** O(V + E) space. Cache-friendly for sequential neighbor iteration (col_idx/weights are contiguous per row). Suitable for SIMD vectorization of neighbor scans.

#### Builder Pattern Consistency

All builder types across the framework use a consistent consuming-builder pattern (not `&mut self`). This enables method chaining and aligns with the "clean one-liners" approach from Rust builder best practices.

**Common builder trait:**

```rust
pub trait Builder {
    type Output;
    type Error;
    fn build(self) -> Result<Self::Output, Self::Error>;
}
```

All builders consume `self` (not `&mut self`), returning `Result<T, Error>` from `build()`. This prevents reuse of partially-configured builders and enables fluent chains.

**GraphBuilder:**

```rust
pub struct GraphBuilder {
    edges: Vec<(usize, usize, f64)>,
    validate_on_construction: bool,
    symmetrize: bool,
}

impl GraphBuilder {
    pub fn new() -> Self;                              // Defaults: validate=true, symmetrize=true
    pub fn with_edges(edges: Vec<(usize, usize, f64)>) -> Self;
    pub fn validate_on_construction(self, flag: bool) -> Self;  // Consuming setter
    pub fn symmetrize(self, flag: bool) -> Self;                // Consuming setter
    pub fn build(self) -> Result<CsrGraph<u32, f64>, GraphError>; // Per FR-029
}
```

**AlgorithmConfig builders (per FR-030):**

Each algorithm config struct provides `new() -> Self` (with canonical defaults) and consuming field setters:

```rust
impl LeidenConfig {
    pub fn new() -> Self;                          // Defaults: convergence_threshold=1e-6, max_iterations=1000, seed=Some(42), gamma=1.0
    pub fn convergence_threshold(self, val: f64) -> Self;
    pub fn max_iterations(self, val: usize) -> Self;
    pub fn seed(self, val: Option<u64>) -> Self;
    pub fn gamma(self, val: f64) -> Self;
    pub fn build(self) -> Result<Leiden, AlgorithmError>;  // Validates config, constructs detector
}
```

Same pattern for `LouvainConfig`, `InfomapConfig`, `LpaConfig`, `FluidConfig` with their respective hyperparameters (e.g., `InfomapConfig::teleportation_rate(self, val: f64) -> Self`, `FluidConfig::target_communities(self, val: usize) -> Self`).

**MetricsBuilder:**

```rust
pub struct MetricsBuilder<'a, G: GraphView> {
    graph: &'a G,
    partition: &'a Partition,
    metrics: Vec<MetricKind>,
}

impl<'a, G: GraphView> MetricsBuilder<'a, G> {
    pub fn new(graph: &'a G, partition: &'a Partition) -> Self;
    pub fn with_metric(self, metric: MetricKind) -> Self;     // Consuming, additive
    pub fn with_metrics(self, metrics: Vec<MetricKind>) -> Self;
    pub fn build(self) -> Result<MetricsReport, MetricsError>; // Computes selected metrics
}
```

**Consistency rules:**
1. All setters consume `self` and return `Self` (fluent API).
2. `build()` is the terminal method returning `Result<T, Error>`.
3. Validation happens in `build()`, not in setters (setters are infallible).
4. `new()` provides sensible defaults; users override via setters.
5. No `&mut self` setters — this prevents accidental reuse of intermediate builder state.

### Key Entities

- **ConvergenceMode**: Enum defining how convergence is measured. Variants: `Absolute` (convergence when `|Q_current - Q_previous| < threshold`) and `Relative` (convergence when `|Q_current - Q_previous| / |Q_current| < threshold`). Default: `Absolute`.

- **StepCallback**: See FR-046 for the trait definition and observer registration mechanism.

- **Graph**: Represents a network topology with nodes and weighted edges. Contains node identifiers and edge connections with optional floating-point weights. Has a directionality flag (undirected by default). The `GraphView` trait defines the core contract with required methods (`node_count`, `edge_count`, `neighbors(node)`, `edge_weight(from, to)`) and provided convenience methods (`degree(node)`, `has_edge(from, to)`, `neighbor_count(node)`) that have default implementations. Implementations MUST provide O(1) neighbor iteration (O(1) amortized per `next()` call; O(d) total for d neighbors). The trait MUST NOT panic on valid inputs; invalid node IDs return `None` or `Result::Err`.
- **Partition**: Represents the output of a community detection algorithm. Maps each node to a community and provides quality metrics. Query API provides borrowed references for individual lookups (`community_of(node_id) -> Option<&CommunityId>`, `quality_score() -> f64`) and owned bulk accessors (`membership_vec() -> Vec<CommunityId>`, `communities() -> Vec<&[NodeId]>`, `community_sizes() -> &[usize]`).
- **Community**: Represents a cluster of nodes. Has an identifier, member nodes, and internal connectivity properties.
- **Resolution Parameter (gamma)**: Controls the granularity of detected communities. Higher values yield more, smaller communities.
- **Quality Metric**: Measures partition quality. Includes Modularity Q, CPM, Map Equation, NMI, and ARI. The `QualityMetric` trait (FR-021) is defined in `communal-core` and provides the evaluation contract: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`. Both `communal-algo` (internal optimization) and `communal-metrics` (public evaluation) implement this trait. All optimization quality metrics (Modularity Q, CPM, Map Equation) implement this trait. The internal dispatch enum in `communal-algo` (used for zero-cost algorithm optimization) is named `QualityFunction` to distinguish it from the public trait.
- **CommunityDetector**: Base trait for all community detection algorithms. Required method: `detect(&self, graph: &G) -> Result<Partition, Error>`. Provided method: `detect_into(&self, graph: &G, partition: &mut Partition)` with default implementation. All five algorithms implement this trait. Errors are typed domain errors (per FR-036).
- **Algorithm Configuration**: Base trait (`AlgorithmConfig`) defining common fields as required methods: `convergence_threshold(&self) -> f64`, `convergence_mode(&self) -> ConvergenceMode`, `max_iterations(&self) -> usize`, `seed(&self) -> Option<u64>`, and `validate(&self) -> Result<(), Error>`. Each algorithm (Leiden, Louvain, Infomap, LPA, Fluid) provides its own config struct implementing this trait, extending it with algorithm-specific hyperparameters.
- **Hierarchical Tree**: Represents multi-level community structure from coarse to fine granularity, supporting incremental updates. Provides `at_level(level: usize) -> Partition` for index-based access, `at_resolution(gamma: f64) -> Partition` for resolution threshold-based access, and `levels() -> Vec<Partition>` for bulk access to all hierarchy levels.
- **EdgeMutation**: Represents a dynamic graph mutation with variants `Insertion { source: NodeId, target: NodeId, weight: f64 }` and `Deletion { source: NodeId, target: NodeId }`.
- **Observable Event**: See `StepEvent` enum definition above.
- **StepIterator**: Resumable iterator interface for synchronous stepping through algorithm execution. See FR-018 for the contract.
- **Step Callback**: Observer callback interface for asynchronous event handling (e.g., TUI integration). Trait bound: `Send` only (not `Send + Sync`), allowing callbacks to mutate local state. Registered via `subscribe` method on detector; deregistered when `Subscription` handle is dropped.
- **AlgorithmPhase**: Enum identifying which algorithm phase emitted an event. Variants: `LocalMoving`, `Refinement`, `Aggregation`, `Convergence`. Used by `IterationBoundary` events to identify the phase in a type-safe manner.

- **StepEvent**: Enum representing discrete algorithmic events. Variants use phase-encoded names (not a separate `PhaseKind` field): `LocalMovingStart { iteration: usize }`, `LocalMovingEnd { iteration: usize }`, `NodeRelocation { node: NodeId, from: CommunityId, to: CommunityId }`, `RefinementSplit { community: CommunityId, into: usize }`, `AggregationContraction { from_communities: usize, to_communities: usize }`, `CommunityMerge { into: CommunityId, merged: CommunityId }`, `CommunitySplit { from: CommunityId, into: Vec<CommunityId> }`, `IterationBoundary { phase: AlgorithmPhase, iteration: usize }`, `ConvergencePlateau { iterations_below_threshold: usize }`, `ConvergenceDetected { total_iterations: usize, final_quality: f64 }`.
- **StreamingDetector**: Trait extending `CommunityDetector` for algorithms supporting incremental dynamic graph updates. Provides `apply_mutation` and `apply_mutations` methods. Maintains internal state between mutations. Opt-in; not all algorithms implement this trait.
- **Subscription**: RAII handle returned by `subscribe` method. Deregisters the observer when dropped. Implements `Drop` trait to remove the observer from the detector's observer list. Methods: `fn is_active(&self) -> bool` (check if subscription still active). Cannot be cloned (unique ownership).
- **Domain Errors**: Per-module error types using `thiserror`: `GraphError` (construction, validation, parsing failures), `AlgorithmError` (configuration, convergence, execution failures), `PartitionError` (query, invalid access), `MetricsError` (computation, comparison failures). Each error variant includes descriptive messages enabling programmatic handling. A unified `CommunalError` re-export MAY be provided at the facade crate level (FR-044).

## Success Criteria *(mandatory)*

### Measurable Outcomes

#### SC-001: Connected Communities Guarantee
The Leiden algorithm produces internally connected communities for 100% of test cases across benchmark graphs (verified via BFS/DFS).

**Measurement Methodology:**
- **Test Input**: Zachary Karate Club, Dolphins, Cora, Enron, LFR benchmarks (N=1k, 10k, 100k), and minimal graphs (single node, single edge, two disconnected nodes)
- **Procedure**: Run Leiden algorithm; for each detected community, perform BFS traversal from an arbitrary member; verify all community members are reachable
- **Pass Threshold**: 100% of communities across all test graphs are internally connected
- **Sample Size**: 5 benchmark graphs × 10 seeds × 3 graph sizes = 150 test cases

#### SC-002: Deterministic Execution
Execution with a fixed seed produces identical partitions across 100 independent runs on the same input.

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3), Zachary Karate Club, and a random graph (N=1000)
- **Procedure**: Run algorithm 100 times with fixed seed=42; compare all partition pairs for bit-for-bit equality
- **Pass Threshold**: 100% of run pairs produce identical membership vectors
- **Sample Size**: 3 graphs × 100 runs = 300 executions

#### SC-003: Benchmark Graph Partitioning
The system correctly partitions standard benchmark graphs (Zachary Karate Club, Dolphins, Cora, Enron) with results matching published ground truth. Reference partition data MUST be included directly in the `contracts/` directory as self-contained test fixtures (ensuring reproducibility and offline execution). Reference partition sources and provenance:

| Dataset | Publication | Ground Truth Definition | Reference File |
|---------|-------------|------------------------|----------------|
| **Zachary Karate Club** | Zachary (1977) [8] | Club fission into 2 factions after administrative dispute | `contracts/reference-partitions/karate-club.membership` |
| **Dolphins** | Lusseau et al. (2003) [9] | Dolphin social network: 2 communities based on frequent associations | `contracts/reference-partitions/dolphins.membership` |
| **Cora** | McCallum et al. (2000) [10] | Citation network communities (subject categories) | `contracts/reference-partitions/cora.membership` |
| **Enron** | Klimt & Yang (2004) [11] | Email network: organizational/departmental structure | `contracts/reference-partitions/enron.membership` |

**Note on ground truth quality**: Real-world benchmarks have noisy or incomplete ground truth (unlike synthetic LFR/SBM benchmarks with exact planted partitions). The NMI >= 0.80 threshold represents "good agreement with the best-known reference partition" and is consistent with literature standards for real-world network evaluation [Lancichinetti & Fortunato 2009].

**NMI variant**: Arithmetic mean normalization (`NMI = 2*I(X,Y) / (H(X) + H(Y))`) is used because it is the most widely adopted variant in the community detection literature, enabling direct comparison with published results. Note that recent research [Jerdee et al. 2023] demonstrates that symmetric NMI normalization introduces bias toward labelings with more groups; the arithmetic mean variant is chosen for interoperability with existing benchmarks, but internal validation MAY use the reduced mutual information variant for unbiased evaluation.

**Measurement Methodology:**
- **Test Input**: Reference partitions from `contracts/` directory (see table above for file paths)
- **Procedure**: Run algorithm on each benchmark; compute NMI (arithmetic mean normalization) between detected and reference partitions
- **Pass Threshold**: NMI >= 0.80 for each benchmark graph (consistent with "good recovery" threshold in literature for real-world networks)
- **Sample Size**: 4 benchmark graphs × 10 seeds = 40 comparisons

#### SC-004: Synthetic Benchmark Validation
Synthetic benchmarks with known ground truth (LFR, SBM) achieve NMI >= 0.95 for algorithm validation.

**Measurement Methodology:**
- **Test Input**: LFR benchmarks (N=10k, μ=0.1, 0.2, 0.3), SBM (N=10k, 4 communities, pin=0.1, pout=0.01)
- **Procedure**: Generate synthetic graphs with known communities; run algorithm; compute NMI against ground truth
- **Pass Threshold**: NMI >= 0.95 for μ <= 0.3; NMI >= 0.70 for μ <= 0.5
- **Sample Size**: 5 μ values × 10 graph instances × 10 seeds = 500 executions per algorithm

#### SC-005: Incremental Update Complexity
Incremental updates complete within O(k) time complexity where k is the number of nodes in the 2-hop neighborhood of the mutated edge (as defined in FR-014).

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3)
- **Procedure**: Perform 1000 random edge insertions and 1000 random edge deletions; record update time and k (2-hop neighborhood size); perform least-squares linear fit of time vs. k
- **Pass Threshold**: R² >= 0.9 for linear fit; median update time scales linearly with k
- **Sample Size**: 2000 mutations total (1000 insertions + 1000 deletions)

#### SC-006: Subtree Stability
Unaffected community subtrees remain identical between streaming mutations (stability verification).

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3)
- **Procedure**: Run full community detection; record hierarchical tree; perform edge insertion; identify subtrees whose members are all outside the 2-hop neighborhood of mutated edge; verify these subtrees have identical member sets, structure, and community IDs
- **Pass Threshold**: 100% of unaffected subtrees remain identical across 200 mutations
- **Sample Size**: 100 edge insertions + 100 edge deletions (non-disconnecting) = 200 mutations

#### SC-007: Descriptive Typed Errors
All public APIs return descriptive typed errors for invalid parameters without panicking.

**Measurement Methodology:**
- **Test Input**: All public APIs with invalid inputs (malformed files, negative weights, invalid configs, empty inputs)
- **Procedure**: For each documented error condition, verify: (a) return type is a typed domain error, (b) error variant matches failure mode, (c) error message includes what went wrong, which parameter caused the issue, and expected format or valid range
- **Pass Threshold**: 100% of induced errors return typed variants with descriptive messages; zero panics across all test cases
- **Sample Size**: All public API functions × all documented error conditions (minimum 20 error scenarios)

#### SC-008: Graceful Edge Case Handling
The system handles edge cases gracefully without errors.

**Measurement Methodology:**
- **Test Input**: Enumerated edge cases: empty graph (0 nodes), single node, single edge, two disconnected nodes, all isolated nodes, complete graph, bipartite graph, self-loops, zero-weight edges, negative weights (with validation), non-contiguous node IDs
- **Procedure**: Run each algorithm on each edge case; verify: (a) no panics, (b) valid partition returned, (c) finite quality scores (no NaN/Inf), (d) typed error for invalid inputs
- **Pass Threshold**: All edge cases produce valid partitions or typed errors; zero panics across all algorithms
- **Sample Size**: 11 edge cases × 5 algorithms = 55 test cases

#### SC-009: Observable Events
Observable events capture 100% of phase transitions during algorithm execution.

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3)
- **Procedure**: Run algorithm with event capture; maintain event log; verify that for each phase transition (phase start, node relocation, refinement split, aggregation contraction, community merge, community split, iteration boundary, convergence plateau), at least one corresponding event is emitted
- **Pass Threshold**: 100% of phase transitions have corresponding events in the log
- **Sample Size**: 10 runs × 5 algorithms = 50 executions

#### SC-010: Consistent Algorithm Interface
Users can switch between algorithms (Leiden, Louvain, Infomap, LPA, Fluid) through a consistent interface with typed algorithm-specific configurations.

**Measurement Methodology:**
- **Test Input**: Single LFR benchmark graph (N=10k, μ=0.3)
- **Procedure**: (a) Verify all five algorithms implement a common `CommunityDetector` trait with `detect(&self, graph: &G) -> Result<Partition, Error>` method; (b) Verify all accept same graph input types; (c) Verify all return `Partition` type; (d) Run all five algorithms on same graph; verify all return valid partitions with same node count as input
- **Pass Threshold**: All algorithms produce valid partitions; trait signatures match; switchability test passes
- **Sample Size**: Single test graph × 5 algorithms = 5 executions

#### SC-011: Edge Deletion Split
Edge deletions that disconnect communities result in immediate community split, preserving the connectedness guarantee.

**Measurement Methodology:**
- **Test Input**: Synthetic graph with known community structure and bridge edges (LFR benchmark with manually inserted bridges)
- **Procedure**: Run full detection; identify bridge edges whose removal disconnects a community; delete bridge edge; verify resulting communities are internally connected via BFS
- **Pass Threshold**: 100% of community-splitting deletions result in internally connected sub-communities
- **Sample Size**: 50 bridge edge deletions across 5 graph instances

#### SC-012: Stepping Mode
Stepping mode works via both iterator and callback interfaces without degrading core algorithm performance when not in use.

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3)
- **Procedure**: (a) Run algorithm via iterator interface; verify each step yields until exhausted; verify final partition matches non-stepping execution; (b) Run algorithm via callback interface; verify callback invoked for each step; verify final partition matches; (c) Verify stepping overhead when enabled is within SC-013 budget
- **Pass Threshold**: Iterator and callback produce identical partitions to non-stepping execution; all steps captured; overhead within budget
- **Sample Size**: 10 runs per interface × 2 interfaces = 20 executions

#### SC-013: Observability Overhead
When full observability is enabled, algorithm execution overhead does not exceed 20% relative to execution with only base observability.

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3)
- **Procedure**: Measure median wall-clock time with base observability (t_base, without `full-observability` flag) and with full observability (t_obs, with `--features full-observability`); compute overhead as (t_obs - t_base) / t_base
- **Pass Threshold**: Overhead <= 0.20 (20%)
- **Sample Size**: 50 runs per configuration (base vs. full) = 100 executions total

#### SC-014: Asymmetric Weight Symmetry
Graphs with asymmetric edge weights produce identical modularity results when treated as undirected (after symmetrization) regardless of edge direction input order.

**Measurement Methodology:**
- **Test Input**: Random directed graph (N=1000) with asymmetric weights
- **Procedure**: Create graph with edge (i→j, w1) and (j→i, w2) where w1 ≠ w2; create reversed version with opposite order; run both as undirected; compare Modularity Q values
- **Pass Threshold**: Modularity Q values identical (within floating-point tolerance of 1e-10) regardless of input order
- **Sample Size**: 100 asymmetric graphs × 2 orderings = 200 executions

#### SC-015: Zero-Trust Logging
When opt-in logging is disabled (default), no graph data, node identifiers, or topology information appears in any log output.

**Measurement Methodology:**
- **Test Input**: LFR benchmark (N=10k, μ=0.3) with known node IDs and topology
- **Procedure**: Run algorithm with default logging (no opt-in); capture all log output; scan for patterns matching node IDs, edge lists, adjacency information, or community member lists
- **Pass Threshold**: Zero occurrences of graph data, node IDs, or topology in log output (automated pattern matching)
- **Sample Size**: 10 runs × 5 algorithms = 50 log scans

## Property-Based Testing Requirements

Per Constitution Principle VI, all algorithms MUST be verified through property-based testing (`proptest`). The following invariants are categorized as common (apply to all algorithms) or algorithm-specific.

### Common Invariants (All Algorithms)

1. **Valid Partition Structure**: For any valid graph input, the partition returned has exactly `n` membership entries (one per node), community IDs are contiguous starting from 0, and all nodes are assigned to exactly one community.
2. **No Panics**: No algorithm panics on any valid graph input (including edge cases: empty graphs, single nodes, disconnected graphs, graphs with self-loops).
3. **Deterministic with Seed**: Running the same algorithm twice with the same seed and same input produces bit-for-bit identical membership vectors.
4. **Finite Quality Scores**: Quality metrics returned are finite (no NaN, no Inf) for all valid inputs.
5. **Termination**: All algorithms terminate within the configured maximum iteration bound.

### Algorithm-Specific Invariants

- **Leiden**: Every detected community is internally connected (verifiable via BFS/DFS). Two disconnected graph components are never merged into the same community.
- **Louvain**: Quality monotonically increases or plateaus across aggregation levels. Two disconnected graph components are never merged into the same community.
- **Infomap**: Flow-based optimization converges to a local optimum of the Map Equation. Random walk probabilities sum to 1.
- **LPA**: Label propagation converges or reaches iteration bound. Tie-breaking is deterministic with fixed seed.
- **Fluid Communities**: Fluid density updates converge. k > n is handled (error or clamp).

## External Dependencies

| Dependency | Version/Reference | Purpose |
|------------|-------------------|---------|
| petgraph | 0.8.3 (https://docs.rs/petgraph/0.8) | Graph data structure interoperability (communal-petgraph crate) |
| rayon | 1.10 (https://docs.rs/rayon/1.10) | Data parallelism for algorithm phases |
| thiserror | 2.0 (https://docs.rs/thiserror/2.0) | Domain-rich error types |
| serde + serde_json | 1.0 (https://docs.rs/serde/1.0) | Serialization for I/O and WASM |
| ratatui | 0.30.2 (https://docs.rs/ratatui/0.30) | TUI rendering |
| crossterm | 0.29.0 (https://docs.rs/crossterm/0.29) | Terminal control for TUI |
| wasm-bindgen | 0.2 (https://docs.rs/wasm-bindgen/0.2) | WebAssembly bindings |
| proptest | 1.6 (https://docs.rs/proptest/1.6) | Property-based testing |
| criterion | 0.8.2 (https://docs.rs/criterion/0.8) | Benchmarking |
| num-traits | 0.2.19 (https://docs.rs/num-traits/0.2) | Numeric trait abstractions |
| tracing | 0.1.44 (https://docs.rs/tracing/0.1) | Structured observability |
| tracing-subscriber | 0.3.23 (https://docs.rs/tracing-subscriber/0.3) | Subscriber implementation for tracing (stdout/file output, EnvFilter, formatting) |
| rolling-file | 0.2.0 (https://docs.rs/rolling-file/0.2) | Size-based log file rotation for file logging destinations |

## License Compatibility

Per Constitution Principle VII (Permissive Dual-Licensing), all original framework code is licensed under MIT OR Apache-2.0. All external dependencies use MIT-compatible licenses (MIT, Apache-2.0, or dual MIT/Apache-2.0); no GPL or copyleft dependencies are used in the default feature set. The CI pipeline SHOULD include a license compatibility check (e.g., `cargo-deny` or `cargo-license`) to verify compliance at build time. The `THIRD_PARTY_LICENSES.md` file (required by T006) will contain upstream attribution for all dependencies.

| SNAP Datasets | Stanford SNAP (https://snap.stanford.edu/data/) | Benchmark graph datasets (Zachary Karate Club, Dolphins, Cora, Enron) |
| LFR Benchmark | Lancichinetti & Fortunato (2009) https://sites.google.com/site/santofortunato/inthepress2 | Synthetic benchmark generator with known ground truth |
| Traag et al. (2019) | "From Louvain to Leiden" https://doi.org/10.1038/s41598-019-41695-z | Leiden algorithm reference implementation |
| Rosvall & Bergstrom (2008) | "Maps of random walks on complex networks" https://doi.org/10.1073/pnas.0706851105 | Map Equation / Infomap reference |
| Raghavan et al. (2007) | "Near linear time to detect community structures in large-scale networks" https://doi.org/10.1103/PhysRevE.76.036106 | LPA algorithm reference |
| Cordasco & Gargano | "Community detection via semi-synchronous label propagation algorithms" https://doi.org/10.1109/SASON.2010.37 | Semi-synchronous LPA reference |
| Parșesan (2011) | Fluid Communities algorithm reference | Fluid density-based partitioning |

## Assumptions

**Tag Key:** [Testable] = verifiable via test/analysis | [Design Decision] = untestable scope/priority decision

- [Testable] Users provide valid graph structures with non-negative edge weights. Negative weights are rejected at input/construction time by default, with an option to defer validation to algorithm execution time for advanced use cases.
- [Testable] Node identifiers are representable as contiguous or mappable to contiguous indices internally.
- [Design Decision] The default resolution parameter (gamma = 1.0) produces community counts in the range k ∈ [2, V/2] for typical benchmark graphs (LFR N=10k, μ=0.1-0.3), validated by SC-003 and SC-004.
- [Testable] Convergence is determined by a configurable quality improvement threshold. The default is 1e-6 per canonical implementations (leidenalg, igraph), producing partitions with modularity within ε=0.01 of the best-known result for benchmark graphs (LFR N=10k). Users may set to 0 for strict convergence (theoretical guarantee: continue until no improvement) or configure function-specific thresholds for early termination.
- [Testable] Deterministic execution with a fixed seed may sacrifice some optimization quality compared to non-deterministic execution. The acceptable quality degradation is bounded: deterministic results MUST remain within 5% of the best-known non-deterministic result (measured by the target quality function, e.g., Modularity Q).
- [Testable] The framework operates across graph scales without configuration changes: small (1K nodes, ~10K edges), medium (100K nodes, ~1M edges), large (1M nodes, ~10M edges). Correctness guarantees (SC-001 through SC-015) hold across all three scales without scale-specific tuning. Performance regression tests MUST cover all three scales.
- [Design Decision] The framework targets graphs fitting in available system memory; out-of-core processing is out of scope for v1.
- [Design Decision] WASM deployment targets single-threaded execution with Leiden and Louvain algorithms only; multi-threading in WASM is deferred due to current WASM runtime performance limitations (not a security boundary). A future WASM build customization tool will allow users to select their desired algorithm subset. The rayon dependency automatically falls back to single-threaded execution in WASM builds (work runs on the calling thread when no worker threads are available); no conditional compilation or alternative implementations are required.
- [Design Decision] Full multilayer network implementation is deferred to v1.1; core traits are designed for forward compatibility only.
- [Testable] The CLI provides full pipeline capabilities: file I/O (EdgeList, JSON, GML), quality metrics computation, algorithm comparison (side-by-side metrics table + pairwise NMI/ARI between algorithm outputs), format conversion (JSON, CSV, GML), and batch processing runs (file batching, algorithm batching, and parameter sweeps via unified batch configuration).
- [Testable] The TUI provides event logging, real-time statistics, force-directed graph layout visualization, and pedagogical didactic explanations generated from static templates with interpolated algorithm state values (hybrid approach).
- [Design Decision] The CLI and TUI are secondary interfaces; the primary interface is the library API.
- [Testable] Graphs are undirected by default; users must explicitly mark graphs as directed for asymmetric analysis.
- [Testable] Algorithm-specific hyperparameters are provided through typed config structs, preventing invalid configurations at compile time.
- [Design Decision] Dynamic graph support in v1 covers edge insertions and deletions only; node mutations are deferred to a future release.

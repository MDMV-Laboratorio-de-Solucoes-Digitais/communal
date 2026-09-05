# Tasks: Community Detection Framework

**Input**: Design documents from `/specs/001-community-detection/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md, constitution.md

**Tests**: Test tasks are included for critical correctness paths (connected communities, determinism, edge cases) per constitution Principle VI.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions
- **Suffix IDs**: Alphabetic suffixes (e.g., T017a, T026a) denote tasks inserted after initial planning

## Path Conventions

- **Workspace root**: `Cargo.toml`, `crates/`
- **Subcrates**: `crates/communal-{core,algo,dynamic,petgraph,metrics,generators,wasm,cli,tui,benches}/`
- **Facade crate**: `communal/src/lib.rs`
- **Tests**: `tests/` within each crate

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace initialization, linting configuration, and basic project structure

- [ ] T001 Create workspace Cargo.toml with all 10 subcrates and facade crate in `Cargo.toml`
- [ ] T002 [P] Configure rustfmt with edition 2024 settings in `rustfmt.toml`
- [ ] T003 [P] Configure clippy lints matching constitution requirements in `.clippy.toml`
- [ ] T004 [P] Add deny attributes to all crate lib.rs files (#![deny(unsafe_code, missing_docs, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::allow_attributes_without_reason)])
- [ ] T004a [P] Add compile-time verification that algorithm hot loops use static dispatch only (Constitution Principle II MUST-4) in `crates/communal-algo/src/lib.rs`. Implement a `dispatch_compliance!()` macro or clippy lint config that fails if `dyn GraphView` appears in `leiden/`, `louvain/`, `infomap/`, `lpa/`, or `fluid/` modules. Document the verification approach in `crates/communal-algo/README.md`.
- [ ] T005 [P] Create LICENSE-MIT and LICENSE-APACHE files in repository root
- [ ] T006 [P] Create THIRD_PARTY_LICENSES.md with upstream attribution in repository root
- [ ] T007 [P] Create .github/workflows/ci.yml with build, test, lint, and doc checks
- [ ] T008 Create facade crate `communal/Cargo.toml` with feature flags (algo, petgraph, metrics, dynamic, wasm, cli, tui, generators, full-observability)
- [ ] T009 Create facade crate `communal/src/lib.rs` with re-exports

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### communal-core Crate

- [ ] T010 Create `crates/communal-core/Cargo.toml` with dependencies (num-traits, thiserror, serde)
- [ ] T011 [P] Implement `NodeId` and `CommunityId` newtypes in `crates/communal-core/src/id.rs`
- [ ] T012 [P] Implement `CsrGraph<N, E>` struct in `crates/communal-core/src/csr.rs`
- [ ] T013 [P] Implement `GraphView` trait with sealed `MultilayerView` marker supertrait for v1.1 forward compatibility in `crates/communal-core/src/graph_view.rs`. The `MultilayerView` trait MUST use the sealed trait pattern (private supertrait or `pub(crate)` method) to prevent downstream implementations while allowing future trait extensions.
- [ ] T014 [P] Implement `Partition` struct with `has_disconnected_communities()` method in `crates/communal-core/src/partition.rs` (per FR-006, type name is `Partition` not `PartitionResult`)
- [ ] T015 [P] Implement `AlgorithmConfig` trait with 5 required methods per spec Key Entities: `convergence_threshold(&self) -> f64` (default 1e-6), `convergence_mode(&self) -> ConvergenceMode`, `max_iterations(&self) -> usize`, `seed(&self) -> Option<u64>`, `validate(&self) -> Result<(), Error>` and `ConvergenceMode` enum (Absolute, Relative) in `crates/communal-core/src/config.rs`. NOTE: The resolution parameter (gamma, FR-008) is NOT part of the base `AlgorithmConfig` trait because it only applies to quality functions that support resolution (Modularity Q, CPM) and is irrelevant to Infomap, LPA, and Fluid Communities. Gamma is defined in algorithm-specific configs (LeidenConfig, LouvainConfig) that use resolution-based quality functions. This follows the separation pattern in python-leidenalg where `resolution_parameter` is a property of `Configuration` (algorithm-specific), not the base class.
- [ ] T016 [P] Implement `AlgorithmPhase` enum (variants: `LocalMoving`, `Refinement`, `Aggregation`, `Convergence`) and `StepEvent` enum with phase-encoded variant names per spec: `LocalMovingStart { iteration: usize }`, `LocalMovingEnd { iteration: usize }`, `NodeRelocation { node: NodeId, from: CommunityId, to: CommunityId }`, `RefinementSplit { community: CommunityId, into: usize }`, `AggregationContraction { from_communities: usize, to_communities: usize }`, `CommunityMerge { into: CommunityId, merged: CommunityId }`, `CommunitySplit { from: CommunityId, into: Vec<CommunityId> }`, `IterationBoundary { phase: AlgorithmPhase, iteration: usize }`, `ConvergencePlateau { iterations_below_threshold: usize }`, `ConvergenceDetected { total_iterations: usize, final_quality: f64 }` in `crates/communal-core/src/step.rs`. NOTE: `IterationBoundary.phase` uses `AlgorithmPhase` enum (type-safe), not free-form String.
- [ ] T017 [P] Implement domain error types in their respective crates: `GraphError` and `PartitionError` in `crates/communal-core/src/error.rs`; `AlgorithmError` in `crates/communal-algo/src/error.rs` (per FR-044 per-module error types)
- [ ] T017a [P] Write comprehensive error scenario tests covering all public APIs × error conditions. Verify 100% of induced errors return typed variants (zero panics). Minimum 28 error scenarios across 4 error enums: GraphError (10 scenarios: negative weights, malformed EdgeList/JSON/GML, unsupported format, empty input, invalid node reference), AlgorithmError (7: invalid threshold, resolution, teleportation rate, target communities, non-convergence, invalid update mode), PartitionError (7: invalid node ID, community ID, hierarchy level, invalid level, split failure, complexity exceeded, mutation error), MetricsError (4: empty partition, size mismatch, invalid ground truth, numerical overflow). Uses assert_matches!, catch_unwind, proptest. Files: crates/communal-core/tests/error_scenarios.rs, crates/communal-algo/tests/error_scenarios.rs, crates/communal-metrics/tests/error_scenarios.rs
- [ ] T018 Implement `Partition` struct query methods in `crates/communal-core/src/partition.rs`: `community_of(node_id) -> Option<&CommunityId>`, `quality_score() -> f64`, `membership_vec() -> Vec<CommunityId>`, `communities() -> Vec<&[NodeId]>`, `community_sizes() -> &[usize]` (per Key Entities)
- [ ] T019 Implement graph validation (non-negative weights, CSR structure) in `crates/communal-core/src/validation.rs`
- [ ] T020 [P] Implement graph builders (from edge list, adjacency list) with `validate_on_construction: bool` flag in `crates/communal-core/src/builder.rs`
- [ ] T021 [P] Implement BFS/DFS traversal utilities in `crates/communal-core/src/traversal.rs`
- [ ] T022 [P] Write unit tests for CsrGraph construction and validation in `crates/communal-core/tests/csr_tests.rs`
- [ ] T023 [P] Write unit tests for graph builders in `crates/communal-core/tests/builder_tests.rs`
- [ ] T024 Write doctests for all public APIs in `crates/communal-core/src/lib.rs`. Cover: NodeId, CommunityId newtypes; CsrGraph construction and query methods; GraphView trait; Partition query API; AlgorithmConfig trait; AlgorithmPhase and StepEvent enums; GraphError, PartitionError error types. Include complexity annotations for all graph operations.
- [ ] T025 [P] Implement edge weight symmetrization (arithmetic mean for undirected graphs) integrated with graph builders for eager application at construction time per FR-031 in `crates/communal-core/src/symmetrize.rs`
- [ ] T026 [P] Implement tiered logging configuration (error, warn, info, debug, trace) in `crates/communal-core/src/logging.rs`
- [ ] T026a Implement stdout and file log output destinations with `tracing-subscriber` integration in `crates/communal-core/src/logging.rs`. Support stdout layer (suitable for piping/redirection without decorative formatting — no ANSI escape sequences, no progress bars, no Unicode drawing characters per FR-037), file layer (configurable path and rotation policies), and sanitization preventing graph data/node identifiers/topology from appearing in logs unless explicitly enabled per FR-032. Implement `RotationPolicy` enum: `Never`, `Size(max_bytes)`, `Daily`, `Hourly`. Aligned with tracing-subender's `RollingFileAppender` for time-based rotation; use `rolling-file` crate for size-based rotation (per research/log-rotation-practices.md).
- [ ] T026b Implement granular tiered logging (error, warn, info, debug, trace) and `max_files: usize` retention policy in `crates/communal-core/src/logging.rs`. When `max_files` is exceeded, delete oldest archived files. Archived files MUST use timestamp suffix naming (e.g., `app.log.2026-09-05-14-30`).
- [ ] T026c [P] Write log rotation behavior tests in `crates/communal-core/tests/log_rotation_tests.rs`. Create a temporary directory. Configure file logging with `RotationPolicy::Size(1KB)` and emit log output exceeding the threshold. Assert multiple rotated files exist with timestamp suffixes. Configure with `RotationPolicy::Never` and assert single unbounded file. Verify all rotated files contain valid log content (no partial lines, no dropped entries per write). Validate sanitization: no node IDs, edge weights, or topology appear in rotated files when `sanitize=true`.
- [ ] T027 [P] Write symmetrization correctness tests verifying (w_ij + w_ji)/2 produces identical modularity regardless of edge input order in `crates/communal-core/tests/symmetrization_tests.rs`
- [ ] T027a [P] Write order-independence tests verifying asymmetric edge weights produce identical modularity results when edge direction input order is reversed in `crates/communal-core/tests/symmetrization_tests.rs` (validates SC-014). NOTE: This is a foundational correctness test for graph construction symmetrization (FR-031), required by ALL algorithms. It is intentionally NOT story-tagged because SC-014 is a framework-wide invariant, not specific to any single user story.
- [ ] T028 [P] Write zero-trust logging verification tests asserting no node IDs, edge weights, or topology data appear in log output when opt-in logging disabled in `crates/communal-core/tests/logging_zero_trust_tests.rs` [DEPENDS ON: T026a, T026b, T026c]
- [ ] T028d [P] Create format schema documentation files per FR-009: `contracts/formats/edgelist.schema.md`, `contracts/formats/json-graph.schema.md`, `contracts/formats/gml.schema.md`, `contracts/formats/csv-graph.schema.md`. Each MUST document: field definitions, required/optional markers, valid value ranges, and example valid/invalid inputs. This task creates the schema files referenced by FR-009; parsers and serializers implement against these contracts. These schemas define the contract that parsers (T049-T052) and serializers (T125a) implement against.
- [ ] T028a [P] Implement `QualityMetric` trait in `crates/communal-core/src/quality.rs` per FR-021: `evaluate<G: GraphView>(&self, graph: &G, partition: &Partition) -> Result<f64, MetricsError>`, `name(&self) -> &'static str`, `range(&self) -> (f64, f64)`. Trait MUST be in communal-core to avoid circular dependencies; communal-metrics and communal-algo both implement it. This is a foundational trait required by T034 (QualityFunction enum) in Phase 3 and T074-T076 (metrics implementations) in Phase 5.
- [ ] T028b Implement `ComparativeMetric` trait in `crates/communal-core/src/quality.rs` per FR-019: `evaluate<G: GraphView>(&self, graph: &G, partition1: &Partition, partition2: &Partition) -> Result<f64, MetricsError>`. This trait is distinct from `QualityMetric` (FR-021) because it requires two partition inputs. Trait MUST be in communal-core to avoid circular dependencies; communal-metrics implements it for NMI and ARI. [DEPENDS ON: T028a]
- [ ] T028c Implement `CommunityDetector` trait with `detect(&self, graph: &G) -> Result<Partition, Error>` required method and `detect_into` provided method in `crates/communal-core/src/detector.rs` per FR-043 and Key Entities section. This is the foundational trait that all algorithms implement.

### communal-generators Crate (Test Infrastructure)

**Purpose**: Synthetic benchmark generators producing graphs with known ground truth for validation tests. These MUST complete before any user story tests that validate algorithm correctness against synthetic benchmarks.

- [ ] T131 Create `crates/communal-generators/Cargo.toml` with dependencies (communal-core, rand)
- [ ] T132 [P] Implement LFR benchmark generator in `crates/communal-generators/src/lfr.rs`
- [ ] T133 [P] Implement SBM generator in `crates/communal-generators/src/sbm.rs`
- [ ] T134 [P] Implement Barabasi-Albert generator in `crates/communal-generators/src/barabasi_albert.rs`
- [ ] T135 [P] Implement Erdos-Renyi generator in `crates/communal-generators/src/erdos_renyi.rs`

---

## Phase 3: User Story 1 - Guaranteed Connected Partitioning via Leiden (Priority: P1) MVP

**Goal**: Implement Leiden algorithm producing internally connected communities with deterministic execution

**Independent Test**: Run Leiden on benchmark graphs and verify via BFS/DFS that every detected community is internally connected. Same seed produces identical partitions.

### Tests for User Story 1

- [ ] T032 [US1] Create `crates/communal-algo/Cargo.toml` with dependencies (communal-core, rayon, rand, tracing)
- [ ] T029 [P] [US1] Write property test for connected communities invariant in `crates/communal-algo/tests/leiden_connected.rs` [DEPENDS ON: T032]
- [ ] T030 [P] [US1] Write determinism test (same seed = same result) in `crates/communal-algo/tests/leiden_determinism.rs` [DEPENDS ON: T032]
- [ ] T031 [P] [US1] Write edge case tests (empty graph, isolated nodes, disconnected components) in `crates/communal-algo/tests/leiden_edge_cases.rs` [DEPENDS ON: T032]

### Implementation for User Story 1

- [ ] T033 [P] [US1] Implement `LeidenConfig` struct in `crates/communal-algo/src/leiden/config.rs`
- [ ] T034 [P] [US1] Implement `QualityFunction` enum (Modularity, CPM, MapEquation) in `crates/communal-algo/src/quality.rs` [DEPENDS ON: T028a — QualityMetric trait in communal-core]
- [ ] T035 [P] [US1] Implement Modularity Q computation in `crates/communal-algo/src/quality/modularity.rs`
- [ ] T036 [P] [US1] Implement Constant Potts Model computation in `crates/communal-algo/src/quality/cpm.rs`
- [ ] T036a [P] [US1] Implement Map Equation computation for Infomap optimization in `crates/communal-algo/src/quality/map_equation.rs`
- [ ] T036b [US1] Thread resolution parameter (gamma) from the algorithm-specific config (LeidenConfig) through to Modularity Q and CPM computations in `crates/communal-algo/src/quality/modularity.rs` and `crates/communal-algo/src/quality/cpm.rs`. Gamma is NOT part of the base `AlgorithmConfig` trait (see T015 note) — it is defined in algorithm-specific configs that use resolution-based quality functions. When gamma is not explicitly set in config, default to 1.0 for standard modularity per the contracts in `contracts/metrics.md`.
- [ ] T037 [US1] Implement smart local move phase in `crates/communal-algo/src/leiden/local_moving.rs`
- [ ] T038 [US1] Implement randomized refinement phase in `crates/communal-algo/src/leiden/refinement.rs`
- [ ] T039 [US1] Implement aggregation phase in `crates/communal-algo/src/leiden/aggregation.rs`
- [ ] T040 [US1] Implement main `Leiden` struct and `CommunityDetector` impl in `crates/communal-algo/src/leiden/mod.rs`
- [ ] T041 [US1] Implement convergence detection (absolute/relative modes) and plateau threshold derivation per FR-033 in `crates/communal-algo/src/leiden/convergence.rs`. The plateau threshold MUST be derived as `max(convergence_threshold / 10, 1e-8)` per FR-033's formula — NOT hardcoded. This ensures the plateau threshold scales with user-configured convergence thresholds.
- [ ] T042 [US1] Implement `detect_with_steps` for observability in `crates/communal-algo/src/leiden/stepping.rs`
- [ ] T043 [US1] Write SNAP benchmark validation tests in `crates/communal-algo/tests/snap_benchmarks.rs`
- [ ] T043b [P] [US1] Create reference partition data files in `contracts/reference-partitions/` per SC-003: `karate-club.membership`, `dolphins.membership`, `cora.membership`, `enron.membership`. Each file contains one community ID per line corresponding to node IDs in the benchmark graph. Source provenance documented in `contracts/reference-partitions/README.md`.
- [ ] T043a [P] [US1] Write synthetic benchmark validation tests verifying NMI >= 0.95 against known ground truth on LFR and SBM benchmarks in `crates/communal-algo/tests/synthetic_ground_truth.rs` (validates SC-004) [DEPENDS ON: T132-T135]

**Checkpoint**: At this point, User Story 1 should be fully functional - Leiden produces connected, deterministic partitions

---

## Phase 4: User Story 2 - Universal Graph Input & Partition Extraction (Priority: P1)

**Goal**: Accept various graph input formats and provide queryable partition results by original node identifiers

**Independent Test**: Accept adjacency lists, edge lists with weights; verify partition results queryable by original node IDs with correct summary statistics.

### Tests for User Story 2

- [ ] T044 [P] [US2] Write tests for EdgeList parsing in `crates/communal-core/tests/edgelist_parser_tests.rs`
- [ ] T045 [P] [US2] Write tests for JSON graph format parsing in `crates/communal-core/tests/json_parser_tests.rs`
- [ ] T046 [P] [US2] Write tests for GML format parsing in `crates/communal-core/tests/gml_parser_tests.rs`
- [ ] T047 [P] [US2] Write tests for non-contiguous node ID mapping in `crates/communal-core/tests/node_mapping_tests.rs`
- [ ] T048 [P] [US2] Write tests for partition summary statistics in `crates/communal-core/tests/partition_stats_tests.rs`

### Implementation for User Story 2

- [ ] T049 [P] [US2] Implement EdgeList parser in `crates/communal-core/src/io/edgelist.rs`
- [ ] T050 [P] [US2] Implement JSON graph parser in `crates/communal-core/src/io/json.rs`
- [ ] T051 [P] [US2] Implement GML parser in `crates/communal-core/src/io/gml.rs`
- [ ] T051a [P] [US2] Implement CSV edge-list parser in `crates/communal-core/src/io/csv.rs`. Format: `source,target,weight` per line (header optional). Validates against `contracts/formats/csv-graph.schema.md`. Fail-fast on malformed input per FR-040.
- [ ] T052 [P] [US2] Implement node ID mapping (non-contiguous to contiguous) in `crates/communal-core/src/mapping.rs`
- [ ] T053 [US2] Implement partition summary statistics in `crates/communal-core/src/partition/stats.rs`
- [ ] T054 [P] Implement edge case handling per FR-026 in `crates/communal-core/src/edge_cases.rs`: empty graph (0 nodes → empty partition, quality 0.0), single node (1 community [0]), single edge (1 community [0,0]), two disconnected nodes (2 communities [0,1]), all isolated nodes (n communities [0,1,...,n-1] per FR-023). NOTE: This implements FR-026 which applies to ALL algorithms. It is intentionally NOT story-tagged because edge case handling is a framework-wide invariant (same pattern as T031a and T027a).
- [ ] T055 [US2] Implement zero-weight edge handling (no division-by-zero in quality metric computation, including zero-weight edge cases per FR-025) in `crates/communal-algo/src/quality/safe_division.rs` — provide safe division utility used by Modularity Q and CPM computations
- [ ] T056 [US2] Implement self-loop handling in degree calculations in `crates/communal-core/src/csr.rs`
- [ ] T056a [US2] Write self-loop quality term tests verifying self-loops are correctly included in quality metric computations per Traag et al. (2019) in `crates/communal-algo/tests/self_loop_quality_tests.rs` (validates FR-027 quality metric coverage)
- [ ] T145a [P] Add property-based test for negative weight rejection (FR-034) in `crates/communal-core/tests/negative_weight_tests.rs`
- [ ] T125a [P] Implement graph and partition serialization in `crates/communal-core/src/io/serialize.rs` per FR-041: `GraphSerializer` trait with `serialize_graph(&self, graph: &G, writer: &mut impl Write) -> Result<(), IoError>` and `PartitionSerializer` trait with `serialize_partition(&self, partition: &Partition, writer: &mut impl Write) -> Result<(), IoError>`. Support JSON, CSV, GML formats. Guarantee round-trip fidelity: serialized output MUST be re-parseable by corresponding parser (FR-040).
- [ ] T125b [P] Write round-trip serialization tests in `crates/communal-core/tests/serialization_roundtrip.rs` verifying FR-041 fidelity guarantee: serialize → deserialize → compare for all formats (JSON, CSV, GML) across multiple graph types.

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Multi-Algorithm Clustering Spectrum (Priority: P2)

**Goal**: Implement Louvain, Infomap, LPA, and Fluid Communities algorithms under unified interface with typed configs

**Independent Test**: Run each algorithm on same graph; verify results match theoretical properties; compute NMI/ARI between partitions.

### Tests for User Story 3

- [ ] T072 [US3] Create `crates/communal-metrics/Cargo.toml` with dependencies (communal-core)
- [ ] T057 [P] [US3] Write Louvain correctness tests in `crates/communal-algo/tests/louvain_tests.rs`
- [ ] T058 [P] [US3] Write Infomap correctness tests in `crates/communal-algo/tests/infomap_tests.rs`
- [ ] T059 [P] [US3] Write LPA correctness tests in `crates/communal-algo/tests/lpa_tests.rs`
- [ ] T060 [P] [US3] Write Fluid Communities correctness tests in `crates/communal-algo/tests/fluid_tests.rs`
- [ ] T060a [P] [US3] Write Louvain property tests verifying quality monotonically increases or plateaus across aggregation levels in `crates/communal-algo/tests/louvain_properties.rs` (Constitution Principle VI)
- [ ] T060b [P] [US3] Write Infomap property tests verifying flow-based optimization converges to local optimum and random walk probabilities sum to 1 in `crates/communal-algo/tests/infomap_properties.rs` (Constitution Principle VI)
- [ ] T060c [P] [US3] Write LPA property tests verifying convergence or iteration bound reached, deterministic tie-breaking with fixed seed in `crates/communal-algo/tests/lpa_properties.rs` (Constitution Principle VI)
- [ ] T060d [P] [US3] Write Fluid Communities property tests verifying density updates converge and k > n handled in `crates/communal-algo/tests/fluid_properties.rs` (Constitution Principle VI)
- [ ] T061 [P] [US3] Write algorithm comparison tests (NMI/ARI) in `crates/communal-metrics/tests/comparison_tests.rs`

### Implementation for User Story 3

- [ ] T062 [P] [US3] Implement `LouvainConfig` struct in `crates/communal-algo/src/louvain/config.rs`
- [ ] T063 [P] [US3] Implement Louvain algorithm (local moving + aggregation) in `crates/communal-algo/src/louvain/mod.rs`
- [ ] T064 [P] [US3] Implement `InfomapConfig` struct in `crates/communal-algo/src/infomap/config.rs`
- [ ] T065 [P] [US3] Implement Infomap algorithm (Map Equation, random walks) in `crates/communal-algo/src/infomap/mod.rs`
- [ ] T066 [P] [US3] Implement `LpaConfig` struct in `crates/communal-algo/src/lpa/config.rs`
- [ ] T067 [P] [US3] Implement LPA algorithm (sync/async modes) in `crates/communal-algo/src/lpa/mod.rs`
- [ ] T068 [P] [US3] Implement `FluidConfig` struct in `crates/communal-algo/src/fluid/config.rs`
- [ ] T069 [P] [US3] Implement Fluid Communities algorithm in `crates/communal-algo/src/fluid/mod.rs`
- [ ] T070 [US3] Verify all algorithms implement `CommunityDetector` trait; add trait implementations if missing in `crates/communal-algo/src/lib.rs`
- [ ] T070a [US3] Write algorithm switchability tests verifying all 5 algorithms can be used through the consistent `CommunityDetector` trait interface on identical input in `crates/communal-algo/tests/switchability_tests.rs` (validates SC-010)
- [ ] T071 [P] [US3] Implement deterministic tie-breaking logic for bipartite/cyclic graphs in `crates/communal-algo/src/tie_breaking.rs`. Default tie-breaking MUST use lexicographic node ordering by NodeId; optional seed-based randomization MUST be available when configured. Each algorithm iteration (local moving pass, diffusion pass, or flow optimization pass) MUST run in O(V + E) time as specified in FR-028. NOTE: Infomap's Louvain-style node moves run in O(E) per sweep [Rosvall 2009; Blondel 2008] — the O(V + E · log V) complexity of the original 2008 greedy variant (agglomerative pair-merging with priority queues) does NOT apply to the modern sweep-based implementation.

### communal-metrics Crate

- [ ] T074 [P] [US3] Implement `Modularity` metric in `crates/communal-metrics/src/modularity.rs`
- [ ] T075 [P] [US3] Implement `ConstantPottsModel` metric in `crates/communal-metrics/src/cpm.rs`
- [ ] T076 [P] [US3] Implement `MapEquation` metric in `crates/communal-metrics/src/map_equation.rs`
- [ ] T077 [P] [US3] Implement `NormalizedMutualInformation` in `crates/communal-metrics/src/nmi.rs`
- [ ] T078 [P] [US3] Implement `AdjustedRandIndex` in `crates/communal-metrics/src/ari.rs`
- [ ] T079 [US3] Implement metric report types (`MetricsReport`, `ComparisonReport`) as free functions following the `QualityMetric` trait pattern in `crates/communal-metrics/src/report.rs`. Per FR-019, no separate `MetricsCalculator` type is introduced; metrics are exposed as structs implementing `QualityMetric` (FR-021). Report types are plain data containers, not a calculator service.
- [ ] T080 [US3] Implement `MetricsError` type in `crates/communal-metrics/src/error.rs`

**Checkpoint**: All five algorithms should produce valid partitions; NMI/ARI comparison works

---

## Phase 6: User Story 4 - Incremental Dynamic Graph Updates for GraphRAG (Priority: P2)

**Goal**: Support edge insertions/deletions with O(k) local updates, immediate community split on disconnection, and hierarchical tree slices

**Independent Test**: Mutate graph (add/remove edges); verify only local boundaries recalculated; unaffected subtrees remain stable; disconnected communities split immediately.

### Tests for User Story 4

- [ ] T086 [US4] Create `crates/communal-dynamic/Cargo.toml` with dependencies (communal-core, communal-algo)
- [ ] T081 [P] [US4] Write edge insertion tests in `crates/communal-dynamic/tests/insertion_tests.rs` [DEPENDS ON: T086]
- [ ] T082 [P] [US4] Write edge deletion tests in `crates/communal-dynamic/tests/deletion_tests.rs` [DEPENDS ON: T086]
- [ ] T083 [P] [US4] Write community split tests (disconnection) in `crates/communal-dynamic/tests/split_tests.rs` [DEPENDS ON: T086]
- [ ] T084 [P] [US4] Write subtree stability tests in `crates/communal-dynamic/tests/stability_tests.rs` [DEPENDS ON: T086]
- [ ] T085 [P] [US4] Write hierarchical tree slice tests in `crates/communal-dynamic/tests/hierarchy_tests.rs` [DEPENDS ON: T086]

### Implementation for User Story 4

- [ ] T087 [P] [US4] Implement `DynamicGraph` trait in `crates/communal-dynamic/src/lib.rs`
- [ ] T088 [P] [US4] Implement dynamic detection wrapper in `crates/communal-dynamic/src/detector.rs` (wraps StreamingDetector per FR-045; NOT a separate trait)
- [ ] T089 [P] [US4] Implement `HierarchicalTree` struct in `crates/communal-dynamic/src/hierarchy.rs`
- [ ] T090 [P] [US4] Implement `IncrementalUpdate` struct in `crates/communal-dynamic/src/update.rs`
- [ ] T091 [P] [US4] Implement `EdgeMutation` enum in `crates/communal-dynamic/src/mutation.rs`
- [ ] T092 [US4] Implement edge insertion with O(k) local update in `crates/communal-dynamic/src/insertion.rs`
- [ ] T093 [US4] Implement edge deletion with immediate split in `crates/communal-dynamic/src/deletion.rs`
- [ ] T094 [US4] Implement community split into connected components in `crates/communal-dynamic/src/split.rs`
- [ ] T095 [US4] Implement subtree stability tracking in `crates/communal-dynamic/src/stability.rs`
- [ ] T096 [US4] Implement `StreamingDetector` in `crates/communal-dynamic/src/streaming.rs`
- [ ] T097 [US4] Implement `DynamicOperationError` enum in `crates/communal-dynamic/src/error.rs` with variants: InvalidLevel, SplitFailure, ComplexityExceeded, MutationError. Add `From<DynamicOperationError>` for `PartitionError` to enable `?` operator in dynamic operations. Per FR-044, this is a sub-error type that converts to the unified model.
- [ ] T098 [P] [US4] Implement hierarchical tree slice API per FR-016: `at_level(level: usize) -> Partition`, `at_resolution(gamma: f64) -> Partition`, and `levels() -> Vec<Partition>` in `crates/communal-dynamic/src/hierarchy.rs`

**Checkpoint**: Dynamic updates work with O(k) complexity; communities split on disconnection; subtrees stable

---

## Phase 7: User Story 5 - Observability & Step-by-Step Algorithmic Stepping (Priority: P3)

**Goal**: Provide iterator and callback interfaces for algorithmic stepping; TUI with force-directed graph and pedagogical explanations

**Independent Test**: Run algorithm in stepping mode; verify discrete events emitted for each phase transition; TUI displays graph with didactic descriptions.

### Tests for User Story 5

- [ ] T099 [P] [US5] Write stepping iterator tests in `crates/communal-algo/tests/stepping_tests.rs`
- [ ] T100 [P] [US5] Write callback observer tests in `crates/communal-algo/tests/callback_tests.rs`. MUST include: (1) single observer receives all emitted events, (2) deregistration via Subscription Drop stops further callbacks, (3) FIFO ordering — when N observers are registered, assert each event dispatches in strict registration order (observer_0 before observer_1 before ... before observer_N-1), (4) dropping a middle observer preserves FIFO order for remaining observers. These tests validate FR-046's FIFO guarantee explicitly.
- [ ] T101 [P] [US5] Write event coverage tests (100% phase transitions) in `crates/communal-algo/tests/event_coverage_tests.rs`
- [ ] T102 [P] [US5] Write zero-cost overhead tests in `crates/communal-algo/tests/overhead_tests.rs`

### Implementation for User Story 5 - Core Stepping

- [ ] T103 [P] [US5] Implement `StepIterator` in `crates/communal-algo/src/stepping/iterator.rs`
- [ ] T104 [P] [US5] Implement `StepCallback` trait in `crates/communal-algo/src/stepping/callback.rs`
- [ ] T104a [US5] Implement `subscribe()` method on `CommunityDetector` returning `Subscription` RAII handle in `crates/communal-algo/src/subscribe.rs`. The `Subscription` MUST deregister the observer on Drop. Support multiple observers with FIFO dispatch per FR-046. The method signature MUST be `fn subscribe(&mut self, observer: impl StepCallback + Send + 'static) -> Subscription` per the spec. Observer bound is `Send` only (not `Send + Sync`) to allow mutable local state in callbacks.
- [ ] T105 [US5] Implement zero-cost event emission (const generics or enum dispatch) in `crates/communal-algo/src/stepping/emission.rs`
- [ ] T106 [US5] Integrate stepping into all algorithm implementations in `crates/communal-algo/src/*/mod.rs`

### communal-tui Crate

- [ ] T107 [US5] Create `crates/communal-tui/Cargo.toml` with dependencies (communal-core, communal-algo, communal-metrics, ratatui, crossterm)
- [ ] T108 [P] [US5] Implement TUI app structure in `crates/communal-tui/src/app.rs`
- [ ] T109 [P] [US5] Implement Fruchterman-Reingold force-directed graph layout in `crates/communal-tui/src/graph_layout.rs`
- [ ] T110 [P] [US5] Implement event log panel in `crates/communal-tui/src/panels/event_log.rs`
- [ ] T111 [P] [US5] Implement statistics panel in `crates/communal-tui/src/panels/stats.rs`
- [ ] T112 [P] [US5] Implement pedagogical explanation templates in `crates/communal-tui/src/pedagogy/templates.rs`
- [ ] T113 [US5] Implement pedagogical explanation renderer (hybrid static + interpolated) in `crates/communal-tui/src/pedagogy/renderer.rs`
- [ ] T114 [US5] Implement TUI stepping controller in `crates/communal-tui/src/stepping.rs`
- [ ] T115 [US5] Implement TUI main loop with crossterm in `crates/communal-tui/src/main.rs`

**Checkpoint**: Stepping mode works via iterator and callback; TUI displays algorithm execution with pedagogical explanations

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: CLI, WASM, generators, petgraph adapters, benchmarks, documentation

### communal-cli Crate

- [ ] T116 Create `crates/communal-cli/Cargo.toml` with dependencies (communal-core, communal-algo, communal-metrics, clap, serde_json)
- [ ] T117 [P] Implement CLI argument parsing in `crates/communal-cli/src/cli.rs`
- [ ] T118 [P] Implement `run` command in `crates/communal-cli/src/commands/run.rs`
- [ ] T119 [P] Implement `compare` command in `crates/communal-cli/src/commands/compare.rs`
- [ ] T120 [P] Implement `batch` command in `crates/communal-cli/src/commands/batch.rs`
- [ ] T121 [P] Implement `convert` command in `crates/communal-cli/src/commands/convert.rs`
- [ ] T122 [P] Implement `metrics` command in `crates/communal-cli/src/commands/metrics.rs`
- [ ] T123 [P] Implement `generate` command in `crates/communal-cli/src/commands/generate.rs`
- [ ] T124 [P] Implement `validate` command in `crates/communal-cli/src/commands/validate.rs`
- [ ] T125 Implement output formatters (JSON, CSV, GML) in `crates/communal-cli/src/output.rs`
- [ ] T126 Implement batch config (TOML) parser in `crates/communal-cli/src/batch_config.rs`

### communal-wasm Crate

- [ ] T127 Create `crates/communal-wasm/Cargo.toml` with dependencies (communal-core, communal-algo, wasm-bindgen, serde-wasm-bindgen)
- [ ] T128 [P] Implement WASM bindings for Leiden in `crates/communal-wasm/src/leiden.rs`
- [ ] T129 [P] Implement WASM bindings for Louvain in `crates/communal-wasm/src/louvain.rs`
- [ ] T130 Implement WASM graph input/output (JSON) in `crates/communal-wasm/src/lib.rs`
- [ ] T130a Implement WASM explicit memory cleanup function in `crates/communal-wasm/src/lib.rs`

### communal-petgraph Crate

- [ ] T136 Create `crates/communal-petgraph/Cargo.toml` with dependencies (communal-core, petgraph)
- [ ] T137 [P] Implement petgraph to CSR conversion in `crates/communal-petgraph/src/from_petgraph.rs`
- [ ] T138 [P] Implement CSR to petgraph conversion in `crates/communal-petgraph/src/to_petgraph.rs`
- [ ] T139 Implement zero-copy petgraph view adapters in `crates/communal-petgraph/src/view.rs`

### communal-benches Crate

- [ ] T140 Create `crates/communal-benches/Cargo.toml` with dependencies (all library crates, criterion)
- [ ] T141 [P] Implement SNAP dataset benchmark harness in `crates/communal-benches/benches/snap_benchmarks.rs`
- [ ] T142 [P] Implement algorithm comparison benchmarks in `crates/communal-benches/benches/algo_comparison.rs`
- [ ] T143 [P] Implement stepping overhead benchmarks verifying observability overhead <= 20% per SC-013 formula: (t_obs - t_base) / t_base <= 0.20 in `crates/communal-benches/benches/stepping_overhead.rs`
- [ ] T144 [P] Implement O(k) incremental update complexity benchmark in `crates/communal-benches/benches/incremental_complexity.rs` — measures update time vs. 2-hop neighborhood size k on LFR (N=10k, μ=0.3); verifies linear scaling via least-squares fit with R² >= 0.9 (validates SC-005). Failure criterion: R² < 0.9 triggers performance investigation.
- [ ] T145 [P] Write README.md with project overview and quickstart
- [ ] T031a [P] Write property test verifying completely disconnected components are never merged into the same community in `crates/communal-algo/tests/disconnected_components.rs`. NOTE: This test validates FR-024 which applies to ALL algorithms (not just Leiden). It is intentionally NOT story-tagged because disconnected component merging is a framework-wide invariant.
- [ ] T146 [P] Write CONTRIBUTING.md with development guidelines
- [ ] T147 [P] Generate rust docs for all crates with `cargo doc --no-deps --all-features`
- [ ] T148 Run quickstart.md validation scenarios
- [ ] T149 Profile performance on LFR benchmark (N=10k, μ=0.3) and optimize hot paths if >5% regression vs baseline
- [ ] T150 Security audit (zero unsafe, zero panics verification)
- [ ] T150a Create `docs/unsafe-exceptions.md` template for tracking approved unsafe code exceptions per FR-039 RFC process

### Doctest Coverage (Constitution Principle IV Compliance)

**Purpose**: Ensure all public APIs across all crates have executable doctests per Constitution Principle IV (Exhaustive Documentation). These tasks complete the documentation requirements for the workspace.

- [ ] T151 [P] Write doctests for communal-algo public APIs in `crates/communal-algo/src/lib.rs`. Cover: Leiden, Louvain, Infomap, LPA, Fluid detectors; QualityFunction enum; StepIterator; StepCallback trait; AlgorithmError variants. Include mathematical formulas for quality functions and algorithmic complexity annotations.
- [ ] T152 [P] Write doctests for communal-metrics public APIs in `crates/communal-metrics/src/lib.rs`. Cover: Modularity, ConstantPottsModel, MapEquation, Nmi, Ari metric structs; QualityMetric trait impls; report types; MetricsError variants. Include mathematical formulas (with brief summaries and external references per FR-020) for each metric.
- [ ] T153 [P] Write doctests for communal-dynamic public APIs in `crates/communal-dynamic/src/lib.rs`. Cover: StreamingDetector trait, HierarchicalTree (at_level, at_resolution, levels), EdgeMutation enum, IncrementalUpdate struct, DynamicOperationError variants. Include O(k) complexity annotations per FR-014.
- [ ] T154 [P] Write doctests for communal-generators public APIs in `crates/communal-generators/src/lib.rs`. Cover: LfrGenerator, SbmGenerator, BarabasiAlbertGenerator, ErdosRenyiGenerator. Include parameter constraints and generation complexity.
- [ ] T155 [P] Write doctests for communal-cli public APIs in `crates/communal-cli/src/lib.rs`. Cover: CLI argument parsing, command implementations (run, compare, batch, convert, metrics, generate, validate), output formatters (JSON, CSV, GML), batch config parser.
- [ ] T156 [P] Write doctests for communal-tui public APIs in `crates/communal-tui/src/lib.rs`. Cover: TUI app structure, Fruchterman-Reingold graph layout, panel rendering (event log, statistics), pedagogy templates, stepping controller.
- [ ] T157 [P] Write doctests for communal-wasm public APIs in `crates/communal-wasm/src/lib.rs`. Cover: WASM bindings for Leiden/Louvain, graph I/O (JSON), explicit memory cleanup function per FR-038.
- [ ] T158 [P] Write doctests for communal-petgraph public APIs in `crates/communal-petgraph/src/lib.rs`. Cover: petgraph to CSR conversion, CSR to petgraph conversion, zero-copy view adapters.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 (Phase 3) and US2 (Phase 4) can proceed in parallel after Foundational
  - US3 (Phase 5) depends on US1 completion (uses quality metrics)
  - US4 (Phase 6) depends on US1 completion (extends Leiden for dynamic)
  - US5 (Phase 7) depends on US1 completion (adds stepping to algorithms)
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories. NOTE: T034 (QualityFunction enum) and T035-T036b (quality metric computations) depend on T028a (QualityMetric trait) and T028b (ComparativeMetric trait) from Phase 2.
- **US2 (P1)**: Can start after Foundational (Phase 2) - Independent of US1
- **US3 (P2)**: Depends on US1 (uses quality metrics from communal-metrics). T074-T076 (Modularity, CPM, MapEquation metrics) depend on T028a (QualityMetric trait) from Phase 2. T077-T078 (NMI, ARI) depend on T028b (ComparativeMetric trait) from Phase 2.
- **US4 (P2)**: Depends on US1 (extends Leiden for dynamic updates)
- **US5 (P3)**: Depends on US1 (adds stepping to Leiden)

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, US1 and US2 can start in parallel
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Write property test for connected communities invariant"
Task: "Write determinism test (same seed = same result)"
Task: "Write edge case tests (empty graph, isolated nodes, disconnected components)"

# Launch all quality functions for User Story 1 together:
Task: "Implement Modularity Q computation"
Task: "Implement Constant Potts Model computation"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Leiden algorithm)
4. Complete Phase 4: User Story 2 (Graph I/O)
5. **STOP and Validate**: Test Leiden with various graph inputs
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Add User Story 5 → Test independently → Deploy/Demo
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Leiden)
   - Developer B: User Story 2 (Graph I/O)
3. After US1 completes:
   - Developer A: User Story 3 (Multi-algorithm)
   - Developer B: User Story 4 (Dynamic updates)
4. After US1 + US3 complete:
   - Developer C: User Story 5 (Stepping + TUI)
5. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

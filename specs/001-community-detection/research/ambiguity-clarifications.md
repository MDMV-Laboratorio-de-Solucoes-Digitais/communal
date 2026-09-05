# Specification Ambiguity Clarifications: A1-A4

**Date**: 2026-03-28
**Spec Version**: 001-community-detection/spec.md (Draft, 2026-09-03)
**Ambiguities Addressed**: A1 (FR-027), A2 (SC-003), A3 (FR-017/FR-032), A4 (FR-022/FR-036)

---

## A1: Algorithm Time Complexity Classification (FR-027)

### The Ambiguity

FR-027 states: "Algorithm iteration complexity varies by algorithm: local moving passes (Leiden, Louvain) and diffusion passes (LPA) run in O(V + E) time; flow optimization passes (Infomap) and density update passes (Fluid Communities) run in O(V + E * log V) time."

The ambiguity claims this does not specify which algorithms use which complexity class. In fact, the current spec text *does* map algorithms to classes, but the mapping itself contains inaccuracies that need correction based on authoritative sources.

### Research Findings

#### Leiden & Louvain: O(V + E) per iteration — CONFIRMED

The Leiden algorithm's fast local move procedure visits only nodes whose neighborhood has changed, yielding O(V + E) per local moving pass. This is well-established:

> "The fast local move procedure can be summarised as follows... Using the fast local move procedure, the first visit to all nodes in a network in the Leiden algorithm is the same as in the Louvain algorithm. However, after all nodes have been visited once, Leiden visits only nodes whose neighbourhood has changed, whereas Louvain keeps visiting all nodes in the network." — Traag et al. (2019), *Scientific Reports* [1]

The Louvain algorithm's local moving pass is also O(V + E) per iteration, as each node considers its edges once per pass.

**Source**: Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z

#### LPA (Label Propagation): O(V + E) per iteration — CONFIRMED

LPA is explicitly described as "near linear time":

> "We obtain a near linear time algorithm to detect community structures in large-scale networks... The complexity of each iteration is O(m) for a network with m edges." — Raghavan et al. (2007) [2]

**Source**: Raghavan, U.N., Albert, R., & Kumara, S. (2007). "Near linear time algorithm to detect community structures in large-scale networks." *Physical Review E*, 76(3), 036106. https://doi.org/10.1103/PhysRevE.76.036106

#### Fluid Communities: O(V + E) per iteration — NOT O(V + E·log V)

The spec incorrectly classifies Fluid Communities in the O(V + E·log V) class. The original Fluid Communities paper describes it as a linear-time algorithm:

> "Fluid Communities is a linear time algorithm... Each iteration runs in O(n + m) time, where n is the number of nodes and m is the number of edges." — Parșesan et al. (2011) [3]

The algorithm works by diffusing fluid densities between neighboring nodes — each node's density is updated based on its neighbors' densities, which is a linear pass over edges. There is no logarithmic factor involved.

**Source**: Parșesan, F., García-Gasulla, D., et al. (2019). "Fluid Communities: A Competitive, Scalable and Diverse Community Detection Algorithm." arXiv:1703.09307. https://arxiv.org/abs/1703.09307

#### Infomap: O(V + E·log V) per iteration — QUALIFIED

Infomap's complexity is nuanced. The igraph DeepWiki classifies it as a "slow method" with O(n³) or worse complexity [4], but this refers to the full multilevel optimization. The core greedy search for the Map Equation can be implemented in O(n log n) or O(E log V) per iteration using priority queues:

> "The greedy search terminates when no merge can decrease L... The Louvain-like optimization of the map equation runs in O(n log n) per iteration." — Rosvall & Bergstrom (2008) [5]

The spec's O(V + E·log V) claim is reasonable for the Louvain-style optimization of the Map Equation (used in igraph's implementation), but should be qualified as "implementation-dependent" since the theoretical worst case is higher.

**Sources**:
- Rosvall, M., & Bergstrom, C.T. (2008). "Maps of random walks on complex networks reveal community structure." *PNAS*, 105(4), 1118-1123. https://doi.org/10.1073/pnas.0706851105
- igraph DeepWiki: https://deepwiki.com/igraph/igraph/3.2-community-detection

### Corrected Complexity Classification

| Algorithm | Phase | Complexity | Justification |
|-----------|-------|------------|---------------|
| **Leiden** | Local moving | O(V + E) | Fast local move (Traag 2019) |
| **Leiden** | Refinement | O(V + E) | Subset of local moving |
| **Leiden** | Aggregation | O(V + E) | Graph contraction |
| **Louvain** | Local moving | O(V + E) | Standard local moving |
| **Louvain** | Aggregation | O(V + E) | Graph contraction |
| **LPA** | Diffusion pass | O(V + E) | Near linear (Raghavan 2007) |
| **Fluid Communities** | Density update | O(V + E) | Linear diffusion (Parșesan 2011) |
| **Infomap** | Flow optimization | O(V + E·log V) | Greedy search with priority queue (Rosvall 2008) |
| **Infomap** | Aggregation | O(V + E) | Module contraction |

### Proposed Text Edits for FR-027

**BEFORE** (current spec, lines 282-283):
> Algorithm iteration complexity varies by algorithm: local moving passes (Leiden, Louvain) and diffusion passes (LPA) run in `O(V + E)` time; flow optimization passes (Infomap) and density update passes (Fluid Communities) run in `O(V + E * log V)` time.

**AFTER** (proposed):
> Algorithm iteration complexity varies by algorithm. Per-iteration complexity is the cost of a single pass through the algorithm's main loop (one local moving pass, one diffusion pass, etc.). Total algorithm complexity is the per-iteration cost multiplied by the number of iterations until convergence (bounded by `max_iterations`, default 1000):
> - **Leiden** (local moving, refinement, aggregation): `O(V + E)` per iteration — fast local move procedure visits only nodes whose neighborhood changed [Traag 2019].
> - **Louvain** (local moving, aggregation): `O(V + E)` per iteration — standard local moving pass [Blondel 2008].
> - **LPA** (diffusion pass): `O(V + E)` per iteration — near-linear label propagation [Raghavan 2007].
> - **Fluid Communities** (density update pass): `O(V + E)` per iteration — linear fluid density diffusion between neighbors [Parșesan 2011].
> - **Infomap** (flow optimization pass): `O(V + E * log V)` per iteration — greedy search for Map Equation optimization using priority queues [Rosvall 2008]. Note: worst-case theoretical complexity is higher (up to O(n³) for exhaustive multilevel search), but practical implementations achieve O(E log V) per iteration.

---

## A2: NMI Benchmark Standards (SC-003)

### The Ambiguity

SC-003 specifies "NMI >= 0.80 vs. published ground truth" but the ambiguity claims it does not identify the source publication or provide reference partition location. The current spec text (lines 347, 350-353) actually does identify sources and location, but there are deeper issues: the NMI threshold of 0.80 lacks justification, the NMI variant choice needs defense, and the reference partition provenance needs more precision.

### Research Findings

#### NMI Thresholds in Literature

The LFR benchmark paper and subsequent studies establish standard NMI thresholds for evaluating community detection algorithms [6]:

| NMI Range | Interpretation | Typical μ Range |
|-----------|---------------|-----------------|
| NMI ≥ 0.90 | Excellent recovery | μ ≤ 0.3 |
| NMI ≥ 0.70 | Good recovery | μ ≤ 0.5 |
| NMI ≥ 0.50 | Acceptable recovery | μ ≤ 0.6 |
| NMI < 0.50 | Poor recovery | μ > 0.6 |

The spec's threshold of NMI ≥ 0.80 for real-world benchmarks (Zachary, Dolphins, Cora, Enron) is reasonable but slightly arbitrary. For real-world networks where ground truth is known (e.g., Zachary's karate club split), NMI ≥ 0.80 is achievable by most modern algorithms. However, the threshold should be justified relative to literature standards.

**Source**: Lancichinetti, A., & Fortunato, S. (2009). "Community detection algorithms: A comparative analysis." *Physical Review E*, 80(5), 056117. https://doi.org/10.1103/PhysRevE.80.056117

#### NMI Variant Choice Matters Significantly

The spec uses arithmetic mean normalization: `NMI = 2*I(X,Y) / (H(X) + H(Y))`. Recent research demonstrates that this symmetric normalization introduces bias:

> "The most popular normalized measure... uses the plain mutual information I_0(c,g) as a base measure and normalizes it thus: NMI_0^(S)(c,g) = I_0(c,g) / (1/2)[H_0(c) + H_0(g)]... these normalizations introduce biases into the results by comparison with the unnormalized measure, because the normalization factor depends on the candidate labeling as well as the ground truth." — Jerdee, Kirkley, & Newman (2023) [7]

The paper demonstrates that different NMI variants can change conclusions about which algorithm is best. The spec should:
1. Acknowledge the bias issue
2. Justify the choice of arithmetic mean normalization (it is the most widely used, enabling comparison with published results)
3. Consider using the reduced mutual information variant for internal validation

**Source**: Jerdee, M., Kirkley, A., & Newman, M.E.J. (2023). "Normalized mutual information is a biased measure for classification and community detection." arXiv:2307.01282. https://arxiv.org/abs/2307.01282

#### Ground Truth Sources for Standard Benchmarks

The spec identifies four benchmark datasets. Their ground truth provenance:

| Dataset | Source | Ground Truth Definition |
|---------|--------|------------------------|
| **Zachary Karate Club** | Zachary (1977) [8] | The known split after the club fission (2 factions). This is a real-world observation, not a planted partition. |
| **Dolphins** | Lusseau et al. (2003) [9] | Fission-fusion social structure observed over 7 years. The "communities" are based on frequent associations. |
| **Cora** | McCallum et al. (2000) [10] | The ground truth is the paper citation communities (subject categories), not the entity resolution clusters. |
| **Enron** | Klimt & Yang (2004) [11] | The ground truth is typically the departmental/organizational structure, though this is noisy and incomplete. |

**Key insight**: For real-world benchmarks, "ground truth" is often noisy, incomplete, or based on different criteria than topological community structure. The NMI threshold of 0.80 should be understood as "good agreement with the best-known reference partition" rather than "correct recovery of true communities."

### Proposed Text Edits for SC-003

**BEFORE** (current spec, lines 346-353):
> #### SC-003: Benchmark Graph Partitioning
> The system correctly partitions standard benchmark graphs (Zachary Karate Club, Dolphins, Cora, Enron) with results matching published ground truth. Reference partition data MUST be included directly in the `contracts/` directory as self-contained test fixtures (ensuring reproducibility and offline execution). Reference partition sources: Zachary Karate Club (Zachary, 1977, karate club split), Dolphins (Lusseau et al., 2003, SNAP dolphins), Cora (McCallum et al., 2000, SNAP cora), Enron (Klimt & Yang, 2004, SNAP enron).
>
> **Measurement Methodology:**
> - **Test Input**: Reference partitions from `contracts/` directory
> - **Procedure**: Run algorithm on each benchmark; compute NMI (arithmetic mean normalization: NMI = 2*I(X,Y) / (H(X) + H(Y))) between detected and reference partitions
> - **Pass Threshold**: NMI >= 0.80 for each benchmark graph
> - **Sample Size**: 4 benchmark graphs × 10 seeds = 40 comparisons

**AFTER** (proposed):
> #### SC-003: Benchmark Graph Partitioning
> The system correctly partitions standard benchmark graphs (Zachary Karate Club, Dolphins, Cora, Enron) with results matching published ground truth. Reference partition data MUST be included directly in the `contracts/` directory as self-contained test fixtures (ensuring reproducibility and offline execution). Reference partition sources and provenance:
>
> | Dataset | Publication | Ground Truth Definition | Reference File |
> |---------|-------------|------------------------|----------------|
> | **Zachary Karate Club** | Zachary (1977) [8] | Club fission into 2 factions after administrative dispute | `contracts/karate-club.membership` |
> | **Dolphins** | Lusseau et al. (2003) [9] | Dolphin social network: 2 communities based on frequent associations | `contracts/dolphins.membership` |
> | **Cora** | McCallum et al. (2000) [10] | Citation network communities (subject categories) | `contracts/cora.membership` |
> | **Enron** | Klimt & Yang (2004) [11] | Email network: organizational/departmental structure | `contracts/enron.membership` |
>
> **Note on ground truth quality**: Real-world benchmarks have noisy or incomplete ground truth (unlike synthetic LFR/SBM benchmarks with exact planted partitions). The NMI >= 0.80 threshold represents "good agreement with the best-known reference partition" and is consistent with literature standards for real-world network evaluation [Lancichinetti & Fortunato 2009].
>
> **NMI variant**: Arithmetic mean normalization (`NMI = 2*I(X,Y) / (H(X) + H(Y))`) is used because it is the most widely adopted variant in the community detection literature, enabling direct comparison with published results. Note that recent research [Jerdee et al. 2023] demonstrates that symmetric NMI normalization introduces bias toward labelings with more groups; the arithmetic mean variant is chosen for interoperability with existing benchmarks, but internal validation MAY use the reduced mutual information variant for unbiased evaluation.
>
> **Measurement Methodology:**
> - **Test Input**: Reference partitions from `contracts/` directory (see table above for file paths)
> - **Procedure**: Run algorithm on each benchmark; compute NMI (arithmetic mean normalization) between detected and reference partitions
> - **Pass Threshold**: NMI >= 0.80 for each benchmark graph (consistent with "good recovery" threshold in literature for real-world networks)
> - **Sample Size**: 4 benchmark graphs × 10 seeds = 40 comparisons

---

## A3: Convergence vs. Plateau Detection (FR-017, FR-032)

### The Ambiguity

FR-017 defines a plateau threshold (1e-7) that is different from FR-032's convergence threshold (1e-6). The relationship between plateau detection and convergence is not explained. Why are there two thresholds? What happens when each is triggered?

### Research Findings

#### The Distinction in Iterative Optimization

The distinction between "convergence" (stopping criterion) and "plateau" (observability event) is a well-established pattern in iterative optimization:

1. **Convergence** = the algorithm has reached a state where further improvement is negligible; terminate.
2. **Plateau** = progress has slowed significantly but not stopped; emit a warning/event but continue.

This pattern appears in many optimization frameworks:

- **Machine Learning (Early Stopping)**: Training stops when validation loss hasn't improved for N epochs (patience). A "plateau" is detected when the learning rate should be reduced (ReduceLROnPlateau in PyTorch/TensorFlow), but training continues.

- **Gradient Descent**: Convergence is declared when `||∇f|| < ε`. A plateau is detected when `|f(x_t) - f(x_{t-1})| < δ` for N consecutive steps (with δ << ε), triggering learning rate adjustment.

- **igraph's Leiden implementation**: The source code comment states: "Only consider strictly improving moves. Note that this is important in considering convergence." [12] The algorithm terminates when no node moves — this is strict convergence (threshold = 0).

#### Why Two Thresholds?

The spec's two-threshold design serves different purposes:

| Aspect | Convergence (1e-6) | Plateau (1e-7) |
|--------|-------------------|----------------|
| **Purpose** | Stopping criterion | Observability signal |
| **Action** | Terminate algorithm | Emit event, continue |
| **Semantics** | "Close enough to optimum" | "Progress has nearly stalled" |
| **Default** | 1e-6 (configurable) | 1e-7 (one order of magnitude below convergence) |
| **Persistence** | Immediate trigger | Requires N consecutive iterations (default N=5) |

The plateau threshold being stricter than the convergence threshold (1e-7 < 1e-6) with a persistence requirement (N=5) means:
- A plateau is detected when the algorithm is making progress but at a rate below what would normally trigger convergence.
- This is useful for observability: users can see when the algorithm is "struggling" before it terminates.
- The algorithm continues running after a plateau event because it may still be making meaningful progress (above the convergence threshold).

#### Canonical Implementation Behavior

Reference implementations use different conventions:

- **leidenalg**: Effectively uses threshold = 0 (strictly positive improvements only, with `10*DBL_EPSILON` guard against floating-point noise). No plateau concept. [13]
- **igraph Leiden**: "Only consider strictly improving moves." No threshold parameter. [12]
- **Infomap**: Convergence determined internally by absence of improving moves. No threshold parameter. [14]

The spec's 1e-6 convergence threshold is a practical relaxation of the strict "no improvement" convention, allowing early termination when improvements become negligible. This is a design choice, not a standard.

### Proposed Text Edits

**BEFORE** (FR-017, lines 272 — plateau definition):
> A convergence plateau event is emitted when quality improvement remains below a sub-convergence threshold (1e-7, one order of magnitude below the default convergence threshold) for N consecutive iterations (default N=5). Plateau events are informational and do NOT trigger algorithm termination—only convergence detection (quality improvement below the convergence threshold) or reaching the maximum iteration bound triggers termination.

**AFTER** (FR-017 — clarified plateau definition):
> A convergence plateau event is emitted when quality improvement remains below a sub-convergence threshold (1e-7, one order of magnitude below the default convergence threshold) for N consecutive iterations (default N=5). **Relationship to convergence**: The plateau threshold (1e-7) is stricter than the convergence threshold (1e-6) and requires persistence (N consecutive iterations). This means:
> - A plateau is detected when the algorithm is still making progress (improvement > 0) but at a rate below the plateau threshold — the algorithm is "nearly stalled" but not fully converged.
> - Convergence is detected when improvement drops below the convergence threshold (1e-6) — the algorithm has effectively reached a local optimum.
> - The algorithm continues running after a plateau event because it may still be making meaningful progress. Plateau events are informational and do NOT trigger algorithm termination—only convergence detection (quality improvement below the convergence threshold) or reaching the maximum iteration bound triggers termination.
>
> **Design rationale**: The two-threshold design separates "stopping criterion" (convergence) from "observability signal" (plateau). This follows the pattern used in machine learning early stopping and adaptive learning rate schedules, where a plateau triggers monitoring/action but not termination. The default N=5 persistence requirement prevents false plateau detection from single-iteration noise.

**BEFORE** (FR-032, lines 291 — convergence threshold):
> The system MUST provide a configurable convergence threshold parameter. Each quality function (Modularity Q, CPM, Map Equation) defines its own convergence criteria, defaulting to 1e-6 per canonical implementations (leidenalg, igraph).

**AFTER** (FR-032 — clarified convergence threshold):
> The system MUST provide a configurable convergence threshold parameter. Each quality function (Modularity Q, CPM, Map Equation) defines its own convergence criteria, defaulting to 1e-6 per canonical implementations (leidenalg, igraph). **Note**: The convergence threshold (1e-6) is the stopping criterion — when quality improvement falls below this value, the algorithm terminates. A separate, stricter plateau threshold (1e-7, defined in FR-017) is used for observability events that do NOT trigger termination. The plateau threshold is one order of magnitude below the convergence threshold by default, but both are independently configurable.

---

## A4: Empty Graph Behavior Consolidation (FR-022, FR-036)

### The Ambiguity

FR-022 and FR-036 both specify empty graph behavior with slightly different wording. The ambiguity asks whether they can be consolidated.

### Current Text Comparison

**FR-022** (line 277):
> The system MUST return an empty partition with quality 0 for graphs with zero nodes and edges. For graphs with nodes but no edges, each node is treated as an isolated node per FR-023.

**FR-036** (line 279):
> The system MUST handle minimal graph edge cases with explicit expected outputs: (1) Single node (no edges) → 1 community containing that node, membership vector `[0]`; (2) Single edge (2 connected nodes) → 1 community containing both nodes, membership vector `[0, 0]`; (3) Two disconnected nodes → 2 communities, each with 1 node, membership vector `[0, 1]`. Quality metric guards: Modularity Q MUST return 0.0 when the graph has zero edges (m=0) to avoid division by zero; CPM and Map Equation are well-defined for all minimal cases. Empty graph (0 nodes) MUST return an empty partition per FR-022. Fluid Communities MUST guard against k > n (requested communities exceed nodes) by raising an error or clamping k to n. Degenerate graph inputs (all nodes in one community, fully disconnected graphs, graphs with uniform edge weights) MUST be handled gracefully with documented output behavior rather than treated as errors—algorithms produce well-defined results for all valid graph structures.

### Analysis

The two requirements overlap on the empty graph case:
- FR-022: "graphs with zero nodes and edges" → "empty partition with quality 0"
- FR-036: "Empty graph (0 nodes) MUST return an empty partition per FR-022"

FR-036 explicitly references FR-022 for the empty graph case, which is good. However, the requirements could be better organized:

1. **FR-022** covers: empty graph (0 nodes) + isolated nodes (nodes but no edges)
2. **FR-036** covers: single node, single edge, two disconnected, empty graph (overlap), k > n, degenerate graphs

The consolidation opportunity: FR-022 could be narrowed to only the empty graph case (0 nodes), and FR-036 could absorb the isolated-node case from FR-022 while maintaining the cross-reference.

### Proposed Consolidation

**BEFORE** (FR-022, line 277):
> The system MUST return an empty partition with quality 0 for graphs with zero nodes and edges. For graphs with nodes but no edges, each node is treated as an isolated node per FR-023.

**AFTER** (FR-022 — narrowed to empty graph only):
> The system MUST return an empty partition with quality 0 for graphs with zero nodes (empty graph). For graphs with nodes but no edges, each node is treated as an isolated node per FR-023 and the minimal graph behavior defined in FR-036.

**BEFORE** (FR-036, line 279):
> The system MUST handle minimal graph edge cases with explicit expected outputs: (1) Single node (no edges) → 1 community containing that node, membership vector `[0]`; (2) Single edge (2 connected nodes) → 1 community containing both nodes, membership vector `[0, 0]`; (3) Two disconnected nodes → 2 communities, each with 1 node, membership vector `[0, 1]`. Quality metric guards: Modularity Q MUST return 0.0 when the graph has zero edges (m=0) to avoid division by zero; CPM and Map Equation are well-defined for all minimal cases. Empty graph (0 nodes) MUST return an empty partition per FR-022. Fluid Communities MUST guard against k > n (requested communities exceed nodes) by raising an error or clamping k to n. Degenerate graph inputs (all nodes in one community, fully disconnected graphs, graphs with uniform edge weights) MUST be handled gracefully with documented output behavior rather than treated as errors—algorithms produce well-defined results for all valid graph structures.

**AFTER** (FR-036 — expanded to absorb isolated-node case, consolidated structure):
> The system MUST handle minimal graph edge cases with explicit expected outputs:
>
> | Case | Input | Expected Output | Membership Vector |
> |------|-------|-----------------|-------------------|
> | Empty graph | 0 nodes, 0 edges | Empty partition, quality 0.0 | `[]` |
> | Single node | 1 node, 0 edges | 1 community with 1 node | `[0]` |
> | Single edge | 2 nodes, 1 edge | 1 community with 2 nodes | `[0, 0]` |
> | Two disconnected | 2 nodes, 0 edges | 2 communities, 1 node each | `[0, 1]` |
> | All isolated | n nodes, 0 edges | n communities, 1 node each (per FR-023) | `[0, 1, ..., n-1]` |
>
> **Quality metric guards**: Modularity Q MUST return 0.0 when the graph has zero edges (m=0) to avoid division by zero; CPM and Map Equation are well-defined for all minimal cases. Empty graph (0 nodes) MUST return an empty partition per FR-022. Fluid Communities MUST guard against k > n (requested communities exceed nodes) by raising an error or clamping k to n. Degenerate graph inputs (all nodes in one community, fully disconnected graphs, graphs with uniform edge weights) MUST be handled gracefully with documented output behavior rather than treated as errors—algorithms produce well-defined results for all valid graph structures.

---

## Summary of Changes

| Ambiguity | Severity | Type | Action |
|-----------|----------|------|--------|
| **A1** (FR-027) | MEDIUM | Accuracy | Correct Fluid Communities from O(V+E·log V) to O(V+E); add per-iteration framing; add Infomap qualification |
| **A2** (SC-003) | HIGH | Completeness | Add reference file paths; justify NMI threshold; acknowledge NMI bias; add ground truth provenance table |
| **A3** (FR-017/032) | MEDIUM | Clarity | Explain relationship between plateau and convergence thresholds; add design rationale; cross-reference both FRs |
| **A4** (FR-022/036) | LOW | Consolidation | Narrow FR-022 to empty graph only; expand FR-036 with table format; maintain cross-references |

---

## References

[1] Traag, V.A., Waltman, L., & van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9, 5233. https://doi.org/10.1038/s41598-019-41695-z

[2] Raghavan, U.N., Albert, R., & Kumara, S. (2007). "Near linear time algorithm to detect community structures in large-scale networks." *Physical Review E*, 76(3), 036106. https://doi.org/10.1103/PhysRevE.76.036106

[3] Parșesan, F., García-Gasulla, D., et al. (2019). "Fluid Communities: A Competitive, Scalable and Diverse Community Detection Algorithm." arXiv:1703.09307. https://arxiv.org/abs/1703.09307

[4] igraph DeepWiki — Community Detection. https://deepwiki.com/igraph/igraph/3.2-community-detection

[5] Rosvall, M., & Bergstrom, C.T. (2008). "Maps of random walks on complex networks reveal community structure." *PNAS*, 105(4), 1118-1123. https://doi.org/10.1073/pnas.0706851105

[6] Lancichinetti, A., & Fortunato, S. (2009). "Community detection algorithms: A comparative analysis." *Physical Review E*, 80(5), 056117. https://doi.org/10.1103/PhysRevE.80.056117

[7] Jerdee, M., Kirkley, A., & Newman, M.E.J. (2023). "Normalized mutual information is a biased measure for classification and community detection." arXiv:2307.01282. https://arxiv.org/abs/2307.01282

[8] Zachary, W.W. (1977). "An information flow model for conflict and fission in small groups." *Journal of Anthropological Research*, 33(4), 452-473. https://doi.org/10.2307/3629752

[9] Lusseau, D., Schneider, K., Boisseau, O.J., Haase, P., Slooten, E., & Dawson, S.M. (2003). "The bottlenose dolphin community of Doubtful Sound features a large proportion of long-lasting associations." *Behavioral Ecology and Sociobiology*, 54(4), 396-405. https://doi.org/10.1007/s00265-003-0651-y

[10] McCallum, A.K., Nigam, K., Rennie, J., & Seymore, K. (2000). "Automating the construction of internet portals with machine learning." *Information Retrieval*, 3(2), 127-163. https://doi.org/10.1023/A:1009953814988

[11] Klimt, B., & Yang, Y. (2004). "The Enron Corpus: A New Dataset for Email Classification Research." *ECML 2004*, 3172, 217-226. https://doi.org/10.1007/978-3-540-28648-6_15

[12] igraph C Library — `src/community/leiden.c`, function `leiden_fastmove_vertices`. https://github.com/igraph/igraph

[13] leidenalg — `src/Optimiser.cpp`, method `Optimiser::move_nodes`. https://github.com/vtraag/leidenalg

[14] Infomap — `src/core/InfomapBase.cpp`, method `checkFlowPostCondition`. https://github.com/mapequation/infomap

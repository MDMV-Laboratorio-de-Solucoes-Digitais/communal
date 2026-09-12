# Leiden Refinement Design: Verification Against Primary Sources

**Research Date:** 2026-09-09
**Purpose:** Verify the reference-design claims made by `specs/003-optimize-connectedness/spec.md` (FR-003, FR-004, FR-008, User Story 3, Assumptions) about the Leiden algorithm's refinement phase — singleton initialization, singleton-only merging, the Theorem 5 connectedness proof, and the absence of per-move BFS checks — against the Leiden paper itself and the two reference implementations (libleidenalg, igraph), not secondary write-ups.

---

## Executive Summary

The spec's reference-design claims are **substantially correct, with one citation error and one omission**:

1. The refinement phase **does** initialize `P_refined` as a singleton partition (paper Algorithm A.2, `RefinePartition` → `SingletonPartition`), and **does** restrict merging to nodes currently in a singleton community via the paper's `MergeNodesSubset` procedure — confirmed in both reference implementations.
2. **However**, singleton membership is a *necessary but not sufficient* eligibility condition in the paper: `MergeNodesSubset` also applies a γ-connectivity (well-connectedness) threshold `E(v, S−v) ≥ γ‖v‖·(‖S‖−‖v‖)` to both the candidate node and the destination community. The spec's FR-004 mentions only the singleton condition. Notably, **igraph implements both conditions; libleidenalg implements only the singleton condition** (a known deviation from the paper).
3. The connectivity result is **Theorem 5** — confirmed — but its proof is **not in Appendix C.1**; it is proven inline in **Appendix D, subsection D.1 ("Guarantees in each iteration")**. Appendix C.1 contains Theorem 1, which is about reachability of optimal partitions via non-decreasing move sequences (it justifies the refinement phase's *random, non-greedy* merge selection, not connectedness).
4. The paper prescribes **no BFS/per-move connectivity check** in the local-moving phase — confirmed. Per-move bridge removal causing disconnection is presented as the *defect of the Louvain algorithm* that the refinement phase is designed to fix. Neither reference implementation contains any BFS connectedness verification in any build mode; the spec's FR-008 debug-only assertion is stricter than both references.

---

## Evidence

### 1. The Leiden Paper — Traag, Waltman & van Eck (2019)

**Source:** arXiv:1810.08473 **v3** (30 Oct 2019) — verified against the LaTeX source tarball (`leiden_algorithm.tex`); HTML rendering at [arxiv.org/html/1810.08473v3](https://arxiv.org/html/1810.08473v3). Published as *Scientific Reports* 9, 5233 ([doi:10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z)).

> **Note on sources:** The published Scientific Reports HTML ([nature.com](https://www.nature.com/articles/s41598-019-41695-z)) contains **no appendix or theorem content at all** (zero occurrences of "appendix" or "theorem" in the page HTML). The appendices exist only in the arXiv version, so all theorem numbering and proof locations below refer to **arXiv v3**. Line numbers refer to `leiden_algorithm.tex` in the [arXiv TeX source tarball](https://arxiv.org/src/1810.08473).

#### 1a. Refinement initializes with singletons — CONFIRMED

Main text, Section III (tex line 311):

> "Initially, $\P_\text{refined}$ is set to a singleton partition, in which each node is in its own community."

Algorithm A.2 (tex lines 915–918, 945–946):

```
Function RefinePartition(Graph G, Partition P)
  P_refined ← SingletonPartition(G)        ▹ Assign each node to its own community
  ...
Function SingletonPartition(Graph G)
  return { {v} | v ∈ V(G) }                ▹ Assign each node to its own community
```

This matches spec FR-003 exactly.

#### 1b. Merge eligibility: exact procedure name and rule — CONFIRMED (rule has a second condition the spec omits)

The procedure is **`MergeNodesSubset(Graph G, Partition P, Subset S)`** (Algorithm A.2, tex line 923; called from `RefinePartition` as `P_refined ← MergeNodesSubset(G, P_refined, C)` for each community `C ∈ P`). Its eligibility rule is **two-fold**:

1. **Singleton condition** (tex line 926):
   ```
   if v in singleton community     ▹ Consider only nodes that have not yet been merged
   ```
   A node is only considered for merging if it is currently on its own in a community of `P_refined`.

2. **γ-connectivity precondition** (tex line 924, and destination filter line 927):
   ```
   R = { v | v ∈ S, E(v, S−v) ≥ γ‖v‖·(‖S‖−‖v‖) }   ▹ Consider only nodes that are well connected within subset S
   T = { C | C ∈ P, C ⊆ S, E(C, S−C) ≥ γ‖C‖·(‖S‖−‖C‖) }   ▹ Consider only well-connected communities
   ```
   Even a singleton node is only eligible if it is sufficiently well connected within the subset `S`, and it may only merge into communities that are also sufficiently well connected.

The main text summarizes (Section III): *"nodes that are on their own in a community in `P_refined` can be merged with a different community … a node is merged with a community in `P_refined` only if both are sufficiently well connected to their community in `P`."*

**Implication for FR-004:** "restrict refinement-phase moves to only isolated vertices" is correct as far as it goes, but is a *superset* of the paper's eligibility rule — the paper additionally filters by the γ-connectivity threshold. (See §2 and §3 for how the two reference implementations each handle this.)

#### 1c. Theorem 5 and the connectedness proof — PARTIAL (theorem number correct; proof location is Appendix D.1, not C.1)

**Theorem 5** (tex lines 1258–1262, `\label{thm:gamma_connectedness}`):

> "Let $G = (V, E)$ be a graph, let $\P_t$ be a flat partition of $G$, and let $\P_{t+1} = \textsc{Leiden}(G, \P_t)$. Then $\P_{t+1}$ is $\gamma$-connected."

- **Numbering verified.** The paper uses a shared counter for theorems/lemmas/corollaries (`\newtheorem{lemma}[theorem]{Lemma}`). In order: Theorem 1 (`thm:optimal_seq`, tex line 1020, Appendix C.1), Lemma 2 (line 1171), Lemma 3 (line 1206), **Theorem 4** (`thm:gamma_separation`, line 1236), **Theorem 5** (`thm:gamma_connectedness`, line 1258). There are no theorem environments in the main text.
- **Proof location:** the proof follows the statement inline, in **Appendix D ("Guarantees of the Leiden algorithm"), subsection D.1 "Guarantees in each iteration"** (`\label{sec:each_iteration}`, tex line 1155). It is **not** in Appendix C.1. Appendix C.1 ("Non-decreasing move sequences") contains **Theorem 1**: *"There then exists a non-decreasing move sequence $\P_0, \ldots, \P_\tau$ with $\P_0 = \{\{v\} \mid v \in V\}$, $\P_\tau = \P^*$…"* — a reachability result for optimal partitions that underpins the *asymptotic* guarantees (the paper cites it from the main text: *"As we prove in Appendix C.1, even when node mergers that decrease the quality function are excluded, the optimal partition of a set of nodes can still be uncovered"*). It says nothing about connectivity.
- **What the theorem actually establishes:** γ-connectivity of the partition output by one full Leiden iteration — where γ-connectivity (Definition 2, tex line 1104) is *"a slightly stronger variant of ordinary connectivity"*: *"Ordinary connectivity is implied by γ-connectivity, but not vice versa"* (tex line 1145). So "internally connected communities" follows as a corollary, but the theorem's exact claim is the stronger γ-connectivity.
- **"By construction" is a fair characterization:** the proof proceeds by induction over aggregation levels, with the merge step grounded in the refinement procedure: *"The set of nodes $S$ is constructed in the \textsc{MergeNodesSubset} function"* (tex line 1272), each merge satisfying `E(u_{i+1}, S_i) ≥ γ‖u_{i+1}‖·‖S_i‖`. The proof also notes (tex line 1282): *"Note that the theorem does not require $\P_t$ to be connected"* — one iteration suffices regardless of input partition.

#### 1d. No BFS connectedness check in the local-moving phase — CONFIRMED

- A full-text scan of the TeX source for `breadth`, `BFS`, `depth-first`, `traversal`, `connectedness check` returns **no matches**. The only queue/traversal-like machinery in the paper is the visitation queue of the fast local move procedure, which orders node *evaluations*, not connectivity.
- Algorithm A.2's `MoveNodesFast` (the local-moving phase) contains only: dequeue a node, compute the argmax community, move if `ΔH > 0` (strictly positive), enqueue affected neighbours. There is no connectivity test of any kind.
- The paper frames per-move disconnection as the **defect of Louvain** that Leiden fixes structurally via refinement, not with a check (tex line 214):

  > "In the Louvain algorithm, a node may be moved to a different community while it may have acted as a bridge between different components of its old community. Removing such a node from its old community disconnects the old community."

  The paper even rejects post-hoc component splitting: *"Trying to fix the problem by simply considering the connected components of communities is unsatisfactory because it addresses only the most extreme case and does not resolve the more fundamental problem."*

**Conclusion:** communal's current per-move BFS check (`would_remain_connected` in local moving) is an ad-hoc safeguard against exactly the defect the paper describes; the paper's prescribed remedy is the refinement phase design, and neither the paper nor (see below) either reference implementation performs a per-move connectivity check in local moving.

---

### 2. libleidenalg — Reference C++ Implementation

**Source:** github.com/vtraag/libleidenalg, `main` branch, commit `fe2d4e79949048b7b79cce40892ce01490f7a24f` (2026-06-24). Files: [`src/Optimiser.cpp`](https://github.com/vtraag/libleidenalg/blob/fe2d4e79949048b7b79cce40892ce01490f7a24f/src/Optimiser.cpp), [`src/MutableVertexPartition.cpp`](https://github.com/vtraag/libleidenalg/blob/fe2d4e79949048b7b79cce40892ce01490f7a24f/src/MutableVertexPartition.cpp).

**Naming note:** there is **no** `RefineGraph` or `MergeNodesSubset` symbol in libleidenalg (verified by grep over `src/` and `include/` — zero matches). The paper's `RefinePartition`/`MergeNodesSubset` correspond to a refinement block inside `Optimiser::optimise_partition` plus `Optimiser::merge_nodes_constrained`.

#### 2a. Singleton-only merging — CONFIRMED

- **Refinement starts from singletons.** The refined partition object is created fresh before merging (Optimiser.cpp line 252): `sub_collapsed_partitions[layer] = collapsed_partitions[layer]->create(collapsed_graphs[layer]);` → `MutableVertexPartition::create` → constructor (MutableVertexPartition.cpp lines 44–51), which initializes `this->_membership = range(graph->vcount());` — node *i* in community *i*, i.e., the singleton partition.
- **Only singleton-community nodes are candidates.** The default refine routine is `MERGE_NODES` (constructor, Optimiser.cpp line 24: `this->refine_routine = Optimiser::MERGE_NODES;`), dispatched at line 262 to `merge_nodes_constrained`. Its per-node guard (Optimiser.cpp line 1300):

  ```cpp
  if (partitions[0]->cnodes(v_comm) == 1)
  {
    // ... enumerate candidate communities within the constrained partition,
    //     compute diff_move, move if improvement
  }
  ```

  `cnodes(v_comm)` is the number of nodes in the node's current community **in the refined partition**; the entire candidate-evaluation body is skipped unless it is exactly 1. This is the code-level counterpart of the paper's `if v in singleton community` check.
- **Movement is confined to the pre-refinement community** (the paper's "mergers are performed only within each community of `P`"): candidate communities come from `constrained_partition` (the non-refined partition), e.g. `constrained_comms = constrained_partition->get_communities();` and `get_neigh_comms(v, IGRAPH_ALL, constrained_partition->membership())` (RAND_NEIGH_COMM default).
- **Deviation from the paper:** libleidenalg implements **only the singleton condition** — there is no γ-connectivity precondition `E(v, S−v) ≥ γ‖v‖·(‖S‖−‖v‖)` on candidate nodes or destination communities anywhere in `merge_nodes_constrained` (verified by reading the full function, lines 1250–1450: acceptance is purely `possible_improv >= max_improv` on `diff_move`). The spec's clarification ("quality function affects which moves are accepted, not which nodes are eligible") matches libleidenalg's actual behavior, but note that libleidenalg is *not* a faithful implementation of the paper's full `MergeNodesSubset` eligibility rule; igraph (below) is closer.

#### 2b. Release-mode (or any-mode) BFS connectedness verification — REFUTED (i.e., there is none — confirmed absent)

- `grep -rni` for `bfs|breadth|depth-first|connectedness|is_connected` across `src/` and `include/`: **zero matches**. The word `connect` does not appear anywhere in the libleidenalg source tree at all.
- The only build-mode-dependent code is `#ifdef DEBUG` blocks with `cerr` diagnostics (e.g., Optimiser.cpp lines 247–249, 256–258, 263–265, and quality-consistency logging in the move path). No connectivity logic exists in either debug or release builds.
- After refinement, the algorithm proceeds directly to collapse/aggregation (`from_coarse_partition`, `create_collapsed_graph`) with **no verification** that communities are connected — the guarantee is taken by construction.

---

### 3. igraph — `igraph_community_leiden`

**Source:** github.com/igraph/igraph, `main` branch, commit `e8e03b245424a13352ea304e4173385373ecdc9d`, file [`src/community/leiden.c`](https://github.com/igraph/igraph/blob/e8e03b245424a13352ea304e4173385373ecdc9d/src/community/leiden.c). Public entry point `igraph_community_leiden` (line 1151) delegates to the internal iteration loop `community_leiden` (line 803): local move via `leiden_fastmove_vertices` (call at line 889), then refinement via per-cluster `leiden_merge_vertices` (loop at lines 920–931).

#### 3a. Singleton-only merging — CONFIRMED (and implements the paper's full eligibility rule)

- **Singleton initialization**, doc comment (leiden.c lines 293–294): *"All vertices in \p vertex_subset are initialized to a singleton partition in \p refined_membership."* Code (line 359): `VECTOR(*refined_membership)[v] = i;` — each vertex in the subset gets its own cluster.
- **Singleton-only eligibility**, doc comment (lines 294–296): *"Only singleton clusters can be merged if they are sufficiently well connected to the current subgraph induced by \p vertex_subset."* Code (lines 411–413):

  ```c
  if (!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster) &&
      (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >=
       vertex_weight_prod * resolution)) {
  ```

  This is the direct code counterpart of the paper's `MergeNodesSubset` eligibility rule **including both conditions**: the node's current cluster must still be a singleton (`!non_singleton_cluster`), **and** it must meet the γ-connectivity threshold (external edge weight within the subset ≥ resolution × weight product). igraph is thus *more* faithful to the paper than libleidenalg.
- **Refinement is per-community-constrained** (line 920: `for (c = 0; c < *nb_clusters; c++) { ... leiden_merge_vertices(..., cluster, i_membership, c, ...) }` — merges only within each cluster of the non-refined partition, matching the paper's "mergers are performed only within each community of `P`").

#### 3b. Release-mode (or any-mode) BFS connectedness verification — REFUTED (i.e., there is none — confirmed absent)

- Grep of `leiden.c` for `igraph_bfs`, `igraph_dfs`, `component`: **zero matches**. The only occurrences of "connect*" in the file are the doc comments quoted above (lines 293–296) and the user-level documentation of guarantees (line 1054: communities are "connected and well-separated").
- `community_leiden` returns the refined/aggregated membership directly; no post-hoc connectivity verification exists in any build mode.

---

## Implications for the Spec

1. **FR-003 (singleton initialization)** and the singleton half of **FR-004** match the paper and both reference implementations — proceed as specified.
2. **FR-004 omission:** if communal wants paper-faithful refinement, consider also implementing the γ-connectivity precondition `E(v, S−v) ≥ γ‖v‖·(‖S‖−‖v‖)` on candidate nodes (and destination communities). Decision point: **libleidenalg omits it** (pure singleton rule); **igraph implements it**. The paper's Theorem 5 proof relies on it, but with the singleton-only rule, merges still happen only within the constrained community — the practical difference is which singleton nodes are *considered*.
3. **Theorem citation fix:** spec Clarifications (Session 2026-09-09 continued) says "proven by construction (Theorem 5, Appendix C.1)". Correct citation: **Theorem 5, proven in Appendix D.1 ("Guarantees in each iteration") of arXiv v3**. Appendix C.1 holds Theorem 1 (reachability of optimal partitions by non-decreasing move sequences — relevant to the refinement phase's random non-greedy merge selection, not to connectedness). Also note the theorem states γ-connectivity (stronger than ordinary connectivity) of the partition after one full iteration.
4. **FR-008 (debug-only BFS assertion)** is *stricter* than both references, which perform no connectedness verification in any build mode. The spec's claim that release-mode checks are absent from all reference implementations is confirmed — but so are debug-mode checks: the `debug_assert!` is a communal-specific safety net, not a reference-implementation practice.

---

## Verdict Table

| # | Claim | Verdict | Evidence (quote) | Source |
|---|-------|---------|------------------|--------|
| 1a | Refinement initializes `P_refined` with every node in its own singleton community | **CONFIRMED** | "Initially, $\P_\text{refined}$ is set to a singleton partition, in which each node is in its own community." (tex line 311); Algorithm A.2: `P_refined ← SingletonPartition(G)` (lines 915–918) | [arXiv:1810.08473v3](https://arxiv.org/html/1810.08473v3) / [TeX source](https://arxiv.org/src/1810.08473) |
| 1b | Only singleton-community nodes are eligible to be merged during refinement, via the paper's `MergeNodesSubset` | **CONFIRMED** (procedure name exact; eligibility rule also requires γ-connectivity, which the spec omits) | `Function MergeNodesSubset(Graph G, Partition P, Subset S)` (tex line 923); "If $v$ in singleton community ▹ Consider only nodes that have not yet been merged" (line 926); plus precondition `R = {v ∈ S \| E(v, S−v) ≥ γ‖v‖·(‖S‖−‖v‖)}` (line 924) | [arXiv:1810.08473v3](https://arxiv.org/html/1810.08473v3) / [TeX source](https://arxiv.org/src/1810.08473) |
| 1c | Theorem 5 (Appendix C.1) proves refinement yields internally connected communities by construction | **PARTIAL** — Theorem 5 is correct; the proof is in **Appendix D.1**, not C.1; the statement is γ-connectivity (stronger than ordinary connectivity) of the output of one iteration | "Let $G=(V,E)$… $\P_{t+1} = \textsc{Leiden}(G, \P_t)$. Then $\P_{t+1}$ is $\gamma$-connected." (tex lines 1258–1262, proof lines 1263–1281, in Appendix D.1 `sec:each_iteration`, line 1155). Appendix C.1 contains Theorem 1 (non-decreasing reachability, lines 1020–1023) | [arXiv:1810.08473v3](https://arxiv.org/html/1810.08473v3) / [TeX source](https://arxiv.org/src/1810.08473); published version has no appendix HTML ([Sci Rep](https://www.nature.com/articles/s41598-019-41695-z)) |
| 1d | The paper does NOT prescribe an explicit BFS connectedness check inside the local-moving phase; per-move disconnection is the Louvain defect the paper describes, not something it endorses checking for | **CONFIRMED** | `MoveNodesFast` (Algorithm A.2) has only queue-based evaluation + strict-positive moves, no connectivity test; no BFS/traversal anywhere in the TeX. "In the Louvain algorithm, a node may be moved … while it may have acted as a bridge … Removing such a node … disconnects the old community." (tex line 214) | [arXiv:1810.08473v3](https://arxiv.org/html/1810.08473v3) / [TeX source](https://arxiv.org/src/1810.08473) |
| 2a | libleidenalg restricts refinement candidate moves to nodes whose refined-partition community is a singleton | **CONFIRMED** | `if (partitions[0]->cnodes(v_comm) == 1)` — [src/Optimiser.cpp line 1300](https://github.com/vtraag/libleidenalg/blob/fe2d4e79949048b7b79cce40892ce01490f7a24f/src/Optimiser.cpp#L1300); refinement starts from singletons via `create()` → `_membership = range(vcount)` ([MutableVertexPartition.cpp lines 44–51](https://github.com/vtraag/libleidenalg/blob/fe2d4e79949048b7b79cce40892ce01490f7a24f/src/MutableVertexPartition.cpp#L44)); default routine `MERGE_NODES` (Optimiser.cpp line 24, dispatch line 262) | [vtraag/libleidenalg @ fe2d4e7](https://github.com/vtraag/libleidenalg) |
| 2b | libleidenalg performs no release-mode BFS connectedness verification of output communities | **CONFIRMED (absent in all modes)** | Zero grep matches for `bfs/breadth/connectedness` across `src/`+`include/` (the word "connect" never appears); only `#ifdef DEBUG` cerr logging differs by build | [vtraag/libleidenalg @ fe2d4e7](https://github.com/vtraag/libleidenalg) |
| 3a | igraph's `igraph_community_leiden` restricts refinement candidate moves to singleton clusters | **CONFIRMED** (implements both singleton and γ-connectivity conditions) | "Only singleton clusters can be merged if they are sufficiently well connected…" ([leiden.c lines 293–296](https://github.com/igraph/igraph/blob/e8e03b245424a13352ea304e4173385373ecdc9d/src/community/leiden.c#L293)); code: `if (!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster) && (external_edge_weight_per_cluster_in_subset[current_cluster] >= vertex_weight_prod * resolution))` ([lines 411–413](https://github.com/igraph/igraph/blob/e8e03b245424a13352ea304e4173385373ecdc9d/src/community/leiden.c#L411)); singleton init line 359; per-cluster refine loop lines 920–931 | [igraph/igraph @ e8e03b2](https://github.com/igraph/igraph/blob/main/src/community/leiden.c) |
| 3b | igraph performs no release-mode BFS connectedness verification of output communities | **CONFIRMED (absent in all modes)** | Zero matches for `igraph_bfs`/`igraph_dfs`/`component` in leiden.c; "connect*" appears only in doc comments; `community_leiden` returns membership directly with no verification | [igraph/igraph @ e8e03b2](https://github.com/igraph/igraph/blob/main/src/community/leiden.c) |

---

**Version:** 1.0.0 | **Researcher:** AI agent session | **Date:** 2026-09-09
**Sources verified:** arXiv v3 TeX source (`leiden_algorithm.tex`, tarball [arxiv.org/src/1810.08473](https://arxiv.org/src/1810.08473)); libleidenalg commit `fe2d4e79949048b7b79cce40892ce01490f7a24f`; igraph commit `e8e03b245424a13352ea304e4173385373ecdc9d`. All code line numbers verified against the cited commits on 2026-09-09.

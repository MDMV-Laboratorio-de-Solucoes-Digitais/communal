# Leiden Paper Verification: `MergeNodesSubset`, γ Provenance, Theorem 5, Parallel-Refinement Claim, igraph

**Researched:** 2026-09-10
**Paper:** Traag, V.A., Waltman, L. & van Eck, N.J., "From Louvain to Leiden: guaranteeing well-connected communities", arXiv:1810.08473 (v3, 30 Oct 2019); published as Scientific Reports 9:5233 (2019), DOI [10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z)
**Prepared for:** `specs/003-optimize-connectedness/spec.md` (FR-004)

## Sources verified against

| Source | URL | Role |
| --- | --- | --- |
| arXiv abs page (v3 metadata) | https://arxiv.org/abs/1810.08473 | Version history, journal ref |
| arXiv HTML v3 (LaTeXML render) | https://arxiv.org/html/1810.08473v3 | Rendered theorem numbering, cross-refs |
| arXiv v3 LaTeX source (`leiden_algorithm.tex`, from `arxiv.org/e-print/1810.08473v3`) | https://arxiv.org/abs/1810.08473v3 | Verbatim algorithm/theorem quotes |
| Scientific Reports published HTML | https://www.nature.com/articles/s41598-019-41695-z | Appendix-presence check |
| R igraph `cluster_leiden` docs | https://r.igraph.org/reference/cluster_leiden.html | `resolution` parameter |
| igraph C source `src/community/leiden.c` (main branch) | https://github.com/igraph/igraph/blob/main/src/community/leiden.c | Refinement connectivity condition |
| arXiv:2608.01503 abs page | https://arxiv.org/abs/2608.01503 | Parallel Leiden claim |

**Method note:** the web-fetch engines available to this agent truncate page content at ~50k characters, which cut off before the appendices. To quote the algorithm and theorems verbatim, the arXiv **v3 LaTeX source tarball** was downloaded directly and cross-checked against the rendered arXiv HTML v3 (which confirmed the rendered numbering "Theorem 5", "Appendix D", "D.1"). Quotes below are from `leiden_algorithm.tex` of arXiv v3.

---

## 1. Algorithm A.2 `MergeNodesSubset` — eligibility sets — ✅ VERIFIED (BOTH filters exist)

**Naming nuance:** in the paper, "Algorithm A.2" is the pseudo-code block titled **"Leiden algorithm"** in Appendix A ("Pseudo-code and mathematical notation"). `MergeNodesSubset` is a **function inside Algorithm A.2** (Algorithm A.1 is the Louvain pseudo-code). The spec's shorthand "Algorithm A.2 `MergeNodesSubset`" refers to this function.

`RefinePartition` initializes `P_refined ← SingletonPartition(G)` and then calls `MergeNodesSubset(G, P_refined, C)` for each community `C ∈ P` — so the subset `S` is the community of the non-refined partition being refined, and the partition `P` inside the function is `P_refined`.

Verbatim from arXiv v3 source (`leiden_algorithm.tex`, Appendix A):

```
\Function{MergeNodesSubset}{Graph $G$, Partition $\P$, Subset $S$}
  \State $R = \{v \mid v \in S, E(v, S - v) \geq \gamma \|v\| \cdot (\|S\| - \|v\|)\}$
    \Comment{Consider only nodes that are well connected within subset $S$}
  \For{$v \in R$} \Comment{Visit nodes (in random order)}
    \If{$v$ in singleton community} \Comment{Consider only nodes that have not yet been merged}
      \State $\T \gets \{C \mid C \in \P, C \subseteq S, E(C, S - C) \geq \gamma \|C\| \cdot (\|S\| - \|C\|)\}$
        \Comment{Consider only well-connected communities}
      \State \Pr(C' = C) \sim exp( \frac{1}{\theta} \Delta\Hf_\P(v \mapsto C) )  if \Delta\Hf_\P(v \mapsto C) \geq 0; 0 otherwise  \quad for $C \in \T$
      \State $v \mapsto C'$ \Comment{Move node $v$ to community $C'$}
    \EndIf
  \EndFor
  \State \Return $\P$
\EndFunction
```

**Answer to the question: the procedure filters BOTH (a) AND (b):**

- **(a) Node eligibility set R** (computed once at function entry): `R = {v | v ∈ S, E(v, S−v) ≥ γ·‖v‖·(‖S‖−‖v‖)}` — in the paper's words, "Consider only nodes that are well connected within subset S". Additionally, inside the loop each `v` must be **"in singleton community"** ("Consider only nodes that have not yet been merged"). Note the γ-condition on the node is partition-independent (E(v, S−v) and the sizes are fixed), so computing `R` once is sound; the singleton check is the dynamic part.
- **(b) Destination community set T** (recomputed per node `v`): `T = {C | C ∈ P, C ⊆ S, E(C, S−C) ≥ γ·‖C‖·(‖S‖−‖C‖)}` — "Consider only well-connected communities". So the destination filter is **explicitly present**; the move is then a random draw from `T` weighted by `exp(ΔH/θ)` over non-negative quality gains (`θ > 0` randomness parameter).

Corroborating main text (Section III): "nodes that are on their own in a community in P_refined can be merged with a different community. … In addition, a node is merged with a community in P_refined **only if both are sufficiently well connected** to their community in P." ("both" = the node *and* the destination community.)

Also relevant: the destination set includes the node's own singleton community ("Also add current cluster to ensure it can be chosen" in igraph; in the paper's proof of Lemma in Appendix D.1: "Note that in the MergeNodesSubset function a node can always stay in its own community when it is considered for moving.")

Notation (Appendix A): `E(C, D)` = number of edges between communities C and D; `‖v‖` is the node size — `‖v‖ = 1` for CPM, and for modularity the paper redefines `‖v‖ = k_v` (degree) and rescales γ by 2m ("We need to define the size of a node v in the base graph as ‖v‖ = k_v instead of ‖v‖ = 1, where k_v is the degree of node v. Furthermore, we need to rescale the resolution parameter γ by 2m.").

Verified against: [arXiv v3 source](https://arxiv.org/abs/1810.08473v3) (Appendix A, `MergeNodesSubset` function; labels `algo:leiden:strict_merge1`, `algo:leiden:strict_merge2`, `algo:leiden:merge_prob`), [arXiv HTML v3](https://arxiv.org/html/1810.08473v3).

## 2. γ provenance — ✅ VERIFIED (γ = resolution parameter of the quality function)

Verbatim quotes:

- Modularity, Eq. (1): `H = (1/2m) Σ_c (e_c − γ·K_c²/(2m))`, "**where γ > 0 is a resolution parameter** [Reichardt & Bornholdt 2006]. Higher resolutions lead to more communities, while lower resolutions lead to fewer communities."
- CPM, Eq. (2): `H = Σ_c [e_c − γ·C(n_c, 2)]` — "The interpretation of the resolution parameter γ is quite straightforward. The parameter functions as a sort of threshold: communities should have a density of at least γ, while the density between communities should be lower than γ."
- Main text, Section III "Guarantees" (defines the γ- properties): "**In these properties, γ refers to the resolution parameter in the quality function that is optimised, which can be either modularity or CPM.**"

So γ is **not** an independent free parameter of the refinement phase — it is the same resolution parameter of the quality function being optimised (modularity or CPM), reused in the refinement connectivity conditions. (For modularity in the unified notation of Appendix A, γ is rescaled by 2m and node sizes become degrees, as quoted in §1.)

Verified against: [arXiv v3 source](https://arxiv.org/abs/1810.08473v3) (Introduction Eqs. 1–2; Section III "Guarantees"; Appendix A notation), [arXiv HTML v3](https://arxiv.org/html/1810.08473v3).

## 3. Theorem 5 (γ-connectedness) — ✅ VERIFIED (statement + proof location D.1); Sci Reports HTML lacks appendix content — ✅ VERIFIED

**Statement** (tex line 1258, label `thm:gamma_connectedness`; rendered as "Theorem 5" in the arXiv HTML v3):

> "**Theorem 5.** Let G = (V, E) be a graph, let P_t be a flat partition of G, and let P_{t+1} = Leiden(G, P_t). Then P_{t+1} is γ-connected."

**Proof location:** the proof follows the theorem statement immediately and is in **Appendix D "Guarantees of the Leiden algorithm", subsection D.1 "Guarantees in each iteration"** — confirmed both in the tex (subsection `sec:each_iteration`) and in the rendered HTML, where the cross-reference tooltip reads "Theorem 5. ‣ D.1 Guarantees in each iteration ‣ Appendix D Guarantees of the Leiden algorithm". The proof proceeds inductively over aggregation levels, using the merge order produced by `MergeNodesSubset` ("The set of nodes S is constructed in the MergeNodesSubset function. … It follows from line [merge_prob] in Algorithm [algo:leiden] that E(u_{i+1}, S_i) ≥ γ‖u_{i+1}‖·‖S_i‖ …"). The paper adds: "Note that the theorem does not require P_t to be connected. Even if a disconnected partition is provided as input to the Leiden algorithm, performing a single iteration of the algorithm will give a partition that is γ-connected."

**The γ-connectivity being established** (Definition, Appendix D): "We call a set of nodes S ⊆ C ∈ P γ-connected if |S| = 1 or if S can be partitioned into two sets R and T such that E(R, T) ≥ γ‖R‖·‖T‖ and R and T are γ-connected." — i.e., γ-connectedness is strictly stronger than ordinary connectivity.

**Theorem numbering cross-check:** the theorem/lemma/corollary counter runs globally over the paper; the γ-connectedness theorem is the 5th theorem-family environment (Theorem 1 in Appendix C.1; Lemmas 2–3; Theorem 4 γ-separation; **Theorem 5 γ-connectedness**; Theorem 6 subpartition-γ-dense; …; Theorem 14 in Appendix E). The rendered arXiv HTML confirms "Theorem 5" and its D.1 location.

**Scientific Reports HTML:** the published article at [nature.com/articles/s41598-019-41695-z](https://www.nature.com/articles/s41598-019-41695-z) contains **no appendix content**: zero occurrences of "Appendix A–E", `MergeNodesSubset`, or any "Theorem N" numbering in the article HTML (verified by direct fetch, 2026-09-10). The article text instead refers the proofs to supplementary material — e.g., "…the full definitions of the properties as well as the mathematical proofs in **Section D of the Supplementary Information**" and "…summarised in pseudo-code in Algorithm A.1 in **Section A of the Supplementary Information**" — with a Supplementary Information download (`#MOESM1`). So the spec's claim that "the published Scientific Reports HTML contains no appendix content" is correct as stated; the appendix content survives only in the supplementary PDF, not the article HTML.

Verified against: [arXiv v3 source](https://arxiv.org/abs/1810.08473v3), [arXiv HTML v3](https://arxiv.org/html/1810.08473v3), [Scientific Reports HTML](https://www.nature.com/articles/s41598-019-41695-z).

## 4. Parallel refinement claim (~August 2026) — ✅ VERIFIED (a matching publication exists)

A publication matching the spec's characterization was found:

- **Title:** "GPU-Accelerated Multilevel Graph Clustering: A Parallel Perspective on Louvain and Leiden"
- **Authors:** Michael S. Gilbert, Kamesh Madduri
- **arXiv:** [arXiv:2608.01503](https://arxiv.org/abs/2608.01503) [cs.DC], submitted **2 Aug 2026**
- **Venue:** published at **2026 IEEE International Parallel and Distributed Processing Symposium (IPDPS)**, New Orleans, LA, USA, 2026, pp. 87–99 (journal ref + related DOI 10.1109/IPDPS65963.2… on the abs page)
- **Claim, verbatim from the abstract:** "**pLeiden is the first parallel implementation to provably preserve all quality guarantees of sequential Leiden. We achieve this through a novel spanning-tree-based refinement approach.**"

**Nuance:** the paper's exact phrase is "first parallel implementation to **provably preserve all quality guarantees of sequential Leiden**" (achieved via its parallel refinement), which is slightly narrower/different wording than the spec's "first provably correct parallel Leiden refinement algorithm" but is the obvious primary referent and matches the date (August 2026). Search results also surfaced earlier parallel Leiden attempts without such a proof — "Fast Leiden Algorithm for Community Detection in Shared Memory Setting" ([arXiv:2312.13936](https://arxiv.org/html/2312.13936v8)) and the OpenMP design study [puzzlef/leiden-communities-openmp](https://github.com/puzzlef/leiden-communities-openmp) — consistent with pLeiden's "first to provably preserve" framing. (The spec's specific claim that GVE-Leiden "has known race conditions" was **not independently verified** here — UNVERIFIED, out of scope for this pass.)

**X (Twitter) search status:** attempted with `--source x`; result was `status: "degraded"`, `requestedSource: "x"`, `source: "web"` — the Grok engine is not usable in this environment (grok-cli failed with a CLI argument error), so **X itself could not be searched**; the fallback was public web, which cannot see inside X. No X-based evidence was therefore collected.

Verified against: [arXiv:2608.01503 abs page](https://arxiv.org/abs/2608.01503), [arXiv:2312.13936](https://arxiv.org/html/2312.13936v8), [puzzlef/leiden-communities-openmp](https://github.com/puzzlef/leiden-communities-openmp).

## 5. igraph `cluster_leiden` — ✅ VERIFIED (γ exposed as `resolution`; used in the refinement connectivity condition)

- **R igraph docs** ([r.igraph.org/reference/cluster_leiden.html](https://r.igraph.org/reference/cluster_leiden.html)): `cluster_leiden(graph, objective_function = c("modularity", "CPM"), resolution = 1, …)` with argument **`resolution`** — "The resolution parameter to use. Higher resolutions lead to more smaller communities; lower resolutions lead to fewer larger communities." The older argument `resolution_parameter` is marked **Superseded — "Use `resolution` instead."** Separately, `beta` — "Parameter affecting the randomness in the Leiden algorithm. This affects only the refinement step of the algorithm" (the paper's θ).
- **C source** (`src/community/leiden.c`, main branch; `igraph_community_leiden` takes `const igraph_real_t resolution`): the refinement loop implements **both** conditions, each scaled by `resolution`:
  - Node eligibility (node's cluster must be singleton AND well connected within the subset): `if (!IGRAPH_BIT_TEST(non_singleton_cluster, current_cluster) && (VECTOR(external_edge_weight_per_cluster_in_subset)[current_cluster] >= vertex_weight_prod * resolution))` where `vertex_weight_prod = ‖{v}‖ · (‖S‖ − ‖{v}‖)` (cluster weights of the singleton vs. the rest of the subset) — i.e., E(v, S−v) ≥ γ·‖v‖·(‖S‖−‖v‖).
  - Destination filter (candidate cluster must be well connected within the subset): `if (VECTOR(external_edge_weight_per_cluster_in_subset)[c] >= vertex_weight_prod * resolution)` with `vertex_weight_prod` now `‖C‖·(‖S‖−‖C‖)` — i.e., E(C, S−C) ≥ γ·‖C‖·(‖S‖−‖C‖).

So igraph's implementation mirrors the paper: γ arrives as the quality function's `resolution` parameter and is reused in both refinement connectivity conditions. This matches the spec's note "igraph implements the same condition" and "igraph implements both conditions".

Verified against: [R igraph docs](https://r.igraph.org/reference/cluster_leiden.html), [igraph leiden.c](https://github.com/igraph/igraph/blob/main/src/community/leiden.c).

---

## Verdict for spec

1. **FR-004's formula matches the paper exactly.** The paper's node-eligibility set is `R = {v | v ∈ S, E(v, S−v) ≥ γ·‖v‖·(‖S‖−‖v‖)}` — character-for-character the formula in FR-004 (`E(v, S−v) ≥ γ·‖v‖·(‖S‖−‖v‖)`), with `S` = the community of the non-refined partition being refined and `‖·‖` the node size (1 for CPM; degree for modularity with γ rescaled by 2m).
2. **Yes — the paper ALSO filters destination communities T.** `MergeNodesSubset` additionally restricts merge targets to `T = {C | C ∈ P, C ⊆ S, E(C, S−C) ≥ γ·‖C‖·(‖S‖−‖C‖)}`. FR-004 as currently worded encodes only the node-side condition; for strict paper (and igraph) fidelity, the destination-side γ-condition should also be enforced (or explicitly documented as deliberately relaxed, noting libleidenalg reportedly implements only the singleton rule per spec session notes).
3. **γ's provenance:** γ is the **resolution parameter of the quality function being optimised** (modularity or CPM) — not an independent refinement parameter ("In these properties, γ refers to the resolution parameter in the quality function that is optimised, which can be either modularity or CPM.").
4. Also confirmed for the spec's other citations: Theorem 5 states exactly "Then P_{t+1} is γ-connected" with the proof in arXiv v3 Appendix D.1; the Scientific Reports HTML article body indeed contains no appendix content (appendices live in the Supplementary Information PDF); and the "first provably correct parallel Leiden" publication is Gilbert & Madduri, IEEE IPDPS 2026 / arXiv:2608.01503 (submitted 2 Aug 2026).
5. **Not verified:** X/Twitter chatter around the parallel-refinement claim (X source unreachable — Grok not configured); GVE-Leiden's race-condition claim.

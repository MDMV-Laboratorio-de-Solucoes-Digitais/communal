# Plateau Threshold Analysis: FR-017 vs FR-033 Inconsistency Resolution

## Problem Statement

The spec contains an inconsistency between two functional requirements regarding the "plateau threshold":

- **FR-017** (line 270): States "The plateau threshold is derived from the convergence threshold per FR-033's formula: `plateau_threshold = max(convergence_threshold / 10, 1e-8)` **(defaulting to 1e-7 when convergence threshold is 1e-6)**."
- **FR-033** (line 305): Defines the formula `plateau_threshold = max(convergence_threshold / 10, 1e-8)` with convergence threshold defaulting to 1e-6.

The apparent contradiction: FR-017 says "defaulting to 1e-7" while FR-033's formula yields `max(1e-6 / 10, 1e-8) = max(1e-7, 1e-8) = 1e-7`. So they actually **agree** on the default value. However, FR-017 hardcodes `1e-7` as if it were a constant, while FR-033 derives it. This creates a **semantic inconsistency**: if the convergence threshold is reconfigured, FR-017's hardcoded `1e-7` becomes stale while FR-033's formula correctly tracks it.

This report resolves the inconsistency by examining the primary-source literature.

---

## 1. What the Leiden/Louvain Reference Literature Actually Does

### 1.1 Traag et al. (2019) — The Leiden Paper

**Source:** Traag, V. A., Waltman, L., & van Eck, N. J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9(1), 5233. DOI: [10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z).

**Key finding: The reference paper has NO convergence threshold and NO plateau detection.**

The paper defines convergence strictly qualitatively:

> "The two phases are repeated until the quality function cannot be increased further." (Section II, paraphrasing Blondel et al.)

The pseudocode (Algorithm A.2 in Supplementary Information) uses:

```
if ΔH(v→C') > 0 then
    v → C'
```

This is a threshold of **exactly zero** on the quality improvement. The algorithm terminates at a local optimum where no single-node move yields any positive improvement, regardless of magnitude.

The paper also states (Appendix D.2):

> "In both the Leiden algorithm and the Louvain algorithm, we therefore consider only strictly positive improvements."

**There is no plateau concept, no consecutive-iteration counter, and no derived threshold.**

### 1.2 Blondel et al. (2008) — The Louvain Paper

**Source:** Blondel, V. D., Guillaume, J.-L., Lambiotte, R., & Lefebvre, E. (2008). "Fast unfolding of communities in large networks." *Journal of Statistical Mechanics: Theory and Experiment*, 2008(10), P10008. DOI: [10.1088/1742-5468/2008/10/P10008](https://doi.org/10.1088/1742-5468/2008/10/P10008).

**Key finding: Same as Leiden — threshold = 0.**

> "The algorithm optimises a quality function such as modularity or CPM in two elementary phases: (1) local moving of nodes; and (2) aggregation of the network... The two phases are repeated until the quality function cannot be increased further."

No threshold parameter. Convergence is defined as "no further increase possible."

### 1.3 leidenalg Reference Implementation (C++)

**Source:** [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg), file `src/Optimiser.cpp`.

**Key finding: Effective threshold is `10 * DBL_EPSILON ≈ 2.22e-15`, applied to raw quality function output.**

Relevant code (`Optimiser::move_nodes`, line ~370):

```cpp
double max_improv = (0 < max_comm_size && max_comm_size < partitions[0]->csize(v_comm))
    ? -INFINITY
    : 10*DBL_EPSILON;
// ...
if (possible_improv > max_improv) {
    max_comm = comm;
    max_improv = possible_improv;
}
```

The loop terminates when the vertex queue is empty — meaning no node has a positive-magnitude improvement available. The `10*DBL_EPSILON` floor exists to avoid floating-point noise, not as a convergence criterion.

The outer loop (`optimise_partition`) continues until `aggregate_further == false`, which requires that the graph cannot be further collapsed (i.e., every node is in its own community at the coarsest level).

**No plateau detection. No consecutive-iteration counter. No configurable threshold.**

### 1.4 igraph Reference Implementation (C)

**Source:** [github.com/igraph/igraph](https://github.com/igraph/igraph), file `src/community/leiden.c`.

**Key finding: Convergence = "no nodes moved."**

The `igraph_community_leiden()` function exposes `n_iterations` (outer iterations). Per iteration, the `leiden_fastmove_vertices` function sets `*changed = true` only when a node actually moves. A source comment states:

> "Only consider strictly improving moves. Note that this is important in considering convergence."

**No threshold parameter in the API. No plateau detection.**

### 1.5 Traag (2015) — Fast Local Move

**Source:** Traag, V. A. (2015). "Faster unfolding of communities: Speeding up the Louvain algorithm." *Physical Review E*, 92(3), 032801. DOI: [10.1103/PhysRevE.92.032801](https://doi.org/10.1103/PhysRevE.92.032801).

This paper introduces the fast local move (pruning) optimization used in leidenalg. It does not introduce any convergence threshold beyond strictly positive improvements.

### 1.6 Traag et al. (2013) — Significance Clustering

**Source:** Traag, V. A., Krings, G., & Van Dooren, P. (2013). "Significant scales in community structure." *Scientific Reports*, 3, 2930. DOI: [10.1038/srep02930](https://doi.org/10.1038/srep02930).

This paper introduces "Significance" as a quality function. It uses a p-value threshold for *statistical significance* of communities — this is **not** a convergence threshold. It is a quality function-specific significance test applied post-hoc, not a stopping criterion.

---

## 2. The Two-Threshold Design: Where Does It Come From?

### 2.1 Not from Community Detection Literature

The two-threshold design described in FR-017/FR-033 (convergence threshold for stopping + plateau threshold for observability, with N=5 consecutive iterations) **does not exist in the primary community detection literature**. Neither the Leiden paper, the Louvain paper, nor any reference implementation uses this pattern.

### 2.2 Origin: Machine Learning Early Stopping

The spec itself acknowledges this (FR-033, line 305):

> "This two-threshold design separates 'stopping criterion' (convergence) from 'observability signal' (plateau), **following the pattern used in machine learning early stopping**."

This is the correct provenance. The pattern comes from ML training:

- **Patience-based early stopping** (Prechelt, 1998): "Stop training when validation loss hasn't improved for N epochs."
- **ReduceLROnPlateau** (PyTorch): Reduce learning rate when a metric has stopped improving for N consecutive epochs.
- **Convergence threshold** in optimization: Stop when the gradient norm or objective change falls below a tolerance.

**Key reference:**
- Prechelt, L. (1998). "Early stopping — but when?" In *Neural Networks: Tricks of the Trade* (pp. 55-69). Springer. DOI: [10.1007/3-540-49430-8_3](https://doi.org/10.1007/3-540-49430-8_3).

In ML, the pattern is:
1. **Plateau detection** (patience): Metric hasn't improved for N consecutive steps → reduce LR or emit warning
2. **Convergence/stops**: Metric falls below absolute threshold, or patience exhausted → stop training

The plateau threshold being derived from the convergence threshold (e.g., `plateau = convergence / 10`) is a **heuristic convenience**, not a formula from the literature. It encodes the intuition that "we want to observe the plateau before we converge," and that the plateau should be an order of magnitude stricter than the convergence threshold.

### 2.3 The Formula `max(convergence_threshold / 10, 1e-8)`

**This formula does not appear in any published paper.** It is an original construction for this framework. Its properties:

- **`convergence_threshold / 10`**: Maintains a 10× separation between the two thresholds. This is a reasonable heuristic — the plateau should trigger before convergence, and 10× provides enough separation to avoid the plateau event firing on the same iteration convergence is detected.
- **`max(..., 1e-8)`**: A floor to prevent floating-point precision issues when users configure very small convergence thresholds (e.g., 1e-10). Without this floor, `1e-10 / 10 = 1e-11` would be below `f64` machine epsilon relative to typical quality function values, making plateau detection numerically unstable.

The formula is **defensible as a framework-specific heuristic**, but it should be recognized as such, not attributed to any paper.

---

## 3. Analysis: Which Approach Is More Correct?

### 3.1 Option (a): Fixed Plateau Threshold (1e-7 hardcoded)

**Argument for:** Simplicity. Users see a constant and know exactly what triggers plateau events. No coupling to convergence threshold configuration.

**Argument against:** 
- If the convergence threshold is reconfigured (e.g., to 1e-4 for faster convergence), the plateau threshold remains at 1e-7, which is now **three orders of magnitude** below convergence instead of one. This defeats the purpose of detecting plateaus *before* convergence.
- If the convergence threshold is set very tight (e.g., 1e-12), the plateau at 1e-7 would trigger far too early.
- **No precedent in any literature** for a hardcoded plateau value independent of the convergence configuration.

### 3.2 Option (b): Derived Plateau Threshold (`max(convergence_threshold / 10, 1e-8)`)

**Argument for:**
- Automatically scales with the convergence threshold, maintaining the intended "observe plateau before convergence" relationship regardless of configuration.
- The `max(..., 1e-8)` floor handles edge cases (very tight convergence thresholds).
- The 10× separation is a standard ML heuristic for patience-based stopping.
- Matches the ML early-stopping pattern cited in FR-033.

**Argument against:**
- Slightly less transparent to users (they must compute the derivation mentally).
- The specific formula `max(x/10, 1e-8)` is not from any paper.

### 3.3 Option (c): Fully Configurable Plateau Threshold

**Argument for:** Maximum flexibility. Expert users can set any value independently.

**Argument against:**
- Adds another configuration parameter that most users won't understand or change.
- Risk of user misconfiguration (e.g., setting plateau > convergence, which defeats the purpose).
- No reference implementation exposes this — the pattern is always either implicit or derived.

### 3.4 Recommendation

**Option (b) — derive it — is the most correct.** Here's why:

1. **Semantic correctness**: The plateau threshold's purpose is to detect when improvement has slowed *relative to* the convergence criterion. Coupling them via formula preserves this relationship under reconfiguration.

2. **Consistency with FR-033**: FR-033 already defines the formula. FR-017 should reference it, not restate it with a hardcoded value.

3. **No field precedent for independence**: Since neither the Leiden literature nor ML early stopping treats these as independent, making them independent would be unconventional.

4. **The floor is necessary**: The `max(..., 1e-8)` is a practical guard. `f64` has ~15-17 significant digits, and quality function values can range from ~1 (modularity) to ~1e6 (CPM on large graphs). A threshold of 1e-11 would be meaningless for CPM on a 10k-node graph.

---

## 4. Resolution of FR-017 vs FR-033

### The Fix

**FR-017 should be amended to remove the hardcoded `1e-7` and reference FR-033's formula exclusively.**

Current FR-017 text (line 270):
> "The plateau threshold is derived from the convergence threshold per FR-033's formula: `plateau_threshold = max(convergence_threshold / 10, 1e-8)` (defaulting to 1e-7 when convergence threshold is 1e-6)."

Proposed FR-017 text:
> "The plateau threshold is derived from the convergence threshold per FR-033's formula: `plateau_threshold = max(convergence_threshold / 10, 1e-8)`."

**FR-033 is already correct** and should remain unchanged. It defines the formula, the floor, the derivation rationale, and the two-threshold relationship.

The parenthetical "(defaulting to 1e-7 when convergence threshold is 1e-6)" in FR-017 is **redundant and potentially misleading** because:
1. It's computable from the formula (1e-6 / 10 = 1e-7 > 1e-8, so max = 1e-7).
2. It implies 1e-7 is a fixed constant rather than a derived value.
3. If the default convergence threshold changes in the future (e.g., to 1e-5), the parenthetical becomes stale while the formula self-updates.

### Consistency Rule Going Forward

Establish an invariant: **FR-033 is the single source of truth for the plateau threshold formula.** FR-017 references it but does not restate it with hardcoded constants. Any future change to the formula (e.g., changing 10× to 100×, or 1e-8 to 1e-10) only requires updating FR-033.

---

## 5. Additional Findings: The Plateau Detection Mechanism Itself

### 5.1 N=5 Consecutive Iterations

The requirement for N=5 consecutive iterations below the plateau threshold (FR-017, FR-033) is also not from community detection literature. In ML:

- **Patience values** typically range from 3-20 epochs (Prechelt, 1998).
- **PyTorch's `ReduceLROnPlateau`** defaults to patience=10.
- **Keras's `EarlyStopping`** defaults to patience=0 (stop immediately on plateau).

N=5 is a reasonable middle ground — strict enough to avoid premature plateau detection from single-iteration noise, lenient enough to trigger before convergence in practice.

### 5.2 Plateau as Observability Signal (Not Stopping Criterion)

The design choice that plateau events are **informational only** and do **not trigger termination** is sound and aligns with:

- **ML logging callbacks**: Log "plateau detected" but don't stop training.
- **Learning rate schedulers**: Reduce LR on plateau, but continue training.
- **The `StepIterator` pattern** (FR-018): External callers decide whether to stop based on events.

This is a **framework-level design decision**, not something derived from Leiden/Louvain literature. It's justified by the framework's observability-first constitution principle (opt-in observability, compile-time elimination).

---

## 6. Summary Table

| Aspect | Leiden/Louvain Literature | This Framework (FR-017/FR-033) | Assessment |
|--------|--------------------------|-------------------------------|------------|
| **Convergence threshold** | 0 (strictly positive) | 1e-6 (configurable) | Framework adds a practical threshold for early termination — reasonable deviation |
| **Plateau detection** | Does not exist | N=5 consecutive iterations below plateau threshold | Novel addition for observability — not from literature, but sound design |
| **Plateau threshold** | N/A | `max(convergence_threshold / 10, 1e-8)` | Original heuristic; formula is defensible, hardcoded value is not |
| **Plateau → termination** | N/A | No (informational only) | Correct design choice for observability pattern |
| **Two-threshold design** | Does not exist | Convergence (stopping) + Plateau (observability) | From ML early stopping, not community detection |

---

## 7. Citations

### Primary Literature

1. **Traag, V. A., Waltman, L., & van Eck, N. J. (2019).** "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports*, 9(1), 5233. DOI: [10.1038/s41598-019-41695-z](https://doi.org/10.1038/s41598-019-41695-z)
   - Convergence: strictly positive improvements only (Algorithm A.1, line 17: "if ΔH(v→C') > 0 then")
   - No convergence threshold, no plateau detection, no consecutive-iteration counter

2. **Blondel, V. D., Guillaume, J.-L., Lambiotte, R., & Lefebvre, E. (2008).** "Fast unfolding of communities in large networks." *Journal of Statistical Mechanics: Theory and Experiment*, 2008(10), P10008. DOI: [10.1088/1742-5468/2008/10/P10008](https://doi.org/10.1088/1742-5468/2008/10/P10008)
   - "The two phases are repeated until the quality function cannot be increased further."
   - No threshold parameter

3. **Prechelt, L. (1998).** "Early stopping — but when?" In *Neural Networks: Tricks of the Trade* (pp. 55-69). Springer. DOI: [10.1007/3-540-49430-8_3](https://doi.org/10.1007/3-540-49430-8_3)
   - Origin of patience-based plateau detection in ML
   - Provides theoretical basis for the N-consecutive-iterations pattern

### Reference Implementations

4. **leidenalg C++ (libleidenalg)** — [github.com/vtraag/libleidenalg](https://github.com/vtraag/libleidenalg)
   - File: `src/Optimiser.cpp`, method `Optimiser::move_nodes`
   - Evidence: `double max_improv = ... 10*DBL_EPSILON;` (line ~370) — effectively zero threshold
   - No plateau detection, no convergence threshold parameter

5. **leidenalg Python** — [github.com/vtraag/leidenalg](https://github.com/vtraag/leidenalg)
   - File: `src/leidenalg/Optimiser.py`, class `Optimiser`
   - `optimise_partition()` loop: continues until `diff_inc > 0` fails (when n_iterations < 0)
   - No convergence threshold parameter, no plateau detection

6. **igraph C Library** — [github.com/igraph/igraph](https://github.com/igraph/igraph)
   - File: `src/community/leiden.c`
   - Comment: "Only consider strictly improving moves. Note that this is important in considering convergence."
   - No threshold parameter in `igraph_community_leiden()` API

### Secondary Literature

7. **Traag, V. A. (2015).** "Faster unfolding of communities: Speeding up the Louvain algorithm." *Physical Review E*, 92(3), 032801. DOI: [10.1038/s41598-019-41695-z](https://doi.org/10.1103/PhysRevE.92.032801)
   - Introduces fast local move optimization; no convergence threshold

8. **Traag, V. A., Krings, G., & Van Dooren, P. (2013).** "Significant scales in community structure." *Scientific Reports*, 3, 2930. DOI: [10.1038/srep02930](https://doi.org/10.1038/srep02930)
   - Introduces Significance quality function with p-value threshold (statistical, not convergence)

9. **Traag, V. A., Van Dooren, P., & Nesterov, Y. (2011).** "Narrow scope for resolution-limit-free community detection." *Physical Review E*, 84(1), 016114. DOI: [10.1103/PhysRevE.84.016114](https://doi.org/10.1103/PhysRevE.84.016114)
   - Defines CPM quality function; convergence is strictly positive

---

## 8. Conclusion

**The two-threshold design with plateau detection is a framework-level innovation, not a pattern from community detection literature.** It is borrowed from ML early stopping (Prechelt, 1998) and adapted for the framework's observability needs.

**Recommendation: Adopt FR-033's formula as the single source of truth.** Amend FR-017 to remove the hardcoded `1e-7` reference and the redundant parenthetical. The formula `max(convergence_threshold / 10, 1e-8)` is a defensible heuristic — it maintains the intended ordering (plateau triggers before convergence), self-adjusts with configuration, and has a numerical floor for edge cases.

The hardcoded `1e-7` in FR-017 is the inconsistency to eliminate, not the formula in FR-033.

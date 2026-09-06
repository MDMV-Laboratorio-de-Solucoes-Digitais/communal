# Leiden Algorithm Refinement Phase Beta Parameter

## Research Question

What should the default value be for Leiden's refinement phase beta parameter, and should it be configurable by the caller?

---

## Executive Summary

**Default value: `beta = 0.01`**

This is the value used in all experiments in the original Traag et al. (2019) paper. The reference implementations (C++ `libleidenalg` and Python `leidenalg`) do **not** expose beta as a configurable parameter — instead, the refinement randomness is controlled through a different mechanism (`refine_consider_comms`). However, the paper explicitly states that values in the range **[0.0005, 0.1]** all produce reasonable results.

**Recommendation:** Use `beta = 0.01` as the default. Make it configurable to allow sensitivity analysis, but restrict to the [0.0005, 0.1] range documented by the authors.

---

## 1. Original Traag et al. (2019) Paper

**Source:** Traag, V.A., Waltman, L., & Van Eck, N.J. (2019). "From Louvain to Leiden: guaranteeing well-connected communities." *Scientific Reports* 9, 5233. arXiv:1810.08473.

### Beta in the Algorithm

In the paper's pseudo-code (Algorithm A.2, `MergeNodesSubset` function, line 38), beta (β) controls the probability of merging a node with a community during the randomized refinement phase:

```
Pr(C' = C) ∝ exp(β · Δ_H_P(v → C))   if Δ_H_P(v ≥ C) ≥ 0
                                     0   otherwise
```

Where `Δ_H_P(v → C)` is the quality difference of moving node `v` to community `C`. Higher β means the algorithm is more likely to select communities that yield larger quality improvements (more "greedy"), while lower β means more uniform randomness.

### Default Value Used

From Section IV (Experimental Analysis), page 7:

> "In all experiments reported here, we used a value of **0.01** for the parameter β that determines the degree of randomness in the refinement phase of the Leiden algorithm. However, values of β within a range of roughly **[0.0005, 0.1]** all provide reasonable results, thus allowing for some, but not too much randomness."

### Key Insight

The paper proves (Appendix C.1) that even excluding moves that decrease the quality function, the optimal partition can still be reached using non-decreasing move sequences. The randomness introduced by β allows broader exploration of partition space, avoiding the pitfalls of purely greedy approaches.

---

## 2. C++ Reference Implementation (libleidenalg)

**Repository:** https://github.com/vtraag/libleidenalg

### Beta Handling

The C++ implementation does **not** use an explicit `beta` parameter in the `Optimiser` class. Instead, the refinement phase randomness is controlled by the `refine_consider_comms` setting, which defaults to `RAND_NEIGH_COMM`.

From `src/Optimiser.cpp` (constructor):

```cpp
Optimiser::Optimiser()
{
  this->consider_comms = Optimiser::ALL_NEIGH_COMMS;
  this->optimise_routine = Optimiser::MOVE_NODES;
  this->refine_consider_comms = Optimiser::RAND_NEIGH_COMM;  // <-- refinement randomness
  this->refine_routine = Optimiser::MERGE_NODES;
  this->refine_partition = true;
  // ...
}
```

### How Refinement Works

In `merge_nodes_constrained()` (the refinement routine), when `consider_comms == RAND_NEIGH_COMM`:

1. For each singleton node, collect all neighboring communities (with duplicates, proportional to edge weight)
2. Select one community uniformly at random from this list
3. Accept the move only if it does not decrease quality (`possible_improv >= 0`)

This is effectively equivalent to a **beta → 0** limit (uniform random selection among valid targets), not the `exp(β · Δ)` weighting described in the paper. The C++ implementation simplifies the paper's formulation.

### Configurability

- **Beta is NOT exposed** as a parameter in the C++ API
- The `refine_consider_comms` can be set to one of: `ALL_COMMS`, `ALL_NEIGH_COMMS`, `RAND_COMM`, `RAND_NEIGH_COMM`
- The `refine_routine` can be `MOVE_NODES` or `MERGE_NODES`
- These are the only "knobs" controlling refinement behavior

**Source files:**
- `include/Optimiser.h` — class definition, no beta member
- `src/Optimiser.cpp` — implementation, `merge_nodes_constrained()` function

---

## 3. Python leidenalg Package

**Repository:** https://github.com/vtraag/leidenalg

### Beta Handling

The Python package is a wrapper around the C++ implementation. It inherits the same behavior — **beta is not directly exposed**.

From `src/leidenalg/Optimiser.py`:

```python
@property
def refine_consider_comms(self):
    """Determine how alternative communities are considered for moving
    a node when *refining* a partition.
    The default is :attr:`leidenalg.ALL_NEIGH_COMMS`.
    """
    return _c_leiden._Optimiser_get_refine_consider_comms(self._optimiser)
```

The Python `Optimiser` class exposes:
- `refine_consider_comms` — controls which communities are considered during refinement
- `refine_routine` — `MOVE_NODES` or `MERGE_NODES`
- `refine_partition` — boolean, whether to refine at all

But **no `beta` property** exists. The randomness mechanism is the same as the C++ version (random neighbor community selection).

### Configurability

- Beta is **not configurable** in the Python package
- Users can only toggle between different `refine_consider_comms` modes
- The underlying C++ code would need modification to expose beta

**Source files:**
- `src/leidenalg/Optimiser.py` — Python Optimiser class
- `src/leidenalg/python_optimiser_interface.cpp` — C++/Python bridge

---

## 4. Comparison of Reference Implementations

| Aspect | Paper (Traag 2019) | C++ libleidenalg | Python leidenalg |
|--------|-------------------|-------------------|-------------------|
| **Beta default** | 0.01 | N/A (implicit ~0) | N/A (implicit ~0) |
| **Beta range** | [0.0005, 0.1] | — | — |
| **Selection method** | `exp(β·Δ)` weighted | Uniform random neighbor | Uniform random neighbor |
| **Configurable?** | Yes (in theory) | No | No |
| **Refinement on?** | Yes (default) | Yes (default) | Yes (default) |

### Important Discrepancy

The reference implementations use a **simplified refinement** compared to the paper:
- **Paper:** Weighted random selection via `exp(β · Δ_H)`
- **Implementation:** Uniform random selection among neighboring communities, accepting if quality doesn't decrease

This means the implementations are effectively using a "beta = 0" approach (maximum randomness among valid moves), which is at the edge of the paper's recommended range.

---

## 5. Best Practices and Sensitivity

### From the Paper

- **β = 0.01** is the canonical default, used for all experimental results
- **Range [0.0005, 0.1]** produces "reasonable results"
- Too much randomness (high β) → less exploration, more greedy
- Too little randomness (low β) → more exploration, slower convergence

### Practical Considerations

1. **For reproducibility:** Use β = 0.01 (matches published results)
2. **For exploration:** Lower β values help escape local optima
3. **For speed:** Higher β values converge faster (more greedy)
4. **For large networks:** The paper shows Leiden is up to 20× faster than Louvain regardless of β

### No Known Sensitivity Analyses

The paper does not include a dedicated sensitivity analysis for β. The [0.0005, 0.1] range is stated without detailed justification. No follow-up studies specifically analyzing β's effect on partition quality were found in the primary sources.

---

## 6. Recommendation

### Default Value

**Use `beta = 0.01`** — this matches the original paper's experimental setting and is the de facto standard.

### Configurability

**Yes, make it configurable**, for these reasons:

1. **The paper explicitly frames β as a tunable parameter** with a documented range
2. **Different use cases benefit from different settings:**
   - Exploratory analysis → lower β (more randomness)
   - Production/reproducible pipelines → β = 0.01 (matches literature)
   - Speed-critical applications → higher β (more greedy)
3. **The reference implementations' simplification** (uniform random vs. exponential weighting) means a faithful implementation of the paper should expose β

### Implementation Guidance

- **Default:** `beta = 0.01`
- **Valid range:** `[0.0005, 0.1]` (warn or clamp outside this range)
- **Effect:** Higher β → more greedy (less random); Lower β → more uniform random
- **Selection formula:** `Pr(C) ∝ exp(β · Δ_H(v → C))` for `Δ_H ≥ 0`

---

## References

1. Traag, V.A., Waltman, L., & Van Eck, N.J. (2019). From Louvain to Leiden: guaranteeing well-connected communities. *Scientific Reports* 9, 5233. https://arxiv.org/abs/1810.08473
2. C++ libleidenalg: https://github.com/vtraag/libleidenalg — `src/Optimiser.cpp`, `include/Optimiser.h`
3. Python leidenalg: https://github.com/vtraag/leidenalg — `src/leidenalg/Optimiser.py`
4. leidenalg documentation: https://leidenalg.readthedocs.io/

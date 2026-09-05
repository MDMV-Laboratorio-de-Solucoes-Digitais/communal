# Gamma (Resolution Parameter) Design Analysis

**Date**: 2026-09-06
**Purpose**: Research how the resolution parameter (gamma) is structured in community detection library APIs (Leiden/Louvain) and resolve a spec inconsistency.

## The Inconsistency

The spec contains a contradiction:

- **T015**: "The resolution parameter (gamma, FR-008) is NOT part of the base `AlgorithmConfig` trait because it only applies to quality functions that support resolution (Modularity Q, CPM) and is irrelevant to Infomap, LPA, and Fluid Communities. Gamma is defined in algorithm-specific configs (LeidenConfig, LouvainConfig) that use resolution-based quality functions."
- **T036b**: "The gamma field MUST be read from AlgorithmConfig"

This analysis determines which is correct.

---

## 1. Evidence from python-leidenalg / libleidenalg Source Code

### Class Hierarchy (C++ headers)

The libleidenalg library (https://github.com/vtraag/libleidenalg) uses this inheritance structure:

```
MutableVertexPartition (base class)
├── ResolutionParameterVertexPartition  ← resolution_parameter lives HERE
│   ├── LinearResolutionParameterVertexPartition
│   │   ├── CPMVertexPartition
│   │   ├── RBConfigurationVertexPartition
│   │   └── RBERVertexPartition
├── ModularityVertexPartition           ← NO resolution_parameter
├── SignificanceVertexPartition         ← NO resolution_parameter
└── SurpriseVertexPartition             ← NO resolution_parameter
```

**Key finding**: The `resolution_parameter` field is declared in `ResolutionParameterVertexPartition`, NOT in the base class `MutableVertexPartition`. This is an intermediate abstract class that only quality functions with resolution support inherit from.

From `ResolutionParameterVertexPartition.h`:
```cpp
class ResolutionParameterVertexPartition : public MutableVertexPartition {
  public:
    double resolution_parameter;
    // ...
};
```

From `MutableVertexPartition.h` (base class):
```cpp
class MutableVertexPartition {
  // NO resolution_parameter field
  // Only: _membership, graph, _csize, _cnodes, etc.
};
```

### Optimiser Class

The `Optimiser` class in libleidenalg has its OWN configuration fields:
```cpp
class Optimiser {
  public:
    int consider_comms;
    int refine_partition;
    int refine_consider_comms;
    int optimise_routine;
    int refine_routine;
    int consider_empty_community;
    size_t min_comm_size;
    size_t max_comm_size;
    double community_constraint_enforcement;
    // NO resolution_parameter — it's a property of the partition, not the optimiser
};
```

The resolution parameter lives in the **partition object** (which encapsulates the quality function), not in the optimiser's operational configuration.

### Python Interface Confirmation

The Python `VertexPartition.py` mirrors this exactly:

- `MutableVertexPartition` (base) — no `resolution_parameter`
- `LinearResolutionParameterVertexPartition` (intermediate) — exposes `resolution_parameter` as a Python property
- `CPMVertexPartition`, `RBConfigurationVertexPartition`, `RBERVertexPartition` — inherit it

```python
class LinearResolutionParameterVertexPartition(MutableVertexPartition):
    @property
    def resolution_parameter(self):
        return _c_leiden._ResolutionParameterVertexPartition_get_resolution(self._partition)

    @resolution_parameter.setter
    def resolution_parameter(self, value):
        _c_leiden._ResolutionParameterVertexPartition_set_resolution(self._partition, value)
```

**Conclusion from python-leidenalg**: The resolution parameter is NOT part of any base trait/class. It is defined per-quality-function, specifically on the intermediate `ResolutionParameterVertexPartition` class that only resolution-supporting quality functions inherit from. Algorithms that don't use resolution-based quality functions (Significance, Surprise) inherit directly from `MutableVertexPartition` and have NO resolution parameter.

---

## 2. Evidence from igraph C Library

The igraph C implementation (https://github.com/igraph/igraph/blob/main/src/community/leiden.c) takes a different structural approach but reinforces the same separation:

### Function Signature

```c
igraph_error_t igraph_community_leiden(
    const igraph_t *graph,
    const igraph_vector_t *edge_weights,
    const igraph_vector_t *vertex_out_weights,
    const igraph_vector_t *vertex_in_weights,
    igraph_real_t resolution,      // ← gamma passed as a function argument
    igraph_real_t beta,
    igraph_bool_t start,
    igraph_int_t n_iterations,
    igraph_vector_int_t *membership,
    igraph_int_t *nb_clusters,
    igraph_real_t *quality);
```

### Key Observations

1. **No config struct at all**: igraph passes `resolution` (gamma) as a bare function parameter, not as part of any configuration object. This means there is no "base AlgorithmConfig" concept — each parameter is passed directly at the call site.

2. **The "simple" wrapper** (`igraph_community_leiden_simple`) takes `resolution` alongside `objective` (an enum selecting CPM, Modularity, or ER), and computes vertex weights based on the chosen objective. The resolution parameter is intrinsically tied to the choice of objective function.

3. **Separation of concerns**: The algorithm implementation (`community_leiden` internal function) receives `resolution` as a parameter but doesn't store it in any persistent config — it's a value that parameterizes the quality function computation, not the algorithm's operational state.

**Conclusion from igraph**: The resolution parameter is treated as a quality-function parameter, not an algorithm-configuration parameter. It's passed at call time alongside the objective function selection, not stored in an operational config struct.

---

## 3. Evidence from Traag et al. 2019 Paper

From "From Louvain to Leiden: guaranteeing well-connected communities" (https://doi.org/10.1038/s41598-019-41695-z):

### Quality Functions with Resolution

The paper defines two quality functions that use gamma:

**Modularity (Eq. 1):**
```
H = (1/2m) Σ_c [e_c - γ(K_c²/2m)]
```
"where γ > 0 is a resolution parameter. Higher resolutions lead to more communities, while lower resolutions lead to fewer communities."

**CPM (Eq. 2):**
```
H = Σ_c [e_c - γ(n_c choose 2)]
```
"The interpretation of the resolution parameter γ is quite straightforward. The parameter functions as a sort of threshold: communities should have a density of at least γ..."

### Guarantees Parameterized by γ

The paper's theoretical guarantees are all parameterized by gamma:
- **γ-separation**: No communities can be merged
- **γ-connectivity**: Communities are internally connected
- **Subpartition γ-density**: Communities are well-connected internally
- **Uniform γ-density**: No subsets can be separated

### What the Paper Does NOT Do

The paper does NOT treat gamma as an algorithmic/operational parameter. It is strictly a parameter of the quality function being optimized. The algorithmic parameters (beta for refinement randomness, convergence thresholds, iteration counts) are conceptually separate.

### Refinement Randomness (beta) is Distinct

The paper introduces beta (randomness in refinement, Section III) as a separate parameter from gamma. Beta controls the stochastic exploration during refinement; gamma controls the resolution of the quality function. These are orthogonal concerns.

**Conclusion from the paper**: Gamma is a quality-function parameter, not an algorithm-configuration parameter. It belongs with the quality function formulation, not with operational parameters like convergence thresholds and iteration limits.

---

## 4. Analysis: Base Trait vs. Algorithm-Specific Config

### Option A: Gamma in Base `AlgorithmConfig` Trait

**Pros**:
- Single place to read gamma for any algorithm
- T036b's wording ("gamma field MUST be read from AlgorithmConfig") is satisfied literally

**Cons**:
- Violates the spec's own rationale: gamma is irrelevant to Infomap, LPA, and Fluid Communities
- Forces algorithms that don't use resolution to either:
  - Carry a meaningless field (wastes memory, confusing API)
  - Panic/ignore it (violates principle of least surprise)
- Inconsistent with how both python-leidenalg and igraph structure their APIs
- Violates the Interface Segregation Principle: clients shouldn't be forced to depend on methods they don't use
- Makes the base trait larger and harder to implement correctly

### Option B: Gamma in Algorithm-Specific Config Structs (LeidenConfig, LouvainConfig)

**Pros**:
- Consistent with python-leidenalg's design (resolution_parameter on quality-function-specific classes)
- Consistent with igraph's design (resolution passed alongside objective function)
- Consistent with the Traag et al. paper (gamma is a quality-function parameter)
- Infomap, LPA, and Fluid configs are clean — no meaningless fields
- Matches the existing T015 task description (already written this way)
- Quality-function parameters and operational parameters are cleanly separated
- Easier to extend: adding a new resolution-based quality function doesn't change the base trait

**Cons**:
- T036b needs to be reworded (but T036b is the task that created the inconsistency — it was written after T015 and contradicts it)
- Slightly more verbose to read gamma (must go through the specific config, not the base trait)
- Requires a way to detect at runtime whether a given algorithm config supports gamma (but this is already handled by Rust's type system — only LeidenConfig/LouvainConfig would have the field)

### Option C: Gamma in QualityFunction Enum/Struct

A third option: attach gamma to the quality function selection rather than the algorithm config.

**Pros**:
- Most mathematically precise: gamma parameterizes the quality function
- Clean separation: algorithm config = operational params, quality function = what to optimize

**Cons**:
- More complex API: must pass quality function and algorithm config separately
- Over-engineering for the current spec scope

---

## 5. Recommendation

**Option B is the correct design.** Gamma should be defined in algorithm-specific config structs (`LeidenConfig`, `LouvainConfig`), NOT in the base `AlgorithmConfig` trait.

### Rationale

1. **Direct evidence from reference implementations**: Both python-leidenalg and igraph treat resolution as a quality-function parameter, not a base algorithm parameter. In python-leidenalg, it lives on `ResolutionParameterVertexPartition`, not on `MutableVertexPartition`. In igraph, it's passed as a call-time parameter alongside the objective function.

2. **Consistency with existing spec patterns**: T015 was already written correctly with this design. The rationale in T015 is sound: "it only applies to quality functions that support resolution (Modularity Q, CPM) and is irrelevant to Infomap, LPA, and Fluid Communities."

3. **Mathematical correctness**: Per Traag et al. 2019, gamma parameterizes the quality function, not the algorithm. The algorithm (Leiden/Louvain) is a procedure for optimizing ANY quality function; gamma is specific to certain quality functions.

4. **API cleanliness**: Algorithms that don't use resolution-based quality functions shouldn't carry meaningless configuration fields. This is a core principle of good API design.

### How to Resolve the Inconsistency

T036b's wording "The gamma field MUST be read from AlgorithmConfig" should be reworded to: "The gamma field MUST be read from the algorithm-specific config (LeidenConfig)." The base `AlgorithmConfig` trait should NOT include gamma.

The T015 note is already correct and consistent with the reference implementations:
> "The resolution parameter (gamma, FR-008) is NOT part of the base `AlgorithmConfig` trait because it only applies to quality functions that support resolution (Modularity Q, CPM) and is irrelevant to Infomap, LPA, and Fluid Communities. Gamma is defined in algorithm-specific configs (LeidenConfig, LouvainConfig) that use resolution-based quality functions."

### Suggested Rewording for T036b

Current (inconsistent):
> "Thread resolution parameter (gamma) from the algorithm-specific config (LeidenConfig) through to Modularity Q and CPM computations... Gamma is NOT part of the base `AlgorithmConfig` trait (see T015 note) — it is defined in algorithm-specific configs that use resolution-based quality functions. When gamma is not explicitly set in config, default to 1.0 for standard modularity per the contracts in `contracts/metrics.md`."

This wording is actually ALREADY CORRECT — it says gamma is NOT in the base trait and IS in algorithm-specific configs. The inconsistency is only with the T036b title/summary which says "The gamma field MUST be read from AlgorithmConfig" — but the body text correctly identifies it as coming from the algorithm-specific config. The title should be reworded to:

> "T036b [US1] Thread resolution parameter (gamma) from the algorithm-specific config (LeidenConfig) through to Modularity Q and CPM computations"

The body text is already correct.

---

## 6. Summary Table

| Aspect | Base Trait | Algorithm-Specific Config | Quality Function |
|--------|-----------|--------------------------|------------------|
| python-leidenalg | `MutableVertexPartition` — NO resolution | N/A (Optimiser has no resolution) | `ResolutionParameterVertexPartition` — HAS resolution |
| igraph C | N/A (no config struct) | N/A | Passed alongside objective function parameter |
| Traag et al. | Algorithm params (beta, iterations) separate | N/A | Gamma parameterizes the quality function |
| **Recommended** | No gamma | **YES — gamma here** | Could also work |

---

## References

1. **python-leidenalg GitHub** — https://github.com/vtraag/leidenalg — Source code showing `ResolutionParameterVertexPartition` with `resolution_parameter` field, not in base `MutableVertexPartition`
2. **libleidenalg GitHub** — https://github.com/vtraag/libleidenalg — C++ headers showing class hierarchy
3. **igraph leiden.c** — https://github.com/igraph/igraph/blob/main/src/community/leiden.c — Resolution passed as function parameter, not stored in config struct
4. **Traag et al. 2019** — "From Louvain to Leiden: guaranteeing well-connected communities" — https://doi.org/10.1038/s41598-019-41695-z — Gamma defined as quality-function parameter in Eqs. (1) and (2)

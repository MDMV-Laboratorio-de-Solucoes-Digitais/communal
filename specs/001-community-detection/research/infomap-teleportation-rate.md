# Infomap Teleportation Rate Parameter — Primary Source Research

## Summary of Findings

| Property | Value | Confirmed By |
|----------|-------|--------------|
| **Type** | `f64` / `double` (64-bit float) | C++ (`double`), Python (`float`), Rust (`f64`) |
| **Default** | `0.15` | Paper, C++, Python, Rust |
| **Valid Range** | `[0.0, 1.0]` (inclusive) | C++ ParameterCatalog, Python Options |
| **CLI Flag** | `-p` / `--teleportation-probability` | C++ ParameterCatalog |
| **Argument Type** | `probability` | C++ ArgType |

---

## 1. Original Paper: Rosvall & Bergstrom (2008)

**Source:** Rosvall, M., & Bergstrom, C. T. (2008). "Maps of random walks on complex networks reveal community structure." *Proceedings of the National Academy of Sciences*, 105(4), 1118–1123.
**DOI:** [10.1073/pnas.0706851105](https://doi.org/10.1073/pnas.0706851105)
**PDF:** [mapequation.org/assets/publications/RosvallBergstromPNAS2008Full.pdf](https://mapequation.org/assets/publications/RosvallBergstromPNAS2008Full.pdf)

### Key Quotes

From the main text (page 4):

> "We introduce a small 'teleportation probability' ε in the random walk: with probability ε, the process jumps to a random node anywhere in the network, which converts our random walker into the sort of 'random surfer' that drives Google's PageRank algorithm."

> "We choose ε = 0.15 corresponding to the well known damping factor d = 0.85 in the PageRank algorithm."

From the SI Appendix (page 8):

> "As in Google's PageRank algorithm, we use ε = 0.15, but emphasize that the results are robust to this choice."

### Robustness Claim

> "Our clustering results are highly robust to the particular choice of the small fraction ε. For example, so long as ε ≤ 0.45 the optimal partitioning of the network in Fig. 1 remains exactly the same. In general, the more significant the regularities, the higher ε can be before frequent teleportation swamps the network structure."

### Paper Conclusions

- **Default:** ε = 0.15 (matching PageRank's d = 0.85)
- **Type:** Real-valued probability (implicitly a floating-point number in [0, 1])
- **Range:** The paper does not hard-define bounds but demonstrates robustness up to ε = 0.45 for the example network. The parameter is a probability, so logically ∈ [0, 1].

---

## 2. Official C++ Implementation (mapequation/infomap)

**Source:** [github.com/mapequation/infomap](https://github.com/mapequation/infomap)
**File:** `src/io/Config.h` (commit `048af68`)

### Type & Default

```cpp
// src/io/Config.h, line ~107
double teleportationProbability = 0.15;
```

- **Type:** `double` (IEEE 754 64-bit floating point)
- **Default:** `0.15`

### Parameter Catalog Definition

**File:** `src/io/ParameterCatalog.cpp`

```cpp
param()
    .shortName('p')
    .longName("teleportation-probability")
    .description("Set the probability of teleporting to a random node or link when calculating flow.")
    .argument(ArgType::probability)
    .group("Algorithm")
    .advanced()
    .defaultValue("0.15")
    .range("0", "1")
    .configTarget(&Config::teleportationProbability),
```

- **CLI flag:** `-p` / `--teleportation-probability`
- **Range:** `[0, 1]` — enforced by `range("0", "1")` which creates a `LowerUpperBoundArgumentOption<double>` with min=0, max=1
- **Argument type:** `ArgType::probability` (defined in `src/io/ProgramInterface.h`)

### ArgType Definition

**File:** `src/io/ProgramInterface.h`

```cpp
struct ArgType {
  static const std::string integer;
  static const std::string number;
  static const std::string string;
  static const std::string path;
  static const std::string probability;
  static const std::string option;
  static const std::string list;
  // ...
};
```

The `probability` argument type is parsed as a `double` with lower/upper bounds of 0 and 1.

---

## 3. Python Package (`infomap` on PyPI)

**Source:** [github.com/mapequation/infomap](https://github.com/mapequation/infomap) — `interfaces/python/src/infomap/_options.py`
**PyPI:** [pypi.org/project/infomap](https://pypi.org/project/infomap)

### Options Definition

```python
# interfaces/python/src/infomap/_options.py
"teleportation_probability": _OptionSpec("--teleportation-probability", "value", 0.15, domain=(0.0, 1.0)),
```

### Dataclass Field

```python
@dataclass(frozen=True, slots=True, repr=False)
class Options(metaclass=_OptionsMeta):
    # ...
    teleportation_probability: float = 0.15
```

### Validation

The `_validate_option_domains` function enforces:

```python
# Bounds are inclusive, mirroring the C++ parser (reject value < min or > max).
if (low is not None and value < low) or (high is not None and value > high):
    raise ValueError(...)
```

### Docstring

> "Set the probability of teleporting to a random node or link when calculating flow. Valid range: between 0.0 and 1.0 (inclusive)."

### Python Conclusions

- **Type:** `float` (Python `float` is C `double` / IEEE 754 64-bit)
- **Default:** `0.15`
- **Range:** `[0.0, 1.0]` inclusive

---

## 4. Rust Implementation (`infomap-rs` crate)

**Source:** [docs.rs/infomap-rs](https://docs.rs/infomap-rs/latest/infomap_rs/struct.InfomapConfig.html)

### Struct Definition

```rust
pub struct InfomapConfig {
    /// Teleportation rate (default: 0.15)
    pub tau: f64,
    pub seed: u64,
    pub num_trials: usize,
    pub hierarchical: bool,
    pub min_improvement: f64,
    pub max_iterations: usize,
    pub max_depth: usize,
}
```

### Rust Conclusions

- **Type:** `f64` (64-bit floating point)
- **Default:** `0.15`
- **Range:** Not explicitly constrained in the struct definition (no built-in bounds check visible in the public API). The field is named `tau` (τ) rather than `teleportation_probability`.

---

## 5. npm Package (`@mapequation/infomap`)

**Source:** [npmjs.com/package/@mapequation/infomap](https://www.npmjs.com/package/@mapequation/infomap)

The npm package is a JavaScript wrapper around the same C++ core (compiled to WebAssembly). It inherits the same parameter semantics:

- **Type:** `number` (JavaScript `Number` is IEEE 754 64-bit float)
- **Default:** `0.15`
- **Range:** `[0, 1]` (inherited from C++ core)

---

## 6. Edge Cases & Constraints

### 6.1 Ergodicity Requirement

The teleportation rate exists to guarantee ergodicity in directed networks. Without it (τ = 0), a random walker can get trapped in sink nodes or strongly connected components with no exit. The paper states:

> "To guarantee a unique steady state distribution for directed networks, we introduce a small teleportation probability ε in the random walk that links every node to every other node with positive probability and thereby convert the random walker into a random surfer."

**Implication:** τ = 0 is technically valid per the range but may cause non-convergence on directed networks with dangling nodes or disconnected components.

### 6.2 Upper Bound Robustness

The paper demonstrates that results are robust up to τ ≈ 0.45 for the example network. Beyond that, "frequent teleportation swamps the network structure." The canonical default of 0.15 is intentionally conservative.

### 6.3 Interaction with Other Parameters

- **`recorded_teleportation`** (`-e` flag): When enabled, teleportation steps are encoded in the codelength calculation. This changes how the teleportation rate affects the optimization.
- **`teleport_to_nodes`** (`--to-nodes`): When enabled, teleportation targets nodes uniformly rather than following link structure.
- **`regularized`**: Activates `--recorded-teleportation` and adds a Bayesian prior network.

### 6.4 Relationship to PageRank

The teleportation rate τ corresponds to the PageRank damping factor d as: **τ = 1 - d**. The canonical value τ = 0.15 corresponds to d = 0.85, which is Google's original PageRank damping factor.

### 6.5 Numerical Precision

The C++ implementation uses `double` (f64) throughout. The flow convergence tolerance is `1e-15` (see `flowTolerance` in Config.h), which is near the limit of f64 precision (~15-17 significant decimal digits). The teleportation probability should be specified with at most ~15 significant digits.

---

## 7. Recommendation

For a Rust implementation of Infomap:

| Property | Recommended Value |
|----------|-------------------|
| **Type** | `f64` |
| **Default** | `0.15` |
| **Valid Range** | `[0.0, 1.0]` (inclusive) |
| **Field Name** | `teleportation_probability` (or `tau` for brevity) |
| **Validation** | Reject values outside `[0.0, 1.0]` with a clear error message |

### Justification

- `f64` matches all canonical implementations (C++ uses `double`, Python uses `float` which is C `double`, Rust crate uses `f64`).
- `0.15` is the universal default across all implementations and the original paper.
- `[0.0, 1.0]` inclusive is the enforced range in both the C++ and Python implementations.
- While τ = 0 is technically valid, consider warning on directed networks since it may break ergodicity.

---

## Source Citations

1. Rosvall, M., & Bergstrom, C. T. (2008). Maps of random walks on complex networks reveal community structure. *PNAS*, 105(4), 1118–1123. https://doi.org/10.1073/pnas.0706851105
2. mapequation/infomap — `src/io/Config.h`, line ~107: `double teleportationProbability = 0.15;` — https://github.com/mapequation/infomap/blob/master/src/io/Config.h
3. mapequation/infomap — `src/io/ParameterCatalog.cpp`, `--teleportation-probability` spec with `.defaultValue("0.15").range("0", "1")` — https://github.com/mapequation/infomap/blob/master/src/io/ParameterCatalog.cpp
4. mapequation/infomap — `src/io/ProgramInterface.h`, `ArgType::probability` — https://github.com/mapequation/infomap/blob/master/src/io/ProgramInterface.h
5. mapequation/infomap — `interfaces/python/src/infomap/_options.py`, `teleportation_probability: float = 0.15` with `domain=(0.0, 1.0)` — https://github.com/mapequation/infomap/blob/master/interfaces/python/src/infomap/_options.py
6. `infomap-rs` crate — `InfomapConfig { tau: f64, ... }` with default `0.15` — https://docs.rs/infomap-rs/latest/infomap_rs/struct.InfomapConfig.html
7. Running Infomap — Infomap 2.15.1 documentation: "teleportation_probability (τ) sets the random-surfer teleportation rate; the default 0.15 is rarely worth changing." — https://mapequation.org/infomap-python-docs/working-with-infomap/running-and-options.html

# Contract: LeidenConfig

**Feature**: 003-leiden-cache-optimization
**Location**: `crates/communal-algo/src/leiden/config.rs`

---

## Struct Definition

```rust
#[derive(Debug, Clone)]
pub struct LeidenConfig {
    pub gamma: f64,                  // Resolution parameter
    pub beta: f64,                   // Refinement randomness (θ)
    pub convergence_threshold: f64,  // |ΔQ| threshold for early termination
    pub convergence_mode: ConvergenceMode, // Absolute or Relative
    pub max_iterations: usize,        // Maximum Leiden passes (≥ 1)
    pub seed: Option<u64>,           // PRNG seed (Some(42) default)
    pub recompute_interval: u32,     // FP drift reset interval (≥ 1, default 100)
}
```

## Defaults

```rust
impl Default for LeidenConfig {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            beta: 0.01,
            convergence_threshold: 1e-6,
            convergence_mode: ConvergenceMode::Absolute,
            max_iterations: 10,
            seed: Some(42),
            recompute_interval: 100,
        }
    }
}
```

## Validation Contract

`LeidenConfig::validate() -> Result<(), AlgorithmError>` MUST enforce:

| Field | Rule | Error Message Pattern |
|-------|------|----------------------|
| `gamma` | `≥ 0`, finite, not NaN | `"gamma must be non-negative and finite, got {gamma}"` |
| `beta` | `≥ 0`, `≤ 1`, finite | `"beta must be in [0, 1], got {beta}"` |
| `convergence_threshold` | `≥ 0`, finite | `"convergence_threshold must be non-negative, got {value}"` |
| `max_iterations` | `≥ 1` | `"max_iterations must be ≥ 1, got {value}"` |
| `recompute_interval` | `≥ 1` | `"recompute_interval must be ≥ 1, got {value}"` |
| `seed` | any `u64` | (no validation needed) |

**Note on gamma**: Current codebase validates `gamma > 0`. Spec requires `gamma ≥ 0` (gamma=0 means Q = Σe_c only, all-in-one-community optimal). This is a breaking change to validation that MUST be updated.

**Note on beta**: Former codebase validated `beta ∈ [0.0005, 0.1]`. Spec requires `beta ∈ [0, 1]` (`beta = 0` = greedy deterministic selection). This is a broadening of the valid range that MUST be updated.

## Field Interaction Contract

| Combination | Behavior |
|-------------|----------|
| `convergence_threshold = 0` | Disables quality-based early termination; only zero-nodes-moved detection (FR-003) and `max_iterations` apply |
| `max_iterations = 1` | Exactly one Leiden pass; convergence checks irrelevant for termination |
| `gamma = 0` | Q = Σe_c (no null model); all-in-one-community becomes optimal |
| `beta → 0` | Greedy maximum-improvement selection during refinement (deterministic) |
| `beta = 1` | Maximum randomness: uniform random selection among eligible communities |
| `recompute_interval = 1` | Full recomputation after every incremental update (safe but slow) |
| `seed = None` | Should not occur with new Default (`Some(42)`); if explicitly set to None, runtime fallback to 42 (legacy behavior, to be removed) |

## Builder Methods

All builder methods consume `Self` and return `Self` (fluent API pattern):

```rust
impl LeidenConfig {
    pub fn with_gamma(mut self, gamma: f64) -> Self;
    pub fn with_beta(mut self, beta: f64) -> Self;
    pub fn with_convergence_threshold(mut self, threshold: f64) -> Self;
    pub fn with_max_iterations(mut self, max: usize) -> Self;
    pub fn with_seed(mut self, seed: u64) -> Self;
    pub fn with_seed_option(mut self, seed: Option<u64>) -> Self;
    pub fn with_recompute_interval(mut self, interval: u32) -> Self;  // NEW
}
```

## AlgorithmConfig Trait Implementation

```rust
impl AlgorithmConfig for LeidenConfig {
    fn convergence_threshold(&self) -> f64;
    fn convergence_mode(&self) -> ConvergenceMode;
    fn max_iterations(&self) -> usize;
    fn seed(&self) -> Option<u64>;
}
```

---

## Change Log (from current codebase)

| Change | Type | Breaking? |
|--------|------|-----------|
| Add `recompute_interval: u32` field | Addition | No (new field with default) |
| Change `seed` default from `None` to `Some(42)` | Modification | **Yes** — changes Default impl |
| Relax `gamma` validation from `> 0` to `≥ 0` | Modification | No (broadens acceptance) |
| Relax `beta` validation from `[0.0005, 0.1]` to `[0, 1]` | Modification | No (broadens acceptance) |
| Add `with_recompute_interval()` builder | Addition | No |

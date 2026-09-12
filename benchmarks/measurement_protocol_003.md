//! Measurement Protocol record — spec 003 (T060 / SC-001 / SC-002 / SC-003 / SC-009).
//!
//! Fields per Measurement Protocol (§157–160, spec.md): release build, median of ≥5 runs,
//! pinned reference machine (`reference-machine-spec.md`), JSON sidecars with `mu`, `avg_degree`,
//! `n_nodes`, `seed`, `generator_version`, `compiler_version`, `rustc_version`, `float_rounding_mode`.
//!
//! Status: setup artifacts committed (benchmark JSONs + edgelists). Timing measurements
//! (`median_ms`, `machine_specs`, `date`, `runs`) not yet recorded — requires release build
//! `cargo run` with LFR graphs per Phase 7 (post-optimization verification).
//!
//! See `benchmarks/lfr_10k_d50.json`, `benchmarks/lfr_50k_d50.json`, `benchmarks/lfr_50k_d10.json`.

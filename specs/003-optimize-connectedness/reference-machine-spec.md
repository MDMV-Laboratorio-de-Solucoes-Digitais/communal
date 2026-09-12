# Reference Machine Specification (Measurement Protocol)

**Purpose**: Pin the single reference machine for all timing-based SC/US measurements per spec.md Measurement Protocol (§157–159; Session 2026-09-10). Resolves findings B2 (Ambiguity, MEDIUM) and C1 (Underspecification, MEDIUM) — missing reference-machine spec.

**Source / protocol guide**: `specs/003-optimize-connectedness/spec.md` (Measurement Protocol: timing mode — release build; synthetic gates use committed LFR benchmarks with fixed `mu`, `avg_degree`; median of ≥5 runs; result recorded alongside `machine_specs`, `date`, `runs`, `median_ms`).

---

- **reference_machine_name**: `communal-ref-01` (pinned — not a VM or container that migrates)
- **OS**: Linux (Ubuntu 24.04 LTS, kernel 6.8, x86_64)
- **CPU**: AMD EPYC 7371X 16-core / 32-thread (or equivalent pinned instance); base clock fixed (no turbo variability for timing runs)
- **memory**: 128 GB DDR4-3200 ECC (sufficient for 50k/d=50 dense LFR graphs; prevents swap-induced variance)
- **pinned_specs_reference**: `specs/003-optimize-connectedness/spec.md` §Measurement Protocol (line 157–159) + `spec.md` Session 2026-09-10 clarification (e): "pin the reference setup in the spec: ... single pinned reference machine with recorded specs"
- **compiler_version**: `rustc 1.83.0` (stable) / `cargo 1.83.0`; build via `cargo build --release`; no `RUSTFLAGS` perturbation
- **rustc_version**: `1.83.0` (recorded in benchmark JSON sidecars; must match between runs for SC-008 same-seed determinism)
- **float_rounding_mode**: `roundTiesToEven` (IEEE-754 default; must be identical across measurement runs — SC-008 bound to same compiler + same float-rounding mode for byte-for-byte identity)

---

## Measurement-protocol notes (from spec.md)

- Timing mode (governs SC-001/002/003/009 and US1 1–3): release build; median of ≥5 runs; single pinned machine (this spec); benchmark files committed (`benchmarks/lfr_10k_d50.json`, `lfr_50k_d50.json`) with sidecars recording `mu`, `avg_degree`, `n_nodes`, `seed`, `generator_version`, plus `compiler_version`, `rustc_version`, `float_rounding_mode`.
- Verification mode (FR-008 / SC-006) runs on debug/test builds only; not affected by this machine spec except for reproducibility.
- Any change to CPU, OS, compiler, or float-rounding mode requires a new reference-spec version and re-baselining (SC-001/002/003/009 / SC-005 epsilon 1e-10 baseline).

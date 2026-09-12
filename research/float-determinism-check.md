# Float Determinism Check — Rust `f64` (CHK025 / SC-008)

> Brief findings for checklist item CHK025 (SC-008 float determinism across platforms). Focus: Rust `f64` arithmetic determinism across platforms / compiler versions, same seed / graph / build mode. Independent of spec files; no spec edits.

---

## 1. Short answer

For a **fixed target, fixed rustc/LLVM version, same build mode (release/debug), and no NaN-producing paths**, Rust `f64` primitive arithmetic (`+ - * / % abs sqrt copysign mul_add` and comparisons) is **byte-for-byte deterministic across runs** on identical hardware/software, because it follows IEEE 754-2008 with round-ties-to-even and no observable floating-point control/status bits.

It is **NOT guaranteed identical across**:
- different rustc / LLVM versions (optimization differences, NaN choices);
- different target architectures (x87 vs SSE vs NEON vs wasm); or
- any use of `-ffast-math` / `-ffinite-math-only` (relaxes IEEE, reorders/reassociates, changes NaN/Inf handling).

So for SC-008 (“same seed / same graph / same build” determinism): acceptable if build is pinned (same rustc, target, mode, no fast-math); **not acceptable** as a cross-platform guarantee without additional pinning or integer/ratio-based substitution.

---

## 2. Primary sources / citations

- **RFC 3514 — Float Semantics** (rust-lang/rfcs, accepted; tracking rust-lang/rust#128288): defines Rust float behavior. States primitive ops match IEEE 754-2008; NaN sign/payload are non-deterministic (target/compiler/version/flags may vary); `const` NaN bit patterns need not match runtime; 32-bit x86 w/o SSE2 uses x87 with 80-bit intermediate precision (non-compliant); x87 return-value NaN alteration; NEON flush-to-zero on ARM. [https://rust-lang.github.io/rfcs/3514-float-semantics.html]
- **rust-doc `f64`**: primitive type docs; notes `ln` / math functions are non-deterministic (vary by platform / Rust version / build). [https://doc.rust-lang.org/std/primitive.f64.html]
- **rust-lang/rust#150323** (and related): confirms “same executable, same hardware = deterministic”; “non-deterministic” only applies to NaN-producing ops / platform/libm functions / compiler updates.
- **IEEE 754-2008**: defines deterministic arithmetic (+, -, *, /, sqrt) for all inputs that are not NaN; does not specify NaN payload/sign; does not guarantee identical bits for NaN outputs across implementations. [https://stackoverflow.com/questions/42181795/]
- **LLVM docs / `-ffast-math`**: `-ffinite-math-only` / `-ffast-math` assume no NaN/Inf, allow reassociation, change rounding; makes results non-deterministic and architecture-sensitive. Rust does not expose fast-math by default; `rustc` uses strict IEEE unless unsafe/intrinsic fast paths are used. [https://discourse.llvm.org/t/propogation-of-fpclass-assumptions-vis-a-vis-fast-math-flags/76554] [https://news.ycombinator.com/item?id=44142472]
- **Bruce Dawson — Floating-Point Determinism** (2013): explains that identical hardware + binary + inputs yields identical floats; differences come from compiler optimization, different instruction sequences (SSE vs x87), different library implementations (libm), and different rounding modes.

---

## 3. Key technical points

### 3.1 `rustc` / LLVM pipeline
- `rustc` lowers to LLVM IR; LLVM defines float semantics via `LangRef`. Rust relies on LLVM’s constant folding and vectorization, which assume strict IEEE by default.
- No `-ffast-math` by default; Rust does not expose a user-facing “fast float” flag in `rustc` for safe code. (Future fast-path intrinsics discussed in RFC, not stable.)
- LLVM can vectorize / reassociate only when safe under IEEE; still, different LLVM versions may choose different instruction sequences, leading to different NaN bits (not different non-NaN arithmetic, unless fast-math is on).

### 3.2 SSE vs x87 (architecture dependence)
- **64-bit x86 (x86_64)**: uses SSE2 (double-precision, IEEE-compliant). Return-value NaN payload alteration is avoided (unlike 32-bit x86 with x87 ABI).
- **32-bit x86 with SSE2** (`i686` / `pentium` targets with SSE2): mostly SSE; but RFC notes NaN payload can change on return due to x87 register-passing ABI (tracking rust-lang/rust#115567). Arithmetic itself is mostly correct, but bit patterns after function returns may shift.
- **32-bit x86 without SSE2** (`i586` / old `pentium`): uses x87 FPU; computes at 80-bit extended precision internally, then rounds to 64-bit. Results can differ from pure IEEE 64-bit arithmetic (e.g., `0.1 + 0.2` may differ in low bits vs SSE). Per RFC, these targets are **documented as non-compliant**.
- **ARM 32-bit NEON**: always flush-to-zero; if LLVM auto-vectorizes, results diverge from IEEE. Documented non-compliant risk (not common for scalar `f64` unless vectorized).
- **Wasm**: NaN propagation rules differ slightly; canonical NaN rules apply.

### 3.3 Rounding mode / float environment
- Rust does **not** expose floating-point control/status bits (no `fesetround` equivalent in safe Rust; changing FP env via inline assembly is UB per RFC §Assumptions).
- Default is round-ties-to-even, no flush-to-zero, no trap-on-NaN. If a platform runs with non-default rounding (e.g., some embedded environments), results will diverge — but Rust assumes default state.
- `-ffast-math` (not default) can assume no NaN/Inf and reorder operations → different rounding and possibly different non-NaN bits.

### 3.4 NaN = explicit non-determinism
- Per RFC §Reference / §Guide: if any arithmetic produces NaN, sign + payload may vary by compiler version, target, flags, even between compile-time (`const`) and run-time.
- For algorithms that never hit NaN (e.g., modularity with positive weights, no division by zero), this source of variance is eliminated — but only if the algorithm is proven NaN-free.

### 3.5 Library / `libm` functions
- `f64::ln`, `sin`, `cos`, `pow`, `sqrt` (library, not primitive) are explicitly documented as non-deterministic across platforms / Rust versions (source: doc comments + rust issue #150323). If the algorithm uses only `+ - * /` and integer casts, this is not a factor; if it uses `ln`/`pow`/`sqrt`, determinism requires pinning `libm` / target.

---

## 4. Implications for SC-008 / CHK025

- **Same seed + same graph + same build mode = deterministic on same binary / target**, assuming no NaN paths and no fast-math / non-default FP env.
- **Cross-platform determinism is NOT guaranteed by Rust alone**. To claim it, the project should either:
  1. Pin `rustc` version + target triple + build profile (release, no override flags) and document that no NaN-producing paths exist in the arithmetic path; or
  2. Replace float-heavy quality/function calculations with integer/ratio arithmetic where feasible (not always possible for modularity-style metrics); or
  3. Accept that float results are deterministic *within* a pinned environment and treat cross-platform equality as a best-effort / testing-gate rather than a spec guarantee.
- `rustc` / LLVM / SSE-vs-x87 / `-ffast-math` should be explicitly mentioned in any float-determinism claim; otherwise reviewers will correctly flag it as underspecified.

---

## 5. Recommendations (brief, not spec edits)

1. Document in SC-008 / verification note that determinism applies to “same `rustc` + same target + same profile + no fast-math + no NaN-producing operations”.
2. Verify the Leiden / modularity arithmetic uses only `+ - * /` (no `ln` / `pow` / `sqrt`) — if yes, library non-determinism is excluded.
3. Add a CI / test gate that runs same-seed on at least two different host architectures (e.g., x86_64 Linux + ARM Linux) and asserts membership/quality match within `1e-9` (or exact if integer-only), rather than claiming exact cross-platform identity unconditionally.
4. Avoid relying on float equality for convergence / partition comparison; use epsilon or integer encoding for membership.

---

*Sources: RFC 3514; rust-doc `f64`; rust-lang/rust#150323 / #128288 / #115567; IEEE 754-2008; LLVM LangRef / fast-math docs; Bruce Dawson “Floating-Point Determinism” (2013).*  
*File written: `research/float-determinism-check.md` (independent of specs / source).*

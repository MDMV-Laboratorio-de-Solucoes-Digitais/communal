# Provenance of the "~2 s" and "~2.7 s" Baselines in specs/003-optimize-connectedness

**Date:** 2026-09-09 (investigation)
**Scope:** Where do SC-001's "currently ~2 seconds" and SC-002's "currently ~2.7 seconds" come from, and can they coexist with the AGENTS.md ">30 s timeout" records?
**Method:** Full read of the source research doc, grep across `specs/`, `research/`, `docs/`, `crates/`, `examples/`, inspection of `benchmarks/`, `crates/communal-benches/`, git history, and the committed LFR corpus. No files other than this report were modified.

---

## 1. Where each number appears (first and only appearance)

Both figures occur **only** inside `specs/003-optimize-connectedness/spec.md` — a Draft, **untracked in git** (`git status` shows `?? specs/003-optimize-connectedness/`), created during the 2026-09-09/10 clarify sessions. There is no commit hash to cite because none exists.

| Claim | Location | First-appearance context |
|---|---|---|
| "~2 seconds" | `specs/003-optimize-connectedness/spec.md:76` (Clarification, Session 2026-09-09) and `:134` (SC-001) | Line 76 records the edit *in situ*: "The spec states 'currently ~10+ seconds' for d=50, n=10,000 **but benchmarks show ~2 seconds**. Should the baseline be updated? → A: Yes — update SC-001 to reflect actual baseline of ~2 seconds." Line 134 then reads: "currently ~2 seconds with BFS — 4× improvement target" |
| "~2.7 seconds" | `specs/003-optimize-connectedness/spec.md:135` (SC-002) | "currently ~2.7 seconds, must not regress" — appears with **no** justification, citation, or clarification entry anywhere in the spec |

The pre-edit draft value ("~10+ seconds", quoted at line 76) is itself unrecorded — no committed or working-tree file contains it; it survives only as a quotation inside the clarification answer.

**Exhaustive grep result:** the strings `~2 seconds`, `2.7 seconds`, and `benchmarks show` match no file under `specs/` or `research/` other than this one spec. `research/leiden-connectedness-optimization.md` (495 lines, the spec's declared input, spec.md:9) contains **no** second-scale timing figures at all — its only timing references are the ">30 s timeout" records (lines 259 and 395).

## 2. What the candidate sources actually contain

### 2.1 The spec's declared source document — no such numbers

`research/leiden-connectedness-optimization.md` (dated 2026-09-07):

- Line 10: `would_remain_connected` is "an **O(V+E) DFS-based connectivity check** called on every node move attempt".
- Line 259: "For the PolBooks graph (105 nodes, 441 edges) **that times out at >30s**, and PolBlogs (1490 nodes, 19090 edges), this is the dominant cost."
- Line 395: "The Tier 3 real-world networks (PolBooks, Les Misérables, NetScience, PolBlogs) that **currently time out at >30s** should see significant speedup…"
- Line 344: expected speedup of removal stated only as a range: "2-5x on the local moving phase".

The document derives cost analytically (§3.2–3.4: O(V + k + e_c) per call, up to n calls per phase → O(n × (V+E)) per iteration). It never measures a 10k- or 50k-node graph and never produces a "~2 s" or "~2.7 s" figure.

### 2.2 The only performance-baseline research doc — an explicit extrapolation, and the numbers don't match

`research/leiden-performance-baseline.md`:

- Line 7: "The reference C++ implementation by Traag et al. does not publish exact numbers for N=10,000 in isolation, but **we can extrapolate from the paper's benchmark data**."
- Lines 11–13: projected **libleidenalg** runtimes on LFR N=10,000, ⟨k⟩=10: "First iteration: approximately 10–50 ms… Full convergence (all iterations): approximately 50–200 ms."
- Lines 149–155 (summary table): C++ ~10–50 ms, Python ~10–55 ms, Java ~20–100 ms, "Rust target (Communal) < 100 ms" — all for the reference implementations / targets, **not** measurements of communal.

This file contains no ~2 s or ~2.7 s figure, and its extrapolations are 20–100× *below* the spec's claimed BFS baseline — it cannot be their origin.

### 2.3 No benchmark infrastructure or recorded output exists

- `research/leiden-scaling-validation.md` §7 (lines 190–209): "`crates/communal-benches/`: **Placeholder crate** — contains only a stub `run_benchmarks()` function with no actual criterion benchmarks implemented… COMMUNAL currently has **no automated benchmark harness**." Verified directly: `crates/communal-benches/src/lib.rs` is 9 lines, and `target/criterion/` does not exist (no criterion baselines ever recorded).
- `benchmarks/` contains only graph files (`lfr_graphs/`, `real_world/`, two archives) — no JSON/CSV/markdown timing outputs of any kind.
- The committed corpus cannot have produced the claimed profiles: `benchmarks/lfr_graphs/` holds only N=1,000/5,000/10,000 graphs, and its only N=10,000 file, `LFR_N10000_mu0.5.edges`, has a header reading `# Nodes: 10000, Edges: 95319` → **average degree ≈ 19.1, not 50**. **No 50,000-node graph exists anywhere in the repo**, so SC-002's profile was never even materializable for measurement.

### 2.4 Every recorded measurement in the repo contradicts the ~2 s claim

- `AGENTS.md` "TEST RESULTS (2026-03-06)" (Tier 3 table; committed in `4f1d36f`, "docs(agents): add bugs found, test results, and performance notes", dated 2026-09-06 18:06): Karate 59 ms, Dolphins 218 ms, **Football (115 nodes) 1.6 s**, then PolBooks (105 nodes!), Les Misérables (77 nodes), NetScience (1,589), PolBlogs (1,490) all **">30 s ✗ Timeout"**.
- The sibling spec written the same day, `specs/003-leiden-cache-optimization/spec.md` (file dated 2026-09-10 10:43, ~11 h before the optimize-connectedness spec at 21:59), records at lines 245 and 247: "SC-001: PolBooks dataset (105 nodes) completes in under 5 seconds (**currently >30s timeout**)"; "SC-002: PolBlogs dataset (1,490 nodes) completes in under 30 seconds (**currently >30s timeout**)".
- `research/leiden-release-verification-research.md:354` repeats: PolBooks (105 nodes) "times out at >30s" with the per-move BFS as "the dominant cost".
- The BFS check is still present in the current code: `crates/communal-algo/src/leiden/local_moving.rs:537` and `:673`, and `crates/communal-algo/src/leiden/refinement.rs:87`.

## 3. Can the AGENTS.md timeouts and the ~2 s claims be reconciled?

**No — not under any single code state or measurement condition.** Three candidate reconciliations were tested:

1. **Different graph profiles (synthetic vs real-world)?** Doesn't help. Per-move check cost is O(V + k + e_c) with up to O(V) calls per phase (`leiden-connectedness-optimization.md` §3.2–3.4). A 10k-node, d=50 graph does ~10⁸–10⁹+ membership-scan/DFS operations *per phase per iteration*, orders of magnitude more per-iteration work than PolBooks (105 calls × ~10² ops). If PolBooks at ~10⁴ ops/iteration exceeds 30 s, a 10k graph cannot finish in ~2 s on the same code.
2. **Different code states (the timeouts pre-date the cache-optimization merge)?** Partially relevant, but not exculpatory. The AGENTS.md table was committed 2026-09-06 (`4f1d36f`); the cache-optimization feature landed 2026-09-09/10 (PRs #31–#38, e.g. `664dad7`, `1e9ac03`), and the current code retains the per-move BFS. The spec explicitly labels its baseline "**currently ~2 seconds with BFS**" (spec.md:134) — with BFS still active, the claim must compete with the same super-linear trajectory the repo itself measured: Karate→Dolphins→Football gives time exponents of ≈2.2–4.2 per node-count ratio; extrapolating Football's 1.6 s at 115 nodes to 10,000 nodes yields **hours**, not ~2 s, even before counting the 50× higher degree.
3. **Measurement noise / machine differences?** Irrelevant at 3+ orders of magnitude (>30 s at 105 nodes vs ~2 s at 10,000 nodes).

The most plausible origin of "~2 s" is an **ad-hoc, unrecorded run during the 2026-09-09 clarify session** — plausibly on the committed `LFR_N10000_mu0.5.edges` (avg degree 19, not the stated 50) on post-cache-optimization code — whose result was then (a) attributed to "with BFS", (b) relabeled as a d=50 baseline, and (c) echoed into SC-002 as "~2.7 seconds" for a graph that was never generated. Nothing in the repository substantiates any part of that reconstruction; it is offered only as the least-contradictory story. Note the "~2.7" figure's specificity suggests *some* run occurred, but no output, protocol, machine, or code state is recorded, and the profile it claims to measure does not exist in the corpus.

## 4. Verdicts

| Number | Verdict | Basis |
|---|---|---|
| SC-001 "currently ~2 seconds" (spec.md:76, :134) | **UNSOURCED** (at best an unrecorded ad-hoc run on a mismatched graph profile: avg degree 19 vs the stated 50) | Zero measurement records repo-wide; contradicted by the spec's own source doc, by AGENTS.md's measured trajectory, and by the sibling spec's ">30 s" records of the same week; corpus has no d=50 N=10k graph |
| SC-002 "currently ~2.7 seconds" (spec.md:135) | **UNSOURCED** | No measurement record; no 50k-node graph exists in the repo; no clarification entry or research doc mentions it |
| (For contrast) libleidenalg ~10–50 ms on N=10k | EXTRAPOLATED, correctly labeled as such | `research/leiden-performance-baseline.md:7, 11–13` — extrapolated from Traag et al. (2019) figures, not communal |
| AGENTS.md Tier-3 table (>30 s timeouts, Football 1.6 s) | MEASURED | Committed in `4f1d36f` (2026-09-06); the only actual timing measurements in the repository |

## 5. What the spec should say

1. **Delete or re-qualify the two baseline clauses.** Replace "currently ~2 seconds with BFS" (spec.md:134) and "currently ~2.7 seconds" (spec.md:135) with the only defensible statements:
   - Qualitative: "The current implementation times out (>30 s) on real-world graphs as small as 105 nodes (AGENTS.md TEST RESULTS; `research/leiden-connectedness-optimization.md` §3.4); runtime scaling is super-linear."
   - Or quantitative-but-conditional: "Baseline not yet measured for the 10k/d=50 and 50k/d=10 profiles; measure per the Measurement Protocol (spec.md:143) before implementation and record machine specs with the results." The Measurement Protocol already *requires* a pinned reference machine with recorded specs — no such record exists for either number.
2. **If a ~2 s run really was made** on 2026-09-09/10, document it: exact input file (note `LFR_N10000_mu0.5` is d≈19, not 50), code state (post-cache-optimization PRs #31–#38, per-move BFS still present at `local_moving.rs:537`), machine, and protocol — then reconcile it explicitly with the AGENTS.md Football 1.6 s @ 115 nodes data point, which it currently contradicts.
3. **Fix the auxiliary inconsistency while editing:** SC-001's "4× improvement target" (2 s → 500 ms) is arithmetic on an unsourced numerator; and Clarification spec.md:76's "benchmarks show" phrasing should be struck or given a citation, since no benchmark output exists anywhere in the repo (`research/leiden-scaling-validation.md` §7: "no automated benchmark harness").

# Research Notes: CLI Argument Contracts

**Branch**: `001-community-detection` | **Date**: 2026-09-04

## Objective

Research CLI patterns from established Rust CLI tools to inform the design of concrete argument contracts for the Communal Community Detection Framework, resolving the deferral in FR-042.

---

## Research Sources

### 1. ripgrep (BurntSushi/ripgrep)

**Key patterns observed:**
- **Short flags** for common operations: `-n` (line numbers), `-i` (case-insensitive), `-C` (context), `-A` (after-context), `-B` (before-context)
- **Long flags** with hyphens for all options: `--count`, `--case-insensitive`, `--context`
- **Mutually exclusive groups**: `--count` vs `--count-matches` — clap enforces exclusivity
- **Config file support**: Via `RIPGREP_CONFIG_PATH` environment variable; args in config file treated as if passed on CLI
- **Global options**: Implemented via `#[clap(global = true)]` so they work before or after subcommands
- **Output formats**: `--json` flag enables JSON output; `--vimgrep` for machine-parseable output
- **Error messages**: "Did you mean?" suggestions for typos in enum values
- **Value parsing**: Custom parsers for byte sizes (`--max-filesize 10MB`), durations (`--timeout 10s`)

**Applicable to Communal:**
- Use short flags for the most common options (`-a`, `-o`, `-f`, `-g`, `-s`)
- Support comma-separated lists for multiple values (`-a leiden,louvain`)
- Use "did you mean" suggestions for algorithm typos
- Config file via environment variable

### 2. fd (sharkdp/fd)

**Key patterns observed:**
- **Smart defaults**: Case-insensitive, ignores hidden files, respects `.gitignore` — all overridable
- **Type enum via `ArgEnum`**: `-t file`, `-t directory`, `-t symlink`, `-t executable`, `-t empty`, `-t socket`, `-t pipe`
- **Extension filter**: `-e rs`, `-e json`
- **Execution mode**: `-x` (per-result) vs `-X` (batch) — clearly distinguished
- **Smart case**: Auto-detects from pattern (uppercase in pattern = case-sensitive)
- **Parallel execution**: `-x` uses parallel processes by default
- **Config file**: `.fdignore` for persistent ignore patterns

**Applicable to Communal:**
- Smart defaults with clear override flags (e.g., `--directed` overrides undirected default)
- Enum types for algorithm, quality function, output format, etc.
- Clear distinction between modes (e.g., `--lpa-mode asynchronous|semi-synchronous`)

### 3. hyperfine (sharkdp/hyperfine)

**Key patterns observed:**
- **Export options with file paths**: `--export-csv FILE`, `--export-json FILE`, `--export-markdown FILE`
- **Parameter scanning**: `--parameter-scan VAR MIN MAX` with `--parameter-step-size DELTA`
- **Parameter lists**: `--parameter-list VAR VALUES` (comma-separated)
- **Style enum**: `--style auto|basic|full|nocolor|color|none` with `auto` default
- **Shell control**: `--shell`, `--no-shell` (alias for `--shell=none`)
- **Input/output redirection**: `--input WHERE`, `--output WHERE` with null/pipe/file/inherit options
- **Runs specification**: `--min-runs`, `--max-runs`, `--runs` (exact — mutually exclusive)
- **Time unit**: `--time-unit microsecond|millisecond|second`

**Applicable to Communal:**
- Export options with file paths for batch output
- Parameter sweep specification (Cartesian product for batch mode)
- Style/enum defaults with `auto` where appropriate
- Clear mutually exclusive options for run control

### 4. cargo

**Key patterns observed:**
- **Feature flags**: `--features`, `--all-features`, `--no-default-features`
- **Build profiles**: `--release`, `--profile <name>`
- **Manifest path**: `--manifest-path <path>` for alternative Cargo.toml
- **Target directory**: `--target-dir <path>`
- **Frozen/locked**: `--frozen`, `--locked` for CI reproducibility
- **Offline mode**: `--offline`
- **Error messages**: Multi-line with "Caused by" chains

**Applicable to Communal:**
- `--config` for alternative config file path
- `--dry-run` for batch mode preview
- `--continue-on-error` for batch resilience
- Error message format with "Caused by" chains

### 5. Rust CLI Recommendations (sunshowers.io)

**Key patterns observed:**
- **Top-level App struct** with `#[clap(flatten)]` for GlobalOpts + Command enum
- **`#[clap(global = true)]`** for options that work anywhere on command line
- **`ArgEnum` derive** for enum-based argument values
- **`parse(from_occurrences)`** for count-based flags (`-v`, `-vv`, `-vvv`)
- **Liberal use of `flatten`** to compose argument groups into subcommands
- **Expected help string tests** — test that help output matches expected format

**Applicable to Communal:**
- Flat architecture with GlobalOpts + Command enum
- Reusable argument groups via flatten
- `ArgEnum` for all enum-derived args

---

## Design Decisions & Rationale

### 1. Argument Naming Convention

**Decision**: Lowercase with hyphens for long flags; single character for short flags.

**Rationale**: Follows ripgrep/fd/hyperfine convention. Rust CLI ecosystem standard. clap supports this natively via `#[clap(long, short = 'a')]`.

Examples:
- `--algorithm` / `-a`
- `--convergence-mode` (not `--convergenceMode`)
- `--teleportation-rate` (not `--teleportationRate`)

### 2. Default Values

**Decision**: Explicit defaults shown in help output with `[default: X]` annotation.

**Rationale**: Follows fd/hyperfine pattern. Users see defaults without consulting documentation. clap provides this automatically with `default_value_t`.

Key defaults:
| Parameter | Default | Source |
|-----------|---------|--------|
| Gamma | 1.0 | FR-008 |
| Seed | 42 (fixed) | FR-007 |
| Teleportation rate | 0.15 | FR-029 |
| LPA mode | Asynchronous | FR-029 |
| Convergence threshold | 1e-6 | FR-032 |
| Convergence mode | Absolute | FR-032 |
| Max iterations | 1000 | FR-027 |
| Log level | warn | Sensible default (errors + warnings) |
| Output format | json | Machine-parseable default |
| Output destination | stdout | Unix piping convention |

### 3. Mutually Exclusive Options

**Decision**: Explicit mutually exclusive groups for conflicting options.

**Rationale**: clap supports `conflicts_with` and `clap::ArgGroup` for enforcing exclusivity. Prevents ambiguous command lines.

Groups:
- `--weighted` vs `--no-weights` (convert command)
- `--directed` vs `--no-directed` (convert command)
- `--metrics all` vs specific `--metrics` values (compare/metrics commands)
- `--probability` vs `--edges` (Erdős-Rényi generator)
- `--seed <u64>` vs `--seed none` (non-deterministic runs)

### 4. Output Format Flags

**Decision**: `--format json|csv|gml` with `--pretty` modifier for human-readable JSON/CSV.

**Rationale**: Machine-parseable default (json) follows Unix philosophy. Pretty flag for human consumption. GML is domain-standard graph format.

### 5. Config File Support

**Decision**: Layered config with CLI > config file > defaults precedence.

**Rationale**: ripgrep uses `RIPGREP_CONFIG_PATH`. Communal adds `--config` flag + `COMMUNAL_CONFIG` env var. TOML format matches ecosystem convention.

Precedence (highest to lowest):
1. CLI flags (explicit user intent)
2. Config file (session defaults)
3. Built-in defaults (sensible fallbacks)

### 6. Parameter Sweep Semantics

**Decision**: Cartesian product of all sweep dimensions.

**Rationale**: Follows hyperfine's `--parameter-scan` and `--parameter-list` pattern. Predictable combinatorics enable users to estimate total runs.

Formula: `total_runs = |files| × |algorithms| × ∏|sweep_values_per_dimension|`

### 7. Exit Codes

**Decision**: Domain-specific exit codes beyond 0/1.

**Rationale**: Enables scripting and pipeline error handling. Hyperfine and ripgrep use meaningful exit codes.

Codes:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments |
| 2 | Input file error |
| 3 | Algorithm error |
| 4 | Output error |
| 5 | Validation failure |
| 10 | Batch config schema error |

### 8. Shared Argument Groups

**Decision**: Define reusable groups (Algorithm Params, Output Control) via clap flatten.

**Rationale**: Follows Rust CLI Recommendations pattern. Avoids repetition across subcommands. Enables composition.

Groups defined:
- `AlgorithmSelection` — `--algorithm`
- `QualityFunction` — `--quality-function`
- `OutputControl` — `--output`, `--format`, `--pretty`
- `CommonAlgorithmParams` — `--gamma`, `--seed`, `--convergence-threshold`, `--convergence-mode`, `--max-iterations`
- `InfomapParams` — `--teleportation-rate`
- `LpaParams` — `--lpa-mode`
- `FluidParams` — `--target-k`

---

## Implementation Notes for T117-T126

### T117: CLI Argument Parsing

```rust
#[derive(Debug, Parser)]
#[clap(name = "communal", version, about = "Community Detection Framework")]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run(RunArgs),
    Compare(CompareArgs),
    Batch(BatchArgs),
    Convert(ConvertArgs),
    Metrics(MetricsArgs),
    Generate(GenerateArgs),
    Validate(ValidateArgs),
    /// Generate shell completions
    Completions(CompletionsArgs),
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    #[clap(long, short = 'L', value_enum, global = true, default_value_t = LogLevel::Warn)]
    pub log_level: LogLevel,
    #[clap(long, global = true)]
    pub log_dest: Option<PathBuf>,
    #[clap(long, global = true)]
    pub no_log: bool,
    #[clap(long, short = 'C', global = true, env = "COMMUNAL_CONFIG")]
    pub config: Option<PathBuf>,
    #[clap(long, global = true)]
    pub log_graph_data: bool,
    #[clap(long, global = true)]
    pub log_rotation: Option<RotationPolicy>,
}
```

### T118-T124: Command Implementations

Each command maps 1:1 with its Args struct:
- `RunArgs` → `commands/run.rs`
- `CompareArgs` → `commands/compare.rs`
- `BatchArgs` → `commands/batch.rs`
- `ConvertArgs` → `commands/convert.rs`
- `MetricsArgs` → `commands/metrics.rs`
- `GenerateArgs` → `commands/generate.rs`
- `ValidateArgs` → `commands/validate.rs`

### T125: Output Formatters

Implement `OutputFormatter` trait with JSON, CSV, GML variants. Support `pretty` flag for indented output.

### T126: Batch Config Parser

TOML-based batch config with schema validation. Use `serde` + `toml` crates. Validate before execution.

---

## References

- [clap documentation](https://docs.rs/clap/latest/clap/)
- [ripgrep source](https://github.com/BurntSushi/ripgrep)
- [fd source](https://github.com/sharkdp/fd)
- [hyperfine source](https://github.com/sharkdp/hyperfine)
- [Rust CLI Recommendations](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html)
- [Building CLIs in Rust: clap and the Secrets of ripgrep, fd, bat](https://www.youngju.dev/blog/2026-06-27-rust-cli-tools.en)

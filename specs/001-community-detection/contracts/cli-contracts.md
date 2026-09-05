# Contract: CLI Argument Contracts

**Branch**: `001-community-detection` | **Date**: 2026-09-04

## Overview

Defines the concrete CLI argument contracts (flags, options, defaults, types) for the Communal command-line interface. This contract resolves the deferral noted in FR-042 of `spec.md` ("Detailed argument contracts deferred to planning").

The binary is invoked as `communal` with seven subcommands: `run`, `compare`, `batch`, `convert`, `metrics`, `generate`, and `validate`.

---

## Global Options

These options apply to all subcommands and may be specified anywhere on the command line (before or after the subcommand) per clap `global = true` convention.

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--log-level` | `-L` | `LogLevel` | `warn` | Logging verbosity tier |
| `--log-dest` | | `PathBuf` | stderr | Log output destination |
| `--no-log` | | `bool` | `false` | Disable all logging output |
| `--config` | `-C` | `PathBuf` | None | Path to layered config file (TOML) |

### Log Level Enum

```
LogLevel = error | warn | info | debug | trace
```

Maps to the tiered logging system defined in FR-031. Respects the zero-trust logging posture: even at `debug` tier, graph data/node identifiers/topology are excluded unless `--log-graph-data` is explicitly set.

| Tier | Description | Use Case |
|------|-------------|----------|
| `error` | Fatal errors only | Production silent operation |
| `warn` | Non-fatal issues | **Default** — surfaces problems without noise |
| `info` | High-level phase transitions | Standard user feedback |
| `debug` | Per-node moves and splits | Algorithm debugging |
| `trace` | Full internal state dumps | Deep diagnostics |

### Additional Global Flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--log-graph-data` | `bool` | `false` | Opt-in: allow graph data, node IDs, and topology in log output (overrides FR-031 zero-trust default) |
| `--log-rotation` | `RotationPolicy` | `never` | Log file rotation policy (only effective with `--log-dest` pointing to a file) |

### Rotation Policy Enum

```
RotationPolicy = never | size <BYTES> | daily | hourly
```

Example: `--log-rotation size 1048576` rotates at 1 MB.

---

## Layered Configuration

Configuration follows a layered precedence model (highest to lowest priority):

1. **CLI flags** — explicit command-line arguments
2. **Config file** — TOML file specified via `--config` or `COMMUNAL_CONFIG` env var
3. **Defaults** — built-in defaults documented below

### Config File Schema (TOML)

```toml
# communal.toml — global configuration

[global]
log_level = "info"
log_dest = "~/.local/share/communal/communal.log"
log_rotation = { size = 1048576 }

[global.output]
format = "json"
pretty = false
```

Environment variable `COMMUNAL_CONFIG` provides an additional config file path (combined with `--config` if both are set; CLI flags always win).

---

## Shared Argument Groups

These argument groups are reused across multiple subcommands via clap's `flatten` pattern.

### Algorithm Selection

| Flag | Short | Type | Description |
|------|-------|------|-------------|
| `--algorithm` | `-a` | `Algorithm` | Single algorithm to execute |

```
Algorithm = leiden | louvain | infomap | lpa | fluid
```

### Quality Function Selection

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--quality-function` | `-q` | `QualityFunction` | `modularity` | Quality function for optimization |

```
QualityFunction = modularity | cpm | map_equation
```

### Graph Directionality

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--directed` | `bool` | `false` | Treat graph as directed (no symmetrization) |

### Input Validation

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--no-validate` | `bool` | `false` | Skip input validation (defer to algorithm execution time) |

### Output Control

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--output` | `-o` | `PathBuf` | stdout | Output file path |
| `--format` | `-f` | `OutputFormat` | `json` | Output format |
| `--pretty` | `-p` | `bool` | `false` | Pretty-print output (JSON/CSV only) |

```
OutputFormat = json | csv | gml
```

### Common Algorithm Parameters

| Flag | Type | Default | Description | Constraints |
|------|------|---------|-------------|-------------|
| `--gamma` | `f64` | `1.0` | Resolution parameter | `> 0.0` |
| `--seed` | `Option<u64>` | `42` | Random seed for determinism | Use `--seed none` for non-deterministic |
| `--convergence-threshold` | `f64` | `1e-6` | Convergence stopping threshold | `>= 0.0` |
| `--convergence-mode` | `ConvergenceMode` | `absolute` | Convergence measurement mode | |
| `--max-iterations` | `usize` | `1000` | Maximum iterations before forced termination | `>= 1` |

```
ConvergenceMode = absolute | relative
```

### Infomap-Specific Parameters

| Flag | Type | Default | Description | Constraints |
|------|------|---------|-------------|-------------|
| `--teleportation-rate` | `f64` | `0.15` | Infomap teleportation rate | `[0.0, 1.0]` |

### LPA-Specific Parameters

| Flag | Type | Default | Description |
|------|-------|------|-------------|
| `--lpa-mode` | `LpaUpdateMode` | `asynchronous` | LPA update mode |

```
LpaUpdateMode = asynchronous | semi-synchronous
```

Note: Synchronous mode is NOT supported (per FR-029, causes oscillation on bipartite graphs).

### Fluid-Specific Parameters

| Flag | Short | Type | Default | Description | Constraints |
|------|-------|------|---------|-------------|-------------|
| `--target-k` | `-k` | `usize` | (auto = n) | Fluid target community count | `[1, n]` where n = node count |

---

## Command: `run`

Executes a single community detection algorithm on an input graph.

### Usage

```bash
communal run <INPUT> --algorithm <ALGORITHM> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `INPUT` | Yes | `PathBuf` | Input graph file path |

### Output Schema (JSON)

```json
{
  "algorithm": "leiden",
  "quality_function": "modularity",
  "partition": {
    "membership": [0, 0, 1, 1, 2],
    "community_count": 3,
    "community_sizes": { "0": 2, "1": 2, "2": 1 }
  },
  "quality": {
    "modularity_q": 0.4521,
    "cpm": null,
    "map_equation": null
  },
  "metadata": {
    "iterations": 12,
    "convergence_delta": 8.3e-7,
    "execution_time_ms": 45,
    "seed_used": 42,
    "gamma_used": 1.0,
    "node_count": 5,
    "edge_count": 6
  }
}
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success — partition computed |
| 1 | Invalid arguments |
| 2 | Input file error (not found, parse failure) |
| 3 | Algorithm error (non-convergence, invalid graph) |
| 4 | Output error (serialization, write failure) |

### Examples

```bash
# Basic Leiden run (defaults: modularity, gamma=1.0, seed=42)
communal run graph.edgelist --algorithm leiden

# Louvain with custom resolution
communal run graph.json --algorithm louvain --gamma 0.8 --format csv

# Infomap with custom teleportation
communal run graph.gml --algorithm infomap --teleportation-rate 0.2 -o result.json

# LPA in semi-synchronous mode
communal run graph.edgelist --algorithm lpa --lpa-mode semi-synchronous

# Fluid with target communities
communal run graph.edgelist --algorithm fluid --target-k 5

# Non-deterministic run
communal run graph.edgelist --algorithm leiden --seed none

# Strict convergence
communal run graph.edgelist --algorithm leiden --convergence-threshold 0.0
```

---

## Command: `compare`

Runs multiple algorithms on the same input and produces a metrics table with pairwise NMI/ARI comparison.

### Usage

```bash
communal compare <INPUT> --algorithms <ALGOS> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `INPUT` | Yes | `PathBuf` | Input graph file path |

### Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--algorithms` | `-a` | `Vec<Algorithm>` | (required) | Comma-separated algorithm list |
| `--metrics` | `-m` | `Vec<MetricType>` | `all` | Metrics to compute for each algorithm |
| `--comparison-metrics` | | `Vec<ComparisonMetric>` | `[nmi, ari]` | Pairwise comparison metrics |

```
MetricType = modularity | cpm | map_equation | all
ComparisonMetric = nmi | ari
```

### Mutually Exclusive Options

- `--metrics all` cannot be combined with specific `--metrics` values.
- Algorithm list requires at least 2 distinct algorithms.

### Output Schema (JSON)

```json
{
  "input": "graph.edgelist",
  "algorithms": [
    {
      "name": "leiden",
      "partition": {
        "membership": [0, 0, 1, 1],
        "community_count": 2,
        "community_sizes": { "0": 2, "1": 2 }
      },
      "quality": {
        "modularity_q": 0.512,
        "cpm": -0.023,
        "map_equation": 0.445
      },
      "execution_time_ms": 45
    },
    {
      "name": "louvain",
      "partition": {
        "membership": [0, 0, 0, 1],
        "community_count": 2,
        "community_sizes": { "0": 3, "1": 1 }
      },
      "quality": {
        "modularity_q": 0.498,
        "cpm": -0.028,
        "map_equation": null
      },
      "execution_time_ms": 32
    }
  ],
  "comparison": {
    "pairs": [
      {
        "algo_a": "leiden",
        "algo_b": "louvain",
        "nmi": 0.892,
        "ari": 0.845
      }
    ]
  },
  "metadata": {
    "total_execution_time_ms": 89,
    "seed": 42,
    "gamma": 1.0
  }
}
```

### Examples

```bash
# Compare two algorithms (default metrics: all)
communal compare graph.edgelist -a leiden,louvain

# Compare all five algorithms, CSV output
communal compare graph.json -a leiden,louvain,infomap,lpa,fluid -f csv

# Compare with specific metrics only
communal compare graph.edgelist -a leiden,louvain -m modularity

# Compare with custom parameters
communal compare graph.gml -a leiden,infomap --gamma 1.2 --seed 123
```

---

## Command: `batch`

Processes multiple files, algorithms, and/or parameter sweeps via a unified batch configuration file.

### Usage

```bash
communal batch --config <CONFIG> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `--config` | Yes | `PathBuf` | Batch configuration file (TOML) |

### Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--config` | `-c` | `PathBuf` | (required) | Batch configuration file path |
| `--output-dir` | `-d` | `PathBuf` | `./communal-output/` | Output directory |
| `--parallel` | `-j` | `usize` | `num_cpus` | Number of parallel batch workers |
| `--dry-run` | | `bool` | `false` | Preview batch without executing |
| `--continue-on-error` | | `bool` | `false` | Continue batch if individual runs fail |
| `--resume` | | `PathBuf` | None | Resume from checkpoint file |

### Batch Configuration Schema (TOML)

```toml
# batch.toml — unified batch configuration

[batch]
# File batching: process multiple files
files = [
    "graphs/karate.edgelist",
    "graphs/dolphins.edgelist",
    "graphs/cora.edgelist"
]

# Algorithm batching: run multiple algorithms on each file
algorithms = ["leiden", "louvain", "infomap"]

# Per-algorithm parameters (optional; inherit global defaults if absent)
[batch.params.leiden]
gamma = 1.0
seed = 42

[batch.params.infomap]
teleportation_rate = 0.15

[batch.params.lpa]
lpa_mode = "asynchronous"

[batch.params.fluid]
target_k = 4

# Parameter sweep: vary gamma across runs
[batch.sweep.gamma]
values = [0.5, 1.0, 1.5, 2.0]

# Parameter sweep: vary seed for stability analysis
[batch.sweep.seed]
values = [42, 123, 456, 789]

# Output configuration
[output]
directory = "./results/"
format = "json"
include_comparison = true
comparison_metrics = ["nmi", "ari"]

# Execution configuration
[execution]
parallel_workers = 4
continue_on_error = true
```

### Parameter Sweep Semantics

- **Cross-product**: All sweep dimensions are combined (Cartesian product).
- Example: `algorithms = ["leiden", "louvain"]` × `gamma = [0.5, 1.0, 2.0]` × `seed = [42, 123]` = 12 runs.
- **Total runs** = |files| × |algorithms| × |sweep dimensions cross-product|.

### Output Structure

```
results/
├── karate_leiden_g1.0_s42.json
├── karate_leiden_g1.0_s123.json
├── karate_louvain_g1.0_s42.json
├── ...
├── dolphins_leiden_g1.0_s42.json
├── ...
├── batch_report.json        # Summary of all runs
├── comparison_matrix.json   # Pairwise NMI/ARI per file
└── batch_checkpoint.json    # Resume state (if --resume)
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All batch runs succeeded |
| 1 | Invalid arguments or config parse error |
| 2 | Input file error |
| 3 | Some runs failed (with `--continue-on-error`) |
| 4 | Output error |
| 10 | Batch config schema validation failed |

### Examples

```bash
# Run batch from config
communal batch --config sweep.toml

# Override output directory and parallelism
communal batch -c sweep.toml -d ./my-results/ -j 8

# Dry run to preview planned executions
communal batch -c sweep.toml --dry-run

# Resume interrupted batch
communal batch -c sweep.toml --resume ./results/batch_checkpoint.json
```

---

## Command: `convert`

Converts between graph serialization formats.

### Usage

```bash
communal convert <INPUT> <OUTPUT> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `INPUT` | Yes | `PathBuf` | Input graph file path |
| `OUTPUT` | Yes | `PathBuf` | Output graph file path |

### Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--from` | | `GraphFormat` | (auto-detect) | Input format override |
| `--to` | | `GraphFormat` | (auto-detect) | Output format override |
| `--directed` | `-d` | `bool` | `false` | Preserve directionality (undirected by default) |
| `--weighted` | `-w` | `bool` | (preserve) | Include weights in output |
| `--no-weights` | | `bool` | `false` | Strip weights from output |

```
GraphFormat = edgelist | json | gml
```

### Format Auto-Detection

Format is auto-detected from file extension if `--from`/`--to` are not specified:

| Extension | Format |
|-----------|--------|
| `.edgelist`, `.edges`, `.txt` | EdgeList |
| `.json` | JSON |
| `.gml` | GML |

### Mutually Exclusive Options

- `--weighted` and `--no-weights` are mutually exclusive.
- `--directed` and `--no-directed` are mutually exclusive.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Conversion successful |
| 1 | Invalid arguments |
| 2 | Input file error |
| 4 | Output error (write failure) |

### Examples

```bash
# Auto-detect formats from extensions
communal convert graph.gml graph.json

# Explicit format conversion
communal convert input.txt output.gml --from edgelist --to gml

# Convert to unweighted EdgeList
communal convert graph.json graph.edgelist --no-weights

# Force directed graph output
communal convert graph.edgelist graph.json --directed
```

---

## Command: `metrics`

Computes quality and comparative metrics for existing partitions without running detection.

### Usage

```bash
communal metrics <INPUT> --partition <PARTITION> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `INPUT` | Yes | `PathBuf` | Input graph file |
| `--partition` | Yes | `PathBuf` | Partition file (JSON membership vector) |

### Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--partition` | `-p` | `PathBuf` | (required) | Path to partition file |
| `--metrics` | `-m` | `Vec<MetricType>` | `all` | Quality metrics to compute |
| `--ground-truth` | `-g` | `PathBuf` | None | Ground truth partition file (enables NMI/ARI) |
| `--comparison` | | `PathBuf` | None | Second partition file for pairwise comparison |
| `--gamma` | | `f64` | `1.0` | Resolution parameter for modularity/CPM |

### Partition File Schema (Input)

```json
{
  "membership": [0, 0, 1, 1, 2],
  "node_ids": ["a", "b", "c", "d", "e"]
}
```

If `node_ids` is omitted, indices are used as identifiers.

### Output Schema (JSON)

```json
{
  "graph": {
    "node_count": 5,
    "edge_count": 6,
    "directed": false
  },
  "partition": {
    "source": "partition.json",
    "community_count": 3,
    "community_sizes": { "0": 2, "1": 2, "2": 1 },
    "has_disconnected_communities": false
  },
  "quality": {
    "modularity_q": 0.4521,
    "cpm": -0.0234,
    "map_equation": null
  },
  "comparison": {
    "ground_truth": "truth.json",
    "nmi": 0.891,
    "ari": 0.845
  }
}
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Metrics computed successfully |
| 1 | Invalid arguments |
| 2 | Input file error (graph or partition) |
| 5 | Partition validation failure (size mismatch, invalid IDs) |

### Examples

```bash
# Compute all quality metrics for a partition
communal metrics graph.edgelist -p partition.json

# Compare partition against ground truth
communal metrics graph.edgelist -p detected.json -g truth.json

# Compute modularity with custom resolution
communal metrics graph.edgelist -p partition.json -m modularity --gamma 1.5

# Pairwise comparison of two partitions (no ground truth needed)
communal metrics graph.edgelist -p algo1.json --comparison algo2.json
```

---

## Command: `generate`

Generates synthetic benchmark graphs with known community structure.

### Usage

```bash
communal generate <TYPE> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `TYPE` | Yes | `GeneratorType` | Graph generator type |

### Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--output` | `-o` | `PathBuf` | stdout | Output file path |
| `--format` | `-f` | `OutputFormat` | `edgelist` | Output format |
| `--seed` | `-s` | `Option<u64>` | `42` | Random seed |
| `--save-membership` | | `PathBuf` | None | Save ground truth membership to file |

### Generator Types and Parameters

#### LFR Benchmark

| Flag | Short | Type | Default | Description | Constraints |
|------|-------|------|---------|-------------|-------------|
| `--nodes` | `-n` | `usize` | (required) | Number of nodes | `>= 1` |
| `--tau1` | | `f64` | (required) | Degree distribution exponent | `> 1.0` |
| `--tau2` | | `f64` | (required) | Community size distribution exponent | `> 1.0` |
| `--mu` | | `f64` | (required) | Mixing parameter | `[0.0, 1.0]` |
| `--avg-degree` | `-k` | `f64` | `10.0` | Average degree | `>= 1.0` |
| `--max-degree` | | `usize` | `(auto)` | Maximum degree | `>= avg_degree` |
| `--min-community` | | `usize` | `20` | Minimum community size | `>= 1` |
| `--max-community` | | `usize` | `(auto)` | Maximum community size | `>= min_community` |

#### Stochastic Block Model (SBM)

| Flag | Short | Type | Default | Description | Constraints |
|------|-------|------|---------|-------------|-------------|
| `--blocks` | `-b` | `usize` | (required) | Number of blocks/communities | `>= 2` |
| `--nodes` | `-n` | `usize` | (required) | Total number of nodes | `>= blocks` |
| `--pin` | | `f64` | (required) | Intra-community edge probability | `[0.0, 1.0]` |
| `--pout` | | `f64` | (required) | Inter-community edge probability | `[0.0, 1.0]` |
| `--size-ratios` | | `Vec<f64>` | (uniform) | Relative block sizes (must sum to 1.0) | |

#### Barabási-Albert Model

| Flag | Short | Type | Default | Description | Constraints |
|------|-------|------|---------|-------------|-------------|
| `--nodes` | `-n` | `usize` | (required) | Number of nodes | `>= 1` |
| `--edges-per-node` | `-m` | `usize` | (required) | Edges per new node | `>= 1` |
| `--initial-clique` | | `usize` | `m + 1` | Initial clique size | `>= edges_per_node` |

#### Erdős-Rényi Model

| Flag | Short | Type | Default | Description | Constraints |
|------|-------|------|---------|-------------|-------------|
| `--nodes` | `-n` | `usize` | (required) | Number of nodes | `>= 1` |
| `--probability` | `-p` | `f64` | (required) | Edge probability | `[0.0, 1.0]` |
| `--edges` | `-e` | `usize` | (alternative to -p) | Exact number of edges | `[0, n*(n-1)/2]` |

Note: `--probability` and `--edges` are mutually exclusive (alternative generation modes).

### Mutually Exclusive Options

- `--probability` and `--edges` (Erdős-Rényi) cannot be combined.

### Output Schema (JSON with `--save-membership`)

The membership file follows the same schema as the partition file:

```json
{
  "membership": [0, 0, 1, 1, 2, 2, 2],
  "generator": "lfr",
  "parameters": {
    "nodes": 7,
    "tau1": 3.0,
    "tau2": 1.5,
    "mu": 0.3
  }
}
```

### Examples

```bash
# LFR benchmark (most common)
communal generate lfr -n 1000 --tau1 3 --tau2 1.5 --mu 0.3 -o lfr.edgelist

# LFR with ground truth saved
communal generate lfr -n 5000 --tau1 2.5 --tau2 1.5 --mu 0.2 --save-membership truth.json

# SBM with 4 blocks
communal generate sbm -b 4 -n 1000 --pin 0.1 --pout 0.01 -o sbm.edgelist

# Scale-free network
communal generate barabasi-albert -n 10000 --edges-per-node 4 -o ba.edgelist

# Random graph
communal generate erdos-renyi -n 500 --probability 0.05 -o er.edgelist

# Erdős-Rényi with exact edge count
communal generate erdos-renyi -n 100 --edges 2500 -o er.edgelist
```

---

## Command: `validate`

Validates graph file structure and content without running algorithms.

### Usage

```bash
communal validate <INPUT> [OPTIONS]
```

### Arguments

| Argument | Required | Type | Description |
|----------|----------|------|-------------|
| `INPUT` | Yes | `PathBuf` | Input graph file path |

### Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--format` | `-f` | `GraphFormat` | (auto-detect) | Force input format |
| `--strict` | | `bool` | `false` | Treat warnings as errors |
| `--max-warnings` | | `usize` | `50` | Cap warning output (prevent flooding) |

### Output Schema (JSON)

```json
{
  "valid": true,
  "format": "edgelist",
  "statistics": {
    "node_count": 100,
    "edge_count": 450,
    "self_loops": 3,
    "zero_weight_edges": 0,
    "negative_weight_edges": 0,
    "duplicate_edges": 2,
    "isolated_nodes": 5
  },
  "issues": [],
  "warnings": [
    {
      "code": "DUPLICATE_EDGE",
      "message": "Duplicate edge between 42 and 17 (weights: 1.0, 0.8) — will be summed",
      "line": 23
    },
    {
      "code": "SELF_LOOP",
      "message": "Self-loop detected at node 42 (weight: 1.0)",
      "line": 56
    }
  ]
}
```

### Warning Codes

| Code | Severity | Description |
|------|----------|-------------|
| `SELF_LOOP` | Warning | Self-loop present (allowed per FR-026) |
| `DUPLICATE_EDGE` | Warning | Multiple edges between same nodes (weights summed) |
| `ZERO_WEIGHT_EDGE` | Warning | Edge with weight 0.0 (allowed, no effect) |
| `ISOLATED_NODE` | Warning | Node with degree 0 |
| `NON_CONTIGUOUS_IDS` | Info | Node IDs are not contiguous (handled automatically) |
| `MISSING_WEIGHTS` | Info | Some edges lack weights (default to 1.0) |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Validation passed (graph is valid) |
| 1 | Invalid arguments |
| 2 | Input file error (not found, unreadable) |
| 5 | Validation failed (graph has critical issues) |

With `--strict`, warnings trigger exit code 5.

### Examples

```bash
# Basic validation
communal validate graph.edgelist

# Strict mode (fail on warnings)
communal validate graph.json --strict

# Validate specific format
communal validate data.txt --format edgelist

# Pipe-friendly output
communal validate graph.gml -f json --max-warnings 10
```

---

## Command-Line Interface Conventions

### Argument Naming Conventions

| Pattern | Convention | Examples |
|---------|------------|----------|
| Long flags | Lowercase with hyphens | `--algorithm`, `--convergence-mode` |
| Short flags | Single uppercase or distinctive lowercase | `-a`, `-o`, `-f`, `-L` |
| Enum values | Lowercase with hyphens | `semi-synchronous`, `as-relative` |
| File paths | Positional or `--flag` | `<INPUT>`, `--output` |
| Numbers | CamelCase in docs, lowercase on CLI | `--max-iterations` |

### Default Value Display

Defaults are shown in help output in square brackets:

```
--gamma <GAMMA>    Resolution parameter [default: 1.0]
--seed <SEED>      Random seed (use 'none' for random) [default: 42]
--lpa-mode <MODE>  LPA update mode [default: asynchronous]
```

### Value Parsing

| Type | Parser | Examples |
|------|--------|----------|
| `f64` | Standard float parsing | `1.0`, `1e-6`, `0.5` |
| `u64` | Standard integer parsing | `42`, `1000`, `0` |
| `usize` | Standard integer parsing | `4`, `100` |
| `bool` | Flag presence (true) / absence | `--directed` = true |
| `PathBuf` | Valid UTF-8 path | `graph.edgelist`, `/tmp/out.json` |
| Enum | Case-insensitive variant name | `leiden`, `LEIDEN`, `LeiDen` |
| `Vec<T>` | Comma-separated or repeated | `-a leiden,louvain` or `-a leiden -a louvain` |
| `Option<T>` | Optional value | `--seed 42` or `--seed none` |

### Validation Rules

1. **Numeric bounds**: Out-of-range values produce descriptive errors:
   ```
   error: invalid value '1.5' for '--teleportation-rate <RATE>': must be in [0.0, 1.0]
   ```

2. **Path validation**: Input files must exist; output paths must be creatable.

3. **Enum validation**: Invalid enum variants produce `possible_values` suggestions:
   ```
   error: invalid value 'leen' for '--algorithm <ALGORITHM>': did you mean 'leiden'?
   [possible values: leiden, louvain, infomap, lpa, fluid]
   ```

4. **Mutually exclusive flags**: Produce clear conflict messages:
   ```
   error: '--weighted' cannot be used with '--no-weights'
   ```

### Shell Completion

```bash
# Generate completions for your shell
communal completions bash > ~/.local/share/bash-completion/completions/communal
communal completions zsh > ~/.local/share/zsh/site-functions/_communal
communal completions fish > ~/.config/fish/completions/communal.fish
```

---

## Environment Variables

| Variable | Description | Overrides |
|----------|-------------|-----------|
| `COMMUNAL_CONFIG` | Path to default config file | `--config` (CLI wins) |
| `COMMUNAL_LOG_LEVEL` | Default log level | `--log-level` (CLI wins) |
| `COMMUNAL_LOG_DEST` | Default log destination | `--log-dest` (CLI wins) |
| `COMMUNAL_LOG_GRAPH_DATA` | Opt-in graph data in logs | `--log-graph-data` (CLI wins) |
| `NO_COLOR` | Disable ANSI colors (if added in future) | — |

---

## Error Messages Format

```
error: <short description>

  Caused by:
    <chain of causes if applicable>

  Suggestion:
    <helpful fix suggestion>
```

Example:

```
error: could not parse graph file 'graph.edgelist'

  Caused by:
    invalid format at line 42: expected 2 or 3 columns, found 1

  Suggestion:
    EdgeList format requires: source target [weight]
    Example: 0 1 0.5
```

---

## Migration from `cli-schema.md`

This contract supersedes the high-level schema defined in `contracts/cli-schema.md`. Key changes:

1. **Concrete types and defaults** replace placeholder descriptions.
2. **Global options** extracted as reusable group.
3. **Shared argument groups** explicitly defined (Algorithm Params, Output Control).
4. **Parameter sweep semantics** formally specified (Cartesian product).
5. **Exit codes** expanded and command-specific.
6. **Environment variables** documented.
7. **Validation rules** standardized.

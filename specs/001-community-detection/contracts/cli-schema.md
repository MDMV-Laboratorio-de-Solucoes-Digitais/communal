# Contract: CLI Command Schema

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

Defines the CLI command structure, input/output formats, and configuration schemas for the Communal command-line interface.

---

## Command Structure

```
communal <COMMAND> [OPTIONS]
```

### Commands

| Command | Description |
|---------|-------------|
| `run` | Execute single algorithm on input graph |
| `compare` | Compare multiple algorithms side-by-side |
| `batch` | Process multiple files/algorithms via config |
| `convert` | Convert between graph formats |
| `metrics` | Compute quality metrics for a partition |
| `generate` | Generate synthetic benchmark graphs |
| `validate` | Validate graph structure |

---

## Command: `run`

Executes a community detection algorithm on an input graph.

### Usage

```bash
communal run <INPUT> --algorithm <ALGORITHM> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `INPUT` | Yes | Input graph file path |

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--algorithm` | `-a` | (required) | Algorithm: leiden, louvain, infomap, lpa, fluid |
| `--output` | `-o` | stdout | Output file path |
| `--format` | `-f` | json | Output format: json, csv, gml |
| `--gamma` | `-g` | 1.0 | Resolution parameter |
| `--seed` | `-s` | None | Random seed for determinism |
| `--directed` | | false | Treat graph as directed |
| `--no-validate` | | false | Skip input validation |
| `--teleportation-rate` | | 0.15 | Infomap teleportation rate |
| `--sync-mode` | | false | LPA synchronous mode |
| `--target-k` | `-k` | (required for fluid) | Fluid target community count |
| `--convergence-threshold` | | 1e-6 | Convergence threshold |
| `--convergence-mode` | | absolute | Convergence mode: absolute, relative |
| `--max-iterations` | | 1000 | Maximum iterations |

### Output Schema (JSON)

```json
{
  "algorithm": "leiden",
  "partition": {
    "node_communities": { "0": 0, "1": 0, "2": 1, ... },
    "community_count": 2,
    "community_sizes": { "0": 2, "1": 1 }
  },
  "quality": {
    "modularity_q": 0.452,
    "cpm": null,
    "map_equation": null
  },
  "metadata": {
    "iterations": 12,
    "convergence_delta": 0.0000008,
    "execution_time_ms": 45
  }
}
```

---

## Command: `compare`

Compares multiple algorithms on the same input graph.

### Usage

```bash
communal compare <INPUT> --algorithms <ALGOS> [OPTIONS]
```

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--algorithms` | `-a` | (required) | Comma-separated list: leiden,louvain,infomap,lpa,fluid |
| `--output` | `-o` | stdout | Output file path |
| `--metrics` | `-m` | all | Metrics to compute: modularity, cpm, all |

### Output Schema (JSON)

```json
{
  "input": "graph.edgelist",
  "algorithms": [
    {
      "name": "leiden",
      "partition": { "community_count": 3, ... },
      "quality": { "modularity_q": 0.512, ... },
      "execution_time_ms": 45
    },
    {
      "name": "louvain",
      "partition": { "community_count": 4, ... },
      "quality": { "modularity_q": 0.498, ... },
      "execution_time_ms": 32
    }
  ],
  "comparison": {
    "leiden_vs_louvain": {
      "nmi": 0.892,
      "ari": 0.845
    }
  }
}
```

---

## Command: `batch`

Processes multiple files and/or algorithms via configuration file.

### Usage

```bash
communal batch --config <CONFIG> [OPTIONS]
```

### Configuration Schema (TOML)

```toml
# batch.toml

[batch]
# File batching: process multiple files
files = [
    "graphs/karate.edgelist",
    "graphs/dolphins.edgelist",
    "graphs/cora.edgelist"
]

# Algorithm batching: run multiple algorithms on each file
algorithms = ["leiden", "louvain", "infomap"]

# Parameter sweep: vary gamma across runs
[batch.sweep]
gamma = [0.5, 1.0, 1.5, 2.0]
# Can also sweep: seed, convergence_threshold, etc.

# Per-algorithm parameters
[algorithms.leiden]
gamma = 1.0
seed = 42

[algorithms.infomap]
teleportation_rate = 0.15

[algorithms.lpa]
sync_mode = false

# Output configuration
[output]
directory = "./results/"
format = "json"
include_comparison = true
```

### Output

Creates output directory with:
```
results/
├── karate_leiden.json
├── karate_louvain.json
├── karate_infomap.json
├── dolphins_leiden.json
├── ...
└── comparison_report.json
```

---

## Command: `convert`

Converts between graph formats.

### Usage

```bash
communal convert <INPUT> <OUTPUT> --from <FORMAT> --to <FORMAT>
```

### Supported Formats

| Format | Extensions | Description |
|--------|------------|-------------|
| EdgeList | `.edgelist`, `.edges`, `.txt` | `source target [weight]` |
| JSON | `.json` | Adjacency list or edge list |
| GML | `.gml` | Graph Modelling Language |

### Options

| Option | Description |
|--------|-------------|
| `--from <FORMAT>` | Input format (auto-detected from extension if omitted) |
| `--to <FORMAT>` | Output format (required) |
| `--weighted` | Include weights in output |
| `--directed` | Preserve directionality |

---

## Command: `metrics`

Computes quality metrics for a partition.

### Usage

```bash
communal metrics <INPUT> --partition <PARTITION> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `INPUT` | Yes | Input graph file |
| `--partition` | Yes | Partition file (JSON with node→community mapping) |

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--ground-truth` | None | Ground truth file for NMI/ARI |
| `--gamma` | 1.0 | Resolution parameter for modularity/CPM |

---

## Command: `generate`

Generates synthetic benchmark graphs.

### Usage

```bash
communal generate <TYPE> --output <OUTPUT> [OPTIONS]
```

### Types

| Type | Description | Key Parameters |
|------|-------------|----------------|
| `lfr` | Lanchichinetti-Fortunato-Radicchi benchmark | `--nodes`, `--tau1`, `--tau2`, `--mu` |
| `sbm` | Stochastic Block Model | `--blocks`, `--pin`, `--pout` |
| `barabasi-albert` | Scale-free network | `--nodes`, `--edges-per-node` |
| `erdos-renyi` | Random graph | `--nodes`, `--probability` |

### Example

```bash
communal generate lfr --nodes 1000 --tau1 3 --tau2 1.5 --mu 0.3 --output lfr_benchmark.edgelist
```

---

## Command: `validate`

Validates graph structure without running algorithms.

### Usage

```bash
communal validate <INPUT> [OPTIONS]
```

### Output

```json
{
  "valid": true,
  "node_count": 100,
  "edge_count": 450,
  "issues": [],
  "warnings": [
    "Self-loop detected at node 42 (allowed but may affect results)"
  ]
}
```

---

## Input Format: EdgeList

Standard edge list format with optional weights.

```
# Comments start with #
# Format: source [target [weight]]
0 1
0 2 0.5
1 2 1.0
2 3
```

- Nodes are non-negative integers
- Weights default to 1.0 if omitted
- Undirected by default (both directions implied)
- Self-loops allowed (e.g., `5 5 1.0`)

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments |
| 2 | Input file error |
| 3 | Algorithm error (non-convergence, etc.) |
| 4 | Output error |
| 5 | Validation failure |

---

## Examples

```bash
# Run Leiden on karate club
communal run karate.edgelist --algorithm leiden --output karate_leiden.json

# Compare algorithms
communal compare karate.edgelist --algorithms leiden,louvain,infomap --output comparison.json

# Batch processing with parameter sweep
communal batch --config sweep.toml

# Convert GML to EdgeList
communal convert graph.gml graph.edgelist --to edgelist

# Generate LFR benchmark
communal generate lfr --nodes 5000 --output benchmark.edgelist

# Validate graph
communal validate input.edgelist
```

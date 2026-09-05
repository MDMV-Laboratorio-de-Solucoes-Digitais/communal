# Quickstart: Community Detection Framework

**Branch**: `001-community-detection` | **Date**: 2026-09-03

## Overview

This guide provides runnable validation scenarios to verify the Community Detection Framework works end-to-end. Each scenario includes prerequisites, commands, and expected outcomes.

---

## Prerequisites

### Build the Project

```bash
# Clone the repository
git clone https://github.com/MDMV-Laboratorio-de-Solucoes-Digitais/communal.git
cd communal

# Build all crates
cargo build --all-features

# Run tests
cargo test --all-features
```

### Test Data

Create a test graph file `test_graph.edgelist`:

```bash
cat > test_graph.edgelist << 'EOF'
# Simple graph with 2 clear communities
# Community 1: nodes 0-3
0 1
0 2
1 2
1 3
2 3
# Community 2: nodes 4-7
4 5
4 6
5 6
5 7
6 7
# Bridge edge (weak connection between communities)
3 4
EOF
```

---

## Scenario 1: Basic Leiden Algorithm

**Goal**: Verify Leiden produces connected communities with correct structure.

### Steps

```bash
# Run Leiden algorithm
cargo run --bin communal -- run test_graph.edgelist \
  --algorithm leiden \
  --output leiden_result.json \
  --seed 42
```

### Expected Output

```json
{
  "algorithm": "leiden",
  "partition": {
    "node_communities": {
      "0": 0, "1": 0, "2": 0, "3": 0,
      "4": 1, "5": 1, "6": 1, "7": 1
    },
    "community_count": 2,
    "community_sizes": { "0": 4, "1": 4 }
  },
  "quality": {
    "modularity_q": 0.3571
  },
  "metadata": {
    "iterations": 3,
    "execution_time_ms": 2
  }
}
```

### Verification

```bash
# Verify connected communities via BFS
cargo test --package communal-algo -- leiden_connected_communities

# Verify determinism (same seed = same result)
cargo run --bin communal -- run test_graph.edgelist \
  --algorithm leiden \
  --output leiden_result2.json \
  --seed 42
diff leiden_result.json leiden_result2.json
# Expected: no output (files identical)
```

---

## Scenario 2: Algorithm Comparison

**Goal**: Verify multiple algorithms produce valid partitions and comparison metrics.

### Steps

```bash
# Compare algorithms
cargo run --bin communal -- compare test_graph.edgelist \
  --algorithms leiden,louvain,infomap \
  --output comparison.json
```

### Expected Output

```json
{
  "algorithms": [
    {
      "name": "leiden",
      "partition": { "community_count": 2 },
      "quality": { "modularity_q": 0.3571 },
      "execution_time_ms": 2
    },
    {
      "name": "louvain",
      "partition": { "community_count": 2 },
      "quality": { "modularity_q": 0.3512 },
      "execution_time_ms": 1
    },
    {
      "name": "infomap",
      "partition": { "community_count": 2 },
      "quality": { "modularity_q": 0.3498 },
      "execution_time_ms": 3
    }
  ],
  "comparison": {
    "leiden_vs_louvain": { "nmi": 0.9523, "ari": 0.9142 },
    "leiden_vs_infomap": { "nmi": 0.9312, "ari": 0.8901 },
    "louvain_vs_infomap": { "nmi": 0.9701, "ari": 0.9432 }
  }
}
```

### Verification

```bash
# Run comparison tests
cargo test --package communal-cli -- compare_algorithms
```

---

## Scenario 3: Incremental Dynamic Updates

**Goal**: Verify edge insertion/deletion updates partition without full recomputation.

### Steps

```bash
# Create initial graph
cat > dynamic_test.edgelist << 'EOF'
0 1
1 2
2 0
3 4
4 5
5 3
EOF

# Run initial detection
cargo run --bin communal -- run dynamic_test.edgelist \
  --algorithm leiden \
  --output initial_partition.json \
  --seed 42

# Insert bridge edge and update incrementally
cargo run --bin communal -- run dynamic_test.edgelist \
  --algorithm leiden \
  --insert-edge 2 3 \
  --output after_insertion.json \
  --seed 42

# Delete bridge edge and update incrementally
cargo run --bin communal -- run dynamic_test.edgelist \
  --algorithm leiden \
  --remove-edge 2 3 \
  --output after_deletion.json \
  --seed 42
```

### Expected Behavior

1. **Initial**: 2 communities `{0,1,2}` and `{3,4,5}`
2. **After insertion**: May merge into 1 community or remain 2 (depends on weight)
3. **After deletion**: Returns to original 2 communities (subtree stability)

### Verification

```bash
# Run dynamic update tests
cargo test --package communal-dynamic -- incremental_updates

# Verify subtree stability
cargo test --package communal-dynamic -- subtree_stability
```

---

## Scenario 4: Stepping Mode (Observability)

**Goal**: Verify algorithm emits step events for inspection.

### Steps

```bash
# Run with stepping enabled
cargo run --bin communal -- run test_graph.edgelist \
  --algorithm leiden \
  --stepping \
  --output stepping_log.json
```

### Expected Output

```json
{
  "steps": [
    {
      "type": "PhaseStart",
      "phase": "LocalMoving",
      "iteration": 0
    },
    {
      "type": "NodeRelocation",
      "node": 3,
      "from": 3,
      "to": 0,
      "delta": 0.042
    },
    {
      "type": "PhaseEnd",
      "phase": "LocalMoving",
      "iteration": 0
    },
    {
      "type": "PhaseStart",
      "phase": "Refinement",
      "iteration": 0
    },
    {
      "type": "ConvergencePlateau",
      "iteration": 2,
      "delta": 0.0000003
    }
  ],
  "final_partition": {
    "community_count": 2
  }
}
```

### Verification

```bash
# Run stepping tests
cargo test --package communal_algo -- stepping_mode

# Verify event count matches phase transitions
cargo test --package communal_algo -- event_coverage
```

---

## Scenario 5: Edge Cases

**Goal**: Verify graceful handling of edge cases.

### Empty Graph

```bash
# Create empty graph
touch empty.edgelist

# Run Leiden
cargo run --bin communal -- run empty.edgelist \
  --algorithm leiden \
  --output empty_result.json
```

**Expected**: Empty partition with quality 0, no error.

### Isolated Nodes

```bash
cat > isolated.edgelist << 'EOF'
# No edges - all nodes isolated
EOF

# Create with node count metadata
echo "# nodes=5" > isolated.edgelist

cargo run --bin communal -- run isolated.edgelist \
  --algorithm leiden \
  --output isolated_result.json
```

**Expected**: 5 communities, each with 1 node.

### Disconnected Components

```bash
cat > disconnected.edgelist << 'EOF'
# Component 1
0 1
1 2
# Component 2 (no connection to component 1)
3 4
4 5
# Component 3 (single node, no edges)
EOF

cargo run --bin communal -- run disconnected.edgelist \
  --algorithm leiden \
  --output disconnected_result.json
```

**Expected**: At least 2 communities (components never merged).

### Verification

```bash
# Run edge case tests
cargo test --package communal_algo -- edge_cases

# Run property-based tests
cargo test --package communal_algo -- proptest
```

---

## Scenario 6: Quality Metrics

**Goal**: Verify quality metrics computation.

### Steps

```bash
# Compute metrics for a partition
cargo run --bin communal -- metrics test_graph.edgelist \
  --partition leiden_result.json \
  --output metrics.json
```

### Expected Output

```json
{
  "modularity_q": 0.3571,
  "cpm": -0.2857,
  "map_equation": 4.2134
}
```

### With Ground Truth

```bash
# Create ground truth
cat > ground_truth.json << 'EOF'
{
  "node_communities": {
    "0": 0, "1": 0, "2": 0, "3": 0,
    "4": 1, "5": 1, "6": 1, "7": 1
  }
}
EOF

# Compute comparison metrics
cargo run --bin communal -- metrics test_graph.edgelist \
  --partition leiden_result.json \
  --ground-truth ground_truth.json \
  --output comparison_metrics.json
```

**Expected**: NMI and ARI close to 1.0 for correct partition.

### Verification

```bash
# Run metrics tests
cargo test --package communal-metrics -- metrics_accuracy
```

---

## Scenario 7: Batch Processing

**Goal**: Verify batch processing with parameter sweep.

### Steps

```bash
# Create batch config
cat > batch_config.toml << 'EOF'
[batch]
files = ["test_graph.edgelist"]
algorithms = ["leiden", "louvain"]

[batch.sweep]
gamma = [0.5, 1.0, 2.0]

[output]
directory = "./batch_results/"
format = "json"
EOF

# Run batch
cargo run --bin communal -- batch --config batch_config.toml
```

### Expected Output

```
batch_results/
├── test_graph_leiden_gamma0.5.json
├── test_graph_leiden_gamma1.0.json
├── test_graph_leiden_gamma2.0.json
├── test_graph_louvain_gamma0.5.json
├── test_graph_louvain_gamma1.0.json
├── test_graph_louvain_gamma2.0.json
└── batch_report.json
```

### Verification

```bash
# Run batch tests
cargo test --package communal-cli -- batch_processing
```

---

## Scenario 8: WASM Build

**Goal**: Verify WASM build succeeds.

### Steps

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Build WASM
cargo build --package communal-wasm --target wasm32-unknown-unknown

# Run WASM tests (if wasm-pack installed)
wasm-pack test --node communal-wasm
```

### Expected Output

```
Compiling communal-wasm v0.1.0
Finished release [optimized] target(s)
```

---

## Scenario 9: TUI Mode

**Goal**: Verify TUI launches and displays algorithm execution.

### Steps

```bash
# Launch TUI with stepping
cargo run --bin communal-tui -- test_graph.edgelist \
  --algorithm leiden \
  --stepping
```

### Expected Behavior

1. Terminal switches to TUI mode
2. Force-directed graph layout displayed
3. Event log shows phase transitions
4. Statistics panel shows quality/iteration
5. Pedagogical explanations describe current state
6. Press 'q' to quit

---

## Running All Validation

```bash
# Run complete test suite
cargo test --all-features

# Run benchmarks (quick mode)
cargo bench --bench quick

# Run property-based tests (extended)
PROPTEST_CASES=1000 cargo test --all-features proptest

# Build documentation
cargo doc --no-deps --all-features
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `linker 'cc' not found` | Install build-essential (Linux) or Xcode (macOS) |
| `wasm32-unknown-unknown not found` | Run `rustup target add wasm32-unknown-unknown` |
| Tests fail on determinism | Check seed is fixed; verify no system-dependent behavior |
| Performance regression | Run `cargo bench` and compare against baseline |
| TUI display issues | Ensure terminal supports Unicode and 256 colors |

---

## Next Steps

After validating all scenarios:

1. Review [research.md](research.md) for technology decisions
2. Review [data-model.md](data-model.md) for entity definitions
3. Review [contracts/](contracts/) for API specifications
4. Run `/speckit-tasks` to generate implementation tasks

# SC-007 Error Testing: Proposed Task Addition

**Date**: 2026-09-04
**Branch**: `001-community-detection`
**Purpose**: Propose a dedicated task for comprehensive error scenario testing to satisfy SC-007 success criteria

---

## 1. SC-007 Requirements Analysis

### 1.1 Full Text (spec.md, lines 382-389)

> **SC-007: Descriptive Typed Errors**
> All public APIs return descriptive typed errors for invalid parameters without panicking.
>
> **Measurement Methodology:**
> - **Test Input**: All public APIs with invalid inputs (malformed files, negative weights, invalid configs, empty inputs)
> - **Procedure**: For each documented error condition, verify: (a) return type is a typed domain error, (b) error variant matches failure mode, (c) error message includes what went wrong, which parameter caused the issue, and expected format or valid range
> - **Pass Threshold**: 100% of induced errors return typed variants with descriptive messages; zero panics across all test cases
> - **Sample Size**: All public API functions × all documented error conditions (minimum 20 error scenarios)

### 1.2 Acceptance Criteria Breakdown

| Criterion | Requirement | Verification Method |
|-----------|-------------|---------------------|
| Typed errors | Return type is a typed domain error (not panic/unwrap) | `assert_matches!` on `Result::Err(variant)` |
| Variant matching | Error variant matches the failure mode | Pattern match on enum variant |
| Descriptive messages | Message includes: what went wrong, which parameter caused it, expected format/valid range | String contains assertions on `error.to_string()` |
| Zero panics | No panics across all test cases | `std::panic::catch_unwind` or proptest with panic detection |
| Coverage | Minimum 20 error scenarios | Count of documented + tested scenarios |

### 1.3 Related Functional Requirements

**FR-035** (typed domain errors):
- Invalid graph input (malformed files, unsupported formats)
- Negative edge weights (when validation enabled)
- Algorithm convergence failure (non-convergence within iteration bound)
- Invalid algorithm configuration

**FR-044** (per-module error enums):
- `GraphError` (graph construction, validation, parsing)
- `AlgorithmError` (configuration, convergence, execution)
- `PartitionError` (query, invalid access)
- `MetricsError` (computation, comparison)
- `CommunalError` unified re-export with `From` conversions

**FR-040** (parser error details):
- Line number where error occurred
- Error type (invalid format, missing field, invalid value)
- Expected format or valid range
- Fail-fast with no silent skipping

---

## 2. Rust Error Testing Best Practices

### 2.1 Testing Specific Error Types

**Pattern 1: `assert_matches!` (Recommended)**
```rust
use std::assert_matches::assert_matches;

#[test]
fn negative_weight_returns_error() {
    let result = CsrGraph::from_edge_list(&[(-1.0, 0, 1)], true);
    assert_matches!(result, Err(GraphError::NegativeWeight { .. }));
}
```

Source: [assert_matches documentation](https://doc.rust-lang.org/std/macro.assert_matches.html)

**Pattern 2: Downcast for dynamic error traits**
```rust
#[test]
fn error_can_be_downcast() {
    let result = GraphBuilder::from malformed_input();
    let err = result.unwrap_err();
    assert!(err.is::<GraphError>());
    let graph_err = err.downcast_ref::<GraphError>().unwrap();
    assert_eq!(graph_err.line_number(), Some(42));
}
```

### 2.2 Verifying No Panics

**Pattern 1: `catch_unwind` for panic detection**
```rust
use std::panic;

#[test]
fn invalid_input_no_panic() {
    let result = panic::catch_unwind(|| {
        let _ = GraphBuilder::from malformed_input();
    });
    assert!(result.is_ok(), "Function panicked instead of returning Result");
}
```

**Pattern 2: `#[should_panic]` absence verification**
- Proptest naturally detects panics as test failures
- Any panic in a proptest block fails the test — verifying panic freedom through property testing

### 2.3 Property-Based Testing for Error Paths (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn never_panics_on_malformed_input(input in any::<Vec<u8>>()) {
        // Property: malformed input never panics, only returns Err
        let result = std::panic::catch_unwind(|| {
            GraphBuilder::from_bytes(&input)
        });
        prop_assert!(result.is_ok());
    }

    #[test]
    fn negative_weights_always_error(weight in any::<f64>().filter(|w| *w < 0.0)) {
        let result = GraphBuilder::with_edge(0, 1, weight);
        prop_assert!(result.is_err());
        prop_assert!(matches!(result, Err(GraphError::NegativeWeight { .. })));
    }
}
```

Source: [proptest-rs/proptest](https://github.com/proptest-rs/proptest), [proptest documentation](https://docs.rs/proptest/latest/proptest/)

### 2.4 Error Message Quality Verification

```rust
#[test]
fn error_message_contains_actionable_info() {
    let err = GraphBuilder::from malformed_input().unwrap_err();
    let msg = err.to_string();

    // Must describe what went wrong
    assert!(!msg.is_empty());
    // Must mention the invalid value or parameter
    assert!(msg.contains("weight") || msg.contains("edge") || msg.contains("node"));
    // Must indicate valid range or expected format
    assert!(
        msg.contains("expected") || msg.contains("valid") || msg.contains("range"),
        "Error message lacks actionable guidance: {}",
        msg
    );
}
```

### 2.5 Testing Patterns Summary

| Pattern | Use Case | Source |
|---------|----------|--------|
| `assert_matches!` | Verify specific error variant | [Rust std](https://doc.rust-lang.org/std/macro.assert_matches.html) |
| `catch_unwind` | Verify no panics | [Rust std panic](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) |
| `proptest` | Property-based error path testing | [proptest-rs](https://github.com/proptest-rs/proptest) |
| `downcast_ref` | Dynamic error trait verification | [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html) |
| String assertions | Error message quality | [thiserror docs](https://docs.rs/thiserror/) |

---

## 3. Proposed New Task Definition

### 3.1 Task: T017a — Comprehensive Error Scenario Tests

```markdown
- [ ] T017a [P] Write comprehensive error scenario tests covering SC-007 in:
  - `crates/communal-core/tests/error_scenarios.rs`
  - `crates/communal-algo/tests/algorithm_error_scenarios.rs`
  - `crates/communal-metrics/tests/metrics_error_scenarios.rs`
  - `crates/communal-dynamic/tests/dynamic_error_scenarios.rs`
```

### 3.2 Full Task Description

**T017a [P] Write comprehensive error scenario tests covering SC-007 (Descriptive Typed Errors)**

**Description**:
Write tests that verify all public APIs return descriptive typed errors for invalid parameters without panicking. This task covers SC-007 acceptance criteria: "100% of induced errors return typed variants with descriptive messages; zero panics across all test cases. Minimum 20 error scenarios."

**Test Files**:
- `crates/communal-core/tests/error_scenarios.rs` — GraphError + PartitionError scenarios
- `crates/communal-algo/tests/algorithm_error_scenarios.rs` — AlgorithmError scenarios
- `crates/communal-metrics/tests/metrics_error_scenarios.rs` — MetricsError scenarios
- `crates/communal-dynamic/tests/dynamic_error_scenarios.rs` — DynamicError scenarios

**Test Patterns to Use**:
1. `assert_matches!` for variant verification
2. `std::panic::catch_unwind` for panic freedom verification
3. `proptest` property-based testing for "never panics on any input" properties
4. String assertions on `Display` impl for message quality

**Acceptance Criteria**:
- [ ] Minimum 20 distinct error scenarios tested (see Section 4 below)
- [ ] Each test verifies: (a) correct variant, (b) descriptive message, (c) no panic
- [ ] All parsers tested with malformed input (missing fields, invalid types, out-of-range values)
- [ ] All config validation tested with invalid parameters
- [ ] Edge case: empty inputs return `Err` not panic
- [ ] Property: `proptest` verifies no panics on arbitrary malformed input
- [ ] Error messages contain: what went wrong + parameter name + valid range/format

**Dependencies**: T017 (domain error types must exist before testing)

**Constitution Alignment**: Principle VI (Test-Driven Development) — tests written first to define expected error behavior before implementation.

---

## 4. Error Scenarios to Cover (28 Scenarios)

### 4.1 GraphError Scenarios (communal-core)

| # | Scenario | Input | Expected Variant |
|---|----------|-------|------------------|
| 1 | Negative edge weight with validation | Edge list with `weight: -1.0` | `GraphError::NegativeWeight { from, to, weight }` |
| 2 | Malformed EdgeList — missing field | `"0 1"` (no weight) | `GraphError::ParseMissingField { line, expected }` |
| 3 | Malformed EdgeList — invalid node ID | `"abc 1 0.5"` | `GraphError::ParseInvalidNode { line, value }` |
| 4 | Malformed EdgeList — invalid weight | `"0 1 abc"` | `GraphError::ParseInvalidWeight { line, value }` |
| 5 | Malformed JSON — missing required field | `{"nodes": 5}` (no edges) | `GraphError::JsonMissingField { field }` |
| 6 | Malformed JSON — invalid type | `{"edges": "not_array"}` | `GraphError::JsonInvalidType { field, expected }` |
| 7 | Malformed GML — syntax error | Missing bracket | `GraphError::GmlSyntaxError { line, context }` |
| 8 | Unsupported file format | `.xyz` format requested | `GraphError::UnsupportedFormat { format }` |
| 9 | Empty file input | Empty string/bytes | `GraphError::EmptyInput` |
| 10 | Edge references non-existent node | Edge `(999, 0, 1.0)` with 5 nodes | `GraphError::InvalidNodeReference { node, max }` |

### 4.2 AlgorithmError Scenarios (communal-algo)

| # | Scenario | Input | Expected Variant |
|---|----------|-------|------------------|
| 11 | Invalid convergence threshold (negative) | `threshold: -0.1` | `AlgorithmError::InvalidThreshold { value, range }` |
| 12 | Invalid resolution parameter (gamma ≤ 0) | `gamma: 0.0` | `AlgorithmError::InvalidResolution { value, range }` |
| 13 | Invalid teleportation rate | `teleportation_rate: 1.5` | `AlgorithmError::InvalidTeleportationRate { value, range }` |
| 14 | Invalid target communities (k = 0) | `target_communities: 0` | `AlgorithmError::InvalidTargetCommunities { value }` |
| 15 | Non-convergence within max iterations | Pathological graph, max_iter=1 | `AlgorithmError::NonConvergence { iterations }` |
| 16 | Invalid LPA update mode combination | Synchronous mode requested | `AlgorithmError::UnsupportedUpdateMode { mode }` |
| 17 | Algorithm receives empty graph with invalid config | Empty graph + k=5 communities | `AlgorithmError::InvalidGraphForAlgorithm { reason }` |

### 4.3 PartitionError Scenarios (communal-core)

| # | Scenario | Input | Expected Variant |
|---|----------|-------|------------------|
| 18 | Query `community_of` with invalid node ID | `community_of(NodeId(999))` on 5-node graph | `PartitionError::InvalidNodeId { node, count }` |
| 19 | Access community by invalid CommunityId | `members(CommunityId(999))` | `PartitionError::InvalidCommunityId { id, count }` |
| 20 | Access hierarchy level beyond valid range | `at_level(100)` on 3-level tree | `PartitionError::InvalidLevel { level, max }` |

### 4.4 MetricsError Scenarios (communal-metrics)

| # | Scenario | Input | Expected Variant |
|---|----------|-------|------------------|
| 21 | Compute modularity on empty partition | Empty partition | `MetricsError::EmptyPartition` |
| 22 | Compare partitions of different sizes | Partition A (5 nodes), Partition B (3 nodes) | `MetricsError::SizeMismatch { expected, actual }` |
| 23 | NMI with invalid ground truth | Ground truth has different node count | `MetricsError::InvalidGroundTruth { reason }` |
| 24 | Numerical overflow in quality computation | Extremely large weights | `MetricsError::NumericalError { context }` |

### 4.5 DynamicError Scenarios (communal-dynamic)

| # | Scenario | Input | Expected Variant |
|---|----------|-------|------------------|
| 25 | Invalid hierarchy level access | `level(100)` on 3-level tree | `DynamicError::InvalidLevel { level, max }` |
| 26 | Community split failure | Forced disconnected subgraph | `DynamicError::SplitFailure { reason }` |
| 27 | Complexity bound exceeded | Mutation affecting > threshold nodes | `DynamicError::ComplexityExceeded` |
| 28 | Mutation on invalid edge | Insert edge `(999, 1000)` | `DynamicError::MutationError(GraphError::InvalidNodeReference)` |

---

## 5. Exact Insertion Point in tasks.md

### 5.1 Location

Insert **T017a** immediately after **T017** in Phase 2 (Foundational), between lines 57 and 58:

```
- [ ] T017 [P] Implement domain error types (GraphError, AlgorithmError, PartitionError) in `crates/communal-core/src/error.rs`
- [ ] T017a [P] Write comprehensive error scenario tests covering SC-007 in: `crates/communal-core/tests/error_scenarios.rs`, `crates/communal-algo/tests/algorithm_error_scenarios.rs`, `crates/communal-metrics/tests/metrics_error_scenarios.rs`, `crates/communal-dynamic/tests/dynamic_error_scenarios.rs`
- [ ] T018 Implement `Partition` struct with query methods in `crates/communal-core/src/partition.rs`
```

### 5.2 Rationale for Placement

1. **Depends on T017**: Error types must be defined before tests can reference their variants
2. **TDD alignment**: Tests written first define expected error behavior before algorithms implement error returns
3. **Foundational phase**: Error testing is infrastructure, not user-story-specific
4. **Early detection**: Placing early ensures error handling is verified as algorithms are built
5. **No story dependencies**: Error testing applies to all user stories, belongs in foundational

### 5.3 Additional Related Tasks

Consider also adding these companion tasks:

```markdown
- [ ] T017b [P] Add proptest-based "never panics" property tests for all public APIs in `crates/communal-core/tests/panic_freedom_tests.rs`
- [ ] T017c [P] Implement error message quality lint asserting all error variants contain actionable guidance in `crates/communal-core/src/error.rs` (compile-time check via doc comment assertions)
```

---

## 6. Implementation Guidance

### 6.1 Test Structure Template

```rust
// crates/communal-core/tests/error_scenarios.rs

use communal_core::{GraphBuilder, GraphError};
use std::assert_matches::assert_matches;

mod graph_errors {
    use super::*;

    #[test]
    fn negative_weight_returns_typed_error() {
        let result = GraphBuilder::from_edge_list(&[(-1.0, 0, 1)], true);
        assert_matches!(result, Err(GraphError::NegativeWeight { .. }));
    }

    #[test]
    fn negative_weight_error_is_descriptive() {
        let result = GraphBuilder::from_edge_list(&[(-1.0, 0, 1)], true);
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("weight"), "Message should mention 'weight': {}", msg);
        assert!(msg.contains("negative") || msg.contains(">= 0"), "Message should indicate valid range: {}", msg);
    }

    #[test]
    fn negative_weight_no_panic() {
        let result = std::panic::catch_unwind(|| {
            let _ = GraphBuilder::from_edge_list(&[(-1.0, 0, 1)], true);
        });
        assert!(result.is_ok(), "Should not panic on negative weight");
    }
}

mod parser_errors {
    use super::*;

    #[test]
    fn malformed_edgelist_returns_error() {
        let result = GraphBuilder::from_edgelist_str("0 1\nabc 2 0.5\n");
        assert_matches!(result, Err(GraphError::ParseInvalidNode { line: 2, .. }));
    }

    #[test]
    fn empty_input_returns_error_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _ = GraphBuilder::from_edgelist_str("");
        });
        assert!(result.is_ok(), "Empty input should not panic");
    }
}
```

### 6.2 Proptest Template

```rust
// crates/communal-core/tests/panic_freedom_tests.rs

use proptest::prelude::*;
use communal_core::GraphBuilder;

proptest! {
    #[test]
    fn graph_builder_never_panics(input in any::<Vec<u8>>()) {
        let result = std::panic::catch_unwind(|| {
            let _ = GraphBuilder::from_bytes(&input);
        });
        prop_assert!(result.is_ok(), "GraphBuilder panicked on input: {:?}", input);
    }

    #[test]
    fn negative_weights_always_error(weight in any::<f64>().filter(|w| *w < 0.0)) {
        let edges = vec![(weight, 0u32, 1u32)];
        let result = GraphBuilder::from_typed_edges(&edges, true);
        prop_assert!(result.is_err(), "Negative weight {} should return Err", weight);
    }
}
```

### 6.3 Verification Checklist

Run these commands to verify SC-007 compliance:

```bash
# Run all error scenario tests
cargo test --package communal-core --test error_scenarios
cargo test --package communal-algo --test algorithm_error_scenarios
cargo test --package communal-metrics --test metrics_error_scenarios
cargo test --package communal-dynamic --test dynamic_error_scenarios

# Run panic freedom property tests
cargo test --package communal-core --test panic_freedom_tests

# Verify no panics in any test (catch_unwind integration)
cargo test --all -- --test-threads=1 2>&1 | grep -i "panic"

# Count error scenario test functions
cargo test --all 2>&1 | grep "test error_scenarios" | wc -l  # Should be >= 20
```

---

## 7. Summary

| Aspect | Recommendation |
|--------|----------------|
| **Task ID** | T017a |
| **Placement** | Phase 2, immediately after T017 (line 57) |
| **Test files** | 4 new test files (core, algo, metrics, dynamic) |
| **Error scenarios** | 28 scenarios across 5 error enums |
| **Test patterns** | `assert_matches!`, `catch_unwind`, `proptest`, string assertions |
| **Dependencies** | T017 (error types must exist first) |
| **Parallel** | Yes [P] — test files are independent |
| **Estimated effort** | Medium — requires understanding each error variant's semantics |

---

## Sources

- [proptest-rs/proptest: Property testing for Rust](https://github.com/proptest-rs/proptest)
- [proptest documentation](https://docs.rs/proptest/latest/proptest/)
- [Rust testing strategies (unit, integration, property)](https://dasroot.net/posts/2026/03/rust-testing-strategies-unit-integration-property-tests/)
- [Testing error variants in Rust](https://github.com/nautilus-cyberneering/testing-in-rust/blob/main/docs/testing-error-variants-in-rust.md)
- [Stack Overflow: How to assert io errors in Rust](https://stackoverflow.com/questions/57234140/how-to-assert-io-errors-in-rust)
- [Stack Overflow: How do you test for a specific Rust error?](https://stackoverflow.com/questions/53124930/how-do-you-test-for-a-specific-rust-error)
- [thiserror documentation](https://docs.rs/thiserror/)
- [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html)
- [Rust RFC Process — Test Plan sections](https://github.com/rust-lang/rfcs)
- [leidenalg Test Suite](https://github.com/vtraag/leidenalg/blob/main/tests/test_VertexPartition.py)

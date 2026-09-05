# Logging Dependencies Research

Research conducted: September 2026
Sources: crates.io API (primary), docs.rs (primary)

---

## 1. tracing-subscriber

| Field | Value |
|-------|-------|
| **Crate** | tracing-subscriber |
| **Latest stable version** | 0.3.23 |
| **Published** | 2026-03-13 |
| **docs.rs URL** | https://docs.rs/tracing-subscriber/latest/tracing_subscriber/ |
| **Repository** | https://github.com/tokio-rs/tracing |
| **Homepage** | https://tokio.rs |
| **License** | MIT |
| **Downloads (total)** | ~587 million |
| **Downloads (recent)** | ~139 million |
| **MSRV** | Rust 1.65.0 |

### Purpose / Role in the Tracing Ecosystem

`tracing-subscriber` is the **official companion crate** for the `tracing` framework. It provides utilities for implementing and composing `tracing` subscribers — the components that receive and process trace data.

Key capabilities:
- **`Layer` trait** — A composable abstraction for building `Subscriber`s. Layers provide modular implementations of specific behaviors and can be composed together to form a complete subscriber.
- **`Filter` trait** — Per-layer filtering, allowing different layers to handle separate subsets of trace data.
- **`fmt` subscriber** — A batteries-included subscriber for printing formatted representations of trace events (requires the `fmt` feature).
- **`EnvFilter`** — Environment-variable-based filtering similar to `env_logger` (requires the `env-filter` feature).
- **`Registry`** — Storage for span data shared by multiple layers.
- **Feature flags** — `std`, `alloc`, `env-filter`, `fmt`, `ansi`, `registry`, `json`, `local-time`, and more.

### Confirmation: Official Companion Crate

**Confirmed.** `tracing-subscriber` is the official companion crate for `tracing`. Evidence:
- It is published from the **same repository** as `tracing`: https://github.com/tokio-rs/tracing
- The crate's homepage is `https://tokio.rs` (the Tokio project, which maintains tracing)
- The crate description explicitly states: *"Utilities for implementing and composing `tracing` subscribers."*
- It is maintained by the same authors as `tracing` (Eliza Weisman, Carl Lerche, Hayden Stainsby — all Tokio contributors)
- The `tracing` crate's own documentation references `tracing-subscriber` as the primary way to set up a subscriber

### Compatibility with tracing 0.1.44

The project depends on `tracing = "0.1.44"`. The `tracing-subscriber` 0.3.x series is the companion crate for `tracing` 0.1.x. They are designed to work together — `tracing-subscriber` depends on `tracing-core` (a shared low-level crate) and is fully compatible with `tracing` 0.1.44.

### Source

- crates.io API: `https://crates.io/api/v1/crates/tracing-subscriber`
- docs.rs: `https://docs.rs/tracing-subscriber/latest/tracing_subscriber/`

---

## 2. rolling-file

| Field | Value |
|-------|-------|
| **Crate** | rolling-file |
| **Latest stable version** | 0.2.0 |
| **Published** | 2023-01-14 |
| **docs.rs URL** | https://docs.rs/rolling-file/latest/rolling_file/ |
| **Repository** | https://github.com/Axcient/rolling-file-rs |
| **License** | MIT/Apache-2.0 |
| **Downloads (total)** | ~6.8 million |
| **MSRV** | Not specified (edition 2018) |

### Purpose / Role for Log Rotation

`rolling-file` is a rolling file appender that provides customizable rolling conditions for log file rotation. It is designed to be used as a backend for logging frameworks (particularly with `tracing` via `tracing_appender::non_blocking::NonBlocking`).

Key capabilities:
- **Date/time-based rolling** — Built-in support for daily, hourly, and per-minute rotation.
- **Size-based rolling** — Rotate when a file exceeds a specified size.
- **Debian-style naming convention** — Rolled files follow the pattern `basename`, `basename.1`, `basename.2`, ..., `basename.N` where N is the maximum number of historical log files to retain.
- **Custom rolling conditions** — The `RollingCondition` trait allows implementing custom rotation logic.
- **Simple API** — `BasicRollingFileAppender::new(path, condition, max_files)` is the primary entry point.

### ⚠️ Maintenance Note

The latest version (0.2.0) was published on **2023-01-14** and there have been no releases since. The crate has only 2 published versions total (0.1.0 and 0.2.0). This may indicate the crate is stable but not actively maintained. For production use, consider evaluating alternatives like `tracing-appender`'s built-in `RollingFileAppender` or `log4rs` if more active maintenance is needed.

### Source

- crates.io API: `https://crates.io/api/v1/crates/rolling-file`
- docs.rs: `https://docs.rs/rolling-file/latest/rolling_file/`
- GitHub: https://github.com/Axcient/rolling-file-rs

---

## Summary

| Crate | Version | docs.rs | Status |
|-------|---------|---------|--------|
| tracing-subscriber | 0.3.23 | https://docs.rs/tracing-subscriber/latest/tracing_subscriber/ | ✅ Actively maintained (latest: 2026-03-13) |
| rolling-file | 0.2.0 | https://docs.rs/rolling-file/latest/rolling_file/ | ⚠️ Stable but not updated since 2023-01-14 |

Both crates are confirmed to exist on crates.io and are compatible with the project's existing `tracing = "0.1.44"` dependency.

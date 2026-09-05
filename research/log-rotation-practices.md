# Log Rotation Best Practices in the Rust `tracing` Ecosystem

> Research compiled from official documentation, crate sources, and production examples.
> Last updated: 2026-01-27

---

## Table of Contents

1. [Overview](#overview)
2. [Recommended Crates](#recommended-crates)
3. [Rotation Strategies](#rotation-strategies)
4. [Implementation with `tracing-appender`](#implementation-with-tracing-appender)
5. [Configuration Options Users Expect](#configuration-options-users-expect)
6. [Production Patterns & Best Practices](#production-patterns--best-practices)
7. [Comparison of Approaches](#comparison-of-approaches)
8. [Citations & References](#citations--references)

---

## Overview

The `tracing` crate is the de facto standard for structured diagnostics in Rust. However, `tracing` itself is a *collector* — it delegates I/O to **subscribers**. For file-based logging with rotation, the ecosystem relies on companion crates, primarily `tracing-appender` (first-party, maintained by the Tokio team) and third-party alternatives like `rolling-file`.

The key abstraction is `tracing-subscriber`'s [`MakeWriter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/writer/trait.MakeWriter.html) trait, which decouples *where* logs go from *how* they are formatted. Any type implementing `std::io::Write` can be plugged in as a log destination. [^makewriter]

[^makewriter]: [MakeWriter trait — tracing-subscriber docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/writer/trait.MakeWriter.html)

---

## Recommended Crates

### 1. `tracing-appender` (First-Party, Recommended)

| Attribute | Value |
|-----------|-------|
| **Maintainer** | Tokio project (`tokio-rs/tracing`) |
| **Version** | 0.2.x |
| **MSRV** | Rust 1.63+ |
| **URL** | <https://docs.rs/tracing-appender> |

This is the **official** companion crate. It provides:

- `RollingFileAppender` — time-based rotation (minutely, hourly, daily, weekly, never)
- `NonBlocking` writer — off-thread, async-friendly I/O via a dedicated logging thread
- `WorkerGuard` — ensures buffered logs are flushed on process exit (including panics)

**Key limitation:** `tracing-appender` does **not** support size-based rotation natively. This is a long-standing open feature request. [^issue1940]

[^issue1940]: [Allow tracing-appender sized log and rotation #1940](https://github.com/tokio-rs/tracing/issues/1940) — Open since Feb 2022, requesting max-size rotation and file count limits.

### 2. `rolling-file` (Third-Party)

| Attribute | Value |
|-----------|-------|
| **Maintainer** | Axcient (`Axcient/rolling-file-rs`) |
| **URL** | <https://docs.rs/rolling-file> |

A rolling file appender that supports **compound conditions** — rotation can be triggered by time *and/or* size simultaneously. Uses Debian-style naming (`log`, `log.1`, `log.2`, ...).

**Advantage over `tracing-appender`:** Supports size-based rotation out of the box.

### 3. `file-rotate` (Third-Party)

| Attribute | Value |
|-----------|-------|
| **URL** | <https://docs.rs/file-rotate> |

A lower-level crate that rotates based on content limits (bytes or lines). Can be used as middleware with any `std::io::Write` pipeline.

### 4. `logroller` (Third-Party)

| Attribute | Value |
|-----------|-------|
| **URL** | <https://lib.rs/crates/logroller> |

A standalone log writer with rotation, not specifically designed for `tracing` but usable with the `log` crate facade.

---

## Rotation Strategies

### Time-Based Rotation

The most common strategy in the `tracing` ecosystem. Files are rotated on a fixed schedule:

| Variant | Naming Pattern | Use Case |
|---------|---------------|----------|
| `Rotation::MINUTELY` | `prefix.log.yyyy-MM-dd-HH-mm` | High-volume debug/temporary logs |
| `Rotation::HOURLY` | `prefix.log.yyyy-MM-dd-HH` | Standard production logging |
| `Rotation::DAILY` | `prefix.log.yyyy-MM-dd` | Long-term archival, low-volume services |
| `Rotation::WEEKLY` | Rotates every Sunday at midnight UTC | Compliance, audit trails |
| `Rotation::NEVER` | `prefix.log` (single file) | Development, containers with external log shipping |

**Source:** [`Rotation` struct docs](https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.Rotation.html) [^rotation]

[^rotation]: [Rotation — tracing-appender docs](https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.Rotation.html)

### Size-Based Rotation

Files are rotated when they exceed a maximum byte size. **Not natively supported** by `tracing-appender` — this is the most requested feature. [^issue1940]

Workarounds:
- Use the `rolling-file` crate with `RollingConditionBasic::new().max_size(N)` [^rolling-file-cond]
- Use `file-rotate` with `ContentLimit::Bytes(N)` [^file-rotate]

[^rolling-file-cond]: [RollingConditionBasic — rolling-file docs](https://docs.rs/rolling-file/latest/rolling_file/struct.RollingConditionBasic.html)
[^file-rotate]: [file-rotate — Docs.rs](https://docs.rs/file-rotate/latest/file_rotate/)

### Compound (Size + Time) Rotation

Rotate when *either* condition is met — e.g., "rotate daily OR when the file exceeds 100 MB." This is the most production-robust strategy because it handles both predictable schedules and traffic spikes.

Supported natively by `rolling-file`:

```rust
use rolling_file::{RollingConditionBasic, RollingFileAppender};

let condition = RollingConditionBasic::new()
    .daily()                    // rotate when date changes
    .max_size(100 * 1024  * 1024);  // OR when file exceeds 100 MB

let appender = RollingFileAppender::new(
    "/var/log/myapp",
    condition,
    10  // keep max 10 historical files
).unwrap();
```

**Source:** [rolling-file examples](https://docs.rs/rolling-file/latest/rolling_file/) [^rolling-file]

[^rolling-file]: [rolling_file — Docs.rs](https://docs.rs/rolling-file/latest/rolling_file/)

---

## Implementation with `tracing-appender`

### Basic: Hourly Rotation, Blocking

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn main() {
    let file_appender = RollingFileAppender::new(
        Rotation::HOURLY,
        "/var/log/myapp",
        "app.log",
    );

    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .init();
}
```

**Source:** [tracing-appender README](https://docs.rs/tracing-appender/latest/tracing_appender/) [^appender-readme]

[^appender-readme]: [tracing_appender — Docs.rs](https://docs.rs/tracing-appender/latest/tracing_appender/)

### Recommended: Non-Blocking + Hourly Rotation

For production async applications (Tokio), the non-blocking writer prevents log I/O from blocking the async runtime:

```rust
use tracing_appender::rolling::Rotation;

fn main() {
    let file_appender = tracing_appender::rolling::hourly("/var/log/myapp", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();

    // _guard MUST remain alive for the duration of the program.
    // It flushes buffered logs on drop (including during panic unwinding).
}
```

**Source:** [tracing-appender non-blocking docs](https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/index.html) [^non-blocking]

[^non-blocking]: [non_blocking — tracing-appender docs](https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/index.html)

### Builder Pattern with File Retention Limit

The `RollingFileAppender::builder()` API provides fine-grained control: [^builder]

[^builder]: [Builder — tracing-appender docs](https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.Builder.html)

```rust
use tracing_appender::rolling::{Rotation, RollingFileAppender};

let appender = RollingFileAppender::builder()
    .rotation(Rotation::DAILY)
    .filename_prefix("myapp.log")
    .filename_suffix("log")
    .max_log_files(30)            // keep last 30 files, delete older
    .latest_symlink("myapp.log.latest")  // symlink to current file
    .build("/var/log/myapp")
    .expect("failed to initialize rolling file appender");
```

### Configuring the Non-Blocking Writer

The `NonBlockingBuilder` allows tuning buffer behavior: [^nonblocking-builder]

[^nonblocking-builder]: [NonBlockingBuilder — tracing-appender docs](https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.NonBlockingBuilder.html)

```rust
use tracing_appender::non_blocking::NonBlockingBuilder;

let file_appender = tracing_appender::rolling::daily("/var/log/myapp", "app.log");

let (non_blocking, _guard) = NonBlockingBuilder::default()
    .buffered_lines_limit(10_000)   // buffer up to 10k lines before dropping/blocking
    .lossy(true)                     // drop logs when buffer full (don't block)
    .thread_name("myapp-log-writer") // custom worker thread name
    .finish(file_appender);
```

| Option | Default | Description |
|--------|---------|-------------|
| `buffered_lines_limit` | `DEFAULT_BUFFERED_LINES_LIMIT` | Max lines queued before backpressure/drop |
| `lossy` | `true` | If `true`, drop logs when full; if `false`, block senders |
| `thread_name` | `"tracing-appender"` | Name for the background worker thread |

---

## Configuration Options Users Expect

Based on production Rust applications and comparison with log rotation in other ecosystems (logrotate, Python's `RotatingFileHandler`, etc.), users typically expect:

| Option | Type | Description | Supported by `tracing-appender` |
|--------|------|-------------|-------------------------------|
| `rotation_frequency` | Enum | How often to rotate (minutely/hourly/daily/weekly/never) | ✅ Yes |
| `max_file_size` | Bytes | Rotate when file exceeds this size | ❌ No (open issue [#1940](https://github.com/tokio-rs/tracing/issues/1940)) |
| `max_files` | Integer | Max number of rotated files to retain | ✅ Yes (`max_log_files`) |
| `max_age` | Duration | Delete files older than this age | ❌ No (must use `max_log_files` as approximation) |
| `compression` | Bool | Gzip rotated files | ❌ No |
| `file_prefix` | String | Prefix for log filenames | ✅ Yes |
| `file_suffix` | String | Suffix/extension for log filenames | ✅ Yes |
| `symlink_latest` | String | Create a symlink pointing to the current log file | ✅ Yes (`latest_symlink`) |
| `non_blocking` | Bool | Use off-thread writer to avoid blocking async runtime | ✅ Yes |
| `buffer_size` | Integer | Number of lines to buffer before backpressure | ✅ Yes (`buffered_lines_limit`) |
| `lossy` | Bool | Drop logs when buffer full vs. block senders | ✅ Yes |

### What's Missing in `tracing-appender`

The most significant gaps compared to mature log rotation systems:

1. **Size-based rotation** — The top-requested feature, still open after 4+ years [^issue1940]
2. **Age-based retention** — No `max_age` option; only count-based (`max_log_files`)
3. **Compression** — No built-in gzip of rotated files
4. **Custom rotation triggers** — No plugin API for custom `Rotation` strategies

For applications that need size-based rotation today, the `rolling-file` crate is the recommended alternative.

---

## Production Patterns & Best Practices

### 1. Always Use Non-Blocking Writers in Async Contexts

Blocking I/O on a Tokio thread starves other tasks. The `NonBlocking` writer spawns a dedicated OS thread for I/O. [^non-blocking]

> **Critical:** The `WorkerGuard` *must* be assigned to a binding that is not `_`, as `_` causes immediate drop. Place it in `main()` or a long-lived scope. [^workerguard]

[^workerguard]: [WorkerGuard — tracing-appender docs](https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html)

### 2. Use the Builder Pattern for Configurability

Hard-coding rotation settings makes the library hard to reuse. Expose configuration via a config struct or environment variables:

```rust
pub struct LogConfig {
    pub rotation: Rotation,
    pub max_files: usize,
    pub log_dir: String,
    pub file_prefix: String,
    pub non_blocking: bool,
    pub buffer_size: usize,
    pub lossy: bool,
}
```

### 3. Set `max_log_files` to Prevent Disk Exhaustion

Without a file count limit, logs grow unbounded. The `Builder::max_log_files(n)` method deletes the oldest matching files when the limit is exceeded. [^builder]

> **Note from docs:** The exact number of retained files can sometimes dip below the maximum. If you need to retain `m` files, specify a max of `m + 1`.

### 4. Use `latest_symlink` for Easy Log Access

The `Builder::latest_symlink("app.log.latest")` creates a symlink that always points to the current log file. This is invaluable for operational tooling that needs to `tail -f` the active log without knowing the timestamp suffix. [^builder]

### 5. Combine with `EnvFilter` for Runtime Log Level Control

```rust
use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_writer(non_blocking)
    .init();
```

This allows setting `RUST_LOG=info` at runtime without recompilation.

### 6. For Size-Based Rotation: Use `rolling-file` with `tracing-appender`

The `rolling-file` crate can be combined with `tracing-appender`'s `NonBlocking` writer: [^rolling-file]

```rust
use rolling_file::{BasicRollingFileAppender, RollingConditionBasic};

let rolling = BasicRollingFileAppender::new(
    "/var/log/myapp",
    RollingConditionBasic::new().daily().max_size(100 * 1024 * 1024),
    30,
).unwrap();

let (non_blocking, _guard) = tracing_appender::non_blocking(rolling);

tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .init();
```

### 7. Graceful Shutdown

Ensure the `WorkerGuard` is dropped during shutdown to flush remaining logs. In a Tokio application:

```rust
#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::daily("/var/log/myapp", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();

    // ... run application ...

    // _guard dropped here on normal exit, flushing all buffered logs
}
```

---

## Comparison of Approaches

| Feature | `tracing-appender` | `rolling-file` | `file-rotate` |
|---------|-------------------|----------------|---------------|
| **Time-based rotation** | ✅ (minutely/hourly/daily/weekly) | ✅ (minutely/hourly/daily) | ❌ |
| **Size-based rotation** | ❌ | ✅ | ✅ |
| **Compound conditions** | ❌ | ✅ (time AND/OR size) | ❌ |
| **File count limit** | ✅ (`max_log_files`) | ✅ | ❌ |
| **File age limit** | ❌ | ❌ | ❌ |
| **Non-blocking I/O** | ✅ (built-in) | ✅ (via `tracing-appender`) | ❌ |
| **Symlink to latest** | ✅ | ❌ | ❌ |
| **Compression** | ❌ | ❌ | ❌ |
| **First-party (Tokio)** | ✅ | ❌ | ❌ |
| **Naming scheme** | `prefix.yyyy-MM-dd-HH` | `base`, `base.1`, `base.2` (Debian-style) | Custom |

### Recommendation

- **For most applications:** Use `tracing-appender` with `Rotation::DAILY` or `Rotation::HOURLY` and `max_log_files`. It's first-party, well-maintained, and integrates seamlessly with the `tracing` ecosystem.
- **When you need size limits:** Use `rolling-file` combined with `tracing-appender`'s `NonBlocking` writer.
- **For simple scripts or containers:** `Rotation::NEVER` with external log shipping (e.g., journald, Docker logging driver) is often sufficient.

---

## Citations & References

| # | Source | URL |
|---|--------|-----|
| 1 | `tracing-subscriber` crate documentation | <https://docs.rs/tracing-subscriber> |
| 2 | `tracing-appender` crate documentation | <https://docs.rs/tracing-appender> |
| 3 | `tracing-appender::rolling` module docs | <https://docs.rs/tracing-appender/latest/tracing_appender/rolling/index.html> |
| 4 | `Rotation` struct — available rotation strategies | <https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.Rotation.html> |
| 5 | `Builder` — RollingFileAppender configuration | <https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.Builder.html> |
| 6 | `NonBlockingBuilder` — buffer/worker tuning | <https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.NonBlockingBuilder.html> |
| 7 | `WorkerGuard` — flush-on-exit semantics | <https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html> |
| 8 | `MakeWriter` trait — tracing-subscriber writer abstraction | <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/writer/trait.MakeWriter.html> |
| 9 | `rolling-file` crate — compound rotation conditions | <https://docs.rs/rolling-file/latest/rolling_file/> |
| 10 | `RollingConditionBasic` — size + time conditions | <https://docs.rs/rolling-file/latest/rolling_file/struct.RollingConditionBasic.html> |
| 11 | GitHub Issue #1940: Size-based rotation feature request | <https://github.com/tokio-rs/tracing/issues/1940> |
| 12 | `file-rotate` crate — content-limit rotation | <https://docs.rs/file-rotate/latest/file_rotate/> |
| 13 | Tokio tracing guide | <https://tokio.rs/tokio/topics/tracing> |
| 14 | Rolling Log Rotation in Rust with tracing-appender (community article) | <https://rust.nicedx.com/tracing-rolling-log-rotation/> |
| 15 | `rolling-file` GitHub repository | <https://github.com/Axcient/rolling-file-rs> |

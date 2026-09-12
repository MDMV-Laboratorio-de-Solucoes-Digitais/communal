# Property-Based Testing Framework Check — CHK024 / SC-006

**Checklist item:** CHK024 (SC-006 framework "Proptest or equivalent")  
**Date:** 2026-09-05  
**Scope:** Research only; no spec modifications made.

## Frameworks reviewed

| Crate | Name | Crate ID | Status / adoption |
|---|---|---|---|
| `proptest` | Proptest (Hypothesis-inspired) | `proptest` | **Most commonly adopted** in modern Rust; 75M+ downloads; composable strategies; strong shrinking; widely used in production crates ([lib.rs](https://lib.rs/crates/proptest), [generalistprogrammer.com](https://generalistprogrammer.com/tutorials/proptest-rust-crate-guide)) |
| `quickcheck` | QuickCheck (Haskell port) | `quickcheck` | Established / older; automatic shrinking; simpler generation model; lower activity vs proptest ([crates.io](https://crates.io/keywords/quickcheck)) |
| `bolero` | Bolero | `bolero` | Newer; fuzzing + property-testing hybrid; `no_std` support; less widespread adoption ([overview from ecosystem surveys](https://www.lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust/)) |

## Brief comparison

- **Proptest** (`proptest`) — composable `Strategy` traits; rich shrinking; closest to Python Hypothesis; best ecosystem fit for this project because of composability and maintenance. Recommended for CHK024.
- **QuickCheck** (`quickcheck`) — minimal API; good for quick checks; weaker strategy composability; older codebase.
- **Bolero** (`bolero`) — blends fuzzing (libFuzzer-style) with property tests; useful if fuzz targets are needed; overkill if only property-based regression checks are required.

## Recommendation for SC-006

Use `proptest` (crate `proptest`) as the "Proptest or equivalent" framework for CHK024. It is the de facto standard for Rust property-based testing today and aligns with the spec's equivalence clause.

## Sources
- [An Introduction To Property-Based Testing In Rust — lpalmieri.com](https://www.lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust/)
- [Proptest — Lib.rs](https://lib.rs/crates/proptest)
- [QuickCheck — crates.io keywords](https://crates.io/keywords/quickcheck)
- [Rust testing libraries — rustfinity.com](https://www.rustfinity.com/blog/rust-testing-libraries)

# StepIterator API Design Research

Research on algorithm stepping/observability iterator APIs in Rust, focusing on the `next()` method contract.

---

## 1. Return Type Patterns for Algorithm Stepping Iterators

### 1.1 `Option<T>` — The Standard Pattern

**Source:** Rust Standard Library `Iterator` trait  
**URL:** https://doc.rust-lang.org/std/iter/trait.Iterator.html

```rust
fn next(&mut self) -> Option<Self::Item>;
```

- `Some(T)` — a value is available
- `None` — iteration is finished (completion signal)

The documentation explicitly states: *"Returns `None` when iteration is finished. Individual iterator implementations may choose to resume iteration, and so calling `next()` again may or may not eventually start returning `Some(Item)` again at some point."*

This is the foundational pattern in Rust. The `Option<T>` return type conflates "no value right now" with "done forever" — the caller cannot distinguish between a temporary lack of output and permanent completion without relying on implementation-specific behavior (e.g., fused iterators).

### 1.2 `Result<Option<T>, E>` — The Fallible Pattern

**Source:** `fallible-iterator` crate — `FallibleIterator` trait  
**URL:** https://docs.rs/fallible-iterator/latest/fallible_iterator/trait.FallibleIterator.html

```rust
fn next(&mut self) -> Result<Option<Self::Item>, Self::Error>;
```

- `Ok(Some(T))` — a value is available
- `Ok(None)` — iteration completed successfully
- `Err(E)` — an error occurred during iteration

This is the canonical solution for distinguishing completion from errors. The documentation states: *"Returns `Ok(None)` when iteration is finished. The behavior of calling this method after a previous call has returned `Ok(None)` or `Err` is implementation defined."*

This pattern is used extensively in database drivers (e.g., `rusqlite`), file format parsers, and any iterator where item computation can fail.

### 1.3 `Poll<Option<T>>` — The Async Pattern

**Source:** `futures` crate — `Stream` trait  
**URL:** https://docs.rs/futures/latest/futures/stream/trait.Stream.html

```rust
fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
```

- `Poll::Pending` — value not ready yet, task will be woken
- `Poll::Ready(Some(T))` — a value is available
- `Poll::Ready(None)` — stream is exhausted

The documentation explains: *"`Poll::Pending` means that this stream's next value is not ready yet. Implementations will ensure that the current task will be notified when the next value may be ready. `Poll::Ready(Some(val))` means that the stream has successfully produced a value. `Poll::Ready(None)` means that the stream has terminated."*

This three-state variant is essential for async contexts where "not yet" is semantically different from "done."

### 1.4 `Option<T>` with Context — The Walker Pattern

**Source:** `petgraph` crate — `Walker` trait  
**URL:** https://docs.rs/petgraph/latest/petgraph/visit/trait.Walker.html

```rust
fn walk_next(&mut self, context: Context) -> Option<Self::Item>;
```

The `Walker` trait passes context (the graph) into each `walk_next` call rather than holding a borrow:

```rust
// From petgraph docs:
while let Some(nx) = dfs.next(&graph) {
    // we can access `graph` mutably here still
    graph[nx] += 1;
}
```

This pattern is specifically designed for graph traversals where holding a borrow would prevent mutation of the graph during traversal. The `Dfs`, `Bfs`, `DfsPostOrder`, and `Topo` types all implement `Walker`.

### 1.5 Custom Enum — The `FoldWhile` Pattern

**Source:** `itertools` crate — `FoldWhile` enum  
**URL:** https://docs.rs/itertools/latest/itertools/enum.FoldWhile.html

```rust
pub enum FoldWhile<T> {
    Continue(T),
    Done(T),
}
```

Used with the `fold_while` method, this allows the closure to signal whether iteration should continue or stop, while still producing a value. This is a more expressive alternative to `Option` when the termination condition is computed inside a fold.

### 1.6 `EitherOrBoth` — The Zipping Pattern

**Source:** `itertools` crate — `EitherOrBoth` enum  
**URL:** https://docs.rs/itertools/latest/itertools/enum.EitherOrBoth.html

```rust
pub enum EitherOrBoth<T, U> {
    Both(T, U),
    Left(T),
    Right(U),
}
```

Used by `zip_longest` to handle iterators of unequal length, where one may be exhausted but the other still has values.

---

## 2. Completion vs Error Distinction

### 2.1 Standard Library: No Error Distinction

The std `Iterator::next() -> Option<T>` has no error channel. Errors must be:
- Encoded in the item type (e.g., `Iterator<Item = Result<T, E>>`)
- Panicked (not recommended)
- Swallowed (lossy)

The `try_fold` and `try_for_each` methods provide a `Try`-based escape hatch:

```rust
fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
where
    F: FnMut(B, Self::Item) -> R,
    R: Try<Output = B>;
```

This allows early termination through the `Try` trait (implemented for `Result`, `Option`, `ControlFlow`, etc.).

### 2.2 `FallibleIterator`: Explicit Error Channel

The `fallible-iterator` crate provides the most explicit separation:

| Return Value | Meaning |
|---|---|
| `Ok(Some(T))` | Value available |
| `Ok(None)` | Successful completion |
| `Err(E)` | Error during iteration |

This is the recommended pattern when:
- The iterator performs I/O or computation that can fail
- The caller needs to distinguish "done" from "broken"
- The iterator is used in a context where `?` propagation is desired

### 2.3 `futures::stream::TryStream`: Error in Item Type

**Source:** `futures` crate — `TryStream` trait

```rust
trait TryStream: Stream {
    type Ok;
    type Error;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Result<Self::Ok, Self::Error>>>;
}
```

Here the return type is `Poll<Option<Result<T, E>>>`, giving four states:
- `Pending` — not ready
- `Ready(None)` — done
- `Ready(Some(Ok(T)))` — value
- `Ready(Some(Err(E)))` — error

### 2.4 `rayon` Parallel Iterator: `Try` Trait for Short-Circuiting

**Source:** `rayon` crate — `ParallelIterator` trait  
**URL:** https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html

Rayon does not have a `next()` method. Instead, it uses terminal operations with the `Try` trait:

```rust
fn try_for_each<OP, R>(self, op: OP) -> R
where
    OP: Fn(Self::Item) -> R + Sync + Send,
    R: Try<Output = ()> + Send;

fn try_reduce<T, OP, ID>(self, identity: ID, op: OP) -> Self::Item
where
    OP: Fn(T, T) -> Self::Item + Sync + Send,
    ID: Fn() -> T + Sync + Send,
    Self::Item: Try<Output = T>;
```

The `Try` trait (from `std::ops`) provides `Output` and `Residual` associated types, allowing `Result`, `Option`, and `ControlFlow` to work as short-circuiting mechanisms in parallel contexts.

---

## 3. Idiomatic Rust Patterns

### 3.1 The `Peekable` Pattern

**Source:** `std::iter::Peekable`

```rust
fn peek(&mut self) -> Option<&Self::Item>;
```

Allows looking at the next value without consuming it. The underlying iterator is still advanced internally on first `peek()` call.

### 3.2 The `scan` Pattern — Stateful Mapping with Early Termination

**Source:** `std::iter::Iterator::scan`

```rust
fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Self, St, F>
where
    F: FnMut(&mut St, Self::Item) -> Option<B>;
```

The closure returns `Some(value)` to yield a value, or `None` to end iteration. This is a powerful pattern for algorithm stepping where internal state determines both the output and whether to continue.

### 3.3 The `coalesce` Pattern — Merging Adjacent Elements

**Source:** `itertools` crate

```rust
fn coalesce<F>(self, f: F) -> Coalesce<Self, F>
where
    F: FnMut(Self::Item, Self::Item)
        -> Result<Self::Item, (Self::Item, Self::Item)>;
```

Uses `Result` to signal whether two adjacent elements should be merged (`Ok(merged)`) or kept separate (`Err((a, b))`). This is an elegant use of `Result` as a two-way decision rather than error/success.

### 3.4 The `map_ok` / `filter_ok` Pattern — Result Transparency

**Source:** `itertools` crate

```rust
fn map_ok<F, T, U, E>(self, f: F) -> MapOk<Self, F>
where
    Self: Iterator<Item = Result<T, E>>,
    F: FnMut(T) -> U;

fn filter_ok<F, T, E>(self, f: F) -> FilterOk<Self, F>
where
    Self: Iterator<Item = Result<T, E>>,
    F: FnMut(&T) -> bool;
```

These adapters "see through" the `Result` layer, applying operations only to `Ok` values while passing `Err` values through unchanged. This avoids the boilerplate of matching on every item.

### 3.5 The `process_results` Pattern — Bulk Result Processing

**Source:** `itertools` crate

```rust
fn process_results<F, T, E, R>(self, processor: F) -> Result<R, E>
where
    Self: Iterator<Item = Result<T, E>>,
    F: FnOnce(ProcessResults<'_, Self, E>) -> R;
```

Collects `Result` items, short-circuiting on the first `Err`, and provides an inner iterator over the `Ok` values to the processor closure.

### 3.6 The `while_some` Pattern — Option as Termination Signal

**Source:** `itertools` crate and `rayon` crate

```rust
// itertools
fn while_some<A>(self) -> WhileSome<Self>
where
    Self: Iterator<Item = Option<A>>;

// rayon
fn while_some<T>(self) -> WhileSome<Self>
where
    Self: ParallelIterator<Item = Option<T>>;
```

Treats `None` as a termination signal, unwrapping `Some` values. Useful when an algorithm produces `Option` values and the first `None` indicates completion.

### 3.7 The `Walker` Pattern — External Context

**Source:** `petgraph` crate

The `Walker` trait separates traversal state from the graph data:

```rust
pub trait Walker<Context> {
    type Item;
    fn walk_next(&mut self, context: Context) -> Option<Self::Item>;
    fn iter(self, context: Context) -> WalkerIter<Self, Context>;
}
```

This allows the graph to be mutated during traversal since the walker doesn't hold a borrow. The `Dfs::next(&mut self, graph: G) -> Option<N>` method is the concrete example.

---

## 4. Common Patterns Across Libraries

| Pattern | Return Type | Completion | Error | Library |
|---|---|---|---|---|
| Standard | `Option<T>` | `None` | N/A | `std::iter::Iterator` |
| Fallible | `Result<Option<T>, E>` | `Ok(None)` | `Err(E)` | `fallible-iterator` |
| Async | `Poll<Option<T>>` | `Ready(None)` | N/A | `futures::Stream` |
| Async+Fallible | `Poll<Option<Result<T,E>>>` | `Ready(None)` | `Ready(Some(Err(E)))` | `futures::TryStream` |
| Walker | `Option<T>` (with context) | `None` | N/A | `petgraph::Walker` |
| Parallel | N/A (uses `Try` trait) | `Try::Output` | `Try::Residual` | `rayon` |
| Stateful | `Option<B>` (from closure) | `None` from closure | N/A | `std::iter::scan` |
| Fold control | `FoldWhile<T>` | `Done(T)` | N/A | `itertools` |

### Key Observations:

1. **`Option<T>` is the universal completion signal** across all synchronous Rust iterator APIs. `None` always means "done."

2. **`Result<Option<T>, E>` is the standard fallible extension**, adding an error channel while preserving the `Ok(None)` completion signal.

3. **The `Try` trait is the unifying abstraction** for short-circuiting in both sequential (`try_fold`, `try_for_each`) and parallel (`rayon`) contexts.

4. **Context-passing (Walker pattern)** is used when the iterator must not hold a borrow on its data source, enabling mutation during traversal.

5. **Custom enums** (`FoldWhile`, `EitherOrBoth`) are used when the binary "continue/stop" or "both/one" distinction needs to carry data.

---

## 5. TUI Integration Considerations

### 5.1 Event Loop Architecture

**Source:** `ratatui` crate documentation  
**URL:** https://ratatui.rs/concepts/event-handling/

Ratatui does not provide its own event catching. The typical pattern with `crossterm` is:

```rust
// Centralized event handling (simplest)
match event::read()? {
    Event::Key(key) => { /* handle key */ }
    Event::Mouse(mouse) => { /* handle mouse */ }
    Event::Resize(w, h) => { /* handle resize */ }
}
```

### 5.2 Integration with Stepping Iterators

For a TUI that observes algorithm stepping, the integration patterns are:

**Pattern A: Iterator as Event Source**

The algorithm iterator produces `Step` events that are rendered:

```rust
while let Some(step) = algorithm.next() {
    // Render current state
    terminal.draw(|frame| render(frame, &step))?;
    
    // Wait for user input (blocking or timeout)
    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(KeyCode::Char(' ')) => continue,  // next step
            Event::Key(KeyCode::Char('q')) => break,      // quit
            _ => {}
        }
    }
}
```

**Pattern B: Channel-Based (for async/threaded algorithms)**

```rust
let (tx, rx) = mpsc::channel();
// Run algorithm in thread, sending steps through channel
std::thread::spawn(move || {
    for step in algorithm {
        tx.send(step).unwrap();
    }
});
// In event loop, non-blocking check for new steps
match rx.try_recv() {
    Ok(step) => current_state = step,
    Err(TryRecvError::Empty) => {},  // no new step yet
    Err(TryRecvError::Disconnected) => break,  // algorithm done
}
```

**Pattern C: Tokio Async Integration**

```rust
// For async event handling with tokio-stream
let mut event_stream = EventStream::new();  // crossterm events as stream
let mut algorithm = algorithm.steps();       // algorithm as stream

loop {
    tokio::select! {
        Some(Ok(event)) = event_stream.next() => {
            match event {
                Event::Key(KeyCode::Char(' ')) => {},
                Event::Key(KeyCode::Char('q')) => break,
                _ => {}
            }
        }
        Some(step) = algorithm.next() => {
            // render step
        }
        else => break,
    }
}
```

### 5.3 Recommended Return Type for TUI Integration

For a stepping iterator that integrates with a TUI event loop:

```rust
pub enum Step<T> {
    /// Algorithm produced a value; rendering should update
    Next(T),
    /// Algorithm completed successfully
    Done,
    /// Algorithm encountered an error
    Error(AlgorithmError),
}
```

This is essentially a named version of `Result<Option<T>, E>`:
- `Step::Next(T)` = `Ok(Some(T))`
- `Step::Done` = `Ok(None)`
- `Step::Error(E)` = `Err(E)`

The named enum is preferred for TUI integration because:
1. **Self-documenting**: `Step::Done` is clearer than `Ok(None)` in a match arm
2. **Extensible**: Can add `Step::Paused`, `Step::Breakpoint`, etc. without changing the type
3. **Match ergonomics**: `match step { Step::Next(t) => ..., Step::Done => ..., Step::Error(e) => ... }` reads better than nested `Result<Option<_>`, _>` matching

---

## 6. Recommended `next()` Return Type

### For a Community Detection Algorithm Stepping Iterator

Based on the research, the recommended approach is:

**Primary recommendation: `Result<Option<Step>, AlgorithmError>`**

This follows the `fallible-iterator` pattern, which is the most widely adopted convention for fallible iteration in the Rust ecosystem. It provides:

- `Ok(Some(step))` — a new step is available for observation
- `Ok(None)` — algorithm completed successfully
- `Err(e)` — algorithm encountered an error and cannot continue

**Alternative: Custom `StepResult` enum** (if named variants improve readability):

```rust
pub enum StepResult<S, E> {
    Step(S),
    Finished,
    Error(E),
}
```

This is isomorphic to `Result<Option<S>, E>` but with domain-specific names.

**For TUI integration specifically**, the custom enum is recommended because:
1. It can be extended with TUI-specific variants (e.g., `StepResult::NeedsInput`)
2. It avoids the cognitive overhead of nested `Result<Option<_>, _>` in match arms
3. It communicates intent more clearly in the TUI event loop

### Summary of Recommendations

| Context | Recommended Return Type |
|---|---|
| Pure computation, no errors | `Option<Step>` |
| Fallible computation | `Result<Option<Step>, Error>` |
| TUI integration | `StepResult<Step, Error>` (named enum) |
| Async/stream context | `Poll<Option<Step>>` or `Poll<Option<Result<Step, Error>>>` |
| Parallel (rayon) | Use `Try` trait with `try_for_each` / `try_fold` |

---

## Sources

- [Rust std Iterator trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [fallible-iterator crate](https://docs.rs/fallible-iterator/latest/fallible_iterator/trait.FallibleIterator.html)
- [rayon ParallelIterator trait](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html)
- [itertools Itertools trait](https://docs.rs/itertools/latest/itertools/trait.Itertools.html)
- [futures Stream trait](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
- [petgraph Dfs struct](https://docs.rs/petgraph/latest/petgraph/visit/struct.Dfs.html)
- [petgraph Walker trait](https://docs.rs/petgraph/latest/petgraph/visit/trait.Walker.html)
- [ratatui Event Handling](https://ratatui.rs/concepts/event-handling/)
- [Tokio Streams tutorial](https://tokio.rs/tokio/tutorial/streams)
- [itertools FoldWhile enum](https://docs.rs/itertools/latest/itertools/enum.FoldWhile.html)
- [itertools EitherOrBoth enum](https://docs.rs/itertools/latest/itertools/enum.EitherOrBoth.html)

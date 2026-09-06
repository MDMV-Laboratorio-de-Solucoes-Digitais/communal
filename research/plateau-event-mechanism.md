# Plateau Event Mechanism Research

**Date:** 2026-02-11
**Context:** Communal framework — community detection library in Rust
**Spec Reference:** FR-009 — algorithm must "emit plateau events without terminating" when modularity improvement drops below a threshold

---

## 1. Summary of `tracing` Capabilities for This Use Case

### 1.1 Structured Events with Fields

The `tracing` crate fully supports structured events with typed fields. The `event!` macro accepts arbitrary key-value pairs:

```rust
event!(
    Level::INFO,
    iteration = 42,
    current_modularity = 0.8534,
    improvement = 0.0003,
    threshold = 0.001,
    plateau_detected = true,
    "plateau reached: improvement {} < threshold {}",
    0.0003, 0.001
);
```

Fields support:
- **Typed values**: integers, floats, booleans, strings, `&str`
- **Debug/Display formatting**: `?expr` for `Debug`, `%expr` for `Display`
- **Nested field names**: `field.subfield = value` (using dot-separated paths)
- **Constant field names**: `{CONSTANT_NAME} = value`
- **Format strings**: implicit `message` field from format args (non-allocating by default)

This means `tracing` can carry all the structured data needed for plateau events: iteration count, current quality metric, improvement delta, threshold value, and community count.

### 1.2 Spans for Algorithm Phases

`tracing` spans represent periods of time with a beginning and end. For community detection:

```rust
let span = info_span!("louvain_phase1", threshold = 0.001);
let _guard = span.enter();
// ... algorithm runs, events emitted within this span context
```

Spans provide temporal context — a subscriber can see that a plateau event occurred during "phase1" of the Louvain algorithm.

### 1.3 Zero-Cost When No Subscriber

`tracing` achieves zero-cost abstractions through the `register_callsite` mechanism:
- Each `event!`/`span!` callsite is registered once with the active subscriber
- If all subscribers return `Interest::never()` for that callsite, the event is never constructed
- When no subscriber is set globally, the macro expands to a minimal check and returns
- The `max_level_hint` allows compile-time level filtering (via `STATIC_MAX_LEVEL` env var or `LevelFilter`)

This satisfies the "zero-cost when observability not needed" requirement.

### 1.4 Subscriber Dispatch is Synchronous

The `Subscriber::event` method is called **synchronously** on the emitting thread. From the `tracing-core` docs:

> `fn event(&self, event: &Event<'_>)` — Records that an `Event` has occurred. This method will be invoked when an Event is constructed by the `Event`'s `dispatch` method. For example, this happens internally when an event macro from `tracing` is called.

There is no internal buffering or async dispatch in the core `tracing` crate. The `event!` macro calls `Event::dispatch()` which calls `dispatcher::get_default(|current| current.event(event))` — a direct synchronous call to the active subscriber.

### 1.5 Composability via `Layer`

The `tracing-subscriber` crate provides the `Layer` trait for composing multiple concerns:

```rust
let subscriber = PlateauLayer::new()
    .and_then(MetricsLayer::new())
    .with_subscriber(FmtSubscriber::new());
```

Each `Layer` implements `on_event`, `on_enter`, `on_exit`, etc., and can be combined. This allows separating plateau detection logic from formatting, metrics export, etc.

---

## 2. How Existing Rust Algorithm Libraries Handle Event Emission

### 2.1 argmin — Callback Trait Pattern (the gold standard)

**argmin** (numerical optimization in pure Rust) uses an `Observe` trait:

```rust
pub trait Observe<I> {
    fn observe_init(&mut self, name: &str, state: &I, kv: &KV) -> Result<(), Error>;
    fn observe_iter(&mut self, state: &I, kv: &KV) -> Result<(), Error>;
    fn observe_final(&mut self, state: &I) -> Result<(), Error>;
}
```

Key design decisions:
- **Generic over state type `I`**: observers are type-aware
- **`ObserverMode` enum**: `Always`, `Never`, `NewBest`, `Every(n)` — controls frequency
- **Multiple observers**: `Observers<I>` is a container that loops over all registered observers
- **Key-value store (`KV`)**: solver-specific metrics passed as a generic store
- **Return `Result`**: observers can signal errors (though not used for control flow)

Usage:
```rust
let res = Executor::new(problem, solver)
    .configure(|config| config.param(init_param).max_iters(2))
    .add_observer(SlogLogger::term_noblock(), ObserverMode::Always)
    .run()?;
```

The `term_noblock()` variant explicitly supports non-blocking output — relevant for real-time UI.

### 2.2 petgraph — No Event Emission

**petgraph** (the most widely-used Rust graph library) does **not** have any built-in event emission, callback mechanism, or observer pattern for algorithms. Algorithms are plain functions:

```rust
pub fn connected_components<G>(g: &G) -> usize
pub fn dijkstra<G, F, K>(g: &G, start: G::NodeId, edge_cost: F) -> HashMap<G::NodeId, K>
```

There is no way to observe iteration-level progress. This is a known limitation — petgraph prioritizes simplicity and performance over observability.

### 2.3 gryf — Builder Pattern, No Iteration Events

**gryf** (a newer graph library) uses a builder pattern for algorithm parameters but does not expose iteration-level callbacks or events. Algorithms are organized by problem type (e.g., `ShortestPaths`) rather than by algorithm name, but observability is not a first-class concern.

### 2.4 ndarray — Functional Iteration, No Events

**ndarray** provides functional iteration methods (`map`, `fold`, `zip`, `foreach`) but no event emission or callback mechanism for algorithm progress. It is a data structure library, not an algorithm framework.

---

## 3. Comparison: Callback Trait vs. Tracing Approaches

### 3.1 Callback Trait Approach

**How it works:** Define a trait like `PlateauObserver` with methods called at specific points in the algorithm.

```rust
trait PlateauObserver {
    fn on_plateau(&mut self, event: &PlateauEvent);
    fn on_iteration(&mut self, event: &IterationEvent);
}
```

| Aspect | Assessment |
|--------|------------|
| **Type safety** | Full compile-time type safety; event data is a struct |
| **Zero-cost** | Yes, via monomorphization (generics) or `#[inline]` |
| **Composability** | Poor — need to manually manage multiple observers or use `Vec<Box<dyn PlateauObserver>>` |
| **Ecosystem integration** | None — custom trait only works within this library |
| **Control flow** | Can return values to influence algorithm (e.g., early termination) |
| **Stepping support** | Natural — callback can block on a channel, wait for user input |
| **Library maintenance** | Must define, document, and maintain the trait |
| **Ergonomics for users** | Must implement a trait; can't compose with other observability tools |

### 3.2 Tracing Approach

**How it works:** Emit `tracing::Event` with structured fields at key points.

```rust
event!(Level::INFO, iteration = 42, improvement = 0.0003, threshold = 0.001, plateau = true);
```

| Aspect | Assessment |
|--------|------------|
| **Type safety** | Weaker — fields are type-erased at the subscriber boundary; no compile-time guarantee all fields present |
| **Zero-cost** | Yes, via `register_callsite` caching and `max_level_hint` |
| **Composability** | Excellent — `Layer` trait allows composing multiple concerns; ecosystem-wide |
| **Ecosystem integration** | Full — works with OpenTelemetry, JSON loggers, `tracing-timing`, `tracing-subscriber`, etc. |
| **Control flow** | None — events are fire-and-forget; cannot influence algorithm |
| **Stepping support** | Indirect — need separate control channel; events alone can't pause the algorithm |
| **Library maintenance** | Minimal — just emit events; subscribers are external |
| **Ergonomics for users** | Users choose their subscriber; no trait to implement |

### 3.3 Hybrid Approach

A third option combines both: use `tracing` for passive observability (logging, metrics, distributed tracing) and a lightweight callback trait for active control (stepping, early termination, interactive debugging).

```rust
// Passive observability — always available
event!(Level::INFO, iteration = 42, plateau = true, improvement = 0.0003);

// Active control — only when stepping engine is connected
if let Some(callback) = &mut self.step_callback {
    callback.on_plateau(&event)?;
}
```

---

## 4. Can `tracing` Support Real-Time TUI Stepping?

### 4.1 Yes, with a Custom Subscriber

Since `Subscriber::event` is called synchronously, a custom subscriber can directly push to a channel or shared state:

```rust
struct SteppingSubscriber {
    sender: Sender<AlgorithmEvent>,
}

impl Subscriber for SteppingSubscriber {
    fn event(&self, event: &Event<'_>) {
        // Parse fields and send to TUI
        let algo_event = parse_event(event);
        self.sender.send(algo_event).ok();
    }
    // ... other required methods
}
```

The TUI can then receive events in real-time via the channel.

### 4.2 But Not for Bidirectional Stepping

The fundamental limitation: `tracing` events are **fire-and-forget**. The algorithm thread continues executing after `event!` returns. For a true stepping engine where the algorithm pauses between iterations waiting for user input, you need bidirectional communication:

- **Algorithm thread** emits event → waits on a "continue" signal
- **UI thread** receives event → renders state → waits for user "step" press → sends "continue" signal

This requires a control channel independent of `tracing`. A callback trait naturally supports this because the callback can block:

```rust
fn on_iteration(&mut self, state: &IterState) -> Result<(), Error> {
    self.send_to_ui(state);
    self.step_receiver.recv()?; // Block until user presses "step"
    Ok(())
}
```

### 4.3 Practical Architecture for TUI Stepping

The recommended architecture:

```
┌─────────────────┐         ┌──────────────────┐
│  Algorithm       │         │  TUI Thread       │
│  Thread          │         │                   │
│                  │  event  │  ┌──────────────┐ │
│  loop {          │────────▶│  │ Event        │ │
│    compute();    │  channel│  │ Receiver     │ │
│    event!(...)   │         │  └──────┬───────┘ │
│    // optional:  │         │         │         │
│    // wait for   │◀────────│  step   │         │
│    // step signal│  channel│  press  │         │
│  }               │         │         ▼         │
└─────────────────┘         │  ┌──────────────┐ │
                            │  │ Render       │ │
                            │  │ State        │ │
                            │  └──────────────┘ │
                            └──────────────────┘
```

- `tracing` handles the event channel (algorithm → UI)
- A separate `oneshot` or `mpsc` channel handles step control (UI → algorithm)
- The algorithm checks for step control only when a stepping callback is connected (zero-cost otherwise)

---

## 5. Recommendation

### 5.1 Primary Recommendation: Hybrid Approach

Use **`tracing` for event emission** (satisfying the constitution's pure-logging mandate) **plus a minimal callback trait for stepping control** (satisfying the TUI requirement).

**Rationale:**

1. **`tracing` satisfies the constitution**: Pure logging via `tracing`, no stdout/print in the library, zero-cost when no subscriber is active.

2. **`tracing` satisfies FR-009 observability**: Plateau events with structured fields (iteration, quality, threshold, improvement) are trivially expressible via `event!`.

3. **`tracing` enables ecosystem integration**: Users can plug in OpenTelemetry, JSON loggers, metrics backends, or custom subscribers without library changes.

4. **A callback trait enables stepping**: The TUI stepping engine needs bidirectional control that `tracing` cannot provide. A minimal `SteppingCallback` trait fills this gap.

5. **Zero-cost is preserved**: The callback is `Option<Box<dyn SteppingCallback>>` — when `None` (the common case), the check is a single branch that predicts not-taken. Combined with `tracing`'s callsite caching, the non-interactive path has negligible overhead.

### 5.2 Implementation Sketch

```rust
// In the algorithm module — emit plateau events via tracing
fn detect_plateau(iteration: u64, improvement: f64, threshold: f64, quality: f64) {
    event!(
        Level::INFO,
        iteration = iteration,
        improvement = improvement,
        threshold = threshold,
        quality = quality,
        plateau = improvement < threshold,
        "plateau detected at iteration {} (improvement {} < threshold {})",
        iteration, improvement, threshold
    );
}

// Optional stepping callback — only for interactive use
pub trait SteppingCallback {
    fn on_plateau(&mut self, event: &PlateauEvent) -> Result<(), StepError>;
    fn on_iteration(&mut self, event: &IterEvent) -> Result<(), StepError>;
}

pub struct LouvainAlgorithm {
    // ... algorithm state
    step_callback: Option<Box<dyn SteppingCallback>>,
}

impl LouvainAlgorithm {
    pub fn set_step_callback(&mut self, callback: Box<dyn SteppingCallback>) {
        self.step_callback = Some(callback);
    }

    fn run_iteration(&mut self) -> Result<(), Error> {
        // ... compute improvement, update communities ...

        // Emit tracing event (always, zero-cost when no subscriber)
        detect_plateau(self.iteration, self.improvement, self.threshold, self.quality);

        // Optional stepping callback (zero-cost when None)
        if let Some(cb) = &mut self.step_callback {
            let event = PlateauEvent { /* ... */ };
            cb.on_plateau(&event)?;  // Can block for stepping
        }

        Ok(())
    }
}
```

### 5.3 Why Not Pure Callback?

A pure callback approach (like argmin's `Observe`) would:
- Require users to implement a trait even for simple logging
- Not integrate with the broader observability ecosystem
- Force the library to maintain observer infrastructure
- Make it harder to compose multiple concerns (logging + metrics + tracing)

### 5.4 Why Not Pure Tracing?

A pure tracing approach would:
- Be unable to support bidirectional stepping (events are fire-and-forget)
- Require a separate control channel anyway for the TUI
- Lose compile-time type safety on event data
- Make it impossible for the algorithm to react to observer feedback

---

## 6. Citations

### Tracing Crate Documentation

- [tracing — Docs.rs](https://docs.rs/tracing/latest/tracing/) — Core crate documentation; structured events, fields, spans, subscribers
- [tracing::Subscriber — Docs.rs](https://docs.rs/tracing/latest/tracing/trait.Subscriber.html) — Subscriber trait with `event`, `enabled`, `register_callsite` methods
- [tracing_subscriber::layer::Layer — Docs.rs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html) — Layer trait for composing subscribers
- [tracing::subscriber::set_global_default — Docs.rs](https://docs.rs/tracing/latest/tracing/subscriber/fn.set_global_default.html) — Global subscriber installation
- [tracing::dispatcher — Docs.rs](https://docs.rs/tracing/latest/tracing/dispatcher/index.html) — Dispatch mechanism (synchronous, no buffering)

### Algorithm Library Patterns

- [argmin — GitHub](https://github.com/argmin-rs/argmin) — Numerical optimization library with `Observe` trait
- [argmin::core::observers — Docs.rs](https://docs.rs/argmin/latest/argmin/core/observers/index.html) — Observer module with `ObserverMode` enum
- [argmin::core::observers::Observe — Docs.rs](https://docs.rs/argmin/latest/argmin/core/observers/trait.Observe.html) — The `Observe` trait definition with `observe_init`, `observe_iter`, `observe_final`
- [petgraph — GitHub](https://github.com/petgraph/petgraph) — Graph library with no event emission mechanism
- [gryf — Introducing gryf](https://pnevyk.github.io/posts/introducing-gryf/) — Newer graph library, builder pattern, no iteration events

### Ecosystem and Observability

- [tracing-subscriber — Docs.rs](https://docs.rs/tracing-subscriber) — Subscriber implementations and utilities
- [How to Use Structured Logging in Rust with tracing](https://www.rustfaq.org/en/how-to-use-structured-logging-in-rust-with-tracing/) — Structured logging patterns
- [Structured fields and spans in Rust tracing | Caroline Morton](https://www.carolinemorton.co.uk/blog/rust-tracing-structured-fields-and-spans/) — Field and span usage patterns
- [How to Structure Logs Properly in Rust with tracing and OpenTelemetry](https://oneuptime.com/blog/post/2026-01-07-rust-tracing-structured-logs/view) — Production-grade structured logging with OpenTelemetry integration

### Community Detection Context

- [Splines/fast-louvain — GitHub](https://github.com/Splines/fast-louvain) — Rust Louvain implementation (no event emission)
- [rust-igraph — Pure-Rust Graph Algorithms](https://totoro-jam.github.io/rust-igraph/) — Community detection library
- [Louvain method — Wikipedia](https://en.wikipedia.org/wiki/Louvain_method) — Algorithm background

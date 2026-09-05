# Rust API Design Patterns: Callback Traits, Observer Patterns, and Event Emission

> Research compiled for the communal project — community detection algorithm library.
> Focus: idiomatic Rust patterns for step-by-step algorithm introspection via callbacks/events.

---

## 1. Callback Trait Design

### 1.1 The Idiomatic Signature: `&self` vs `&mut self`

The dominant pattern across the Rust ecosystem for observer/callback traits is **`&self` (immutable receiver)**, not `&mut self`. This is critical for composability.

**Primary evidence — `tracing-subscriber` Layer trait (v0.1.x):**

```rust
// Source: tracing-subscriber/src/layer/mod.rs
pub trait Layer<S>
where
    S: Subscriber,
    Self: 'static,
{
    // ALL methods take &self — NOT &mut self:
    fn on_event(&self, event: &Event, ctx: Context<'_, S>) { ... }
    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) { ... }
    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) { ... }
    fn on_close(&self, id: Id, ctx: Context<'_, S>) { ... }
    fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<'_, S>) { ... }
    fn enabled(&self, metadata: &Metadata, ctx: Context<'_, S>) -> bool { ... }
    fn on_layer(&mut self, subscriber: &mut S) { ... }  // ONLY mutable one
}
```

Source: [tracing-subscriber Layer trait](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html)

**Key insight:** `on_layer` is the only `&mut self` method, used exclusively for late initialization when attaching to a subscriber. All event notification methods use `&self`.

**Primary evidence — `log` crate Log trait (v0.4.x):**

```rust
// Source: log/src/lib.rs
pub trait Log: Sync + Send {
    fn enabled(&self, metadata: &Metadata) -> bool;  // &self
    fn log(&self, record: &Record);                   // &self
    fn flush(&self);                                  // &self
}
```

Source: [log crate Log trait](https://docs.rs/log/latest/log/trait.Log.html)

**Primary evidence — `tracing-core` Subscriber trait (v0.1.x):**

```rust
// Source: tracing-core/src/subscriber.rs
pub trait Subscriber: 'static {
    fn enabled(&self, metadata: &Metadata) -> bool;   // &self
    fn event(&self, event: &Event);                    // &self
    fn enter(&self, span: &Id);                        // &self
    fn exit(&self, span: &Id);                         // &self
    fn new_span(&self, span: &Attributes) -> Id;       // &self
}
```

Source: [tracing-core Subscriber trait](https://docs.rs/tracing/0.1.44/tracing/trait.Subscriber.html)

### 1.2 Why `&self` Dominates

The reason is **composition via `&dyn Trait`**. When all methods take `&self`, the trait object `&dyn Layer<S>` or `&dyn Log` can be shared across threads and composed into `Vec<Box<dyn Layer<S>>>` without interior mutability. The `tracing-subscriber` crate explicitly supports `Vec<L>` as a `Layer` implementation:

```rust
// tracing-subscriber implements Layer for Vec<L> where L: Layer<S>
impl<S, L> Layer<S> for Vec<L>
where
    L: Layer<S>,
    S: Subscriber,
{ /* iterates and calls each layer's &self methods */ }
```

Source: [tracing-subscriber Layer for Vec](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html#impl-Layer%3CS%3E-for-Vec%3CL%3E)

### 1.3 When `&mut self` Appears

`&mut self` is used only for **lifecycle/initialization** methods, not event notification:

- `Layer::on_layer(&mut self, subscriber: &mut S)` — called once during attachment
- `Iterator::next(&mut self)` — the fundamental advancement operation (by necessity)

### 1.4 Thread Safety Bounds: `Send + Sync`

**Universal requirement:** Observer traits in Rust are virtually always bounded by `Send + Sync + 'static`.

| Trait | Bounds | Source |
|-------|--------|--------|
| `log::Log` | `Sync + Send` | [docs.rs/log](https://docs.rs/log/latest/log/trait.Log.html) |
| `tracing::Subscriber` | `'static` (implements `Send + Sync` via object) | [docs.rs/tracing](https://docs.rs/tracing/0.1.44/tracing/trait.Subscriber.html) |
| `tracing_subscriber::Layer` | `Self: 'static` + `Send + Sync` for boxed | [docs.rs/tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html) |
| `bevy::Event` | `Send + Sync + 'static` | [docs.rs/bevy](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html) |

The `bevy::Event` trait is explicit:

```rust
pub trait Event: Sized + Send + Sync + 'static {
    type Trigger<'a>: Trigger<Self>;
}
```

Source: [bevy Event trait](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html)

### 1.5 Recommendation for `StepCallback`

Based on the evidence, the idiomatic signature is:

```rust
pub trait StepCallback: Send + Sync + 'static {
    fn on_event(&self, event: &StepEvent);
}
```

Use `&self` (not `&mut self`) for the event handler. This enables:
- `Arc<dyn StepCallback>` sharing across threads
- `Vec<Box<dyn StepCallback>>` composition
- `Box<dyn StepCallback + Send + Sync>` type erasure

If mutable state is needed internally, the implementor should use interior mutability (`Mutex`, `RefCell`, `AtomicUsize`).

---

## 2. Observer Registration Patterns

### 2.1 The `tracing` Pattern: Layer Stack with `on_layer`

`tracing` uses a **builder + stack** pattern. Observers (Layers) are registered via composition:

```rust
// Each layer wraps the previous one
let subscriber = Layer1::new()
    .and_then(Layer2::new())
    .and_then(Layer3::new())
    .with_subscriber(InnerSubscriber::new());
```

The `on_layer(&mut self, subscriber: &mut S)` method is called during attachment for late initialization. The `register_callsite(&self, metadata: &'static Metadata) -> Interest` method is called once per callsite for static filtering.

Source: [tracing-subscriber Layer](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html)

**Deregistration:** Layers are removed by dropping the `Layered<>` handle or by using `tracing_subscriber::reload::Layer` for runtime reconfiguration.

### 2.2 The `log` Pattern: Global Singleton

`log` uses a **global static** observer pattern:

```rust
pub trait Log: Sync + Send {
    fn enabled(&self, metadata: &Metadata) -> bool;
    fn log(&self, record: &Record);
    fn flush(&self);
}

// Registration is global:
pub fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError>
// Or with boxing:
pub fn set_boxed_logger(logger: Box<dyn Log + Send + Sync>) -> Result<(), SetLoggerError>
```

Source: [log crate set_logger](https://docs.rs/log/latest/log/fn.set_logger.html)

**Deregistration:** Not supported — the global logger lives for the program's lifetime. This is a deliberate design choice for a lightweight logging facade.

### 2.3 The `winit` Pattern: Closure-Based Event Loop

`winit` uses a **closure-based** observer pattern where the event handler is a closure passed to `EventLoop::run_app`:

```rust
// Source: winit/src/event_loop.rs
pub fn run_app(&mut self, app: Box<dyn ApplicationHandler<T>>) -> Result<(), ...>
```

The `ApplicationHandler` trait (v0.30+) defines lifecycle methods:

```rust
trait ApplicationHandler<T> {
    fn new_events(&mut self, cause: StartCause);
    fn window_event(&mut self, window_id: WindowId, event: WindowEvent);
    fn device_event(&mut self, device_id: DeviceId, event: DeviceEvent);
    fn user_event(&mut self, event: T);
    fn suspended(&mut self);
    fn resumed(&mut self);
    fn about_to_wait(&mut self);
    fn memory_warning(&mut self);
}
```

Source: [winit ApplicationHandler](https://docs.rs/winit/latest/winit/event_loop/trait.ApplicationHandler.html)

**Deregistration:** The handler is consumed by `run_app`; to "deregister," you drop the event loop.

### 2.4 The `bevy` Pattern: Entity-Triggered Observers

Bevy (v0.14+) uses a **trigger-based** observer pattern:

```rust
// Define an event:
#[derive(Event)]
struct Speak { message: String }

// Register an observer:
world.add_observer(|speak: On<Speak>| {
    println!("{}", speak.message);
});

// Trigger the event:
world.trigger(Speak { message: "Hello!".to_string() });
```

Source: [bevy Event trait](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html)

**Deregistration:** Observers are entities; despawning the observer entity removes it.

### 2.5 Observer Lifecycle Contract

| Library | Registration | Deregistration | Lifetime |
|---------|-------------|----------------|----------|
| `tracing` | `and_then()` / `with_subscriber()` | Drop `Layered<>` or `reload::Layer` | `'static` |
| `log` | `set_logger()` | Not supported | `'static` (global) |
| `winit` | `run_app(handler)` | Drop `EventLoop` | Consumed by run |
| `bevy` | `world.add_observer()` | Despawn entity | Entity lifetime |
| `tokio` | `Handle::spawn()` | `JoinHandle::abort()` | Task lifetime |

**Key contract:** All Rust observer patterns require `'static` lifetime for the observer. The observer cannot borrow data from its registration scope — it must own its state or use `Arc`.

---

## 3. Event Emission Ordering Guarantees

### 3.1 `tracing-subscriber`: Strict FIFO Layer Ordering

The `Layered<L, S, impl Layer<S>>` type guarantees **strict sequential ordering**:

> "The returned `Layer` will call the methods on this `Layer` and then those of the new `Layer`, before calling the methods on the subscriber it wraps."

Source: [tracing-subscriber Layer::and_then](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html#method.and_then)

For `Vec<L>`, the ordering is **insertion-order FIFO** — layers are notified in the order they appear in the vector.

### 3.2 `winit`: Phase-Based Event Ordering

`winit` provides **per-phase ordering** via the `Event<T>` enum:

```rust
pub enum Event<T: 'static> {
    NewEvents(StartCause),      // 1. Batch start
    WindowEvent { window_id, event },  // 2. Window events
    DeviceEvent { device_id, event },  // 3. Device events
    UserEvent(T),               // 4. User events
    Suspended,                  // 5. App lifecycle
    Resumed,                    // 6. App lifecycle
    AboutToWait,                // 7. Batch end (idle)
    LoopExiting,                // 8. Loop shutdown
    MemoryWarning,              // 9. Memory pressure
}
```

Source: [winit Event enum](https://docs.rs/winit/latest/winit/event/enum.Event.html)

**Ordering guarantee:** Events within a single `run_app` iteration follow the phase order above. `NewEvents` always precedes other events; `AboutToWait` always follows them. But there is **no ordering guarantee between windows** — window events from different windows may be interleaved.

### 3.3 `bevy`: Trigger-Defined Ordering

Bevy's `Event` trait has an associated `Trigger` type that explicitly controls ordering:

> "Every `Event` has an associated `Trigger` implementation... which defines which observers will run, what data will be passed to them, and **the order they will be run in**."

Source: [bevy Event trait](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html)

The `GlobalTrigger` runs all observers; `EntityTrigger` runs entity-specific observers; `PropagateEntityTrigger` runs with propagation.

### 3.4 `petgraph`: Traversal-Defined Ordering

For algorithm libraries, `petgraph` provides **traversal-ordering** guarantees:

- `Dfs` (depth-first search): nodes emitted in **preorder** (when first discovered)
- `Bfs` (breadth-first search): nodes emitted in **level-order** (FIFO by distance)

Source: [petgraph Dfs](https://docs.rs/petgraph/latest/petgraph/visit/struct.Dfs.html), [petgraph Bfs](https://docs.rs/petgraph/latest/petgraph/visit/struct.Bfs.html)

### 3.5 Summary of Ordering Guarantees

| Library | Ordering | Guarantee Level |
|---------|----------|-----------------|
| `tracing-subscriber` | FIFO (insertion order) | Strict, documented |
| `winit` | Phase-based | Per-iteration, documented |
| `bevy` | Trigger-defined | Per-event-type, documented |
| `petgraph` | Traversal-defined | Algorithmic, documented |
| `log` | None (single logger) | N/A |

---

## 4. Iterator + Callback Hybrid Interfaces

### 4.1 The `petgraph` Pattern: `Walker` Trait + `iter()` Adapter

`petgraph` provides the canonical example of a library exposing **both** iterator and callback interfaces for the same underlying traversal:

```rust
// Callback-style: manual stepping
let mut dfs = Dfs::new(&graph, start_node);
while let Some(node) = dfs.next(&graph) {
    // Can mutate graph between steps!
    graph[node] += 1;
}

// Iterator-style: via Walker trait
impl<G> Walker<G> for Dfs<G::NodeId, G::Map>
where
    G: IntoNeighbors + Visitable,
{
    type Item = G::NodeId;
    fn walk_next(&mut self, context: G) -> Option<Self::Item> { ... }
    fn iter(self, context: G) -> WalkerIter<Self, G> { ... }
}
```

Source: [petgraph Dfs](https://docs.rs/petgraph/latest/petgraph/visit/struct.Dfs.html), [petgraph Walker trait](https://docs.rs/petgraph/latest/petgraph/visit/trait.Walker.html)

**Pause/resume semantics:** The `Walker` trait's `walk_next(&mut self, context: G)` method is the fundamental primitive. The iterator adapter `WalkerIter` wraps it. Between calls to `next()`, the caller has full access to the graph (and any mutable state), enabling pause/resume.

### 4.2 The `walkdir` Pattern: `IntoIter` with `skip_current_dir`

`walkdir` provides an iterator interface with **control flow** for pause/resume:

```rust
let mut it = WalkDir::new("foo").into_iter();
loop {
    let entry = match it.next() {
        None => break,
        Some(Err(err)) => panic!("ERROR: {}", err),
        Some(Ok(entry)) => entry,
    };
    if is_hidden(&entry) {
        if entry.file_type().is_dir() {
            it.skip_current_dir();  // Pause/skip semantics
        }
        continue;
    }
}
```

Source: [walkdir IntoIter](https://docs.rs/walkdir/latest/walkdir/struct.IntoIter.html)

**Pause/resume semantics:** `skip_current_dir(&mut self)` provides explicit control. The iterator can be held between `next()` calls, allowing the caller to inspect state and decide whether to continue.

### 4.3 The Standard Library Pattern: `Iterator::try_fold` and `try_for_each`

Rust's standard library provides `try_fold` for callback-style early termination:

```rust
// Callback with early termination:
let result = iterator.try_fold(initial_state, |acc, item| {
    if should_stop(&item) {
        Err(ControlFlow::Break(result))
    } else {
        Ok(ControlFlow::Continue(new_acc))
    }
});
```

Source: [std::iter::Iterator::try_fold](https://doc.rust-lang.org/stable/core/iter/traits/iterator/trait.Iterator.html#method.try_fold)

### 4.4 The `tracing` Pattern: `on_event` as Callback + `Layered` as Composition

`tracing` uses a callback-style `on_event` but composes observers via the `Layer` trait. The equivalent of "pause/resume" is the `enabled` method:

```rust
fn enabled(&self, metadata: &Metadata, ctx: Context<'_, S>) -> bool {
    // Return false to skip this event (pause filtering)
    should_log(metadata)
}
```

### 4.5 Recommendation for Hybrid Interface

For a community detection library, the `petgraph` pattern is the most relevant:

```rust
// Callback-style: step-by-step with full control
pub struct CommunityDetection {
    // internal state
}

impl CommunityDetection {
    pub fn step(&mut self) -> Option<StepEvent> {
        // Advance one step, return event
    }

    pub fn run(&mut self) {
        while let Some(event) = self.step() {
            // Default handler
        }
    }
}

// Iterator-style: via IntoIterator or a dedicated iterator
impl IntoIterator for CommunityDetection {
    type Item = StepEvent;
    type IntoIter = CommunityDetectionIter;
    fn into_iter(self) -> Self::IntoIter { ... }
}
```

**Pause/resume in iterator case:** The iterator holds `&mut` access to the algorithm state. Between `next()` calls, the caller can inspect the algorithm's current state via shared references (if the algorithm exposes progress metrics), then call `next()` to resume.

---

## 5. StepEvent Enum Design

### 5.1 The `crossterm` Pattern: Struct Variants with Typed Payloads

`crossterm` (v0.28+) provides the canonical example of an event enum with associated data:

```rust
pub enum Event {
    FocusGained,
    FocusLost,
    Key(KeyEvent),        // Struct variant with named type
    Mouse(MouseEvent),    // Struct variant with named type
    Paste(String),        // Tuple variant with inline type
    Resize(u16, u16),     // Tuple variant with inline types
}
```

Source: [crossterm Event enum](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html)

**Key design choices:**
- Unit variants for simple signals (`FocusGained`, `FocusLost`)
- Struct variants wrapping dedicated types (`Key(KeyEvent)`, `Mouse(MouseEvent)`)
- Tuple variants for simple inline data (`Resize(u16, u16)`)
- Convenience methods: `is_key_press()`, `as_key_event()`, `as_key_press_event()`

The `KeyEvent` and `MouseEvent` are separate structs with named fields:

```rust
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
    pub state: KeyEventState,
}
```

### 5.2 The `winit` Pattern: Struct Variants with Named Fields

`winit` (v0.30+) uses struct variants with explicit named fields:

```rust
pub enum Event<T: 'static> {
    NewEvents(StartCause),
    WindowEvent {
        window_id: WindowId,
        event: WindowEvent,
    },
    DeviceEvent {
        device_id: DeviceId,
        event: DeviceEvent,
    },
    UserEvent(T),
    Suspended,
    Resumed,
    AboutToWait,
    LoopExiting,
    MemoryWarning,
}
```

Source: [winit Event enum](https://docs.rs/winit/latest/winit/event/enum.Event.html)

**Key design choices:**
- Named fields in struct variants for clarity
- Generic `UserEvent(T)` for extensibility
- Lifecycle events as unit variants
- Nested enums (`WindowEvent`, `DeviceEvent`) for domain-specific events

### 5.3 The `bevy` Pattern: Derive Macro + Struct Events

Bevy (v0.14+) takes a different approach: **events are structs, not enum variants**:

```rust
#[derive(Event)]
struct Speak {
    message: String,
}

// For enum-style events, bevy_enum_event provides:
#[derive(EnumEvent)]
enum MyEvent {
    Start,
    Stop { reason: String },
}
// Generates: mod my_event { struct Start; struct Stop { reason: String } }
```

Source: [bevy Event trait](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html), [bevy_enum_event](https://docs.rs/bevy_enum_event/latest/bevy_enum_event/index.html)

**Key design choices:**
- Events are structs (one type per event)
- Derive macro for boilerplate
- Enum events generate a module with one struct per variant
- Each variant becomes a separate event type

### 5.4 The `tokio` Pattern: `Async` Event Channels

`tokio` uses **channel-based** events rather than enum dispatch:

```rust
// tokio::sync::mpsc::Sender<T> and Receiver<T>
// Events are sent as values through channels
let (tx, mut rx) = mpsc::channel::<StepEvent>(100);
tx.send(StepEvent::Iteration { n: 42 }).await;
```

Source: [tokio mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)

### 5.5 Recommendation for `StepEvent`

Based on the evidence, the idiomatic design combines `crossterm` and `winit` patterns:

```rust
/// Events emitted during community detection algorithm execution.
#[derive(Debug, Clone, PartialEq)]
pub enum StepEvent {
    /// Algorithm has started initialization.
    Started,
    /// A new iteration/step has begun.
    Iteration {
        step_number: usize,
        current_modularity: f64,
    },
    /// A node was moved to a different community.
    NodeMoved {
        node_id: NodeId,
        from_community: CommunityId,
        to_community: CommunityityId,
        gain: f64,
    },
    /// A community was merged into another.
    CommunitiesMerged {
        source: CommunityId,
        target: CommunityId,
        delta_modularity: f64,
    },
    /// The algorithm has converged.
    Converged {
        total_steps: usize,
        final_modularity: f64,
    },
    /// The algorithm was stopped before convergence.
    Stopped {
        reason: StopReason,
    },
}
```

**Design principles derived from evidence:**
1. **Unit variants** for simple lifecycle signals (`Started`)
2. **Struct variants with named fields** for events with multiple data fields (`Iteration`, `NodeMoved`)
3. **Derive `Debug`, `Clone`, `PartialEq`** — all standard for event types
4. **No generic parameter** on the enum itself (unlike `winit::Event<T>`) — keep it concrete
5. **Convenience methods** following `crossterm` pattern: `is_converged()`, `as_iteration()`

---

## 6. Synthesis: Recommended API Shape

Combining all findings, the recommended API for a community detection library with step-by-step introspection:

```rust
// === Event Type ===
#[derive(Debug, Clone, PartialEq)]
pub enum StepEvent {
    Started,
    Iteration { step_number: usize, current_modularity: f64 },
    NodeMoved { node_id: NodeId, from: CommunityId, to: CommunityId, gain: f64 },
    Converged { total_steps: usize, final_modularity: f64 },
}

// === Callback Trait (following tracing/log pattern) ===
pub trait StepCallback: Send + Sync + 'static {
    fn on_event(&self, event: &StepEvent);
}

// === Algorithm with Registration (following tracing pattern) ===
pub struct CommunityDetection<C: StepCallback> {
    callback: C,
    // ... internal state
}

impl<C: StepCallback> CommunityDetection<C> {
    pub fn new(callback: C) -> Self { ... }
    pub fn run(&mut self) -> Result<DetectionResult, DetectionError> { ... }
    pub fn step(&mut self) -> Option<StepEvent> { ... }
}

// === Iterator Adapter (following petgraph pattern) ===
pub struct CommunityDetectionIter<C: StepCallback> {
    algorithm: CommunityDetection<C>,
}

impl<C: StepCallback> Iterator for CommunityDetectionIter<C> {
    type Item = StepEvent;
    fn next(&mut self) -> Option<Self::Item> {
        self.algorithm.step()
    }
}
```

---

## Sources

### Official Rust Documentation
- [std::ops::FnMut](https://doc.rust-lang.org/stable/core/ops/trait.FnMut.html) — The mutable closure trait
- [std::ops::Fn](https://doc.rust-lang.org/stable/core/ops/trait.Fn.html) — The immutable closure trait
- [std::ops::FnOnce](https://doc.rust-lang.org/stable/core/ops/trait.FnOnce.html) — The by-value closure trait
- [std::iter::Iterator::try_fold](https://doc.rust-lang.org/stable/core/iter/traits/iterator/trait.Iterator.html#method.try_fold) — Early-termination fold

### Library Source Code & Documentation
- [tracing-subscriber Layer trait](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html) — Composable observer pattern with `&self` methods
- [tracing-core Subscriber trait](https://docs.rs/tracing/0.1.44/tracing/trait.Subscriber.html) — Fundamental subscriber interface
- [log crate Log trait](https://docs.rs/log/latest/log/trait.Log.html) — Global logger trait with `Send + Sync` bounds
- [crossterm Event enum](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html) — Event enum with struct variants
- [winit Event enum](https://docs.rs/winit/latest/winit/event/enum.Event.html) — Phase-based event enum with named fields
- [bevy Event trait](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html) — Trigger-based event system
- [petgraph Dfs](https://docs.rs/petgraph/latest/petgraph/visit/struct.Dfs.html) — Iterator + callback hybrid traversal
- [petgraph Bfs](https://docs.rs/petgraph/latest/petgraph/visit/struct.Bfs.html) — Breadth-first traversal
- [petgraph Walker trait](https://docs.rs/petgraph/latest/petgraph/visit/trait.Walker.html) — Step-by-step walker abstraction
- [petgraph Visitable trait](https://docs.rs/petgraph/latest/petgraph/visit/trait.Visitable.html) — Visit-map tracking
- [walkdir IntoIter](https://docs.rs/walkdir/latest/walkdir/struct.IntoIter.html) — Iterator with `skip_current_dir` control
- [ratatui Event Handling](https://ratatui.rs/concepts/event-handling/) — TUI event handling patterns
- [bevy_enum_event](https://docs.rs/bevy_enum_event/latest/bevy_enum_event/index.html) — Enum-to-struct event generation

### Articles & References
- [Idiomatic Callbacks in Rust (Telex)](https://telex-tui.github.io/blog/rust-patterns-closure-traits.html) — Fn vs FnMut vs FnOnce guidance
- [Rust Design Patterns (rust-unofficial)](https://rust-unofficial.github.io/patterns/) — Pattern catalogue
- [Observer in Rust (refactoring.guru)](https://refactoring.guru/design-patterns/observer/rust/example) — Observer pattern example
- [Bevy Events Cheat Book](https://bevy-cheatbook.github.io/programming/events.html) — Bevy event system explained

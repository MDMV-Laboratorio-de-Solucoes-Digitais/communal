use communal_core::step::StepEvent;
use std::fmt;

/// A listener callback type for [`EventEmitter`].
pub type StepListener = Box<dyn Fn(&StepEvent) + Send>;

/// Emits events to registered observer listeners.
///
/// `EventEmitter` maintains a list of closures that are each invoked whenever
/// [`emit`](Self::emit) is called. Listeners can be added via
/// [`subscribe`](Self::subscribe).
#[derive(Default)]
pub struct EventEmitter {
    listeners: Vec<StepListener>,
}

impl fmt::Debug for EventEmitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventEmitter")
            .field("listener_count", &self.listeners.len())
            .finish()
    }
}

impl EventEmitter {
    /// Creates a new, empty event emitter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a listener closure that will be called on every emitted event.
    pub fn subscribe(&mut self, listener: StepListener) {
        self.listeners.push(listener);
    }

    /// Emits an event to all registered listeners.
    pub fn emit(&self, event: &StepEvent) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}

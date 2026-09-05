use communal_core::step::StepEvent;

/// Resumable iterator for synchronous stepping through algorithm execution.
///
/// Wraps a pre-computed list of [`StepEvent`]s and yields them one at a time,
/// allowing callers to pause, inspect, and resume algorithm event streams
/// without re-execution.
#[derive(Debug)]
pub struct StepIterator {
    events: Vec<StepEvent>,
    position: usize,
}

impl StepIterator {
    /// Creates a new step iterator from a list of events.
    pub fn new(events: Vec<StepEvent>) -> Self {
        Self { events, position: 0 }
    }

    /// Returns the current position within the event stream.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns `true` if there are more events to consume.
    pub fn has_next(&self) -> bool {
        self.position < self.events.len()
    }
}

impl Iterator for StepIterator {
    type Item = StepEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.events.len() {
            let event = self.events[self.position].clone();
            self.position += 1;
            Some(event)
        } else {
            None
        }
    }
}

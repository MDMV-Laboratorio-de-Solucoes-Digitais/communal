use communal_core::step::StepEvent;

/// Observer callback interface for asynchronous event handling.
///
/// Implementors of this trait can be registered with an event source to
/// receive [`StepEvent`] notifications as the algorithm progresses.
///
/// The trait is `Send` so callbacks can be invoked across thread boundaries.
pub trait StepCallback: Send {
    /// Called when a step event is emitted.
    fn on_event(&mut self, event: &StepEvent);
}

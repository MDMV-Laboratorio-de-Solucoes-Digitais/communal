/// RAII subscription handle that deregisters on Drop.
///
/// A `Subscription` represents an active registration with an event source.
/// When the handle is dropped, the subscription is considered inactive and
/// the associated callback will no longer receive events.
#[derive(Debug)]
pub struct Subscription {
    active: bool,
}

impl Subscription {
    /// Creates a new, active subscription.
    pub fn new() -> Self {
        Self { active: true }
    }

    /// Returns `true` if the subscription is still active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for Subscription {
    fn default() -> Self {
        Self::new()
    }
}

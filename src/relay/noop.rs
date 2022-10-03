use crate::{EventBase, Relay};

/// A [`Relay`] that won't do anything with events
#[derive(Debug, Default, Clone, Copy)]
pub struct Noop;

impl Noop {
    /// Creates an instance of [`Noop`] [`Relay`]
    pub fn new() -> Self {
        Self
    }
}

impl Relay for Noop {
    fn transport(&self, _event_base: EventBase, _event: bytes::Bytes) {}
}

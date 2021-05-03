use super::Relay;

/// A [`Relay`] that will print events to standard output
#[derive(Debug, Default, Clone, Copy)]
pub struct Noop;

impl Noop {
    /// Creates an instance of [`Noop`] [`Relay`]
    pub fn new() -> Self {
        Self
    }
}

impl Relay for Noop {
    fn transport(&self, event: bytes::Bytes) -> crate::Result<()> {
        println!("event: {:?}", event);

        Ok(())
    }
}

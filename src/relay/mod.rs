//! [`Relay`] is an abstraction on where events will be sent to

mod http;
mod noop;
mod udp;

pub use self::http::*;
pub use self::noop::*;
pub use self::udp::*;

/// Trait for event transportation
pub trait Relay {
    /// Accepts serialized event as bytes that should be sent to a certain protocol, such as:
    /// - HTTP
    /// - TCP
    /// - UDP
    fn transport(&self, event_base: crate::EventBase, event: bytes::Bytes) -> crate::Result<()>;
}

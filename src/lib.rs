#![doc = include_str!("../README.md")]
#![deny(
    warnings,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    deprecated,
    unconditional_recursion,
    unknown_lints,
    unreachable_code,
    unused_mut,
    clippy::wildcard_imports
)]

mod http_relay;
mod udp_relay;

pub use self::http_relay::*;
pub use self::udp_relay::*;

use serde::Serialize;
use std::time::SystemTime;
use std::{
    hint::spin_loop,
    sync::atomic::{AtomicUsize, Ordering},
};

static mut RELAY: &dyn Relay = &NopRelay;
static STATE: AtomicUsize = AtomicUsize::new(0);

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

fn set_relay_inner<F>(make_relay: F) -> Result<(), SetRelayError>
where
    F: FnOnce() -> &'static dyn Relay,
{
    let old_state = match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            unsafe {
                RELAY = make_relay();
            }
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            while STATE.load(Ordering::SeqCst) == INITIALIZING {
                spin_loop();
            }
            Err(SetRelayError(()))
        }
        _ => Err(SetRelayError(())),
    }
}

fn relay() -> &'static dyn Relay {
    if STATE.load(Ordering::SeqCst) != INITIALIZED {
        static NOP: NopRelay = NopRelay;
        &NOP
    } else {
        unsafe { RELAY }
    }
}

/// Sets the global [Relay]
pub fn set_relay<T: 'static + Relay>(relay: T) -> Result<(), SetRelayError> {
    set_relay_inner(|| Box::leak(Box::new(relay)))
}

/// The type returned by [`set_relay`] if [`set_relay`] has already been called.
pub struct SetRelayError(());

impl std::fmt::Debug for SetRelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetRelayError").finish()
    }
}

impl std::fmt::Display for SetRelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "attempted to set relay after the relay was already initialized".fmt(f)
    }
}

impl std::error::Error for SetRelayError {}

/// Tracks event
#[tracing::instrument(skip(debug_pin, tracking_data))]
pub fn track<T>(event: &'static str, portal: &'static str, debug_pin: Option<i32>, tracking_data: T)
where
    T: Serialize,
{
    #[derive(Serialize)]
    struct EventWithTrackingData<'a, T>
    where
        T: Serialize,
    {
        #[serde(flatten)]
        metadata: &'a Metadata,

        #[serde(flatten)]
        tracking_data: T,
    }

    let metadata = Metadata {
        event,
        portal,
        time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_millis(),
        debug_pin,
    };

    let event_with_tracking_data = {
        EventWithTrackingData {
            metadata: &metadata,
            tracking_data,
        }
    };

    match serde_json::to_vec(&event_with_tracking_data) {
        Ok(bytes) => relay().transport(metadata, bytes),
        Err(error) => {
            tracing::error!(%error, "Could not serialize event")
        }
    }
}

/// Event metadata
#[derive(Debug, Serialize)]
pub struct Metadata {
    event: &'static str,

    portal: &'static str,

    time: u128,

    debug_pin: Option<i32>,
}

impl Metadata {
    /// Event name
    pub fn event(&self) -> &'static str {
        self.event
    }

    /// Portal it's happening in
    pub fn portal(&self) -> &'static str {
        self.portal
    }

    /// Current time in milliseconds since unix epoch
    pub fn time(&self) -> u128 {
        self.time
    }

    /// Debug pin
    pub fn debug_pin(&self) -> Option<i32> {
        self.debug_pin
    }
}

/// An abstraction for event transmission over the wire
pub trait Relay {
    /// Accepts event, serialized in JSON, in a form of bytes.
    ///
    /// Use these bytes to send the event over the wire using protocols, such as:
    /// - HTTP
    /// - TCP
    /// - UDP
    fn transport(&self, metadata: Metadata, serialized_event: Vec<u8>);
}

struct NopRelay;

impl Relay for NopRelay {
    fn transport(&self, _: Metadata, _: Vec<u8>) {}
}

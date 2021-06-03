//! A Rust port of <https://github.com/vinted/event-tracker>
//!
//! In your application executable:
//! ```
//! use vinted_event_tracker::*;
//! use serde::Serialize;
//!
//! #[tokio::main]
//! async fn main() {
//!     let addr = "0.0.0.0:5005".parse().expect("valid addr");
//!
//!     let udp_relay = Udp::new(addr);
//!
//!     let _ = set_relay(udp_relay);
//!
//!     track_events(1_000)
//! }
//!
//! fn track_events(iterations: i32) {
//!     #[derive(Debug, Serialize)]
//!     struct SearchEvent {
//!         iteration: i32,
//!     }
//!
//!     for iteration in 1..iterations {
//!         let event = Event::new("event", "portal", SearchEvent { iteration });
//!
//!         let _ = track(event);
//!     }
//! }
//! ```
//! In your library code, create an event structure and use it for tracking
//! ```
//! # use vinted_event_tracker::*;
//! # use vinted_event_tracker::relay::*;
//! # use serde::Serialize;
//! #[derive(Debug, Serialize)]
//! struct SearchEvent {
//!     total: i32,
//!     timed_out: bool,
//!     query: String,
//! }
//!
//! let search_event = SearchEvent {
//!     total: 123,
//!     timed_out: false,
//!     query: "shoes".to_string(),
//! };
//!
//! let event = Event::new("event", "portal", search_event);
//!
//! let _ = vinted_event_tracker::track(event);
//! ```
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
    unused_mut
)]

#[macro_use]
extern crate tracing;

mod error;
mod event;
pub mod relay;

pub use self::error::*;
pub use self::event::*;
pub use self::relay::*;

use std::{
    hint::spin_loop,
    sync::atomic::{AtomicUsize, Ordering},
};

static mut RELAY: &dyn Relay = &Noop;
static STATE: AtomicUsize = AtomicUsize::new(0);

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

/// Result type alias with consistent error type
type Result<T> = std::result::Result<T, Error>;

fn set_relay_inner<F>(make_relay: F) -> Result<()>
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
            Err(Error::RelayAlreadyInitialized)
        }
        _ => Err(Error::RelayAlreadyInitialized),
    }
}

fn relay() -> &'static dyn Relay {
    if STATE.load(Ordering::SeqCst) != INITIALIZED {
        static NOP: Noop = Noop;
        &NOP
    } else {
        unsafe { RELAY }
    }
}

/// Initializes [`Relay`] for the whole application
pub fn set_relay<T: 'static + Relay>(relay: T) -> Result<()> {
    set_relay_inner(|| Box::leak(Box::new(relay)))
}

/// Tracks the actual event
pub fn track<T>(event: Event<T>) -> Result<()>
where
    T: std::fmt::Debug + serde::Serialize,
{
    match serde_json::to_vec(&event) {
        Ok(buf) => relay().transport(event.base, bytes::Bytes::from(buf)),
        Err(error) => {
            error!(%error, "Couldn't serialize event");

            Err(error.into())
        }
    }
}

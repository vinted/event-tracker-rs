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

use reqwest::{header, Client, Url};
use serde::Serialize;
use std::{
    hint::spin_loop,
    io,
    net::{SocketAddr, ToSocketAddrs},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::net::UdpSocket;

static mut RELAY: &dyn Relay = &NopRelay;
static STATE: AtomicUsize = AtomicUsize::new(0);

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

/// Initializes [`Relay`] for the whole application
pub fn set_relay<T: 'static + Relay>(relay: T) -> Result<(), SetRelayError> {
    set_relay_inner(|| Box::leak(Box::new(relay)))
}

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

/// The type returned by [`set_relay`] if [`set_relay`] has already been called.
#[derive(Debug)]
pub struct SetRelayError(());

impl std::fmt::Display for SetRelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "attempted to set relay after the relay was already initialized".fmt(f)
    }
}

impl std::error::Error for SetRelayError {}

// ====== Event ======

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Event to track
#[derive(Debug, Serialize)]
pub struct Event {
    event: &'static str,

    portal: &'static str,

    time: u128,

    debug_pin: Option<i32>,
}

impl Event {
    /// Creates an instance of [`Event`]
    pub fn track<T>(
        event: &'static str,
        portal: &'static str,
        debug_pin: Option<i32>,
        tracking_data: T,
    ) -> Result<(), BoxError>
    where
        T: std::fmt::Debug + Serialize,
    {
        #[derive(Serialize)]
        struct SerializedEvent<'a, T> {
            #[serde(flatten)]
            base: &'a Event,

            #[serde(flatten)]
            tracking_data: T,
        }

        let event = Self {
            event,
            portal,
            time: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX_EPOCH")
                .as_millis(),
            debug_pin,
        };

        let event_to_serialize = SerializedEvent {
            base: &event,
            tracking_data,
        };

        let serialized_event = serde_json::to_vec(&event_to_serialize)?;

        relay().transport(event, serialized_event)
    }
}

// ====== Relay ======

/// Trait for event transportation
pub trait Relay {
    /// Accepts serialized event and serialized bytes to be sent to a certain protocol, such as:
    /// - HTTP
    /// - HTTPS
    /// - TCP
    /// - UDP
    /// - Kafka
    fn transport(&self, event: Event, bytes: Vec<u8>) -> Result<(), BoxError>;
}

#[derive(Debug, Default, Clone, Copy)]
struct NopRelay;

impl Relay for NopRelay {
    fn transport(&self, _: Event, _: Vec<u8>) -> Result<(), BoxError> {
        Ok(())
    }
}

/// Transports events to a UDP backend
#[derive(Debug)]
pub struct UdpRelay {
    udp_socket: Arc<UdpSocket>,
}

impl UdpRelay {
    /// [UdpRelay] will bind to the given remote_addr
    pub async fn bind<S>(remote_addrs: S) -> Result<Self, io::Error>
    where
        S: ToSocketAddrs,
    {
        let mut remote_addrs = remote_addrs.to_socket_addrs()?;
        let remote_addr = remote_addrs
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no remote addresses"))?;

        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()
        .expect("valid socket address");

        let udp_socket = UdpSocket::bind(local_addr).await?;

        udp_socket.connect(&remote_addr).await?;

        Ok(Self {
            udp_socket: Arc::new(udp_socket),
        })
    }
}

impl Relay for UdpRelay {
    fn transport(&self, _: Event, bytes: Vec<u8>) -> Result<(), BoxError> {
        let udp_socket = self.udp_socket.clone();

        let _ = tokio::spawn(async move { udp_socket.send(&bytes).await });

        Ok(())
    }
}

/// Transports events to a HTTP backend
#[derive(Debug, Clone)]
pub struct HttpRelay {
    client: Client,
    url: Url,
}

impl HttpRelay {
    /// Creates an instance of [`HttpRelay`]
    pub fn new(url: Url) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }
}

impl Relay for HttpRelay {
    fn transport(&self, event: Event, bytes: Vec<u8>) -> Result<(), BoxError> {
        let mut request = self
            .client
            .post(self.url.clone())
            .body(bytes)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Local-Time", event.time.to_string())
            .header("X-Platform", "web")
            .header("X-Portal", event.portal);

        if let Some(debug_pin) = event.debug_pin {
            request = request.header("X-Debug-Pin", debug_pin);
        }

        let _ = tokio::spawn(async move {
            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    tracing::error!(%error, "Couldn't send data to HTTP relay");
                    return;
                }
            };

            if response.status().is_client_error() || response.status().is_server_error() {
                let status_code = response.status().as_u16();

                let error = match response.text().await {
                    Ok(error) => error,
                    Err(error) => error.to_string(),
                };

                tracing::error!(%status_code, %error, "Couldn't complete HTTP request successfully");
            }
        });

        Ok(())
    }
}

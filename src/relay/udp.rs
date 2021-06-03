use super::Relay;
use crate::EventBase;
use bytes::Bytes;
use futures_channel::mpsc::{self, Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::{net::UdpSocket, time};
use tokio_util::{codec::BytesCodec, udp::UdpFramed};

const DEFAULT_BUFFER: usize = 512;
const DEFAULT_TIMEOUT: u32 = 10_000;

/// A [`Relay`] that will print events to UDP listener
#[derive(Debug, Clone)]
pub struct Udp {
    sender: Sender<Bytes>,
}

impl Udp {
    /// Creates an instance of [`Udp`] [`Relay`]
    pub fn new(addr: SocketAddr) -> Self {
        let (sender, receiver) = mpsc::channel::<Bytes>(DEFAULT_BUFFER);

        background_task(addr, receiver);

        Self { sender }
    }
}

impl Relay for Udp {
    fn transport(&self, _event_base: EventBase, event: Bytes) -> crate::Result<()> {
        if let Err(ref error) = self.sender.clone().try_send(event) {
            tracing::error!(%error, "Couldn't send data to UDP relay");
        }

        Ok(())
    }
}

fn background_task(addr: SocketAddr, mut receiver: Receiver<Bytes>) {
    let task = Box::pin(async move {
        // Reconnection loop
        loop {
            handle_udp_connection(addr, &mut receiver).await;

            // Sleep before re-attempting
            time::sleep(time::Duration::from_millis(DEFAULT_TIMEOUT as u64)).await;
        }
    });

    let _ = tokio::spawn(task);
}

async fn handle_udp_connection(addr: SocketAddr, receiver: &mut Receiver<Bytes>) {
    let bind_addr = if addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };

    let udp_socket = match UdpSocket::bind(bind_addr).await {
        Ok(ok) => ok,
        Err(ref error) => {
            tracing::error!(%error, "Couldn't bind to UDP socket");

            return;
        }
    };

    let udp_stream = UdpFramed::new(udp_socket, BytesCodec::new());

    let (mut sink, _) = udp_stream.split();

    while let Some(bytes) = receiver.next().await {
        if let Err(ref error) = sink.send((bytes, addr)).await {
            tracing::error!(%error, "Couldn't send data to UDP relay");

            break;
        };
    }
}

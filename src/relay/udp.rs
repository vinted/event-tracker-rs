use crate::{Error, EventBase, Relay};
use bytes::Bytes;
use futures_channel::mpsc::{self, Receiver, Sender};
use futures_util::StreamExt;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::UdpSocket;

/// UDP request timeout
pub const DEFAULT_TIMEOUT: u32 = 10_000;

/// Default UDP socket mpsc channel buffer
pub const DEFAULT_BUFFER: usize = 10_000;

/// A [`Relay`] that will print events to UDP listener
#[derive(Debug, Clone)]
pub struct Udp {
    sender: Sender<Bytes>,
}

impl Udp {
    /// [Udp] relay will bind to the given remote_addr
    pub async fn bind<S>(remote_addrs: S) -> Result<Self, Error>
    where
        S: ToSocketAddrs,
    {
        let mut remote_addrs = remote_addrs.to_socket_addrs()?;
        let remote_addr = remote_addrs.next().ok_or(Error::NoRemoteAddr)?;

        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()?;

        let udp_socket = UdpSocket::bind(local_addr).await?;

        udp_socket.connect(&remote_addr).await?;

        let (sender, receiver) = mpsc::channel(DEFAULT_BUFFER);

        Self::background_task(udp_socket, receiver);

        Ok(Self { sender })
    }

    fn background_task(udp_socket: UdpSocket, mut receiver: Receiver<Bytes>) {
        let _ = tokio::spawn(Box::pin(async move {
            Self::send(udp_socket, &mut receiver).await;
        }));
    }

    async fn send(udp_socket: UdpSocket, receiver: &mut Receiver<Bytes>) {
        while let Some(bytes) = receiver.next().await {
            if let Err(ref error) = udp_socket.send(&bytes).await {
                tracing::error!(%error, "Couldn't send data to UDP relay");
            };
        }
    }
}

impl Relay for Udp {
    fn transport(&self, _event_base: EventBase, event: Bytes) -> Result<(), Error> {
        if let Err(ref error) = self.sender.clone().try_send(event) {
            tracing::error!(%error, "Couldn't send data to UDP relay");
        }

        Ok(())
    }
}

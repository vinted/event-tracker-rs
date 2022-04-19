use crate::*;
use bytes::Bytes;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

/// A [`Relay`] that will print events to UDP listener
#[derive(Debug, Clone)]
pub struct Udp {
    udp_socket: Arc<UdpSocket>,
}

impl Udp {
    /// [Udp] relay will bind to the given remote_addr
    pub async fn bind(remote_addr: SocketAddr) -> Result<Self, Error> {
        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()?;

        let udp_socket = UdpSocket::bind(local_addr).await?;

        udp_socket.connect(&remote_addr).await?;

        Ok(Self {
            udp_socket: Arc::new(udp_socket),
        })
    }

    async fn send(udp_socket: Arc<UdpSocket>, event: Bytes) {
        if let Err(error) = udp_socket.send(&event).await {
            tracing::error!(%error, "Couldn't send data to UDP relay");
        }
    }
}

impl Relay for Udp {
    fn transport(&self, _event_base: EventBase, event: Bytes) -> Result<(), Error> {
        let udp_socket = self.udp_socket.clone();
        let _ = tokio::spawn(Self::send(udp_socket, event));

        Ok(())
    }
}

use crate::{Metadata, Relay};
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};
use tokio::net::UdpSocket;

/// A [`Relay`] that will print events to UDP listener
#[derive(Debug)]
pub struct UdpRelay {
    udp_socket: Arc<UdpSocket>,
}

impl UdpRelay {
    /// [UdpRelay] will bind to the given remote_addr
    pub async fn new<S>(remote_addrs: S) -> Result<Self, io::Error>
    where
        S: ToSocketAddrs,
    {
        let mut remote_addrs = remote_addrs.to_socket_addrs()?;
        let remote_addr = remote_addrs
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no remote address found"))?;

        let local_addr = match remote_addr.is_ipv4() {
            true => Self::local_addr_v4(),
            false => Self::local_addr_v6(),
        };

        let udp_socket = UdpSocket::bind(local_addr).await?;

        udp_socket.connect(&remote_addr).await?;

        Ok(Self {
            udp_socket: Arc::new(udp_socket),
        })
    }

    fn local_addr_v4() -> SocketAddr {
        "0.0.0.0:0".parse().unwrap()
    }

    fn local_addr_v6() -> SocketAddr {
        "[::]:0".parse().unwrap()
    }
}

impl Relay for UdpRelay {
    fn transport(&self, _: Metadata, serialized_event: Vec<u8>) {
        let udp_socket = self.udp_socket.clone();

        let result = tokio::spawn(async move { udp_socket.send(&serialized_event).await });

        drop(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_local_addr_correctly() {
        let _ = UdpRelay::local_addr_v4();
        let _ = UdpRelay::local_addr_v6();
    }
}

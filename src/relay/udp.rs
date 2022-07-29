use crate::{Error, EventBase, Relay};
use bytes::Bytes;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

/// A [`Relay`] that will print events to UDP listener
#[derive(Debug)]
pub struct Udp {
    udp_socket: UdpSocket,
}

impl Udp {
    /// [Udp] relay will bind to the given remote_addr
    pub fn bind<S>(remote_addrs: S) -> Result<Self, Error>
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

        let udp_socket = UdpSocket::bind(local_addr)?;

        udp_socket.connect(&remote_addr)?;

        Ok(Self { udp_socket })
    }
}

impl Relay for Udp {
    fn transport(&self, _event_base: EventBase, event: Bytes) -> Result<(), Error> {
        if let Err(ref error) = self.udp_socket.send(&event) {
            tracing::error!(%error, "Couldn't send data to UDP socket");
        };

        Ok(())
    }
}

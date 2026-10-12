#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(unix), allow(dead_code))]
#![doc = "App-owned IPv4 UDP sessions through public iOS BSD sockets."]

#[cfg(not(unix))]
compile_error!("ios-datagram requires a Unix socket platform");

mod socket;

pub use framework_datagram::{
    Datagram, DatagramBindConfig, DatagramError, DatagramSession, Ipv4Endpoint, Ipv6Datagram,
    Ipv6DatagramBindConfig, Ipv6DatagramSession, Ipv6Endpoint, MAX_DATAGRAM_PAYLOAD_BYTES,
    MAX_IPV6_DATAGRAM_PAYLOAD_BYTES, ReceiveWait,
};
pub use socket::{IosDatagramBackend, IosIpv6DatagramBackend};

/// A statically selected Rust-native iOS IPv4 UDP session.
pub type IosDatagramSession = DatagramSession<IosDatagramBackend>;

/// A statically selected Rust-native iOS IPv6 UDP session.
pub type IosIpv6DatagramSession = Ipv6DatagramSession<IosIpv6DatagramBackend>;

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(unix), allow(dead_code))]
#![doc = "App-owned IPv4 UDP sessions through public iOS BSD sockets."]

#[cfg(not(unix))]
compile_error!("ios-datagram requires a Unix socket platform");

mod socket;

pub use framework_datagram::{
    Datagram, DatagramBindConfig, DatagramError, DatagramSession, Ipv4Endpoint,
    MAX_DATAGRAM_PAYLOAD_BYTES, ReceiveWait,
};
pub use socket::IosDatagramBackend;

/// A statically selected Rust-native iOS IPv4 UDP session.
pub type IosDatagramSession = DatagramSession<IosDatagramBackend>;

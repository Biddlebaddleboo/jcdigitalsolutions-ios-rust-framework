use core::num::NonZeroU32;
use ios_datagram::{DatagramBindConfig, IosDatagramSession, Ipv4Endpoint, ReceiveWait};

#[unsafe(no_mangle)]
pub extern "C" fn ios_datagram_link_probe() -> u8 {
    let config = DatagramBindConfig::new(Ipv4Endpoint::new([127, 0, 0, 1], 0))
        .with_reuse_address(true)
        .with_broadcast(true);
    let Ok(session) = IosDatagramSession::bind(config) else {
        return 0;
    };
    let Ok(local) = session.local_endpoint() else {
        return 0;
    };
    let destination = Ipv4Endpoint::new([127, 0, 0, 1], local.port());
    let _ = session.send_to(destination, &[]);
    let _ = session.receive(
        1_200,
        ReceiveWait::TimeoutMillis(NonZeroU32::new(1).unwrap()),
    );
    let _ = session.cancel_receive();
    let _ = session.close();
    1
}

fn main() {}

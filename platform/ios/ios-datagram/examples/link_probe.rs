use core::num::NonZeroU32;
use ios_datagram::{
    DatagramBindConfig, IosDatagramSession, IosIpv6DatagramSession, Ipv4Endpoint,
    Ipv6DatagramBindConfig, Ipv6Endpoint, ReceiveWait,
};

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

    let ipv6_config = Ipv6DatagramBindConfig::new(Ipv6Endpoint::new(
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        0,
        0,
    ))
    .with_reuse_address(true);
    let Ok(ipv6_session) = IosIpv6DatagramSession::bind(ipv6_config) else {
        return 0;
    };
    let Ok(ipv6_local) = ipv6_session.local_endpoint() else {
        return 0;
    };
    let ipv6_destination = Ipv6Endpoint::new(
        ipv6_local.address(),
        ipv6_local.port(),
        ipv6_local.scope_id(),
    )
    .with_flowinfo(ipv6_local.flowinfo());
    let _ = ipv6_session.send_to(ipv6_destination, &[]);
    let _ = ipv6_session.receive(
        1_200,
        ReceiveWait::TimeoutMillis(NonZeroU32::new(1).unwrap()),
    );
    let _ = ipv6_session.cancel_receive();
    let _ = ipv6_session.close();
    1
}

fn main() {}

#![deny(warnings)]

#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::task::{Context, Waker};
#[cfg(target_os = "ios")]
use framework_connection::{Connection, Endpoint};
#[cfg(target_os = "ios")]
use ios_connection::IosConnectionBackend;

#[cfg(target_os = "ios")]
fn main() {
    let mut connection = Connection::new(IosConnectionBackend::new());
    let mut context = Context::from_waker(Waker::noop());
    {
        let mut connect = core::pin::pin!(
            connection
                .connect(Endpoint::new("example.invalid", 443).expect("fixed endpoint is valid"),)
        );
        if core::hint::black_box(false) {
            let _ = connect.as_mut().poll(&mut context);
        }
    }

    {
        let mut send = core::pin::pin!(connection.send(b"probe"));
        if core::hint::black_box(false) {
            let _ = send.as_mut().poll(&mut context);
        }
    }

    {
        let mut receive = core::pin::pin!(
            connection.receive(core::num::NonZeroU32::new(1).expect("one is nonzero"),)
        );
        if core::hint::black_box(false) {
            let _ = receive.as_mut().poll(&mut context);
        }
    }

    {
        let mut close = core::pin::pin!(connection.close());
        if core::hint::black_box(false) {
            let _ = close.as_mut().poll(&mut context);
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

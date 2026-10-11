#![deny(warnings)]

#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::task::{Context, Waker};
#[cfg(target_os = "ios")]
use framework_connection::TcpListener;
#[cfg(target_os = "ios")]
use ios_connection::IosTcpListenerBackend;

#[cfg(target_os = "ios")]
fn main() {
    let mut listener = TcpListener::new(IosTcpListenerBackend::new());
    let mut context = Context::from_waker(Waker::noop());

    {
        let mut bind = core::pin::pin!(listener.bind(0));
        if core::hint::black_box(false) {
            let _ = bind.as_mut().poll(&mut context);
        }
    }

    {
        let mut accept = core::pin::pin!(listener.accept());
        if core::hint::black_box(false)
            && let core::task::Poll::Ready(Ok(mut stream)) = accept.as_mut().poll(&mut context)
        {
            {
                let mut send = core::pin::pin!(stream.send(b"probe"));
                let _ = send.as_mut().poll(&mut context);
            }
            {
                let mut receive = core::pin::pin!(
                    stream.receive(core::num::NonZeroU32::new(1).expect("one is nonzero"))
                );
                let _ = receive.as_mut().poll(&mut context);
            }
            {
                let mut close = core::pin::pin!(stream.close());
                let _ = close.as_mut().poll(&mut context);
            }
        }
    }

    {
        let mut close = core::pin::pin!(listener.close());
        if core::hint::black_box(false) {
            let _ = close.as_mut().poll(&mut context);
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

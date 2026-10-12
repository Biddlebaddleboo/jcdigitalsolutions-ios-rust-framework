#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::Pin;
#[cfg(target_os = "ios")]
use core::task::{Context, Poll, Waker};
#[cfg(target_os = "ios")]
use std::env;

#[cfg(target_os = "ios")]
use framework_service_discovery::{BonjourBrowserBackend, BonjourQuery};
#[cfg(target_os = "ios")]
use ios_service_discovery::IosBonjourBrowserBackend;

#[cfg(target_os = "ios")]
fn poll_once<F: Future>(mut future: Pin<&mut F>) -> Poll<F::Output> {
    let mut context = Context::from_waker(Waker::noop());
    future.as_mut().poll(&mut context)
}

#[cfg(target_os = "ios")]
fn main() {
    if env::args().any(|argument| argument == "--run-link-probe") {
        let mut browser = IosBonjourBrowserBackend::new();
        let query = BonjourQuery::new("_example._tcp", Some("local.")).unwrap();
        let mut start = Box::pin(browser.start(query));
        let _ = poll_once(start.as_mut());
        drop(start);
        let mut next = Box::pin(browser.next_event());
        let _ = poll_once(next.as_mut());
        drop(next);
        browser.cancel();
        let mut close = Box::pin(browser.close());
        let _ = poll_once(close.as_mut());
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

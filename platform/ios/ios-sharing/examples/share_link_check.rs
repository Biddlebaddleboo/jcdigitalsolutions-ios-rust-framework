#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::pin;
#[cfg(target_os = "ios")]
use core::task::{Context, Waker};
#[cfg(target_os = "ios")]
use framework_sharing::{ShareClient, ShareItem, ShareRequest};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_sharing::IosShareBackend;
#[cfg(target_os = "ios")]
use objc2::MainThreadMarker;
#[cfg(target_os = "ios")]
use objc2_core_foundation::CGRect;
#[cfg(target_os = "ios")]
use objc2_ui_kit::{UIView, UIViewController};

#[cfg(target_os = "ios")]
fn poll_linked_operation<F: Future>(future: F) {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let _ = future.as_mut().poll(&mut context);
}

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Some(marker) = MainThreadMarker::new() else {
        return;
    };
    let presenter = UIViewController::new(marker);
    let Some(marker) = MainThreadMarker::new() else {
        return;
    };
    let source_view = UIView::new(marker);
    let backend = IosShareBackend::new(main_thread, &presenter, &source_view, CGRect::ZERO);
    let mut client = ShareClient::new(backend);
    let _ = client.availability();

    // Link-only branch. Build this probe but do not execute it; polling presents share UI.
    if core::hint::black_box(false) {
        poll_linked_operation(client.share(ShareRequest::new(ShareItem::text(String::from(
            "link probe",
        )))));
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::pin;
#[cfg(target_os = "ios")]
use core::task::{Context, Waker};
#[cfg(target_os = "ios")]
use framework_sharing::Clipboard;
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_sharing::IosClipboardBackend;

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
    let mut clipboard = Clipboard::new(IosClipboardBackend::new(main_thread));
    let _ = clipboard.availability();

    // Link-only branch. Build this probe but do not execute it; polling accesses the live pasteboard.
    if core::hint::black_box(false) {
        poll_linked_operation(clipboard.read());
        poll_linked_operation(clipboard.write("link probe"));
        poll_linked_operation(clipboard.clear());
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

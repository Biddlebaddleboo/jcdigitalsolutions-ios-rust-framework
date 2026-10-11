#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_sharing::{IosDocumentExportFile, IosDocumentExportSession};
#[cfg(target_os = "ios")]
use objc2::MainThreadMarker;
#[cfg(target_os = "ios")]
use objc2_foundation::{NSString, NSURL};
#[cfg(target_os = "ios")]
use objc2_ui_kit::UIViewController;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Some(marker) = MainThreadMarker::new() else {
        return;
    };
    let presenter = UIViewController::new(marker);
    let path = NSString::from_str("/tmp/ios-sharing-document-export-link-check.txt");
    let source = NSURL::fileURLWithPath(&path);
    let mut session = IosDocumentExportSession::new(main_thread, &presenter);

    // Link-only branch. Do not execute this example; it presents UIKit system UI.
    if core::hint::black_box(false) {
        let _ = session.start(&source, |_| {});
        let staged = IosDocumentExportFile::from_bytes("ridesharebot-ios-logs.txt", b"log bytes")
            .expect("static link probe file name and payload are valid");
        let _ = session.start_staged(staged, |_| {});
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

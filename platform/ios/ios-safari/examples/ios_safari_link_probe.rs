#[cfg(target_os = "ios")]
use framework_format::Uri;
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_safari::IosSafariViewController;

#[cfg(target_os = "ios")]
fn main() {
    if std::hint::black_box(false) {
        let Some(main_thread) = MainThread::current() else {
            return;
        };
        let Ok(uri) = Uri::new("https://example.test/") else {
            return;
        };
        if let Ok(controller) = IosSafariViewController::new(main_thread, uri) {
            std::hint::black_box(controller.view_controller());
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

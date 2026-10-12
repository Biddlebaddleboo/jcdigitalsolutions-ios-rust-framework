#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use ios_photo_picker::IosPhotoPickerSession;
    use ios_runtime::main_thread::MainThread;
    use objc2_ui_kit::UIViewController;

    if std::hint::black_box(false) {
        let Some(presenter_thread) = MainThread::current() else {
            return;
        };
        let presenter = UIViewController::new(presenter_thread.into_objc2());
        let Some(picker_thread) = MainThread::current() else {
            return;
        };
        let session =
            IosPhotoPickerSession::present(picker_thread, &presenter, 32 * 1024 * 1024, |_| {});
        std::hint::black_box(&session);
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

#![deny(warnings)]

#[cfg(target_os = "ios")]
mod probe {
    use ios_runtime::main_thread::MainThread;
    use ios_sharing::IosDocumentImportSession;
    use objc2_ui_kit::UIViewController;

    pub fn document_import_link_probe(main_thread: MainThread, presenter: &UIViewController) {
        let mut session = IosDocumentImportSession::new(main_thread, presenter);
        let _ = session.start(&["public.plain-text"], |_| {});
    }
}

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(
        probe::document_import_link_probe
            as fn(ios_runtime::main_thread::MainThread, &objc2_ui_kit::UIViewController),
    );
}

#[cfg(not(target_os = "ios"))]
fn main() {}

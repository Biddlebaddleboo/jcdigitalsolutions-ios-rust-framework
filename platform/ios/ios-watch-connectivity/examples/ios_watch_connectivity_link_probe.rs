#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use framework_watch_connectivity::WatchConnectivityBackend;
    use ios_watch_connectivity::IosWatchConnectivityBackend;

    let _ = core::hint::black_box(IosWatchConnectivityBackend::session_support());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

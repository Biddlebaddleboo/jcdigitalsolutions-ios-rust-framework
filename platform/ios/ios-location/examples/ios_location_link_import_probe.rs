#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_location::LocationBackend;
#[cfg(target_os = "ios")]
use ios_location::IosLocationBackend;
#[cfg(target_os = "ios")]
use objc2::MainThreadMarker;

#[cfg(target_os = "ios")]
fn main() {
    let marker = MainThreadMarker::new().expect("the import probe runs on the iOS main thread");
    let backend = IosLocationBackend::new(marker);
    let _ = core::hint::black_box(backend.availability());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

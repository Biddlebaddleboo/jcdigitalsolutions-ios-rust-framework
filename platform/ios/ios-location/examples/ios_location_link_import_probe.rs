#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_location::{AccuracyTarget, LocationBackend, LocationRequest};
#[cfg(target_os = "ios")]
use ios_location::IosLocationBackend;
#[cfg(target_os = "ios")]
use objc2::MainThreadMarker;

#[cfg(target_os = "ios")]
fn main() {
    let marker = MainThreadMarker::new().expect("the import probe runs on the iOS main thread");
    let backend = IosLocationBackend::new(marker);
    let _ = core::hint::black_box(backend.availability());

    if core::hint::black_box(false) {
        let marker = MainThreadMarker::new().expect("the import probe runs on the iOS main thread");
        let mut backend = IosLocationBackend::new(marker);
        let mut authorization = backend.request_always_authorization();
        let waker = std::task::Waker::noop();
        let mut context = std::task::Context::from_waker(waker);
        let _ = core::future::Future::poll(core::pin::Pin::new(&mut authorization), &mut context);
        drop(authorization);
        let request = LocationRequest::new(AccuracyTarget::new(0.0).expect("finite accuracy"));
        let options = ios_location::IosBackgroundLocationOptions::new()
            .with_activity_type(ios_location::IosLocationActivityType::OtherNavigation)
            .with_automatic_pausing(false)
            .with_background_indicator(true);
        let _ = core::hint::black_box(backend.start_background_updates(request, options));
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

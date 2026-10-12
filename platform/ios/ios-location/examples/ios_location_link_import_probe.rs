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
        let _ = core::hint::black_box(backend.start_significant_change_updates());
        let request = LocationRequest::new(AccuracyTarget::new(0.0).expect("finite accuracy"));
        let options = ios_location::IosBackgroundLocationOptions::new()
            .with_activity_type(ios_location::IosLocationActivityType::OtherNavigation)
            .with_automatic_pausing(false)
            .with_background_indicator(true);
        let _ = core::hint::black_box(backend.start_background_updates(request, options));
        if let Ok(open) = backend.open_geofence_monitor("B556Probe") {
            let mut open = core::pin::pin!(open);
            if let core::task::Poll::Ready(Ok(mut monitor)) =
                core::future::Future::poll(open.as_mut(), &mut context)
            {
                let region = ios_location::IosGeofence::new("probe", 0.0, 0.0, 10.0)
                    .expect("valid geofence");
                let _ = monitor.add_or_replace(&region);
                let _ = monitor.list();
                let _ = monitor.remove(region.identifier());
                let mut event = core::pin::pin!(monitor.next_event());
                if let core::task::Poll::Ready(Some(event)) =
                    core::future::Future::poll(event.as_mut(), &mut context)
                {
                    let _ = core::hint::black_box((
                        event.identifier(),
                        event.state(),
                        event.timestamp_unix_millis(),
                        event.issues(),
                        event.missed_before(),
                    ));
                }
            }
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

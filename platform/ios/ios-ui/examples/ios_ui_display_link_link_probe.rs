#[cfg(target_os = "ios")]
use ios_ui::IosDisplayLink;
#[cfg(target_os = "ios")]
use objc2::MainThreadMarker;

#[cfg(target_os = "ios")]
fn main() {
    let marker = MainThreadMarker::new().expect("the probe must run on the main thread");
    let mut link = IosDisplayLink::new(marker, |frame| {
        let _timing = (
            frame.timestamp_seconds(),
            frame.target_timestamp_seconds(),
            frame.duration_seconds(),
        );
    });
    link.pause();
    assert!(link.is_paused());
    link.resume();
    assert!(link.is_valid());
    link.invalidate();
    assert!(!link.is_valid());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

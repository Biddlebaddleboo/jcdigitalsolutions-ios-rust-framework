#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_media::MediaTime;
#[cfg(target_os = "ios")]
use ios_media::IosMediaTime;

#[cfg(target_os = "ios")]
fn main() {
    let Ok(time) = MediaTime::new(1001, 30_000) else {
        return;
    };
    let native = IosMediaTime::from_portable(time);
    core::hint::black_box(native.as_cm_time());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

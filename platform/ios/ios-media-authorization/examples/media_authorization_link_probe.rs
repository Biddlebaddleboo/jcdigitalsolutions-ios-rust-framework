#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::Pin;
#[cfg(target_os = "ios")]
use core::task::{Context, Waker};
#[cfg(target_os = "ios")]
use framework_media_authorization::{CaptureMedia, MediaAuthorization};
#[cfg(target_os = "ios")]
use ios_media_authorization::IosMediaAuthorization;

#[cfg(target_os = "ios")]
fn main() {
    let _ = IosMediaAuthorization::authorization_status(CaptureMedia::Camera);
    let mut adapter = IosMediaAuthorization;
    let mut request = adapter.request_authorization(CaptureMedia::Camera);
    let mut context = Context::from_waker(Waker::noop());
    let _ = Pin::new(&mut request).poll(&mut context);
}

#[cfg(not(target_os = "ios"))]
fn main() {}

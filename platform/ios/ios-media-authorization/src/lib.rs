#![deny(missing_docs)]
#![doc = "An iOS AVFoundation adapter for camera and microphone authorization."]

#[cfg(any(target_os = "ios", test))]
mod status;

#[cfg(any(target_os = "ios", test))]
mod operation;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
mod request;

#[cfg(target_os = "ios")]
pub use platform::IosMediaAuthorization;

#[cfg(target_os = "ios")]
pub use request::IosMediaAuthorizationFuture;

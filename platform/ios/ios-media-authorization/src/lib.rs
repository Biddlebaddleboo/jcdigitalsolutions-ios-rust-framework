#![deny(missing_docs)]
#![doc = "An iOS AVFoundation adapter for camera and microphone authorization status only."]

#[cfg(any(target_os = "ios", test))]
mod status;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::IosMediaAuthorization;

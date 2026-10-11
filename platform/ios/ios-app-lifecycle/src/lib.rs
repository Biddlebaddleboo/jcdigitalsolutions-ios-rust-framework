#![deny(missing_docs)]
#![doc = "UIKit application lifecycle notifications for the iOS backend."]

#[cfg(any(target_os = "ios", test))]
mod queue;

#[cfg(any(target_os = "ios", test))]
mod mapping;

#[cfg(any(target_os = "ios", test))]
mod observers;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosApplicationLifecycle, OpenApplicationUriError, open_application_settings,
    open_application_uri,
};

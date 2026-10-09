#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS-only MediaPlayer media-library authorization status query."]
#![doc = "\n\nThis crate reports the current authorization status only. It does not request access, show a prompt, read library content, or access Apple Music catalog or service data. The API is available from iOS 9.3. It has no portable facade, permission request, callback, or entitlement requirement."]

#[cfg(target_os = "ios")]
mod platform;
#[cfg(target_os = "ios")]
mod status;

#[cfg(target_os = "ios")]
pub use platform::media_library_authorization_status;
#[cfg(target_os = "ios")]
pub use status::MediaLibraryAuthorizationStatus;

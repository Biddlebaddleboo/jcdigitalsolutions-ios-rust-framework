#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A status-only Vision text-recognition revision query for iOS."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::text_recognition_revision_support;

/// Returns no iOS Vision revision status on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn text_recognition_revision_support(
    _revision: u32,
) -> Option<framework_vision::TextRecognitionRevisionSupport> {
    None
}

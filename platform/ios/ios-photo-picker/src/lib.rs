#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "User-mediated single-image selection through public PhotosUI"]

mod copy;
mod session_policy;

#[cfg(target_os = "ios")]
mod platform;

pub use copy::IosPickedImage;
pub use session_policy::IosPhotoPickerError;

/// A terminal outcome from a user-mediated single-image PhotosUI session.
#[derive(Debug)]
#[non_exhaustive]
pub enum IosPhotoPickerEvent {
    /// One provider image was copied byte-for-byte into owned app-temporary storage.
    Selected(IosPickedImage),
    /// The user or caller cancelled the picker.
    Cancelled,
    /// PhotosUI selection or provider transfer failed.
    Failed(IosPhotoPickerError),
}

#[cfg(target_os = "ios")]
pub use platform::IosPhotoPickerSession;

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "User-mediated image selection through public PhotosUI"]

mod copy;
mod session_policy;

#[cfg(target_os = "ios")]
mod platform;

pub use copy::IosPickedImage;
pub use session_policy::IosPhotoPickerError;

/// Maximum number of images accepted by a multi-image picker session.
pub const MAX_MULTI_IMAGE_SELECTION: u8 = 10;
/// Minimum number of images allowed by a multi-image picker session.
pub const MIN_MULTI_IMAGE_SELECTION: u8 = 2;
/// Maximum aggregate provider-file size accepted by a multi-image picker session.
pub const MAX_MULTI_IMAGE_AGGREGATE_BYTES: u64 = 256 * 1024 * 1024;

/// A terminal outcome from a user-mediated PhotosUI session.
#[derive(Debug)]
#[non_exhaustive]
pub enum IosPhotoPickerEvent {
    /// One provider image was copied byte-for-byte into owned app-temporary storage.
    Selected(IosPickedImage),
    /// Provider images were copied byte-for-byte in PhotosUI selection order.
    SelectedMultiple(Vec<IosPickedImage>),
    /// The user or caller cancelled the picker.
    Cancelled,
    /// PhotosUI selection or provider transfer failed.
    Failed(IosPhotoPickerError),
}

#[cfg(target_os = "ios")]
pub use platform::IosPhotoPickerSession;

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "iOS PhotoKit read/write authorization backend for `framework-photos`"]

mod operation;
mod save;
mod status;

#[cfg(target_os = "ios")]
mod import;
#[cfg(target_os = "ios")]
mod platform;

pub use save::{PhotosAssetFileKind, PhotosSaveError, PhotosSourcePathError};
pub use status::{add_only_authorization_status_from_native, authorization_status_from_native};

#[cfg(target_os = "ios")]
pub use import::IosPhotosSaveFuture;
#[cfg(target_os = "ios")]
pub use platform::{
    IosPhotosAddOnlyAuthorizationFuture, IosPhotosAuthorizationFuture, IosPhotosBackend,
};

use framework_core::{Error, ErrorKind, PlatformErrorCode};
#[cfg(target_os = "ios")]
use framework_secure_storage::SecureStorageError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeStatus {
    Success,
    ItemNotFound,
    Failure(i32),
}

impl NativeStatus {
    pub(super) const fn from_os_status(status: i32, item_not_found: i32) -> Self {
        if status == 0 {
            Self::Success
        } else if status == item_not_found {
            Self::ItemNotFound
        } else {
            Self::Failure(status)
        }
    }
}

pub(super) fn native_error(status: i32) -> Error {
    Error::new(ErrorKind::Platform).with_platform_code(
        PlatformErrorCode::new(status).expect("native failure status is nonzero"),
    )
}

#[cfg(target_os = "ios")]
pub(super) fn storage_error(status: i32) -> SecureStorageError {
    SecureStorageError::Backend(native_error(status))
}

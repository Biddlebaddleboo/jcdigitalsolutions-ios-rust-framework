#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable status-only authorization contract for camera and microphone access."]

/// A capture-media category whose authorization status can be queried.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CaptureMedia {
    /// Camera capture permission.
    Camera,
    /// Microphone recording permission.
    Microphone,
}

/// A snapshot of the operating system's authorization status for capture media.
///
/// `Authorized` reports permission state only. It does not establish that suitable hardware is
/// present, available, or ready for capture. Query again when current state matters because the
/// user or operating system may change the status outside this process.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MediaAuthorizationStatus {
    /// The user has not yet made an authorization choice.
    NotDetermined,
    /// System policy prevents access and the user cannot change the status in the app.
    Restricted,
    /// Access is not authorized.
    Denied,
    /// The operating system reports that access is authorized.
    Authorized,
    /// The backend returned a status value it does not recognize.
    Unknown,
}

/// A platform implementation of the status-only camera/microphone contract.
///
/// This trait has no request method and cannot start a capture or audio session.
pub trait MediaAuthorization {
    /// Reads the current authorization status for the selected media category.
    fn authorization_status(media: CaptureMedia) -> MediaAuthorizationStatus;
}

#[cfg(test)]
mod tests {
    use super::{CaptureMedia, MediaAuthorizationStatus};

    #[test]
    fn camera_and_microphone_are_distinct_categories() {
        assert_ne!(CaptureMedia::Camera, CaptureMedia::Microphone);
    }

    #[test]
    fn every_authorization_status_is_distinct() {
        let statuses = [
            MediaAuthorizationStatus::NotDetermined,
            MediaAuthorizationStatus::Restricted,
            MediaAuthorizationStatus::Denied,
            MediaAuthorizationStatus::Authorized,
            MediaAuthorizationStatus::Unknown,
        ];
        for left in 0..statuses.len() {
            for right in left + 1..statuses.len() {
                assert_ne!(statuses[left], statuses[right]);
            }
        }
    }
}

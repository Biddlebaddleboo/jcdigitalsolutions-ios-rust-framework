use objc2_media_player::MPMediaLibrary;

use crate::MediaLibraryAuthorizationStatus;

/// Return the point-in-time MediaPlayer library authorization status.
///
/// This synchronous query calls `+[MPMediaLibrary authorizationStatus]`. It does not call
/// `requestAuthorization:`, create a media-library object, read media items, or contact Apple
/// Music services. Re-query when current state matters. Call only on iOS 9.3 or later; the host
/// app must set a compatible deployment target. This API makes no thread-safety claim.
pub fn media_library_authorization_status() -> MediaLibraryAuthorizationStatus {
    // SAFETY: The generated binding marks Objective-C message sends unsafe. This class method
    // takes no caller-provided pointer or object and returns a scalar status. The caller must use
    // this function only on the iOS 9.3+ API range declared by Apple's SDK.
    let status = unsafe { MPMediaLibrary::authorizationStatus() };
    MediaLibraryAuthorizationStatus::from_raw_value(status.0 as i64)
}

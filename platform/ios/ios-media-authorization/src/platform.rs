use framework_media_authorization::{CaptureMedia, MediaAuthorization, MediaAuthorizationStatus};
use objc2_av_foundation::{
    AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio, AVMediaTypeVideo,
};

use crate::status::{AvMediaType, av_media_type, map_av_authorization_status};

/// A zero-sized AVFoundation implementation of the status-only media contract.
///
/// This query does not prompt, create a capture device/input/session, or access camera or
/// microphone samples.
pub struct IosMediaAuthorization;

impl MediaAuthorization for IosMediaAuthorization {
    fn authorization_status(media: CaptureMedia) -> MediaAuthorizationStatus {
        // SAFETY: These immutable AVFoundation constants identify only the two accepted values
        // for `authorizationStatusForMediaType:`. A nil constant is handled as Unknown below.
        let Some(av_media_type) = av_media_type(media) else {
            return MediaAuthorizationStatus::Unknown;
        };
        let media_type = unsafe {
            match av_media_type {
                AvMediaType::Video => AVMediaTypeVideo,
                AvMediaType::Audio => AVMediaTypeAudio,
            }
        };
        let Some(media_type) = media_type else {
            return MediaAuthorizationStatus::Unknown;
        };

        // SAFETY: The generated API marks this call unsafe because invalid media-type strings
        // raise an Objective-C exception. `media_type` is exactly AVMediaTypeVideo or
        // AVMediaTypeAudio, as required by the public SDK declaration.
        let status: AVAuthorizationStatus =
            unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
        map_av_authorization_status(status.0)
    }
}

use framework_media_authorization::{CaptureMedia, MediaAuthorizationStatus};

/// The only AV media-type values this adapter supplies to AVFoundation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AvMediaType {
    Video,
    Audio,
}

/// Maps the portable camera/microphone categories to their exact AVFoundation constants.
pub(crate) const fn av_media_type(media: CaptureMedia) -> Option<AvMediaType> {
    match media {
        CaptureMedia::Camera => Some(AvMediaType::Video),
        CaptureMedia::Microphone => Some(AvMediaType::Audio),
        _ => None,
    }
}

/// Maps the `AVAuthorizationStatus` `NSInteger` value to the portable status.
///
/// Values match the public `AVCaptureDevice.h` `NS_ENUM` declarations: NotDetermined=0,
/// Restricted=1, Denied=2, and Authorized=3.
pub(crate) const fn map_av_authorization_status(raw: isize) -> MediaAuthorizationStatus {
    match raw {
        0 => MediaAuthorizationStatus::NotDetermined,
        1 => MediaAuthorizationStatus::Restricted,
        2 => MediaAuthorizationStatus::Denied,
        3 => MediaAuthorizationStatus::Authorized,
        _ => MediaAuthorizationStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{AvMediaType, av_media_type, map_av_authorization_status};
    use framework_media_authorization::{CaptureMedia, MediaAuthorizationStatus};

    #[test]
    fn camera_maps_to_video_and_microphone_maps_to_audio() {
        assert_eq!(
            av_media_type(CaptureMedia::Camera),
            Some(AvMediaType::Video)
        );
        assert_eq!(
            av_media_type(CaptureMedia::Microphone),
            Some(AvMediaType::Audio)
        );
    }

    #[test]
    fn native_authorization_values_map_with_unknown_fallback() {
        assert_eq!(
            map_av_authorization_status(0),
            MediaAuthorizationStatus::NotDetermined
        );
        assert_eq!(
            map_av_authorization_status(1),
            MediaAuthorizationStatus::Restricted
        );
        assert_eq!(
            map_av_authorization_status(2),
            MediaAuthorizationStatus::Denied
        );
        assert_eq!(
            map_av_authorization_status(3),
            MediaAuthorizationStatus::Authorized
        );
        assert_eq!(
            map_av_authorization_status(-1),
            MediaAuthorizationStatus::Unknown
        );
        assert_eq!(
            map_av_authorization_status(4),
            MediaAuthorizationStatus::Unknown
        );
    }
}

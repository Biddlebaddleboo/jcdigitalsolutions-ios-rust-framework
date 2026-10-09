#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A status-only AVFoundation default video-device query for iOS"]

/// Return `true` if AVFoundation reports a current default video capture device
///
/// This point-in-time result reports device presence only. It does not report
/// camera authorization, capture readiness, session state, or future access
///
/// On iOS this calls `AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`
/// only. It does not request authorization, create an `AVCaptureDeviceInput` or
/// `AVCaptureSession`, access samples, or show UI. A host app still needs
/// `NSCameraUsageDescription` before it requests camera access or creates an input
///
/// On non-iOS targets this function returns `false`
pub fn has_default_video_capture_device() -> bool {
    #[cfg(target_os = "ios")]
    {
        use objc2_av_foundation::{AVCaptureDevice, AVMediaTypeVideo};

        if !objc2::available!(ios = 4.0, ..) {
            return false;
        }

        // SAFETY: this reads the public AVFoundation constant for AVMediaTypeVideo
        let Some(media_type) = (unsafe { AVMediaTypeVideo }) else {
            return false;
        };

        // SAFETY: the API floor is checked above and the media type is the public video constant
        unsafe { AVCaptureDevice::defaultDeviceWithMediaType(media_type) }.is_some()
    }

    #[cfg(not(target_os = "ios"))]
    {
        false
    }
}

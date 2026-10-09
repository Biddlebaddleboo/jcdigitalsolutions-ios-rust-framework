#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Point-in-time ReplayKit availability status for iOS, without starting capture."]

use framework_media::ReplayKitAvailability;

/// Reads `RPScreenRecorder.isAvailable` without starting recording or capture.
///
/// The synchronous query does not present a picker, request consent, enable the microphone or
/// camera, start or stop recording, or start broadcasting. The returned bit can become stale
/// immediately and does not guarantee a later start will succeed. On non-iOS targets this iOS
/// adapter reports `false`.
///
/// ReplayKit's native availability property is available from iOS 9.0. Apple currently marks the
/// property deprecated in its documentation and points to ScreenCaptureKit; this status adapter
/// is limited to existing ReplayKit use and does not expose or activate the newer capture APIs.
pub fn availability_snapshot() -> ReplayKitAvailability {
    #[cfg(target_os = "ios")]
    {
        use objc2_replay_kit::RPScreenRecorder;

        if objc2::available!(ios = 9.0, ..) {
            // SAFETY: Apple's documented singleton accessor returns this app's recorder; no
            // caller-owned pointers or state are passed, and this query starts no operation.
            let recorder = unsafe { RPScreenRecorder::sharedRecorder() };
            // SAFETY: the retained recorder is valid for this documented read-only status query.
            let available_for_recording = unsafe { recorder.isAvailable() };
            ReplayKitAvailability::new(available_for_recording)
        } else {
            ReplayKitAvailability::new(false)
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        ReplayKitAvailability::new(false)
    }
}

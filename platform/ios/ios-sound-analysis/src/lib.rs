#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Built-in SoundAnalysis classifier recognition snapshot for iOS, without audio analysis."]

use framework_media::SoundAnalysisSupportSnapshot;

/// Checks whether iOS recognizes its built-in SoundAnalysis classifier request.
///
/// On iOS 15.0+, this creates `SNClassifySoundRequest` with
/// `SNClassifierIdentifierVersion1` and checks that the request exposes at least one known
/// classification. It does not create an analyzer, supply audio, request microphone access, or
/// establish analysis success. On older iOS releases and non-iOS targets, it returns `false`.
pub fn support_snapshot() -> SoundAnalysisSupportSnapshot {
    #[cfg(target_os = "ios")]
    {
        if !objc2::available!(ios = 15.0, ..) {
            return SoundAnalysisSupportSnapshot::new(false);
        }

        use objc2::AnyThread;
        use objc2_sound_analysis::{SNClassifierIdentifierVersion1, SNClassifySoundRequest};

        // SAFETY: this immutable framework constant is the documented built-in classifier ID.
        let Some(classifier_identifier) = (unsafe { SNClassifierIdentifierVersion1 }) else {
            return SoundAnalysisSupportSnapshot::new(false);
        };
        // SAFETY: this initializer receives Apple's built-in classifier ID and does not attach an
        // analyzer, supply audio, or start an analysis operation.
        let request = unsafe {
            SNClassifySoundRequest::initWithClassifierIdentifier_error(
                SNClassifySoundRequest::alloc(),
                classifier_identifier,
            )
        };
        let available = match request {
            Ok(request) => {
                // SAFETY: the retained request is valid for its documented immutable label query.
                unsafe { request.knownClassifications() }.count() > 0
            }
            Err(_) => false,
        };
        SoundAnalysisSupportSnapshot::new(available)
    }

    #[cfg(not(target_os = "ios"))]
    {
        SoundAnalysisSupportSnapshot::new(false)
    }
}

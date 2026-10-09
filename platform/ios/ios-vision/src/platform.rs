use framework_vision::TextRecognitionRevisionSupport;
use objc2::{ClassType, msg_send, rc::Retained};
use objc2_foundation::NSIndexSet;
use objc2_vision::VNRecognizeTextRequest;

/// Queries whether the runtime lists a Vision text-recognition request revision as supported.
///
/// Returns `None` before iOS 13.0. This query sends only the `supportedRevisions` class property
/// to `VNRecognizeTextRequest`; it does not create a request, read image data, or invoke a request
/// handler.
pub fn text_recognition_revision_support(revision: u32) -> Option<TextRecognitionRevisionSupport> {
    if !objc2::available!(ios = 13.0, ..) {
        return None;
    }

    let request_class = VNRecognizeTextRequest::class();
    // SAFETY: `VNRecognizeTextRequest` inherits the declared `VNRequest.supportedRevisions`
    // class property. The receiver is that concrete request class, its `NSIndexSet` copy result
    // matches the generated objc2 binding signature, and the iOS 13.0 availability guard is above.
    // This sends no image, request-handler, authorization, or UI selector.
    let supported_revisions: Retained<NSIndexSet> =
        unsafe { msg_send![request_class, supportedRevisions] };
    let supported = supported_revisions.containsIndex(revision as usize);

    Some(TextRecognitionRevisionSupport::from_system(
        revision, supported,
    ))
}

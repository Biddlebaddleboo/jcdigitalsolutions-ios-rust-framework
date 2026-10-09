#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS Speech authorization status query"]

use objc2_speech::{SFSpeechRecognizer, SFSpeechRecognizerAuthorizationStatus};

/// The app's current Speech authorization state
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechAuthorizationStatus {
    /// The app has no saved Speech authorization choice
    NotDetermined,
    /// Apple reports that the app does not have Speech authorization
    Denied,
    /// The system restricts Speech authorization for this app
    Restricted,
    /// The app has Speech authorization
    Authorized,
    /// Speech APIs do not exist on the current iOS version
    Unavailable,
    /// Apple returned a status code this crate does not name
    Unknown(isize),
}

/// Read the app's current Speech authorization state
///
/// This reads `SFSpeechRecognizer::authorizationStatus` only. It does not ask for permission,
/// create an `SFSpeechRecognizer`, accept audio, or start a recognition task. `Authorized` reports
/// the app's saved authorization state only; it does not report service availability or guarantee
/// that a recognition request could succeed
///
/// This returns [`SpeechAuthorizationStatus::Unavailable`] below iOS 10.0. The Speech API docs do
/// not state a main-thread requirement for this class method
pub fn authorization_status() -> SpeechAuthorizationStatus {
    if !objc2::available!(ios = 10.0, ..) {
        return SpeechAuthorizationStatus::Unavailable;
    }

    // SAFETY: this branch enforces the iOS 10.0 class API floor; the method has no pointer inputs
    let status = unsafe { SFSpeechRecognizer::authorizationStatus() };

    match status {
        SFSpeechRecognizerAuthorizationStatus::NotDetermined => {
            SpeechAuthorizationStatus::NotDetermined
        }
        SFSpeechRecognizerAuthorizationStatus::Denied => SpeechAuthorizationStatus::Denied,
        SFSpeechRecognizerAuthorizationStatus::Restricted => SpeechAuthorizationStatus::Restricted,
        SFSpeechRecognizerAuthorizationStatus::Authorized => SpeechAuthorizationStatus::Authorized,
        other => SpeechAuthorizationStatus::Unknown(other.0),
    }
}

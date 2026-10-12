#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "An opt-in iOS VoIP PushKit client with typed token events and a one-shot incoming-push completion."]
#![doc = "\n\nThis package owns only the client-side PushKit registry. The host app owns APNs server configuration, required entitlements and background mode, payload semantics, and CallKit reporting."]

#[cfg(any(test, target_os = "ios"))]
mod completion;
mod token;

#[cfg(target_os = "ios")]
mod platform;

pub use token::VoipPushToken;

#[cfg(target_os = "ios")]
pub use platform::{
    IncomingVoipPush, IosVoipPushService, PayloadEncodingError, PushKitError, PushKitEvent,
};

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use crate::VoipPushToken;
    use crate::completion::OneShotCompletion;

    #[test]
    fn token_preserves_exact_bytes() {
        let input = vec![0, 1, 0xff, 0, 0x7f];
        let token = VoipPushToken::from_bytes(input.clone());

        assert_eq!(token.as_bytes(), input.as_slice());
        assert_eq!(token.clone().into_bytes(), input);
    }

    #[test]
    fn token_accepts_empty_bytes_without_inventing_registration_state() {
        let token = VoipPushToken::from_bytes(Vec::new());

        assert!(token.as_bytes().is_empty());
    }

    #[test]
    fn completion_runs_once_when_explicitly_finished() {
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let mut completion = OneShotCompletion::new(move || observed.set(observed.get() + 1));

        completion.complete();
        completion.complete();

        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn dropped_completion_runs_once_as_fallback() {
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let completion = OneShotCompletion::new(move || observed.set(observed.get() + 1));

        drop(completion);

        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn panicking_completion_does_not_unwind_from_drop() {
        let completion = OneShotCompletion::new(|| panic!("completion fixture"));

        drop(completion);
    }
}

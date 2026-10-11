#[cfg(any(test, target_os = "ios"))]
use std::fmt::Write;
use std::string::String;
use std::vec;
use std::vec::Vec;

/// Framework-owned, fixed-width identity for one call.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CallId([u8; 16]);

impl CallId {
    /// Create a call identity from 16 caller-owned bytes.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Return the fixed-width call identity bytes.
    pub const fn as_bytes(self) -> [u8; 16] {
        self.0
    }

    #[cfg(any(test, target_os = "ios"))]
    pub(crate) fn uuid_string(self) -> String {
        let mut value = String::with_capacity(36);
        for (index, byte) in self.0.iter().enumerate() {
            if matches!(index, 4 | 6 | 8 | 10) {
                value.push('-');
            }
            write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
        }
        value
    }

    #[cfg(any(test, target_os = "ios"))]
    pub(crate) fn from_uuid_string(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 36
            || ![8, 13, 18, 23]
                .into_iter()
                .all(|index| bytes[index] == b'-')
        {
            return None;
        }
        let mut result = [0_u8; 16];
        let mut output_index = 0;
        let mut high_nibble = None;
        for byte in bytes.iter().copied().filter(|byte| *byte != b'-') {
            let nibble = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return None,
            };
            if let Some(high) = high_nibble.take() {
                *result.get_mut(output_index)? = (high << 4) | nibble;
                output_index += 1;
            } else {
                high_nibble = Some(nibble);
            }
        }
        (output_index == 16 && high_nibble.is_none()).then(|| Self::from_bytes(result))
    }
}

/// The public CallKit remote-handle kind supported by an incoming call.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CallHandleKind {
    /// An app-defined opaque handle.
    Generic,
    /// A phone-number handle.
    PhoneNumber,
    /// An email-address handle.
    EmailAddress,
}

/// An owned incoming-call description.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncomingCall {
    pub(crate) id: CallId,
    pub(crate) handle_kind: CallHandleKind,
    pub(crate) handle: String,
    pub(crate) display_name: Option<String>,
    pub(crate) has_video: bool,
}

impl IncomingCall {
    /// Create an incoming call with a framework call identity and remote handle.
    pub fn new(id: CallId, handle_kind: CallHandleKind, handle: impl Into<String>) -> Self {
        Self {
            id,
            handle_kind,
            handle: handle.into(),
            display_name: None,
            has_video: false,
        }
    }

    /// Set the provider-defined localized caller name.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Mark this call as including video.
    pub const fn with_video(mut self, has_video: bool) -> Self {
        self.has_video = has_video;
        self
    }

    /// Return this call's framework identity.
    pub const fn id(&self) -> CallId {
        self.id
    }

    #[cfg(any(test, target_os = "ios"))]
    pub(crate) fn validate(&self, config: &CallProviderConfig) -> Result<(), CallProviderError> {
        if self.handle.is_empty()
            || !config.supported_handles.contains(&self.handle_kind)
            || (self.has_video && !config.supports_video)
        {
            return Err(CallProviderError::InvalidInput);
        }
        Ok(())
    }
}

/// Configuration for one explicitly owned `CXProvider`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallProviderConfig {
    pub(crate) localized_name: String,
    pub(crate) supported_handles: Vec<CallHandleKind>,
    pub(crate) supports_video: bool,
    pub(crate) maximum_call_groups: u32,
    pub(crate) maximum_calls_per_group: u32,
    pub(crate) includes_calls_in_recents: Option<bool>,
}

impl CallProviderConfig {
    /// Create configuration with generic handles, one call group, and one call per group.
    pub fn new(localized_name: impl Into<String>) -> Self {
        Self {
            localized_name: localized_name.into(),
            supported_handles: vec![CallHandleKind::Generic],
            supports_video: false,
            maximum_call_groups: 1,
            maximum_calls_per_group: 1,
            includes_calls_in_recents: None,
        }
    }

    /// Set the handle types this provider can accept.
    pub fn with_supported_handles(mut self, handles: impl Into<Vec<CallHandleKind>>) -> Self {
        self.supported_handles = handles.into();
        self
    }

    /// Enable provider and call-update video support.
    pub const fn with_video(mut self, supports_video: bool) -> Self {
        self.supports_video = supports_video;
        self
    }

    /// Set the maximum number of simultaneous call groups.
    pub const fn with_maximum_call_groups(mut self, count: u32) -> Self {
        self.maximum_call_groups = count;
        self
    }

    /// Set the maximum number of calls in one group.
    pub const fn with_maximum_calls_per_group(mut self, count: u32) -> Self {
        self.maximum_calls_per_group = count;
        self
    }

    /// Set whether ended calls appear in system Recents where that option is available.
    ///
    /// The option was added in iOS 11. `None` leaves the iOS 10 system default unchanged;
    /// requesting either explicit value on iOS 10 returns [`CallProviderError::UnavailableOption`].
    pub const fn with_includes_calls_in_recents(mut self, includes: Option<bool>) -> Self {
        self.includes_calls_in_recents = includes;
        self
    }

    #[cfg(any(test, target_os = "ios"))]
    pub(crate) fn validate(&self) -> Result<(), CallProviderError> {
        if self.localized_name.trim().is_empty()
            || self.supported_handles.is_empty()
            || self.maximum_call_groups == 0
            || self.maximum_calls_per_group == 0
        {
            return Err(CallProviderError::InvalidInput);
        }
        for (index, handle) in self.supported_handles.iter().enumerate() {
            if self.supported_handles[index + 1..].contains(handle) {
                return Err(CallProviderError::InvalidInput);
            }
        }
        Ok(())
    }
}

/// A CallKit action or provider lifecycle event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CallProviderEvent {
    /// The system asks the app to answer an active call.
    AnswerRequested {
        /// The framework-owned call identity.
        call_id: CallId,
    },
    /// The system asks the app to end an active call.
    EndRequested {
        /// The framework-owned call identity.
        call_id: CallId,
    },
    /// CallKit reset the provider and discarded its current calls.
    ProviderReset,
    /// CallKit timed out an action; that action is already failed and must not be completed again.
    ActionTimedOut,
}

/// Immediate response to a CallKit answer or end action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionDecision {
    /// Complete the native action successfully.
    Fulfill,
    /// Fail the native action.
    Fail,
}

/// Reason to report a call as ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEndReason {
    /// The remote party ended the call.
    RemoteEnded,
    /// The call failed.
    Failed,
    /// The call was not answered.
    Unanswered,
    /// The call was answered on another device.
    AnsweredElsewhere,
    /// The call was declined on another device.
    DeclinedElsewhere,
}

/// Outcome of an end request against the provider's local state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndCallOutcome {
    /// The active call was reported ended to CallKit.
    Reported,
    /// The call is still being reported; its end report will follow report acceptance.
    DeferredUntilReport,
    /// The call is already ending.
    AlreadyEnding,
    /// The call identity is not currently owned by this provider.
    NotFound,
}

/// A bounded provider, call, or availability error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CallProviderError {
    /// The provider configuration or incoming-call data is invalid.
    InvalidInput,
    /// A call with this identity is already owned by the provider.
    DuplicateCallId,
    /// The provider has been invalidated or CallKit reset during the operation.
    ProviderUnavailable,
    /// The system disallowed the incoming call.
    SystemRejected,
    /// An explicitly requested option is not available on this iOS release.
    UnavailableOption,
    /// A native identifier could not be represented by the framework type.
    NativeConversionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_id_uuid_text_round_trips() {
        let id = CallId::from_bytes([
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ]);
        let text = id.uuid_string();
        assert_eq!(text, "00112233-4455-6677-8899-aabbccddeeff");
        assert_eq!(CallId::from_uuid_string(&text), Some(id));
        assert_eq!(
            CallId::from_uuid_string("00112233-4455-6677-8899-AABBCCDDEEFF"),
            Some(id)
        );
    }

    #[test]
    fn call_id_rejects_malformed_uuid_text() {
        for text in [
            "",
            "00112233-4455-6677-8899-aabbccddeefg",
            "001122334455-6677-8899-aabbccddeeff",
            "00112233-4455-6677-8899-aabbccddeeff-",
        ] {
            assert_eq!(CallId::from_uuid_string(text), None);
        }
    }

    #[test]
    fn provider_config_rejects_empty_or_ambiguous_values() {
        assert_eq!(
            CallProviderConfig::new("  ").validate(),
            Err(CallProviderError::InvalidInput)
        );
        assert_eq!(
            CallProviderConfig::new("Example")
                .with_supported_handles(Vec::new())
                .validate(),
            Err(CallProviderError::InvalidInput)
        );
        assert_eq!(
            CallProviderConfig::new("Example")
                .with_supported_handles(vec![CallHandleKind::Generic, CallHandleKind::Generic])
                .validate(),
            Err(CallProviderError::InvalidInput)
        );
        assert_eq!(
            CallProviderConfig::new("Example")
                .with_maximum_call_groups(0)
                .validate(),
            Err(CallProviderError::InvalidInput)
        );
    }

    #[test]
    fn incoming_call_requires_supported_handle_and_video_config() {
        let config = CallProviderConfig::new("Example");
        let id = CallId::from_bytes([1; 16]);
        assert_eq!(
            IncomingCall::new(id, CallHandleKind::Generic, "").validate(&config),
            Err(CallProviderError::InvalidInput)
        );
        assert_eq!(
            IncomingCall::new(id, CallHandleKind::PhoneNumber, "+15555550100").validate(&config),
            Err(CallProviderError::InvalidInput)
        );
        assert_eq!(
            IncomingCall::new(id, CallHandleKind::Generic, "call:1")
                .with_video(true)
                .validate(&config),
            Err(CallProviderError::InvalidInput)
        );
        assert!(
            IncomingCall::new(id, CallHandleKind::Generic, "call:1")
                .validate(&config)
                .is_ok()
        );
    }
}

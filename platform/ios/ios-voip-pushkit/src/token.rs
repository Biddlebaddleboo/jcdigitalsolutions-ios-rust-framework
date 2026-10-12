use std::vec::Vec;

/// The opaque byte token PushKit supplies for the VoIP notification type.
///
/// The bytes are not text or a stable account identifier. Forward them only to the app's APNs
/// provider over an authenticated, encrypted channel; do not log or persist them as plain text.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VoipPushToken(Vec<u8>);

impl VoipPushToken {
    /// Creates a token from exact PushKit token bytes.
    ///
    /// This constructor does not imply that the token was issued by PushKit or accepted by APNs.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the token bytes without changing their representation.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Transfers ownership of the exact token bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

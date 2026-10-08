//! Owned values for the result of a local-notification interaction.
//!
//! This module models response data only. It does not receive, deliver, route, or observe
//! notification responses.

use alloc::string::String;
use framework_core::ErrorKind;

use super::NotificationId;

/// A caller-visible identifier for a custom notification action.
///
/// Action identifiers are exact, case-sensitive UTF-8 strings. Construction rejects an empty
/// value or NUL byte and performs no normalization, prefixing, or truncation. The action need not
/// have been configured by this crate; action and category registration are outside this module.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NotificationActionId(String);

impl NotificationActionId {
    /// Takes ownership of a non-empty action identifier that contains no NUL byte.
    pub fn new(value: String) -> Result<Self, NotificationResponseError> {
        if value.is_empty() || value.as_bytes().contains(&0) {
            return Err(NotificationResponseError::InvalidActionIdentifier);
        }
        Ok(Self(value))
    }

    /// Borrows the exact action identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Transfers the owned action identifier to the caller.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// A validation error for owned notification-response values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NotificationResponseError {
    /// A custom or text-input action identifier is empty or contains a NUL byte.
    InvalidActionIdentifier,
}

impl NotificationResponseError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidActionIdentifier => ErrorKind::InvalidInput,
        }
    }
}

/// The semantic kind and owned data for a local-notification interaction.
///
/// `Default` represents the standard/default activation. `Dismiss` represents a dismissal.
/// Custom actions carry their caller-visible action identifier. Text-input actions carry that
/// identifier and the submitted user text. No operating-system action strings or action-definition
/// objects are exposed.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NotificationResponseKind {
    /// The user selected the standard/default notification activation.
    Default,
    /// The user dismissed the notification.
    Dismiss,
    /// The user selected a custom action with this exact identifier.
    CustomAction(NotificationActionId),
    /// The user submitted text for a custom text-input action.
    TextInput {
        /// The exact identifier of the text-input action.
        action_id: NotificationActionId,
        /// The submitted owned UTF-8 text, including an empty value if submitted.
        text: String,
    },
}

impl NotificationResponseKind {
    /// Borrows the custom action identifier, if this response kind carries one.
    pub const fn action_id(&self) -> Option<&NotificationActionId> {
        match self {
            Self::Default | Self::Dismiss => None,
            Self::CustomAction(action_id) | Self::TextInput { action_id, .. } => Some(action_id),
        }
    }

    /// Borrows submitted text for a text-input response, or returns `None` for other kinds.
    pub fn user_text(&self) -> Option<&str> {
        match self {
            Self::TextInput { text, .. } => Some(text),
            Self::Default | Self::Dismiss | Self::CustomAction(_) => None,
        }
    }
}

/// An owned value for one interaction with a local notification.
///
/// The notification ID is D3's validated, exact ID type. Response kind data is owned and can be
/// moved into or out of this value without a response-layer copy. Constructing this value does not
/// imply that a notification was delivered or that any callback or delegate ran.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NotificationResponse {
    notification_id: NotificationId,
    kind: NotificationResponseKind,
}

impl NotificationResponse {
    /// Takes ownership of a validated notification ID and response kind.
    /// See the `notification-responses.md` guide for a construction example.
    pub const fn new(notification_id: NotificationId, kind: NotificationResponseKind) -> Self {
        Self {
            notification_id,
            kind,
        }
    }

    /// Borrows the validated notification ID associated with this response.
    pub const fn notification_id(&self) -> &NotificationId {
        &self.notification_id
    }

    /// Borrows the response kind and its owned action or text data.
    pub const fn kind(&self) -> &NotificationResponseKind {
        &self.kind
    }

    /// Transfers the notification ID and response data to the caller without a copy.
    pub fn into_parts(self) -> (NotificationId, NotificationResponseKind) {
        (self.notification_id, self.kind)
    }
}

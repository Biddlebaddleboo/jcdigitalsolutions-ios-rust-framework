#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "User-mediated single-contact selection through public ContactsUI APIs."]

/// A terminal result from one user-mediated ContactsUI selection session.
#[cfg(any(test, target_os = "ios"))]
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContactPickerEvent {
    /// The user selected one contact; its common fields and local identifier are owned by Rust.
    Selected(framework_contacts::Contact),
    /// The user cancelled the picker or the caller explicitly cancelled the session.
    Cancelled,
}

#[cfg(any(test, target_os = "ios"))]
mod session_policy;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::IosContactPickerSession;

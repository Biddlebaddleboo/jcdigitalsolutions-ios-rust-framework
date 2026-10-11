#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Bounded UIKit acknowledgement-alert and destructive-confirmation presentation for iOS."]

#[cfg(any(target_os = "ios", test))]
mod callback;
#[cfg(any(target_os = "ios", test))]
mod policy;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    DestructiveConfirmationError, DestructiveConfirmationStartError, DestructiveConfirmationText,
    PopoverAnchor, PresentationError, present_acknowledgement, present_destructive_confirmation,
};

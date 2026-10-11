#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A Rust-owned, iOS-only CallKit provider for incoming-call reports and answer/end actions."]
#![doc = "\n\nThe crate exposes framework-owned call identifiers and events. It does not implement PushKit, APNs, audio, call placement, or application background execution."]

#[cfg(any(test, target_os = "ios"))]
mod registry;
mod types;

#[cfg(target_os = "ios")]
mod platform;

pub use types::{
    ActionDecision, CallEndReason, CallHandleKind, CallId, CallProviderConfig, CallProviderError,
    CallProviderEvent, EndCallOutcome, IncomingCall,
};

#[cfg(target_os = "ios")]
pub use platform::CallProvider;

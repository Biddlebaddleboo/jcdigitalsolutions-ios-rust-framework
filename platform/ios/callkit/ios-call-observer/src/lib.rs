#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A one-shot, iOS-only snapshot of active CallKit call state."]
#![doc = "\n\nThe snapshot contains only a fixed-width call count and aggregate state flags. It does not expose CallKit objects, call UUIDs, caller data, callbacks, call control, or a portable contract. The initial `CXCallObserver.calls` read may block. Call only on iOS 10.0 or later and keep the call off UI-critical work."]

#[cfg(target_os = "ios")]
mod platform;
#[cfg(target_os = "ios")]
mod snapshot;

#[cfg(target_os = "ios")]
pub use platform::active_call_snapshot;
#[cfg(target_os = "ios")]
pub use snapshot::{
    CALL_STATE_CONNECTED, CALL_STATE_ENDED, CALL_STATE_ON_HOLD, CALL_STATE_OUTGOING,
    CallActivitySnapshot,
};

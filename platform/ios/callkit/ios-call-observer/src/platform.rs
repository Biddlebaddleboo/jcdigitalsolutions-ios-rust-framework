use objc2_call_kit::CXCallObserver;

use crate::{
    CALL_STATE_CONNECTED, CALL_STATE_ENDED, CALL_STATE_ON_HOLD, CALL_STATE_OUTGOING,
    CallActivitySnapshot,
};

/// Read and copy a one-shot summary of the active calls returned by CallKit.
///
/// This synchronous query constructs a `CXCallObserver` and reads its `calls` property. CallKit
/// may block while it retrieves the initial call state, so do not call this on a UI-critical
/// thread. The result contains only a `u64` count and `u32` aggregate flags. No native object,
/// UUID, caller data, callback, or call-control handle escapes this function. Call only on iOS
/// 10.0 or later.
pub fn active_call_snapshot() -> CallActivitySnapshot {
    // SAFETY: This function is compiled only for iOS. The caller must set an iOS 10.0+ deployment
    // target, matching CXCallObserver's availability. The constructor takes no caller-owned args.
    let observer = unsafe { CXCallObserver::new() };
    // SAFETY: The same iOS 10.0+ availability applies. The returned NSArray is retained and is
    // immutable by API contract for this local snapshot read.
    let calls = unsafe { observer.calls() };
    let count = calls.len() as u64;
    let mut flags = 0;

    for index in 0..calls.len() {
        let call = calls.objectAtIndex(index);

        // SAFETY: These generated accessors read scalar CXCall properties from a retained object
        // returned by CXCallObserver.calls, on the same thread that owns the observer and array.
        unsafe {
            if call.isOutgoing() {
                flags |= CALL_STATE_OUTGOING;
            }
            if call.hasConnected() {
                flags |= CALL_STATE_CONNECTED;
            }
            if call.isOnHold() {
                flags |= CALL_STATE_ON_HOLD;
            }
            if call.hasEnded() {
                flags |= CALL_STATE_ENDED;
            }
        }
    }

    CallActivitySnapshot::from_parts(count, flags)
}

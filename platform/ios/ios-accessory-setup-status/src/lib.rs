#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A one-shot AccessorySetupKit previously-selected accessory count for iOS"]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
mod platform;

/// An error while requesting an AccessorySetupKit accessory count
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AccessorySetupStatusError {
    /// The current target is not iOS
    NativeApiUnavailable,
    /// The session was invalidated before its activation snapshot arrived
    SessionInvalidated,
}

/// Requests the count of accessories previously selected for the containing app
///
/// On iOS 18.0 and later, this creates and activates an `ASAccessorySession`, then reads only the
/// count of its `accessories` array after the `.activated` event on the main dispatch queue. The
/// callback runs once on that queue. This function never presents a picker, discovers nearby
/// accessories, or reads accessory names or identifiers. A successful result is only a point-in-
/// time count of previously selected app accessories; it does not mean they are nearby, connected,
/// authorized for every transport, or ready for use. The host app must configure AccessorySetupKit
/// in its information property list and set its minimum iOS deployment target to 18.0 because the
/// typed framework binding links AccessorySetupKit strongly. Non-iOS targets invoke the callback
/// with `NativeApiUnavailable`
#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub fn request_previously_selected_accessory_count<F>(completion: F)
where
    F: FnOnce(Result<usize, AccessorySetupStatusError>) + Send + 'static,
{
    platform::request_previously_selected_accessory_count(completion)
}

/// Requests the count of accessories previously selected for the containing app
///
/// Non-iOS and Mac Catalyst targets invoke the callback immediately with `NativeApiUnavailable`
#[cfg(not(all(target_os = "ios", not(target_abi = "macabi"))))]
pub fn request_previously_selected_accessory_count<F>(completion: F)
where
    F: FnOnce(Result<usize, AccessorySetupStatusError>) + Send + 'static,
{
    completion(Err(AccessorySetupStatusError::NativeApiUnavailable));
}

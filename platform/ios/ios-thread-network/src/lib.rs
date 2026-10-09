#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A read-only preferred Thread network availability query for entitled iOS apps."]

mod platform;

pub use platform::PreferredThreadNetworkAvailabilityFuture;

/// Starts a one-shot query for preferred Thread network availability.
///
/// The native request starts when this function runs, not when the returned future is first
/// polled. Its Boolean result means only that `THClient` reports whether a preferred Thread
/// network is available; it does not report Thread radio support, active connectivity, or
/// border-router capability. The host app must have Apple's
/// `com.apple.developer.networking.manage-thread-network-credentials` entitlement, with Apple's
/// distribution access when publishing. Dropping the future abandons Rust interest and does not
/// guarantee cancellation of the native operation.
pub fn request_preferred_network_availability() -> PreferredThreadNetworkAvailabilityFuture {
    platform::start()
}

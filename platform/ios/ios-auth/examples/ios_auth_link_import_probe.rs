#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_auth::{
    AppTrackingAuthorizationBackend, AuthenticationBackend, AuthenticationPolicy,
};
#[cfg(target_os = "ios")]
use ios_auth::{IosAppTrackingAuthorizationBackend, IosAuthenticationBackend};

#[cfg(target_os = "ios")]
fn main() {
    let backend = IosAuthenticationBackend::new();
    let _ = core::hint::black_box(backend.availability(AuthenticationPolicy::BiometricsOnly));
    let tracking = IosAppTrackingAuthorizationBackend::new();
    let _ = core::hint::black_box(tracking.status());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

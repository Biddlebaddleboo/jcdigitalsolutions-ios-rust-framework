#![deny(missing_docs)]
#![doc = "iOS CloudKit account-status and privacy-minimal iCloud Drive identity-presence backends."]

#[cfg(target_os = "ios")]
mod cloudkit;
#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use cloudkit::{IosCloudAccountBackend, IosCloudAccountStatusFuture};
#[cfg(target_os = "ios")]
pub use platform::IosUbiquityIdentityBackend;

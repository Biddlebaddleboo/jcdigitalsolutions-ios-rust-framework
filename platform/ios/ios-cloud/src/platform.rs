use framework_cloud::{UbiquityIdentityBackend, UbiquityIdentitySnapshot};
use objc2_foundation::NSFileManager;

/// A stateless iOS backend for synchronous iCloud Drive identity-token presence checks.
pub struct IosUbiquityIdentityBackend;

impl IosUbiquityIdentityBackend {
    /// Creates a backend without querying Foundation or changing host configuration.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for IosUbiquityIdentityBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl UbiquityIdentityBackend for IosUbiquityIdentityBackend {
    fn snapshot(&self) -> UbiquityIdentitySnapshot {
        let token_present = NSFileManager::defaultManager()
            .ubiquityIdentityToken()
            .is_some();
        UbiquityIdentitySnapshot::from_token_presence(token_present)
    }
}

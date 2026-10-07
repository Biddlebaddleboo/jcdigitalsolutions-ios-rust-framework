use core::num::NonZeroU16;

/// A semantic platform identity. It does not expose a native handle or API type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum Platform {
    /// The target platform is not known to this framework version.
    Unknown = 0,
    /// Apple iOS.
    Ios = 1,
    /// Android.
    Android = 2,
    /// Apple macOS.
    MacOS = 3,
    /// Microsoft Windows.
    Windows = 4,
    /// Linux.
    Linux = 5,
    /// A WebAssembly target with browser-facing semantics.
    Web = 6,
    /// A known target without a dedicated semantic variant.
    Other = 255,
}

impl Platform {
    /// Returns the compile-time target classification, or `Unknown` when no variant applies.
    pub const fn current() -> Self {
        #[cfg(target_os = "ios")]
        {
            return Self::Ios;
        }
        #[cfg(target_os = "android")]
        {
            return Self::Android;
        }
        #[cfg(target_os = "macos")]
        {
            return Self::MacOS;
        }
        #[cfg(target_os = "windows")]
        {
            return Self::Windows;
        }
        #[cfg(target_os = "linux")]
        {
            return Self::Linux;
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            return Self::Web;
        }
        #[allow(unreachable_code)]
        Self::Unknown
    }
}

/// A known framework capability. Numeric values are stable semantic IDs within V1.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u16)]
pub enum Capability {
    /// File access and file operations.
    Files = 1,
    /// Non-secure user preferences.
    Preferences = 2,
    /// Protected credential and secret storage.
    SecureStorage = 3,
    /// HTTP and related network requests.
    Network = 4,
    /// User-visible notifications.
    Notifications = 5,
    /// Device location.
    Location = 6,
    /// Camera access and capture.
    Camera = 7,
    /// Audio input and output.
    Audio = 8,
    /// Bluetooth communication.
    Bluetooth = 9,
    /// Clipboard access.
    Clipboard = 10,
    /// User authentication.
    Authentication = 11,
    /// Background work scheduled under platform policy.
    BackgroundWork = 12,
    /// Motion sensors and activity data.
    Motion = 13,
    /// Native user-interface and window operations.
    Ui = 14,
    /// Native share-sheet or equivalent operations.
    Share = 15,
    /// Native payment or purchase operations.
    Payments = 16,
}

impl Capability {
    /// Returns the fixed-width identifier for this known capability.
    pub const fn id(self) -> CapabilityId {
        match NonZeroU16::new(self as u16) {
            Some(value) => CapabilityId(value),
            None => panic!("known capability IDs are nonzero"),
        }
    }
}

/// A nonzero fixed-width capability identifier, including IDs unknown to this version.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct CapabilityId(NonZeroU16);

impl CapabilityId {
    /// Creates an ID from a nonzero `u16` value.
    pub const fn new(raw: u16) -> Option<Self> {
        match NonZeroU16::new(raw) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the fixed-width numeric ID.
    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

/// A portable summary of whether a capability can be used in the current context.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum Availability {
    /// The backend has not determined availability.
    Unknown = 0,
    /// The capability is currently usable.
    Available = 1,
    /// The capability does not exist on this platform or target.
    Unsupported = 2,
    /// User permission is required before use.
    RequiresPermission = 3,
    /// A platform entitlement or privileged configuration is required.
    RequiresEntitlement = 4,
    /// The capability exists but is temporarily unavailable.
    TemporarilyUnavailable = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_ids_are_nonzero_and_fixed_width() {
        assert_eq!(Capability::Files.id().get(), 1);
        assert_eq!(Capability::Payments.id().get(), 16);
        assert_eq!(CapabilityId::new(u16::MAX).unwrap().get(), u16::MAX);
    }

    #[test]
    fn current_platform_is_a_compile_time_value() {
        #[cfg(target_os = "ios")]
        assert_eq!(Platform::current(), Platform::Ios);
        #[cfg(target_os = "android")]
        assert_eq!(Platform::current(), Platform::Android);
        #[cfg(target_os = "macos")]
        assert_eq!(Platform::current(), Platform::MacOS);
        #[cfg(target_os = "windows")]
        assert_eq!(Platform::current(), Platform::Windows);
        #[cfg(target_os = "linux")]
        assert_eq!(Platform::current(), Platform::Linux);
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        assert_eq!(Platform::current(), Platform::Web);
    }
}

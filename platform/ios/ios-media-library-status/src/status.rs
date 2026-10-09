/// The current permission state for access to a user's MediaPlayer library.
///
/// `Restricted` is preserved as a distinct state because Apple documents that the app may access
/// some library content in that state. An unrecognized native status is retained as its signed
/// 64-bit raw value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaLibraryAuthorizationStatus {
    /// The user has not yet chosen whether to authorize library access.
    NotDetermined,
    /// The app may not access library items.
    Denied,
    /// The app may access some library content.
    Restricted,
    /// The app may access library items.
    Authorized,
    /// The OS returned a status value not represented by the known variants.
    Unknown(i64),
}

impl MediaLibraryAuthorizationStatus {
    /// Convert a native status raw value while preserving unknown values.
    pub const fn from_raw_value(raw_value: i64) -> Self {
        match raw_value {
            0 => Self::NotDetermined,
            1 => Self::Denied,
            2 => Self::Restricted,
            3 => Self::Authorized,
            other => Self::Unknown(other),
        }
    }

    /// Return the signed 64-bit raw status value.
    pub const fn raw_value(self) -> i64 {
        match self {
            Self::NotDetermined => 0,
            Self::Denied => 1,
            Self::Restricted => 2,
            Self::Authorized => 3,
            Self::Unknown(raw_value) => raw_value,
        }
    }
}

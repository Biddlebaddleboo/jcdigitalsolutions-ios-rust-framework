use core::num::NonZeroI32;

/// The stable, allocation-free category for a framework error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum ErrorKind {
    /// The cause is not classified by this version.
    Unknown = 0,
    /// An argument or value is invalid for the requested operation.
    InvalidInput = 1,
    /// The target platform does not support the requested operation.
    Unsupported = 2,
    /// The operation or resource is unavailable in the current context.
    Unavailable = 3,
    /// User authorization was denied or is insufficient.
    PermissionDenied = 4,
    /// The operation was cancelled.
    Cancelled = 5,
    /// The operation exceeded a caller or platform time limit.
    Timeout = 6,
    /// The requested resource does not exist.
    NotFound = 7,
    /// A resource with the requested identity already exists.
    AlreadyExists = 8,
    /// A bounded resource or allocation could not be obtained.
    ResourceExhausted = 9,
    /// A platform reported an error that has no more specific portable category.
    Platform = 10,
    /// An internal framework invariant or operation failed.
    Internal = 11,
}

/// A nonzero platform error code, kept separate from portable error semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct PlatformErrorCode(NonZeroI32);

impl PlatformErrorCode {
    /// Creates a platform code; zero is reserved to mean that no code was supplied.
    pub const fn new(code: i32) -> Option<Self> {
        match NonZeroI32::new(code) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the original signed 32-bit platform code.
    pub const fn get(self) -> i32 {
        self.0.get()
    }
}

/// An allocation-free framework error with an optional native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    platform_code: Option<PlatformErrorCode>,
}

impl Error {
    /// Creates an error with a portable category and no platform code.
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            platform_code: None,
        }
    }

    /// Attaches a platform code without replacing the portable category.
    pub const fn with_platform_code(mut self, code: PlatformErrorCode) -> Self {
        self.platform_code = Some(code);
        self
    }

    /// Returns the portable category.
    pub const fn kind(self) -> ErrorKind {
        self.kind
    }

    /// Returns the optional platform code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        self.platform_code
    }
}

/// The portable result type used by framework APIs.
pub type Result<T> = core::result::Result<T, Error>;

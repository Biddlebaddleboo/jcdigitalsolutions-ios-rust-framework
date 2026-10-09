#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable read-only packaged-resource lookup and a static backend contract."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A validated UTF-8 slash-separated path relative to the packaged-resource root.
///
/// The path is borrowed exactly as supplied and is never normalized. Unicode normalization and
/// case folding are not performed or promised.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourcePath<'a> {
    relative: &'a str,
}

impl<'a> ResourcePath<'a> {
    /// Creates a relative resource path without normalizing its text.
    pub fn new(relative: &'a str) -> Result<Self, ResourceError> {
        if !valid_relative_path(relative) {
            return Err(ResourceError::InvalidPath);
        }
        Ok(Self { relative })
    }

    /// Returns the caller-borrowed relative path text.
    pub const fn relative(self) -> &'a str {
        self.relative
    }
}

fn valid_relative_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.ends_with('/') {
        return false;
    }
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return false;
    }
    if bytes.iter().any(|&byte| byte == 0 || byte == b'\\') {
        return false;
    }
    path.split('/')
        .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// A stable resource-facade error that preserves portable category and native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ResourceError {
    /// The path is empty, absolute, drive-prefixed, or contains a forbidden segment or character.
    InvalidPath,
    /// Resource bytes were not valid UTF-8 for a string read.
    InvalidUtf8,
    /// The selected backend returned a framework error, including `NotFound` for absent paths.
    Backend(Error),
}

impl ResourceError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidPath | Self::InvalidUtf8 => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidPath | Self::InvalidUtf8 => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for read-only packaged-resource lookup.
///
/// Methods are synchronous and may block. The backend owns platform availability, bundle
/// location, file-system security, and resource-copy costs. It must report an absent resource as
/// `ErrorKind::NotFound`. This contract does not provide cancellation, directory enumeration,
/// arbitrary URL lookup, localized-resource selection, asset-catalog access, or resource-format
/// decoding.
pub trait ResourceBackend {
    /// Reports whether packaged-resource lookup is usable in the current backend context.
    fn availability(&self) -> Availability;

    /// Reads one exact relative path into caller-owned bytes.
    fn read(&mut self, path: ResourcePath<'_>) -> Result<Vec<u8>, ResourceError>;
}

/// A thin facade over caller-owned, statically selected resource-backend state.
pub struct Resources<B> {
    backend: B,
}

impl<B: ResourceBackend> Resources<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Reads exact-path resource bytes into a caller-owned vector.
    pub fn read(&mut self, path: ResourcePath<'_>) -> Result<Vec<u8>, ResourceError> {
        self.backend.read(path)
    }

    /// Reads exact-path resource bytes and transfers their allocation into a UTF-8 string.
    ///
    /// This conversion consumes the returned vector and makes no second facade-level byte copy.
    pub fn read_string(&mut self, path: ResourcePath<'_>) -> Result<String, ResourceError> {
        String::from_utf8(self.read(path)?).map_err(|_| ResourceError::InvalidUtf8)
    }

    /// Borrows the selected backend.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the selected backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

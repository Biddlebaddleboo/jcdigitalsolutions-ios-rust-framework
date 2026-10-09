#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable contract for one bounded P-256 public-key algorithm-suitability query."]

use framework_core::{Error, ErrorKind, Result};

/// The byte length of an uncompressed ANSI X9.63 P-256 public key.
pub const P256_PUBLIC_KEY_X963_LEN: usize = 65;

/// A borrowed, caller-owned uncompressed ANSI X9.63 P-256 public key.
///
/// This validates only the uncompressed-point marker. The backend validates the point when it
/// imports the key. The bytes represent a public key, not a private key or secret.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct P256PublicKey<'a> {
    x963_bytes: &'a [u8; P256_PUBLIC_KEY_X963_LEN],
}

impl<'a> P256PublicKey<'a> {
    /// Creates a borrowed key value when the bytes use the X9.63 uncompressed-point marker.
    ///
    /// This does not validate the curve point. A backend can reject invalid point coordinates.
    pub const fn from_x963_uncompressed(
        x963_bytes: &'a [u8; P256_PUBLIC_KEY_X963_LEN],
    ) -> Result<Self> {
        if x963_bytes[0] == 0x04 {
            Ok(Self { x963_bytes })
        } else {
            Err(Error::new(ErrorKind::InvalidInput))
        }
    }

    /// Returns the borrowed X9.63 bytes without copying them.
    pub const fn as_x963_uncompressed_bytes(self) -> &'a [u8; P256_PUBLIC_KEY_X963_LEN] {
        self.x963_bytes
    }
}

/// The result of a platform query for P-256 ECDSA/SHA-256 message verification suitability.
///
/// `true` reports only that the backend considers the imported public key suitable for this
/// operation. It does not verify a signature or guarantee that a later verification call succeeds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct P256VerificationSupport {
    is_supported: bool,
}

impl P256VerificationSupport {
    /// Creates a value from the platform-reported suitability result.
    pub const fn from_platform_query(is_supported: bool) -> Self {
        Self { is_supported }
    }

    /// Returns whether the platform reports the selected key/operation/algorithm as suitable.
    pub const fn is_supported(self) -> bool {
        self.is_supported
    }
}

/// A statically selected backend for the single P-256 ECDSA/SHA-256 verification query.
pub trait P256VerificationSupportBackend {
    /// Imports the supplied public key for this query and reports algorithm suitability.
    ///
    /// The operation is only `Verify` with the ECDSA/SHA-256 message algorithm. Implementations
    /// must not generate a private key, persist the imported public key, or perform verification.
    fn query_p256_ecdsa_sha256_message_verification(
        &self,
        public_key: P256PublicKey<'_>,
    ) -> Result<P256VerificationSupport>;
}

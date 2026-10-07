#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable preference keys, byte values, and a static backend contract."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A validated UTF-8 preference key borrowed for one backend call.
///
/// Keys are exact and case-sensitive. The facade performs no Unicode normalization or prefixing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PreferenceKey<'a>(&'a str);

impl<'a> PreferenceKey<'a> {
    /// Creates a non-empty key that contains no NUL byte.
    pub fn new(value: &'a str) -> Result<Self, PreferenceError> {
        if value.is_empty() || value.as_bytes().contains(&0) {
            return Err(PreferenceError::InvalidKey);
        }
        Ok(Self(value))
    }

    /// Returns the original caller-borrowed key text.
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// A stable preference-facade error that preserves portable category and native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PreferenceError {
    /// The key is empty or contains a NUL byte.
    InvalidKey,
    /// Stored bytes were not valid UTF-8 for a string read.
    InvalidUtf8,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl PreferenceError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidKey | Self::InvalidUtf8 => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidKey | Self::InvalidUtf8 => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// The required atomicity for one preference-key update.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AtomicityRequirement {
    /// Accept either an atomic or a non-atomic backend update.
    AllowNonAtomic,
    /// Fail with `Unsupported` unless the backend can provide atomic key replacement.
    RequireAtomic,
}

/// The atomicity a backend reports for one preference-key update.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UpdateAtomicity {
    /// Readers of this store see the complete old or complete new value for this one key.
    Atomic,
    /// The backend makes no non-torn update guarantee.
    NotGuaranteed,
}

/// Options for a single preference-key update.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WriteOptions {
    atomicity: AtomicityRequirement,
}

impl WriteOptions {
    /// Creates options with the requested one-key atomicity requirement.
    pub const fn new(atomicity: AtomicityRequirement) -> Self {
        Self { atomicity }
    }

    /// Creates options that accept the backend's reported atomicity.
    pub const fn allow_non_atomic() -> Self {
        Self::new(AtomicityRequirement::AllowNonAtomic)
    }

    /// Returns the atomicity requirement.
    pub const fn atomicity(self) -> AtomicityRequirement {
        self.atomicity
    }
}

/// The backend's reported result for one preference-key update.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WriteOutcome {
    atomicity: UpdateAtomicity,
}

impl WriteOutcome {
    /// Creates a write result with the backend-reported atomicity.
    pub const fn new(atomicity: UpdateAtomicity) -> Self {
        Self { atomicity }
    }

    /// Returns the backend-reported visibility guarantee, not a crash-durability promise.
    pub const fn atomicity(self) -> UpdateAtomicity {
        self.atomicity
    }
}

/// A compile-time-selected backend for non-secure preference values.
///
/// A successful set updates one key only; a later get through the same backend returns that value
/// unless another writer changes the key. No multi-key transaction or crash-durability guarantee
/// exists in this contract. The backend may copy a borrowed value but must not retain the borrow.
pub trait PreferencesBackend {
    /// Reports whether non-secure preferences are usable in the current context.
    fn availability(&self) -> Availability;

    /// Reads an owned byte value, or `None` when the key has no value.
    fn get_bytes(&mut self, key: PreferenceKey<'_>) -> Result<Option<Vec<u8>>, PreferenceError>;

    /// Sets one key and reports the atomicity the backend can guarantee.
    ///
    /// If `RequireAtomic` is set but unsupported, the backend must return `Unsupported`. A
    /// successful set makes this key's value visible to a later get through the same backend,
    /// unless another writer changes it. Persistence across a crash is not promised.
    fn set_bytes(
        &mut self,
        key: PreferenceKey<'_>,
        value: &[u8],
        options: WriteOptions,
    ) -> Result<WriteOutcome, PreferenceError>;

    /// Removes one key and reports whether a value existed.
    fn remove(&mut self, key: PreferenceKey<'_>) -> Result<bool, PreferenceError>;
}

/// A thin preference facade over a caller-owned, statically selected backend.
pub struct Preferences<B> {
    backend: B,
}

impl<B: PreferencesBackend> Preferences<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without performing a global lookup.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Reads an owned byte vector; the facade adds no copy after the backend result.
    pub fn get_bytes(
        &mut self,
        key: PreferenceKey<'_>,
    ) -> Result<Option<Vec<u8>>, PreferenceError> {
        self.backend.get_bytes(key)
    }

    /// Reads owned UTF-8 text by transferring the backend vector into a `String` without a
    /// second facade-level byte copy.
    pub fn get_string(
        &mut self,
        key: PreferenceKey<'_>,
    ) -> Result<Option<String>, PreferenceError> {
        self.get_bytes(key)?
            .map(|bytes| String::from_utf8(bytes).map_err(|_| PreferenceError::InvalidUtf8))
            .transpose()
    }

    /// Sets one key from a borrowed byte slice; the backend may copy it but may not retain it.
    pub fn set_bytes(
        &mut self,
        key: PreferenceKey<'_>,
        value: &[u8],
        options: WriteOptions,
    ) -> Result<WriteOutcome, PreferenceError> {
        self.backend.set_bytes(key, value, options)
    }

    /// Sets one key from borrowed UTF-8 bytes without a string transcode or facade copy.
    pub fn set_string(
        &mut self,
        key: PreferenceKey<'_>,
        value: &str,
        options: WriteOptions,
    ) -> Result<WriteOutcome, PreferenceError> {
        self.set_bytes(key, value.as_bytes(), options)
    }

    /// Removes one key and reports whether it existed.
    pub fn remove(&mut self, key: PreferenceKey<'_>) -> Result<bool, PreferenceError> {
        self.backend.remove(key)
    }

    /// Borrows the backend for preference types not yet modeled by this facade.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for preference types not yet modeled by this facade.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    struct Backend {
        value: Option<Vec<u8>>,
        atomic: UpdateAtomicity,
    }

    impl PreferencesBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        fn get_bytes(
            &mut self,
            _key: PreferenceKey<'_>,
        ) -> Result<Option<Vec<u8>>, PreferenceError> {
            Ok(self.value.clone())
        }

        fn set_bytes(
            &mut self,
            _key: PreferenceKey<'_>,
            value: &[u8],
            options: WriteOptions,
        ) -> Result<WriteOutcome, PreferenceError> {
            if options.atomicity() == AtomicityRequirement::RequireAtomic
                && self.atomic != UpdateAtomicity::Atomic
            {
                return Err(PreferenceError::Backend(Error::new(ErrorKind::Unsupported)));
            }
            self.value = Some(value.to_vec());
            Ok(WriteOutcome::new(self.atomic))
        }

        fn remove(&mut self, _key: PreferenceKey<'_>) -> Result<bool, PreferenceError> {
            Ok(self.value.take().is_some())
        }
    }

    #[test]
    fn keys_are_exact_and_nonempty() {
        assert_eq!(PreferenceKey::new("theme").unwrap().as_str(), "theme");
        assert_eq!(PreferenceKey::new(""), Err(PreferenceError::InvalidKey));
        assert_eq!(PreferenceKey::new("a\0b"), Err(PreferenceError::InvalidKey));
    }

    #[test]
    fn strings_and_bytes_have_owned_read_and_borrowed_write_contracts() {
        let mut prefs = Preferences::new(Backend {
            value: None,
            atomic: UpdateAtomicity::Atomic,
        });
        let key = PreferenceKey::new("theme").unwrap();
        let outcome = prefs
            .set_string(key, "dark", WriteOptions::allow_non_atomic())
            .unwrap();
        assert_eq!(outcome.atomicity(), UpdateAtomicity::Atomic);
        let key = PreferenceKey::new("theme").unwrap();
        assert_eq!(prefs.get_string(key), Ok(Some(String::from("dark"))));
        let key = PreferenceKey::new("theme").unwrap();
        assert!(prefs.remove(key).unwrap());
    }

    #[test]
    fn malformed_utf8_and_failed_atomicity_are_reported() {
        let mut prefs = Preferences::new(Backend {
            value: Some(vec![0xff]),
            atomic: UpdateAtomicity::NotGuaranteed,
        });
        let key = PreferenceKey::new("data").unwrap();
        assert_eq!(prefs.get_string(key), Err(PreferenceError::InvalidUtf8));
        let key = PreferenceKey::new("data").unwrap();
        let options = WriteOptions::new(AtomicityRequirement::RequireAtomic);
        assert_eq!(
            prefs.set_bytes(key, b"next", options),
            Err(PreferenceError::Backend(Error::new(ErrorKind::Unsupported)))
        );
        assert_eq!(prefs.backend().value.as_deref(), Some(&[0xff][..]));
    }
}

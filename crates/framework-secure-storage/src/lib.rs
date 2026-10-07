#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable opaque-byte secure-storage contract with static backend selection."]

extern crate alloc;

use alloc::vec::Vec;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A validated, exact service identifier borrowed for one backend call.
///
/// Service identifiers are UTF-8 text. This type rejects empty strings and NUL bytes without
/// normalization, case folding, prefixing, or copying.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ServiceId<'a>(&'a str);

impl<'a> ServiceId<'a> {
    /// Creates a non-empty service identifier that contains no NUL byte.
    pub fn new(value: &'a str) -> Result<Self, SecureStorageError> {
        if value.is_empty() || value.as_bytes().contains(&0) {
            return Err(SecureStorageError::InvalidServiceId);
        }
        Ok(Self(value))
    }

    /// Returns the original caller-borrowed service identifier.
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// A validated, exact item identifier borrowed for one backend call.
///
/// Item identifiers are UTF-8 text. This type rejects empty strings and NUL bytes without
/// normalization, case folding, prefixing, or copying.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ItemId<'a>(&'a str);

impl<'a> ItemId<'a> {
    /// Creates a non-empty item identifier that contains no NUL byte.
    pub fn new(value: &'a str) -> Result<Self, SecureStorageError> {
        if value.is_empty() || value.as_bytes().contains(&0) {
            return Err(SecureStorageError::InvalidItemId);
        }
        Ok(Self(value))
    }

    /// Returns the original caller-borrowed item identifier.
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// A portable minimum access policy for one stored secret.
///
/// A required `true` property must be honored by the backend. A backend may apply a stronger
/// effective policy; callers should inspect the reported policy because stronger restrictions can
/// reduce availability. `device_unlock_required` means reads require the device to be unlocked;
/// it does not request biometric or per-access user authentication. `device_bound` means the item
/// must not be made available by restore or migration to a different device.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct AccessPolicy {
    device_unlock_required: bool,
    device_bound: bool,
}

impl AccessPolicy {
    /// Creates a policy from the device-unlock and same-device requirements.
    pub const fn new(device_unlock_required: bool, device_bound: bool) -> Self {
        Self {
            device_unlock_required,
            device_bound,
        }
    }

    /// Creates a policy that requires neither device unlock nor same-device storage.
    pub const fn unrestricted() -> Self {
        Self::new(false, false)
    }

    /// Returns whether the device must be unlocked for reads.
    pub const fn device_unlock_required(self) -> bool {
        self.device_unlock_required
    }

    /// Returns whether the stored item must remain on one device.
    pub const fn device_bound(self) -> bool {
        self.device_bound
    }

    /// Reports whether this effective policy satisfies every requirement in `required`.
    pub const fn satisfies(self, required: Self) -> bool {
        (!required.device_unlock_required || self.device_unlock_required)
            && (!required.device_bound || self.device_bound)
    }
}

/// A stable secure-storage error with an optional native status code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum SecureStorageError {
    /// The service identifier is empty or contains a NUL byte.
    InvalidServiceId,
    /// The item identifier is empty or contains a NUL byte.
    InvalidItemId,
    /// The backend cannot meet the requested access policy.
    UnsupportedPolicy,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl SecureStorageError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidServiceId | Self::InvalidItemId => ErrorKind::InvalidInput,
            Self::UnsupportedPolicy => ErrorKind::Unsupported,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native status code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidServiceId | Self::InvalidItemId | Self::UnsupportedPolicy => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// The effective access policy for one successfully stored secret.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StoreOutcome {
    effective_policy: AccessPolicy,
}

impl StoreOutcome {
    /// Creates a store result with the backend's effective policy.
    pub const fn new(effective_policy: AccessPolicy) -> Self {
        Self { effective_policy }
    }

    /// Returns the access policy the backend applied to the stored item.
    pub const fn effective_policy(self) -> AccessPolicy {
        self.effective_policy
    }
}

/// A compile-time-selected backend for opaque secret bytes.
///
/// The backend owns its state and may use any supported platform store. It must not retain
/// borrowed identifiers or secret input after a call. A successful `store` must return an
/// effective policy that satisfies the required policy. If it cannot do so, it must return
/// `UnsupportedPolicy` before mutating stored state. A `read` returns an owned byte vector; this
/// necessarily permits a backend copy from its native representation.
pub trait SecureStorageBackend {
    /// Reports whether secure storage is usable in the current context.
    fn availability(&self) -> Availability;

    /// Reads an owned secret byte vector, or `None` when the item does not exist.
    ///
    /// Returned bytes are plaintext accessible to the caller and are not protected by this
    /// facade after return. The backend may copy native storage into the caller-owned vector.
    fn read(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
    ) -> Result<Option<Vec<u8>>, SecureStorageError>;

    /// Stores opaque secret bytes and reports the effective access policy.
    ///
    /// `secret` is borrowed only for this synchronous call. The backend must copy or otherwise
    /// persist the bytes without retaining this borrow. It must reject an unsupported required
    /// policy with `UnsupportedPolicy` before mutation. A successful outcome must satisfy the
    /// requested policy. No secrecy, encryption, zeroization, or durability guarantee is implied
    /// by this portable contract.
    fn store(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
        secret: &[u8],
        required_policy: AccessPolicy,
    ) -> Result<StoreOutcome, SecureStorageError>;

    /// Removes one item and reports whether it existed.
    fn remove(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
    ) -> Result<bool, SecureStorageError>;
}

/// A thin facade over caller-owned, statically selected secure-storage backend state.
pub struct SecureStorage<B> {
    backend: B,
}

impl<B: SecureStorageBackend> SecureStorage<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Reads an owned byte vector; the facade adds no copy after the backend result.
    pub fn read(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
    ) -> Result<Option<Vec<u8>>, SecureStorageError> {
        self.backend.read(service, item)
    }

    /// Stores borrowed opaque bytes and reports the effective access policy.
    pub fn store(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
        secret: &[u8],
        required_policy: AccessPolicy,
    ) -> Result<StoreOutcome, SecureStorageError> {
        self.backend.store(service, item, secret, required_policy)
    }

    /// Removes one item and reports whether it existed.
    pub fn remove(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
    ) -> Result<bool, SecureStorageError> {
        self.backend.remove(service, item)
    }

    /// Borrows the backend for secure-storage operations not yet modeled by this facade.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for secure-storage operations not yet modeled by this facade.
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
        secret: Option<Vec<u8>>,
        policy: AccessPolicy,
        mutations: u32,
    }

    impl SecureStorageBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        fn read(
            &mut self,
            _service: ServiceId<'_>,
            _item: ItemId<'_>,
        ) -> Result<Option<Vec<u8>>, SecureStorageError> {
            Ok(self.secret.clone())
        }

        fn store(
            &mut self,
            _service: ServiceId<'_>,
            _item: ItemId<'_>,
            secret: &[u8],
            required_policy: AccessPolicy,
        ) -> Result<StoreOutcome, SecureStorageError> {
            if !self.policy.satisfies(required_policy) {
                return Err(SecureStorageError::UnsupportedPolicy);
            }
            self.secret = Some(secret.to_vec());
            self.mutations += 1;
            Ok(StoreOutcome::new(self.policy))
        }

        fn remove(
            &mut self,
            _service: ServiceId<'_>,
            _item: ItemId<'_>,
        ) -> Result<bool, SecureStorageError> {
            let existed = self.secret.take().is_some();
            if existed {
                self.mutations += 1;
            }
            Ok(existed)
        }
    }

    fn ids() -> (ServiceId<'static>, ItemId<'static>) {
        (
            ServiceId::new("account").unwrap(),
            ItemId::new("access-token").unwrap(),
        )
    }

    fn backend(secret: Option<Vec<u8>>, policy: AccessPolicy) -> Backend {
        Backend {
            secret,
            policy,
            mutations: 0,
        }
    }

    #[test]
    fn identifiers_are_exact_nonempty_text_without_nul() {
        assert_eq!(ServiceId::new("svc.name").unwrap().as_str(), "svc.name");
        assert_eq!(ItemId::new("key/name").unwrap().as_str(), "key/name");
        assert_eq!(
            ServiceId::new(""),
            Err(SecureStorageError::InvalidServiceId)
        );
        assert_eq!(
            ServiceId::new("a\0b"),
            Err(SecureStorageError::InvalidServiceId)
        );
        assert_eq!(ItemId::new(""), Err(SecureStorageError::InvalidItemId));
        assert_eq!(ItemId::new("a\0b"), Err(SecureStorageError::InvalidItemId));
    }

    #[test]
    fn unsupported_policy_is_rejected_before_mutation() {
        let (service, item) = ids();
        let mut storage = SecureStorage::new(backend(
            Some(vec![b'o', b'l', b'd']),
            AccessPolicy::unrestricted(),
        ));
        let required = AccessPolicy::new(true, true);
        assert_eq!(
            storage.store(service, item, b"new", required),
            Err(SecureStorageError::UnsupportedPolicy)
        );
        assert_eq!(storage.backend().mutations, 0);
        assert_eq!(storage.backend().secret.as_deref(), Some(&b"old"[..]));
        assert_eq!(
            SecureStorageError::UnsupportedPolicy.kind(),
            ErrorKind::Unsupported
        );
    }

    #[test]
    fn reads_return_owned_bytes_and_store_reports_effective_policy() {
        let (service, item) = ids();
        let policy = AccessPolicy::new(true, false);
        let mut storage = SecureStorage::new(backend(Some(vec![b's', b'e', b'c']), policy));
        let outcome = storage
            .store(service, item, b"next", AccessPolicy::new(true, false))
            .unwrap();
        assert_eq!(outcome.effective_policy(), policy);
        let mut secret = storage.read(service, item).unwrap().unwrap();
        secret[0] = b'x';
        assert_eq!(secret, b"xext");
        assert_eq!(storage.backend().secret.as_deref(), Some(&b"next"[..]));
    }

    #[test]
    fn remove_reports_presence_then_absence() {
        let (service, item) = ids();
        let mut storage = SecureStorage::new(backend(None, AccessPolicy::unrestricted()));
        assert_eq!(storage.remove(service, item), Ok(false));
        storage
            .store(service, item, b"secret", AccessPolicy::unrestricted())
            .unwrap();
        assert_eq!(storage.remove(service, item), Ok(true));
        assert_eq!(storage.remove(service, item), Ok(false));
    }

    #[test]
    fn backend_errors_preserve_category_and_native_status() {
        let code = PlatformErrorCode::new(-42).unwrap();
        let error =
            SecureStorageError::Backend(Error::new(ErrorKind::Platform).with_platform_code(code));
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code(), Some(code));
    }
}

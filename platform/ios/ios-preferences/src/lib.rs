#![cfg(target_os = "ios")]
#![deny(missing_docs)]
//! iOS non-secure preference adapter backed by `NSUserDefaults` property-list data values.

use framework_core::{Availability, Error, ErrorKind};
use framework_preferences::{
    AtomicityRequirement, PreferenceError, PreferenceKey, PreferencesBackend, UpdateAtomicity,
    WriteOptions, WriteOutcome,
};
use objc2::{rc::Retained, rc::autoreleasepool, runtime::AnyObject};
use objc2_foundation::{NSData, NSString, NSUserDefaults};

/// A caller-owned handle to the standard app preferences domain.
///
/// Calls are synchronous. `NSUserDefaults` is safe to call from multiple threads; this backend
/// adds no callback, executor, or framework-wide registry. Values are non-secure preferences, not
/// credentials or Keychain items.
pub struct IosPreferences {
    defaults: Retained<NSUserDefaults>,
}

impl IosPreferences {
    /// Retains Foundation's standard defaults object for the current application.
    pub fn new() -> Self {
        autoreleasepool(|_| Self {
            defaults: NSUserDefaults::standardUserDefaults(),
        })
    }

    /// Borrows Foundation's native defaults object for operations not modeled by the portable API.
    ///
    /// Native mutations can change values observed by `Preferences<IosPreferences>`.
    pub fn native_defaults(&self) -> &NSUserDefaults {
        &self.defaults
    }
}

impl Default for IosPreferences {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesBackend for IosPreferences {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn get_bytes(&mut self, key: PreferenceKey<'_>) -> Result<Option<Vec<u8>>, PreferenceError> {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let Some(value) = self.defaults.objectForKey(&key) else {
                return Ok(None);
            };
            match value.downcast::<NSData>() {
                Ok(data) => Ok(Some(data.to_vec())),
                Err(_) => Err(PreferenceError::Backend(Error::new(
                    ErrorKind::InvalidInput,
                ))),
            }
        })
    }

    fn set_bytes(
        &mut self,
        key: PreferenceKey<'_>,
        value: &[u8],
        options: WriteOptions,
    ) -> Result<WriteOutcome, PreferenceError> {
        if options.atomicity() == AtomicityRequirement::RequireAtomic {
            return Err(PreferenceError::Backend(Error::new(ErrorKind::Unsupported)));
        }
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let data = NSData::with_bytes(value);
            let value: &AnyObject = &data;
            // SAFETY: `value` is an immutable `NSData`, a valid property-list object, retained
            // through this synchronous call; Foundation copies/stores its value.
            unsafe { self.defaults.setObject_forKey(Some(value), &key) };
            Ok(WriteOutcome::new(UpdateAtomicity::NotGuaranteed))
        })
    }

    fn remove(&mut self, key: PreferenceKey<'_>) -> Result<bool, PreferenceError> {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let existed = self.defaults.objectForKey(&key).is_some();
            self.defaults.removeObjectForKey(&key);
            Ok(existed)
        })
    }
}

#![cfg(target_os = "ios")]
#![deny(missing_docs)]
//! iOS non-secure preference adapter backed by `NSUserDefaults` property-list data values.

use framework_core::{Availability, Error, ErrorKind};
use framework_preferences::{
    PreferenceError, PreferenceKey, PreferencesBackend, WriteOptions, WriteOutcome,
};
use objc2::{rc::Retained, rc::autoreleasepool, runtime::AnyObject};
use objc2_core_foundation::{CFBoolean, CFType};
use objc2_foundation::{NSData, NSDate, NSNumber, NSString, NSUserDefaults};

mod value_policy;
mod write_policy;
pub use value_policy::IosPreferencesValueError;
use value_policy::map_optional_value;
use write_policy::accepted_write_outcome;

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

    /// Reads an existing Foundation string without coercing other property-list types.
    ///
    /// `Ok(None)` means the key has no value. A value that is not `NSString` returns
    /// `IosPreferencesValueError::WrongType`; a string with unpaired UTF-16 surrogates returns
    /// `IosPreferencesValueError::InvalidUtf16`.
    pub fn get_string(
        &self,
        key: PreferenceKey<'_>,
    ) -> Result<Option<String>, IosPreferencesValueError> {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            map_optional_value(self.defaults.objectForKey(&key), |value| {
                let string = value
                    .downcast::<NSString>()
                    .map_err(|_| IosPreferencesValueError::WrongType)?;
                let length = string.length();
                let mut code_units = Vec::with_capacity(length);
                for index in 0..length {
                    code_units.push(string.characterAtIndex(index));
                }
                value_policy::decode_utf16(&code_units)
            })
        })
    }

    /// Stores a Rust string as an `NSString` under the exact validated key.
    pub fn set_string(&self, key: PreferenceKey<'_>, value: &str) {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let value = NSString::from_str(value);
            let value: &AnyObject = &value;
            // SAFETY: `value` is an immutable `NSString`, a valid property-list object, retained
            // through this synchronous call; Foundation copies/stores its value.
            unsafe { self.defaults.setObject_forKey(Some(value), &key) };
        });
    }

    /// Reads an existing Foundation Boolean without coercing numbers or other property-list types.
    ///
    /// `Ok(None)` means the key has no value. A value that is not a Core Foundation Boolean
    /// returns `IosPreferencesValueError::WrongType`.
    pub fn get_bool(
        &self,
        key: PreferenceKey<'_>,
    ) -> Result<Option<bool>, IosPreferencesValueError> {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            map_optional_value(self.defaults.objectForKey(&key), |value| {
                let number = value
                    .downcast::<NSNumber>()
                    .map_err(|_| IosPreferencesValueError::WrongType)?;
                let number: &NSNumber = &number;
                // SAFETY: Foundation documents CFBoolean and NSNumber as toll-free bridged types,
                // so this property-list NSNumber has the Core Foundation object layout.
                let value = unsafe { &*(number as *const NSNumber).cast::<CFType>() };
                value
                    .downcast_ref::<CFBoolean>()
                    .map(CFBoolean::value)
                    .ok_or(IosPreferencesValueError::WrongType)
            })
        })
    }

    /// Stores a Rust Boolean as an `NSNumber` Boolean under the exact validated key.
    pub fn set_bool(&self, key: PreferenceKey<'_>, value: bool) {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let value = NSNumber::numberWithBool(value);
            let value: &AnyObject = &value;
            // SAFETY: `value` is an immutable Boolean `NSNumber`, a valid property-list object,
            // retained through this synchronous call; Foundation copies/stores its value.
            unsafe { self.defaults.setObject_forKey(Some(value), &key) };
        });
    }

    /// Reads a native date as finite seconds since 1970-01-01T00:00:00Z.
    ///
    /// `Ok(None)` means the key has no value. A value that is not `NSDate` returns
    /// `IosPreferencesValueError::WrongType`; a non-finite Foundation time interval returns
    /// `IosPreferencesValueError::InvalidDateSeconds`. The returned `f64` uses Foundation's
    /// `NSTimeInterval` precision.
    pub fn get_date_seconds(
        &self,
        key: PreferenceKey<'_>,
    ) -> Result<Option<f64>, IosPreferencesValueError> {
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            map_optional_value(self.defaults.objectForKey(&key), |value| {
                let date = value
                    .downcast::<NSDate>()
                    .map_err(|_| IosPreferencesValueError::WrongType)?;
                value_policy::validate_date_seconds(date.timeIntervalSince1970())
            })
        })
    }

    /// Stores finite seconds since 1970-01-01T00:00:00Z as a native `NSDate`.
    ///
    /// Foundation stores dates as `NSTimeInterval` floating-point seconds. Fractional-second
    /// values are therefore bounded by `f64` precision; this method does not promise
    /// nanosecond or bit-for-bit round trips through `NSDate`.
    pub fn set_date_seconds(
        &self,
        key: PreferenceKey<'_>,
        seconds: f64,
    ) -> Result<(), IosPreferencesValueError> {
        let seconds = value_policy::validate_date_seconds(seconds)?;
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let value = NSDate::dateWithTimeIntervalSince1970(seconds);
            let value: &AnyObject = &value;
            // SAFETY: `value` is an immutable `NSDate`, a valid property-list object, retained
            // through this synchronous call; Foundation copies/stores its value.
            unsafe { self.defaults.setObject_forKey(Some(value), &key) };
        });
        Ok(())
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
        let outcome = accepted_write_outcome(options)?;
        autoreleasepool(|_| {
            let key = NSString::from_str(key.as_str());
            let data = NSData::with_bytes(value);
            let value: &AnyObject = &data;
            // SAFETY: `value` is an immutable `NSData`, a valid property-list object, retained
            // through this synchronous call; Foundation copies/stores its value.
            unsafe { self.defaults.setObject_forKey(Some(value), &key) };
            Ok(outcome)
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

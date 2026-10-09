use alloc::boxed::Box;
use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{
    FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus, FrameworkStr, catch_unwind_status,
};
use framework_core::Error;
use framework_preferences::{AtomicityRequirement, PreferenceError, PreferenceKey};
#[cfg(target_os = "ios")]
use framework_preferences::{Preferences, UpdateAtomicity, WriteOptions};

/// Fixed-width preferences availability tag.
pub type FrameworkIosPreferencesAvailability = u32;
/// The current availability is unknown.
pub const FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN: FrameworkIosPreferencesAvailability = 0;
/// The preferences backend is available.
pub const FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_AVAILABLE: FrameworkIosPreferencesAvailability = 1;
/// Preferences are unsupported on this target.
pub const FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNSUPPORTED: FrameworkIosPreferencesAvailability =
    2;
/// Preferences require user permission.
pub const FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_PERMISSION:
    FrameworkIosPreferencesAvailability = 3;
/// Preferences require a platform entitlement.
pub const FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_ENTITLEMENT:
    FrameworkIosPreferencesAvailability = 4;
/// Preferences are temporarily unavailable.
pub const FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_TEMPORARILY_UNAVAILABLE:
    FrameworkIosPreferencesAvailability = 5;

/// Fixed-width requirement for one preference update.
pub type FrameworkIosPreferencesUpdateRequirement = u32;
/// Accept the backend's reported update atomicity.
pub const FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC: FrameworkIosPreferencesUpdateRequirement = 0;
/// Require atomic replacement; the current B1 backend rejects this before mutation.
pub const FRAMEWORK_IOS_PREFERENCES_REQUIRE_ATOMIC: FrameworkIosPreferencesUpdateRequirement = 1;

/// Fixed-width atomicity result for one preference update.
pub type FrameworkIosPreferencesUpdateAtomicity = u32;
/// No update atomicity result is available.
pub const FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN:
    FrameworkIosPreferencesUpdateAtomicity = 0;
/// The update is atomic for one key.
pub const FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_ATOMIC:
    FrameworkIosPreferencesUpdateAtomicity = 1;
/// The backend makes no non-torn update guarantee.
pub const FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_NOT_GUARANTEED:
    FrameworkIosPreferencesUpdateAtomicity = 2;

/// One explicit, caller-owned preferences handle.
pub struct FrameworkIosPreferences {
    #[cfg(target_os = "ios")]
    preferences: Preferences<::ios_preferences::IosPreferences>,
}

fn preference_status(error: PreferenceError) -> FrameworkStatus {
    FrameworkStatus::from_error(Error::new(error.kind()))
}

fn valid_span(data: *const u8, length: u64) -> bool {
    match usize::try_from(length) {
        Ok(length) if length <= isize::MAX as usize => {
            if length == 0 {
                data.is_null()
            } else {
                !data.is_null()
            }
        }
        _ => false,
    }
}

unsafe fn input_key<'a>(key: FrameworkStr) -> Result<PreferenceKey<'a>, FrameworkStatus> {
    if !valid_span(key.data(), key.length()) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The caller keeps the UTF-8 bytes readable and unchanged for the call.
    let key = unsafe { key.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)?;
    PreferenceKey::new(key).map_err(preference_status)
}

unsafe fn input_value<'a>(value: FrameworkSlice) -> Result<&'a [u8], FrameworkStatus> {
    if !valid_span(value.data(), value.length()) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The caller keeps the bytes readable and unchanged for the call.
    unsafe { value.as_bytes() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

fn update_requirement(
    value: FrameworkIosPreferencesUpdateRequirement,
) -> Option<AtomicityRequirement> {
    match value {
        FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC => Some(AtomicityRequirement::AllowNonAtomic),
        FRAMEWORK_IOS_PREFERENCES_REQUIRE_ATOMIC => Some(AtomicityRequirement::RequireAtomic),
        _ => None,
    }
}

#[cfg(target_os = "ios")]
fn availability_tag(value: framework_core::Availability) -> FrameworkIosPreferencesAvailability {
    match value {
        framework_core::Availability::Unknown => FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN,
        framework_core::Availability::Available => FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_AVAILABLE,
        framework_core::Availability::Unsupported => {
            FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNSUPPORTED
        }
        framework_core::Availability::RequiresPermission => {
            FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_PERMISSION
        }
        framework_core::Availability::RequiresEntitlement => {
            FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_ENTITLEMENT
        }
        framework_core::Availability::TemporarilyUnavailable => {
            FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_TEMPORARILY_UNAVAILABLE
        }
        _ => FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN,
    }
}

/// Creates a retained handle to the app's standard preferences domain.
///
/// # Safety
/// `out_preferences` must point to a writable slot that does not hold a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_preferences_create(
    out_preferences: *mut *mut FrameworkIosPreferences,
) -> FrameworkStatus {
    if out_preferences.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises that this is an empty writable handle slot.
    unsafe { out_preferences.write(ptr::null_mut()) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let preferences = Preferences::new(::ios_preferences::IosPreferences::new());
            let handle = Box::new(FrameworkIosPreferences { preferences });
            // SAFETY: The output slot was initialized and is exclusively owned by the caller.
            unsafe { out_preferences.write(Box::into_raw(handle)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Destroys one preferences handle and clears its original slot.
///
/// # Safety
/// `preferences` must be null or the original writable slot returned by create. Do not copy the
/// handle or call another function with it at the same time. A null slot or null handle is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_preferences_destroy(
    preferences: *mut *mut FrameworkIosPreferences,
) {
    if preferences.is_null() {
        return;
    }
    // SAFETY: The caller supplies the original writable handle slot.
    let value = unsafe { preferences.read() };
    if value.is_null() {
        return;
    }
    // SAFETY: Clear the original slot before dropping its unique allocation.
    unsafe { preferences.write(ptr::null_mut()) };
    // SAFETY: Create returned this Box allocation and the caller has not copied its handle.
    unsafe { drop(Box::from_raw(value)) };
}

/// Reports backend availability without promising that a later operation will succeed.
///
/// # Safety
/// On iOS, `preferences` must be a live handle. `out_availability` must point to writable storage
/// that does not alias the handle or another output. Calls on one handle must be serialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_preferences_availability(
    preferences: *const FrameworkIosPreferences,
    out_availability: *mut FrameworkIosPreferencesAvailability,
) -> FrameworkStatus {
    if out_availability.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable output slot.
    unsafe {
        out_availability.write(FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN);
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if preferences.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps a live handle and serializes operations on it.
            let preferences = unsafe { &*preferences };
            // SAFETY: Output storage is distinct from the immutable handle.
            unsafe {
                out_availability.write(availability_tag(preferences.preferences.availability()));
            }
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = preferences;
            // SAFETY: The caller supplies a writable output slot.
            unsafe {
                out_availability.write(FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNSUPPORTED);
            }
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Reads one preference value into a framework-owned byte buffer.
///
/// # Safety
/// On iOS, `preferences` must be a live handle. `key` must name readable UTF-8 bytes that remain
/// unchanged through the call. Required outputs must be writable, aligned, and distinct from each
/// other, inputs, and the handle. `out_value` must not hold a live framework-owned allocation on
/// entry. Calls on one handle must be serialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_preferences_get(
    preferences: *mut FrameworkIosPreferences,
    key: FrameworkStr,
    out_found: *mut u8,
    out_value: *mut FrameworkOwnedBuffer,
) -> FrameworkStatus {
    // SAFETY: The caller promises that each non-null required output is writable.
    unsafe {
        if !out_found.is_null() {
            out_found.write(0);
        }
        if !out_value.is_null() {
            out_value.write(FrameworkOwnedBuffer::default());
        }
    }
    if out_found.is_null() || out_value.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: The caller keeps the key bytes readable and unchanged through this call.
        let key = match unsafe { input_key(key) } {
            Ok(key) => key,
            Err(status) => return status,
        };
        #[cfg(target_os = "ios")]
        {
            if preferences.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps a live, exclusive handle against destroy and other calls.
            let preferences = unsafe { &mut *preferences };
            match preferences.preferences.get_bytes(key) {
                Ok(None) => FrameworkStatus::OK,
                Ok(Some(bytes)) => match FrameworkOwnedBuffer::try_from_vec(bytes) {
                    Ok(value) => {
                        // SAFETY: Required outputs were initialized and are disjoint.
                        unsafe {
                            out_value.write(value);
                            out_found.write(1);
                        }
                        FrameworkStatus::OK
                    }
                    Err(bytes) => {
                        drop(bytes);
                        FrameworkStatus::RESOURCE_EXHAUSTED
                    }
                },
                Err(error) => preference_status(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (preferences, key);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Writes one preference value and reports its update atomicity.
///
/// # Safety
/// On iOS, `preferences` must be a live handle. `key` and `value` must name readable storage that
/// remains unchanged through the call. Required outputs must be writable and distinct from each
/// other, inputs, and the handle. Calls on one handle must be serialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_preferences_set(
    preferences: *mut FrameworkIosPreferences,
    key: FrameworkStr,
    value: FrameworkSlice,
    required_atomicity: FrameworkIosPreferencesUpdateRequirement,
    out_atomicity: *mut FrameworkIosPreferencesUpdateAtomicity,
) -> FrameworkStatus {
    if out_atomicity.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable output slot.
    unsafe { out_atomicity.write(FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN) };
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: The caller keeps the key bytes readable and unchanged through this call.
        let key = match unsafe { input_key(key) } {
            Ok(key) => key,
            Err(status) => return status,
        };
        // SAFETY: The caller keeps the value bytes readable and unchanged through this call.
        let value = match unsafe { input_value(value) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        let Some(atomicity) = update_requirement(required_atomicity) else {
            return FrameworkStatus::INVALID_ARGUMENT;
        };
        #[cfg(target_os = "ios")]
        {
            if preferences.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps a live, exclusive handle against destroy and other calls.
            let preferences = unsafe { &mut *preferences };
            match preferences
                .preferences
                .set_bytes(key, value, WriteOptions::new(atomicity))
            {
                Ok(outcome) => {
                    let atomicity = match outcome.atomicity() {
                        UpdateAtomicity::Atomic => {
                            FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_ATOMIC
                        }
                        UpdateAtomicity::NotGuaranteed => {
                            FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_NOT_GUARANTEED
                        }
                        _ => return FrameworkStatus::INTERNAL_ERROR,
                    };
                    // SAFETY: The output was validated and initialized above.
                    unsafe { out_atomicity.write(atomicity) };
                    FrameworkStatus::OK
                }
                Err(error) => preference_status(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (preferences, key, value, atomicity);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Removes one preference key and reports whether its value was visible before removal.
///
/// # Safety
/// On iOS, `preferences` must be a live handle. `key` must name readable UTF-8 bytes that remain
/// unchanged through the call. Required outputs must be writable and distinct from the input and
/// handle. Calls on one handle must be serialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_preferences_remove(
    preferences: *mut FrameworkIosPreferences,
    key: FrameworkStr,
    out_removed: *mut u8,
) -> FrameworkStatus {
    if out_removed.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable output slot.
    unsafe { out_removed.write(0) };
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: The caller keeps the key bytes readable and unchanged through this call.
        let key = match unsafe { input_key(key) } {
            Ok(key) => key,
            Err(status) => return status,
        };
        #[cfg(target_os = "ios")]
        {
            if preferences.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps a live, exclusive handle against destroy and other calls.
            let preferences = unsafe { &mut *preferences };
            match preferences.preferences.remove(key) {
                Ok(removed) => {
                    // SAFETY: The output was validated and initialized above.
                    unsafe { out_removed.write(u8::from(removed)) };
                    FrameworkStatus::OK
                }
                Err(error) => preference_status(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (preferences, key);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

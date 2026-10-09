use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use block2::RcBlock;
use framework_core::{ErrorKind, PlatformErrorCode};
use framework_health_authorization::{
    HealthAuthorizationBackend, HealthAuthorizationCompletion, HealthAuthorizationError,
    HealthAuthorizationRequest, HealthDataType, HealthDataTypeKind,
};
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSSet, NSString};
use objc2_health_kit::{HKHealthStore, HKObjectType, HKSampleType};

/// The statically selected iOS implementation of HealthKit availability and authorization.
///
/// The backend stores no HealthKit object until an authorization request is accepted. Its
/// availability method calls `HKHealthStore.isHealthDataAvailable`; each request checks that API
/// again before creating an `HKHealthStore`. The native completion may run on an arbitrary
/// background queue. The callback reports only HealthKit request-flow completion or an error,
/// never permission state.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosHealthAuthorizationBackend;

impl IosHealthAuthorizationBackend {
    /// Creates a zero-state backend without calling HealthKit.
    pub const fn new() -> Self {
        Self
    }
}

impl HealthAuthorizationBackend for IosHealthAuthorizationBackend {
    fn is_available(&self) -> bool {
        // SAFETY: This is the public class-level availability query. It has no arguments and
        // HealthKit documents it as the first API to call before other HealthKit operations.
        unsafe { HKHealthStore::isHealthDataAvailable() }
    }

    fn request_authorization<F>(
        &self,
        request: &HealthAuthorizationRequest<'_, '_>,
        completion: F,
    ) -> Result<(), HealthAuthorizationError>
    where
        F: FnOnce(HealthAuthorizationCompletion) + Send + 'static,
    {
        if !self.is_available() {
            return Err(HealthAuthorizationError::backend(
                ErrorKind::Unavailable,
                None,
            ));
        }

        let read_values = request
            .read_types()
            .iter()
            .map(resolve_read_type)
            .collect::<Result<Vec<_>, _>>()?;
        let share_values = request
            .share_types()
            .iter()
            .map(resolve_share_type)
            .collect::<Result<Vec<_>, _>>()?;
        let read_types = (!read_values.is_empty())
            .then(|| NSSet::<HKObjectType>::from_retained_slice(&read_values));
        let share_types = (!share_values.is_empty())
            .then(|| NSSet::<HKSampleType>::from_retained_slice(&share_values));
        // SAFETY: The request already passed HealthKit's documented availability check; the
        // generated `new` call initializes an ordinary `HKHealthStore` with no caller state.
        let store = unsafe { HKHealthStore::new() };
        let callback_state = Arc::new(Mutex::new(Some(completion)));
        let callback: RcBlock<dyn Fn(Bool, *mut NSError)> =
            RcBlock::new(move |success: Bool, error: *mut NSError| {
                let completion = callback_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                let Some(completion) = completion else {
                    return;
                };
                let outcome = match unsafe { error.as_ref() } {
                    Some(error) => Err(native_error(error)),
                    None if success.as_bool() => Ok(()),
                    None => Err(HealthAuthorizationError::backend(ErrorKind::Unknown, None)),
                };
                let _ = catch_unwind(AssertUnwindSafe(|| completion(outcome)));
            });

        // SAFETY: The type sets contain retained values returned by public HKObjectType
        // factories. The completion captures only Send state, satisfying the binding's
        // sendability requirement. HealthKit owns/copies the block for its asynchronous call.
        unsafe {
            store.requestAuthorizationToShareTypes_readTypes_completion(
                share_types.as_deref(),
                read_types.as_deref(),
                &callback,
            );
        }
        Ok(())
    }
}

fn resolve_read_type(
    data_type: &HealthDataType<'_>,
) -> Result<Retained<HKObjectType>, HealthAuthorizationError> {
    let identifier = data_type.identifier().map(NSString::from_str);
    let resolved = match data_type.kind() {
        HealthDataTypeKind::Quantity => {
            let identifier = identifier
                .as_deref()
                .expect("quantity type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::quantityTypeForIdentifier(identifier) }
                .map(|value| value.into_super().into_super())
        }
        HealthDataTypeKind::Category => {
            let identifier = identifier
                .as_deref()
                .expect("category type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::categoryTypeForIdentifier(identifier) }
                .map(|value| value.into_super().into_super())
        }
        HealthDataTypeKind::Characteristic => {
            let identifier = identifier
                .as_deref()
                .expect("characteristic type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::characteristicTypeForIdentifier(identifier) }
                .map(Retained::into_super)
        }
        HealthDataTypeKind::Correlation => {
            let identifier = identifier
                .as_deref()
                .expect("correlation type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::correlationTypeForIdentifier(identifier) }
                .map(|value| value.into_super().into_super())
        }
        HealthDataTypeKind::Workout => {
            // SAFETY: This is Apple's public iOS 8+ workout-type factory.
            Some(
                unsafe { HKObjectType::workoutType() }
                    .into_super()
                    .into_super(),
            )
        }
        _ => None,
    };
    resolved.ok_or(HealthAuthorizationError::InvalidTypeIdentifier)
}

fn resolve_share_type(
    data_type: &HealthDataType<'_>,
) -> Result<Retained<HKSampleType>, HealthAuthorizationError> {
    let identifier = data_type.identifier().map(NSString::from_str);
    let resolved = match data_type.kind() {
        HealthDataTypeKind::Quantity => {
            let identifier = identifier
                .as_deref()
                .expect("quantity type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::quantityTypeForIdentifier(identifier) }.map(Retained::into_super)
        }
        HealthDataTypeKind::Category => {
            let identifier = identifier
                .as_deref()
                .expect("category type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::categoryTypeForIdentifier(identifier) }.map(Retained::into_super)
        }
        HealthDataTypeKind::Correlation => {
            let identifier = identifier
                .as_deref()
                .expect("correlation type has an identifier");
            // SAFETY: The generated binding exposes Apple's public factory; the NSString is a
            // live, immutable identifier and an unknown identifier returns None.
            unsafe { HKObjectType::correlationTypeForIdentifier(identifier) }
                .map(Retained::into_super)
        }
        HealthDataTypeKind::Workout => {
            // SAFETY: This is Apple's public iOS 8+ workout-type factory.
            Some(unsafe { HKObjectType::workoutType() }.into_super())
        }
        HealthDataTypeKind::Characteristic => {
            return Err(HealthAuthorizationError::NonSampleShareType);
        }
        _ => return Err(HealthAuthorizationError::InvalidTypeIdentifier),
    };
    resolved.ok_or(HealthAuthorizationError::InvalidTypeIdentifier)
}

fn native_error(error: &NSError) -> HealthAuthorizationError {
    let native_code = i32::try_from(error.code())
        .ok()
        .and_then(PlatformErrorCode::new);
    HealthAuthorizationError::backend(ErrorKind::Platform, native_code)
}

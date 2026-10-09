#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable HealthKit authorization request values and a static backend contract."]

use framework_core::{Error, ErrorKind, PlatformErrorCode};

/// A HealthKit object-type family supported by this request contract.
///
/// The family names are semantic Rust values. The iOS backend resolves each identifier with the
/// matching public `HKObjectType` factory. Document, series, scored-assessment, and per-object
/// authorization types are outside this slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HealthDataTypeKind {
    /// A quantity sample such as step count or heart rate.
    Quantity,
    /// A category sample such as sleep analysis.
    Category,
    /// A user characteristic such as date of birth.
    Characteristic,
    /// A correlation sample such as blood pressure.
    Correlation,
    /// A workout sample represented by `HKObjectType.workoutType()` on iOS.
    Workout,
}

impl HealthDataTypeKind {
    const fn is_sample(self) -> bool {
        !matches!(self, Self::Characteristic)
    }
}

/// A framework-owned HealthKit type family and type identifier.
///
/// Identifiers are borrowed UTF-8 strings, not HealthKit authorization results. Use a string
/// constant supplied by Apple for the requested family. The workout type has no caller-supplied
/// identifier and must be created with [`HealthDataType::workout`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HealthDataType<'a> {
    kind: HealthDataTypeKind,
    identifier: Option<&'a str>,
}

impl<'a> HealthDataType<'a> {
    /// Creates a data type from an Apple HealthKit identifier.
    ///
    /// The identifier must be non-empty and contain no NUL character. HealthKit may still reject
    /// a syntactically valid but unknown identifier when the backend resolves it.
    pub fn new(
        kind: HealthDataTypeKind,
        identifier: &'a str,
    ) -> Result<Self, HealthAuthorizationError> {
        if kind == HealthDataTypeKind::Workout {
            return Err(HealthAuthorizationError::WorkoutIdentifierNotAllowed);
        }
        if identifier.is_empty() || identifier.contains('\0') {
            return Err(HealthAuthorizationError::InvalidTypeIdentifier);
        }
        Ok(Self {
            kind,
            identifier: Some(identifier),
        })
    }

    /// Returns the workout sample type.
    pub const fn workout() -> Self {
        Self {
            kind: HealthDataTypeKind::Workout,
            identifier: None,
        }
    }

    /// Returns the HealthKit type family.
    pub const fn kind(self) -> HealthDataTypeKind {
        self.kind
    }

    /// Returns the borrowed Apple HealthKit identifier, or `None` for the workout type.
    pub const fn identifier(self) -> Option<&'a str> {
        self.identifier
    }
}

/// A request to ask HealthKit for read and/or share access to selected types.
///
/// Read and share are separate sets. A characteristic type may appear only in `read_types`
/// because HealthKit accepts only sample types in its share set. The request stores borrowed
/// slices and does not copy or retain identifiers. Empty requests are rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthAuthorizationRequest<'request, 'identifier> {
    read_types: &'request [HealthDataType<'identifier>],
    share_types: &'request [HealthDataType<'identifier>],
}

impl<'request, 'identifier> HealthAuthorizationRequest<'request, 'identifier> {
    /// Creates a request for the supplied read and share types.
    pub fn new(
        read_types: &'request [HealthDataType<'identifier>],
        share_types: &'request [HealthDataType<'identifier>],
    ) -> Result<Self, HealthAuthorizationError> {
        if read_types.is_empty() && share_types.is_empty() {
            return Err(HealthAuthorizationError::EmptyRequest);
        }
        if share_types
            .iter()
            .any(|data_type| !data_type.kind.is_sample())
        {
            return Err(HealthAuthorizationError::NonSampleShareType);
        }
        Ok(Self {
            read_types,
            share_types,
        })
    }

    /// Returns the read types requested by the caller.
    pub const fn read_types(&self) -> &'request [HealthDataType<'identifier>] {
        self.read_types
    }

    /// Returns the share types requested by the caller.
    pub const fn share_types(&self) -> &'request [HealthDataType<'identifier>] {
        self.share_types
    }
}

/// A portable error from request validation or a native backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HealthAuthorizationError {
    /// No read or share types were supplied.
    EmptyRequest,
    /// A type identifier was empty or contained a NUL character.
    InvalidTypeIdentifier,
    /// The workout type was given a caller-supplied identifier.
    WorkoutIdentifierNotAllowed,
    /// A characteristic type was included in the share set, which accepts samples only.
    NonSampleShareType,
    /// A native backend error with a stable category and optional native code.
    Backend(Error),
}

impl HealthAuthorizationError {
    /// Creates an error from a native backend category and optional native code.
    pub const fn backend(kind: ErrorKind, native_code: Option<PlatformErrorCode>) -> Self {
        let error = Error::new(kind);
        match native_code {
            Some(code) => Self::Backend(error.with_platform_code(code)),
            None => Self::Backend(error),
        }
    }

    /// Returns the portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::EmptyRequest
            | Self::InvalidTypeIdentifier
            | Self::WorkoutIdentifierNotAllowed
            | Self::NonSampleShareType => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::EmptyRequest
            | Self::InvalidTypeIdentifier
            | Self::WorkoutIdentifierNotAllowed
            | Self::NonSampleShareType => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// The result of the asynchronous native authorization-request flow.
///
/// `Ok(())` means only that HealthKit completed processing the request flow without reporting an
/// error. It does not reveal whether the user allowed sharing, whether the user allowed reading,
/// or whether read data will be returned. HealthKit intentionally hides read denials.
pub type HealthAuthorizationCompletion = Result<(), HealthAuthorizationError>;

/// A statically selected backend for HealthKit availability and authorization requests.
///
/// Availability reports only whether the HealthKit data store can be used on the current device
/// and context. It does not report an entitlement, per-type permission, or read grant. The
/// completion callback may run on any backend-selected execution context and must be thread-safe
/// to move. A backend must invoke it at most once; it may not guarantee a completion deadline or
/// delivery if the process exits or the platform does not complete the request. An accepted
/// request is never a permission result.
pub trait HealthAuthorizationBackend {
    /// Reports whether HealthKit data is available in the current device and context.
    fn is_available(&self) -> bool;

    /// Starts an explicit request for read and/or share access to the selected types.
    ///
    /// `Ok(())` means the backend accepted the request for native processing. It does not mean a
    /// prompt appeared or that any authorization was granted. If this method returns an error,
    /// it must not invoke `completion`. If it returns `Ok(())`, the callback may run inline or
    /// later on a backend-selected context, at most once. No executor or boxed backend is used.
    fn request_authorization<F>(
        &self,
        request: &HealthAuthorizationRequest<'_, '_>,
        completion: F,
    ) -> Result<(), HealthAuthorizationError>
    where
        F: FnOnce(HealthAuthorizationCompletion) + Send + 'static;
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    struct FakeBackend;

    impl HealthAuthorizationBackend for FakeBackend {
        fn is_available(&self) -> bool {
            true
        }

        fn request_authorization<F>(
            &self,
            _request: &HealthAuthorizationRequest<'_, '_>,
            completion: F,
        ) -> Result<(), HealthAuthorizationError>
        where
            F: FnOnce(HealthAuthorizationCompletion) + Send + 'static,
        {
            completion(Ok(()));
            Ok(())
        }
    }

    #[test]
    fn type_ids_reject_empty_and_nul_but_keep_family() {
        assert_eq!(
            HealthDataType::new(HealthDataTypeKind::Quantity, ""),
            Err(HealthAuthorizationError::InvalidTypeIdentifier)
        );
        assert_eq!(
            HealthDataType::new(HealthDataTypeKind::Quantity, "x\0y"),
            Err(HealthAuthorizationError::InvalidTypeIdentifier)
        );
        let quantity = HealthDataType::new(
            HealthDataTypeKind::Quantity,
            "HKQuantityTypeIdentifierStepCount",
        )
        .unwrap();
        assert_eq!(quantity.kind(), HealthDataTypeKind::Quantity);
        assert_eq!(
            quantity.identifier(),
            Some("HKQuantityTypeIdentifierStepCount")
        );
    }

    #[test]
    fn workout_is_fixed_and_cannot_take_an_identifier() {
        assert_eq!(
            HealthDataType::new(HealthDataTypeKind::Workout, "unused"),
            Err(HealthAuthorizationError::WorkoutIdentifierNotAllowed)
        );
        assert_eq!(HealthDataType::workout().identifier(), None);
    }

    #[test]
    fn request_requires_a_type_and_rejects_characteristic_sharing() {
        assert_eq!(
            HealthAuthorizationRequest::new(&[], &[]),
            Err(HealthAuthorizationError::EmptyRequest)
        );
        let characteristic = HealthDataType::new(
            HealthDataTypeKind::Characteristic,
            "HKCharacteristicTypeIdentifierDateOfBirth",
        )
        .unwrap();
        assert_eq!(
            HealthAuthorizationRequest::new(&[], &[characteristic]),
            Err(HealthAuthorizationError::NonSampleShareType)
        );
        let read_types = [characteristic];
        let read_only = HealthAuthorizationRequest::new(&read_types, &[]).unwrap();
        assert_eq!(read_only.read_types(), &[characteristic]);
        assert!(read_only.share_types().is_empty());
    }

    #[test]
    fn sample_share_types_and_workout_are_accepted() {
        let step_count = HealthDataType::new(
            HealthDataTypeKind::Quantity,
            "HKQuantityTypeIdentifierStepCount",
        )
        .unwrap();
        let workout = HealthDataType::workout();
        let share_types = [step_count, workout];
        let request = HealthAuthorizationRequest::new(&[], &share_types).unwrap();
        assert_eq!(request.share_types(), &[step_count, workout]);
    }

    #[test]
    fn static_backend_reports_availability_and_flow_completion_only() {
        let step_count = HealthDataType::new(
            HealthDataTypeKind::Quantity,
            "HKQuantityTypeIdentifierStepCount",
        )
        .unwrap();
        let read_types = [step_count];
        let request = HealthAuthorizationRequest::new(&read_types, &[]).unwrap();
        let called = std::sync::Arc::new(core::sync::atomic::AtomicBool::new(false));
        let callback_called = std::sync::Arc::clone(&called);
        assert!(FakeBackend.is_available());
        FakeBackend
            .request_authorization(&request, move |result| {
                assert_eq!(result, Ok(()));
                callback_called.store(true, core::sync::atomic::Ordering::SeqCst);
            })
            .unwrap();
        assert!(called.load(core::sync::atomic::Ordering::SeqCst));
    }
}

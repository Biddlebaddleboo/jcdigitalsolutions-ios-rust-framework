#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable semantic types shared by framework crates. This crate is allocator-free and does not depend on a platform backend."]

mod error;
mod handle;
mod ids;
mod platform;
mod state;

pub use error::{Error, ErrorKind, PlatformErrorCode, Result};
pub use handle::{CompactHandle, Generation};
pub use ids::OperationId;
pub use platform::{Availability, Capability, CapabilityId, Platform};
pub use state::{AuthorizationState, Cancellation, PermissionState};

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    fn fixed_width_values_have_expected_layout() {
        assert_eq!(
            (size_of::<CapabilityId>(), align_of::<CapabilityId>()),
            (2, 2)
        );
        assert_eq!((size_of::<Capability>(), align_of::<Capability>()), (2, 2));
        assert_eq!((size_of::<Generation>(), align_of::<Generation>()), (4, 4));
        assert_eq!(
            (size_of::<OperationId>(), align_of::<OperationId>()),
            (8, align_of::<u64>())
        );
        assert_eq!(
            (size_of::<CompactHandle>(), align_of::<CompactHandle>()),
            (8, align_of::<u64>())
        );
        assert_eq!(
            (
                size_of::<PlatformErrorCode>(),
                align_of::<PlatformErrorCode>()
            ),
            (4, 4)
        );
        assert_eq!((size_of::<Error>(), align_of::<Error>()), (8, 4));
        assert_eq!((size_of::<Platform>(), align_of::<Platform>()), (1, 1));
        assert_eq!(
            (size_of::<Availability>(), align_of::<Availability>()),
            (1, 1)
        );
        assert_eq!((size_of::<ErrorKind>(), align_of::<ErrorKind>()), (1, 1));
        assert_eq!(
            (size_of::<PermissionState>(), align_of::<PermissionState>()),
            (1, 1)
        );
        assert_eq!(
            (
                size_of::<AuthorizationState>(),
                align_of::<AuthorizationState>()
            ),
            (1, 1)
        );
        assert_eq!(
            (size_of::<Cancellation>(), align_of::<Cancellation>()),
            (1, 1)
        );
    }

    #[test]
    fn zero_is_reserved_for_identifiers_and_generations() {
        assert_eq!(CapabilityId::new(0), None);
        assert_eq!(OperationId::new(0), None);
        assert_eq!(Generation::new(0), None);
        assert_eq!(CompactHandle::from_raw(0), None);
    }

    #[test]
    fn generation_wrap_skips_reserved_zero() {
        assert_eq!(
            Generation::new(u32::MAX).unwrap().next(),
            Generation::initial()
        );
        assert_eq!(Generation::initial().next().get(), 2);
    }

    #[test]
    fn compact_handle_round_trips_fixed_width_parts() {
        let generation = Generation::new(0x1234_5678).unwrap();
        let handle = CompactHandle::new(0x9abc_def0, generation);
        assert_eq!(handle.to_raw(), 0x1234_5678_9abc_def0);
        assert_eq!(CompactHandle::from_raw(handle.to_raw()), Some(handle));
        assert_eq!(handle.index(), 0x9abc_def0);
        assert_eq!(handle.generation(), generation);
        assert_eq!(
            CompactHandle::from_raw(0x0000_0001_0000_0000)
                .unwrap()
                .index(),
            0
        );
    }

    #[test]
    fn errors_keep_a_stable_category_and_optional_platform_code() {
        let error = Error::new(ErrorKind::Platform)
            .with_platform_code(PlatformErrorCode::new(-42).unwrap());
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);
        assert_eq!(Error::new(ErrorKind::Cancelled).platform_code(), None);
    }
}

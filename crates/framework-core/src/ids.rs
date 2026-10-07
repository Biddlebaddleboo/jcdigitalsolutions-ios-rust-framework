use core::num::NonZeroU64;

/// A nonzero fixed-width identity for an asynchronous operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct OperationId(NonZeroU64);

impl OperationId {
    /// Creates an operation ID; zero is reserved to mean no operation.
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the fixed-width operation ID.
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

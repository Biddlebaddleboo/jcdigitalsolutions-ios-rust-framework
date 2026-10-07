use core::num::NonZeroU32;

/// A nonzero slot generation used to reject stale compact handles.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Generation(NonZeroU32);

impl Generation {
    /// Creates a generation; zero is reserved as an invalid-handle marker.
    pub const fn new(raw: u32) -> Option<Self> {
        match NonZeroU32::new(raw) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the first valid generation.
    pub const fn initial() -> Self {
        Self(NonZeroU32::MIN)
    }

    /// Returns the fixed-width generation value.
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    /// Advances the generation and wraps from `u32::MAX` to one, never zero.
    pub const fn next(self) -> Self {
        if self.get() == u32::MAX {
            Self::initial()
        } else {
            Self(NonZeroU32::new(self.get() + 1).unwrap())
        }
    }
}

/// A compact generational identity that is independent of native pointer width.
///
/// The low 32 bits store the slot index and the high 32 bits store a nonzero generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct CompactHandle(u64);

impl CompactHandle {
    /// Creates a handle for a slot index and nonzero generation.
    pub const fn new(index: u32, generation: Generation) -> Self {
        Self(((generation.get() as u64) << 32) | index as u64)
    }

    /// Decodes a raw handle, rejecting generation zero.
    pub const fn from_raw(raw: u64) -> Option<Self> {
        if (raw >> 32) == 0 {
            None
        } else {
            Some(Self(raw))
        }
    }

    /// Returns the raw fixed-width representation for internal or ABI conversion.
    pub const fn to_raw(self) -> u64 {
        self.0
    }

    /// Returns the zero-based slot index.
    pub const fn index(self) -> u32 {
        self.0 as u32
    }

    /// Returns the nonzero generation encoded in this handle.
    pub const fn generation(self) -> Generation {
        match Generation::new((self.0 >> 32) as u32) {
            Some(value) => value,
            None => panic!("CompactHandle invariant violated"),
        }
    }
}

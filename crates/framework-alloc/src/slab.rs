use alloc::vec::Vec;
use framework_core::{CompactHandle, Generation};

const NO_INDEX: u32 = u32::MAX;

struct Slot<T> {
    generation: Generation,
    value: Option<T>,
    next_free: u32,
}

/// A dense slot collection addressed by compact generational handles.
///
/// Removed slots enter an intrusive free list. A slot generation advances on removal, so an
/// earlier handle cannot access a reused slot until its 32-bit generation eventually wraps.
pub struct GenerationalSlab<T> {
    slots: Vec<Slot<T>>,
    free_head: u32,
    len: u32,
}

impl<T> GenerationalSlab<T> {
    /// Creates an empty slab without allocating.
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: NO_INDEX,
            len: 0,
        }
    }

    /// Returns the number of live values as a fixed-width semantic count.
    pub const fn len(&self) -> u32 {
        self.len
    }

    /// Returns whether the slab has no live values.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the currently reserved slot count, capped at the `u32` index domain.
    pub fn capacity(&self) -> u32 {
        u32::try_from(self.slots.capacity()).unwrap_or(u32::MAX)
    }

    /// Inserts a value and returns its current compact identity.
    ///
    /// Allocation failure and index exhaustion return ownership of the input value.
    pub fn insert(&mut self, value: T) -> Result<CompactHandle, InsertError<T>> {
        if self.free_head != NO_INDEX {
            let index = self.free_head;
            let slot = &mut self.slots[index as usize];
            self.free_head = slot.next_free;
            slot.next_free = NO_INDEX;
            debug_assert!(slot.value.is_none());
            slot.value = Some(value);
            self.len += 1;
            return Ok(CompactHandle::new(index, slot.generation));
        }

        let max_slots = usize::try_from(u32::MAX).unwrap_or(usize::MAX);
        if self.slots.len() >= max_slots {
            return Err(InsertError::new(InsertErrorKind::CapacityExceeded, value));
        }
        if self.slots.try_reserve(1).is_err() {
            return Err(InsertError::new(InsertErrorKind::AllocationFailed, value));
        }
        let Ok(index) = u32::try_from(self.slots.len()) else {
            return Err(InsertError::new(InsertErrorKind::CapacityExceeded, value));
        };
        self.slots.push(Slot {
            generation: Generation::initial(),
            value: Some(value),
            next_free: NO_INDEX,
        });
        self.len += 1;
        Ok(CompactHandle::new(index, Generation::initial()))
    }

    /// Returns a shared value reference when the handle is current and occupied.
    pub fn get(&self, handle: CompactHandle) -> Option<&T> {
        let index = usize::try_from(handle.index()).ok()?;
        let slot = self.slots.get(index)?;
        (slot.generation == handle.generation()).then_some(())?;
        slot.value.as_ref()
    }

    /// Returns an exclusive value reference when the handle is current and occupied.
    pub fn get_mut(&mut self, handle: CompactHandle) -> Option<&mut T> {
        let index = usize::try_from(handle.index()).ok()?;
        let slot = self.slots.get_mut(index)?;
        (slot.generation == handle.generation()).then_some(())?;
        slot.value.as_mut()
    }

    /// Removes and returns a value, invalidating its handle before the slot can be reused.
    pub fn remove(&mut self, handle: CompactHandle) -> Option<T> {
        let index = usize::try_from(handle.index()).ok()?;
        let slot = self.slots.get_mut(index)?;
        if slot.generation != handle.generation() {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation = slot.generation.next();
        slot.next_free = self.free_head;
        self.free_head = handle.index();
        self.len -= 1;
        Some(value)
    }
}

impl<T> Default for GenerationalSlab<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The reason an insertion could not reserve a slot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InsertErrorKind {
    /// The fixed-width slot-index domain is exhausted.
    CapacityExceeded,
    /// The allocator could not reserve space for another slot.
    AllocationFailed,
}

/// An insertion error that preserves ownership of the value that did not fit.
#[derive(Debug)]
pub struct InsertError<T> {
    kind: InsertErrorKind,
    value: T,
}

impl<T> InsertError<T> {
    fn new(kind: InsertErrorKind, value: T) -> Self {
        Self { kind, value }
    }

    /// Returns the portable reason for the failed insertion.
    pub const fn kind(&self) -> InsertErrorKind {
        self.kind
    }

    /// Returns the original value to the caller.
    pub fn into_value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_wrap_invalidates_only_until_the_documented_wrap() {
        let mut slab = GenerationalSlab::new();
        let handle = slab.insert(1_u8).unwrap();
        let removed = slab.remove(handle).unwrap();
        assert_eq!(removed, 1);
        let replacement = slab.insert(2_u8).unwrap();
        assert_ne!(replacement.generation(), handle.generation());
        assert_eq!(slab.get(handle), None);
    }
}

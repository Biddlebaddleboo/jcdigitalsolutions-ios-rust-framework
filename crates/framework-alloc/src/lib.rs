#![no_std]
#![deny(missing_docs)]
#![doc = "Allocator-backed portable collections with fixed-width semantic identities."]

extern crate alloc;

mod bit_set;
mod slab;

pub use bit_set::{BitSet, BitSetError};
pub use slab::{GenerationalSlab, InsertError, InsertErrorKind};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slab_rejects_stale_handles_after_slot_reuse() {
        let mut slab = GenerationalSlab::new();
        let first = slab.insert("first").unwrap();
        assert_eq!(slab.get(first), Some(&"first"));
        assert_eq!(slab.remove(first), Some("first"));
        assert_eq!(slab.get(first), None);

        let second = slab.insert("second").unwrap();
        assert_eq!(second.index(), first.index());
        assert_ne!(second.generation(), first.generation());
        assert_eq!(slab.get(first), None);
        assert_eq!(slab.get(second), Some(&"second"));
        assert_eq!(slab.len(), 1);
    }

    #[test]
    fn slab_mutation_and_generated_reuse_sequence_preserve_invariants() {
        let mut slab = GenerationalSlab::new();
        let mut handles = alloc::vec::Vec::new();
        for value in 0_u32..128 {
            handles.push(slab.insert(value).unwrap());
        }
        for (index, handle) in handles.iter().copied().enumerate() {
            *slab.get_mut(handle).unwrap() += 1;
            assert_eq!(slab.get(handle), Some(&(index as u32 + 1)));
        }
        for handle in handles.iter().copied().step_by(2) {
            assert!(slab.remove(handle).is_some());
            assert!(slab.remove(handle).is_none());
        }
        assert_eq!(slab.len(), 64);
        assert!(slab.capacity() >= 128);
    }

    #[test]
    fn bit_set_handles_word_edges_and_rejects_out_of_range_bits() {
        let mut bits = BitSet::try_new(130).unwrap();
        for index in [0, 63, 64, 127, 128, 129] {
            assert_eq!(bits.set(index, true), Ok(false));
            assert!(bits.get(index).unwrap());
        }
        assert_eq!(bits.count_ones(), 6);
        assert_eq!(bits.get(130), None);
        assert_eq!(bits.set(130, true), Err(BitSetError::OutOfRange));
        assert_eq!(bits.set(64, false), Ok(true));
        assert_eq!(bits.count_ones(), 5);
    }

    #[test]
    fn empty_bit_set_has_no_words_and_zero_count() {
        let bits = BitSet::try_new(0).unwrap();
        assert_eq!(bits.bit_len(), 0);
        assert_eq!(bits.count_ones(), 0);
        assert_eq!(bits.get(0), None);
    }
}

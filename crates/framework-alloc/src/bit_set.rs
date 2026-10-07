use alloc::vec::Vec;

/// An allocation-free description of why a bit-set operation failed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BitSetError {
    /// The requested bit index is outside the set's fixed domain.
    OutOfRange,
    /// The allocator could not reserve the backing words.
    AllocationFailed,
}

/// A fixed-domain dense bit set backed by contiguous `u64` words.
pub struct BitSet {
    bit_len: u32,
    words: Vec<u64>,
}

impl BitSet {
    /// Creates a zeroed bit set with the requested fixed-width bit count.
    pub fn try_new(bit_len: u32) -> Result<Self, BitSetError> {
        let word_count = u64::from(bit_len).div_ceil(64);
        let word_count = usize::try_from(word_count).map_err(|_| BitSetError::AllocationFailed)?;
        let mut words = Vec::new();
        words
            .try_reserve_exact(word_count)
            .map_err(|_| BitSetError::AllocationFailed)?;
        words.resize(word_count, 0);
        Ok(Self { bit_len, words })
    }

    /// Returns the fixed-width number of bits in the set.
    pub const fn bit_len(&self) -> u32 {
        self.bit_len
    }

    /// Returns the value at an in-range bit index, or `None` when out of range.
    pub fn get(&self, index: u32) -> Option<bool> {
        if index >= self.bit_len {
            return None;
        }
        let word = usize::try_from(index / 64).ok()?;
        Some((self.words[word] & (1_u64 << (index % 64))) != 0)
    }

    /// Sets an in-range bit and returns its previous value.
    pub fn set(&mut self, index: u32, value: bool) -> Result<bool, BitSetError> {
        if index >= self.bit_len {
            return Err(BitSetError::OutOfRange);
        }
        let word = usize::try_from(index / 64).map_err(|_| BitSetError::OutOfRange)?;
        let mask = 1_u64 << (index % 64);
        let previous = (self.words[word] & mask) != 0;
        if value {
            self.words[word] |= mask;
        } else {
            self.words[word] &= !mask;
        }
        Ok(previous)
    }

    /// Clears all bits without changing the fixed domain.
    pub fn clear(&mut self) {
        self.words.fill(0);
    }

    /// Counts set bits and returns the fixed-width total.
    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|word| word.count_ones()).sum()
    }
}

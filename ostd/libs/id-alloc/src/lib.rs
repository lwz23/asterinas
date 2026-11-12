// SPDX-License-Identifier: MPL-2.0

#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::{fmt::Debug, ops::Range};

const BITS_PER_WORD: usize = u64::BITS as usize;

/// An id allocator implemented by a bitmap.
/// The true bit implies that the id is allocated, and vice versa.
//
// The bitmap is hand-rolled on `Vec<u64>` instead of using the `bitvec`
// crate: `bitvec` 1.x materializes `&mut BitSlice` references from tags
// created by zero-size retags, which violates Stacked Borrows and is
// reported as undefined behavior when the kernel runs under (K)Miri
// (same family as asterinas#3871).
#[derive(Clone)]
pub struct IdAlloc {
    words: Vec<u64>,
    len: usize,
    first_available_id: usize,
}

impl IdAlloc {
    /// Constructs a new id allocator with a maximum capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            words: alloc::vec![0u64; capacity.div_ceil(BITS_PER_WORD)],
            len: capacity,
            first_available_id: 0,
        }
    }

    fn get(&self, id: usize) -> bool {
        assert!(id < self.len);
        self.words[id / BITS_PER_WORD] & (1u64 << (id % BITS_PER_WORD)) != 0
    }

    fn set(&mut self, id: usize, value: bool) {
        assert!(id < self.len);
        let word = &mut self.words[id / BITS_PER_WORD];
        let mask = 1u64 << (id % BITS_PER_WORD);
        if value {
            *word |= mask;
        } else {
            *word &= !mask;
        }
    }

    /// Allocates and returns a new `id`.
    ///
    /// If allocation is not possible, it returns `None`.
    pub fn alloc(&mut self) -> Option<usize> {
        if self.first_available_id < self.len {
            let id = self.first_available_id;
            self.set(id, true);
            self.update_first_available_id(id + 1);
            Some(id)
        } else {
            None
        }
    }

    /// Allocates a consecutive range of new `id`s.
    ///
    /// The `count` is the number of consecutive `id`s to allocate. If it is 0, return `None`.
    ///
    /// If allocation is not possible, it returns `None`.
    ///
    /// TODO: Choose a more efficient strategy.
    pub fn alloc_consecutive(&mut self, count: usize) -> Option<Range<usize>> {
        if count == 0 {
            return None;
        }

        let end = self.first_available_id.checked_add(count)?;
        if end > self.len {
            return None;
        }

        // Scan the bitmap from the position `first_available_id`
        // for the first `count` number of consecutive 0's.
        let allocated_range = {
            // Invariance: all bits within `curr_range` are 0's
            let mut curr_range = self.first_available_id..self.first_available_id + 1;
            while curr_range.len() < count && curr_range.end < self.len {
                if !self.is_allocated(curr_range.end) {
                    curr_range.end += 1;
                } else {
                    curr_range = curr_range.end + 1..curr_range.end + 1;
                }
            }

            if curr_range.len() < count {
                return None;
            }

            curr_range
        };

        // Set every bit to 1 within the allocated range
        for id in allocated_range.clone() {
            self.set(id, true);
        }

        // In case we need to update first_available_id
        if self.is_allocated(self.first_available_id) {
            self.update_first_available_id(allocated_range.end);
        }

        Some(allocated_range)
    }

    /// Releases the consecutive range of allocated `id`s.
    ///
    /// # Panics
    ///
    /// If the `range` is out of bounds, this method will panic.
    pub fn free_consecutive(&mut self, range: Range<usize>) {
        if range.is_empty() {
            return;
        }

        let range_start = range.start;
        for id in range {
            debug_assert!(self.is_allocated(id));
            self.set(id, false);
        }

        if range_start < self.first_available_id {
            self.first_available_id = range_start
        }
    }

    /// Releases the allocated `id`.
    ///
    /// # Panics
    ///
    /// If the `id` is out of bounds, this method will panic.
    pub fn free(&mut self, id: usize) {
        debug_assert!(self.is_allocated(id));

        self.set(id, false);
        if id < self.first_available_id {
            self.first_available_id = id;
        }
    }

    /// Allocates a specific ID.
    ///
    /// If the ID is already allocated, it returns `None`, otherwise it
    /// returns the allocated ID.
    ///
    /// # Panics
    ///
    /// If the `id` is out of bounds, this method will panic.
    pub fn alloc_specific(&mut self, id: usize) -> Option<usize> {
        if self.get(id) {
            return None;
        }
        self.set(id, true);
        if id == self.first_available_id {
            self.update_first_available_id(id + 1);
        }
        Some(id)
    }

    /// Returns true if the `id` is allocated.
    ///
    /// # Panics
    ///
    /// If the `id` is out of bounds, this method will panic.
    pub fn is_allocated(&self, id: usize) -> bool {
        self.get(id)
    }

    /// Updates the `first_available_id` field to the first zero index at or after `start`.
    fn update_first_available_id(&mut self, start: usize) {
        let mut id = start;
        while id < self.len && self.get(id) {
            id += 1;
        }
        self.first_available_id = id;
    }
}

impl Debug for IdAlloc {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("IdAlloc")
            .field("len", &self.len)
            .field("first_available_id", &self.first_available_id)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::IdAlloc;

    #[test]
    fn bitmap_alloc_out_of_bounds() {
        let capacity = 16;
        let mut bitmap = IdAlloc::with_capacity(capacity);

        for _ in 0..capacity {
            assert!(bitmap.alloc().is_some());
        }

        // Allocating one more ID should fail since the
        // bitmap's `first_available_id` + `count` is out of bounds.
        assert!(bitmap.alloc_consecutive(1).is_none());
    }

    #[test]
    fn alloc_free_realloc() {
        let mut bitmap = IdAlloc::with_capacity(8);
        assert_eq!(bitmap.alloc(), Some(0));
        assert_eq!(bitmap.alloc(), Some(1));
        bitmap.free(0);
        assert!(!bitmap.is_allocated(0));
        assert_eq!(bitmap.alloc(), Some(0));
        assert_eq!(bitmap.alloc_specific(5), Some(5));
        assert_eq!(bitmap.alloc_specific(5), None);
        assert_eq!(bitmap.alloc_consecutive(3), Some(2..5));
        assert_eq!(bitmap.alloc(), Some(6));
    }
}

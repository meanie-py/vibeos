//! Bitmap allocators for the interrupt controllers' message vectors: which LPIs (its.rs)
//! and SPIs (v2m.rs) are handed out, and which events each ITS device has bound.
//!
//! Pure (no MMIO, no kernel imports), so tools/test_arm64_firmware_tables.py runs the
//! tests below on the host.

use core::sync::atomic::{AtomicU64, Ordering};

/// A lock-free set of `64 * W` slots, claimed lowest-first.
pub struct Bits<const W: usize>([AtomicU64; W]);

impl<const W: usize> Bits<W> {
    pub const fn new() -> Self {
        Bits([const { AtomicU64::new(0) }; W])
    }

    /// Claim the lowest free slot below `limit`.
    pub fn claim(&self, limit: u32) -> Option<u32> {
        let limit = limit.min((W * 64) as u32);
        for (w, word) in self.0.iter().enumerate() {
            let base = (w * 64) as u32;
            if base >= limit {
                break;
            }
            let mut current = word.load(Ordering::Relaxed);
            loop {
                let free = !current & valid_below(limit - base);
                if free == 0 {
                    break;
                }
                let bit = free.trailing_zeros();
                match word.compare_exchange_weak(current, current | 1 << bit, Ordering::AcqRel, Ordering::Relaxed) {
                    Ok(_) => return Some(base + bit),
                    Err(seen) => current = seen,
                }
            }
        }
        None
    }

    /// Claim `count` consecutive free slots below `limit`, the first of which plus
    /// `bias` is a multiple of `count` (a power of two), all or nothing: a multiple
    /// message MSI block, whose device varies the low bits of one data value, so the
    /// values must be consecutive and aligned. `bias` is what slot 0 stands for (a
    /// v2m frame's first SPI), so the alignment is of the value the device sends.
    pub fn claim_block(&self, count: u32, limit: u32, bias: u32) -> Option<u32> {
        if !count.is_power_of_two() {
            return None;
        }
        let limit = limit.min((W * 64) as u32);
        let mut first = (count - bias % count) % count;
        while first + count <= limit {
            if (first..first + count).all(|n| !self.is_set(n)) {
                let mut got = 0;
                while got < count && self.try_set(first + got) {
                    got += 1;
                }
                if got == count {
                    return Some(first);
                }
                // Lost a race for one of them: give back what this call took.
                for n in first..first + got {
                    self.release(n);
                }
            }
            first += count;
        }
        None
    }

    /// Set one slot; false when it was already set.
    fn try_set(&self, n: u32) -> bool {
        let Some(word) = self.0.get(n as usize / 64) else {
            return false;
        };
        let bit = 1u64 << (n % 64);
        word.fetch_or(bit, Ordering::AcqRel) & bit == 0
    }

    /// Give a slot back. False when it was not claimed: a double release.
    pub fn release(&self, n: u32) -> bool {
        let Some(word) = self.0.get(n as usize / 64) else {
            return false;
        };
        let bit = 1u64 << (n % 64);
        word.fetch_and(!bit, Ordering::AcqRel) & bit != 0
    }

    pub fn is_set(&self, n: u32) -> bool {
        self.0.get(n as usize / 64).is_some_and(|w| w.load(Ordering::Acquire) & 1 << (n % 64) != 0)
    }

    /// Slots claimed below `limit`.
    pub fn count(&self, limit: u32) -> u32 {
        let limit = limit.min((W * 64) as u32);
        self.0
            .iter()
            .enumerate()
            .take_while(|(w, _)| ((w * 64) as u32) < limit)
            .map(|(w, word)| (word.load(Ordering::Relaxed) & valid_below(limit - (w * 64) as u32)).count_ones())
            .sum()
    }
}

/// The mask of a word's slots that lie below `remaining` (which may exceed 64).
fn valid_below(remaining: u32) -> u64 {
    if remaining >= 64 { !0 } else { (1u64 << remaining) - 1 }
}

/// The `pick`th CPU (0-based, wrapping) of `set`, or None for an empty set. How a
/// message is placed: a rotor counts messages and each takes the next CPU in turn.
pub fn nth_cpu(set: u64, pick: u32) -> Option<u32> {
    if set == 0 {
        return None;
    }
    let mut rest = set;
    for _ in 0..pick % set.count_ones() {
        rest &= rest - 1;
    }
    Some(rest.trailing_zeros())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_lowest_first_and_reuses_what_is_released() {
        let b = Bits::<2>::new();
        assert_eq!((0..3).map(|_| b.claim(128).unwrap()).collect::<Vec<_>>(), [0, 1, 2]);
        assert!(b.release(1));
        assert!(!b.release(1), "a double release is reported");
        assert_eq!(b.claim(128), Some(1));
        assert_eq!(b.count(128), 3);
    }

    #[test]
    fn honours_the_limit_across_words() {
        let b = Bits::<2>::new();
        assert_eq!((0..70).filter_map(|_| b.claim(70)).count(), 70);
        assert_eq!(b.claim(70), None);
        assert!(b.is_set(69) && !b.is_set(70));
        assert_eq!(b.count(70), 70);
        assert_eq!(b.count(64), 64);
        // A limit past the bitmap is the bitmap.
        assert_eq!(Bits::<1>::new().count(1000), 0);
        let one = Bits::<1>::new();
        assert_eq!((0..100).filter_map(|_| one.claim(1000)).count(), 64);
    }

    #[test]
    fn out_of_range_slots_are_never_set() {
        let b = Bits::<1>::new();
        assert!(!b.release(64));
        assert!(!b.is_set(4096));
    }

    #[test]
    fn claims_aligned_blocks() {
        let b = Bits::<2>::new();
        assert_eq!(b.claim(128), Some(0));
        // Slot 0 is taken, so a block of 4 starts at the next multiple of 4.
        assert_eq!(b.claim_block(4, 128, 0), Some(4));
        assert!((4..8).all(|n| b.is_set(n)) && !b.is_set(8));
        // Biased: slot n stands for value n + 30, so the block's values start at 40.
        assert_eq!(b.claim_block(4, 128, 30), Some(10));
        // One taken slot inside every candidate leaves nothing for a block of 64.
        assert_eq!(b.claim_block(64, 128, 0), Some(64));
        assert_eq!(b.claim_block(64, 128, 0), None);
        assert_eq!(b.claim_block(3, 128, 0), None, "not a power of two");
        assert_eq!(b.claim_block(1, 128, 7), Some(1), "a block of one is any slot");
        assert_eq!(b.count(128), 1 + 4 + 4 + 64 + 1);
    }

    #[test]
    fn spreads_over_the_set_in_turn() {
        assert_eq!((0..5).map(|n| nth_cpu(0b1011, n).unwrap()).collect::<Vec<_>>(), [0, 1, 3, 0, 1]);
        assert_eq!(nth_cpu(0b100, 7), Some(2));
        assert_eq!(nth_cpu(0, 0), None);
    }
}

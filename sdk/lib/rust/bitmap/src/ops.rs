//! Core bitmap operations on `&[W]` slices, generic over word type.
//!
//! All functions match the i386 reference implementation in `sdk/lib/rtl/bitmap.c`.
//! Generic over `Word` trait — `u32` for production, `u8` for Kani verification.
//! The algorithms are identical regardless of word width; only the word boundary
//! positions change.

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr};

/// Sentinel: "not found" return value for find operations.
const MAXINDEX: u32 = 0xFFFF_FFFF;

// ============================================================================
// Word trait — abstraction over u8/u16/u32/u64
// ============================================================================

/// Trait for bitmap word types. Implemented for u8, u16, u32, u64.
pub trait Word:
    Copy
    + Eq
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitAndAssign
    + BitOrAssign
    + Not<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
{
    const BITS: u32;
    const ZERO: Self;
    const MAX: Self;
    fn trailing_zeros(self) -> u32;
    fn leading_zeros(self) -> u32;
    fn count_ones(self) -> u32;
}

macro_rules! impl_word {
    ($t:ty) => {
        impl Word for $t {
            const BITS: u32 = <$t>::BITS;
            const ZERO: Self = 0;
            const MAX: Self = !0;
            #[inline]
            fn trailing_zeros(self) -> u32 {
                <$t>::trailing_zeros(self)
            }
            #[inline]
            fn leading_zeros(self) -> u32 {
                <$t>::leading_zeros(self)
            }
            #[inline]
            fn count_ones(self) -> u32 {
                <$t>::count_ones(self)
            }
        }
    };
}

impl_word!(u8);
impl_word!(u16);
impl_word!(u32);
impl_word!(u64);

/// Returns `1` as a `W` value.
#[inline]
fn one<W: Word>() -> W {
    W::MAX >> (W::BITS - 1)
}

// ============================================================================
// Internal helpers (match RtlpGetLengthOfRunClear / RtlpGetLengthOfRunSet)
// ============================================================================

/// Length of a contiguous run of clear (0) bits starting at `start`.
pub fn run_length_clear<W: Word>(buf: &[W], size: u32, start: u32, max_len: u32) -> u32 {
    if start >= size {
        return 0;
    }
    let max_len = max_len.min(size - start);
    let mut idx = (start / W::BITS) as usize;
    let bit_pos = start & (W::BITS - 1);
    let max_idx = idx + ((bit_pos + max_len + W::BITS - 1) / W::BITS) as usize;

    // First word: mask off bits below start
    let mut value = buf[idx] >> bit_pos << bit_pos;
    idx += 1;

    // Skip all-zero words
    while value == W::ZERO && idx < max_idx {
        value = buf[idx];
        idx += 1;
    }

    if value == W::ZERO {
        return max_len;
    }

    // Found a set bit — count clear bits before it
    let bit_pos = value.trailing_zeros();
    let length = (idx as u32) * W::BITS - start + bit_pos - W::BITS;
    length.min(size - start)
}

/// Length of a contiguous run of set (1) bits starting at `start`.
pub fn run_length_set<W: Word>(buf: &[W], size: u32, start: u32, max_len: u32) -> u32 {
    if start >= size {
        return 0;
    }
    let max_len = max_len.min(size - start);
    let mut idx = (start / W::BITS) as usize;
    let bit_pos = start & (W::BITS - 1);
    let max_idx = idx + ((bit_pos + max_len + W::BITS - 1) / W::BITS) as usize;

    // First word: invert, then mask off bits below start
    let mut inv_value = !buf[idx] >> bit_pos << bit_pos;
    idx += 1;

    // Skip all-set words (inverted = 0)
    while inv_value == W::ZERO && idx < max_idx {
        inv_value = !buf[idx];
        idx += 1;
    }

    if inv_value == W::ZERO {
        return max_len;
    }

    // Found a clear bit — count set bits before it
    let bit_pos = inv_value.trailing_zeros();
    let length = (idx as u32) * W::BITS - start + bit_pos - W::BITS;
    length.min(size - start)
}

// ============================================================================
// Public functions (match bitmap.c API exactly)
// ============================================================================

/// `RtlClearAllBits`
pub fn clear_all<W: Word>(buf: &mut [W], size: u32) {
    let words = ((size + W::BITS - 1) / W::BITS) as usize;
    for w in buf[..words].iter_mut() {
        *w = W::ZERO;
    }
}

/// `RtlSetAllBits`
pub fn set_all<W: Word>(buf: &mut [W], size: u32) {
    let words = ((size + W::BITS - 1) / W::BITS) as usize;
    for w in buf[..words].iter_mut() {
        *w = W::MAX;
    }
}

/// `RtlClearBit`
pub fn clear_bit<W: Word>(buf: &mut [W], bit: u32) {
    buf[(bit / W::BITS) as usize] &= !(one::<W>() << (bit & (W::BITS - 1)));
}

/// `RtlSetBit`
pub fn set_bit<W: Word>(buf: &mut [W], bit: u32) {
    buf[(bit / W::BITS) as usize] |= one::<W>() << (bit & (W::BITS - 1));
}

/// `RtlTestBit`
pub fn test_bit<W: Word>(buf: &[W], bit: u32) -> bool {
    (buf[(bit / W::BITS) as usize] >> (bit & (W::BITS - 1))) & one::<W>() != W::ZERO
}

/// `RtlClearBits` — clear a range of bits.
pub fn clear_bits<W: Word>(buf: &mut [W], start: u32, count: u32) {
    if count == 0 {
        return;
    }
    let mut idx = (start / W::BITS) as usize;
    let mut bits = start & (W::BITS - 1);
    let mut remaining = count;

    // Handle unaligned start
    if bits != 0 {
        let mut mask = W::MAX << bits;
        let avail = W::BITS - bits;
        if remaining < avail {
            // Only clearing within this word
            bits = avail - remaining;
            mask = mask << bits >> bits;
            buf[idx] &= !mask;
            return;
        }
        buf[idx] &= !mask;
        idx += 1;
        remaining -= avail;
    }

    // Clear full words
    let full_words = (remaining / W::BITS) as usize;
    for w in buf[idx..idx + full_words].iter_mut() {
        *w = W::ZERO;
    }
    idx += full_words;
    remaining &= W::BITS - 1;

    // Handle remaining bits
    if remaining != 0 {
        let mask = W::MAX << remaining;
        buf[idx] &= mask;
    }
}

/// `RtlSetBits` — set a range of bits.
pub fn set_bits<W: Word>(buf: &mut [W], start: u32, count: u32) {
    if count == 0 {
        return;
    }
    let mut idx = (start / W::BITS) as usize;
    let mut bits = start & (W::BITS - 1);
    let mut remaining = count;

    // Handle unaligned start
    if bits != 0 {
        let mut mask = W::MAX << bits;
        let avail = W::BITS - bits;
        if remaining < avail {
            bits = avail - remaining;
            mask = mask << bits >> bits;
            buf[idx] |= mask;
            return;
        }
        buf[idx] |= mask;
        idx += 1;
        remaining -= avail;
    }

    // Set full words
    let full_words = (remaining / W::BITS) as usize;
    for w in buf[idx..idx + full_words].iter_mut() {
        *w = W::MAX;
    }
    idx += full_words;
    remaining &= W::BITS - 1;

    // Handle remaining bits
    if remaining != 0 {
        let mask = W::MAX << remaining;
        buf[idx] |= !mask;
    }
}

/// `RtlAreBitsClear`
pub fn are_bits_clear<W: Word>(buf: &[W], size: u32, start: u32, length: u32) -> bool {
    if start + length > size || start + length <= start {
        return false;
    }
    run_length_clear(buf, size, start, length) >= length
}

/// `RtlAreBitsSet`
pub fn are_bits_set<W: Word>(buf: &[W], size: u32, start: u32, length: u32) -> bool {
    if start + length > size || start + length <= start {
        return false;
    }
    run_length_set(buf, size, start, length) >= length
}

/// `RtlNumberOfSetBits` — uses `count_ones()` per word (no unsafe, no lookup table).
pub fn number_of_set_bits<W: Word>(buf: &[W], size: u32) -> u32 {
    let full_words = (size / W::BITS) as usize;
    let mut count: u32 = 0;
    for i in 0..full_words {
        count += buf[i].count_ones();
    }
    let rem = size % W::BITS;
    if rem != 0 {
        // Mask: keep only the low `rem` bits of the last partial word
        let mask = !(W::MAX << rem);
        count += (buf[full_words] & mask).count_ones();
    }
    count
}

/// `RtlNumberOfClearBits`
pub fn number_of_clear_bits<W: Word>(buf: &[W], size: u32) -> u32 {
    size - number_of_set_bits(buf, size)
}

/// `RtlFindClearBits`
pub fn find_clear_bits<W: Word>(buf: &[W], size: u32, number_to_find: u32, hint: u32) -> u32 {
    if number_to_find > size {
        return MAXINDEX;
    }
    let mut hint = if hint >= size { 0 } else { hint };
    if number_to_find == 0 {
        return hint & !7;
    }

    let mut margin = size;

    loop {
        let mut current = hint;
        while current + number_to_find <= margin {
            current += run_length_set(buf, size, current, MAXINDEX);
            let clear_len = run_length_clear(buf, size, current, number_to_find);
            if clear_len >= number_to_find {
                return current;
            }
            current += clear_len;
        }

        if hint != 0 {
            margin = (hint + number_to_find).min(size);
            hint = 0;
        } else {
            break;
        }
    }

    MAXINDEX
}

/// `RtlFindSetBits`
pub fn find_set_bits<W: Word>(buf: &[W], size: u32, number_to_find: u32, hint: u32) -> u32 {
    if number_to_find > size {
        return MAXINDEX;
    }
    let mut hint = if hint >= size { 0 } else { hint };
    if number_to_find == 0 {
        return hint & !7;
    }

    let mut margin = size;

    loop {
        let mut current = hint;
        while current + number_to_find <= margin {
            current += run_length_clear(buf, size, current, MAXINDEX);
            let set_len = run_length_set(buf, size, current, number_to_find);
            if set_len >= number_to_find {
                return current;
            }
            current += set_len;
        }

        if hint != 0 {
            margin = (hint + number_to_find).min(size);
            hint = 0;
        } else {
            break;
        }
    }

    MAXINDEX
}

/// `RtlFindClearBitsAndSet`
pub fn find_clear_bits_and_set<W: Word>(
    buf: &mut [W],
    size: u32,
    number_to_find: u32,
    hint: u32,
) -> u32 {
    let pos = find_clear_bits(buf, size, number_to_find, hint);
    if pos != MAXINDEX {
        set_bits(buf, pos, number_to_find);
    }
    pos
}

/// `RtlFindSetBitsAndClear`
pub fn find_set_bits_and_clear<W: Word>(
    buf: &mut [W],
    size: u32,
    number_to_find: u32,
    hint: u32,
) -> u32 {
    let pos = find_set_bits(buf, size, number_to_find, hint);
    if pos != MAXINDEX {
        clear_bits(buf, pos, number_to_find);
    }
    pos
}

/// `RtlFindNextForwardRunClear` — returns (starting_index, length).
pub fn find_next_forward_run_clear<W: Word>(buf: &[W], size: u32, from: u32) -> (u32, u32) {
    if from >= size {
        return (from, 0);
    }
    let set_len = run_length_set(buf, size, from, MAXINDEX);
    let start = from + set_len;
    let clear_len = run_length_clear(buf, size, start, MAXINDEX);
    (start, clear_len)
}

/// `RtlFindNextForwardRunSet` — returns (starting_index, length).
pub fn find_next_forward_run_set<W: Word>(buf: &[W], size: u32, from: u32) -> (u32, u32) {
    if from >= size {
        return (from, 0);
    }
    let clear_len = run_length_clear(buf, size, from, MAXINDEX);
    let start = from + clear_len;
    let set_len = run_length_set(buf, size, start, MAXINDEX);
    (start, set_len)
}

/// `RtlFindFirstRunClear`
pub fn find_first_run_clear<W: Word>(buf: &[W], size: u32) -> (u32, u32) {
    find_next_forward_run_clear(buf, size, 0)
}

/// `RtlFindLastBackwardRunClear` — returns (starting_index, length).
pub fn find_last_backward_run_clear<W: Word>(buf: &[W], size: u32, from: u32) -> (u32, u32) {
    if size == 0 {
        return (0, 0);
    }
    let from = from.min(size - 1);
    let mut idx = (from / W::BITS) as usize;
    let bit_pos = (W::BITS - 1) - (from & (W::BITS - 1));

    // Invert, mask off bits above `from`
    let mut inv_value = !buf[idx] << bit_pos >> bit_pos;

    // Skip all-set words (inverted = 0), scanning backward
    while inv_value == W::ZERO {
        if idx == 0 {
            return (0, 0); // No clear bits found
        }
        idx -= 1;
        inv_value = !buf[idx];
    }

    // Found a clear bit via BitScanReverse
    let high_bit = (W::BITS - 1) - inv_value.leading_zeros();
    let end_index = (idx as u32) * W::BITS + high_bit;

    // Now scan backward for set bits to find the start of the clear run
    let mut value = !inv_value << ((W::BITS - 1) - high_bit) >> ((W::BITS - 1) - high_bit);

    while value == W::ZERO && idx > 0 {
        idx -= 1;
        value = buf[idx];
    }

    let start_index = if value != W::ZERO {
        let high_bit = (W::BITS - 1) - value.leading_zeros();
        (idx as u32) * W::BITS + high_bit + 1
    } else {
        0
    };

    // C code has off-by-one: returns (FromIndex - StartingRunIndex) without +1.
    // We return the correct inclusive length.
    (start_index, end_index - start_index + 1)
}

/// `RtlFindClearRuns` — find multiple clear runs.
/// Returns the number of runs found. Writes to `runs` slice as (start, length) pairs.
pub fn find_clear_runs<W: Word>(
    buf: &[W],
    size: u32,
    runs: &mut [(u32, u32)],
    locate_longest: bool,
) -> u32 {
    let max_runs = runs.len();
    let mut from: u32 = 0;
    let mut count: usize = 0;
    let mut smallest: usize = 0;

    // Fill the runs array
    while count < max_runs {
        let (start, length) = find_next_forward_run_clear(buf, size, from);
        if length == 0 {
            break;
        }
        runs[count] = (start, length);
        if length < runs[smallest].1 {
            smallest = count;
        }
        count += 1;
        from = start + length;
    }

    if count < max_runs || !locate_longest {
        return count as u32;
    }

    // Continue scanning, replacing the smallest run if we find a longer one
    loop {
        let (start, length) = find_next_forward_run_clear(buf, size, from);
        if length == 0 {
            break;
        }
        if length > runs[smallest].1 {
            runs[smallest] = (start, length);
            // Find new smallest
            for i in 0..max_runs {
                if runs[i].1 < runs[smallest].1 {
                    smallest = i;
                }
            }
        }
        from += length;
    }

    count as u32
}

/// `RtlFindLongestRunClear`
pub fn find_longest_run_clear<W: Word>(buf: &[W], size: u32) -> (u32, u32) {
    let mut from: u32 = 0;
    let mut best_start: u32 = 0;
    let mut best_len: u32 = 0;

    loop {
        let (start, length) = find_next_forward_run_clear(buf, size, from);
        if length == 0 {
            break;
        }
        if length > best_len {
            best_len = length;
            best_start = start;
        }
        from = start + length;
    }

    (best_start, best_len)
}

/// `RtlFindLongestRunSet`
pub fn find_longest_run_set<W: Word>(buf: &[W], size: u32) -> (u32, u32) {
    let mut from: u32 = 0;
    let mut best_start: u32 = 0;
    let mut best_len: u32 = 0;

    loop {
        let (start, length) = find_next_forward_run_set(buf, size, from);
        if length == 0 {
            break;
        }
        if length > best_len {
            best_len = length;
            best_start = start;
        }
        from = start + length;
    }

    (best_start, best_len)
}

/// `RtlFindMostSignificantBit` (not generic — operates on u64 directly)
pub fn find_most_significant_bit(value: u64) -> i8 {
    if value == 0 {
        return -1;
    }
    (63 - value.leading_zeros()) as i8
}

/// `RtlFindLeastSignificantBit` (not generic — operates on u64 directly)
pub fn find_least_significant_bit(value: u64) -> i8 {
    if value == 0 {
        return -1;
    }
    value.trailing_zeros() as i8
}

// ============================================================================
// Unit tests — generic implementations instantiated for u8 and u32
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make<W: Word>(nwords: usize) -> Vec<W> {
        vec![W::ZERO; nwords]
    }

    // --- Generic test implementations ---

    fn check_clear_all<W: Word>() {
        let mut buf = vec![W::MAX; 4];
        clear_all(&mut buf, 4 * W::BITS);
        assert!(buf.iter().all(|&w| w == W::ZERO));
    }

    fn check_set_all<W: Word>() {
        let mut buf = make::<W>(4);
        set_all(&mut buf, 4 * W::BITS);
        assert!(buf.iter().all(|&w| w == W::MAX));
    }

    fn check_set_and_test_bit<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_bit(&mut buf, 0);
        set_bit(&mut buf, 7);
        set_bit(&mut buf, W::BITS - 1); // last bit of first word
        set_bit(&mut buf, size - 1); // last bit overall
        assert!(test_bit(&buf, 0));
        assert!(test_bit(&buf, 7));
        assert!(test_bit(&buf, W::BITS - 1));
        assert!(test_bit(&buf, size - 1));
        assert!(!test_bit(&buf, 1));
        assert!(!test_bit(&buf, size - 2));
    }

    fn check_clear_bit<W: Word>() {
        let mut buf = make::<W>(2);
        set_bit(&mut buf, 7);
        assert!(test_bit(&buf, 7));
        clear_bit(&mut buf, 7);
        assert!(!test_bit(&buf, 7));
    }

    fn check_set_bits_range<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_bits(&mut buf, 10, 10);
        assert_eq!(number_of_set_bits(&buf, size), 10);
        assert!(!test_bit(&buf, 9));
        assert!(test_bit(&buf, 10));
        assert!(test_bit(&buf, 19));
        assert!(!test_bit(&buf, 20));
    }

    fn check_clear_bits_range<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_all(&mut buf, size);
        clear_bits(&mut buf, 10, 10);
        assert_eq!(number_of_set_bits(&buf, size), size - 10);
        assert!(test_bit(&buf, 9));
        assert!(!test_bit(&buf, 10));
        assert!(!test_bit(&buf, 19));
        assert!(test_bit(&buf, 20));
    }

    fn check_set_bits_unaligned<W: Word>() {
        let mut buf = make::<W>(2);
        // Cross word boundary: start 2 bits before end of first word
        let start = W::BITS - 2;
        set_bits(&mut buf, start, 4);
        assert!(!test_bit(&buf, start - 1));
        assert!(test_bit(&buf, start));
        assert!(test_bit(&buf, start + 1));
        assert!(test_bit(&buf, start + 2));
        assert!(test_bit(&buf, start + 3));
        assert!(!test_bit(&buf, start + 4));
    }

    fn check_number_of_set_bits_empty<W: Word>() {
        let buf = make::<W>(8);
        assert_eq!(number_of_set_bits(&buf, 8 * W::BITS), 0);
    }

    fn check_number_of_set_bits_full<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_all(&mut buf, size);
        assert_eq!(number_of_set_bits(&buf, size), size);
    }

    fn check_find_clear_bits_basic<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_bits(&mut buf, 10, 10);
        let idx = find_clear_bits(&buf, size, 5, 0);
        assert_ne!(idx, MAXINDEX);
        assert!(idx < 10 || idx >= 20);
    }

    fn check_find_set_bits_basic<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_bits(&mut buf, 10, 10);
        let idx = find_set_bits(&buf, size, 5, 0);
        assert_ne!(idx, MAXINDEX);
        assert!(idx >= 10 && idx <= 15);
    }

    fn check_find_clear_bits_full<W: Word>() {
        let size = 8 * W::BITS;
        let mut buf = make::<W>(8);
        set_all(&mut buf, size);
        assert_eq!(find_clear_bits(&buf, size, 1, 0), MAXINDEX);
    }

    fn check_find_clear_bits_and_set<W: Word>() {
        let size = 2 * W::BITS;
        let mut buf = make::<W>(2);
        let pos = find_clear_bits_and_set(&mut buf, size, 10, 0);
        assert_ne!(pos, MAXINDEX);
        assert_eq!(number_of_set_bits(&buf, size), 10);
    }

    fn check_are_bits_clear_and_set<W: Word>() {
        let size = 2 * W::BITS;
        let mut buf = make::<W>(2);
        set_bits(&mut buf, 2, 10);
        assert!(are_bits_set(&buf, size, 2, 10));
        assert!(are_bits_clear(&buf, size, 0, 2));
        assert!(!are_bits_set(&buf, size, 1, 10));
    }

    fn check_find_longest_run_clear<W: Word>() {
        let size = 4 * W::BITS;
        let mut buf = make::<W>(4);
        set_all(&mut buf, size);
        clear_bits(&mut buf, 2, 5);
        clear_bits(&mut buf, W::BITS + 2, 20);
        clear_bits(&mut buf, 3 * W::BITS, 3);
        let (start, len) = find_longest_run_clear(&buf, size);
        assert_eq!(start, W::BITS + 2);
        assert_eq!(len, 20);
    }

    fn check_backward_run_clear<W: Word>() {
        let size = 4 * W::BITS;
        let mut buf = make::<W>(4);
        set_all(&mut buf, size);
        clear_bits(&mut buf, W::BITS + 2, 20);
        let (start, len) = find_last_backward_run_clear(&buf, size, size - 1);
        assert_eq!(start, W::BITS + 2);
        assert_eq!(len, 20);
    }

    // --- u32 tests (production word size) ---
    mod u32_tests {
        use super::*;
        #[test] fn clear_all() { check_clear_all::<u32>() }
        #[test] fn set_all() { check_set_all::<u32>() }
        #[test] fn set_and_test_bit() { check_set_and_test_bit::<u32>() }
        #[test] fn clear_bit() { check_clear_bit::<u32>() }
        #[test] fn set_bits_range() { check_set_bits_range::<u32>() }
        #[test] fn clear_bits_range() { check_clear_bits_range::<u32>() }
        #[test] fn set_bits_unaligned() { check_set_bits_unaligned::<u32>() }
        #[test] fn number_of_set_bits_empty() { check_number_of_set_bits_empty::<u32>() }
        #[test] fn number_of_set_bits_full() { check_number_of_set_bits_full::<u32>() }
        #[test] fn find_clear_bits_basic() { check_find_clear_bits_basic::<u32>() }
        #[test] fn find_set_bits_basic() { check_find_set_bits_basic::<u32>() }
        #[test] fn find_clear_bits_full() { check_find_clear_bits_full::<u32>() }
        #[test] fn find_clear_bits_and_set() { check_find_clear_bits_and_set::<u32>() }
        #[test] fn are_bits_clear_and_set() { check_are_bits_clear_and_set::<u32>() }
        #[test] fn find_longest_run_clear() { check_find_longest_run_clear::<u32>() }
        #[test] fn backward_run_clear() { check_backward_run_clear::<u32>() }
    }

    // --- u8 tests (Kani-compatible word size) ---
    mod u8_tests {
        use super::*;
        #[test] fn clear_all() { check_clear_all::<u8>() }
        #[test] fn set_all() { check_set_all::<u8>() }
        #[test] fn set_and_test_bit() { check_set_and_test_bit::<u8>() }
        #[test] fn clear_bit() { check_clear_bit::<u8>() }
        #[test] fn set_bits_range() { check_set_bits_range::<u8>() }
        #[test] fn clear_bits_range() { check_clear_bits_range::<u8>() }
        #[test] fn set_bits_unaligned() { check_set_bits_unaligned::<u8>() }
        #[test] fn number_of_set_bits_empty() { check_number_of_set_bits_empty::<u8>() }
        #[test] fn number_of_set_bits_full() { check_number_of_set_bits_full::<u8>() }
        #[test] fn find_clear_bits_basic() { check_find_clear_bits_basic::<u8>() }
        #[test] fn find_set_bits_basic() { check_find_set_bits_basic::<u8>() }
        #[test] fn find_clear_bits_full() { check_find_clear_bits_full::<u8>() }
        #[test] fn find_clear_bits_and_set() { check_find_clear_bits_and_set::<u8>() }
        #[test] fn are_bits_clear_and_set() { check_are_bits_clear_and_set::<u8>() }
        #[test] fn find_longest_run_clear() { check_find_longest_run_clear::<u8>() }
        #[test] fn backward_run_clear() { check_backward_run_clear::<u8>() }
    }

    // --- Non-generic tests (MSB/LSB operate on u64 directly) ---
    #[test]
    fn find_most_significant_bit_works() {
        assert_eq!(find_most_significant_bit(0), -1);
        assert_eq!(find_most_significant_bit(1), 0);
        assert_eq!(find_most_significant_bit(0x8000_0000_0000_0000), 63);
        assert_eq!(find_most_significant_bit(0xFF), 7);
    }

    #[test]
    fn find_least_significant_bit_works() {
        assert_eq!(find_least_significant_bit(0), -1);
        assert_eq!(find_least_significant_bit(1), 0);
        assert_eq!(find_least_significant_bit(0x8000_0000_0000_0000), 63);
        assert_eq!(find_least_significant_bit(0x100), 8);
    }
}

// ============================================================================
// Kani verification proofs — using u8 for tractable state space
// ============================================================================

#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn set_bit_test_bit_roundtrip() {
        let mut buf = [0u8; 1];
        let bit: u32 = kani::any();
        kani::assume(bit < 8);
        set_bit(&mut buf, bit);
        assert!(test_bit(&buf, bit));
    }

    #[kani::proof]
    fn clear_bit_test_bit_roundtrip() {
        let mut buf = [0xFFu8; 1];
        let bit: u32 = kani::any();
        kani::assume(bit < 8);
        clear_bit(&mut buf, bit);
        assert!(!test_bit(&buf, bit));
    }

    #[kani::proof]
    fn set_bit_does_not_disturb_others() {
        let mut buf = [0u8; 1];
        let bit: u32 = kani::any();
        let other: u32 = kani::any();
        kani::assume(bit < 8 && other < 8 && bit != other);
        set_bit(&mut buf, bit);
        assert!(!test_bit(&buf, other));
    }

    #[kani::proof]
    fn clear_all_then_number_of_set_is_zero() {
        let mut buf = [kani::any::<u8>(); 1];
        clear_all(&mut buf, 8);
        assert_eq!(number_of_set_bits(&buf, 8), 0);
    }

    #[kani::proof]
    fn set_all_then_number_of_set_is_size() {
        let mut buf = [kani::any::<u8>(); 1];
        set_all(&mut buf, 8);
        assert_eq!(number_of_set_bits(&buf, 8), 8);
    }

    #[kani::proof]
    #[kani::unwind(4)] // 2 u8 words → max 2 loop iterations + termination
    fn set_clear_bits_range_cancel() {
        let mut buf = [0u8; 2];
        let start: u32 = kani::any();
        let count: u32 = kani::any();
        kani::assume(start < 16 && count > 0 && count <= 16 - start);
        set_bits(&mut buf, start, count);
        assert_eq!(number_of_set_bits(&buf, 16), count);
        clear_bits(&mut buf, start, count);
        assert_eq!(number_of_set_bits(&buf, 16), 0);
    }

    #[kani::proof]
    fn find_msb_matches_leading_zeros() {
        let value: u64 = kani::any();
        let result = find_most_significant_bit(value);
        if value == 0 {
            assert_eq!(result, -1);
        } else {
            assert_eq!(result, (63 - value.leading_zeros()) as i8);
        }
    }

    #[kani::proof]
    fn find_lsb_matches_trailing_zeros() {
        let value: u64 = kani::any();
        let result = find_least_significant_bit(value);
        if value == 0 {
            assert_eq!(result, -1);
        } else {
            assert_eq!(result, value.trailing_zeros() as i8);
        }
    }

    #[kani::proof]
    #[kani::unwind(4)] // 2 u8 words → max 2 loop iterations + termination
    fn are_bits_set_after_set_bits() {
        let mut buf = [0u8; 2];
        let start: u32 = kani::any();
        let count: u32 = kani::any();
        kani::assume(start < 16 && count > 0 && count <= 16 - start);
        set_bits(&mut buf, start, count);
        assert!(are_bits_set(&buf, 16, start, count));
    }
}

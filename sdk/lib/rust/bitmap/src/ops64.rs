//! 64-bit bitmap operations (`RTL_BITMAP64`) — `u64` word **and**
//! `u64` index. Faithful mirror of `bitmap.c` compiled with
//! `USE_RTL_BITMAP64` (`_BITCOUNT=64`, `BITMAP_INDEX/BUFFER=ULONG64`,
//! `MAXINDEX=0xFFFFFFFFFFFFFFFF`).
//!
//! This is a deliberate, isolated copy of the proven generic 32-bit
//! algorithms in [`crate::ops`] re-typed to 64-bit indices rather
//! than an index-genericization of that module: `ops.rs` is
//! kernel-linked on all 3 arches with no differential-test safety net,
//! and `bitmap64.c` is amd64-only with zero in-tree callers, so the
//! risk asymmetry favours duplication over refactoring load-bearing
//! code. The algorithm structure is identical to `ops.rs`; only the
//! word/index width changes.

const BITS: u64 = 64;
const MAXINDEX: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[inline]
fn words_for(size: u64) -> usize {
    ((size + BITS - 1) / BITS) as usize
}

/// Length of a contiguous run of clear bits starting at `start`
/// (mirrors `RtlpGetLengthOfRunClear`).
pub fn run_length_clear(buf: &[u64], size: u64, start: u64, max_len: u64) -> u64 {
    if start >= size {
        return 0;
    }
    let max_len = max_len.min(size - start);
    let mut idx = (start / BITS) as usize;
    let bit_pos = (start & (BITS - 1)) as u32;
    let max_idx = idx + (((start & (BITS - 1)) + max_len + BITS - 1) / BITS) as usize;

    let mut value = buf[idx] >> bit_pos << bit_pos;
    idx += 1;

    while value == 0 && idx < max_idx {
        value = buf[idx];
        idx += 1;
    }

    if value == 0 {
        return max_len;
    }

    let bit_pos = value.trailing_zeros() as u64;
    let length = (idx as u64) * BITS - start + bit_pos - BITS;
    length.min(size - start)
}

/// Length of a contiguous run of set bits starting at `start`.
pub fn run_length_set(buf: &[u64], size: u64, start: u64, max_len: u64) -> u64 {
    if start >= size {
        return 0;
    }
    let max_len = max_len.min(size - start);
    let mut idx = (start / BITS) as usize;
    let bit_pos = (start & (BITS - 1)) as u32;
    let max_idx = idx + (((start & (BITS - 1)) + max_len + BITS - 1) / BITS) as usize;

    let mut inv_value = !buf[idx] >> bit_pos << bit_pos;
    idx += 1;

    while inv_value == 0 && idx < max_idx {
        inv_value = !buf[idx];
        idx += 1;
    }

    if inv_value == 0 {
        return max_len;
    }

    let bit_pos = inv_value.trailing_zeros() as u64;
    let length = (idx as u64) * BITS - start + bit_pos - BITS;
    length.min(size - start)
}

pub fn clear_all(buf: &mut [u64], size: u64) {
    for w in buf[..words_for(size)].iter_mut() {
        *w = 0;
    }
}

pub fn set_all(buf: &mut [u64], size: u64) {
    for w in buf[..words_for(size)].iter_mut() {
        *w = !0;
    }
}

pub fn clear_bit(buf: &mut [u64], bit: u64) {
    buf[(bit / BITS) as usize] &= !(1u64 << (bit & (BITS - 1)));
}

pub fn set_bit(buf: &mut [u64], bit: u64) {
    buf[(bit / BITS) as usize] |= 1u64 << (bit & (BITS - 1));
}

pub fn test_bit(buf: &[u64], bit: u64) -> bool {
    (buf[(bit / BITS) as usize] >> (bit & (BITS - 1))) & 1 != 0
}

pub fn clear_bits(buf: &mut [u64], start: u64, count: u64) {
    if count == 0 {
        return;
    }
    let mut idx = (start / BITS) as usize;
    let mut bits = start & (BITS - 1);
    let mut remaining = count;

    if bits != 0 {
        let mut mask = u64::MAX << bits;
        let avail = BITS - bits;
        if remaining < avail {
            bits = avail - remaining;
            mask = mask << bits >> bits;
            buf[idx] &= !mask;
            return;
        }
        buf[idx] &= !mask;
        idx += 1;
        remaining -= avail;
    }

    let full_words = (remaining / BITS) as usize;
    for w in buf[idx..idx + full_words].iter_mut() {
        *w = 0;
    }
    idx += full_words;
    remaining &= BITS - 1;

    if remaining != 0 {
        let mask = u64::MAX << remaining;
        buf[idx] &= mask;
    }
}

pub fn set_bits(buf: &mut [u64], start: u64, count: u64) {
    if count == 0 {
        return;
    }
    let mut idx = (start / BITS) as usize;
    let mut bits = start & (BITS - 1);
    let mut remaining = count;

    if bits != 0 {
        let mut mask = u64::MAX << bits;
        let avail = BITS - bits;
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

    let full_words = (remaining / BITS) as usize;
    for w in buf[idx..idx + full_words].iter_mut() {
        *w = !0;
    }
    idx += full_words;
    remaining &= BITS - 1;

    if remaining != 0 {
        let mask = u64::MAX << remaining;
        buf[idx] |= !mask;
    }
}

pub fn are_bits_clear(buf: &[u64], size: u64, start: u64, length: u64) -> bool {
    if start + length > size || start + length <= start {
        return false;
    }
    run_length_clear(buf, size, start, length) >= length
}

pub fn are_bits_set(buf: &[u64], size: u64, start: u64, length: u64) -> bool {
    if start + length > size || start + length <= start {
        return false;
    }
    run_length_set(buf, size, start, length) >= length
}

pub fn number_of_set_bits(buf: &[u64], size: u64) -> u64 {
    let full_words = (size / BITS) as usize;
    let mut count: u64 = 0;
    for &w in buf.iter().take(full_words) {
        count += w.count_ones() as u64;
    }
    let rem = (size % BITS) as u32;
    if rem != 0 {
        let mask = !(u64::MAX << rem);
        count += (buf[full_words] & mask).count_ones() as u64;
    }
    count
}

pub fn number_of_clear_bits(buf: &[u64], size: u64) -> u64 {
    size - number_of_set_bits(buf, size)
}

pub fn find_clear_bits(buf: &[u64], size: u64, number_to_find: u64, hint: u64) -> u64 {
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

pub fn find_set_bits(buf: &[u64], size: u64, number_to_find: u64, hint: u64) -> u64 {
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

pub fn find_clear_bits_and_set(buf: &mut [u64], size: u64, n: u64, hint: u64) -> u64 {
    let pos = find_clear_bits(buf, size, n, hint);
    if pos != MAXINDEX {
        set_bits(buf, pos, n);
    }
    pos
}

pub fn find_set_bits_and_clear(buf: &mut [u64], size: u64, n: u64, hint: u64) -> u64 {
    let pos = find_set_bits(buf, size, n, hint);
    if pos != MAXINDEX {
        clear_bits(buf, pos, n);
    }
    pos
}

pub fn find_next_forward_run_clear(buf: &[u64], size: u64, from: u64) -> (u64, u64) {
    if from >= size {
        return (from, 0);
    }
    let set_len = run_length_set(buf, size, from, MAXINDEX);
    let start = from + set_len;
    let clear_len = run_length_clear(buf, size, start, MAXINDEX);
    (start, clear_len)
}

pub fn find_next_forward_run_set(buf: &[u64], size: u64, from: u64) -> (u64, u64) {
    if from >= size {
        return (from, 0);
    }
    let clear_len = run_length_clear(buf, size, from, MAXINDEX);
    let start = from + clear_len;
    let set_len = run_length_set(buf, size, start, MAXINDEX);
    (start, set_len)
}

pub fn find_first_run_clear(buf: &[u64], size: u64) -> (u64, u64) {
    find_next_forward_run_clear(buf, size, 0)
}

pub fn find_last_backward_run_clear(buf: &[u64], size: u64, from: u64) -> (u64, u64) {
    if size == 0 {
        return (0, 0);
    }
    let from = from.min(size - 1);
    let mut idx = (from / BITS) as usize;
    let bit_pos = ((BITS - 1) - (from & (BITS - 1))) as u32;

    let mut inv_value = !buf[idx] << bit_pos >> bit_pos;

    while inv_value == 0 {
        if idx == 0 {
            return (0, 0);
        }
        idx -= 1;
        inv_value = !buf[idx];
    }

    let high_bit = (BITS - 1) - inv_value.leading_zeros() as u64;
    let end_index = (idx as u64) * BITS + high_bit;

    let sh = ((BITS - 1) - high_bit) as u32;
    let mut value = !inv_value << sh >> sh;

    while value == 0 && idx > 0 {
        idx -= 1;
        value = buf[idx];
    }

    let start_index = if value != 0 {
        let hb = (BITS - 1) - value.leading_zeros() as u64;
        (idx as u64) * BITS + hb + 1
    } else {
        0
    };

    (start_index, end_index - start_index + 1)
}

pub fn find_clear_runs(
    buf: &[u64],
    size: u64,
    runs: &mut [(u64, u64)],
    locate_longest: bool,
) -> u64 {
    let max_runs = runs.len();
    let mut from: u64 = 0;
    let mut count: usize = 0;
    let mut smallest: usize = 0;

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
        return count as u64;
    }

    loop {
        let (start, length) = find_next_forward_run_clear(buf, size, from);
        if length == 0 {
            break;
        }
        if length > runs[smallest].1 {
            runs[smallest] = (start, length);
            for i in 0..max_runs {
                if runs[i].1 < runs[smallest].1 {
                    smallest = i;
                }
            }
        }
        from += length;
    }

    count as u64
}

pub fn find_longest_run_clear(buf: &[u64], size: u64) -> (u64, u64) {
    let mut from: u64 = 0;
    let mut best = (0u64, 0u64);
    loop {
        let (start, length) = find_next_forward_run_clear(buf, size, from);
        if length == 0 {
            break;
        }
        if length > best.1 {
            best = (start, length);
        }
        from = start + length;
    }
    best
}

pub fn find_longest_run_set(buf: &[u64], size: u64) -> (u64, u64) {
    let mut from: u64 = 0;
    let mut best = (0u64, 0u64);
    loop {
        let (start, length) = find_next_forward_run_set(buf, size, from);
        if length == 0 {
            break;
        }
        if length > best.1 {
            best = (start, length);
        }
        from = start + length;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_test_clear_roundtrip() {
        let mut buf = vec![0u64; 8];
        let size = 8 * BITS;
        set_bit(&mut buf, 0);
        set_bit(&mut buf, 63);
        set_bit(&mut buf, 64);
        set_bit(&mut buf, size - 1);
        assert!(test_bit(&buf, 0));
        assert!(test_bit(&buf, 63));
        assert!(test_bit(&buf, 64));
        assert!(test_bit(&buf, size - 1));
        assert!(!test_bit(&buf, 1));
        clear_bit(&mut buf, 63);
        assert!(!test_bit(&buf, 63));
    }

    #[test]
    fn range_ops_cross_word() {
        let mut buf = vec![0u64; 8];
        let size = 8 * BITS;
        set_bits(&mut buf, 62, 5); // spans the 64-bit word boundary
        assert!(!test_bit(&buf, 61));
        for b in 62..67 {
            assert!(test_bit(&buf, b));
        }
        assert!(!test_bit(&buf, 67));
        assert_eq!(number_of_set_bits(&buf, size), 5);
        clear_bits(&mut buf, 62, 5);
        assert_eq!(number_of_set_bits(&buf, size), 0);
    }

    #[test]
    fn find_and_runs() {
        let size = 4 * BITS;
        let mut buf = vec![0u64; 4];
        set_all(&mut buf, size);
        clear_bits(&mut buf, 2, 5);
        clear_bits(&mut buf, BITS + 2, 20);
        clear_bits(&mut buf, 3 * BITS, 3);
        let (start, len) = find_longest_run_clear(&buf, size);
        assert_eq!((start, len), (BITS + 2, 20));
        // last (rightmost) backward run from the end is the 3*BITS run.
        let (bstart, blen) = find_last_backward_run_clear(&buf, size, size - 1);
        assert_eq!((bstart, blen), (3 * BITS, 3));
        let pos = find_clear_bits(&buf, size, 5, 0);
        assert_eq!(pos, 2);
        assert_eq!(find_clear_bits(&buf, size, 1, 0), 2);
        set_all(&mut buf, size);
        assert_eq!(find_clear_bits(&buf, size, 1, 0), MAXINDEX);
    }

    // Differential vs the proven generic ops.rs at W=u64 (u32 index;
    // small maps fit) — pins the 64-bit-index retype.
    #[test]
    fn diff_vs_generic_ops() {
        use crate::ops;
        let sz: u32 = 4 * 64;
        let mut a = vec![0u64; 4];
        ops::set_all(&mut a, sz);
        ops::clear_bits(&mut a, 2, 5);
        ops::clear_bits(&mut a, 66, 20);
        ops::clear_bits(&mut a, 192, 3);
        let mut b = vec![0u64; 4];
        set_all(&mut b, 256);
        clear_bits(&mut b, 2, 5);
        clear_bits(&mut b, 66, 20);
        clear_bits(&mut b, 192, 3);
        assert_eq!(a, b, "buffers diverge after set/clear");
        assert_eq!(
            ops::number_of_set_bits(&a, sz) as u64,
            number_of_set_bits(&b, 256),
            "popcount"
        );
        let (gs, gl) = ops::find_longest_run_clear(&a, sz);
        let (s, l) = find_longest_run_clear(&b, 256);
        assert_eq!((gs as u64, gl as u64), (s, l), "longest run");
    }

    #[test]
    fn find_clear_bits_and_set_marks() {
        let size = 2 * BITS;
        let mut buf = vec![0u64; 2];
        let pos = find_clear_bits_and_set(&mut buf, size, 100, 0);
        assert_ne!(pos, MAXINDEX);
        assert_eq!(number_of_set_bits(&buf, size), 100);
    }
}

//! `mem*` core. Non-aliasing slice ops — the *overlap* direction logic
//! for the `memcpy`/`memmove` C symbols lives in the cabi shim (that's
//! the only place raw possibly-aliasing pointers exist). ReactOS
//! `memcpy` is itself overlap-safe (it's a memmove), which the shim +
//! differential test pin.

/// `memset` core: fill `dst` with `val` (low byte). Total.
#[inline]
pub fn set(dst: &mut [u8], val: u8) {
    for b in dst.iter_mut() {
        *b = val;
    }
}

/// `memcmp` core: unsigned lexicographic compare of equal-length
/// regions. Returns the C `memcmp` value (signed difference of the
/// first differing byte, 0 if equal). Caller passes equal-length
/// slices (the shim slices both to `n`).
#[inline]
pub fn cmp(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return *x as i32 - *y as i32;
        }
    }
    0
}

/// `memchr` core: first index of `val` in `hay`, or `None`.
#[inline]
pub fn chr(hay: &[u8], val: u8) -> Option<usize> {
    hay.iter().position(|&b| b == val)
}

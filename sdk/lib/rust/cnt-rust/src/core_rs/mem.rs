//! `mem*` core. Non-aliasing slice ops — the *overlap* direction logic
//! for the `memcpy`/`memmove` C symbols lives in the cabi shim (that's
//! the only place raw possibly-aliasing pointers exist). ReactOS
//! `memcpy` is itself overlap-safe (it's a memmove), which the shim +
//! differential test pin.

/// `memset` core: fill `dst` with `val` truncated to a byte (the C
/// `int val` → `unsigned char` store). Takes the raw C int so the
/// truncation is the core's faithful responsibility, not the shim's.
#[inline]
pub fn set(dst: &mut [u8], val: i32) {
    let b = val as u8;
    for x in dst.iter_mut() {
        *x = b;
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

/// `memchr` core, faithful to ReactOS `memchr.c`: it compares
/// `*p == c` where `*p` is a **signed `char`** promoted to `int` and
/// `c` is the raw `int`. So a byte matches iff its signed-char value
/// equals `c` — meaning `c` outside `-128..=127` NEVER matches (e.g.
/// `memchr(buf, 0x90, n)` finds nothing even if byte 0x90 is present;
/// you must pass `c = -112`). Textbook libc casts to `unsigned char`;
/// this deliberate deviation is pinned by the diff harness.
#[inline]
pub fn chr(hay: &[u8], c: i32) -> Option<usize> {
    hay.iter().position(|&b| (b as i8 as i32) == c)
}

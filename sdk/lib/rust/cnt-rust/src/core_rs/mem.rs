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

/// `_memicmp` core, faithful to ReactOS `memicmp.c`: ASCII
/// case-insensitive compare of two equal-length regions. The C folds
/// each byte through `toupper` (libcntpr `ctype.c` toupper — ASCII
/// `a`..`z` → `A`..`Z` only, every other byte incl. `>=0x80`
/// untouched) and, at the first folded-difference, returns the **raw
/// unfolded** `unsigned char` difference `*p1 - *p2` (NOT the folded
/// one). Equal → 0. Caller passes equal-length slices (the shim slices
/// both to `n`); the ReactOS Vista `NULL`/`n!=0` `_NLSCMPERROR` guard
/// is dead under the NT 5.x kernel (OSMajorVersion < 6), so NULL is
/// never validly passed and is not modeled here.
#[inline]
pub fn icmp(a: &[u8], b: &[u8]) -> i32 {
    #[inline]
    fn up(c: u8) -> u8 {
        if c.is_ascii_lowercase() {
            c - 32
        } else {
            c
        }
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if up(*x) != up(*y) {
            return *x as i32 - *y as i32;
        }
    }
    0
}

/// `_memccpy` core, faithful to ReactOS `memccpy.c`: copy bytes from
/// `src` into `dst` (both length `count`) one at a time; stop after
/// copying a byte equal to `c` → report its index + 1, or after
/// copying a NUL (`'\0'`) → report nothing, or when `count` is
/// exhausted → nothing. Returns `Some(i + 1)` (the shim forms
/// `dst.add(i + 1)`) or `None` → NULL.
///
/// Two preserved misfeatures: (1) the C tests `'\0'` BEFORE `c`, so
/// `_memccpy(.., c = 0, ..)` always returns NULL (the NUL break wins);
/// (2) the match `t == c` compares a **signed `char` `t`** (sign-
/// extended) against the raw `int c` — exactly the `memchr.c` deal —
/// so a byte `0x90` matches only `c == -112` (or `0xFFFFFF90`), never
/// `c == 0x90`. The byte IS copied before either test.
#[inline]
pub fn ccpy(dst: &mut [u8], src: &[u8], c: i32) -> Option<usize> {
    for i in 0..dst.len() {
        let t = src[i];
        dst[i] = t;
        if t == 0 {
            return None;
        }
        if (t as i8 as i32) == c {
            return Some(i + 1);
        }
    }
    None
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

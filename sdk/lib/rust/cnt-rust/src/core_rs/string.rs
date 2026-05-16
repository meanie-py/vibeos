//! `str*` core (u8 today; generalizes to `u16` for the `wcs*` family
//! via the same logic — deferred until the wstring batch).
//!
//! Inputs are the *content* slices (the cabi shim resolves NUL
//! termination and NULL pointers — those are ABI concerns, not
//! algorithm concerns).

/// `strlen` core: length of the content slice (the shim resolves
/// NUL-termination and the `NULL → empty` guard). Trivial, but kept as
/// a core fn so the "core is the only impl" rule holds uniformly and
/// it's Kani-targetable.
#[inline]
pub fn len_of(s: &[u8]) -> usize {
    s.len()
}

/// `strcmp` core, faithful to ReactOS `tcscmp.h`: `_TCHAR` is **signed
/// `char`** on x86, so the result is the *signed*-char difference of
/// the first differing position (textbook/POSIX `strcmp` is unsigned —
/// this is a deliberate ReactOS deviation, pinned by the diff harness).
/// Slices are NUL-excluded contents; the implicit terminator (0) at
/// each end participates exactly as in the C `while(*s1==*s2)` loop.
#[inline]
pub fn cmp(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().max(b.len());
    for i in 0..=n {
        let ca = if i < a.len() { a[i] } else { 0 };
        let cb = if i < b.len() { b[i] } else { 0 };
        if ca != cb {
            return (ca as i8 as i32) - (cb as i8 as i32);
        }
        if ca == 0 {
            return 0; // both reached the terminator equal
        }
    }
    0
}

/// `strstr` core: first index of `needle` in `hay`, or `None`. Empty
/// needle → `Some(0)` (C `strstr` returns `hay`).
#[inline]
pub fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > hay.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// `strncmp` core, faithful to ReactOS `tcsncmp.h`: signed-char
/// (`_TCHAR`=signed `char`), bounded by `n`, stops at the first NUL,
/// `n==0 → 0`. Slices are NUL-excluded contents. Returns the signed
/// difference at the first differing position within `n`, else 0.
#[inline]
pub fn ncmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    for i in 0..n {
        let ca = if i < a.len() { a[i] } else { 0 };
        let cb = if i < b.len() { b[i] } else { 0 };
        if ca != cb {
            return (ca as i8 as i32) - (cb as i8 as i32);
        }
        if ca == 0 {
            return 0;
        }
    }
    0
}

/// `strchr` core, faithful to ReactOS `tcschr.h`: `cc = (char)c`,
/// scans content; **the terminating NUL is a valid target** — if
/// `(c as u8) == 0` it returns `Some(len)` (pointer to the NUL), not
/// `None`. Otherwise first index whose byte equals `c as u8`, else
/// `None`. `s` is the NUL-excluded content slice.
#[inline]
pub fn find_char(s: &[u8], c: i32) -> Option<usize> {
    let target = c as u8;
    if let Some(i) = s.iter().position(|&b| b == target) {
        return Some(i);
    }
    if target == 0 {
        Some(s.len()) // the implicit terminator
    } else {
        None
    }
}

/// `strrchr` core, faithful to ReactOS `tcsrchr.h`: last occurrence;
/// `(c as u8)==0 → Some(len)` (the terminator), else last matching
/// index or `None`.
#[inline]
pub fn find_last_char(s: &[u8], c: i32) -> Option<usize> {
    let target = c as u8;
    if target == 0 {
        return Some(s.len());
    }
    s.iter().rposition(|&b| b == target)
}

/// `_stricmp` core, faithful to ReactOS `_stricmp_nt.c` (libcntpr NT
/// variant): per position load both bytes; if they differ, ASCII-fold
/// **only** `A..=Z`→`a..=z` (locale-independent — high-bit bytes are
/// never folded), and if still differing return the **signed-char**
/// difference (`char` is signed on x86 — `chr1 - chr2` promoted). The
/// terminating NUL participates: equal NULs → 0; a NUL vs non-NUL
/// returns the signed diff (shorter string orders first). Slices are
/// NUL-excluded contents.
#[inline]
pub fn cmp_ci(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().max(b.len());
    for i in 0..=n {
        let r1 = if i < a.len() { a[i] } else { 0 };
        let r2 = if i < b.len() { b[i] } else { 0 };
        if r1 != r2 {
            let mut c1 = r1 as i8 as i32;
            let mut c2 = r2 as i8 as i32;
            if r1.is_ascii_uppercase() {
                c1 += 32;
            }
            if r2.is_ascii_uppercase() {
                c2 += 32;
            }
            if c1 != c2 {
                return c1 - c2;
            }
        } else if r1 == 0 {
            return 0;
        }
    }
    0
}

/// `_strnicmp` core, faithful to ReactOS `_strnicmp_nt.c`: as
/// [`cmp_ci`] but bounded — `while (remaining-- != 0)` so `n == 0 → 0`,
/// compares at most `n` positions, and the equal-NUL break still ends
/// early. Signed-char fold + diff.
#[inline]
pub fn ncmp_ci(a: &[u8], b: &[u8], n: usize) -> i32 {
    for i in 0..n {
        let r1 = if i < a.len() { a[i] } else { 0 };
        let r2 = if i < b.len() { b[i] } else { 0 };
        if r1 != r2 {
            let mut c1 = r1 as i8 as i32;
            let mut c2 = r2 as i8 as i32;
            if r1.is_ascii_uppercase() {
                c1 += 32;
            }
            if r2.is_ascii_uppercase() {
                c2 += 32;
            }
            if c1 != c2 {
                return c1 - c2;
            }
        } else if r1 == 0 {
            return 0;
        }
    }
    0
}

/// `strncpy` core. `dst` is the fixed n-length destination; `src` is
/// the source contents (NUL-excluded). Faithful to ReactOS `tcsncpy`:
/// copy `min(src.len(), n)` bytes; if `src` ended first, pad the rest
/// of `dst` with 0; if `dst` filled first, **do not NUL-terminate**.
///
/// Returns `truncated`: `true` when `dst` filled before `src`'s NUL
/// (the load-bearing misfeature — no terminator). The C `strncpy`
/// shim discards this; `_rs` callers get it as the honest signal.
#[inline]
pub fn ncpy(dst: &mut [u8], src: &[u8]) -> bool {
    let n = dst.len();
    let copy = src.len().min(n);
    dst[..copy].copy_from_slice(&src[..copy]);
    if copy < n {
        // src's NUL reached first → pad remainder with 0.
        for b in dst[copy..].iter_mut() {
            *b = 0;
        }
        false
    } else {
        // dst filled first (copy == n) → truncated, NOT terminated.
        true
    }
}

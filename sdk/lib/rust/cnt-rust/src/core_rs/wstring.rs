//! `wcs*` core (`u16`). Mirrors `string` logic but `wchar_t` on
//! Windows is **unsigned 16-bit**, so the comparison difference
//! promotes value-preservingly to `int` — `(u16 as i32) - (u16 as
//! i32)` (NOT signed-16, unlike the signed-`char` narrow family).
//!
//! Inputs are the NUL-excluded content slices (the cabi shim resolves
//! NUL termination + the NULL guard — ABI concerns, not algorithm).

/// `_wcsicmp` core, faithful to ReactOS `_wcsicmp_nt.c` (libcntpr NT
/// variant): per position, if the two wide chars differ, ASCII-fold
/// **only** `A..=Z`→`a..=z` (no Unicode/locale case folding — values
/// ≥ 0x80 are never folded), then return the **raw** difference (NOT
/// normalized to {-1,0,1}). The terminating NUL participates: equal
/// NULs → 0; NUL vs non-NUL → the raw signed diff.
#[inline]
pub fn cmp_ci(a: &[u16], b: &[u16]) -> i32 {
    let n = a.len().max(b.len());
    for i in 0..=n {
        let r1 = if i < a.len() { a[i] } else { 0 };
        let r2 = if i < b.len() { b[i] } else { 0 };
        if r1 != r2 {
            let mut c1 = r1 as i32;
            let mut c2 = r2 as i32;
            if (b'A' as u16..=b'Z' as u16).contains(&r1) {
                c1 += 32;
            }
            if (b'A' as u16..=b'Z' as u16).contains(&r2) {
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

/// `_wcsnicmp` core, faithful to ReactOS `_wcsnicmp_nt.c`: as
/// [`cmp_ci`] but bounded — `while (remaining-- != 0)` so `n == 0 → 0`,
/// at most `n` positions, equal-NUL break ends early. Raw u16 diff.
#[inline]
pub fn ncmp_ci(a: &[u16], b: &[u16], n: usize) -> i32 {
    for i in 0..n {
        let r1 = if i < a.len() { a[i] } else { 0 };
        let r2 = if i < b.len() { b[i] } else { 0 };
        if r1 != r2 {
            let mut c1 = r1 as i32;
            let mut c2 = r2 as i32;
            if (b'A' as u16..=b'Z' as u16).contains(&r1) {
                c1 += 32;
            }
            if (b'A' as u16..=b'Z' as u16).contains(&r2) {
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

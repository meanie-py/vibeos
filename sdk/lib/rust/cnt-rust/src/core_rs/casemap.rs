//! Wave H — ASCII case fold (libcntpr `_strlwr`/`_strupr`/`_wcslwr`/
//! `_wcsupr`, `tolower`). All four in-place folders and `tolower` are
//! **ASCII-only** in the genuine ReactOS source: the C literally tests
//! `'A'..'Z'` / `'a'..'z'` and adds ±32 — Latin-1 and the wide range
//! are deliberately untouched (the wide `_wcslwr`/`_wcsupr` fold ONLY
//! ASCII, not e.g. `À`). Pure, no `unsafe`.
//!
//! The NLS-aware `towupper` (`RtlUpcaseUnicodeChar`) is NOT here — it
//! reuses [`crate::core_rs::mbcp::upcase`] (the CP1252/Rtl model from
//! Wave E2), exactly as the genuine `towupper_nt.c` delegates to Rtl.

/// Code unit for the in-place folders: `u8` (narrow) / `u16` (wide).
pub trait CaseUnit: Copy {
    fn v(self) -> u32;
    fn from_u32(x: u32) -> Self;
}
impl CaseUnit for u8 {
    #[inline]
    fn v(self) -> u32 {
        self as u32
    }
    #[inline]
    fn from_u32(x: u32) -> u8 {
        x as u8
    }
}
impl CaseUnit for u16 {
    #[inline]
    fn v(self) -> u32 {
        self as u32
    }
    #[inline]
    fn from_u32(x: u32) -> u16 {
        x as u16
    }
}

/// In-place ASCII lower: `A`..=`Z` (0x41..=0x5A) `+= 32`. Faithful to
/// `_strlwr_nt.c`/`_wcslwr_nt.c` — ASCII range ONLY (bytes/units
/// outside are left as-is; the wide variant does NOT fold Latin-1).
#[inline]
pub fn lower_ascii<E: CaseUnit>(buf: &mut [E]) {
    for e in buf.iter_mut() {
        let c = e.v();
        if (b'A' as u32..=b'Z' as u32).contains(&c) {
            *e = E::from_u32(c + 32);
        }
    }
}

/// In-place ASCII upper: `a`..=`z` (0x61..=0x7A) `-= 32`. Faithful to
/// `_strupr_nt.c`/`_wcsupr_nt.c` — ASCII range ONLY.
#[inline]
pub fn upper_ascii<E: CaseUnit>(buf: &mut [E]) {
    for e in buf.iter_mut() {
        let c = e.v();
        if (b'a' as u32..=b'z' as u32).contains(&c) {
            *e = E::from_u32(c - 32);
        }
    }
}

/// `int tolower(int _C)` (`tolower_nt.c`). The fold *decision* uses the
/// **signed-`char`** low byte (`(char)_C`), but the returned value is
/// the FULL `int` `+ 32` — so `tolower(0x141) == 0x161`, and any `_C`
/// whose low byte is `>= 0x80` (negative as signed char) is returned
/// unchanged regardless of its high bits.
#[inline]
pub fn tolower_ascii(c: i32) -> i32 {
    let lc = c as i8 as i32;
    if (b'A' as i32..=b'Z' as i32).contains(&lc) {
        c + 32
    } else {
        c
    }
}

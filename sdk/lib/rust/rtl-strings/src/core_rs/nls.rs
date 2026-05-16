//! Wave RTL-2 — the case-aware (NLS) string ops. The genuine
//! `RtlpUpcaseUnicodeChar` (sdk/lib/rtl/nls.c) walks a 3-level
//! `NlsUnicodeUpcaseTable` loaded from an NLS file at runtime; its
//! ASCII fast path (`Source<'a' ⇒ id`, `'a'..'z' ⇒ -0x20`) is exact C
//! but everything ≥0x7B is table-driven. Per the Wave-E2 discipline
//! (`[[project-rtl-strings-track]]`, `[[cnt-rust-prep]]`): we model
//! the table with the **shared** cnt-rust CP1252 `upcase` — ONE TRUTH,
//! used identically by the Rust core AND the C reference stub in
//! `rtl-difftest/build.rs` — so the diff pins the `unicode.c` wrapper
//! logic (the `min()`, `/sizeof(WCHAR)`, signed-`char` diff, the
//! `(int)L1-(int)L2` tiebreak, the NULL guards, the hash's
//! ASCII-only-vs-NLS split, the `MaximumLength` clamp), NOT the NLS
//! table contents. Pure, no `unsafe`.

/// `RtlpUpcaseUnicodeChar` / `RtlUpcaseUnicodeChar` — the shared
/// CP1252 model. mbcp::upcase already coincides with the genuine ASCII
/// fast path (`<0x61 ⇒ id`, `0x61..=0x7A ⇒ -0x20`) so a straight
/// delegate is faithful for every code unit.
#[inline]
pub fn upcase_uc(w: u16) -> u16 {
    cnt_rust::core_rs::mbcp::upcase(w)
}

/// `RtlUpperChar(IN CHAR Source)` (nls.c). `Source` is **signed**
/// `char`: `Source<='z'` is true for every high byte (0x80–0xFF, i.e.
/// negative) which then fails `Source>='a'` and is returned unchanged;
/// 0x7B–0x7F take the NLS branch but CP1252 maps them to themselves.
/// Net: a–z ⇒ A–Z, every other byte unchanged. Returns the (signed)
/// `CHAR` byte.
#[inline]
pub fn upper_ansi(c: u8) -> u8 {
    if c.is_ascii_lowercase() {
        c - 0x20
    } else {
        c
    }
}

/// `RtlCompareString` (nls.c-driven). `len = min(L1,L2)`; per-byte
/// **signed-`char`** diff of (optionally upcased) bytes; if all equal,
/// `ret = (int)L1 - (int)L2` (the USHORT byte lengths). `ci` selects
/// `RtlUpperChar`.
#[inline]
pub fn compare_ansi(a: &[u8], b: &[u8], ci: bool) -> i32 {
    let key = |x: u8| -> i32 {
        (if ci { upper_ansi(x) } else { x }) as i8 as i32
    };
    let len = a.len().min(b.len());
    let mut ret = 0i32;
    let mut i = 0;
    while ret == 0 && i < len {
        ret = key(a[i]) - key(b[i]);
        i += 1;
    }
    if ret == 0 {
        ret = a.len() as i32 - b.len() as i32;
    }
    ret
}

/// `RtlPrefixString` body (buffers known non-null, `L2 >= L1` already
/// checked by the caller). First `L1` bytes equal under the chosen
/// fold.
#[inline]
pub fn prefix_ansi(s1: &[u8], s2: &[u8], ci: bool) -> bool {
    let f = |x: u8| if ci { upper_ansi(x) } else { x };
    s1.iter().zip(s2.iter()).all(|(&x, &y)| f(x) == f(y))
}

/// `RtlCompareUnicodeString`. `a`/`b` are the `Length/sizeof(WCHAR)`
/// unit views; `l1`/`l2` are the raw USHORT **byte** lengths (the
/// tiebreak is on bytes, and odd lengths are preserved — the genuine
/// `min(L1,L2)/2 == min(L1/2,L2/2)`). WCHAR promotes to a non-negative
/// `int`; `ci` upcases via the shared model.
#[inline]
pub fn compare_uni(a: &[u16], b: &[u16], l1: i32, l2: i32, ci: bool) -> i32 {
    let key = |x: u16| -> i32 {
        (if ci { upcase_uc(x) } else { x }) as i32
    };
    let len = a.len().min(b.len());
    let mut ret = 0i32;
    let mut i = 0;
    while ret == 0 && i < len {
        ret = key(a[i]) - key(b[i]);
        i += 1;
    }
    if ret == 0 {
        ret = l1 - l2;
    }
    ret
}

/// `RtlPrefixUnicodeString` body (buffers non-null, `L2 >= L1`
/// pre-checked). `NumChars = L1 / sizeof(WCHAR)` units equal under the
/// fold.
#[inline]
pub fn prefix_uni(s1: &[u16], s2: &[u16], ci: bool) -> bool {
    let f = |x: u16| if ci { upcase_uc(x) } else { x };
    s1.iter().zip(s2.iter()).all(|(&x, &y)| f(x) == f(y))
}

/// `RtlHashUnicodeString` X65599. The case-insensitive fold here is
/// **ASCII-only** in the genuine source (`'a'..'z'` ⇒ `-'a'+'A'`),
/// NOT the NLS upcase — a deliberate split this pins. Wrapping u32.
#[inline]
pub fn hash_uni(units: &[u16], ci: bool) -> u32 {
    let mut h: u32 = 0;
    for &c in units {
        let v = if ci && (0x61..=0x7A).contains(&c) {
            (c - 0x61 + 0x41) as u32
        } else {
            c as u32
        };
        h = h.wrapping_mul(65599).wrapping_add(v);
    }
    h
}

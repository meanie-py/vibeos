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
///
/// That model is the HOST harness's only: a kernel/ntdll or boot-loader build calls the
/// real `RtlpUpcaseUnicodeChar` (cabi::nlsapi), as unicode.c does, so the compare/prefix/
/// hash/upcase helpers below fold with the loaded tables (or nlsboot's ASCII rule).
#[cfg(not(any(feature = "kernel", feature = "bldr")))]
#[inline]
pub fn upcase_uc(w: u16) -> u16 {
    cnt_rust::core_rs::mbcp::upcase(w)
}
#[cfg(any(feature = "kernel", feature = "bldr"))]
#[inline]
pub fn upcase_uc(w: u16) -> u16 {
    crate::cabi::nlsapi::upcase(w)
}

/// `RtlpDowncaseUnicodeChar` / `RtlDowncaseUnicodeChar` — the shared
/// symmetric model (see [`cnt_rust::core_rs::mbcp::downcase`]).
/// The real `RtlpDowncaseUnicodeChar` outside the host harness, as for [`upcase_uc`].
#[cfg(not(any(feature = "kernel", feature = "bldr")))]
#[inline]
pub fn downcase_uc(w: u16) -> u16 {
    cnt_rust::core_rs::mbcp::downcase(w)
}
#[cfg(any(feature = "kernel", feature = "bldr"))]
#[inline]
pub fn downcase_uc(w: u16) -> u16 {
    crate::cabi::nlsapi::downcase(w)
}

/// `RtlUpperChar(IN CHAR Source)` (nls.c). `Source` is **signed**
/// `char`: `Source<='z'` is true for every high byte (0x80–0xFF, i.e.
/// negative) which then fails `Source>='a'` and is returned unchanged;
/// 0x7B–0x7F take the NLS branch but CP1252 maps them to themselves.
/// Net: a–z ⇒ A–Z, every other byte unchanged. Returns the (signed)
/// `CHAR` byte.
/// The real `RtlUpperChar` outside the host harness, as for [`upcase_uc`].
#[cfg(any(feature = "kernel", feature = "bldr"))]
#[inline]
pub fn upper_ansi(c: u8) -> u8 {
    crate::cabi::nlsapi::upper(c)
}
#[cfg(not(any(feature = "kernel", feature = "bldr")))]
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

// ── Wave RTL-3: the single-byte (CP1252-model) converter core ──────
//
// nls.c's converters branch on NlsMbCodePageTag / DBCSCodePage; the
// default-install ANSI/OEM code page is the single-byte CP1252 (the
// SAME model cnt-rust uses for the CRT mb/wc path — ONE truth). The
// DBCS branch is unreachable here (documented TODO, mirrors
// cnt-rust/mbcp). The diff pins the structural faithfulness of these
// (`Size = min(...)` formulas, `ResultSize` byte-vs-char semantics,
// and the `RtlUnicodeToOemN` BUFFER_OVERFLOW-vs-SUCCESS asymmetry),
// with the table contents factored out by the shared model.

/// `NlsUnicodeToAnsiTable[w]` / `NlsUnicodeToOemTable[w]` — the
/// reverse map; unrepresentable ⇒ the default char `'?'` (the shared
/// value used on both diff sides, matching cnt-rust `wctomb`).
#[inline]
pub fn enc_byte(w: u16) -> u8 {
    cnt_rust::core_rs::mbcp::enc(w).unwrap_or(b'?')
}

/// `dec` re-export so the cabi layer needn't name the cnt-rust path.
#[inline]
pub fn dec_byte(b: u8) -> u16 {
    cnt_rust::core_rs::mbcp::dec(b)
}

/// `RtlMultiByteToUnicodeN` / `RtlOemToUnicodeN` body (single-byte).
/// `Size = min(MbSize, UnicodeSize/sizeof(WCHAR))`; `*ResultSize`
/// (caller) `= Size * sizeof(WCHAR)`. Returns units written.
#[inline]
pub fn mb_to_uni(dst: &mut [u16], src: &[u8]) -> usize {
    let n = src.len().min(dst.len());
    for i in 0..n {
        dst[i] = dec_byte(src[i]);
    }
    n
}

/// `RtlUnicodeToMultiByteN` body (single-byte). `Size =
/// (UnicodeSize > MbSize*sizeof(WCHAR)) ? MbSize : UnicodeSize/2`
/// ≡ `min(MbSize, UnicodeUnits)`; `*ResultSize = Size` (bytes==chars).
#[inline]
pub fn uni_to_mb(dst: &mut [u8], src: &[u16]) -> usize {
    let n = dst.len().min(src.len());
    for i in 0..n {
        dst[i] = enc_byte(src[i]);
    }
    n
}

/// `RtlUpcaseUnicodeToMultiByteN` / `RtlUpcaseUnicodeToOemN` body —
/// the `min`-precompute shape (NOT the `while`), ALWAYS
/// `STATUS_SUCCESS`. Upcases via the shared model first.
#[inline]
pub fn upcase_uni_to_mb(dst: &mut [u8], src: &[u16]) -> usize {
    let n = dst.len().min(src.len());
    for i in 0..n {
        dst[i] = enc_byte(upcase_uc(src[i]));
    }
    n
}

/// `RtlUnicodeToOemN` body (single-byte) — the `while` shape: writes
/// `min(OemSize, UnicodeUnits)`, and returns `true` (⇒
/// `STATUS_BUFFER_OVERFLOW`) iff UNICODE input was left over (i.e.
/// `UnicodeUnits > OemSize`). This SUCCESS-vs-OVERFLOW split (absent
/// from the otherwise-identical `RtlUnicodeToMultiByteN`) is the
/// pinned asymmetry.
#[inline]
pub fn uni_to_oem(dst: &mut [u8], src: &[u16]) -> (usize, bool) {
    let n = dst.len().min(src.len());
    for i in 0..n {
        dst[i] = enc_byte(src[i]);
    }
    (n, src.len() > n)
}

/// Shared-model default chars (the `NlsOemDefaultChar` /
/// `NlsUnicodeDefaultChar` globals): unmappable ⇒ `'?'` on the byte
/// side, `0` on the wide side — same values the C reference uses.
pub const OEM_DEFAULT: u8 = b'?';
pub const UNI_DEFAULT: u16 = 0;

/// `RtlpDidUnicodeToOemWork` (single-byte). FALSE iff some output byte
/// is the default char while the matching source wchar was NOT the
/// unicode default — i.e. a real unmappable substitution happened.
/// (Note the genuine misfeature: a legitimate input `'?'` also trips
/// this, since it can't tell substitution from a real `'?'`.)
#[inline]
pub fn did_oem_work(uni_src: &[u16], oem: &[u8]) -> bool {
    for i in 0..oem.len() {
        if oem[i] == OEM_DEFAULT && uni_src.get(i).copied() != Some(UNI_DEFAULT) {
            return false;
        }
    }
    true
}

/// `RtlCompareUnicodeStrings(PCWCH s1, SIZE_T l1, PCWCH s2, SIZE_T l2,
/// BOOLEAN ci)` — the *raw-buffer* compare (NOT
/// `RtlCompareUnicodeString`'s counted-struct one). `l1`/`l2` are
/// **unit** counts (`SIZE_T`); the tiebreak is the genuine
/// `String1Length - String2Length` `SIZE_T` subtraction truncated to
/// `LONG` (modelled as host `u64` wrap → `i32`, matching the x86-64
/// reference). The per-unit diff is `WCHAR - WCHAR` (each promoted to
/// a non-negative `int`), early-returning on the first mismatch.
#[inline]
pub fn compare_unicode_strings(a: &[u16], b: &[u16], ci: bool) -> i32 {
    let m = a.len().min(b.len());
    for k in 0..m {
        let (c1, c2) = if ci {
            (upcase_uc(a[k]), upcase_uc(b[k]))
        } else {
            (a[k], b[k])
        };
        let r = c1 as i32 - c2 as i32;
        if r != 0 {
            return r;
        }
    }
    ((a.len() as u64).wrapping_sub(b.len() as u64) as u32) as i32
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

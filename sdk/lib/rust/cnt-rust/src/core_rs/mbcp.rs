//! Multibyte ⇄ wide cores (`mbtowc`/`mbstowcs`/`wctomb`/`wcstombs`/
//! `toupper_nt_mb`). The genuine ReactOS `.c` are thin wrappers around
//! the NT NLS primitives (`RtlMultiByteToUnicodeN` …) whose behavior
//! depends on the *loaded* code-page tables. We model the **default
//! single-byte CP1252** (ReactOS's default-install ANSI code page, and
//! what the CRT narrow path uses): a fixed deterministic table, zero
//! global NLS state — so the diff can't fail at random.
//!
//! Faithful within the single-byte-CP domain: 0x00–0x7F = ASCII,
//! 0xA0–0xFF = Latin-1 = identity, 0x80–0x9F = the fixed 32-entry
//! CP1252 table (the 5 "undefined" slots are best-fit identity, as
//! Windows does). DBCS / lead-byte code pages (932/936/…) are the
//! documented TODO for a future real-NLS library — the wrapper logic
//! (query mode, count semantics, the `mbtowc` space-guard, the
//! no-NUL-terminate behavior) is still pinned by the diff harness,
//! which links the genuine wrapper `.c` against an identical C model.

/// CP1252 0x80..=0x9F → Unicode (BMP, fits u16). Indices are
/// `byte - 0x80`. The 5 Windows-undefined slots (0x81/8D/8F/90/9D)
/// best-fit to their identity code point, matching Windows.
const HIGH: [u16; 32] = [
    0x20AC, 0x0081, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, // 80-87
    0x02C6, 0x2030, 0x0160, 0x2039, 0x0152, 0x008D, 0x017D, 0x008F, // 88-8F
    0x0090, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, // 90-97
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x009D, 0x017E, 0x0178, // 98-9F
];

/// CP1252 byte → Unicode wchar (total: every byte maps).
#[inline]
pub fn dec(b: u8) -> u16 {
    if (0x80..0xA0).contains(&b) {
        HIGH[(b - 0x80) as usize]
    } else {
        b as u16
    }
}

/// Unicode wchar → CP1252 byte, or `None` if unrepresentable (the
/// wrappers then emit the default char `'?'`).
#[inline]
pub fn enc(w: u16) -> Option<u8> {
    if w < 0x80 || (0xA0..=0xFF).contains(&w) {
        return Some(w as u8);
    }
    let mut i = 0;
    while i < 32 {
        if HIGH[i] == w {
            return Some(0x80 + i as u8);
        }
        i += 1;
    }
    None
}

/// Simple Unicode upcase over the CP1252-reachable set (deterministic
/// fixed rules; real-NLS table exactness is the documented TODO — both
/// the core and the harness stub use *these* rules so the diff is
/// internally consistent and pins the wrapper packing logic).
#[inline]
pub fn upcase(w: u16) -> u16 {
    match w {
        0x61..=0x7A => w - 0x20,                 // a-z → A-Z
        0xE0..=0xF6 | 0xF8..=0xFE => w - 0x20,   // à-ö ø-þ → À-Þ (skip ÷0xF7)
        0xFF => 0x0178,                          // ÿ → Ÿ
        0xB5 => 0x039C,                          // µ → Μ (Unicode simple)
        0x0161 => 0x0160,                        // š → Š
        0x0153 => 0x0152,                        // œ → Œ
        0x017E => 0x017D,                        // ž → Ž
        _ => w,
    }
}

/// `mbtowc` result. `ret` is the C return; `wc` is written to `*wchar`
/// **only when `ret != -1`** (the space-guard suppresses the write).
pub struct MbToWc {
    pub wc: u16,
    pub ret: i32,
}

/// `mbtowc` core, faithful to `mbstowcs_nt.c`. `mb` is the up-to-2
/// bytes the C `memcpy(mbarr, mbchar, min(count,2))` would capture
/// (caller passes `&mbchar[..min(count,2)]`; `mbarr` is zero-filled).
/// Single-byte CP ⇒ `RtlAnsiCharToUnicodeChar` consumes 1 byte and
/// returns 1; the space-guard (`wc==' ' && mb[0]!=' '`) is modeled
/// though CP1252 never triggers it (only 0x20 maps to U+0020).
#[inline]
pub fn mbtowc(mb: &[u8]) -> MbToWc {
    let b0 = mb.first().copied().unwrap_or(0);
    let wc = dec(b0);
    if wc == 0x20 && b0 != 0x20 {
        MbToWc { wc, ret: -1 }
    } else {
        MbToWc { wc, ret: 1 }
    }
}

/// `mbstowcs` convert: writes `min(src.len(), count)` decoded wchars
/// into `dst` (NO NUL appended — the faithful misfeature). Returns the
/// wchar count. Caller handles the `dst==NULL` query (= `src.len()`).
#[inline]
pub fn mbstowcs(dst: &mut [u16], src: &[u8], count: usize) -> usize {
    let n = src.len().min(count);
    let mut i = 0;
    while i < n {
        dst[i] = dec(src[i]);
        i += 1;
    }
    n
}

/// `wcstombs` convert: writes `min(src.len(), count)` encoded bytes
/// (`'?'` for unrepresentable) into `dst`, NO NUL. Returns byte count.
#[inline]
pub fn wcstombs(dst: &mut [u8], src: &[u16], count: usize) -> usize {
    let n = src.len().min(count);
    let mut i = 0;
    while i < n {
        dst[i] = enc(src[i]).unwrap_or(b'?');
        i += 1;
    }
    n
}

/// `wctomb` core: single wchar → 1 CP1252 byte (`'?'` if unmappable).
#[inline]
pub fn wctomb(w: u16) -> u8 {
    enc(w).unwrap_or(b'?')
}

/// `toupper_nt_mb` core: low byte of `_C` → decode → upcase → re-encode
/// → packed return. Single-byte CP ⇒ `mbSize==1` always, so the result
/// is the upcased byte (`'?'` if the upcased code point is outside
/// CP1252); the `mbSize==2` DBCS-packing branch is structurally
/// present but unexercised here (DBCS = TODO).
#[inline]
pub fn toupper_nt_mb(c: i32) -> i32 {
    let b = (c & 0xFF) as u8;
    let up = upcase(dec(b));
    enc(up).unwrap_or(b'?') as i32
}

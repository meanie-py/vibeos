//! Wave RTL-4b slice 1 — the pure search / validate / GUID-parse
//! kernels of `sdk/lib/rtl/unicode.c`. No NLS table (the only
//! case-fold is the shared `upcase_uc`), no allocation, no `unsafe`.
//! Faithful misfeatures:
//!
//!  - `RtlpIsCharInUnicodeString` case-folds **both** sides with the
//!    shared `RtlpUpcaseUnicodeChar` model;
//!  - `RtlFindCharInUnicodeString` start-at-end uses a `USHORT`
//!    counter compared as `(SHORT)i >= 0`, so a zero-length search
//!    (`Length-1 == 0xFFFF`, `(SHORT) == -1`) iterates nothing; the
//!    forward hit reports `(i+1)*2`, the reverse hit `i*2`;
//!  - `RtlGUIDFromString` is the verbatim Wine state machine: it
//!    walks fixed format positions and writes the GUID bytes
//!    **in place as it parses**, so an `INVALID_PARAMETER` midway
//!    still leaves the already-parsed bytes mutated (the diff pins
//!    that partial write). It reads the buffer **unbounded** (up to
//!    38 units) regardless of `str->Length`.

use super::nls::upcase_uc;

/// `static RtlpIsCharInUnicodeString` (file-local in C; no exported
/// symbol — exercised through `RtlFindCharInUnicodeString`).
#[inline]
fn is_char_in(ch: u16, set: &[u16], ci: bool) -> bool {
    let c = if ci { upcase_uc(ch) } else { ch };
    set.iter().any(|&o| {
        let o = if ci { upcase_uc(o) } else { o };
        c == o
    })
}

/// `RtlFindCharInUnicodeString` core. `search`/`set` are the
/// `Length / sizeof(WCHAR)` unit views. `want = !complement`. Returns
/// the `*Position` byte offset on a hit.
#[inline]
pub fn find_char(
    search: &[u16],
    set: &[u16],
    start_at_end: bool,
    want: bool,
    ci: bool,
) -> Option<u16> {
    if start_at_end {
        for idx in (0..search.len()).rev() {
            if is_char_in(search[idx], set, ci) == want {
                return Some((idx * 2) as u16);
            }
        }
    } else {
        for idx in 0..search.len() {
            if is_char_in(search[idx], set, ci) == want {
                return Some(((idx + 1) * 2) as u16);
            }
        }
    }
    None
}

/// The `RtlValidateUnicodeString` field predicate (the `String !=
/// NULL` arm): not `(Buffer==NULL && (Length!=0 || Max!=0))`, both
/// lengths `% sizeof(WCHAR) == 0`, and `Length <= Max`.
#[inline]
pub fn unicode_string_valid(buffer_null: bool, length: u16, maxlength: u16) -> bool {
    !(buffer_null && (length != 0 || maxlength != 0))
        && length % 2 == 0
        && maxlength % 2 == 0
        && length <= maxlength
}

// IS_TEXT_UNICODE_* (winnt). UNICODE_MASK=0x0F, REVERSE_MASK=0xF0,
// NOT_UNICODE_MASK=0xF00, NOT_ASCII_MASK=0xF000.
const ITU_ASCII16: u32 = 1;
const ITU_STATISTICS: u32 = 2;
const ITU_CONTROLS: u32 = 4;
const ITU_SIGNATURE: u32 = 8;
const ITU_REVERSE_ASCII16: u32 = 16;
const ITU_REVERSE_STATISTICS: u32 = 32;
const ITU_REVERSE_CONTROLS: u32 = 64;
const ITU_REVERSE_SIGNATURE: u32 = 128;
const ITU_ILLEGAL_CHARS: u32 = 256;
const ITU_ODD_LENGTH: u32 = 512;
const ITU_NULL_BYTES: u32 = 4096;
const ITU_UNICODE_MASK: u32 = 0x000F;
const ITU_REVERSE_MASK: u32 = 0x00F0;
const ITU_NOT_UNICODE_MASK: u32 = 0x0F00;
const ITU_NOT_ASCII_MASK: u32 = 0xF000;

/// Wine `strchrW` semantics: the do-while tests the terminating NUL
/// too, so a search char of 0 "matches" the terminator (non-NULL
/// return). The control-char sets are short literals + an implicit 0.
#[inline]
fn strchrw_match(set: &[u16], ch: u16) -> bool {
    ch == 0 || set.contains(&ch)
}

/// `RtlIsTextUnicode(const void* buf, int len, int* pf)`. `buf` is the
/// full byte view; `pf_in` is `Some(*pf)` iff a non-NULL `pf` was
/// passed. Returns `(result, pf_out)`; the DBCS-leadbyte branch is
/// dead in the shared model (`NlsMbCodePageTag == 0`) so `weight`
/// stays 3. Faithful: the signed-`int` `len` arithmetic, the
/// trailing-zero-byte `len--`, the 256-unit cap, the strchrW NUL
/// match, the `out_flags &= *pf` filter. NOTE: genuine's guard is
/// `len < sizeof(WCHAR)` — an *unsigned* (`size_t`) compare, so a
/// negative `len` wraps past it into an OOB read (genuine UB,
/// excluded from the diff). For every defined (`len >= 0`) input the
/// signed `len < 2` here is identical.
pub fn is_text_unicode(buf: &[u8], len: i32, pf_in: Option<i32>) -> (bool, Option<i32>) {
    const STD_CTRL: &[u16] = &[0x000D, 0x000A, 0x0009, 0x0020, 0x3000];
    const REV_CTRL: &[u16] = &[0x0D00, 0x0A00, 0x0900, 0x2000];

    if len < 2 {
        return (false, pf_in.map(|_| 0));
    }

    let flags: u32 = pf_in.map(|v| v as u32).unwrap_or(0xFFFF_FFFF);
    let mut out_flags: u32 = 0;

    if len & 1 != 0 {
        out_flags |= ITU_ODD_LENGTH;
    }

    let mut len = len;
    if buf[(len - 1) as usize] == 0 {
        len -= 1;
    }
    len /= 2;
    if len > 256 {
        len = 256;
    }
    let len = len as usize;

    let unit = |i: usize| -> u16 {
        u16::from_le_bytes([buf[2 * i], buf[2 * i + 1]])
    };

    let s0 = unit(0);
    if s0 == 0xFEFF {
        out_flags |= ITU_SIGNATURE;
    }
    if s0 == 0xFFFE {
        out_flags |= ITU_REVERSE_SIGNATURE;
    }

    let mut last_lo: u8 = 0;
    let mut last_hi: u8 = 0;
    let mut lo_diff: u32 = 0;
    let mut hi_diff: u32 = 0;
    for i in 0..len {
        let w = unit(i);
        let lo = (w & 0xFF) as u8;
        let hi = (w >> 8) as u8;
        lo_diff = lo_diff
            .wrapping_add(lo.max(last_lo) as u32 - lo.min(last_lo) as u32);
        hi_diff = hi_diff
            .wrapping_add(hi.max(last_hi) as u32 - hi.min(last_hi) as u32);
        last_lo = lo;
        last_hi = hi;
        match w {
            0xFFFE | 0x0000 | 0x0A0D | 0xFFFF => out_flags |= ITU_ILLEGAL_CHARS,
            _ => {}
        }
    }

    let weight: u32 = 3; // DBCS branch dead (NlsMbCodePageTag == 0)

    if lo_diff < 127 && hi_diff == 0 {
        out_flags |= ITU_ASCII16;
    }
    if hi_diff != 0 && lo_diff == 0 {
        out_flags |= ITU_REVERSE_ASCII16;
    }
    if weight.wrapping_mul(lo_diff) < hi_diff {
        out_flags |= ITU_REVERSE_STATISTICS;
    }
    if (flags & ITU_STATISTICS) != 0 && weight.wrapping_mul(hi_diff) < lo_diff {
        out_flags |= ITU_STATISTICS;
    }

    if flags & ITU_NULL_BYTES != 0 {
        for i in 0..len {
            let w = unit(i);
            if (w & 0xFF) == 0 || (w >> 8) == 0 {
                out_flags |= ITU_NULL_BYTES;
                break;
            }
        }
    }
    if flags & ITU_CONTROLS != 0 {
        for i in 0..len {
            if strchrw_match(STD_CTRL, unit(i)) {
                out_flags |= ITU_CONTROLS;
                break;
            }
        }
    }
    if flags & ITU_REVERSE_CONTROLS != 0 {
        for i in 0..len {
            if strchrw_match(REV_CTRL, unit(i)) {
                out_flags |= ITU_REVERSE_CONTROLS;
                break;
            }
        }
    }

    let pf_out = pf_in.map(|v| {
        out_flags &= v as u32;
        out_flags as i32
    });

    if out_flags & (ITU_REVERSE_MASK | ITU_NOT_UNICODE_MASK) != 0 {
        return (false, pf_out);
    }
    if out_flags & ITU_NOT_ASCII_MASK != 0 {
        return (true, pf_out);
    }
    if out_flags & ITU_UNICODE_MASK != 0 {
        return (true, pf_out);
    }
    (false, pf_out)
}

#[inline]
fn hex_nibble(w: u16) -> Option<u8> {
    match w {
        0x30..=0x39 => Some((w - 0x30) as u8),
        0x61..=0x66 => Some((w - 0x61 + 10) as u8),
        0x41..=0x46 => Some((w - 0x41 + 10) as u8),
        _ => None,
    }
}

/// `RtlGUIDFromString` — the verbatim Wine fixed-format state machine
/// (`{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`), little-endian byte
/// placement. Mutates `out` in place as it parses (partial-write on
/// failure is faithful). Returns `false` ⇒ `STATUS_INVALID_PARAMETER`.
/// Reads `s` unbounded up to 38 units (genuine UB if shorter — caller
/// must guarantee length).
pub fn guid_from_string(s: &[u16], out: &mut [u8; 16]) -> bool {
    let mut i: i32 = 0;
    let mut li: usize = 0; // lpszCLSID index
    let mut op: usize = 0; // lpOut base byte offset

    while i <= 37 {
        match i {
            0 => {
                if s[li] != b'{' as u16 {
                    return false;
                }
            }
            9 | 14 | 19 | 24 => {
                if s[li] != b'-' as u16 {
                    return false;
                }
            }
            37 => {
                if s[li] != b'}' as u16 {
                    return false;
                }
            }
            _ => {
                let (ch, ch2) = match (hex_nibble(s[li]), hex_nibble(s[li + 1])) {
                    (Some(a), Some(b)) => (a, b),
                    _ => return false,
                };
                let byte = (ch << 4) | ch2;
                match i {
                    1 => out[op + 3] = byte,
                    3 => out[op + 2] = byte,
                    5 => out[op + 1] = byte,
                    7 => {
                        out[op] = byte;
                        op += 4;
                    }
                    10 | 15 => out[op + 1] = byte,
                    12 | 17 => {
                        out[op] = byte;
                        op += 2;
                    }
                    _ => {
                        out[op] = byte;
                        op += 1;
                    }
                }
                li += 1; // skip 2nd hex char
                i += 1;
            }
        }
        li += 1;
        i += 1;
    }
    true
}

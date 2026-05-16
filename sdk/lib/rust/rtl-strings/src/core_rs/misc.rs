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

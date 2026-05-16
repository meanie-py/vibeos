//! Wave RTL-5 — `sdk/lib/rtl/utf8.c` (`RtlUnicodeToUTF8N` /
//! `RtlUTF8ToUnicodeN`). Clean, standalone, no NLS/alloc, no `unsafe`.
//! Faithful to the genuine streaming logic: the running
//! `*_bytes_max` counter (set to 0 on first non-fit so every later
//! unit also fails), the `STATUS_SOME_NOT_MAPPED` accumulation on
//! every U+FFFD substitution, the lone/!-paired surrogate handling,
//! the overlong/invalid-codepoint → 3×U+FFFD with `len =
//! trail_bytes`, and the `i = utf8_bytes` early-out on a truncated
//! trailing sequence (genuine sets the loop counter past the end so
//! the *current* replacement char is still encoded, then the loop
//! exits).

pub const STATUS_SUCCESS: i32 = 0x0000_0000;
pub const STATUS_SOME_NOT_MAPPED: i32 = 0x0000_0107; // NT_SUCCESS true
pub const STATUS_BUFFER_TOO_SMALL: i32 = 0xC000_0023u32 as i32;

/// `RtlUnicodeToUTF8N` body (after the NULL/param checks). `src` is
/// the `uni_bytes / sizeof(WCHAR)` unit view; `dest` is the
/// `utf8_bytes_max`-sized output (None ⇒ size-query). Returns
/// `(bytes_written, status)`.
pub fn unicode_to_utf8(
    src: &[u16],
    mut dest: Option<&mut [u8]>,
    max: u32,
) -> (u32, i32) {
    let mut written: u32 = 0;
    let mut status = STATUS_SUCCESS;
    let mut max_left = max;
    let mut pos: usize = 0;

    let n = src.len();
    let mut i = 0usize;
    while i < n {
        let mut ch: u32 = src[i] as u32;
        if (0xDC00..=0xDFFF).contains(&ch) {
            ch = 0xFFFD;
            status = STATUS_SOME_NOT_MAPPED;
        } else if (0xD800..=0xDBFF).contains(&ch) {
            if i + 1 < n {
                ch -= 0xD800;
                ch <<= 10;
                let lo = src[i + 1] as u32;
                if (0xDC00..=0xDFFF).contains(&lo) {
                    ch |= lo - 0xDC00;
                    ch += 0x0001_0000;
                    i += 1;
                } else {
                    ch = 0xFFFD;
                    status = STATUS_SOME_NOT_MAPPED;
                }
            } else {
                ch = 0xFFFD;
                status = STATUS_SOME_NOT_MAPPED;
            }
        }

        // encode ch as UTF-8 (ch <= 0x10FFFF < 0x200000 always).
        let mut utf8_ch = [0u8; 4];
        let utf8_ch_len: usize = if ch < 0x80 {
            utf8_ch[0] = (ch & 0x7F) as u8;
            1
        } else if ch < 0x800 {
            utf8_ch[0] = 0xC0 | ((ch >> 6) & 0x1F) as u8;
            utf8_ch[1] = 0x80 | (ch & 0x3F) as u8;
            2
        } else if ch < 0x1_0000 {
            utf8_ch[0] = 0xE0 | ((ch >> 12) & 0x0F) as u8;
            utf8_ch[1] = 0x80 | ((ch >> 6) & 0x3F) as u8;
            utf8_ch[2] = 0x80 | (ch & 0x3F) as u8;
            3
        } else {
            utf8_ch[0] = 0xF0 | ((ch >> 18) & 0x07) as u8;
            utf8_ch[1] = 0x80 | ((ch >> 12) & 0x3F) as u8;
            utf8_ch[2] = 0x80 | ((ch >> 6) & 0x3F) as u8;
            utf8_ch[3] = 0x80 | (ch & 0x3F) as u8;
            4
        };

        match dest {
            None => {
                written += utf8_ch_len as u32;
            }
            Some(ref mut d) => {
                if max_left >= utf8_ch_len as u32 {
                    d[pos..pos + utf8_ch_len]
                        .copy_from_slice(&utf8_ch[..utf8_ch_len]);
                    pos += utf8_ch_len;
                    max_left -= utf8_ch_len as u32;
                    written += utf8_ch_len as u32;
                } else {
                    max_left = 0;
                    status = STATUS_BUFFER_TOO_SMALL;
                }
            }
        }
        i += 1;
    }
    (written, status)
}

/// `RtlUTF8ToUnicodeN` body (after the NULL/param checks). `src` is
/// the `utf8_bytes` byte view; `dest` is the `uni_bytes_max`-sized
/// (in bytes) WCHAR output (None ⇒ size-query). Returns
/// `(units_written, status)` — the caller multiplies units by
/// `sizeof(WCHAR)`.
pub fn utf8_to_unicode(
    src: &[u8],
    mut dest: Option<&mut [u16]>,
    max: u32,
) -> (u32, i32) {
    let mut written: u32 = 0;
    let mut status = STATUS_SUCCESS;
    let mut uni_max = max;
    let mut pos: usize = 0;

    let utf8_bytes = src.len();
    let mut i = 0usize;
    while i < utf8_bytes {
        let mut ch: u32 = src[i] as u32;
        let mut trail: u32 = 0;
        if ch >= 0xF5 {
            ch = 0xFFFD;
            status = STATUS_SOME_NOT_MAPPED;
        } else if ch >= 0xF0 {
            ch &= 0x07;
            trail = 3;
        } else if ch >= 0xE0 {
            ch &= 0x0F;
            trail = 2;
        } else if ch >= 0xC2 {
            ch &= 0x1F;
            trail = 1;
        } else if ch >= 0x80 {
            ch = 0xFFFD;
            status = STATUS_SOME_NOT_MAPPED;
        }

        if i + (trail as usize) < utf8_bytes {
            for _ in 0..trail {
                if src[i + 1] & 0xC0 == 0x80 {
                    ch <<= 6;
                    ch |= (src[i + 1] & 0x3F) as u32;
                    i += 1;
                } else {
                    ch = 0xFFFD;
                    trail = 0;
                    status = STATUS_SOME_NOT_MAPPED;
                    break;
                }
            }
        } else {
            ch = 0xFFFD;
            trail = 0;
            status = STATUS_SOME_NOT_MAPPED;
            i = utf8_bytes; // genuine: forces the outer loop to end
        }

        let mut utf16 = [0u16; 3];
        let utf16_len: usize = if ch > 0x10_FFFF
            || (0xD800..=0xDFFF).contains(&ch)
            || (trail == 2 && ch < 0x0080_0)
            || (trail == 3 && ch < 0x1_0000)
        {
            utf16[0] = 0xFFFD;
            utf16[1] = 0xFFFD;
            utf16[2] = 0xFFFD;
            status = STATUS_SOME_NOT_MAPPED;
            trail as usize
        } else if ch >= 0x1_0000 {
            ch -= 0x0001_0000;
            utf16[0] = (0xD800 + ((ch >> 10) & 0x3FF)) as u16;
            utf16[1] = (0xDC00 + (ch & 0x3FF)) as u16;
            2
        } else {
            utf16[0] = ch as u16;
            1
        };

        match dest {
            None => {
                written += utf16_len as u32;
            }
            Some(ref mut d) => {
                for &u in utf16.iter().take(utf16_len) {
                    if uni_max >= 2 {
                        d[pos] = u;
                        pos += 1;
                        uni_max -= 2;
                        written += 1;
                    } else {
                        uni_max = 0;
                        status = STATUS_BUFFER_TOO_SMALL;
                    }
                }
            }
        }
        i += 1;
    }
    (written, status)
}

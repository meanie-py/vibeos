//! UTF-16 ↔ UTF-8 conversion — pure algorithm, no unsafe.
//!
//! Implements the exact semantics of Windows NT's RtlUnicodeToUTF8N and
//! RtlUTF8ToUnicodeN, including surrogate pair handling, overlong encoding
//! detection, and replacement character (U+FFFD) substitution.

/// NTSTATUS constants (as u32 for no_std compatibility).
pub const STATUS_SUCCESS: u32 = 0x0000_0000;
pub const STATUS_SOME_NOT_MAPPED: u32 = 0x0000_0107;
pub const STATUS_BUFFER_TOO_SMALL: u32 = 0xC000_0023;

/// Replacement character for invalid sequences.
const REPLACEMENT: u32 = 0xFFFD;

/// Result of a conversion operation.
pub struct ConvResult {
    pub status: u32,
    pub bytes_written: usize,
}

/// Convert UTF-16 to UTF-8, writing into `dst`. Returns bytes written and status.
///
/// If `dst` is empty slice of length 0 and you want count-only, use `utf16_to_utf8_count`.
pub fn utf16_to_utf8(dst: &mut [u8], uni_src: &[u16]) -> ConvResult {
    let mut written = 0usize;
    let mut status = STATUS_SUCCESS;
    let mut remaining = dst.len();
    let mut i = 0usize;

    while i < uni_src.len() {
        let (ch, advance) = decode_one_utf16(uni_src, i);
        if ch == REPLACEMENT {
            status = STATUS_SOME_NOT_MAPPED;
        }
        i += advance;

        let (bytes, len) = encode_utf8(ch);
        if remaining >= len {
            dst[written..written + len].copy_from_slice(&bytes[..len]);
            remaining -= len;
            written += len;
        } else {
            remaining = 0;
            status = STATUS_BUFFER_TOO_SMALL;
        }
    }

    ConvResult {
        status,
        bytes_written: written,
    }
}

/// Count how many UTF-8 bytes are needed for the given UTF-16 input.
pub fn utf16_to_utf8_count(uni_src: &[u16]) -> ConvResult {
    let mut written = 0usize;
    let mut status = STATUS_SUCCESS;
    let mut i = 0usize;

    while i < uni_src.len() {
        let (ch, advance) = decode_one_utf16(uni_src, i);
        if ch == REPLACEMENT {
            status = STATUS_SOME_NOT_MAPPED;
        }
        i += advance;

        let (_, len) = encode_utf8(ch);
        written += len;
    }

    ConvResult {
        status,
        bytes_written: written,
    }
}

/// Convert UTF-8 to UTF-16, writing into `dst`. Returns bytes (not units) written.
pub fn utf8_to_utf16(dst: &mut [u16], utf8_src: &[u8]) -> ConvResult {
    let mut written_units = 0usize;
    let mut status = STATUS_SUCCESS;
    let mut remaining_units = dst.len();
    let mut i = 0usize;

    while i < utf8_src.len() {
        let (ch, trail_bytes, advance) = decode_one_utf8(utf8_src, i);
        i += advance;

        if ch == REPLACEMENT {
            status = STATUS_SOME_NOT_MAPPED;
        }

        let (units, unit_count) = encode_utf16(ch, trail_bytes);

        for j in 0..unit_count {
            if remaining_units > 0 {
                dst[written_units] = units[j];
                remaining_units -= 1;
                written_units += 1;
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
    }

    ConvResult {
        status,
        bytes_written: written_units * 2,
    }
}

/// Count how many UTF-16 bytes are needed for the given UTF-8 input.
pub fn utf8_to_utf16_count(utf8_src: &[u8]) -> ConvResult {
    let mut written_units = 0usize;
    let mut status = STATUS_SUCCESS;
    let mut i = 0usize;

    while i < utf8_src.len() {
        let (ch, trail_bytes, advance) = decode_one_utf8(utf8_src, i);
        i += advance;

        if ch == REPLACEMENT {
            status = STATUS_SOME_NOT_MAPPED;
        }

        let (_units, unit_count) = encode_utf16(ch, trail_bytes);
        written_units += unit_count;
    }

    ConvResult {
        status,
        bytes_written: written_units * 2,
    }
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Decode one codepoint from UTF-16 at position `i`. Returns (codepoint, chars_consumed).
fn decode_one_utf16(src: &[u16], i: usize) -> (u32, usize) {
    let ch = src[i] as u32;

    if ch >= 0xDC00 && ch <= 0xDFFF {
        // Unpaired low surrogate
        return (REPLACEMENT, 1);
    }

    if ch >= 0xD800 && ch <= 0xDBFF {
        // High surrogate
        if i + 1 < src.len() {
            let low = src[i + 1] as u32;
            if low >= 0xDC00 && low <= 0xDFFF {
                let cp = ((ch - 0xD800) << 10) | (low - 0xDC00);
                return (cp + 0x10000, 2);
            }
        }
        return (REPLACEMENT, 1);
    }

    (ch, 1)
}

/// Decode one codepoint from UTF-8 at position `i`.
/// Returns (codepoint, trail_bytes_used_for_encode, bytes_consumed).
fn decode_one_utf8(src: &[u8], i: usize) -> (u32, usize, usize) {
    let byte = src[i];
    let mut ch: u32;
    let trail_bytes: usize;

    if byte >= 0xF5 {
        return (REPLACEMENT, 0, 1);
    } else if byte >= 0xF0 {
        ch = (byte & 0x07) as u32;
        trail_bytes = 3;
    } else if byte >= 0xE0 {
        ch = (byte & 0x0F) as u32;
        trail_bytes = 2;
    } else if byte >= 0xC2 {
        ch = (byte & 0x1F) as u32;
        trail_bytes = 1;
    } else if byte >= 0x80 {
        return (REPLACEMENT, 0, 1);
    } else {
        return (byte as u32, 0, 1);
    }

    // Read trail bytes
    if i + 1 + trail_bytes > src.len() {
        // Truncated sequence — skip to end
        return (REPLACEMENT, 0, src.len() - i);
    }

    let mut consumed = 1usize;
    for j in 0..trail_bytes {
        let b = src[i + 1 + j];
        if (b & 0xC0) == 0x80 {
            ch = (ch << 6) | (b & 0x3F) as u32;
            consumed += 1;
        } else {
            // Invalid trail byte — consumed only up to this point
            return (REPLACEMENT, 0, consumed);
        }
    }

    // Check for overlong encodings and invalid codepoints
    if (ch > 0x10FFFF)
        || (ch >= 0xD800 && ch <= 0xDFFF)
        || (trail_bytes == 2 && ch < 0x800)
        || (trail_bytes == 3 && ch < 0x10000)
    {
        return (REPLACEMENT, trail_bytes, consumed);
    }

    (ch, trail_bytes, consumed)
}

/// Encode a Unicode codepoint as UTF-8 bytes. Returns (bytes, length).
fn encode_utf8(ch: u32) -> ([u8; 4], usize) {
    let mut buf = [0u8; 4];
    if ch < 0x80 {
        buf[0] = ch as u8;
        (buf, 1)
    } else if ch < 0x800 {
        buf[0] = 0xC0 | ((ch >> 6) & 0x1F) as u8;
        buf[1] = 0x80 | (ch & 0x3F) as u8;
        (buf, 2)
    } else if ch < 0x10000 {
        buf[0] = 0xE0 | ((ch >> 12) & 0x0F) as u8;
        buf[1] = 0x80 | ((ch >> 6) & 0x3F) as u8;
        buf[2] = 0x80 | (ch & 0x3F) as u8;
        (buf, 3)
    } else {
        buf[0] = 0xF0 | ((ch >> 18) & 0x07) as u8;
        buf[1] = 0x80 | ((ch >> 12) & 0x3F) as u8;
        buf[2] = 0x80 | ((ch >> 6) & 0x3F) as u8;
        buf[3] = 0x80 | (ch & 0x3F) as u8;
        (buf, 4)
    }
}

/// Encode a Unicode codepoint as UTF-16 units. Returns (units, count).
fn encode_utf16(ch: u32, trail_bytes: usize) -> ([u16; 3], usize) {
    let mut buf = [REPLACEMENT as u16; 3];

    if ch == REPLACEMENT {
        // Replacement chars — one per trail byte (matching NT behavior), minimum 1
        let count = if trail_bytes > 0 { trail_bytes } else { 1 };
        return (buf, count);
    }

    if ch >= 0x10000 {
        let adj = ch - 0x10000;
        buf[0] = (0xD800 + ((adj >> 10) & 0x3FF)) as u16;
        buf[1] = (0xDC00 + (adj & 0x3FF)) as u16;
        (buf, 2)
    } else {
        buf[0] = ch as u16;
        (buf, 1)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- UTF-16 to UTF-8 tests ---

    #[test]
    fn test_ascii() {
        let src = [0x48u16, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
        let mut dst = [0u8; 16];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 5);
        assert_eq!(&dst[..5], b"Hello");
    }

    #[test]
    fn test_two_byte_utf8() {
        let src = [0x00E9u16]; // é
        let mut dst = [0u8; 8];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 2);
        assert_eq!(&dst[..2], &[0xC3, 0xA9]);
    }

    #[test]
    fn test_three_byte_utf8() {
        let src = [0x4E16u16]; // 世
        let mut dst = [0u8; 8];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 3);
        assert_eq!(&dst[..3], &[0xE4, 0xB8, 0x96]);
    }

    #[test]
    fn test_surrogate_pair() {
        let src = [0xD83Du16, 0xDE00]; // U+1F600 😀
        let mut dst = [0u8; 8];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 4);
        assert_eq!(&dst[..4], &[0xF0, 0x9F, 0x98, 0x80]);
    }

    #[test]
    fn test_unpaired_low_surrogate() {
        let src = [0xDC00u16];
        let mut dst = [0u8; 8];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SOME_NOT_MAPPED);
        assert_eq!(r.bytes_written, 3);
        assert_eq!(&dst[..3], &[0xEF, 0xBF, 0xBD]); // U+FFFD
    }

    #[test]
    fn test_unpaired_high_surrogate() {
        let src = [0xD800u16, 0x0041]; // High surrogate + 'A'
        let mut dst = [0u8; 16];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SOME_NOT_MAPPED);
        assert_eq!(r.bytes_written, 4); // FFFD(3) + A(1)
        assert_eq!(&dst[..4], &[0xEF, 0xBF, 0xBD, 0x41]);
    }

    #[test]
    fn test_high_surrogate_at_end() {
        let src = [0xD800u16];
        let mut dst = [0u8; 8];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_SOME_NOT_MAPPED);
        assert_eq!(r.bytes_written, 3);
        assert_eq!(&dst[..3], &[0xEF, 0xBF, 0xBD]);
    }

    #[test]
    fn test_count_only() {
        let src = [0x48u16, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
        let r = utf16_to_utf8_count(&src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 5);
    }

    #[test]
    fn test_buffer_too_small() {
        let src = [0x4E16u16]; // 世 needs 3 bytes
        let mut dst = [0u8; 2];
        let r = utf16_to_utf8(&mut dst, &src);
        assert_eq!(r.status, STATUS_BUFFER_TOO_SMALL);
    }

    // --- UTF-8 to UTF-16 tests ---

    #[test]
    fn test_utf8_ascii() {
        let src = b"Hello";
        let mut dst = [0u16; 16];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 10); // 5 units * 2 bytes
        assert_eq!(&dst[..5], &[0x48, 0x65, 0x6C, 0x6C, 0x6F]);
    }

    #[test]
    fn test_utf8_two_byte() {
        let src = &[0xC3u8, 0xA9]; // é
        let mut dst = [0u16; 4];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 2);
        assert_eq!(dst[0], 0x00E9);
    }

    #[test]
    fn test_utf8_three_byte() {
        let src = &[0xE4u8, 0xB8, 0x96]; // 世
        let mut dst = [0u16; 4];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 2);
        assert_eq!(dst[0], 0x4E16);
    }

    #[test]
    fn test_utf8_four_byte() {
        let src = &[0xF0u8, 0x9F, 0x98, 0x80]; // 😀
        let mut dst = [0u16; 4];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 4); // surrogate pair
        assert_eq!(dst[0], 0xD83D);
        assert_eq!(dst[1], 0xDE00);
    }

    #[test]
    fn test_utf8_overlong() {
        let src = &[0xC1u8, 0x81]; // Overlong 'A'
        let mut dst = [0u16; 4];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SOME_NOT_MAPPED);
    }

    #[test]
    fn test_utf8_invalid_lead() {
        let src = &[0xF5u8];
        let mut dst = [0u16; 4];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SOME_NOT_MAPPED);
        assert_eq!(dst[0], 0xFFFD);
    }

    #[test]
    fn test_utf8_truncated() {
        let src = &[0xE4u8, 0xB8]; // Missing third byte
        let mut dst = [0u16; 4];
        let r = utf8_to_utf16(&mut dst, src);
        assert_eq!(r.status, STATUS_SOME_NOT_MAPPED);
    }

    #[test]
    fn test_utf8_count_only() {
        let src = b"Hello";
        let r = utf8_to_utf16_count(src);
        assert_eq!(r.status, STATUS_SUCCESS);
        assert_eq!(r.bytes_written, 10);
    }

    #[test]
    fn test_roundtrip_ascii() {
        let original = [0x48u16, 0x65, 0x6C, 0x6C, 0x6F];
        let mut utf8 = [0u8; 32];
        let r1 = utf16_to_utf8(&mut utf8, &original);
        assert_eq!(r1.status, STATUS_SUCCESS);

        let mut back = [0u16; 16];
        let r2 = utf8_to_utf16(&mut back, &utf8[..r1.bytes_written]);
        assert_eq!(r2.status, STATUS_SUCCESS);
        assert_eq!(&back[..r2.bytes_written / 2], &original);
    }

    #[test]
    fn test_roundtrip_surrogate() {
        let original = [0xD83Du16, 0xDE00]; // 😀
        let mut utf8 = [0u8; 8];
        let r1 = utf16_to_utf8(&mut utf8, &original);
        assert_eq!(r1.status, STATUS_SUCCESS);

        let mut back = [0u16; 4];
        let r2 = utf8_to_utf16(&mut back, &utf8[..r1.bytes_written]);
        assert_eq!(r2.status, STATUS_SUCCESS);
        assert_eq!(&back[..r2.bytes_written / 2], &original);
    }

    #[test]
    fn test_empty_input() {
        let mut dst_utf8 = [0u8; 8];
        let r1 = utf16_to_utf8(&mut dst_utf8, &[]);
        assert_eq!(r1.status, STATUS_SUCCESS);
        assert_eq!(r1.bytes_written, 0);

        let mut dst_utf16 = [0u16; 8];
        let r2 = utf8_to_utf16(&mut dst_utf16, &[]);
        assert_eq!(r2.status, STATUS_SUCCESS);
        assert_eq!(r2.bytes_written, 0);
    }

    #[test]
    fn test_mixed_bmp_and_supplementary() {
        // "A世😀" = U+0041, U+4E16, U+1F600
        let src = [0x0041u16, 0x4E16, 0xD83D, 0xDE00];
        let mut utf8 = [0u8; 16];
        let r1 = utf16_to_utf8(&mut utf8, &src);
        assert_eq!(r1.status, STATUS_SUCCESS);
        // A(1) + 世(3) + 😀(4) = 8
        assert_eq!(r1.bytes_written, 8);

        let mut back = [0u16; 8];
        let r2 = utf8_to_utf16(&mut back, &utf8[..r1.bytes_written]);
        assert_eq!(r2.status, STATUS_SUCCESS);
        assert_eq!(&back[..r2.bytes_written / 2], &src);
    }
}

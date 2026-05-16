//! Wave RTL-6 — `sdk/lib/rtl/dos8dot3.c` pure kernels: the
//! short-name illegal-char bitmap (`RtlpIsShortIllegal`) and the
//! 8.3 checksum (`RtlpGetCheckSum`). No NLS/alloc, no `unsafe`.
//! All hash arithmetic is faithful `USHORT` wrapping (the C does
//! `int` math then truncates to a `USHORT` field).

/// `RtlpShortIllegals` (dos8dot3.c) — 4×ULONG bitmap over 0..127.
const SHORT_ILLEGALS: [u32; 4] = [0xFFFF_FFFF, 0xFC00_9C04, 0x3800_0000, 0x1000_0000];

/// `RtlpIsShortIllegal`: `Char < 128 && (bitmap[Char/32] &
/// (1 << (Char % 32)))`.
#[inline]
pub fn is_short_illegal(ch: u16) -> bool {
    ch < 128 && (SHORT_ILLEGALS[(ch / 32) as usize] & (1u32 << (ch % 32))) != 0
}

/// `RtlpGetCheckSum(PUNICODE_STRING)`. `buf` is the WCHAR-unit view;
/// `length_bytes` is `Name->Length` (USHORT byte count). Mirrors the
/// genuine pointer walk with `USHORT` (wrapping) arithmetic.
pub fn get_checksum(buf: &[u16], length_bytes: u16) -> u16 {
    if length_bytes == 0 {
        return 0;
    }
    if length_bytes == 2 {
        return buf[0];
    }
    // Hash = (*CurrentChar << 8) + *(CurrentChar + 1)  (USHORT)
    let mut hash: u16 =
        (((buf[0] as u32) << 8).wrapping_add(buf[1] as u32)) as u16;
    if length_bytes == 4 {
        return hash;
    }
    let nunits = length_bytes / 2;
    let mut saved: u16 = hash;
    let mut length: u16 = 2;
    let mut cc: usize = 0; // CurrentChar index (WCHAR units)
    loop {
        cc += 2;
        // Hash = (Hash << 7) + *CurrentChar
        hash = (((hash as u32) << 7).wrapping_add(buf[cc] as u32)) as u16;
        // Hash = (Saved >> 1) + (Hash << 8)
        hash = (((saved >> 1) as u32).wrapping_add((hash as u32) << 8)) as u16;
        if length + 1 < nunits {
            // Hash += *(CurrentChar + 1)
            hash = ((hash as u32).wrapping_add(buf[cc + 1] as u32)) as u16;
        }
        saved = hash;
        length += 2;
        if length >= nunits {
            break;
        }
    }
    hash
}

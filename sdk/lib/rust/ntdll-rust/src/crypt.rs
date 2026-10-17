//! ntdll's MD4 / MD5 / SHA-1 exports: `MD4Init/Update/Final`, `MD5Init/Update/Final`,
//! `A_SHAInit/Update/Final`. Windows has exported these from ntdll since Vista (advapi32's
//! copies forward here); our advapi32 still links sdk/lib/cryptlib for its own.
//!
//! The hash cores are Colin Plumb's public-domain MD4/MD5 and Steve Reid's public-domain
//! SHA-1, as in Wine's dlls/ntdll/crypt.c and sdk/lib/cryptlib. Everything a CALLER can
//! observe in the context, though, follows real Windows, measured black-box by
//! the ntdll-gated-oracle harness on Windows 10.0.26300 (GetProcAddress + call + dump the
//! whole context, then a differential fuzz of these bodies against it), where it
//! differs from both Wine and cryptlib:
//!
//!   * `A_SHA_CTX` is `{ Buffer[64]; State[5]; Count[2] }` (cryptlib's order). Wine puts six
//!     `Unknown` ULONGs first; Windows never writes there, it writes Buffer at offset 0.
//!   * Init zeroes the 64-byte input buffer (both MD and SHA). The MD `digest` field is left
//!     alone until Final.
//!   * Update hashes whole blocks straight from the caller's data whenever the context
//!     buffer is empty, so the buffer only ever holds a PARTIAL block's bytes. Wine/cryptlib
//!     copy every block through the buffer first.
//!   * MD Final pads through Update (so the bit count `i` includes the padding, RSA-style),
//!     writes the digest into both `buf` and `digest`, and zeroes `in`.
//!   * SHA Final writes the big-endian digest to Result, then zeroes Buffer and re-inits.
//!
//! Counts are `u32` pairs exactly as the C keeps them; every add wraps as C unsigned does.

#![allow(non_snake_case, non_camel_case_types)]

/// `MD4_CTX` (Windows / Wine / cryptlib md4.h): 104 bytes.
#[repr(C)]
pub struct MD4_CTX {
    pub buf: [u32; 4],
    pub i: [u32; 2],
    pub in_: [u8; 64],
    pub digest: [u8; 16],
}

/// `MD5_CTX` (Windows / Wine / cryptlib md5.h): 104 bytes. Note `i` comes FIRST here.
#[repr(C)]
pub struct MD5_CTX {
    pub i: [u32; 2],
    pub buf: [u32; 4],
    pub in_: [u8; 64],
    pub digest: [u8; 16],
}

/// `A_SHA_CTX`: the 92 bytes Windows reads and writes (see module docs for the order).
#[repr(C)]
pub struct A_SHA_CTX {
    pub Buffer: [u8; 64],
    pub State: [u32; 5],
    pub Count: [u32; 2],
}

const MD_INIT: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
const SHA_INIT: [u32; 5] = [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];

#[inline(always)]
fn le_words(block: &[u8; 64]) -> [u32; 16] {
    let mut w = [0u32; 16];
    for (k, word) in w.iter_mut().enumerate() {
        *word = u32::from_le_bytes([block[4 * k], block[4 * k + 1], block[4 * k + 2], block[4 * k + 3]]);
    }
    w
}

// ---- MD4 core (Wine crypt.c MD4Transform) --------------------------------------------------

fn md4_transform(buf: &mut [u32; 4], block: &[u8; 64]) {
    let x = le_words(block);
    let (mut a, mut b, mut c, mut d) = (buf[0], buf[1], buf[2], buf[3]);
    macro_rules! ff { ($a:ident,$b:ident,$c:ident,$d:ident,$k:expr,$s:expr) => {
        $a = $a.wrapping_add(($b & $c) | (!$b & $d)).wrapping_add(x[$k]).rotate_left($s);
    } }
    macro_rules! gg { ($a:ident,$b:ident,$c:ident,$d:ident,$k:expr,$s:expr) => {
        $a = $a.wrapping_add(($b & $c) | ($b & $d) | ($c & $d)).wrapping_add(x[$k])
            .wrapping_add(0x5a82_7999).rotate_left($s);
    } }
    macro_rules! hh { ($a:ident,$b:ident,$c:ident,$d:ident,$k:expr,$s:expr) => {
        $a = $a.wrapping_add($b ^ $c ^ $d).wrapping_add(x[$k]).wrapping_add(0x6ed9_eba1).rotate_left($s);
    } }
    for k in [0usize, 4, 8, 12] {
        ff!(a, b, c, d, k, 3); ff!(d, a, b, c, k + 1, 7); ff!(c, d, a, b, k + 2, 11); ff!(b, c, d, a, k + 3, 19);
    }
    for k in [0usize, 1, 2, 3] {
        gg!(a, b, c, d, k, 3); gg!(d, a, b, c, k + 4, 5); gg!(c, d, a, b, k + 8, 9); gg!(b, c, d, a, k + 12, 13);
    }
    for k in [0usize, 2, 1, 3] {
        hh!(a, b, c, d, k, 3); hh!(d, a, b, c, k + 8, 9); hh!(c, d, a, b, k + 4, 11); hh!(b, c, d, a, k + 12, 15);
    }
    buf[0] = buf[0].wrapping_add(a);
    buf[1] = buf[1].wrapping_add(b);
    buf[2] = buf[2].wrapping_add(c);
    buf[3] = buf[3].wrapping_add(d);
}

// ---- MD5 core (Wine crypt.c MD5Transform) --------------------------------------------------

const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];
const MD5_S: [u32; 16] = [7, 12, 17, 22, 5, 9, 14, 20, 4, 11, 16, 23, 6, 10, 15, 21];

fn md5_transform(buf: &mut [u32; 4], block: &[u8; 64]) {
    let x = le_words(block);
    let (mut a, mut b, mut c, mut d) = (buf[0], buf[1], buf[2], buf[3]);
    for step in 0..64 {
        let round = step / 16;
        let (f, k) = match round {
            0 => (d ^ (b & (c ^ d)), step),                 // F1
            1 => (c ^ (d & (b ^ c)), (5 * step + 1) % 16),  // F2 = F1(z, x, y)
            2 => (b ^ c ^ d, (3 * step + 5) % 16),          // F3
            _ => (c ^ (b | !d), (7 * step) % 16),           // F4
        };
        let s = MD5_S[round * 4 + step % 4];
        let rotated = a.wrapping_add(f).wrapping_add(x[k]).wrapping_add(MD5_K[step]).rotate_left(s);
        a = d;
        d = c;
        c = b;
        b = b.wrapping_add(rotated);
    }
    buf[0] = buf[0].wrapping_add(a);
    buf[1] = buf[1].wrapping_add(b);
    buf[2] = buf[2].wrapping_add(c);
    buf[3] = buf[3].wrapping_add(d);
}

// ---- shared MD4/MD5 framing -----------------------------------------------------------------

/// MDxUpdate over (bit count `i`, chaining `buf`, block buffer `in_`).
unsafe fn md_update(
    i: &mut [u32; 2],
    buf: &mut [u32; 4],
    in_: &mut [u8; 64],
    data: *const u8,
    len: u32,
    transform: fn(&mut [u32; 4], &[u8; 64]),
) {
    // `i` is a 64-bit BIT count. Windows advances it as a whole-byte count: whatever stray
    // low three bits a context carries are dropped, even by an empty update (measured:
    // i[0] = 7, len 0 -> 0). Wine/cryptlib keep them; (count & ~7) + 8*len, carried into
    // i[1], is the Windows arithmetic exactly.
    let t = i[0];
    let base = t & !7;
    i[0] = base.wrapping_add(len << 3);
    if i[0] < base {
        i[1] = i[1].wrapping_add(1); // carry from low to high
    }
    i[1] = i[1].wrapping_add(len >> 29);

    if len == 0 {
        return; // `data` may be NULL for an empty update; touch nothing.
    }
    let mut src = core::slice::from_raw_parts(data, len as usize);

    let used = ((t >> 3) & 0x3f) as usize;
    if used != 0 {
        let fill = 64 - used;
        if src.len() < fill {
            in_[used..used + src.len()].copy_from_slice(src);
            return;
        }
        in_[used..].copy_from_slice(&src[..fill]);
        transform(buf, in_);
        src = &src[fill..];
    }
    // Whole blocks straight from the caller's data: Windows leaves `in_` untouched here.
    while src.len() >= 64 {
        let (block, rest) = src.split_at(64);
        transform(buf, &*(block.as_ptr() as *const [u8; 64]));
        src = rest;
    }
    in_[..src.len()].copy_from_slice(src);
}

/// MDxFinal: pad through Update (0x80, zeros to 56 mod 64, then the 64-bit LE bit count).
unsafe fn md_final(
    i: &mut [u32; 2],
    buf: &mut [u32; 4],
    in_: &mut [u8; 64],
    digest: &mut [u8; 16],
    transform: fn(&mut [u32; 4], &[u8; 64]),
) {
    // The appended length is the whole-byte count in bits: stray low bits of a context
    // that never went through Init/Update are dropped here too (measured).
    let mut bits = [0u8; 8];
    bits[..4].copy_from_slice(&(i[0] & !7).to_le_bytes());
    bits[4..].copy_from_slice(&i[1].to_le_bytes());

    let used = ((i[0] >> 3) & 0x3f) as u32;
    let pad_len = if used < 56 { 56 - used } else { 120 - used };
    let mut padding = [0u8; 64];
    padding[0] = 0x80;
    // The counter Windows leaves behind is the BYTE count taken as 32 bits, advanced by
    // the padding, then shifted back into bits: of i[1] only its low three bits survive
    // (measured: i = {0x18, 0x6dd0c18e} -> {0x200, 6}; {0xfffffff8, 0xffffffff} ->
    // {0x200, 0}). The digest itself covers the full 64-bit count in `bits`.
    let bytes = ((i[1] << 29) | (i[0] >> 3)).wrapping_add(pad_len + 8);
    md_update(i, buf, in_, padding.as_ptr(), pad_len, transform);
    md_update(i, buf, in_, bits.as_ptr(), 8, transform);
    *i = [bytes << 3, bytes >> 29];

    for (k, word) in buf.iter().enumerate() {
        digest[4 * k..4 * k + 4].copy_from_slice(&word.to_le_bytes());
    }
    // `buf` already holds the digest bytes in memory order on every little-endian target.
    in_.fill(0);
}

#[no_mangle]
pub unsafe extern "system" fn MD4Init(ctx: *mut MD4_CTX) {
    (*ctx).buf = MD_INIT;
    (*ctx).i = [0, 0];
    (*ctx).in_.fill(0);
}

#[no_mangle]
pub unsafe extern "system" fn MD4Update(ctx: *mut MD4_CTX, buf: *const u8, len: u32) {
    let c = &mut *ctx;
    md_update(&mut c.i, &mut c.buf, &mut c.in_, buf, len, md4_transform);
}

#[no_mangle]
pub unsafe extern "system" fn MD4Final(ctx: *mut MD4_CTX) {
    let c = &mut *ctx;
    md_final(&mut c.i, &mut c.buf, &mut c.in_, &mut c.digest, md4_transform);
}

#[no_mangle]
pub unsafe extern "system" fn MD5Init(ctx: *mut MD5_CTX) {
    (*ctx).buf = MD_INIT;
    (*ctx).i = [0, 0];
    (*ctx).in_.fill(0);
}

#[no_mangle]
pub unsafe extern "system" fn MD5Update(ctx: *mut MD5_CTX, buf: *const u8, len: u32) {
    let c = &mut *ctx;
    md_update(&mut c.i, &mut c.buf, &mut c.in_, buf, len, md5_transform);
}

#[no_mangle]
pub unsafe extern "system" fn MD5Final(ctx: *mut MD5_CTX) {
    let c = &mut *ctx;
    md_final(&mut c.i, &mut c.buf, &mut c.in_, &mut c.digest, md5_transform);
}

// ---- SHA-1 (Steve Reid's public-domain core, as in Wine crypt.c / cryptlib sha1.c) ----------

fn sha1_transform(state: &mut [u32; 5], block: &[u8; 64]) {
    let mut w = [0u32; 80];
    for k in 0..16 {
        w[k] = u32::from_be_bytes([block[4 * k], block[4 * k + 1], block[4 * k + 2], block[4 * k + 3]]);
    }
    for k in 16..80 {
        w[k] = (w[k - 3] ^ w[k - 8] ^ w[k - 14] ^ w[k - 16]).rotate_left(1);
    }
    let (mut a, mut b, mut c, mut d, mut e) = (state[0], state[1], state[2], state[3], state[4]);
    for (k, &wk) in w.iter().enumerate() {
        let (f, konst) = match k / 20 {
            0 => (d ^ (b & (c ^ d)), 0x5A82_7999),
            1 => (b ^ c ^ d, 0x6ED9_EBA1),
            2 => ((b & c) | (d & (b | c)), 0x8F1B_BCDC),
            _ => (b ^ c ^ d, 0xCA62_C1D6),
        };
        let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(konst).wrapping_add(wk);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = t;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

#[no_mangle]
pub unsafe extern "system" fn A_SHAInit(ctx: *mut A_SHA_CTX) {
    (*ctx).Buffer.fill(0);
    (*ctx).State = SHA_INIT;
    (*ctx).Count = [0, 0];
}

/// Count[1] is the low 32 bits of the BYTE count, Count[0] the high part (Wine/cryptlib).
#[no_mangle]
pub unsafe extern "system" fn A_SHAUpdate(ctx: *mut A_SHA_CTX, data: *const u8, len: u32) {
    let c = &mut *ctx;
    let used = (c.Count[1] & 63) as usize;
    c.Count[1] = c.Count[1].wrapping_add(len);
    if c.Count[1] < len {
        c.Count[0] = c.Count[0].wrapping_add(1);
    }
    c.Count[0] = c.Count[0].wrapping_add(len >> 29);

    if len == 0 {
        return;
    }
    let mut src = core::slice::from_raw_parts(data, len as usize);
    if used != 0 {
        let fill = 64 - used;
        if src.len() < fill {
            c.Buffer[used..used + src.len()].copy_from_slice(src);
            return;
        }
        c.Buffer[used..].copy_from_slice(&src[..fill]);
        sha1_transform(&mut c.State, &c.Buffer);
        src = &src[fill..];
    }
    while src.len() >= 64 {
        let (block, rest) = src.split_at(64);
        sha1_transform(&mut c.State, &*(block.as_ptr() as *const [u8; 64]));
        src = rest;
    }
    c.Buffer[..src.len()].copy_from_slice(src);
}

/// Result receives the 20-byte digest (five ULONGs in big-endian byte order).
#[no_mangle]
pub unsafe extern "system" fn A_SHAFinal(ctx: *mut A_SHA_CTX, result: *mut u32) {
    let c = &mut *ctx;
    let used = c.Count[1] & 63;
    let pad = if used >= 56 { 56 + 64 - used } else { 56 - used } as usize;
    let length_hi = (c.Count[0] << 3) | (c.Count[1] >> 29);
    let length_lo = c.Count[1] << 3;

    let mut tail = [0u8; 72];
    tail[0] = 0x80;
    tail[pad..pad + 4].copy_from_slice(&length_hi.to_be_bytes());
    tail[pad + 4..pad + 8].copy_from_slice(&length_lo.to_be_bytes());
    A_SHAUpdate(ctx, tail.as_ptr(), (pad + 8) as u32);

    let mut digest = [0u8; 20];
    for (k, word) in (*ctx).State.iter().enumerate() {
        digest[4 * k..4 * k + 4].copy_from_slice(&word.to_be_bytes());
    }
    // Result is a PULONG but callers pass byte buffers: no alignment is assumed.
    core::ptr::copy_nonoverlapping(digest.as_ptr(), result as *mut u8, 20);
    A_SHAInit(ctx);
}

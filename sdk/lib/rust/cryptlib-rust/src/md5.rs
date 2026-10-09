//! md5.rs — MD5 message-digest (sdk/lib/cryptlib/md5.c).
//! ABI: NTAPI = extern "system" (stdcall on i386, cdecl on amd64/arm64).
//!
//! FIXES applied:
//!   - extern "C"/"system" { fn memcpy/memset/byteReverse/MD5Transform } blocks removed.
//!     All replaced with core::ptr intrinsics and local fns.
//!   - MD5_CTX defined in crate::types (field order: i[2], buf[4], in_0[64], digest[16]).
//!     NOTE: MD5_CTX has i[] BEFORE buf[] — different from MD4_CTX!

use crate::types::*;

/// byteReverse is a no-op on all our LE targets.
#[inline(always)]
unsafe fn byteReverse(_buf: *mut u8, _longs: u32) {
    // little-endian no-op
}

#[no_mangle]
pub unsafe extern "system" fn MD5Init(ctx: *mut MD5_CTX) {
    unsafe {
        (*ctx).buf[0] = 0x67452301u32;
        (*ctx).buf[1] = 0xefcdab89u32;
        (*ctx).buf[2] = 0x98badcfeu32;
        (*ctx).buf[3] = 0x10325476u32;
        (*ctx).i[1] = 0;
        (*ctx).i[0] = (*ctx).i[1];
    }
}

unsafe fn MD5Transform(buf: *mut u32, in_: *const u32) {
    let mut a: u32 = *buf.offset(0);
    let mut b: u32 = *buf.offset(1);
    let mut c: u32 = *buf.offset(2);
    let mut d: u32 = *buf.offset(3);

    // Round 1
    a = a.wrapping_add(
        (d ^ b & (c ^ d))
            .wrapping_add(*in_.offset(0))
            .wrapping_add(0xd76aa478u32),
    );
    a = (a << 7) | (a >> (32 - 7));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (c ^ a & (b ^ c))
            .wrapping_add(*in_.offset(1))
            .wrapping_add(0xe8c7b756u32),
    );
    d = (d << 12) | (d >> (32 - 12));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (b ^ d & (a ^ b))
            .wrapping_add(*in_.offset(2))
            .wrapping_add(0x242070dbu32),
    );
    c = (c << 17) | (c >> (32 - 17));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (a ^ c & (d ^ a))
            .wrapping_add(*in_.offset(3))
            .wrapping_add(0xc1bdceeeu32),
    );
    b = (b << 22) | (b >> (32 - 22));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (d ^ b & (c ^ d))
            .wrapping_add(*in_.offset(4))
            .wrapping_add(0xf57c0fafu32),
    );
    a = (a << 7) | (a >> (32 - 7));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (c ^ a & (b ^ c))
            .wrapping_add(*in_.offset(5))
            .wrapping_add(0x4787c62au32),
    );
    d = (d << 12) | (d >> (32 - 12));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (b ^ d & (a ^ b))
            .wrapping_add(*in_.offset(6))
            .wrapping_add(0xa8304613u32),
    );
    c = (c << 17) | (c >> (32 - 17));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (a ^ c & (d ^ a))
            .wrapping_add(*in_.offset(7))
            .wrapping_add(0xfd469501u32),
    );
    b = (b << 22) | (b >> (32 - 22));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (d ^ b & (c ^ d))
            .wrapping_add(*in_.offset(8))
            .wrapping_add(0x698098d8u32),
    );
    a = (a << 7) | (a >> (32 - 7));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (c ^ a & (b ^ c))
            .wrapping_add(*in_.offset(9))
            .wrapping_add(0x8b44f7afu32),
    );
    d = (d << 12) | (d >> (32 - 12));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (b ^ d & (a ^ b))
            .wrapping_add(*in_.offset(10))
            .wrapping_add(0xffff5bb1u32),
    );
    c = (c << 17) | (c >> (32 - 17));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (a ^ c & (d ^ a))
            .wrapping_add(*in_.offset(11))
            .wrapping_add(0x895cd7beu32),
    );
    b = (b << 22) | (b >> (32 - 22));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (d ^ b & (c ^ d))
            .wrapping_add(*in_.offset(12))
            .wrapping_add(0x6b901122u32),
    );
    a = (a << 7) | (a >> (32 - 7));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (c ^ a & (b ^ c))
            .wrapping_add(*in_.offset(13))
            .wrapping_add(0xfd987193u32),
    );
    d = (d << 12) | (d >> (32 - 12));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (b ^ d & (a ^ b))
            .wrapping_add(*in_.offset(14))
            .wrapping_add(0xa679438eu32),
    );
    c = (c << 17) | (c >> (32 - 17));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (a ^ c & (d ^ a))
            .wrapping_add(*in_.offset(15))
            .wrapping_add(0x49b40821u32),
    );
    b = (b << 22) | (b >> (32 - 22));
    b = b.wrapping_add(c);

    // Round 2
    a = a.wrapping_add(
        (c ^ d & (b ^ c))
            .wrapping_add(*in_.offset(1))
            .wrapping_add(0xf61e2562u32),
    );
    a = (a << 5) | (a >> (32 - 5));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ c & (a ^ b))
            .wrapping_add(*in_.offset(6))
            .wrapping_add(0xc040b340u32),
    );
    d = (d << 9) | (d >> (32 - 9));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ b & (d ^ a))
            .wrapping_add(*in_.offset(11))
            .wrapping_add(0x265e5a51u32),
    );
    c = (c << 14) | (c >> (32 - 14));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ a & (c ^ d))
            .wrapping_add(*in_.offset(0))
            .wrapping_add(0xe9b6c7aau32),
    );
    b = (b << 20) | (b >> (32 - 20));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (c ^ d & (b ^ c))
            .wrapping_add(*in_.offset(5))
            .wrapping_add(0xd62f105du32),
    );
    a = (a << 5) | (a >> (32 - 5));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ c & (a ^ b))
            .wrapping_add(*in_.offset(10))
            .wrapping_add(0x02441453u32),
    );
    d = (d << 9) | (d >> (32 - 9));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ b & (d ^ a))
            .wrapping_add(*in_.offset(15))
            .wrapping_add(0xd8a1e681u32),
    );
    c = (c << 14) | (c >> (32 - 14));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ a & (c ^ d))
            .wrapping_add(*in_.offset(4))
            .wrapping_add(0xe7d3fbc8u32),
    );
    b = (b << 20) | (b >> (32 - 20));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (c ^ d & (b ^ c))
            .wrapping_add(*in_.offset(9))
            .wrapping_add(0x21e1cde6u32),
    );
    a = (a << 5) | (a >> (32 - 5));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ c & (a ^ b))
            .wrapping_add(*in_.offset(14))
            .wrapping_add(0xc33707d6u32),
    );
    d = (d << 9) | (d >> (32 - 9));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ b & (d ^ a))
            .wrapping_add(*in_.offset(3))
            .wrapping_add(0xf4d50d87u32),
    );
    c = (c << 14) | (c >> (32 - 14));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ a & (c ^ d))
            .wrapping_add(*in_.offset(8))
            .wrapping_add(0x455a14edu32),
    );
    b = (b << 20) | (b >> (32 - 20));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (c ^ d & (b ^ c))
            .wrapping_add(*in_.offset(13))
            .wrapping_add(0xa9e3e905u32),
    );
    a = (a << 5) | (a >> (32 - 5));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ c & (a ^ b))
            .wrapping_add(*in_.offset(2))
            .wrapping_add(0xfcefa3f8u32),
    );
    d = (d << 9) | (d >> (32 - 9));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ b & (d ^ a))
            .wrapping_add(*in_.offset(7))
            .wrapping_add(0x676f02d9u32),
    );
    c = (c << 14) | (c >> (32 - 14));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ a & (c ^ d))
            .wrapping_add(*in_.offset(12))
            .wrapping_add(0x8d2a4c8au32),
    );
    b = (b << 20) | (b >> (32 - 20));
    b = b.wrapping_add(c);

    // Round 3
    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*in_.offset(5))
            .wrapping_add(0xfffa3942u32),
    );
    a = (a << 4) | (a >> (32 - 4));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*in_.offset(8))
            .wrapping_add(0x8771f681u32),
    );
    d = (d << 11) | (d >> (32 - 11));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*in_.offset(11))
            .wrapping_add(0x6d9d6122u32),
    );
    c = (c << 16) | (c >> (32 - 16));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*in_.offset(14))
            .wrapping_add(0xfde5380cu32),
    );
    b = (b << 23) | (b >> (32 - 23));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*in_.offset(1))
            .wrapping_add(0xa4beea44u32),
    );
    a = (a << 4) | (a >> (32 - 4));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*in_.offset(4))
            .wrapping_add(0x4bdecfa9u32),
    );
    d = (d << 11) | (d >> (32 - 11));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*in_.offset(7))
            .wrapping_add(0xf6bb4b60u32),
    );
    c = (c << 16) | (c >> (32 - 16));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*in_.offset(10))
            .wrapping_add(0xbebfbc70u32),
    );
    b = (b << 23) | (b >> (32 - 23));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*in_.offset(13))
            .wrapping_add(0x289b7ec6u32),
    );
    a = (a << 4) | (a >> (32 - 4));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*in_.offset(0))
            .wrapping_add(0xeaa127fau32),
    );
    d = (d << 11) | (d >> (32 - 11));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*in_.offset(3))
            .wrapping_add(0xd4ef3085u32),
    );
    c = (c << 16) | (c >> (32 - 16));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*in_.offset(6))
            .wrapping_add(0x04881d05u32),
    );
    b = (b << 23) | (b >> (32 - 23));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*in_.offset(9))
            .wrapping_add(0xd9d4d039u32),
    );
    a = (a << 4) | (a >> (32 - 4));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*in_.offset(12))
            .wrapping_add(0xe6db99e5u32),
    );
    d = (d << 11) | (d >> (32 - 11));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*in_.offset(15))
            .wrapping_add(0x1fa27cf8u32),
    );
    c = (c << 16) | (c >> (32 - 16));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*in_.offset(2))
            .wrapping_add(0xc4ac5665u32),
    );
    b = (b << 23) | (b >> (32 - 23));
    b = b.wrapping_add(c);

    // Round 4
    a = a.wrapping_add(
        (c ^ (b | !d))
            .wrapping_add(*in_.offset(0))
            .wrapping_add(0xf4292244u32),
    );
    a = (a << 6) | (a >> (32 - 6));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ (a | !c))
            .wrapping_add(*in_.offset(7))
            .wrapping_add(0x432aff97u32),
    );
    d = (d << 10) | (d >> (32 - 10));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ (d | !b))
            .wrapping_add(*in_.offset(14))
            .wrapping_add(0xab9423a7u32),
    );
    c = (c << 15) | (c >> (32 - 15));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ (c | !a))
            .wrapping_add(*in_.offset(5))
            .wrapping_add(0xfc93a039u32),
    );
    b = (b << 21) | (b >> (32 - 21));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (c ^ (b | !d))
            .wrapping_add(*in_.offset(12))
            .wrapping_add(0x655b59c3u32),
    );
    a = (a << 6) | (a >> (32 - 6));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ (a | !c))
            .wrapping_add(*in_.offset(3))
            .wrapping_add(0x8f0ccc92u32),
    );
    d = (d << 10) | (d >> (32 - 10));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ (d | !b))
            .wrapping_add(*in_.offset(10))
            .wrapping_add(0xffeff47du32),
    );
    c = (c << 15) | (c >> (32 - 15));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ (c | !a))
            .wrapping_add(*in_.offset(1))
            .wrapping_add(0x85845dd1u32),
    );
    b = (b << 21) | (b >> (32 - 21));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (c ^ (b | !d))
            .wrapping_add(*in_.offset(8))
            .wrapping_add(0x6fa87e4fu32),
    );
    a = (a << 6) | (a >> (32 - 6));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ (a | !c))
            .wrapping_add(*in_.offset(15))
            .wrapping_add(0xfe2ce6e0u32),
    );
    d = (d << 10) | (d >> (32 - 10));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ (d | !b))
            .wrapping_add(*in_.offset(6))
            .wrapping_add(0xa3014314u32),
    );
    c = (c << 15) | (c >> (32 - 15));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ (c | !a))
            .wrapping_add(*in_.offset(13))
            .wrapping_add(0x4e0811a1u32),
    );
    b = (b << 21) | (b >> (32 - 21));
    b = b.wrapping_add(c);

    a = a.wrapping_add(
        (c ^ (b | !d))
            .wrapping_add(*in_.offset(4))
            .wrapping_add(0xf7537e82u32),
    );
    a = (a << 6) | (a >> (32 - 6));
    a = a.wrapping_add(b);

    d = d.wrapping_add(
        (b ^ (a | !c))
            .wrapping_add(*in_.offset(11))
            .wrapping_add(0xbd3af235u32),
    );
    d = (d << 10) | (d >> (32 - 10));
    d = d.wrapping_add(a);

    c = c.wrapping_add(
        (a ^ (d | !b))
            .wrapping_add(*in_.offset(2))
            .wrapping_add(0x2ad7d2bbu32),
    );
    c = (c << 15) | (c >> (32 - 15));
    c = c.wrapping_add(d);

    b = b.wrapping_add(
        (d ^ (c | !a))
            .wrapping_add(*in_.offset(9))
            .wrapping_add(0xeb86d391u32),
    );
    b = (b << 21) | (b >> (32 - 21));
    b = b.wrapping_add(c);

    *buf.offset(0) = (*buf.offset(0)).wrapping_add(a);
    *buf.offset(1) = (*buf.offset(1)).wrapping_add(b);
    *buf.offset(2) = (*buf.offset(2)).wrapping_add(c);
    *buf.offset(3) = (*buf.offset(3)).wrapping_add(d);
}

#[no_mangle]
pub unsafe extern "system" fn MD5Update(
    mut ctx: *mut MD5_CTX,
    mut buf: *const u8,
    mut len: u32,
) {
    // FIX: extern "C" { fn memcpy/byteReverse/MD5Transform } removed;
    // replaced with core::ptr::copy_nonoverlapping and local fns.
    let mut t: u32;
    t = (*ctx).i[0 as usize];
    (*ctx).i[0 as usize] = t.wrapping_add(len << 3);
    if (*ctx).i[0 as usize] < t {
        (*ctx).i[1 as usize] = (*ctx).i[1 as usize].wrapping_add(1);
    }
    (*ctx).i[1 as usize] = (*ctx).i[1 as usize].wrapping_add(len >> 29);
    t = (t >> 3) & 0x3f;
    if t != 0 {
        let p: *mut u8 = (&raw mut (*ctx).in_0 as *mut u8).offset(t as isize);
        t = 64u32.wrapping_sub(t);
        if len < t {
            core::ptr::copy_nonoverlapping(buf as *const u8, p, len as usize);
            return;
        }
        core::ptr::copy_nonoverlapping(buf as *const u8, p, t as usize);
        byteReverse(&raw mut (*ctx).in_0 as *mut u8, 16);
        MD5Transform(
            &raw mut (*ctx).buf as *mut u32,
            &raw mut (*ctx).in_0 as *mut u8 as *mut u32 as *const u32,
        );
        buf = buf.offset(t as isize);
        len = len.wrapping_sub(t);
    }
    while len >= 64 {
        core::ptr::copy_nonoverlapping(
            buf as *const u8,
            &raw mut (*ctx).in_0 as *mut u8,
            64usize,
        );
        byteReverse(&raw mut (*ctx).in_0 as *mut u8, 16);
        MD5Transform(
            &raw mut (*ctx).buf as *mut u32,
            &raw mut (*ctx).in_0 as *mut u8 as *mut u32 as *const u32,
        );
        buf = buf.offset(64isize);
        len = len.wrapping_sub(64);
    }
    core::ptr::copy_nonoverlapping(
        buf as *const u8,
        &raw mut (*ctx).in_0 as *mut u8,
        len as usize,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MD5Final(ctx: *mut MD5_CTX) {
    // FIX: extern "system" { fn memset/memcpy/MD5Transform } removed;
    // replaced with core::ptr::write_bytes, core::ptr::copy_nonoverlapping, local MD5Transform.
    let mut count: u32;
    let mut p: *mut u8;

    count = ((*ctx).i[0] as u32).wrapping_shr(3) & 0x3F_u32;
    p = (&raw mut (*ctx).in_0 as *mut u8).offset(count as isize);
    let fresh4 = p;
    p = p.offset(1);
    *fresh4 = 0x80_u8;
    count = (64u32 - 1u32).wrapping_sub(count);

    if count < 8u32 {
        core::ptr::write_bytes(p as *mut core::ffi::c_void, 0, count as usize);
        MD5Transform(
            &raw mut (*ctx).buf as *mut u32,
            &raw mut (*ctx).in_0 as *mut u8 as *mut u32 as *const u32,
        );
        core::ptr::write_bytes(
            &raw mut (*ctx).in_0 as *mut u8 as *mut core::ffi::c_void,
            0,
            56usize,
        );
    } else {
        core::ptr::write_bytes(
            p as *mut core::ffi::c_void,
            0,
            count.wrapping_sub(8u32) as usize,
        );
    }

    *((&raw mut (*ctx).in_0 as *mut u8 as *mut u32).offset(14isize)) = (*ctx).i[0];
    *((&raw mut (*ctx).in_0 as *mut u8 as *mut u32).offset(15isize)) = (*ctx).i[1];

    MD5Transform(
        &raw mut (*ctx).buf as *mut u32,
        &raw mut (*ctx).in_0 as *mut u8 as *mut u32 as *const u32,
    );

    core::ptr::copy_nonoverlapping(
        &raw const (*ctx).buf as *const u8,
        &raw mut (*ctx).digest as *mut u8,
        16usize,
    );

    core::ptr::write_bytes(
        &raw mut (*ctx).in_0 as *mut u8 as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[u8; 64]>() as usize,
    );
}

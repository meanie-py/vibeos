//! md4.rs — MD4 message-digest (sdk/lib/cryptlib/md4.c).
//! ABI: NTAPI = extern "system" (stdcall on i386, cdecl on amd64/arm64).
//!
//! FIXES applied:
//!   - extern "C" { fn byteReverse } removed; replaced with local no-op inline fn.
//!     All our targets are little-endian; byteReverse is a compile-time no-op.
//!     The C source compiles byteReverse away with #ifdef __BYTE_ORDER__ == BIG_ENDIAN.
//!   - MD4_CTX defined in crate::types (sourced from c2rust + md4.h, layout verified).

use crate::types::*;

/// byteReverse is a no-op on all our LE targets (x86/amd64/arm64).
/// The C util.c conditionally provides it only for big-endian.
#[inline(always)]
unsafe fn byteReverse(_buf: *mut u8, _longs: u32) {
    // little-endian no-op
}

#[no_mangle]
pub unsafe extern "system" fn MD4Init(ctx: *mut MD4_CTX) {
    (*ctx).buf[0] = 0x67452301u32;
    (*ctx).buf[1] = 0xefcdab89u32;
    (*ctx).buf[2] = 0x98badcfeu32;
    (*ctx).buf[3] = 0x10325476u32;
    (*ctx).i[1] = 0u32;
    (*ctx).i[0] = (*ctx).i[1];
}

#[no_mangle]
pub unsafe extern "system" fn MD4Update(ctx: *mut MD4_CTX, mut buf: *const u8, mut len: u32) {
    let mut t: u32 = (*ctx).i[0 as usize];

    // Update bitcount
    (*ctx).i[0 as usize] = t.wrapping_add(len << 3);
    if (*ctx).i[0 as usize] < t {
        (*ctx).i[1 as usize] = (*ctx).i[1 as usize].wrapping_add(1);
    }
    (*ctx).i[1 as usize] = (*ctx).i[1 as usize].wrapping_add(len >> 29);
    t = (t >> 3) & 0x3f;

    // Handle any leading odd-sized chunks
    if t != 0 {
        let mut p: *mut u8 = (&raw mut (*ctx).in_0 as *mut u8).offset(t as isize);
        t = 64u32.wrapping_sub(t);
        if len < t {
            core::ptr::copy_nonoverlapping(buf as *const u8, p, len as usize);
            return;
        }
        core::ptr::copy_nonoverlapping(buf as *const u8, p, t as usize);
        byteReverse(&raw mut (*ctx).in_0 as *mut u8, 16);
        MD4Transform(
            &raw mut (*ctx).buf as *mut u32,
            &raw mut (*ctx).in_0 as *mut u8 as *mut u32 as *const u32,
        );
        buf = buf.offset(t as isize);
        len = len.wrapping_sub(t);
    }

    // Process data in 64-byte chunks
    while len >= 64 {
        core::ptr::copy_nonoverlapping(
            buf as *const u8,
            &raw mut (*ctx).in_0 as *mut u8,
            64,
        );
        byteReverse(&raw mut (*ctx).in_0 as *mut u8, 16);
        MD4Transform(
            &raw mut (*ctx).buf as *mut u32,
            &raw mut (*ctx).in_0 as *mut u8 as *mut u32 as *const u32,
        );
        buf = buf.offset(64);
        len = len.wrapping_sub(64);
    }

    // Handle any remaining bytes of data
    core::ptr::copy_nonoverlapping(
        buf as *const u8,
        &raw mut (*ctx).in_0 as *mut u8,
        len as usize,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MD4Final(ctx: *mut MD4_CTX) {
    let mut count: u32;
    let mut p: *mut u8;

    count = ((*ctx).i[0] >> 3) & 0x3F;

    p = core::ptr::addr_of_mut!((*ctx).in_0) as *mut u8;
    p = p.add(count as usize);
    *p = 0x80;
    p = p.add(1);

    count = (64u32)
        .wrapping_sub(1u32)
        .wrapping_sub(count);

    if count < 8 {
        core::ptr::write_bytes(p as *mut core::ffi::c_void, 0, count as usize);
        byteReverse(core::ptr::addr_of_mut!((*ctx).in_0) as *mut u8, 16);
        MD4Transform(
            core::ptr::addr_of_mut!((*ctx).buf) as *mut u32,
            core::ptr::addr_of!((*ctx).in_0) as *const u32,
        );
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*ctx).in_0) as *mut core::ffi::c_void,
            0,
            56,
        );
    } else {
        core::ptr::write_bytes(
            p as *mut core::ffi::c_void,
            0,
            count.wrapping_sub(8) as usize,
        );
    }

    byteReverse(core::ptr::addr_of_mut!((*ctx).in_0) as *mut u8, 14);

    *(core::ptr::addr_of_mut!((*ctx).in_0) as *mut u32)
        .add(14) = (*ctx).i[0];
    *(core::ptr::addr_of_mut!((*ctx).in_0) as *mut u32)
        .add(15) = (*ctx).i[1];

    MD4Transform(
        core::ptr::addr_of_mut!((*ctx).buf) as *mut u32,
        core::ptr::addr_of!((*ctx).in_0) as *const u32,
    );

    byteReverse(
        core::ptr::addr_of_mut!((*ctx).buf) as *mut u8,
        4,
    );

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*ctx).buf) as *const u8,
        core::ptr::addr_of_mut!((*ctx).digest) as *mut u8,
        16,
    );

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*ctx).in_0) as *mut core::ffi::c_void,
        0,
        64,
    );
}

unsafe fn MD4Transform(buf: *mut u32, r#in: *const u32) {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut d: u32;
    a = *buf.add(0);
    b = *buf.add(1);
    c = *buf.add(2);
    d = *buf.add(3);

    // FF rounds
    a = a.wrapping_add((b & c | !b & d).wrapping_add(*r#in.add(0)));
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add((a & b | !a & c).wrapping_add(*r#in.add(1)));
    d = (d << 7u32) | (d >> 25u32);
    c = c.wrapping_add((d & a | !d & b).wrapping_add(*r#in.add(2)));
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add((c & d | !c & a).wrapping_add(*r#in.add(3)));
    b = (b << 19u32) | (b >> 13u32);
    a = a.wrapping_add((b & c | !b & d).wrapping_add(*r#in.add(4)));
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add((a & b | !a & c).wrapping_add(*r#in.add(5)));
    d = (d << 7u32) | (d >> 25u32);
    c = c.wrapping_add((d & a | !d & b).wrapping_add(*r#in.add(6)));
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add((c & d | !c & a).wrapping_add(*r#in.add(7)));
    b = (b << 19u32) | (b >> 13u32);
    a = a.wrapping_add((b & c | !b & d).wrapping_add(*r#in.add(8)));
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add((a & b | !a & c).wrapping_add(*r#in.add(9)));
    d = (d << 7u32) | (d >> 25u32);
    c = c.wrapping_add((d & a | !d & b).wrapping_add(*r#in.add(10)));
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add((c & d | !c & a).wrapping_add(*r#in.add(11)));
    b = (b << 19u32) | (b >> 13u32);
    a = a.wrapping_add((b & c | !b & d).wrapping_add(*r#in.add(12)));
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add((a & b | !a & c).wrapping_add(*r#in.add(13)));
    d = (d << 7u32) | (d >> 25u32);
    c = c.wrapping_add((d & a | !d & b).wrapping_add(*r#in.add(14)));
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add((c & d | !c & a).wrapping_add(*r#in.add(15)));
    b = (b << 19u32) | (b >> 13u32);

    // GG rounds
    a = a.wrapping_add(
        (b & c | b & d | c & d)
            .wrapping_add(*r#in.add(0))
            .wrapping_add(0x5a827999u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a & b | a & c | b & c)
            .wrapping_add(*r#in.add(4))
            .wrapping_add(0x5a827999u32),
    );
    d = (d << 5u32) | (d >> 27u32);
    c = c.wrapping_add(
        (d & a | d & b | a & b)
            .wrapping_add(*r#in.add(8))
            .wrapping_add(0x5a827999u32),
    );
    c = (c << 9u32) | (c >> 23u32);
    b = b.wrapping_add(
        (c & d | c & a | d & a)
            .wrapping_add(*r#in.add(12))
            .wrapping_add(0x5a827999u32),
    );
    b = (b << 13u32) | (b >> 19u32);
    a = a.wrapping_add(
        (b & c | b & d | c & d)
            .wrapping_add(*r#in.add(1))
            .wrapping_add(0x5a827999u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a & b | a & c | b & c)
            .wrapping_add(*r#in.add(5))
            .wrapping_add(0x5a827999u32),
    );
    d = (d << 5u32) | (d >> 27u32);
    c = c.wrapping_add(
        (d & a | d & b | a & b)
            .wrapping_add(*r#in.add(9))
            .wrapping_add(0x5a827999u32),
    );
    c = (c << 9u32) | (c >> 23u32);
    b = b.wrapping_add(
        (c & d | c & a | d & a)
            .wrapping_add(*r#in.add(13))
            .wrapping_add(0x5a827999u32),
    );
    b = (b << 13u32) | (b >> 19u32);
    a = a.wrapping_add(
        (b & c | b & d | c & d)
            .wrapping_add(*r#in.add(2))
            .wrapping_add(0x5a827999u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a & b | a & c | b & c)
            .wrapping_add(*r#in.add(6))
            .wrapping_add(0x5a827999u32),
    );
    d = (d << 5u32) | (d >> 27u32);
    c = c.wrapping_add(
        (d & a | d & b | a & b)
            .wrapping_add(*r#in.add(10))
            .wrapping_add(0x5a827999u32),
    );
    c = (c << 9u32) | (c >> 23u32);
    b = b.wrapping_add(
        (c & d | c & a | d & a)
            .wrapping_add(*r#in.add(14))
            .wrapping_add(0x5a827999u32),
    );
    b = (b << 13u32) | (b >> 19u32);
    a = a.wrapping_add(
        (b & c | b & d | c & d)
            .wrapping_add(*r#in.add(3))
            .wrapping_add(0x5a827999u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a & b | a & c | b & c)
            .wrapping_add(*r#in.add(7))
            .wrapping_add(0x5a827999u32),
    );
    d = (d << 5u32) | (d >> 27u32);
    c = c.wrapping_add(
        (d & a | d & b | a & b)
            .wrapping_add(*r#in.add(11))
            .wrapping_add(0x5a827999u32),
    );
    c = (c << 9u32) | (c >> 23u32);
    b = b.wrapping_add(
        (c & d | c & a | d & a)
            .wrapping_add(*r#in.add(15))
            .wrapping_add(0x5a827999u32),
    );
    b = (b << 13u32) | (b >> 19u32);

    // HH rounds
    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*r#in.add(0))
            .wrapping_add(0x6ed9eba1u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*r#in.add(8))
            .wrapping_add(0x6ed9eba1u32),
    );
    d = (d << 9u32) | (d >> 23u32);
    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*r#in.add(4))
            .wrapping_add(0x6ed9eba1u32),
    );
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*r#in.add(12))
            .wrapping_add(0x6ed9eba1u32),
    );
    b = (b << 15u32) | (b >> 17u32);
    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*r#in.add(2))
            .wrapping_add(0x6ed9eba1u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*r#in.add(10))
            .wrapping_add(0x6ed9eba1u32),
    );
    d = (d << 9u32) | (d >> 23u32);
    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*r#in.add(6))
            .wrapping_add(0x6ed9eba1u32),
    );
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*r#in.add(14))
            .wrapping_add(0x6ed9eba1u32),
    );
    b = (b << 15u32) | (b >> 17u32);
    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*r#in.add(1))
            .wrapping_add(0x6ed9eba1u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*r#in.add(9))
            .wrapping_add(0x6ed9eba1u32),
    );
    d = (d << 9u32) | (d >> 23u32);
    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*r#in.add(5))
            .wrapping_add(0x6ed9eba1u32),
    );
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*r#in.add(13))
            .wrapping_add(0x6ed9eba1u32),
    );
    b = (b << 15u32) | (b >> 17u32);
    a = a.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(*r#in.add(3))
            .wrapping_add(0x6ed9eba1u32),
    );
    a = (a << 3u32) | (a >> 29u32);
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(*r#in.add(11))
            .wrapping_add(0x6ed9eba1u32),
    );
    d = (d << 9u32) | (d >> 23u32);
    c = c.wrapping_add(
        (d ^ a ^ b)
            .wrapping_add(*r#in.add(7))
            .wrapping_add(0x6ed9eba1u32),
    );
    c = (c << 11u32) | (c >> 21u32);
    b = b.wrapping_add(
        (c ^ d ^ a)
            .wrapping_add(*r#in.add(15))
            .wrapping_add(0x6ed9eba1u32),
    );
    b = (b << 15u32) | (b >> 17u32);

    *buf = (*buf).wrapping_add(a);
    *buf.add(1) = (*buf.add(1)).wrapping_add(b);
    *buf.add(2) = (*buf.add(2)).wrapping_add(c);
    *buf.add(3) = (*buf.add(3)).wrapping_add(d);
}

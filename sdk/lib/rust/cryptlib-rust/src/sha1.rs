//! sha1.rs — SHA-1 message-digest (sdk/lib/cryptlib/sha1.c).
//! ABI: NTAPI = extern "system" (stdcall on i386, cdecl on amd64/arm64).
//!
//! FIXES applied:
//!   - BLOCKER (infinite loop): A_SHAUpdate had `BufferContentSize`, `Buffer`, `BufferSize`
//!     declared without `let mut`. The while-loop mutates all three — without mut the compiler
//!     silently uses the initial values and the loop never terminates. Fixed: all three are now
//!     `let mut`.
//!   - `use ntkrnl_sys::*` stray imports removed (not available in no_std kernel crate without
//!     the ntkrnl-sys dep being pulled in; also not needed — all types are in crate::types).
//!   - `extern "C" { fn memcpy }` removed; replaced with core::ptr::copy_nonoverlapping.
//!   - SHA_CTX / PSHA_CTX defined in crate::types (sourced from sha1.h).

use crate::types::*;

unsafe fn SHA1Transform(State: *mut u32, Buffer: *mut u8) {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut d: u32;
    let mut e: u32;
    let Block = Buffer as *mut u32;

    a = *State.add(0);
    b = *State.add(1);
    c = *State.add(2);
    d = *State.add(3);
    e = *State.add(4);

    // Index 0
    let blk_val = ((*Block.add(0) << 24 | *Block.add(0) >> 8) & 0xff00ff00)
                | ((*Block.add(0) << 8 | *Block.add(0) >> 24) & 0x00ff00ff);
    *Block.add(0) = blk_val;
    e = e.wrapping_add(
        (d ^ (b & (c ^ d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 1
    let blk_val = ((*Block.add(1) << 24 | *Block.add(1) >> 8) & 0xff00ff00)
                | ((*Block.add(1) << 8 | *Block.add(1) >> 24) & 0x00ff00ff);
    *Block.add(1) = blk_val;
    d = d.wrapping_add(
        (c ^ (a & (b ^ c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 2
    let blk_val = ((*Block.add(2) << 24 | *Block.add(2) >> 8) & 0xff00ff00)
                | ((*Block.add(2) << 8 | *Block.add(2) >> 24) & 0x00ff00ff);
    *Block.add(2) = blk_val;
    c = c.wrapping_add(
        (b ^ (e & (a ^ b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 3
    let blk_val = ((*Block.add(3) << 24 | *Block.add(3) >> 8) & 0xff00ff00)
                | ((*Block.add(3) << 8 | *Block.add(3) >> 24) & 0x00ff00ff);
    *Block.add(3) = blk_val;
    b = b.wrapping_add(
        (a ^ (d & (e ^ a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 4
    let blk_val = ((*Block.add(4) << 24 | *Block.add(4) >> 8) & 0xff00ff00)
                | ((*Block.add(4) << 8 | *Block.add(4) >> 24) & 0x00ff00ff);
    *Block.add(4) = blk_val;
    a = a.wrapping_add(
        (e ^ (c & (d ^ e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 5
    let blk_val = ((*Block.add(5) << 24 | *Block.add(5) >> 8) & 0xff00ff00)
                | ((*Block.add(5) << 8 | *Block.add(5) >> 24) & 0x00ff00ff);
    *Block.add(5) = blk_val;
    e = e.wrapping_add(
        (d ^ (b & (c ^ d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 6
    let blk_val = ((*Block.add(6) << 24 | *Block.add(6) >> 8) & 0xff00ff00)
                | ((*Block.add(6) << 8 | *Block.add(6) >> 24) & 0x00ff00ff);
    *Block.add(6) = blk_val;
    d = d.wrapping_add(
        (c ^ (a & (b ^ c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 7
    let blk_val = ((*Block.add(7) << 24 | *Block.add(7) >> 8) & 0xff00ff00)
                | ((*Block.add(7) << 8 | *Block.add(7) >> 24) & 0x00ff00ff);
    *Block.add(7) = blk_val;
    c = c.wrapping_add(
        (b ^ (e & (a ^ b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 8
    let blk_val = ((*Block.add(8) << 24 | *Block.add(8) >> 8) & 0xff00ff00)
                | ((*Block.add(8) << 8 | *Block.add(8) >> 24) & 0x00ff00ff);
    *Block.add(8) = blk_val;
    b = b.wrapping_add(
        (a ^ (d & (e ^ a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 9
    let blk_val = ((*Block.add(9) << 24 | *Block.add(9) >> 8) & 0xff00ff00)
                | ((*Block.add(9) << 8 | *Block.add(9) >> 24) & 0x00ff00ff);
    *Block.add(9) = blk_val;
    a = a.wrapping_add(
        (e ^ (c & (d ^ e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 10
    let blk_val = ((*Block.add(10) << 24 | *Block.add(10) >> 8) & 0xff00ff00)
                | ((*Block.add(10) << 8 | *Block.add(10) >> 24) & 0x00ff00ff);
    *Block.add(10) = blk_val;
    e = e.wrapping_add(
        (d ^ (b & (c ^ d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 11
    let blk_val = ((*Block.add(11) << 24 | *Block.add(11) >> 8) & 0xff00ff00)
                | ((*Block.add(11) << 8 | *Block.add(11) >> 24) & 0x00ff00ff);
    *Block.add(11) = blk_val;
    d = d.wrapping_add(
        (c ^ (a & (b ^ c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 12
    let blk_val = ((*Block.add(12) << 24 | *Block.add(12) >> 8) & 0xff00ff00)
                | ((*Block.add(12) << 8 | *Block.add(12) >> 24) & 0x00ff00ff);
    *Block.add(12) = blk_val;
    c = c.wrapping_add(
        (b ^ (e & (a ^ b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 13
    let blk_val = ((*Block.add(13) << 24 | *Block.add(13) >> 8) & 0xff00ff00)
                | ((*Block.add(13) << 8 | *Block.add(13) >> 24) & 0x00ff00ff);
    *Block.add(13) = blk_val;
    b = b.wrapping_add(
        (a ^ (d & (e ^ a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 14
    let blk_val = ((*Block.add(14) << 24 | *Block.add(14) >> 8) & 0xff00ff00)
                | ((*Block.add(14) << 8 | *Block.add(14) >> 24) & 0x00ff00ff);
    *Block.add(14) = blk_val;
    a = a.wrapping_add(
        (e ^ (c & (d ^ e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 15
    let blk_val = ((*Block.add(15) << 24 | *Block.add(15) >> 8) & 0xff00ff00)
                | ((*Block.add(15) << 8 | *Block.add(15) >> 24) & 0x00ff00ff);
    *Block.add(15) = blk_val;
    e = e.wrapping_add(
        (d ^ (b & (c ^ d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 16
    let xor_val = *Block.add(29 & 15) ^ *Block.add(24 & 15) ^ *Block.add(18 & 15) ^ *Block.add(16 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(16 & 15) = blk_val;
    d = d.wrapping_add(
        (c ^ (a & (b ^ c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 17
    let xor_val = *Block.add(30 & 15) ^ *Block.add(25 & 15) ^ *Block.add(19 & 15) ^ *Block.add(17 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(17 & 15) = blk_val;
    c = c.wrapping_add(
        (b ^ (e & (a ^ b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 18
    let xor_val = *Block.add(31 & 15) ^ *Block.add(26 & 15) ^ *Block.add(20 & 15) ^ *Block.add(18 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(18 & 15) = blk_val;
    b = b.wrapping_add(
        (a ^ (d & (e ^ a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 19
    let xor_val = *Block.add(32 & 15) ^ *Block.add(27 & 15) ^ *Block.add(21 & 15) ^ *Block.add(19 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(19 & 15) = blk_val;
    a = a.wrapping_add(
        (e ^ (c & (d ^ e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x5a827999u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 20
    let xor_val = *Block.add(33 & 15) ^ *Block.add(28 & 15) ^ *Block.add(22 & 15) ^ *Block.add(20 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(20 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 21
    let xor_val = *Block.add(34 & 15) ^ *Block.add(29 & 15) ^ *Block.add(23 & 15) ^ *Block.add(21 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(21 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 22
    let xor_val = *Block.add(35 & 15) ^ *Block.add(30 & 15) ^ *Block.add(24 & 15) ^ *Block.add(22 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(22 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 23
    let xor_val = *Block.add(36 & 15) ^ *Block.add(31 & 15) ^ *Block.add(25 & 15) ^ *Block.add(23 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(23 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 24
    let xor_val = *Block.add(37 & 15) ^ *Block.add(32 & 15) ^ *Block.add(26 & 15) ^ *Block.add(24 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(24 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 25
    let xor_val = *Block.add(38 & 15) ^ *Block.add(33 & 15) ^ *Block.add(27 & 15) ^ *Block.add(25 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(25 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 26
    let xor_val = *Block.add(39 & 15) ^ *Block.add(34 & 15) ^ *Block.add(28 & 15) ^ *Block.add(26 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(26 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 27
    let xor_val = *Block.add(40 & 15) ^ *Block.add(35 & 15) ^ *Block.add(29 & 15) ^ *Block.add(27 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(27 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 28
    let xor_val = *Block.add(41 & 15) ^ *Block.add(36 & 15) ^ *Block.add(30 & 15) ^ *Block.add(28 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(28 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 29
    let xor_val = *Block.add(42 & 15) ^ *Block.add(37 & 15) ^ *Block.add(31 & 15) ^ *Block.add(29 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(29 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 30
    let xor_val = *Block.add(43 & 15) ^ *Block.add(38 & 15) ^ *Block.add(32 & 15) ^ *Block.add(30 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(30 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 31
    let xor_val = *Block.add(44 & 15) ^ *Block.add(39 & 15) ^ *Block.add(33 & 15) ^ *Block.add(31 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(31 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 32
    let xor_val = *Block.add(45 & 15) ^ *Block.add(40 & 15) ^ *Block.add(34 & 15) ^ *Block.add(32 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(32 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 33
    let xor_val = *Block.add(46 & 15) ^ *Block.add(41 & 15) ^ *Block.add(35 & 15) ^ *Block.add(33 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(33 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 34
    let xor_val = *Block.add(47 & 15) ^ *Block.add(42 & 15) ^ *Block.add(36 & 15) ^ *Block.add(34 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(34 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 35
    let xor_val = *Block.add(48 & 15) ^ *Block.add(43 & 15) ^ *Block.add(37 & 15) ^ *Block.add(35 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(35 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 36
    let xor_val = *Block.add(49 & 15) ^ *Block.add(44 & 15) ^ *Block.add(38 & 15) ^ *Block.add(36 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(36 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 37
    let xor_val = *Block.add(50 & 15) ^ *Block.add(45 & 15) ^ *Block.add(39 & 15) ^ *Block.add(37 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(37 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 38
    let xor_val = *Block.add(51 & 15) ^ *Block.add(46 & 15) ^ *Block.add(40 & 15) ^ *Block.add(38 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(38 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 39
    let xor_val = *Block.add(52 & 15) ^ *Block.add(47 & 15) ^ *Block.add(41 & 15) ^ *Block.add(39 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(39 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0x6ed9eba1u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 40
    let xor_val = *Block.add(53 & 15) ^ *Block.add(48 & 15) ^ *Block.add(42 & 15) ^ *Block.add(40 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(40 & 15) = blk_val;
    e = e.wrapping_add(
        ((b & c) | (d & (b | c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 41
    let xor_val = *Block.add(54 & 15) ^ *Block.add(49 & 15) ^ *Block.add(43 & 15) ^ *Block.add(41 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(41 & 15) = blk_val;
    d = d.wrapping_add(
        ((a & b) | (c & (a | b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 42
    let xor_val = *Block.add(55 & 15) ^ *Block.add(50 & 15) ^ *Block.add(44 & 15) ^ *Block.add(42 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(42 & 15) = blk_val;
    c = c.wrapping_add(
        ((e & a) | (b & (e | a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 43
    let xor_val = *Block.add(56 & 15) ^ *Block.add(51 & 15) ^ *Block.add(45 & 15) ^ *Block.add(43 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(43 & 15) = blk_val;
    b = b.wrapping_add(
        ((d & e) | (a & (d | e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 44
    let xor_val = *Block.add(57 & 15) ^ *Block.add(52 & 15) ^ *Block.add(46 & 15) ^ *Block.add(44 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(44 & 15) = blk_val;
    a = a.wrapping_add(
        ((c & d) | (e & (c | d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 45
    let xor_val = *Block.add(58 & 15) ^ *Block.add(53 & 15) ^ *Block.add(47 & 15) ^ *Block.add(45 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(45 & 15) = blk_val;
    e = e.wrapping_add(
        ((b & c) | (d & (b | c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 46
    let xor_val = *Block.add(59 & 15) ^ *Block.add(54 & 15) ^ *Block.add(48 & 15) ^ *Block.add(46 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(46 & 15) = blk_val;
    d = d.wrapping_add(
        ((a & b) | (c & (a | b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 47
    let xor_val = *Block.add(60 & 15) ^ *Block.add(55 & 15) ^ *Block.add(49 & 15) ^ *Block.add(47 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(47 & 15) = blk_val;
    c = c.wrapping_add(
        ((e & a) | (b & (e | a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 48
    let xor_val = *Block.add(61 & 15) ^ *Block.add(56 & 15) ^ *Block.add(50 & 15) ^ *Block.add(48 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(48 & 15) = blk_val;
    b = b.wrapping_add(
        ((d & e) | (a & (d | e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 49
    let xor_val = *Block.add(62 & 15) ^ *Block.add(57 & 15) ^ *Block.add(51 & 15) ^ *Block.add(49 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(49 & 15) = blk_val;
    a = a.wrapping_add(
        ((c & d) | (e & (c | d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 50
    let xor_val = *Block.add(63 & 15) ^ *Block.add(58 & 15) ^ *Block.add(52 & 15) ^ *Block.add(50 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(50 & 15) = blk_val;
    e = e.wrapping_add(
        ((b & c) | (d & (b | c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 51
    let xor_val = *Block.add(64 & 15) ^ *Block.add(59 & 15) ^ *Block.add(53 & 15) ^ *Block.add(51 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(51 & 15) = blk_val;
    d = d.wrapping_add(
        ((a & b) | (c & (a | b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 52
    let xor_val = *Block.add(65 & 15) ^ *Block.add(60 & 15) ^ *Block.add(54 & 15) ^ *Block.add(52 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(52 & 15) = blk_val;
    c = c.wrapping_add(
        ((e & a) | (b & (e | a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 53
    let xor_val = *Block.add(66 & 15) ^ *Block.add(61 & 15) ^ *Block.add(55 & 15) ^ *Block.add(53 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(53 & 15) = blk_val;
    b = b.wrapping_add(
        ((d & e) | (a & (d | e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 54
    let xor_val = *Block.add(67 & 15) ^ *Block.add(62 & 15) ^ *Block.add(56 & 15) ^ *Block.add(54 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(54 & 15) = blk_val;
    a = a.wrapping_add(
        ((c & d) | (e & (c | d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 55
    let xor_val = *Block.add(68 & 15) ^ *Block.add(63 & 15) ^ *Block.add(57 & 15) ^ *Block.add(55 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(55 & 15) = blk_val;
    e = e.wrapping_add(
        ((b & c) | (d & (b | c)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 56
    let xor_val = *Block.add(69 & 15) ^ *Block.add(64 & 15) ^ *Block.add(58 & 15) ^ *Block.add(56 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(56 & 15) = blk_val;
    d = d.wrapping_add(
        ((a & b) | (c & (a | b)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 57
    let xor_val = *Block.add(70 & 15) ^ *Block.add(65 & 15) ^ *Block.add(59 & 15) ^ *Block.add(57 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(57 & 15) = blk_val;
    c = c.wrapping_add(
        ((e & a) | (b & (e | a)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 58
    let xor_val = *Block.add(71 & 15) ^ *Block.add(66 & 15) ^ *Block.add(60 & 15) ^ *Block.add(58 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(58 & 15) = blk_val;
    b = b.wrapping_add(
        ((d & e) | (a & (d | e)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 59
    let xor_val = *Block.add(72 & 15) ^ *Block.add(67 & 15) ^ *Block.add(61 & 15) ^ *Block.add(59 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(59 & 15) = blk_val;
    a = a.wrapping_add(
        ((c & d) | (e & (c | d)))
            .wrapping_add(blk_val)
            .wrapping_add(0x8f1bbcdcu32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 60
    let xor_val = *Block.add(73 & 15) ^ *Block.add(68 & 15) ^ *Block.add(62 & 15) ^ *Block.add(60 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(60 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 61
    let xor_val = *Block.add(74 & 15) ^ *Block.add(69 & 15) ^ *Block.add(63 & 15) ^ *Block.add(61 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(61 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 62
    let xor_val = *Block.add(75 & 15) ^ *Block.add(70 & 15) ^ *Block.add(64 & 15) ^ *Block.add(62 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(62 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 63
    let xor_val = *Block.add(76 & 15) ^ *Block.add(71 & 15) ^ *Block.add(65 & 15) ^ *Block.add(63 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(63 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 64
    let xor_val = *Block.add(77 & 15) ^ *Block.add(72 & 15) ^ *Block.add(66 & 15) ^ *Block.add(64 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(64 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 65
    let xor_val = *Block.add(78 & 15) ^ *Block.add(73 & 15) ^ *Block.add(67 & 15) ^ *Block.add(65 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(65 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 66
    let xor_val = *Block.add(79 & 15) ^ *Block.add(74 & 15) ^ *Block.add(68 & 15) ^ *Block.add(66 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(66 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 67
    let xor_val = *Block.add(80 & 15) ^ *Block.add(75 & 15) ^ *Block.add(69 & 15) ^ *Block.add(67 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(67 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 68
    let xor_val = *Block.add(81 & 15) ^ *Block.add(76 & 15) ^ *Block.add(70 & 15) ^ *Block.add(68 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(68 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 69
    let xor_val = *Block.add(82 & 15) ^ *Block.add(77 & 15) ^ *Block.add(71 & 15) ^ *Block.add(69 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(69 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 70
    let xor_val = *Block.add(83 & 15) ^ *Block.add(78 & 15) ^ *Block.add(72 & 15) ^ *Block.add(70 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(70 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 71
    let xor_val = *Block.add(84 & 15) ^ *Block.add(79 & 15) ^ *Block.add(73 & 15) ^ *Block.add(71 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(71 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 72
    let xor_val = *Block.add(85 & 15) ^ *Block.add(80 & 15) ^ *Block.add(74 & 15) ^ *Block.add(72 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(72 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 73
    let xor_val = *Block.add(86 & 15) ^ *Block.add(81 & 15) ^ *Block.add(75 & 15) ^ *Block.add(73 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(73 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 74
    let xor_val = *Block.add(87 & 15) ^ *Block.add(82 & 15) ^ *Block.add(76 & 15) ^ *Block.add(74 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(74 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Index 75
    let xor_val = *Block.add(88 & 15) ^ *Block.add(83 & 15) ^ *Block.add(77 & 15) ^ *Block.add(75 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(75 & 15) = blk_val;
    e = e.wrapping_add(
        (b ^ c ^ d)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((a << 5) | (a >> 27)),
    );
    b = (b << 30) | (b >> 2);

    // Index 76
    let xor_val = *Block.add(89 & 15) ^ *Block.add(84 & 15) ^ *Block.add(78 & 15) ^ *Block.add(76 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(76 & 15) = blk_val;
    d = d.wrapping_add(
        (a ^ b ^ c)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((e << 5) | (e >> 27)),
    );
    a = (a << 30) | (a >> 2);

    // Index 77
    let xor_val = *Block.add(90 & 15) ^ *Block.add(85 & 15) ^ *Block.add(79 & 15) ^ *Block.add(77 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(77 & 15) = blk_val;
    c = c.wrapping_add(
        (e ^ a ^ b)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((d << 5) | (d >> 27)),
    );
    e = (e << 30) | (e >> 2);

    // Index 78
    let xor_val = *Block.add(91 & 15) ^ *Block.add(86 & 15) ^ *Block.add(80 & 15) ^ *Block.add(78 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(78 & 15) = blk_val;
    b = b.wrapping_add(
        (d ^ e ^ a)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((c << 5) | (c >> 27)),
    );
    d = (d << 30) | (d >> 2);

    // Index 79
    let xor_val = *Block.add(92 & 15) ^ *Block.add(87 & 15) ^ *Block.add(81 & 15) ^ *Block.add(79 & 15);
    let blk_val = (xor_val << 1) | (xor_val >> 31);
    *Block.add(79 & 15) = blk_val;
    a = a.wrapping_add(
        (c ^ d ^ e)
            .wrapping_add(blk_val)
            .wrapping_add(0xca62c1d6u32)
            .wrapping_add((b << 5) | (b >> 27)),
    );
    c = (c << 30) | (c >> 2);

    // Add back to state
    *State.add(0) = (*State.add(0)).wrapping_add(a);
    *State.add(1) = (*State.add(1)).wrapping_add(b);
    *State.add(2) = (*State.add(2)).wrapping_add(c);
    *State.add(3) = (*State.add(3)).wrapping_add(d);
    *State.add(4) = (*State.add(4)).wrapping_add(e);

    // Wipe variables
    a = 0;
    b = 0;
    c = 0;
    d = 0;
    e = 0;
    let _ = (a, b, c, d, e);
}

#[no_mangle]
pub unsafe extern "system" fn A_SHAInit(Context: *mut SHA_CTX) {
    (*Context).State[0] = 0x67452301u32;
    (*Context).State[1] = 0xEFCDAB89u32;
    (*Context).State[2] = 0x98BADCFEu32;
    (*Context).State[3] = 0x10325476u32;
    (*Context).State[4] = 0xC3D2E1F0u32;
    (*Context).Count[1] = 0u32;
    (*Context).Count[0] = (*Context).Count[1];
}

#[no_mangle]
pub unsafe extern "system" fn A_SHAUpdate(
    Context: *mut SHA_CTX,
    mut Buffer: *const u8,
    mut BufferSize: u32,
) {
    // BLOCKER FIX: All three of BufferContentSize, Buffer, BufferSize were missing
    // `let mut` in the assembled translation — without mutation the while-loop never
    // terminates. Buffer and BufferSize are fn params (already mut above).
    // BufferContentSize must also be `let mut`.
    let mut BufferContentSize = ((*Context).Count[1] & 63) as u32;
    (*Context).Count[1] = (*Context).Count[1].wrapping_add(BufferSize);
    if (*Context).Count[1] < BufferSize {
        (*Context).Count[0] = (*Context).Count[0].wrapping_add(1);
    }
    (*Context).Count[0] = (*Context).Count[0].wrapping_add(BufferSize >> 29);

    if BufferContentSize.wrapping_add(BufferSize) < 64 {
        core::ptr::copy_nonoverlapping(
            Buffer,
            (&raw mut (*Context).Buffer as *mut u8).add(BufferContentSize as usize),
            BufferSize as usize,
        );
    } else {
        while BufferContentSize.wrapping_add(BufferSize) >= 64 {
            core::ptr::copy_nonoverlapping(
                Buffer,
                (&raw mut (*Context).Buffer as *mut u8).add(BufferContentSize as usize),
                (64u32.wrapping_sub(BufferContentSize)) as usize,
            );
            Buffer = Buffer.add((64u32.wrapping_sub(BufferContentSize)) as usize);
            BufferSize = BufferSize.wrapping_sub(64u32.wrapping_sub(BufferContentSize));
            SHA1Transform(
                &raw mut (*Context).State as *mut u32,
                &raw mut (*Context).Buffer as *mut u8,
            );
            BufferContentSize = 0;
        }
        core::ptr::copy_nonoverlapping(
            Buffer,
            (&raw mut (*Context).Buffer as *mut u8).add(BufferContentSize as usize),
            BufferSize as usize,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn A_SHAFinal(Context: *mut SHA_CTX, Result: *mut u32) {
    let mut Pad: i32;
    let mut Index: i32;
    let mut Buffer: [u8; 72] = [0u8; 72];
    let mut Count: *mut u32;
    let mut BufferContentSize: u32;
    let mut LengthHi: u32;
    let mut LengthLo: u32;

    BufferContentSize = (*Context).Count[1] & 63u32;
    if BufferContentSize >= 56u32 {
        Pad = (56u32 + 64u32 - BufferContentSize) as i32;
    } else {
        Pad = (56u32 - BufferContentSize) as i32;
    }

    LengthHi = (*Context).Count[0] << 3 | (*Context).Count[1] >> (32 - 3);
    LengthLo = (*Context).Count[1] << 3;

    core::ptr::write_bytes(
        Buffer.as_mut_ptr().add(1) as *mut u8,
        0u8,
        (Pad - 1) as usize,
    );

    Buffer[0] = 0x80u8;
    // `Buffer + Pad` is misaligned whenever the byte count is not a multiple of 4. The C
    // (`Count = (ULONG*)(Buffer + Pad)`) is UB there too — UBSan reports it at sha1.c:190 —
    // and in Rust a misaligned `*mut u32` store is UB that LLVM may exploit, so store
    // unaligned: the bytes written are identical.
    Count = Buffer.as_mut_ptr().add(Pad as usize) as *mut u32;
    Count.offset(0).write_unaligned(
        (LengthHi >> 24) & 0xff
            | (LengthHi >> 8) & 0xff00
            | (LengthHi << 8) & 0xff0000
            | (LengthHi << 24) & 0xff000000,
    );
    Count.offset(1).write_unaligned(
        (LengthLo >> 24) & 0xff
            | (LengthLo >> 8) & 0xff00
            | (LengthLo << 8) & 0xff0000
            | (LengthLo << 24) & 0xff000000,
    );

    A_SHAUpdate(Context, Buffer.as_ptr(), (Pad + 8) as u32);

    Index = 0;
    while Index < 5 {
        *Result.offset(Index as isize) = (*Context).State[Index as usize] >> 24 & 0xff
            | (*Context).State[Index as usize] >> 8 & 0xff00
            | (*Context).State[Index as usize] << 8 & 0xff0000
            | (*Context).State[Index as usize] << 24 & 0xff000000;
        Index += 1;
    }

    core::ptr::write_bytes(
        (*Context).Buffer.as_mut_ptr() as *mut u8,
        0u8,
        64usize,
    );

    A_SHAInit(Context);
}

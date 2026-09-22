//! Bit rotates and byte swaps — `sdk/lib/ucrt/stdlib/{rotl,rotr,byteswap}.cpp`.
//!
//! The first UCRT family taken over, chosen because it is the one with no
//! semantic delta to argue about: these are total functions of their arguments,
//! with no locale, no errno, no stream state and no `_MSVCR_VER` history. What
//! they prove is the PIPELINE — crate to archive to link to gate to burn-down —
//! not any interesting behaviour.
//!
//! They also delete more C than they replace. clang treats `_rotl` and friends
//! as builtins, so the Microsoft C could not define them under our toolchain:
//! `sdk/lib/ucrt/CMakeLists.txt` renamed each one (`-D_rotl=___rotl`) and
//! `stdlib/clang-hacks.s` then created the real symbol as an assembly `jmp` to
//! the renamed body. Rust has no builtin to collide with, so all three .cpp and
//! the whole .s workaround go away together.
//!
//! ABI notes, both of which are silent if wrong:
//!
//! * The names are written exactly as C spells them. rustc applies the target's
//!   own decoration to a `#[no_mangle] extern "C"` symbol, so `_rotl` emits
//!   `__rotl` on i386 and `_rotl` on amd64/arm64 — the same thing the C
//!   compiler does, which is why the .spec's `_rotl` resolves to it on every
//!   arch. Writing `__rotl` here to "pre-decorate" would export `___rotl`.
//!
//! * Windows is LLP64: `unsigned long` is 32 bits on all three of our arches,
//!   so `_lrotl`/`_lrotr`/`_byteswap_ulong` are u32 everywhere, NOT usize.
//!
//! The shift count is masked rather than passed through. C's `_rotl(v, 32)` is
//! `v`, and Rust's `rotate_left` already reduces modulo the width, but the mask
//! is written out because it is also what makes a NEGATIVE shift agree with the
//! C: `shift as u32 & 31` reproduces two's-complement `shift & 31` exactly.

/// `unsigned int _rotl(unsigned int value, int shift)`
#[no_mangle]
pub extern "C" fn _rotl(value: u32, shift: i32) -> u32 {
    value.rotate_left(shift as u32 & 31)
}

/// `unsigned int _rotr(unsigned int value, int shift)`
#[no_mangle]
pub extern "C" fn _rotr(value: u32, shift: i32) -> u32 {
    value.rotate_right(shift as u32 & 31)
}

/// `unsigned long _lrotl(unsigned long value, int shift)` — 32-bit, LLP64.
#[no_mangle]
pub extern "C" fn _lrotl(value: u32, shift: i32) -> u32 {
    value.rotate_left(shift as u32 & 31)
}

/// `unsigned long _lrotr(unsigned long value, int shift)` — 32-bit, LLP64.
#[no_mangle]
pub extern "C" fn _lrotr(value: u32, shift: i32) -> u32 {
    value.rotate_right(shift as u32 & 31)
}

/// `unsigned __int64 _rotl64(unsigned __int64 value, int shift)`
#[no_mangle]
pub extern "C" fn _rotl64(value: u64, shift: i32) -> u64 {
    value.rotate_left(shift as u32 & 63)
}

/// `unsigned __int64 _rotr64(unsigned __int64 value, int shift)`
#[no_mangle]
pub extern "C" fn _rotr64(value: u64, shift: i32) -> u64 {
    value.rotate_right(shift as u32 & 63)
}

/// `unsigned short _byteswap_ushort(unsigned short)`
#[no_mangle]
pub extern "C" fn _byteswap_ushort(value: u16) -> u16 {
    value.swap_bytes()
}

/// `unsigned long _byteswap_ulong(unsigned long)` — 32-bit, LLP64.
#[no_mangle]
pub extern "C" fn _byteswap_ulong(value: u32) -> u32 {
    value.swap_bytes()
}

/// `unsigned __int64 _byteswap_uint64(unsigned __int64)`
#[no_mangle]
pub extern "C" fn _byteswap_uint64(value: u64) -> u64 {
    value.swap_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_agree_with_the_c_contract() {
        assert_eq!(_rotl(0x1234_5678, 8), 0x3456_7812);
        assert_eq!(_rotr(0x1234_5678, 8), 0x7812_3456);
        assert_eq!(_rotl64(0x0123_4567_89ab_cdef, 16), 0x4567_89ab_cdef_0123);
        assert_eq!(_rotr64(0x0123_4567_89ab_cdef, 16), 0xcdef_0123_4567_89ab);
    }

    #[test]
    fn a_shift_of_a_whole_width_is_the_identity() {
        // C reduces modulo the width; a raw `<<` would be UB here and a raw
        // Rust shift would panic in debug.
        assert_eq!(_rotl(0xdead_beef, 32), 0xdead_beef);
        assert_eq!(_rotr(0xdead_beef, 32), 0xdead_beef);
        assert_eq!(_rotl64(0xdead_beef_feed_face, 64), 0xdead_beef_feed_face);
    }

    #[test]
    fn a_negative_shift_masks_the_way_two_s_complement_c_does() {
        // C: `shift & 31` on int -1 is 31. This is the case the mask exists for.
        assert_eq!(_rotl(1, -1), 1u32.rotate_left(31));
        assert_eq!(_rotr(1, -1), 1u32.rotate_right(31));
    }

    #[test]
    fn byteswaps() {
        assert_eq!(_byteswap_ushort(0x1234), 0x3412);
        assert_eq!(_byteswap_ulong(0x1234_5678), 0x7856_3412);
        assert_eq!(_byteswap_uint64(0x0123_4567_89ab_cdef), 0xefcd_ab89_6745_2301);
    }
}

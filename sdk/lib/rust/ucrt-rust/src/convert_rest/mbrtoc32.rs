//! `convert/mbrtoc32.cpp`: `mbrtoc32` and its UTF-8 engine
//! `__crt_mbstring::__mbrtoc32_utf8`, which every other UTF-8 conversion in
//! this cluster (`mbrtoc16`, `c16rtomb`/`c32rtomb` via `mbrtowc`/`wcrtomb`,
//! `mbrtowc`/`mbsrtowcs`) is built on top of.
//!
//! Quirks kept faithfully:
//!   * ASCII (`first_byte & 0x80 == 0`) is fast-pathed only when the state is
//!     already initial -- a stray ASCII byte arriving mid-sequence (state
//!     nonzero) is NOT special-cased and falls into the "otherwise" arm,
//!     which validates `length`/`bytes_needed` from the stored state and
//!     rejects it as a bad continuation byte.
//!   * Overlong encodings and surrogate code points (`0xd800..=0xdfff`) are
//!     both `EILSEQ`, checked AFTER the byte count is fully consumed (so a
//!     short/incomplete overlong sequence reports `INCOMPLETE`, not
//!     `INVALID`, until enough bytes exist to tell).
//!   * `c32 == U'\0'` returns 0, not `bytes_needed` -- a NUL is reported as
//!     end-of-string even when spelled as a (technically invalid, but this
//!     function doesn't reject it) multi-byte overlong encoding of 0.
//!   * A `ps == null` caller gets a function-local `static` state, exactly
//!     the C's non-restartable behaviour: state leaks across unrelated calls
//!     that both pass null, which is why `mbrtoc32.cpp`'s `mbrtoc32()` and
//!     this cluster's other public entry points do NOT special-case this --
//!     it is Microsoft's documented (mis)behaviour for a null `mbstate_t*`.

use super::common::{
    reset_and_return, return_illegal_sequence, MbstateT, INCOMPLETE,
};
use crate::ptd_host::PtdHost;
use core::ffi::c_char;

/// `__crt_mbstring::__mbrtoc32_utf8`.
///
/// # Safety
/// `s`, if non-null, must be readable for at least `n` bytes (or up to the
/// first NUL); `ps` and `pc32`, if non-null, must be valid for read/write.
pub unsafe fn mbrtoc32_utf8(
    pc32: *mut u32,
    s: *const c_char,
    n: usize,
    ps: *mut MbstateT,
    ptd: &mut PtdHost,
) -> usize {
    static mut INTERNAL_PST: MbstateT = MbstateT::zeroed();
    let mut s = s;
    let mut n = n;
    let mut pc32 = pc32;
    let ps = if ps.is_null() { core::ptr::addr_of_mut!(INTERNAL_PST) } else { ps };

    if s.is_null() {
        // A static empty string, as the C does with `s = ""`.
        static EMPTY: c_char = 0;
        s = core::ptr::addr_of!(EMPTY);
        n = 1;
        pc32 = core::ptr::null_mut();
    }

    if n == 0 {
        return INCOMPLETE;
    }

    let length: u8;
    let bytes_needed: u8;
    let mut c32: u32;
    // `s - begin`, tracked as a plain counter instead of pointer subtraction:
    // 1 after consuming the lead byte (`init_state`), 0 when resuming from a
    // stored partial sequence (the C's `begin == s` on entry in that arm).
    let mut consumed_since_begin: usize;

    if (*ps).state == 0 {
        let first_byte = *s as u8;
        s = s.add(1);
        consumed_since_begin = 1;

        if first_byte & 0x80 == 0 {
            if !pc32.is_null() {
                *pc32 = first_byte as u32;
            }
            return if first_byte != 0 { 1 } else { 0 };
        }

        if first_byte & 0xe0 == 0xc0 {
            length = 2;
        } else if first_byte & 0xf0 == 0xe0 {
            length = 3;
        } else if first_byte & 0xf8 == 0xf0 {
            length = 4;
        } else {
            return return_illegal_sequence(ps, ptd);
        }
        bytes_needed = length;
        // Mask out the length bits: `first_byte & ((1 << (7 - length)) - 1)`.
        c32 = (first_byte & (((1u32 << (7 - length as u32)) - 1) as u8)) as u32;
    } else {
        c32 = (*ps).wchar;
        length = (*ps).byte as u8;
        bytes_needed = (*ps).state as u8;
        consumed_since_begin = 0;

        if length < 2 || length > 4 || bytes_needed < 1 || bytes_needed >= length {
            return return_illegal_sequence(ps, ptd);
        }
    }

    if (bytes_needed as usize) < n {
        n = bytes_needed as usize;
    }

    while consumed_since_begin < n {
        let current_byte = *s as u8;
        s = s.add(1);
        consumed_since_begin += 1;
        if current_byte & 0xc0 != 0x80 {
            return return_illegal_sequence(ps, ptd);
        }
        c32 = (c32 << 6) | (current_byte & 0x3f) as u32;
    }

    if n < bytes_needed as usize {
        let bytes_remaining = bytes_needed - n as u8;
        (*ps).wchar = c32;
        (*ps).byte = length as u16;
        (*ps).state = bytes_remaining as u16;
        return INCOMPLETE;
    }

    if (0xd800..=0xdfff).contains(&c32) || c32 > 0x10ffff {
        return return_illegal_sequence(ps, ptd);
    }

    const MIN_LEGAL: [u32; 3] = [0x80, 0x800, 0x10000];
    if c32 < MIN_LEGAL[(length - 2) as usize] {
        return return_illegal_sequence(ps, ptd);
    }

    if !pc32.is_null() {
        *pc32 = c32;
    }

    reset_and_return(if c32 == 0 { 0 } else { bytes_needed as usize }, ps)
}

/// `mbrtoc32` -- always assumes UTF-8 (`// TODO: Bug 13307590`, kept as the
/// C states it; not our bug to fix here).
#[no_mangle]
pub unsafe extern "C" fn mbrtoc32(
    pc32: *mut u32,
    s: *const c_char,
    n: usize,
    ps: *mut MbstateT,
) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    mbrtoc32_utf8(pc32, s, n, ps, &mut ptd)
}

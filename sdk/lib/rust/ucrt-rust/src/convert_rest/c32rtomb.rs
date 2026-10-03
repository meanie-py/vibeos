//! `convert/c32rtomb.cpp`: `c32rtomb` and `__crt_mbstring::__c32rtomb_utf8`,
//! which `wctomb`/`wcrtomb`'s UTF-8 path (via `c16rtomb`'s single-non-surrogate
//! arm) and `c16rtomb` itself both call into.

use super::common::{reset_and_return, return_illegal_sequence, MbstateT};
use crate::ptd_host::PtdHost;
use core::ffi::c_char;

/// `__crt_mbstring::__c32rtomb_utf8`.
///
/// # Safety
/// `s`, if non-null, must have room for at least 4 bytes; `ps` must be valid
/// for read/write.
pub unsafe fn c32rtomb_utf8(s: *mut c_char, c32: u32, ps: *mut MbstateT, ptd: &mut PtdHost) -> usize {
    if s.is_null() {
        // Equivalent to c32rtomb(buf, U'\0', ps) for some internal buffer buf.
        *ps = MbstateT::zeroed();
        return 1;
    }

    if c32 == 0 {
        *s = 0;
        *ps = MbstateT::zeroed();
        return 1;
    }

    // Fast path for ASCII.
    if c32 & !0x7f == 0 {
        *s = c32 as c_char;
        return 1;
    }

    let trail_bytes: usize;
    let lead_byte: u8;
    if c32 & !0x7ff == 0 {
        trail_bytes = 1;
        lead_byte = 0xc0;
    } else if c32 & !0xffff == 0 {
        if (0xd800..=0xdfff).contains(&c32) {
            return return_illegal_sequence(ps, ptd);
        }
        trail_bytes = 2;
        lead_byte = 0xe0;
    } else if c32 & !0x001f_ffff == 0 {
        if c32 > 0x10ffff {
            return return_illegal_sequence(ps, ptd);
        }
        trail_bytes = 3;
        lead_byte = 0xf0;
    } else {
        return return_illegal_sequence(ps, ptd);
    }

    // Put six bits into each of the trail bytes, filling back to front.
    let mut c32 = c32;
    for i in (1..=trail_bytes).rev() {
        *s.add(i) = ((c32 & 0x3f) | 0x80) as c_char;
        c32 >>= 6;
    }

    *s = (c32 as u8 | lead_byte) as c_char;

    reset_and_return(trail_bytes + 1, ps)
}

/// `c32rtomb` -- always assumes UTF-8 (`// TODO: Bug 13307590`).
#[no_mangle]
pub unsafe extern "C" fn c32rtomb(s: *mut c_char, c32: u32, ps: *mut MbstateT) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    c32rtomb_utf8(s, c32, ps, &mut ptd)
}

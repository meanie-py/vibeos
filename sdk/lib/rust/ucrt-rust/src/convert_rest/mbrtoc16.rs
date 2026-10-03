//! `convert/mbrtoc16.cpp`: `mbrtoc16` and `__crt_mbstring::__mbrtoc16_utf8`.
//!
//! Surrogate-pair state is stashed in `_State == (unsigned short)-1` (a
//! sentinel distinct from every real `bytes_needed` value `mbrtoc32_utf8`
//! ever stores, since that is always `1..=3`), with the pending low
//! surrogate's code point kept in `_Wchar`. Kept exactly as the C spells it.

use super::common::{reset_and_return, return_illegal_sequence, MbstateT, INCOMPLETE, INVALID};
use super::mbrtoc32::mbrtoc32_utf8;
use crate::ptd_host::PtdHost;
use core::ffi::c_char;

const SURROGATE_STATE: u16 = u16::MAX;

unsafe fn begin_surrogate_state(pc16: *mut u16, c32: u32, retval: usize, ps: *mut MbstateT) -> usize {
    (*ps).wchar = c32;
    (*ps).state = SURROGATE_STATE;
    if !pc16.is_null() {
        *pc16 = ((((c32 - 0x10000) & 0xfffff) >> 10) | 0xd800) as u16;
    }
    retval
}

unsafe fn end_surrogate_state(pc16: *mut u16, ps: *mut MbstateT) -> usize {
    if !pc16.is_null() {
        *pc16 = (((*ps).wchar.wrapping_sub(0x10000)) & 0x03ff | 0xdc00) as u16;
    }
    reset_and_return(usize::MAX - 2, ps) // `static_cast<size_t>(-3)`
}

unsafe fn is_surrogate_state(ps: *const MbstateT) -> bool {
    (*ps).state == SURROGATE_STATE
}

/// `__crt_mbstring::__mbrtoc16_utf8`.
///
/// # Safety
/// Same contract as `mbrtoc32_utf8`.
pub unsafe fn mbrtoc16_utf8(
    pc16: *mut u16,
    s: *const c_char,
    n: usize,
    ps: *mut MbstateT,
    ptd: &mut PtdHost,
) -> usize {
    static mut INTERNAL_PST: MbstateT = MbstateT::zeroed();
    let ps = if ps.is_null() { core::ptr::addr_of_mut!(INTERNAL_PST) } else { ps };

    if is_surrogate_state(ps) {
        return end_surrogate_state(pc16, ps);
    }

    let mut c32: u32 = 0;
    let retval = mbrtoc32_utf8(&mut c32, s, n, ps, ptd);
    if s.is_null() || retval == INVALID || retval == INCOMPLETE {
        return retval;
    } else if c32 > 0x10ffff {
        return return_illegal_sequence(ps, ptd);
    }

    if c32 <= 0xffff {
        if !pc16.is_null() {
            *pc16 = c32 as u16;
        }
        reset_and_return(retval, ps)
    } else {
        begin_surrogate_state(pc16, c32, retval, ps)
    }
}

/// `mbrtoc16` -- always assumes UTF-8 (`// TODO: Bug 13307590`).
#[no_mangle]
pub unsafe extern "C" fn mbrtoc16(
    pc16: *mut u16,
    s: *const c_char,
    n: usize,
    ps: *mut MbstateT,
) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    mbrtoc16_utf8(pc16, s, n, ps, &mut ptd)
}

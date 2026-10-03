//! `convert/c16rtomb.cpp`: `c16rtomb` and `__crt_mbstring::__c16rtomb_utf8`,
//! the per-UTF-16-unit engine `wcrtomb.cpp`'s `__wcsrtombs_utf8` drives one
//! wide character at a time.

use super::c32rtomb::c32rtomb_utf8;
use super::common::{reset_and_return, return_illegal_sequence, MbstateT};
use crate::ptd_host::PtdHost;
use core::ffi::c_char;

/// Note: `_Wchar` is the only state field this function uses -- it stores the
/// decoded high surrogate's contribution, pre-shifted so completing it is
/// just `+ (low & !0xdc00)`.
unsafe fn store_first_surrogate(c16: u16, ps: *mut MbstateT) {
    let c32 = ((c16 as u32) & !0xd800u32) << 10;
    (*ps).wchar = c32.wrapping_add(0x10000);
}

unsafe fn combine_second_surrogate(c16: u16, ps: *mut MbstateT) -> u32 {
    (*ps).wchar.wrapping_add((c16 as u32) & !0xdc00u32)
}

fn has_surrogate(ps: *const MbstateT) -> bool {
    unsafe { (*ps).wchar != 0 }
}

fn is_first_surrogate(c16: u16) -> bool {
    (0xd800..=0xdbff).contains(&c16)
}

fn is_second_surrogate(c16: u16) -> bool {
    (0xdc00..=0xdfff).contains(&c16)
}

/// `__crt_mbstring::__c16rtomb_utf8`.
///
/// # Safety
/// `s`, if non-null, must have room for at least 4 bytes; `ps` must be valid
/// for read/write.
pub unsafe fn c16rtomb_utf8(s: *mut c_char, c16: u16, ps: *mut MbstateT, ptd: &mut PtdHost) -> usize {
    static mut INTERNAL_PST: MbstateT = MbstateT::zeroed();
    let ps = if ps.is_null() { core::ptr::addr_of_mut!(INTERNAL_PST) } else { ps };

    if !has_surrogate(ps) {
        if is_second_surrogate(c16) {
            return_illegal_sequence(ps, ptd)
        } else if is_first_surrogate(c16) {
            store_first_surrogate(c16, ps);
            0
        } else {
            c32rtomb_utf8(s, c16 as u32, ps, ptd)
        }
    } else {
        if !is_second_surrogate(c16) {
            return return_illegal_sequence(ps, ptd);
        }
        let c32 = combine_second_surrogate(c16, ps);
        let mut temp = MbstateT::zeroed();
        let retval = c32rtomb_utf8(s, c32, &mut temp, ptd);
        reset_and_return(retval, ps)
    }
}

/// `c16rtomb` -- always assumes UTF-8 (`// TODO: Bug 13307590`).
#[no_mangle]
pub unsafe extern "C" fn c16rtomb(s: *mut c_char, c16: u16, ps: *mut MbstateT) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    c16rtomb_utf8(s, c16, ps, &mut ptd)
}

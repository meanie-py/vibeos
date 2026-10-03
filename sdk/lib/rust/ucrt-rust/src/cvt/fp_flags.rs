//! `convert/fp_flags.cpp`: `__fastflag` and `__setfflag`. Neither is exported
//! (ucrtbase.spec names neither) and nothing in the tree reads the flag; kept
//! with its C names because the C defined them `extern "C"`.

use core::ffi::c_int;

/// `int __fastflag` -- "speed up transcendentals at the expense of error
/// checking". Zero-initialised, as the C's `{0}`.
#[no_mangle]
pub static mut __fastflag: c_int = 0;

/// `__setfflag`: store the new flag, return the old one.
///
/// # Safety
/// Unsynchronised, as the C.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __setfflag(new_flag: c_int) -> c_int {
    let old_flag = core::ptr::addr_of!(__fastflag).read();
    core::ptr::addr_of_mut!(__fastflag).write(new_flag);
    old_flag
}

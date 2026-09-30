//! `putch.cpp` and `putwch.cpp`: write one character to the console.
//!
//! `_putch` goes through `_putwch`: the byte is converted to UTF-16 in the
//! CRT's code page (not the console's), buffering a DBCS lead byte in the
//! per-thread `_putch_buffer` until its trail byte arrives. `_putwch` writes
//! one UTF-16 unit with `__dcrt_write_console`.

use core::ffi::c_int;

use super::initcon::{__dcrt_lowio_ensure_console_output_initialized, __dcrt_write_console};
use super::{with_conio_lock, EOF, WEOF};
use crate::ptd_host::PtdHost;

extern "C" {
    fn isleadbyte(c: c_int) -> c_int;
    fn _mbtowc_internal(dst: *mut u16, src: *const u8, src_size: usize, ptd: *mut PtdHost) -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn _putch(c: c_int) -> c_int {
    with_conio_lock(|| unsafe { _putch_nolock(c) })
}

/// Returns `c` on success (so a `'\xFF'` passed sign-extended reads back as
/// EOF), EOF on failure. A buffered lead byte is "success" with nothing
/// written.
#[no_mangle]
pub unsafe extern "C" fn _putch_nolock_internal(c: c_int, ptd: *mut PtdHost) -> c_int {
    let raw_ptd = (*ptd).raw_ptd();
    let ch_buf = core::ptr::addr_of_mut!((*raw_ptd).putch_buffer) as *mut u8;
    let pch_buf_used = core::ptr::addr_of_mut!((*raw_ptd).putch_buffer_used);

    let mut result = c;

    if *pch_buf_used == 1 {
        // (_ASSERTE(isleadbyte(ch_buf[0])) in the C.)
        *ch_buf.add(1) = c as u8;
    } else {
        *ch_buf = c as u8;
    }

    if *pch_buf_used == 0 && isleadbyte(*ch_buf as c_int) != 0 {
        // Wait for the trail byte.
        *pch_buf_used = 1;
    } else {
        let mut wchar: u16 = 0;
        if _mbtowc_internal(&mut wchar, ch_buf, *pch_buf_used as usize + 1, ptd) == -1 || _putwch_nolock(wchar) == WEOF
        {
            result = EOF;
        }

        // A whole character has been handled.
        *pch_buf_used = 0;
    }

    result
}

#[no_mangle]
pub unsafe extern "C" fn _putch_nolock(c: c_int) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    _putch_nolock_internal(c, &mut ptd)
}

#[no_mangle]
pub unsafe extern "C" fn _putwch(c: u16) -> u16 {
    with_conio_lock(|| unsafe { _putwch_nolock(c) })
}

#[no_mangle]
pub unsafe extern "C" fn _putwch_nolock(c: u16) -> u16 {
    if __dcrt_lowio_ensure_console_output_initialized() == 0 {
        return WEOF;
    }

    let mut chars_written: u32 = 0;
    if __dcrt_write_console((&c as *const u16).cast(), 1, &mut chars_written) == 0 {
        return WEOF;
    }

    c
}

/// The ptd is not needed; the parameter exists for the traits call shape.
#[no_mangle]
pub unsafe extern "C" fn _putwch_nolock_internal(c: u16, _ptd: *mut PtdHost) -> u16 {
    _putwch_nolock(c)
}

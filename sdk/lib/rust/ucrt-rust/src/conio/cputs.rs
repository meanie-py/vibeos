//! `cputs.cpp` and `cputws.cpp`: write a string straight to the console, no
//! newline appended. 0 on success, -1 on failure.

use core::ffi::{c_char, c_int};

use super::initcon::{__dcrt_lowio_ensure_console_output_initialized, __dcrt_write_console};
use super::putch::_putch_nolock;
use super::{invalid_clear_oserr, with_conio_lock, EINVAL, EOF};

extern "C" {
    fn wcslen(s: *const u16) -> usize;
}

/// Character by character through `_putch_nolock`, each `char` sign-extended
/// to `int` -- so a `'\xFF'` byte stops the write (see the module notes).
/// Never touches the console for an empty string.
#[no_mangle]
pub unsafe extern "C" fn _cputs(string: *const c_char) -> c_int {
    if string.is_null() {
        invalid_clear_oserr(EINVAL);
        return -1;
    }

    with_conio_lock(|| unsafe {
        let mut p = string;
        while *p != 0 {
            if _putch_nolock(*p as c_int) == EOF {
                return -1;
            }
            p = p.add(1);
        }
        0
    })
}

/// Checks the console BEFORE the string (so `L""` fails without one), then
/// writes in chunks of at most 32767 UTF-16 units.
#[no_mangle]
pub unsafe extern "C" fn _cputws(string: *const u16) -> c_int {
    if string.is_null() {
        invalid_clear_oserr(EINVAL);
        return -1;
    }

    if __dcrt_lowio_ensure_console_output_initialized() == 0 {
        return -1;
    }

    let mut string = string;
    let mut length = wcslen(string);

    with_conio_lock(|| unsafe {
        while length > 0 {
            const MAX_WRITE_BYTES: usize = 65535;
            const MAX_WRITE_WCHARS: usize = MAX_WRITE_BYTES / 2;

            let wchars_to_write = if length > MAX_WRITE_WCHARS { MAX_WRITE_WCHARS } else { length } as u32;

            let mut wchars_written: u32 = 0;
            if __dcrt_write_console(string.cast(), wchars_to_write, &mut wchars_written) == 0 {
                return -1;
            }

            string = string.add(wchars_to_write as usize);
            length -= wchars_to_write as usize;
        }
        0
    })
}

//! `cgets.cpp` and `cgetws.cpp`: read a line from the console in cooked mode.
//!
//! `_cgetws_s` reads with `ReadConsoleW` (line input + echo + processed) and
//! strips the trailing CR LF; `_cgets_s` calls it ONE wide character at a
//! time and converts each with `wctomb_s`, parking a character that does not
//! fit in `__console_wchar_buffer` for the next `_cgetws_s`. `_cgets`/
//! `_cgetws` are the legacy `buf[0] = max, buf[1] = got, text at buf+2`
//! shapes over them.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::addr_of_mut;

use super::initconin::{
    __dcrt_get_input_console_mode, __dcrt_lowio_ensure_console_input_initialized, __dcrt_read_console,
    __dcrt_set_input_console_mode,
};
use super::{invalid_clear_oserr, set_errno, with_conio_lock, GetLastError, _errno, __acrt_errno_map_os_error, EINVAL};

const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
const ENABLE_LINE_INPUT: u32 = 0x0002;
const ENABLE_ECHO_INPUT: u32 = 0x0004;

extern "C" {
    fn wctomb_s(size_converted: *mut c_int, mbch: *mut u8, size_in_bytes: usize, wch: u16) -> c_int;
}

/// A character `_cgets_s` could not store, handed to the next `_cgetws_s`.
#[no_mangle]
pub static mut __console_wchar_buffer: u16 = 0;
/// Set by `_cgets_s` when it parks a character; never cleared by anyone.
#[no_mangle]
pub static mut __console_wchar_buffer_used: c_int = 0;

// ── cgetws.cpp ───────────────────────────────────────────────────────────────

/// The locked body of `_cgetws_s`. Error paths after the mode switch return
/// WITHOUT restoring the console mode, exactly as the C `__leave`s past it.
unsafe fn cgetws_s_locked(string_buffer: *mut u16, size_in_words: usize, size_read: *mut usize) -> c_int {
    let mut string = string_buffer;

    // ReadConsole does not terminate; keep room for it.
    let mut size_remaining = size_in_words - 1;
    *size_read = 0;

    if size_remaining == 0 {
        return 0;
    }

    // A character parked by _cgets_s goes first.
    if *addr_of_mut!(__console_wchar_buffer_used) != 0 {
        *string = *addr_of_mut!(__console_wchar_buffer);
        string = string.add(1);
        size_remaining -= 1;
        *size_read += 1;

        if *addr_of_mut!(__console_wchar_buffer) == 0 {
            size_remaining = 0;
        }

        *addr_of_mut!(__console_wchar_buffer) = 0;
    }

    if size_remaining == 0 {
        return 0;
    }

    if __dcrt_lowio_ensure_console_input_initialized() == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return *_errno();
    }

    let mut old_state: u32 = 0;
    __dcrt_get_input_console_mode(&mut old_state);
    __dcrt_set_input_console_mode(ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT | ENABLE_ECHO_INPUT);

    let mut wchars_read: u32 = 0;
    let read_console_result = __dcrt_read_console(string.cast::<c_void>(), size_remaining as u32, &mut wchars_read);

    if read_console_result == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return *_errno();
    }

    let wchars_read = wchars_read as usize;
    if wchars_read >= 2 && *string.add(wchars_read - 2) == b'\r' as u16 {
        *size_read += wchars_read - 2;
        *string.add(wchars_read - 2) = 0;
    } else if wchars_read == size_remaining && *string.add(wchars_read.wrapping_sub(1)) == b'\r' as u16 {
        // \r\n straddles the boundary.
        *string.add(wchars_read - 1) = 0;
        *size_read += wchars_read - 1;
    } else if wchars_read == 1 && *string == b'\n' as u16 {
        // A lone \n.
        *string = 0;
    } else {
        *size_read += wchars_read;
        *string.add(wchars_read) = 0;
    }

    __dcrt_set_input_console_mode(old_state);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _cgetws_s(string_buffer: *mut u16, size_in_words: usize, size_read: *mut usize) -> c_int {
    if string_buffer.is_null() {
        invalid_clear_oserr(EINVAL);
        return EINVAL;
    }
    if size_in_words == 0 {
        invalid_clear_oserr(EINVAL);
        return EINVAL;
    }
    // _RESET_STRING: the first element only in a release build.
    *string_buffer = 0;

    if size_read.is_null() {
        invalid_clear_oserr(EINVAL);
        return EINVAL;
    }

    with_conio_lock(|| unsafe { cgetws_s_locked(string_buffer, size_in_words, size_read) })
}

/// Note: does NOT see `_ungetwch`'s push-back.
#[no_mangle]
pub unsafe extern "C" fn _cgetws(string: *mut u16) -> *mut u16 {
    if string.is_null() {
        invalid_clear_oserr(EINVAL);
        return core::ptr::null_mut();
    }
    // `string[0] > 0` on a wchar_t (unsigned).
    if *string == 0 {
        invalid_clear_oserr(EINVAL);
        return core::ptr::null_mut();
    }

    let size_in_words = *string as usize;

    let mut size_read: usize = 0;
    let result = _cgetws_s(string.add(2), size_in_words, &mut size_read);

    *string.add(1) = size_read as u16;

    if result == 0 {
        string.add(2)
    } else {
        core::ptr::null_mut()
    }
}

// ── cgets.cpp ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _cgets_s(source_string: *mut c_char, size_in_bytes: usize, size_read: *mut usize) -> c_int {
    if source_string.is_null() {
        invalid_clear_oserr(EINVAL);
        return EINVAL;
    }
    if size_in_bytes == 0 {
        invalid_clear_oserr(EINVAL);
        return EINVAL;
    }
    *source_string = 0; // _RESET_STRING

    if size_read.is_null() {
        invalid_clear_oserr(EINVAL);
        return EINVAL;
    }

    let mut error: c_int = 0;
    let mut string = source_string as *mut u8;

    with_conio_lock(|| unsafe {
        // One wide character at a time: there is no way to know how many
        // UTF-16 units will fit the multibyte buffer.
        *size_read = 0;

        let mut available = size_in_bytes - 1;
        loop {
            let mut wchar_buff: [u16; 2] = [0; 2];
            let mut size_read_w: usize = 0;

            error = _cgetws_s(wchar_buff.as_mut_ptr(), 2, &mut size_read_w);
            if error != 0 {
                break;
            }

            if wchar_buff[0] == 0 {
                break;
            }

            let mut size_converted: c_int = 0;
            let wctomb_result = wctomb_s(&mut size_converted, string, available, wchar_buff[0]);
            if wctomb_result != 0 {
                // Keep the character for the next wide read.
                *addr_of_mut!(__console_wchar_buffer) = wchar_buff[0];
                *addr_of_mut!(__console_wchar_buffer_used) = 1;
                break;
            }

            string = string.offset(size_converted as isize);
            *size_read = (*size_read).wrapping_add(size_converted as isize as usize);
            available = available.wrapping_sub(size_converted as isize as usize);

            if available == 0 {
                break;
            }
        }
    });

    *string = 0;

    if error != 0 {
        set_errno(error);
    }

    error
}

#[no_mangle]
pub unsafe extern "C" fn _cgets(string: *mut c_char) -> *mut c_char {
    if string.is_null() {
        invalid_clear_oserr(EINVAL);
        return core::ptr::null_mut();
    }
    // `string[0] > 0` on a SIGNED char: 0x80..0xFF are rejected.
    if *string <= 0 {
        invalid_clear_oserr(EINVAL);
        return core::ptr::null_mut();
    }

    let size_in_bytes = *string as usize;

    let mut size_read: usize = 0;
    let result = _cgets_s(string.add(2), size_in_bytes, &mut size_read);

    *string.add(1) = size_read as c_char;

    if result == 0 {
        string.add(2)
    } else {
        core::ptr::null_mut()
    }
}

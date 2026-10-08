//! `internal/report_runtime_error.cpp`: `_set_app_type`, `_query_app_type`
//! and `__acrt_report_runtime_error` (the non-enclave, non-debug build).
//!
//! The report goes to stderr -- each UTF-16 unit TRUNCATED to a byte, at most
//! 499 of them -- when `_set_error_mode(_REPORT_ERRMODE)` is `_OUT_TO_STDERR`,
//! or `_OUT_TO_DEFAULT` in a console app; otherwise into a message box whose
//! text is built in a static buffer:
//! `"Runtime Error!\n\nProgram: " + <module path> + "\n\n" + message`.
//! The debug-notification step (`_CrtDbgReportW`) exists only in `_DEBUG`
//! builds and is `false` here.

use super::{errcheck, w, wcscat_s, wcscpy_s, wcsncpy_s, wcslen, MAX_PATH};
use core::ffi::{c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

/// `_crt_app_type::_crt_unknown_app`
const CRT_UNKNOWN_APP: c_int = 0;
/// `_crt_app_type::_crt_console_app`
const CRT_CONSOLE_APP: c_int = 1;

/// `_REPORT_ERRMODE`
const REPORT_ERRMODE: c_int = 3;
/// `_OUT_TO_DEFAULT`
const CRT_OUT_TO_DEFAULT: c_int = 0;
/// `_OUT_TO_STDERR`
const CRT_OUT_TO_STDERR: c_int = 1;

/// `STD_ERROR_HANDLE`, `(DWORD)-12`
const STD_ERROR_HANDLE: u32 = -12i32 as u32;
const INVALID_HANDLE_VALUE: isize = -1;

const MB_OK: u32 = 0x0000_0000;
const MB_ICONHAND: u32 = 0x0000_0010;
const MB_TASKMODAL: u32 = 0x0000_2000;
const MB_SETFOREGROUND: u32 = 0x0001_0000;

/// `sizeof(MSGTEXTPREFIX) / sizeof(wchar_t)`, NUL included.
const PREFIX_COUNT: usize = "Runtime Error!\n\nProgram: ".len() + 1;
/// `_countof(outmsg)`: prefix + program name + 2 newlines + message text.
const OUTMSG_COUNT: usize = PREFIX_COUNT + MAX_PATH + 2 + 500;
const _: () = assert!(OUTMSG_COUNT == 788);
/// `MAXLINELEN` (report_runtime_error.cpp; assert.cpp has its own, 64)
const RTERR_MAXLINELEN: usize = 60;

extern "C" {
    fn _set_error_mode(new_error_mode: c_int) -> c_int;
}

extern "system" {
    fn GetStdHandle(id: u32) -> *mut c_void;
    fn WriteFile(
        handle: *mut c_void,
        buffer: *const c_void,
        bytes_to_write: u32,
        bytes_written: *mut u32,
        overlapped: *mut c_void,
    ) -> c_int;
    fn GetModuleFileNameW(module: *mut c_void, filename: *mut u16, size: u32) -> u32;
}

/// `__acrt_app_type` (file-static in the C): with `__error_mode`, decides where
/// runtime error messages go.
static mut ACRT_APP_TYPE: c_int = CRT_UNKNOWN_APP;

/// `_set_app_type` -- stores any value, unchecked.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _set_app_type(new_app_type: c_int) {
    ACRT_APP_TYPE = new_app_type;
}

/// `_query_app_type` -- `_crt_unknown_app` until someone sets it.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _query_app_type() -> c_int {
    ACRT_APP_TYPE
}

/// `issue_debug_notification`: the `_DEBUG`-only `_CrtDbgReportW` path; always
/// `false` in this (release) build.
fn issue_debug_notification(_message: *const u16) -> bool {
    false
}

/// `should_write_error_to_console`
unsafe fn should_write_error_to_console() -> bool {
    let error_mode = _set_error_mode(REPORT_ERRMODE);

    if error_mode == CRT_OUT_TO_STDERR {
        return true;
    }

    if error_mode == CRT_OUT_TO_DEFAULT && ACRT_APP_TYPE == CRT_CONSOLE_APP {
        return true;
    }

    false
}

/// `write_string_to_console`: truncate each UTF-16 unit to a byte ("the text
/// for each runtime error consists only of ASCII") into a 500-byte buffer and
/// write it, without the terminator, to the stderr HANDLE.
unsafe fn write_string_to_console(wide_string: *const u16) {
    let handle = GetStdHandle(STD_ERROR_HANDLE);
    if handle.is_null() || handle as isize == INVALID_HANDLE_VALUE {
        return;
    }

    const NARROW_BUFFER_COUNT: usize = 500;
    let mut narrow_buffer = [0u8; NARROW_BUFFER_COUNT];

    let narrow_first = narrow_buffer.as_mut_ptr();
    let narrow_last = narrow_first.add(NARROW_BUFFER_COUNT);

    // Copies the terminator too if the loop ends before running out of room:
    // do { *n = (char)*w; } while (++n != last && *w++ != '\0');
    let mut narrow_it = narrow_first;
    let mut wide_it = wide_string;
    loop {
        *narrow_it = *wide_it as u8;
        narrow_it = narrow_it.add(1);
        if narrow_it == narrow_last {
            break;
        }
        let wc = *wide_it;
        wide_it = wide_it.add(1);
        if wc == 0 {
            break;
        }
    }

    // If we did run out of buffer space, this terminates what was copied:
    *narrow_last.sub(1) = 0;

    let bytes_to_write = (narrow_it as usize - narrow_first as usize - 1) as u32; // minus the terminator
    let mut bytes_written: u32 = 0;
    WriteFile(handle, narrow_first.cast(), bytes_to_write, &mut bytes_written, null_mut());
}

/// `outmsg`: the message-box text, static as in the C.
static mut OUTMSG: [u16; OUTMSG_COUNT] = [0; OUTMSG_COUNT];

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_report_runtime_error(message: *const u16) {
    // Before we report the error via the normal path, report it using the
    // debug notification.
    if issue_debug_notification(message) {
        return;
    }

    if should_write_error_to_console() {
        write_string_to_console(message);
    } else {
        let outmsg: *mut u16 = addr_of_mut!(OUTMSG).cast();
        let progname = outmsg.add(PREFIX_COUNT - 1);
        let progname_size = OUTMSG_COUNT - (PREFIX_COUNT - 1);
        let mut pch = progname;

        errcheck(wcscpy_s(outmsg, OUTMSG_COUNT, w!("Runtime Error!\n\nProgram: ").as_ptr()));

        *progname.add(MAX_PATH) = 0;
        if GetModuleFileNameW(null_mut(), progname, MAX_PATH as u32) == 0 {
            errcheck(wcscpy_s(progname, progname_size, w!("<program name unknown>").as_ptr()));
        }

        // Keeps the FRONT of a long path: "..." (and its terminator) lands at
        // len + 1 - RTERR_MAXLINELEN, cutting the tail off. Microsoft's, verbatim.
        if wcslen(pch) + 1 > RTERR_MAXLINELEN {
            pch = pch.add(wcslen(progname) + 1 - RTERR_MAXLINELEN);
            let used = (pch as usize - progname as usize) / core::mem::size_of::<u16>();
            errcheck(wcsncpy_s(pch, progname_size - used, w!("...").as_ptr(), 3));
        }

        errcheck(wcscat_s(outmsg, OUTMSG_COUNT, w!("\n\n").as_ptr()));
        errcheck(wcscat_s(outmsg, OUTMSG_COUNT, message));

        // The result is ignored: the only caller is abort(), so the process
        // ends shortly either way.
        super::crtmbox::__acrt_show_wide_message_box(
            outmsg,
            w!("Microsoft Visual C++ Runtime Library").as_ptr(),
            MB_OK | MB_ICONHAND | MB_SETFOREGROUND | MB_TASKMODAL,
        );
    }
}

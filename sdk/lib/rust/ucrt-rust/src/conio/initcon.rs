//! `initcon.cpp`: the cached console OUTPUT handle (`CONOUT$`), opened on
//! first use and closed at CRT shutdown, and `__dcrt_write_console`, which
//! retries a failed write once on a freshly reopened handle.

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

use super::{widen, GetLastError, CONSOLE_INVALID_HANDLE, CONSOLE_UNINITIALIZED_HANDLE, ERROR_INVALID_HANDLE};

// initcon.cpp carried `_CRT_LINKER_FORCE_INCLUDE(__dcrt_console_output_terminator)`.
crate::force_include!(__dcrt_console_output_terminator);

const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_SHARE_READ: u32 = 0x1;
const FILE_SHARE_WRITE: u32 = 0x2;
const OPEN_EXISTING: u32 = 3;

static CONOUT: [u16; 8] = widen(b"CONOUT$\0");

extern "system" {
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        sa: *mut c_void,
        disposition: u32,
        flags: u32,
        template: *mut c_void,
    ) -> *mut c_void;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn WriteConsoleW(
        console: *mut c_void,
        buffer: *const c_void,
        chars_to_write: u32,
        chars_written: *mut u32,
        reserved: *mut c_void,
    ) -> c_int;
}

/// `static HANDLE __dcrt_lowio_console_output_handle` (file-local in the C).
static mut CONSOLE_OUTPUT_HANDLE: *mut c_void = CONSOLE_UNINITIALIZED_HANDLE as *mut c_void;

unsafe fn handle() -> *mut c_void {
    *addr_of_mut!(CONSOLE_OUTPUT_HANDLE)
}

/// `__dcrt_lowio_initialize_console_output`. A failure caches
/// `INVALID_HANDLE_VALUE`.
unsafe fn initialize_console_output() {
    *addr_of_mut!(CONSOLE_OUTPUT_HANDLE) = CreateFileW(
        CONOUT.as_ptr(),
        GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        null_mut(),
        OPEN_EXISTING,
        0,
        null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_lowio_ensure_console_output_initialized() -> c_int {
    if handle() as isize == CONSOLE_UNINITIALIZED_HANDLE {
        initialize_console_output();
    }
    if handle() as isize == CONSOLE_INVALID_HANDLE {
        return 0;
    }
    1
}

/// Closes the handle if one was ever opened. The cached value is NOT reset.
#[no_mangle]
pub unsafe extern "C" fn __dcrt_terminate_console_output() {
    let h = handle() as isize;
    if h != CONSOLE_INVALID_HANDLE && h != CONSOLE_UNINITIALIZED_HANDLE {
        CloseHandle(handle());
    }
}

/// `console_output_reopen_and_retry`.
unsafe fn reopen_and_retry(fp: impl Fn() -> c_int) -> c_int {
    let mut result = fp();
    if result == 0 && GetLastError() == ERROR_INVALID_HANDLE {
        __dcrt_terminate_console_output();
        initialize_console_output();
        result = fp();
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_write_console(
    buffer: *const c_void,
    number_of_chars_to_write: u32,
    number_of_chars_written: *mut u32,
) -> c_int {
    reopen_and_retry(|| unsafe {
        WriteConsoleW(handle(), buffer, number_of_chars_to_write, number_of_chars_written, null_mut())
    })
}

//! `initconin.cpp`: the cached console INPUT handle (`CONIN$`) and the six
//! console-input wrappers, each retrying once on a freshly reopened handle
//! when the call fails with `ERROR_INVALID_HANDLE`.

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of_mut, null_mut};

use super::{
    widen, GetLastError, InputRecord, CONSOLE_INVALID_HANDLE, CONSOLE_UNINITIALIZED_HANDLE, ERROR_INVALID_HANDLE,
};

// initconin.cpp carried `_CRT_LINKER_FORCE_INCLUDE(__dcrt_console_input_terminator)`.
crate::force_include!(__dcrt_console_input_terminator);

const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_SHARE_READ: u32 = 0x1;
const FILE_SHARE_WRITE: u32 = 0x2;
const OPEN_EXISTING: u32 = 3;

static CONIN: [u16; 7] = widen(b"CONIN$\0");

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
    fn ReadConsoleInputW(console: *mut c_void, buffer: *mut InputRecord, length: u32, read: *mut u32) -> c_int;
    fn ReadConsoleW(
        console: *mut c_void,
        buffer: *mut c_void,
        chars_to_read: u32,
        chars_read: *mut u32,
        control: *mut c_void,
    ) -> c_int;
    fn GetNumberOfConsoleInputEvents(console: *mut c_void, events: *mut u32) -> c_int;
    fn PeekConsoleInputA(console: *mut c_void, buffer: *mut InputRecord, length: u32, read: *mut u32) -> c_int;
    fn GetConsoleMode(console: *mut c_void, mode: *mut u32) -> c_int;
    fn SetConsoleMode(console: *mut c_void, mode: u32) -> c_int;
}

/// `static HANDLE __dcrt_lowio_console_input_handle` (file-local in the C;
/// getch.cpp's `extern "C" intptr_t` redeclaration of it is never used).
static mut CONSOLE_INPUT_HANDLE: *mut c_void = CONSOLE_UNINITIALIZED_HANDLE as *mut c_void;

unsafe fn handle() -> *mut c_void {
    *addr_of_mut!(CONSOLE_INPUT_HANDLE)
}

unsafe fn initialize_console_input() {
    *addr_of_mut!(CONSOLE_INPUT_HANDLE) = CreateFileW(
        CONIN.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        null_mut(),
        OPEN_EXISTING,
        0,
        null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_lowio_ensure_console_input_initialized() -> c_int {
    if handle() as isize == CONSOLE_UNINITIALIZED_HANDLE {
        initialize_console_input();
    }
    if handle() as isize == CONSOLE_INVALID_HANDLE {
        return 0;
    }
    1
}

/// Closes the handle if one was ever opened. The cached value is NOT reset.
#[no_mangle]
pub unsafe extern "C" fn __dcrt_terminate_console_input() {
    let h = handle() as isize;
    if h != CONSOLE_INVALID_HANDLE && h != CONSOLE_UNINITIALIZED_HANDLE {
        CloseHandle(handle());
    }
}

/// `console_input_reopen_and_retry`.
unsafe fn reopen_and_retry(fp: impl Fn() -> c_int) -> c_int {
    let mut result = fp();
    if result == 0 && GetLastError() == ERROR_INVALID_HANDLE {
        __dcrt_terminate_console_input();
        initialize_console_input();
        result = fp();
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_read_console_input(
    buffer: *mut InputRecord,
    length: u32,
    number_of_events_read: *mut u32,
) -> c_int {
    reopen_and_retry(|| unsafe { ReadConsoleInputW(handle(), buffer, length, number_of_events_read) })
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_read_console(
    buffer: *mut c_void,
    number_of_chars_to_read: u32,
    number_of_chars_read: *mut u32,
) -> c_int {
    reopen_and_retry(|| unsafe {
        ReadConsoleW(handle(), buffer, number_of_chars_to_read, number_of_chars_read, null_mut())
    })
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_get_number_of_console_input_events(number_of_events: *mut u32) -> c_int {
    reopen_and_retry(|| unsafe { GetNumberOfConsoleInputEvents(handle(), number_of_events) })
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_peek_console_input_a(
    buffer: *mut InputRecord,
    length: u32,
    number_of_events_read: *mut u32,
) -> c_int {
    reopen_and_retry(|| unsafe { PeekConsoleInputA(handle(), buffer, length, number_of_events_read) })
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_get_input_console_mode(mode: *mut u32) -> c_int {
    reopen_and_retry(|| unsafe { GetConsoleMode(handle(), mode) })
}

#[no_mangle]
pub unsafe extern "C" fn __dcrt_set_input_console_mode(mode: u32) -> c_int {
    reopen_and_retry(|| unsafe { SetConsoleMode(handle(), mode) })
}

//! The six narrow wrappers, one per C file under `sdk/lib/ucrt/internal/`:
//! convert every string argument to UTF-16 in the code page
//! `__acrt_get_utf8_acp_compatibility_codepage` picks (UTF-8 under a UTF-8
//! locale, else ANSI or OEM as `AreFileApisANSI` says -- asked afresh for each
//! argument, as the C evaluates it per call), and call the `W` API. A failed
//! conversion returns the failure value without calling the API; errno is
//! whatever the conversion left (`__acrt_errno_map_os_error` of the
//! MultiByteToWideChar error, or ENOMEM).
//!
//! All `__cdecl` except `__acrt_OutputDebugStringA`, which is `WINAPI`.

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno};
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{null, null_mut};

const ERANGE: c_int = 34;
const MAX_PATH: usize = 260;

/// `STARTUPINFOW` / `PROCESS_INFORMATION`, field for field as `exec.rs` and
/// `conio/popen.rs` declare them (`CreateProcessW` must not clash).
#[repr(C)]
#[allow(dead_code)]
pub struct StartupInfoW {
    cb: u32,
    lp_reserved: *mut u16,
    lp_desktop: *mut u16,
    lp_title: *mut u16,
    dw_x: u32,
    dw_y: u32,
    dw_x_size: u32,
    dw_y_size: u32,
    dw_x_count_chars: u32,
    dw_y_count_chars: u32,
    dw_fill_attribute: u32,
    dw_flags: u32,
    w_show_window: u16,
    cb_reserved2: u16,
    lp_reserved2: *mut u8,
    h_std_input: *mut c_void,
    h_std_output: *mut c_void,
    h_std_error: *mut c_void,
}

#[repr(C)]
#[allow(dead_code)]
pub struct ProcessInformation {
    h_process: *mut c_void,
    h_thread: *mut c_void,
    process_id: u32,
    thread_id: u32,
}

extern "system" {
    fn CreateProcessW(
        app: *const u16,
        cmdline: *mut u16,
        process_attrs: *mut c_void,
        thread_attrs: *mut c_void,
        inherit_handles: i32,
        creation_flags: u32,
        environment: *mut c_void,
        current_dir: *const u16,
        startup_info: *mut StartupInfoW,
        process_info: *mut ProcessInformation,
    ) -> i32;
    fn GetModuleFileNameW(module: *mut c_void, filename: *mut u16, size: u32) -> u32;
    fn LoadLibraryExW(file_name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn OutputDebugStringW(text: *const u16);
    fn SetCurrentDirectoryW(path: *const u16) -> c_int;
    fn SetEnvironmentVariableW(name: *const u16, value: *const u16) -> c_int;
    fn GetLastError() -> u32;
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn __acrt_errno_map_os_error(err: u32);
    fn __acrt_WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
    fn strlen(s: *const core::ffi::c_char) -> usize;
    fn mbstowcs_s(converted: *mut usize, dst: *mut u16, size: usize, src: *const c_char, count: usize) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

/// `__acrt_CreateProcessA`. Takes a STARTUPINFO**W** -- the narrow entry point
/// converts only the strings. The converted command line is not copied back
/// (CreateProcessA callers do not expect it to change).
///
/// # Safety
/// As `CreateProcessW`, with narrow NUL-terminated strings.
#[no_mangle]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn __acrt_CreateProcessA(
    lp_application_name: *const u8,
    lp_command_line: *mut u8,
    lp_process_attributes: *mut c_void,
    lp_thread_attributes: *mut c_void,
    b_inherit_handles: i32,
    dw_creation_flags: u32,
    lp_environment: *mut c_void,
    lp_current_directory: *const u8,
    lp_startup_info: *mut StartupInfoW,
    lp_process_information: *mut ProcessInformation,
) -> i32 {
    let Ok(wide_application_name) = mbs_to_wcs(lp_application_name, file_codepage()) else {
        return 0;
    };
    let Ok(wide_command_line) = mbs_to_wcs(lp_command_line, file_codepage()) else {
        return 0;
    };

    // A NULL current directory never evaluates the code page.
    let mut wide_current_directory = None;
    if !lp_current_directory.is_null() {
        let Ok(w) = mbs_to_wcs(lp_current_directory, file_codepage()) else {
            return 0;
        };
        wide_current_directory = w;
    }

    CreateProcessW(
        wide_application_name.as_ref().map_or(null(), |b| b.0 as *const u16),
        wide_command_line.as_ref().map_or(null_mut(), |b| b.0),
        lp_process_attributes,
        lp_thread_attributes,
        b_inherit_handles,
        dw_creation_flags,
        lp_environment,
        wide_current_directory.as_ref().map_or(null(), |b| b.0 as *const u16),
        lp_startup_info,
        lp_process_information,
    )
}

/// `__acrt_wcs_to_mbs_cp` into a caller's fixed buffer
/// (`__crt_no_alloc_win32_buffer`): "allocating" is an ERANGE failure that
/// also sets errno. Returns (error, size written excluding the NUL).
unsafe fn wcs_to_mbs_cp_no_alloc(input: *const u16, buffer: *mut u8, capacity: usize, code_page: u32) -> (c_int, usize) {
    // Empty input: needs one element of room for the NUL.
    if *input == 0 {
        if capacity == 0 {
            *_errno() = ERANGE;
            return (ERANGE, 0);
        }
        *buffer = 0;
        return (0, 0);
    }

    let required_size_plus_null_terminator =
        __acrt_WideCharToMultiByte(code_page, 0, input, -1, null_mut(), 0, null(), null_mut()) as usize;
    if required_size_plus_null_terminator == 0 {
        return (os_errno(), 0);
    }

    if required_size_plus_null_terminator > capacity {
        *_errno() = ERANGE;
        return (ERANGE, 0);
    }

    let chars_converted_plus_null_terminator = __acrt_WideCharToMultiByte(
        code_page,
        0,
        input,
        -1,
        buffer.cast(),
        capacity as c_int,
        null(),
        null_mut(),
    ) as usize;
    if chars_converted_plus_null_terminator == 0 {
        return (os_errno(), 0);
    }

    (0, chars_converted_plus_null_terminator - 1)
}

/// `__acrt_GetModuleFileNameA`: the wide name (at most MAX_PATH + 1 units),
/// converted into the caller's buffer. Returns the converted length, 0 when
/// the conversion failed or the buffer was too small -- the conversion's
/// error is otherwise ignored, as in the C.
///
/// # Safety
/// `lp_filename` must hold `n_size` bytes.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_GetModuleFileNameA(h_module: *mut c_void, lp_filename: *mut c_char, n_size: u32) -> u32 {
    const WIDE_BUFFER_SIZE: usize = MAX_PATH + 1;
    let mut wide_buffer = [0u16; WIDE_BUFFER_SIZE];

    let amount_copied = GetModuleFileNameW(h_module, wide_buffer.as_mut_ptr(), WIDE_BUFFER_SIZE as u32);
    if amount_copied == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return 0;
    }

    let (_cvt, size) =
        wcs_to_mbs_cp_no_alloc(wide_buffer.as_ptr(), lp_filename.cast(), n_size as usize, file_codepage());
    size as u32
}

/// `__acrt_LoadLibraryExA`.
///
/// # Safety
/// As `LoadLibraryExW`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_LoadLibraryExA(lp_filename: *const c_char, h_file: *mut c_void, dw_flags: u32) -> *mut c_void {
    let Ok(wide_file_name) = mbs_to_wcs(lp_filename.cast(), file_codepage()) else {
        return null_mut();
    };
    LoadLibraryExW(wide_file_name.as_ref().map_or(null(), |b| b.0 as *const u16), h_file, dw_flags)
}

/// `__acrt_OutputDebugStringA` (WINAPI). Converted with `mbstowcs_s` in the
/// CURRENT LOCALE, not the file code page, and into stack memory ("this code
/// is executed during asserts"). The C `_alloca`s any length; here a string
/// that does not fit the stack buffer goes to the heap instead (and is simply
/// dropped if that fails) -- where the C would have overflowed the stack.
///
/// # Safety
/// `text` is NULL or NUL-terminated.
#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn __acrt_OutputDebugStringA(text: *const c_char) {
    if text.is_null() {
        return;
    }
    let text_size = strlen(text).wrapping_add(1);
    if text_size == 0 {
        return;
    }

    const STACK_UNITS: usize = 512;
    let mut stack = [0u16; STACK_UNITS];
    let (text_wide, heap) = if text_size <= STACK_UNITS {
        (stack.as_mut_ptr(), null_mut())
    } else {
        let p = malloc(text_size.wrapping_mul(2)) as *mut u16;
        if p.is_null() {
            return;
        }
        (p, p)
    };

    let mut converted: usize = 0;
    if mbstowcs_s(&mut converted, text_wide, text_size, text, text_size - 1) == 0 {
        OutputDebugStringW(text_wide);
    }

    if !heap.is_null() {
        free(heap.cast());
    }
}

/// `__acrt_SetCurrentDirectoryA`.
///
/// # Safety
/// `lp_path_name` is NULL or NUL-terminated.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_SetCurrentDirectoryA(lp_path_name: *const c_char) -> c_int {
    let Ok(wide_path_name) = mbs_to_wcs(lp_path_name.cast(), file_codepage()) else {
        return 0;
    };
    SetCurrentDirectoryW(wide_path_name.as_ref().map_or(null(), |b| b.0 as *const u16))
}

/// `__acrt_SetEnvironmentVariableA`. A NULL value (delete) converts to NULL.
///
/// # Safety
/// Both are NULL or NUL-terminated.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_SetEnvironmentVariableA(lp_name: *const c_char, lp_value: *const c_char) -> c_int {
    let Ok(wide_name) = mbs_to_wcs(lp_name.cast(), file_codepage()) else {
        return 0;
    };
    let Ok(wide_value) = mbs_to_wcs(lp_value.cast(), file_codepage()) else {
        return 0;
    };
    SetEnvironmentVariableW(
        wide_name.as_ref().map_or(null(), |b| b.0 as *const u16),
        wide_value.as_ref().map_or(null(), |b| b.0 as *const u16),
    )
}

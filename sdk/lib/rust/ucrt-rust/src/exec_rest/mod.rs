//! The rest of `sdk/lib/ucrt/exec/` -- everything but the spawn machinery,
//! which is `exec.rs`. Replaces four C files in full:
//!
//! | C file | exports |
//! |---|---|
//! | `wait.cpp` | `_cwait` |
//! | `system.cpp` | `system`, `_wsystem` |
//! | `getproc.cpp` | `_getdllprocaddr` |
//! | `loaddll.cpp` | `_loaddll`, `_unloaddll` |
//!
//! None of them defines an internal symbol that other ucrt C calls.
//! `system` reaches the environment through `_dupenv_s`/`_wdupenv_s` (env/)
//! and the spawn machinery through `_spawnve`/`_spawnvpe` and their wide
//! forms, called here as the Rust functions in `exec.rs` by their C names.
//!
//! ## Details that are silent if wrong
//!
//! * **`_cwait` CLOSES the handle it waits on**, on success AND on failure
//!   (`__crt_unique_handle` over `process_id`): the caller's handle is dead
//!   afterwards. Only the -1/-2 rejection returns before the handle is
//!   adopted, so those two are never closed.
//! * **`_cwait` presets `*status` to -1 (`(DWORD)-1`), and sets it again on
//!   failure**; the -1/-2 rejection is `_VALIDATE_RETURN_NOEXC` (errno =
//!   ECHILD, no handler) and leaves the preset in place.
//! * **`ERROR_INVALID_HANDLE` maps to ECHILD with `_doserrno` = 6**; every
//!   other failure goes through `__acrt_errno_map_os_error`. The error is the
//!   LAST one from WaitForSingleObject/GetExitCodeProcess, read before the
//!   handle is closed.
//! * **`action_code` is ignored.**
//! * **`system(NULL)` answers "is `%COMSPEC%` set and accessible"** --
//!   `_access_s(comspec, 0) == 0` -- and never looks for cmd.exe.
//! * **`system(cmd)` tries `%COMSPEC% /c cmd` with `_spawnve` and falls back to
//!   `cmd.exe` via `_spawnvpe` only when COMSPEC is unset or the spawn failed
//!   with ENOENT/EACCES**; any other spawn error is returned as is. A
//!   successful COMSPEC spawn restores the caller's errno.
//! * **`_getdllprocaddr` wants EXACTLY ONE of name/ordinal**: a name needs
//!   `ordinal == -1`; no name needs `ordinal <= 65535` -- a SIGNED compare, so
//!   a negative ordinal passes and reaches GetProcAddress cast to a pointer.
//!   Anything else is NULL with no error set.
//! * **`_unloaddll` returns the raw `GetLastError()`**, not an errno.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::exec::{_spawnve, _spawnvpe, _wspawnve, _wspawnvpe};

const EINVAL: c_int = 22;
const ENOENT: c_int = 2;
const EACCES: c_int = 13;
const ECHILD: c_int = 10;
const ERROR_INVALID_HANDLE: u32 = 6;
const INVALID_HANDLE_VALUE: isize = -1;
/// `_P_WAIT`.
const P_WAIT: c_int = 0;

extern "system" {
    fn WaitForSingleObject(handle: *mut c_void, millis: u32) -> u32;
    fn GetExitCodeProcess(handle: *mut c_void, code: *mut u32) -> i32;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn GetLastError() -> u32;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> c_int;
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn __doserrno() -> *mut u32;
    fn __acrt_errno_map_os_error(err: u32);
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    fn free(p: *mut c_void);
    fn _access_s(path: *const u8, mode: c_int) -> c_int;
    fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
    fn _dupenv_s(buf: *mut *mut u8, len: *mut usize, name: *const u8) -> c_int;
    fn _wdupenv_s(buf: *mut *mut u16, len: *mut usize, name: *const u16) -> c_int;
    /// corecrt_internal.h: `HMODULE __cdecl __acrt_LoadLibraryExA(LPCSTR, HANDLE, DWORD)`.
    fn __acrt_LoadLibraryExA(file_name: *const c_char, file: *mut c_void, flags: u32) -> *mut c_void;
}

// ── wait.cpp ─────────────────────────────────────────────────────────────────

/// `__crt_unique_handle`: CloseHandle at scope exit unless INVALID_HANDLE_VALUE.
struct UniqueHandle(*mut c_void);

impl Drop for UniqueHandle {
    fn drop(&mut self) {
        if self.0 as isize != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _cwait(exit_code_result: *mut c_int, process_id: isize, _action_code: c_int) -> isize {
    if !exit_code_result.is_null() {
        *exit_code_result = -1; // static_cast<DWORD>(-1)
    }

    // -1 is the current process and -2 the current thread: waitable forever.
    // _VALIDATE_RETURN_NOEXC: errno only.
    if !(process_id != -1 && process_id != -2) {
        *_errno() = ECHILD;
        return -1;
    }

    let process_handle = UniqueHandle(process_id as *mut c_void);

    let mut exit_code: u32 = 0;
    if WaitForSingleObject(process_handle.0, u32::MAX) == 0 && GetExitCodeProcess(process_handle.0, &mut exit_code) != 0
    {
        if !exit_code_result.is_null() {
            *exit_code_result = exit_code as c_int;
        }
        return process_id;
    }

    let os_error = GetLastError();
    if os_error == ERROR_INVALID_HANDLE {
        *_errno() = ECHILD;
        *__doserrno() = os_error;
    } else {
        __acrt_errno_map_os_error(os_error);
    }

    if !exit_code_result.is_null() {
        *exit_code_result = -1;
    }

    -1
}

// ── system.cpp ───────────────────────────────────────────────────────────────

trait SysChar: Copy + 'static {
    const COMSPEC: &'static [Self];
    const CMD_EXE: &'static [Self];
    const SLASH_C: &'static [Self];
    unsafe fn tdupenv_s_crt(buf: *mut *mut Self, len: *mut usize, name: *const Self) -> c_int;
    unsafe fn taccess_s(path: *const Self, mode: c_int) -> c_int;
    unsafe fn tspawnve(mode: c_int, file: *const Self, argv: *const *const Self, envp: *const *const Self) -> isize;
    unsafe fn tspawnvpe(mode: c_int, file: *const Self, argv: *const *const Self, envp: *const *const Self) -> isize;
}

impl SysChar for u8 {
    const COMSPEC: &'static [u8] = b"COMSPEC\0";
    const CMD_EXE: &'static [u8] = b"cmd.exe\0";
    const SLASH_C: &'static [u8] = b"/c\0";
    unsafe fn tdupenv_s_crt(buf: *mut *mut u8, len: *mut usize, name: *const u8) -> c_int {
        _dupenv_s(buf, len, name)
    }
    unsafe fn taccess_s(path: *const u8, mode: c_int) -> c_int {
        _access_s(path, mode)
    }
    unsafe fn tspawnve(mode: c_int, file: *const u8, argv: *const *const u8, envp: *const *const u8) -> isize {
        _spawnve(mode, file, argv, envp)
    }
    unsafe fn tspawnvpe(mode: c_int, file: *const u8, argv: *const *const u8, envp: *const *const u8) -> isize {
        _spawnvpe(mode, file, argv, envp)
    }
}

const fn widen<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut w = [0u16; N];
    let mut i = 0;
    while i < N {
        w[i] = s[i] as u16;
        i += 1;
    }
    w
}

const W_COMSPEC: [u16; 8] = widen(b"COMSPEC\0");
const W_CMD_EXE: [u16; 8] = widen(b"cmd.exe\0");
const W_SLASH_C: [u16; 3] = widen(b"/c\0");

impl SysChar for u16 {
    const COMSPEC: &'static [u16] = &W_COMSPEC;
    const CMD_EXE: &'static [u16] = &W_CMD_EXE;
    const SLASH_C: &'static [u16] = &W_SLASH_C;
    unsafe fn tdupenv_s_crt(buf: *mut *mut u16, len: *mut usize, name: *const u16) -> c_int {
        _wdupenv_s(buf, len, name)
    }
    unsafe fn taccess_s(path: *const u16, mode: c_int) -> c_int {
        _waccess_s(path, mode)
    }
    unsafe fn tspawnve(mode: c_int, file: *const u16, argv: *const *const u16, envp: *const *const u16) -> isize {
        _wspawnve(mode, file, argv, envp)
    }
    unsafe fn tspawnvpe(mode: c_int, file: *const u16, argv: *const *const u16, envp: *const *const u16) -> isize {
        _wspawnvpe(mode, file, argv, envp)
    }
}

/// `__crt_unique_heap_ptr<Character>`.
struct HeapPtr<T>(*mut T);

impl<T> Drop for HeapPtr<T> {
    fn drop(&mut self) {
        unsafe { free(self.0.cast()) }
    }
}

unsafe fn common_system<C: SysChar>(command: *const C) -> c_int {
    let mut comspec_value: HeapPtr<C> = HeapPtr(null_mut());
    // _ERRCHECK_EINVAL: Watson on EINVAL, any other error passes.
    if C::tdupenv_s_crt(&mut comspec_value.0, null_mut(), C::COMSPEC.as_ptr()) == EINVAL {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }

    // NULL command: is %COMSPEC% set and does its file exist?
    if command.is_null() {
        if comspec_value.0.is_null() {
            return 0;
        }
        return (C::taccess_s(comspec_value.0, 0) == 0) as c_int;
    }

    let mut arguments: [*const C; 4] = [comspec_value.0, C::SLASH_C.as_ptr(), command, core::ptr::null()];

    if !comspec_value.0.is_null() {
        let saved_errno = *_errno();
        *_errno() = 0;

        let result = C::tspawnve(P_WAIT, arguments[0], arguments.as_ptr(), core::ptr::null()) as c_int;
        if result != -1 {
            *_errno() = saved_errno;
            return result;
        }

        if *_errno() != ENOENT && *_errno() != EACCES {
            return result;
        }

        // ENOENT/EACCES: retry with cmd.exe.
        *_errno() = saved_errno;
    }

    arguments[0] = C::CMD_EXE.as_ptr();
    C::tspawnvpe(P_WAIT, arguments[0], arguments.as_ptr(), core::ptr::null()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn system(command: *const u8) -> c_int {
    common_system(command)
}

#[no_mangle]
pub unsafe extern "C" fn _wsystem(command: *const u16) -> c_int {
    common_system(command)
}

// ── getproc.cpp ──────────────────────────────────────────────────────────────

/// Returns `int (__cdecl*)()`: a code pointer, carried as a data pointer (same
/// ABI on every target).
#[no_mangle]
pub unsafe extern "C" fn _getdllprocaddr(
    module_handle_value: isize,
    procedure_name: *mut c_char,
    ordinal: isize,
) -> *mut c_void {
    let module_handle = module_handle_value as *mut c_void;
    if procedure_name.is_null() {
        if ordinal <= 65535 {
            // The ordinal travels as the name pointer (MAKEINTRESOURCE).
            let ordinal_as_string = ordinal as *const c_char;
            return GetProcAddress(module_handle, ordinal_as_string);
        }
    } else if ordinal == -1 {
        return GetProcAddress(module_handle, procedure_name);
    }

    null_mut()
}

// ── loaddll.cpp ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _loaddll(sz_name: *mut c_char) -> isize {
    __acrt_LoadLibraryExA(sz_name, null_mut(), 0) as isize
}

#[no_mangle]
pub unsafe extern "C" fn _unloaddll(h_mod: isize) -> c_int {
    if FreeLibrary(h_mod as *mut c_void) == 0 {
        return GetLastError() as c_int;
    }
    0
}

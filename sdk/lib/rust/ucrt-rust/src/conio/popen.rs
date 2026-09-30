//! `popen.cpp`: `_popen`/`_wpopen` start `%COMSPEC% /c command` (default
//! `cmd.exe`, found through `%PATH%` if not accessible as named) with one end
//! of a pipe as its stdin or stdout, and return a stream on the other end;
//! `_pclose` waits for it. A global table pairs each stream with its child's
//! process handle.
//!
//! Every C++ RAII guard of the C is a Rust `Drop` guard here, declared in the
//! same order so they unwind in the same order: the duplicated child-end
//! handle is ALWAYS closed (the child has its own copy by then), the stream is
//! `fclose`d and the table entry released on any failure.

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr::{addr_of_mut, null, null_mut};

use super::pipe::_pipe;
use super::{errcheck, invalid, set_errno, _errno, __acrt_lock, __acrt_unlock, ACRT_POPEN_LOCK, EBADF, EINTR, EINVAL};

const STDIN: c_int = 0;
const STDOUT: c_int = 1;
const MAX_PATH: usize = 260;
const O_TEXT: c_int = 0x4000;
const O_BINARY: c_int = 0x8000;
const O_NOINHERIT: c_int = 0x0080;
const STARTF_USESTDHANDLES: u32 = 0x0000_0100;
const DUPLICATE_SAME_ACCESS: u32 = 0x2;
const INVALID_HANDLE_VALUE: isize = -1;
/// `_WAIT_GRANDCHILD`.
const WAIT_GRANDCHILD: c_int = 1;

// ── Win32 records ────────────────────────────────────────────────────────────

/// `STARTUPINFOW` (public, stable Win32 ABI; same shape as exec.rs's).
#[repr(C)]
struct StartupInfoW {
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

#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<StartupInfoW>() == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<StartupInfoW>() == 104);

/// `PROCESS_INFORMATION`.
#[repr(C)]
struct ProcessInformation {
    h_process: *mut c_void,
    h_thread: *mut c_void,
    process_id: u32,
    thread_id: u32,
}

extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn DuplicateHandle(
        source_process: *mut c_void,
        source_handle: *mut c_void,
        target_process: *mut c_void,
        target_handle: *mut *mut c_void,
        desired_access: u32,
        inherit_handle: c_int,
        options: u32,
    ) -> c_int;
    fn CloseHandle(handle: *mut c_void) -> i32;
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
}

extern "C" {
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn _recalloc(block: *mut c_void, count: usize, size: usize) -> *mut c_void;
    fn _close(fh: c_int) -> c_int;
    fn _fdopen(fh: c_int, mode: *const c_char) -> *mut c_void;
    fn _wfdopen(fh: c_int, mode: *const u16) -> *mut c_void;
    fn fclose(stream: *mut c_void) -> c_int;
    fn _cwait(status: *mut c_int, process_id: isize, action_code: c_int) -> isize;
    fn _dupenv_s(buf: *mut *mut u8, len: *mut usize, name: *const u8) -> c_int;
    fn _wdupenv_s(buf: *mut *mut u16, len: *mut usize, name: *const u16) -> c_int;
    fn _access_s(path: *const u8, mode: c_int) -> c_int;
    fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
    fn __acrt_getpath(paths: *const u8, buf: *mut u8, bufcount: usize) -> *mut u8;
    fn __acrt_wgetpath(paths: *const u16, buf: *mut u16, bufcount: usize) -> *mut u16;
    fn strlen(s: *const c_char) -> usize;
    fn wcslen(s: *const u16) -> usize;
    fn strcpy_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    fn wcscpy_s(dst: *mut u16, destsz: usize, src: *const u16) -> c_int;
    fn strcat_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    fn wcscat_s(dst: *mut u16, destsz: usize, src: *const u16) -> c_int;
    fn _mbsrchr(s: *const u8, c: c_uint) -> *mut u8;
    /// internal/CreateProcessA.cpp. Takes a STARTUPINFOW even though narrow.
    fn __acrt_CreateProcessA(
        app: *const u8,
        cmdline: *mut u8,
        process_attrs: *mut c_void,
        thread_attrs: *mut c_void,
        inherit_handles: i32,
        creation_flags: u32,
        environment: *mut c_void,
        current_dir: *const u8,
        startup_info: *mut StartupInfoW,
        process_info: *mut ProcessInformation,
    ) -> i32;
}

// ── the stream / process-handle table ────────────────────────────────────────

/// `process_handle_pair`. A free entry has a NULL stream.
#[repr(C)]
struct ProcessHandlePair {
    stream: *mut c_void,
    process_handle: isize,
}

static mut IDTABSIZ: c_uint = 0;
static mut IDPAIRS: *mut ProcessHandlePair = null_mut();

/// `idtab`: the entry for `stream`; for NULL, a free entry -- the table grows
/// by exactly ONE entry when none is free. Caller holds the popen lock.
unsafe fn idtab(stream: *mut c_void) -> *mut ProcessHandlePair {
    let size = *addr_of_mut!(IDTABSIZ);
    let first = *addr_of_mut!(IDPAIRS);
    let mut i = 0;
    while i < size as usize {
        let it = first.add(i);
        if (*it).stream == stream {
            return it;
        }
        i += 1;
    }

    if !stream.is_null() {
        return null_mut();
    }

    if size.wrapping_add(1) < size {
        return null_mut();
    }

    if size as usize + 1 >= usize::MAX / core::mem::size_of::<ProcessHandlePair>() {
        return null_mut();
    }

    let newptr: *mut ProcessHandlePair =
        _recalloc(first.cast(), size as usize + 1, core::mem::size_of::<ProcessHandlePair>()).cast();
    if newptr.is_null() {
        return null_mut();
    }

    *addr_of_mut!(IDPAIRS) = newptr;
    let pairptr = newptr.add(size as usize);
    *addr_of_mut!(IDTABSIZ) = size + 1;

    pairptr
}

// ── RAII guards ──────────────────────────────────────────────────────────────

/// `__crt_unique_handle`.
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

/// `unique_stream`: `fclose` unless NULL.
struct UniqueStream(*mut c_void);
impl Drop for UniqueStream {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                fclose(self.0);
            }
        }
    }
}

/// `unique_process_handle_pair`: marks the entry free unless detached.
struct UniquePair(*mut ProcessHandlePair);
impl Drop for UniquePair {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                (*self.0).process_handle = 0;
                (*self.0).stream = null_mut();
            }
        }
    }
}

/// `__crt_unique_heap_ptr`.
struct HeapPtr<T>(*mut T);
impl<T> Drop for HeapPtr<T> {
    fn drop(&mut self) {
        unsafe { free(self.0.cast()) }
    }
}

// ── the character trait (`__acrt_stdio_char_traits`) ─────────────────────────

trait PopenChar: Copy + PartialEq + 'static {
    const COMSPEC: &'static [Self];
    const PATH: &'static [Self];
    const CMD_EXE: &'static [Self];
    const SLASH_C: &'static [Self];
    const BACKSLASH: &'static [Self];
    fn ch(c: u8) -> Self;
    unsafe fn tdupenv_s_crt(buf: *mut *mut Self, name: *const Self) -> c_int;
    unsafe fn taccess_s(path: *const Self, mode: c_int) -> c_int;
    unsafe fn tgetpath(paths: *const Self, buf: *mut Self, n: usize) -> *mut Self;
    unsafe fn tcslen(s: *const Self) -> usize;
    unsafe fn tcscpy_s(d: *mut Self, n: usize, s: *const Self) -> c_int;
    unsafe fn tcscat_s(d: *mut Self, n: usize, s: *const Self) -> c_int;
    unsafe fn tfdopen(fh: c_int, mode: *const Self) -> *mut c_void;
    /// `__crt_stdio_path_requires_backslash`.
    unsafe fn path_requires_backslash(first: *const Self) -> bool;
    unsafe fn create_process(
        app: *const Self,
        cmdline: *mut Self,
        startup_info: *mut StartupInfoW,
        process_info: *mut ProcessInformation,
    ) -> i32;
}

const fn widen<const N: usize>(s: &[u8; N]) -> [u16; N] {
    super::widen(s)
}

const W_COMSPEC: [u16; 8] = widen(b"COMSPEC\0");
const W_PATH: [u16; 5] = widen(b"PATH\0");
const W_CMD_EXE: [u16; 8] = widen(b"cmd.exe\0");
const W_SLASH_C: [u16; 5] = widen(b" /c \0");
const W_BACKSLASH: [u16; 2] = widen(b"\\\0");

impl PopenChar for u8 {
    const COMSPEC: &'static [u8] = b"COMSPEC\0";
    const PATH: &'static [u8] = b"PATH\0";
    const CMD_EXE: &'static [u8] = b"cmd.exe\0";
    const SLASH_C: &'static [u8] = b" /c \0";
    const BACKSLASH: &'static [u8] = b"\\\0";
    fn ch(c: u8) -> u8 {
        c
    }
    unsafe fn tdupenv_s_crt(buf: *mut *mut u8, name: *const u8) -> c_int {
        _dupenv_s(buf, null_mut(), name)
    }
    unsafe fn taccess_s(path: *const u8, mode: c_int) -> c_int {
        _access_s(path, mode)
    }
    unsafe fn tgetpath(paths: *const u8, buf: *mut u8, n: usize) -> *mut u8 {
        __acrt_getpath(paths, buf, n)
    }
    unsafe fn tcslen(s: *const u8) -> usize {
        strlen(s.cast())
    }
    unsafe fn tcscpy_s(d: *mut u8, n: usize, s: *const u8) -> c_int {
        strcpy_s(d, n, s)
    }
    unsafe fn tcscat_s(d: *mut u8, n: usize, s: *const u8) -> c_int {
        strcat_s(d, n, s)
    }
    unsafe fn tfdopen(fh: c_int, mode: *const u8) -> *mut c_void {
        _fdopen(fh, mode.cast())
    }
    /// The narrow form: a trailing `\` that is really a DBCS trail byte (not
    /// the last `\` `_mbsrchr` finds) still needs a separator.
    unsafe fn path_requires_backslash(first: *const u8) -> bool {
        let last = first.add(strlen(first.cast()));
        if first == last {
            return false;
        }
        if *last.sub(1) == b'\\' {
            return last.sub(1) != _mbsrchr(first, b'\\' as c_uint) as *const u8;
        }
        if *last.sub(1) == b'/' {
            return false;
        }
        true
    }
    unsafe fn create_process(
        app: *const u8,
        cmdline: *mut u8,
        startup_info: *mut StartupInfoW,
        process_info: *mut ProcessInformation,
    ) -> i32 {
        __acrt_CreateProcessA(app, cmdline, null_mut(), null_mut(), 1, 0, null_mut(), null(), startup_info, process_info)
    }
}

impl PopenChar for u16 {
    const COMSPEC: &'static [u16] = &W_COMSPEC;
    const PATH: &'static [u16] = &W_PATH;
    const CMD_EXE: &'static [u16] = &W_CMD_EXE;
    const SLASH_C: &'static [u16] = &W_SLASH_C;
    const BACKSLASH: &'static [u16] = &W_BACKSLASH;
    fn ch(c: u8) -> u16 {
        c as u16
    }
    unsafe fn tdupenv_s_crt(buf: *mut *mut u16, name: *const u16) -> c_int {
        _wdupenv_s(buf, null_mut(), name)
    }
    unsafe fn taccess_s(path: *const u16, mode: c_int) -> c_int {
        _waccess_s(path, mode)
    }
    unsafe fn tgetpath(paths: *const u16, buf: *mut u16, n: usize) -> *mut u16 {
        __acrt_wgetpath(paths, buf, n)
    }
    unsafe fn tcslen(s: *const u16) -> usize {
        wcslen(s)
    }
    unsafe fn tcscpy_s(d: *mut u16, n: usize, s: *const u16) -> c_int {
        wcscpy_s(d, n, s)
    }
    unsafe fn tcscat_s(d: *mut u16, n: usize, s: *const u16) -> c_int {
        wcscat_s(d, n, s)
    }
    unsafe fn tfdopen(fh: c_int, mode: *const u16) -> *mut c_void {
        _wfdopen(fh, mode)
    }
    unsafe fn path_requires_backslash(first: *const u16) -> bool {
        let last = first.add(wcslen(first));
        if first == last {
            return false;
        }
        if *last.sub(1) == b'\\' as u16 {
            return false;
        }
        if *last.sub(1) == b'/' as u16 {
            return false;
        }
        true
    }
    unsafe fn create_process(
        app: *const u16,
        cmdline: *mut u16,
        startup_info: *mut StartupInfoW,
        process_info: *mut ProcessInformation,
    ) -> i32 {
        CreateProcessW(app, cmdline, null_mut(), null_mut(), 1, 0, null_mut(), null(), startup_info, process_info)
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// `convert_popen_type_to_fdopen_mode`: `[wr] *[tb]?`, spaces allowed around
/// the first letter. A failed check RETURNS WHAT WAS FILLED SO FAR: a bad
/// second character still yields mode "r"/"w" (after the handler ran).
unsafe fn convert_popen_type_to_fdopen_mode<C: PopenChar>(type_: *const C) -> [C; 3] {
    let mut result = [C::ch(0); 3];

    let mut type_it = type_;

    while *type_it == C::ch(b' ') {
        type_it = type_it.add(1);
    }

    if !(*type_it == C::ch(b'w') || *type_it == C::ch(b'r')) {
        invalid(EINVAL);
        return result;
    }
    result[0] = *type_it;
    type_it = type_it.add(1);

    while *type_it == C::ch(b' ') {
        type_it = type_it.add(1);
    }

    if !(*type_it == C::ch(0) || *type_it == C::ch(b't') || *type_it == C::ch(b'b')) {
        invalid(EINVAL);
        return result;
    }
    result[1] = *type_it;

    result
}

/// `_ERRCHECK_EINVAL(tdupenv_s_crt(...)) != 0 ? nullptr : value`.
unsafe fn dupenv_or_null<C: PopenChar>(name: *const C) -> *mut C {
    let mut value: *mut C = null_mut();
    let e = C::tdupenv_s_crt(&mut value, name);
    if e == EINVAL {
        super::_invoke_watson(null(), null(), null(), 0, 0);
    }
    if e != 0 {
        return null_mut();
    }
    value
}

/// `get_executable_path`: `executable` itself if accessible, else the first
/// `%PATH%` directory holding it (a new heap string), else NULL. With `%PATH%`
/// unset the search walks a NULL string, as the C does.
unsafe fn get_executable_path<C: PopenChar>(executable: *const C) -> *const C {
    if C::taccess_s(executable, 0) == 0 {
        return executable;
    }

    let mut buffer: HeapPtr<C> = HeapPtr(calloc(MAX_PATH, core::mem::size_of::<C>()).cast());
    if buffer.0.is_null() {
        return null();
    }

    let path: HeapPtr<C> = HeapPtr(dupenv_or_null(C::PATH.as_ptr()));

    let mut current: *const C = path.0;
    loop {
        current = C::tgetpath(current, buffer.0, MAX_PATH - 1);
        if current.is_null() {
            break;
        }

        if C::path_requires_backslash(buffer.0) {
            errcheck(C::tcscat_s(buffer.0, MAX_PATH, C::BACKSLASH.as_ptr()));
        }

        if C::tcslen(buffer.0) + C::tcslen(executable) >= MAX_PATH {
            return null();
        }

        errcheck(C::tcscat_s(buffer.0, MAX_PATH, executable));

        if C::taccess_s(buffer.0, 0) == 0 {
            let found = buffer.0;
            buffer.0 = null_mut();
            return found;
        }
    }

    null()
}

// ── _popen ───────────────────────────────────────────────────────────────────

unsafe fn common_popen_nolock<C: PopenChar>(
    command: *const C,
    fdopen_mode: *const C,
    std_fh: c_int,
    pipe_handles: &mut [c_int; 2],
) -> *mut c_void {
    let process_handle = GetCurrentProcess();

    // Only the second pipe end goes back to the caller; the first is needed
    // only as an (inheritable) OS handle.
    let mut new_pipe_handle = UniqueHandle(INVALID_HANDLE_VALUE as *mut c_void);
    if DuplicateHandle(
        process_handle,
        crate::lowio::osfhnd(pipe_handles[0]) as *mut c_void,
        process_handle,
        &mut new_pipe_handle.0,
        0,
        1,
        DUPLICATE_SAME_ACCESS,
    ) == 0
    {
        return null_mut();
    }

    _close(pipe_handles[0]);
    pipe_handles[0] = -1;

    let mut pipe_stream = UniqueStream(C::tfdopen(pipe_handles[1], fdopen_mode));
    if pipe_stream.0.is_null() {
        return null_mut();
    }

    let mut id_pair = UniquePair(idtab(null_mut()));
    if id_pair.0.is_null() {
        return null_mut();
    }

    let comspec_variable: HeapPtr<C> = HeapPtr(dupenv_or_null(C::COMSPEC.as_ptr()));
    let cmd_exe: *const C = if !comspec_variable.0.is_null() { comspec_variable.0 } else { C::CMD_EXE.as_ptr() };

    let mut startup_info: StartupInfoW = core::mem::zeroed();
    startup_info.cb = core::mem::size_of::<StartupInfoW>() as u32;

    startup_info.dw_flags = STARTF_USESTDHANDLES;
    startup_info.h_std_input =
        if std_fh == STDIN { new_pipe_handle.0 } else { crate::lowio::osfhnd(0) as *mut c_void };
    startup_info.h_std_output =
        if std_fh == STDOUT { new_pipe_handle.0 } else { crate::lowio::osfhnd(1) as *mut c_void };
    startup_info.h_std_error = crate::lowio::osfhnd(2) as *mut c_void;

    let command_line_count = C::tcslen(cmd_exe) + C::tcslen(C::SLASH_C.as_ptr()) + C::tcslen(command) + 1;

    let command_line: HeapPtr<C> = HeapPtr(calloc(command_line_count, core::mem::size_of::<C>()).cast());
    if command_line.0.is_null() {
        return null_mut();
    }

    errcheck(C::tcscpy_s(command_line.0, command_line_count, cmd_exe));
    errcheck(C::tcscat_s(command_line.0, command_line_count, C::SLASH_C.as_ptr()));
    errcheck(C::tcscat_s(command_line.0, command_line_count, command));

    let selected_cmd_exe = get_executable_path(cmd_exe);
    if selected_cmd_exe.is_null() {
        return null_mut();
    }

    // A path other than the one given is ours to free.
    let _owned_final_exe_path: HeapPtr<C> =
        HeapPtr(if selected_cmd_exe != cmd_exe { selected_cmd_exe as *mut C } else { null_mut() });

    let mut process_info: ProcessInformation = core::mem::zeroed();
    let child_status = C::create_process(selected_cmd_exe, command_line.0, &mut startup_info, &mut process_info);

    if child_status == 0 {
        return null_mut();
    }

    let result_stream = core::mem::replace(&mut pipe_stream.0, null_mut());

    CloseHandle(process_info.h_thread);
    (*id_pair.0).process_handle = process_info.h_process as isize;
    (*id_pair.0).stream = result_stream;
    id_pair.0 = null_mut();
    result_stream
}

unsafe fn common_popen<C: PopenChar>(command: *const C, type_: *const C) -> *mut c_void {
    if command.is_null() {
        invalid(EINVAL);
        return null_mut();
    }
    if type_.is_null() {
        invalid(EINVAL);
        return null_mut();
    }

    let fdopen_mode = convert_popen_type_to_fdopen_mode(type_);
    if fdopen_mode[0] == C::ch(0) {
        return null_mut();
    }

    // Neither end is inheritable; the child's end is duplicated inheritable.
    let mut pipe_mode = O_NOINHERIT;
    if fdopen_mode[1] == C::ch(b't') {
        pipe_mode |= O_TEXT;
    }
    if fdopen_mode[1] == C::ch(b'b') {
        pipe_mode |= O_BINARY;
    }

    let mut pipe_handles: [c_int; 2] = [0; 2];
    if _pipe(pipe_handles.as_mut_ptr(), 1024, pipe_mode) == -1 {
        return null_mut();
    }

    let std_fh = if fdopen_mode[0] == C::ch(b'w') { STDIN } else { STDOUT };

    // [0] = the child's end, [1] = ours.
    let mut ordered_pipe_handles: [c_int; 2] = [
        if std_fh == STDIN { pipe_handles[0] } else { pipe_handles[1] },
        if std_fh == STDIN { pipe_handles[1] } else { pipe_handles[0] },
    ];

    __acrt_lock(ACRT_POPEN_LOCK);

    let saved_errno = *_errno();

    let return_value = common_popen_nolock(command, fdopen_mode.as_ptr(), std_fh, &mut ordered_pipe_handles);

    set_errno(saved_errno);

    if return_value.is_null() {
        // On failure every CRT handle still open is closed here.
        for fh in ordered_pipe_handles {
            if fh != -1 {
                _close(fh);
            }
        }
    }

    __acrt_unlock(ACRT_POPEN_LOCK);

    return_value
}

#[no_mangle]
pub unsafe extern "C" fn _popen(command: *const u8, type_: *const u8) -> *mut c_void {
    common_popen(command, type_)
}

#[no_mangle]
pub unsafe extern "C" fn _wpopen(command: *const u16, type_: *const u16) -> *mut c_void {
    common_popen(command, type_)
}

// ── _pclose ──────────────────────────────────────────────────────────────────

/// The child's exit code (the `_cwait` status), or -1. A stream `_popen` did
/// not return is EBADF with no handler.
#[no_mangle]
pub unsafe extern "C" fn _pclose(stream: *mut c_void) -> c_int {
    if stream.is_null() {
        invalid(EINVAL);
        return -1;
    }

    let mut return_value = -1;

    __acrt_lock(ACRT_POPEN_LOCK);

    let id_pair = idtab(stream);
    if id_pair.is_null() {
        set_errno(EBADF);
    } else {
        fclose(stream);

        let process_handle = (*id_pair).process_handle;

        // Free the entry (_cwait closes the handle).
        (*id_pair).stream = null_mut();
        (*id_pair).process_handle = 0;

        let saved_errno = *_errno();
        set_errno(0);

        let mut status: c_int = 0;
        if _cwait(&mut status, process_handle, WAIT_GRANDCHILD) != -1 || *_errno() == EINTR {
            set_errno(saved_errno);
            return_value = status;
        } else {
            set_errno(saved_errno);
        }
    }

    __acrt_unlock(ACRT_POPEN_LOCK);

    return_value
}

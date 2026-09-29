//! `sdk/lib/ucrt/conio/` in full: direct console I/O, the console-adapted
//! printf/scanf entry points, and `_pipe`/`_popen`/`_pclose`. Replaces all
//! fourteen C files:
//!
//! | C file | here |
//! |---|---|
//! | `initcon.cpp` | [`initcon`] -- the cached `CONOUT$` handle, `__dcrt_lowio_ensure_console_output_initialized`, `__dcrt_terminate_console_output`, `__dcrt_write_console` |
//! | `initconin.cpp` | [`initconin`] -- the cached `CONIN$` handle and its six retrying wrappers |
//! | `getch.cpp` | [`getch`] -- `_getch`/`_getche`/`_kbhit`/`_ungetch` (+`_nolock`), `_getextendedkeycode` and its two key tables |
//! | `getwch.cpp` | [`getwch`] -- `_getwch`/`_getwche`/`_ungetwch` (+`_nolock`) |
//! | `putch.cpp`, `putwch.cpp` | [`putch`] -- `_putch`/`_putwch` (+`_nolock`, `_nolock_internal`) |
//! | `cputs.cpp`, `cputws.cpp` | [`cputs`] -- `_cputs`/`_cputws` |
//! | `cgets.cpp`, `cgetws.cpp` | [`cgets`] -- `_cgets(_s)`/`_cgetws(_s)` and `__console_wchar_buffer(_used)` |
//! | `cprintf.cpp` | [`cprintf`] -- the six `__conio_common_vc(w)printf(_s/_p)` |
//! | `cscanf.cpp` | [`cscanf`] -- `__conio_common_vc(w)scanf` |
//! | `pipe.cpp` | [`pipe`] -- `_pipe` |
//! | `popen.cpp` | [`popen`] -- `_popen`/`_wpopen`/`_pclose` and the stream/process table |
//!
//! ## Names other code reads
//!
//! `initializers/console_{output,input}_initializer.cpp` (still C) call
//! `__dcrt_terminate_console_output`/`__dcrt_terminate_console_input` at
//! shutdown; `initcon.cpp`/`initconin.cpp` force-included those initializers,
//! and so does this module. `lowio/write` calls `_putwch_nolock`. The
//! remaining UCRT C that formats to the console reaches `_putch_nolock_internal`
//! / `_putwch_nolock_internal` / `_getche_nolock` / `_ungetch_nolock` (and wide)
//! through `corecrt_internal_traits.h`. Every one of those keeps its C name.
//!
//! ## The printf/scanf engines
//!
//! `cprintf.cpp` and `cscanf.cpp` only instantiate the engines with a console
//! adapter. The engines are `crate::output` (printf) and `crate::stdio::input`
//! (scanf), so this module needs the `output` and `stdio-input` features; the
//! adapters here reproduce `console_output_adapter` and
//! `console_input_adapter` exactly.
//!
//! ## Details that are silent if wrong
//!
//! * **The `_getch` push-back buffer is FIFO and three deep**, not the single
//!   character the comments claim: `_ungetch` fails only when three are
//!   already queued (and the queue only empties once fully read), and
//!   `_getch` returns them in the order they were pushed.
//! * **`_ungetwch` keeps only the LOW BYTE**: `_ungetwch(0x263A)` returns and
//!   later yields `0x3A`. The wide push-back is one character deep.
//! * **`_kbhit` is TRUE whenever the narrow push-back holds anything**, and
//!   never looks at the wide one.
//! * **The console handles are opened once and cached**, and a failed open is
//!   cached too (`INVALID_HANDLE_VALUE`): without a console every later call
//!   fails at once, and a reopen is attempted only after an API call on a
//!   once-valid handle fails with `ERROR_INVALID_HANDLE`.
//! * **`_cputs("")` succeeds without a console; `_cputws(L"")` does not**:
//!   `_cputws` checks the console before looking at the string, `_cputs`
//!   writes character by character and so never touches it.
//! * **`_cputs` stops at a `'\xFF'`**: each `char` goes to `_putch_nolock`
//!   sign-extended, and `_putch` returns its argument -- -1, i.e. EOF.
//! * **`_putch` buffers a DBCS lead byte in the per-thread data** and writes
//!   nothing until the trail byte arrives.
//! * **`__console_wchar_buffer_used` is never reset** by `_cgetws_s`: once
//!   `_cgets_s` has parked a character there, every later `_cgetws_s` starts
//!   with the parked value (reset to 0 after the first).
//! * **`_popen(cmd, "rx")` sets EINVAL and calls the handler but STILL runs
//!   `cmd`**: the mode validator returns its partly-filled result, `"r"`.
//! * **`_popen`'s child gets `_osfhnd(2)` as its stderr** (and `_osfhnd(0/1)`
//!   for the end it does not pipe), whatever that slot holds -- including
//!   `_NO_CONSOLE_FILENO` (-2) in a GUI process.
//! * **`_cwait` closes the child's process handle**, so `_pclose` does not.

use core::ffi::c_int;

#[cfg(not(all(feature = "output", feature = "stdio-input")))]
compile_error!(
    "conio needs the printf and scanf engines: Cargo.toml's `conio` feature must \
     include \"output\" and \"stdio-input\""
);

pub mod cgets;
pub mod cprintf;
pub mod cputs;
pub mod cscanf;
pub mod getch;
pub mod getwch;
pub mod initcon;
pub mod initconin;
pub mod pipe;
pub mod popen;
pub mod putch;

pub(crate) const EINVAL: c_int = 22;
pub(crate) const EBADF: c_int = 9;
pub(crate) const EMFILE: c_int = 24;
pub(crate) const EINTR: c_int = 4;
pub(crate) const EILSEQ: c_int = 42;
pub(crate) const ERANGE: c_int = 34;

pub(crate) const EOF: c_int = -1;
pub(crate) const WEOF: u16 = 0xFFFF;

pub(crate) const ERROR_INVALID_HANDLE: u32 = 6;

/// `__acrt_lock_id::__acrt_conio_lock` (corecrt_internal.h: heap, debug, exit,
/// signal, locale, multibyte_cp, time, lowio_index, stdio_index, then this).
pub(crate) const ACRT_CONIO_LOCK: c_int = 9;
/// `__acrt_lock_id::__acrt_popen_lock`, the next enumerator.
pub(crate) const ACRT_POPEN_LOCK: c_int = 10;

/// `_console_invalid_handle` / `_console_uninitialized_handle`
/// (corecrt_internal_lowio.h).
pub(crate) const CONSOLE_INVALID_HANDLE: isize = -1;
pub(crate) const CONSOLE_UNINITIALIZED_HANDLE: isize = -2;

extern "C" {
    pub(crate) fn _errno() -> *mut c_int;
    pub(crate) fn __doserrno() -> *mut u32;
    pub(crate) fn _invalid_parameter_noinfo();
    pub(crate) fn __acrt_errno_map_os_error(err: u32);
    pub(crate) fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    pub(crate) fn __acrt_lock(lock: c_int);
    pub(crate) fn __acrt_unlock(lock: c_int);
}

extern "system" {
    pub(crate) fn GetLastError() -> u32;
}

#[inline]
pub(crate) unsafe fn set_errno(e: c_int) {
    *_errno() = e;
}

/// `_VALIDATE_CLEAR_OSSERR_RETURN*`'s failure arm: `_doserrno = 0`, errno,
/// then the handler.
#[cold]
pub(crate) unsafe fn invalid_clear_oserr(e: c_int) {
    *__doserrno() = 0;
    set_errno(e);
    _invalid_parameter_noinfo();
}

/// `_VALIDATE_RETURN*`'s failure arm.
#[cold]
pub(crate) unsafe fn invalid(e: c_int) {
    set_errno(e);
    _invalid_parameter_noinfo();
}

/// `_ERRCHECK(e)`.
#[inline]
pub(crate) unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

/// `__acrt_lock(__acrt_conio_lock)` ... `__finally { __acrt_unlock(...) }`, and
/// `__acrt_lock_and_call(__acrt_conio_lock, ...)`.
#[inline]
pub(crate) unsafe fn with_conio_lock<R>(f: impl FnOnce() -> R) -> R {
    __acrt_lock(ACRT_CONIO_LOCK);
    let r = f();
    __acrt_unlock(ACRT_CONIO_LOCK);
    r
}

/// An ASCII literal widened at compile time (NUL included by the caller).
pub(crate) const fn widen<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut w = [0u16; N];
    let mut i = 0;
    while i < N {
        w[i] = s[i] as u16;
        i += 1;
    }
    w
}

// ── console records (wincon.h / consoleapi.h) ────────────────────────────────

/// `KEY_EVENT_RECORD`. Public, stable Win32 ABI.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyEventRecord {
    pub key_down: c_int,
    pub repeat_count: u16,
    pub virtual_key_code: u16,
    pub virtual_scan_code: u16,
    /// `uChar`: `UnicodeChar`, or `AsciiChar` in its low byte.
    pub u_char: u16,
    pub control_key_state: u32,
}

/// `INPUT_RECORD`'s `Event` union: the largest members (key and mouse
/// records) are 16 bytes, 4-aligned.
#[repr(C)]
#[derive(Clone, Copy)]
pub union InputEvent {
    pub key_event: KeyEventRecord,
    pub raw: [u32; 4],
}

/// `INPUT_RECORD`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InputRecord {
    pub event_type: u16,
    pub event: InputEvent,
}

const _: () = assert!(core::mem::size_of::<KeyEventRecord>() == 16);
const _: () = assert!(core::mem::offset_of!(KeyEventRecord, virtual_scan_code) == 8);
const _: () = assert!(core::mem::offset_of!(KeyEventRecord, u_char) == 10);
const _: () = assert!(core::mem::offset_of!(KeyEventRecord, control_key_state) == 12);
const _: () = assert!(core::mem::size_of::<InputRecord>() == 20);
const _: () = assert!(core::mem::offset_of!(InputRecord, event) == 4);

pub(crate) const KEY_EVENT: u16 = 0x0001;

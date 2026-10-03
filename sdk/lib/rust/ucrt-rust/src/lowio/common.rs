//! What the lowio submodules share beyond `mod.rs`'s core: the Win32 and CRT
//! functions more than one of them calls, the `fcntl.h` constants, and the
//! PTD-propagating (`_UCRT_*`) forms of the validation macros.
//!
//! Every Win32 prototype here is `extern "system"` (stdcall on i386) and was
//! copied from an existing declaration in the crate where one existed, so the
//! crate-wide `clashing_extern_declarations` deny stays quiet: `ReadFile`
//! takes its handle as `isize` because `stdio/files/raw_lowio.rs` does,
//! `SetFilePointerEx` takes `LARGE_INTEGER` as `i64` because
//! `stdio/bufio/refill_flush.rs` does (an 8-byte union passed by value is an
//! `i64` on all three ABIs).

use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

pub use super::{EBADF, EINVAL};

/// `fcntl.h`.
pub const CRT_O_RDWR: c_int = 0x0002;
pub const CRT_O_CREAT: c_int = 0x0100;
pub const CRT_O_TRUNC: c_int = 0x0200;
pub const CRT_O_TEXT: c_int = 0x4000;
pub const CRT_O_BINARY: c_int = 0x8000;
pub const CRT_O_WTEXT: c_int = 0x10000;
pub const CRT_O_U16TEXT: c_int = 0x20000;
pub const CRT_O_U8TEXT: c_int = 0x40000;
/// `share.h`.
pub const CRT_SH_DENYNO: c_int = 0x40;

/// `errno.h` values the core does not already name.
pub const CRT_EEXIST: c_int = 17;
pub const CRT_EDEADLOCK: c_int = 36;
pub const CRT_EILSEQ: c_int = 42;

/// winerror.h.
pub const ERROR_ACCESS_DENIED_W: u32 = 5;
pub const ERROR_NOT_ENOUGH_MEMORY_W: u32 = 8;
pub const ERROR_BROKEN_PIPE_W: u32 = 109;

/// `SEEK_SET`/`SEEK_CUR`/`SEEK_END` == `FILE_BEGIN`/`FILE_CURRENT`/`FILE_END`
/// (lseek.cpp `#error`s if they ever differ).
pub const SEEK_SET_W: c_int = 0;
pub const SEEK_CUR_W: c_int = 1;
pub const SEEK_END_W: c_int = 2;

pub const CP_UTF8_W: c_uint = 65001;

extern "C" {
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(count: usize, size: usize) -> *mut c_void;
    pub fn free(p: *mut c_void);
    /// misc/errno.cpp (Rust: `misc/errno.rs`).
    pub fn __acrt_errno_map_os_error_ptd(oserrno: c_ulong, ptd: *mut PtdHost);
    /// corecrt_internal_win32_buffer / nls wrappers.
    pub fn __acrt_MultiByteToWideChar(
        code_page: u32,
        flags: u32,
        mb: *const c_char,
        cb_mb: c_int,
        wide: *mut u16,
        cch_wide: c_int,
    ) -> c_int;
    pub fn __acrt_WideCharToMultiByte(
        code_page: u32,
        flags: u32,
        wide: *const u16,
        cch_wide: c_int,
        mb: *mut c_char,
        cb_mb: c_int,
        default_char: *const c_char,
        used_default_char: *mut c_int,
    ) -> c_int;
}

extern "system" {
    pub fn GetLastError() -> u32;
    pub fn ReadFile(handle: isize, buffer: *mut c_void, bytes_to_read: u32, bytes_read: *mut u32, overlapped: *mut c_void) -> i32;
    pub fn WriteFile(handle: *mut c_void, buffer: *const c_void, bytes_to_write: u32, bytes_written: *mut u32, overlapped: *mut c_void) -> c_int;
    pub fn SetFilePointerEx(handle: *mut c_void, distance: i64, new_pos: *mut i64, method: c_uint) -> c_int;
    pub fn GetConsoleMode(handle: *mut c_void, mode: *mut u32) -> c_int;
}

/// `ptd.get_errno().value_or(alternative)`.
pub unsafe fn errno_value_or(ptd: &mut PtdHost, alternative: c_int) -> c_int {
    ptd.errno().value_or(alternative)
}

/// `_UCRT_VALIDATE_CLEAR_OSSERR_RETURN`'s failure arm: `_doserrno = 0` and
/// errno into the host cache, then `_invalid_parameter_internal`. The caller
/// returns.
pub unsafe fn ptd_invalid_clear_oserr(ptd: &mut PtdHost, e: c_int) {
    ptd.doserrno().set(0);
    ptd.invalid_parameter(e);
}

/// `_UCRT_CHECK_FH_CLEAR_OSSERR_RETURN`: `true` means the caller returns,
/// having recorded `_doserrno = 0; errno = e` in the host (no handler).
pub unsafe fn ptd_no_console_fh(ptd: &mut PtdHost, fh: c_int, e: c_int) -> bool {
    if fh as isize == super::NO_CONSOLE_FILENO {
        ptd.doserrno().set(0);
        ptd.errno().set(e);
        return true;
    }
    false
}

/// The three-step validation almost every PTD-propagating lowio entry point
/// opens with (`_UCRT_CHECK_FH_CLEAR_OSSERR_RETURN`, then the range and
/// `FOPEN` `_UCRT_VALIDATE_CLEAR_OSSERR_RETURN`s, all with `EBADF`). `false`
/// means the caller returns its failure value.
pub unsafe fn ptd_validate_fh(ptd: &mut PtdHost, fh: c_int) -> bool {
    if ptd_no_console_fh(ptd, fh, EBADF) {
        return false;
    }
    if !super::in_range(fh) {
        ptd_invalid_clear_oserr(ptd, EBADF);
        return false;
    }
    if super::osfile(fh) & super::FOPEN == 0 {
        ptd_invalid_clear_oserr(ptd, EBADF);
        return false;
    }
    true
}

/// The same opening sequence for the non-PTD forms (`_CHECK_FH_CLEAR_OSSERR_RETURN`
/// then two `_VALIDATE_CLEAR_OSSERR_RETURN`s, all `EBADF`).
pub unsafe fn validate_fh_clear_oserr(fh: c_int) -> bool {
    if super::no_console_fh(fh, EBADF) {
        return false;
    }
    if !super::in_range(fh) {
        super::invalid_clear_oserr(EBADF);
        return false;
    }
    if super::osfile(fh) & super::FOPEN == 0 {
        super::invalid_clear_oserr(EBADF);
        return false;
    }
    true
}

/// `__crt_unique_heap_ptr<T>`: frees on scope exit.
pub struct HeapPtr(pub *mut c_void);
impl Drop for HeapPtr {
    fn drop(&mut self) {
        unsafe { free(self.0) }
    }
}

//! `_open`/`_sopen`/`_wopen`/`_wsopen` and their `_dispatch`/`_s` forms.
//! Replaces `sdk/lib/ucrt/lowio/open.cpp` in full.
//!
//! ## Variadic entry points -- via the `varargs-rust` shim, not defined here
//!
//! `_open`, `_sopen`, `_wopen`, `_wsopen` are all `(..., int oflag, ...)` with
//! exactly one further `int pmode` fetched via a single `va_arg` -- and the C
//! reads it UNCONDITIONALLY, regardless of `oflag` (no `_O_CREAT` gate: even
//! `_open(path, _O_RDONLY)` reads a `pmode` argument nobody passed, off
//! whatever happens to be in the caller's frame at that stack slot -- ported
//! faithfully, not "fixed").
//!
//! Stable Rust cannot define a C-variadic function or `va_arg` a `va_list`
//! (`c_variadic` is nightly-only). `dll/win32/ucrtbase/ucrt_varargs_shim.c`
//! supplies the actual `...` entry points (`_open`/`_wopen`/`_sopen`/`_wsopen`
//! themselves, each a three-line `va_start` trampoline) and this file supplies
//! their fixed-arity Rust twins, taking the STARTED `va_list` by pointer
//! (`void *ap`, wrapped in [`varargs_rust::VaArgs`]):
//!
//! * `_open_v(path, oflag, ap)` -- pulls `pmode`, calls [`common_open_narrow`].
//! * `_wopen_v(path, oflag, ap)` -- symmetric, [`common_open_wide`].
//! * `_sopen_v(path, oflag, shflag, ap)` -- pulls `pmode`, calls
//!   `_sopen_dispatch(path, oflag, shflag, pmode, &fh, FALSE)`, returns `fh`
//!   or `-1` -- exactly `_sopen`'s own (non-`_dispatch`) body.
//! * `_wsopen_v` -- symmetric, via `_wsopen_dispatch`.
//!
//! `_open`/`_wopen`/`_sopen`/`_wsopen` themselves are NOT defined here -- the
//! shim owns those four names; this file only ever exports the `_v` twins the
//! shim declares and calls.
//!
//! ## Details that are silent if wrong
//!
//! * **`decode_access_flags`'s "unreachable" default arm is REACHABLE from
//!   the public API**: `oflag & (_O_RDONLY|_O_WRONLY|_O_RDWR) == 3` (i.e.
//!   `_O_WRONLY | _O_RDWR`, illegal but not rejected earlier) hits it, and
//!   the C's `_VALIDATE_RETURN` there calls the invalid-parameter handler
//!   BEFORE returning `DWORD(-1)` as the access mask -- with no handler
//!   installed (this tree's default), that call never returns
//!   (`_invoke_watson`), so the bogus access mask is never actually used.
//!   Only an application that installs a NON-terminating handler observes
//!   `CreateFileW` called with `access == 0xFFFFFFFF`. Ported faithfully:
//!   this file does not add a check the C doesn't have.
//! * **`decode_sharing_flags` rejects `_SH_COMPAT` (0x00)** -- it is a valid
//!   `_SH_*` constant but has no case in the switch, so it hits the same
//!   invalid-parameter path as a garbage `shflag`. Not a typo in the C: the
//!   compatibility mode has no `CreateFileW` sharing-flag equivalent.
//! * **Read access is added for append-mode text writes**: `_O_WRONLY |
//!   _O_APPEND` with any of `_O_WTEXT|_O_U16TEXT|_O_U8TEXT` gets
//!   `GENERIC_READ | GENERIC_WRITE`, because the BOM must be read back to
//!   pick the encoding even though the caller asked for write-only. If that
//!   combined access fails, [`create_file`] is retried write-only (losing the
//!   ability to detect the BOM, so the default encoding is assumed) --
//!   twice, once in `_wsopen_nolock` itself around the open, and, if the
//!   ORIGINAL call asked for exactly `_O_WRONLY`, a THIRD time at the very
//!   end to drop back to write-only access after the BOM has been sorted out
//!   (closing and reopening the file, not just dropping a flag).
//! * **The BOM dance only runs for `_O_WTEXT`/`_O_U16TEXT`/`_O_U8TEXT`.**
//!   Binary and plain `_O_TEXT` (ANSI) files never read or write one.
//!   Devices (`FDEV`) never check for one even if a wide text mode was
//!   requested -- there is nothing to seek on a device.
//! * **Writing a BOM only happens for a genuinely NEW/EMPTY file**
//!   (`CREATE_NEW`/`CREATE_ALWAYS`/`TRUNCATE_EXISTING`, or `OPEN_EXISTING`/
//!   `OPEN_ALWAYS` when a seek-to-end lands back at offset 0). Reading one
//!   only happens when the file has read access and (for `OPEN_EXISTING`/
//!   `OPEN_ALWAYS`) is non-empty.
//! * **A UTF-8 BOM match falls through into the UTF-16 case** in the C's
//!   `switch (count)` (no `break` after the `UTF8_BOMLEN` case when the BOM
//!   does NOT match `UTF8_BOM`) -- ported here as an explicit fallthrough,
//!   not an oversight. A 3-byte read that isn't a UTF-8 BOM is re-examined as
//!   a (2-byte-prefix) UTF-16 BOM.
//! * **A UTF-16 BIG-endian BOM is a hard error** (`EINVAL`, an `_ASSERTE`
//!   fires in a debug C runtime) -- only little-endian UTF-16 and UTF-8 are
//!   accepted for reads. Anything else with no recognizable BOM seeks back
//!   to the start and keeps the mode chosen before the BOM check.
//! * **`_O_RDWR` files get a trailing Ctrl+Z stripped** before the BOM is
//!   even looked at (`truncate_ctrl_z_if_present`), skipped entirely for
//!   devices/pipes and for binary-mode files.
//! * **`_wsopen_nolock`/`_sopen_nolock` hold the file-handle lock on return
//!   in EVERY case, success or failure** -- the caller
//!   (`common_open_narrow`/`common_open_wide`/`_sopen_dispatch`/
//!   `_wsopen_dispatch`) is the one that unlocks, via `punlock_flag`. This
//!   file follows the C's `__try`/`__finally` shape with an explicit
//!   `unlock_flag` out-parameter for exactly that reason -- Rust has no SEH
//!   `__finally`, and the C relies on it running even if an exception
//!   propagates through `tsopen_nolock` (which nothing here can throw, but
//!   the shape is kept for a faithful diff against the C).
//! * **`_sopen_nolock`'s codepage-conversion failure returns errno_t `-1`
//!   literally**, not the real conversion error code -- `if (cvt != 0) return
//!   -1;` in the C, preserved verbatim (a plainly odd `errno_t`, but it is
//!   what ships).
//! * **`_umaskval` narrows `FILE_ATTRIBUTE_READONLY`**: a newly-created file
//!   is marked read-only only if `(pmode & ~_umaskval) & _S_IWRITE == 0`,
//!   i.e. the process umask can suppress write permission even when the
//!   caller asked for it.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{null, null_mut};

use crate::findfile::{file_codepage, mbs_to_wcs};
use varargs_rust::VaArgs;

// ── errno / OS constants (transcribed, not recalled) ────────────────────────

const EINVAL: c_int = 22;
const CRT_EACCES: c_int = 13;
const CRT_EMFILE: c_int = 24;

/// `SEEK_SET`/`SEEK_END` (`stdio.h`).
const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

/// `ERROR_NEGATIVE_SEEK` (`winerror.h`).
const ERROR_NEGATIVE_SEEK: u32 = 131;

// `_O_*` (`fcntl.h`).
const O_RDONLY: c_int = 0x0000;
const O_WRONLY: c_int = 0x0001;
const O_RDWR: c_int = 0x0002;
const O_APPEND: c_int = 0x0008;
const O_RANDOM: c_int = 0x0010;
const O_SEQUENTIAL: c_int = 0x0020;
const O_TEMPORARY: c_int = 0x0040;
const O_NOINHERIT: c_int = 0x0080;
const O_CREAT: c_int = 0x0100;
const O_TRUNC: c_int = 0x0200;
const O_EXCL: c_int = 0x0400;
const O_SHORT_LIVED: c_int = 0x1000;
const O_OBTAIN_DIR: c_int = 0x2000;
const O_TEXT: c_int = 0x4000;
const O_BINARY: c_int = 0x8000;
const O_WTEXT: c_int = 0x10000;
const O_U16TEXT: c_int = 0x20000;
const O_U8TEXT: c_int = 0x40000;

// `_SH_*` (`share.h`).
const SH_DENYRW: c_int = 0x10;
const SH_DENYWR: c_int = 0x20;
const SH_DENYRD: c_int = 0x30;
const SH_DENYNO: c_int = 0x40;
const SH_SECURE: c_int = 0x80;

// `_S_IREAD`/`_S_IWRITE` (`sys/stat.h`).
const S_IREAD: c_int = 0x0100;
const S_IWRITE: c_int = 0x0080;

// Win32 (`winnt_old.h`/`setypes.h`/`winbase.h`).
const DELETE: u32 = 0x0001_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const GENERIC_READ_WRITE: u32 = GENERIC_READ | GENERIC_WRITE;
const FILE_SHARE_READ: u32 = 0x0000_0001;
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
const FILE_SHARE_DELETE: u32 = 0x0000_0004;
const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
const FILE_FLAG_RANDOM_ACCESS: u32 = 0x1000_0000;
const FILE_FLAG_SEQUENTIAL_SCAN: u32 = 0x0800_0000;
const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const CREATE_NEW: u32 = 1;
const CREATE_ALWAYS: u32 = 2;
const OPEN_EXISTING: u32 = 3;
const OPEN_ALWAYS: u32 = 4;
const TRUNCATE_EXISTING: u32 = 5;
const INVALID_HANDLE_VALUE: isize = -1;

const FILE_TYPE_UNKNOWN: u32 = 0;
const FILE_TYPE_CHAR: u32 = 2;
const FILE_TYPE_PIPE: u32 = 3;

// `__crt_lowio_handle_data::osfile` bits (`corecrt_internal_lowio.h`).
const FOPEN: u8 = 0x01;
const FPIPE: u8 = 0x08;
const FNOINHERIT: u8 = 0x10;
const FAPPEND: u8 = 0x20;
const FDEV: u8 = 0x40;
const FTEXT: u8 = 0x80;

// `__crt_lowio_text_mode` (`corecrt_internal_lowio.h`).
const TEXTMODE_ANSI: u8 = 0;
const TEXTMODE_UTF8: u8 = 1;
const TEXTMODE_UTF16LE: u8 = 2;

// BOM constants, from `open.cpp` itself.
const UTF16LE_BOM: i32 = 0xFEFF;
const UTF16BE_BOM: i32 = 0xFFFE;
const BOM_MASK: i32 = 0xFFFF;
const UTF8_BOM: i32 = 0xBFBBEF;
const UTF16_BOMLEN: u32 = 2;
const UTF8_BOMLEN: u32 = 3;

type Handle = *mut c_void;

#[repr(C)]
struct SecurityAttributes {
    n_length: u32,
    lp_security_descriptor: *mut c_void,
    b_inherit_handle: i32,
}

extern "system" {
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        sa: *mut SecurityAttributes,
        disposition: u32,
        flags: u32,
        template: Handle,
    ) -> Handle;
    fn CloseHandle(h: Handle) -> c_int;
    fn GetFileType(h: Handle) -> u32;
    fn GetLastError() -> u32;
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn __acrt_errno_map_os_error(err: u32);

    fn _alloc_osfhnd() -> c_int;
    fn _free_osfhnd(fh: c_int) -> c_int;
    fn __acrt_lowio_set_os_handle(fh: c_int, handle: isize) -> c_int;
    fn __acrt_lowio_lock_fh(fh: c_int);
    fn __acrt_lowio_unlock_fh(fh: c_int);

    fn _chsize_nolock(fh: c_int, size: i64) -> c_int;
    fn _close_nolock(fh: c_int) -> c_int;
    fn _read_nolock(fh: c_int, buf: *mut c_void, max: u32) -> c_int;
    fn _lseeki64_nolock(fh: c_int, offset: i64, origin: c_int) -> i64;
    fn _write(fh: c_int, buf: *const c_void, max: u32) -> c_int;
    fn _get_fmode(pmode: *mut c_int) -> c_int;

    static _umaskval: c_int;
}

// ── `__crt_lowio_handle_data` write accessors ───────────────────────────────
//
// `crate::stdio_layout::lowio` already measures and pins this record's
// layout, but only exposes READ accessors (`osfile`, `osfhnd`) and keeps its
// offsets `mod`-private, because nothing there needed to WRITE a handle
// record. `_wsopen_nolock` does (it is what CREATES the record's live
// content), so the offsets below are the SAME measured values, duplicated
// rather than re-measured (`stdio_layout.rs`'s own doc comment: "measured
// with the build's own compiler flags on all three targets and asserted on
// every build" -- if they were ever wrong, that module's own `pin_stream!`-
// style asserts would already fail the build). Not independently
// re-measured for this batch; flagged in the report.
mod handle {
    use core::ffi::c_int;

    use super::__pioinfo;

    const IOINFO_L2E: c_int = 6;
    const IOINFO_ARRAY_ELTS: c_int = 1 << IOINFO_L2E;
    const WIDE: bool = cfg!(target_pointer_width = "64");

    /// Same values as `stdio_layout::handle`.
    const HANDLE_DATA_SIZE: usize = if WIDE { 72 } else { 56 };
    const HANDLE_OSFHND_OFFSET: usize = if WIDE { 40 } else { 24 };
    const HANDLE_OSFILE_OFFSET: usize = if WIDE { 56 } else { 40 };
    const HANDLE_TEXTMODE_OFFSET: usize = if WIDE { 57 } else { 41 };
    const HANDLE_UNICODE_BYTE: usize = if WIDE { 61 } else { 45 };

    unsafe fn pioinfo_mut(fh: c_int) -> *mut u8 {
        let array = __pioinfo[(fh >> IOINFO_L2E) as usize];
        array.add((fh & (IOINFO_ARRAY_ELTS - 1)) as usize * HANDLE_DATA_SIZE)
    }

    /// `_osfile(fh) = v` / `_osfile(fh) &= ~v` / `_osfile(fh) |= v` (unchecked,
    /// as the macro is).
    pub unsafe fn set_osfile(fh: c_int, v: u8) {
        *pioinfo_mut(fh).add(HANDLE_OSFILE_OFFSET) = v;
    }
    pub unsafe fn osfile(fh: c_int) -> u8 {
        *pioinfo_mut(fh).add(HANDLE_OSFILE_OFFSET)
    }
    pub unsafe fn clear_osfile_bits(fh: c_int, bits: u8) {
        let p = pioinfo_mut(fh).add(HANDLE_OSFILE_OFFSET);
        *p &= !bits;
    }
    pub unsafe fn set_osfile_bits(fh: c_int, bits: u8) {
        let p = pioinfo_mut(fh).add(HANDLE_OSFILE_OFFSET);
        *p |= bits;
    }

    /// `_osfhnd(fh) = v`.
    pub unsafe fn set_osfhnd(fh: c_int, v: isize) {
        *(pioinfo_mut(fh).add(HANDLE_OSFHND_OFFSET) as *mut isize) = v;
    }

    /// `_textmode(fh) = v`.
    pub unsafe fn set_textmode(fh: c_int, v: u8) {
        *pioinfo_mut(fh).add(HANDLE_TEXTMODE_OFFSET) = v;
    }

    /// `_tm_unicode(fh) = v` -- bit 0 of the byte following the 3-byte
    /// `_pipe_lookahead`.
    pub unsafe fn set_tm_unicode(fh: c_int, v: bool) {
        let p = pioinfo_mut(fh).add(HANDLE_UNICODE_BYTE);
        if v {
            *p |= 1;
        } else {
            *p &= !1;
        }
    }
}

extern "C" {
    /// `__crt_lowio_handle_data* __pioinfo[IOINFO_ARRAYS]`
    /// (`sdk/lib/ucrt/lowio/ioinit.cpp`, still C). Same symbol
    /// `stdio_layout.rs` declares; redeclared here because that one is
    /// private to its module.
    static __pioinfo: [*mut u8; 128];
}

// ── small helpers ────────────────────────────────────────────────────────────

/// `_VALIDATE_RETURN(expr, EINVAL, retexpr)`'s failure arm, `errno_t` shape.
#[cold]
unsafe fn invalid_parameter_errno(err: c_int) -> c_int {
    *_errno() = err;
    _invalid_parameter_noinfo();
    err
}

/// Same, but the C's call site wants `DWORD(-1)` back rather than an errno.
#[cold]
unsafe fn invalid_parameter_dword_neg1() -> u32 {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
    u32::MAX
}

// ── flag decoding (`decode_access_flags`, `decode_open_create_flags`,
//    `decode_sharing_flags`, `is_text_mode`, `decode_options`) ─────────────

struct FileOptions {
    crt_flags: u8,
    access: u32,
    create: u32,
    share: u32,
    attributes: u32,
    flags: u32,
}

unsafe fn decode_access_flags(oflag: c_int) -> u32 {
    match oflag & (O_RDONLY | O_WRONLY | O_RDWR) {
        O_RDONLY => GENERIC_READ,
        O_WRONLY => {
            // Append mode needs read access too, to read the BOM.
            if (oflag & O_APPEND) != 0 && (oflag & (O_WTEXT | O_U16TEXT | O_U8TEXT)) != 0 {
                GENERIC_READ | GENERIC_WRITE
            } else {
                GENERIC_WRITE
            }
        }
        O_RDWR => GENERIC_READ | GENERIC_WRITE,
        _ => invalid_parameter_dword_neg1(),
    }
}

unsafe fn decode_open_create_flags(oflag: c_int) -> u32 {
    match oflag & (O_CREAT | O_EXCL | O_TRUNC) {
        0 | O_EXCL => OPEN_EXISTING,
        O_CREAT => OPEN_ALWAYS,
        v if v == O_CREAT | O_EXCL || v == O_CREAT | O_TRUNC | O_EXCL => CREATE_NEW,
        v if v == O_TRUNC || v == O_TRUNC | O_EXCL => TRUNCATE_EXISTING,
        v if v == O_CREAT | O_TRUNC => CREATE_ALWAYS,
        _ => invalid_parameter_dword_neg1(),
    }
}

unsafe fn decode_sharing_flags(shflag: c_int, access: u32) -> u32 {
    match shflag {
        SH_DENYRW => 0,
        SH_DENYWR => FILE_SHARE_READ,
        SH_DENYRD => FILE_SHARE_WRITE,
        SH_DENYNO => FILE_SHARE_READ | FILE_SHARE_WRITE,
        SH_SECURE => {
            if access == GENERIC_READ {
                FILE_SHARE_READ
            } else {
                0
            }
        }
        _ => invalid_parameter_dword_neg1(),
    }
}

unsafe fn is_text_mode(oflag: c_int) -> bool {
    if oflag & O_BINARY != 0 {
        return false;
    }
    if oflag & (O_TEXT | O_WTEXT | O_U16TEXT | O_U8TEXT) != 0 {
        return true;
    }
    let mut fmode: c_int = 0;
    errcheck(_get_fmode(&mut fmode));
    fmode != O_BINARY
}

/// `_ERRCHECK(e)`.
#[inline]
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson_stub();
    }
}

/// `_invoke_watson(nullptr, nullptr, nullptr, 0, 0)` -- declared as an
/// `extern` to the (feature-independent) real symbol rather than called by
/// name directly, so this file links whether `invalid-parameter` is enabled
/// in this crate or still served by the vendored C.
unsafe fn _invoke_watson_stub() -> ! {
    extern "C" {
        fn _invoke_watson(
            expression: *const u16,
            function_name: *const u16,
            file_name: *const u16,
            line_number: u32,
            reserved: usize,
        ) -> !;
    }
    _invoke_watson(null(), null(), null(), 0, 0)
}

unsafe fn decode_options(oflag: c_int, shflag: c_int, pmode: c_int) -> FileOptions {
    let access = decode_access_flags(oflag);
    let create = decode_open_create_flags(oflag);
    let share = decode_sharing_flags(shflag, access);
    let mut attributes = FILE_ATTRIBUTE_NORMAL;
    let mut flags = 0u32;
    let mut crt_flags = 0u8;

    if oflag & O_NOINHERIT != 0 {
        crt_flags |= FNOINHERIT;
    }
    if is_text_mode(oflag) {
        crt_flags |= FTEXT;
    }
    if oflag & O_CREAT != 0 {
        if (pmode & !_umaskval) & S_IWRITE == 0 {
            attributes = FILE_ATTRIBUTE_READONLY;
        }
    }

    let mut access = access;
    let mut share = share;
    if oflag & O_TEMPORARY != 0 {
        flags |= FILE_FLAG_DELETE_ON_CLOSE;
        access |= DELETE;
        share |= FILE_SHARE_DELETE;
    }
    if oflag & O_SHORT_LIVED != 0 {
        attributes |= FILE_ATTRIBUTE_TEMPORARY;
    }
    if oflag & O_OBTAIN_DIR != 0 {
        flags |= FILE_FLAG_BACKUP_SEMANTICS;
    }
    if oflag & O_SEQUENTIAL != 0 {
        flags |= FILE_FLAG_SEQUENTIAL_SCAN;
    } else if oflag & O_RANDOM != 0 {
        flags |= FILE_FLAG_RANDOM_ACCESS;
    }

    FileOptions { crt_flags, access, create, share, attributes, flags }
}

unsafe fn create_file(path: *const u16, sa: *mut SecurityAttributes, o: &FileOptions) -> Handle {
    CreateFileW(path, o.access, o.share, sa, o.create, o.flags | o.attributes, null_mut())
}

// ── BOM / text-mode configuration ───────────────────────────────────────────

unsafe fn truncate_ctrl_z_if_present(fh: c_int) -> c_int {
    if handle::osfile(fh) & (FDEV | FPIPE) != 0 {
        return 0;
    }
    if handle::osfile(fh) & FTEXT == 0 {
        return 0;
    }

    let last_char_position = _lseeki64_nolock(fh, -1, SEEK_END);
    if last_char_position == -1 {
        extern "C" {
            fn __doserrno() -> *mut u32;
        }
        if *__doserrno() == ERROR_NEGATIVE_SEEK {
            return 0;
        }
        return *_errno();
    }

    let mut c: u16 = 0;
    if _read_nolock(fh, &mut c as *mut u16 as *mut c_void, 1) == 0 && c == 26 {
        if _chsize_nolock(fh, last_char_position) == -1 {
            return *_errno();
        }
    }

    if _lseeki64_nolock(fh, 0, SEEK_SET) == -1 {
        return *_errno();
    }
    0
}

unsafe fn configure_text_mode(
    fh: c_int,
    options: &FileOptions,
    mut oflag: c_int,
) -> Result<u8, c_int> {
    let mut text_mode = TEXTMODE_ANSI;

    if handle::osfile(fh) & FTEXT == 0 {
        return Ok(text_mode);
    }

    let text_mode_mask = O_TEXT | O_WTEXT | O_U16TEXT | O_U8TEXT;
    if oflag & text_mode_mask == 0 {
        let mut fmode: c_int = 0;
        errcheck(_get_fmode(&mut fmode));
        if fmode & text_mode_mask == 0 {
            oflag |= O_TEXT;
        } else {
            oflag |= fmode & text_mode_mask;
        }
    }

    match oflag & text_mode_mask {
        O_TEXT => text_mode = TEXTMODE_ANSI,
        v if v == O_WTEXT || v == (O_WTEXT | O_TEXT) => {
            if (oflag & (O_WRONLY | O_CREAT | O_TRUNC)) == (O_WRONLY | O_CREAT | O_TRUNC) {
                text_mode = TEXTMODE_UTF16LE;
            }
        }
        v if v == O_U16TEXT || v == (O_U16TEXT | O_TEXT) => text_mode = TEXTMODE_UTF16LE,
        v if v == O_U8TEXT || v == (O_U8TEXT | O_TEXT) => text_mode = TEXTMODE_UTF8,
        _ => {}
    }

    if oflag & (O_WTEXT | O_U16TEXT | O_U8TEXT) == 0 {
        return Ok(text_mode);
    }
    if options.crt_flags & FDEV != 0 {
        return Ok(text_mode);
    }

    let mut check_bom = false;
    let mut write_bom = false;
    match options.access & GENERIC_READ_WRITE {
        GENERIC_READ => check_bom = true,
        v if v == GENERIC_WRITE || v == GENERIC_READ_WRITE => match options.create {
            OPEN_EXISTING | OPEN_ALWAYS => {
                if _lseeki64_nolock(fh, 0, SEEK_END) != 0 {
                    if _lseeki64_nolock(fh, 0, SEEK_SET) == -1 {
                        return Err(*_errno());
                    }
                    check_bom = (options.access & GENERIC_READ) != 0;
                } else {
                    write_bom = true;
                }
            }
            CREATE_NEW | CREATE_ALWAYS | TRUNCATE_EXISTING => {
                write_bom = true;
            }
            _ => {}
        },
        _ => {}
    }

    if check_bom {
        let mut bom: i32 = 0;
        let count = _read_nolock(fh, &mut bom as *mut i32 as *mut c_void, UTF8_BOMLEN);

        match count {
            -1 => return Err(*_errno()),
            c if c as u32 == UTF8_BOMLEN && bom == UTF8_BOM => {
                text_mode = TEXTMODE_UTF8;
            }
            c if c as u32 == UTF8_BOMLEN || c as u32 == UTF16_BOMLEN => {
                // Fall-through case in the C: a 3-byte read that isn't a
                // UTF-8 BOM is re-examined as a UTF-16 BOM, same as a
                // genuine 2-byte read.
                if (bom & BOM_MASK) == UTF16BE_BOM {
                    *_errno() = EINVAL;
                    return Err(EINVAL);
                }
                if (bom & BOM_MASK) == UTF16LE_BOM {
                    if _lseeki64_nolock(fh, UTF16_BOMLEN as i64, SEEK_SET) == -1 {
                        return Err(*_errno());
                    }
                    text_mode = TEXTMODE_UTF16LE;
                } else if _lseeki64_nolock(fh, 0, SEEK_SET) == -1 {
                    return Err(*_errno());
                }
            }
            _ => {
                if _lseeki64_nolock(fh, 0, SEEK_SET) == -1 {
                    return Err(*_errno());
                }
            }
        }
    }

    if write_bom {
        let (bom, bom_length): (i32, u32) = match text_mode {
            TEXTMODE_UTF16LE => (UTF16LE_BOM, UTF16_BOMLEN),
            TEXTMODE_UTF8 => (UTF8_BOM, UTF8_BOMLEN),
            _ => (0, 0),
        };

        let mut total_written: u32 = 0;
        while bom_length > total_written {
            let bom_begin = (&bom as *const i32 as *const u8).add(total_written as usize);
            let written = _write(fh, bom_begin as *const c_void, bom_length - total_written);
            if written == -1 {
                return Err(*_errno());
            }
            total_written += written as u32;
        }
    }

    Ok(text_mode)
}

// ── `_wsopen_nolock` / `_sopen_nolock` ──────────────────────────────────────

unsafe fn wsopen_nolock(
    unlock_flag: *mut c_int,
    pfh: *mut c_int,
    path: *const u16,
    oflag: c_int,
    shflag: c_int,
    pmode: c_int,
) -> c_int {
    let options = decode_options(oflag, shflag, pmode);
    if options.share == u32::MAX {
        extern "C" {
            fn __doserrno() -> *mut u32;
        }
        *__doserrno() = 0;
        *pfh = -1;
        return *_errno();
    }

    *pfh = _alloc_osfhnd();
    if *pfh == -1 {
        extern "C" {
            fn __doserrno() -> *mut u32;
        }
        *__doserrno() = 0;
        *pfh = -1;
        *_errno() = CRT_EMFILE;
        return CRT_EMFILE;
    }

    // From this point on, `*pfh` must not change: the caller unlocks it.
    *unlock_flag = 1;

    let mut sa = SecurityAttributes {
        n_length: core::mem::size_of::<SecurityAttributes>() as u32,
        lp_security_descriptor: null_mut(),
        b_inherit_handle: (oflag & O_NOINHERIT == 0) as i32,
    };

    let mut options = options;
    let mut os_handle = create_file(path, &mut sa, &options);
    if os_handle as isize == INVALID_HANDLE_VALUE
        && (options.access & GENERIC_READ_WRITE) == GENERIC_READ_WRITE
        && (oflag & O_WRONLY) != 0
    {
        options.access &= !GENERIC_READ;
        os_handle = create_file(path, &mut sa, &options);
    }

    if os_handle as isize == INVALID_HANDLE_VALUE {
        handle::clear_osfile_bits(*pfh, FOPEN);
        __acrt_errno_map_os_error(GetLastError());
        return *_errno();
    }

    let file_type = GetFileType(os_handle);
    if file_type == FILE_TYPE_UNKNOWN {
        let last_error = GetLastError();
        __acrt_errno_map_os_error(last_error);
        handle::clear_osfile_bits(*pfh, FOPEN);
        CloseHandle(os_handle);
        if last_error == 0 {
            *_errno() = CRT_EACCES;
        }
        return *_errno();
    }

    if file_type == FILE_TYPE_CHAR {
        options.crt_flags |= FDEV;
    } else if file_type == FILE_TYPE_PIPE {
        options.crt_flags |= FPIPE;
    }

    __acrt_lowio_set_os_handle(*pfh, os_handle as isize);

    options.crt_flags |= FOPEN;
    handle::set_osfile(*pfh, options.crt_flags);

    handle::set_textmode(*pfh, TEXTMODE_ANSI);

    if oflag & O_RDWR != 0 {
        let result = truncate_ctrl_z_if_present(*pfh);
        if result != 0 {
            _close_nolock(*pfh);
            return result;
        }
    }

    let text_mode = match configure_text_mode(*pfh, &options, oflag) {
        Ok(m) => m,
        Err(e) => {
            _close_nolock(*pfh);
            return e;
        }
    };

    handle::set_textmode(*pfh, text_mode);
    handle::set_tm_unicode(*pfh, oflag & O_WTEXT != 0);

    if (options.crt_flags & (FDEV | FPIPE)) == 0 && (oflag & O_APPEND) != 0 {
        handle::set_osfile_bits(*pfh, FAPPEND);
    }

    if (options.access & GENERIC_READ_WRITE) == GENERIC_READ_WRITE && (oflag & O_WRONLY) != 0 {
        CloseHandle(os_handle);
        options.access &= !GENERIC_READ;
        os_handle = create_file(path, &mut sa, &options);

        if os_handle as isize == INVALID_HANDLE_VALUE {
            __acrt_errno_map_os_error(GetLastError());
            handle::clear_osfile_bits(*pfh, FOPEN);
            _free_osfhnd(*pfh);
            return *_errno();
        } else {
            handle::set_osfhnd(*pfh, os_handle as isize);
        }
    }

    0
}

unsafe fn sopen_nolock(
    unlock_flag: *mut c_int,
    pfh: *mut c_int,
    path: *const c_char,
    oflag: c_int,
    shflag: c_int,
    pmode: c_int,
) -> c_int {
    let cp = file_codepage();
    match mbs_to_wcs(path as *const u8, cp) {
        Ok(Some(wide)) => wsopen_nolock(unlock_flag, pfh, wide.0, oflag, shflag, pmode),
        Ok(None) => {
            // NULL path: `_VALIDATE_RETURN_ERRCODE(path != nullptr, EINVAL)`
            // fires later on the wide side too, but `__acrt_mbs_to_wcs_cp`
            // itself treats a NULL input as success (`Ok(None)`) -- the
            // narrow path forwards it through as a NULL wide pointer so the
            // SAME validation the wide functions already do is what catches
            // it, not a check duplicated here.
            wsopen_nolock(unlock_flag, pfh, null(), oflag, shflag, pmode)
        }
        // `if (cvt != 0) return -1;` -- verbatim, see the module doc.
        Err(_) => -1,
    }
}

// ── the eight `open.cpp` exports, minus the four variadic ones ─────────────

/// `_sopen_dispatch`. `common_sopen_dispatch<char>`.
#[no_mangle]
pub unsafe extern "C" fn _sopen_dispatch(
    path: *const c_char,
    oflag: c_int,
    shflag: c_int,
    pmode: c_int,
    pfh: *mut c_int,
    secure: c_int,
) -> c_int {
    if pfh.is_null() {
        return invalid_parameter_errno(EINVAL);
    }
    *pfh = -1;
    if path.is_null() {
        return invalid_parameter_errno(EINVAL);
    }
    if secure != 0 && (pmode & !((S_IREAD | S_IWRITE))) != 0 {
        return invalid_parameter_errno(EINVAL);
    }

    let mut unlock_flag: c_int = 0;
    let error_code = sopen_nolock(&mut unlock_flag, pfh, path, oflag, shflag, pmode);
    if unlock_flag != 0 {
        if error_code != 0 {
            handle::clear_osfile_bits(*pfh, FOPEN);
        }
        __acrt_lowio_unlock_fh(*pfh);
    }

    if error_code != 0 {
        *pfh = -1;
    }
    error_code
}

/// `_wsopen_dispatch`. `common_sopen_dispatch<wchar_t>`.
#[no_mangle]
pub unsafe extern "C" fn _wsopen_dispatch(
    path: *const u16,
    oflag: c_int,
    shflag: c_int,
    pmode: c_int,
    pfh: *mut c_int,
    secure: c_int,
) -> c_int {
    if pfh.is_null() {
        return invalid_parameter_errno(EINVAL);
    }
    *pfh = -1;
    if path.is_null() {
        return invalid_parameter_errno(EINVAL);
    }
    if secure != 0 && (pmode & !((S_IREAD | S_IWRITE))) != 0 {
        return invalid_parameter_errno(EINVAL);
    }

    let mut unlock_flag: c_int = 0;
    let error_code = wsopen_nolock(&mut unlock_flag, pfh, path, oflag, shflag, pmode);
    if unlock_flag != 0 {
        if error_code != 0 {
            handle::clear_osfile_bits(*pfh, FOPEN);
        }
        __acrt_lowio_unlock_fh(*pfh);
    }

    if error_code != 0 {
        *pfh = -1;
    }
    error_code
}

#[no_mangle]
pub unsafe extern "C" fn _sopen_s(
    pfh: *mut c_int,
    path: *const c_char,
    oflag: c_int,
    shflag: c_int,
    pmode: c_int,
) -> c_int {
    _sopen_dispatch(path, oflag, shflag, pmode, pfh, 1)
}

#[no_mangle]
pub unsafe extern "C" fn _wsopen_s(
    pfh: *mut c_int,
    path: *const u16,
    oflag: c_int,
    shflag: c_int,
    pmode: c_int,
) -> c_int {
    _wsopen_dispatch(path, oflag, shflag, pmode, pfh, 1)
}

// ── `common_open<char>` / `common_open<wchar_t>`, and the four `_v` twins
//    the `varargs-rust` shim (`dll/win32/ucrtbase/ucrt_varargs_shim.c`) calls
//    from `_open`/`_wopen`/`_sopen`/`_wsopen` ─────────────────────────────

/// `common_open<char>`.
unsafe fn common_open_narrow(path: *const c_char, oflag: c_int, pmode: c_int) -> c_int {
    if path.is_null() {
        invalid_parameter_errno(EINVAL);
        return -1;
    }
    let mut fh: c_int = -1;
    let mut unlock_flag: c_int = 0;
    let error_code = sopen_nolock(&mut unlock_flag, &mut fh, path, oflag, SH_DENYNO, pmode);
    if unlock_flag != 0 {
        if error_code != 0 {
            handle::clear_osfile_bits(fh, FOPEN);
        }
        __acrt_lowio_unlock_fh(fh);
    }
    if error_code != 0 {
        *_errno() = error_code;
        return -1;
    }
    fh
}

/// `common_open<wchar_t>`.
unsafe fn common_open_wide(path: *const u16, oflag: c_int, pmode: c_int) -> c_int {
    if path.is_null() {
        invalid_parameter_errno(EINVAL);
        return -1;
    }
    let mut fh: c_int = -1;
    let mut unlock_flag: c_int = 0;
    let error_code = wsopen_nolock(&mut unlock_flag, &mut fh, path, oflag, SH_DENYNO, pmode);
    if unlock_flag != 0 {
        if error_code != 0 {
            handle::clear_osfile_bits(fh, FOPEN);
        }
        __acrt_lowio_unlock_fh(fh);
    }
    if error_code != 0 {
        *_errno() = error_code;
        return -1;
    }
    fh
}

/// `_open`'s body, minus the `va_start`/`va_end` the shim already did.
/// `ap` is `&ap` of the shim's started `va_list`; `pmode` is read
/// UNCONDITIONALLY, exactly as the C's `va_arg(arglist, int)` is -- there is
/// no `_O_CREAT` gate here or in the C.
#[no_mangle]
pub unsafe extern "C" fn _open_v(path: *const c_char, oflag: c_int, ap: *mut c_void) -> c_int {
    let mut va = VaArgs::from_ptr(ap);
    let pmode: i32 = va.arg::<i32>();
    common_open_narrow(path, oflag, pmode)
}

/// `_wopen`'s body.
#[no_mangle]
pub unsafe extern "C" fn _wopen_v(path: *const u16, oflag: c_int, ap: *mut c_void) -> c_int {
    let mut va = VaArgs::from_ptr(ap);
    let pmode: i32 = va.arg::<i32>();
    common_open_wide(path, oflag, pmode)
}

/// `_sopen`'s body: pulls `pmode`, then calls `_sopen_dispatch(path, oflag,
/// shflag, pmode, &fh, /* secure = */ FALSE)` and returns `fh` (or -1) --
/// NOT the dispatch's own `errno_t`, exactly as the C.
#[no_mangle]
pub unsafe extern "C" fn _sopen_v(
    path: *const c_char,
    oflag: c_int,
    shflag: c_int,
    ap: *mut c_void,
) -> c_int {
    let mut va = VaArgs::from_ptr(ap);
    let pmode: i32 = va.arg::<i32>();
    let mut fh: c_int = -1;
    let result = _sopen_dispatch(path, oflag, shflag, pmode, &mut fh, 0);
    if result != 0 {
        -1
    } else {
        fh
    }
}

/// `_wsopen`'s body, symmetric to [`_sopen_v`].
#[no_mangle]
pub unsafe extern "C" fn _wsopen_v(
    path: *const u16,
    oflag: c_int,
    shflag: c_int,
    ap: *mut c_void,
) -> c_int {
    let mut va = VaArgs::from_ptr(ap);
    let pmode: i32 = va.arg::<i32>();
    let mut fh: c_int = -1;
    let result = _wsopen_dispatch(path, oflag, shflag, pmode, &mut fh, 0);
    if result != 0 {
        -1
    } else {
        fh
    }
}

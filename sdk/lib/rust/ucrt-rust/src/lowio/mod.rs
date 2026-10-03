//! The lowio handle table: `lowio/ioinit.cpp` and `lowio/osfinfo.cpp`, ported,
//! plus the header-inline machinery every other lowio file uses (`_pioinfo`,
//! `_osfile`, `__acrt_lowio_lock_fh_and_call`, the `_CHECK_FH_*` /
//! `_VALIDATE_CLEAR_OSSERR_*` macros).
//!
//! The rest of `lowio/` is ported as submodules over this one. They call one
//! another through the C names, like stdio's pieces do, so a caller still in C
//! (conio's `pipe.cpp`, exec) links against the same symbols it always did.
//!
//! Details that are silent if wrong:
//! * `__pioinfo`, `_nhandle` and `__badioinfo` keep their C names and types:
//!   `stdio_layout.rs`, `lowio_open.rs` and stdio read `__pioinfo` by name.
//! * `__acrt_lowio_ensure_fh_exists` is called by `__acrt_initialize_lowio` with
//!   the index lock already held. The CRT locks are critical sections, so the
//!   recursion is fine; do not "fix" it with a `_nolock` split.
//! * A freshly allocated handle from `_alloc_osfhnd` is returned LOCKED; callers
//!   unlock it when they have filled it in.
//! * `__badioinfo.osfile` is `FTEXT`, not 0: `_osfile_safe(-1) & FTEXT` is how
//!   stdio's text-mode checks treat a bad handle.

use crate::stdio_layout::CriticalSection;
use core::ffi::{c_int, c_long, c_ulong, c_void};
use core::ptr::{addr_of_mut, null_mut};

/// `__crt_lowio_handle_data`.
#[repr(C)]
pub struct HandleData {
    pub lock: CriticalSection,
    /// The OS `HANDLE`.
    pub osfhnd: isize,
    /// The file position that matches the start of the stdio buffer.
    pub startpos: i64,
    pub osfile: u8,
    /// `__crt_lowio_text_mode`: ansi 0, utf8 1, utf16le 2.
    pub textmode: u8,
    pub pipe_lookahead: [u8; 3],
    /// `unicode:1, utf8translations:1, dbcsBufferUsed:1`.
    pub bits: u8,
    pub mb_buffer: [u8; MB_LEN_MAX],
}

const WIDE: bool = cfg!(target_pointer_width = "64");
const _: () = {
    // The same offsets stdio_layout.rs measured against the build's compiler.
    assert!(core::mem::size_of::<HandleData>() == if WIDE { 72 } else { 56 });
    assert!(core::mem::offset_of!(HandleData, osfhnd) == if WIDE { 40 } else { 24 });
    assert!(core::mem::offset_of!(HandleData, startpos) == if WIDE { 48 } else { 32 });
    assert!(core::mem::offset_of!(HandleData, osfile) == if WIDE { 56 } else { 40 });
    assert!(core::mem::offset_of!(HandleData, textmode) == if WIDE { 57 } else { 41 });
    assert!(core::mem::offset_of!(HandleData, bits) == if WIDE { 61 } else { 45 });
};

impl HandleData {
    pub const BIT_UNICODE: u8 = 0x1;
    pub const BIT_UTF8_TRANSLATIONS: u8 = 0x2;
    pub const BIT_DBCS_BUFFER_USED: u8 = 0x4;

    pub fn unicode(&self) -> bool {
        self.bits & Self::BIT_UNICODE != 0
    }
    pub fn set_bit(&mut self, bit: u8, on: bool) {
        if on {
            self.bits |= bit;
        } else {
            self.bits &= !bit;
        }
    }
}

pub const MB_LEN_MAX: usize = 5;

/// `_osfile` flag bits (corecrt_internal_lowio.h).
pub const FOPEN: u8 = 0x01;
pub const FEOFLAG: u8 = 0x02;
pub const FCRLF: u8 = 0x04;
pub const FPIPE: u8 = 0x08;
pub const FNOINHERIT: u8 = 0x10;
pub const FAPPEND: u8 = 0x20;
pub const FDEV: u8 = 0x40;
pub const FTEXT: u8 = 0x80;

/// `__crt_lowio_text_mode`.
pub const TEXT_MODE_ANSI: u8 = 0;
pub const TEXT_MODE_UTF8: u8 = 1;
pub const TEXT_MODE_UTF16LE: u8 = 2;

pub const LF: u8 = 10;
pub const CR: u8 = 13;
pub const CTRLZ: u8 = 26;

pub const IOINFO_L2E: c_int = 6;
pub const IOINFO_ARRAY_ELTS: c_int = 1 << IOINFO_L2E;
pub const IOINFO_ARRAYS: usize = 128;
/// `_NHANDLE_`.
pub const NHANDLE: c_int = IOINFO_ARRAYS as c_int * IOINFO_ARRAY_ELTS;
pub const STDIO_HANDLES_COUNT: c_int = 3;
/// `_NO_CONSOLE_FILENO`.
pub const NO_CONSOLE_FILENO: isize = -2;
pub const INVALID_HANDLE_VALUE: isize = -1;

/// `__acrt_lowio_index_lock`.
pub const LOWIO_INDEX_LOCK: c_int = 7;
const CORECRT_SPINCOUNT: u32 = 4000;

pub const EBADF: c_int = 9;
pub const ENOMEM: c_int = 12;
pub const EACCES: c_int = 13;
pub const EINVAL: c_int = 22;
pub const EMFILE: c_int = 24;
pub const ENOSPC: c_int = 28;

const STD_INPUT_HANDLE: u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
const STD_ERROR_HANDLE: u32 = -12i32 as u32;
const FILE_TYPE_UNKNOWN: u32 = 0;
const FILE_TYPE_CHAR: u32 = 2;
const FILE_TYPE_PIPE: u32 = 3;
const CRT_CONSOLE_APP: c_int = 1;

/// `_O_TEXT` and friends (fcntl.h), for `_open_osfhandle`.
const O_APPEND: c_int = 0x0008;
const O_TEXT: c_int = 0x4000;
const O_NOINHERIT: c_int = 0x0080;

extern "C" {
    fn _calloc_base(count: usize, size: usize) -> *mut c_void;
    fn _free_base(p: *mut c_void);
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    pub fn _errno() -> *mut c_int;
    pub fn __doserrno() -> *mut c_ulong;
    pub fn _invalid_parameter_noinfo();
    pub fn __acrt_errno_map_os_error(oserrno: c_ulong);
    fn _query_app_type() -> c_int;
}

extern "system" {
    fn __acrt_InitializeCriticalSectionEx(cs: *mut CriticalSection, spin_count: u32, flags: u32) -> c_int;
    fn EnterCriticalSection(cs: *mut CriticalSection);
    fn LeaveCriticalSection(cs: *mut CriticalSection);
    fn DeleteCriticalSection(cs: *mut CriticalSection);
    fn GetFileType(handle: *mut c_void) -> u32;
    fn GetLastError() -> c_ulong;
    fn GetStdHandle(id: u32) -> *mut c_void;
    fn SetStdHandle(id: u32, handle: *mut c_void) -> c_int;
    fn GetStartupInfoW(info: *mut StartupInfoW);
}

/// `STARTUPINFOW`, for its `cbReserved2` / `lpReserved2` inheritance block.
#[repr(C)]
struct StartupInfoW {
    cb: u32,
    reserved: *mut u16,
    desktop: *mut u16,
    title: *mut u16,
    x: u32,
    y: u32,
    x_size: u32,
    y_size: u32,
    x_count_chars: u32,
    y_count_chars: u32,
    fill_attribute: u32,
    flags: u32,
    show_window: u16,
    cb_reserved2: u16,
    lp_reserved2: *mut u8,
    std_input: isize,
    std_output: isize,
    std_error: isize,
}
const _: () = assert!(core::mem::size_of::<StartupInfoW>() == if WIDE { 104 } else { 68 });

// ── ioinit.cpp: the table ───────────────────────────────────────────────────

/// `__badioinfo`: what `_pioinfo_safe` answers for handles -1 and -2.
#[no_mangle]
pub static mut __badioinfo: HandleData = HandleData {
    lock: CriticalSection::ZERO,
    osfhnd: -1,
    startpos: 0,
    osfile: FTEXT,
    textmode: TEXT_MODE_ANSI,
    pipe_lookahead: [LF, LF, LF],
    bits: 0,
    mb_buffer: [0; MB_LEN_MAX],
};

/// `int _nhandle`: every `fh` in `[0, _nhandle)` has a record.
#[no_mangle]
pub static mut _nhandle: c_int = 0;

/// `__crt_lowio_handle_data* __pioinfo[IOINFO_ARRAYS]`.
#[no_mangle]
pub static mut __pioinfo: [*mut HandleData; IOINFO_ARRAYS] = [null_mut(); IOINFO_ARRAYS];

/// `_pioinfo(fh)`: unchecked, for a handle already known to be `< _nhandle`.
pub unsafe fn pioinfo(fh: c_int) -> *mut HandleData {
    __pioinfo[(fh >> IOINFO_L2E) as usize].add((fh & (IOINFO_ARRAY_ELTS - 1)) as usize)
}

/// `_pioinfo_safe(fh)`: `__badioinfo` for -1 and -2, no other range check.
pub unsafe fn pioinfo_safe(fh: c_int) -> *mut HandleData {
    if fh == -1 || fh == -2 {
        return addr_of_mut!(__badioinfo);
    }
    pioinfo(fh)
}

pub unsafe fn osfile(fh: c_int) -> u8 {
    (*pioinfo(fh)).osfile
}
pub unsafe fn osfhnd(fh: c_int) -> isize {
    (*pioinfo(fh)).osfhnd
}

/// `fh >= 0 && (unsigned)fh < (unsigned)_nhandle`.
pub unsafe fn in_range(fh: c_int) -> bool {
    fh >= 0 && (fh as u32) < (_nhandle as u32)
}

/// `fh` is in range and `FOPEN`: the three-line validation most of lowio opens with.
pub unsafe fn is_open(fh: c_int) -> bool {
    in_range(fh) && osfile(fh) & FOPEN != 0
}

pub unsafe fn set_errno(e: c_int) {
    *_errno() = e;
}
pub unsafe fn set_doserrno(e: c_ulong) {
    *__doserrno() = e;
}

/// The failure arm of `_VALIDATE_CLEAR_OSSERR_RETURN`: `_doserrno = 0`,
/// `errno = e`, the invalid-parameter handler. The caller returns.
pub unsafe fn invalid_clear_oserr(e: c_int) {
    set_doserrno(0);
    set_errno(e);
    _invalid_parameter_noinfo();
}

/// The failure arm of `_VALIDATE_RETURN`: errno, then the handler.
pub unsafe fn invalid(e: c_int) {
    set_errno(e);
    _invalid_parameter_noinfo();
}

/// `_CHECK_FH_CLEAR_OSSERR_RETURN`'s condition and effect: `true` means the
/// caller returns, having had `_doserrno = 0; errno = e` (no handler).
pub unsafe fn no_console_fh(fh: c_int, e: c_int) -> bool {
    if fh as isize == NO_CONSOLE_FILENO {
        set_doserrno(0);
        set_errno(e);
        return true;
    }
    false
}

/// `__acrt_lowio_lock_fh_and_call`: `action` with the handle locked. The C
/// guards the unlock with SEH; this build's SEH is the dummy one, so the unlock
/// is simply sequenced after.
pub unsafe fn with_fh_locked<R>(fh: c_int, action: impl FnOnce() -> R) -> R {
    __acrt_lowio_lock_fh(fh);
    let r = action();
    __acrt_lowio_unlock_fh(fh);
    r
}

unsafe fn with_index_lock<R>(action: impl FnOnce() -> R) -> R {
    __acrt_lock(LOWIO_INDEX_LOCK);
    let r = action();
    __acrt_unlock(LOWIO_INDEX_LOCK);
    r
}

fn std_handle_id(fh: c_int) -> u32 {
    match fh {
        0 => STD_INPUT_HANDLE,
        1 => STD_OUTPUT_HANDLE,
        _ => STD_ERROR_HANDLE,
    }
}

/// Copies the parent's `_osfile` / `_osfhnd` pairs out of `lpReserved2`, the
/// block `_spawn*` / `_exec*` hand a child.
unsafe fn initialize_inherited_file_handles_nolock() {
    let mut startup_info: StartupInfoW = core::mem::zeroed();
    GetStartupInfoW(&mut startup_info);
    if startup_info.cb_reserved2 == 0 || startup_info.lp_reserved2.is_null() {
        return;
    }
    let block = startup_info.lp_reserved2;
    let handle_count = (block as *const c_int).read_unaligned();
    let first_file = block.add(core::mem::size_of::<c_int>());
    let first_handle = first_file.offset(handle_count as isize) as *const isize;

    let mut handles_to_inherit = if handle_count < NHANDLE { handle_count } else { NHANDLE };
    // Allocate what is needed; if that fails, inherit what fits.
    let _ = __acrt_lowio_ensure_fh_exists(handles_to_inherit);
    if handles_to_inherit > _nhandle {
        handles_to_inherit = _nhandle;
    }
    for fh in 0..handles_to_inherit {
        let file = *first_file.add(fh as usize);
        let handle = first_handle.add(fh as usize).read_unaligned();
        if handle == INVALID_HANDLE_VALUE || handle == NO_CONSOLE_FILENO {
            continue;
        }
        if file & FOPEN == 0 {
            continue;
        }
        // A pipe's handle may be one the child cannot query yet; trust the flag.
        if file & FPIPE == 0 && GetFileType(handle as *mut c_void) == FILE_TYPE_UNKNOWN {
            continue;
        }
        let pio = pioinfo(fh);
        (*pio).osfhnd = handle;
        (*pio).osfile = file;
    }
}

unsafe fn initialize_stdio_handles_nolock() {
    for fh in 0..STDIO_HANDLES_COUNT {
        let pio = pioinfo(fh);
        // Inherited: just force text mode.
        if (*pio).osfhnd != INVALID_HANDLE_VALUE && (*pio).osfhnd != NO_CONSOLE_FILENO {
            (*pio).osfile |= FTEXT;
            continue;
        }
        (*pio).osfile = FOPEN | FTEXT;
        let os_handle = GetStdHandle(std_handle_id(fh)) as isize;
        let is_valid_handle = os_handle != INVALID_HANDLE_VALUE && os_handle != 0;
        let handle_type = if is_valid_handle { GetFileType(os_handle as *mut c_void) } else { FILE_TYPE_UNKNOWN };
        if handle_type != FILE_TYPE_UNKNOWN {
            (*pio).osfhnd = os_handle;
            if handle_type & 0xff == FILE_TYPE_CHAR {
                (*pio).osfile |= FDEV;
            } else if handle_type & 0xff == FILE_TYPE_PIPE {
                (*pio).osfile |= FPIPE;
            }
        } else {
            // No usable handle: a GUI app with no console. Mark it so stdio can
            // tell "no console" from "failed to open".
            (*pio).osfile |= FDEV;
            (*pio).osfhnd = NO_CONSOLE_FILENO;
            let piob = crate::stdio::__piob;
            if !piob.is_null() {
                (**piob.add(fh as usize)).file = NO_CONSOLE_FILENO as c_long;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_lowio() -> bool {
    with_index_lock(|| {
        if __acrt_lowio_ensure_fh_exists(0) != 0 {
            return false;
        }
        initialize_inherited_file_handles_nolock();
        initialize_stdio_handles_nolock();
        true
    })
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_lowio(_terminating: bool) -> bool {
    for i in 0..IOINFO_ARRAYS {
        if __pioinfo[i].is_null() {
            continue;
        }
        __acrt_lowio_destroy_handle_array(__pioinfo[i]);
        __pioinfo[i] = null_mut();
    }
    true
}

// ── osfinfo.cpp: handle allocation ──────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn __acrt_lowio_create_handle_array() -> *mut HandleData {
    let array: *mut HandleData =
        _calloc_base(IOINFO_ARRAY_ELTS as usize, core::mem::size_of::<HandleData>()).cast();
    if array.is_null() {
        return null_mut();
    }
    for i in 0..IOINFO_ARRAY_ELTS as usize {
        let it = array.add(i);
        __acrt_InitializeCriticalSectionEx(addr_of_mut!((*it).lock), CORECRT_SPINCOUNT, 0);
        (*it).osfhnd = INVALID_HANDLE_VALUE;
        (*it).startpos = 0;
        (*it).osfile = 0;
        (*it).textmode = TEXT_MODE_ANSI;
        (*it).pipe_lookahead = [LF, LF, LF];
        (*it).bits = 0;
        (*it).mb_buffer = [0; MB_LEN_MAX];
    }
    array
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_lowio_destroy_handle_array(array: *mut HandleData) {
    if array.is_null() {
        return;
    }
    for i in 0..IOINFO_ARRAY_ELTS as usize {
        DeleteCriticalSection(addr_of_mut!((*array.add(i)).lock));
    }
    _free_base(array.cast());
}

/// Makes sure the record for `fh` exists, allocating arrays up to it.
#[no_mangle]
pub unsafe extern "C" fn __acrt_lowio_ensure_fh_exists(fh: c_int) -> c_int {
    if (fh as u32) >= NHANDLE as u32 {
        invalid(EBADF);
        return EBADF;
    }
    with_index_lock(|| {
        let mut i = 0usize;
        while fh >= _nhandle {
            if __pioinfo[i].is_null() {
                __pioinfo[i] = __acrt_lowio_create_handle_array();
                if __pioinfo[i].is_null() {
                    return ENOMEM;
                }
                _nhandle += IOINFO_ARRAY_ELTS;
            }
            i += 1;
        }
        0
    })
}

/// Finds (or makes room for) an unused handle, marks it `FOPEN` and returns it
/// LOCKED; -1 if the table is full.
#[no_mangle]
pub unsafe extern "C" fn _alloc_osfhnd() -> c_int {
    with_index_lock(|| {
        for i in 0..IOINFO_ARRAYS {
            // The first unallocated array: allocate it and take its first handle.
            if __pioinfo[i].is_null() {
                __pioinfo[i] = __acrt_lowio_create_handle_array();
                if __pioinfo[i].is_null() {
                    return -1;
                }
                _nhandle += IOINFO_ARRAY_ELTS;
                let fh = i as c_int * IOINFO_ARRAY_ELTS;
                __acrt_lowio_lock_fh(fh);
                (*pioinfo(fh)).osfile = FOPEN;
                return fh;
            }
            let first = __pioinfo[i];
            for j in 0..IOINFO_ARRAY_ELTS as usize {
                let pio = first.add(j);
                if (*pio).osfile & FOPEN != 0 {
                    continue;
                }
                // Recheck under the handle's own lock: another thread may have
                // taken it between the test and the lock.
                EnterCriticalSection(addr_of_mut!((*pio).lock));
                if (*pio).osfile & FOPEN != 0 {
                    LeaveCriticalSection(addr_of_mut!((*pio).lock));
                    continue;
                }
                (*pio).osfile = FOPEN;
                (*pio).osfhnd = INVALID_HANDLE_VALUE;
                return i as c_int * IOINFO_ARRAY_ELTS + j as c_int;
            }
        }
        -1
    })
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_lowio_set_os_handle(fh: c_int, value: isize) -> c_int {
    if in_range(fh) && osfhnd(fh) == INVALID_HANDLE_VALUE {
        if _query_app_type() == CRT_CONSOLE_APP && (0..STDIO_HANDLES_COUNT).contains(&fh) {
            SetStdHandle(std_handle_id(fh), value as *mut c_void);
        }
        (*pioinfo(fh)).osfhnd = value;
        return 0;
    }
    set_errno(EBADF);
    set_doserrno(0); // Not an OS error.
    -1
}

#[no_mangle]
pub unsafe extern "C" fn _free_osfhnd(fh: c_int) -> c_int {
    if in_range(fh) && osfile(fh) & FOPEN != 0 && osfhnd(fh) != INVALID_HANDLE_VALUE {
        if _query_app_type() == CRT_CONSOLE_APP && (0..STDIO_HANDLES_COUNT).contains(&fh) {
            SetStdHandle(std_handle_id(fh), null_mut());
        }
        (*pioinfo(fh)).osfhnd = INVALID_HANDLE_VALUE;
        return 0;
    }
    set_errno(EBADF);
    set_doserrno(0);
    -1
}

#[no_mangle]
pub unsafe extern "C" fn _get_osfhandle(fh: c_int) -> isize {
    if no_console_fh(fh, EBADF) {
        return -1;
    }
    if !in_range(fh) || osfile(fh) & FOPEN == 0 {
        invalid_clear_oserr(EBADF);
        return -1;
    }
    osfhnd(fh)
}

#[no_mangle]
pub unsafe extern "C" fn _open_osfhandle(osfhandle: isize, source_flags: c_int) -> c_int {
    let mut file_flags: u8 = 0;
    if source_flags & O_APPEND != 0 {
        file_flags |= FAPPEND;
    }
    if source_flags & O_TEXT != 0 {
        file_flags |= FTEXT;
    }
    if source_flags & O_NOINHERIT != 0 {
        file_flags |= FNOINHERIT;
    }
    let file_type = GetFileType(osfhandle as *mut c_void);
    if file_type == FILE_TYPE_UNKNOWN {
        __acrt_errno_map_os_error(GetLastError());
        return -1;
    }
    if file_type == FILE_TYPE_CHAR {
        file_flags |= FDEV;
    } else if file_type == FILE_TYPE_PIPE {
        file_flags |= FPIPE;
    }
    let fh = _alloc_osfhnd();
    if fh == -1 {
        set_errno(EMFILE);
        set_doserrno(0);
        return -1;
    }
    __acrt_lowio_set_os_handle(fh, osfhandle);
    let pio = pioinfo(fh);
    (*pio).osfile = file_flags | FOPEN;
    (*pio).textmode = TEXT_MODE_ANSI;
    (*pio).set_bit(HandleData::BIT_UNICODE, false);
    __acrt_lowio_unlock_fh(fh);
    fh
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_lowio_lock_fh(fh: c_int) {
    EnterCriticalSection(addr_of_mut!((*pioinfo(fh)).lock));
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_lowio_unlock_fh(fh: c_int) {
    LeaveCriticalSection(addr_of_mut!((*pioinfo(fh)).lock));
}

// ── The rest of lowio/: one submodule per C file or family ──────────────────
// (the porting agent adds `mod` lines here and nowhere else in this file)
mod chsize;
mod close;
mod common;
mod dup;
mod lseek;
mod mktemp;
mod read;
mod setmode;
mod small;
mod write;

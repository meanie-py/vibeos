//! The one-or-two-function files: `misc/getpid.cpp`, `misc/is_wctype.cpp`,
//! `misc/resetstk.cpp`, `misc/set_error_mode.cpp`, `misc/seterrm.cpp`,
//! `misc/slbeep.cpp`, `misc/systime.cpp`.
//!
//! ## Details that are silent if wrong
//!
//! * `_set_error_mode`'s state (`__acrt_error_mode`) is PROCESS-wide (one
//!   plain `static int`, no per-thread/locking), starting at `_OUT_TO_DEFAULT`
//!   (0); `_REPORT_ERRMODE` (3) reads it back without changing it; any other
//!   value is EINVAL through the invalid-parameter handler and returns -1
//!   (which is otherwise indistinguishable from `_OUT_TO_MSGBOX`'s (2) return
//!   value... except -1 is never a valid PRIOR mode, so callers checking for
//!   error must compare against -1, not against the mode constants).
//! * `_seterrormode` (from `seterrm.cpp`, despite the filename's "signal
//!   handling" purpose comment -- a stale header comment in the C itself,
//!   copied here verbatim in the module list above but NOT reproduced as
//!   this module's behaviour) is a THIN, unconditional `SetErrorMode`
//!   passthrough: no validation, no return value.
//! * `_resetstkoflw` NEVER invokes the invalid-parameter handler or touches
//!   errno on any path -- every failure (`VirtualQuery`, insufficient
//!   headroom, `VirtualAlloc`/`VirtualProtect`) is a plain `return 0`.
//! * `_resetstkoflw`'s guard-page size is `max(stack guarantee + 1 page,
//!   2 pages)`, rounded UP to a page boundary if a stack guarantee is set,
//!   BEFORE the `+1 page`/`max(2 pages)` adjustments -- getting the order
//!   wrong changes the final size when the guarantee isn't already
//!   page-aligned.
//! * `_getsystime`/`_setsystime` operate on **local**, not UTC, time
//!   (`GetLocalTime`/`SetLocalTime`) despite the generic "systime" name.
//! * `_getsystime` sets `tm_isdst = -1` (let `mktime` decide) and does NOT
//!   fill `tm_yday`/other derived fields itself -- it calls `_mktime32` on
//!   the result purely to NORMALIZE it (the return value of `_mktime32` is
//!   discarded), then returns the separately-captured milliseconds field
//!   (which `struct tm` has no room for).
//! * `_setsystime` also calls `_mktime32` on the SOURCE `tm` first (again
//!   discarding the return value) purely to normalize it before copying
//!   fields into the `SYSTEMTIME` -- an out-of-range `source` (e.g.
//!   `tm_mon = 13`) is corrected before being sent to `SetLocalTime`, not
//!   rejected.
//! * `_setsystime` returns the RAW `GetLastError()` value on failure
//!   (truncated to `int` in the C, but the function itself returns
//!   `unsigned`), 0 on success -- it never touches `errno`/`_doserrno` at
//!   all.

use core::ffi::{c_int, c_uint, c_ulong, c_void};

// ── getpid.cpp ───────────────────────────────────────────────────────────

extern "system" {
    fn GetCurrentProcessId() -> u32;
}

/// `_getpid`.
#[no_mangle]
pub unsafe extern "C" fn _getpid() -> c_int {
    GetCurrentProcessId() as c_int
}

// ── is_wctype.cpp ────────────────────────────────────────────────────────

extern "C" {
    /// Matches `ctype.rs`'s existing definition of `iswctype` exactly
    /// (`u16, u16 -> c_int`), which is what `wint_t`/`wctype_t` (both
    /// `unsigned short`) actually are.
    fn iswctype(c: u16, mask: u16) -> c_int;
}

/// `is_wctype` -- the obsolete pre-`iswctype` name, kept as a thin alias.
#[no_mangle]
pub unsafe extern "C" fn is_wctype(c: u16, mask: u16) -> c_int {
    iswctype(c, mask)
}

// ── resetstk.cpp ─────────────────────────────────────────────────────────

const MIN_STACK_REQ_WINNT: u32 = 2;
const MEM_COMMIT: u32 = 0x1000;
const PAGE_READWRITE: u32 = 0x04;
const PAGE_GUARD: u32 = 0x100;

/// `MEMORY_BASIC_INFORMATION` (`sdk/include/xdk/winnt_old.h`) -- this tree's
/// headers do NOT have the newer `PartitionId` member some SDKs add.
#[repr(C)]
struct MemoryBasicInformation {
    base_address: *mut c_void,
    allocation_base: *mut c_void,
    allocation_protect: u32,
    region_size: usize,
    state: u32,
    protect: u32,
    ty: u32,
}
// i386: 4+4+4+4+4+4+4 = 28; amd64/arm64: 8+8+4(+4 pad)+8+4+4+4(+4 pad) = 48.
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<MemoryBasicInformation>() == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<MemoryBasicInformation>() == 48);

#[repr(C)]
struct SystemInfo {
    // A `union` of `dwOemId` and the two `wProcessorArchitecture`/reserved
    // WORDs in the real struct; only `dwPageSize` (the next field) is read,
    // so the union's exact interior is irrelevant here -- kept as the same
    // 4 bytes wide as the real union.
    oem_id_union: u32,
    page_size: u32,
    min_app_addr: *mut c_void,
    max_app_addr: *mut c_void,
    active_processor_mask: usize,
    number_of_processors: u32,
    processor_type: u32,
    alloc_granularity: u32,
    processor_level: u16,
    processor_revision: u16,
}

extern "system" {
    fn VirtualQuery(addr: *const c_void, mbi: *mut MemoryBasicInformation, len: usize) -> usize;
    fn GetSystemInfo(si: *mut SystemInfo);
    fn VirtualAlloc(addr: *mut c_void, size: usize, alloc_type: u32, protect: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, new_protect: u32, old_protect: *mut u32) -> c_int;
}

// `BOOL WINAPI __acrt_SetThreadStackGuarantee` (corecrt_internal.h): stdcall on i386.
extern "system" {
    fn __acrt_SetThreadStackGuarantee(size: *mut u32) -> c_int;
}

/// `_resetstkoflw`. The C takes the current stack pointer via `_alloca(1)`;
/// a local stack variable's address is the equivalent Rust has (both are
/// "some address on the current stack frame close to the top", which is all
/// `pStack`'s role here requires -- it is only ever masked down to a page
/// boundary and offset by whole pages, never dereferenced).
#[no_mangle]
pub unsafe extern "C" fn _resetstkoflw() -> c_int {
    let stack_marker: u8 = 0;
    let p_stack = &stack_marker as *const u8 as usize;

    let mut mbi: MemoryBasicInformation = core::mem::zeroed();
    if VirtualQuery(p_stack as *const c_void, &mut mbi, core::mem::size_of::<MemoryBasicInformation>()) == 0 {
        return 0;
    }
    let p_stack_base = mbi.allocation_base as usize;

    let mut si: SystemInfo = core::mem::zeroed();
    GetSystemInfo(&mut si);
    let page_size = si.page_size as usize;

    let mut region_size: usize = 0;
    let mut stack_size_in_bytes: u32 = 0;
    if __acrt_SetThreadStackGuarantee(&mut stack_size_in_bytes) != 0 && stack_size_in_bytes > 0 {
        region_size = stack_size_in_bytes as usize;
    }

    region_size = (region_size + page_size - 1) & !(page_size - 1);

    if region_size != 0 {
        region_size += page_size;
    }
    if region_size < (MIN_STACK_REQ_WINNT as usize) * page_size {
        region_size = (MIN_STACK_REQ_WINNT as usize) * page_size;
    }

    let p_max_guard = (p_stack & !(page_size - 1)).wrapping_sub(region_size);
    let p_min_guard = p_stack_base + page_size;

    if p_max_guard < p_min_guard {
        return 0;
    }

    if VirtualAlloc(p_max_guard as *mut c_void, region_size, MEM_COMMIT, PAGE_READWRITE).is_null() {
        return 0;
    }
    let mut old_protect: u32 = 0;
    if VirtualProtect(p_max_guard as *mut c_void, region_size, PAGE_READWRITE | PAGE_GUARD, &mut old_protect) == 0 {
        return 0;
    }
    1
}

// ── set_error_mode.cpp ───────────────────────────────────────────────────

const OUT_TO_DEFAULT: c_int = 0;
const OUT_TO_STDERR: c_int = 1;
const OUT_TO_MSGBOX: c_int = 2;
const REPORT_ERRMODE: c_int = 3;
const EINVAL: c_int = 22;

extern "C" {
    fn _invalid_parameter_noinfo();
}

/// `__acrt_error_mode` -- process-wide, unsynchronized, exactly as the C.
static mut ACRT_ERROR_MODE: c_int = OUT_TO_DEFAULT;

/// `_set_error_mode`.
#[no_mangle]
pub unsafe extern "C" fn _set_error_mode(new_error_mode: c_int) -> c_int {
    match new_error_mode {
        OUT_TO_DEFAULT | OUT_TO_STDERR | OUT_TO_MSGBOX => {
            let old = ACRT_ERROR_MODE;
            ACRT_ERROR_MODE = new_error_mode;
            old
        }
        REPORT_ERRMODE => ACRT_ERROR_MODE,
        _ => {
            *crate::misc::errno::_errno() = EINVAL;
            _invalid_parameter_noinfo();
            -1
        }
    }
}

// ── seterrm.cpp ──────────────────────────────────────────────────────────

extern "system" {
    fn SetErrorMode(mode: u32) -> u32;
}

/// `_seterrormode`. Despite the file's header comment (about critical-error
/// handling modes), the body is an unconditional, unvalidated
/// `SetErrorMode` passthrough -- no return value, no error checking.
#[no_mangle]
pub unsafe extern "C" fn _seterrormode(mode: c_int) {
    SetErrorMode(mode as u32);
}

// ── slbeep.cpp ───────────────────────────────────────────────────────────

extern "system" {
    fn Sleep(ms: u32);
    fn Beep(freq: u32, duration: u32) -> c_int;
}

/// `_sleep`. `duration == 0` is bumped to 1 -- `Sleep(0)` (yield without
/// delay) is never actually reached through this entry point, even though
/// the doc comment describes `_SLEEP_MINIMUM` (0) as meaning exactly that;
/// the C's own body silently overrides it.
#[no_mangle]
pub unsafe extern "C" fn _sleep(mut duration: c_ulong) {
    if duration == 0 {
        duration += 1;
    }
    Sleep(duration);
}

/// `_beep`.
#[no_mangle]
pub unsafe extern "C" fn _beep(frequency: c_uint, duration: c_uint) {
    Beep(frequency, duration);
}

// ── systime.cpp ──────────────────────────────────────────────────────────

/// `struct tm` (`corecrt_wtime.h`): nine plain `int`s, no padding -- kept
/// private to this file (matching `ctime.rs`'s own private `Tm`, which this
/// is NOT the same type as for `clashing_extern_declarations` purposes,
/// since `_mktime32` is a distinct symbol name from anything `ctime.rs`
/// declares).
#[repr(C)]
pub struct Tm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
}

#[repr(C)]
struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

extern "system" {
    fn GetLocalTime(st: *mut SystemTime);
    fn SetLocalTime(st: *const SystemTime) -> c_int;
    fn GetLastError() -> u32;
}

extern "C" {
    /// `__time32_t __cdecl _mktime32(struct tm*)` (`sdk/include/crt/time.h`).
    fn _mktime32(tm: *mut Tm) -> i32;
}

const ERROR_INVALID_PARAMETER: u32 = 87;

/// `_getsystime`.
#[no_mangle]
pub unsafe extern "C" fn _getsystime(result: *mut Tm) -> c_uint {
    if result.is_null() {
        *crate::misc::errno::_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }

    let mut local: SystemTime = core::mem::zeroed();
    GetLocalTime(&mut local);

    (*result).tm_isdst = -1;
    (*result).tm_sec = local.second as c_int;
    (*result).tm_min = local.minute as c_int;
    (*result).tm_hour = local.hour as c_int;
    (*result).tm_mday = local.day as c_int;
    (*result).tm_mon = local.month as c_int - 1;
    (*result).tm_year = local.year as c_int - 1900;
    (*result).tm_wday = local.day_of_week as c_int;

    _mktime32(result);

    local.milliseconds as c_uint
}

/// `_setsystime`.
#[no_mangle]
pub unsafe extern "C" fn _setsystime(source: *mut Tm, milliseconds: c_uint) -> c_uint {
    if source.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    _mktime32(source);

    let mut local: SystemTime = core::mem::zeroed();
    local.year = ((*source).tm_year + 1900) as u16;
    local.month = ((*source).tm_mon + 1) as u16;
    local.day = (*source).tm_mday as u16;
    local.hour = (*source).tm_hour as u16;
    local.minute = (*source).tm_min as u16;
    local.second = (*source).tm_sec as u16;
    local.milliseconds = milliseconds as u16;

    if SetLocalTime(&local) == 0 {
        return GetLastError();
    }
    0
}

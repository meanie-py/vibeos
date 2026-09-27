//! `_findfirst`/`_findnext`/`_findclose` — all 17. Replaces
//! `sdk/lib/ucrt/filesystem/findfile.cpp` in full.
//!
//! Every search is WIDE underneath: the narrow exports convert the pattern to
//! UTF-16, run the wide search, and convert the found name back, both through
//! the codepage `__acrt_get_utf8_acp_compatibility_codepage` picks (UTF-8 if the
//! locale is UTF-8, else the OEM codepage when file APIs are OEM, else the ANSI
//! one). Eight record layouts -- 32/64-bit times x 32/64-bit sizes x narrow/wide
//! -- are one generic struct here, pinned below against the compiler's own
//! offsets on all three targets.
//!
//! ## Details that are silent if wrong
//!
//! * **`FILE_ATTRIBUTE_NORMAL` reads back as 0**, every other attribute word as
//!   itself.
//! * **A zero FILETIME is -1, not the epoch**, and so is any time the OS cannot
//!   convert; times are LOCAL (`SystemTimeToTzSpecificLocalTime`, then
//!   `__loctotime*_t` with DST "unknown").
//! * **A 32-bit `size` is the LOW word only** -- a 5 GB file reports 1 GB, silently.
//! * **The narrow forms LEAK the find handle** when the found name does not
//!   convert back to the codepage: they return -1 having already opened it.
//!   Transcribed, not fixed -- a caller cannot close a handle it was never given
//!   either way, and fixing it would be the only difference observable.
//! * **Validation order**: `_findfirst` checks `result` before converting the
//!   pattern and the wide search checks it again, then the pattern; a NULL
//!   narrow pattern therefore converts to NULL and fails in the WIDE check.
//!   `_findnext` rejects handle 0 and -1 before looking at `result`.
//! * **`_findclose` never raises the invalid-parameter handler**: a bad handle is
//!   EINVAL and -1, quietly.

use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

const EINVAL: c_int = 22;
const ENOENT: c_int = 2;
const ENOMEM: c_int = 12;

const ERROR_FILE_NOT_FOUND: u32 = 2;
const ERROR_PATH_NOT_FOUND: u32 = 3;
const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
const ERROR_NO_MORE_FILES: u32 = 18;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const CP_ACP: u32 = 0;
const CP_OEMCP: u32 = 1;
const CP_UTF8: u32 = 65001;
const MB_PRECOMPOSED: u32 = 0x1;
const MB_ERR_INVALID_CHARS: u32 = 0x8;
/// `_countof(_finddata_t::name)`
const NAME_LEN: usize = 260;

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: isize = -1;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct FileTime {
    pub(crate) low: u32,
    pub(crate) high: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[repr(C)]
struct Win32FindDataW {
    attributes: u32,
    creation: FileTime,
    last_access: FileTime,
    last_write: FileTime,
    size_high: u32,
    size_low: u32,
    reserved0: u32,
    reserved1: u32,
    file_name: [u16; 260],
    alternate_file_name: [u16; 14],
}
const _: () = assert!(core::mem::size_of::<Win32FindDataW>() == 592);

extern "system" {
    fn FindFirstFileExW(
        name: *const u16,
        info_level: c_int,
        data: *mut Win32FindDataW,
        search_op: c_int,
        filter: *mut c_void,
        flags: u32,
    ) -> Handle;
    fn FindNextFileW(handle: Handle, data: *mut Win32FindDataW) -> c_int;
    fn FindClose(handle: Handle) -> c_int;
    fn GetLastError() -> u32;
    pub(crate) fn FileTimeToSystemTime(ft: *const FileTime, st: *mut SystemTime) -> c_int;
    pub(crate) fn SystemTimeToTzSpecificLocalTime(
        tz: *const c_void,
        utc: *const SystemTime,
        local: *mut SystemTime,
    ) -> c_int;
    fn __acrt_AreFileApisANSI() -> c_int;
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    fn __acrt_errno_map_os_error(err: u32);
    fn __acrt_MultiByteToWideChar(
        cp: u32,
        flags: u32,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
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
    fn __loctotime32_t(yr: c_int, mo: c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dst: c_int)
        -> i32;
    fn __loctotime64_t(yr: c_int, mo: c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dst: c_int)
        -> i64;
    fn strcpy_s(dst: *mut u8, size: usize, src: *const u8) -> c_int;
    fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn _malloc_base(size: usize) -> *mut c_void;
    fn _free_base(p: *mut c_void);
}

// ── the eight record layouts ────────────────────────────────────────────────

#[repr(C)]
pub struct FindData<T, S, C> {
    attrib: u32,
    time_create: T,
    time_access: T,
    time_write: T,
    size: S,
    name: [C; NAME_LEN],
}

pub type FindData32 = FindData<i32, u32, u8>;
pub type FindData32i64 = FindData<i32, i64, u8>;
pub type FindData64i32 = FindData<i64, u32, u8>;
pub type FindData64 = FindData<i64, i64, u8>;
pub type WFindData32 = FindData<i32, u32, u16>;
pub type WFindData32i64 = FindData<i32, i64, u16>;
pub type WFindData64i32 = FindData<i64, u32, u16>;
pub type WFindData64 = FindData<i64, i64, u16>;

// Measured with the target compiler's own offsetof on i386, amd64 and arm64 --
// identical on all three, since every field is fixed-width.
macro_rules! pin {
    ($t:ty, $size:expr, $size_off:expr, $name_off:expr) => {
        const _: () = assert!(core::mem::size_of::<$t>() == $size);
        const _: () = assert!(core::mem::offset_of!($t, size) == $size_off);
        const _: () = assert!(core::mem::offset_of!($t, name) == $name_off);
    };
}
pin!(FindData32, 280, 16, 20);
pin!(FindData32i64, 288, 16, 24);
pin!(FindData64i32, 296, 32, 36);
pin!(FindData64, 304, 32, 40);
pin!(WFindData32, 540, 16, 20);
pin!(WFindData32i64, 544, 16, 24);
pin!(WFindData64i32, 560, 32, 36);
pin!(WFindData64, 560, 32, 40);

pub(crate) trait CrtTime: Copy {
    const INVALID: Self;
    unsafe fn from_local(st: &SystemTime) -> Self;
}
impl CrtTime for i32 {
    const INVALID: Self = -1;
    unsafe fn from_local(st: &SystemTime) -> Self {
        __loctotime32_t(
            st.year as c_int,
            st.month as c_int,
            st.day as c_int,
            st.hour as c_int,
            st.minute as c_int,
            st.second as c_int,
            -1,
        )
    }
}
impl CrtTime for i64 {
    const INVALID: Self = -1;
    unsafe fn from_local(st: &SystemTime) -> Self {
        __loctotime64_t(
            st.year as c_int,
            st.month as c_int,
            st.day as c_int,
            st.hour as c_int,
            st.minute as c_int,
            st.second as c_int,
            -1,
        )
    }
}

trait FileSize: Copy {
    fn from_parts(high: u32, low: u32) -> Self;
}
impl FileSize for u32 {
    fn from_parts(_high: u32, low: u32) -> Self {
        low
    }
}
impl FileSize for i64 {
    fn from_parts(high: u32, low: u32) -> Self {
        (high as i64).wrapping_mul(0x1_0000_0000).wrapping_add(low as i64)
    }
}

/// `convert_file_time_to_time_t`
unsafe fn to_time<T: CrtTime>(ft: &FileTime) -> T {
    if ft.low == 0 && ft.high == 0 {
        return T::INVALID;
    }
    let mut utc = SystemTime::default();
    if FileTimeToSystemTime(ft, &mut utc) == 0 {
        return T::INVALID;
    }
    let mut local = SystemTime::default();
    if SystemTimeToTzSpecificLocalTime(core::ptr::null(), &utc, &mut local) == 0 {
        return T::INVALID;
    }
    T::from_local(&local)
}

#[cold]
unsafe fn invalid_parameter() {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
}

/// The find-error switch shared by first and next.
unsafe fn set_find_errno() {
    *_errno() = match GetLastError() {
        ERROR_NO_MORE_FILES | ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND => ENOENT,
        ERROR_NOT_ENOUGH_MEMORY => ENOMEM,
        _ => EINVAL,
    };
}

unsafe fn fill<T: CrtTime, S: FileSize>(wfd: &Win32FindDataW, result: *mut FindData<T, S, u16>) {
    (*result).attrib = if wfd.attributes == FILE_ATTRIBUTE_NORMAL { 0 } else { wfd.attributes };
    (*result).time_create = to_time(&wfd.creation);
    (*result).time_access = to_time(&wfd.last_access);
    (*result).time_write = to_time(&wfd.last_write);
    (*result).size = S::from_parts(wfd.size_high, wfd.size_low);
    // `_ERRCHECK(wcscpy_s(...))`: cFileName is 260 wide, so this cannot fail.
    if wcscpy_s((*result).name.as_mut_ptr(), NAME_LEN, wfd.file_name.as_ptr()) != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

/// `common_find_first_wide`
unsafe fn first_wide<T: CrtTime, S: FileSize>(
    pattern: *const u16,
    result: *mut FindData<T, S, u16>,
) -> isize {
    if result.is_null() {
        invalid_parameter();
        return -1;
    }
    if pattern.is_null() {
        invalid_parameter();
        return -1;
    }
    let mut wfd: Win32FindDataW = core::mem::zeroed();
    // FindExInfoStandard = 0, FindExSearchNameMatch = 0
    let h = FindFirstFileExW(pattern, 0, &mut wfd, 0, null_mut(), 0);
    if h as isize == INVALID_HANDLE_VALUE {
        set_find_errno();
        return -1;
    }
    fill(&wfd, result);
    h as isize
}

/// `common_find_next_wide`
unsafe fn next_wide<T: CrtTime, S: FileSize>(handle: isize, result: *mut FindData<T, S, u16>) -> c_int {
    if handle == 0 || handle == INVALID_HANDLE_VALUE || result.is_null() {
        invalid_parameter();
        return -1;
    }
    let mut wfd: Win32FindDataW = core::mem::zeroed();
    if FindNextFileW(handle as Handle, &mut wfd) == 0 {
        set_find_errno();
        return -1;
    }
    fill(&wfd, result);
    0
}

/// `__acrt_get_utf8_acp_compatibility_codepage`
pub(crate) unsafe fn file_codepage() -> u32 {
    let lu = LocaleUpdate::new(null_mut());
    if (*(*lu.locinfo()).public_ref()).locale_lc_codepage == CP_UTF8 {
        return CP_UTF8;
    }
    if __acrt_AreFileApisANSI() == 0 {
        CP_OEMCP
    } else {
        CP_ACP
    }
}

/// A heap buffer that frees itself -- the `__crt_internal_win32_buffer` role.
pub(crate) struct Buf<C>(pub(crate) *mut C);
impl<C> Drop for Buf<C> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { _free_base(self.0 as *mut c_void) };
        }
    }
}
impl<C> Buf<C> {
    unsafe fn alloc(count: usize) -> Result<Self, c_int> {
        let p = _malloc_base(count * core::mem::size_of::<C>()) as *mut C;
        if p.is_null() {
            return Err(ENOMEM);
        }
        Ok(Buf(p))
    }
}

/// `__acrt_convert_wcs_mbs_cp`'s failure arm: map the OS error, return errno.
unsafe fn os_errno() -> c_int {
    __acrt_errno_map_os_error(GetLastError());
    *_errno()
}

/// `__acrt_mbs_to_wcs_cp`. `Ok(None)` for a NULL input, as the C's
/// `set_to_nullptr`.
pub(crate) unsafe fn mbs_to_wcs(s: *const u8, cp: u32) -> Result<Option<Buf<u16>>, c_int> {
    if s.is_null() {
        return Ok(None);
    }
    if *s == 0 {
        let b = Buf::<u16>::alloc(1)?;
        *b.0 = 0;
        return Ok(Some(b));
    }
    let flags = MB_PRECOMPOSED | MB_ERR_INVALID_CHARS;
    let need = __acrt_MultiByteToWideChar(cp, flags, s.cast(), -1, null_mut(), 0);
    if need == 0 {
        return Err(os_errno());
    }
    let b = Buf::<u16>::alloc(need as usize)?;
    if __acrt_MultiByteToWideChar(cp, flags, s.cast(), -1, b.0, need) == 0 {
        return Err(os_errno());
    }
    Ok(Some(b))
}

/// `__acrt_wcs_to_mbs_cp` over a non-NULL name.
unsafe fn wcs_to_mbs(s: *const u16, cp: u32) -> Result<Buf<u8>, c_int> {
    if *s == 0 {
        let b = Buf::<u8>::alloc(1)?;
        *b.0 = 0;
        return Ok(b);
    }
    let need = __acrt_WideCharToMultiByte(cp, 0, s, -1, null_mut(), 0, core::ptr::null(), null_mut());
    if need == 0 {
        return Err(os_errno());
    }
    let b = Buf::<u8>::alloc(need as usize)?;
    if __acrt_WideCharToMultiByte(cp, 0, s, -1, b.0.cast(), need, core::ptr::null(), null_mut()) == 0 {
        return Err(os_errno());
    }
    Ok(b)
}

/// `copy_wide_to_narrow_find_data`
unsafe fn to_narrow<T: CrtTime, S: FileSize>(
    w: &FindData<T, S, u16>,
    r: *mut FindData<T, S, u8>,
    cp: u32,
) -> bool {
    let name = match wcs_to_mbs(w.name.as_ptr(), cp) {
        Ok(b) => b,
        Err(_) => return false,
    };
    if strcpy_s((*r).name.as_mut_ptr(), NAME_LEN, name.0) != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
    (*r).attrib = w.attrib;
    (*r).time_create = w.time_create;
    (*r).time_access = w.time_access;
    (*r).time_write = w.time_write;
    (*r).size = w.size;
    true
}

/// `common_find_first_narrow`
unsafe fn first_narrow<T: CrtTime, S: FileSize>(
    pattern: *const u8,
    result: *mut FindData<T, S, u8>,
    cp: u32,
) -> isize {
    if result.is_null() {
        invalid_parameter();
        return -1;
    }
    let wide = match mbs_to_wcs(pattern, cp) {
        Ok(w) => w,
        Err(_) => return -1,
    };
    let wide_ptr = wide.as_ref().map_or(core::ptr::null(), |b| b.0 as *const u16);
    let mut wide_result: FindData<T, S, u16> = core::mem::zeroed();
    let handle = first_wide(wide_ptr, &mut wide_result);
    if handle == -1 {
        return -1;
    }
    if !to_narrow(&wide_result, result, cp) {
        return -1; // the handle leaks, as in the C
    }
    handle
}

/// `common_find_next_narrow`
unsafe fn next_narrow<T: CrtTime, S: FileSize>(
    handle: isize,
    result: *mut FindData<T, S, u8>,
    cp: u32,
) -> c_int {
    let mut wide_result: FindData<T, S, u16> = core::mem::zeroed();
    if next_wide(handle, &mut wide_result) == -1 {
        return -1;
    }
    if !to_narrow(&wide_result, result, cp) {
        return -1;
    }
    0
}

macro_rules! find_exports {
    ($narrow:ty, $wide:ty, $ff:ident, $fn_:ident, $wff:ident, $wfn:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $ff(pattern: *const u8, result: *mut $narrow) -> isize {
            first_narrow(pattern, result, file_codepage())
        }
        #[no_mangle]
        pub unsafe extern "C" fn $fn_(handle: isize, result: *mut $narrow) -> c_int {
            next_narrow(handle, result, file_codepage())
        }
        #[no_mangle]
        pub unsafe extern "C" fn $wff(pattern: *const u16, result: *mut $wide) -> isize {
            first_wide(pattern, result)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $wfn(handle: isize, result: *mut $wide) -> c_int {
            next_wide(handle, result)
        }
    };
}

find_exports!(FindData32, WFindData32, _findfirst32, _findnext32, _wfindfirst32, _wfindnext32);
find_exports!(
    FindData32i64,
    WFindData32i64,
    _findfirst32i64,
    _findnext32i64,
    _wfindfirst32i64,
    _wfindnext32i64
);
find_exports!(
    FindData64i32,
    WFindData64i32,
    _findfirst64i32,
    _findnext64i32,
    _wfindfirst64i32,
    _wfindnext64i32
);
find_exports!(FindData64, WFindData64, _findfirst64, _findnext64, _wfindfirst64, _wfindnext64);

#[no_mangle]
pub unsafe extern "C" fn _findclose(handle: isize) -> c_int {
    if FindClose(handle as Handle) == 0 {
        *_errno() = EINVAL;
        return -1;
    }
    0
}

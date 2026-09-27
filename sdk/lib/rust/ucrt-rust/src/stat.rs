//! `_stat*`, `_wstat*`, `_fstat*` -- all 12. Replaces
//! `sdk/lib/ucrt/filesystem/stat.cpp` in full.
//!
//! Every path is WIDE underneath: the narrow forms convert through
//! `__acrt_get_utf8_acp_compatibility_codepage`, as `_findfirst` does, and share
//! its converter. Four record layouts -- 32/64-bit times x 32/64-bit sizes --
//! are one generic struct, pinned below against the compiler's own offsets.
//!
//! ## Details that are silent if wrong
//!
//! * **A zero FILETIME is the FALLBACK, not -1** (unlike `_findfirst`): 0 for
//!   the write time, then the write time for access and creation. A time the OS
//!   cannot convert IS -1.
//! * **A 32-bit `st_size` refuses a big file** with EOVERFLOW (the file is over
//!   `LONG_MAX`, or has a high word at all); a 64-bit one refuses only a high
//!   word over `LONG_MAX`. Either way the record is zeroed and -1 returned --
//!   `_findfirst32` truncates instead.
//! * **A pipe's `st_size` is `PeekNamedPipe`'s count cast through a 32-bit
//!   `_off_t`**, even in the 64-bit-size records, and `st_dev`/`st_rdev` are the
//!   descriptor number for pipes and devices.
//! * **Roots are fabricated.** `C:\` and `\\server\share\` cannot be opened, so
//!   when `CreateFileW` fails a path that `_wfullpath` resolves to a root, and
//!   whose drive type is known, gets a directory record dated 1980-01-01 local.
//! * **"X:" alone is ENOENT** even when CreateFileW opens it (it names that
//!   drive's current directory), because the drive number is taken from the path.
//! * **A failed narrow conversion returns -1 WITHOUT zeroing the record**, and a
//!   NULL narrow path reaches the wide validation (record first, then path).
//! * **`_fstat` on `_NO_CONSOLE_FILENO` (-2) is EBADF without the handler**;
//!   any other bad descriptor raises it. Both clear `_doserrno`.

use crate::findfile::{
    mbs_to_wcs, file_codepage, CrtTime, FileTime, FileTimeToSystemTime, SystemTime,
    SystemTimeToTzSpecificLocalTime,
};
use crate::stdio_layout::lowio;
use core::ffi::{c_int, c_void};
use core::ptr::{null, null_mut};

const EINVAL: c_int = 22;
/// errno.h EBADF (freeldr has an ARC status of the same name).
const ERRNO_EBADF: c_int = 9;
const ERANGE: c_int = 34;
const EOVERFLOW: c_int = 132;

const ERROR_FILE_NOT_FOUND: u32 = 2;
const LONG_MAX: u32 = 0x7fff_ffff;
const MAX_PATH: usize = 260;

const FILE_ATTRIBUTE_READONLY: u32 = 0x01;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_SHARE_READ_WRITE_DELETE: u32 = 0x1 | 0x2 | 0x4;
const OPEN_EXISTING: u32 = 3;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

const FILE_TYPE_UNKNOWN: u32 = 0;
const FILE_TYPE_DISK: u32 = 1;
const FILE_TYPE_CHAR: u32 = 2;
const FILE_TYPE_PIPE: u32 = 3;
const FILE_TYPE_REMOTE: u32 = 0x8000;

const S_IFIFO: u16 = 0x1000;
const S_IFCHR: u16 = 0x2000;
const S_IFDIR: u16 = 0x4000;
const S_IFREG: u16 = 0x8000;
const S_IREAD: u16 = 0x0100;
const S_IWRITE: u16 = 0x0080;
const S_IEXEC: u16 = 0x0040;

/// `_NO_CONSOLE_FILENO`
const NO_CONSOLE_FILENO: c_int = -2;
/// `FOPEN`, bit 0 of `__crt_lowio_handle_data::osfile`.
const FOPEN: u8 = 0x01;

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: isize = -1;

#[repr(C)]
struct ByHandleFileInformation {
    attributes: u32,
    creation: FileTime,
    last_access: FileTime,
    last_write: FileTime,
    volume_serial: u32,
    size_high: u32,
    size_low: u32,
    links: u32,
    index_high: u32,
    index_low: u32,
}
const _: () = assert!(core::mem::size_of::<ByHandleFileInformation>() == 52);

extern "system" {
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        sa: *mut c_void,
        disposition: u32,
        flags: u32,
        template: Handle,
    ) -> Handle;
    fn CloseHandle(h: Handle) -> c_int;
    fn GetFileType(h: Handle) -> u32;
    fn GetFileInformationByHandle(h: Handle, info: *mut ByHandleFileInformation) -> c_int;
    fn PeekNamedPipe(
        h: Handle,
        buf: *mut c_void,
        size: u32,
        read: *mut u32,
        avail: *mut u32,
        left: *mut u32,
    ) -> c_int;
    fn GetDriveTypeW(root: *const u16) -> u32;
    fn GetLastError() -> u32;
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn __doserrno() -> *mut u32;
    fn _invalid_parameter_noinfo();
    fn __acrt_errno_map_os_error(err: u32);
    fn __loctotime32_t(yr: c_int, mo: c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dst: c_int)
        -> i32;
    fn __loctotime64_t(yr: c_int, mo: c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dst: c_int)
        -> i64;
    fn _wfullpath(buf: *mut u16, path: *const u16, size: usize) -> *mut u16;
    fn _getdrive() -> c_int;
    fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    fn wcsrchr(s: *const u16, c: u16) -> *const u16;
    fn wcspbrk(s: *const u16, set: *const u16) -> *const u16;
    fn wcslen(s: *const u16) -> usize;
    fn free(p: *mut c_void);
    fn __acrt_lowio_lock_fh(fh: c_int);
    fn __acrt_lowio_unlock_fh(fh: c_int);
    static _nhandle: c_int;
}

// ── the four record layouts ─────────────────────────────────────────────────

#[repr(C)]
pub struct Stat<S, T> {
    st_dev: u32,
    st_ino: u16,
    st_mode: u16,
    st_nlink: i16,
    st_uid: i16,
    st_gid: i16,
    st_rdev: u32,
    st_size: S,
    st_atime: T,
    st_mtime: T,
    st_ctime: T,
}

pub type Stat32 = Stat<i32, i32>;
pub type Stat32i64 = Stat<i64, i32>;
pub type Stat64i32 = Stat<i32, i64>;
pub type Stat64 = Stat<i64, i64>;

// Measured with the target compiler's own offsetof -- identical on i386, amd64
// and arm64, since every field is fixed-width.
macro_rules! pin {
    ($t:ty, $size:expr, $rdev:expr, $sz:expr, $at:expr, $mt:expr, $ct:expr) => {
        const _: () = assert!(core::mem::size_of::<$t>() == $size);
        const _: () = assert!(core::mem::offset_of!($t, st_rdev) == $rdev);
        const _: () = assert!(core::mem::offset_of!($t, st_size) == $sz);
        const _: () = assert!(core::mem::offset_of!($t, st_atime) == $at);
        const _: () = assert!(core::mem::offset_of!($t, st_mtime) == $mt);
        const _: () = assert!(core::mem::offset_of!($t, st_ctime) == $ct);
    };
}
pin!(Stat32, 36, 16, 20, 24, 28, 32);
pin!(Stat32i64, 48, 16, 24, 32, 36, 40);
pin!(Stat64i32, 48, 16, 20, 24, 32, 40);
pin!(Stat64, 56, 16, 24, 32, 40, 48);

trait StatTime: CrtTime + Default {
    unsafe fn loctotime(yr: c_int, mo: c_int, dy: c_int) -> Self;
}
impl StatTime for i32 {
    unsafe fn loctotime(yr: c_int, mo: c_int, dy: c_int) -> Self {
        __loctotime32_t(yr, mo, dy, 0, 0, 0, -1)
    }
}
impl StatTime for i64 {
    unsafe fn loctotime(yr: c_int, mo: c_int, dy: c_int) -> Self {
        __loctotime64_t(yr, mo, dy, 0, 0, 0, -1)
    }
}

trait StatSize: Copy + Default {
    /// The `compute_size` overload for this width: `None` is EOVERFLOW.
    fn compute(high: u32, low: u32) -> Option<Self>;
    /// `static_cast<_off_t>(available)`: through a 32-bit `long` either way.
    fn from_off_t(v: i32) -> Self;
}
impl StatSize for i32 {
    fn compute(high: u32, low: u32) -> Option<Self> {
        if high == 0 && low <= LONG_MAX {
            Some(low as i32)
        } else {
            None
        }
    }
    fn from_off_t(v: i32) -> Self {
        v
    }
}
impl StatSize for i64 {
    fn compute(high: u32, low: u32) -> Option<Self> {
        if high <= LONG_MAX {
            Some(((high as u64) << 32 | low as u64) as i64)
        } else {
            None
        }
    }
    fn from_off_t(v: i32) -> Self {
        v as i64
    }
}

/// `*result = StatStruct{}` -- bytewise, so the padding in the mixed-width
/// records (4 bytes in each) reads back as zero too, as the C's does.
unsafe fn zero<S, T>(result: *mut Stat<S, T>) {
    core::ptr::write_bytes(result, 0, 1);
}

/// `_VALIDATE_CLEAR_OSSERR_RETURN`'s failure arm.
#[cold]
unsafe fn invalid_parameter_clear_oss(err: c_int) {
    *__doserrno() = 0;
    *_errno() = err;
    _invalid_parameter_noinfo();
}

fn is_slash(c: u16) -> bool {
    c == b'\\' as u16 || c == b'/' as u16
}

/// `__ascii_iswalpha`
fn is_ascii_alpha(c: u16) -> bool {
    (b'A' as u16..=b'Z' as u16).contains(&c) || (b'a' as u16..=b'z' as u16).contains(&c)
}

/// `call_wfullpath`: the full path into `buffer`, or on ERANGE into a heap
/// block left in `heap` for the caller to free. errno is restored either way
/// unless `_wfullpath` failed for a reason other than ERANGE.
unsafe fn call_wfullpath(buffer: &mut [u16; MAX_PATH], path: *const u16, heap: &mut *mut u16) -> *mut u16 {
    let saved = *_errno();
    *_errno() = 0;
    let result = _wfullpath(buffer.as_mut_ptr(), path, MAX_PATH);
    if !result.is_null() {
        *_errno() = saved;
        return result;
    }
    if *_errno() != ERANGE {
        return null_mut();
    }
    *_errno() = saved;
    *heap = _wfullpath(null_mut(), path, 0);
    *heap
}

/// `has_executable_extension`
unsafe fn has_executable_extension(path: *const u16) -> bool {
    if path.is_null() {
        return false;
    }
    let dot = wcsrchr(path, b'.' as u16);
    if dot.is_null() {
        return false;
    }
    const EXTS: [[u16; 5]; 4] = [
        [b'.' as u16, b'e' as u16, b'x' as u16, b'e' as u16, 0],
        [b'.' as u16, b'c' as u16, b'm' as u16, b'd' as u16, 0],
        [b'.' as u16, b'b' as u16, b'a' as u16, b't' as u16, 0],
        [b'.' as u16, b'c' as u16, b'o' as u16, b'm' as u16, 0],
    ];
    EXTS.iter().any(|e| _wcsicmp(dot, e.as_ptr()) == 0)
}

/// `is_root_or_empty`
unsafe fn is_root_or_empty(path: *const u16) -> bool {
    if path.is_null() {
        return false;
    }
    let start = if is_ascii_alpha(*path) && *path.add(1) == b':' as u16 { path.add(2) } else { path };
    *start == 0 || (is_slash(*start) && *start.add(1) == 0)
}

/// `convert_to_stat_mode`
unsafe fn convert_to_stat_mode(attributes: u32, path: *const u16) -> u16 {
    let os_mode = attributes & 0xff;
    let is_directory = os_mode & FILE_ATTRIBUTE_DIRECTORY != 0;
    let mut mode = if is_directory || is_root_or_empty(path) { S_IFDIR | S_IEXEC } else { S_IFREG };
    mode |= if os_mode & FILE_ATTRIBUTE_READONLY != 0 { S_IREAD } else { S_IREAD | S_IWRITE };
    if has_executable_extension(path) {
        mode |= S_IEXEC;
    }
    mode |= (mode & 0o700) >> 3;
    mode |= (mode & 0o700) >> 6;
    mode
}

/// `get_drive_number_from_path`: `None` if and only if the path is a bare
/// "X:", which has already mapped ERROR_FILE_NOT_FOUND.
unsafe fn get_drive_number_from_path(path: *const u16) -> Option<c_int> {
    if is_ascii_alpha(*path) && *path.add(1) == b':' as u16 {
        if *path.add(2) == 0 {
            __acrt_errno_map_os_error(ERROR_FILE_NOT_FOUND);
            return None;
        }
        let c = *path;
        let lower = if (b'A' as u16..=b'Z' as u16).contains(&c) { c - b'A' as u16 + b'a' as u16 } else { c };
        Some(lower as c_int - b'a' as c_int + 1)
    } else {
        Some(_getdrive())
    }
}

/// `is_root_unc_name`: `//server/share` with at most one trailing slash.
unsafe fn is_root_unc_name(path: *const u16) -> bool {
    if wcslen(path) < 5 {
        return false;
    }
    if !is_slash(*path) || !is_slash(*path.add(1)) || is_slash(*path.add(2)) {
        return false;
    }
    let mut p = path.add(2);
    loop {
        p = p.add(1);
        if *p == 0 || is_slash(*p) {
            break;
        }
    }
    if *p == 0 || *p.add(1) == 0 {
        return false;
    }
    loop {
        p = p.add(1);
        if *p == 0 || is_slash(*p) {
            break;
        }
    }
    *p == 0 || *p.add(1) == 0
}

/// `is_usable_drive_or_unc_root`
unsafe fn is_usable_drive_or_unc_root(path: *const u16) -> bool {
    const SEPARATORS: [u16; 4] = [b'.' as u16, b'/' as u16, b'\\' as u16, 0];
    if wcspbrk(path, SEPARATORS.as_ptr()).is_null() {
        return false;
    }
    let mut buffer = [0u16; MAX_PATH];
    let mut heap: *mut u16 = null_mut();
    let full = call_wfullpath(&mut buffer, path, &mut heap);
    let usable = !full.is_null()
        && (wcslen(full) == 3 || is_root_unc_name(full))
        && GetDriveTypeW(path) > 1;
    // `__crt_unique_heap_ptr<wchar_t, __crt_public_free_policy>`
    free(heap as *mut c_void);
    usable
}

/// `convert_filetime_to_time_t`
unsafe fn convert_filetime<T: StatTime>(ft: &FileTime, fallback: T) -> T {
    if ft.low == 0 && ft.high == 0 {
        return fallback;
    }
    let mut utc: SystemTime = core::mem::zeroed();
    let mut local: SystemTime = core::mem::zeroed();
    if FileTimeToSystemTime(ft, &mut utc) == 0
        || SystemTimeToTzSpecificLocalTime(null(), &utc, &mut local) == 0
    {
        return T::INVALID;
    }
    T::from_local(&local)
}

/// `common_stat_handle_file_not_opened`
unsafe fn handle_file_not_opened<S: StatSize, T: StatTime>(path: *const u16, r: &mut Stat<S, T>) -> bool {
    if !is_usable_drive_or_unc_root(path) {
        __acrt_errno_map_os_error(ERROR_FILE_NOT_FOUND);
        return false;
    }
    r.st_mode = convert_to_stat_mode(FILE_ATTRIBUTE_DIRECTORY, path);
    r.st_nlink = 1;
    let Some(drive) = get_drive_number_from_path(path) else {
        return false;
    };
    r.st_rdev = (drive - 1) as u32;
    r.st_dev = (drive - 1) as u32;
    r.st_mtime = T::loctotime(1980, 1, 1);
    r.st_atime = r.st_mtime;
    r.st_ctime = r.st_mtime;
    true
}

/// `common_stat_handle_file_opened`. `path` is NULL for `_fstat`.
unsafe fn handle_file_opened<S: StatSize, T: StatTime>(
    path: *const u16,
    fh: c_int,
    handle: Handle,
    r: &mut Stat<S, T>,
) -> bool {
    let file_type = GetFileType(handle) & !FILE_TYPE_REMOTE;
    match file_type {
        FILE_TYPE_DISK => {}
        FILE_TYPE_CHAR | FILE_TYPE_PIPE => {
            r.st_mode = if file_type == FILE_TYPE_CHAR { S_IFCHR } else { S_IFIFO };
            r.st_nlink = 1;
            r.st_rdev = fh as u32;
            r.st_dev = fh as u32;
            if file_type != FILE_TYPE_CHAR {
                let mut available: u32 = 0;
                if PeekNamedPipe(handle, null_mut(), 0, null_mut(), &mut available, null_mut()) != 0 {
                    r.st_size = S::from_off_t(available as i32);
                }
            }
            return true;
        }
        FILE_TYPE_UNKNOWN => {
            *_errno() = ERRNO_EBADF;
            return false;
        }
        _ => {
            __acrt_errno_map_os_error(GetLastError());
            return false;
        }
    }

    r.st_nlink = 1;
    if !path.is_null() {
        let Some(drive) = get_drive_number_from_path(path) else {
            return false;
        };
        r.st_rdev = (drive - 1) as u32;
        r.st_dev = (drive - 1) as u32;
    }

    let mut info: ByHandleFileInformation = core::mem::zeroed();
    if GetFileInformationByHandle(handle, &mut info) == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return false;
    }

    r.st_mode = convert_to_stat_mode(info.attributes, path);
    r.st_mtime = convert_filetime(&info.last_write, T::default());
    r.st_atime = convert_filetime(&info.last_access, r.st_mtime);
    r.st_ctime = convert_filetime(&info.creation, r.st_mtime);

    // `_VALIDATE_RETURN_NOEXC(..., EOVERFLOW, false)`, after `size = 0`.
    match S::compute(info.size_high, info.size_low) {
        Some(size) => {
            r.st_size = size;
            true
        }
        None => {
            r.st_size = S::default();
            *_errno() = EOVERFLOW;
            false
        }
    }
}

/// `common_stat` over a wide path.
unsafe fn stat_wide<S: StatSize, T: StatTime>(path: *const u16, result: *mut Stat<S, T>) -> c_int {
    if result.is_null() {
        invalid_parameter_clear_oss(EINVAL);
        return -1;
    }
    zero(result);
    if path.is_null() {
        invalid_parameter_clear_oss(EINVAL);
        return -1;
    }
    let r = &mut *result;

    let handle = CreateFileW(
        path,
        FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ_WRITE_DELETE,
        null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        null_mut(),
    );
    let ok = if handle as isize != INVALID_HANDLE_VALUE {
        let ok = handle_file_opened(path, -1, handle, r);
        CloseHandle(handle);
        ok
    } else {
        handle_file_not_opened(path, r)
    };
    if !ok {
        zero(result);
        return -1;
    }
    0
}

/// `common_stat` over a narrow path.
unsafe fn stat_narrow<S: StatSize, T: StatTime>(path: *const u8, result: *mut Stat<S, T>) -> c_int {
    if path.is_null() {
        return stat_wide(null(), result);
    }
    match mbs_to_wcs(path, file_codepage()) {
        Ok(Some(wide)) => stat_wide(wide.0, result),
        Ok(None) | Err(_) => -1,
    }
}

/// `common_fstat`
unsafe fn fstat<S: StatSize, T: StatTime>(fh: c_int, result: *mut Stat<S, T>) -> c_int {
    if result.is_null() {
        invalid_parameter_clear_oss(EINVAL);
        return -1;
    }
    zero(result);

    // `_CHECK_FH_CLEAR_OSSERR_RETURN`: no handler.
    if fh == NO_CONSOLE_FILENO {
        *__doserrno() = 0;
        *_errno() = ERRNO_EBADF;
        return -1;
    }
    if !(fh >= 0 && fh < _nhandle) {
        invalid_parameter_clear_oss(ERRNO_EBADF);
        return -1;
    }
    if lowio::osfile(fh) & FOPEN == 0 {
        invalid_parameter_clear_oss(ERRNO_EBADF);
        return -1;
    }

    // `__acrt_lowio_lock_fh_and_call`
    __acrt_lowio_lock_fh(fh);
    let rc = if lowio::osfile(fh) & FOPEN == 0 {
        *_errno() = ERRNO_EBADF;
        -1
    } else if !handle_file_opened(null(), fh, lowio::osfhnd(fh) as Handle, &mut *result) {
        zero(result);
        -1
    } else {
        0
    };
    __acrt_lowio_unlock_fh(fh);
    rc
}

macro_rules! stat_exports {
    ($t:ty, $stat:ident, $wstat:ident, $fstat:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $stat(path: *const u8, result: *mut $t) -> c_int {
            stat_narrow(path, result)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $wstat(path: *const u16, result: *mut $t) -> c_int {
            stat_wide(path, result)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $fstat(fh: c_int, result: *mut $t) -> c_int {
            fstat(fh, result)
        }
    };
}

stat_exports!(Stat32, _stat32, _wstat32, _fstat32);
stat_exports!(Stat32i64, _stat32i64, _wstat32i64, _fstat32i64);
stat_exports!(Stat64i32, _stat64i32, _wstat64i32, _fstat64i32);
stat_exports!(Stat64, _stat64, _wstat64, _fstat64);

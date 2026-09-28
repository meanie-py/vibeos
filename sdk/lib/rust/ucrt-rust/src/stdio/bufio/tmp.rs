//! `tmpfile.cpp`, `rmtmp.cpp`, `gettemppath.cpp`.

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};
use core::ptr::{addr_of_mut, null_mut};

use super::{calloc, free};
use crate::ptd::AcrtPtd;
use crate::stdio::{allocate_stream, free_stream, Stream};
use crate::stdio_layout::flags::*;

const ENOMEM: c_int = 12;
const ENOENT: c_int = 2;
const CRT_EMFILE: c_int = 24;
const EINVAL: c_int = 22;
const EEXIST: c_int = 17;

/// `L_tmpnam` (`_MAX_PATH`).
const L_TMPNAM: usize = 260;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: c_uint, r: usize) -> !;
    fn strlen(s: *const c_char) -> usize;
    fn wcslen(s: *const u16) -> usize;
    fn strcpy_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    fn wcscpy_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn strrchr(s: *const u8, c: c_int) -> *mut u8;
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn strtoull(s: *const u8, endptr: *mut *mut u8, base: c_int) -> u64;
    fn wcstoull(s: *const u16, endptr: *mut *mut u16, base: c_int) -> u64;
    fn _ui64toa_s(value: u64, buffer: *mut u8, size: usize, radix: c_int) -> c_int;
    fn _ui64tow_s(value: u64, buffer: *mut u16, size: usize, radix: c_int) -> c_int;
    fn _access_s(path: *const u8, mode: c_int) -> c_int;
    fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
    fn _strdup(s: *const c_char) -> *mut c_char;
    fn _sopen_s(pfh: *mut c_int, path: *const c_char, oflag: c_int, shflag: c_int, pmode: c_int) -> c_int;
    fn _close(fh: c_int) -> c_int;
    /// `__acrt_getptd_noexit()`: the non-aborting per-thread-data lookup (see
    /// `invalid_parameter.rs`, which declares the same symbol locally).
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
}

extern "system" {
    fn __acrt_GetTempPath2W(len: u32, buf: *mut u16) -> u32;
    fn __acrt_AreFileApisANSI() -> c_int;
    fn GetCurrentProcessId() -> u32;
}

extern "C" {
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
}

/// `_ERRCHECK(e)`: Watson if `e != 0`.
#[cold]
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

// ── gettemppath.cpp ──────────────────────────────────────────────────────────

const CP_ACP: u32 = 0;
const CP_OEMCP: u32 = 1;

/// `__acrt_GetTempPath2A`: calls the wide `__acrt_GetTempPath2W` and converts
/// with the ACP or OEMCP -- not UTF-8, deliberately: MSDN guarantees this
/// buffer does not depend on the current locale.
#[no_mangle]
pub unsafe extern "C" fn __acrt_GetTempPath2A(buffer_length: u32, buffer: *mut c_char) -> u32 {
    let mut wide_buffer = [0u16; 261]; // _MAX_PATH + 1

    let result = __acrt_GetTempPath2W(wide_buffer.len() as u32, wide_buffer.as_mut_ptr());
    if result == 0 {
        return 0;
    }

    let code_page = if __acrt_AreFileApisANSI() == 0 { CP_OEMCP } else { CP_ACP };
    let converted = __acrt_WideCharToMultiByte(
        code_page,
        0,
        wide_buffer.as_ptr(),
        -1,
        buffer,
        buffer_length as c_int,
        core::ptr::null(),
        null_mut(),
    );
    if converted == 0 {
        return 0;
    }

    // `WideCharToMultiByte`'s return includes the null terminator; `GetTempPathA`'s does not.
    (converted - 1) as u32
}

// ── tmpfile.cpp: the shared per-buffer-id name generator ────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum BufferId {
    Tmpnam,
    Tmpfile,
    TmpnamS,
}
const BUFFER_ID_COUNT: usize = 3;

static mut NARROW_BUFFERS: [*mut u8; BUFFER_ID_COUNT] = [null_mut(); BUFFER_ID_COUNT];
static mut WIDE_BUFFERS: [*mut u16; BUFFER_ID_COUNT] = [null_mut(); BUFFER_ID_COUNT];

#[no_mangle]
pub unsafe extern "C" fn __acrt_stdio_free_tmpfile_name_buffers_nolock() {
    for i in 0..BUFFER_ID_COUNT {
        let p = addr_of_mut!(NARROW_BUFFERS[i]);
        free((*p).cast());
        *p = null_mut();
    }
    for i in 0..BUFFER_ID_COUNT {
        let p = addr_of_mut!(WIDE_BUFFERS[i]);
        free((*p).cast());
        *p = null_mut();
    }
}

unsafe fn get_tmpfile_buffer_narrow(id: BufferId) -> *mut u8 {
    let slot = addr_of_mut!(NARROW_BUFFERS[id as usize]);
    if (*slot).is_null() {
        *slot = calloc(L_TMPNAM, 1).cast();
    }
    *slot
}
unsafe fn get_tmpfile_buffer_wide(id: BufferId) -> *mut u16 {
    let slot = addr_of_mut!(WIDE_BUFFERS[id as usize]);
    if (*slot).is_null() {
        *slot = calloc(L_TMPNAM, 2).cast();
    }
    *slot
}

/// `initialize_tmpfile_buffer_nolock<char>`: `[buffer id][pid in base 36].0`.
unsafe fn initialize_tmpfile_buffer_narrow(id: BufferId) -> bool {
    let buffer = get_tmpfile_buffer_narrow(id);
    if buffer.is_null() {
        return false;
    }
    let buffer_count = L_TMPNAM;
    // Room for `[id][pid up to 7 chars].[unique up to 13 chars]\0`, at most 22.
    let max_supported = buffer_count - 22;

    let temp_path_length = __acrt_GetTempPath2A(buffer_count as u32, buffer.cast()) as usize;
    if temp_path_length == 0 || temp_path_length > max_supported {
        *buffer = 0;
        return false;
    }

    let mut tail = buffer.add(temp_path_length);
    *tail = match id {
        BufferId::Tmpnam => b's',
        BufferId::Tmpfile => b't',
        BufferId::TmpnamS => b'u',
    };
    tail = tail.add(1);

    let tail_count = buffer_count - (tail.offset_from(buffer) as usize);
    errcheck(_ui64toa_s(GetCurrentProcessId() as u64, tail, tail_count, 36));
    tail = tail.add(strlen(tail.cast()));

    *tail = b'.';
    tail = tail.add(1);
    *tail = b'0';
    tail = tail.add(1);
    *tail = 0;
    true
}

unsafe fn initialize_tmpfile_buffer_wide(id: BufferId) -> bool {
    let buffer = get_tmpfile_buffer_wide(id);
    if buffer.is_null() {
        return false;
    }
    let buffer_count = L_TMPNAM;
    let max_supported = buffer_count - 22;

    let temp_path_length = __acrt_GetTempPath2W(buffer_count as u32, buffer) as usize;
    if temp_path_length == 0 || temp_path_length > max_supported {
        *buffer = 0;
        return false;
    }

    let mut tail = buffer.add(temp_path_length);
    *tail = match id {
        BufferId::Tmpnam => b'v' as u16,
        BufferId::Tmpfile => b'w' as u16,
        BufferId::TmpnamS => b'x' as u16,
    };
    tail = tail.add(1);

    let tail_count = buffer_count - (tail.offset_from(buffer) as usize);
    errcheck(_ui64tow_s(GetCurrentProcessId() as u64, tail, tail_count, 36));
    tail = tail.add(wcslen(tail));

    *tail = b'.' as u16;
    tail = tail.add(1);
    *tail = b'0' as u16;
    tail = tail.add(1);
    *tail = 0;
    true
}

/// `generate_tmpfile_file_name<char>`: increments the base-36 suffix after
/// the last `.`.
unsafe fn generate_tmpfile_file_name_narrow(file_name: *mut u8, file_name_count: usize) -> bool {
    let dot = strrchr(file_name, b'.' as c_int);
    if dot.is_null() {
        _invalid_parameter_noinfo();
        return false;
    }
    if (dot as usize) < (file_name as usize) {
        _invalid_parameter_noinfo();
        return false;
    }
    let dot_offset = dot.offset_from(file_name) as usize;
    if file_name_count <= dot_offset {
        _invalid_parameter_noinfo();
        return false;
    }

    let unique_id = dot.add(1);
    let unique_id_count = file_name_count - dot_offset - 1;

    let next_identifier = strtoull(unique_id, null_mut(), 36).wrapping_add(1);
    if next_identifier == 0 {
        return false;
    }
    errcheck(_ui64toa_s(next_identifier, unique_id, unique_id_count, 36));
    true
}

unsafe fn generate_tmpfile_file_name_wide(file_name: *mut u16, file_name_count: usize) -> bool {
    let dot = wcsrchr(file_name, b'.' as u16);
    if dot.is_null() {
        _invalid_parameter_noinfo();
        return false;
    }
    if (dot as usize) < (file_name as usize) {
        _invalid_parameter_noinfo();
        return false;
    }
    let dot_offset = dot.offset_from(file_name) as usize;
    if file_name_count <= dot_offset {
        _invalid_parameter_noinfo();
        return false;
    }

    let unique_id = dot.add(1);
    let unique_id_count = file_name_count - dot_offset - 1;

    let next_identifier = wcstoull(unique_id, null_mut(), 36).wrapping_add(1);
    if next_identifier == 0 {
        return false;
    }
    errcheck(_ui64tow_s(next_identifier, unique_id, unique_id_count, 36));
    true
}

// ── tmpnam / tmpnam_s / _wtmpnam / _wtmpnam_s ────────────────────────────────

unsafe fn common_tmpnam_nolock_narrow(
    result_buffer: *mut u8,
    result_buffer_count: usize,
    id: BufferId,
) -> c_int {
    let global_buffer = get_tmpfile_buffer_narrow(id);
    let global_buffer_count = L_TMPNAM;
    if global_buffer.is_null() {
        return ENOMEM;
    }

    if *global_buffer == 0 {
        initialize_tmpfile_buffer_narrow(id);
    } else if !generate_tmpfile_file_name_narrow(global_buffer, global_buffer_count) {
        return ENOENT;
    }

    while _access_s(global_buffer, 0) == 0 {
        if !generate_tmpfile_file_name_narrow(global_buffer, global_buffer_count) {
            return ENOENT;
        }
    }

    if !result_buffer.is_null() {
        if id != BufferId::Tmpnam && strlen(global_buffer.cast()) >= result_buffer_count {
            if result_buffer_count != 0 {
                *result_buffer = 0;
            }
            return ERANGE;
        }
        errcheck(strcpy_s(result_buffer, result_buffer_count, global_buffer));
        return 0;
    }

    debug_assert!(id == BufferId::Tmpnam);
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return ENOMEM;
    }
    let slot = addr_of_mut!((*ptd).tmpnam_narrow_buffer);
    if (*slot).is_null() {
        *slot = calloc(global_buffer_count, 1).cast();
        if (*slot).is_null() {
            return ENOMEM;
        }
    }
    errcheck(strcpy_s(*slot, global_buffer_count, global_buffer));
    0
}

unsafe fn common_tmpnam_nolock_wide(
    result_buffer: *mut u16,
    result_buffer_count: usize,
    id: BufferId,
) -> c_int {
    let global_buffer = get_tmpfile_buffer_wide(id);
    let global_buffer_count = L_TMPNAM;
    if global_buffer.is_null() {
        return ENOMEM;
    }

    if *global_buffer == 0 {
        initialize_tmpfile_buffer_wide(id);
    } else if !generate_tmpfile_file_name_wide(global_buffer, global_buffer_count) {
        return ENOENT;
    }

    while _waccess_s(global_buffer, 0) == 0 {
        if !generate_tmpfile_file_name_wide(global_buffer, global_buffer_count) {
            return ENOENT;
        }
    }

    if !result_buffer.is_null() {
        if id != BufferId::Tmpnam && wcslen(global_buffer) >= result_buffer_count {
            if result_buffer_count != 0 {
                *result_buffer = 0;
            }
            return ERANGE;
        }
        errcheck(wcscpy_s(result_buffer, result_buffer_count, global_buffer));
        return 0;
    }

    debug_assert!(id == BufferId::Tmpnam);
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return ENOMEM;
    }
    let slot = addr_of_mut!((*ptd).tmpnam_wide_buffer);
    if (*slot).is_null() {
        *slot = calloc(global_buffer_count, 2).cast();
        if (*slot).is_null() {
            return ENOMEM;
        }
    }
    errcheck(wcscpy_s(*slot, global_buffer_count, global_buffer));
    0
}

const ERANGE: c_int = 34;

/// `__acrt_lock_id` position 12 (see `crate::stdio::TEMPNAM_LOCK`).
use crate::stdio::TEMPNAM_LOCK;
extern "C" {
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
}

unsafe fn common_tmpnam_narrow(
    result_buffer: *mut u8,
    result_buffer_count: usize,
    id: BufferId,
) -> (c_int, *mut u8) {
    __acrt_lock(TEMPNAM_LOCK);
    let saved_errno = *_errno();
    let rc = common_tmpnam_nolock_narrow(result_buffer, result_buffer_count, id);
    let result_pointer = if rc != 0 {
        *_errno() = rc;
        result_buffer
    } else {
        *_errno() = saved_errno;
        if !result_buffer.is_null() {
            result_buffer
        } else {
            (*__acrt_getptd_noexit()).tmpnam_narrow_buffer
        }
    };
    __acrt_unlock(TEMPNAM_LOCK);
    (rc, result_pointer)
}

unsafe fn common_tmpnam_wide(
    result_buffer: *mut u16,
    result_buffer_count: usize,
    id: BufferId,
) -> (c_int, *mut u16) {
    __acrt_lock(TEMPNAM_LOCK);
    let saved_errno = *_errno();
    let rc = common_tmpnam_nolock_wide(result_buffer, result_buffer_count, id);
    let result_pointer = if rc != 0 {
        *_errno() = rc;
        result_buffer
    } else {
        *_errno() = saved_errno;
        if !result_buffer.is_null() {
            result_buffer
        } else {
            (*__acrt_getptd_noexit()).tmpnam_wide_buffer
        }
    };
    __acrt_unlock(TEMPNAM_LOCK);
    (rc, result_pointer)
}

#[no_mangle]
pub unsafe extern "C" fn tmpnam_s(result_buffer: *mut c_char, result_buffer_count: usize) -> c_int {
    if result_buffer.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }
    common_tmpnam_narrow(result_buffer.cast(), result_buffer_count, BufferId::TmpnamS).0
}

#[no_mangle]
pub unsafe extern "C" fn _wtmpnam_s(result_buffer: *mut u16, result_buffer_count: usize) -> c_int {
    if result_buffer.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }
    common_tmpnam_wide(result_buffer, result_buffer_count, BufferId::TmpnamS).0
}

#[no_mangle]
pub unsafe extern "C" fn tmpnam(result_buffer: *mut c_char) -> *mut c_char {
    common_tmpnam_narrow(result_buffer.cast(), L_TMPNAM, BufferId::Tmpnam).1.cast()
}

#[no_mangle]
pub unsafe extern "C" fn _wtmpnam(result_buffer: *mut u16) -> *mut u16 {
    common_tmpnam_wide(result_buffer, L_TMPNAM, BufferId::Tmpnam).1
}

// ── tmpfile / tmpfile_s ──────────────────────────────────────────────────────

const O_CREAT: c_int = 0x0100;
const O_EXCL: c_int = 0x0400;
const O_RDWR: c_int = 0x0002;
const O_BINARY: c_int = 0x8000;
const O_TEMPORARY: c_int = 0x0040;
const S_IREAD: c_int = 0x0100;
const S_IWRITE: c_int = 0x0080;
const SH_DENYNO: c_int = 0x40;
const SH_DENYRW: c_int = 0x10;

unsafe fn common_tmpfile_nolock(stream: *mut *mut c_void, sh_flag: c_int) -> c_int {
    let global_buffer = get_tmpfile_buffer_narrow(BufferId::Tmpfile);
    let global_buffer_count = L_TMPNAM;
    if global_buffer.is_null() {
        return ENOMEM;
    }

    if *global_buffer == 0 {
        if !initialize_tmpfile_buffer_narrow(BufferId::Tmpfile) {
            return EINVAL;
        }
    } else if !generate_tmpfile_file_name_narrow(global_buffer, global_buffer_count) {
        return EINVAL;
    }

    let local_stream = allocate_stream();
    if !local_stream.valid() {
        return CRT_EMFILE;
    }

    let mut result;
    let saved_errno = *_errno();
    *_errno() = 0;

    let mut fh: c_int = 0;
    let open_flag = O_CREAT | O_EXCL | O_RDWR | O_BINARY | O_TEMPORARY;
    let permission_mode = S_IREAD | S_IWRITE;

    loop {
        result = _sopen_s(&mut fh, global_buffer.cast(), open_flag, sh_flag, permission_mode);
        if result != EEXIST {
            break;
        }
        if !generate_tmpfile_file_name_narrow(global_buffer, global_buffer_count) {
            break;
        }
    }

    if *_errno() == 0 {
        *_errno() = saved_errno;
    }

    // Matches the C's `__finally`: `_file` is set away from allocate_stream's
    // `-1` default ONLY on the fully-successful path below, so freeing the
    // stream exactly when `_file == -1` here covers both "never opened" and
    // "opened but `_strdup` failed" without duplicating the check.
    if fh != -1 {
        let tmpfname = _strdup(global_buffer.cast());
        if tmpfname.is_null() {
            _close(fh);
            result = ENOMEM;
        } else {
            let d = local_stream.data();
            (*d).tmpfname = tmpfname;
            (*d).cnt = 0;
            (*d).base = null_mut();
            (*d).ptr = null_mut();
            local_stream.set_flags((super::_commode as c_long) | STREAM_UPDATE);
            (*d).file = fh;
            *stream = local_stream.public_stream();
            result = 0;
        }
    }

    if (*local_stream.data()).file == -1 {
        free_stream(local_stream);
    }
    local_stream.unlock();

    result
}

unsafe fn common_tmpfile(stream: *mut *mut c_void, sh_flag: c_int) -> c_int {
    if stream.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }
    *stream = null_mut();

    __acrt_lock(TEMPNAM_LOCK);
    let rc = common_tmpfile_nolock(stream, sh_flag);
    if rc != 0 {
        *_errno() = rc;
    }
    __acrt_unlock(TEMPNAM_LOCK);
    rc
}

#[no_mangle]
pub unsafe extern "C" fn tmpfile() -> *mut c_void {
    let mut stream = null_mut();
    common_tmpfile(&mut stream, SH_DENYNO);
    stream
}

#[no_mangle]
pub unsafe extern "C" fn tmpfile_s(stream: *mut *mut c_void) -> c_int {
    common_tmpfile(stream, SH_DENYRW)
}

// `#ifndef CRTDLL`-gated in the C (ucrtbase IS `CRTDLL`, so `__acrt_force_use_of_tmpfile`
// and the `__acrt_tmpfile_used` linker-force-include mechanism never fire here; ported
// as a no-op stub only so a caller from the old linkage story still resolves).
#[no_mangle]
pub unsafe extern "C" fn __acrt_force_use_of_tmpfile() {}

// ── rmtmp.cpp ────────────────────────────────────────────────────────────────

use crate::stdio::STDIO_INDEX_LOCK;

#[no_mangle]
pub static mut _tempoff: c_uint = 1;
#[no_mangle]
pub static mut _old_pfxlen: c_uint = 0;

#[no_mangle]
pub unsafe extern "C" fn _rmtmp() -> c_int {
    let mut count = 0;

    __acrt_lock(STDIO_INDEX_LOCK);
    for i in 0..crate::stdio::_nstream {
        let stream = Stream(*crate::stdio::__piob.add(i as usize));
        if !stream.valid() {
            continue;
        }

        stream.lock();
        if stream.is_in_use() && !(*stream.data()).tmpfname.is_null() {
            super::_fclose_nolock(stream.public_stream());
            count += 1;
        }
        stream.unlock();
    }
    __acrt_unlock(STDIO_INDEX_LOCK);

    count
}

// The deleted C carried `_CRT_LINKER_FORCE_INCLUDE(__acrt_tmpfile_terminator)`:
// without the reference the terminator's member is never linked, and
// tmpfile()'s files outlive the process.
crate::force_include!(__acrt_tmpfile_terminator);

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_tmpfile() {
    __acrt_stdio_free_tmpfile_name_buffers_nolock();
    _rmtmp();
}

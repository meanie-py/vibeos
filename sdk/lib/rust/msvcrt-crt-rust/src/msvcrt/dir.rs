use crate::prelude::*;
use core::ffi::c_void;
use crate::{wcscpy, wcslen};


// ── hand-integration: names the island cannot supply ────────────────────────
// wine's debug macros. A MACRO, never a no-op fn: an out-of-line no-op is an
// optimization barrier that uncloaks latent ABI mismatches (-> ud2 -> BUGCHECK).
// Same shape errno.rs, file.rs and the other translated usermode modules use.

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
unsafe extern "C" fn msvcrt_fttofd(fd: *const WIN32_FIND_DATAA, ft: *mut _finddata_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = (dw) as i32;
    (*ft).size = (*fd).nFileSizeLow;
    strcpy(
        core::ptr::addr_of_mut!((*ft).name) as *mut _,
        core::ptr::addr_of!((*fd).cFileName) as *const _,
    );
}
unsafe extern "C" fn msvcrt_wfttofd(fd: *const WIN32_FIND_DATAW, ft: *mut _wfinddata_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = (dw) as i32;
    (*ft).size = (*fd).nFileSizeLow;
    wcscpy((*ft).name.as_mut_ptr(), (*fd).cFileName.as_ptr());
}
unsafe extern "C" fn msvcrt_wfttofd32(fd: *const WIN32_FIND_DATAW, ft: *mut _wfinddata32_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = (dw) as i32;
    (*ft).size = (*fd).nFileSizeLow;
    wcscpy(
        core::ptr::addr_of_mut!((*ft).name) as *mut wchar_t,
        core::ptr::addr_of!((*fd).cFileName) as *const wchar_t,
    );
}
unsafe extern "C" fn msvcrt_fttofdi64(fd: *const WIN32_FIND_DATAA, ft: *mut _finddatai64_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = dw as _;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = dw as _;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = dw as _;
    (*ft).size = (((*fd).nFileSizeHigh as __int64) << 32) | (*fd).nFileSizeLow as __int64;
    strcpy(
        (*ft).name.as_mut_ptr(),
        (*fd).cFileName.as_ptr(),
    );
}
unsafe extern "C" fn msvcrt_wfttofd64(fd: *const WIN32_FIND_DATAW, ft: *mut _wfinddata64_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = dw as _;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = dw as _;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = dw as _;
    (*ft).size = (((*fd).nFileSizeHigh as i64) << 32) | ((*fd).nFileSizeLow as i64);
    wcscpy((*ft).name.as_mut_ptr(), (*fd).cFileName.as_ptr());
}
unsafe extern "C" fn msvcrt_wfttofdi64(fd: *const WIN32_FIND_DATAW, ft: *mut _wfinddatai64_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = (dw) as i32;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = (dw) as i32;
    (*ft).size = (((*fd).nFileSizeHigh as __int64) << 32)
        | (*fd).nFileSizeLow as __int64;
    wcscpy(
        (*ft).name.as_mut_ptr(),
        (*fd).cFileName.as_ptr(),
    );
}
unsafe extern "C" fn msvcrt_wfttofd64i32(fd: *const WIN32_FIND_DATAW, ft: *mut _wfinddata64i32_t) {
    let mut dw: DWORD = 0;

    if (*fd).dwFileAttributes == FILE_ATTRIBUTE_NORMAL {
        (*ft).attrib = 0;
    } else {
        (*ft).attrib = (*fd).dwFileAttributes;
    }

    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftCreationTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_create = dw as _;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastAccessTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_access = dw as _;
    RtlTimeToSecondsSince1970(
        core::ptr::addr_of!((*fd).ftLastWriteTime) as *const LARGE_INTEGER,
        &mut dw,
    );
    (*ft).time_write = dw as _;
    (*ft).size = (*fd).nFileSizeLow as _;
    wcscpy(
        core::ptr::addr_of_mut!((*ft).name) as *mut wchar_t,
        core::ptr::addr_of!((*fd).cFileName) as *const wchar_t,
    );
}

#[no_mangle]
pub unsafe extern "C" fn _chdir(newdir: *const core::ffi::c_char) -> core::ffi::c_int {
    let mut newdirW: *mut wchar_t = core::ptr::null_mut();
    let ret: core::ffi::c_int;

    if !newdir.is_null() {
        newdirW = wstrdupa_utf8(newdir);
        if newdirW.is_null() {
            return -1;
        }
    }
    ret = _wchdir(newdirW);
    free(newdirW as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wchdir(newdir: *const wchar_t) -> core::ffi::c_int {
    if SetCurrentDirectoryW(newdir) == 0 {
        msvcrt_set_errno(if !newdir.is_null() { (GetLastError()) as i32 } else { 0 });
        return -1;
    }

    let mut fulldir: [wchar_t; MAX_PATH as usize] = [MAX_PATH as wchar_t; MAX_PATH as usize];
    if GetCurrentDirectoryW(ARRAYSIZE!(fulldir) as DWORD, fulldir.as_mut_ptr()) >= 2 {
        if fulldir[1 as usize] == b':' as wchar_t {
            let envvar: [wchar_t; 4 as usize] = [
                b'=' as wchar_t,
                towupper(fulldir[0 as usize] as _) as wchar_t,
                b':' as wchar_t,
                b'\0' as wchar_t,
            ];
            SetEnvironmentVariableW(envvar.as_ptr(), fulldir.as_ptr());
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _chdrive(newdrive: i32) -> i32 {
    let mut buffer: [wchar_t; 3] = [b'A' as wchar_t, b':' as wchar_t, 0];

    buffer[0] += (newdrive - 1) as wchar_t;
    if SetCurrentDirectoryW(buffer.as_mut_ptr()) == 0 {
        msvcrt_set_errno((GetLastError()) as i32);
        if newdrive <= 0 {
            *_errno() = (EACCES) as i32;
        }
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _findclose(hand: intptr_t) -> core::ffi::c_int {
    if FindClose(hand as HANDLE) == 0 {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _findfirst(fspec: *const core::ffi::c_char, ft: *mut _finddata_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAA = core::mem::zeroed();
    let hfind: HANDLE = FindFirstFileA(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_fttofd(&find_data, ft);
    hfind as intptr_t
}

#[no_mangle]
pub unsafe extern "C" fn _wfindfirst(fspec: *const wchar_t, ft: *mut _wfinddata_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();
    let hfind: HANDLE;

    hfind = FindFirstFileW(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_wfttofd(&find_data, ft);

    hfind as intptr_t
}

#[no_mangle]
pub unsafe extern "C" fn _wfindfirst32(fspec: *const wchar_t, ft: *mut _wfinddata32_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();
    let hfind: HANDLE;

    hfind = FindFirstFileW(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_wfttofd32(&find_data, ft);

    hfind as intptr_t
}
unsafe extern "C" fn finddata32_wtoa(wfd: *const _wfinddata32_t, fd: *mut _finddata32_t) -> core::ffi::c_int {
    (*fd).attrib = (*wfd).attrib;
    (*fd).time_create = (*wfd).time_create;
    (*fd).time_access = (*wfd).time_access;
    (*fd).time_write = (*wfd).time_write;
    (*fd).size = (*wfd).size;
    convert_wcs_to_acp_utf8(
        (*wfd).name.as_ptr(),
        (*fd).name.as_mut_ptr(),
        ARRAY_SIZE!((*fd).name) as core::ffi::c_int,
    )
}

#[no_mangle]
pub unsafe extern "C" fn _findfirst32(fspec: *const core::ffi::c_char, ft: *mut _finddata32_t) -> intptr_t {
    let mut wft: _wfinddata32_t = core::mem::zeroed();
    let mut fspecW: *mut wchar_t = core::ptr::null_mut();
    let ret: intptr_t;

    if !fspec.is_null() {
        fspecW = wstrdupa_utf8(fspec);
        if fspecW.is_null() {
            return -1;
        }
    }
    ret = _wfindfirst32(fspecW, &mut wft);
    free(fspecW as *mut core::ffi::c_void);
    if ret != -1 && finddata32_wtoa(&wft, ft) == 0 {
        _findclose(ret);
        return -1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _findfirsti64(fspec: *const core::ffi::c_char, ft: *mut _finddatai64_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAA = core::mem::zeroed();
    let hfind: HANDLE = FindFirstFileA(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_fttofdi64(&find_data, ft);

    hfind as intptr_t
}

#[no_mangle]
pub unsafe extern "C" fn _wfindfirst64(fspec: *const wchar_t, ft: *mut _wfinddata64_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();
    let hfind: HANDLE;

    hfind = FindFirstFileW(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_wfttofd64(&find_data, ft);

    hfind as intptr_t
}
unsafe extern "C" fn finddata64_wtoa(wfd: *const _wfinddata64_t, fd: *mut _finddata64_t) -> core::ffi::c_int {
    (*fd).attrib = (*wfd).attrib;
    (*fd).time_create = (*wfd).time_create;
    (*fd).time_access = (*wfd).time_access;
    (*fd).time_write = (*wfd).time_write;
    (*fd).size = (*wfd).size;
    convert_wcs_to_acp_utf8(
        (*wfd).name.as_ptr(),
        (*fd).name.as_mut_ptr(),
        ARRAY_SIZE!((*fd).name) as core::ffi::c_int,
    )
}

#[no_mangle]
pub unsafe extern "C" fn _findfirst64(fspec: *const core::ffi::c_char, ft: *mut _finddata64_t) -> intptr_t {
    let mut wft: _wfinddata64_t = core::mem::zeroed();
    let mut fspecW: *mut wchar_t = core::ptr::null_mut();
    let ret: intptr_t;

    if !fspec.is_null() {
        fspecW = wstrdupa_utf8(fspec);
        if fspecW.is_null() {
            return -1;
        }
    }
    ret = _wfindfirst64(fspecW, &mut wft);
    free(fspecW as *mut core::ffi::c_void);
    if ret != -1 && finddata64_wtoa(&wft, ft) == 0 {
        _findclose(ret);
        return -1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wfindfirst64i32(fspec: *const wchar_t, ft: *mut _wfinddata64i32_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();
    let hfind: HANDLE;

    hfind = FindFirstFileW(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_wfttofd64i32(&find_data, ft);

    hfind as intptr_t
}
unsafe extern "C" fn finddata64i32_wtoa(wfd: *const _wfinddata64i32_t, fd: *mut _finddata64i32_t) -> core::ffi::c_int {
    (*fd).attrib = (*wfd).attrib;
    (*fd).time_create = (*wfd).time_create;
    (*fd).time_access = (*wfd).time_access;
    (*fd).time_write = (*wfd).time_write;
    (*fd).size = (*wfd).size;
    convert_wcs_to_acp_utf8(
        (*wfd).name.as_ptr(),
        (*fd).name.as_mut_ptr(),
        ARRAY_SIZE!((*fd).name) as core::ffi::c_int,
    )
}

#[no_mangle]
pub unsafe extern "C" fn _findfirst64i32(fspec: *const core::ffi::c_char, ft: *mut _finddata64i32_t) -> intptr_t {
    let mut wft: _wfinddata64i32_t = core::mem::zeroed();
    let mut fspecW: *mut wchar_t = core::ptr::null_mut();
    let ret: intptr_t;

    if !fspec.is_null() {
        fspecW = wstrdupa_utf8(fspec);
        if fspecW.is_null() {
            return -1;
        }
    }
    ret = _wfindfirst64i32(fspecW, &mut wft);
    free(fspecW as *mut core::ffi::c_void);
    if ret != -1 && finddata64i32_wtoa(&wft, ft) == 0 {
        _findclose(ret);
        return -1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wfindfirsti64(fspec: *const wchar_t, ft: *mut _wfinddatai64_t) -> intptr_t {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();
    let hfind: HANDLE;

    hfind = FindFirstFileW(fspec, &mut find_data);
    if hfind == INVALID_HANDLE_VALUE {
        msvcrt_set_errno((GetLastError()) as i32);
        return -1;
    }
    msvcrt_wfttofdi64(&find_data, ft);

    hfind as intptr_t
}

#[no_mangle]
pub unsafe extern "C" fn _findnext(hand: intptr_t, ft: *mut _finddata_t) -> core::ffi::c_int {
    let mut find_data: WIN32_FIND_DATAA = core::mem::zeroed();

    if FindNextFileA(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_fttofd(&find_data, ft);
    0
}
#[no_mangle]
pub unsafe extern "C" fn _wfindnext32(hand: intptr_t, ft: *mut _wfinddata32_t) -> core::ffi::c_int {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();

    if FindNextFileW(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_wfttofd32(&find_data, ft);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _findnext32(hand: intptr_t, ft: *mut _finddata32_t) -> core::ffi::c_int {
    let mut wft: _wfinddata32_t = core::mem::zeroed();
    let mut ret = _wfindnext32(hand, &mut wft);
    if ret == 0 && finddata32_wtoa(&wft, ft) == 0 {
        ret = -1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wfindnext(hand: intptr_t, ft: *mut _wfinddata_t) -> ::core::ffi::c_int {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();

    if FindNextFileW(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_wfttofd(&find_data, ft);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _findnexti64(hand: intptr_t, ft: *mut _finddatai64_t) -> i32 {
    let mut find_data: WIN32_FIND_DATAA = core::mem::zeroed();

    if FindNextFileA(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_fttofdi64(&find_data, ft);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _wfindnext64(hand: intptr_t, ft: *mut _wfinddata64_t) -> i32 {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();

    if FindNextFileW(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_wfttofd64(&find_data, ft);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _findnext64(hand: intptr_t, ft: *mut _finddata64_t) -> core::ffi::c_int {
    let mut wft: _wfinddata64_t = core::mem::zeroed();
    let mut ret: core::ffi::c_int;

    ret = _wfindnext64(hand, &mut wft);
    if ret == 0 && finddata64_wtoa(&wft, ft) == 0 {
        ret = -1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wfindnexti64(hand: intptr_t, ft: *mut _wfinddatai64_t) -> i32 {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();

    if FindNextFileW(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_wfttofdi64(&find_data, ft);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _wfindnext64i32(hand: intptr_t, ft: *mut _wfinddata64i32_t) -> core::ffi::c_int {
    let mut find_data: WIN32_FIND_DATAW = core::mem::zeroed();

    if FindNextFileW(hand as HANDLE, &mut find_data) == 0 {
        *_errno() = (ENOENT) as i32;
        return -1;
    }

    msvcrt_wfttofd64i32(&find_data, ft);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _findnext64i32(hand: intptr_t, ft: *mut _finddata64i32_t) -> core::ffi::c_int {
    let mut wft: _wfinddata64i32_t = core::mem::zeroed();
    let mut ret: core::ffi::c_int;

    ret = _wfindnext64i32(hand, &mut wft);
    if ret == 0 && finddata64i32_wtoa(&wft, ft) == 0 {
        ret = -1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _getcwd(buf: *mut core::ffi::c_char, size: core::ffi::c_int) -> *mut core::ffi::c_char {
    let mut dirW: [wchar_t; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let len: core::ffi::c_int;

    if _wgetcwd(dirW.as_mut_ptr(), ARRAY_SIZE!(dirW) as core::ffi::c_int).is_null() {
        return core::ptr::null_mut();
    }

    if buf.is_null() {
        return astrdupw_utf8(dirW.as_mut_ptr());
    }
    len = convert_wcs_to_acp_utf8(dirW.as_mut_ptr(), core::ptr::null_mut(), 0);
    if len == 0 {
        return core::ptr::null_mut();
    }
    if len > size {
        *_errno() = (ERANGE) as i32;
        return core::ptr::null_mut();
    }
    convert_wcs_to_acp_utf8(dirW.as_mut_ptr(), buf, size);
    buf
}

#[no_mangle]
pub unsafe extern "C" fn _wgetcwd(mut buf: *mut wchar_t, mut size: core::ffi::c_int) -> *mut wchar_t {
    let mut dir: [wchar_t; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let dir_len = GetCurrentDirectoryW(MAX_PATH, dir.as_mut_ptr()) as core::ffi::c_int;

    if dir_len < 1 {
        return core::ptr::null_mut();
    }

    if buf.is_null() {
        if size <= dir_len {
            size = dir_len + 1;
        }
        buf = malloc((size as usize * core::mem::size_of::<wchar_t>()) as _) as *mut wchar_t;
        if buf.is_null() {
            return core::ptr::null_mut();
        }
    } else if dir_len >= size {
        *_errno() = (ERANGE) as i32;
        return core::ptr::null_mut();
    }
    wcscpy(buf, dir.as_ptr());
    buf
}

#[no_mangle]
pub unsafe extern "C" fn _getdrive() -> core::ffi::c_int {
    let mut buffer: [wchar_t; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    if GetCurrentDirectoryW(MAX_PATH, buffer.as_mut_ptr()) != 0
        && buffer[0] >= 'A' as wchar_t
        && buffer[0] <= 'z' as wchar_t
        && buffer[1] == ':' as wchar_t
    {
        return (towupper((buffer[0] as core::ffi::c_int) as u16) - ('A' as core::ffi::c_int) as u16 + 1) as i32;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn _getdcwd(drive: core::ffi::c_int, buf: *mut core::ffi::c_char, size: core::ffi::c_int) -> *mut core::ffi::c_char {
    let mut dirW: [wchar_t; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let len: core::ffi::c_int;

    if _wgetdcwd(drive, dirW.as_mut_ptr(), ARRAY_SIZE!(dirW) as core::ffi::c_int).is_null() {
        return core::ptr::null_mut();
    }

    if buf.is_null() {
        return astrdupw_utf8(dirW.as_mut_ptr());
    }
    len = convert_wcs_to_acp_utf8(dirW.as_mut_ptr(), core::ptr::null_mut(), 0);
    if len == 0 {
        return core::ptr::null_mut();
    }
    if len > size {
        *_errno() = (ERANGE) as i32;
        return core::ptr::null_mut();
    }
    convert_wcs_to_acp_utf8(dirW.as_mut_ptr(), buf, size);
    buf
}

#[no_mangle]
pub unsafe extern "C" fn _wgetdcwd(drive: core::ffi::c_int, buf: *mut wchar_t, size: core::ffi::c_int) -> *mut wchar_t {
    static mut dummy: *mut wchar_t = core::ptr::null_mut();

    if drive == 0 || drive == _getdrive() {
        return _wgetcwd(buf, size);
    } else {
        let mut dir: [wchar_t; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
        let mut drivespec: [wchar_t; 4] = [
            'A' as wchar_t,
            ':' as wchar_t,
            '\\' as wchar_t,
            0,
        ];
        let dir_len: core::ffi::c_int;

        drivespec[0] += (drive - 1) as wchar_t;
        if GetDriveTypeW(drivespec.as_ptr()) < DRIVE_REMOVABLE {
            *_errno() = (EACCES) as i32;
            return core::ptr::null_mut();
        }

        // C (dir.c:879/888): `int dir_len; dir_len = GetFullPathNameW(...)` — the DWORD
        // narrows to int implicitly and silently. Kept as an explicit cast rather than a
        // checked conversion: a panic here would be a NEW failure mode the C does not have.
        dir_len = GetFullPathNameW(
            drivespec.as_ptr(),
            MAX_PATH,
            dir.as_mut_ptr(),
            core::ptr::addr_of_mut!(dummy),
        ) as core::ffi::c_int;
        if dir_len >= size || dir_len < 1 {
            *_errno() = (ERANGE) as i32;
            return core::ptr::null_mut();
        }

        if buf.is_null() {
            return _wcsdup(dir.as_ptr());
        }
        wcscpy(buf, dir.as_ptr());
    }
    buf
}

#[no_mangle]
pub unsafe extern "C" fn _getdiskfree(disk: core::ffi::c_uint, d: *mut _diskfree_t) -> core::ffi::c_uint {
    let mut drivespec: [WCHAR; 4] = [b'@' as WCHAR, b':' as WCHAR, b'\\' as WCHAR, 0];
    let mut ret: [DWORD; 4] = [0; 4];
    let err: core::ffi::c_uint;

    if disk > 26 {
        return ERROR_INVALID_PARAMETER;
    }

    drivespec[0] += disk as WCHAR;

    if GetDiskFreeSpaceW(
        if disk == 0 { core::ptr::null_mut() } else { drivespec.as_mut_ptr() },
        ret.as_mut_ptr(),
        ret.as_mut_ptr().add(1),
        ret.as_mut_ptr().add(2),
        ret.as_mut_ptr().add(3),
    ) != 0
    {
        (*d).sectors_per_cluster = ret[0];
        (*d).bytes_per_sector = ret[1];
        (*d).avail_clusters = ret[2];
        (*d).total_clusters = ret[3];
        return 0;
    }
    err = GetLastError();
    msvcrt_set_errno((err) as i32);
    err
}

#[no_mangle]
pub unsafe extern "C" fn _mkdir(newdir: *const core::ffi::c_char) -> i32 {
    let mut newdirW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !newdir.is_null() {
        newdirW = wstrdupa_utf8(newdir);
        if newdirW.is_null() {
            return -1;
        }
    }
    ret = _wmkdir(newdirW);
    free(newdirW as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wmkdir(newdir: *const wchar_t) -> i32 {
    if CreateDirectoryW(newdir, core::ptr::null_mut()) != 0 {
        return 0;
    }
    msvcrt_set_errno((GetLastError()) as i32);
    -1
}

#[no_mangle]
pub unsafe extern "C" fn _rmdir(dir: *const core::ffi::c_char) -> i32 {
    let mut dirW: *mut wchar_t = core::ptr::null_mut();
    let ret: i32;

    if !dir.is_null() {
        dirW = wstrdupa_utf8(dir);
        if dirW.is_null() {
            return -1;
        }
    }
    ret = _wrmdir(dirW);
    free((dirW).cast());
    ret
}

#[no_mangle]
pub unsafe extern "C" fn _wrmdir(dir: *const wchar_t) -> i32 {
    if RemoveDirectoryW(dir) != 0 {
        return 0;
    }
    msvcrt_set_errno((GetLastError()) as i32);
    -1
}

#[no_mangle]
pub unsafe extern "C" fn _splitpath_s(
    mut inpath: *const core::ffi::c_char,
    drive: *mut core::ffi::c_char,
    sz_drive: usize,
    dir: *mut core::ffi::c_char,
    sz_dir: usize,
    fname: *mut core::ffi::c_char,
    sz_fname: usize,
    ext: *mut core::ffi::c_char,
    sz_ext: usize,
) -> errno_t {
    let mut p: *const core::ffi::c_char;
    let mut end: *const core::ffi::c_char;

    if inpath.is_null()
        || (drive.is_null() && sz_drive != 0)
        || (!drive.is_null() && sz_drive == 0)
        || (dir.is_null() && sz_dir != 0)
        || (!dir.is_null() && sz_dir == 0)
        || (fname.is_null() && sz_fname != 0)
        || (!fname.is_null() && sz_fname == 0)
        || (ext.is_null() && sz_ext != 0)
        || (!ext.is_null() && sz_ext == 0)
    {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    'do_error: {
        if *inpath.offset(0) != 0
            && *inpath.offset(1) == b':' as core::ffi::c_char
        {
            if !drive.is_null() {
                if sz_drive <= 2 {
                    break 'do_error;
                }
                *drive.offset(0) = *inpath.offset(0);
                *drive.offset(1) = *inpath.offset(1);
                *drive.offset(2) = 0;
            }
            inpath = inpath.offset(2);
        } else if !drive.is_null() {
            *drive.offset(0) = b'\0' as core::ffi::c_char;
        }

        end = core::ptr::null();
        p = inpath;
        while *p != 0 {
            if _ismbblead(*p as core::ffi::c_uchar as core::ffi::c_uint) != 0 {
                p = p.offset(1);
                p = p.offset(1);
                continue;
            }
            if *p == b'/' as core::ffi::c_char || *p == b'\\' as core::ffi::c_char {
                end = p.offset(1);
            }
            p = p.offset(1);
        }

        if !end.is_null() {
            if !dir.is_null() {
                if (sz_dir as isize) <= end.offset_from(inpath) {
                    break 'do_error;
                }
                memcpy(
                    dir as *mut core::ffi::c_void,
                    inpath as *const core::ffi::c_void,
                    end.offset_from(inpath) as usize,
                );
                *dir.offset(end.offset_from(inpath)) = 0;
            }
            inpath = end;
        } else if !dir.is_null() {
            *dir.offset(0) = 0;
        }

        end = core::ptr::null();
        p = inpath;
        while *p != 0 {
            if *p == b'.' as core::ffi::c_char {
                end = p;
            }
            p = p.offset(1);
        }

        if end.is_null() {
            end = p;
        }

        if !fname.is_null() {
            if (sz_fname as isize) <= end.offset_from(inpath) {
                break 'do_error;
            }
            memcpy(
                fname as *mut core::ffi::c_void,
                inpath as *const core::ffi::c_void,
                end.offset_from(inpath) as usize,
            );
            *fname.offset(end.offset_from(inpath)) = 0;
        }
        if !ext.is_null() {
            if sz_ext <= strlen(end) {
                break 'do_error;
            }
            strcpy(ext, end);
        }
        return 0;
    }

    if !drive.is_null() {
        *drive.offset(0) = b'\0' as core::ffi::c_char;
    }
    if !dir.is_null() {
        *dir.offset(0) = b'\0' as core::ffi::c_char;
    }
    if !fname.is_null() {
        *fname.offset(0) = b'\0' as core::ffi::c_char;
    }
    if !ext.is_null() {
        *ext.offset(0) = b'\0' as core::ffi::c_char;
    }
    *_errno() = (ERANGE) as i32;
    (ERANGE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _splitpath(
    inpath: *const core::ffi::c_char,
    drv: *mut core::ffi::c_char,
    dir: *mut core::ffi::c_char,
    fname: *mut core::ffi::c_char,
    ext: *mut core::ffi::c_char,
) {
    _splitpath_s(
        inpath,
        drv,
        if !drv.is_null() { _MAX_DRIVE as usize } else { 0 },
        dir,
        if !dir.is_null() { _MAX_DIR as usize } else { 0 },
        fname,
        if !fname.is_null() { _MAX_FNAME as usize } else { 0 },
        ext,
        if !ext.is_null() { _MAX_EXT as usize } else { 0 },
    );
}

#[no_mangle]
pub unsafe extern "C" fn _wsplitpath_s(
    mut inpath: *const wchar_t,
    drive: *mut wchar_t,
    sz_drive: usize,
    dir: *mut wchar_t,
    sz_dir: usize,
    fname: *mut wchar_t,
    sz_fname: usize,
    ext: *mut wchar_t,
    sz_ext: usize,
) -> i32 {
    let mut p: *const wchar_t;
    let mut end: *const wchar_t;

    if inpath.is_null()
        || (drive.is_null() && sz_drive != 0)
        || (!drive.is_null() && sz_drive == 0)
        || (dir.is_null() && sz_dir != 0)
        || (!dir.is_null() && sz_dir == 0)
        || (fname.is_null() && sz_fname != 0)
        || (!fname.is_null() && sz_fname == 0)
        || (ext.is_null() && sz_ext != 0)
        || (!ext.is_null() && sz_ext == 0)
    {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    'do_error: {
        if *inpath.offset(0) != 0
            && *inpath.offset(1) == ':' as wchar_t
        {
            if !drive.is_null() {
                if sz_drive <= 2 {
                    break 'do_error;
                }
                *drive.offset(0) = *inpath.offset(0);
                *drive.offset(1) = *inpath.offset(1);
                *drive.offset(2) = 0;
            }
            inpath = inpath.offset(2);
        } else if !drive.is_null() {
            *drive.offset(0) = '\0' as wchar_t;
        }

        end = core::ptr::null();
        p = inpath;
        while *p != 0 {
            if *p == '/' as wchar_t || *p == '\\' as wchar_t {
                end = p.offset(1);
            }
            p = p.offset(1);
        }

        if !end.is_null() {
            if !dir.is_null() {
                if (sz_dir as isize) <= end.offset_from(inpath) {
                    break 'do_error;
                }
                memcpy(
                    dir as *mut core::ffi::c_void,
                    inpath as *const core::ffi::c_void,
                    (end.offset_from(inpath) as usize) * core::mem::size_of::<wchar_t>(),
                );
                *dir.offset(end.offset_from(inpath)) = 0;
            }
            inpath = end;
        } else if !dir.is_null() {
            *dir.offset(0) = 0;
        }

        end = core::ptr::null();
        p = inpath;
        while *p != 0 {
            if *p == '.' as wchar_t {
                end = p;
            }
            p = p.offset(1);
        }

        if end.is_null() {
            end = p;
        }

        if !fname.is_null() {
            if (sz_fname as isize) <= end.offset_from(inpath) {
                break 'do_error;
            }
            memcpy(
                fname as *mut core::ffi::c_void,
                inpath as *const core::ffi::c_void,
                (end.offset_from(inpath) as usize) * core::mem::size_of::<wchar_t>(),
            );
            *fname.offset(end.offset_from(inpath)) = 0;
        }
        if !ext.is_null() {
            if sz_ext <= wcslen(end) {
                break 'do_error;
            }
            wcscpy(ext, end);
        }
        return 0;
    }

    if !drive.is_null() {
        *drive.offset(0) = '\0' as wchar_t;
    }
    if !dir.is_null() {
        *dir.offset(0) = '\0' as wchar_t;
    }
    if !fname.is_null() {
        *fname.offset(0) = '\0' as wchar_t;
    }
    if !ext.is_null() {
        *ext.offset(0) = '\0' as wchar_t;
    }
    *_errno() = (ERANGE) as i32;
    (ERANGE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _wsplitpath(
    inpath: *const wchar_t,
    drv: *mut wchar_t,
    dir: *mut wchar_t,
    fname: *mut wchar_t,
    ext: *mut wchar_t,
) {
    _wsplitpath_s(
        inpath,
        drv,
        if !drv.is_null() { _MAX_DRIVE as usize } else { 0 },
        dir,
        if !dir.is_null() { _MAX_DIR as usize } else { 0 },
        fname,
        if !fname.is_null() { _MAX_FNAME as usize } else { 0 },
        ext,
        if !ext.is_null() { _MAX_EXT as usize } else { 0 },
    );
}

#[no_mangle]
pub unsafe extern "C" fn _wfullpath(
    absPath: *mut wchar_t,
    relPath: *const wchar_t,
    mut size: usize,
) -> *mut wchar_t {
    let rc: u32;
    let buffer: *mut wchar_t;
    let mut lastpart: *mut wchar_t = core::ptr::null_mut();
    let mut alloced: i32 = (FALSE) as i32;

    if relPath.is_null() || *relPath == 0 {
        return _wgetcwd(absPath, size as core::ffi::c_int);
    }

    if absPath.is_null() {
        buffer = malloc(MAX_PATH as usize * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
        size = (MAX_PATH) as usize;
        alloced = (TRUE) as i32;
    } else {
        buffer = absPath;
    }

    if size < 4 {
        *_errno() = (ERANGE) as i32;
        return core::ptr::null_mut();
    }

    rc = GetFullPathNameW(relPath, size as u32, buffer, &mut lastpart);

    if rc > 0 && rc as usize <= size {
        return buffer;
    } else {
        if alloced != 0 {
            free(buffer as *mut core::ffi::c_void);
        }
        return core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn _fullpath(
    abs_path: *mut core::ffi::c_char,
    rel_path: *const core::ffi::c_char,
    size: usize,
) -> *mut core::ffi::c_char {
    let mut abs_pathW: [wchar_t; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut rel_pathW: *mut wchar_t = core::ptr::null_mut();
    let retW: *mut wchar_t;
    let len: usize;

    if !rel_path.is_null() {
        rel_pathW = wstrdupa_utf8(rel_path);
        if rel_pathW.is_null() {
            return core::ptr::null_mut();
        }
    }
    retW = _wfullpath(
        abs_pathW.as_mut_ptr(),
        rel_pathW,
        ARRAY_SIZE!(abs_pathW) as size_t,
    );
    free((rel_pathW).cast());
    if retW.is_null() {
        return core::ptr::null_mut();
    }

    if abs_path.is_null() {
        return astrdupw_utf8(abs_pathW.as_mut_ptr());
    }
    len = (convert_wcs_to_acp_utf8(abs_pathW.as_mut_ptr(), core::ptr::null_mut(), 0)) as usize;
    if len == 0 {
        return core::ptr::null_mut();
    }
    if len > size {
        *_errno() = (ERANGE) as i32;
        return core::ptr::null_mut();
    }
    convert_wcs_to_acp_utf8(abs_pathW.as_mut_ptr(), abs_path, (size) as i32);
    abs_path
}

#[no_mangle]
pub unsafe extern "C" fn _makepath(
    path: *mut core::ffi::c_char,
    drive: *const core::ffi::c_char,
    directory: *const core::ffi::c_char,
    filename: *const core::ffi::c_char,
    extension: *const core::ffi::c_char,
) {
    let mut p = path;

    if path.is_null() {
        return;
    }

    if !drive.is_null() && *drive.offset(0) != 0 {
        *p = *drive.offset(0);
        p = p.offset(1);
        *p = b':' as core::ffi::c_char;
        p = p.offset(1);
    }
    if !directory.is_null() && *directory.offset(0) != 0 {
        let len = strlen(directory) as core::ffi::c_uint;
        memmove(
            p as *mut core::ffi::c_void,
            directory as *const core::ffi::c_void,
            len as usize,
        );
        p = p.offset(len as isize);
        if *p.offset(-1) != b'/' as core::ffi::c_char
            && *p.offset(-1) != b'\\' as core::ffi::c_char
        {
            *p = b'\\' as core::ffi::c_char;
            p = p.offset(1);
        }
    }
    if !filename.is_null() && *filename.offset(0) != 0 {
        let len = strlen(filename) as core::ffi::c_uint;
        memmove(
            p as *mut core::ffi::c_void,
            filename as *const core::ffi::c_void,
            len as usize,
        );
        p = p.offset(len as isize);
    }
    if !extension.is_null() && *extension.offset(0) != 0 {
        if *extension.offset(0) != b'.' as core::ffi::c_char {
            *p = b'.' as core::ffi::c_char;
            p = p.offset(1);
        }
        strcpy(p, extension);
    } else {
        *p = b'\0' as core::ffi::c_char;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _wmakepath(path: *mut wchar_t, drive: *const wchar_t, directory: *const wchar_t,
                                    filename: *const wchar_t, extension: *const wchar_t) {
    let mut p = path;

    if path.is_null() {
        return;
    }

    if !drive.is_null() && *drive.offset(0) != 0 {
        *p = *drive.offset(0);
        p = p.offset(1);
        *p = ':' as wchar_t;
        p = p.offset(1);
    }
    if !directory.is_null() && *directory.offset(0) != 0 {
        let len = wcslen(directory) as u32;
        memmove(p as *mut _, directory as *const _, len as usize * core::mem::size_of::<wchar_t>());
        p = p.offset(len as isize);
        if *p.offset(-1) != '/' as wchar_t && *p.offset(-1) != '\\' as wchar_t {
            *p = '\\' as wchar_t;
            p = p.offset(1);
        }
    }
    if !filename.is_null() && *filename.offset(0) != 0 {
        let len = wcslen(filename) as u32;
        memmove(p as *mut _, filename as *const _, len as usize * core::mem::size_of::<wchar_t>());
        p = p.offset(len as isize);
    }
    if !extension.is_null() && *extension.offset(0) != 0 {
        if *extension.offset(0) != '.' as wchar_t {
            *p = '.' as wchar_t;
            p = p.offset(1);
        }
        wcscpy(p, extension);
    } else {
        *p = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _makepath_s(
    path: *mut core::ffi::c_char,
    mut size: usize,
    drive: *const core::ffi::c_char,
    directory: *const core::ffi::c_char,
    filename: *const core::ffi::c_char,
    extension: *const core::ffi::c_char,
) -> core::ffi::c_int {
    let mut p = path;

    if path.is_null() || size == 0 {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    'range: {
        if !drive.is_null() && *drive.offset(0) != 0 {
            if size <= 2 {
                break 'range;
            }

            *p = *drive.offset(0);
            p = p.add(1);
            *p = b':' as core::ffi::c_char;
            p = p.add(1);
            size -= 2;
        }

        if !directory.is_null() && *directory.offset(0) != 0 {
            let len = strlen(directory) as core::ffi::c_uint;
            let needs_separator = (*directory.offset((len - 1) as isize) != b'/' as core::ffi::c_char
                && *directory.offset((len - 1) as isize) != b'\\' as core::ffi::c_char) as core::ffi::c_uint;
            let copylen = min!(size - 1, len as usize) as core::ffi::c_uint;

            if size < 2 {
                break 'range;
            }

            memmove(p as *mut _, directory as *const _, copylen as usize);

            if size <= len as usize {
                break 'range;
            }

            p = p.add(copylen as usize);
            size -= copylen as usize;

            if needs_separator != 0 {
                if size < 2 {
                    break 'range;
                }

                *p = b'\\' as core::ffi::c_char;
                p = p.add(1);
                size -= 1;
            }
        }

        if !filename.is_null() && *filename.offset(0) != 0 {
            let len = strlen(filename) as core::ffi::c_uint;
            let copylen = min!(size - 1, len as usize) as core::ffi::c_uint;

            if size < 2 {
                break 'range;
            }

            memmove(p as *mut _, filename as *const _, copylen as usize);

            if size <= len as usize {
                break 'range;
            }

            p = p.add(len as usize);
            size -= len as usize;
        }

        if !extension.is_null() && *extension.offset(0) != 0 {
            let len = strlen(extension) as core::ffi::c_uint;
            let needs_period = (*extension.offset(0) != b'.' as core::ffi::c_char) as core::ffi::c_uint;
            let copylen;

            if size < 2 {
                break 'range;
            }

            if needs_period != 0 {
                *p = b'.' as core::ffi::c_char;
                p = p.add(1);
                size -= 1;
            }

            copylen = min!(size - 1, len as usize) as core::ffi::c_uint;
            memcpy(p as *mut _, extension as *const _, copylen as usize);

            if size <= len as usize {
                break 'range;
            }

            p = p.add(copylen as usize);
        }

        *p = b'\0' as core::ffi::c_char;
        return 0;
    }

    *path.offset(0) = b'\0' as core::ffi::c_char;
    *_errno() = (ERANGE) as i32;
    (ERANGE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _wmakepath_s(
    path: *mut wchar_t,
    mut size: usize,
    drive: *const wchar_t,
    directory: *const wchar_t,
    filename: *const wchar_t,
    extension: *const wchar_t,
) -> i32 {
    let mut p = path;

    if path.is_null() || size == 0 {
        *_errno() = (EINVAL) as i32;
        return (EINVAL) as i32;
    }

    'range: {
        if !drive.is_null() && *drive.offset(0) != 0 {
            if size <= 2 {
                break 'range;
            }

            *p = *drive.offset(0);
            p = p.add(1);
            *p = ':' as wchar_t;
            p = p.add(1);
            size -= 2;
        }

        if !directory.is_null() && *directory.offset(0) != 0 {
            let len = wcslen(directory) as u32;
            let needs_separator = (*directory.offset((len - 1) as isize) != '/' as wchar_t
                && *directory.offset((len - 1) as isize) != '\\' as wchar_t) as u32;
            let copylen = min!((size - 1) as u32, len);

            if size < 2 {
                break 'range;
            }

            memcpy(
                p as *mut core::ffi::c_void,
                directory as *const core::ffi::c_void,
                copylen as usize * core::mem::size_of::<wchar_t>(),
            );

            if size <= len as usize {
                break 'range;
            }

            p = p.add(copylen as usize);
            size -= copylen as usize;

            if needs_separator != 0 {
                if size < 2 {
                    break 'range;
                }

                *p = '\\' as wchar_t;
                p = p.add(1);
                size -= 1;
            }
        }

        if !filename.is_null() && *filename.offset(0) != 0 {
            let len = wcslen(filename) as u32;
            let copylen = min!((size - 1) as u32, len);

            if size < 2 {
                break 'range;
            }

            memcpy(
                p as *mut core::ffi::c_void,
                filename as *const core::ffi::c_void,
                copylen as usize * core::mem::size_of::<wchar_t>(),
            );

            if size <= len as usize {
                break 'range;
            }

            p = p.add(len as usize);
            size -= len as usize;
        }

        if !extension.is_null() && *extension.offset(0) != 0 {
            let len = wcslen(extension) as u32;
            let needs_period = (*extension.offset(0) != '.' as wchar_t) as u32;
            let copylen;

            if size < 2 {
                break 'range;
            }

            if needs_period != 0 {
                *p = '.' as wchar_t;
                p = p.add(1);
                size -= 1;
            }

            copylen = min!((size - 1) as u32, len);
            memmove(
                p as *mut core::ffi::c_void,
                extension as *const core::ffi::c_void,
                copylen as usize * core::mem::size_of::<wchar_t>(),
            );

            if size <= len as usize {
                break 'range;
            }

            p = p.add(copylen as usize);
        }

        *p = '\0' as wchar_t;
        return 0;
    }

    *path.offset(0) = '\0' as wchar_t;
    *_errno() = (ERANGE) as i32;
    (ERANGE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _searchenv_s(
    file: *const core::ffi::c_char,
    env: *const core::ffi::c_char,
    buf: *mut core::ffi::c_char,
    count: usize,
) -> core::ffi::c_int {
    let envVal: *mut core::ffi::c_char;
    let mut penv: *mut core::ffi::c_char;
    let mut end: *mut core::ffi::c_char;
    let mut path: [core::ffi::c_char; MAX_PATH as usize] =
        [0; MAX_PATH as usize];
    let mut path_len: usize;
    let fname_len: usize;
    let mut old_errno: core::ffi::c_int;
    let mut access: core::ffi::c_int;

    if MSVCRT_CHECK_PMT!(!file.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(!buf.is_null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(count > 0) == 0 {
        return (EINVAL) as i32;
    }

    if count > (MAX_PATH) as usize {
        FIXME!("count > MAX_PATH not supported\n");
    }

    fname_len = strlen(file);
    *buf = 0;

    old_errno = *_errno();
    access = _access(file, 0);
    *_errno() = old_errno;
    if access == 0 {
        if _fullpath(buf, file, count).is_null() {
            return *_errno();
        }
        return 0;
    }

    envVal = getenv(env);
    if envVal.is_null() {
        *_errno() = (ENOENT) as i32;
        return (ENOENT) as i32;
    }

    penv = envVal;

    while *penv != 0 {
        end = penv;
        path_len = 0;
        while *end != 0 && *end != b';' as core::ffi::c_char && path_len < (MAX_PATH) as usize {
            if *end == b'"' as core::ffi::c_char {
                end = end.add(1);
                while *end != 0
                    && *end != b'"' as core::ffi::c_char
                    && path_len < (MAX_PATH) as usize
                {
                    path[path_len] = *end;
                    path_len += 1;
                    end = end.add(1);
                }
                if *end == b'"' as core::ffi::c_char {
                    end = end.add(1);
                }
                continue;
            }

            path[path_len] = *end;
            path_len += 1;
            end = end.add(1);
        }

        if path_len != 0 && path_len < (MAX_PATH) as usize {
            if path[path_len - 1] != b'/' as core::ffi::c_char
                && path[path_len - 1] != b'\\' as core::ffi::c_char
            {
                path[path_len] = b'\\' as core::ffi::c_char;
                path_len += 1;
            }
            if path_len + fname_len < (MAX_PATH) as usize {
                memcpy(
                    path.as_mut_ptr().add(path_len).cast(),
                    file.cast(),
                    fname_len + 1,
                );

                old_errno = *_errno();
                access = _access(path.as_ptr(), 0);
                *_errno() = old_errno;
                if access == 0 {
                    if path_len + fname_len + 1 > count {
                        MSVCRT_INVALID_PMT!(
                            b"buf[count] is too small\0".as_ptr() as *const core::ffi::c_char,
                            (ERANGE) as i32,
                        );
                        return (ERANGE) as i32;
                    }
                    memcpy(
                        buf.cast(),
                        path.as_ptr().cast(),
                        path_len + fname_len + 1,
                    );
                    return 0;
                }
            }
        }

        penv = if *end != 0 { end.add(1) } else { end };
    }

    *_errno() = (ENOENT) as i32;
    (ENOENT) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _searchenv(
    file: *const core::ffi::c_char,
    env: *const core::ffi::c_char,
    buf: *mut core::ffi::c_char,
) {
    _searchenv_s(file, env, buf, (MAX_PATH) as usize);
}

#[no_mangle]
pub unsafe extern "C" fn _wsearchenv_s(
    file: *const wchar_t,
    env: *const wchar_t,
    buf: *mut wchar_t,
    count: usize,
) -> i32 {
    let mut penv: *mut wchar_t;
    let mut end: *mut wchar_t;
    let mut path: [wchar_t; (MAX_PATH) as usize];
    let mut path_len: usize;
    let fname_len: usize;

    path = [0; (MAX_PATH) as usize];

    if MSVCRT_CHECK_PMT!(file != core::ptr::null()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!(buf != core::ptr::null_mut()) == 0 {
        return (EINVAL) as i32;
    }
    if MSVCRT_CHECK_PMT!((count > 0) as _) == 0 {
        return (EINVAL) as i32;
    }

    if count > (MAX_PATH) as usize {
        FIXME!("count > MAX_PATH not supported\n");
    }

    fname_len = wcslen(file);
    *buf = '\0' as wchar_t;

    if GetFileAttributesW(file) != INVALID_FILE_ATTRIBUTES {
        if _wfullpath(buf, file, count).is_null() {
            return *_errno();
        }
        return 0;
    }

    let envVal = _wgetenv(env);
    if envVal.is_null() {
        *_errno() = (ENOENT) as i32;
        return (ENOENT) as i32;
    }

    penv = envVal;

    while *penv != 0 {
        end = penv;
        path_len = 0;
        while *end != 0 && *end != ';' as wchar_t && path_len < (MAX_PATH) as usize {
            if *end == '"' as wchar_t {
                end = end.add(1);
                while *end != 0 && *end != '"' as wchar_t && path_len < (MAX_PATH) as usize {
                    path[path_len] = *end;
                    path_len += 1;
                    end = end.add(1);
                }
                if *end == '"' as wchar_t {
                    end = end.add(1);
                }
                continue;
            }

            path[path_len] = *end;
            path_len += 1;
            end = end.add(1);
        }

        if path_len == 0 || path_len >= (MAX_PATH) as usize {
            penv = if *end != 0 { end.add(1) } else { end };
            continue;
        }

        if path[path_len - 1] != '/' as wchar_t
            && path[path_len - 1] != '\\' as wchar_t
        {
            path[path_len] = '\\' as wchar_t;
            path_len += 1;
        }
        if path_len + fname_len >= (MAX_PATH) as usize {
            penv = if *end != 0 { end.add(1) } else { end };
            continue;
        }

        memcpy(
            path.as_mut_ptr().add(path_len) as *mut _,
            file as *const _,
            (fname_len + 1) * core::mem::size_of::<wchar_t>(),
        );

        if GetFileAttributesW(path.as_ptr()) != INVALID_FILE_ATTRIBUTES {
            if path_len + fname_len + 1 > count {
                MSVCRT_INVALID_PMT!(
                    b"buf[count] is too small\0".as_ptr(),
                    (ERANGE) as i32,
                );
                return (ERANGE) as i32;
            }
            memcpy(
                buf as *mut _,
                path.as_ptr() as *const _,
                (path_len + fname_len + 1) * core::mem::size_of::<wchar_t>(),
            );
            return 0;
        }

        penv = if *end != 0 { end.add(1) } else { end };
    }

    *_errno() = (ENOENT) as i32;
    (ENOENT) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _wsearchenv(
    file: *const wchar_t,
    env: *const wchar_t,
    buf: *mut wchar_t,
) {
    _wsearchenv_s(file, env, buf, (MAX_PATH) as usize);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll drive/directory functions
//  *
//  * Copyright 1996,1998 Marcus Meissner
//  * Copyright 1996 Jukka Iivonen
//  * Copyright 1997,2000 Uwe Bonnes
//  * Copyright 2000 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <corecrt_io.h>
// #include <mbctype.h>
// #include <stdarg.h>
// #include <stdlib.h>
// #include <direct.h>
//
// #include "windef.h"
// #include "winbase.h"
// #include "winternl.h"
// #include "msvcrt.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// #undef _wfindfirst
// #undef _wfindfirsti64
// #undef _wfindnext
// #undef _wfindnexti64
// #undef _wfindfirst32
// #undef _wfindnext32
// #undef _wfindfirst64i32
// #undef _wfindfirst64
// #undef _wfindnext64i32
// #undef _wfindnext64
//
// #undef _findfirst
// #undef _findfirsti64
// #undef _findnext
// #undef _findnexti64
// #undef _findfirst32
// #undef _findnext32
// #undef _findfirst64i32
// #undef _findfirst64
// #undef _findnext64i32
// #undef _findnext64
//
// /* INTERNAL: Translate WIN32_FIND_DATAA to finddata_t  */
// static void msvcrt_fttofd( const WIN32_FIND_DATAA *fd, struct _finddata_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = fd->nFileSizeLow;
//   strcpy(ft->name, fd->cFileName);
// }
//
// /* INTERNAL: Translate WIN32_FIND_DATAW to wfinddata_t  */
// static void msvcrt_wfttofd( const WIN32_FIND_DATAW *fd, struct _wfinddata_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = fd->nFileSizeLow;
//   wcscpy(ft->name, fd->cFileName);
// }
//
// /* INTERNAL: Translate WIN32_FIND_DATAW to wfinddata32_t  */
// static void msvcrt_wfttofd32(const WIN32_FIND_DATAW *fd, struct _wfinddata32_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = fd->nFileSizeLow;
//   wcscpy(ft->name, fd->cFileName);
// }
//
// /* INTERNAL: Translate WIN32_FIND_DATAA to finddatai64_t  */
// static void msvcrt_fttofdi64( const WIN32_FIND_DATAA *fd, struct _finddatai64_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = ((__int64)fd->nFileSizeHigh) << 32 | fd->nFileSizeLow;
//   strcpy(ft->name, fd->cFileName);
// }
//
// /* INTERNAL: Translate WIN32_FIND_DATAW to wfinddata64_t  */
// static void msvcrt_wfttofd64( const WIN32_FIND_DATAW *fd, struct _wfinddata64_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = ((__int64)fd->nFileSizeHigh) << 32 | fd->nFileSizeLow;
//   wcscpy(ft->name, fd->cFileName);
// }
//
// /* INTERNAL: Translate WIN32_FIND_DATAW to wfinddatai64_t  */
// static void msvcrt_wfttofdi64( const WIN32_FIND_DATAW *fd, struct _wfinddatai64_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = ((__int64)fd->nFileSizeHigh) << 32 | fd->nFileSizeLow;
//   wcscpy(ft->name, fd->cFileName);
// }
//
// /* INTERNAL: Translate WIN32_FIND_DATAW to wfinddata64i32_t  */
// static void msvcrt_wfttofd64i32( const WIN32_FIND_DATAW *fd, struct _wfinddata64i32_t* ft)
// {
//   DWORD dw;
//
//   if (fd->dwFileAttributes == FILE_ATTRIBUTE_NORMAL)
//     ft->attrib = 0;
//   else
//     ft->attrib = fd->dwFileAttributes;
//
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftCreationTime, &dw );
//   ft->time_create = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastAccessTime, &dw );
//   ft->time_access = dw;
//   RtlTimeToSecondsSince1970( (const LARGE_INTEGER *)&fd->ftLastWriteTime, &dw );
//   ft->time_write = dw;
//   ft->size = fd->nFileSizeLow;
//   wcscpy(ft->name, fd->cFileName);
// }
//
// /*********************************************************************
//  *		_chdir (MSVCRT.@)
//  *
//  * Change the current working directory.
//  *
//  * PARAMS
//  *  newdir [I] Directory to change to
//  *
//  * RETURNS
//  *  Success: 0. The current working directory is set to newdir.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See SetCurrentDirectoryA.
//  */
// int CDECL _chdir(const char * newdir)
// {
//     wchar_t *newdirW = NULL;
//     int ret;
//
//     if (newdir && !(newdirW = wstrdupa_utf8(newdir))) return -1;
//     ret = _wchdir(newdirW);
//     free(newdirW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wchdir (MSVCRT.@)
//  *
//  * Unicode version of _chdir.
//  */
// int CDECL _wchdir(const wchar_t * newdir)
// {
//   if (!SetCurrentDirectoryW(newdir))
//   {
//     msvcrt_set_errno(newdir?GetLastError():0);
//     return -1;
//   }
// #ifdef __REACTOS__
//     /* Update the drive-specific current directory variable */
//     WCHAR fulldir[MAX_PATH];
//     if (GetCurrentDirectoryW(ARRAYSIZE(fulldir), fulldir) >= 2)
//     {
//         if (fulldir[1] == L':')
//         {
//             WCHAR envvar[4] = { L'=', towupper(fulldir[0]), L':', L'\0' };
//             SetEnvironmentVariableW(envvar, fulldir);
//         }
//     }
// #endif
//   return 0;
// }
//
// /*********************************************************************
//  *		_chdrive (MSVCRT.@)
//  *
//  * Change the current drive.
//  *
//  * PARAMS
//  *  newdrive [I] Drive number to change to (1 = 'A', 2 = 'B', ...)
//  *
//  * RETURNS
//  *  Success: 0. The current drive is set to newdrive.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See SetCurrentDirectoryA.
//  */
// int CDECL _chdrive(int newdrive)
// {
//   WCHAR buffer[] = L"A:";
//
//   buffer[0] += newdrive - 1;
//   if (!SetCurrentDirectoryW( buffer ))
//   {
//     msvcrt_set_errno(GetLastError());
//     if (newdrive <= 0)
//       *_errno() = EACCES;
//     return -1;
//   }
//   return 0;
// }
//
// /*********************************************************************
//  *		_findclose (MSVCRT.@)
//  *
//  * Close a handle returned by _findfirst().
//  *
//  * PARAMS
//  *  hand [I] Handle to close
//  *
//  * RETURNS
//  *  Success: 0. All resources associated with hand are freed.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See FindClose.
//  */
// int CDECL _findclose(intptr_t hand)
// {
//   TRACE(":handle %Iu\n",hand);
//   if (!FindClose((HANDLE)hand))
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   return 0;
// }
//
// /*********************************************************************
//  *		_findfirst (MSVCRT.@)
//  *
//  * Open a handle for iterating through a directory.
//  *
//  * PARAMS
//  *  fspec [I] File specification of files to iterate.
//  *  ft    [O] Information for the first file found.
//  *
//  * RETURNS
//  *  Success: A handle suitable for passing to _findnext() and _findclose().
//  *           ft is populated with the details of the found file.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See FindFirstFileA.
//  */
// intptr_t CDECL _findfirst(const char * fspec, struct _finddata_t* ft)
// {
//   WIN32_FIND_DATAA find_data;
//   HANDLE hfind;
//
//   hfind  = FindFirstFileA(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_fttofd(&find_data,ft);
//   TRACE(":got handle %p\n",hfind);
//   return (intptr_t)hfind;
// }
//
// /*********************************************************************
//  *             _wfindfirst (MSVCRT.@)
//  *
//  * Unicode version of _findfirst.
//  */
// intptr_t CDECL _wfindfirst(const wchar_t * fspec, struct _wfinddata_t* ft)
// {
//   WIN32_FIND_DATAW find_data;
//   HANDLE hfind;
//
//   hfind  = FindFirstFileW(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_wfttofd(&find_data,ft);
//   TRACE(":got handle %p\n",hfind);
//   return (intptr_t)hfind;
// }
//
// /*********************************************************************
//  *              _wfindfirst32 (MSVCRT.@)
//  *
//  * Unicode version of _findfirst32.
//  */
// intptr_t CDECL _wfindfirst32(const wchar_t * fspec, struct _wfinddata32_t* ft)
// {
//   WIN32_FIND_DATAW find_data;
//   HANDLE hfind;
//
//   hfind = FindFirstFileW(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_wfttofd32(&find_data, ft);
//   TRACE(":got handle %p\n", hfind);
//   return (intptr_t)hfind;
// }
//
// static int finddata32_wtoa(const struct _wfinddata32_t *wfd, struct _finddata32_t *fd)
// {
//     fd->attrib = wfd->attrib;
//     fd->time_create = wfd->time_create;
//     fd->time_access = wfd->time_access;
//     fd->time_write = wfd->time_write;
//     fd->size = wfd->size;
//     return convert_wcs_to_acp_utf8(wfd->name, fd->name, ARRAY_SIZE(fd->name));
// }
//
// /*********************************************************************
//  *              _findfirst32 (MSVCRT.@)
//  */
// intptr_t CDECL _findfirst32(const char *fspec, struct _finddata32_t *ft)
// {
//     struct _wfinddata32_t wft;
//     wchar_t *fspecW = NULL;
//     intptr_t ret;
//
//     if (fspec && !(fspecW = wstrdupa_utf8(fspec))) return -1;
//     ret = _wfindfirst32(fspecW, &wft);
//     free(fspecW);
//     if (ret != -1 && !finddata32_wtoa(&wft, ft))
//     {
//         _findclose(ret);
//         return -1;
//     }
//     return ret;
// }
//
// /*********************************************************************
//  *		_findfirsti64 (MSVCRT.@)
//  *
//  * 64-bit version of _findfirst.
//  */
// intptr_t CDECL _findfirsti64(const char * fspec, struct _finddatai64_t* ft)
// {
//   WIN32_FIND_DATAA find_data;
//   HANDLE hfind;
//
//   hfind  = FindFirstFileA(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_fttofdi64(&find_data,ft);
//   TRACE(":got handle %p\n",hfind);
//   return (intptr_t)hfind;
// }
//
// /*********************************************************************
//  *		_wfindfirst64 (MSVCRT.@)
//  *
//  * Unicode version of _findfirst64.
//  */
// intptr_t CDECL _wfindfirst64(const wchar_t * fspec, struct _wfinddata64_t* ft)
// {
//   WIN32_FIND_DATAW find_data;
//   HANDLE hfind;
//
//   hfind  = FindFirstFileW(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_wfttofd64(&find_data,ft);
//   TRACE(":got handle %p\n",hfind);
//   return (intptr_t)hfind;
// }
//
// static int finddata64_wtoa(const struct _wfinddata64_t *wfd, struct _finddata64_t *fd)
// {
//     fd->attrib = wfd->attrib;
//     fd->time_create = wfd->time_create;
//     fd->time_access = wfd->time_access;
//     fd->time_write = wfd->time_write;
//     fd->size = wfd->size;
//     return convert_wcs_to_acp_utf8(wfd->name, fd->name, ARRAY_SIZE(fd->name));
// }
//
// /*********************************************************************
//  *		_findfirst64 (MSVCRT.@)
//  *
//  * 64-bit version of _findfirst.
//  */
// intptr_t CDECL _findfirst64(const char *fspec, struct _finddata64_t *ft)
// {
//     struct _wfinddata64_t wft;
//     wchar_t *fspecW = NULL;
//     intptr_t ret;
//
//     if (fspec && !(fspecW = wstrdupa_utf8(fspec))) return -1;
//     ret = _wfindfirst64(fspecW, &wft);
//     free(fspecW);
//     if (ret != -1 && !finddata64_wtoa(&wft, ft))
//     {
//         _findclose(ret);
//         return -1;
//     }
//     return ret;
// }
//
// /*********************************************************************
//  *		_wfindfirst64i32 (MSVCRT.@)
//  *
//  * Unicode version of _findfirst64i32.
//  */
// intptr_t CDECL _wfindfirst64i32(const wchar_t * fspec, struct _wfinddata64i32_t* ft)
// {
//   WIN32_FIND_DATAW find_data;
//   HANDLE hfind;
//
//   hfind  = FindFirstFileW(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_wfttofd64i32(&find_data,ft);
//   TRACE(":got handle %p\n",hfind);
//   return (intptr_t)hfind;
// }
//
// static int finddata64i32_wtoa(const struct _wfinddata64i32_t *wfd, struct _finddata64i32_t *fd)
// {
//     fd->attrib = wfd->attrib;
//     fd->time_create = wfd->time_create;
//     fd->time_access = wfd->time_access;
//     fd->time_write = wfd->time_write;
//     fd->size = wfd->size;
//     return convert_wcs_to_acp_utf8(wfd->name, fd->name, ARRAY_SIZE(fd->name));
// }
//
// /*********************************************************************
//  *		_findfirst64i32 (MSVCRT.@)
//  *
//  * 64-bit/32-bit version of _findfirst.
//  */
// intptr_t CDECL _findfirst64i32(const char *fspec, struct _finddata64i32_t *ft)
// {
//     struct _wfinddata64i32_t wft;
//     wchar_t *fspecW = NULL;
//     intptr_t ret;
//
//     if (fspec && !(fspecW = wstrdupa_utf8(fspec))) return -1;
//     ret = _wfindfirst64i32(fspecW, &wft);
//     free(fspecW);
//     if (ret != -1 && !finddata64i32_wtoa(&wft, ft))
//     {
//         _findclose(ret);
//         return -1;
//     }
//     return ret;
// }
//
// /*********************************************************************
//  *		_wfindfirsti64 (MSVCRT.@)
//  *
//  * Unicode version of _findfirsti64.
//  */
// intptr_t CDECL _wfindfirsti64(const wchar_t * fspec, struct _wfinddatai64_t* ft)
// {
//   WIN32_FIND_DATAW find_data;
//   HANDLE hfind;
//
//   hfind  = FindFirstFileW(fspec, &find_data);
//   if (hfind == INVALID_HANDLE_VALUE)
//   {
//     msvcrt_set_errno(GetLastError());
//     return -1;
//   }
//   msvcrt_wfttofdi64(&find_data,ft);
//   TRACE(":got handle %p\n",hfind);
//   return (intptr_t)hfind;
// }
//
// /*********************************************************************
//  *		_findnext (MSVCRT.@)
//  *
//  * Find the next file from a file search handle.
//  *
//  * PARAMS
//  *  hand  [I] Handle to the search returned from _findfirst().
//  *  ft    [O] Information for the file found.
//  *
//  * RETURNS
//  *  Success: 0. ft is populated with the details of the found file.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See FindNextFileA.
//  */
// int CDECL _findnext(intptr_t hand, struct _finddata_t * ft)
// {
//   WIN32_FIND_DATAA find_data;
//
//   if (!FindNextFileA((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_fttofd(&find_data,ft);
//   return 0;
// }
//
// /*********************************************************************
//  *               _wfindnext32 (MSVCRT.@)
//  */
// int CDECL _wfindnext32(intptr_t hand, struct _wfinddata32_t * ft)
// {
//   WIN32_FIND_DATAW find_data;
//
//   if (!FindNextFileW((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_wfttofd32(&find_data, ft);
//   return 0;
// }
//
// /*********************************************************************
//  *               _findnext32 (MSVCRT.@)
//  */
// int CDECL _findnext32(intptr_t hand, struct _finddata32_t *ft)
// {
//     struct _wfinddata32_t wft;
//     int ret;
//
//     ret = _wfindnext32(hand, &wft);
//     if (!ret && !finddata32_wtoa(&wft, ft)) ret = -1;
//     return ret;
// }
//
// /*********************************************************************
//  *             _wfindnext (MSVCRT.@)
//  *
//  * Unicode version of _findnext.
//  */
// int CDECL _wfindnext(intptr_t hand, struct _wfinddata_t * ft)
// {
//   WIN32_FIND_DATAW find_data;
//
//   if (!FindNextFileW((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_wfttofd(&find_data,ft);
//   return 0;
// }
//
// /*********************************************************************
//  *		_findnexti64 (MSVCRT.@)
//  *
//  * 64-bit version of _findnext.
//  */
// int CDECL _findnexti64(intptr_t hand, struct _finddatai64_t * ft)
// {
//   WIN32_FIND_DATAA find_data;
//
//   if (!FindNextFileA((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_fttofdi64(&find_data,ft);
//   return 0;
// }
//
// /*********************************************************************
//  *		_wfindnext64 (MSVCRT.@)
//  *
//  * Unicode version of _wfindnext64.
//  */
// int CDECL _wfindnext64(intptr_t hand, struct _wfinddata64_t * ft)
// {
//   WIN32_FIND_DATAW find_data;
//
//   if (!FindNextFileW((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_wfttofd64(&find_data,ft);
//   return 0;
// }
//
// /*********************************************************************
//  *		_findnext64 (MSVCRT.@)
//  *
//  * 64-bit version of _findnext.
//  */
// int CDECL _findnext64(intptr_t hand, struct _finddata64_t * ft)
// {
//     struct _wfinddata64_t wft;
//     int ret;
//
//     ret = _wfindnext64(hand, &wft);
//     if (!ret && !finddata64_wtoa(&wft, ft)) ret = -1;
//     return ret;
// }
//
// /*********************************************************************
//  *		_wfindnexti64 (MSVCRT.@)
//  *
//  * Unicode version of _findnexti64.
//  */
// int CDECL _wfindnexti64(intptr_t hand, struct _wfinddatai64_t * ft)
// {
//   WIN32_FIND_DATAW find_data;
//
//   if (!FindNextFileW((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_wfttofdi64(&find_data,ft);
//   return 0;
// }
//
// /*********************************************************************
//  *		_wfindnext64i32 (MSVCRT.@)
//  *
//  * Unicode version of _findnext64i32.
//  */
// int CDECL _wfindnext64i32(intptr_t hand, struct _wfinddata64i32_t * ft)
// {
//   WIN32_FIND_DATAW find_data;
//
//   if (!FindNextFileW((HANDLE)hand, &find_data))
//   {
//     *_errno() = ENOENT;
//     return -1;
//   }
//
//   msvcrt_wfttofd64i32(&find_data,ft);
//   return 0;
// }
//
// /*********************************************************************
//  *		_findnext64i32 (MSVCRT.@)
//  *
//  * 64-bit/32-bit version of _findnext.
//  */
// int CDECL _findnext64i32(intptr_t hand, struct _finddata64i32_t *ft)
// {
//     struct _wfinddata64i32_t wft;
//     int ret;
//
//     ret = _wfindnext64i32(hand, &wft);
//     if (!ret && !finddata64i32_wtoa(&wft, ft)) ret = -1;
//     return ret;
// }
//
// /*********************************************************************
//  *		_getcwd (MSVCRT.@)
//  *
//  * Get the current working directory.
//  *
//  * PARAMS
//  *  buf  [O] Destination for current working directory.
//  *  size [I] Size of buf in characters
//  *
//  * RETURNS
//  * Success: If buf is NULL, returns an allocated string containing the path.
//  *          Otherwise populates buf with the path and returns it.
//  * Failure: NULL. errno indicates the error.
//  */
// char*  _getcwd(char * buf, int size)
// {
//     wchar_t dirW[MAX_PATH];
//     int len;
//
//     if (!_wgetcwd(dirW, ARRAY_SIZE(dirW))) return NULL;
//
//     if (!buf) return astrdupw_utf8(dirW);
//     len = convert_wcs_to_acp_utf8(dirW, NULL, 0);
//     if (!len) return NULL;
//     if (len > size)
//     {
//         *_errno() = ERANGE;
//         return NULL;
//     }
//     convert_wcs_to_acp_utf8(dirW, buf, size);
//     return buf;
// }
//
// /*********************************************************************
//  *		_wgetcwd (MSVCRT.@)
//  *
//  * Unicode version of _getcwd.
//  */
// wchar_t*  _wgetcwd(wchar_t * buf, int size)
// {
//   wchar_t dir[MAX_PATH];
//   int dir_len = GetCurrentDirectoryW(MAX_PATH,dir);
//
//   if (dir_len < 1)
//     return NULL; /* FIXME: Real return value untested */
//
//   if (!buf)
//   {
//       if (size <= dir_len) size = dir_len + 1;
//       if (!(buf = malloc( size * sizeof(WCHAR) ))) return NULL;
//   }
//   else if (dir_len >= size)
//   {
//     *_errno() = ERANGE;
//     return NULL; /* buf too small */
//   }
//   wcscpy(buf,dir);
//   return buf;
// }
//
// /*********************************************************************
//  *		_getdrive (MSVCRT.@)
//  *
//  * Get the current drive number.
//  *
//  * PARAMS
//  *  None.
//  *
//  * RETURNS
//  *  Success: The drive letter number from 1 to 26 ("A:" to "Z:").
//  *  Failure: 0.
//  */
// int CDECL _getdrive(void)
// {
//     WCHAR buffer[MAX_PATH];
//     if (GetCurrentDirectoryW( MAX_PATH, buffer ) &&
//         buffer[0] >= 'A' && buffer[0] <= 'z' && buffer[1] == ':')
//         return towupper(buffer[0]) - 'A' + 1;
//     return 0;
// }
//
// /*********************************************************************
//  *		_getdcwd (MSVCRT.@)
//  *
//  * Get the current working directory on a given disk.
//  *
//  * PARAMS
//  *  drive [I] Drive letter to get the current working directory from.
//  *  buf   [O] Destination for the current working directory.
//  *  size  [I] Length of drive in characters.
//  *
//  * RETURNS
//  *  Success: If drive is NULL, returns an allocated string containing the path.
//  *           Otherwise populates drive with the path and returns it.
//  *  Failure: NULL. errno indicates the error.
//  */
// char*  _getdcwd(int drive, char * buf, int size)
// {
//     wchar_t dirW[MAX_PATH];
//     int len;
//
//     if (!_wgetdcwd(drive, dirW, ARRAY_SIZE(dirW))) return NULL;
//
//     if (!buf) return astrdupw_utf8(dirW);
//     len = convert_wcs_to_acp_utf8(dirW, NULL, 0);
//     if (!len) return NULL;
//     if (len > size)
//     {
//         *_errno() = ERANGE;
//         return NULL;
//     }
//     convert_wcs_to_acp_utf8(dirW, buf, size);
//     return buf;
// }
//
// /*********************************************************************
//  *		_wgetdcwd (MSVCRT.@)
//  *
//  * Unicode version of _wgetdcwd.
//  */
// wchar_t*  _wgetdcwd(int drive, wchar_t * buf, int size)
// {
//   static wchar_t* dummy;
//
//   TRACE(":drive %d(%c), size %d\n",drive, drive + 'A' - 1, size);
//
//   if (!drive || drive == _getdrive())
//     return _wgetcwd(buf,size); /* current */
//   else
//   {
//     wchar_t dir[MAX_PATH];
//     wchar_t drivespec[4] = L"A:\\";
//     int dir_len;
//
//     drivespec[0] += drive - 1;
//     if (GetDriveTypeW(drivespec) < DRIVE_REMOVABLE)
//     {
//       *_errno() = EACCES;
//       return NULL;
//     }
//
//     dir_len = GetFullPathNameW(drivespec,MAX_PATH,dir,&dummy);
//     if (dir_len >= size || dir_len < 1)
//     {
//       *_errno() = ERANGE;
//       return NULL; /* buf too small */
//     }
//
//     TRACE(":returning %s\n", debugstr_w(dir));
//     if (!buf)
//       return _wcsdup(dir); /* allocate */
//     wcscpy(buf,dir);
//   }
//   return buf;
// }
//
// /*********************************************************************
//  *		_getdiskfree (MSVCRT.@)
//  *
//  * Get information about the free space on a drive.
//  *
//  * PARAMS
//  *  disk [I] Drive number to get information about (1 = 'A', 2 = 'B', ...)
//  *  info [O] Destination for the resulting information.
//  *
//  * RETURNS
//  *  Success: 0. info is updated with the free space information.
//  *  Failure: An error code from GetLastError().
//  *
//  * NOTES
//  *  See GetLastError().
//  */
// unsigned int CDECL _getdiskfree(unsigned int disk, struct _diskfree_t * d)
// {
//   WCHAR drivespec[] = L"@:\\";
//   DWORD ret[4];
//   unsigned int err;
//
//   if (disk > 26)
//     return ERROR_INVALID_PARAMETER; /* MSVCRT doesn't set errno here */
//
//   drivespec[0] += disk; /* make a drive letter */
//
//   if (GetDiskFreeSpaceW(disk==0?NULL:drivespec,ret,ret+1,ret+2,ret+3))
//   {
//     d->sectors_per_cluster = ret[0];
//     d->bytes_per_sector = ret[1];
//     d->avail_clusters = ret[2];
//     d->total_clusters = ret[3];
//     return 0;
//   }
//   err = GetLastError();
//   msvcrt_set_errno(err);
//   return err;
// }
//
// /*********************************************************************
//  *		_mkdir (MSVCRT.@)
//  *
//  * Create a directory.
//  *
//  * PARAMS
//  *  newdir [I] Name of directory to create.
//  *
//  * RETURNS
//  *  Success: 0. The directory indicated by newdir is created.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See CreateDirectoryA.
//  */
// int CDECL _mkdir(const char * newdir)
// {
//     wchar_t *newdirW = NULL;
//     int ret;
//
//     if (newdir && !(newdirW = wstrdupa_utf8(newdir))) return -1;
//     ret = _wmkdir(newdirW);
//     free(newdirW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wmkdir (MSVCRT.@)
//  *
//  * Unicode version of _mkdir.
//  */
// int CDECL _wmkdir(const wchar_t* newdir)
// {
//   if (CreateDirectoryW(newdir,NULL))
//     return 0;
//   msvcrt_set_errno(GetLastError());
//   return -1;
// }
//
// /*********************************************************************
//  *		_rmdir (MSVCRT.@)
//  *
//  * Delete a directory.
//  *
//  * PARAMS
//  *  dir [I] Name of directory to delete.
//  *
//  * RETURNS
//  *  Success: 0. The directory indicated by newdir is deleted.
//  *  Failure: -1. errno indicates the error.
//  *
//  * NOTES
//  *  See RemoveDirectoryA.
//  */
// int CDECL _rmdir(const char * dir)
// {
//     wchar_t *dirW = NULL;
//     int ret;
//
//     if (dir && !(dirW = wstrdupa_utf8(dir))) return -1;
//     ret = _wrmdir(dirW);
//     free(dirW);
//     return ret;
// }
//
// /*********************************************************************
//  *		_wrmdir (MSVCRT.@)
//  *
//  * Unicode version of _rmdir.
//  */
// int CDECL _wrmdir(const wchar_t * dir)
// {
//   if (RemoveDirectoryW(dir))
//     return 0;
//   msvcrt_set_errno(GetLastError());
//   return -1;
// }
//
// /******************************************************************
//  *		_splitpath_s (MSVCRT.@)
//  */
// errno_t CDECL _splitpath_s(const char* inpath,
//         char* drive, size_t sz_drive,
//         char* dir, size_t sz_dir,
//         char* fname, size_t sz_fname,
//         char* ext, size_t sz_ext)
// {
//     const char *p, *end;
//
//     if (!inpath || (!drive && sz_drive) ||
//             (drive && !sz_drive) ||
//             (!dir && sz_dir) ||
//             (dir && !sz_dir) ||
//             (!fname && sz_fname) ||
//             (fname && !sz_fname) ||
//             (!ext && sz_ext) ||
//             (ext && !sz_ext))
//     {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if (inpath[0] && inpath[1] == ':')
//     {
//         if (drive)
//         {
//             if (sz_drive <= 2) goto do_error;
//             drive[0] = inpath[0];
//             drive[1] = inpath[1];
//             drive[2] = 0;
//         }
//         inpath += 2;
//     }
//     else if (drive) drive[0] = '\0';
//
//     /* look for end of directory part */
//     end = NULL;
//     for (p = inpath; *p; p++)
//     {
//         if (_ismbblead((unsigned char)*p))
//         {
//             p++;
//             continue;
//         }
//         if (*p == '/' || *p == '\\') end = p + 1;
//     }
//
//     if (end)  /* got a directory */
//     {
//         if (dir)
//         {
//             if (sz_dir <= end - inpath) goto do_error;
//             memcpy( dir, inpath, (end - inpath) );
//             dir[end - inpath] = 0;
//         }
//         inpath = end;
//     }
//     else if (dir) dir[0] = 0;
//
//     /* look for extension: what's after the last dot */
//     end = NULL;
//     for (p = inpath; *p; p++) if (*p == '.') end = p;
//
//     if (!end) end = p; /* there's no extension */
//
//     if (fname)
//     {
//         if (sz_fname <= end - inpath) goto do_error;
//         memcpy( fname, inpath, (end - inpath) );
//         fname[end - inpath] = 0;
//     }
//     if (ext)
//     {
//         if (sz_ext <= strlen(end)) goto do_error;
//         strcpy( ext, end );
//     }
//     return 0;
// do_error:
//     if (drive)  drive[0] = '\0';
//     if (dir)    dir[0] = '\0';
//     if (fname)  fname[0]= '\0';
//     if (ext)    ext[0]= '\0';
//     *_errno() = ERANGE;
//     return ERANGE;
// }
//
// /*********************************************************************
//  *              _splitpath (MSVCRT.@)
//  */
// void CDECL _splitpath(const char *inpath, char *drv, char *dir,
//         char *fname, char *ext)
// {
//     _splitpath_s(inpath, drv, drv ? _MAX_DRIVE : 0, dir, dir ? _MAX_DIR : 0,
//             fname, fname ? _MAX_FNAME : 0, ext, ext ? _MAX_EXT : 0);
// }
//
// /******************************************************************
//  *		_wsplitpath_s (MSVCRT.@)
//  *
//  * Secure version of _wsplitpath
//  */
// int CDECL _wsplitpath_s(const wchar_t* inpath,
//                   wchar_t* drive, size_t sz_drive,
//                   wchar_t* dir, size_t sz_dir,
//                   wchar_t* fname, size_t sz_fname,
//                   wchar_t* ext, size_t sz_ext)
// {
//     const wchar_t *p, *end;
//
//     if (!inpath || (!drive && sz_drive) ||
//             (drive && !sz_drive) ||
//             (!dir && sz_dir) ||
//             (dir && !sz_dir) ||
//             (!fname && sz_fname) ||
//             (fname && !sz_fname) ||
//             (!ext && sz_ext) ||
//             (ext && !sz_ext))
//     {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if (inpath[0] && inpath[1] == ':')
//     {
//         if (drive)
//         {
//             if (sz_drive <= 2) goto do_error;
//             drive[0] = inpath[0];
//             drive[1] = inpath[1];
//             drive[2] = 0;
//         }
//         inpath += 2;
//     }
//     else if (drive) drive[0] = '\0';
//
//     /* look for end of directory part */
//     end = NULL;
//     for (p = inpath; *p; p++) if (*p == '/' || *p == '\\') end = p + 1;
//
//     if (end)  /* got a directory */
//     {
//         if (dir)
//         {
//             if (sz_dir <= end - inpath) goto do_error;
//             memcpy( dir, inpath, (end - inpath) * sizeof(wchar_t) );
//             dir[end - inpath] = 0;
//         }
//         inpath = end;
//     }
//     else if (dir) dir[0] = 0;
//
//     /* look for extension: what's after the last dot */
//     end = NULL;
//     for (p = inpath; *p; p++) if (*p == '.') end = p;
//
//     if (!end) end = p; /* there's no extension */
//
//     if (fname)
//     {
//         if (sz_fname <= end - inpath) goto do_error;
//         memcpy( fname, inpath, (end - inpath) * sizeof(wchar_t) );
//         fname[end - inpath] = 0;
//     }
//     if (ext)
//     {
//         if (sz_ext <= wcslen(end)) goto do_error;
//         wcscpy( ext, end );
//     }
//     return 0;
// do_error:
//     if (drive)  drive[0] = '\0';
//     if (dir)    dir[0] = '\0';
//     if (fname)  fname[0]= '\0';
//     if (ext)    ext[0]= '\0';
//     *_errno() = ERANGE;
//     return ERANGE;
// }
//
// /*********************************************************************
//  *		_wsplitpath (MSVCRT.@)
//  *
//  * Unicode version of _splitpath.
//  */
// void CDECL _wsplitpath(const wchar_t *inpath, wchar_t *drv, wchar_t *dir,
//         wchar_t *fname, wchar_t *ext)
// {
//     _wsplitpath_s(inpath, drv, drv ? _MAX_DRIVE : 0, dir, dir ? _MAX_DIR : 0,
//             fname, fname ? _MAX_FNAME : 0, ext, ext ? _MAX_EXT : 0);
// }
//
// /*********************************************************************
//  *		_wfullpath (MSVCRT.@)
//  *
//  * Unicode version of _fullpath.
//  */
// wchar_t *  _wfullpath(wchar_t * absPath, const wchar_t* relPath, size_t size)
// {
//   DWORD rc;
//   WCHAR* buffer;
//   WCHAR* lastpart;
//   BOOL alloced = FALSE;
//
//   if (!relPath || !*relPath)
//     return _wgetcwd(absPath, size);
//
//   if (absPath == NULL)
//   {
//       buffer = malloc(MAX_PATH * sizeof(WCHAR));
//       size = MAX_PATH;
//       alloced = TRUE;
//   }
//   else
//       buffer = absPath;
//
//   if (size < 4)
//   {
//     *_errno() = ERANGE;
//     return NULL;
//   }
//
//   TRACE(":resolving relative path %s\n",debugstr_w(relPath));
//
//   rc = GetFullPathNameW(relPath,size,buffer,&lastpart);
//
//   if (rc > 0 && rc <= size )
//     return buffer;
//   else
//   {
//       if (alloced)
//           free(buffer);
//         return NULL;
//   }
// }
//
// /*********************************************************************
//  *		_fullpath (MSVCRT.@)
//  *
//  * Create an absolute path from a relative path.
//  *
//  * PARAMS
//  *  absPath [O] Destination for absolute path
//  *  relPath [I] Relative path to convert to absolute
//  *  size    [I] Length of absPath in characters.
//  *
//  * RETURNS
//  * Success: If absPath is NULL, returns an allocated string containing the path.
//  *          Otherwise populates absPath with the path and returns it.
//  * Failure: NULL. errno indicates the error.
//  */
// char *  _fullpath(char *abs_path, const char *rel_path, size_t size)
// {
//     wchar_t abs_pathW[MAX_PATH], *rel_pathW = NULL, *retW;
//     size_t len;
//
//     if (rel_path && !(rel_pathW = wstrdupa_utf8(rel_path))) return NULL;
//     retW = _wfullpath(abs_pathW, rel_pathW, ARRAY_SIZE(abs_pathW));
//     free(rel_pathW);
//     if (!retW) return NULL;
//
//     if (!abs_path) return astrdupw_utf8(abs_pathW);
//     len = convert_wcs_to_acp_utf8(abs_pathW, NULL, 0);
//     if (!len) return NULL;
//     if (len > size)
//     {
//         *_errno() = ERANGE;
//         return NULL;
//     }
//     convert_wcs_to_acp_utf8(abs_pathW, abs_path, size);
//     return abs_path;
// }
//
// /*********************************************************************
//  *		_makepath (MSVCRT.@)
//  *
//  * Create a pathname.
//  *
//  * PARAMS
//  *  path      [O] Destination for created pathname
//  *  drive     [I] Drive letter (e.g. "A:")
//  *  directory [I] Directory
//  *  filename  [I] Name of the file, excluding extension
//  *  extension [I] File extension (e.g. ".TXT")
//  *
//  * RETURNS
//  *  Nothing. If path is not large enough to hold the resulting pathname,
//  *  random process memory will be overwritten.
//  */
// VOID CDECL _makepath(char * path, const char * drive,
//                      const char *directory, const char * filename,
//                      const char * extension)
// {
//     char *p = path;
//
//     TRACE("(%s %s %s %s)\n", debugstr_a(drive), debugstr_a(directory),
//           debugstr_a(filename), debugstr_a(extension) );
//
//     if ( !path )
//         return;
//
//     if (drive && drive[0])
//     {
//         *p++ = drive[0];
//         *p++ = ':';
//     }
//     if (directory && directory[0])
//     {
//         unsigned int len = strlen(directory);
//         memmove(p, directory, len);
//         p += len;
//         if (p[-1] != '/' && p[-1] != '\\')
//             *p++ = '\\';
//     }
//     if (filename && filename[0])
//     {
//         unsigned int len = strlen(filename);
//         memmove(p, filename, len);
//         p += len;
//     }
//     if (extension && extension[0])
//     {
//         if (extension[0] != '.')
//             *p++ = '.';
//         strcpy(p, extension);
//     }
//     else
//         *p = '\0';
//     TRACE("returning %s\n",path);
// }
//
// /*********************************************************************
//  *		_wmakepath (MSVCRT.@)
//  *
//  * Unicode version of _wmakepath.
//  */
// VOID CDECL _wmakepath(wchar_t *path, const wchar_t *drive, const wchar_t *directory,
//                       const wchar_t *filename, const wchar_t *extension)
// {
//     wchar_t *p = path;
//
//     TRACE("%s %s %s %s\n", debugstr_w(drive), debugstr_w(directory),
//           debugstr_w(filename), debugstr_w(extension));
//
//     if ( !path )
//         return;
//
//     if (drive && drive[0])
//     {
//         *p++ = drive[0];
//         *p++ = ':';
//     }
//     if (directory && directory[0])
//     {
//         unsigned int len = wcslen(directory);
//         memmove(p, directory, len * sizeof(wchar_t));
//         p += len;
//         if (p[-1] != '/' && p[-1] != '\\')
//             *p++ = '\\';
//     }
//     if (filename && filename[0])
//     {
//         unsigned int len = wcslen(filename);
//         memmove(p, filename, len * sizeof(wchar_t));
//         p += len;
//     }
//     if (extension && extension[0])
//     {
//         if (extension[0] != '.')
//             *p++ = '.';
//         wcscpy(p, extension);
//     }
//     else
//         *p = '\0';
//
//     TRACE("returning %s\n", debugstr_w(path));
// }
//
// /*********************************************************************
//  *		_makepath_s (MSVCRT.@)
//  *
//  * Safe version of _makepath.
//  */
// int CDECL _makepath_s(char *path, size_t size, const char *drive,
//                       const char *directory, const char *filename,
//                       const char *extension)
// {
//     char *p = path;
//
//     if (!path || !size)
//     {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if (drive && drive[0])
//     {
//         if (size <= 2)
//             goto range;
//
//         *p++ = drive[0];
//         *p++ = ':';
//         size -= 2;
//     }
//
//     if (directory && directory[0])
//     {
//         unsigned int len = strlen(directory);
//         unsigned int needs_separator = directory[len - 1] != '/' && directory[len - 1] != '\\';
//         unsigned int copylen = min(size - 1, len);
//
//         if (size < 2)
//             goto range;
//
//         memmove(p, directory, copylen);
//
//         if (size <= len)
//             goto range;
//
//         p += copylen;
//         size -= copylen;
//
//         if (needs_separator)
//         {
//             if (size < 2)
//                 goto range;
//
//             *p++ = '\\';
//             size -= 1;
//         }
//     }
//
//     if (filename && filename[0])
//     {
//         unsigned int len = strlen(filename);
//         unsigned int copylen = min(size - 1, len);
//
//         if (size < 2)
//             goto range;
//
//         memmove(p, filename, copylen);
//
//         if (size <= len)
//             goto range;
//
//         p += len;
//         size -= len;
//     }
//
//     if (extension && extension[0])
//     {
//         unsigned int len = strlen(extension);
//         unsigned int needs_period = extension[0] != '.';
//         unsigned int copylen;
//
//         if (size < 2)
//             goto range;
//
//         if (needs_period)
//         {
//             *p++ = '.';
//             size -= 1;
//         }
//
//         copylen = min(size - 1, len);
//         memcpy(p, extension, copylen);
//
//         if (size <= len)
//             goto range;
//
//         p += copylen;
//     }
//
//     *p = '\0';
//     return 0;
//
// range:
//     path[0] = '\0';
//     *_errno() = ERANGE;
//     return ERANGE;
// }
//
// /*********************************************************************
//  *		_wmakepath_s (MSVCRT.@)
//  *
//  * Safe version of _wmakepath.
//  */
// int CDECL _wmakepath_s(wchar_t *path, size_t size, const wchar_t *drive,
//                        const wchar_t *directory, const wchar_t *filename,
//                        const wchar_t *extension)
// {
//     wchar_t *p = path;
//
//     if (!path || !size)
//     {
//         *_errno() = EINVAL;
//         return EINVAL;
//     }
//
//     if (drive && drive[0])
//     {
//         if (size <= 2)
//             goto range;
//
//         *p++ = drive[0];
//         *p++ = ':';
//         size -= 2;
//     }
//
//     if (directory && directory[0])
//     {
//         unsigned int len = wcslen(directory);
//         unsigned int needs_separator = directory[len - 1] != '/' && directory[len - 1] != '\\';
//         unsigned int copylen = min(size - 1, len);
//
//         if (size < 2)
//             goto range;
//
//         memmove(p, directory, copylen * sizeof(wchar_t));
//
//         if (size <= len)
//             goto range;
//
//         p += copylen;
//         size -= copylen;
//
//         if (needs_separator)
//         {
//             if (size < 2)
//                 goto range;
//
//             *p++ = '\\';
//             size -= 1;
//         }
//     }
//
//     if (filename && filename[0])
//     {
//         unsigned int len = wcslen(filename);
//         unsigned int copylen = min(size - 1, len);
//
//         if (size < 2)
//             goto range;
//
//         memmove(p, filename, copylen * sizeof(wchar_t));
//
//         if (size <= len)
//             goto range;
//
//         p += len;
//         size -= len;
//     }
//
//     if (extension && extension[0])
//     {
//         unsigned int len = wcslen(extension);
//         unsigned int needs_period = extension[0] != '.';
//         unsigned int copylen;
//
//         if (size < 2)
//             goto range;
//
//         if (needs_period)
//         {
//             *p++ = '.';
//             size -= 1;
//         }
//
//         copylen = min(size - 1, len);
//         memcpy(p, extension, copylen * sizeof(wchar_t));
//
//         if (size <= len)
//             goto range;
//
//         p += copylen;
//     }
//
//     *p = '\0';
//     return 0;
//
// range:
//     path[0] = '\0';
//     *_errno() = ERANGE;
//     return ERANGE;
// }
//
// /*********************************************************************
//  *		_searchenv_s (MSVCRT.@)
//  */
// int CDECL _searchenv_s(const char* file, const char* env, char *buf, size_t count)
// {
//   char *envVal, *penv, *end;
//   char path[MAX_PATH];
//   size_t path_len, fname_len;
//   int old_errno, access;
//
//   if (!MSVCRT_CHECK_PMT(file != NULL)) return EINVAL;
//   if (!MSVCRT_CHECK_PMT(buf != NULL)) return EINVAL;
//   if (!MSVCRT_CHECK_PMT(count > 0)) return EINVAL;
//
//   if (count > MAX_PATH)
//       FIXME("count > MAX_PATH not supported\n");
//
//   fname_len = strlen(file);
//   *buf = '\0';
//
//   /* Try CWD first */
//   old_errno = *_errno();
//   access = _access(file, 0);
//   *_errno() = old_errno;
//   if (!access)
//   {
//     if (!_fullpath(buf, file, count))
//       return *_errno();
//     return 0;
//   }
//
//   /* Search given environment variable */
//   envVal = getenv(env);
//   if (!envVal)
//   {
//     *_errno() = ENOENT;
//     return ENOENT;
//   }
//
//   penv = envVal;
//   TRACE(":searching for %s in paths %s\n", file, envVal);
//
//   for(; *penv; penv = (*end ? end + 1 : end))
//   {
//     end = penv;
//     path_len = 0;
//     while(*end && *end != ';' && path_len < MAX_PATH)
//     {
//         if (*end == '"')
//         {
//             end++;
//             while(*end && *end != '"' && path_len < MAX_PATH)
//             {
//                 path[path_len++] = *end;
//                 end++;
//             }
//             if (*end == '"') end++;
//             continue;
//         }
//
//         path[path_len++] = *end;
//         end++;
//     }
//     if (!path_len || path_len >= MAX_PATH)
//       continue;
//
//     if (path[path_len - 1] != '/' && path[path_len - 1] != '\\')
//       path[path_len++] = '\\';
//     if (path_len + fname_len >= MAX_PATH)
//       continue;
//
//     memcpy(path + path_len, file, fname_len + 1);
//     TRACE("Checking for file %s\n", path);
//     old_errno = *_errno();
//     access = _access(path, 0);
//     *_errno() = old_errno;
//     if (!access)
//     {
//       if (path_len + fname_len + 1 > count)
//       {
//         MSVCRT_INVALID_PMT("buf[count] is too small", ERANGE);
//         return ERANGE;
//       }
//       memcpy(buf, path, path_len + fname_len + 1);
//       return 0;
//     }
//   }
//
//   *_errno() = ENOENT;
//   return ENOENT;
// }
//
// /*********************************************************************
//  *		_searchenv (MSVCRT.@)
//  */
// void CDECL _searchenv(const char* file, const char* env, char *buf)
// {
//     _searchenv_s(file, env, buf, MAX_PATH);
// }
//
// /*********************************************************************
//  *		_wsearchenv_s (MSVCRT.@)
//  */
// int CDECL _wsearchenv_s(const wchar_t* file, const wchar_t* env,
//                         wchar_t *buf, size_t count)
// {
//   wchar_t *envVal, *penv, *end;
//   wchar_t path[MAX_PATH];
//   size_t path_len, fname_len;
//
//   if (!MSVCRT_CHECK_PMT(file != NULL)) return EINVAL;
//   if (!MSVCRT_CHECK_PMT(buf != NULL)) return EINVAL;
//   if (!MSVCRT_CHECK_PMT(count > 0)) return EINVAL;
//
//   if (count > MAX_PATH)
//       FIXME("count > MAX_PATH not supported\n");
//
//   fname_len = wcslen(file);
//   *buf = '\0';
//
//   /* Try CWD first */
//   if (GetFileAttributesW( file ) != INVALID_FILE_ATTRIBUTES)
//   {
//     if (!_wfullpath(buf, file, count))
//         return *_errno();
//     return 0;
//   }
//
//   /* Search given environment variable */
//   envVal = _wgetenv(env);
//   if (!envVal)
//   {
//     *_errno() = ENOENT;
//     return ENOENT;
//   }
//
//   penv = envVal;
//   TRACE(":searching for %s in paths %s\n", debugstr_w(file), debugstr_w(envVal));
//
//   for(; *penv; penv = (*end ? end + 1 : end))
//   {
//     end = penv;
//     path_len = 0;
//     while(*end && *end != ';' && path_len < MAX_PATH)
//     {
//         if (*end == '"')
//         {
//             end++;
//             while(*end && *end != '"' && path_len < MAX_PATH)
//             {
//                 path[path_len++] = *end;
//                 end++;
//             }
//             if (*end == '"') end++;
//             continue;
//         }
//
//         path[path_len++] = *end;
//         end++;
//     }
//     if (!path_len || path_len >= MAX_PATH)
//       continue;
//
//     if (path[path_len - 1] != '/' && path[path_len - 1] != '\\')
//       path[path_len++] = '\\';
//     if (path_len + fname_len >= MAX_PATH)
//       continue;
//
//     memcpy(path + path_len, file, (fname_len + 1) * sizeof(wchar_t));
//     TRACE("Checking for file %s\n", debugstr_w(path));
//     if (GetFileAttributesW( path ) != INVALID_FILE_ATTRIBUTES)
//     {
//       if (path_len + fname_len + 1 > count)
//       {
//         MSVCRT_INVALID_PMT("buf[count] is too small", ERANGE);
//         return ERANGE;
//       }
//       memcpy(buf, path, (path_len + fname_len + 1) * sizeof(wchar_t));
//       return 0;
//     }
//   }
//
//   *_errno() = ENOENT;
//   return ENOENT;
// }
//
// /*********************************************************************
//  *      _wsearchenv (MSVCRT.@)
//  */
// void CDECL _wsearchenv(const wchar_t* file, const wchar_t* env, wchar_t *buf)
// {
//     _wsearchenv_s(file, env, buf, MAX_PATH);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

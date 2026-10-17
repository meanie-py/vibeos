use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::cursoricon::FindResourceW;
use crate::misc::imm::GetProcAddress;
use crate::windows::cursoricon::LoadResource;
use crate::internal::RtlUnicodeToMultiByteN;
#[allow(unused_imports)]
use core::ffi::c_void;

// ── LOWORD lives in crate::internal.
use crate::internal::LOWORD;

// winuser.h: RT_STRING := MAKEINTRESOURCE(6).
const RT_STRING: UINT = 6;

// winuser.h: MAKEINTRESOURCEW(i) := (LPWSTR)(ULONG_PTR)(WORD)(i).
#[inline]
fn MAKEINTRESOURCEW(i: WORD) -> LPWSTR {
    i as usize as LPWSTR
}

// <algorithm> min, reproduced (mirrors combo.rs).
#[inline]
fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

extern "C" {
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}

extern "system" {
    fn FreeLibrary(hLibModule: HINSTANCE) -> BOOL;
    fn LoadLibraryA(lpLibFileName: LPCSTR) -> HINSTANCE;
    fn SetLastError(dwErrCode: DWORD);
    // ── kernel32 resource APIs (lpName/lpType passed as MAKEINTRESOURCEW → LPWSTR).
    fn LockResource(hResData: HGLOBAL) -> LPVOID;
    // ── ntdll string conversion.
}

// ---- file-scope preamble (translated) ----
pub const CR_SUCCESS: DWORD = 0x00000000;
pub const CR_OUT_OF_MEMORY: DWORD = 0x00000002;
pub const CR_INVALID_POINTER: DWORD = 0x00000003;
pub const CR_INVALID_DATA: DWORD = 0x0000001F;

pub type CMP_REGNOTIFY =
    Option<unsafe extern "system" fn(HANDLE, LPVOID, DWORD, PHDEVNOTIFY) -> DWORD>;
pub type CMP_UNREGNOTIFY =
    Option<unsafe extern "system" fn(HDEVNOTIFY) -> DWORD>;

static mut hSetupApi: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn LoadStringW(
    instance: HINSTANCE,
    resource_id: UINT,
    buffer: LPWSTR,
    buflen: INT,
) -> INT {
    let hmem: HGLOBAL;
    let hrsrc: HRSRC;
    let mut p: *mut WCHAR;
    let string_num: i32;
    let mut i: i32;

    if buffer.is_null() {
        return 0;
    }

    hrsrc = FindResourceW(
        instance,
        MAKEINTRESOURCEW((LOWORD(resource_id) >> 4) + 1),
        RT_STRING as LPWSTR,
    );
    if hrsrc.is_null() {
        return 0;
    }
    hmem = LoadResource(instance, hrsrc);
    if hmem.is_null() {
        return 0;
    }

    p = LockResource(hmem) as *mut WCHAR;
    string_num = (resource_id & 0x000f) as i32;
    i = 0;
    while i < string_num {
        p = p.offset((*p as isize) + 1);
        i += 1;
    }

    if buflen == 0 {
        *(buffer as *mut LPWSTR) = p.offset(1);
        return *p as INT;
    }

    i = min((buflen - 1) as i32, *p as i32);
    if i > 0 {
        memcpy(
            buffer as LPVOID,
            p.offset(1) as LPVOID,
            (i as usize) * core::mem::size_of::<WCHAR>(),
        );
        *buffer.offset(i as isize) = 0;
    } else {
        if buflen > 1 {
            *buffer.offset(0) = 0;
            return 0;
        }
    }

    i
}

#[no_mangle]
pub unsafe extern "system" fn LoadStringA(
    instance: HINSTANCE,
    resource_id: UINT,
    buffer: LPSTR,
    buflen: INT,
) -> INT {
    let hmem: HGLOBAL;
    let hrsrc: HRSRC;
    let mut retval: DWORD = 0;

    if buflen == 0 {
        return -1;
    }

    hrsrc = FindResourceW(
        instance,
        MAKEINTRESOURCEW((LOWORD(resource_id) >> 4) + 1),
        RT_STRING as LPWSTR,
    );
    if !hrsrc.is_null() {
        hmem = LoadResource(instance, hrsrc);
        if !hmem.is_null() {
            let mut p = LockResource(hmem) as *const WCHAR;
            let mut id = resource_id & 0x000f;

            while id != 0 {
                id -= 1;
                p = p.add(*p as usize + 1);
            }

            if buflen != 1 {
                RtlUnicodeToMultiByteN(
                    buffer,
                    (buflen - 1) as ULONG,
                    &mut retval,
                    p.add(1) as PWSTR,
                    (*p as ULONG) * core::mem::size_of::<WCHAR>() as ULONG,
                );
            }
        }
    }

    *buffer.add(retval as usize) = 0;
    retval as INT
}

#[no_mangle]
pub unsafe extern "system" fn RegisterDeviceNotificationW(
    hRecipient: HANDLE,
    NotificationFilter: LPVOID,
    Flags: DWORD,
) -> HDEVNOTIFY {

    type CMP_REGNOTIFY = Option<
        unsafe extern "system" fn(HANDLE, LPVOID, DWORD, *mut HDEVNOTIFY) -> DWORD,
    >;

    let mut ConfigRet: DWORD;
    let RegNotify: CMP_REGNOTIFY;
    let mut hDevNotify: HDEVNOTIFY = core::ptr::null_mut();

    if hSetupApi.is_null() {
        hSetupApi = LoadLibraryA(b"SETUPAPI.DLL\0".as_ptr() as LPCSTR);
    }
    if hSetupApi.is_null() {
        return core::ptr::null_mut();
    }

    RegNotify = core::mem::transmute::<_, CMP_REGNOTIFY>(GetProcAddress(
        hSetupApi,
        b"CMP_RegisterNotification\0".as_ptr() as LPCSTR,
    ));
    if RegNotify.is_none() {
        FreeLibrary(hSetupApi);
        hSetupApi = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    ConfigRet = (RegNotify.unwrap())(hRecipient, NotificationFilter, Flags, &mut hDevNotify);
    if ConfigRet != CR_SUCCESS {
        match ConfigRet {
            CR_OUT_OF_MEMORY => SetLastError(ERROR_NOT_ENOUGH_MEMORY),
            CR_INVALID_POINTER => SetLastError(ERROR_INVALID_PARAMETER),
            CR_INVALID_DATA => SetLastError(ERROR_INVALID_DATA),
            _ => SetLastError(ERROR_SERVICE_SPECIFIC_ERROR),
        }
    }

    hDevNotify
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterDeviceNotification(Handle: HDEVNOTIFY) -> BOOL {
    let mut ConfigRet: DWORD;
    let UnRegNotify: CMP_UNREGNOTIFY;

    if hSetupApi.is_null() {
        hSetupApi = LoadLibraryA(b"SETUPAPI.DLL\0".as_ptr() as LPSTR);
    }
    if hSetupApi.is_null() {
        return 0;
    }

    let proc = GetProcAddress(hSetupApi, b"CMP_UnregisterNotification\0".as_ptr() as LPSTR);
    UnRegNotify = core::mem::transmute::<_, CMP_UNREGNOTIFY>(proc);
    if UnRegNotify.is_none() {
        FreeLibrary(hSetupApi);
        hSetupApi = core::ptr::null_mut();
        return 0;
    }

    ConfigRet = (UnRegNotify.unwrap())(Handle);
    if ConfigRet != CR_SUCCESS {
        match ConfigRet {
            CR_INVALID_POINTER => {
                SetLastError(ERROR_INVALID_PARAMETER);
            }
            CR_INVALID_DATA => {
                SetLastError(ERROR_INVALID_DATA);
            }
            _ => {
                SetLastError(ERROR_SERVICE_SPECIFIC_ERROR);
            }
        }
        return 0;
    }

    1
}

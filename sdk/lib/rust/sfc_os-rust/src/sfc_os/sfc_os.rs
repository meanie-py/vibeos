use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hLibModule: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;


#[repr(C)]
pub struct _PROTECTED_FILE_DATA {
    pub FileName: [WCHAR; (MAX_PATH) as _],
    pub FileNumber: DWORD,
}

pub type PROTECTED_FILE_DATA = _PROTECTED_FILE_DATA;
pub type PPROTECTED_FILE_DATA = *mut _PROTECTED_FILE_DATA;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            hLibModule = hinstDLL;
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SfcIsFileProtected(RpcHandle: HANDLE, ProtFileName: LPCWSTR) -> BOOL {
    static mut reported: BOOL = (FALSE as i32);

    if reported != 0 {
        DPRINT!("(%p, %S) stub\n", RpcHandle, ProtFileName);
    } else {
        DPRINT1!("(%p, %S) stub\n", RpcHandle, ProtFileName);
        reported = (TRUE as i32);
    }

    SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SfcIsKeyProtected(hKey: HKEY, lpSubKey: LPCWSTR, samDesired: REGSAM) -> BOOL {
    static mut reported: BOOL = (FALSE as i32);

    if reported != 0 {
        DPRINT!("(%p, %S) stub\n", hKey, lpSubKey);
    } else {
        DPRINT1!("(%p, %S) stub\n", hKey, lpSubKey);
        reported = (TRUE as i32);
    }

    if hKey.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SfcGetNextProtectedFile(RpcHandle: HANDLE, ProtFileData: PPROTECTED_FILE_DATA) -> BOOL {
    if ProtFileData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    UNIMPLEMENTED!();
    SetLastError(ERROR_NO_MORE_FILES);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SfcFileException(
    dwUnknown0: DWORD,
    pwszFile: PWCHAR,
    dwUnknown1: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    0
}

#[no_mangle]
pub unsafe extern "system" fn SfcWLEventLogoff(pInfo: PWLX_NOTIFICATION_INFO) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn SfcWLEventLogon(pInfo: PWLX_NOTIFICATION_INFO) {
    // UNIMPLEMENTED;
}

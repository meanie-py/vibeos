// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7E1E5EB0D078: [u16; 13] = [
    0x0073u16, 0x0072, 0x0063, 0x006C, 0x0069, 0x0065, 0x006E, 0x0074, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hLibModule: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL);
            hLibModule = hinstDLL;
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn sfc_8() -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn sfc_9() -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SRSetRestorePointA(pRestorePtSpec: PRESTOREPOINTINFOA, pSMgrStatus: PSTATEMGRSTATUS) -> BOOL {
    let mut hModule: HMODULE;
    let mut pSRSRPA: PSRSRPA;
    let mut bStatus: BOOL = (FALSE as i32);
    let mut lpLibFileName: LPCWSTR = RUSTCONV_WSTR_7E1E5EB0D078.as_ptr();
    let mut lpProcName: LPCSTR = b"SRSetRestorePointA\0".as_ptr() as LPCSTR;

    hModule = LoadLibraryW(lpLibFileName) as HMODULE;
    if !hModule.is_null() {
        pSRSRPA = ::core::mem::transmute::<FARPROC, PSRSRPA>(
            GetProcAddress(hModule as HINSTANCE, lpProcName),
        );
        if pSRSRPA.is_some() {
            let pSRSRPA = pSRSRPA.unwrap_unchecked();
            bStatus = pSRSRPA(pRestorePtSpec, pSMgrStatus);
        } else {
            if !pSMgrStatus.is_null() {
                (*pSMgrStatus).nStatus = ERROR_CALL_NOT_IMPLEMENTED;
            }
        }
        FreeLibrary(hModule);
    } else {
        if !pSMgrStatus.is_null() {
            (*pSMgrStatus).nStatus = ERROR_CALL_NOT_IMPLEMENTED;
        }
    }

    return bStatus;
}

#[no_mangle]
pub unsafe extern "system" fn SRSetRestorePointW(
    pRestorePtSpec: PRESTOREPOINTINFOW,
    pSMgrStatus: PSTATEMGRSTATUS,
) -> BOOL {
    let mut hModule: HMODULE;
    let mut pSRSRPW: PSRSRPW;
    let mut bStatus: BOOL = (FALSE as i32);
    let mut lpLibFileName: LPCWSTR = RUSTCONV_WSTR_7E1E5EB0D078.as_ptr();
    let mut lpProcName: LPCSTR = b"SRSetRestorePointW\0".as_ptr() as LPCSTR;

    hModule = LoadLibraryW(lpLibFileName) as HMODULE;
    if !hModule.is_null() {
        pSRSRPW = ::core::mem::transmute::<FARPROC, PSRSRPW>(
            GetProcAddress(hModule as HINSTANCE, lpProcName),
        );
        if pSRSRPW.is_some() {
            bStatus = (pSRSRPW.unwrap_unchecked())(pRestorePtSpec, pSMgrStatus);
        } else {
            if !pSMgrStatus.is_null() {
                (*pSMgrStatus).nStatus = ERROR_CALL_NOT_IMPLEMENTED;
            }
        }
        FreeLibrary(hModule);
    } else {
        if !pSMgrStatus.is_null() {
            (*pSMgrStatus).nStatus = ERROR_CALL_NOT_IMPLEMENTED;
        }
    }

    return bStatus;
}

#[no_mangle]
pub unsafe extern "system" fn SfpVerifyFile(pszFileName: LPCSTR, pszError: LPSTR, dwErrSize: DWORD) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

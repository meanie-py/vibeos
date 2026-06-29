use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type fnMsiProvideComponentFromDescriptor = core::option::Option<
    unsafe extern "system" fn(LPCWSTR, LPWSTR, *mut DWORD, *mut DWORD) -> UINT,
>;

#[no_mangle]
pub unsafe extern "system" fn CommandLineFromMsiDescriptor(
    szDescriptor: *mut WCHAR,
    szCommandLine: *mut WCHAR,
    pcchCommandLine: *mut DWORD,
) -> DWORD {
    let szMsi: [WCHAR; 4] = [b'm' as WCHAR, b's' as WCHAR, b'i' as WCHAR, 0];
    let mut r: UINT = ERROR_CALL_NOT_IMPLEMENTED;

    TRACE!(
        b"%S %p %p\n\0".as_ptr() as *const i8,
        szDescriptor,
        szCommandLine,
        pcchCommandLine,
    );

    let hmsi: HMODULE = LoadLibraryW(szMsi.as_ptr());
    if hmsi.is_null() {
        return r;
    }
    let proc = GetProcAddress(
        hmsi,
        b"MsiProvideComponentFromDescriptorW\0".as_ptr() as *const i8,
    );
    if !(proc as *mut core::ffi::c_void).is_null() {
        let mpcfd: fnMsiProvideComponentFromDescriptor = core::mem::transmute(proc);
        r = (mpcfd.unwrap())(
            szDescriptor,
            szCommandLine,
            pcchCommandLine,
            core::ptr::null_mut(),
        );
    }
    FreeLibrary(hmsi);
    r
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/misc/msi.c
//  * PURPOSE:         advapi32.dll MSI interface funcs
//  * NOTES:           Copied from Wine
//  *                  Copyright 1995 Sven Verdoolaege
//  */
//
// #include <advapi32.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
// #ifndef _UNICODE
// #define debugstr_aw debugstr_a
// #else
// #define debugstr_aw debugstr_w
// #endif
//
//
// typedef UINT (WINAPI *fnMsiProvideComponentFromDescriptor)(LPCWSTR,LPWSTR,DWORD*,DWORD*);
//
// DWORD WINAPI CommandLineFromMsiDescriptor( WCHAR *szDescriptor,
//                     WCHAR *szCommandLine, DWORD *pcchCommandLine )
// {
//     static const WCHAR szMsi[] = { 'm','s','i',0 };
//     fnMsiProvideComponentFromDescriptor mpcfd;
//     HMODULE hmsi;
//     UINT r = ERROR_CALL_NOT_IMPLEMENTED;
//
//     TRACE("%S %p %p\n", szDescriptor, szCommandLine, pcchCommandLine);
//
//     hmsi = LoadLibraryW( szMsi );
//     if (!hmsi)
//         return r;
//     mpcfd = (void*) GetProcAddress( hmsi, "MsiProvideComponentFromDescriptorW" );
//     if (mpcfd)
//         r = mpcfd( szDescriptor, szCommandLine, pcchCommandLine, NULL );
//     FreeLibrary( hmsi );
//     return r;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

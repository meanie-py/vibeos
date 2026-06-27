use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn DisableThreadLibraryCalls(hLibModule: HINSTANCE) -> i32;
}
// CloseLogonLsaHandle (logon.c, Wave 3) is plain C (cdecl), NOT WINAPI →
// extern "C" (i386: `_CloseLogonLsaHandle`, no @N stdcall decoration).
extern "C" {
    fn CloseLogonLsaHandle();
}

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, or table declarations to translate.

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDll: HINSTANCE,
    dwReason: u32,
    reserved: LPVOID,
) -> i32 {

    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDll);
            RegInitialize();
        }
        DLL_PROCESS_DETACH => {
            CloseLogonLsaHandle();
            RegCleanup();
            UnloadNtMarta();
            CloseKsecDdHandle();
        }
        _ => {}
    }

    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/misc/dllmain.c
//  * PURPOSE:         Library main function
//  * PROGRAMMER:      ???
//  * UPDATE HISTORY:
//  *                  Created ???
//  */
//
// #include <advapi32.h>
//
// extern BOOL RegInitialize(VOID);
// extern BOOL RegCleanup(VOID);
// extern VOID UnloadNtMarta(VOID);
// extern VOID CloseKsecDdHandle(VOID);
//
// BOOL
// WINAPI
// DllMain(
//     HINSTANCE hinstDll,
//     DWORD dwReason,
//     LPVOID reserved)
// {
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             DisableThreadLibraryCalls(hinstDll);
//             RegInitialize();
//             break;
//
//         case DLL_PROCESS_DETACH:
//             CloseLogonLsaHandle();
//             RegCleanup();
//             UnloadNtMarta();
//             CloseKsecDdHandle();
//             break;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

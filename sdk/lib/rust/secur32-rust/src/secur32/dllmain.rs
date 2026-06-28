use crate::prelude::*;
use core::ffi::c_void;

// All three are plain-C (cdecl, VOID …(VOID)) internal helpers — NOT WINAPI →
// extern "C" (i386: `_Name`, no @N stdcall decoration). LsapInit/CloseLsaPort
// live in lsalpc.c; SECUR32_freeProviders in sspi.c.
extern "C" {
    fn LsapInitLsaPort();
    fn LsapCloseLsaPort();
    fn SECUR32_freeProviders();
}

// ---- file-scope preamble (translated) ----
pub static mut Secur32Heap: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hInstance: HINSTANCE,
    Reason: ULONG,
    Reserved: PVOID,
) -> BOOL {

    match Reason {
        DLL_PROCESS_ATTACH => {
            Secur32Heap = RtlCreateHeap(
                0,
                core::ptr::null_mut(),
                0,
                4096,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if Secur32Heap.is_null() {
                return 0;
            }
            LsapInitLsaPort();
        }
        DLL_PROCESS_DETACH => {
            SECUR32_freeProviders();
            LsapCloseLsaPort();
            if RtlDestroyHeap(Secur32Heap).is_null() {
                return 0;
            }
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
//  * FILE:            lib/secur32/lsa.c
//  * PURPOSE:         Client-side LSA functions
//  * UPDATE HISTORY:
//  *                  Created 05/08/00
//  */
//
// #include "precomp.h"
//
// VOID LsapInitLsaPort(VOID);
// VOID LsapCloseLsaPort(VOID);
//
// /* GLOBALS *******************************************************************/
//
// HANDLE Secur32Heap;
//
// /* FUNCTIONS *****************************************************************/
//
// #ifdef __REACTOS__
// // See sspi.c
// extern void SECUR32_freeProviders(void);
// #endif
//
// BOOL
// WINAPI
// DllMain(HINSTANCE hInstance,
//         ULONG Reason,
//         PVOID Reserved)
// {
//     switch (Reason)
//     {
//         case DLL_PROCESS_ATTACH:
//             Secur32Heap = RtlCreateHeap(0, NULL, 0, 4096, NULL, NULL);
//             if (Secur32Heap == 0)
//             {
//                 return FALSE;
//             }
//             LsapInitLsaPort();
//             break;
//
//         case DLL_PROCESS_DETACH:
// #ifdef __REACTOS__
//             SECUR32_freeProviders();
// #endif
//             LsapCloseLsaPort();
//             if (!RtlDestroyHeap(Secur32Heap))
//             {
//                 return FALSE;
//             }
//             break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

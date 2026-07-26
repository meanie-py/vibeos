use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    Instance: HINSTANCE,
    Reason: u32,
    Reserved: LPVOID,
) -> u8 {
    if Reason == DLL_PROCESS_ATTACH {
        DisableThreadLibraryCalls(Instance);
    }
    (TRUE) as u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Winsock 2 SPI
//  * FILE:        lib/mswsock/lib/init.c
//  * PURPOSE:     DLL Initialization
//  */
//
// #include "precomp.h"
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// WINAPI
// DllMain(HINSTANCE Instance,
//         DWORD Reason,
//         LPVOID Reserved)
// {
//     /* Check if we're being attached */
//     if (Reason == DLL_PROCESS_ATTACH)
//     {
//         /* Let's disable TLC calls as an optimization */
//         DisableThreadLibraryCalls(Instance);
//     }
//
//     /* We're done */
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    InstDLL: HINSTANCE,
    Reason: DWORD,
    Reserved: LPVOID,
) -> BOOL {
    (TRUE) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock DLL
//  * FILE:        stubs.c
//  * PURPOSE:     Stub functions
//  * PROGRAMMERS: Ge van Geldorp (ge@gse.nl)
//  * REVISIONS:
//  */
//
// #include <windef.h>
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// DllMain(HINSTANCE InstDLL,
//         DWORD Reason,
//         LPVOID Reserved)
// {
//   return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

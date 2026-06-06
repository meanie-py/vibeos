use ntdll_sys::nt::*;

#[no_mangle]
pub unsafe extern "system" fn SmpCheckForCrashDump(_FileName: PUNICODE_STRING) -> BOOLEAN {
    0u8
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/crashdmp.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// BOOLEAN
// NTAPI
// SmpCheckForCrashDump(IN PUNICODE_STRING FileName)
// {
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

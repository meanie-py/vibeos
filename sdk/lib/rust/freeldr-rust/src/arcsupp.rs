use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCSTR, PSTR};
use core::ffi::c_void;

extern "C" {
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn strlen(s: PCSTR) -> usize;
}

#[no_mangle]
pub unsafe extern "C" fn GetNextArgumentValue(
    Argc: u32,
    Argv: *mut PSTR,
    LastIndex: *mut u32,
    ArgumentName: PCSTR,
) -> PSTR {

    let ArgNameLen = strlen(ArgumentName);
    let start_index = if !LastIndex.is_null() { *LastIndex } else { 0 };

    for i in start_index..Argc {
        let argv_i = *Argv.add(i as usize);
        if !argv_i.is_null() &&
            strlen(argv_i as PCSTR) >= ArgNameLen + 1 &&
            _strnicmp(argv_i as PCSTR, ArgumentName, ArgNameLen) == 0 &&
            *argv_i.add(ArgNameLen) == b'=' as i8
        {
            if !LastIndex.is_null() {
                *LastIndex = i;
            }
            return argv_i.add(ArgNameLen + 1);
        }
    }

    if !LastIndex.is_null() {
        *LastIndex = u32::MAX;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn GetArgumentValue(
    Argc: u32,
    Argv: *mut PSTR,
    ArgumentName: PCSTR,
) -> PSTR {
    GetNextArgumentValue(Argc, Argv, core::ptr::null_mut(), ArgumentName)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Generic ARC Support Functions
//  * COPYRIGHT:   Copyright 2019-2024 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <freeldr.h>
//
// /* FUNCTIONS *****************************************************************/
//
// PSTR
// GetNextArgumentValue(
//     _In_ ULONG Argc,
//     _In_ PCHAR Argv[],
//     _Inout_opt_ PULONG LastIndex,
//     _In_ PCSTR ArgumentName)
// {
//     ULONG i;
//     SIZE_T ArgNameLen = strlen(ArgumentName);
//
//     for (i = (LastIndex ? *LastIndex : 0); i < Argc; ++i)
//     {
//         if (Argv[i] /* NULL pointer is a valid entry in Argv: skip it */ &&
//             (strlen(Argv[i]) >= ArgNameLen + 1 /* Count the '=' sign */) &&
//             (_strnicmp(Argv[i], ArgumentName, ArgNameLen) == 0) &&
//             (Argv[i][ArgNameLen] == '='))
//         {
//             /* Found it, return the value */
//             if (LastIndex) *LastIndex = i;
//             return &Argv[i][ArgNameLen + 1];
//         }
//     }
//
//     if (LastIndex) *LastIndex = (ULONG)-1;
//     return NULL;
// }
//
// PSTR
// GetArgumentValue(
//     _In_ ULONG Argc,
//     _In_ PCHAR Argv[],
//     _In_ PCSTR ArgumentName)
// {
//     return GetNextArgumentValue(Argc, Argv, NULL, ArgumentName);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

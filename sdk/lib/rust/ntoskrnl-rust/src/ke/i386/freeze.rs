use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;

use ntkrnl_sys::ddk::{
    KCONTINUE_STATUS,
    _KCONTINUE_STATUS_ContinueError as ContinueError,
};

#[no_mangle]
pub unsafe extern "system" fn KxFreezeExecution() {
    // UNIMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KxThawExecution() {
    // UNIMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KxSwitchKdProcessor(ProcessorIndex: ULONG) -> KCONTINUE_STATUS {
    // UNIMPLEMENTED
    ContinueError
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Processor freeze support for i386
//  * COPYRIGHT:
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// KxFreezeExecution(
//     VOID)
// {
//     UNIMPLEMENTED;
// }
//
// VOID
// NTAPI
// KxThawExecution(
//     VOID)
// {
//     UNIMPLEMENTED;
// }
//
// KCONTINUE_STATUS
// NTAPI
// KxSwitchKdProcessor(
//     _In_ ULONG ProcessorIndex)
// {
//     UNIMPLEMENTED;
//     return ContinueError;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

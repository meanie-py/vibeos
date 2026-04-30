use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{HCELL_INDEX, PCM_KEY_BODY, PCM_KEY_CONTROL_BLOCK, PHHIVE};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;
#[no_mangle]
pub unsafe extern "system" fn CmpReportNotify(
    Kcb: PCM_KEY_CONTROL_BLOCK,
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    Filter: u32,
) {
    let Kcb = opaque_ptr(Kcb);
    let Hive = opaque_ptr(Hive);
    return;
}

#[no_mangle]
pub unsafe extern "system" fn CmpFlushNotify(
    KeyBody: PCM_KEY_BODY,
    LockHeld: u8,
) {
    let KeyBody = opaque_ptr(KeyBody);
    return;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmnotify.c
//  * PURPOSE:         Configuration Manager - Wrappers for Hive Operations
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// CmpReportNotify(IN PCM_KEY_CONTROL_BLOCK Kcb,
//                 IN PHHIVE Hive,
//                 IN HCELL_INDEX Cell,
//                 IN ULONG Filter)
// {
//     /* FIXME: TODO */
//     return;
// }
//
// VOID
// NTAPI
// CmpFlushNotify(IN PCM_KEY_BODY KeyBody,
//                IN BOOLEAN LockHeld)
// {
//     /* FIXME: TODO */
//     return;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

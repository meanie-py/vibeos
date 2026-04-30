use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCMHIVE};
use core::ffi::c_void;

use ntkrnl_sys::opaque_ptr;
#[no_mangle]
pub unsafe extern "system" fn CmpInitSecurityCache(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    let mut i: u32;

    (*Hive).SecurityCount = 0;
    (*Hive).SecurityCacheSize = 0;
    (*Hive).SecurityHitHint = -1i32;
    (*Hive).SecurityCache = core::ptr::null_mut();

    i = 0;
    while i < 64 {
        InitializeListHead(core::ptr::addr_of_mut!((*Hive).SecurityHash[i as usize]));
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpDestroySecurityCache(Hive: PCMHIVE) {
    let Hive = opaque_ptr(Hive);
    (*Hive).SecurityCount = 0;
    (*Hive).SecurityCacheSize = 0;
    (*Hive).SecurityHitHint = -1i32;
    (*Hive).SecurityCache = core::ptr::null_mut();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmsecach.c
//  * PURPOSE:         Configuration Manager - Security Cache
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// CmpInitSecurityCache(IN PCMHIVE Hive)
// {
//     ULONG i;
//
//     /* Reset data */
//     Hive->SecurityCount = 0;
//     Hive->SecurityCacheSize = 0;
//     Hive->SecurityHitHint = -1;
//     Hive->SecurityCache = NULL;
//
//     /* Loop every security hash */
//     for (i = 0; i < CMP_SECURITY_HASH_LISTS; i++)
//     {
//         /* Initialize it */
//         InitializeListHead(&Hive->SecurityHash[i]);
//     }
// }
//
// VOID
// NTAPI
// CmpDestroySecurityCache(IN PCMHIVE Hive)
// {
//     /* FIXME: clean Hive->SecurityHash and/or Hive->SecurityCache */
//
//     /* Reset data */
//     Hive->SecurityCount = 0;
//     Hive->SecurityCacheSize = 0;
//     Hive->SecurityHitHint = -1;
//     Hive->SecurityCache = NULL;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

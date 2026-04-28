use ntkrnl_sys::base::BOOLEAN;

// Globals originally defined in ntoskrnl/config/cmquota.c (now owned by this Rust file
// since cmquota.c is commented out in ntos.cmake).
#[no_mangle]
pub static mut CmpGlobalQuota: BOOLEAN = 0;
#[no_mangle]
pub static mut CmpGlobalQuotaAllowed: BOOLEAN = 0;

#[no_mangle]
pub unsafe extern "system" fn CmpSetGlobalQuotaAllowed() {
    CmpGlobalQuotaAllowed = CmpGlobalQuota;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmquota.c
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
// BOOLEAN CmpGlobalQuota;
// BOOLEAN CmpGlobalQuotaAllowed;
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// NTAPI
// CmpSetGlobalQuotaAllowed(VOID)
// {
//     /* Set flag ON if quota enabled */
//     CmpGlobalQuotaAllowed = CmpGlobalQuota;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

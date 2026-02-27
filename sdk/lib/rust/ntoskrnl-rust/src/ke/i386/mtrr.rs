use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiInitializeMTRR(FinalCpu: BOOLEAN) {
    // FIXME: Support this
    // DPRINT("MTRR support detected but not yet taken advantage of\n");
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn KiAmdK6InitializeMTRR() {
    // FIXME: Support this
    // DPRINT("AMD MTRR support detected but not yet taken advantage of\n");
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ke/i386/mtrr.c
// * PURPOSE:         Support for MTRR and AMD K6 MTRR
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeMTRR(IN BOOLEAN FinalCpu)
// {
//     /* FIXME: Support this */
//     DPRINT("MTRR support detected but not yet taken advantage of\n");
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiAmdK6InitializeMTRR(VOID)
// {
//     /* FIXME: Support this */
//     DPRINT("AMD MTRR support detected but not yet taken advantage of\n");
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

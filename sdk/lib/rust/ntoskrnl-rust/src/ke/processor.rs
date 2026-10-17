use super::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::KAFFINITY;

#[no_mangle]
pub static mut KeActiveProcessors: KAFFINITY = 0;

#[no_mangle]
pub static mut KeNumberProcessors: CCHAR = 0;

#[no_mangle]
pub static mut KeMaximumProcessors: ULONG = 32;

#[no_mangle]
pub static mut KeNumprocSpecified: ULONG = 0;

#[no_mangle]
pub static mut KeBootprocSpecified: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn KeQueryActiveProcessors() -> KAFFINITY {
    KeActiveProcessors
}

/// The most processors the system can have. With no processor hot-add, that is the count
/// started at boot. Exported at every version because KMDF's driver tracker probes for it
/// (or the Ex form) and, finding neither, falls back to a version test that accepts only
/// XP and Vista (`fxbugcheckcallback.cpp`), which fails with `ASSERT(FALSE)` at 10.0.
#[no_mangle]
pub unsafe extern "system" fn KeQueryMaximumProcessorCount() -> ULONG {
    KeNumberProcessors as ULONG
}

/// One processor group here: group 0 or ALL_PROCESSOR_GROUPS (0xFFFF) is every processor,
/// any other group has none.
#[no_mangle]
pub unsafe extern "system" fn KeQueryMaximumProcessorCountEx(GroupNumber: USHORT) -> ULONG {
    if GroupNumber == 0 || GroupNumber == 0xFFFF {
        KeNumberProcessors as ULONG
    } else {
        0
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Portable processor related routines
//  * COPYRIGHT:   Copyright 2025 Timo Kreuzer <timo.kreuzer@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// KAFFINITY KeActiveProcessors = 0;
//
// /* Number of processors */
// CCHAR KeNumberProcessors = 0;
//
// #ifdef CONFIG_SMP
//
// /* Theoretical maximum number of processors that can be handled.
//  * Set once at run-time. Returned by KeQueryMaximumProcessorCount(). */
// ULONG KeMaximumProcessors = MAXIMUM_PROCESSORS;
//
// /* Maximum number of logical processors that can be started
//  * (including dynamically) at run-time. If 0: do not perform checks. */
// ULONG KeNumprocSpecified = 0;
//
// /* Maximum number of logical processors that can be started
//  * at boot-time. If 0: do not perform checks. */
// ULONG KeBootprocSpecified = 0;
//
// #endif // CONFIG_SMP
//
// /* FUNCTIONS *****************************************************************/
//
// KAFFINITY
// NTAPI
// KeQueryActiveProcessors(VOID)
// {
//     return KeActiveProcessors;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

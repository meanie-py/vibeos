use crate::ke::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    KAFFINITY,
    PKIPI_BROADCAST_WORKER,
};

const IPI_APC: ULONG = 1;
const IPI_DPC: ULONG = 2;
const IPI_FREEZE: ULONG = 0x4;

extern "system" {
    fn HalSendSoftwareInterrupt(TargetSet: KAFFINITY, Irql: KIRQL);
    fn HalSendNMI(TargetSet: KAFFINITY);
    fn DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "C" fn KiIpiSend(
    TargetSet: KAFFINITY,
    IpiRequest: ULONG,
) {
    if IpiRequest == IPI_APC {
        HalSendSoftwareInterrupt(TargetSet, APC_LEVEL);
    } else if IpiRequest == IPI_DPC {
        HalSendSoftwareInterrupt(TargetSet, DISPATCH_LEVEL);
    } else if IpiRequest == IPI_FREEZE {
        HalSendNMI(TargetSet);
    } else {
        core::debug_assert!(false);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeIpiGenericCall(
    Function: PKIPI_BROADCAST_WORKER,
    Argument: ULONG_PTR,
) -> ULONG_PTR {
    DbgBreakPoint();
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     IPI code for x64
//  * COPYRIGHT:   Copyright 2023 Timo Kreuzer <timo.kreuzer@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// FASTCALL
// KiIpiSend(
//     _In_ KAFFINITY TargetSet,
//     _In_ ULONG IpiRequest)
// {
//     /* Check if we can send the IPI directly */
//     if (IpiRequest == IPI_APC)
//     {
//         HalSendSoftwareInterrupt(TargetSet, APC_LEVEL);
//     }
//     else if (IpiRequest == IPI_DPC)
//     {
//         HalSendSoftwareInterrupt(TargetSet, DISPATCH_LEVEL);
//     }
//     else if (IpiRequest == IPI_FREEZE)
//     {
//         /* On x64 the freeze IPI is an NMI */
//         HalSendNMI(TargetSet);
//     }
//     else
//     {
//         ASSERT(FALSE);
//     }
// }
//
// ULONG_PTR
// NTAPI
// KeIpiGenericCall(
//     _In_ PKIPI_BROADCAST_WORKER Function,
//     _In_ ULONG_PTR Argument)
// {
//     __debugbreak();
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

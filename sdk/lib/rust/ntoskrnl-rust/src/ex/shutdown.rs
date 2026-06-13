use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ob::*;

use ntkrnl_sys::ddk::{
    POWER_ACTION,
    POWER_ACTION_PowerActionShutdown,
    POWER_ACTION_PowerActionShutdownReset,
    POWER_ACTION_PowerActionShutdownOff,
    _SYSTEM_POWER_STATE_PowerSystemSleeping3,
    NtSetSystemPowerState,
    PEPROCESS,
};

const PowerActionShutdown: POWER_ACTION = POWER_ACTION_PowerActionShutdown;
const PowerActionShutdownReset: POWER_ACTION = POWER_ACTION_PowerActionShutdownReset;
const PowerActionShutdownOff: POWER_ACTION = POWER_ACTION_PowerActionShutdownOff;
const PowerSystemSleeping3: u32 = _SYSTEM_POWER_STATE_PowerSystemSleeping3;

extern "system" {
    static mut ExpDefaultErrorPort: PVOID;
    static mut ExpDefaultErrorPortProcess: PEPROCESS;
}

const POWER_ACTION_OVERRIDE_APPS: ULONG = 0x00000004;
const POWER_ACTION_DISABLE_WAKES: ULONG = 0x40000000;
const POWER_ACTION_CRITICAL: ULONG = 0x80000000;

const ShutdownNoReboot: i32 = 0;
const ShutdownReboot: i32 = 1;
const ShutdownPowerOff: i32 = 2;

#[no_mangle]
pub unsafe extern "system" fn ExShutdownSystem() {
    if !ExpDefaultErrorPort.is_null() {
        ObDereferenceObject(ExpDefaultErrorPort);
        ExpDefaultErrorPort = core::ptr::null_mut();
    }
    if !ExpDefaultErrorPortProcess.is_null() {
        ObDereferenceObject(ExpDefaultErrorPortProcess as PVOID);
        ExpDefaultErrorPortProcess = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtShutdownSystem(Action: i32) -> NTSTATUS {
    let PowerAction: POWER_ACTION;

    if Action == ShutdownNoReboot {
        PowerAction = PowerActionShutdown;
    } else if Action == ShutdownReboot {
        PowerAction = PowerActionShutdownReset;
    } else if Action == ShutdownPowerOff {
        PowerAction = PowerActionShutdownOff;
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    NtSetSystemPowerState(
        PowerAction,
        PowerSystemSleeping3,
        POWER_ACTION_OVERRIDE_APPS | POWER_ACTION_DISABLE_WAKES | POWER_ACTION_CRITICAL,
    )
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/ex/shutdown.c
//  * PURPOSE:         Power management
//  *
//  * PROGRAMMERS:     David Welch (welch@cwcom.net)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// ExShutdownSystem(VOID)
// {
//     /* Dereference the hard-error port and process objects */
//     if (ExpDefaultErrorPort)
//     {
//         ObDereferenceObject(ExpDefaultErrorPort);
//         ExpDefaultErrorPort = NULL;
//     }
//     if (ExpDefaultErrorPortProcess)
//     {
//         ObDereferenceObject(ExpDefaultErrorPortProcess);
//         ExpDefaultErrorPortProcess = NULL;
//     }
// }
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtShutdownSystem(IN SHUTDOWN_ACTION Action)
// {
//     POWER_ACTION PowerAction;
//
//     /* Convert to power action */
//     if (Action == ShutdownNoReboot)
//     {
//         PowerAction = PowerActionShutdown;
//     }
//     else if (Action == ShutdownReboot)
//     {
//         PowerAction = PowerActionShutdownReset;
//     }
//     else if (Action == ShutdownPowerOff)
//     {
//         PowerAction = PowerActionShutdownOff;
//     }
//     else
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Now call the power manager */
//     DPRINT("Setting state to: %lx\n", PowerAction);
//     return NtSetSystemPowerState(PowerAction,
//                                  PowerSystemSleeping3,
//                                  POWER_ACTION_OVERRIDE_APPS |
//                                  POWER_ACTION_DISABLE_WAKES |
//                                  POWER_ACTION_CRITICAL);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use ntdll_sys::nt::*;
use crate::prelude::*;
use crate::substrate::{TRUE, FALSE};
use crate::k32_externs::RtlZeroMemory;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn GetSystemPowerStatus(
    PowerStatus: LPSYSTEM_POWER_STATUS,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut BattState: SYSTEM_BATTERY_STATE = core::mem::zeroed();
    let mut Max: ULONG;
    let mut Current: ULONG;

    Status = NtPowerInformation(
        SystemBatteryState,
        core::ptr::null_mut(),
        0,
        &mut BattState as *mut SYSTEM_BATTERY_STATE as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_BATTERY_STATE>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    RtlZeroMemory(
        PowerStatus as *mut core::ffi::c_void,
        core::mem::size_of::<SYSTEM_POWER_STATUS>() as SIZE_T,
    );

    (*PowerStatus).BatteryLifeTime = BATTERY_LIFE_UNKNOWN as DWORD;
    (*PowerStatus).BatteryFullLifeTime = BATTERY_LIFE_UNKNOWN as DWORD;
    (*PowerStatus).BatteryLifePercent = BATTERY_PERCENTAGE_UNKNOWN as BYTE;
    (*PowerStatus).ACLineStatus = AC_LINE_ONLINE as BYTE;

    Max = BattState.MaxCapacity;
    Current = BattState.RemainingCapacity;
    if Max != 0 {
        if Current <= Max {
            (*PowerStatus).BatteryLifePercent = ((100u32)
                .wrapping_mul(Current)
                .wrapping_add(Max.wrapping_div(2))
                .wrapping_div(Max)) as BYTE;
        } else {
            (*PowerStatus).BatteryLifePercent = 100 as BYTE;
        }

        if ((*PowerStatus).BatteryLifePercent as i32) <= 4 {
            (*PowerStatus).BatteryFlag =
                ((*PowerStatus).BatteryFlag as i32 | BATTERY_FLAG_CRITICAL as i32) as BYTE;
        }

        if ((*PowerStatus).BatteryLifePercent as i32) <= 32 {
            (*PowerStatus).BatteryFlag =
                ((*PowerStatus).BatteryFlag as i32 | BATTERY_FLAG_LOW as i32) as BYTE;
        }

        if ((*PowerStatus).BatteryLifePercent as i32) >= 67 {
            (*PowerStatus).BatteryFlag =
                ((*PowerStatus).BatteryFlag as i32 | BATTERY_FLAG_HIGH as i32) as BYTE;
        }
    }

    if BattState.BatteryPresent == 0 {
        (*PowerStatus).BatteryFlag =
            ((*PowerStatus).BatteryFlag as i32 | BATTERY_FLAG_NO_BATTERY as i32) as BYTE;
    }

    if BattState.Charging != 0 {
        (*PowerStatus).BatteryFlag =
            ((*PowerStatus).BatteryFlag as i32 | BATTERY_FLAG_CHARGING as i32) as BYTE;
    }

    if BattState.AcOnLine == 0 && BattState.BatteryPresent != 0 {
        (*PowerStatus).ACLineStatus = AC_LINE_OFFLINE as BYTE;
    }

    if BattState.EstimatedTime != 0 {
        (*PowerStatus).BatteryLifeTime = BattState.EstimatedTime;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetSystemPowerState(
    fSuspend: BOOL,
    fForce: BOOL,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtInitiatePowerAction(
        if fSuspend != FALSE {
            PowerActionSleep
        } else {
            PowerActionHibernate
        },
        if fSuspend != FALSE {
            PowerSystemSleeping1
        } else {
            PowerSystemHibernate
        },
        if fForce == FALSE { 1u32 } else { 0u32 },
        0u8,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0i32;
    }
    1i32
}

#[no_mangle]
pub unsafe extern "system" fn GetDevicePowerState(
    hDevice: HANDLE,
    pfOn: *mut BOOL,
) -> BOOL {
    let mut DevicePowerState: DEVICE_POWER_STATE = PowerDeviceUnspecified;
    let mut Status: NTSTATUS = 0;

    Status = NtGetDevicePowerState(hDevice, &mut DevicePowerState as *mut DEVICE_POWER_STATE);
    if NT_SUCCESS(Status) {
        *pfOn = (DevicePowerState == PowerDeviceUnspecified
            || DevicePowerState == PowerDeviceD0) as i32;
        return TRUE;
    }

    BaseSetLastNTError(Status);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn RequestDeviceWakeup(hDevice: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtRequestDeviceWakeup(hDevice);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn RequestWakeupLatency(latency: LATENCY_TIME) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtRequestWakeupLatency(latency);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn CancelDeviceWakeupRequest(hDevice: HANDLE) -> BOOL {
    let Status: NTSTATUS = NtCancelDeviceWakeupRequest(hDevice);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn IsSystemResumeAutomatic() -> BOOL {
    return NtIsSystemResumeAutomatic() as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetMessageWaitingIndicator(
    hMsgIndicator: HANDLE,
    ulMsgCount: ULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadExecutionState(mut esFlags: EXECUTION_STATE) -> EXECUTION_STATE {
    let mut Status: NTSTATUS = 0;
    Status = NtSetThreadExecutionState(esFlags, &raw mut esFlags);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return esFlags;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/power.c
//  * PURPOSE:         Power Management Functions
//  * PROGRAMMER:      Dmitry Chapyshev <dmitry@reactos.org>
//  *
//  * UPDATE HISTORY:
//  *                  01/15/2009 Created
//  */
//
// #include <k32.h>
//
// #include <ndk/pofuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetSystemPowerStatus(IN LPSYSTEM_POWER_STATUS PowerStatus)
// {
//     NTSTATUS Status;
//     SYSTEM_BATTERY_STATE BattState;
//     ULONG Max, Current;
//
//     Status = NtPowerInformation(SystemBatteryState,
//                                 NULL,
//                                 0,
//                                 &BattState,
//                                 sizeof(SYSTEM_BATTERY_STATE));
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     RtlZeroMemory(PowerStatus, sizeof(SYSTEM_POWER_STATUS));
//
//     PowerStatus->BatteryLifeTime = BATTERY_LIFE_UNKNOWN;
//     PowerStatus->BatteryFullLifeTime = BATTERY_LIFE_UNKNOWN;
//     PowerStatus->BatteryLifePercent = BATTERY_PERCENTAGE_UNKNOWN;
//     PowerStatus->ACLineStatus = AC_LINE_ONLINE;
//
//     Max = BattState.MaxCapacity;
//     Current = BattState.RemainingCapacity;
//     if (Max)
//     {
//         if (Current <= Max)
//         {
//             PowerStatus->BatteryLifePercent = (UCHAR)((100 * Current + Max / 2) / Max);
//         }
//         else
//         {
//             PowerStatus->BatteryLifePercent = 100;
//         }
//
//         if (PowerStatus->BatteryLifePercent <= 4)
//             PowerStatus->BatteryFlag |= BATTERY_FLAG_CRITICAL;
//
//         if (PowerStatus->BatteryLifePercent <= 32)
//             PowerStatus->BatteryFlag |= BATTERY_FLAG_LOW;
//
//         if (PowerStatus->BatteryLifePercent >= 67)
//             PowerStatus->BatteryFlag |= BATTERY_FLAG_HIGH;
//     }
//
//     if (!BattState.BatteryPresent)
//         PowerStatus->BatteryFlag |= BATTERY_FLAG_NO_BATTERY;
//
//     if (BattState.Charging)
//         PowerStatus->BatteryFlag |= BATTERY_FLAG_CHARGING;
//
//     if (!(BattState.AcOnLine) && (BattState.BatteryPresent))
//         PowerStatus->ACLineStatus = AC_LINE_OFFLINE;
//
//     if (BattState.EstimatedTime)
//         PowerStatus->BatteryLifeTime = BattState.EstimatedTime;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetSystemPowerState(IN BOOL fSuspend,
//                     IN BOOL fForce)
// {
//     NTSTATUS Status;
//
//     Status = NtInitiatePowerAction((fSuspend != FALSE) ? PowerActionSleep     : PowerActionHibernate,
//                                    (fSuspend != FALSE) ? PowerSystemSleeping1 : PowerSystemHibernate,
//                                    (fForce == FALSE),
//                                    FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetDevicePowerState(IN HANDLE hDevice,
//                     OUT BOOL *pfOn)
// {
//     DEVICE_POWER_STATE DevicePowerState;
//     NTSTATUS Status;
//
//     Status = NtGetDevicePowerState(hDevice, &DevicePowerState);
//     if (NT_SUCCESS(Status))
//     {
//         *pfOn = (DevicePowerState == PowerDeviceUnspecified) ||
//                 (DevicePowerState == PowerDeviceD0);
//         return TRUE;
//     }
//
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// RequestDeviceWakeup(IN HANDLE hDevice)
// {
//     NTSTATUS Status;
//
//     Status = NtRequestDeviceWakeup(hDevice);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// RequestWakeupLatency(IN LATENCY_TIME latency)
// {
//     NTSTATUS Status;
//
//     Status = NtRequestWakeupLatency(latency);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CancelDeviceWakeupRequest(IN HANDLE hDevice)
// {
//     NTSTATUS Status;
//
//     Status = NtCancelDeviceWakeupRequest(hDevice);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsSystemResumeAutomatic(VOID)
// {
//     return (BOOL)NtIsSystemResumeAutomatic();
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetMessageWaitingIndicator(IN HANDLE hMsgIndicator,
//                            IN ULONG ulMsgCount)
// {
//     /* This is the correct Windows implementation */
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// EXECUTION_STATE
// WINAPI
// SetThreadExecutionState(EXECUTION_STATE esFlags)
// {
//     NTSTATUS Status;
//
//     Status = NtSetThreadExecutionState(esFlags, &esFlags);
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return 0;
//     }
//
//     return esFlags;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

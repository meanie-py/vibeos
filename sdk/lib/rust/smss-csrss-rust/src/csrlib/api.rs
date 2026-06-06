use ntdll_sys::nt::*;
#[no_mangle]
pub unsafe extern "system" fn CsrNewThread() -> NTSTATUS {
    NtRegisterThreadTerminatePort(CsrApiPort)
}

#[no_mangle]
pub unsafe extern "system" fn CsrIdentifyAlertableThread() -> NTSTATUS {
    #[cfg(not(feature = "ntddi_ws03"))]
    {
        let mut ApiMessage: CSR_API_MESSAGE = core::mem::zeroed();
        let IdentifyAlertableThread = &mut ApiMessage
            .__bindgen_anon_1
            .__bindgen_anon_1
            .Data
            .IdentifyAlertableThread;

        IdentifyAlertableThread.Cid = (*NtCurrentTeb()).ClientId;

        let Status = CsrClientCallServer(
            &mut ApiMessage,
            core::ptr::null_mut(),
            ((CSRSRV_SERVERDLL_INDEX as u32) << 16) | (CsrpIdentifyAlertableThread as u32),
            core::mem::size_of_val(IdentifyAlertableThread) as u32,
        );

        Status
    }

    #[cfg(feature = "ntddi_ws03")]
    {
        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn CsrSetPriorityClass(
    Process: HANDLE,
    PriorityClass: *mut ULONG,
) -> NTSTATUS {
    #[cfg(not(feature = "ntddi_ws03"))]
    {
        let mut ApiMessage: CSR_API_MESSAGE = core::mem::zeroed();
        let SetPriorityClass = &mut ApiMessage
            .__bindgen_anon_1
            .__bindgen_anon_1
            .Data
            .SetPriorityClass;

        SetPriorityClass.hProcess = Process;
        SetPriorityClass.PriorityClass = *PriorityClass;

        let Status = CsrClientCallServer(
            &mut ApiMessage,
            core::ptr::null_mut(),
            ((CSRSRV_SERVERDLL_INDEX as u32) << 16) | (CsrpSetPriorityClass as u32),
            core::mem::size_of_val(SetPriorityClass) as u32,
        );

        if *PriorityClass != 0 {
            *PriorityClass = SetPriorityClass.PriorityClass;
        }

        Status
    }

    #[cfg(feature = "ntddi_ws03")]
    {
        let _ = Process;
        let _ = PriorityClass;
        0xC000000Du32 as NTSTATUS // STATUS_INVALID_PARAMETER
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Client/Server Runtime SubSystem
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     CSR Client Library - API LPC Implementation
//  * COPYRIGHT:   Copyright 2005-2012 Alex Ionescu <alex@relsoft.net>
//  *              Copyright 2012-2022 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "csrlib.h"
//
// #define NTOS_MODE_USER
// #include <ndk/psfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CsrNewThread(VOID)
// {
//     /* Register the termination port to CSR's */
//     return NtRegisterThreadTerminatePort(CsrApiPort);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CsrIdentifyAlertableThread(VOID)
// {
// #if (NTDDI_VERSION < NTDDI_WS03)
//     NTSTATUS Status;
//     CSR_API_MESSAGE ApiMessage;
//     PCSR_IDENTIFY_ALERTABLE_THREAD IdentifyAlertableThread;
//
//     /* Set up the data for CSR */
//     IdentifyAlertableThread = &ApiMessage.Data.IdentifyAlertableThread;
//     IdentifyAlertableThread->Cid = NtCurrentTeb()->ClientId;
//
//     /* Call it */
//     Status = CsrClientCallServer(&ApiMessage,
//                                  NULL,
//                                  CSR_CREATE_API_NUMBER(CSRSRV_SERVERDLL_INDEX, CsrpIdentifyAlertableThread),
//                                  sizeof(*IdentifyAlertableThread));
//
//     /* Return to caller */
//     return Status;
// #else
//     /* Deprecated */
//     return STATUS_SUCCESS;
// #endif
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CsrSetPriorityClass(
//     _In_ HANDLE Process,
//     _Inout_ PULONG PriorityClass)
// {
// #if (NTDDI_VERSION < NTDDI_WS03)
//     NTSTATUS Status;
//     CSR_API_MESSAGE ApiMessage;
//     PCSR_SET_PRIORITY_CLASS SetPriorityClass = &ApiMessage.Data.SetPriorityClass;
//
//     /* Set up the data for CSR */
//     SetPriorityClass->hProcess = Process;
//     SetPriorityClass->PriorityClass = *PriorityClass;
//
//     /* Call it */
//     Status = CsrClientCallServer(&ApiMessage,
//                                  NULL,
//                                  CSR_CREATE_API_NUMBER(CSRSRV_SERVERDLL_INDEX, CsrpSetPriorityClass),
//                                  sizeof(*SetPriorityClass));
//
//     /* Return what we got, if requested */
//     if (*PriorityClass) *PriorityClass = SetPriorityClass->PriorityClass;
//
//     /* Return to caller */
//     return Status;
// #else
//     UNREFERENCED_PARAMETER(Process);
//     UNREFERENCED_PARAMETER(PriorityClass);
//
//     /* Deprecated */
//     return STATUS_INVALID_PARAMETER;
// #endif
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

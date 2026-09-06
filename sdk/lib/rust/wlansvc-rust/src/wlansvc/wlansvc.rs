use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static SERVICE_NAME: [u16; 13] = [
    0x57, 0x4C, 0x41, 0x4E, 0x20,
    0x53, 0x65, 0x72, 0x76, 0x69,
    0x63, 0x65, 0x0
];

pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
pub static mut SvcStatus: SERVICE_STATUS = unsafe { core::mem::zeroed() };

pub static ServiceName: [u16; 8] = [
    0x57, 0x6C, 0x61, 0x6E,
    0x53, 0x76, 0x63, 0x0
];
unsafe extern "C" fn UpdateServiceStatus(hServiceStatus: HANDLE, NewStatus: DWORD, Increment: DWORD) {
    let svc_ptr = core::ptr::addr_of_mut!(SvcStatus);
    if Increment > 0 {
        (*svc_ptr).dwCheckPoint += Increment;
    } else {
        (*svc_ptr).dwCheckPoint = 0;
    }
    (*svc_ptr).dwCurrentState = NewStatus;
    SetServiceStatus(hServiceStatus as SERVICE_STATUS_HANDLE, svc_ptr);
}
// The C is `static DWORD WINAPI ServiceControlHandler(...)`. WINAPI == stdcall on i386 and
// cdecl elsewhere, which is exactly what `extern "system"` resolves to per arch — so the one
// definition replaces the previous cfg-split pair (identical bodies) and lets the fn item be
// passed to RegisterServiceCtrlHandlerExW as `Some(..)` with no transmute.
unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_SHUTDOWN | SERVICE_CONTROL_STOP => {
            UpdateServiceStatus(ServiceStatusHandle as HANDLE, SERVICE_STOP_PENDING, 1);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(ServiceStatusHandle as HANDLE, SERVICE_STOPPED, 0);
            NO_ERROR
        }
        SERVICE_CONTROL_INTERROGATE => NO_ERROR,
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}

// C: `static VOID __stdcall ServiceMain(DWORD argc, LPWSTR *argv)` — same collapse as above.
unsafe extern "system" fn ServiceMain(argc: DWORD, argv: *mut LPWSTR) {
    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    // DPRINT removed

    let svc_status = core::ptr::addr_of_mut!(SvcStatus);
    (*svc_status).dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    (*svc_status).dwCurrentState = SERVICE_START_PENDING;
    (*svc_status).dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    (*svc_status).dwCheckPoint = 0;
    (*svc_status).dwWin32ExitCode = NO_ERROR;
    (*svc_status).dwServiceSpecificExitCode = 0;
    (*svc_status).dwWaitHint = 4000;

    let service_name_ptr = core::ptr::addr_of!(ServiceName) as *const WCHAR;
    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        service_name_ptr,
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );

    UpdateServiceStatus(ServiceStatusHandle as HANDLE, SERVICE_RUNNING, 0);

    // C casts RpcThreadRoutine to LPTHREAD_START_ROUTINE; both are WINAPI (LPVOID)->DWORD, so
    // the cast was a no-op and the fn item is simply wrapped here.
    let h_thread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcThreadRoutine),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );

    if h_thread.is_null() {
        // DPRINT removed
        UpdateServiceStatus(ServiceStatusHandle as HANDLE, SERVICE_STOPPED, 0);
    } else {
        CloseHandle(h_thread);
    }

    // DPRINT removed
}
#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut u16) -> i32 {
    let service_table: [SERVICE_TABLE_ENTRYW; 2] = [
        SERVICE_TABLE_ENTRYW {
            // C passes the (non-const) `static WCHAR ServiceName[]` array; SCM only reads it.
            lpServiceName: ServiceName.as_ptr() as LPWSTR,
            lpServiceProc: Some(ServiceMain),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];

    // UNREFERENCED_PARAMETER(argc);
    // UNREFERENCED_PARAMETER(argv);
    // DPRINT("wlansvc: main() started\n");

    StartServiceCtrlDispatcherW(service_table.as_ptr());

    // DPRINT("wlansvc: main() done\n");

    ExitThread(0);

    0
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             base/services/wlansvc/wlansvc.c
//  * PURPOSE:          WLAN Service
//  * PROGRAMMER:       Christoph von Wittich
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ******************************************************************/
//
// #define SERVICE_NAME L"WLAN Service"
//
// SERVICE_STATUS_HANDLE ServiceStatusHandle;
// SERVICE_STATUS SvcStatus;
// static WCHAR ServiceName[] = L"WlanSvc";
//
// DWORD WINAPI RpcThreadRoutine(LPVOID lpParameter);
//
// /* FUNCTIONS *****************************************************************/
//
// static void UpdateServiceStatus(HANDLE hServiceStatus, DWORD NewStatus, DWORD Increment)
// {
//     if (Increment > 0)
//         SvcStatus.dwCheckPoint += Increment;
//     else
//         SvcStatus.dwCheckPoint = 0;
//
//     SvcStatus.dwCurrentState = NewStatus;
//     SetServiceStatus(hServiceStatus, &SvcStatus);
// }
//
// static DWORD WINAPI
// ServiceControlHandler(DWORD dwControl,
//                       DWORD dwEventType,
//                       LPVOID lpEventData,
//                       LPVOID lpContext)
// {
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_SHUTDOWN:
//         case SERVICE_CONTROL_STOP:
//             UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOP_PENDING, 1);
//             RpcMgmtStopServerListening(NULL);
//             UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOPPED, 0);
//             break;
//         case SERVICE_CONTROL_INTERROGATE:
//             return NO_ERROR;
//         default:
//             return ERROR_CALL_NOT_IMPLEMENTED;
//     }
//     return NO_ERROR;
// }
//
// static VOID __stdcall
// ServiceMain(DWORD argc, LPWSTR *argv)
// {
//     HANDLE hThread;
//
//     UNREFERENCED_PARAMETER(argc);
//     UNREFERENCED_PARAMETER(argv);
//
//     DPRINT("ServiceMain() called\n");
//
//     SvcStatus.dwServiceType             = SERVICE_WIN32_OWN_PROCESS;
//     SvcStatus.dwCurrentState            = SERVICE_START_PENDING;
//     SvcStatus.dwControlsAccepted        = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
//     SvcStatus.dwCheckPoint              = 0;
//     SvcStatus.dwWin32ExitCode           = NO_ERROR;
//     SvcStatus.dwServiceSpecificExitCode = 0;
//     SvcStatus.dwWaitHint                = 4000;
//
//     ServiceStatusHandle = RegisterServiceCtrlHandlerExW(ServiceName,
//                                                         ServiceControlHandler,
//                                                         NULL);
//
//     UpdateServiceStatus(ServiceStatusHandle, SERVICE_RUNNING, 0);
//
//     hThread = CreateThread(NULL,
//                            0,
//                            (LPTHREAD_START_ROUTINE)
//                            RpcThreadRoutine,
//                            NULL,
//                            0,
//                            NULL);
//
//     if (!hThread)
//     {
//         DPRINT("Can't create RpcThread\n");
//         UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOPPED, 0);
//     }
//     else
//     {
//         CloseHandle(hThread);
//     }
//
//     DPRINT("ServiceMain() done\n");
// }
//
// int
// wmain(int argc, WCHAR *argv[])
// {
//     SERVICE_TABLE_ENTRYW ServiceTable[2] =
//     {
//         {ServiceName, ServiceMain},
//         {NULL, NULL}
//     };
//
//     UNREFERENCED_PARAMETER(argc);
//     UNREFERENCED_PARAMETER(argv);
//
//     DPRINT("wlansvc: main() started\n");
//
//     StartServiceCtrlDispatcherW(ServiceTable);
//
//     DPRINT("wlansvc: main() done\n");
//
//     ExitThread(0);
//
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hDllInstance: HINSTANCE = core::ptr::null_mut();
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
pub static mut SvcStatus: SERVICE_STATUS = SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};
pub static ServiceName: [u16; 9] = [
    'D' as u16,
    'n' as u16,
    's' as u16,
    'c' as u16,
    'a' as u16,
    'c' as u16,
    'h' as u16,
    'e' as u16,
    0,
];
unsafe extern "system" {
    fn RpcThreadRoutine(lpParameter: LPVOID) -> DWORD;
}
unsafe extern "C" fn UpdateServiceStatus(
    hServiceStatus: SERVICE_STATUS_HANDLE,
    NewStatus: u32,
    Increment: u32,
) {
    if Increment > 0 {
        (*core::ptr::addr_of_mut!(SvcStatus)).dwCheckPoint += Increment;
    } else {
        (*core::ptr::addr_of_mut!(SvcStatus)).dwCheckPoint = 0;
    }
    (*core::ptr::addr_of_mut!(SvcStatus)).dwCurrentState = NewStatus;
    SetServiceStatus(hServiceStatus, core::ptr::addr_of_mut!(SvcStatus));
}
unsafe extern "system" fn ServiceControlHandler(
    dwControl: u32,
    dwEventType: u32,
    lpEventData: *mut core::ffi::c_void,
    lpContext: *mut core::ffi::c_void,
) -> u32 {
    match dwControl {
        SERVICE_CONTROL_SHUTDOWN | SERVICE_CONTROL_STOP => {
            UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOP_PENDING, 1);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            DnsIntCacheFree();
            UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOPPED, 0);
            NO_ERROR
        }
        SERVICE_CONTROL_INTERROGATE => NO_ERROR,
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}


#[no_mangle]
pub unsafe extern "system" fn ServiceMain(
    argc: u32,
    argv: *mut *mut u16,
) {
    let _ = (argc, argv);

    SvcStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    SvcStatus.dwCurrentState = SERVICE_START_PENDING;
    SvcStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    SvcStatus.dwCheckPoint = 0;
    SvcStatus.dwWin32ExitCode = NO_ERROR;
    SvcStatus.dwServiceSpecificExitCode = 0;
    SvcStatus.dwWaitHint = 4000;

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of!(ServiceName) as *const u16,
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );

    if ServiceStatusHandle.is_null() {
        return;
    }

    DnsIntCacheInitialize();
    ReadHostsFile();

    let hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcThreadRoutine as unsafe extern "system" fn(*mut core::ffi::c_void) -> u32),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );

    if hThread.is_null() {
        DnsIntCacheFree();
        UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOPPED, 0);
    } else {
        CloseHandle(hThread);
    }

    UpdateServiceStatus(ServiceStatusHandle, SERVICE_RUNNING, 0);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: u32,
    pvReserved: *mut core::ffi::c_void,
) -> i32 {
    let _ = pvReserved;
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL);
            hDllInstance = hinstDLL;
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    1i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS DNS Resolver
//  * FILE:        base/services/dnsrslvr/dnsrslvr.c
//  * PURPOSE:     DNS Resolver Service
//  * PROGRAMMER:  Christoph von Wittich
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
// HINSTANCE hDllInstance;
// SERVICE_STATUS_HANDLE ServiceStatusHandle;
// SERVICE_STATUS SvcStatus;
// static WCHAR ServiceName[] = L"Dnscache";
//
// DWORD WINAPI RpcThreadRoutine(LPVOID lpParameter);
//
// /* FUNCTIONS *****************************************************************/
//
// static
// VOID
// UpdateServiceStatus(
//     HANDLE hServiceStatus,
//     DWORD NewStatus,
//     DWORD Increment)
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
// static
// DWORD
// WINAPI
// ServiceControlHandler(
//     DWORD dwControl,
//     DWORD dwEventType,
//     LPVOID lpEventData,
//     LPVOID lpContext)
// {
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_SHUTDOWN:
//         case SERVICE_CONTROL_STOP:
//             UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOP_PENDING, 1);
//             RpcMgmtStopServerListening(NULL);
//             DnsIntCacheFree();
//             UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOPPED, 0);
//             break;
//
//         case SERVICE_CONTROL_INTERROGATE:
//             return NO_ERROR;
//
//         default:
//             return ERROR_CALL_NOT_IMPLEMENTED;
//     }
//     return NO_ERROR;
// }
//
// VOID
// WINAPI
// ServiceMain(
//     DWORD argc,
//     LPWSTR *argv)
// {
//     HANDLE hThread;
//
//     let _ = argc;
//     let _ = argv;
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
//     if (!ServiceStatusHandle)
//     {
//         DPRINT1("DNSRSLVR: Unable to register service control handler (%lx)\n", GetLastError());
//         return; // FALSE
//     }
//
//     DnsIntCacheInitialize();
//
//     ReadHostsFile();
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
//         DnsIntCacheFree();
//         DPRINT("Can't create RpcThread\n");
//         UpdateServiceStatus(ServiceStatusHandle, SERVICE_STOPPED, 0);
//     }
//     else
//     {
//         CloseHandle(hThread);
//     }
//
//     DPRINT("ServiceMain() done\n");
//     UpdateServiceStatus(ServiceStatusHandle, SERVICE_RUNNING, 0);
// }
//
// BOOL
// WINAPI
// DllMain(
//      HINSTANCE hinstDLL,
//      DWORD fdwReason,
//      PVOID pvReserved)
// {
//     let _ = pvReserved;
//
//     switch (fdwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             DisableThreadLibraryCalls(hinstDLL);
//             hDllInstance = hinstDLL;
//             break;
//
//         case DLL_PROCESS_DETACH:
//             break;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

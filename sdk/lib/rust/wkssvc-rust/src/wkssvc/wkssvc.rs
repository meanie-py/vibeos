use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut ServiceName: [u16; 18] = [
    0x006c, 0x0061, 0x006e, 0x006d, 0x0061, 0x006e, 0x0077,
    0x006f, 0x0072, 0x006b, 0x0073, 0x0074, 0x0061, 0x0074,
    0x0069, 0x006f, 0x006e, 0x0000,
];
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
pub static mut ServiceStatus: SERVICE_STATUS = SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};
pub static mut VersionInfo: OSVERSIONINFOW = OSVERSIONINFOW {
    dwOSVersionInfoSize: 0,
    dwMajorVersion: 0,
    dwMinorVersion: 0,
    dwBuildNumber: 0,
    dwPlatformId: 0,
    szCSDVersion: [0; 128],
};
pub static mut LsaHandle: HANDLE = core::ptr::null_mut();
pub static mut LsaAuthenticationPackage: ULONG = 0;
unsafe extern "C" fn UpdateServiceStatus(dwState: u32) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS;
    ServiceStatus.dwCurrentState = dwState;
    ServiceStatus.dwControlsAccepted = 0;
    ServiceStatus.dwWin32ExitCode = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCheckPoint = 0;

    if dwState == SERVICE_START_PENDING ||
        dwState == SERVICE_STOP_PENDING ||
        dwState == SERVICE_PAUSE_PENDING ||
        dwState == SERVICE_CONTINUE_PENDING {
        ServiceStatus.dwWaitHint = 10000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    if dwState == SERVICE_RUNNING {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    }

    SetServiceStatus(ServiceStatusHandle, &raw mut ServiceStatus);
}
unsafe extern "C" fn ServiceInit() -> u32 {
    let mut process_name = STRING {
        Length: 11,
        MaximumLength: 12,
        Buffer: b"Workstation\0".as_ptr() as *mut i8,
    };

    let msv1 = b"MICROSOFT_AUTHENTICATION_PACKAGE_V1_0\0";
    let mut package_name = STRING {
        Length: (msv1.len() - 1) as u16,
        MaximumLength: msv1.len() as u16,
        Buffer: msv1.as_ptr() as *mut i8,
    };

    let mut mode: LSA_OPERATIONAL_MODE = core::mem::zeroed();
    let mut h_thread: HANDLE;

    VersionInfo.dwOSVersionInfoSize = (core::mem::size_of::<OSVERSIONINFOW>()) as u32;
    GetVersionExW(&mut VersionInfo);

    InitWorkstationInfo();

    if !NT_SUCCESS(LsaRegisterLogonProcess(core::ptr::addr_of_mut!(process_name) as PLSA_STRING, &mut LsaHandle, &mut mode)) {
        return 1;
    }

    if !NT_SUCCESS(LsaLookupAuthenticationPackage(LsaHandle, core::ptr::addr_of_mut!(package_name) as PLSA_STRING, &mut LsaAuthenticationPackage)) {
        return 1;
    }

    h_thread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcThreadRoutine),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );

    if h_thread.is_null() {
        return GetLastError();
    } else {
        CloseHandle(h_thread);
    }

    SetServiceBits(ServiceStatusHandle, SV_TYPE_WORKSTATION, 1, 1);

    return 0;
}
unsafe extern "C" fn ServiceShutdown() {
    let Status = LsaDeregisterLogonProcess(LsaHandle);
    if !NT_SUCCESS(Status) {
        return;
    }
}
unsafe extern "system" fn ServiceControlHandler(
    dwControl: u32,
    dwEventType: u32,
    lpEventData: *mut core::ffi::c_void,
    lpContext: *mut core::ffi::c_void,
) -> u32 {
    match dwControl {
        SERVICE_CONTROL_STOP => {
            TRACE!("  SERVICE_CONTROL_STOP received\n");
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            ServiceShutdown();
            UpdateServiceStatus(SERVICE_STOPPED);
            ERROR_SUCCESS
        }
        SERVICE_CONTROL_PAUSE => {
            TRACE!("  SERVICE_CONTROL_PAUSE received\n");
            UpdateServiceStatus(SERVICE_PAUSED);
            ERROR_SUCCESS
        }
        SERVICE_CONTROL_CONTINUE => {
            TRACE!("  SERVICE_CONTROL_CONTINUE received\n");
            UpdateServiceStatus(SERVICE_RUNNING);
            ERROR_SUCCESS
        }
        SERVICE_CONTROL_INTERROGATE => {
            TRACE!("  SERVICE_CONTROL_INTERROGATE received\n");
            SetServiceStatus(
                ServiceStatusHandle,
                core::ptr::addr_of!(ServiceStatus) as *mut _,
            );
            ERROR_SUCCESS
        }
        SERVICE_CONTROL_SHUTDOWN => {
            TRACE!("  SERVICE_CONTROL_SHUTDOWN received\n");
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            ServiceShutdown();
            UpdateServiceStatus(SERVICE_STOPPED);
            ERROR_SUCCESS
        }
        _ => {
            TRACE!("  Control {} received\n", dwControl);
            ERROR_CALL_NOT_IMPLEMENTED
        }
    }
}


#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: u32, argv: *mut *mut u16) {
    let _ = argc;
    let _ = argv;

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        ServiceName.as_ptr(),
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );

    if ServiceStatusHandle.is_null() {
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    let dwError = ServiceInit();
    if dwError != ERROR_SUCCESS {
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    UpdateServiceStatus(SERVICE_RUNNING);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: *mut core::ffi::c_void,
    fdwReason: u32,
    _lpvReserved: *mut core::ffi::c_void,
) -> i32 {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
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
//  *  ReactOS Services
//  *  Copyright (C) 2015 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Services
//  * FILE:             base/services/wkssvc/wkssvc.c
//  * PURPOSE:          Workstation service
//  * PROGRAMMER:       Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(wkssvc);
//
// /* GLOBALS ******************************************************************/
//
// static WCHAR ServiceName[] = L"lanmanworkstation";
//
// static SERVICE_STATUS_HANDLE ServiceStatusHandle;
// static SERVICE_STATUS ServiceStatus;
//
// OSVERSIONINFOW VersionInfo;
// HANDLE LsaHandle = NULL;
// ULONG LsaAuthenticationPackage = 0;
//
// /* FUNCTIONS *****************************************************************/
//
// static VOID
// UpdateServiceStatus(DWORD dwState)
// {
//     ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS;
//     ServiceStatus.dwCurrentState = dwState;
//     ServiceStatus.dwControlsAccepted = 0;
//     ServiceStatus.dwWin32ExitCode = 0;
//     ServiceStatus.dwServiceSpecificExitCode = 0;
//     ServiceStatus.dwCheckPoint = 0;
//
//     if (dwState == SERVICE_START_PENDING ||
//         dwState == SERVICE_STOP_PENDING ||
//         dwState == SERVICE_PAUSE_PENDING ||
//         dwState == SERVICE_CONTINUE_PENDING)
//         ServiceStatus.dwWaitHint = 10000;
//     else
//         ServiceStatus.dwWaitHint = 0;
//
//     if (dwState == SERVICE_RUNNING)
//         ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
//
//     SetServiceStatus(ServiceStatusHandle,
//                      &ServiceStatus);
// }
//
//
// static
// DWORD
// ServiceInit(VOID)
// {
//     LSA_STRING ProcessName = RTL_CONSTANT_STRING("Workstation");
//     LSA_STRING PackageName = RTL_CONSTANT_STRING(MSV1_0_PACKAGE_NAME);
//     LSA_OPERATIONAL_MODE Mode;
//     HANDLE hThread;
//     NTSTATUS Status;
//
//     ERR("ServiceInit()\n");
//
//     /* Get the OS version */
//     VersionInfo.dwOSVersionInfoSize = sizeof(VersionInfo);
//     GetVersionExW(&VersionInfo);
//
//     InitWorkstationInfo();
//
//     Status = LsaRegisterLogonProcess(&ProcessName,
//                                      &LsaHandle,
//                                      &Mode);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsaRegisterLogonProcess() failed! (Status 0x%08lx)\n", Status);
//         return 1;
//     }
//
//     Status = LsaLookupAuthenticationPackage(LsaHandle,
//                                             &PackageName,
//                                             &LsaAuthenticationPackage);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsaLookupAuthenticationPackage() failed! (Status 0x%08lx)\n", Status);
//         return 1;
//     }
//
//     hThread = CreateThread(NULL,
//                            0,
//                            (LPTHREAD_START_ROUTINE)RpcThreadRoutine,
//                            NULL,
//                            0,
//                            NULL);
//
//     if (!hThread)
//     {
//         ERR("Can't create PortThread\n");
//         return GetLastError();
//     }
//     else
//         CloseHandle(hThread);
//
//     /* Report a running workstation service */
//     SetServiceBits(ServiceStatusHandle,
//                    SV_TYPE_WORKSTATION,
//                    TRUE,
//                    TRUE);
//
//     return ERROR_SUCCESS;
// }
//
//
// static
// VOID
// ServiceShutdown(VOID)
// {
//     NTSTATUS Status;
//
//     ERR("ServiceShutdown()\n");
//
//     Status = LsaDeregisterLogonProcess(LsaHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("LsaDeRegisterLogonProcess() failed! (Status 0x%08lx)\n", Status);
//         return;
//     }
// }
//
//
// static DWORD WINAPI
// ServiceControlHandler(DWORD dwControl,
//                       DWORD dwEventType,
//                       LPVOID lpEventData,
//                       LPVOID lpContext)
// {
//     TRACE("ServiceControlHandler() called\n");
//
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_STOP:
//             TRACE("  SERVICE_CONTROL_STOP received\n");
//             UpdateServiceStatus(SERVICE_STOP_PENDING);
//             /* Stop listening to incoming RPC messages */
//             RpcMgmtStopServerListening(NULL);
//             ServiceShutdown();
//             UpdateServiceStatus(SERVICE_STOPPED);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_PAUSE:
//             TRACE("  SERVICE_CONTROL_PAUSE received\n");
//             UpdateServiceStatus(SERVICE_PAUSED);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_CONTINUE:
//             TRACE("  SERVICE_CONTROL_CONTINUE received\n");
//             UpdateServiceStatus(SERVICE_RUNNING);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_INTERROGATE:
//             TRACE("  SERVICE_CONTROL_INTERROGATE received\n");
//             SetServiceStatus(ServiceStatusHandle,
//                              &ServiceStatus);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_SHUTDOWN:
//             TRACE("  SERVICE_CONTROL_SHUTDOWN received\n");
//             UpdateServiceStatus(SERVICE_STOP_PENDING);
//             /* Stop listening to incoming RPC messages */
//             RpcMgmtStopServerListening(NULL);
//             ServiceShutdown();
//             UpdateServiceStatus(SERVICE_STOPPED);
//             return ERROR_SUCCESS;
//
//         default :
//             TRACE("  Control %lu received\n", dwControl);
//             return ERROR_CALL_NOT_IMPLEMENTED;
//     }
// }
//
//
// VOID WINAPI
// ServiceMain(DWORD argc, LPTSTR *argv)
// {
//     DWORD dwError;
//
//     let _ = argc;
//     let _ = argv;
//
//     TRACE("ServiceMain() called\n");
//
//     ServiceStatusHandle = RegisterServiceCtrlHandlerExW(ServiceName,
//                                                         ServiceControlHandler,
//                                                         NULL);
//     if (!ServiceStatusHandle)
//     {
//         ERR("RegisterServiceCtrlHandlerExW() failed! (Error %lu)\n", GetLastError());
//         return;
//     }
//
//     UpdateServiceStatus(SERVICE_START_PENDING);
//
//     dwError = ServiceInit();
//     if (dwError != ERROR_SUCCESS)
//     {
//         ERR("Service stopped (dwError: %lu\n", dwError);
//         UpdateServiceStatus(SERVICE_STOPPED);
//         return;
//     }
//
//     UpdateServiceStatus(SERVICE_RUNNING);
// }
//
//
// BOOL WINAPI
// DllMain(HINSTANCE hinstDLL,
//         DWORD fdwReason,
//         LPVOID lpvReserved)
// {
//     switch (fdwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             DisableThreadLibraryCalls(hinstDLL as HMODULE);
//             break;
//
//         case DLL_PROCESS_DETACH:
//             break;
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

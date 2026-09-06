use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hDllInstance: HINSTANCE = core::ptr::null_mut();
pub static mut lpServiceGlobals: PSVCHOST_GLOBAL_DATA = core::ptr::null_mut();
pub static ServiceName: [u16; 8] = [0x62, 0x72, 0x6f, 0x77, 0x73, 0x65, 0x72, 0x00];
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
pub static mut ServiceStatus: SERVICE_STATUS = SERVICE_STATUS::zeroed();
unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    ServiceStatus.dwCurrentState = dwState;

    if dwState == SERVICE_RUNNING {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_SHUTDOWN | SERVICE_ACCEPT_STOP;
    } else {
        ServiceStatus.dwControlsAccepted = 0;
    }

    ServiceStatus.dwWin32ExitCode = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCheckPoint = 0;

    if dwState == SERVICE_START_PENDING
        || dwState == SERVICE_STOP_PENDING
        || dwState == SERVICE_PAUSE_PENDING
        || dwState == SERVICE_CONTINUE_PENDING
    {
        ServiceStatus.dwWaitHint = 10000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
}
// C: `static DWORD WINAPI ServiceControlHandler(...)`. WINAPI is __stdcall on i386 and cdecl
// elsewhere, which is precisely what `extern "system"` means — so the one definition covers all
// three targets. (The translation had emitted two cfg-split copies with byte-identical bodies.)
unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP => {
            unsafe {
                TRACE!(
                    b"  SERVICE_CONTROL_STOP received\n\0".as_ptr() as *const i8,
                );
            }
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED);
            ERROR_SUCCESS
        }

        SERVICE_CONTROL_PAUSE => {
            unsafe {
                TRACE!(
                    b"  SERVICE_CONTROL_PAUSE received\n\0".as_ptr() as *const i8,
                );
            }
            UpdateServiceStatus(SERVICE_PAUSED);
            ERROR_SUCCESS
        }

        SERVICE_CONTROL_CONTINUE => {
            unsafe {
                TRACE!(
                    b"  SERVICE_CONTROL_CONTINUE received\n\0".as_ptr() as *const i8,
                );
            }
            UpdateServiceStatus(SERVICE_RUNNING);
            ERROR_SUCCESS
        }

        SERVICE_CONTROL_INTERROGATE => {
            unsafe {
                TRACE!(
                    b"  SERVICE_CONTROL_INTERROGATE received\n\0".as_ptr()
                        as *const i8,
                );
            }
            SetServiceStatus(
                ServiceStatusHandle,
                core::ptr::addr_of_mut!(ServiceStatus),
            );
            ERROR_SUCCESS
        }

        SERVICE_CONTROL_SHUTDOWN => {
            unsafe {
                TRACE!(
                    b"  SERVICE_CONTROL_SHUTDOWN received\n\0".as_ptr() as *const i8,
                );
            }
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED);
            ERROR_SUCCESS
        }

        _ => {
            unsafe {
                TRACE!(
                    b"  Control %lu received\n\0".as_ptr() as *const i8,
                    dwControl as usize,
                );
            }
            ERROR_CALL_NOT_IMPLEMENTED
        }
    }
}

unsafe extern "C" fn ServiceInit() -> u32 {
    // C cast `(LPTHREAD_START_ROUTINE)RpcThreadRoutine` is a no-op there — the signatures already
    // match — so no transmute is needed here either; CreateThread's prelude declaration spells the
    // start routine as the same `extern "system" fn(*mut c_void) -> DWORD` RpcThreadRoutine is.
    let hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcThreadRoutine),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if hThread.is_null() {
        return GetLastError();
    } else {
        CloseHandle(hThread);
    }
    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ServiceMain(ArgCount: DWORD, ArgVector: *mut LPWSTR) {
    let _ = (ArgCount, ArgVector);

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        ServiceName.as_ptr(),
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );

    if ServiceStatusHandle.is_null() {
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    let dwError: DWORD = ServiceInit();
    if dwError != ERROR_SUCCESS {
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    UpdateServiceStatus(SERVICE_RUNNING);
}

#[no_mangle]
pub unsafe extern "system" fn SvchostPushServiceGlobals(
    lpGlobals: PSVCHOST_GLOBAL_DATA,
) {
    lpServiceGlobals = lpGlobals;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: u32,
    pvReserved: LPVOID,
) -> i32 {
    UNREFERENCED_PARAMETER!(pvReserved);
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL);
            hDllInstance = hinstDLL;
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    1
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Browser Service
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Browser service RPC server
//  * COPYRIGHT:   Eric Kohl 2020 <eric.kohl@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(browser);
//
//
// /* GLOBALS ******************************************************************/
//
// HINSTANCE hDllInstance;
// PSVCHOST_GLOBAL_DATA lpServiceGlobals;
//
// static WCHAR ServiceName[] = L"browser";
//
// static SERVICE_STATUS_HANDLE ServiceStatusHandle;
// static SERVICE_STATUS ServiceStatus;
//
//
// /* FUNCTIONS *****************************************************************/
//
// static
// VOID
// UpdateServiceStatus(
//     DWORD dwState)
// {
//     ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
//     ServiceStatus.dwCurrentState = dwState;
//
//     if (dwState == SERVICE_RUNNING)
//         ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_SHUTDOWN | SERVICE_ACCEPT_STOP;
//     else
//         ServiceStatus.dwControlsAccepted = 0;
//
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
//     SetServiceStatus(ServiceStatusHandle,
//                      &ServiceStatus);
// }
//
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
//     TRACE("ServiceControlHandler()\n");
//
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_STOP:
//             TRACE("  SERVICE_CONTROL_STOP received\n");
//             /* Stop listening to incoming RPC messages */
//             RpcMgmtStopServerListening(NULL);
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
//             /* Stop listening to incoming RPC messages */
//             RpcMgmtStopServerListening(NULL);
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
// static
// DWORD
// ServiceInit(VOID)
// {
//     HANDLE hThread;
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
//     return ERROR_SUCCESS;
// }
//
//
// VOID
// WINAPI
// ServiceMain(
//      DWORD ArgCount,
//      LPWSTR *ArgVector)
// {
//     DWORD dwError;
//
//     UNREFERENCED_PARAMETER(ArgCount);
//     UNREFERENCED_PARAMETER(ArgVector);
//
//     FIXME("ServiceMain(%d %p)\n", ArgCount, ArgVector);
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
// VOID
// WINAPI
// SvchostPushServiceGlobals(
//      PSVCHOST_GLOBAL_DATA lpGlobals)
// {
//     TRACE("SvchostPushServiceGlobals(%p)\n", lpGlobals);
//     lpServiceGlobals = lpGlobals;
// }
//
//
// BOOL
// WINAPI
// DllMain(
//      HINSTANCE hinstDLL,
//      DWORD fdwReason,
//      PVOID pvReserved)
// {
//     UNREFERENCED_PARAMETER(pvReserved);
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

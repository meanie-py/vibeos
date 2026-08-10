// rustconv-assembly: whole-tu
use crate::prelude::*;
// cross-TU references: the C is one link unit, so a static in another .c is simply
// visible; in Rust each TU is a module and the reference has to be spelled.
use crate::umpnpmgr::event::PnpEventThread;
use crate::umpnpmgr::install::{DeviceInstallThread, SetupIsActive};
use crate::umpnpmgr::rpcserver::RpcServerThread;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut ServiceName: [u16; 9] = [
    0x0050, 0x006C, 0x0075, 0x0067, 0x0050, 0x006C, 0x0061, 0x0079, 0x0000,
];
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = 0 as SERVICE_STATUS_HANDLE;
pub static mut ServiceStatus: SERVICE_STATUS = SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};
// precomp.h declares every one of these `extern`: they are module globals with EXTERNAL
// linkage, shared across the four TUs. Without #[no_mangle] each definition is mangled
// while sibling modules reference the plain name, and the crate links against undefined
// symbols — the failure only appears at LINK, never in `cargo check`.
#[no_mangle]
pub static mut hEnumKey: HKEY = core::ptr::null_mut();
#[no_mangle]
pub static mut hClassKey: HKEY = core::ptr::null_mut();
#[no_mangle]
pub static mut g_IsUISuppressed: BOOL = 0;
#[no_mangle]
pub static mut g_ShuttingDown: BOOL = 0;
#[no_mangle]
pub static mut g_IsLiveMedium: BOOL = 0;
unsafe extern "C" fn UpdateServiceStatus(
    dwState: u32,
    dwCheckPoint: u32,
) {
    if (dwState == (SERVICE_STOPPED) as u32) || (dwState == (SERVICE_STOP_PENDING) as u32) {
        g_ShuttingDown = (TRUE) as i32;
    }

    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    ServiceStatus.dwCurrentState = dwState;
    ServiceStatus.dwWin32ExitCode = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCheckPoint = dwCheckPoint;

    if dwState == (SERVICE_RUNNING) as u32 {
        ServiceStatus.dwControlsAccepted = (SERVICE_ACCEPT_SHUTDOWN) as u32;
    } else {
        ServiceStatus.dwControlsAccepted = 0;
    }

    if dwState == (SERVICE_START_PENDING) as u32 ||
        dwState == (SERVICE_STOP_PENDING) as u32 ||
        dwState == (SERVICE_PAUSE_PENDING) as u32 ||
        dwState == (SERVICE_CONTINUE_PENDING) as u32 {
        ServiceStatus.dwWaitHint = 10000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    SetServiceStatus(ServiceStatusHandle,
                          &mut ServiceStatus);
}
unsafe extern "system" fn ServiceControlHandler(
    dwControl: u32,
    dwEventType: u32,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> u32 {
    // DPRINT removed

    match dwControl {
        SERVICE_CONTROL_STOP => {
            // DPRINT removed
            UpdateServiceStatus(SERVICE_STOP_PENDING, 1);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED, 0);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_PAUSE => {
            // DPRINT removed
            UpdateServiceStatus(SERVICE_PAUSED, 0);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_CONTINUE => {
            // DPRINT removed
            UpdateServiceStatus(SERVICE_RUNNING, 0);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_INTERROGATE => {
            // DPRINT removed
            SetServiceStatus(
                ServiceStatusHandle,
                core::ptr::addr_of_mut!(ServiceStatus),
            );
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_SHUTDOWN => {
            // DPRINT removed
            UpdateServiceStatus(SERVICE_STOP_PENDING, 1);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED, 0);
            return ERROR_SUCCESS;
        }

        _ => {
            // DPRINT removed
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}
unsafe extern "C" fn GetBooleanRegValue(
    hKey: HKEY,
    lpSubKey: PCWSTR,
    lpValue: PCWSTR,
    pValue: PBOOL,
) -> u32 {
    let mut dwError: u32;
    let mut dwType: u32 = 0;
    let mut dwData: u32 = 0;
    let mut cbData: u32 = core::mem::size_of::<u32>() as u32;
    let mut hSubKey: HKEY = core::ptr::null_mut();

    *pValue = (FALSE) as i32;

    dwError = (RegOpenKeyExW(
        hKey,
        lpSubKey,
        0,
        KEY_READ,
        &mut hSubKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return dwError;
    }

    dwError = (RegQueryValueExW(
        hSubKey,
        lpValue,
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwData as *mut u32 as PBYTE,
        &mut cbData,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        RegCloseKey(hSubKey);
        return dwError;
    }
    if dwType != (REG_DWORD) as u32 {
        // DPRINT removed
        RegCloseKey(hSubKey);
        return dwError;
    }

    *pValue = if dwData == 1 { 1 } else { 0 };

    RegCloseKey(hSubKey);

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn GetSuppressNewUIValue() -> BOOL {
    let mut bSuppressNewHWUI: BOOL = FALSE as BOOL;

    GetBooleanRegValue(
        HKEY_LOCAL_MACHINE,
        w!("Software\\Policies\\Microsoft\\Windows\\DeviceInstall\\Settings") as PCWSTR,
        w!("SuppressNewHWUI") as PCWSTR,
        &mut bSuppressNewHWUI,
    );
    if bSuppressNewHWUI != 0 {
        // DPRINT removed
    }

    bSuppressNewHWUI
}
#[no_mangle]
pub unsafe extern "C" fn RunningOnLiveMedium() -> BOOL {
    let mut Options: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut CurrentOption: PWSTR;
    let mut NextOption: PWSTR;
    let mut hControlKey: HKEY = core::ptr::null_mut();
    let mut dwType: u32 = 0;
    let mut dwSize: u32;
    let mut dwError: u32;
    let mut LiveMedium: BOOL = (FALSE) as i32;

    // DPRINT removed

    'done: {
        dwError = (RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("SYSTEM\\CurrentControlSet\\Control") as PCWSTR,
            0,
            KEY_QUERY_VALUE,
            &mut hControlKey,
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwSize = core::mem::size_of_val(&Options) as u32;
        dwError = (RegQueryValueExW(
            hControlKey,
            w!("SystemStartOptions") as PCWSTR,
            core::ptr::null_mut(),
            &mut dwType,
            Options.as_mut_ptr() as LPBYTE,
            &mut dwSize,
        )) as u32;
        if (dwError != ERROR_SUCCESS) || (dwType != (REG_SZ) as u32) {
            break 'done;
        }

        CurrentOption = Options.as_mut_ptr();
        while !CurrentOption.is_null() {
            NextOption = wcschr(CurrentOption, 'L' as u16);
            if !NextOption.is_null() {
                *NextOption = 0;
            }
            if _wcsicmp(CurrentOption, w!("MININT") as PCWSTR) == 0 {
                // DPRINT removed
                LiveMedium = (TRUE) as i32;
                break 'done;
            }
            CurrentOption = if !NextOption.is_null() {
                NextOption.offset(1)
            } else {
                core::ptr::null_mut()
            };
        }
    }

    RegCloseKey(hControlKey);

    LiveMedium
}
#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: u32, argv: *mut LPTSTR) {
    let mut hThread: *mut core::ffi::c_void;
    let mut dwThreadId: u32 = 0;

    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    // DPRINT removed

    g_IsLiveMedium = RunningOnLiveMedium();

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        ServiceName.as_ptr(),
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        // DPRINT removed
        return;
    }

    UpdateServiceStatus((SERVICE_START_PENDING) as u32, 1);

    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(PnpEventThread),
        core::ptr::null_mut(),
        0,
        &mut dwThreadId,
    );
    if hThread != core::ptr::null_mut() {
        CloseHandle(hThread);
    }

    UpdateServiceStatus((SERVICE_START_PENDING) as u32, 2);

    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcServerThread),
        core::ptr::null_mut(),
        0,
        &mut dwThreadId,
    );
    if hThread != core::ptr::null_mut() {
        CloseHandle(hThread);
    }

    UpdateServiceStatus((SERVICE_START_PENDING) as u32, 3);

    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(DeviceInstallThread),
        core::ptr::null_mut(),
        0,
        &mut dwThreadId,
    );
    if hThread != core::ptr::null_mut() {
        CloseHandle(hThread);
    }

    UpdateServiceStatus((SERVICE_RUNNING) as u32, 0);

    // DPRINT removed
}
unsafe extern "C" fn InitializePnPManager() -> u32 {
    let mut OldValue: u8 = 0;
    let mut dwError: u32;

    // DPRINT removed

    RtlAdjustPrivilege(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE, (TRUE) as u8, (FALSE) as u8, &mut OldValue);

    hInstallEvent = CreateEventW(core::ptr::null_mut(), (TRUE) as i32, (SetupIsActive()) as i32, core::ptr::null_mut());
    if hInstallEvent == core::ptr::null_mut() {
        dwError = GetLastError();
        // DPRINT removed
        return dwError;
    }

    hNoPendingInstalls = CreateEventW(core::ptr::null_mut(),
                                      (TRUE) as i32,
                                      (FALSE) as i32,
                                      w!("Global\\PnP_No_Pending_Install_Events") as PCWSTR);
    if hNoPendingInstalls == core::ptr::null_mut() {
        dwError = GetLastError();
        // DPRINT removed
        return dwError;
    }

    hDeviceInstallListNotEmpty = CreateEventW(core::ptr::null_mut(), (FALSE) as i32, (FALSE) as i32, core::ptr::null_mut());
    if hDeviceInstallListNotEmpty == core::ptr::null_mut() {
        dwError = GetLastError();
        // DPRINT removed
        return dwError;
    }

    hDeviceInstallListMutex = CreateMutexW(core::ptr::null_mut(), (FALSE) as i32, core::ptr::null_mut());
    if hDeviceInstallListMutex == core::ptr::null_mut() {
        dwError = GetLastError();
        // DPRINT removed
        return dwError;
    }
    InitializeListHead(&mut DeviceInstallListHead);

    GetBooleanRegValue(HKEY_LOCAL_MACHINE,
                       w!("System\\CurrentControlSet\\Services\\PlugPlay\\Parameters") as PCWSTR,
                       w!("SuppressUI") as PCWSTR,
                       &mut g_IsUISuppressed);
    if g_IsUISuppressed != 0 {
        // DPRINT removed
    }

    dwError = (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            w!("System\\CurrentControlSet\\Enum") as PCWSTR,
                            0,
                            KEY_ALL_ACCESS,
                            &mut hEnumKey)) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return dwError;
    }

    dwError = (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                            w!("System\\CurrentControlSet\\Control\\Class") as PCWSTR,
                            0,
                            KEY_ALL_ACCESS,
                            &mut hClassKey)) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return dwError;
    }

    // DPRINT removed

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: u32,
    lpvReserved: LPVOID,
) -> i32 {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL);
            InitializePnPManager();
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    (TRUE) as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2005 ReactOS Team
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
//  * PROJECT:          ReactOS kernel
//  * FILE:             base/services/umpnpmgr/umpnpmgr.c
//  * PURPOSE:          User-mode Plug and Play manager
//  * PROGRAMMER:       Eric Kohl (eric.kohl@reactos.org)
//  *                   Hervé Poussineau (hpoussin@reactos.org)
//  *                   Colin Finck (colin@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ******************************************************************/
//
// static WCHAR ServiceName[] = L"PlugPlay";
//
// static SERVICE_STATUS_HANDLE ServiceStatusHandle;
// static SERVICE_STATUS ServiceStatus;
//
// HKEY hEnumKey = NULL;
// HKEY hClassKey = NULL;
// BOOL g_IsUISuppressed = FALSE;
// BOOL g_ShuttingDown = FALSE;
// BOOL g_IsLiveMedium = FALSE;
//
// /* FUNCTIONS *****************************************************************/
//
// static VOID
// UpdateServiceStatus(
//      DWORD dwState,
//      DWORD dwCheckPoint)
// {
//     if ((dwState == SERVICE_STOPPED) || (dwState == SERVICE_STOP_PENDING))
//         g_ShuttingDown = TRUE;
//
//     ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
//     ServiceStatus.dwCurrentState = dwState;
//     ServiceStatus.dwWin32ExitCode = 0;
//     ServiceStatus.dwServiceSpecificExitCode = 0;
//     ServiceStatus.dwCheckPoint = dwCheckPoint;
//
//     if (dwState == SERVICE_RUNNING)
//         ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_SHUTDOWN;
//     else
//         ServiceStatus.dwControlsAccepted = 0;
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
// static DWORD WINAPI
// ServiceControlHandler(DWORD dwControl,
//                       DWORD dwEventType,
//                       LPVOID lpEventData,
//                       LPVOID lpContext)
// {
//     DPRINT1("ServiceControlHandler() called\n");
//
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_STOP:
//             DPRINT1("  SERVICE_CONTROL_STOP received\n");
//             UpdateServiceStatus(SERVICE_STOP_PENDING, 1);
//             /* Stop listening to RPC Messages */
//             RpcMgmtStopServerListening(NULL);
//             UpdateServiceStatus(SERVICE_STOPPED, 0);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_PAUSE:
//             DPRINT1("  SERVICE_CONTROL_PAUSE received\n");
//             UpdateServiceStatus(SERVICE_PAUSED, 0);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_CONTINUE:
//             DPRINT1("  SERVICE_CONTROL_CONTINUE received\n");
//             UpdateServiceStatus(SERVICE_RUNNING, 0);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_INTERROGATE:
//             DPRINT1("  SERVICE_CONTROL_INTERROGATE received\n");
//             SetServiceStatus(ServiceStatusHandle,
//                              &ServiceStatus);
//             return ERROR_SUCCESS;
//
//         case SERVICE_CONTROL_SHUTDOWN:
//             DPRINT1("  SERVICE_CONTROL_SHUTDOWN received\n");
//             UpdateServiceStatus(SERVICE_STOP_PENDING, 1);
//             /* Stop listening to RPC Messages */
//             RpcMgmtStopServerListening(NULL);
//             UpdateServiceStatus(SERVICE_STOPPED, 0);
//             return ERROR_SUCCESS;
//
//         default :
//             DPRINT1("  Control %lu received\n", dwControl);
//             return ERROR_CALL_NOT_IMPLEMENTED;
//     }
// }
//
// static DWORD
// GetBooleanRegValue(
//      HKEY hKey,
//      PCWSTR lpSubKey,
//      PCWSTR lpValue,
//      PBOOL pValue)
// {
//     DWORD dwError, dwType, dwData;
//     DWORD cbData = sizeof(dwData);
//     HKEY hSubKey = NULL;
//
//     /* Default value */
//     *pValue = FALSE;
//
//     dwError = RegOpenKeyExW(hKey,
//                             lpSubKey,
//                             0,
//                             KEY_READ,
//                             &hSubKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("GetBooleanRegValue(): RegOpenKeyExW() has failed to open '%S' key! (Error: %lu)\n",
//                lpSubKey, dwError);
//         return dwError;
//     }
//
//     dwError = RegQueryValueExW(hSubKey,
//                                lpValue,
//                                0,
//                                &dwType,
//                                (PBYTE)&dwData,
//                                &cbData);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("GetBooleanRegValue(): RegQueryValueExW() has failed to query '%S' value! (Error: %lu)\n",
//                lpValue, dwError);
//         goto Cleanup;
//     }
//     if (dwType != REG_DWORD)
//     {
//         DPRINT("GetBooleanRegValue(): The value is not of REG_DWORD type!\n");
//         goto Cleanup;
//     }
//
//     /* Return the value */
//     *pValue = (dwData == 1);
//
// Cleanup:
//     RegCloseKey(hSubKey);
//
//     return dwError;
// }
//
// BOOL
// GetSuppressNewUIValue(VOID)
// {
//     BOOL bSuppressNewHWUI = FALSE;
//
//     /*
//      * Query the SuppressNewHWUI policy registry value. Don't cache it
//      * as we want to update our behaviour in consequence.
//      */
//     GetBooleanRegValue(HKEY_LOCAL_MACHINE,
//                        L"Software\\Policies\\Microsoft\\Windows\\DeviceInstall\\Settings",
//                        L"SuppressNewHWUI",
//                        &bSuppressNewHWUI);
//     if (bSuppressNewHWUI)
//         DPRINT("GetSuppressNewUIValue(): newdev.dll's wizard UI won't be shown!\n");
//
//     return bSuppressNewHWUI;
// }
//
// BOOL
// RunningOnLiveMedium(VOID)
// {
//     WCHAR Options[MAX_PATH];
//     PWSTR CurrentOption, NextOption;
//     HKEY hControlKey;
//     DWORD dwType;
//     DWORD dwSize;
//     DWORD dwError;
//     BOOL LiveMedium = FALSE;
//
//     DPRINT("RunningOnLiveMedium()\n");
//
//     /* Open the Setup key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Control",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hControlKey);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Read the CmdLine value */
//     dwSize = sizeof(Options);
//     dwError = RegQueryValueExW(hControlKey,
//                                L"SystemStartOptions",
//                                NULL,
//                                &dwType,
//                                (LPBYTE)Options,
//                                &dwSize);
//     if ((dwError != ERROR_SUCCESS) || (dwType != REG_SZ))
//         goto done;
//
//     /* Check for the '-mini' option */
//     CurrentOption = Options;
//     while (CurrentOption)
//     {
//         NextOption = wcschr(CurrentOption, L' ');
//         if (NextOption)
//             *NextOption = L'\0';
//         if (_wcsicmp(CurrentOption, L"MININT") == 0)
//         {
//             DPRINT("Found 'MININT' boot option\n");
//             LiveMedium = TRUE;
//             goto done;
//         }
//         CurrentOption = NextOption ? NextOption + 1 : NULL;
//     }
//
// done:
//     RegCloseKey(hControlKey);
//
//     return LiveMedium;
// }
//
// VOID WINAPI
// ServiceMain(DWORD argc, LPTSTR *argv)
// {
//     HANDLE hThread;
//     DWORD dwThreadId;
//
//     UNREFERENCED_PARAMETER!(argc);
//     UNREFERENCED_PARAMETER!(argv);
//
//     DPRINT("ServiceMain() called\n");
//
//     g_IsLiveMedium = RunningOnLiveMedium();
//
//     ServiceStatusHandle = RegisterServiceCtrlHandlerExW(ServiceName,
//                                                         Some(ServiceControlHandler),
//                                                         NULL);
//     if (!ServiceStatusHandle)
//     {
//         DPRINT1("RegisterServiceCtrlHandlerExW() failed! (Error %lu)\n", GetLastError());
//         return;
//     }
//
//     UpdateServiceStatus(SERVICE_START_PENDING, 1);
//
//     hThread = CreateThread(NULL,
//                            0,
//                            PnpEventThread,
//                            NULL,
//                            0,
//                            &dwThreadId);
//     if (hThread != NULL)
//         CloseHandle(hThread);
//
//     UpdateServiceStatus(SERVICE_START_PENDING, 2);
//
//     hThread = CreateThread(NULL,
//                            0,
//                            RpcServerThread,
//                            NULL,
//                            0,
//                            &dwThreadId);
//     if (hThread != NULL)
//         CloseHandle(hThread);
//
//     UpdateServiceStatus(SERVICE_START_PENDING, 3);
//
//     hThread = CreateThread(NULL,
//                            0,
//                            DeviceInstallThread,
//                            NULL,
//                            0,
//                            &dwThreadId);
//     if (hThread != NULL)
//         CloseHandle(hThread);
//
//     UpdateServiceStatus(SERVICE_RUNNING, 0);
//
//     DPRINT("ServiceMain() done\n");
// }
//
// static DWORD
// InitializePnPManager(VOID)
// {
//     BOOLEAN OldValue;
//     DWORD dwError;
//
//     DPRINT("UMPNPMGR: InitializePnPManager() started\n");
//
//     /* We need this privilege for using CreateProcessAsUserW */
//     RtlAdjustPrivilege(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE, TRUE, FALSE, &OldValue);
//
//     hInstallEvent = CreateEventW(NULL, TRUE, SetupIsActive()/*FALSE*/, NULL);
//     if (hInstallEvent == NULL)
//     {
//         dwError = GetLastError();
//         DPRINT1("Could not create the Install Event! (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     hNoPendingInstalls = CreateEventW(NULL,
//                                       TRUE,
//                                       FALSE,
//                                       L"Global\\PnP_No_Pending_Install_Events");
//     if (hNoPendingInstalls == NULL)
//     {
//         dwError = GetLastError();
//         DPRINT1("Could not create the Pending-Install Event! (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     /*
//      * Initialize the device-install event list
//      */
//
//     hDeviceInstallListNotEmpty = CreateEventW(NULL, FALSE, FALSE, NULL);
//     if (hDeviceInstallListNotEmpty == NULL)
//     {
//         dwError = GetLastError();
//         DPRINT1("Could not create the List Event! (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     hDeviceInstallListMutex = CreateMutexW(NULL, FALSE, NULL);
//     if (hDeviceInstallListMutex == NULL)
//     {
//         dwError = GetLastError();
//         DPRINT1("Could not create the List Mutex! (Error %lu)\n", dwError);
//         return dwError;
//     }
//     InitializeListHead(&DeviceInstallListHead);
//
//     /* Query the SuppressUI registry value and cache it for our whole lifetime */
//     GetBooleanRegValue(HKEY_LOCAL_MACHINE,
//                        L"System\\CurrentControlSet\\Services\\PlugPlay\\Parameters",
//                        L"SuppressUI",
//                        &g_IsUISuppressed);
//     if (g_IsUISuppressed)
//         DPRINT("UMPNPMGR: newdev.dll's wizard UI won't be shown!\n");
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Enum",
//                             0,
//                             KEY_ALL_ACCESS,
//                             &hEnumKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Could not open the Enum Key! (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Control\\Class",
//                             0,
//                             KEY_ALL_ACCESS,
//                             &hClassKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Could not open the Class Key! (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     DPRINT("UMPNPMGR: InitializePnPManager() done\n");
//
//     return 0;
// }
//
// BOOL WINAPI
// DllMain(HINSTANCE hinstDLL,
//         DWORD fdwReason,
//         LPVOID lpvReserved)
// {
//     switch (fdwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             DisableThreadLibraryCalls(hinstDLL);
//             InitializePnPManager();
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

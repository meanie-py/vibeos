use winlogon_sys::nt::*;
use crate::preamble::*;
#[no_mangle]
pub unsafe extern "C" fn GetSetupType() -> DWORD {
    let mut dwError: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwSetupType: DWORD = 0;

    let key_name: [u16; 13] = [
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16,
        b'\\' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'u' as u16, b'p' as u16, 0,
    ];

    let value_name: [u16; 10] = [
        b'S' as u16, b'e' as u16, b't' as u16, b'u' as u16, b'p' as u16,
        b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, 0,
    ];

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        key_name.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return 0;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hKey,
        value_name.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwSetupType as *mut DWORD as LPBYTE,
        &mut dwSize,
    ) as DWORD;

    RegCloseKey(hKey);
    if dwError != ERROR_SUCCESS
        || dwType != REG_DWORD
        || dwSize != core::mem::size_of::<DWORD>() as DWORD
    {
        return 0;
    }

    dwSetupType
}
// LPTHREAD_START_ROUTINE: the C declared this WINAPI. bindgen emits the
// callback type as `extern "stdcall"` on i386 and `extern "C"` on
// amd64/arm64 — and Rust treats `extern "system"` as a DISTINCT fn-ptr
// type from both even when the ABI is identical. So match bindgen exactly
// per arch with thin wrappers over a single internal-ABI body.
unsafe extern "system" fn RunSetupThreadProc(lpParameter: LPVOID) -> u32 {
    RunSetupThreadProcImpl(lpParameter)
}

unsafe fn RunSetupThreadProcImpl(lpParameter: LPVOID) -> u32 {
    let _ = lpParameter;
    // Wide-string literals as aligned [u16; N] statics (NOT
    // `b"..".as_ptr() as *const u16`, which is align-1 and faults on
    // amd64 WCHAR reads — feedback_byte_literal_wchar_misalign).
    static SYSTEM_SETUP: [u16; 13] = [
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16,
        b'\\' as u16, b'S' as u16, b'e' as u16, b't' as u16, b'u' as u16, b'p' as u16, 0,
    ];
    static CMDLINE: [u16; 8] = [
        b'C' as u16, b'm' as u16, b'd' as u16, b'L' as u16, b'i' as u16, b'n' as u16,
        b'e' as u16, 0,
    ];
    static WINSTA0_DEFAULT: [u16; 16] = [
        b'W' as u16, b'i' as u16, b'n' as u16, b'S' as u16, b't' as u16, b'a' as u16,
        b'0' as u16, b'\\' as u16, b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16,
        b'u' as u16, b'l' as u16, b't' as u16, 0,
    ];

    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut Shell: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut CommandLine: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let Result: i32;
    let mut dwError: u32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: u32 = 0;
    let mut dwSize: u32;
    let mut dwExitCode: u32 = 0;

    /* Open key */
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        SYSTEM_SETUP.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) as u32;
    if dwError != ERROR_SUCCESS {
        return 0;
    }

    /* Read key */
    dwSize = core::mem::size_of_val(&Shell) as u32;
    dwError = RegQueryValueExW(
        hKey,
        CMDLINE.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        Shell.as_mut_ptr() as LPBYTE,
        &mut dwSize,
    ) as u32;
    RegCloseKey(hKey);
    if dwError != ERROR_SUCCESS {
        return 0;
    }

    /* Finish string */
    Shell[(dwSize as usize) / core::mem::size_of::<u16>()] = 0;

    /* Expand string (if applicable) */
    if dwType == REG_EXPAND_SZ {
        ExpandEnvironmentStringsW(Shell.as_ptr(), CommandLine.as_mut_ptr(), CommandLine.len() as u32);
    } else if dwType == REG_SZ {
        wcscpy(CommandLine.as_mut_ptr(), Shell.as_ptr());
    } else {
        return 0;
    }

    SwitchDesktop((*WLSession).ApplicationDesktop);

    /* Start process */
    StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    StartupInfo.lpReserved = core::ptr::null_mut();
    StartupInfo.lpDesktop = WINSTA0_DEFAULT.as_ptr() as *mut u16;
    StartupInfo.lpTitle = core::ptr::null_mut();
    StartupInfo.dwFlags = 0;
    StartupInfo.cbReserved2 = 0;
    StartupInfo.lpReserved2 = core::ptr::null_mut();

    Result = CreateProcessW(
        core::ptr::null(),
        CommandLine.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        DETACHED_PROCESS,
        core::ptr::null_mut(),
        core::ptr::null(),
        &mut StartupInfo,
        &mut ProcessInformation,
    );
    if Result == 0 {
        SwitchDesktop((*WLSession).WinlogonDesktop);
        return 0;
    }

    /* Wait for process termination */
    WaitForSingleObject(ProcessInformation.hProcess, INFINITE);

    GetExitCodeProcess(ProcessInformation.hProcess, &mut dwExitCode);

    /* Close handles */
    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    1
}
#[no_mangle]
pub unsafe extern "C" fn RunSetup() -> BOOL {
    let hThread: HANDLE = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RunSetupThreadProc),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if hThread != core::ptr::null_mut() {
        CloseHandle(hThread);
    }

    (hThread != core::ptr::null_mut()) as BOOL
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Winlogon
//  * FILE:            base/system/winlogon/setup.c
//  * PURPOSE:         Setup support functions
//  * PROGRAMMERS:     Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "winlogon.h"
//
// /* FUNCTIONS ****************************************************************/
//
// DWORD
// GetSetupType(VOID)
// {
//     DWORD dwError;
//     HKEY hKey;
//     DWORD dwType;
//     DWORD dwSize;
//     DWORD dwSetupType;
//
//     TRACE("GetSetupType()\n");
//
//     /* Open key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\Setup",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hKey);
//     if (dwError != ERROR_SUCCESS)
//         return 0;
//
//     /* Read key */
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hKey,
//                                L"SetupType",
//                                NULL,
//                                &dwType,
//                                (LPBYTE)&dwSetupType,
//                                &dwSize);
//
//     /* Close key, and check if returned values are correct */
//     RegCloseKey(hKey);
//     if (dwError != ERROR_SUCCESS || dwType != REG_DWORD || dwSize != sizeof(DWORD))
//         return 0;
//
//     TRACE("GetSetupType() returns %lu\n", dwSetupType);
//     return dwSetupType;
// }
//
//
// static
// DWORD
// WINAPI
// RunSetupThreadProc(
//     IN LPVOID lpParameter)
// {
//     PROCESS_INFORMATION ProcessInformation;
//     STARTUPINFOW StartupInfo;
//     WCHAR Shell[MAX_PATH];
//     WCHAR CommandLine[MAX_PATH];
//     BOOL Result;
//     DWORD dwError;
//     HKEY hKey;
//     DWORD dwType;
//     DWORD dwSize;
//     DWORD dwExitCode;
//
//     TRACE("RunSetup() called\n");
//
//     /* Open key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\Setup",
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hKey);
//     if (dwError != ERROR_SUCCESS)
//         return FALSE;
//
//     /* Read key */
//     dwSize = sizeof(Shell);
//     dwError = RegQueryValueExW(hKey,
//                                L"CmdLine",
//                                NULL,
//                                &dwType,
//                                (LPBYTE)Shell,
//                                &dwSize);
//     RegCloseKey(hKey);
//     if (dwError != ERROR_SUCCESS)
//         return FALSE;
//
//     /* Finish string */
//     Shell[dwSize / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Expand string (if applicable) */
//     if (dwType == REG_EXPAND_SZ)
//         ExpandEnvironmentStringsW(Shell, CommandLine, ARRAYSIZE(CommandLine));
//     else if (dwType == REG_SZ)
//         wcscpy(CommandLine, Shell);
//     else
//         return FALSE;
//
//     TRACE("Should run '%s' now\n", debugstr_w(CommandLine));
//
//     SwitchDesktop(WLSession->ApplicationDesktop);
//
//     /* Start process */
//     StartupInfo.cb = sizeof(StartupInfo);
//     StartupInfo.lpReserved = NULL;
//     StartupInfo.lpDesktop = L"WinSta0\\Default";
//     StartupInfo.lpTitle = NULL;
//     StartupInfo.dwFlags = 0;
//     StartupInfo.cbReserved2 = 0;
//     StartupInfo.lpReserved2 = 0;
//
//     Result = CreateProcessW(NULL,
//                             CommandLine,
//                             NULL,
//                             NULL,
//                             FALSE,
//                             DETACHED_PROCESS,
//                             NULL,
//                             NULL,
//                             &StartupInfo,
//                             &ProcessInformation);
//     if (!Result)
//     {
//         TRACE("Failed to run setup process\n");
//         SwitchDesktop(WLSession->WinlogonDesktop);
//         return FALSE;
//     }
//
//     /* Wait for process termination */
//     WaitForSingleObject(ProcessInformation.hProcess, INFINITE);
//
//     GetExitCodeProcess(ProcessInformation.hProcess, &dwExitCode);
//
//     /* Close handles */
//     CloseHandle(ProcessInformation.hThread);
//     CloseHandle(ProcessInformation.hProcess);
//
//     // SwitchDesktop(WLSession->WinlogonDesktop);
//
//     TRACE ("RunSetup() done\n");
//
//     return TRUE;
// }
//
//
// BOOL
// RunSetup(VOID)
// {
//     HANDLE hThread;
//
//     hThread = CreateThread(NULL,
//                            0,
//                            RunSetupThreadProc,
//                            NULL,
//                            0,
//                            NULL);
//     if (hThread != NULL)
//         CloseHandle(hThread);
//
//     return hThread != NULL;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

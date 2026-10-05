use crate::prelude::*;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

// Service name strings — `static` so they have a stable address for the table
// (L"NETLOGON", L"SAMSS"). UTF-16, NUL-terminated.
static NETLOGON_W: [u16; 9] = [0x004E, 0x0045, 0x0054, 0x004C, 0x004F, 0x0047, 0x004F, 0x004E, 0x0000];
static SAMSS_W: [u16; 6] = [0x0053, 0x0041, 0x004D, 0x0053, 0x0053, 0x0000];

pub static mut ServiceTable: [SERVICE_TABLE_ENTRYW; 3] = [
    _SERVICE_TABLE_ENTRYW {
        lpServiceName: NETLOGON_W.as_ptr() as LPWSTR,
        lpServiceProc: Some(NetlogonServiceMain),
    },
    _SERVICE_TABLE_ENTRYW {
        lpServiceName: SAMSS_W.as_ptr() as LPWSTR,
        lpServiceProc: Some(SamSsServiceMain),
    },
    _SERVICE_TABLE_ENTRYW {
        lpServiceName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        lpServiceProc: None,
    },
];

pub type PNETLOGONMAIN = Option<unsafe extern "system" fn(ArgCount: i32, ArgVector: *mut *mut u16)>;

// WINAPI (stdcall on i386) — matches LPSERVICE_MAIN_FUNCTIONW; a cdecl cast would
// drift ESP on i386 when the SCM dispatches through the table.
unsafe extern "system" fn NetlogonServiceMain(dwArgc: DWORD, pszArgv: *mut LPWSTR) {
    let mut hNetlogon: HINSTANCE = core::ptr::null_mut();
    let mut pNetlogonMain: PNETLOGONMAIN = None;

    // TRACE("NetlogonServiceMain(%lu %p)\n", dwArgc, pszArgv)
    if (*(__wine_dbch___default)).flags as u32 & (1u32 << __WINE_DBCL_TRACE as u32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                .as_ptr() as *const i8,
            b"NetlogonServiceMain\0".as_ptr() as *const i8,
            40,
            b"NetlogonServiceMain(%lu %p)\n\0".as_ptr() as *const i8,
            dwArgc,
            pszArgv,
        );
    }

    hNetlogon = LoadLibraryW(
        core::mem::transmute::<[u8; 26], [u16; 13]>(*b"N\0e\0t\0l\0o\0g\0o\0n\0.\0d\0l\0l\0\0\0")
            .as_ptr() as *const u16,
    );
    if hNetlogon.is_null() {
        // ERR("LoadLibrary() failed!\n")
        if (*(__wine_dbch___default)).flags as u32 & (1u32 << __WINE_DBCL_ERR as u32) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                    .as_ptr() as *const i8,
                b"NetlogonServiceMain\0".as_ptr() as *const i8,
                45,
                b"LoadLibrary() failed!\n\0".as_ptr() as *const i8,
            );
        }
        return;
    }

    pNetlogonMain = core::mem::transmute::<FARPROC, PNETLOGONMAIN>(GetProcAddress(
        hNetlogon,
        b"NlNetlogonMain\0".as_ptr() as *const i8,
    ));
    if pNetlogonMain.is_none() {
        // ERR("GetProcAddress(NlNetlogonMain) failed!\n")
        if (*(__wine_dbch___default)).flags as u32 & (1u32 << __WINE_DBCL_ERR as u32) != 0 {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                    .as_ptr() as *const i8,
                b"NetlogonServiceMain\0".as_ptr() as *const i8,
                52,
                b"GetProcAddress(NlNetlogonMain) failed!\n\0".as_ptr() as *const i8,
            );
        }
        FreeLibrary(hNetlogon as HMODULE);
        return;
    }

    // TRACE("NlNetlogonMain %p\n", pNetlogonMain)
    if (*(__wine_dbch___default)).flags as u32 & (1u32 << __WINE_DBCL_TRACE as u32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                .as_ptr() as *const i8,
            b"NetlogonServiceMain\0".as_ptr() as *const i8,
            57,
            b"NlNetlogonMain %p\n\0".as_ptr() as *const i8,
            pNetlogonMain,
        );
    }

    pNetlogonMain.unwrap()(dwArgc as i32, pszArgv);
}

unsafe extern "system" fn SamSsControlHandler(fdwControl: DWORD) {
    if (*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                .as_ptr()
                as *const core::ffi::c_char,
            b"SamSsControlHandler\0".as_ptr() as *const core::ffi::c_char,
            69,
            b"SamSsControlHandler(%lu)\n\0".as_ptr() as *const core::ffi::c_char,
            fdwControl,
        );
    }
}
unsafe extern "system" fn SamSsServiceMain(dwArgc: DWORD, pszArgv: *mut LPWSTR) {
    let mut hStatus: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS = SERVICE_STATUS {
        dwServiceType: 0,
        dwCurrentState: 0,
        dwControlsAccepted: 0,
        dwWin32ExitCode: 0,
        dwServiceSpecificExitCode: 0,
        dwCheckPoint: 0,
        dwWaitHint: 0,
    };
    if ((*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0".as_ptr() as *const i8,
            b"SamSsServiceMain\0".as_ptr() as *const i8,
            83,
            b"SamSsServiceMain(%lu %p)\n\0".as_ptr() as *const i8,
            dwArgc,
            pszArgv,
        );
    }
    hStatus = RegisterServiceCtrlHandlerW(
        SAMSS_W.as_ptr() as LPCWSTR,
        Some(SamSsControlHandler as unsafe extern "system" fn(DWORD)),
    );
    if hStatus.is_null() {
        return;
    }
    ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS as DWORD;
    ServiceStatus.dwCurrentState = SERVICE_START_PENDING as DWORD;
    ServiceStatus.dwControlsAccepted = 0;
    ServiceStatus.dwWin32ExitCode = ERROR_SUCCESS as DWORD;
    ServiceStatus.dwServiceSpecificExitCode = ERROR_SUCCESS as DWORD;
    ServiceStatus.dwCheckPoint = 1;
    ServiceStatus.dwWaitHint = 0x7530;
    SetServiceStatus(hStatus, &raw mut ServiceStatus);
    ServiceStatus.dwCurrentState = SERVICE_RUNNING as DWORD;
    ServiceStatus.dwCheckPoint = 0;
    ServiceStatus.dwWaitHint = 0;
    SetServiceStatus(hStatus, &raw mut ServiceStatus);
}

unsafe extern "system" fn DispatcherThread(pParameter: *mut core::ffi::c_void) -> u32 {
    let mut hEvent: *mut core::ffi::c_void = core::ptr::null_mut();

    // TRACE
    if (*(__wine_dbch___default as *mut __wine_debug_channel)).flags as u32 & (1u32 << __WINE_DBCL_TRACE) != 0 {
        let __dbch = __wine_dbch___default;
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                .as_ptr() as *const i8,
            b"DispatcherThread\0".as_ptr() as *const i8,
            117i32,
            b"DispatcherThread(%p)\n\0".as_ptr() as *const i8,
            pParameter,
        );
    }

    // Best-effort, BOUNDED wait for the SCM's SECURITY_SERVICES_STARTED signal,
    // then connect. This wait is intentionally neither fatal nor INFINITE: the
    // event is cross-process-shared with the SCM (services.exe) and that sharing
    // is timing-fragile in this kernel — when lsasrv won the create race the
    // SCM's SetEvent intermittently did NOT wake this waiter, so the old INFINITE
    // wait hung DispatcherThread forever, it never reached
    // StartServiceCtrlDispatcher, and the SCM timed SamSs out at 30s (connect
    // result=1053: a 30s stall before SC_AutoStartComplete on EVERY boot, all
    // arches; any added debug print perturbed the timing enough to hide it). The
    // real sync point is the SamSs control pipe (\pipe\net\NtControlPipe0), which
    // the SCM creates just BEFORE it signals this event and which — unlike the
    // event — shares reliably across processes; ScConnectControlPipe (advapi32)
    // now polls for it, so we proceed to the connect regardless of this event.
    // Open the SCM's object if present, else create our own to wait on briefly.
    let event_name = win32macros_rust::wide_bytes!(b"S\0E\0C\0U\0R\0I\0T\0Y\0_\0S\0E\0R\0V\0I\0C\0E\0S\0_\0S\0T\0A\0R\0T\0E\0D\0\0\0") as *const u16;
    hEvent = OpenEventW(SYNCHRONIZE, 0i32, event_name);
    if hEvent.is_null() {
        hEvent = CreateEventW(
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            1i32, // TRUE (manual reset)
            0i32, // FALSE
            event_name,
        );
    }
    if !hEvent.is_null() {
        let _ = WaitForSingleObject(hEvent, 3000);
        CloseHandle(hEvent);
    }

    I_ScIsSecurityProcess();

    // TRACE
    if (*(__wine_dbch___default as *mut __wine_debug_channel)).flags as u32 & (1u32 << __WINE_DBCL_TRACE) != 0 {
        let __dbch = __wine_dbch___default;
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                .as_ptr() as *const i8,
            b"DispatcherThread\0".as_ptr() as *const i8,
            156i32,
            b"Start the security services!\n\0".as_ptr() as *const i8,
        );
    }

    if StartServiceCtrlDispatcherW(
        core::ptr::addr_of!(ServiceTable) as *const SERVICE_TABLE_ENTRYW,
    ) == 0
    {
        return GetLastError();
    }

    // TRACE
    if (*(__wine_dbch___default as *mut __wine_debug_channel)).flags as u32 & (1u32 << __WINE_DBCL_TRACE) != 0 {
        let __dbch = __wine_dbch___default;
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __dbch,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/service.c\0"
                .as_ptr() as *const i8,
            b"DispatcherThread\0".as_ptr() as *const i8,
            160i32,
            b"Done!\n\0".as_ptr() as *const i8,
        );
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ServiceInit() -> NTSTATUS {
    let mut dwThreadId: u32 = 0;
    TRACE!(b"ServiceInit()\n\0".as_ptr() as *const core::ffi::c_char);
    let hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(DispatcherThread as unsafe extern "system" fn(*mut core::ffi::c_void) -> u32),
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(dwThreadId),
    );
    if hThread.is_null() {
        return GetLastError() as NTSTATUS;
    }
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/lsasrv/service.c
//  * PURPOSE:     Security service
//  * COPYRIGHT:   Copyright 2016, 2019 Eric Kohl <eric.kohl@reactos.org>
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "lsasrv.h"
// #include <winsvc.h>
// #include <winsvc_undoc.h>
//
// typedef VOID (WINAPI *PNETLOGONMAIN)(INT ArgCount, PWSTR *ArgVector);
//
// static VOID WINAPI NetlogonServiceMain(DWORD dwArgc, PWSTR *pszArgv);
// static VOID WINAPI SamSsServiceMain(DWORD dwArgc, PWSTR *pszArgv);
//
// SERVICE_TABLE_ENTRYW ServiceTable[] =
// {
//     {L"NETLOGON", NetlogonServiceMain},
//     {L"SAMSS", SamSsServiceMain},
//     {NULL, NULL}
// };
//
//
// /* FUNCTIONS ***************************************************************/
//
// static
// VOID
// WINAPI
// NetlogonServiceMain(
//      DWORD dwArgc,
//      PWSTR *pszArgv)
// {
//     HINSTANCE hNetlogon = NULL;
//     PNETLOGONMAIN pNetlogonMain = NULL;
//
//     TRACE("NetlogonServiceMain(%lu %p)\n", dwArgc, pszArgv);
//
//     hNetlogon = LoadLibraryW(L"Netlogon.dll");
//     if (hNetlogon == NULL)
//     {
//         ERR("LoadLibrary() failed!\n");
//         return;
//     }
//
//     pNetlogonMain = (PNETLOGONMAIN)GetProcAddress(hNetlogon, "NlNetlogonMain");
//     if (pNetlogonMain == NULL)
//     {
//         ERR("GetProcAddress(NlNetlogonMain) failed!\n");
//         FreeLibrary(hNetlogon);
//         return;
//     }
//
//     TRACE("NlNetlogonMain %p\n", pNetlogonMain);
//
//     pNetlogonMain(dwArgc, pszArgv);
// }
//
//
// static
// VOID
// WINAPI
// SamSsControlHandler(
//      DWORD fdwControl)
// {
//     TRACE("SamSsControlHandler(%lu)\n", fdwControl);
// }
//
//
// static
// VOID
// WINAPI
// SamSsServiceMain(
//      DWORD dwArgc,
//      PWSTR *pszArgv)
// {
//     SERVICE_STATUS_HANDLE hStatus;
//     SERVICE_STATUS ServiceStatus;
//
//     TRACE("SamSsServiceMain(%lu %p)\n", dwArgc, pszArgv);
//
//     hStatus = RegisterServiceCtrlHandlerW(L"SAMSS",
//                                           SamSsControlHandler);
//     if (hStatus == NULL)
//         return;
//
//     ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS;
//     ServiceStatus.dwCurrentState = SERVICE_START_PENDING;
//     ServiceStatus.dwControlsAccepted = 0;
//     ServiceStatus.dwWin32ExitCode = ERROR_SUCCESS;
//     ServiceStatus.dwServiceSpecificExitCode = ERROR_SUCCESS;
//     ServiceStatus.dwCheckPoint = 1;
//     ServiceStatus.dwWaitHint = 0x7530;
//
//     SetServiceStatus(hStatus, &ServiceStatus);
//
//     ServiceStatus.dwCurrentState = SERVICE_RUNNING;
//     ServiceStatus.dwCheckPoint = 0;
//     ServiceStatus.dwWaitHint = 0;
//
//     SetServiceStatus(hStatus, &ServiceStatus);
// }
//
//
// static
// DWORD
// WINAPI
// DispatcherThread(
//      PVOID pParameter)
// {
//     HANDLE hEvent;
//     DWORD dwError;
//
//     TRACE("DispatcherThread(%p)\n", pParameter);
//
//     /* Create or open the SECURITY_SERVICES_STARTED event */
//     hEvent = CreateEventW(NULL,
//                           TRUE,
//                           FALSE,
//                           L"SECURITY_SERVICES_STARTED");
//     if (hEvent == NULL)
//     {
//         dwError = GetLastError();
//         if (dwError != ERROR_ALREADY_EXISTS)
//             return dwError;
//
//         hEvent = OpenEventW(SYNCHRONIZE,
//                             FALSE,
//                             L"SECURITY_SERVICES_STARTED");
//         if (hEvent == NULL)
//             return GetLastError();
//     }
//
//     /* Wait for the SECURITY_SERVICES_STARTED event to be signaled */
//     TRACE("Waiting for the SECURITY_SERVICES_STARTED event!\n");
//     dwError = WaitForSingleObject(hEvent, INFINITE);
//     TRACE("WaitForSingleObject returned %lu\n", dwError);
//
//     /* Close the event handle */
//     CloseHandle(hEvent);
//
//     /* Fail, if the event was not signaled */
//     if (dwError != WAIT_OBJECT_0)
//     {
//         ERR("Wait failed!\n");
//         return dwError;
//     }
//
//     /* This is the security process */
//     I_ScIsSecurityProcess();
//
//     /* Start the services */
//     TRACE("Start the security services!\n");
//     if (!StartServiceCtrlDispatcherW(ServiceTable))
//         return GetLastError();
//
//     TRACE("Done!\n");
//
//     return ERROR_SUCCESS;
// }
//
//
// NTSTATUS
// WINAPI
// ServiceInit(VOID)
// {
//     HANDLE hThread;
//     DWORD dwThreadId;
//
//     TRACE("ServiceInit()\n");
//
//     hThread = CreateThread(NULL,
//                            0,
//                            DispatcherThread,
//                            NULL,
//                            0,
//                            &dwThreadId);
//     if (hThread == NULL)
//        return (NTSTATUS)GetLastError();
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

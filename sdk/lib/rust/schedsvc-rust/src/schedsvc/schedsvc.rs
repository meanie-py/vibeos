use crate::schedsvc::job::GetNextJobTimeout;
use crate::schedsvc::job::LoadJobs;
use crate::schedsvc::rpcserver::RpcThreadRoutine;
use crate::schedsvc::job::RunCurrentJobs;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"S\0c\0h\0e\0d\0u\0l\0e\0\0\0") };

static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

static mut ServiceStatus: SERVICE_STATUS = _SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};

#[no_mangle]
pub static mut Events: [HANDLE; 3] = [NULL, NULL, NULL];

// WINE_DEFAULT_DEBUG_CHANNEL(schedsvc): debug channel name for this translation unit

unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS as DWORD;
    ServiceStatus.dwCurrentState = dwState;

    if dwState == SERVICE_PAUSED as DWORD || dwState == SERVICE_RUNNING as DWORD {
        ServiceStatus.dwControlsAccepted = (SERVICE_ACCEPT_STOP
            | SERVICE_ACCEPT_SHUTDOWN
            | SERVICE_ACCEPT_PAUSE_CONTINUE) as DWORD;
    } else {
        ServiceStatus.dwControlsAccepted = 0 as DWORD;
    }

    ServiceStatus.dwWin32ExitCode = 0 as DWORD;
    ServiceStatus.dwServiceSpecificExitCode = 0 as DWORD;
    ServiceStatus.dwCheckPoint = 0 as DWORD;

    if dwState == SERVICE_START_PENDING as DWORD
        || dwState == SERVICE_STOP_PENDING as DWORD
        || dwState == SERVICE_PAUSE_PENDING as DWORD
        || dwState == SERVICE_CONTINUE_PENDING as DWORD
    {
        ServiceStatus.dwWaitHint = 10000 as DWORD;
    } else {
        ServiceStatus.dwWaitHint = 0 as DWORD;
    }

    SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
}

unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"base/services/schedsvc/schedsvc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ServiceControlHandler\0" as *const u8 as *const ::core::ffi::c_char,
                    87 as ::core::ffi::c_int,
                    b"  SERVICE_CONTROL_STOP/SERVICE_CONTROL_SHUTDOWN received\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                );
            }
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            /* Stop listening to incoming RPC messages */
            RpcMgmtStopServerListening(core::ptr::null_mut());
            if !Events[0 as usize].is_null() {
                SetEvent(Events[0 as usize]);
            }
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_PAUSE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"base/services/schedsvc/schedsvc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ServiceControlHandler\0" as *const u8 as *const ::core::ffi::c_char,
                    96 as ::core::ffi::c_int,
                    b"  SERVICE_CONTROL_PAUSE received\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            UpdateServiceStatus(SERVICE_PAUSED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_CONTINUE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"base/services/schedsvc/schedsvc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ServiceControlHandler\0" as *const u8 as *const ::core::ffi::c_char,
                    101 as ::core::ffi::c_int,
                    b"  SERVICE_CONTROL_CONTINUE received\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            UpdateServiceStatus(SERVICE_RUNNING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_INTERROGATE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"base/services/schedsvc/schedsvc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ServiceControlHandler\0" as *const u8 as *const ::core::ffi::c_char,
                    106 as ::core::ffi::c_int,
                    b"  SERVICE_CONTROL_INTERROGATE received\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
            return ERROR_SUCCESS;
        }

        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"base/services/schedsvc/schedsvc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ServiceControlHandler\0" as *const u8 as *const ::core::ffi::c_char,
                    122 as ::core::ffi::c_int,
                    b"  Control %lu received\n\0" as *const u8 as *const ::core::ffi::c_char,
                    dwControl,
                );
            }
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

unsafe extern "C" fn ServiceInit() -> DWORD {
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = 0;

    /* Initialize the job list */
    InitializeListHead(core::ptr::addr_of_mut!(JobListHead));

    /* Initialize the job list lock */
    RtlInitializeResource(core::ptr::addr_of_mut!(JobListLock));

    /* Initialize the start list */
    InitializeListHead(core::ptr::addr_of_mut!(StartListHead));

    /* Initialize the start list lock */
    RtlInitializeResource(core::ptr::addr_of_mut!(StartListLock));

    /* Load stored jobs from the registry */
    dwError = LoadJobs() as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Start the RPC thread */
    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        core::mem::transmute::<
            Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
            LPTHREAD_START_ROUTINE,
        >(Some(
            RpcThreadRoutine as unsafe extern "system" fn(LPVOID) -> DWORD,
        )),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if hThread.is_null() {
        return GetLastError();
    }

    CloseHandle(hThread);

    /* Create the stop event */
    Events[0 as usize] = CreateEvent(
        core::ptr::null_mut(),
        (FALSE) as i32,
        (FALSE) as i32,
        core::ptr::null(),
    );
    if Events[0 as usize].is_null() {
        return GetLastError();
    }

    /* Create the update event */
    Events[1 as usize] = CreateEvent(
        core::ptr::null_mut(),
        (FALSE) as i32,
        (FALSE) as i32,
        core::ptr::null(),
    );
    if Events[1 as usize].is_null() {
        CloseHandle(Events[0 as usize]);
        return GetLastError();
    }

    Events[2 as usize] = CreateWaitableTimerW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        core::ptr::null(),
    );
    if Events[2 as usize].is_null() {
        CloseHandle(Events[1 as usize]);
        CloseHandle(Events[0 as usize]);
        return GetLastError();
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SchedServiceMain(argc: DWORD, argv: *mut LPTSTR) {
    let mut dwWait: DWORD;
    let mut dwError: DWORD;

    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        &raw mut ServiceName as *mut WCHAR as LPCWSTR,
        Some(ServiceControlHandler as unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    dwError = ServiceInit();
    if dwError != ERROR_SUCCESS {
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    UpdateServiceStatus(SERVICE_RUNNING);

    GetNextJobTimeout(Events[2]);

    loop {
        /* Wait for the next event */
        dwWait = WaitForMultipleObjects(
            3,
            &raw mut Events as *mut HANDLE,
            (FALSE) as BOOL,
            INFINITE,
        );
        if dwWait == WAIT_OBJECT_0 {
            break;
        } else if dwWait == WAIT_OBJECT_0 + 1 {
            RtlAcquireResourceShared(&raw mut JobListLock, TRUE as BOOLEAN);
            GetNextJobTimeout(Events[2]);
            RtlReleaseResource(&raw mut JobListLock);
        } else if dwWait == WAIT_OBJECT_0 + 2 {
            RtlAcquireResourceExclusive(&raw mut JobListLock, TRUE as BOOLEAN);
            RunCurrentJobs();
            GetNextJobTimeout(Events[2]);
            RtlReleaseResource(&raw mut JobListLock);
        }
    }

    /* Close the start and update event handles */
    CloseHandle(Events[0]);
    CloseHandle(Events[1]);
    CloseHandle(Events[2]);

    /* Stop the service */
    UpdateServiceStatus(SERVICE_STOPPED);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return TRUE as BOOL;
}

use crate::srvsvc::rpcserver::RpcThreadRoutine;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [WCHAR; 13]>(*b"L\0a\0n\0m\0a\0n\0s\0e\0r\0v\0e\0r\0\0\0")
};

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
pub static mut dwServiceBits: DWORD = 0 as DWORD;


unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS as DWORD;
    ServiceStatus.dwCurrentState = dwState;
    ServiceStatus.dwControlsAccepted = 0 as DWORD;
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

    if dwState == SERVICE_RUNNING as DWORD {
        ServiceStatus.dwControlsAccepted = (SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN) as DWORD;
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
        SERVICE_CONTROL_STOP => {
            TRACE!("  SERVICE_CONTROL_STOP received\n");
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            /* Stop listening to incoming RPC messages */
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_PAUSE => {
            TRACE!("  SERVICE_CONTROL_PAUSE received\n");
            UpdateServiceStatus(SERVICE_PAUSED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_CONTINUE => {
            TRACE!("  SERVICE_CONTROL_CONTINUE received\n");
            UpdateServiceStatus(SERVICE_RUNNING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_INTERROGATE => {
            TRACE!("  SERVICE_CONTROL_INTERROGATE received\n");
            SetServiceStatus(
                ServiceStatusHandle,
                core::ptr::addr_of_mut!(ServiceStatus),
            );
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_SHUTDOWN => {
            TRACE!("  SERVICE_CONTROL_SHUTDOWN received\n");
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            /* Stop listening to incoming RPC messages */
            RpcMgmtStopServerListening(core::ptr::null_mut());
            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }

        _ => {
            TRACE!("  Control %lu received\n", dwControl);
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    };
}

unsafe extern "C" fn ServiceInit() -> DWORD {
    let mut hThread: HANDLE = core::ptr::null_mut();
    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcThreadRoutine),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );

    if hThread.is_null() {
        ERR!("Can't create PortThread\n");
        return GetLastError();
    } else {
        CloseHandle(hThread);
    }

    /* Report a running server service */
    SetServiceBits(
        ServiceStatusHandle,
        SV_TYPE_SERVER,
        (TRUE as i32),
        (TRUE as i32),
    );

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: DWORD, argv: *mut LPTSTR) {
    let mut dwError: DWORD;

    let _ = argc;
    let _ = argv;

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        ERR!("RegisterServiceCtrlHandlerExW() failed! (Error %lu)\n", GetLastError());
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    dwError = ServiceInit();
    if dwError != ERROR_SUCCESS {
        ERR!("Service stopped (dwError: %lu\n", dwError);
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    UpdateServiceStatus(SERVICE_RUNNING);
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

    return (TRUE) as BOOL;
}

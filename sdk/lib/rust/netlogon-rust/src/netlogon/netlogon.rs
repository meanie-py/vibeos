use crate::netlogon::rpcserver::RpcThreadRoutine;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut ServiceName: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"n\0e\0t\0l\0o\0g\0o\0n\0\0\0") };

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


unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    ServiceStatus.dwCurrentState = dwState;
    ServiceStatus.dwControlsAccepted = 0;
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
unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP => {
            TRACE!("  SERVICE_CONTROL_STOP received\n");
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
            SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
            return ERROR_SUCCESS;
        }
        SERVICE_CONTROL_SHUTDOWN => {
            TRACE!("  SERVICE_CONTROL_SHUTDOWN received\n");
            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }
        _ => {
            TRACE!("  Control %lu received\n", dwControl);
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
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

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn NlNetlogonMain(ArgCount: INT, ArgVector: *mut PWSTR) {
    let mut dwError: DWORD;

    UNREFERENCED_PARAMETER!(ArgCount);
    UNREFERENCED_PARAMETER!(ArgVector);

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(ServiceName) as PWSTR,
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
    pvReserved: PVOID,
) -> BOOL {
    UNREFERENCED_PARAMETER!(pvReserved);

    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            hDllInstance = hinstDLL;
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return (TRUE) as BOOL;
}

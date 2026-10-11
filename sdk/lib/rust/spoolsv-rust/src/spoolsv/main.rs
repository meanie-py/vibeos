use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hServiceStatus: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

#[no_mangle]
pub static mut ServiceStatus: SERVICE_STATUS = _SERVICE_STATUS {
    dwServiceType: 0,
    dwCurrentState: 0,
    dwControlsAccepted: 0,
    dwWin32ExitCode: 0,
    dwServiceSpecificExitCode: 0,
    dwCheckPoint: 0,
    dwWaitHint: 0,
};

#[no_mangle]
pub static mut wszServiceName: [WCHAR; 8] =
    unsafe { ::core::mem::transmute::<[u8; 16], [WCHAR; 8]>(*b"S\0p\0o\0o\0l\0e\0r\0\0\0") };

unsafe extern "C" fn _UpdateServiceStatus(dwNewStatus: DWORD, dwCheckPoint: DWORD) {
    ServiceStatus.dwCheckPoint = dwCheckPoint;
    ServiceStatus.dwCurrentState = dwNewStatus;
    SetServiceStatus(hServiceStatus, core::ptr::addr_of_mut!(ServiceStatus));
}

unsafe extern "system" fn _ServiceControlHandlerEx(dwControl: DWORD, dwEventType: DWORD, lpEventData: LPVOID, lpContext: LPVOID) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_SHUTDOWN | SERVICE_CONTROL_STOP => {
            _UpdateServiceStatus(SERVICE_STOP_PENDING, 1);
            RpcMgmtStopServerListening(core::ptr::null_mut());
            _UpdateServiceStatus(SERVICE_STOPPED, 0);
            return NO_ERROR;
        }
        SERVICE_CONTROL_INTERROGATE => {
            return NO_ERROR;
        }
        _ => {
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

unsafe extern "system" fn _ServiceMain(dwArgc: DWORD, lpszArgv: *mut LPWSTR) {
    let mut hThread: HANDLE = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(dwArgc);
    UNREFERENCED_PARAMETER!(lpszArgv);

    // Register our service for control
    hServiceStatus = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(wszServiceName) as LPCWSTR,
        Some(_ServiceControlHandlerEx as unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD),
        core::ptr::null_mut(),
    );

    // Report initial SERVICE_START_PENDING status
    ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    ServiceStatus.dwWaitHint = 4000;
    ServiceStatus.dwWin32ExitCode = NO_ERROR;
    _UpdateServiceStatus(SERVICE_START_PENDING, 0);

    // Create a thread for serving RPC requests
    hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        ::core::mem::transmute::<
            Option<unsafe extern "system" fn(LPVOID) -> DWORD>,
            LPTHREAD_START_ROUTINE,
        >(Some(LrpcThreadProc as unsafe extern "system" fn(LPVOID) -> DWORD)),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if hThread.is_null() {
        _UpdateServiceStatus(SERVICE_STOPPED, 0);
        return;
    }

    // We don't need the thread handle. Keeping it open blocks the thread from terminating.
    CloseHandle(hThread);

    // Initialize the routing layer in spoolss.dll
    if InitializeRouter(hServiceStatus as HANDLE) == 0 {
        _UpdateServiceStatus(SERVICE_STOPPED, 0);
        return;
    }

    // We're alive!
    _UpdateServiceStatus(SERVICE_RUNNING, 0);
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut WCHAR) -> i32 {
    let mut ServiceTable: [SERVICE_TABLE_ENTRYW; 2] = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::addr_of_mut!(wszServiceName) as LPWSTR,
            lpServiceProc: Some(_ServiceMain as unsafe extern "system" fn(DWORD, *mut LPWSTR)),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];

    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    StartServiceCtrlDispatcherW(core::ptr::addr_of_mut!(ServiceTable) as *mut SERVICE_TABLE_ENTRYW);

    return 0;
}

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"w\0u\0a\0u\0s\0e\0r\0v\0\0\0") };

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

static mut hStopEvent: HANDLE = NULL;

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
        ServiceStatus.dwWaitHint = 1000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    if dwState == SERVICE_RUNNING {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    }

    SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
    DPRINT1!("WU UpdateServiceStatus(%lu) called\n", dwState);
}

unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP => {
            DPRINT1!("WU ServiceControlHandler(SERVICE_CONTROL_STOP) received\n");
            SetEvent(hStopEvent);
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_PAUSE => {
            DPRINT1!("WU ServiceControlHandler(SERVICE_CONTROL_PAUSE) received\n");
            UpdateServiceStatus(SERVICE_PAUSED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_CONTINUE => {
            DPRINT1!("WU ServiceControlHandler(SERVICE_CONTROL_CONTINUE) received\n");
            UpdateServiceStatus(SERVICE_RUNNING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_INTERROGATE => {
            DPRINT1!("WU ServiceControlHandler(SERVICE_CONTROL_INTERROGATE) received\n");
            SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_SHUTDOWN => {
            DPRINT1!("WU ServiceControlHandler(SERVICE_CONTROL_SHUTDOWN) received\n");
            SetEvent(hStopEvent);
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            return ERROR_SUCCESS;
        }

        _ => {
            DPRINT1!("WU ServiceControlHandler(Control %lu) received\n", dwControl);
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: DWORD, argv: *mut LPTSTR) {
    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    DPRINT!("WU ServiceMain() called\n");

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of!(ServiceName) as *const WCHAR,
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        DPRINT1!("RegisterServiceCtrlHandlerExW() failed (Error %lu)\n", GetLastError());
        return;
    }

    hStopEvent = CreateEventW(
        core::ptr::null_mut(),
        (TRUE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null_mut(),
    );
    'done: {
        if hStopEvent.is_null() {
            DPRINT1!("CreateEvent() failed (Error %lu)\n", GetLastError());
            break 'done;
        }

        UpdateServiceStatus(SERVICE_RUNNING);

        WaitForSingleObject(hStopEvent, INFINITE);
        CloseHandle(hStopEvent);
    }

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

    return (TRUE) as BOOL;
}

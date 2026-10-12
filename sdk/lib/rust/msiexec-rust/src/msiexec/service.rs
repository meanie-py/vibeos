use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hstatus: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

static mut thread: HANDLE = ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut kill_event: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

// #define WIN32_LEAN_AND_MEAN: empty object-like macro, no value to emit.
// WINE_DEFAULT_DEBUG_CHANNEL(msiexec): debug-channel registration, handled by the DPRINT macros.

unsafe extern "C" fn KillService() {
    WINE_TRACE!(b"Killing service\n\0".as_ptr() as *const core::ffi::c_char);
    SetEvent(kill_event);
}

unsafe extern "C" fn UpdateSCMStatus(dwCurrentState: DWORD, dwWin32ExitCode: DWORD, dwServiceSpecificExitCode: DWORD) -> BOOL {
    let mut status: SERVICE_STATUS = core::mem::zeroed();

    status.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    status.dwCurrentState = dwCurrentState;

    if dwCurrentState == SERVICE_START_PENDING
        || dwCurrentState == SERVICE_STOP_PENDING
        || dwCurrentState == SERVICE_STOPPED
    {
        status.dwControlsAccepted = 0;
    } else {
        status.dwControlsAccepted = SERVICE_ACCEPT_STOP |
                                    SERVICE_ACCEPT_PAUSE_CONTINUE |
                                    SERVICE_ACCEPT_SHUTDOWN;
    }

    if dwServiceSpecificExitCode == 0 {
        status.dwWin32ExitCode = dwWin32ExitCode;
    } else {
        status.dwWin32ExitCode = ERROR_SERVICE_SPECIFIC_ERROR;
    }

    status.dwServiceSpecificExitCode = dwServiceSpecificExitCode;
    status.dwCheckPoint = 0;
    status.dwWaitHint = 0;

    if SetServiceStatus(hstatus, core::ptr::addr_of_mut!(status)) == 0 {
        report_error(b"Failed to set service status\n\0".as_ptr() as *const i8);
        KillService();
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "system" fn ServiceCtrlHandler(code: DWORD) {
    WINE_TRACE!(b"%ld\n\0".as_ptr() as *const i8, code);

    if code == SERVICE_CONTROL_SHUTDOWN || code == SERVICE_CONTROL_STOP {
        UpdateSCMStatus(SERVICE_STOP_PENDING, NO_ERROR, 0);
        KillService();
    } else {
        report_error(b"Unhandled service control code: %ld\n\0".as_ptr() as *const i8, code);
        UpdateSCMStatus(SERVICE_RUNNING, NO_ERROR, 0);
    }
}
unsafe extern "system" fn ServiceExecutionThread(param: LPVOID) -> DWORD {
    WaitForSingleObject(kill_event, INFINITE);

    return 0;
}

unsafe extern "C" fn StartServiceThread() -> BOOL {
    let mut id: DWORD = 0;
    thread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(ServiceExecutionThread),
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(id),
    );
    if thread.is_null() {
        report_error(b"Failed to create thread\n\0".as_ptr() as *const ::core::ffi::c_char);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "system" fn ServiceMain(argc: DWORD, argv: *mut LPSTR) {
    hstatus = RegisterServiceCtrlHandlerA(b"MSIServer\0".as_ptr() as LPCSTR, Some(ServiceCtrlHandler));
    if hstatus.is_null() {
        report_error(b"Failed to register service ctrl handler\n\0".as_ptr() as *const i8);
        return;
    }

    UpdateSCMStatus(SERVICE_START_PENDING, NO_ERROR, 0);

    kill_event = CreateEventW(core::ptr::null_mut(), (TRUE) as BOOL, (FALSE) as BOOL, core::ptr::null());
    if kill_event.is_null() {
        report_error(b"Failed to create event\n\0".as_ptr() as *const i8);
        KillService();
        UpdateSCMStatus(SERVICE_STOPPED, NO_ERROR, 0);
        return;
    }

    if StartServiceThread() == 0 {
        KillService();
        UpdateSCMStatus(SERVICE_STOPPED, NO_ERROR, 0);
        return;
    }

    UpdateSCMStatus(SERVICE_RUNNING, NO_ERROR, 0);
    WaitForSingleObject(thread, INFINITE);
    UpdateSCMStatus(SERVICE_STOPPED, NO_ERROR, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DoService() -> DWORD {
    // service.c: `char service_name[] = "MSIServer";` (the translation built "MSServerr").
    let mut service_name: [i8; 10] =
        core::mem::transmute::<[u8; 10], [i8; 10]>(*b"MSIServer\0");
    let service: [SERVICE_TABLE_ENTRYA; 2] = [
        SERVICE_TABLE_ENTRYA {
            lpServiceName: service_name.as_mut_ptr(),
            lpServiceProc: Some(ServiceMain),
        },
        SERVICE_TABLE_ENTRYA {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];

    WINE_TRACE!(b"Starting MSIServer service\n\0".as_ptr() as *const i8);

    if StartServiceCtrlDispatcherA(service.as_ptr()) == 0 {
        report_error(b"Failed to start MSIServer service\n\0".as_ptr() as *const i8);
        return 1;
    }

    return 0;
}

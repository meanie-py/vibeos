use crate::dcomlaunch::network::DealWithDeviceEvent;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [WCHAR; 11]>(*b"d\0c\0o\0m\0l\0a\0u\0n\0c\0h\0\0\0")
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

static mut ShutdownEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

unsafe extern "C" fn UpdateServiceStatus(mut dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS as DWORD;
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
    SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
}

unsafe extern "system" fn ServiceControlHandler(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    DPRINT1!("ServiceControlHandler() called\n");

    match dwControl {
        SERVICE_CONTROL_STOP => {
            DPRINT1!("  SERVICE_CONTROL_STOP received\n");
            SetEvent(ShutdownEvent);
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_PAUSE => {
            DPRINT1!("  SERVICE_CONTROL_PAUSE received\n");
            UpdateServiceStatus(SERVICE_PAUSED);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_CONTINUE => {
            DPRINT1!("  SERVICE_CONTROL_CONTINUE received\n");
            UpdateServiceStatus(SERVICE_RUNNING);
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_INTERROGATE => {
            DPRINT1!("  SERVICE_CONTROL_INTERROGATE received\n");
            SetServiceStatus(
                ServiceStatusHandle,
                core::ptr::addr_of_mut!(ServiceStatus),
            );
            return ERROR_SUCCESS;
        }

        SERVICE_CONTROL_SHUTDOWN => {
            DPRINT1!("  SERVICE_CONTROL_SHUTDOWN received\n");
            SetEvent(ShutdownEvent);
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            return ERROR_SUCCESS;
        }

        _ => {
            DPRINT1!("  Control %lu received\n", dwControl);
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ServiceMain(mut argc: DWORD, mut argv: *mut LPTSTR) {
    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    DPRINT!("ServiceMain() called\n");

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        Some(ServiceControlHandler as unsafe extern "system" fn(DWORD, DWORD, LPVOID, LPVOID) -> DWORD),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        DPRINT1!("RegisterServiceCtrlHandlerExW() failed! (Error %lu)\n", GetLastError());
        return;
    }

    ShutdownEvent = CreateEventW(core::ptr::null_mut(), (TRUE) as BOOL, (FALSE) as BOOL, core::ptr::null_mut());

    DealWithDeviceEvent();

    UpdateServiceStatus(SERVICE_RUNNING);

    WaitForSingleObject(ShutdownEvent, INFINITE);
    CloseHandle(ShutdownEvent);

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

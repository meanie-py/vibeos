// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5DD76177866D: [u16; 22] = [
    0x0047u16, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x005C, 0x0054, 0x0068, 0x0065,
    0x006D, 0x0065, 0x0053, 0x0074, 0x006F, 0x0070, 0x0045, 0x0076, 0x0065, 0x006E,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_614A1D97B2BD: [u16; 23] = [
    0x0047u16, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x005C, 0x0054, 0x0068, 0x0065,
    0x006D, 0x0065, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0045, 0x0076, 0x0065,
    0x006E, 0x0074, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ServiceName: [WCHAR; 7] =
    unsafe { ::core::mem::transmute::<[u8; 14], [WCHAR; 7]>(*b"T\0h\0e\0m\0e\0s\0\0\0") };

#[no_mangle]
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE =
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
pub static mut StartEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut StopEvent: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "shsvcs";

unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    ServiceStatus.dwCurrentState = dwState;
    if dwState == SERVICE_RUNNING {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_PAUSE_CONTINUE | SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    } else if dwState == SERVICE_PAUSED {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_PAUSE_CONTINUE;
    } else {
        ServiceStatus.dwControlsAccepted = 0;
    }
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

unsafe extern "system" fn ServiceControlHandler(dwControl: DWORD, dwEventType: DWORD, lpEventData: LPVOID, lpContext: LPVOID) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP => {
            TRACE!("  SERVICE_CONTROL_STOP received\n");

            /* Signal the theme server in winlogon to remove theme hooks */
            ResetEvent(StartEvent);
            SetEvent(StopEvent);
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
            TRACE!("  Control %lu received\n");
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ThemeServiceMain(argc: DWORD, argv: *mut LPTSTR) {
    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        ERR!("RegisterServiceCtrlHandlerExW() failed! (Error %lu)\n", GetLastError());
        return;
    }

    StartEvent = CreateEventW(
        core::ptr::null_mut(),
        (TRUE) as BOOL,
        (FALSE) as BOOL,
        RUSTCONV_WSTR_614A1D97B2BD.as_ptr() as LPCWSTR,
    );
    StopEvent = CreateEventW(
        core::ptr::null_mut(),
        (TRUE) as BOOL,
        (FALSE) as BOOL,
        RUSTCONV_WSTR_5DD76177866D.as_ptr() as LPCWSTR,
    );

    UpdateServiceStatus(SERVICE_RUNNING);

    /* Signal the theme server in winlogon to install theme hooks */
    ResetEvent(StopEvent);
    SetEvent(StartEvent);
}

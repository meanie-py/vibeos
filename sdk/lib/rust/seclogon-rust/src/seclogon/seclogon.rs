use crate::seclogon::rpcserver::StartRpcServer;
use crate::seclogon::rpcserver::StopRpcServer;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut lpServiceGlobals: PSVCHOST_GLOBAL_DATA =
    ::core::ptr::null::<_SVCHOST_GLOBAL_DATA>() as *mut _SVCHOST_GLOBAL_DATA;

static mut ServiceName: [WCHAR; 9] =
    unsafe { ::core::mem::transmute::<[u8; 18], [WCHAR; 9]>(*b"s\0e\0c\0l\0o\0g\0o\0n\0\0\0") };

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

pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "seclogon";

unsafe extern "C" fn UpdateServiceStatus(dwState: DWORD) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS;
    ServiceStatus.dwCurrentState = dwState;

    if dwState == SERVICE_PAUSED || dwState == SERVICE_RUNNING {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP |
                                           SERVICE_ACCEPT_SHUTDOWN |
                                           SERVICE_ACCEPT_PAUSE_CONTINUE;
    } else {
        ServiceStatus.dwControlsAccepted = 0;
    }

    ServiceStatus.dwWin32ExitCode = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCheckPoint = 0;

    if dwState == SERVICE_START_PENDING ||
        dwState == SERVICE_STOP_PENDING ||
        dwState == SERVICE_PAUSE_PENDING ||
        dwState == SERVICE_CONTINUE_PENDING {
        ServiceStatus.dwWaitHint = 10000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    SetServiceStatus(ServiceStatusHandle,
                     core::ptr::addr_of_mut!(ServiceStatus));
}

unsafe extern "system" fn ServiceControlHandlerEx(
    dwControl: DWORD,
    dwEventType: DWORD,
    lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP => {
            TRACE!("  SERVICE_CONTROL_STOP received\n");
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            StopRpcServer();
            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }
        SERVICE_CONTROL_PAUSE => {
            TRACE!("  SERVICE_CONTROL_PAUSE received\n");
            UpdateServiceStatus(SERVICE_PAUSE_PENDING);
            StopRpcServer();
            UpdateServiceStatus(SERVICE_PAUSED);
            return ERROR_SUCCESS;
        }
        SERVICE_CONTROL_CONTINUE => {
            TRACE!("  SERVICE_CONTROL_CONTINUE received\n");
            UpdateServiceStatus(SERVICE_CONTINUE_PENDING);
            StartRpcServer();
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
            StopRpcServer();
            UpdateServiceStatus(SERVICE_STOPPED);
            return ERROR_SUCCESS;
        }
        _ => {
            TRACE!("  Control %lu received\n", dwControl);
            return ERROR_CALL_NOT_IMPLEMENTED;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SvchostPushServiceGlobals(mut lpGlobals: PSVCHOST_GLOBAL_DATA) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"base/services/seclogon/seclogon.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SvchostPushServiceGlobals\0" as *const u8 as *const ::core::ffi::c_char,
            119 as ::core::ffi::c_int,
            b"SvchostPushServiceGlobals(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            lpGlobals,
        );
    }
    lpServiceGlobals = lpGlobals;
}

#[no_mangle]
pub unsafe extern "system" fn SvcEntry_Seclogon(ArgCount: INT, ArgVector: *mut PWSTR) {
    let mut dwError: DWORD = 0;

    UNREFERENCED_PARAMETER!(ArgCount);
    UNREFERENCED_PARAMETER!(ArgVector);

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(ServiceName) as *mut WCHAR as LPCWSTR,
        Some(ServiceControlHandlerEx),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle.is_null() {
        ERR!("RegisterServiceCtrlHandlerExW() failed! (Error %lu)\n", GetLastError());
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    dwError = StartRpcServer();
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

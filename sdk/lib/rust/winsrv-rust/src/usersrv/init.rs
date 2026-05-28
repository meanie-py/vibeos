use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
// Sibling fns referenced by the dispatch table + callbacks.
use super::harderror::{UserInitHardErrorsCache, UserServerHardError};
use super::register::{SrvRegisterLogonProcess, SrvRegisterServicesProcess};
use super::shutdown::{
    SrvEndTask, SrvExitWindowsEx, SrvRecordShutdownReason, UserClientShutdown,
};

#[no_mangle]
pub static mut UserServerDllInstance: HINSTANCE = core::ptr::null_mut();

#[no_mangle]
pub static mut ghPowerRequestEvent: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut ghMediaRequestEvent: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut CsrApiPort: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut UserServerHeap: HANDLE = core::ptr::null_mut();

static mut UserServerApiDispatchTable: [PCSR_API_ROUTINE; 10] = [
    Some(SrvExitWindowsEx),
    Some(SrvEndTask),
    Some(SrvLogon),
    Some(SrvRegisterServicesProcess),
    Some(SrvActivateDebugger),
    Some(SrvGetThreadConsoleDesktop),
    Some(SrvDeviceEvent),
    Some(SrvRegisterLogonProcess),
    Some(SrvCreateSystemThreads),
    Some(SrvRecordShutdownReason),
];

static mut UserServerApiServerValidTable: [BOOLEAN; 10] = [
    0,  // SrvExitWindowsEx
    0,  // SrvEndTask
    0,  // SrvLogon
    0,  // SrvRegisterServicesProcess
    0,  // SrvActivateDebugger
    1,  // SrvGetThreadConsoleDesktop
    0,  // SrvDeviceEvent
    0,  // SrvRegisterLogonProcess
    0,  // SrvCreateSystemThreads
    0,  // SrvRecordShutdownReason
];

#[cfg(feature = "CSR_DBG")]
static mut UserServerApiNameTable: [*const i8; 10] = [
    b"SrvExitWindowsEx\0".as_ptr() as *const i8,
    b"SrvEndTask\0".as_ptr() as *const i8,
    b"SrvLogon\0".as_ptr() as *const i8,
    b"SrvRegisterServicesProcess\0".as_ptr() as *const i8,
    b"SrvActivateDebugger\0".as_ptr() as *const i8,
    b"SrvGetThreadConsoleDesktop\0".as_ptr() as *const i8,
    b"SrvDeviceEvent\0".as_ptr() as *const i8,
    b"SrvRegisterLogonProcess\0".as_ptr() as *const i8,
    b"SrvCreateSystemThreads\0".as_ptr() as *const i8,
    b"SrvRecordShutdownReason\0".as_ptr() as *const i8,
];

#[no_mangle]
pub unsafe extern "system" fn FindTopLevelWnd(
    hWnd: HWND,
    lParam: LPARAM,
) -> i32 {
    if GetWindow(hWnd, GW_OWNER as u32).is_null() {
        *(lParam as *mut HWND) = hWnd;
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn _UserSoundSentry() -> i32 {
    1
}

unsafe extern "system" fn CreateSystemThreads(pParam: PVOID) -> ULONG {
    NtUserCallOneParam(pParam as DWORD_PTR, ONEPARAM_ROUTINE_CREATESYSTEMTHREADS);
    RtlExitUserThread(0);
    0
}

unsafe extern "system" fn SrvCreateSystemThreads(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyStatus: *mut ULONG,
) -> ULONG {
    let Status = CsrExecServerThread(Some(CreateSystemThreads), core::ptr::null_mut());
    if Status < 0 {
        // DPRINT1 omitted
    }
    Status as ULONG
}

unsafe extern "system" fn SrvActivateDebugger(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyStatus: *mut ULONG,
) -> ULONG {
    STATUS_NOT_IMPLEMENTED as ULONG
}

unsafe extern "system" fn SrvGetThreadConsoleDesktop(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyStatus: *mut ULONG,
) -> ULONG {
    let GetThreadConsoleDesktopRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as *mut _USER_API_MESSAGE)).Data.GetThreadConsoleDesktopRequest
    );

    let Status = GetThreadConsoleDesktop(
        (*GetThreadConsoleDesktopRequest).ThreadId,
        core::ptr::addr_of_mut!((*GetThreadConsoleDesktopRequest).ConsoleDesktop),
    );

    if Status < 0 {
        // DPRINT1 omitted
    }

    STATUS_SUCCESS as ULONG
}

unsafe extern "system" fn SrvDeviceEvent(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyStatus: *mut ULONG,
) -> ULONG {
    STATUS_NOT_IMPLEMENTED as ULONG
}

unsafe extern "system" fn SrvLogon(
    ApiMessage: *mut _CSR_API_MESSAGE,
    ReplyStatus: *mut ULONG,
) -> ULONG {
    let LogonRequest = core::ptr::addr_of_mut!(
        (*(ApiMessage as *mut _USER_API_MESSAGE)).Data.LogonRequest
    );

    if CsrImpersonateClient(core::ptr::null_mut()) == 0 {
        return STATUS_UNSUCCESSFUL as ULONG;
    }

    GetTimeouts(core::ptr::addr_of_mut!(ShutdownSettings));

    CsrRevertToSelf();
    STATUS_SUCCESS as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn UserClientConnect(
    CsrProcess: PCSR_PROCESS,
    ConnectionInfo: PVOID,
    ConnectionInfoLength: *mut ULONG,
) -> NTSTATUS {
    let ConnectInfo = ConnectionInfo as *mut _USERSRV_API_CONNECTINFO;

    if CsrApiPort.is_null() {
        CsrApiPort = CsrQueryApiPort();

        let Status = NtUserSetInformationThread(
            NtCurrentThread(),
            UserThreadCsrApiPort as i32,
            core::ptr::addr_of!(CsrApiPort) as *const core::ffi::c_void,
            core::mem::size_of::<HANDLE>() as ULONG,
        );
        if Status < 0 {
            return Status;
        }
    }

    if ConnectionInfo.is_null() ||
       ConnectionInfoLength.is_null() ||
       *ConnectionInfoLength != core::mem::size_of::<_USERSRV_API_CONNECTINFO>() as ULONG {
        return STATUS_INVALID_PARAMETER;
    }

    (*ConnectInfo).dwDispatchCount = 0;
    let Status = NtUserProcessConnect(
        (*CsrProcess).ProcessHandle,
        ConnectInfo as *mut core::ffi::c_void,
        *ConnectionInfoLength,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn UserServerDllInitialization(
    LoadedServerDll: *mut _CSR_SERVER_DLL,
) -> NTSTATUS {
    UserServerHeap = RtlGetProcessHeap();

    (*LoadedServerDll).ApiBase = USERSRV_FIRST_API_NUMBER;
    (*LoadedServerDll).HighestApiSupported = UserpMaxApiNumber;
    (*LoadedServerDll).DispatchTable = UserServerApiDispatchTable.as_mut_ptr();
    (*LoadedServerDll).ValidTable = UserServerApiServerValidTable.as_mut_ptr();
    #[cfg(feature = "CSR_DBG")]
    {
        (*LoadedServerDll).NameTable = UserServerApiNameTable.as_mut_ptr() as *mut *mut i8;
    }
    (*LoadedServerDll).SizeOfProcessData = 0;
    (*LoadedServerDll).ConnectCallback = Some(UserClientConnect);
    (*LoadedServerDll).DisconnectCallback = None;
    (*LoadedServerDll).HardErrorCallback = Some(UserServerHardError);
    (*LoadedServerDll).ShutdownProcessCallback = Some(UserClientShutdown);

    UserServerDllInstance = (*LoadedServerDll).ServerHandle;

    let mut Status = NtCreateEvent(
        core::ptr::addr_of_mut!(ghPowerRequestEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent as i32,
        0,
    );
    if Status < 0 {
        return Status;
    }

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!(ghMediaRequestEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent as i32,
        0,
    );
    if Status < 0 {
        return Status;
    }

    BaseSetProcessCreateNotify(Some(NtUserNotifyProcessCreate));

    UserInitHardErrorsCache();

    Status = NtUserInitialize(
        USER_VERSION,
        ghPowerRequestEvent,
        ghMediaRequestEvent,
    );
    if Status < 0 {
        return Status;
    }

    STATUS_SUCCESS
}
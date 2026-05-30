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

// Shadow PCSR_API_ROUTINE locally with extern "system" — bindgen
// emits arch-specific ABI literals (extern "stdcall" on i386,
// extern "C" elsewhere) and Rust's type system treats those as
// distinct from extern "system" even when codegen is identical.
// `LoadedServerDll.DispatchTable` is `*mut c_void` so this pointer
// is cast at the assignment site. Mirrors basesrv/init.rs in
// smss-csrss-rust (memory feedback_extern_c_abi).
type CsrApiRoutineSystem = Option<unsafe extern "system" fn(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS>;

static mut UserServerApiDispatchTable: [CsrApiRoutineSystem; 10] = [
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
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let Status = CsrExecServerThread(CreateSystemThreads as PVOID, 0);
    if Status < 0 {
        // DPRINT1 omitted
    }
    Status
}

unsafe extern "system" fn SrvActivateDebugger(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn SrvGetThreadConsoleDesktop(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
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

    STATUS_SUCCESS
}

unsafe extern "system" fn SrvDeviceEvent(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn SrvLogon(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    if CsrImpersonateClient(core::ptr::null_mut()) == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    GetTimeouts(core::ptr::addr_of_mut!(ShutdownSettings));

    CsrRevertToSelf();
    STATUS_SUCCESS
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
            UserThreadCsrApiPort as u32,
            core::ptr::addr_of!(CsrApiPort) as *mut core::ffi::c_void,
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
        ConnectInfo,
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
    (*LoadedServerDll).DispatchTable = UserServerApiDispatchTable.as_mut_ptr() as *mut PCSR_API_ROUTINE;
    (*LoadedServerDll).ValidTable = UserServerApiServerValidTable.as_mut_ptr();
    #[cfg(feature = "CSR_DBG")]
    {
        (*LoadedServerDll).NameTable = UserServerApiNameTable.as_mut_ptr() as *mut *mut i8;
    }
    (*LoadedServerDll).SizeOfProcessData = 0;
    // ConnectCallback / HardErrorCallback / ShutdownProcessCallback are
    // bindgen-typed `extern "stdcall"`; our Rust impls use `extern "system"`.
    // On i386 these have identical codegen; transmute is sound. On
    // amd64/arm64 the calling-convention literal is irrelevant (single
    // ABI) and the layout matches. Mirror of basesrv/init.rs pattern.
    (*LoadedServerDll).ConnectCallback =
        Some(core::mem::transmute(UserClientConnect as *const ()));
    (*LoadedServerDll).DisconnectCallback = None;
    (*LoadedServerDll).HardErrorCallback =
        Some(core::mem::transmute(UserServerHardError as *const ()));
    (*LoadedServerDll).ShutdownProcessCallback =
        Some(core::mem::transmute(UserClientShutdown as *const ()));

    UserServerDllInstance = (*LoadedServerDll).ServerHandle as HINSTANCE;

    let mut Status = NtCreateEvent(
        core::ptr::addr_of_mut!(ghPowerRequestEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0,
    );
    if Status < 0 {
        return Status;
    }

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!(ghMediaRequestEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
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
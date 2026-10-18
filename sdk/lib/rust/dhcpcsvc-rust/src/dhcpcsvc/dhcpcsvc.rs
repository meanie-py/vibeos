use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- winsvc.h control code; the crate's substrate does not carry it, so this arm
// bound every control code the service handler was passed. ----
const SERVICE_CONTROL_INTERROGATE: u32 = 0x00000004;

// ---- file-scope preamble (translated) ----
pub static ServiceName: [u16; 5] = [0x44, 0x48, 0x43, 0x50, 0];
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
pub static mut ServiceStatus: SERVICE_STATUS = unsafe { core::mem::zeroed() };
pub static mut hStopEvent: HANDLE = core::ptr::null_mut();
pub static mut hAdapterStateChangedEvent: HANDLE = core::ptr::null_mut();
extern "C" {
    pub static mut DhcpSocket: SOCKET;
}

// ---- MIDL_user_allocate ----
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: usize) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), 0x00000008, len)
}

// ---- MIDL_user_free ----
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

// ---- PDHCP_SERVER_NAME_bind ----
#[no_mangle]
pub unsafe extern "system" fn PDHCP_SERVER_NAME_bind(
    pszServerName: *const u16,
) -> *mut core::ffi::c_void {
    let mut hBinding: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pszStringBinding: *mut u16 = core::ptr::null_mut();
    let status: i32;

    // DPRINT!("PDHCP_SERVER_NAME_bind() called\n");

    let ncacn_np: &[u16] = &[
        110, 99, 97, 99, 110, 95, 110, 112, 0,
    ];
    let pipe_dhcpcsvc: &[u16] = &[
        92, 112, 105, 112, 101, 92, 100, 104, 99, 112, 99, 115, 118, 99, 0,
    ];

    let status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        pszServerName as *mut u16,
        pipe_dhcpcsvc.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );

    if status != 0 {
        // DPRINT1!("RpcStringBindingCompose returned 0x%x\n", status);
        return core::ptr::null_mut();
    }

    let status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != 0 {
        // DPRINT1!("RpcBindingFromStringBinding returned 0x%x\n", status);
    }

    let status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {
        // DPRINT1!("RpcStringFree returned 0x%x\n", status);
    }

    hBinding
}

// ---- PDHCP_SERVER_NAME_unbind ----
#[no_mangle]
pub unsafe extern "system" fn PDHCP_SERVER_NAME_unbind(
    pszServerName: PDHCP_SERVER_NAME,
    mut hBinding: handle_t,
) {
    DbgPrint(b"PDHCP_SERVER_NAME_unbind() called\n\0".as_ptr() as *const i8);

    let status = RpcBindingFree(&mut hBinding as *mut handle_t);
    if status != 0 {
        DbgPrint(
            b"RpcBindingFree returned 0x%x\n\0".as_ptr() as *const i8,
            status,
        );
    }
}

// ---- DhcpCApiInitialize ----
#[no_mangle]
pub unsafe extern "system" fn DhcpCApiInitialize(Version: LPDWORD) -> DWORD {
    *Version = 2;
    ERROR_SUCCESS
}

// ---- DhcpCApiCleanup ----
#[no_mangle]
pub unsafe extern "system" fn DhcpCApiCleanup() {
    // (empty in the C too — dhcpcsvc.c:116)
}

// ---- DhcpAcquireParameters ----
#[no_mangle]
pub unsafe extern "system" fn DhcpAcquireParameters(AdapterName: PWSTR) -> DWORD {
    DbgPrint(b"DhcpAcquireParameters(%S)\n\0".as_ptr() as *const _, AdapterName);
    let ret = rpc_try(|| Client_AcquireParameters(core::ptr::null_mut(), AdapterName)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    ret
}

// ---- DhcpEnumClasses ----
#[no_mangle]
pub unsafe extern "system" fn DhcpEnumClasses(
    Unknown1: DWORD,
    AdapterName: PWSTR,
    Unknown3: DWORD,
    Unknown4: DWORD,
) -> DWORD {
    DbgPrint(
        (b"DhcpEnumClasses(%lx %S %lx %lx)\n\0".as_ptr()) as *const i8,
        Unknown1,
        AdapterName,
        Unknown3,
        Unknown4,
    );
    0
}

// ---- DhcpHandlePnPEvent ----
#[no_mangle]
pub unsafe extern "system" fn DhcpHandlePnPEvent(
    Unknown1: DWORD,
    Unknown2: DWORD,
    AdapterName: PWSTR,
    Unknown4: DWORD,
    Unknown5: DWORD,
) -> DWORD {
    DbgPrint(
        b"DhcpHandlePnPEvent(%lx %lx %S %lx %lx)\n\0".as_ptr() as *const i8,
        Unknown1,
        Unknown2,
        AdapterName,
        Unknown4,
        Unknown5,
    );
    0
}

// ---- DhcpNotifyConfigChange ----
#[no_mangle]
pub unsafe extern "system" fn DhcpNotifyConfigChange(
    ServerName: *const u16,
    AdapterName: *const u16,
    NewIpAddress: i32,
    IpIndex: u32,
    IpAddress: u32,
    SubnetMask: u32,
    DhcpAction: i32,
) -> u32 {
    DbgPrint(
        b"DHCPCSVC: DhcpNotifyConfigChange not implemented yet\n\0".as_ptr() as *const i8,
    );
    DbgPrint(
        b"DhcpNotifyConfigChange(%S %S %lu %lu %lu %lu %d)\n\0".as_ptr() as *const i8,
        ServerName,
        AdapterName,
        NewIpAddress,
        IpIndex,
        IpAddress,
        SubnetMask,
        DhcpAction,
    );

    if AdapterName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    // dhcpcsvc.c:227 `UNIMPLEMENTED;` — that macro is only __NOTICE(WARNING, ...), i.e. a
    // DbgPrint. It does NOT block: the C falls straight through to `return ERROR_SUCCESS`.
    // The translation rendered it as `loop {}`, which would have HUNG the calling thread
    // forever instead of returning.
    DbgPrint(
        b"DHCPCSVC: DhcpNotifyConfigChange is UNIMPLEMENTED!\n\0".as_ptr() as *const i8,
    );
    ERROR_SUCCESS
}

// ---- DhcpQueryHWInfo ----
#[no_mangle]
pub unsafe extern "system" fn DhcpQueryHWInfo(
    AdapterIndex: u32,
    MediaType: *mut u32,
    Mtu: *mut u32,
    Speed: *mut u32,
) -> u32 {
    let ret = Client_QueryHWInfo(core::ptr::null_mut(), AdapterIndex, MediaType, Mtu, Speed);
    if ret == ERROR_SUCCESS { 1 } else { 0 }
}

// ---- DhcpReleaseParameters ----
#[no_mangle]
pub unsafe extern "system" fn DhcpReleaseParameters(AdapterName: PWSTR) -> u32 {
    let ret = rpc_try(|| Client_ReleaseParameters(core::ptr::null_mut(), AdapterName)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    ret
}

// ---- DhcpRemoveDNSRegistrations ----
#[no_mangle]
pub unsafe extern "system" fn DhcpRemoveDNSRegistrations() -> DWORD
{
    DbgPrint(b"DhcpRemoveDNSRegistrations()\n\0".as_ptr() as PCHAR);
    let ret = rpc_try(|| Client_RemoveDNSRegistrations(core::ptr::null_mut())).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    ret
}

// ---- DhcpStaticRefreshParams ----
#[no_mangle]
pub unsafe extern "system" fn DhcpStaticRefreshParams(
    AdapterIndex: DWORD,
    Address: DWORD,
    Netmask: DWORD,
) -> DWORD {
    DbgPrint((b"DhcpStaticRefreshParams()\n\0".as_ptr() as *const u8) as *const i8);
    let ret = rpc_try(|| Client_StaticRefreshParams(core::ptr::null_mut(), AdapterIndex, Address, Netmask)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if ret == ERROR_SUCCESS { 1 } else { 0 }
}

// ---- DhcpRequestParams ----
#[no_mangle]
pub unsafe extern "system" fn DhcpRequestParams(
    Flags: u32,
    Reserved: *mut core::ffi::c_void,
    AdapterName: *mut u16,
    ClassId: *mut _DHCPCAPI_CLASSID,
    SendParams: DHCPCAPI_PARAMS_ARRAY,
    RecdParams: DHCPCAPI_PARAMS_ARRAY,
    Buffer: *mut u8,
    pSize: *mut u32,
    RequestIdStr: *mut u16,
) -> u32 {
    0
}

// ---- UpdateServiceStatus ----
unsafe fn UpdateServiceStatus(dwState: u32) {
    ServiceStatus.dwServiceType = SERVICE_WIN32_SHARE_PROCESS;
    ServiceStatus.dwCurrentState = dwState;

    if dwState == SERVICE_RUNNING {
        ServiceStatus.dwControlsAccepted = SERVICE_ACCEPT_SHUTDOWN | SERVICE_ACCEPT_STOP;
    } else {
        ServiceStatus.dwControlsAccepted = 0;
    }

    ServiceStatus.dwWin32ExitCode = 0;
    ServiceStatus.dwServiceSpecificExitCode = 0;
    ServiceStatus.dwCheckPoint = 0;

    if dwState == SERVICE_START_PENDING || dwState == SERVICE_STOP_PENDING {
        ServiceStatus.dwWaitHint = 1000;
    } else {
        ServiceStatus.dwWaitHint = 0;
    }

    SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
}

// ---- DllMain ----
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: u32,
    lpvReserved: *mut core::ffi::c_void,
) -> i32 {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls((hinstDLL) as *mut c_void);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    1
}

// ---- ServiceControlHandler ----
unsafe extern "system" fn ServiceControlHandler(dwControl: DWORD, dwEventType: DWORD, lpEventData: LPVOID, lpContext: LPVOID) -> DWORD {
    match dwControl {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            UpdateServiceStatus(SERVICE_STOP_PENDING);
            if !hStopEvent.is_null() {
                SetEvent(hStopEvent);
            }
            ERROR_SUCCESS
        }
        SERVICE_CONTROL_INTERROGATE => {
            SetServiceStatus(ServiceStatusHandle, core::ptr::addr_of_mut!(ServiceStatus));
            ERROR_SUCCESS
        }
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}

// ---- ServiceMain ----
#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: u32, argv: *mut *mut u16) {
    let mut hPipeThread: HANDLE = INVALID_HANDLE_VALUE;
    let mut hDiscoveryThread: HANDLE = INVALID_HANDLE_VALUE;
    let mut ret: u32;

    ServiceStatusHandle = RegisterServiceCtrlHandlerExW(
        ServiceName.as_ptr(),
        Some(ServiceControlHandler),
        core::ptr::null_mut(),
    );
    if ServiceStatusHandle == core::ptr::null_mut() {
        DbgPrint(
            b"DHCPCSVC: Unable to register service control handler (%lx)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    hStopEvent = CreateEventW(core::ptr::null_mut(), 1i32, 0i32, core::ptr::null_mut());
    if hStopEvent == core::ptr::null_mut() {
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    hAdapterStateChangedEvent = CreateEvent(
        core::ptr::null_mut(),
        0i32,
        0i32,
        core::ptr::null_mut(),
    );
    if hAdapterStateChangedEvent == core::ptr::null_mut() {
        CloseHandle(hStopEvent);
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    if init_client() == 0 {
        DbgPrint(
            b"DHCPCSVC: init_client() failed!\n\0".as_ptr() as *const i8,
        );
        CloseHandle(hAdapterStateChangedEvent);
        CloseHandle(hStopEvent);
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    UpdateServiceStatus(SERVICE_START_PENDING);

    hPipeThread = InitRpc();
    if hPipeThread == INVALID_HANDLE_VALUE {
        DbgPrint(
            b"DHCPCSVC: PipeInit() failed!\n\0".as_ptr() as *const i8,
        );
        stop_client();
        CloseHandle(hAdapterStateChangedEvent);
        CloseHandle(hStopEvent);
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    hDiscoveryThread = StartAdapterDiscovery(hStopEvent);
    if hDiscoveryThread == INVALID_HANDLE_VALUE {
        DbgPrint(
            b"DHCPCSVC: StartAdapterDiscovery() failed!\n\0".as_ptr() as *const i8,
        );
        ShutdownRpc();
        stop_client();
        CloseHandle(hAdapterStateChangedEvent);
        CloseHandle(hStopEvent);
        UpdateServiceStatus(SERVICE_STOPPED);
        return;
    }

    DH_DbgPrint!(
        MID_TRACE,
        b"DHCP Service Started\n\0".as_ptr() as *const i8,
    );

    UpdateServiceStatus(SERVICE_RUNNING);

    DH_DbgPrint!(
        MID_TRACE,
        b"Going into dispatch()\n\0".as_ptr() as *const i8,
    );
    DH_DbgPrint!(
        MID_TRACE,
        b"DHCPCSVC: DHCP service is starting up\n\0".as_ptr() as *const i8,
    );

    dispatch(hStopEvent);

    DH_DbgPrint!(
        MID_TRACE,
        b"DHCPCSVC: DHCP service is shutting down\n\0".as_ptr() as *const i8,
    );

    ShutdownRpc();
    stop_client();

    DbgPrint(
        b"Wait for pipe thread to close! %p\n\0".as_ptr() as *const i8,
        hPipeThread,
    );
    if hPipeThread != INVALID_HANDLE_VALUE {
        DbgPrint(
            b"Waiting for pipe thread\n\0".as_ptr() as *const i8,
        );
        ret = WaitForSingleObject(hPipeThread, 5000);
        DbgPrint(
            b"Done %lx\n\0".as_ptr() as *const i8,
            ret,
        );
    }

    DbgPrint(
        b"Wait for discovery thread to close! %p\n\0".as_ptr() as *const i8,
        hDiscoveryThread,
    );
    if hDiscoveryThread != INVALID_HANDLE_VALUE {
        DbgPrint(
            b"Waiting for discovery thread\n\0".as_ptr() as *const i8,
        );
        ret = WaitForSingleObject(hDiscoveryThread, 5000);
        DbgPrint(
            b"Done %lx\n\0".as_ptr() as *const i8,
            ret,
        );
    }

    DbgPrint(
        b"Closing events!\n\0".as_ptr() as *const i8,
    );
    CloseHandle(hAdapterStateChangedEvent);
    CloseHandle(hStopEvent);

    if DhcpSocket != INVALID_SOCKET {
        closesocket(DhcpSocket);
    }

    CloseHandle(hDiscoveryThread);
    CloseHandle(hPipeThread);

    DbgPrint(
        b"Done!\n\0".as_ptr() as *const i8,
    );

    UpdateServiceStatus(SERVICE_STOPPED);
}

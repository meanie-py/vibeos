use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- winsvc.h control code; the crate's substrate does not carry it, so this arm
// bound every control code the service handler was passed. ----
const SERVICE_CONTROL_INTERROGATE: u32 = 0x00000004;
// winnls.h; SetBinaryClassId's WideCharToMultiByte. Not in the crate's island.
const CP_ACP: UINT = 0;

// ---- file-scope preamble (translated) ----
pub static ServiceName: [u16; 5] = [0x44, 0x48, 0x43, 0x50, 0];
pub static mut ServiceStatusHandle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
pub static mut ServiceStatus: SERVICE_STATUS = unsafe { core::mem::zeroed() };
pub static mut hStopEvent: HANDLE = core::ptr::null_mut();
// The C declares this `extern`; one definition lives in dhcp/adapter.rs (scan_extern_copy).
pub use crate::dhcp::adapter::hAdapterStateChangedEvent;
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

// ---- SetBinaryClassId ----
// DhcpHandlePnPEvent's class-id half (upstream 1f8f73d3b2e): copy the adapter's
// DhcpClassId (REG_SZ) into DhcpClassIdBin as its ANSI bytes.
//
// Divergences from the C: the C `_swprintf`s the key name into a 256-WCHAR buffer with no
// bound, so a long adapter name overflows the stack; here an over-long name is
// ERROR_INVALID_PARAMETER. And it hands WideCharToMultiByte(-1) a registry string that need
// not be NUL-terminated; the buffer here carries one extra, zeroed WCHAR.
unsafe fn SetBinaryClassId(pszAdapterName: PWSTR) -> DWORD {
    const PREFIX: &str = "System\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\";
    let mut szKeyName = [0u16; 256];
    let mut n = 0usize;
    for c in PREFIX.encode_utf16() {
        szKeyName[n] = c;
        n += 1;
    }
    let mut p = pszAdapterName;
    while *p != 0 {
        if n + 1 >= szKeyName.len() {
            DPRINT1!("Error %lu\n", ERROR_INVALID_PARAMETER);
            return ERROR_INVALID_PARAMETER;
        }
        szKeyName[n] = *p;
        n += 1;
        p = p.add(1);
    }

    DPRINT!("SetBinaryClassId(%S)\n", pszAdapterName);

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyName.as_ptr(),
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hKey,
    );
    if dwError != ERROR_SUCCESS {
        DPRINT1!("Error %lu\n", dwError);
        return dwError;
    }

    let value: [u16; 12] = [
        'D' as u16, 'h' as u16, 'c' as u16, 'p' as u16, 'C' as u16, 'l' as u16, 'a' as u16,
        's' as u16, 's' as u16, 'I' as u16, 'd' as u16, 0,
    ];
    let value_bin: [u16; 15] = [
        'D' as u16, 'h' as u16, 'c' as u16, 'p' as u16, 'C' as u16, 'l' as u16, 'a' as u16,
        's' as u16, 's' as u16, 'I' as u16, 'd' as u16, 'B' as u16, 'i' as u16, 'n' as u16, 0,
    ];

    let mut pszUnicodeBuffer: *mut u16 = core::ptr::null_mut();
    let mut pszAnsiBuffer: *mut c_char = core::ptr::null_mut();
    let mut dwUnicodeSize: DWORD = 0;

    'done: {
        RegQueryValueExW(hKey, value.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), &mut dwUnicodeSize);
        DPRINT!("UnicodeSize %lu\n", dwUnicodeSize);

        // HEAP_ZERO_MEMORY, plus one WCHAR: the terminator the registry need not have stored.
        pszUnicodeBuffer = HeapAlloc(GetProcessHeap(), 0x00000008, dwUnicodeSize as usize + 2) as *mut u16;
        if pszUnicodeBuffer.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            DPRINT1!("Error %lu\n", dwError);
            break 'done;
        }

        dwError = RegQueryValueExW(
            hKey,
            value.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pszUnicodeBuffer as *mut BYTE,
            &mut dwUnicodeSize,
        );
        if dwError != ERROR_SUCCESS {
            DPRINT1!("Error %lu\n", dwError);
            break 'done;
        }

        let dwAnsiSize = WideCharToMultiByte(CP_ACP, 0, pszUnicodeBuffer, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
        DPRINT!("AnsiSize %lu\n", dwAnsiSize);

        pszAnsiBuffer = HeapAlloc(GetProcessHeap(), 0, dwAnsiSize as usize) as *mut c_char;
        if pszAnsiBuffer.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            DPRINT1!("Error %lu\n", dwError);
            break 'done;
        }

        WideCharToMultiByte(CP_ACP, 0, pszUnicodeBuffer, -1, pszAnsiBuffer, dwAnsiSize, core::ptr::null(), core::ptr::null_mut());
        DPRINT!("AnsiBuffer %s\n", pszAnsiBuffer);

        dwError = RegSetValueExW(
            hKey,
            value_bin.as_ptr(),
            0,
            REG_BINARY,
            pszAnsiBuffer as *const BYTE,
            strlen(pszAnsiBuffer) as DWORD,
        );
        if dwError != ERROR_SUCCESS {
            DPRINT1!("Error %lu\n", dwError);
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }
    if !pszAnsiBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszAnsiBuffer as *mut c_void);
    }
    if !pszUnicodeBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pszUnicodeBuffer as *mut c_void);
    }

    dwError
}

// ---- FindParam ----
unsafe fn FindParam(ParamsArray: *mut DHCPCAPI_PARAMS_ARRAY, OptionId: ULONG, IsVendor: BOOL) -> PDHCPAPI_PARAMS {
    for i in 0..(*ParamsArray).nParams as usize {
        let p = (*ParamsArray).Params.add(i);
        if (*p).OptionId == OptionId && (*p).IsVendor == IsVendor {
            return p;
        }
    }
    core::ptr::null_mut()
}

// ---- MapRequestResultData ----
// Lays the service's results out in the caller's Buffer and points each matching RecdParams
// entry at its bytes. Divergence from the C: a result whose offset and size fall outside the
// returned data is skipped rather than copied from beyond it.
unsafe fn MapRequestResultData(
    Results: LPDHCPCAPI_RESULT_ARRAY,
    RecdParams: *mut DHCPCAPI_PARAMS_ARRAY,
    Buffer: LPBYTE,
    pSize: LPDWORD,
) -> DWORD {
    if (*Results).DataSize > *pSize {
        *pSize = (*Results).DataSize;
        return ERROR_MORE_DATA;
    }

    let mut Ptr = Buffer;
    for i in 0..(*Results).ResultsCount as usize {
        let r = &*(*Results).Results.add(i);
        let end = (r.DataOffset as u64) + (r.DataSize as u64);
        if end > (*Results).DataSize as u64 {
            continue;
        }
        let Param = FindParam(RecdParams, r.OptionId, r.IsVendor);
        if !Param.is_null() {
            core::ptr::copy_nonoverlapping((*Results).Data.add(r.DataOffset as usize), Ptr, r.DataSize as usize);
            (*Param).nBytesData = r.DataSize;
            (*Param).Data = Ptr;
            Ptr = Ptr.add(r.DataSize as usize);
        }
    }

    *pSize = (*Results).DataSize;

    ERROR_SUCCESS
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

// ---- DhcpAcquireParametersByBroadcast ----
#[no_mangle]
pub unsafe extern "system" fn DhcpAcquireParametersByBroadcast(AdapterName: PWSTR) -> DWORD {
    DPRINT!("DhcpAcquireParametersByBroadcast(%S)\n", AdapterName);
    rpc_try(|| Client_AcquireParametersByBroadcast(core::ptr::null_mut(), AdapterName))
        .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- DhcpDeRegisterParamChange ----
#[no_mangle]
pub unsafe extern "system" fn DhcpDeRegisterParamChange(Flags: DWORD, Reserved: LPVOID, Event: LPVOID) -> DWORD {
    DPRINT1!("DhcpDeRegisterParamChange is UNIMPLEMENTED!\n");
    0
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

// ---- DhcpFallbackRefreshParams ----
#[no_mangle]
pub unsafe extern "system" fn DhcpFallbackRefreshParams(AdapterName: PWSTR) -> DWORD {
    DPRINT!("DhcpFallbackRefreshParams(%S)\n", AdapterName);
    rpc_try(|| Client_FallbackRefreshParams(core::ptr::null_mut(), AdapterName))
        .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- DhcpHandlePnPEvent ----
#[no_mangle]
pub unsafe extern "system" fn DhcpHandlePnPEvent(
    Unknown1: DWORD,
    Unknown2: DWORD,
    AdapterName: PWSTR,
    PnpEvent: PDHCP_PNP_EVENT,
    Unknown5: DWORD,
) -> DWORD {
    DPRINT1!(
        "DhcpHandlePnPEvent(%lx %lx %S %p %lx)\n",
        Unknown1,
        Unknown2,
        AdapterName,
        PnpEvent,
        Unknown5,
    );

    if Unknown1 != 0 || Unknown2 != 1 || PnpEvent.is_null() || Unknown5 != 0 {
        return ERROR_INVALID_PARAMETER;
    }

    if (*PnpEvent).Unknown5 != 0 {
        return SetBinaryClassId(AdapterName);
    }

    ERROR_SUCCESS
}

// ---- DhcpNotifyConfigChange ----
#[no_mangle]
pub unsafe extern "system" fn DhcpNotifyConfigChange(
    ServerName: LPWSTR,
    AdapterName: LPWSTR,
    NewIpAddress: BOOL,
    IpIndex: DWORD,
    IpAddress: DWORD,
    SubnetMask: DWORD,
    DhcpAction: INT,
) -> DWORD {
    DPRINT!(
        "DhcpNotifyConfigChange(%S %S %lu %lu %lu %lu %d)\n",
        ServerName,
        AdapterName,
        NewIpAddress,
        IpIndex,
        IpAddress,
        SubnetMask,
        DhcpAction,
    );

    DhcpNotifyConfigChangeEx(ServerName, AdapterName, NewIpAddress, IpIndex, IpAddress, SubnetMask, DhcpAction, 0)
}

// ---- DhcpNotifyConfigChangeEx ----
// Upstream ed5124b7a86: actions 1/2 enable/disable DHCP on the adapter, 0 changes nothing.
//
// Divergence from the C: the C calls `Server_EnableDhcp` -- the service's own handler --
// directly from this client DLL. That only works inside the DHCP service's process; anywhere
// else (netcfgx, ipconfig) it runs against the caller's empty adapter list and uninitialised
// API lock instead of reaching the service. This goes through the RPC client stub, as every
// other Dhcp* entry point in this file does.
#[no_mangle]
pub unsafe extern "system" fn DhcpNotifyConfigChangeEx(
    ServerName: LPWSTR,
    AdapterName: LPWSTR,
    NewIpAddress: BOOL,
    IpIndex: DWORD,
    IpAddress: DWORD,
    SubnetMask: DWORD,
    DhcpAction: INT,
    Unknown8: DWORD,
) -> DWORD {
    DPRINT1!("DHCPCSVC: DhcpNotifyConfigChangeEx not implemented yet\n");
    DPRINT1!(
        "DhcpNotifyConfigChangeEx(%S %S %lu %lu %lu %lu %d %lu)\n",
        ServerName,
        AdapterName,
        NewIpAddress,
        IpIndex,
        IpAddress,
        SubnetMask,
        DhcpAction,
        Unknown8,
    );

    match DhcpAction {
        // Enable DHCP
        1 => {
            if NewIpAddress != 0 || IpIndex != 0 || IpAddress != 0 || SubnetMask != 0 {
                return ERROR_INVALID_PARAMETER;
            }
            /* TODO: Remove static IP address(es) */
            rpc_try(|| Client_EnableDhcp(ServerName, AdapterName, TRUE))
                .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
        }
        // Disable DHCP
        2 => {
            if NewIpAddress == 0 || IpIndex != 0 || IpAddress == 0 || SubnetMask == 0 {
                return ERROR_INVALID_PARAMETER;
            }
            rpc_try(|| Client_EnableDhcp(ServerName, AdapterName, FALSE))
                .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
        }
        // Do not modify
        0 => ERROR_SUCCESS,
        _ => ERROR_INVALID_PARAMETER,
    }
}

// ---- DhcpRegisterParamChange ----
#[no_mangle]
pub unsafe extern "system" fn DhcpRegisterParamChange(
    Flags: DWORD,
    Reserved: LPVOID,
    AdapterName: LPWSTR,
    ClassId: LPDHCPCAPI_CLASSID,
    Params: DHCPCAPI_PARAMS_ARRAY,
    Handle: LPVOID,
) -> DWORD {
    DPRINT1!("DhcpRegisterParamChange(%lx %p %S)\n", Flags, Reserved, AdapterName);

    if Flags != DHCPCAPI_REGISTER_HANDLE_EVENT {
        return ERROR_INVALID_PARAMETER;
    }
    if !Reserved.is_null() || AdapterName.is_null() || Handle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    DPRINT1!("DhcpRegisterParamChange is UNIMPLEMENTED!\n");
    0
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
// Upstream c00b0a40358 / 1d6dae3bb24: ask the service for options from the adapter's active
// lease and lay them out in Buffer, pointing each RecdParams entry at its data.
#[no_mangle]
pub unsafe extern "system" fn DhcpRequestParams(
    Flags: DWORD,
    Reserved: PVOID,
    AdapterName: LPWSTR,
    ClassId: LPDHCPCAPI_CLASSID,
    mut SendParams: DHCPCAPI_PARAMS_ARRAY,
    mut RecdParams: DHCPCAPI_PARAMS_ARRAY,
    Buffer: LPBYTE,
    pSize: LPDWORD,
    RequestIdStr: LPWSTR,
) -> DWORD {
    let mut Results: LPDHCPCAPI_RESULT_ARRAY = core::ptr::null_mut();
    let mut ret: DWORD = ERROR_SUCCESS;

    DPRINT!(
        "DhcpRequestParams(%lx %p %S %p %lu %p %lu %p %p %p %S)\n",
        Flags,
        Reserved,
        AdapterName,
        ClassId,
        SendParams.nParams,
        SendParams.Params,
        RecdParams.nParams,
        RecdParams.Params,
        Buffer,
        pSize,
        RequestIdStr,
    );

    if Flags != DHCPCAPI_REQUEST_SYNCHRONOUS
        && Flags != DHCPCAPI_REQUEST_PERSISTENT
        && Flags != (DHCPCAPI_REQUEST_SYNCHRONOUS | DHCPCAPI_REQUEST_PERSISTENT)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if !Reserved.is_null() || AdapterName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if !ClassId.is_null() {
        if (*ClassId).Flags != 0 {
            return ERROR_INVALID_PARAMETER;
        }
        if (*ClassId).Data.is_null() || (*ClassId).nBytesData == 0 {
            return ERROR_INVALID_PARAMETER;
        }
    }

    if SendParams.nParams != 0 && SendParams.Params.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if RecdParams.Params.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if Flags & DHCPCAPI_REQUEST_SYNCHRONOUS != 0 {
        let (send, recd, results) = (&mut SendParams as *mut _, &mut RecdParams as *mut _, &mut Results as *mut _);
        ret = rpc_try(|| Client_RequestParams(core::ptr::null_mut(), AdapterName, ClassId, send, recd, results))
            .unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    }

    if !Results.is_null() {
        DPRINT!("Results: Count %lu  DataSize %lu\n", (*Results).ResultsCount, (*Results).DataSize);

        ret = MapRequestResultData(Results, &mut RecdParams, Buffer, pSize);

        if !(*Results).Results.is_null() {
            MIDL_user_free((*Results).Results as *mut c_void);
        }
        if !(*Results).Data.is_null() {
            MIDL_user_free((*Results).Data as *mut c_void);
        }
        MIDL_user_free(Results as *mut c_void);
    }

    if ret != ERROR_SUCCESS {
        return ret;
    }

    if Flags & DHCPCAPI_REQUEST_PERSISTENT != 0 {
        /* FIXME: Store the request */
        DPRINT1!("DHCPCAPI_REQUEST_PERSISTENT is not supported yet!\n");
    }

    ret
}

// ---- DhcpUndoRequestParams ----
#[no_mangle]
pub unsafe extern "system" fn DhcpUndoRequestParams(
    Flags: DWORD,
    Reserved: LPVOID,
    AdapterName: LPWSTR,
    RequestIdStr: LPWSTR,
) -> DWORD {
    DPRINT1!("DhcpUndoRequestParams(%lx %p %S %S)\n", Flags, Reserved, AdapterName, RequestIdStr);

    if Flags != 0 || !Reserved.is_null() || RequestIdStr.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    DPRINT1!("DhcpUndoRequestParams is UNIMPLEMENTED!\n");
    STATUS_SUCCESS as DWORD
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

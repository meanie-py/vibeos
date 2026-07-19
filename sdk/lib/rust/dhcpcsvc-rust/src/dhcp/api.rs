use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
static mut ApiCriticalSection: CRITICAL_SECTION = unsafe { core::mem::zeroed() };

extern "C" {
    pub static mut hAdapterStateChangedEvent: HANDLE;
}

// ---- ApiInit ----
#[no_mangle]
pub unsafe extern "system" fn ApiInit() {
    InitializeCriticalSection(&mut ApiCriticalSection);
}

// ---- ApiLock ----
#[no_mangle]
pub unsafe extern "system" fn ApiLock() {
    EnterCriticalSection(core::ptr::addr_of_mut!(ApiCriticalSection));
}

// ---- ApiUnlock ----
pub unsafe fn ApiUnlock() {
    LeaveCriticalSection(core::ptr::addr_of_mut!(ApiCriticalSection));
}

// ---- ApiFree ----
unsafe fn ApiFree() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(ApiCriticalSection));
}

// ---- RpcThreadRoutine ----
#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: *mut core::ffi::c_void) -> u32 {
    let protseq: [u16; 9] = [
        0x006e, 0x0063, 0x0061, 0x0063, 0x006e, 0x005f, 0x006e, 0x0070, 0x0000,
    ];
    let pipe: [u16; 15] = [
        0x005c, 0x0070, 0x0069, 0x0070, 0x0065, 0x005c, 0x0064, 0x0068, 0x0063,
        0x0070, 0x0063, 0x0073, 0x0076, 0x0063, 0x0000,
    ];

    let mut Status: u32;

    Status = (RpcServerUseProtseqEpW(protseq.as_ptr() as *mut u16, 20, pipe.as_ptr() as *mut u16, core::ptr::null_mut())) as u32;
    if Status != (RPC_S_OK) as u32 {
        // DPRINT1!("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
        return 0;
    }

    Status = RpcServerRegisterIf(
        Server_dhcpcsvc_v0_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status != (RPC_S_OK) as u32 {
        // DPRINT1!("RpcServerRegisterIf() failed (Status %lx)\n", Status);
        return 0;
    }

    Status = (RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, 0)) as u32;
    if Status != (RPC_S_OK) as u32 {
        // DPRINT1!("RpcServerListen() failed (Status %lx)\n", Status);
    }

    0
}

// ---- InitRpc ----
#[no_mangle]
pub unsafe extern "system" fn InitRpc() -> HANDLE {
    CreateThread(
        core::ptr::null_mut(),
        0,
        Some(RpcThreadRoutine),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    )
}

// ---- ShutdownRpc ----
pub unsafe fn ShutdownRpc() {
    RpcMgmtStopServerListening(core::ptr::null_mut());
}

// ---- Server_RemoveDNSRegistrations ----
#[no_mangle]
pub unsafe extern "system" fn Server_RemoveDNSRegistrations(
    ServerName: PDHCP_SERVER_NAME,
) -> u32 {
    // DPRINT1!("Server_RemoveDNSRegistrations()\n");
    /* FIXME: Call dnsapi.DnsRemoveRegistrations() */
    return ERROR_SUCCESS;
}

// ---- Server_AcquireParameters ----
#[no_mangle]
pub unsafe extern "system" fn Server_AcquireParameters(
    ServerName: PDHCP_SERVER_NAME,
    AdapterName: LPWSTR,
) -> DWORD {
    let Adapter: PDHCP_ADAPTER;
    let proto: *mut protocol;
    let mut ret: DWORD = ERROR_SUCCESS;

    ApiLock();

    'done: {
    Adapter = AdapterFindName(AdapterName);
    if Adapter.is_null() || (*Adapter).DhclientState.state == S_STATIC {
        ret = ERROR_FILE_NOT_FOUND;
        break 'done;
    }

    proto = find_protocol_by_adapter(&mut (*Adapter).DhclientInfo);
    if !proto.is_null() {
        remove_protocol(proto);
    }

    add_protocol(
        (*Adapter).DhclientInfo.name.as_mut_ptr(),
        (*Adapter).DhclientInfo.rfdesc,
        got_one,
        core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void,
    );

    (*(*Adapter).DhclientInfo.client).state = S_INIT;
    state_reboot(core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void);

    if !hAdapterStateChangedEvent.is_null() {
        SetEvent(hAdapterStateChangedEvent);
    }

    }
    // C `done:` handler (fall-through target of `goto done`)
    ApiUnlock();

    ret
}

// ---- Server_ReleaseParameters ----
#[no_mangle]
pub unsafe extern "system" fn Server_ReleaseParameters(
    ServerName: PDHCP_SERVER_NAME,
    AdapterName: LPWSTR,
) -> u32 {
    let Adapter: PDHCP_ADAPTER;
    let proto: *mut protocol;
    let mut ret: u32 = ERROR_SUCCESS;

    // DPRINT!("Server_ReleaseParameters()\n");

    ApiLock();

    Adapter = AdapterFindName(AdapterName);
    if Adapter.is_null() {
        ret = ERROR_FILE_NOT_FOUND;
        // goto done
    } else {
        // DPRINT!("Adapter: %p\n", Adapter);

        state_release((core::ptr::addr_of_mut!((*Adapter).DhclientInfo)) as *mut c_void);

        proto = find_protocol_by_adapter(core::ptr::addr_of_mut!((*Adapter).DhclientInfo));
        if !proto.is_null() {
            remove_protocol(proto);
        }

        if !hAdapterStateChangedEvent.is_null() {
            SetEvent(hAdapterStateChangedEvent);
        }
    }

    ApiUnlock();

    ret
}

// ---- Server_QueryHWInfo ----
#[no_mangle]
pub unsafe extern "system" fn Server_QueryHWInfo(
    ServerName: PDHCP_SERVER_NAME,
    AdapterIndex: DWORD,
    MediaType: PDWORD,
    Mtu: PDWORD,
    Speed: PDWORD,
) -> DWORD {
    let mut ret: DWORD = ERROR_SUCCESS;

    ApiLock();

    let adapter = AdapterFindIndex(AdapterIndex);
    if adapter.is_null() {
        ret = ERROR_FILE_NOT_FOUND;
    } else {
        *MediaType = (*adapter).IfMib.dwType;
        *Mtu = (*adapter).IfMib.dwMtu;
        *Speed = (*adapter).IfMib.dwSpeed;
    }

    ApiUnlock();

    ret
}

// ---- Server_StaticRefreshParams ----
#[no_mangle]
pub unsafe extern "system" fn Server_StaticRefreshParams(
    ServerName: PDHCP_SERVER_NAME,
    AdapterIndex: DWORD,
    Address: DWORD,
    Netmask: DWORD,
) -> DWORD {
    let mut ret = ERROR_SUCCESS;

    ApiLock();

    let Adapter = AdapterFindIndex(AdapterIndex);
    if Adapter.is_null() {
        ret = ERROR_FILE_NOT_FOUND;
    } else {
        if (*Adapter).NteContext != 0 {
            DeleteIPAddress((*Adapter).NteContext);
            (*Adapter).NteContext = 0;
        }
        if (*Adapter).RouterMib.dwForwardNextHop != 0 {
            DeleteIpForwardEntry(addr_of_mut!((*Adapter).RouterMib));
            (*Adapter).RouterMib.dwForwardNextHop = 0;
        }

        (*Adapter).DhclientState.state = _DHCPState_S_STATIC;
        let proto = find_protocol_by_adapter(addr_of_mut!((*Adapter).DhclientInfo));
        if !proto.is_null() {
            remove_protocol(proto);
        }

        ret = AddIPAddress(
            Address,
            Netmask,
            AdapterIndex,
            addr_of_mut!((*Adapter).NteContext),
            addr_of_mut!((*Adapter).NteInstance),
        );

        if !hAdapterStateChangedEvent.is_null() {
            SetEvent(hAdapterStateChangedEvent);
        }
    }

    ApiUnlock();

    ret
}

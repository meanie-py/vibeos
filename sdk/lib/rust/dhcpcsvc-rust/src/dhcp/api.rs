use crate::prelude::*;
use core::ffi::c_void;
use crate::dhcp::adapter::{FindAdapterKey, LoadAlternateConfiguration};
use crate::dhcpcsvc::dhcpcsvc::{MIDL_user_allocate, MIDL_user_free};

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

// ---- GetVendorOption ----
// Finds `OptionId` in a DHCP vendor-encapsulated option blob (option 43): a run of
// [id][len][len bytes] records. Returns the record (its id byte), or null.
//
// Divergence from the C: the C reads each record's length byte and returns a record without
// checking either against `Length`, so a short or lying blob from the DHCP server walks off
// the end of the lease's option buffer, and a found record can claim more data than the blob
// holds. Here a record is only examined, or returned, when it lies entirely inside the blob.
unsafe fn GetVendorOption(Data: PBYTE, Length: ULONG, OptionId: ULONG) -> PBYTE {
    let mut Offset: DWORD = 0;
    while Offset.saturating_add(2) <= Length {
        let Ptr = Data.add(Offset as usize);
        let OptionLength = *Ptr.add(1) as DWORD;
        if Offset + 2 + OptionLength > Length {
            break;
        }
        if *Ptr as DWORD == OptionId {
            return Ptr;
        }
        Offset += OptionLength + 2;
    }
    core::ptr::null_mut()
}

// ---- Server_EnableDhcp ----
// Function 0 (upstream ed5124b7a86): turn DHCP on or off for one adapter, the server half of
// DhcpNotifyConfigChangeEx's enable/disable actions.
#[no_mangle]
pub unsafe extern "system" fn Server_EnableDhcp(
    ServerName: PDHCP_SERVER_NAME,
    AdapterName: LPWSTR,
    Enable: BOOL,
) -> DWORD {
    let mut ret: DWORD = ERROR_SUCCESS;

    DPRINT1!("Server_EnableDhcp(%S %u)\n", AdapterName, Enable);

    ApiLock();

    'done: {
        let Adapter = AdapterFindName(AdapterName);
        if Adapter.is_null() {
            ret = ERROR_FILE_NOT_FOUND;
            break 'done;
        }

        DPRINT1!("Adapter: %p\n", Adapter);

        if Enable != 0 {
            DPRINT1!("Enable DHCP for Adapter: %p\n", Adapter);

            if (*Adapter).DhclientState.state != S_STATIC {
                DPRINT1!("The Adapter is already enabled!\n");
                break 'done;
            }

            add_protocol(
                (*Adapter).DhclientInfo.name.as_mut_ptr(),
                (*Adapter).DhclientInfo.rfdesc,
                got_one,
                core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void,
            );

            (*(*Adapter).DhclientInfo.client).state = S_INIT;
            state_reboot(core::ptr::addr_of_mut!((*Adapter).DhclientInfo) as *mut c_void);
        } else {
            DPRINT1!("Disable DHCP for Adapter: %p\n", Adapter);

            if (*Adapter).DhclientState.state == S_STATIC {
                DPRINT1!("The Adapter is already disabled!\n");
                break 'done;
            }

            (*Adapter).DhclientState.state = S_STATIC;
            let proto = find_protocol_by_adapter(core::ptr::addr_of_mut!((*Adapter).DhclientInfo));
            if !proto.is_null() {
                remove_protocol(proto);
            }
        }

        if !hAdapterStateChangedEvent.is_null() {
            SetEvent(hAdapterStateChangedEvent);
        }
    }

    ApiUnlock();

    ret
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

    DPRINT!("Server_AcquireParameters(%S)\n", AdapterName);

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

// ---- Server_AcquireParametersByBroadcast ----
// Function 2 (upstream cbfc755c2f8): a stub upstream too.
#[no_mangle]
pub unsafe extern "system" fn Server_AcquireParametersByBroadcast(
    ServerName: PDHCP_SERVER_NAME,
    AdapterName: LPWSTR,
) -> DWORD {
    DPRINT1!("Server_AcquireParametersByBroadcast(%S) is unimplemented!\n", AdapterName);
    ERROR_SUCCESS
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

    DPRINT!("Server_ReleaseParameters(%S)\n", AdapterName);

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

// ---- Server_FallbackRefreshParams ----
// Function 4 (upstream 4b9adaa971f / 10092f84660): reload the adapter's alternate
// configuration from the registry, replacing the one it had. Replaces QueryHWInfo, which
// upstream removed from the interface (cbfc755c2f8).
#[no_mangle]
pub unsafe extern "system" fn Server_FallbackRefreshParams(
    ServerName: PDHCP_SERVER_NAME,
    AdapterName: LPWSTR,
) -> DWORD {
    let mut ret: DWORD = ERROR_SUCCESS;

    DPRINT!("Server_FallbackRefreshParams(%S)\n", AdapterName);

    ApiLock();

    let Adapter = AdapterFindName(AdapterName);
    if Adapter.is_null() {
        ret = ERROR_FILE_NOT_FOUND;
    } else {
        if !(*Adapter).AlternateConfiguration.is_null() {
            free((*Adapter).AlternateConfiguration as *mut c_void);
            (*Adapter).AlternateConfiguration = core::ptr::null_mut();
        }

        let hAdapterKey = FindAdapterKey(Adapter);
        if !hAdapterKey.is_null() {
            ret = LoadAlternateConfiguration(Adapter, hAdapterKey);
            RegCloseKey(hAdapterKey);
        }
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

// ---- RequestedOption ----
// The lease data one RequestParams entry asks for: a standard option from the lease's option
// table, or a sub-option of the vendor-encapsulated option (43). (data, len), or None when the
// lease does not carry it. Both passes of Server_RequestParams resolve through this, so the
// sizing pass and the copy pass cannot disagree.
//
// Divergence from the C: the C indexes `Options[OptionId]` with the caller's OptionId as is.
// The table has 256 entries and the id comes off the RPC wire, so any local caller could make
// the service read past it. An id outside the table is simply not present.
unsafe fn RequestedOption(Options: *mut option_data, Param: *const DHCPAPI_PARAMS) -> Option<(PBYTE, DWORD)> {
    let OptionId = (*Param).OptionId;
    if (*Param).IsVendor == 0 {
        if OptionId >= 256 {
            return None;
        }
        let opt = &*Options.add(OptionId as usize);
        DPRINT!("Option %u: Length %d \n", OptionId, opt.len);
        if opt.len > 0 {
            return Some((opt.data, opt.len as DWORD));
        }
        return None;
    }

    let vendor = &*Options.add(DHO_VENDOR_ENCAPSULATED_OPTIONS as usize);
    if vendor.len <= 0 {
        return None;
    }
    DPRINT!("Have vendor options!\n");
    let OptionPtr = GetVendorOption(vendor.data, vendor.len as ULONG, OptionId);
    if OptionPtr.is_null() {
        return None;
    }
    let DataSize = *OptionPtr.add(1) as DWORD;
    DPRINT!("Vendor option %lu: Length %lu\n", OptionId, DataSize);
    Some((OptionPtr.add(2), DataSize))
}

// ---- Server_RequestParams ----
// Function 7 (upstream c00b0a40358, 1d6dae3bb24): answer DhcpRequestParams from the
// adapter's active lease. The result array and its data are MIDL-allocated; the client stub
// marshals them back and DhcpRequestParams frees them.
//
// Divergence from the C beyond RequestedOption's bound: when the second or third allocation
// fails, the C returns ERROR_NOT_ENOUGH_MEMORY and leaks what it had allocated. Freed here.
#[no_mangle]
pub unsafe extern "system" fn Server_RequestParams(
    ServerName: PDHCP_SERVER_NAME,
    AdapterName: LPWSTR,
    ClassId: *mut DHCPCAPI_CLASSID,
    SendParams: *mut DHCPCAPI_PARAMS_ARRAY,
    RecdParams: *mut DHCPCAPI_PARAMS_ARRAY,
    RecdResults: *mut LPDHCPCAPI_RESULT_ARRAY,
) -> DWORD {
    let mut ret: DWORD = ERROR_SUCCESS;

    DPRINT!("Server_RequestParams(%S %p %p %p %p)\n", AdapterName, ClassId, SendParams, RecdParams, RecdResults);

    ApiLock();

    'done: {
        let Adapter = AdapterFindName(AdapterName);
        if Adapter.is_null() {
            DPRINT1!("Adapter not found\n");
            ret = ERROR_FILE_NOT_FOUND;
            break 'done;
        }

        DPRINT!("Adapter: %p\n", Adapter);

        if (*Adapter).DhclientState.state != S_BOUND {
            DPRINT1!("Adapter is not in S_BOUND state!\n");
            ret = ERROR_FILE_NOT_FOUND;
            break 'done;
        }

        DPRINT!("ActiveLease: %p \n", (*Adapter).DhclientState.active);
        let active = (*Adapter).DhclientState.active;
        if active.is_null() || RecdParams.is_null() {
            break 'done;
        }

        let Options = (*active).options.as_mut_ptr();
        let nParams = (*RecdParams).nParams as usize;
        let Params = (*RecdParams).Params;

        let mut dwReturnCount: DWORD = 0;
        let mut dwReturnLength: DWORD = 0;
        for i in 0..nParams {
            if let Some((_, DataSize)) = RequestedOption(Options, Params.add(i)) {
                dwReturnLength += DataSize;
                dwReturnCount += 1;
            }
        }

        DPRINT!("Return count: %lu  Return length: %lu\n", dwReturnCount, dwReturnLength);

        let Results = MIDL_user_allocate(core::mem::size_of::<DHCPCAPI_RESULT_ARRAY>()) as LPDHCPCAPI_RESULT_ARRAY;
        if Results.is_null() {
            DPRINT1!("Result allocation failed!\n");
            ret = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        (*Results).ResultsCount = dwReturnCount;
        (*Results).Results =
            MIDL_user_allocate(dwReturnCount as usize * core::mem::size_of::<DHCPCAPI_RESULTS>()) as LPDHCPCAPI_RESULTS;
        if (*Results).Results.is_null() {
            DPRINT1!("Results allocation failed!\n");
            MIDL_user_free(Results as *mut c_void);
            ret = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        (*Results).DataSize = dwReturnLength;
        (*Results).Data = MIDL_user_allocate(dwReturnLength as usize) as LPBYTE;
        if (*Results).Data.is_null() {
            DPRINT1!("Data allocation failed!\n");
            MIDL_user_free((*Results).Results as *mut c_void);
            MIDL_user_free(Results as *mut c_void);
            ret = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        let mut Offset: DWORD = 0;
        let mut Index: usize = 0;
        for i in 0..nParams {
            let Param = Params.add(i);
            if let Some((Data, DataSize)) = RequestedOption(Options, Param) {
                let r = &mut *(*Results).Results.add(Index);
                r.OptionId = (*Param).OptionId;
                r.IsVendor = ((*Param).IsVendor != 0) as BOOL;
                r.DataSize = DataSize;
                r.DataOffset = Offset;

                core::ptr::copy_nonoverlapping(Data, (*Results).Data.add(Offset as usize), DataSize as usize);

                Index += 1;
                Offset += DataSize;
            }
        }

        *RecdResults = Results;
    }

    ApiUnlock();

    ret
}

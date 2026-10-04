use crate::prelude_rpc::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (standard netapi32 wine-debug channel; TRACE/FIXME reference it) ----
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};


// ---- LOGONSRV_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn LOGONSRV_HANDLE_bind(pszSystemName: LOGONSRV_HANDLE) -> handle_t {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();

    // Wide string literals as u16 arrays
    let ncacn_np: &[u16] = &[
        0x006e, 0x0063, 0x0061, 0x0063, 0x006e, 0x005f, 0x006e, 0x0070, 0x0000,
    ];
    let pipe_netlogon: &[u16] = &[
        0x005c, 0x0070, 0x0069, 0x0070, 0x0065, 0x005c, 0x006e, 0x0065, 0x0074,
        0x006c, 0x006f, 0x0067, 0x006f, 0x006e, 0x0000,
    ];

    let mut status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        pszSystemName as *mut u16,
        pipe_netlogon.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    // If status fails, we still free the string.

    status = RpcStringFreeW(&mut pszStringBinding);
    // status ignored

    hBinding
}

// ---- LOGONSRV_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn LOGONSRV_HANDLE_unbind(
    pszSystemName: LOGONSRV_HANDLE,
    mut hBinding: handle_t,
) {
    // RpcBindingFree NULLs the caller's handle through the pointer, so this needs `mut`.
    let status: RPC_STATUS = RpcBindingFree(&mut hBinding as *mut handle_t);
    if status != 0 {
        // Do nothing
    }
}

// ---- DsAddressToSiteNamesW ----
#[no_mangle]
pub unsafe extern "system" fn DsAddressToSiteNamesW(
    ComputerName: LPCWSTR,
    EntryCount: DWORD,
    SocketAddresses: *mut core::ffi::c_void,
    SiteNames: *mut *mut PWSTR,
) -> DWORD {
    // Two hand-rolled shadow structs stood here (`_NL_SITE_NAME_ENTRY`, and an
    // `_NL_SITE_NAME_ARRAY` whose SiteNames was a FAKE `[u8; 0]` flex array). Both shadowed the
    // real netlogon_c.h wire types, whose SiteNames is a PRPC_UNICODE_STRING pointer. Removed —
    // wire layouts come from the island, never from a hand-written guess.
    let mut SiteNameArray: *mut _NL_SITE_NAME_ARRAY = core::ptr::null_mut();
    let mut SiteNamesBuffer: *mut PWSTR = core::ptr::null_mut();
    let mut BufferSize: ULONG;
    let mut status: NET_API_STATUS;

    if EntryCount == 0 {
        return 87; // ERROR_INVALID_PARAMETER
    }

    *SiteNames = core::ptr::null_mut();

    // This is the body of RpcTryExcept / RpcEndExcept (exception handler omitted)
    status = rpc_try(|| DsrAddressToSiteNamesW(
        ComputerName as PWSTR,
        EntryCount,
        // netlogon.c:235 — `(PNL_SOCKET_ADDRESS)SocketAddresses`
        SocketAddresses as PNL_SOCKET_ADDRESS,
        &mut SiteNameArray,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == 0 {
        let entry_count = (*SiteNameArray).EntryCount;
        if entry_count == 0 {
            status = 87; // ERROR_INVALID_PARAMETER
        } else {
            BufferSize = entry_count * core::mem::size_of::<PWSTR>() as ULONG;
            let mut i: ULONG = 0;
            while i < entry_count {
                // netlogon_c.h: `PRPC_UNICODE_STRING SiteNames` is a POINTER, so the C's
                // `SiteNames[i]` is plain pointer arithmetic over RPC_UNICODE_STRING.
                let entry = &*(*SiteNameArray).SiteNames.add(i as usize);
                BufferSize += entry.Length as ULONG + 2; // sizeof(WCHAR) = 2
                i += 1;
            }

            status = NetApiBufferAllocate(
                BufferSize,
                &mut SiteNamesBuffer as *mut *mut PWSTR as *mut *mut core::ffi::c_void,
            );
            if status == 0 {
                ZeroMemory(
                    SiteNamesBuffer as *mut core::ffi::c_void,
                    BufferSize as usize,
                );

                let mut Ptr =
                    (SiteNamesBuffer as usize + entry_count as usize * core::mem::size_of::<PWSTR>())
                        as PWSTR;
                let mut i: ULONG = 0;
                while i < entry_count {
                    let entry = &*(*SiteNameArray).SiteNames.add(i as usize);
                    let buffer_addr =
                        SiteNamesBuffer as usize + i as usize * core::mem::size_of::<PWSTR>();
                    *(buffer_addr as *mut PWSTR) = Ptr;
                    CopyMemory(
                        Ptr as *mut core::ffi::c_void,
                        (*entry).Buffer as *const core::ffi::c_void,
                        (*entry).Length as usize,
                    );
                    Ptr = (Ptr as usize + (*entry).Length as usize + 2) as PWSTR;
                    i += 1;
                }

                *SiteNames = SiteNamesBuffer;
            }
        }

        MIDL_user_free(SiteNameArray as *mut core::ffi::c_void);
    }

    status as DWORD
}

// ---- DsAddressToSiteNamesExW ----
#[no_mangle]
pub unsafe extern "system" fn DsAddressToSiteNamesExW(
    ComputerName: *const u16,
    EntryCount: u32,
    SocketAddresses: *mut u8,
    SiteNames: *mut *mut *mut u16,
    SubnetNames: *mut *mut *mut u16,
) -> u32 {
    // Constants
    const NERR_Success: u32 = 0;
    const ERROR_INVALID_PARAMETER: u32 = 87;

    // NOTE: two hand-rolled shadow structs stood here (`NL_SITE_NAME_EX { Length: u32, Buffer }`
    // and `NL_SITE_NAME_EX_ARRAY`). Their LAYOUT was wrong: netlogon_c.h types the entries as
    // RPC_UNICODE_STRING (USHORT Length + USHORT MaximumLength + PWSTR Buffer), so a u32 Length
    // would have mis-read every entry off the wire. Use the island types.
    if EntryCount == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    *SiteNames = core::ptr::null_mut();
    *SubnetNames = core::ptr::null_mut();

    // Equivalent of RpcTryExcept: execute body directly (no SEH handler)
    let mut SiteNameArray: PNL_SITE_NAME_EX_ARRAY = core::ptr::null_mut();
    let mut SiteNamesBuffer: *mut PWSTR = core::ptr::null_mut();
    let mut SubnetNamesBuffer: *mut PWSTR = core::ptr::null_mut();
    let mut status = rpc_try(|| DsrAddressToSiteNamesExW(
        ComputerName as PWSTR,
        EntryCount,
        // netlogon.c:471 — `(PNL_SOCKET_ADDRESS)SocketAddresses`
        SocketAddresses as PNL_SOCKET_ADDRESS,
        &mut SiteNameArray,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == NERR_Success {
        if (*SiteNameArray).EntryCount == 0 {
            // netlogon.c:477 — the C ASSIGNS here; the translation left only a comment, so the
            // empty-array case silently returned NERR_Success.
            status = ERROR_INVALID_PARAMETER;
        } else {
            let entry_count = (*SiteNameArray).EntryCount;
            let mut SiteNameBufferSize =
                entry_count * core::mem::size_of::<PWSTR>() as ULONG;
            let mut SubnetNameBufferSize =
                entry_count * core::mem::size_of::<PWSTR>() as ULONG;
            let mut i: ULONG = 0;
            while i < entry_count {
                SiteNameBufferSize +=
                    (*(*SiteNameArray).SiteNames.add(i as usize)).Length as ULONG + 2;
                SubnetNameBufferSize +=
                    (*(*SiteNameArray).SubnetNames.add(i as usize)).Length as ULONG + 2;
                i += 1;
            }

            status = NetApiBufferAllocate(
                SiteNameBufferSize,
                &mut SiteNamesBuffer as *mut *mut PWSTR as *mut *mut core::ffi::c_void,
            );
            if status == NERR_Success {
                ZeroMemory(
                    SiteNamesBuffer as *mut core::ffi::c_void,
                    SiteNameBufferSize as usize,
                );
                let mut Ptr = (SiteNamesBuffer as usize
                    + entry_count as usize * core::mem::size_of::<PWSTR>())
                    as PWSTR;
                let mut i: ULONG = 0;
                while i < entry_count {
                    let entry = &*(*SiteNameArray).SiteNames.add(i as usize);
                    *SiteNamesBuffer.add(i as usize) = Ptr;
                    CopyMemory(
                        Ptr as *mut core::ffi::c_void,
                        entry.Buffer as *const core::ffi::c_void,
                        entry.Length as usize,
                    );
                    Ptr = (Ptr as usize + entry.Length as usize + 2) as PWSTR;
                    i += 1;
                }
                *SiteNames = SiteNamesBuffer;
            }

            // netlogon.c:508 reuses `status` (it is NOT a separate status2 local): the value
            // this function returns is the SUBNET allocation's status.
            status = NetApiBufferAllocate(
                SubnetNameBufferSize,
                &mut SubnetNamesBuffer as *mut *mut PWSTR as *mut *mut core::ffi::c_void,
            );
            if status == NERR_Success {
                ZeroMemory(
                    SubnetNamesBuffer as *mut core::ffi::c_void,
                    SubnetNameBufferSize as usize,
                );
                let mut Ptr = (SubnetNamesBuffer as usize
                    + entry_count as usize * core::mem::size_of::<PWSTR>())
                    as PWSTR;
                let mut i: ULONG = 0;
                while i < entry_count {
                    let entry = &*(*SiteNameArray).SubnetNames.add(i as usize);
                    *SubnetNamesBuffer.add(i as usize) = Ptr;
                    CopyMemory(
                        Ptr as *mut core::ffi::c_void,
                        entry.Buffer as *const core::ffi::c_void,
                        entry.Length as usize,
                    );
                    Ptr = (Ptr as usize + entry.Length as usize + 2) as PWSTR;
                    i += 1;
                }
                *SubnetNames = SubnetNamesBuffer;
            }
        }
        MIDL_user_free(SiteNameArray as *mut core::ffi::c_void);
    }

    status
}

// ---- DsDeregisterDnsHostRecordsW ----
#[no_mangle]
pub unsafe extern "system" fn DsDeregisterDnsHostRecordsW(
    ServerName: PWSTR,
    DnsDomainName: PWSTR,
    DomainGuid: *mut GUID,
    DsaGuid: *mut GUID,
    DnsHostName: PWSTR,
) -> DWORD {
    rpc_try(|| DsrDeregisterDnsHostRecords(ServerName, DnsDomainName, DomainGuid, DsaGuid, DnsHostName)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- DsEnumerateDomainTrustsW ----
#[no_mangle]
pub unsafe extern "system" fn DsEnumerateDomainTrustsW(
    ServerName: LPWSTR,
    Flags: ULONG,
    Domains: *mut PDS_DOMAIN_TRUSTSW,
    DomainCount: PULONG,
) -> DWORD {
    let mut DomainsArray: NETLOGON_TRUSTED_DOMAIN_ARRAY = core::mem::zeroed();
    let status: DWORD;

    // RpcTryExcept body (exception handler omitted as per translation rules)
    status = rpc_try(|| DsrEnumerateDomainTrusts(ServerName, Flags, &mut DomainsArray)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if status == 0 {
        *Domains = DomainsArray.Domains;
        *DomainCount = DomainsArray.DomainCount;
    }

    status
}

// ---- DsGetDcNameWithAccountW ----
#[no_mangle]
pub unsafe extern "system" fn DsGetDcNameWithAccountW(
    ComputerName: LPCWSTR,
    AccountName: LPCWSTR,
    AccountControlBits: ULONG,
    DomainName: LPCWSTR,
    DomainGuid: *mut GUID,
    SiteName: LPCWSTR,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
) -> DWORD {
    let status = rpc_try(|| DsrGetDcNameEx2(
        ComputerName as PWSTR,
        AccountName as PWSTR,
        AccountControlBits,
        DomainName as PWSTR,
        DomainGuid,
        SiteName as PWSTR,
        Flags,
        DomainControllerInfo,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- DsGetDcSiteCoverageW ----
#[no_mangle]
pub unsafe extern "system" fn DsGetDcSiteCoverageW(
    ServerName: LPCWSTR,
    EntryCount: *mut ULONG,
    SiteNames: *mut *mut LPWSTR,
) -> DWORD {
    // C: `PNL_SITE_NAME_ARRAY SiteNameArray = NULL;` — the translation added a pointer level.
    let mut SiteNameArray: PNL_SITE_NAME_ARRAY = core::ptr::null_mut();
    let mut SiteNamesBuffer: *mut LPWSTR = core::ptr::null_mut();
    let mut Ptr: PWSTR;
    let mut BufferSize: ULONG;
    let mut i: ULONG;
    let mut status: NET_API_STATUS;

    *EntryCount = 0;
    *SiteNames = core::ptr::null_mut();

    // Execute the try body directly (PSEH2_EXCEPT removed)
    status = rpc_try(|| DsrGetDcSiteCoverageW(ServerName as PWSTR, &mut SiteNameArray)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if status == 0 {
        // NERR_Success == 0
        let entry_count = (*SiteNameArray).EntryCount;
        if entry_count == 0 {
            status = 87; // ERROR_INVALID_PARAMETER
        } else {
            BufferSize = entry_count.wrapping_mul(core::mem::size_of::<PWSTR>() as ULONG);
            i = 0;
            while i < entry_count {
                // `SiteNames` is PRPC_UNICODE_STRING — a pointer, not an inline array.
                let len = (*(*SiteNameArray).SiteNames.add(i as usize)).Length as ULONG;
                BufferSize = BufferSize.wrapping_add(len.wrapping_add(2));
                i += 1;
            }

            status = NetApiBufferAllocate(BufferSize, &mut SiteNamesBuffer as *mut *mut LPWSTR as *mut *mut core::ffi::c_void);
            if status == 0 {
                // NERR_Success
                ZeroMemory(
                    SiteNamesBuffer as *mut core::ffi::c_void,
                    BufferSize as usize,
                );

                Ptr = (SiteNamesBuffer as *mut u8)
                    .add((entry_count as usize).wrapping_mul(core::mem::size_of::<PWSTR>()))
                    as PWSTR;

                i = 0;
                while i < entry_count {
                    let idx = i as usize;
                    (*SiteNamesBuffer.offset(idx as isize)) = Ptr;
                    CopyMemory(
                        Ptr as *mut core::ffi::c_void,
                        (*(*SiteNameArray).SiteNames.add(idx)).Buffer as *const core::ffi::c_void,
                        (*(*SiteNameArray).SiteNames.add(idx)).Length as usize,
                    );

                    Ptr = (Ptr as *mut u8)
                        .add(
                            ((*(*SiteNameArray).SiteNames.add(idx)).Length as usize).wrapping_add(2),
                        )
                        as PWSTR;

                    i += 1;
                }

                *EntryCount = entry_count;
                *SiteNames = SiteNamesBuffer;
            }
        }

        MIDL_user_free(SiteNameArray as *mut core::ffi::c_void);
    }

    status
}

// ---- DsGetForestTrustInformationW ----
#[no_mangle]
pub unsafe extern "system" fn DsGetForestTrustInformationW(
    ServerName: LPCWSTR,
    TrustedDomainName: LPCWSTR,
    Flags: DWORD,
    ForestTrustInfo: *mut *mut LSA_FOREST_TRUST_INFORMATION,
) -> DWORD {
    let status: NET_API_STATUS =
        rpc_try(|| DsrGetForestTrustInformation(
            ServerName as *mut u16,
            TrustedDomainName as *mut u16,
            Flags,
            ForestTrustInfo,
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- DsGetSiteNameW ----
#[no_mangle]
pub unsafe extern "system" fn DsGetSiteNameW(
    ComputerName: LPCWSTR,
    SiteName: *mut LPWSTR,
) -> DWORD {
    rpc_try(|| DsrGetSiteName(ComputerName as PWSTR, SiteName)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _)
}

// ---- DsMergeForestTrustInformationW ----
#[no_mangle]
pub unsafe extern "system" fn DsMergeForestTrustInformationW(
    DomainName: *const u16,
    NewForestTrustInfo: *const core::ffi::c_void,
    OldForestTrustInfo: *const core::ffi::c_void,
    ForestTrustInfo: *mut *const core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}

// ---- DsValidateSubnetNameA ----
#[no_mangle]
pub unsafe extern "system" fn DsValidateSubnetNameA(
    _SubnetName: LPCSTR,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

// ---- DsValidateSubnetNameW ----
#[no_mangle]
pub unsafe extern "system" fn DsValidateSubnetNameW(SubnetName: LPCWSTR) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

// ---- NetEnumerateTrustedDomains ----
#[no_mangle]
pub unsafe extern "system" fn NetEnumerateTrustedDomains(
    ServerName: LPWSTR,
    DomainNames: *mut LPWSTR,
) -> NTSTATUS {
    // A local `struct DOMAIN_NAME_BUFFER { DomainCount: u32, DomainNames: LPWSTR }` shadowed the
    // real wire type, whose fields are `DomainNameByteCount: ULONG` + `DomainNames: *mut UCHAR`
    // (netlogon_c.h) — different names AND a different element type. Use the island layout.
    let mut DomainNameBuffer = DOMAIN_NAME_BUFFER {
        DomainNameByteCount: 0,
        DomainNames: core::ptr::null_mut(),
    };
    let Status = rpc_try(|| NetrEnumerateTrustedDomains(ServerName, &mut DomainNameBuffer)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if NT_SUCCESS(Status) {
        // netlogon.c:1487 — `*DomainNames = (LPWSTR)DomainNameBuffer.DomainNames;`
        *DomainNames = DomainNameBuffer.DomainNames as LPWSTR;
    }
    Status
}

// ---- NetGetAnyDCName ----
#[no_mangle]
pub unsafe extern "system" fn NetGetAnyDCName(
    ServerName: LPCWSTR,
    DomainName: LPCWSTR,
    BufPtr: *mut *mut u8,
) -> NET_API_STATUS {
    let mut Status: NET_API_STATUS;
    *BufPtr = core::ptr::null_mut();
    Status = rpc_try(|| NetrGetAnyDCName(
        ServerName as PWSTR,
        DomainName as PWSTR,
        BufPtr as *mut PWSTR,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    Status
}

// ---- NetLogonGetTimeServiceParentDomain ----
#[no_mangle]
pub unsafe extern "system" fn NetLogonGetTimeServiceParentDomain(
    ServerName: LPWSTR,
    DomainName: *mut LPWSTR,
    PdcSameSite: LPBOOL,
) -> NET_API_STATUS {
    let Status = rpc_try(|| NetrLogonGetTimeServiceParentDomain(ServerName, DomainName, PdcSameSite)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    Status
}

// ---- NetLogonSetServiceBits ----
#[no_mangle]
pub unsafe extern "system" fn NetLogonSetServiceBits(
    ServerName: LPWSTR,
    ServiceBitsOfInterest: DWORD,
    ServiceBits: DWORD,
) -> NTSTATUS {
    rpc_try(|| NetrLogonSetServiceBits(ServerName, ServiceBitsOfInterest, ServiceBits)).unwrap_or_else(|code| code as _)
}

// ---- DsAddressToSiteNamesA ----
#[no_mangle]
pub unsafe extern "system" fn DsAddressToSiteNamesA(
    ComputerName: LPCSTR,
    EntryCount: DWORD,
    SocketAddresses: *mut core::ffi::c_void,
    // netlogon.c:113 — `LPSTR **SiteNames` (ANSI).
    SiteNames: *mut *mut PSTR,
) -> DWORD {
    let mut pComputerNameW: PWSTR = core::ptr::null_mut();
    let mut pSiteNamesW: *mut PWSTR = core::ptr::null_mut();
    // netlogon.c:116 — `PSTR *pSiteNamesA = NULL, Ptr;` (ANSI: i8, not u8).
    let mut pSiteNamesA: *mut PSTR = core::ptr::null_mut();
    let mut ptr: PSTR;
    let mut unicode_string: UNICODE_STRING = core::mem::zeroed();
    let mut ansi_string: ANSI_STRING = core::mem::zeroed();
    let mut buffer_size: ULONG;
    let mut i: ULONG;
    let mut status: NTSTATUS;
    let mut net_status: DWORD = 0; // NERR_Success

    if EntryCount == 0 {
        return 87; // ERROR_INVALID_PARAMETER
    }

    if !ComputerName.is_null() {
        pComputerNameW = NetpAllocWStrFromAnsiStr(ComputerName as PSTR);
        if pComputerNameW.is_null() {
            net_status = 8; // ERROR_NOT_ENOUGH_MEMORY
            // (the `if net_status != 0` guard below is what emulates the C's `goto done`;
            // a dead `loop { break; }` placeholder stood here)
        }
    }

    // jump to done if we had allocation failure
    if net_status != 0 {
        // cleanup will happen in done block
    } else {
        net_status = DsAddressToSiteNamesW(
            pComputerNameW,
            EntryCount,
            SocketAddresses,
            &mut pSiteNamesW,
        );
        if net_status != 0 {
            // goto done
        } else {
            buffer_size = EntryCount * core::mem::size_of::<*mut u8>() as ULONG;
            i = 0;
            while i < EntryCount {
                if !(*pSiteNamesW.add(i as usize)).is_null() {
                    RtlInitUnicodeString(
                        &mut unicode_string,
                        *pSiteNamesW.add(i as usize),
                    );
                    buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);
                }
                i += 1;
            }

            net_status = NetApiBufferAllocate(
                buffer_size,
                &mut pSiteNamesA as *mut _ as *mut *mut core::ffi::c_void,
            );
            if net_status != 0 {
                // goto done
            } else {
                ptr = (pSiteNamesA as usize
                    + (EntryCount as usize) * core::mem::size_of::<PSTR>())
                    as PSTR;
                buffer_size -= EntryCount * core::mem::size_of::<PSTR>() as ULONG;

                i = 0;
                'convert: while i < EntryCount {
                    if !(*pSiteNamesW.add(i as usize)).is_null() {
                        *pSiteNamesA.add(i as usize) = ptr;
                        RtlInitUnicodeString(
                            &mut unicode_string,
                            *pSiteNamesW.add(i as usize),
                        );
                        ansi_string.Length = 0;
                        ansi_string.MaximumLength = buffer_size as USHORT;
                        ansi_string.Buffer = ptr;

                        status = RtlUnicodeStringToAnsiString(
                            &mut ansi_string,
                            &unicode_string,
                            0, // FALSE
                        );
                        if !(status >= 0) {
                            // NT_SUCCESS
                            net_status = RtlNtStatusToDosError(status);
                            // The C `goto done`s out of the conversion loop here. A no-op
                            // `loop { break; }` stood in its place, so the loop kept converting
                            // entries after a failure. `break 'convert` reproduces the C.
                            break 'convert;
                        }

                        ptr = ptr.offset(ansi_string.Length as isize + 1); // sizeof(CHAR) = 1
                        buffer_size -= ansi_string.Length as ULONG + 1;
                    }
                    i += 1;
                }

                *SiteNames = pSiteNamesA;
                pSiteNamesA = core::ptr::null_mut();
            }
        }
    }

    // done:
    if !pSiteNamesA.is_null() {
        NetApiBufferFree(pSiteNamesA as *mut core::ffi::c_void);
    }
    if !pSiteNamesW.is_null() {
        NetApiBufferFree(pSiteNamesW as *mut core::ffi::c_void);
    }
    if !pComputerNameW.is_null() {
        NetApiBufferFree(pComputerNameW as *mut core::ffi::c_void);
    }

    net_status
}

// ---- DsAddressToSiteNamesExA ----
#[no_mangle]
pub unsafe extern "system" fn DsAddressToSiteNamesExA(
    // netlogon.c:285-289 — the ANSI variant: LPCSTR / LPSTR** (i8), not u8. `Status` is an
    // NTSTATUS (i32) distinct from the NET_API_STATUS `status` (u32); the translation had
    // flattened all of these to u8/u32.
    ComputerName: LPCSTR,
    EntryCount: u32,
    SocketAddresses: *mut u8,
    SiteNames: *mut *mut LPSTR,
    SubnetNames: *mut *mut LPSTR,
) -> u32 {
    let mut pComputerNameW: *mut u16 = core::ptr::null_mut();
    let mut pSiteNamesW: *mut *mut u16 = core::ptr::null_mut();
    let mut pSubnetNamesW: *mut *mut u16 = core::ptr::null_mut();
    let mut pSiteNamesA: *mut LPSTR = core::ptr::null_mut();
    let mut pSubnetNamesA: *mut LPSTR = core::ptr::null_mut();
    let mut Ptr: LPSTR;
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut BufferSize: u32;
    let mut i: u32;
    let mut Status: NTSTATUS;
    let mut status: u32 = 0; // NERR_Success

    if EntryCount == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    // The C uses `goto done` to jump to the shared cleanup tail. Rust's equivalent is a
    // labeled block: `break 'done` leaves the block and falls into the same cleanup below.
    'done: {
    if !ComputerName.is_null() {
        pComputerNameW = NetpAllocWStrFromAnsiStr(ComputerName as PSTR);
        if pComputerNameW.is_null() {
            status = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }
    }

    status = DsAddressToSiteNamesExW(
        pComputerNameW,
        EntryCount,
        SocketAddresses,
        &mut pSiteNamesW,
        &mut pSubnetNamesW,
    );
    if status != 0 {
        break 'done;
    }

    // Calculate required buffer size for site names ANSI strings
    BufferSize = EntryCount * core::mem::size_of::<*mut u8>() as u32;
    i = 0;
    while i < EntryCount {
        if !(*pSiteNamesW.add(i as usize)).is_null() {
            RtlInitUnicodeString(&mut UnicodeString, *pSiteNamesW.add(i as usize));
            BufferSize += RtlUnicodeStringToAnsiSize(&UnicodeString);
        }
        i += 1;
    }

    // Allocate site names ANSI buffer
    status = NetApiBufferAllocate(BufferSize as u32, &mut pSiteNamesA as *mut *mut LPSTR as *mut *mut core::ffi::c_void);
    if status != 0 {
        break 'done;
    }

    // Convert site names
    Ptr = (pSiteNamesA as usize + (EntryCount as usize * core::mem::size_of::<LPSTR>())) as LPSTR;
    BufferSize -= EntryCount * core::mem::size_of::<*mut u8>() as u32;

    i = 0;
    while i < EntryCount {
        if !(*pSiteNamesW.add(i as usize)).is_null() {
            *pSiteNamesA.add(i as usize) = Ptr;
            RtlInitUnicodeString(&mut UnicodeString, *pSiteNamesW.add(i as usize));
            AnsiString.Length = 0;
            AnsiString.MaximumLength = BufferSize as u16;
            AnsiString.Buffer = Ptr;

            Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &UnicodeString, 0);
            if !NT_SUCCESS(Status) {
                status = RtlNtStatusToDosError(Status);
                break 'done;
            }

            Ptr = Ptr.add(AnsiString.Length as usize + 1);
            BufferSize -= AnsiString.Length as u32 + 1;
        }
        i += 1;
    }

    // Calculate required buffer size for subnet names ANSI strings
    BufferSize = EntryCount * core::mem::size_of::<*mut u8>() as u32;
    i = 0;
    while i < EntryCount {
        if !(*pSubnetNamesW.add(i as usize)).is_null() {
            RtlInitUnicodeString(&mut UnicodeString, *pSubnetNamesW.add(i as usize));
            BufferSize += RtlUnicodeStringToAnsiSize(&UnicodeString);
        }
        i += 1;
    }

    // Allocate subnet names ANSI buffer
    status = NetApiBufferAllocate(BufferSize as u32, &mut pSubnetNamesA as *mut *mut LPSTR as *mut *mut core::ffi::c_void);
    if status != 0 {
        break 'done;
    }

    // Convert subnet names
    Ptr = (pSubnetNamesA as usize + (EntryCount as usize * core::mem::size_of::<LPSTR>())) as LPSTR;
    BufferSize -= EntryCount * core::mem::size_of::<*mut u8>() as u32;

    i = 0;
    while i < EntryCount {
        if !(*pSubnetNamesW.add(i as usize)).is_null() {
            *pSubnetNamesA.add(i as usize) = Ptr;
            RtlInitUnicodeString(&mut UnicodeString, *pSubnetNamesW.add(i as usize));
            AnsiString.Length = 0;
            AnsiString.MaximumLength = BufferSize as u16;
            AnsiString.Buffer = Ptr;

            Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &UnicodeString, 0);
            if !NT_SUCCESS(Status) {
                status = RtlNtStatusToDosError(Status);
                break 'done;
            }

            Ptr = Ptr.add(AnsiString.Length as usize + 1);
            BufferSize -= AnsiString.Length as u32 + 1;
        }
        i += 1;
    }

    *SiteNames = pSiteNamesA;
    *SubnetNames = pSubnetNamesA;
    pSiteNamesA = core::ptr::null_mut();
    pSubnetNamesA = core::ptr::null_mut();
    } // 'done — the C's `done:` label; cleanup below runs on every path
    if !pSubnetNamesA.is_null() {
        NetApiBufferFree(pSubnetNamesA as *mut core::ffi::c_void);
    }
    if !pSiteNamesA.is_null() {
        NetApiBufferFree(pSiteNamesA as *mut core::ffi::c_void);
    }
    if !pSubnetNamesW.is_null() {
        NetApiBufferFree(pSubnetNamesW as *mut core::ffi::c_void);
    }
    if !pSiteNamesW.is_null() {
        NetApiBufferFree(pSiteNamesW as *mut core::ffi::c_void);
    }
    if !pComputerNameW.is_null() {
        NetApiBufferFree(pComputerNameW as *mut core::ffi::c_void);
    }

    status
}

// ---- DsDeregisterDnsHostRecordsA ----
#[no_mangle]
pub unsafe extern "system" fn DsDeregisterDnsHostRecordsA(
    ServerName: LPSTR,
    DnsDomainName: LPSTR,
    DomainGuid: *mut GUID,
    DsaGuid: *mut GUID,
    DnsHostName: LPSTR,
) -> DWORD {
    let mut pServerNameW: PWSTR = core::ptr::null_mut();
    let mut pDnsDomainNameW: PWSTR = core::ptr::null_mut();
    let mut pDnsHostNameW: PWSTR = core::ptr::null_mut();
    let mut status: DWORD = 0; // NERR_Success

    'done: loop {
        if !ServerName.is_null() {
            pServerNameW = NetpAllocWStrFromAnsiStr(ServerName as PSTR);
            if pServerNameW.is_null() {
                status = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                break 'done;
            }
        }

        if !DnsDomainName.is_null() {
            pDnsDomainNameW = NetpAllocWStrFromAnsiStr(DnsDomainName as PSTR);
            if pDnsDomainNameW.is_null() {
                status = ERROR_NOT_ENOUGH_MEMORY as DWORD;
                break 'done;
            }
        }

        pDnsHostNameW = NetpAllocWStrFromAnsiStr(DnsHostName as PSTR);
        if pDnsHostNameW.is_null() {
            status = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'done;
        }

        status = DsDeregisterDnsHostRecordsW(
            pServerNameW,
            pDnsDomainNameW,
            DomainGuid,
            DsaGuid,
            pDnsHostNameW,
        );

        break 'done;
    }

    if !pDnsHostNameW.is_null() {
        NetApiBufferFree(pDnsHostNameW as *mut core::ffi::c_void);
    }
    if !pDnsDomainNameW.is_null() {
        NetApiBufferFree(pDnsDomainNameW as *mut core::ffi::c_void);
    }
    if !pServerNameW.is_null() {
        NetApiBufferFree(pServerNameW as *mut core::ffi::c_void);
    }

    status
}

// ---- DsEnumerateDomainTrustsA ----
#[no_mangle]
pub unsafe extern "system" fn DsEnumerateDomainTrustsA(
    ServerName: LPSTR,
    Flags: ULONG,
    Domains: *mut PDS_DOMAIN_TRUSTSA,
    DomainCount: PULONG,
) -> DWORD {
    let mut pServerNameW: PWSTR = core::ptr::null_mut();
    let mut pDomainsW: PDS_DOMAIN_TRUSTSW = core::ptr::null_mut();
    let mut pDomainsA: PDS_DOMAIN_TRUSTSA = core::ptr::null_mut();
    let mut unicode_string = core::mem::MaybeUninit::<UNICODE_STRING>::uninit();
    let mut ansi_string = core::mem::MaybeUninit::<ANSI_STRING>::uninit();
    let mut status: DWORD = 0; // NERR_Success
    let mut nt_status: NTSTATUS;

    'outer: {
        if !ServerName.is_null() {
            pServerNameW = NetpAllocWStrFromAnsiStr(ServerName as PSTR);
            if pServerNameW.is_null() {
                status = 8; // ERROR_NOT_ENOUGH_MEMORY
                break 'outer;
            }
        }

        status = DsEnumerateDomainTrustsW(
            pServerNameW,
            Flags,
            &mut pDomainsW,
            DomainCount,
        );
        if status != 0 {
            // NERR_Success is 0
            break 'outer;
        }

        let count = *DomainCount;
        let mut buffer_size: ULONG = count as ULONG * core::mem::size_of::<DS_DOMAIN_TRUSTSA>() as ULONG;

        for i in 0..count {
            let trust_w = &*pDomainsW.add(i as usize);
            RtlInitUnicodeString(unicode_string.as_mut_ptr(), trust_w.NetbiosDomainName);
            buffer_size += RtlUnicodeStringToAnsiSize(unicode_string.as_ptr());

            if !trust_w.DnsDomainName.is_null() {
                RtlInitUnicodeString(unicode_string.as_mut_ptr(), trust_w.DnsDomainName);
                buffer_size += RtlUnicodeStringToAnsiSize(unicode_string.as_ptr());
            }

            buffer_size += RtlLengthSid(trust_w.DomainSid) as ULONG;
        }

        // NetApiBufferAllocate takes a DWORD size and returns NET_API_STATUS (u32).
        status = NetApiBufferAllocate(buffer_size as DWORD, &mut pDomainsA as *mut _ as *mut *mut core::ffi::c_void);
        if status != 0 {
            break 'outer;
        }

        let mut ptr: *mut u8 = (pDomainsA as usize + count as usize * core::mem::size_of::<DS_DOMAIN_TRUSTSA>()) as *mut u8;

        for i in 0..count {
            let trust_w = &*pDomainsW.add(i as usize);
            let trust_a = &mut *pDomainsA.add(i as usize);

            // NetbiosDomainName
            trust_a.NetbiosDomainName = ptr as PSTR;
            RtlInitUnicodeString(unicode_string.as_mut_ptr(), trust_w.NetbiosDomainName);
            (*ansi_string.as_mut_ptr()).Length = 0;
            (*ansi_string.as_mut_ptr()).MaximumLength = buffer_size as u16;
            (*ansi_string.as_mut_ptr()).Buffer = ptr as PSTR;

            nt_status = RtlUnicodeStringToAnsiString(ansi_string.as_mut_ptr(), unicode_string.as_ptr(), 0);
            if !NT_SUCCESS(nt_status) {
                status = RtlNtStatusToDosError(nt_status);
                break 'outer;
            }

            ptr = ptr.add((*ansi_string.as_mut_ptr()).Length as usize + 1); // + sizeof(CHAR)
            buffer_size -= (*ansi_string.as_mut_ptr()).Length as ULONG + 1u32;

            // DnsDomainName if present
            if !trust_w.DnsDomainName.is_null() {
                trust_a.DnsDomainName = ptr as PSTR;
                RtlInitUnicodeString(unicode_string.as_mut_ptr(), trust_w.DnsDomainName);
                (*ansi_string.as_mut_ptr()).Length = 0;
                (*ansi_string.as_mut_ptr()).MaximumLength = buffer_size as u16;
                (*ansi_string.as_mut_ptr()).Buffer = ptr as PSTR;

                nt_status = RtlUnicodeStringToAnsiString(ansi_string.as_mut_ptr(), unicode_string.as_ptr(), 0);
                if !NT_SUCCESS(nt_status) {
                    status = RtlNtStatusToDosError(nt_status);
                    break 'outer;
                }

                ptr = ptr.add((*ansi_string.as_mut_ptr()).Length as usize + 1);
                buffer_size -= (*ansi_string.as_mut_ptr()).Length as ULONG + 1u32;
            }

            trust_a.Flags = trust_w.Flags;
            trust_a.ParentIndex = trust_w.ParentIndex;
            trust_a.TrustType = trust_w.TrustType;
            trust_a.TrustAttributes = trust_w.TrustAttributes;

            // DomainSid
            let sid_length = RtlLengthSid(trust_w.DomainSid);
            trust_a.DomainSid = ptr as PSID;
            nt_status = RtlCopySid(sid_length, ptr as PSID, trust_w.DomainSid);
            if !NT_SUCCESS(nt_status) {
                status = RtlNtStatusToDosError(nt_status);
                break 'outer;
            }

            ptr = ptr.add(sid_length as usize);
            buffer_size -= sid_length as ULONG;

            // Copy GUID
            core::ptr::copy_nonoverlapping(
                &trust_w.DomainGuid as *const _ as *const u8,
                &mut trust_a.DomainGuid as *mut _ as *mut u8,
                core::mem::size_of::<GUID>(),
            );
        }

        *Domains = pDomainsA;
        pDomainsA = core::ptr::null_mut();
    }

    // Cleanup
    if !pDomainsA.is_null() {
        NetApiBufferFree(pDomainsA as *mut core::ffi::c_void);
    }
    if !pDomainsW.is_null() {
        NetApiBufferFree(pDomainsW as *mut core::ffi::c_void);
    }
    if !pServerNameW.is_null() {
        NetApiBufferFree(pServerNameW as *mut core::ffi::c_void);
    }

    status
}

// ---- DsGetDcNameW ----
#[no_mangle]
pub unsafe extern "system" fn DsGetDcNameW(
    ComputerName: LPCWSTR,
    DomainName: LPCWSTR,
    DomainGuid: *mut GUID,
    SiteName: LPCWSTR,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOW,
) -> DWORD {
    DsGetDcNameWithAccountW(
        ComputerName,
        core::ptr::null_mut(),
        0,
        DomainName,
        DomainGuid,
        SiteName,
        Flags,
        DomainControllerInfo,
    )
}

// ---- DsGetDcNameWithAccountA ----
#[no_mangle]
pub unsafe extern "system" fn DsGetDcNameWithAccountA(
    ComputerName: LPCSTR,
    AccountName: LPCSTR,
    AccountControlBits: ULONG,
    DomainName: LPCSTR,
    DomainGuid: *mut GUID,
    SiteName: LPCSTR,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOA,
) -> DWORD {
    let mut pComputerNameW: PWSTR = core::ptr::null_mut();
    let mut pAccountNameW: PWSTR = core::ptr::null_mut();
    let mut pDomainNameW: PWSTR = core::ptr::null_mut();
    let mut pSiteNameW: PWSTR = core::ptr::null_mut();
    let mut pDomainControllerInfoW: PDOMAIN_CONTROLLER_INFOW = core::ptr::null_mut();
    let mut pDomainControllerInfoA: PDOMAIN_CONTROLLER_INFOA = core::ptr::null_mut();
    let mut unicode_string: UNICODE_STRING = core::mem::zeroed();
    let mut ansi_string: ANSI_STRING = core::mem::zeroed();
    let mut ptr: PSTR = core::ptr::null_mut();
    let mut buffer_size: ULONG;
    let mut nt_status: NTSTATUS;
    let mut status: DWORD = 0; // NERR_Success

    if !ComputerName.is_null() {
        pComputerNameW = NetpAllocWStrFromAnsiStr(ComputerName as *mut i8);
        if pComputerNameW.is_null() {
            status = 8; // ERROR_NOT_ENOUGH_MEMORY
            // goto done
            // We'll use a loop with break for goto pattern
        }
    }

    if status != 0 {
        // no need to continue, will go to cleanup
    } else if !AccountName.is_null() {
        pAccountNameW = NetpAllocWStrFromAnsiStr(AccountName as *mut i8);
        if pAccountNameW.is_null() {
            status = 8;
        }
    }

    if status != 0 {
        // continue to cleanup
    } else {
        pDomainNameW = NetpAllocWStrFromAnsiStr(DomainName as *mut i8);
        if pDomainNameW.is_null() {
            status = 8;
        }
    }

    if status != 0 {
        // continue to cleanup
    } else {
        pSiteNameW = NetpAllocWStrFromAnsiStr(SiteName as *mut i8);
        if pSiteNameW.is_null() {
            status = 8;
        }
    }

    if status != 0 {
        // cleanup
    } else {
        status = DsGetDcNameWithAccountW(
            pComputerNameW,
            pAccountNameW,
            AccountControlBits,
            pDomainNameW,
            DomainGuid,
            pSiteNameW,
            Flags,
            &mut pDomainControllerInfoW,
        );
        if status != 0 {
            // goto done
        }
    }

    if status == 0 {
        buffer_size = core::mem::size_of::<DOMAIN_CONTROLLER_INFOA>() as ULONG;

        RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DomainControllerName);
        buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);

        RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DomainControllerAddress);
        buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);

        RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DomainName);
        buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);

        RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DnsForestName);
        buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);

        if !(*pDomainControllerInfoW).DcSiteName.is_null() {
            RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DcSiteName);
            buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);
        }

        if !(*pDomainControllerInfoW).ClientSiteName.is_null() {
            RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).ClientSiteName);
            buffer_size += RtlUnicodeStringToAnsiSize(&unicode_string);
        }

        status = NetApiBufferAllocate(buffer_size, &mut pDomainControllerInfoA as *mut _ as *mut _);
        if status != 0 {
            // goto done
        } else {
            (*pDomainControllerInfoA).DomainControllerAddressType = (*pDomainControllerInfoW).DomainControllerAddressType;
            (*pDomainControllerInfoA).Flags = (*pDomainControllerInfoW).Flags;

            core::ptr::copy_nonoverlapping(
                &(*pDomainControllerInfoW).DomainGuid as *const _ as *const u8,
                &mut (*pDomainControllerInfoA).DomainGuid as *mut _ as *mut u8,
                core::mem::size_of::<GUID>(),
            );

            ptr = (pDomainControllerInfoA as usize + core::mem::size_of::<DOMAIN_CONTROLLER_INFOA>()) as PSTR;
            buffer_size -= core::mem::size_of::<DOMAIN_CONTROLLER_INFOA>() as ULONG;

            (*pDomainControllerInfoA).DomainControllerName = ptr;
            RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DomainControllerName);
            ansi_string.Length = 0;
            ansi_string.MaximumLength = buffer_size as u16;
            ansi_string.Buffer = ptr;

            nt_status = RtlUnicodeStringToAnsiString(&mut ansi_string, &mut unicode_string, 0u8);
            if !NT_SUCCESS(nt_status) {
                status = RtlNtStatusToDosError(nt_status);
                // goto done
            } else {
                ptr = ptr.offset((ansi_string.Length as usize + 1) as isize);
                buffer_size -= ansi_string.Length as ULONG + 1;

                (*pDomainControllerInfoA).DomainControllerAddress = ptr;
                RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DomainControllerAddress);
                ansi_string.Length = 0;
                ansi_string.MaximumLength = buffer_size as u16;
                ansi_string.Buffer = ptr;

                nt_status = RtlUnicodeStringToAnsiString(&mut ansi_string, &mut unicode_string, 0u8);
                if !NT_SUCCESS(nt_status) {
                    status = RtlNtStatusToDosError(nt_status);
                } else {
                    ptr = ptr.offset((ansi_string.Length as usize + 1) as isize);
                    buffer_size -= ansi_string.Length as ULONG + 1;

                    (*pDomainControllerInfoA).DomainName = ptr;
                    RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DomainName);
                    ansi_string.Length = 0;
                    ansi_string.MaximumLength = buffer_size as u16;
                    ansi_string.Buffer = ptr;

                    nt_status = RtlUnicodeStringToAnsiString(&mut ansi_string, &mut unicode_string, 0u8);
                    if !NT_SUCCESS(nt_status) {
                        status = RtlNtStatusToDosError(nt_status);
                    } else {
                        ptr = ptr.offset((ansi_string.Length as usize + 1) as isize);
                        buffer_size -= ansi_string.Length as ULONG + 1;

                        (*pDomainControllerInfoA).DnsForestName = ptr;
                        RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DnsForestName);
                        ansi_string.Length = 0;
                        ansi_string.MaximumLength = buffer_size as u16;
                        ansi_string.Buffer = ptr;

                        nt_status = RtlUnicodeStringToAnsiString(&mut ansi_string, &mut unicode_string, 0u8);
                        if !NT_SUCCESS(nt_status) {
                            status = RtlNtStatusToDosError(nt_status);
                        } else {
                            ptr = ptr.offset((ansi_string.Length as usize + 1) as isize);
                            buffer_size -= ansi_string.Length as ULONG + 1;

                            if !(*pDomainControllerInfoW).DcSiteName.is_null() {
                                (*pDomainControllerInfoA).DcSiteName = ptr;
                                RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).DcSiteName);
                                ansi_string.Length = 0;
                                ansi_string.MaximumLength = buffer_size as u16;
                                ansi_string.Buffer = ptr;

                                nt_status = RtlUnicodeStringToAnsiString(&mut ansi_string, &mut unicode_string, 0u8);
                                if !NT_SUCCESS(nt_status) {
                                    status = RtlNtStatusToDosError(nt_status);
                                } else {
                                    ptr = ptr.offset((ansi_string.Length as usize + 1) as isize);
                                    buffer_size -= ansi_string.Length as ULONG + 1;
                                }
                            }

                            if status == 0 && !(*pDomainControllerInfoW).ClientSiteName.is_null() {
                                (*pDomainControllerInfoA).ClientSiteName = ptr;
                                RtlInitUnicodeString(&mut unicode_string, (*pDomainControllerInfoW).ClientSiteName);
                                ansi_string.Length = 0;
                                ansi_string.MaximumLength = buffer_size as u16;
                                ansi_string.Buffer = ptr;

                                nt_status = RtlUnicodeStringToAnsiString(&mut ansi_string, &mut unicode_string, 0u8);
                                if !NT_SUCCESS(nt_status) {
                                    status = RtlNtStatusToDosError(nt_status);
                                }
                            }

                            if status == 0 {
                                *DomainControllerInfo = pDomainControllerInfoA;
                                pDomainControllerInfoA = core::ptr::null_mut();
                            }
                        }
                    }
                }
            }
        }
    }

    // Cleanup (done label)
    if !pDomainControllerInfoA.is_null() {
        NetApiBufferFree(pDomainControllerInfoA as *mut _);
    }
    if !pDomainControllerInfoW.is_null() {
        NetApiBufferFree(pDomainControllerInfoW as *mut _);
    }
    if !pSiteNameW.is_null() {
        NetApiBufferFree(pSiteNameW as *mut _);
    }
    if !pDomainNameW.is_null() {
        NetApiBufferFree(pDomainNameW as *mut _);
    }
    if !pAccountNameW.is_null() {
        NetApiBufferFree(pAccountNameW as *mut _);
    }
    if !pComputerNameW.is_null() {
        NetApiBufferFree(pComputerNameW as *mut _);
    }

    status
}

// ---- NetGetDCName ----
#[no_mangle]
pub unsafe extern "system" fn NetGetDCName(
    ServerName: LPCWSTR,
    DomainName: LPCWSTR,
    BufPtr: *mut LPBYTE,
) -> DWORD {
    const NERR_Success: DWORD = 0;
    const NERR_DCNotFound: DWORD = 2453;

    let mut pDomainControllerInfo: *mut DOMAIN_CONTROLLER_INFOW = core::ptr::null_mut();
    let mut Status: DWORD;

    // The `'done` label was emitted AFTER the two `break 'done` sites, so the breaks referred
    // to a label not yet in scope. The C's `goto done` jumps FORWARD to the cleanup tail, so
    // the labeled block must ENCLOSE the body and the cleanup must follow it.
    'done: {
        if ServerName.is_null() || *ServerName == 0 {
            Status = DsGetDcNameWithAccountW(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                DomainName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                &mut pDomainControllerInfo,
            );
            if Status != NERR_Success {
                break 'done;
            }

            let len = wcslen((*pDomainControllerInfo).DomainControllerName);
            let size = ((len + 1) * 2) as DWORD; // sizeof(WCHAR) = 2
            Status = NetApiBufferAllocate(size, BufPtr as *mut *mut core::ffi::c_void);
            if Status != NERR_Success {
                break 'done;
            }

            wcscpy(*BufPtr as PWSTR, (*pDomainControllerInfo).DomainControllerName);
        } else {
            Status = NERR_DCNotFound;
        }
    }

    if !pDomainControllerInfo.is_null() {
        NetApiBufferFree(pDomainControllerInfo as *mut core::ffi::c_void);
    }

    Status
}

// ---- DsGetDcSiteCoverageA ----
#[no_mangle]
pub unsafe extern "system" fn DsGetDcSiteCoverageA(
    ServerName: LPCSTR,
    EntryCount: *mut ULONG,
    SiteNames: *mut *mut LPSTR,
) -> DWORD {
    let mut pServerNameW: PWSTR = core::ptr::null_mut();
    let mut pSiteNamesW: *mut LPWSTR = core::ptr::null_mut();
    let mut pSiteNamesA: *mut LPSTR = core::ptr::null_mut();
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut Ptr: LPSTR = core::ptr::null_mut();
    let mut BufferSize: ULONG = 0;
    let mut i: ULONG;
    let mut Status: NTSTATUS = 0;
    let mut status: DWORD = 0; // NERR_Success

    // NOTE: every `goto done` in this function had been emitted as `'done: loop { break; }` —
    // a NO-OP that falls through instead of jumping to the cleanup tail. On a failure path that
    // kept running (e.g. walking pSiteNamesW after DsGetDcSiteCoverageW had failed) instead of
    // bailing out. Restructured into the real labeled block the C's `goto done` means.
    'done: {
    if !ServerName.is_null() {
        pServerNameW = NetpAllocWStrFromAnsiStr(ServerName as *mut i8);
        if pServerNameW.is_null() {
            status = ERROR_NOT_ENOUGH_MEMORY as DWORD;
            break 'done;
        }
    }

    status = DsGetDcSiteCoverageW(pServerNameW, EntryCount, &mut pSiteNamesW);
    if status != 0 { // ERROR_SUCCESS
        break 'done;
    }

    BufferSize = (*EntryCount) * core::mem::size_of::<LPSTR>() as ULONG;
    i = 0;
    while i < *EntryCount {
        // The C passes pSiteNamesW[i] (the string); `.offset(i)` alone passes the address OF
        // the array slot instead. Deref to get the PWSTR.
        RtlInitUnicodeString(&mut UnicodeString, *pSiteNamesW.add(i as usize));
        BufferSize += RtlUnicodeStringToAnsiSize(&UnicodeString);
        i += 1;
    }

    status = NetApiBufferAllocate(BufferSize, &mut pSiteNamesA as *mut *mut LPSTR as *mut *mut core::ffi::c_void) as DWORD;
    if status != 0 {
        break 'done;
    }

    core::ptr::write_bytes(pSiteNamesA as *mut u8, 0u8, BufferSize as usize);

    Ptr = (pSiteNamesA as usize + (*EntryCount as usize) * core::mem::size_of::<LPSTR>()) as LPSTR;
    i = 0;
    while i < *EntryCount {
        *pSiteNamesA.offset(i as isize) = Ptr;

        // The C passes pSiteNamesW[i] (the string); `.offset(i)` alone passes the address OF
        // the array slot instead. Deref to get the PWSTR.
        RtlInitUnicodeString(&mut UnicodeString, *pSiteNamesW.add(i as usize));

        AnsiString.Length = 0;
        AnsiString.MaximumLength = BufferSize as USHORT;
        AnsiString.Buffer = Ptr;

        Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &UnicodeString, 0u8);
        if !NT_SUCCESS(Status) {
            status = RtlNtStatusToDosError(Status);
            break 'done;
        }

        Ptr = (Ptr as usize + AnsiString.Length as usize + core::mem::size_of::<i8>()) as LPSTR;
        BufferSize -= (AnsiString.Length as ULONG + 1);
        i += 1;
    }

    *SiteNames = pSiteNamesA;
    pSiteNamesA = core::ptr::null_mut();
    } // 'done — the C's `done:` label; the cleanup below runs on every path

    if status != 0 && !pSiteNamesA.is_null() {
        NetApiBufferFree(pSiteNamesA as *mut core::ffi::c_void);
    }

    if !pSiteNamesW.is_null() {
        NetApiBufferFree(pSiteNamesW as *mut core::ffi::c_void);
    }

    if !pServerNameW.is_null() {
        NetApiBufferFree(pServerNameW as *mut core::ffi::c_void);
    }

    return status;
}

// ---- DsGetSiteNameA ----
#[no_mangle]
pub unsafe extern "system" fn DsGetSiteNameA(
    ComputerName: LPCSTR,
    SiteName: *mut LPSTR,
) -> DWORD {
    let mut pComputerNameW: PWSTR = core::ptr::null_mut();
    let mut pSiteNameW: PWSTR = core::ptr::null_mut();
    let mut status = ERROR_SUCCESS;

    if !ComputerName.is_null() {
        pComputerNameW = NetpAllocWStrFromAnsiStr(ComputerName as PSTR);
        if pComputerNameW.is_null() {
            status = ERROR_NOT_ENOUGH_MEMORY;
        }
    }

    if status == ERROR_SUCCESS {
        status = DsGetSiteNameW(pComputerNameW, &mut pSiteNameW);
    }

    if status == ERROR_SUCCESS {
        *SiteName = NetpAllocAnsiStrFromWStr(pSiteNameW);
        if (*SiteName).is_null() {
            status = ERROR_NOT_ENOUGH_MEMORY;
        }
    }

    if !pSiteNameW.is_null() {
        NetApiBufferFree(pSiteNameW as *mut core::ffi::c_void);
    }
    if !pComputerNameW.is_null() {
        NetApiBufferFree(pComputerNameW as *mut core::ffi::c_void);
    }

    status
}

// ---- DsGetDcNameA ----
#[no_mangle]
pub unsafe extern "system" fn DsGetDcNameA(
    ComputerName: LPCSTR,
    DomainName: LPCSTR,
    DomainGuid: *mut GUID,
    SiteName: LPCSTR,
    Flags: ULONG,
    DomainControllerInfo: *mut PDOMAIN_CONTROLLER_INFOA,
) -> DWORD {
    DsGetDcNameWithAccountA(
        ComputerName,
        core::ptr::null_mut(),
        0,
        DomainName,
        DomainGuid,
        SiteName,
        Flags,
        DomainControllerInfo,
    )
}

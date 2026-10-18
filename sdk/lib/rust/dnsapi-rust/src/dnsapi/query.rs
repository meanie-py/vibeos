use crate::prelude::*;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;


// ---- ParseIpv4Address ----
unsafe fn ParseIpv4Address(
    AddressString: PCWSTR,
    pAddress: PIN_ADDR,
) -> i32 {
    let mut pTerminator: PCWSTR = core::ptr::null();
    let Status: NTSTATUS;

    Status = RtlIpv4StringToAddressW(
        AddressString,
        1u8, // TRUE
        &mut pTerminator as *mut PCWSTR,
        pAddress,
    );

    if NT_SUCCESS(Status) && !pTerminator.is_null() && *pTerminator == 0u16 {
        return 1;
    }

    0
}

// ---- ParseIpv6Address ----
unsafe fn ParseIpv6Address(AddressString: PCWSTR, pAddress: PIN6_ADDR) -> i32 {
    let mut pTerminator: *const u16 = core::ptr::null();
    let Status = RtlIpv6StringToAddressW(AddressString, &mut pTerminator, pAddress);
    if NT_SUCCESS(Status) && !pTerminator.is_null() && *pTerminator == 0u16 {
        1
    } else {
        0
    }
}

// ---- DnsWToC ----
unsafe fn DnsWToC(WideString: PCWSTR) -> *mut i8 {
    let AnsiLen = WideCharToMultiByte(CP_ACP,
                                      0,
                                      WideString,
                                      -1,
                                      core::ptr::null_mut(),
                                      0,
                                      core::ptr::null_mut(),
                                      core::ptr::null_mut());
    if AnsiLen == 0 {
        return core::ptr::null_mut();
    }
    let AnsiString = RtlAllocateHeap(RtlGetProcessHeap(), 0, AnsiLen as usize) as *mut i8;
    if AnsiString == core::ptr::null_mut() {
        return core::ptr::null_mut();
    }
    WideCharToMultiByte(CP_ACP,
                        0,
                        WideString,
                        -1,
                        AnsiString,
                        AnsiLen,
                        core::ptr::null_mut(),
                        core::ptr::null_mut());
    AnsiString
}

// ---- DnsCToW ----
unsafe fn DnsCToW(NarrowString: *const i8) -> *mut u16 {
    let WideLen: i32 = MultiByteToWideChar(0, 0, NarrowString, -1, core::ptr::null_mut(), 0);
    if WideLen == 0 {
        return core::ptr::null_mut();
    }
    let WideString = RtlAllocateHeap(RtlGetProcessHeap(), 0, (WideLen as usize) * core::mem::size_of::<u16>()) as *mut u16;
    if WideString.is_null() {
        return core::ptr::null_mut();
    }
    MultiByteToWideChar(0, 0, NarrowString, -1, WideString, WideLen);
    WideString
}

// ---- DnsWToUTF8 ----
unsafe fn DnsWToUTF8(WideString: *const u16) -> *mut u8 {
    let AnsiLen = WideCharToMultiByte(65001, 0, WideString, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    if AnsiLen == 0 {
        return core::ptr::null_mut();
    }
    let AnsiString = RtlAllocateHeap(RtlGetProcessHeap(), 0, AnsiLen as usize) as *mut u8;
    if AnsiString.is_null() {
        return core::ptr::null_mut();
    }
    WideCharToMultiByte(65001, 0, WideString, -1, AnsiString as *mut i8, AnsiLen, core::ptr::null_mut(), core::ptr::null_mut());
    AnsiString
}

// ---- DnsUTF8ToW ----
unsafe fn DnsUTF8ToW(NarrowString: *const i8) -> *mut u16 {
    const CP_UTF8: u32 = 65001;

    let WideLen = MultiByteToWideChar(CP_UTF8, 0, NarrowString, -1, core::ptr::null_mut(), 0);
    if WideLen == 0 {
        return core::ptr::null_mut();
    }

    let WideString = RtlAllocateHeap(RtlGetProcessHeap(), 0, (WideLen as usize) * 2) as *mut u16;
    if WideString.is_null() {
        return core::ptr::null_mut();
    }

    MultiByteToWideChar(CP_UTF8, 0, NarrowString, -1, WideString, WideLen);
    WideString
}

// ---- xstrsave ----
unsafe fn xstrsave(str: *const u16) -> *mut u16 {
    let mut len: usize = 0;
    // FIXME: how much instead of MAX_PATH?
    StringCbLengthW(str, MAX_PATH, &mut len);
    len += core::mem::size_of::<u16>(); // include null terminator

    let p = RtlAllocateHeap(RtlGetProcessHeap(), 0, len) as *mut u16;
    if !p.is_null() {
        StringCbCopyW(p, len, str);
    }
    p
}

// ---- xstrsaveA ----
unsafe fn xstrsaveA(str: *const CHAR) -> *mut CHAR {
    let mut len: usize = 0;
    // FIXME: how much instead of MAX_PATH?
    StringCbLengthA(str, MAX_PATH, &mut len);
    len += 1;

    let p = RtlAllocateHeap(RtlGetProcessHeap(), 0, len) as *mut CHAR;

    if !p.is_null() {
        StringCbCopyA(p, len, str);
    }

    p
}

// ---- CheckForCurrentHostname ----
unsafe fn CheckForCurrentHostname(Name: *const CHAR, network_info: *const FIXED_INFO) -> IP4_ADDRESS {
    let mut TempName: PCHAR;
    let mut AdapterAddressesSize: DWORD;
    let mut Status: DWORD;
    let mut ret: IP4_ADDRESS = 0;
    let mut Address: IP4_ADDRESS = 0;
    let mut Addresses: PIP_ADAPTER_ADDRESSES = core::ptr::null_mut();
    let mut pip: PIP_ADAPTER_ADDRESSES;
    let mut Found: BOOL = 0;

    if (*network_info).DomainName[0] != 0 {
        let mut StringLength: usize = 0;
        let mut TempSize: usize = 2;
        StringCchLengthA((*network_info).HostName.as_ptr(), size_of_val(&(*network_info).HostName) as usize, &mut StringLength);
        TempSize += StringLength;
        StringCchLengthA((*network_info).DomainName.as_ptr(), size_of_val(&(*network_info).DomainName) as usize, &mut StringLength);
        TempSize += StringLength;
        TempName = RtlAllocateHeap(RtlGetProcessHeap(), 0, TempSize) as PCHAR;
        StringCchCopyA(TempName, TempSize, (*network_info).HostName.as_ptr());
        StringCchCatA(TempName, TempSize, b".\0".as_ptr() as PCHAR);
        StringCchCatA(TempName, TempSize, (*network_info).DomainName.as_ptr());
    } else {
        TempName = RtlAllocateHeap(RtlGetProcessHeap(), 0, 1) as PCHAR;
        *TempName = 0;
    }
    Found = if _stricmp(Name, (*network_info).HostName.as_ptr()) == 0 || _stricmp(Name, TempName) == 0 { 1 } else { 0 };
    RtlFreeHeap(RtlGetProcessHeap(), 0, TempName as *mut core::ffi::c_void);
    if Found == 0 {
        return 0;
    }
    // get adapter info
    AdapterAddressesSize = 0;
    GetAdaptersAddresses(
        AF_INET,
        GAA_FLAG_SKIP_FRIENDLY_NAME | GAA_FLAG_SKIP_DNS_SERVER |
        GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST,
        core::ptr::null_mut(),
        Addresses,
        &mut AdapterAddressesSize,
    );
    if AdapterAddressesSize == 0 {
        return 0;
    }
    Addresses = RtlAllocateHeap(RtlGetProcessHeap(), 0, AdapterAddressesSize as usize) as PIP_ADAPTER_ADDRESSES;
    Status = GetAdaptersAddresses(
        AF_INET,
        GAA_FLAG_SKIP_FRIENDLY_NAME | GAA_FLAG_SKIP_DNS_SERVER |
        GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST,
        core::ptr::null_mut(),
        Addresses,
        &mut AdapterAddressesSize,
    );
    if Status != 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Addresses as *mut core::ffi::c_void);
        return 0;
    }
    pip = Addresses;
    while pip != core::ptr::null_mut() {
        Address = (*((*(*pip).FirstUnicastAddress).Address.lpSockaddr as *const SOCKADDR_IN)).sin_addr.S_un.S_addr;
        if Address != ntohl(INADDR_LOOPBACK) {
            break;
        }
        pip = (*pip).Next;
    }
    if Address != 0 && Address != ntohl(INADDR_LOOPBACK) {
        ret = Address;
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, Addresses as *mut core::ffi::c_void);
    ret
}

// ---- DnsIntFreeRecordList ----
pub unsafe fn DnsIntFreeRecordList(mut ToDelete: PDNS_RECORD) {
    let mut next: PDNS_RECORD = core::ptr::null_mut();

    while !ToDelete.is_null() {
        if !(*ToDelete).pName.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, (*ToDelete).pName);
        }

        match (*ToDelete).wType as u32 {
            DNS_TYPE_CNAME
            | DNS_TYPE_PTR
            | DNS_TYPE_NS
            | DNS_TYPE_MB
            | DNS_TYPE_MD
            | DNS_TYPE_MF
            | DNS_TYPE_MG
            | DNS_TYPE_MR => {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*ToDelete).Data.PTR.as_ref().pNameHost);
            }
            DNS_TYPE_MINFO | DNS_TYPE_MX => {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*ToDelete).Data.MX.as_ref().pNameExchange);
            }
            DNS_TYPE_HINFO => {
                let count = (*ToDelete).Data.TXT.as_ref().dwStringCount as usize;
                for i in 0..count {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, *(*ToDelete).Data.TXT.as_ref().pStringArray.as_ptr().add(i));
                }
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*ToDelete).Data.TXT.as_mut().pStringArray.as_mut_ptr());
            }
            _ => {}
        }

        next = (*ToDelete).pNext;
        RtlFreeHeap(RtlGetProcessHeap(), 0, ToDelete);
        ToDelete = next;
    }
}

// ---- DnsFlushResolverCache ----
#[no_mangle]
pub unsafe extern "system" fn DnsFlushResolverCache() -> i32 {
    extern "system" {
        fn R_ResolverFlushCache(param: *mut core::ffi::c_void) -> u32;
    }
    let Status: u32 = rpc_try(|| R_ResolverFlushCache(core::ptr::null_mut())).unwrap_or_else(|code| code);
    (Status == 0) as i32
}

// ---- DnsFlushResolverCacheEntry_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsFlushResolverCacheEntry_W(pszEntry: *const u16) -> i32 {
    let mut status: i32 = 0;
    if pszEntry.is_null() {
        return 0;
    }
    status = rpc_try(|| R_ResolverFlushCacheEntry(core::ptr::null_mut(), pszEntry, DNS_TYPE_ANY as u16))
        .unwrap_or_else(|code| code) as i32;
    if status == 0 { 1 } else { 0 }
}

// ---- DnsGetCacheDataTable ----
#[no_mangle]
pub unsafe extern "system" fn DnsGetCacheDataTable(
    DnsCache: *mut *mut DNS_CACHE_ENTRY,
) -> i32 {
    if DnsCache.is_null() {
        return 0i32;
    }

    let mut CacheEntries: *mut DNS_CACHE_ENTRY = core::ptr::null_mut();
    let Status = rpc_try(|| CRrReadCache(core::ptr::null_mut(), &mut CacheEntries)).unwrap_or_else(|code| code as _);

    if Status != ERROR_SUCCESS {
        return 0i32;
    }

    if CacheEntries.is_null() {
        return 0i32;
    }

    *DnsCache = CacheEntries;
    1i32
}

// ---- GetCurrentTimeInSeconds ----
#[no_mangle]
pub unsafe extern "system" fn GetCurrentTimeInSeconds() -> DWORD {
    let mut time: FILETIME = core::mem::zeroed();
    let mut adjustment: FILETIME = core::mem::zeroed();
    let mut l_time: ULARGE_INTEGER = core::mem::zeroed();
    let mut l_adj: ULARGE_INTEGER = core::mem::zeroed();
    let st: SYSTEMTIME = SYSTEMTIME {
        wYear: 1970,
        wMonth: 1,
        wDayOfWeek: 0,
        wDay: 1,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };

    SystemTimeToFileTime(&st, &mut adjustment);
    core::ptr::copy_nonoverlapping(
        &adjustment as *const _ as *const u8,
        &mut l_adj as *mut _ as *mut u8,
        core::mem::size_of::<ULARGE_INTEGER>(),
    );
    GetSystemTimeAsFileTime(&mut time);
    core::ptr::copy_nonoverlapping(
        &time as *const _ as *const u8,
        &mut l_time as *mut _ as *mut u8,
        core::mem::size_of::<ULARGE_INTEGER>(),
    );
    l_time.QuadPart -= l_adj.QuadPart;
    (l_time.QuadPart / 10_000_000) as DWORD
}

// ---- CreateRecordForIpAddress ----
unsafe fn CreateRecordForIpAddress(
    Name: PCWSTR,
    Type: WORD,
) -> PDNS_RECORDW {
    let mut Ip4Address: IN_ADDR = core::mem::zeroed();
    let mut Ip6Address: IN6_ADDR = core::mem::zeroed();
    let mut pRecord: PDNS_RECORDW = core::ptr::null_mut();

    if Type == DNS_TYPE_A as u16 {
        if ParseIpv4Address(Name, &mut Ip4Address) != 0 {
            pRecord = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<DNS_RECORDW>(),
            ) as PDNS_RECORDW;
            if pRecord == core::ptr::null_mut() {
                return core::ptr::null_mut();
            }
            let name_len = wcslen(Name);
            (*pRecord).pName = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                (name_len + 1) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;
            if pRecord == core::ptr::null_mut() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, pRecord as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }
            wcscpy((*pRecord).pName, Name);
            (*pRecord).wType = DNS_TYPE_A as u16;
            (*pRecord).wDataLength = core::mem::size_of::<DNS_A_DATA>() as WORD;
            (*pRecord).Flags.S.as_mut().set_Section(DnsSectionQuestion);
            (*pRecord).Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
            (*pRecord).dwTtl = 7 * 24 * 60 * 60;
            (*pRecord).Data.A.as_mut().IpAddress = Ip4Address.S_un.S_addr;
            return pRecord;
        }
    } else if Type == DNS_TYPE_AAAA as u16 {
        if ParseIpv6Address(Name, &mut Ip6Address) != 0 {
            pRecord = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<DNS_RECORDW>(),
            ) as PDNS_RECORDW;
            if pRecord == core::ptr::null_mut() {
                return core::ptr::null_mut();
            }
            let name_len = wcslen(Name);
            (*pRecord).pName = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                (name_len + 1) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;
            if pRecord == core::ptr::null_mut() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, pRecord as *mut core::ffi::c_void);
                return core::ptr::null_mut();
            }
            wcscpy((*pRecord).pName, Name);
            (*pRecord).wType = DNS_TYPE_AAAA as u16;
            (*pRecord).wDataLength = core::mem::size_of::<DNS_AAAA_DATA>() as WORD;
            (*pRecord).Flags.S.as_mut().set_Section(DnsSectionQuestion);
            (*pRecord).Flags.S.as_mut().set_CharSet(DnsCharSetUnicode);
            (*pRecord).dwTtl = 7 * 24 * 60 * 60;
            core::ptr::copy_nonoverlapping(
                &Ip6Address.u.Byte as *const u8,
                &mut (*pRecord).Data.AAAA.as_mut().Ip6Address as *mut _ as *mut u8,
                core::mem::size_of::<IN6_ADDR>(),
            );
            return pRecord;
        }
    }

    core::ptr::null_mut()
}

// ---- DnsFlushResolverCacheEntry_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsFlushResolverCacheEntry_A(pszEntry: LPCSTR) -> BOOL {
    let mut Status: u32 = ERROR_SUCCESS;
    let pszUnicodeEntry: LPWSTR;

    if pszEntry.is_null() {
        return 0;
    }

    pszUnicodeEntry = DnsCToW(pszEntry);
    if pszUnicodeEntry.is_null() {
        return 0;
    }

    Status = rpc_try(|| R_ResolverFlushCacheEntry(core::ptr::null_mut(), pszUnicodeEntry, DNS_TYPE_ANY as u16))
        .unwrap_or_else(|code| code);

    RtlFreeHeap(RtlGetProcessHeap(), 0, pszUnicodeEntry as *mut core::ffi::c_void);

    if Status == ERROR_SUCCESS { 1 } else { 0 }
}

// ---- DnsFlushResolverCacheEntry_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsFlushResolverCacheEntry_UTF8(pszEntry: LPCSTR) -> BOOL {
    let mut Status: u32 = ERROR_SUCCESS;

    if pszEntry.is_null() {
        return 0i32;
    }

    let pszUnicodeEntry = DnsCToW(pszEntry);
    if pszUnicodeEntry.is_null() {
        return 0i32;
    }

    Status = rpc_try(|| R_ResolverFlushCacheEntry(core::ptr::null_mut(), pszUnicodeEntry, DNS_TYPE_ANY as u16))
        .unwrap_or_else(|code| code);

    RtlFreeHeap(RtlGetProcessHeap(), 0, pszUnicodeEntry as *mut core::ffi::c_void);
    (Status == ERROR_SUCCESS) as BOOL
}

// ---- Query_Main ----
#[no_mangle]
pub unsafe extern "system" fn Query_Main(
    Name: LPCWSTR,
    Type: WORD,
    Options: DWORD,
    QueryResultSet: *mut PDNS_RECORD,
) -> u32 {
    let mut astate: adns_state = core::mem::zeroed();
    let quflags: i32 = if (Options & DNS_QUERY_NO_RECURSION) == 0 {
        adns_qf_search as i32
    } else {
        0
    };
    let mut adns_error: i32 = 0;
    let mut answer: *mut adns_answer = core::ptr::null_mut();
    let mut CurrentName: LPSTR = core::ptr::null_mut();
    let mut CNameLoop: u32 = 0;
    let mut network_info: *mut FIXED_INFO = core::ptr::null_mut();
    let mut network_info_blen: ULONG = 0;
    let mut network_info_result: DWORD;
    let mut pip: *mut IP_ADDR_STRING = core::ptr::null_mut();
    let Address: IP4_ADDRESS;
    let mut addr: in_addr = core::mem::zeroed();
    let mut HostWithDomainName: PCHAR = core::ptr::null_mut();
    let mut AnsiName: PCHAR = core::ptr::null_mut();
    let mut NameLen: usize = 0;

    if Name.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if QueryResultSet.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *QueryResultSet = core::ptr::null_mut();

    match Type as u32 {
        DNS_TYPE_A => {
            // Convert Unicode name to ANSI
            NameLen = WideCharToMultiByte(
                CP_ACP,
                0,
                Name,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as usize;
            AnsiName = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                NameLen,
            ) as PCHAR;
            if AnsiName.is_null() {
                return ERROR_OUTOFMEMORY;
            }
            WideCharToMultiByte(
                CP_ACP,
                0,
                Name,
                -1,
                AnsiName,
                NameLen as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            NameLen -= 1;

            // Get network params
            network_info_result = GetNetworkParams(core::ptr::null_mut(), &mut network_info_blen);
            network_info = RtlAllocateHeap(RtlGetProcessHeap(), 0, network_info_blen as usize) as *mut FIXED_INFO;
            if network_info.is_null() {
                return ERROR_OUTOFMEMORY;
            }
            network_info_result = GetNetworkParams(network_info, &mut network_info_blen);
            if network_info_result != ERROR_SUCCESS {
                RtlFreeHeap(RtlGetProcessHeap(), 0, network_info as *mut core::ffi::c_void);
                return network_info_result;
            }

            // Check if it's the current hostname
            let check_name: PCHAR = if NameLen != 0 {
                AnsiName
            } else {
                (*network_info).HostName.as_mut_ptr()
            };
            Address = CheckForCurrentHostname(check_name, network_info);
            if Address != 0 {
                // Build host+domain string
                let mut TempLen: usize = 2;
                let mut StringLength: usize = 0;
                RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);

                // Need sizeof arrays here; assume 256 for HostName and DomainName
                const HOSTNAME_SIZE: usize = 256;
                const DOMAINNAME_SIZE: usize = 256;
                StringCchLengthA(
                    (*network_info).HostName.as_mut_ptr(),
                    HOSTNAME_SIZE,
                    &mut StringLength,
                );
                TempLen += StringLength;
                StringCchLengthA(
                    (*network_info).DomainName.as_mut_ptr(),
                    DOMAINNAME_SIZE,
                    &mut StringLength,
                );
                TempLen += StringLength;

                HostWithDomainName = RtlAllocateHeap(RtlGetProcessHeap(), 0, TempLen) as PCHAR;
                StringCchCopyA(
                    HostWithDomainName,
                    TempLen,
                    (*network_info).HostName.as_mut_ptr(),
                );
                if (*network_info).DomainName[0] != 0 {
                    StringCchCatA(HostWithDomainName, TempLen, b".\0".as_ptr() as PCHAR);
                    StringCchCatA(
                        HostWithDomainName,
                        TempLen,
                        (*network_info).DomainName.as_mut_ptr(),
                    );
                }
                RtlFreeHeap(RtlGetProcessHeap(), 0, network_info as *mut core::ffi::c_void);

                *QueryResultSet = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    core::mem::size_of::<DNS_RECORD>(),
                ) as PDNS_RECORD;
                if (*QueryResultSet).is_null() {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, HostWithDomainName as *mut core::ffi::c_void);
                    return ERROR_OUTOFMEMORY;
                }

                (*(*QueryResultSet)).pNext = core::ptr::null_mut();
                (*(*QueryResultSet)).wType = Type;
                (*(*QueryResultSet)).wDataLength = core::mem::size_of::<DNS_A_DATA>() as u16;
                (*(*QueryResultSet)).Flags.S.as_mut().set_Section(DnsSectionAnswer as u32);
                (*(*QueryResultSet)).Flags.S.as_mut().set_CharSet(DnsCharSetUnicode as u32);
                (*(*QueryResultSet)).Data.A.as_mut().IpAddress = Address;

                (*(*QueryResultSet)).pName = DnsCToW(HostWithDomainName) as LPWSTR;
                RtlFreeHeap(RtlGetProcessHeap(), 0, HostWithDomainName as *mut core::ffi::c_void);
                if (*(*QueryResultSet)).pName.is_null() {
                    return ERROR_OUTOFMEMORY;
                }
                return ERROR_SUCCESS;
            }

            // Check no wire query flag
            if (Options & DNS_QUERY_NO_WIRE_QUERY) != 0 {
                RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
                RtlFreeHeap(RtlGetProcessHeap(), 0, network_info as *mut core::ffi::c_void);
                return ERROR_FILE_NOT_FOUND;
            }

            // Initialize adns
            adns_error = adns_init(
                &mut astate,
                adns_if_noenv | adns_if_noerrprint | adns_if_noserverwarn,
                core::ptr::null_mut(),
            );
            if adns_error != adns_s_ok as i32 {
                RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
                RtlFreeHeap(RtlGetProcessHeap(), 0, network_info as *mut core::ffi::c_void);
                return DnsIntTranslateAdnsToDNS_STATUS(adns_error);
            }

            // Add DNS servers from network info
            pip = &mut (*network_info).DnsServerList;
            while !pip.is_null() {
                addr.S_un.S_addr = inet_addr((*pip).IpAddress.String.as_ptr() as LPCSTR);
                if addr.S_un.S_addr != INADDR_ANY && addr.S_un.S_addr != INADDR_NONE {
                    adns_addserver(astate, addr);
                }
                pip = (*pip).Next;
            }
            if (*network_info).DomainName[0] != 0 {
                adns_ccf_search(
                    astate,
                    b"LOCALDOMAIN\0".as_ptr() as LPCSTR,
                    -1,
                    (*network_info).DomainName.as_mut_ptr(),
                );
            }
            RtlFreeHeap(RtlGetProcessHeap(), 0, network_info as *mut core::ffi::c_void);

            if adns_numservers(astate) == 0 {
                adns_finish(astate);
                RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
                return ERROR_FILE_NOT_FOUND;
            }

            // Resolve with CNAME chain loop
            CurrentName = AnsiName;
            const CNAME_LOOP_MAX: u32 = 16;
            CNameLoop = 0;
            loop {
                if CNameLoop >= CNAME_LOOP_MAX {
                    break;
                }
                adns_error = adns_synchronous(
                    astate,
                    CurrentName,
                    adns_r_addr,
                    quflags as u32,
                    &mut answer,
                );
                if adns_error != adns_s_ok as i32 {
                    adns_finish(astate);
                    if CurrentName != AnsiName {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentName as *mut core::ffi::c_void);
                    }
                    RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
                    return DnsIntTranslateAdnsToDNS_STATUS(adns_error);
                }

                if !answer.is_null() && !((*answer).rrs.addr).is_null() {
                    if CurrentName != AnsiName {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentName as *mut core::ffi::c_void);
                    }
                    RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);

                    *QueryResultSet = RtlAllocateHeap(
                        RtlGetProcessHeap(),
                        0,
                        core::mem::size_of::<DNS_RECORD>(),
                    ) as PDNS_RECORD;
                    if (*QueryResultSet).is_null() {
                        adns_finish(astate);
                        return ERROR_OUTOFMEMORY;
                    }

                    (*(*QueryResultSet)).pNext = core::ptr::null_mut();
                    (*(*QueryResultSet)).wType = Type;
                    (*(*QueryResultSet)).wDataLength = core::mem::size_of::<DNS_A_DATA>() as u16;
                    (*(*QueryResultSet)).Flags.S.as_mut().set_Section(DnsSectionAnswer as u32);
                    (*(*QueryResultSet)).Flags.S.as_mut().set_CharSet(DnsCharSetUnicode as u32);
                    (*(*QueryResultSet)).Data.A.as_mut().IpAddress =
                        (*(*answer).rrs.addr).addr.inet.as_ref().sin_addr.S_un.S_addr;

                    adns_finish(astate);
                    (*(*QueryResultSet)).pName = xstrsave(Name) as LPWSTR;
                    if (*(*QueryResultSet)).pName.is_null() {
                        return ERROR_OUTOFMEMORY;
                    }
                    return ERROR_SUCCESS;
                }

                if answer.is_null()
                    || (*answer).status != adns_s_prohibitedcname as u32
                    || (*answer).cname.is_null()
                {
                    adns_finish(astate);
                    if CurrentName != AnsiName {
                        RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentName as *mut core::ffi::c_void);
                    }
                    RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
                    return ERROR_FILE_NOT_FOUND;
                }

                if CurrentName != AnsiName {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentName as *mut core::ffi::c_void);
                }
                CurrentName = xstrsaveA((*answer).cname);
                if CurrentName.is_null() {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
                    adns_finish(astate);
                    return ERROR_OUTOFMEMORY;
                }
                CNameLoop += 1;
            }

            adns_finish(astate);
            RtlFreeHeap(RtlGetProcessHeap(), 0, AnsiName as *mut core::ffi::c_void);
            RtlFreeHeap(RtlGetProcessHeap(), 0, CurrentName as *mut core::ffi::c_void);
            return ERROR_FILE_NOT_FOUND;
        }
        _ => {
            return ERROR_OUTOFMEMORY;
        }
    }
}

// ---- DnsQuery_W ----
#[no_mangle]
pub unsafe extern "system" fn DnsQuery_W(
    Name: LPCWSTR,
    Type: WORD,
    Options: DWORD,
    Extra: PVOID,
    QueryResultSet: *mut PDNS_RECORD,
    Reserved: *mut PVOID,
) -> DNS_STATUS {
    let mut dwRecords: DWORD = 0;
    let mut pRecord: PDNS_RECORDW = core::ptr::null_mut();
    let NameLen: usize;
    let mut i: usize;
    let mut Status: DNS_STATUS = ERROR_SUCCESS;

    if Name.is_null() || QueryResultSet.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *QueryResultSet = core::ptr::null_mut();

    pRecord = CreateRecordForIpAddress(Name, Type);
    if !pRecord.is_null() {
        *QueryResultSet = pRecord as PDNS_RECORD;
        return ERROR_SUCCESS;
    }

    NameLen = wcslen(Name) as usize;
    if NameLen == 0 {
        return ERROR_INVALID_NAME;
    }

    if (*Name as u16) == ('-' as u16) || (*Name as u16) == ('_' as u16) ||
       (*Name.add(NameLen - 1) as u16) == ('-' as u16) || (*Name.add(NameLen - 1) as u16) == ('_' as u16)
    {
        return ERROR_INVALID_NAME;
    }

    let dotdot: [u16; 3] = [0x002Eu16, 0x002Eu16, 0x0000u16];
    if !wcsstr(Name, dotdot.as_ptr()).is_null() {
        return ERROR_INVALID_NAME;
    }

    i = 0;
    while i < NameLen {
        let c = *Name.add(i) as u16;
        if !((c >= ('a' as u16) && c <= ('z' as u16)) ||
             (c >= ('A' as u16) && c <= ('Z' as u16)) ||
             (c >= ('0' as u16) && c <= ('9' as u16)) ||
             c == ('-' as u16) || c == ('_' as u16) || c == ('.' as u16))
        {
            return DNS_ERROR_INVALID_NAME_CHAR;
        }
        i += 1;
    }

    // RpcTryExcept, as the C: before the DNS Client service is up the stub raises
    // RPC_S_SERVER_UNAVAILABLE, and that is this call's answer, not the caller's death.
    Status = rpc_try(|| {
        rpc_try(|| R_ResolverQuery(
            core::ptr::null_mut(),
            Name,
            Type,
            Options,
            &mut dwRecords as *mut DWORD,
            QueryResultSet as *mut *mut DNS_RECORDW,
        )).unwrap_or_else(|code| code as _)
    })
    .unwrap_or_else(|code| code);

    Status
}

// ---- DnsQuery_CodePage ----
#[no_mangle]
pub unsafe extern "system" fn DnsQuery_CodePage(
    CodePage: u32,
    Name: LPCSTR,
    Type: u16,
    Options: u32,
    Extra: *mut core::ffi::c_void,
    mut QueryResultSet: *mut PDNS_RECORD,
    Reserved: *mut *mut core::ffi::c_void,
) -> u32 {
    let mut i: u32;
    let mut Buffer: *mut u16 = core::ptr::null_mut();
    let mut Status: u32;
    let mut QueryResultWide: PDNS_RECORD = core::ptr::null_mut();
    let mut ConvertedRecord: PDNS_RECORD = core::ptr::null_mut();
    let mut LastRecord: PDNS_RECORD = core::ptr::null_mut();

    if Name.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if QueryResultSet.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    match CodePage {
        CP_ACP => Buffer = DnsCToW(Name),
        CP_UTF8 => Buffer = DnsUTF8ToW(Name),
        _ => return ERROR_INVALID_PARAMETER,
    }

    Status = DnsQuery_W(Buffer, Type, Options, Extra, &mut QueryResultWide, Reserved);

    while Status == ERROR_SUCCESS && !QueryResultWide.is_null() {
        match (*QueryResultWide).wType as u32 {
            DNS_TYPE_A | DNS_TYPE_WKS | DNS_TYPE_CNAME | DNS_TYPE_PTR |
            DNS_TYPE_NS | DNS_TYPE_MB | DNS_TYPE_MD | DNS_TYPE_MF |
            DNS_TYPE_MG | DNS_TYPE_MR => {
                ConvertedRecord = RtlAllocateHeap(RtlGetProcessHeap(), 0, core::mem::size_of::<DNS_RECORD>()) as PDNS_RECORD;
            }
            DNS_TYPE_MINFO => {
                ConvertedRecord = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    core::mem::size_of::<DNS_TXT_DATA>() + (*QueryResultWide).Data.TXT.as_ref().dwStringCount as usize
                ) as PDNS_RECORD;
            }
            DNS_TYPE_NULL => {
                ConvertedRecord = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    core::mem::size_of::<DNS_NULL_DATA>() + (*QueryResultWide).Data.Null.as_ref().dwByteCount as usize
                ) as PDNS_RECORD;
            }
            _ => {}
        }

        if ConvertedRecord.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as *mut core::ffi::c_void);
            DnsIntFreeRecordList(QueryResultWide);
            QueryResultSet = core::ptr::null_mut();
            return ERROR_OUTOFMEMORY;
        }

        if CodePage == CP_ACP {
            (*ConvertedRecord).pName = DnsWToC((*QueryResultWide).pName as *mut u16) as LPWSTR;
            (*ConvertedRecord).Flags.S.as_mut().set_CharSet(DnsCharSetAnsi);
        } else {
            (*ConvertedRecord).pName = DnsWToUTF8((*QueryResultWide).pName as *mut u16) as LPWSTR;
            (*ConvertedRecord).Flags.S.as_mut().set_CharSet(DnsCharSetUtf8);
        }

        (*ConvertedRecord).wType = (*QueryResultWide).wType;

        match (*QueryResultWide).wType as u32 {
            DNS_TYPE_A | DNS_TYPE_WKS => {
                (*ConvertedRecord).wDataLength = (*QueryResultWide).wDataLength;
                memcpy(
                    core::ptr::addr_of_mut!((*ConvertedRecord).Data).cast::<core::ffi::c_void>(),
                    core::ptr::addr_of!((*QueryResultWide).Data).cast::<core::ffi::c_void>(),
                    (*QueryResultWide).wDataLength as usize,
                );
            }
            DNS_TYPE_CNAME | DNS_TYPE_PTR | DNS_TYPE_NS | DNS_TYPE_MB |
            DNS_TYPE_MD | DNS_TYPE_MF | DNS_TYPE_MG | DNS_TYPE_MR => {
                (*ConvertedRecord).wDataLength = core::mem::size_of::<DNS_PTR_DATA>() as u16;
                if CodePage == CP_ACP {
                    (*ConvertedRecord).Data.PTR.as_mut().pNameHost = DnsWToC((*QueryResultWide).Data.PTR.as_ref().pNameHost as *mut u16) as LPWSTR;
                } else {
                    (*ConvertedRecord).Data.PTR.as_mut().pNameHost = DnsWToUTF8((*QueryResultWide).Data.PTR.as_ref().pNameHost as *mut u16) as LPWSTR;
                }
            }
            DNS_TYPE_MINFO => {
                (*ConvertedRecord).wDataLength = core::mem::size_of::<DNS_MINFO_DATA>() as u16;
                if CodePage == CP_ACP {
                    (*ConvertedRecord).Data.MINFO.as_mut().pNameMailbox = DnsWToC((*QueryResultWide).Data.MINFO.as_ref().pNameMailbox as *mut u16) as LPWSTR;
                    (*ConvertedRecord).Data.MINFO.as_mut().pNameErrorsMailbox = DnsWToC((*QueryResultWide).Data.MINFO.as_ref().pNameErrorsMailbox as *mut u16) as LPWSTR;
                } else {
                    (*ConvertedRecord).Data.MINFO.as_mut().pNameMailbox = DnsWToUTF8((*QueryResultWide).Data.MINFO.as_ref().pNameMailbox as *mut u16) as LPWSTR;
                    (*ConvertedRecord).Data.MINFO.as_mut().pNameErrorsMailbox = DnsWToUTF8((*QueryResultWide).Data.MINFO.as_ref().pNameErrorsMailbox as *mut u16) as LPWSTR;
                }
            }
            DNS_TYPE_MX => {
                (*ConvertedRecord).wDataLength = core::mem::size_of::<DNS_MX_DATA>() as u16;
                if CodePage == CP_ACP {
                    (*ConvertedRecord).Data.MX.as_mut().pNameExchange = DnsWToC((*QueryResultWide).Data.MX.as_ref().pNameExchange as *mut u16) as LPWSTR;
                } else {
                    (*ConvertedRecord).Data.MX.as_mut().pNameExchange = DnsWToUTF8((*QueryResultWide).Data.MX.as_ref().pNameExchange as *mut u16) as LPWSTR;
                }
                (*ConvertedRecord).Data.MX.as_mut().wPreference = (*QueryResultWide).Data.MX.as_ref().wPreference;
            }
            DNS_TYPE_HINFO => {
                (*ConvertedRecord).wDataLength = (core::mem::size_of::<DNS_TXT_DATA>() as u32
                    + core::mem::size_of::<*mut core::ffi::c_void>() as u32 * (*QueryResultWide).Data.TXT.as_ref().dwStringCount) as u16;
                (*ConvertedRecord).Data.TXT.as_mut().dwStringCount = (*QueryResultWide).Data.TXT.as_ref().dwStringCount;
                let src_array = (*QueryResultWide).Data.TXT.as_ref().pStringArray.as_ptr();
                let dest_array = (*ConvertedRecord).Data.TXT.as_mut().pStringArray.as_mut_ptr();
                for i in 0..(*ConvertedRecord).Data.TXT.as_ref().dwStringCount {
                    let src_ptr = *src_array.add(i as usize) as *mut u16;
                    let dest_ptr = if CodePage == CP_ACP {
                        DnsWToC(src_ptr) as LPWSTR
                    } else {
                        DnsWToUTF8(src_ptr) as LPWSTR
                    };
                    *dest_array.add(i as usize) = dest_ptr;
                }
            }
            DNS_TYPE_NULL => {
                (*ConvertedRecord).wDataLength = (core::mem::size_of::<DNS_NULL_DATA>() + (*QueryResultWide).Data.Null.as_ref().dwByteCount as usize) as u16;
                (*ConvertedRecord).Data.Null.as_mut().dwByteCount = (*QueryResultWide).Data.Null.as_ref().dwByteCount;
                memcpy(
                    core::ptr::addr_of_mut!((*ConvertedRecord).Data.Null.as_mut().Data).cast::<core::ffi::c_void>(),
                    core::ptr::addr_of!((*QueryResultWide).Data.Null.as_ref().Data).cast::<core::ffi::c_void>(),
                    (*QueryResultWide).Data.Null.as_ref().dwByteCount as usize,
                );
            }
            _ => {}
        }

        if LastRecord.is_null() {
            LastRecord = ConvertedRecord;
            *QueryResultSet = ConvertedRecord;
        } else {
            (*LastRecord).pNext = ConvertedRecord;
            LastRecord = (*LastRecord).pNext;
        }

        QueryResultWide = (*QueryResultWide).pNext;
    }

    if !LastRecord.is_null() {
        (*LastRecord).pNext = core::ptr::null_mut();
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as *mut core::ffi::c_void);
    if !QueryResultWide.is_null() {
        DnsIntFreeRecordList(QueryResultWide);
    }

    Status
}

// ---- DnsQuery_A ----
#[no_mangle]
pub unsafe extern "system" fn DnsQuery_A(
    Name: LPCSTR,
    Type: u16,
    Options: u32,
    Extra: *mut core::ffi::c_void,
    mut QueryResultSet: *mut PDNS_RECORD,
    Reserved: *mut *mut core::ffi::c_void,
) -> u32 {
    DnsQuery_CodePage(CP_ACP, Name, Type, Options, Extra, QueryResultSet, Reserved)
}

// ---- DnsQuery_UTF8 ----
#[no_mangle]
pub unsafe extern "system" fn DnsQuery_UTF8(
    Name: LPCSTR,
    Type: WORD,
    Options: DWORD,
    Extra: PVOID,
    QueryResultSet: *mut PDNS_RECORD,
    Reserved: *mut PVOID,
) -> DNS_STATUS {
    DnsQuery_CodePage(65001u32, Name, Type, Options, Extra, QueryResultSet, Reserved)
}

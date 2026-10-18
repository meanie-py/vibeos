use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::mswsock::mswhelper::strlen;
use crate::mswsock::mswhelper::HeapAlloc;
use core::ffi::c_void;

// Win32 API + DNS API: stdcall on i386 (extern "system" → `_Name@N`).
extern "system" {
    fn WSASetLastError(iError: INT);
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(hHeap: HANDLE, dwFlags: u32, lpMem: LPVOID) -> i32;
    fn GetComputerNameExW(NameType: u32, lpBuffer: PWSTR, nSize: LPDWORD) -> i32;
    fn DnsQuery_W( pszName: PCWSTR, wType: WORD, Options: DWORD, pExtra: PVOID, ppQueryResults: *mut PDNS_RECORDW, pReserved: *mut PVOID, ) -> DNS_STATUS;
    fn DnsRecordListFree(pRecordList: PDNS_RECORDW, FreeType: ::core::ffi::c_int);
    fn CreateFileW( lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: LPVOID, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE, ) -> HANDLE;
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: u32) -> u32;
    fn RegCloseKey(hKey: HKEY) -> i32;
    fn RegOpenKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: DWORD, phkResult: *mut HKEY, ) -> i32;
    fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> i32;
    fn WSAGetLastError() -> i32;
    fn CloseHandle(hObject: HANDLE) -> i32;
    fn ReadFile( hFile: HANDLE, lpBuffer: LPVOID, nNumberOfBytesToRead: DWORD, lpNumberOfBytesRead: LPDWORD, lpOverlapped: *mut core::ffi::c_void, ) -> i32;
    // Removed — NO linkable symbol, provided as inline shims in the prelude instead:
    // StringCbCopyA / StringCchCat / StringCchLength (strsafe.h `static inline`s) and
    // IsEqualGUID (a guiddef.h inline). The Reg*/CreateFile/GetSystemDirectory/
    // ExpandEnvironmentStrings decls above were renamed to their real …W symbols (UNICODE macros).
}
// mswsock-internal helpers: DEFINED in mswhelper.rs as `#[no_mangle] extern "C"` (cdecl, per the
// plain-C mswhelper.h — no WINAPI). Declaring them `extern "system"` here is the exact
// stdcall-vs-cdecl decoration mismatch the sister prelude warns about ("one declaration, one
// truth") — silent on amd64/arm64, a `_Name@N` link break on i386. They are cdecl → extern "C".
extern "C" {
    fn StrA2WHeapAlloc(hHeap: HANDLE, str: *mut i8) -> PWSTR;
    fn StrCpyHeapAllocW(hHeap: HANDLE, str: PWSTR) -> PWSTR;
    fn StrAryCpyHeapAllocWToA(hHeap: HANDLE, src: *mut *mut WCHAR) -> *mut *mut CHAR;
    fn StrAryCpyHeapAllocA(hHeap: HANDLE, ary: *mut PCHAR) -> *mut PCHAR;
    fn StrW2AHeapAlloc(hHeap: HANDLE, str: PWSTR) -> PCHAR;
    fn mswBufferInit(buf: *mut MSW_BUFFER, buffer: LPBYTE, size: DWORD);
    fn mswBufferIncUsed(buf: *mut MSW_BUFFER, bytesToInc: DWORD);
    fn mswBufferEndPtr(buf: *mut MSW_BUFFER) -> LPVOID;
    fn mswBufferAppendStrW(buf: *mut MSW_BUFFER, str: LPCWSTR) -> i32;
    fn mswBufferAppendAddr_AddrInfoW(buf: *mut MSW_BUFFER, lpRes: LPWSAQUERYSETW, addr4: DWORD) -> i32;
    fn mswBufferAppendBlob_Hostent(buf: *mut MSW_BUFFER, lpRes: LPWSAQUERYSETW, aliases: *mut *mut i8, hostname: *mut i8, addr4: DWORD) -> i32;
    fn mswBufferAppendBlob_Servent(buf: *mut MSW_BUFFER, lpRes: LPWSAQUERYSETW, servname: *mut i8, aliases: *mut *mut i8, proto: *mut i8, port: u16) -> i32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut guid_NULL: GUID = _GUID {
    Data1: 0 as ::core::ffi::c_uint,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

static mut guid_HOSTNAME: GUID = _GUID {
    Data1: 0x2a800 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut guid_INET_HOSTADDRBYINETSTRING: GUID = _GUID {
    Data1: 0x2a801 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut guid_INET_HOSTADDRBYNAME: GUID = _GUID {
    Data1: 0x2a803 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut guid_INET_SERVICEBYNAME: GUID = _GUID {
    Data1: 0x2a802 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0 as ::core::ffi::c_ushort,
    Data4: [
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut guid_mswsock_TcpIp: GUID = _GUID {
    Data1: 0x22059d40 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7e9e as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut guid_mswsock_NLA: GUID = _GUID {
    Data1: 0x6642243a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x3ba8 as ::core::ffi::c_ushort,
    Data3: 0x4aa6 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub const NSP_CALLID_DNS: u32 = 0x0001;
pub const NSP_CALLID_HOSTNAME: u32 = 0x0002;
pub const NSP_CALLID_HOSTBYNAME: u32 = 0x0003;
pub const NSP_CALLID_SERVICEBYNAME: u32 = 0x0004;
pub const WS2_INTERNAL_MAX_ALIAS: u32 = 512;

#[repr(C)]
pub struct WSHOSTINFOINTERN {
    pub hostnameW: PWSTR,
    pub addr4: DWORD,
    pub servnameW: PWSTR,
    pub servprotoW: PWSTR,
    pub servaliasesA: *mut PCHAR,
    pub servport: WORD,
}
pub type PWSHOSTINFOINTERN = *mut WSHOSTINFOINTERN;

#[repr(C)]
pub struct WSHANDLEINTERN {
    pub providerId: GUID,
    pub dwControlFlags: DWORD,
    pub CallID: DWORD,
    pub CallIDCounter: DWORD,
    pub hostnameW: PWSTR,
}
pub type PWSHANDLEINTERN = *mut WSHANDLEINTERN;

#[no_mangle]
pub unsafe extern "system" fn mwsNSPCleanUp(lpProviderId: LPGUID) -> INT {
    return ERROR_SUCCESS as INT;
}
#[no_mangle]
pub unsafe extern "C" fn mwsNSPInit() -> INT {
    return ERROR_SUCCESS as INT;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPLookupServiceBegin(
    lpProviderId: LPGUID,
    lpqsRestrictions: LPWSAQUERYSETW,
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
    dwControlFlags: u32,
    lphLookup: LPHANDLE,
) -> i32 {

    let pLook: PWSHANDLEINTERN;
    let mut wsaErr: i32;

    if IsEqualGUID(lpProviderId, core::ptr::addr_of!(guid_mswsock_TcpIp)) != 0 {
    } else if IsEqualGUID(lpProviderId, core::ptr::addr_of!(guid_mswsock_NLA)) != 0 {
        WSASetLastError(WSASERVICE_NOT_FOUND as i32);
        return SOCKET_ERROR;
    } else {
        return ERROR_CALL_NOT_IMPLEMENTED as i32;
    }

    pLook = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WSHANDLEINTERN>()) as SIZE_T,
    ) as PWSHANDLEINTERN;
    if pLook.is_null() {
        WSASetLastError(WSAEFAULT as i32);
        return SOCKET_ERROR;
    }

    *lphLookup = pLook as *mut core::ffi::c_void;

    RtlZeroMemory(
        pLook as *mut core::ffi::c_void,
        core::mem::size_of::<WSHANDLEINTERN>(),
    );

    (*pLook).dwControlFlags = dwControlFlags;
    (*pLook).providerId = *lpProviderId;

    wsaErr = ERROR_CALL_NOT_IMPLEMENTED as i32;
    if IsEqualGUID((*lpqsRestrictions).lpServiceClassId, core::ptr::addr_of!(guid_NULL)) != 0 {
        wsaErr = ERROR_CALL_NOT_IMPLEMENTED as i32;
    } else if IsEqualGUID((*lpqsRestrictions).lpServiceClassId, core::ptr::addr_of!(guid_HOSTNAME)) != 0 {
        wsaErr = NSP_LookupServiceBeginW(
            pLook,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            NSP_CALLID_HOSTNAME,
        );
    } else if IsEqualGUID((*lpqsRestrictions).lpServiceClassId, core::ptr::addr_of!(guid_INET_HOSTADDRBYNAME)) != 0 {
        wsaErr = NSP_LookupServiceBeginW(
            pLook,
            core::ptr::null_mut(),
            (*lpqsRestrictions).lpszServiceInstanceName,
            NSP_CALLID_HOSTBYNAME,
        );
    } else if IsEqualGUID((*lpqsRestrictions).lpServiceClassId, core::ptr::addr_of!(guid_INET_SERVICEBYNAME)) != 0 {
        wsaErr = NSP_LookupServiceBeginW(
            pLook,
            core::ptr::null_mut(),
            (*lpqsRestrictions).lpszServiceInstanceName,
            NSP_CALLID_SERVICEBYNAME,
        );
    } else if IsEqualGUID((*lpqsRestrictions).lpServiceClassId, core::ptr::addr_of!(guid_INET_HOSTADDRBYINETSTRING)) != 0 {
        wsaErr = ERROR_CALL_NOT_IMPLEMENTED as i32;
    }

    if wsaErr != NO_ERROR as i32 {
        WSASetLastError(wsaErr);
        return SOCKET_ERROR;
    }
    return NO_ERROR as i32;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPLookupServiceNext(
    hLookup: HANDLE,
    dwControlFlags: DWORD,
    lpdwBufferLength: LPDWORD,
    lpqsResults: LPWSAQUERYSETW,
) -> INT {
    let pLook: PWSHANDLEINTERN = hLookup as PWSHANDLEINTERN;
    let mut wsaErr: ::core::ffi::c_int = 0;

    if lpdwBufferLength.is_null() || *lpdwBufferLength == 0 {
        wsaErr = WSA_NOT_ENOUGH_MEMORY as INT;
    } else {
        core::ptr::write_bytes(
            lpqsResults as *mut ::core::ffi::c_void,
            0,
            *lpdwBufferLength as usize,
        );
        (*lpqsResults).dwSize = core::mem::size_of::<_WSAQuerySetW>() as DWORD;
        wsaErr = NSP_LookupServiceNextW(pLook, dwControlFlags, lpqsResults, lpdwBufferLength);
    }

    if wsaErr != 0 {
        WSASetLastError(wsaErr);
        return SOCKET_ERROR;
    }
    return NO_ERROR as INT;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPIoCtl(
    hLookup: HANDLE,
    dwControlCode: DWORD,
    lpvInBuffer: LPVOID,
    cbInBuffer: DWORD,
    lpvOutBuffer: LPVOID,
    cbOutBuffer: DWORD,
    lpcbBytesReturned: LPDWORD,
    lpCompletion: LPWSACOMPLETION,
    lpThreadId: LPWSATHREADID,
) -> INT {
    WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED as INT);
    return ERROR_CALL_NOT_IMPLEMENTED as INT;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPLookupServiceEnd(hLookup: HANDLE) -> INT {
    let pLook: PWSHANDLEINTERN;
    let hHeap: HANDLE;
    let res: INT;

    res = NO_ERROR as INT;
    pLook = hLookup as PWSHANDLEINTERN;
    hHeap = GetProcessHeap();

    if !(*pLook).hostnameW.is_null() {
        HeapFree(hHeap, 0, (*pLook).hostnameW as LPVOID);
    }

    HeapFree(hHeap, 0, pLook as LPVOID);
    return res;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPSetService(
    lpProviderId: LPGUID,
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
    lpqsRegInfo: LPWSAQUERYSETW,
    essOperation: WSAESETSERVICEOP,
    dwControlFlags: DWORD,
) -> INT {
    WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED as INT);
    return ERROR_CALL_NOT_IMPLEMENTED as INT;
}
#[no_mangle]
pub unsafe extern "system" fn mwsNSPInstallServiceClass(
    lpProviderId: LPGUID,
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
) -> INT {
    WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED as INT);
    return ERROR_CALL_NOT_IMPLEMENTED as INT;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPRemoveServiceClass(
    lpProviderId: LPGUID,
    lpServiceClassId: LPGUID,
) -> INT {
    WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED as INT);
    return ERROR_CALL_NOT_IMPLEMENTED as INT;
}

#[no_mangle]
pub unsafe extern "system" fn mwsNSPGetServiceClassInfo(
    lpProviderId: LPGUID,
    lpdwBufSize: LPDWORD,
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
) -> INT {
    
    WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED as INT);
    return ERROR_CALL_NOT_IMPLEMENTED as INT;
}
#[no_mangle]
pub unsafe extern "C" fn NSP_LookupServiceBeginW(
    data: PWSHANDLEINTERN,
    hostnameA: *mut i8,
    hostnameW: PWSTR,
    CallID: u32,
) -> i32 {
    let mut hHeap: HANDLE;

    if (*data).CallID != 0 {
        return WSAEFAULT as INT;
    }

    (*data).CallID = CallID;

    if (CallID == NSP_CALLID_HOSTBYNAME) ||
       (CallID == NSP_CALLID_SERVICEBYNAME)
    {
        hHeap = GetProcessHeap();

        if !(*data).hostnameW.is_null() {
            HeapFree(hHeap, 0, (*data).hostnameW as LPVOID);
        }

        if !hostnameA.is_null() {
            (*data).hostnameW = StrA2WHeapAlloc(hHeap, hostnameA);
        } else {
            (*data).hostnameW = StrCpyHeapAllocW(hHeap, hostnameW);
        }
    }

    WSASetLastError(0);

    return ERROR_SUCCESS as INT;
}
#[no_mangle]
pub unsafe extern "C" fn NSP_GetHostNameHeapAllocW(hostname: *mut PWSTR) -> i32 {

    let hHeap = GetProcessHeap();
    let mut bufCharLen: u32 = (MAX_COMPUTERNAME_LENGTH + 1) as u32;
    let bufByteLen: u32 = bufCharLen * core::mem::size_of::<u16>() as u32;

    let name = HeapAlloc(hHeap, 0, (bufByteLen) as SIZE_T) as PWSTR;

    if GetComputerNameExW(ComputerNameDnsHostname, name, &mut bufCharLen) == 0 {
        HeapFree(hHeap, 0, name as *mut core::ffi::c_void);
        WSASetLastError(WSAEFAULT as i32);
        return -1;
    }

    *hostname = name;
    return ERROR_SUCCESS as i32;
}
#[no_mangle]
pub unsafe extern "C" fn NSP_GetHostByNameHeapAllocW(
    data: PWSHANDLEINTERN,
    dwControlFlags: DWORD,
    hostinfo: PWSHOSTINFOINTERN,
) -> INT {

    let hHeap: HANDLE = GetProcessHeap();
    let mut dns_status: DNS_STATUS = 0;
    let mut dp: PDNS_RECORDW = core::ptr::null_mut();
    let mut curr: PDNS_RECORDW;
    let mut result: INT = ERROR_SUCCESS as INT;
    let mut dwQueryFlags: DWORD = DNS_QUERY_STANDARD;
    let mut Aliases: [PWCHAR; 512] = [core::ptr::null_mut(); 512];
    let mut AliasIndex: ::core::ffi::c_int = 0;

    if (*data).hostnameW.is_null() {
        result = ERROR_INVALID_PARAMETER as INT;
    } else {
        if (*data).dwControlFlags & LUP_DEEP == 0 {
            dwQueryFlags |= DNS_QUERY_NO_RECURSION;
        }

        dns_status = DnsQuery_W(
            (*data).hostnameW,
            DNS_TYPE_A as u16,
            dwQueryFlags,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dp),
            core::ptr::null_mut(),
        );

        if dns_status == ERROR_INVALID_NAME {
            WSASetLastError(WSAEFAULT as INT);
            result = ERROR_INVALID_PARAMETER as INT;
        } else if dns_status != 0 || dp.is_null() {
            result = WSAHOST_NOT_FOUND as INT;
        } else {
            curr = dp;
            while !(*curr).pNext.is_null() || (*curr).wType != DNS_TYPE_A as u16 {
                if (*curr).wType == DNS_TYPE_CNAME as u16 {
                    Aliases[AliasIndex as usize] = (*curr).Data.Cname.as_ref().pNameHost;
                    AliasIndex += 1;
                }
                curr = (*curr).pNext;
            }

            if (*curr).wType != DNS_TYPE_A as u16 {
                result = WSASERVICE_NOT_FOUND as INT;
            } else {
                (*hostinfo).hostnameW = StrCpyHeapAllocW(hHeap, (*curr).pName);
                (*hostinfo).addr4 = (*curr).Data.A.as_ref().IpAddress;
                if AliasIndex != 0 {
                    (*hostinfo).servaliasesA = StrAryCpyHeapAllocWToA(hHeap, Aliases.as_mut_ptr());
                }
                result = ERROR_SUCCESS as INT;
            }
        }
    }

    if !dp.is_null() {
        DnsRecordListFree(dp, DnsFreeRecordList as ::core::ffi::c_int);
    }

    result
}
unsafe extern "C" fn DecodeServEntFromString(
    mut ServiceString: PCHAR,
    ServiceName: *mut PCHAR,
    PortNumberStr: *mut PCHAR,
    ProtocolStr: *mut PCHAR,
    mut Aliases: *mut PCHAR,
    MaxAlias: DWORD,
) -> BOOL {
    let mut NAliases: u32 = 0;

    while *ServiceString != 0 && isspace(*ServiceString as i32) != 0 {
        ServiceString = ServiceString.offset(1);
    }
    if *ServiceString == 0 {
        return FALSE as INT;
    }

    *ServiceName = ServiceString;

    while *ServiceString != 0 && isspace(*ServiceString as i32) == 0 {
        ServiceString = ServiceString.offset(1);
    }
    if *ServiceString == 0 {
        return FALSE as INT;
    } else {
        *ServiceString = 0;
        ServiceString = ServiceString.offset(1);
    }

    while *ServiceString != 0 && isspace(*ServiceString as i32) != 0 {
        ServiceString = ServiceString.offset(1);
    }
    if *ServiceString == 0 {
        return FALSE as INT;
    }

    *PortNumberStr = ServiceString;

    while *ServiceString != 0 && isspace(*ServiceString as i32) == 0 {
        ServiceString = ServiceString.offset(1);
    }
    if !(*ServiceString == 0) {
        *ServiceString = 0;
        ServiceString = ServiceString.offset(1);
    }

    while *ServiceString != 0 && NAliases < MaxAlias.wrapping_sub(1) {
        while *ServiceString != 0 && isspace(*ServiceString as i32) != 0 {
            ServiceString = ServiceString.offset(1);
        }
        if *ServiceString == 0 {
            break;
        }

        if *ServiceString != 0 {
            while *ServiceString != 0 && isspace(*ServiceString as i32) != 0 {
                ServiceString = ServiceString.offset(1);
            }

            if strlen(ServiceString) != 0 {
                let fresh0 = Aliases;
                Aliases = Aliases.offset(1);
                *fresh0 = ServiceString;
                NAliases = NAliases.wrapping_add(1);
            }

            while *ServiceString != 0 && isspace(*ServiceString as i32) == 0 {
                ServiceString = ServiceString.offset(1);
            }
            if !(*ServiceString == 0) {
                *ServiceString = 0;
                ServiceString = ServiceString.offset(1);
            }
        }
    }

    *Aliases = core::ptr::null_mut();

    *ProtocolStr = strchr(*PortNumberStr, '/' as i32) as PCHAR;

    if (*ProtocolStr).is_null() {
        return FALSE as INT;
    }

    **ProtocolStr = 0;
    *ProtocolStr = (*ProtocolStr).offset(1);

    return TRUE as INT;
}
#[no_mangle]
pub unsafe extern "system" fn OpenNetworkDatabase(Name: LPCWSTR) -> HANDLE {
    let mut ExpandedPath: PWSTR;
    let mut DatabasePath: PWSTR;
    let mut ErrorCode: i32;
    let mut DatabaseKey: HKEY = core::ptr::null_mut();
    let mut RegType: DWORD = 0;
    let mut RegSize: DWORD = 0;
    let mut StringLength: usize = 0;
    let mut ret: HANDLE;

    ExpandedPath = HeapAlloc(
        GetProcessHeap(),
        0,
        ((MAX_PATH as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;
    if ExpandedPath.is_null() {
        return INVALID_HANDLE_VALUE;
    }

    ErrorCode = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services\\Tcpip\\Parameters") as LPCWSTR,
        0,
        KEY_READ,
        &mut DatabaseKey,
    ) as i32;

    // Divergence from the C, which trusted the key to carry the value: nothing in our hives
    // sets DatabasePath, and Tcpip\Parameters only appears once a NIC's TCP/IP settings are
    // written. With the key present and the value absent, the C took RegSize = 0, made a
    // zero-byte block, ignored the failed read and expanded that uninitialised block as the
    // path -- so getservbyname() worked or failed depending on whether the NIC had been
    // configured yet (the amd64-only `ftp/tcp: unknown service` flake, 2026-10-14). Any
    // failure here now falls back to the default directory.
    let mut have_path = false;
    if ErrorCode == NO_ERROR as i32 {
        ErrorCode = RegQueryValueExW(
            DatabaseKey,
            w!("DatabasePath") as LPCWSTR,
            core::ptr::null_mut(),
            &mut RegType,
            core::ptr::null_mut(),
            &mut RegSize,
        ) as i32;

        if ErrorCode == NO_ERROR as i32 && RegSize as usize >= core::mem::size_of::<u16>() {
            // One WCHAR more than the value, for a terminator the registry need not store.
            DatabasePath = HeapAlloc(
                GetProcessHeap(),
                0,
                (RegSize as usize + core::mem::size_of::<u16>()) as SIZE_T,
            ) as PWSTR;
            if DatabasePath.is_null() {
                RegCloseKey(DatabaseKey);
                HeapFree(GetProcessHeap(), 0, ExpandedPath as LPVOID);
                return INVALID_HANDLE_VALUE;
            }

            ErrorCode = RegQueryValueExW(
                DatabaseKey,
                w!("DatabasePath") as LPCWSTR,
                core::ptr::null_mut(),
                &mut RegType,
                DatabasePath as LPBYTE,
                &mut RegSize,
            ) as i32;
            if ErrorCode == NO_ERROR as i32 {
                *DatabasePath.add(RegSize as usize / core::mem::size_of::<u16>()) = 0;
                let n = ExpandEnvironmentStringsW(DatabasePath, ExpandedPath, MAX_PATH);
                have_path = n != 0 && n <= MAX_PATH as _ && *ExpandedPath != 0;
            }

            HeapFree(GetProcessHeap(), 0, DatabasePath as LPVOID);
        }

        RegCloseKey(DatabaseKey);
    }
    if !have_path {
        GetSystemDirectoryW(ExpandedPath, MAX_PATH);
        StringCchLength(ExpandedPath, MAX_PATH as usize, &mut StringLength);
        if *ExpandedPath.offset((StringLength - 1) as isize) != b'\\' as u16 {
            StringCchCat(ExpandedPath, MAX_PATH as usize, w!("\\") as LPCWSTR);
        }
        StringCchCat(ExpandedPath, MAX_PATH as usize, w!("DRIVERS\\ETC\\") as LPCWSTR);
    }

    StringCchLength(ExpandedPath, MAX_PATH as usize, &mut StringLength);
    if *ExpandedPath.offset((StringLength - 1) as isize) != b'\\' as u16 {
        StringCchCat(ExpandedPath, MAX_PATH as usize, w!("\\") as LPCWSTR);
    }

    StringCchCat(ExpandedPath, MAX_PATH as usize, Name);

    ret = CreateFileW(
        ExpandedPath,
        FILE_READ_DATA,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    HeapFree(GetProcessHeap(), 0, ExpandedPath as LPVOID);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NSP_GetServiceByNameHeapAllocW(
    data: PWSHANDLEINTERN,
    dwControlFlags: DWORD,
    hostinfo: PWSHOSTINFOINTERN,
) -> INT {
    let mut Found: BOOL = FALSE as BOOL;
    let mut ServicesFile: HANDLE;
    let mut ServiceDBData: [i8; 1024] = [0; 1024];
    let mut ThisLine: PCHAR = core::ptr::null_mut();
    let mut NextLine: PCHAR = core::ptr::null_mut();
    let mut ServiceName: PCHAR = core::ptr::null_mut();
    let mut PortNumberStr: PCHAR = core::ptr::null_mut();
    let mut ProtocolStr: PCHAR = core::ptr::null_mut();
    let mut Comment: PCHAR = core::ptr::null_mut();
    let mut EndValid: PCHAR;
    let mut Aliases: [PCHAR; 512] = [core::ptr::null_mut(); 512];
    let mut AliasPtr: *mut PCHAR;
    let mut i: u32;
    let mut ReadSize: DWORD = 0;
    // Init to null: on the `hostnameW == NULL` early-return path (below) hHeap is read by the
    // cleanup block before line 691's GetProcessHeap(), but nameA/nameServiceA are still NULL
    // there so no HeapFree actually fires — the C left hHeap uninitialized; null is the safe
    // equivalent that satisfies the borrow checker.
    let mut hHeap: HANDLE = core::ptr::null_mut();
    let mut nameA: PCHAR = core::ptr::null_mut();
    let mut nameServiceA: PCHAR = core::ptr::null_mut();
    let mut nameProtoA: PCHAR;
    let mut res: INT = WSANO_RECOVERY as INT;

    if (*data).hostnameW.is_null() {
        res = WSANO_RECOVERY as INT;
        'End: {
            if !nameA.is_null() {
                HeapFree(hHeap, 0, nameA as LPVOID);
            }
            if !nameServiceA.is_null() {
                HeapFree(hHeap, 0, nameServiceA as LPVOID);
            }
            return res;
        }
    }

    hHeap = GetProcessHeap();
    nameA = StrW2AHeapAlloc(hHeap, (*data).hostnameW);

    nameProtoA = strchr(nameA, b'/' as i32);
    if nameProtoA.is_null() {
        res = WSANO_RECOVERY as INT;
        'End: {
            if !nameA.is_null() {
                HeapFree(hHeap, 0, nameA as LPVOID);
            }
            if !nameServiceA.is_null() {
                HeapFree(hHeap, 0, nameServiceA as LPVOID);
            }
            return res;
        }
    }

    nameProtoA = nameProtoA.offset(1);
    i = (nameProtoA.offset_from(nameA) as isize - 1) as u32;
    nameServiceA = HeapAlloc(hHeap, 0, (i + 1) as SIZE_T) as PCHAR;
    StringCbCopyA(nameServiceA, (i + 1) as usize, nameA);
    *nameServiceA.offset(i as isize) = 0;

    ServicesFile = OpenNetworkDatabase(w!("services") as LPCWSTR);
    if ServicesFile == INVALID_HANDLE_VALUE {
        return WSANO_RECOVERY as INT;
    }

    ReadFile(
        ServicesFile,
        ServiceDBData.as_mut_ptr() as LPVOID,
        (core::mem::size_of_val(&ServiceDBData) - 1) as DWORD,
        core::ptr::addr_of_mut!(ReadSize),
        core::ptr::null_mut(),
    );

    NextLine = ServiceDBData.as_mut_ptr();
    ThisLine = NextLine;
    EndValid = ServiceDBData.as_mut_ptr().offset(ReadSize as isize);
    ServiceDBData[core::mem::size_of_val(&ServiceDBData) - 1] = 0;

    while ReadSize != 0 {
        loop {
            if *NextLine == b'\r' as i8 || *NextLine == b'\n' as i8 {
                break;
            }
            if NextLine == EndValid {
                let LineLen = NextLine.offset_from(ThisLine) as isize as i32;
                if ThisLine == ServiceDBData.as_mut_ptr() {
                    return WSANO_RECOVERY as INT;
                }
                memmove(
                    ServiceDBData.as_mut_ptr() as *mut core::ffi::c_void,
                    ThisLine as *const core::ffi::c_void,
                    LineLen as usize,
                );
                ReadFile(
                    ServicesFile,
                    ServiceDBData.as_mut_ptr().offset(LineLen as isize) as LPVOID,
                    (core::mem::size_of_val(&ServiceDBData) - 1 - LineLen as usize) as DWORD,
                    core::ptr::addr_of_mut!(ReadSize),
                    core::ptr::null_mut(),
                );
                EndValid = ServiceDBData.as_mut_ptr()
                    .offset(LineLen as isize)
                    .offset(ReadSize as isize);
                NextLine = ServiceDBData.as_mut_ptr().offset(LineLen as isize);
                ThisLine = ServiceDBData.as_mut_ptr();
                if ReadSize == 0 {
                    break;
                }
            }
            NextLine = NextLine.offset(1);
        }

        *NextLine = 0;
        Comment = strchr(ThisLine, b'#' as i32);
        if !Comment.is_null() {
            *Comment = 0;
        }

        if DecodeServEntFromString(
            ThisLine,
            core::ptr::addr_of_mut!(ServiceName),
            core::ptr::addr_of_mut!(PortNumberStr),
            core::ptr::addr_of_mut!(ProtocolStr),
            Aliases.as_mut_ptr(),
            512,
        ) != 0
            && (strlen(nameProtoA) == 0 || strcmp(ProtocolStr, nameProtoA) == 0)
        {
            Found = (strcmp(ServiceName, nameServiceA) == 0
                || strcmp(PortNumberStr, nameServiceA) == 0) as BOOL;
            AliasPtr = Aliases.as_mut_ptr();
            while Found == 0 && !(*AliasPtr).is_null() {
                Found = (strcmp(*AliasPtr, nameServiceA) == 0) as BOOL;
                AliasPtr = AliasPtr.offset(1);
            }
            if Found != 0 {
                break;
            }
        }
        NextLine = NextLine.offset(1);
        ThisLine = NextLine;
    }

    CloseHandle(ServicesFile);

    if Found == 0 {
        return WSANO_DATA as INT;
    }

    (*hostinfo).addr4 = 0;
    (*hostinfo).servnameW = StrA2WHeapAlloc(hHeap, ServiceName);
    (*hostinfo).servprotoW = StrA2WHeapAlloc(hHeap, ProtocolStr);
    (*hostinfo).servaliasesA = StrAryCpyHeapAllocA(hHeap, Aliases.as_mut_ptr());
    (*hostinfo).servport = atoi(PortNumberStr) as u16;

    res = NO_ERROR as INT;

    if !nameA.is_null() {
        HeapFree(hHeap, 0, nameA as LPVOID);
    }
    if !nameServiceA.is_null() {
        HeapFree(hHeap, 0, nameServiceA as LPVOID);
    }

    res
}
#[no_mangle]
pub unsafe extern "C" fn NSP_LookupServiceNextW(
    data: PWSHANDLEINTERN,
    dwControlFlags: DWORD,
    lpRes: LPWSAQUERYSETW,
    lpResLen: LPDWORD,
) -> INT {

    let mut buf: MSW_BUFFER = core::mem::zeroed();
    let mut hostinfo: WSHOSTINFOINTERN = core::mem::zeroed();
    let mut result: INT;
    let hHeap: HANDLE = GetProcessHeap();
    let mut ServiceInstanceNameW: PWSTR = core::ptr::null_mut();
    let mut ServiceInstanceNameA: *mut i8 = core::ptr::null_mut();
    let mut ServiceProtocolNameA: *mut i8 = core::ptr::null_mut();

    RtlZeroMemory(&mut hostinfo as *mut _ as LPVOID, core::mem::size_of::<WSHOSTINFOINTERN>() as usize);

    mswBufferInit(&mut buf, lpRes as LPBYTE, *lpResLen);
    mswBufferIncUsed(&mut buf, core::mem::size_of::<_WSAQuerySetW>() as DWORD);

    (*lpRes).dwSize = core::mem::size_of::<_WSAQuerySetW>() as DWORD;
    (*lpRes).dwNameSpace = NS_DNS as DWORD;

    'End: {
        if (*data).CallID == NSP_CALLID_HOSTNAME as DWORD
            || (*data).CallID == NSP_CALLID_HOSTBYNAME as DWORD
            || (*data).CallID == NSP_CALLID_SERVICEBYNAME as DWORD
        {
            if (*data).CallIDCounter >= 1 {
                result = WSAENOMORE as INT;
                break 'End;
            }
        } else {
            result = WSANO_RECOVERY as INT;
            break 'End;
        }

        (*data).CallIDCounter += 1;

        if (*data).CallID == NSP_CALLID_HOSTNAME as DWORD {
            result = NSP_GetHostNameHeapAllocW(&mut hostinfo.hostnameW);

            if result != ERROR_SUCCESS as INT {
                break 'End;
            }

            hostinfo.addr4 = 0;
        } else if (*data).CallID == NSP_CALLID_HOSTBYNAME as DWORD {
            result = NSP_GetHostByNameHeapAllocW(data, dwControlFlags, &mut hostinfo);
            if result != ERROR_SUCCESS as INT {
                break 'End;
            }
        } else {
            result = NSP_GetServiceByNameHeapAllocW(data, dwControlFlags, &mut hostinfo);
            if result != ERROR_SUCCESS as INT {
                break 'End;
            }
        }

        if ((LUP_RETURN_BLOB as DWORD & (*data).dwControlFlags) != 0)
            || ((LUP_RETURN_NAME as DWORD & (*data).dwControlFlags) != 0)
        {
            if (*data).CallID == NSP_CALLID_HOSTNAME as DWORD || (*data).CallID == NSP_CALLID_HOSTBYNAME as DWORD {
                ServiceInstanceNameW = hostinfo.hostnameW;
                ServiceInstanceNameA = StrW2AHeapAlloc(hHeap, ServiceInstanceNameW) as *mut i8;
                if ServiceInstanceNameA.is_null() {
                    result = WSAEFAULT as INT;
                    break 'End;
                }
            }
            if (*data).CallID == NSP_CALLID_SERVICEBYNAME as DWORD {
                ServiceInstanceNameW = hostinfo.servnameW;
                ServiceInstanceNameA = StrW2AHeapAlloc(hHeap, ServiceInstanceNameW) as *mut i8;
                if ServiceInstanceNameA.is_null() {
                    result = WSAEFAULT as INT;
                    break 'End;
                }
                ServiceProtocolNameA = StrW2AHeapAlloc(hHeap, hostinfo.servprotoW) as *mut i8;
                if ServiceProtocolNameA.is_null() {
                    result = WSAEFAULT as INT;
                    break 'End;
                }
            }
        }

        if (LUP_RETURN_ADDR as DWORD & (*data).dwControlFlags) != 0 {
            if mswBufferAppendAddr_AddrInfoW(&mut buf, lpRes, hostinfo.addr4) == 0 {
                *lpResLen = buf.bytesUsed;
                result = WSAEFAULT as INT;
                break 'End;
            }
        }

        if (LUP_RETURN_BLOB as DWORD & (*data).dwControlFlags) != 0 {
            if (*data).CallID == NSP_CALLID_HOSTBYNAME as DWORD {
                if mswBufferAppendBlob_Hostent(
                    &mut buf,
                    lpRes,
                    if (LUP_RETURN_ALIASES as DWORD & (*data).dwControlFlags) != 0 {
                        hostinfo.servaliasesA
                    } else {
                        core::ptr::null_mut()
                    },
                    ServiceInstanceNameA,
                    hostinfo.addr4,
                ) == 0
                {
                    *lpResLen = buf.bytesUsed;
                    result = WSAEFAULT as INT;
                    break 'End;
                }
            } else if (*data).CallID == NSP_CALLID_SERVICEBYNAME as DWORD {
                if mswBufferAppendBlob_Servent(
                    &mut buf,
                    lpRes,
                    ServiceInstanceNameA,
                    if (LUP_RETURN_ALIASES as DWORD & (*data).dwControlFlags) != 0 {
                        hostinfo.servaliasesA
                    } else {
                        core::ptr::null_mut()
                    },
                    ServiceProtocolNameA,
                    hostinfo.servport,
                ) == 0
                {
                    *lpResLen = buf.bytesUsed;
                    result = WSAEFAULT as INT;
                    break 'End;
                }
            } else {
                result = WSANO_RECOVERY as INT;
                break 'End;
            }
        }

        if (LUP_RETURN_NAME as DWORD & (*data).dwControlFlags) != 0 {
            (*lpRes).lpszServiceInstanceName = mswBufferEndPtr(&mut buf) as LPWSTR;
            if mswBufferAppendStrW(&mut buf, ServiceInstanceNameW) == 0 {
                (*lpRes).lpszServiceInstanceName = core::ptr::null_mut();
                *lpResLen = buf.bytesUsed;
                result = WSAEFAULT as INT;
                break 'End;
            }
        }

        *lpResLen = buf.bytesUsed;
        result = ERROR_SUCCESS as INT;
    }

    if !ServiceInstanceNameA.is_null() {
        HeapFree(hHeap, 0, ServiceInstanceNameA as LPVOID);
    }

    if !ServiceProtocolNameA.is_null() {
        HeapFree(hHeap, 0, ServiceProtocolNameA as LPVOID);
    }

    if !hostinfo.hostnameW.is_null() {
        HeapFree(hHeap, 0, hostinfo.hostnameW as LPVOID);
    }

    if !hostinfo.servnameW.is_null() {
        HeapFree(hHeap, 0, hostinfo.servnameW as LPVOID);
    }

    if !hostinfo.servprotoW.is_null() {
        HeapFree(hHeap, 0, hostinfo.servprotoW as LPVOID);
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn NSPStartup(
    lpProviderId: LPGUID,
    lpRout: LPNSP_ROUTINE,
) -> INT {
    let mut ret: INT;

    if lpRout.is_null()
        || (*lpRout).cbSize as usize != core::mem::size_of::<NSP_ROUTINE>()
    {
        WSASetLastError(ERROR_INVALID_PARAMETER as INT);
        return ERROR_INVALID_PARAMETER as INT;
    }

    mwsNSPInit();

    (*lpRout).dwMajorVersion = 1;
    (*lpRout).dwMinorVersion = 1;
    (*lpRout).cbSize = (core::mem::size_of::<_NSP_ROUTINE>()
        - core::mem::size_of::<LPNSPIOCTL>()) as DWORD;
    (*lpRout).NSPCleanup = Some(mwsNSPCleanUp);
    (*lpRout).NSPLookupServiceBegin = Some(mwsNSPLookupServiceBegin);
    (*lpRout).NSPLookupServiceNext = Some(mwsNSPLookupServiceNext);
    (*lpRout).NSPLookupServiceEnd = Some(mwsNSPLookupServiceEnd);
    (*lpRout).NSPSetService = Some(mwsNSPSetService);
    (*lpRout).NSPInstallServiceClass = Some(mwsNSPInstallServiceClass);
    (*lpRout).NSPRemoveServiceClass = Some(mwsNSPRemoveServiceClass);
    (*lpRout).NSPGetServiceClassInfo = Some(mwsNSPGetServiceClassInfo);
    (*lpRout).NSPIoctl = None;

    ret = NO_ERROR as INT;

    ret
}
// enforce_import_abi: moved 5 import decl(s) into the extern block their
// exporting DLL's .spec requires: atoi->C, memmove->C, strchr->C, strcmp->C, strlen->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "C" {
    fn atoi(str: PCHAR) -> i32;
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn strchr(s: PCHAR, c: i32) -> PCHAR;
    fn strcmp(s1: PCHAR, s2: PCHAR) -> i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// #include <stdlib.h>
// #include <ws2spi.h>
// #include <nspapi.h>
// #include <winuser.h>
// #include <windns.h>
// #include <guiddef.h>
// #include <svcguid.h>
// #include <iptypes.h>
// #include <strsafe.h>
// #include <winreg.h>
//
// #include "mswhelper.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NSP_CALLID_DNS 0x0001
// #define NSP_CALLID_HOSTNAME 0x0002
// #define NSP_CALLID_HOSTBYNAME 0x0003
// #define NSP_CALLID_SERVICEBYNAME 0x0004
//
// #ifndef BUFSIZ
// #define BUFSIZ 1024
// #endif // BUFSIZ
// #ifndef WS2_INTERNAL_MAX_ALIAS
// #define WS2_INTERNAL_MAX_ALIAS 512
// #endif // WS2_INTERNAL_MAX_ALIAS
//
// //#define IP_LOCALHOST 0x0100007F
//
// //#define NSP_REDIRECT
//
// typedef struct {
//   WCHAR* hostnameW;
//   DWORD addr4;
//   WCHAR* servnameW;
//   WCHAR* servprotoW;
//   CHAR** servaliasesA; /* array */
//   WORD servport;
// } WSHOSTINFOINTERN, *PWSHOSTINFOINTERN;
//
// typedef struct {
//   GUID providerId; /* Provider-ID */
//   DWORD dwControlFlags; /* dwControlFlags (WSALookupServiceBegin) */
//   DWORD CallID; /* List for LookupServiceNext-Calls */
//   DWORD CallIDCounter; /* call-count of the current CallID. */
//   WCHAR* hostnameW; /* hostbyname */
// #ifdef NSP_REDIRECT
//   HANDLE rdrLookup;
//   NSP_ROUTINE rdrproc;
// #endif
// } WSHANDLEINTERN, *PWSHANDLEINTERN;
//
// static const GUID guid_NULL = {0};
// static const GUID guid_HOSTNAME = SVCID_HOSTNAME;
// static const GUID guid_INET_HOSTADDRBYINETSTRING = SVCID_INET_HOSTADDRBYINETSTRING;
// static const GUID guid_INET_HOSTADDRBYNAME = SVCID_INET_HOSTADDRBYNAME;
// static const GUID guid_INET_SERVICEBYNAME = SVCID_INET_SERVICEBYNAME;
//
// /* GUIDs - maybe they should be loaded from registry? */
// /* Namespace: 32 */
// static const GUID guid_mswsock_TcpIp = {/*Data1:*/ 0x22059D40,
//                                         /*Data2:*/ 0x7E9E,
//                                         /*Data3:*/ 0x11CF,
//                                         /*Data4:*/ {0xAE, 0x5A, 0x00, 0xAA, 0x00, 0xA7, 0x11, 0x2B}};
//
// /* {6642243A-3BA8-4AA6-BAA5-2E0BD71FDD83} */
// /* Namespace: 15 */
// static const GUID guid_mswsock_NLA = {/*Data1:*/ 0x6642243A,
//                                       /*Data2:*/ 0x3BA8,
//                                       /*Data3:*/ 0x4AA6,
//                                       /*Data4:*/ {0xBA, 0xA5, 0x2E, 0x0B, 0xD7, 0x1F, 0xDD, 0x83}};
//
// #ifdef NSP_REDIRECT
//
// typedef INT
// (__stdcall *lpRdrNSPStartup)(
//   LPGUID lpProviderId,
//   LPNSP_ROUTINE lpRout);
//
// const rdrLib = "mswsock.dll-original";
// lpRdrNSPStartup rdrNSPStartup;
// HANDLE hLib;
// NSP_ROUTINE rdrproc_tcpip;
// NSP_ROUTINE rdrproc_nla;
//
// #endif /* NSP_REDIRECT */
//
// /* Forwards */
// INT
// WINAPI
// mswNSPStartup(
//   LPGUID lpProviderId,
//   LPNSP_ROUTINE lpRout);
//
// INT
// NSP_LookupServiceBeginW(
//   PWSHANDLEINTERN data,
//   CHAR* hostnameA,
//   WCHAR* hostnameW,
//   DWORD CallID);
//
// INT
// NSP_LookupServiceNextW(
//    PWSHANDLEINTERN data,
//    DWORD dwControlFlags,
//    LPWSAQUERYSETW lpRes,
//    LPDWORD lpResLen);
//
// INT
// NSP_GetHostNameHeapAllocW(
//    WCHAR** hostname);
//
// INT
// NSP_GetHostByNameHeapAllocW(
//    PWSHANDLEINTERN data,
//    DWORD dwControlFlags,
//    PWSHOSTINFOINTERN hostinfo);
//
// INT
// NSP_GetServiceByNameHeapAllocW(
//    PWSHANDLEINTERN data,
//    DWORD dwControlFlags,
//    PWSHOSTINFOINTERN hostinfo);
//
// /* Implementations - Internal */
//
// INT
// __stdcall
// mwsNSPCleanUp( LPGUID lpProviderId)
// {
//     //WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     //return ERROR_CALL_NOT_IMPLEMENTED;
//     return ERROR_SUCCESS;
// }
//
// INT
// mwsNSPInit(VOID)
// {
//     return ERROR_SUCCESS;
// }
//
// INT
// __stdcall
// mwsNSPLookupServiceBegin( LPGUID lpProviderId,
//                           LPWSAQUERYSETW lpqsRestrictions,
//                           LPWSASERVICECLASSINFOW lpServiceClassInfo,
//                           DWORD dwControlFlags,
//                           LPHANDLE lphLookup)
// {
//     PWSHANDLEINTERN pLook;
//     int wsaErr;
//
//     if (IsEqualGUID(lpProviderId, &guid_mswsock_TcpIp))
//     {
//         //OK
//     }
//     else if (IsEqualGUID(lpProviderId, &guid_mswsock_NLA))
//     {
//         WSASetLastError(WSASERVICE_NOT_FOUND);
//         return SOCKET_ERROR;
//     }
//     else
//     {
//         return ERROR_CALL_NOT_IMPLEMENTED;
//     }
//
//     /* allocate internal structure */
//     pLook = HeapAlloc(GetProcessHeap(), 0, sizeof(WSHANDLEINTERN));
//     if (!pLook)
//     {
//         WSASetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     *lphLookup = (HANDLE)pLook;
//
//     RtlZeroMemory(pLook, sizeof(*pLook));
//
//     /* Anyway the ControlFlags "should" be needed
//        in NSPLookupServiceNext. (see doku) But
//        thats not the fact ATM. */
//     pLook->dwControlFlags = dwControlFlags;
//     pLook->providerId = *lpProviderId;
//
// #ifdef NSP_REDIRECT
//
//     if (IsEqualGUID(lpProviderId, &guid_mswsock_TcpIp))
//     {
//         pLook->rdrproc = rdrproc_tcpip;
//     }
//     else if (IsEqualGUID(lpProviderId, &guid_mswsock_NLA))
//     {
//         pLook->rdrproc = rdrproc_nla;
//     }
//     else
//     {
//         return ERROR_CALL_NOT_IMPLEMENTED;
//     }
//
//     if (pLook->rdrproc.NSPLookupServiceBegin(lpProviderId,
//                                              lpqsRestrictions,
//                                              lpServiceClassInfo,
//                                              dwControlFlags,
//                                              &pLook->rdrLookup) == NO_ERROR)
//     {
//         wsaErr = NO_ERROR;
//     }
//     else
//     {
//         wsaErr = WSAGetLastError();
//     }
//
//     /*
//     if (res)
//         res = WSAGetLastError();
//     */
//
// #else /* NSP_REDIRECT */
//
//     wsaErr = ERROR_CALL_NOT_IMPLEMENTED;
//     if (IsEqualGUID(lpqsRestrictions->lpServiceClassId, &guid_NULL))
//     {
//         wsaErr = ERROR_CALL_NOT_IMPLEMENTED;
//     }
//     else if (IsEqualGUID(lpqsRestrictions->lpServiceClassId, &guid_HOSTNAME))
//     {
//         wsaErr = NSP_LookupServiceBeginW(pLook,
//                                          NULL,
//                                          NULL,
//                                          NSP_CALLID_HOSTNAME);
//     }
//     else if (IsEqualGUID(lpqsRestrictions->lpServiceClassId,
//                          &guid_INET_HOSTADDRBYNAME))
//     {
//        wsaErr = NSP_LookupServiceBeginW(pLook,
//                                         NULL,
//                                         lpqsRestrictions->lpszServiceInstanceName,
//                                         NSP_CALLID_HOSTBYNAME);
//     }
//     else if (IsEqualGUID(lpqsRestrictions->lpServiceClassId,
//                          &guid_INET_SERVICEBYNAME))
//     {
//        wsaErr = NSP_LookupServiceBeginW(pLook,
//                                         NULL,
//                                         lpqsRestrictions->lpszServiceInstanceName,
//                                         NSP_CALLID_SERVICEBYNAME);
//     }
//     else if (IsEqualGUID(lpqsRestrictions->lpServiceClassId,
//                          &guid_INET_HOSTADDRBYINETSTRING))
//     {
//         wsaErr = ERROR_CALL_NOT_IMPLEMENTED;
//     }
//
// #endif /* NSP_REDIRECT */
//
//     if (wsaErr != NO_ERROR)
//     {
//         WSASetLastError(wsaErr);
//         return SOCKET_ERROR;
//     }
//     return NO_ERROR;
// }
//
// INT
// __stdcall
// mwsNSPLookupServiceNext( HANDLE hLookup,
//                          DWORD dwControlFlags,
//                          LPDWORD lpdwBufferLength,
//                         //
//                         LPWSAQUERYSETW lpqsResults)
// {
//     PWSHANDLEINTERN pLook = hLookup;
//     int wsaErr = 0;
//
// #ifdef NSP_REDIRECT
//
//     INT res = pLook->rdrproc.NSPLookupServiceNext(pLook->rdrLookup,
//                                                   dwControlFlags,
//                                                   lpdwBufferLength,
//                                                   lpqsResults);
//     wsaErr = WSAGetLastError();
//     if (res != ERROR_SUCCESS)
//     {
//         wsaErr = WSAGetLastError();
//
//         if (wsaErr == 0)
//             wsaErr = 0xFFFFFFFF;
//     }
//
// #else /* NSP_REDIRECT */
//
//     if ((lpdwBufferLength == NULL) || (*lpdwBufferLength == 0))
//     {
//         wsaErr = WSA_NOT_ENOUGH_MEMORY;
//         goto End;
//     }
//
//     RtlZeroMemory(lpqsResults, *lpdwBufferLength);
//     lpqsResults->dwSize = sizeof(*lpqsResults);
//
//     wsaErr = NSP_LookupServiceNextW(pLook,
//                                     dwControlFlags,
//                                     lpqsResults,
//                                     lpdwBufferLength);
//
//
// #endif /* NSP_REDIRECT */
//
// End:
//     if (wsaErr != 0)
//     {
//         WSASetLastError(wsaErr);
//         return SOCKET_ERROR;
//     }
//     return NO_ERROR;
// }
//
// INT
// __stdcall
// mwsNSPIoCtl( HANDLE hLookup,
//              DWORD dwControlCode,
//              LPVOID lpvInBuffer,
//              DWORD cbInBuffer,
//              LPVOID lpvOutBuffer,
//              DWORD cbOutBuffer,
//              LPDWORD lpcbBytesReturned,
//              LPWSACOMPLETION lpCompletion,
//              LPWSATHREADID lpThreadId)
// {
//     WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// INT
// __stdcall
// mwsNSPLookupServiceEnd( HANDLE hLookup)
// {
//     PWSHANDLEINTERN pLook;
//     HANDLE hHeap;
//     INT res;
//
//     res = NO_ERROR;
//     pLook = (PWSHANDLEINTERN)hLookup;
//     hHeap = GetProcessHeap();
//
// #ifdef NSP_REDIRECT
//     res = pLook->rdrproc.NSPLookupServiceEnd(pLook->rdrLookup);
// #endif
//
//     if (pLook->hostnameW != NULL)
//         HeapFree(hHeap, 0, pLook->hostnameW);
//
//     HeapFree(hHeap, 0, pLook);
//     return res;
// }
//
// INT
// __stdcall
// mwsNSPSetService( LPGUID lpProviderId,
//                   LPWSASERVICECLASSINFOW lpServiceClassInfo,
//                   LPWSAQUERYSETW lpqsRegInfo,
//                   WSAESETSERVICEOP essOperation,
//                   DWORD dwControlFlags)
// {
//     WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// INT
// __stdcall
// mwsNSPInstallServiceClass( LPGUID lpProviderId,
//                            LPWSASERVICECLASSINFOW lpServiceClassInfo)
// {
//     WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// INT
// __stdcall
// mwsNSPRemoveServiceClass( LPGUID lpProviderId,
//                           LPGUID lpServiceClassId)
// {
//     WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// INT
// __stdcall
// mwsNSPGetServiceClassInfo( LPGUID lpProviderId,
//                            LPDWORD lpdwBufSize,
//                            LPWSASERVICECLASSINFOW lpServiceClassInfo)
// {
//     WSASetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// /*
//     hostnameA / hostnameW
//     * only used by HOSTBYNAME
//     * only one should be set
//
// */
// INT
// NSP_LookupServiceBeginW(PWSHANDLEINTERN data,
//                         CHAR* hostnameA,
//                         WCHAR* hostnameW,
//                         DWORD CallID)
// {
//     HANDLE hHeap;
//
//     if (data->CallID != 0)
//         return WSAEFAULT;
//
//     data->CallID = CallID;
//
//     if ((CallID == NSP_CALLID_HOSTBYNAME) ||
//         (CallID == NSP_CALLID_SERVICEBYNAME))
//     {
//         hHeap = GetProcessHeap();
//
//         if (data->hostnameW != NULL)
//             HeapFree(hHeap, 0, data->hostnameW);
//
//         if (hostnameA != NULL)
//         {
//             data->hostnameW = StrA2WHeapAlloc(hHeap, hostnameA);
//         }
//         else
//         {
//             data->hostnameW = StrCpyHeapAllocW(hHeap, hostnameW);
//         }
//     }
//
//     WSASetLastError(0);
//
//     return ERROR_SUCCESS;
// }
//
// INT
// NSP_GetHostNameHeapAllocW( WCHAR** hostname)
// {
//     WCHAR* name;
//     HANDLE hHeap = GetProcessHeap();
//     DWORD bufCharLen = MAX_COMPUTERNAME_LENGTH + 1;
//     DWORD bufByteLen = bufCharLen * sizeof(WCHAR);
//
//     name = HeapAlloc(hHeap, 0, bufByteLen);
//
//     if (!GetComputerNameExW(ComputerNameDnsHostname,
//                             name,
//                             &bufCharLen))
//     {
//         HeapFree(hHeap, 0, name);
//         WSASetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     *hostname = name;
//     return ERROR_SUCCESS;
// }
//
// INT
// NSP_GetHostByNameHeapAllocW( PWSHANDLEINTERN data,
//                              DWORD dwControlFlags,
//                              PWSHOSTINFOINTERN hostinfo)
// {
//     HANDLE hHeap = GetProcessHeap();
//     DNS_STATUS dns_status = { 0 };
//     /* include/WinDNS.h -- look up DNS_RECORD on MSDN */
//     PDNS_RECORDW dp;
//     PDNS_RECORDW curr;
//     INT result = ERROR_SUCCESS;
//     DWORD dwQueryFlags = DNS_QUERY_STANDARD;
//     PWCHAR Aliases[WS2_INTERNAL_MAX_ALIAS] = { 0 };
//     int AliasIndex = 0;
//
//     /* needed to be cleaned up if != NULL */
//     dp = NULL;
//
//     if (data->hostnameW == NULL)
//     {
//         result = ERROR_INVALID_PARAMETER;
//         goto cleanup;
//     }
//
//     if ((data->dwControlFlags & LUP_DEEP) == 0)
//     {
//         dwQueryFlags |= DNS_QUERY_NO_RECURSION;
//     }
//
//     /* DNS_TYPE_A: include/WinDNS.h */
//     /* DnsQuery -- lib/dnsapi/dnsapi/query.c */
//     dns_status = DnsQuery_W(data->hostnameW,
//                             DNS_TYPE_A,
//                             dwQueryFlags,
//                             NULL /* extra dns servers */,
//                             &dp,
//                             NULL);
//     if (dns_status == ERROR_INVALID_NAME)
//     {
//         WSASetLastError(WSAEFAULT);
//         result = ERROR_INVALID_PARAMETER;
//         goto cleanup;
//     }
//
//     if ((dns_status != 0) || (dp == NULL))
//     {
//         result = WSAHOST_NOT_FOUND;
//         goto cleanup;
//     }
//
//     //ASSERT(dp->wType == DNS_TYPE_A);
//     //ASSERT(dp->wDataLength == sizeof(DNS_A_DATA));
//     curr = dp;
//     while ((curr->pNext != NULL) || (curr->wType != DNS_TYPE_A))
//     {
//         if (curr->wType == DNS_TYPE_CNAME)
//         {
//             Aliases[AliasIndex++] = curr->Data.Cname.pNameHost;
//         }
//         curr = curr->pNext;
//     }
//
//     if (curr->wType != DNS_TYPE_A)
//     {
//         result = WSASERVICE_NOT_FOUND;
//         goto cleanup;
//     }
//     hostinfo->hostnameW = StrCpyHeapAllocW(hHeap, curr->pName);
//     hostinfo->addr4 = curr->Data.A.IpAddress;
//     if (AliasIndex)
//     {
//         hostinfo->servaliasesA = StrAryCpyHeapAllocWToA(hHeap, (WCHAR**)&Aliases);
//     }
//     result = ERROR_SUCCESS;
//
// cleanup:
//     if (dp != NULL)
//         DnsRecordListFree(dp, DnsFreeRecordList);
//
//     return result;
// }
//
// #define SKIPWS(ptr, act) \
// {while(*ptr && isspace(*ptr)) ptr++; if(!*ptr) act;}
//
// #define SKIPANDMARKSTR(ptr, act) \
// {while(*ptr && !isspace(*ptr)) ptr++; \
//  if(!*ptr) {act;} else { *ptr = 0; ptr++; }}
//
// static
// BOOL
// DecodeServEntFromString(IN PCHAR ServiceString,
//                         OUT PCHAR *ServiceName,
//                         OUT PCHAR *PortNumberStr,
//                         OUT PCHAR *ProtocolStr,
//                         IN PCHAR *Aliases,
//                         IN DWORD MaxAlias)
// {
//     UINT NAliases = 0;
//
//     //WS_DbgPrint(MAX_TRACE, ("Parsing service ent [%s]\n", ServiceString));
//
//     SKIPWS(ServiceString, return FALSE);
//     *ServiceName = ServiceString;
//     SKIPANDMARKSTR(ServiceString, return FALSE);
//     SKIPWS(ServiceString, return FALSE);
//     *PortNumberStr = ServiceString;
//     SKIPANDMARKSTR(ServiceString, ;);
//
//     while (*ServiceString && NAliases < MaxAlias - 1)
//     {
//         SKIPWS(ServiceString, break);
//         if (*ServiceString)
//         {
//             SKIPWS(ServiceString, ;);
//             if (strlen(ServiceString))
//             {
//                 //WS_DbgPrint(MAX_TRACE, ("Alias: %s\n", ServiceString));
//                 *Aliases++ = ServiceString;
//                 NAliases++;
//             }
//             SKIPANDMARKSTR(ServiceString, ;);
//         }
//     }
//     *Aliases = NULL;
//
//     *ProtocolStr = strchr(*PortNumberStr, '/');
//
//     if (!*ProtocolStr)
//         return FALSE;
//
//     **ProtocolStr = 0;
//     (*ProtocolStr)++;
//
//     //WS_DbgPrint(MAX_TRACE, ("Parsing done: %s %s %s %d\n",
//     //           *ServiceName, *ProtocolStr, *PortNumberStr,
//     //            NAliases));
//
//     return TRUE;
// }
//
// HANDLE
// __stdcall
// OpenNetworkDatabase( LPCWSTR Name)
// {
//     PWSTR ExpandedPath;
//     PWSTR DatabasePath;
//     INT ErrorCode;
//     HKEY DatabaseKey;
//     DWORD RegType;
//     DWORD RegSize = 0;
//     size_t StringLength;
//     HANDLE ret;
//
//     ExpandedPath = HeapAlloc(GetProcessHeap(), 0, MAX_PATH*sizeof(WCHAR));
//     if (!ExpandedPath)
//         return INVALID_HANDLE_VALUE;
//
//     /* Open the database path key */
//     ErrorCode = RegOpenKeyEx(HKEY_LOCAL_MACHINE,
//                              L"System\\CurrentControlSet\\Services\\Tcpip\\Parameters",
//                              0,
//                              KEY_READ,
//                              &DatabaseKey);
//     if (ErrorCode == NO_ERROR)
//     {
//         /* Read the actual path */
//         ErrorCode = RegQueryValueEx(DatabaseKey,
//                                     L"DatabasePath",
//                                     NULL,
//                                     &RegType,
//                                     NULL,
//                                     &RegSize);
//
//         DatabasePath = HeapAlloc(GetProcessHeap(), 0, RegSize);
//         if (!DatabasePath)
//         {
//             HeapFree(GetProcessHeap(), 0, ExpandedPath);
//             return INVALID_HANDLE_VALUE;
//         }
//
//         /* Read the actual path */
//         ErrorCode = RegQueryValueEx(DatabaseKey,
//                                     L"DatabasePath",
//                                     NULL,
//                                     &RegType,
//                                     (LPBYTE)DatabasePath,
//                                     &RegSize);
//
//         /* Close the key */
//         RegCloseKey(DatabaseKey);
//
//         /* Expand the name */
//         ExpandEnvironmentStrings(DatabasePath, ExpandedPath, MAX_PATH);
//
//         HeapFree(GetProcessHeap(), 0, DatabasePath);
//     }
//     else
//     {
//         /* Use defalt path */
//         GetSystemDirectory(ExpandedPath, MAX_PATH);
//         StringCchLength(ExpandedPath, MAX_PATH, &StringLength);
//         if (ExpandedPath[StringLength - 1] != L'\\')
//         {
//             /* It isn't, so add it ourselves */
//             StringCchCat(ExpandedPath, MAX_PATH, L"\\");
//         }
//         StringCchCat(ExpandedPath, MAX_PATH, L"DRIVERS\\ETC\\");
//     }
//
//     /* Make sure that the path is backslash-terminated */
//     StringCchLength(ExpandedPath, MAX_PATH, &StringLength);
//     if (ExpandedPath[StringLength - 1] != L'\\')
//     {
//         /* It isn't, so add it ourselves */
//         StringCchCat(ExpandedPath, MAX_PATH, L"\\");
//     }
//
//     /* Add the database name */
//     StringCchCat(ExpandedPath, MAX_PATH, Name);
//
//     /* Return a handle to the file */
//     ret = CreateFile(ExpandedPath,
//                       FILE_READ_DATA,
//                       FILE_SHARE_READ,
//                       NULL,
//                       OPEN_EXISTING,
//                       FILE_ATTRIBUTE_NORMAL,
//                       NULL);
//
//     HeapFree(GetProcessHeap(), 0, ExpandedPath);
//     return ret;
// }
//
// INT
// NSP_GetServiceByNameHeapAllocW( PWSHANDLEINTERN data,
//                                 DWORD dwControlFlags,
//                                 PWSHOSTINFOINTERN hostinfo)
// {
//     BOOL Found = FALSE;
//     HANDLE ServicesFile;
//     CHAR ServiceDBData[BUFSIZ * sizeof(WCHAR)] = {0};
//     PCHAR ThisLine = 0, NextLine = 0, ServiceName = 0, PortNumberStr = 0,
//     ProtocolStr = 0, Comment = 0, EndValid;
//     PCHAR Aliases[WS2_INTERNAL_MAX_ALIAS] = {0};
//     PCHAR* AliasPtr;
//     UINT i = 0;
//     DWORD ReadSize = 0;
//     HANDLE hHeap;
//     PCHAR nameA = NULL;
//     PCHAR nameServiceA = NULL;
//     PCHAR nameProtoA = NULL;
//     INT res = WSANO_RECOVERY;
//
//     if (!data->hostnameW)
//     {
//         res = WSANO_RECOVERY;
//         goto End;
//     }
//
//     hHeap = GetProcessHeap();
//     nameA = StrW2AHeapAlloc(hHeap, data->hostnameW);
//
//     /* nameA has the form <service-name>/<protocol>
//        we split these now */
//     nameProtoA = strchr(nameA, '/');
//     if (nameProtoA == NULL)
//     {
//         res = WSANO_RECOVERY;
//         goto End;
//     }
//
//     nameProtoA++;
//     i = (DWORD)(nameProtoA - nameA - 1);
//     nameServiceA = (PCHAR)HeapAlloc(hHeap, 0, i + 1);
//     StringCbCopyA(nameServiceA, i + 1, nameA);
//     nameServiceA[i] = '\0';
//
//     ServicesFile = OpenNetworkDatabase(L"services");
//     if (ServicesFile == INVALID_HANDLE_VALUE)
//     {
//         return WSANO_RECOVERY;
//     }
//
//     /* Scan the services file ...
//     *
//     * We will be share the buffer on the lines. If the line does not fit in
//     * the buffer, then moving it to the beginning of the buffer and read
//     * the remnants of line from file.
//     */
//
//     /* Initial Read */
//     ReadFile(ServicesFile,
//              ServiceDBData,
//              sizeof( ServiceDBData ) - 1,
//              &ReadSize,
//              NULL);
//
//     ThisLine = NextLine = ServiceDBData;
//     EndValid = ServiceDBData + ReadSize;
//     ServiceDBData[sizeof(ServiceDBData) - 1] = '\0';
//
//     while (ReadSize)
//     {
//         for (; *NextLine != '\r' && *NextLine != '\n'; NextLine++)
//         {
//             if (NextLine == EndValid)
//             {
//                 int LineLen = NextLine - ThisLine;
//
//                 if (ThisLine == ServiceDBData)
//                 {
//                     //WS_DbgPrint(MIN_TRACE,("Line too long"));
//                     return WSANO_RECOVERY;
//                 }
//
//                 memmove(ServiceDBData, ThisLine, LineLen);
//
//                 ReadFile(ServicesFile,
//                          ServiceDBData + LineLen,
//                          sizeof( ServiceDBData )-1 - LineLen,
//                          &ReadSize,
//                          NULL);
//
//                 EndValid = ServiceDBData + LineLen + ReadSize;
//                 NextLine = ServiceDBData + LineLen;
//                 ThisLine = ServiceDBData;
//
//                 if (!ReadSize) break;
//             }
//         }
//
//         *NextLine = '\0';
//         Comment = strchr(ThisLine, '#');
//
//         if (Comment)
//             *Comment = '\0'; /* Terminate at comment start */
//
//         if (DecodeServEntFromString(ThisLine,
//                                     &ServiceName,
//                                     &PortNumberStr,
//                                     &ProtocolStr,
//                                     Aliases,
//                                     WS2_INTERNAL_MAX_ALIAS) &&
//             (strlen(nameProtoA) == 0 || strcmp(ProtocolStr, nameProtoA) == 0))
//         {
//             Found = (strcmp(ServiceName, nameServiceA) == 0 || strcmp(PortNumberStr, nameServiceA) == 0);
//             AliasPtr = Aliases;
//             while ((!Found) && (*AliasPtr != NULL))
//             {
//                 Found = (strcmp(*AliasPtr, nameServiceA) == 0);
//                 AliasPtr++;
//             }
//             if (Found)
//                 break;
//         }
//         NextLine++;
//         ThisLine = NextLine;
//     }
//
//     /* This we'll do no matter what */
//     CloseHandle(ServicesFile);
//
//     if (!Found)
//     {
//         return WSANO_DATA;
//     }
//
//     hostinfo->addr4 = 0;
//     hostinfo->servnameW = StrA2WHeapAlloc(hHeap, ServiceName);
//     hostinfo->servprotoW = StrA2WHeapAlloc(hHeap, ProtocolStr);
//     hostinfo->servaliasesA = StrAryCpyHeapAllocA(hHeap, (char**)&Aliases);
//     hostinfo->servport = atoi(PortNumberStr);
//
//     res = NO_ERROR;
//
// End:
//     if (nameA != NULL)
//         HeapFree(hHeap, 0, nameA);
//
//     if (nameServiceA != NULL)
//         HeapFree(hHeap, 0, nameServiceA);
//
//     return res;
// }
//
// INT
// NSP_LookupServiceNextW( PWSHANDLEINTERN data,
//                         DWORD dwControlFlags,
//                         LPWSAQUERYSETW lpRes,
//                         LPDWORD lpResLen)
// {
//     MSW_BUFFER buf;
//     WSHOSTINFOINTERN hostinfo;
//     INT result;
//     HANDLE hHeap = GetProcessHeap();
//     WCHAR* ServiceInstanceNameW = NULL;
//     /* cleanup-vars */
//     CHAR* ServiceInstanceNameA = NULL;
//     CHAR* ServiceProtocolNameA = NULL;
//
//     RtlZeroMemory(&hostinfo, sizeof(hostinfo));
//
//     /* init and build result-buffer */
//     mswBufferInit(&buf, (BYTE*)lpRes, *lpResLen);
//     mswBufferIncUsed(&buf, sizeof(*lpRes));
//
//     /* QueryDataSet-Size without "blob-data"-size! */
//     lpRes->dwSize = sizeof(*lpRes);
//     lpRes->dwNameSpace = NS_DNS;
//
//     if ((data->CallID == NSP_CALLID_HOSTNAME) ||
//         (data->CallID == NSP_CALLID_HOSTBYNAME) ||
//         (data->CallID == NSP_CALLID_SERVICEBYNAME))
//     {
//         if (data->CallIDCounter >= 1)
//         {
//             result = WSAENOMORE;
//             goto End;
//         }
//     }
//     else
//     {
//         result = WSANO_RECOVERY;
//         goto End;
//     }
//     data->CallIDCounter++;
//
//     if (data->CallID == NSP_CALLID_HOSTNAME)
//     {
//         result = NSP_GetHostNameHeapAllocW(&hostinfo.hostnameW);
//
//         if (result != ERROR_SUCCESS)
//             goto End;
//
//         hostinfo.addr4 = 0;
//     }
//     else if (data->CallID == NSP_CALLID_HOSTBYNAME)
//     {
//         result = NSP_GetHostByNameHeapAllocW(data,
//                                              dwControlFlags,
//                                              &hostinfo);
//         if (result != ERROR_SUCCESS)
//             goto End;
//     }
//     else
//     {
//         ASSERT(data->CallID == NSP_CALLID_SERVICEBYNAME);
//         result = NSP_GetServiceByNameHeapAllocW(data,
//                                                 dwControlFlags,
//                                                 &hostinfo);
//         if (result != ERROR_SUCCESS)
//             goto End;
//     }
//
//     if (((LUP_RETURN_BLOB & data->dwControlFlags) != 0) ||
//         ((LUP_RETURN_NAME & data->dwControlFlags) != 0))
//     {
//         if (data->CallID == NSP_CALLID_HOSTNAME || data->CallID == NSP_CALLID_HOSTBYNAME)
//         {
//             ServiceInstanceNameW = hostinfo.hostnameW;
//             ServiceInstanceNameA = StrW2AHeapAlloc(hHeap, ServiceInstanceNameW);
//             if (ServiceInstanceNameA == NULL)
//             {
//                 result = WSAEFAULT;
//                 goto End;
//
//             }
//         }
//         if (data->CallID == NSP_CALLID_SERVICEBYNAME)
//         {
//             ServiceInstanceNameW = hostinfo.servnameW;
//             ServiceInstanceNameA = StrW2AHeapAlloc(hHeap, ServiceInstanceNameW);
//             if (ServiceInstanceNameA == NULL)
//             {
//                 result = WSAEFAULT;
//                 goto End;
//
//             }
//             ServiceProtocolNameA = StrW2AHeapAlloc(hHeap, hostinfo.servprotoW);
//             if (ServiceProtocolNameA == NULL)
//             {
//                 result = WSAEFAULT;
//                 goto End;
//
//             }
//         }
//     }
//
//     if ((LUP_RETURN_ADDR & data->dwControlFlags) != 0)
//     {
//         if (!mswBufferAppendAddr_AddrInfoW(&buf, lpRes, hostinfo.addr4))
//         {
//             *lpResLen = buf.bytesUsed;
//             result = WSAEFAULT;
//             goto End;
//         }
//     }
//
//     if ((LUP_RETURN_BLOB & data->dwControlFlags) != 0)
//     {
//         if (data->CallID == NSP_CALLID_HOSTBYNAME)
//         {
//             /* Write data for PBLOB (hostent) */
//             if (!mswBufferAppendBlob_Hostent(&buf,
//                                              lpRes,
//                                              (LUP_RETURN_ALIASES & data->dwControlFlags) != 0 ? hostinfo.servaliasesA : NULL,
//                                              ServiceInstanceNameA,
//                                              hostinfo.addr4))
//             {
//                 *lpResLen = buf.bytesUsed;
//                 result = WSAEFAULT;
//                 goto End;
//             }
//         }
//         else if (data->CallID == NSP_CALLID_SERVICEBYNAME)
//         {
//             /* Write data for PBLOB (servent) */
//             if (!mswBufferAppendBlob_Servent(&buf,
//                                              lpRes,
//                                              ServiceInstanceNameA,/* ServiceName */
//                                              (LUP_RETURN_ALIASES & data->dwControlFlags) != 0 ? hostinfo.servaliasesA : NULL,
//                                              ServiceProtocolNameA,
//                                              hostinfo.servport))
//             {
//                 *lpResLen = buf.bytesUsed;
//                 result = WSAEFAULT;
//                 goto End;
//             }
//         }
//         else
//         {
//             result = WSANO_RECOVERY;
//             goto End;
//         }
//     }
//
//     if ((LUP_RETURN_NAME & data->dwControlFlags) != 0)
//     {
//         /* HostByName sets the ServiceInstanceName to a
//            (UNICODE)copy of hostent.h_name */
//         lpRes->lpszServiceInstanceName = (LPWSTR)mswBufferEndPtr(&buf);
//         if (!mswBufferAppendStrW(&buf, ServiceInstanceNameW))
//         {
//             lpRes->lpszServiceInstanceName = NULL;
//             *lpResLen = buf.bytesUsed;
//             result = WSAEFAULT;
//             goto End;
//         }
//     }
//
//     *lpResLen = buf.bytesUsed;
//
//     result = ERROR_SUCCESS;
// End:
//     /* cleanup */
//     if (ServiceInstanceNameA != NULL)
//         HeapFree(hHeap, 0, ServiceInstanceNameA);
//
//     if (ServiceProtocolNameA != NULL)
//         HeapFree(hHeap, 0, ServiceProtocolNameA);
//
//     if (hostinfo.hostnameW != NULL)
//         HeapFree(hHeap, 0, hostinfo.hostnameW);
//
//     if (hostinfo.servnameW != NULL)
//         HeapFree(hHeap, 0, hostinfo.servnameW);
//
//     if (hostinfo.servprotoW != NULL)
//         HeapFree(hHeap, 0, hostinfo.servprotoW);
//
//     return result;
// }
//
// /* Implementations - Exports */
// /*
//  * @implemented
//  */
// int
// WINAPI
// NSPStartup( LPGUID lpProviderId,
//             LPNSP_ROUTINE lpRout)
// {
//     INT ret;
//
//     if ((lpRout == NULL) ||
//         (lpRout->cbSize != sizeof(NSP_ROUTINE)))
//     {
//         WSASetLastError(ERROR_INVALID_PARAMETER);
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     mwsNSPInit();
//
//     /* set own Provider GUID - maybe we need
//        here to set the original mswsock-GUID?! */
//
//     /* Win2k3 returns
//        - Version 1.1
//        - no NSPIoctl
//        - sets cbSize to 44! */
//     lpRout->dwMajorVersion = 1;
//     lpRout->dwMinorVersion = 1;
//     lpRout->cbSize = sizeof(*lpRout) - sizeof(lpRout->NSPIoctl);
//     lpRout->NSPCleanup = &mwsNSPCleanUp;
//     lpRout->NSPLookupServiceBegin = &mwsNSPLookupServiceBegin;
//     lpRout->NSPLookupServiceNext = &mwsNSPLookupServiceNext;
//     lpRout->NSPLookupServiceEnd = &mwsNSPLookupServiceEnd;
//     lpRout->NSPSetService = &mwsNSPSetService;
//     lpRout->NSPInstallServiceClass = &mwsNSPInstallServiceClass;
//     lpRout->NSPRemoveServiceClass = &mwsNSPRemoveServiceClass;
//     lpRout->NSPGetServiceClassInfo = &mwsNSPGetServiceClassInfo;
//     lpRout->NSPIoctl = NULL;// &mwsNSPIoCtl;
//
//     ret = NO_ERROR;
//
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

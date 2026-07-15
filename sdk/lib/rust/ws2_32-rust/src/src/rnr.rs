use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dcatalog::WsTcGetEntryFromAf;
use crate::src::dllmain::WSASetLastError;
use crate::src::dprocess::WsProcGetNsCatalog;
use crate::src::nsquery::WsNqDereference;
use crate::src::nsquery::WsNqLookupServiceEnd;
use crate::src::nsquery::WsNqLookupServiceNext;
use crate::src::nsquery::WsNqValidateAndReference;
use crate::src::qshelpr::MapAnsiQuerySetToUnicode;
use crate::src::qshelpr::MapUnicodeQuerySetToAnsi;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut HostnameGuid: GUID = _GUID {
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

pub static mut AddressGuid: GUID = _GUID {
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

pub static mut HostAddrByNameGuid: GUID = _GUID {
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

pub static mut IANAGuid: GUID = _GUID {
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

pub static mut InetHostName: GUID = _GUID {
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

pub static mut Ipv6Guid: GUID = _GUID {
    Data1: ((0x9 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | 53 as ::core::ffi::c_int)
        as ::core::ffi::c_uint,
    Data2: 0 as ::core::ffi::c_ushort,
    Data3: 0x1c as ::core::ffi::c_ushort,
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

// No file-scope declarations to translate from the given preamble

const CP_ACP: u32 = 0;

#[no_mangle]
pub unsafe extern "system" fn WSAAddressToStringA(
    lpsaAddress: LPSOCKADDR,
    dwAddressLength: DWORD,
    lpProtocolInfo: LPWSAPROTOCOL_INFOA,
    lpszAddressString: LPSTR,
    lpdwAddressStringLength: LPDWORD,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Status: INT;
    let mut CatalogEntryId: DWORD;
    let Catalog: PTCATALOG;
    let mut CatalogEntry: PTCATALOG_ENTRY = core::ptr::null_mut();
    let UnicodeString: LPWSTR;
    let Length: DWORD = *lpdwAddressStringLength;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    UnicodeString = HeapAlloc(WsSockHeap, 0, (((Length as usize).wrapping_mul(2) as u64)) as SIZE_T) as LPWSTR;
    if UnicodeString.is_null() {
        SetLastError(WSAENOBUFS as DWORD);
        return SOCKET_ERROR;
    }

    Catalog = WsProcGetTCatalog(Process);

    if !lpProtocolInfo.is_null() {
        CatalogEntryId = (*lpProtocolInfo).dwCatalogEntryId;
        ErrorCode = (WsTcGetEntryFromCatalogEntryId(Catalog, CatalogEntryId, core::ptr::addr_of_mut!(CatalogEntry)) as i32);
    } else {
        ErrorCode = (WsTcGetEntryFromAf(Catalog, (*lpsaAddress).sa_family as INT, core::ptr::addr_of_mut!(CatalogEntry)) as i32);
    }

    if ErrorCode == (ERROR_SUCCESS as i32) {
        Status = ((*(*CatalogEntry).Provider).Service.lpWSPAddressToString.unwrap_unchecked())(
            lpsaAddress,
            dwAddressLength,
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo),
            UnicodeString,
            lpdwAddressStringLength,
            core::ptr::addr_of_mut!(ErrorCode),
        );
        if Status == (ERROR_SUCCESS as i32) {
            WideCharToMultiByte(
                CP_ACP,
                0,
                UnicodeString,
                -1,
                lpszAddressString,
                Length as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
        }

        WsTcEntryDereference(CatalogEntry);
        HeapFree(WsSockHeap, 0, UnicodeString as LPVOID);

        if Status == (ERROR_SUCCESS as i32) {
            return (ERROR_SUCCESS as i32);
        }
    } else {
        HeapFree(WsSockHeap, 0, UnicodeString as LPVOID);
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAAddressToStringW(
    lpsaAddress: LPSOCKADDR,
    dwAddressLength: DWORD,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    lpszAddressString: LPWSTR,
    lpdwAddressStringLength: LPDWORD,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Status: INT;
    let mut CatalogEntryId: DWORD;
    let Catalog: PTCATALOG;
    let mut CatalogEntry: PTCATALOG_ENTRY = core::ptr::null_mut();

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    Catalog = WsProcGetTCatalog(Process);

    if !lpProtocolInfo.is_null() {
        CatalogEntryId = (*lpProtocolInfo).dwCatalogEntryId;
        ErrorCode = WsTcGetEntryFromCatalogEntryId(
            Catalog,
            CatalogEntryId,
            core::ptr::addr_of_mut!(CatalogEntry),
        ) as INT;
    } else {
        ErrorCode = WsTcGetEntryFromAf(
            Catalog,
            (*lpsaAddress).sa_family as INT,
            core::ptr::addr_of_mut!(CatalogEntry),
        ) as INT;
    }

    if ErrorCode == (ERROR_SUCCESS as i32) {
        Status = ((*(*CatalogEntry).Provider).Service.lpWSPAddressToString.unwrap_unchecked())(
            lpsaAddress,
            dwAddressLength,
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo),
            lpszAddressString,
            lpdwAddressStringLength,
            core::ptr::addr_of_mut!(ErrorCode),
        );

        WsTcEntryDereference(CatalogEntry);

        if Status == (ERROR_SUCCESS as i32) {
            return (ERROR_SUCCESS as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSALookupServiceEnd(hLookup: HANDLE) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let Query: PNSQUERY = hLookup as PNSQUERY;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != ERROR_SUCCESS as INT {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    if IsBadReadPtr(
        Query as *const core::ffi::c_void,
        core::mem::size_of::<NSQUERY>(),
    ) != 0
        || WsNqValidateAndReference(Query) == 0
    {
        SetLastError(WSA_INVALID_HANDLE as DWORD);
        return SOCKET_ERROR;
    }

    ErrorCode = (WsNqLookupServiceEnd(Query) as i32);

    WsNqDereference(Query);

    WsNqDereference(Query);

    ERROR_SUCCESS as INT
}

#[no_mangle]
pub unsafe extern "system" fn WSALookupServiceBeginA(
    lpqsRestrictions: LPWSAQUERYSETA,
    dwControlFlags: DWORD,
    lphLookup: LPHANDLE,
) -> INT {

    let mut ErrorCode: INT;
    let mut UnicodeQuerySet: LPWSAQUERYSETW = core::ptr::null_mut();
    let mut UnicodeQuerySetSize: SIZE_T = 0;

    if IsBadReadPtr(
        lpqsRestrictions as *const core::ffi::c_void,
        core::mem::size_of::<WSAQUERYSETA>(),
    ) != 0
        || IsBadReadPtr(
            (*lpqsRestrictions).lpServiceClassId as *const core::ffi::c_void,
            core::mem::size_of::<GUID>(),
        ) != 0
    {
        SetLastError(WSAEFAULT as DWORD);
        return SOCKET_ERROR;
    }

    (*lpqsRestrictions).dwOutputFlags = 0;
    (*lpqsRestrictions).lpszComment = core::ptr::null_mut();
    (*lpqsRestrictions).dwNumberOfCsAddrs = 0;

    ErrorCode = MapAnsiQuerySetToUnicode(
        lpqsRestrictions,
        core::ptr::addr_of_mut!(UnicodeQuerySetSize),
        UnicodeQuerySet,
    );

    if ErrorCode == WSAEFAULT as INT {
        UnicodeQuerySet = HeapAlloc(WsSockHeap, 0, UnicodeQuerySetSize) as LPWSAQUERYSETW;
        if !UnicodeQuerySet.is_null() {
            ErrorCode = MapAnsiQuerySetToUnicode(
                lpqsRestrictions,
                core::ptr::addr_of_mut!(UnicodeQuerySetSize),
                UnicodeQuerySet,
            );
            if ErrorCode == ERROR_SUCCESS as INT {
                ErrorCode = WSALookupServiceBeginW(UnicodeQuerySet, dwControlFlags, lphLookup);
            }

            HeapFree(WsSockHeap, 0, UnicodeQuerySet as LPVOID);
        } else {
            ErrorCode = WSAEFAULT as INT;
        }
    }

    if ErrorCode != ERROR_SUCCESS as INT {
        SetLastError(ErrorCode as DWORD);
    }

    if ErrorCode == ERROR_SUCCESS as INT {
        ErrorCode
    } else {
        SOCKET_ERROR
    }
}

#[no_mangle]
pub unsafe extern "system" fn WSALookupServiceBeginW(
    lpqsRestrictions: LPWSAQUERYSETW,
    dwControlFlags: DWORD,
    lphLookup: LPHANDLE,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Query: PNSQUERY;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != ERROR_SUCCESS as INT {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    if IsBadWritePtr(
        lphLookup as LPVOID,
        core::mem::size_of::<LPHANDLE>(),
    ) != 0
        || IsBadReadPtr(
            lpqsRestrictions as *const core::ffi::c_void,
            core::mem::size_of::<WSAQUERYSETW>(),
        ) != 0
        || IsBadReadPtr(
            (*lpqsRestrictions).lpServiceClassId as *const core::ffi::c_void,
            core::mem::size_of::<LPGUID>(),
        ) != 0
    {
        SetLastError(WSAEFAULT as DWORD);
        return SOCKET_ERROR;
    }

    Query = WsNqAllocate();
    if !Query.is_null() {
        WsNqInitialize(Query);
        ErrorCode = WsNqLookupServiceBegin(
            Query,
            lpqsRestrictions,
            dwControlFlags,
            WsProcGetNsCatalog(Process),
        ) as INT;
        if ErrorCode == ERROR_SUCCESS as INT {
            *lphLookup = Query as HANDLE;
        } else {
            *lphLookup = core::ptr::null_mut();
            WsNqDelete(Query);
        }
    } else {
        ErrorCode = SOCKET_ERROR;
        SetLastError(WSAENOBUFS as DWORD);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WSALookupServiceNextW(
    hLookup: HANDLE,
    dwControlFlags: DWORD,
    lpdwBufferLength: LPDWORD,
    lpqsResults: LPWSAQUERYSETW,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let Query: PNSQUERY = hLookup as PNSQUERY;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != ERROR_SUCCESS as INT {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    if IsBadReadPtr(
        lpdwBufferLength as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>(),
    ) != 0
        || IsBadWritePtr(lpqsResults as LPVOID, (*lpdwBufferLength as usize)) != 0
    {
        SetLastError(WSAEFAULT as DWORD);
        return SOCKET_ERROR;
    }

    if IsBadReadPtr(
        Query as *const core::ffi::c_void,
        core::mem::size_of::<NSQUERY>(),
    ) != 0
        || WsNqValidateAndReference(Query) == 0
    {
        SetLastError(WSA_INVALID_HANDLE as DWORD);
        return SOCKET_ERROR;
    }

    ErrorCode = (WsNqLookupServiceNext(Query, dwControlFlags, lpdwBufferLength, lpqsResults) as i32);

    WsNqDereference(Query);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WSALookupServiceNextA(
    hLookup: HANDLE,
    dwControlFlags: DWORD,
    lpdwBufferLength: LPDWORD,
    lpqsResults: LPWSAQUERYSETA,
) -> INT {

    let mut UnicodeQuerySet: LPWSAQUERYSETW;
    let mut UnicodeQuerySetSize: DWORD;
    let mut ErrorCode: INT;

    if IsBadReadPtr(
        lpdwBufferLength as LPCVOID,
        (core::mem::size_of::<DWORD>() as UINT_PTR as usize),
    ) != 0
        || IsBadWritePtr(lpqsResults as LPVOID, ((*lpdwBufferLength) as UINT_PTR as usize)) != 0
    {
        SetLastError(WSAEFAULT as DWORD);
        return SOCKET_ERROR;
    }

    UnicodeQuerySetSize = *lpdwBufferLength;

    if (UnicodeQuerySetSize as usize) >= core::mem::size_of::<WSAQUERYSETW>() {
        UnicodeQuerySet = HeapAlloc(WsSockHeap, 0, UnicodeQuerySetSize as SIZE_T) as LPWSAQUERYSETW;
        if UnicodeQuerySet.is_null() {
            UnicodeQuerySetSize = 0;
        }
    } else {
        UnicodeQuerySet = core::ptr::null_mut();
        UnicodeQuerySetSize = 0;
    }

    ErrorCode = WSALookupServiceNextW(
        hLookup,
        dwControlFlags,
        core::ptr::addr_of_mut!(UnicodeQuerySetSize),
        UnicodeQuerySet,
    );

    if UnicodeQuerySet.is_null()
        && (*lpdwBufferLength as usize) >= core::mem::size_of::<WSAQUERYSETW>()
        && ErrorCode == SOCKET_ERROR
        && GetLastError() == WSAEFAULT as DWORD
    {
        UnicodeQuerySet = HeapAlloc(WsSockHeap, 0, UnicodeQuerySetSize as SIZE_T) as LPWSAQUERYSETW;
        if !UnicodeQuerySet.is_null() {
            ErrorCode = WSALookupServiceNextW(
                hLookup,
                dwControlFlags,
                core::ptr::addr_of_mut!(UnicodeQuerySetSize),
                UnicodeQuerySet,
            );
        }
    }

    if ErrorCode == ERROR_SUCCESS as INT {
        let mut SetSize: SIZE_T = (*lpdwBufferLength) as SIZE_T;

        ErrorCode = MapUnicodeQuerySetToAnsi(
            UnicodeQuerySet,
            core::ptr::addr_of_mut!(SetSize),
            lpqsResults,
        );
        if ErrorCode != ERROR_SUCCESS as INT {
            SetLastError(ErrorCode as DWORD);
        } else if SetSize > MAXDWORD as SIZE_T {
            ErrorCode = ERROR_ARITHMETIC_OVERFLOW as INT;
            SetLastError(ErrorCode as DWORD);
        } else {
            *lpdwBufferLength = SetSize as DWORD;
        }
    } else {
        if GetLastError() == WSAEFAULT as DWORD {
            *lpdwBufferLength = (UnicodeQuerySetSize as usize)
                .wrapping_add(
                    (core::mem::size_of::<ULONG>() * 6)
                        .wrapping_sub(6 * 1),
                ) as DWORD;
        }
    }

    if !UnicodeQuerySet.is_null() {
        HeapFree(WsSockHeap, 0, UnicodeQuerySet as LPVOID);
    }

    if ErrorCode == ERROR_SUCCESS as INT {
        ErrorCode
    } else {
        SOCKET_ERROR
    }
}

#[no_mangle]
pub unsafe extern "system" fn WSANSPIoctl(
    hLookup: HANDLE,
    dwControlCode: DWORD,
    lpvInBuffer: LPVOID,
    cbInBuffer: DWORD,
    lpvOutBuffer: LPVOID,
    cbOutBuffer: DWORD,
    lpcbBytesReturned: LPDWORD,
    lpCompletion: LPWSACOMPLETION,
) -> INT {
    return 0 as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WSARemoveServiceClass(lpServiceClassId: LPGUID) -> INT {
    SetLastError(WSAEINVAL as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSASetServiceA(
    lpqsRegInfo: LPWSAQUERYSETA,
    essOperation: WSAESETSERVICEOP,
    dwControlFlags: DWORD,
) -> INT {
    SetLastError(WSAEINVAL as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSASetServiceW(
    lpqsRegInfo: LPWSAQUERYSETW,
    essOperation: WSAESETSERVICEOP,
    dwControlFlags: DWORD,
) -> INT {

    SetLastError(WSAEINVAL as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAGetServiceClassInfoA(
    lpProviderId: LPGUID,
    lpServiceClassId: LPGUID,
    lpdwBufferLength: LPDWORD,
    lpServiceClassInfo: LPWSASERVICECLASSINFOA,
) -> INT {

    SetLastError(WSAEINVAL as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAGetServiceClassInfoW(
    lpProviderId: LPGUID,
    lpServiceClassId: LPGUID,
    lpdwBufferLength: LPDWORD,
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
) -> INT {

    SetLastError(WSAEINVAL as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAGetServiceClassNameByClassIdA(
    lpServiceClassId: LPGUID,
    lpszServiceClassName: LPSTR,
    lpdwBufferLength: LPDWORD,
) -> INT {

    SetLastError(WSAEINVAL as u32);
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn WSAGetServiceClassNameByClassIdW(
    lpServiceClassId: LPGUID,
    lpszServiceClassName: LPWSTR,
    lpdwBufferLength: LPDWORD,
) -> INT {
    SetLastError(WSAEINVAL as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAInstallServiceClassA(
    lpServiceClassInfo: LPWSASERVICECLASSINFOA,
) -> INT {

    SetLastError(WSAEINVAL as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAInstallServiceClassW(
    lpServiceClassInfo: LPWSASERVICECLASSINFOW,
) -> INT {
    SetLastError(WSAEINVAL as u32);
    return SOCKET_ERROR;
}
#[no_mangle]
pub unsafe extern "system" fn NSProviderInfoFromContext(
    Entry: PNSCATALOG_ENTRY,
    Context: PNSPROVIDER_ENUM_CONTEXT,
) {
    let size: i32 = if (*Context).Unicode != 0 {
        core::mem::size_of::<WSANAMESPACE_INFOW>()
    } else {
        core::mem::size_of::<WSANAMESPACE_INFOA>()
    } as i32;

    let size1: i32 = if !(*Entry).ProviderName.is_null() {
        (wcslen((*Entry).ProviderName) + 1) as i32
    } else {
        0
    };

    let size2: i32 = if (*Context).Unicode != 0 {
        size1 * core::mem::size_of::<u16>() as i32
    } else {
        size1 * core::mem::size_of::<i8>() as i32
    };

    let mut infoW: WSANAMESPACE_INFOW = core::mem::zeroed();

    infoW.dwNameSpace = (*Entry).NamespaceId;
    infoW.dwVersion = (*Entry).Version;
    infoW.fActive = (*Entry).Enabled as i32;

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*Entry).ProviderId) as *const u8,
        core::ptr::addr_of_mut!(infoW.NSProviderId) as *mut u8,
        core::mem::size_of::<LPGUID>(),
    );

    if size2 != 0 {
        infoW.lpszIdentifier = ((*Context).ProtocolBuffer as usize)
            .wrapping_add((*Context).BufferUsed as usize)
            .wrapping_add(size as usize) as LPWSTR;
    } else {
        infoW.lpszIdentifier = core::ptr::null_mut();
    }

    if ((*Context).BufferUsed as u32)
        .wrapping_add(size as u32)
        .wrapping_add(size2 as u32)
        <= (*Context).BufferLength
    {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(infoW) as *const u8,
            ((*Context).ProtocolBuffer as usize)
                .wrapping_add((*Context).BufferUsed as usize) as *mut u8,
            size as usize,
        );

        if size2 != 0 {
            if (*Context).Unicode != 0 {
                core::ptr::copy_nonoverlapping(
                    (*Entry).ProviderName as *const u8,
                    ((*Context).ProtocolBuffer as usize)
                        .wrapping_add((*Context).BufferUsed as usize)
                        .wrapping_add(size as usize) as *mut u8,
                    size2 as usize,
                );
            } else {
                WideCharToMultiByte(
                    0 as u32,
                    0,
                    (*Entry).ProviderName,
                    -1,
                    ((*Context).ProtocolBuffer as usize)
                        .wrapping_add((*Context).BufferUsed as usize)
                        .wrapping_add(size as usize) as LPSTR,
                    size2,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            }
        }

        (*Context).Count = (*Context).Count.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NSProvidersEnumerationProc(
    EnumContext: LPVOID,
    Entry: PNSCATALOG_ENTRY,
) -> BOOL {

    let Context: PNSPROVIDER_ENUM_CONTEXT = EnumContext as PNSPROVIDER_ENUM_CONTEXT;

    let size1: INT = (if !(*Entry).ProviderName.is_null() {
        wcslen((*Entry).ProviderName).wrapping_add(1)
    } else {
        0
    }) as INT;

    let size2: INT = (if (*Context).Unicode != 0 {
        (size1 as usize).wrapping_mul(core::mem::size_of::<u16>())
    } else {
        (size1 as usize).wrapping_mul(core::mem::size_of::<u8>())
    }) as INT;

    NSProviderInfoFromContext(Entry, Context);

    (*Context).BufferUsed = ((*Context).BufferUsed as u64).wrapping_add(
        (if (*Context).Unicode != 0 {
            (core::mem::size_of::<WSANAMESPACE_INFOW>()).wrapping_add(size2 as usize)
        } else {
            (core::mem::size_of::<WSANAMESPACE_INFOA>()).wrapping_add(size2 as usize)
        }) as u64,
    ) as DWORD;

    1
}

#[no_mangle]
pub unsafe extern "system" fn WSAEnumNameSpaceProvidersInternal(
    lpdwBufferLength: LPDWORD,
    lpnspBuffer: LPWSANAMESPACE_INFOA,
    Unicode: u8,
) -> i32 {
    let mut Status: i32;
    let WsProcess: PWSPROCESS;
    let Catalog: PNSCATALOG;
    let mut Context: NSPROVIDER_ENUM_CONTEXT = core::mem::zeroed();

    if lpdwBufferLength.is_null() {
        WSASetLastError((WSAEFAULT as i32));
        return SOCKET_ERROR;
    }

    WsProcess = WsGetProcess();
    Catalog = WsProcGetNsCatalog(WsProcess);
    if Catalog.is_null() {
        WSASetLastError((WSA_NOT_ENOUGH_MEMORY as i32));
        return SOCKET_ERROR;
    }

    Context.ProtocolBuffer = lpnspBuffer as LPVOID;
    Context.BufferLength = if !lpnspBuffer.is_null() {
        *lpdwBufferLength
    } else {
        0
    };
    Context.BufferUsed = 0;
    Context.Count = 0;
    Context.Unicode = Unicode;
    Context.ErrorCode = ERROR_SUCCESS as i32;

    WsNcEnumerateCatalogItems(Catalog, Some(NSProvidersEnumerationProc), core::ptr::addr_of_mut!(Context) as LPVOID);

    Status = Context.Count as i32;

    if Context.ErrorCode == ERROR_SUCCESS as i32 {
        if Context.BufferLength < Context.BufferUsed {
            *lpdwBufferLength = Context.BufferUsed;
            WSASetLastError((WSAEFAULT as i32));
            Status = SOCKET_ERROR;
        }
    } else {
        Status = SOCKET_ERROR;
        WSASetLastError(Context.ErrorCode);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn WSAEnumNameSpaceProvidersA(
    lpdwBufferLength: LPDWORD,
    lpnspBuffer: LPWSANAMESPACE_INFOA,
) -> i32 {
    return WSAEnumNameSpaceProvidersInternal(lpdwBufferLength, lpnspBuffer, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn WSAEnumNameSpaceProvidersW(
    lpdwBufferLength: LPDWORD,
    lpnspBuffer: LPWSANAMESPACE_INFOW,
) -> i32 {
    return WSAEnumNameSpaceProvidersInternal(
        lpdwBufferLength,
        lpnspBuffer as LPWSANAMESPACE_INFOA,
        1u8,
    );
}

#[no_mangle]
pub unsafe extern "system" fn WSAStringToAddressA(
    AddressString: LPSTR,
    AddressFamily: INT,
    lpProtocolInfo: LPWSAPROTOCOL_INFOA,
    lpAddress: LPSOCKADDR,
    lpAddressLength: LPINT,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Status: INT;
    let mut CatalogEntryId: DWORD;
    let Catalog: PTCATALOG;
    let mut CatalogEntry: PTCATALOG_ENTRY = core::ptr::null_mut();
    let UnicodeString: LPWSTR;
    let Length: DWORD = (strlen(AddressString as *const i8) as DWORD).wrapping_add(1);

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread),
    );
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    UnicodeString = HeapAlloc(WsSockHeap, 0, (((Length as usize).wrapping_mul(2) as u64)) as SIZE_T) as LPWSTR;
    if UnicodeString.is_null() {
        SetLastError(WSAENOBUFS as DWORD);
        return SOCKET_ERROR;
    }

    MultiByteToWideChar(
        CP_ACP,
        0,
        AddressString as *const i8,
        -1,
        UnicodeString,
        Length as i32,
    );

    Catalog = WsProcGetTCatalog(Process);

    if !lpProtocolInfo.is_null() {
        CatalogEntryId = (*lpProtocolInfo).dwCatalogEntryId;
        ErrorCode = (WsTcGetEntryFromCatalogEntryId(
            Catalog,
            CatalogEntryId,
            core::ptr::addr_of_mut!(CatalogEntry),
        ) as i32);
    } else {
        ErrorCode = (WsTcGetEntryFromAf(
            Catalog,
            AddressFamily,
            core::ptr::addr_of_mut!(CatalogEntry),
        ) as i32);
    }

    if ErrorCode == (ERROR_SUCCESS as i32) {
        Status = (*(*CatalogEntry).Provider)
            .Service
            .lpWSPStringToAddress.unwrap_unchecked()(
            UnicodeString,
            AddressFamily,
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo),
            lpAddress,
            lpAddressLength,
            core::ptr::addr_of_mut!(ErrorCode),
        );

        WsTcEntryDereference(CatalogEntry);

        HeapFree(WsSockHeap, 0, UnicodeString as LPVOID);

        if Status == (ERROR_SUCCESS as i32) {
            return (ERROR_SUCCESS as i32);
        }
    } else {
        HeapFree(WsSockHeap, 0, UnicodeString as LPVOID);
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAStringToAddressW(
    AddressString: LPWSTR,
    AddressFamily: INT,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    lpAddress: LPSOCKADDR,
    lpAddressLength: LPINT,
) -> INT {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: INT;
    let mut Status: INT;
    let mut CatalogEntryId: DWORD;
    let mut Catalog: PTCATALOG;
    let mut CatalogEntry: PTCATALOG_ENTRY = core::ptr::null_mut();

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode != (ERROR_SUCCESS as i32) {
        SetLastError(ErrorCode as DWORD);
        return SOCKET_ERROR;
    }

    Catalog = WsProcGetTCatalog(Process);

    if !lpProtocolInfo.is_null() {
        CatalogEntryId = (*lpProtocolInfo).dwCatalogEntryId;
        ErrorCode = WsTcGetEntryFromCatalogEntryId(
            Catalog,
            CatalogEntryId,
            core::ptr::addr_of_mut!(CatalogEntry),
        ) as INT;
    } else {
        ErrorCode = WsTcGetEntryFromAf(
            Catalog,
            AddressFamily,
            core::ptr::addr_of_mut!(CatalogEntry),
        ) as INT;
    }

    if ErrorCode == (ERROR_SUCCESS as i32) {
        Status = (*(*CatalogEntry).Provider)
            .Service
            .lpWSPStringToAddress.unwrap_unchecked()(
            AddressString,
            AddressFamily,
            core::ptr::addr_of_mut!((*CatalogEntry).ProtocolInfo),
            lpAddress,
            lpAddressLength,
            core::ptr::addr_of_mut!(ErrorCode),
        );

        WsTcEntryDereference(CatalogEntry);

        if Status == (ERROR_SUCCESS as i32) {
            return (ERROR_SUCCESS as i32);
        }
    }

    SetLastError(ErrorCode as DWORD);
    return SOCKET_ERROR;
}
// enforce_import_abi: moved 4 import decl(s) into the extern block their
// exporting DLL's .spec requires: HeapAlloc->system, HeapFree->system, MultiByteToWideChar->system, SetLastError->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/rnr.c
//  * PURPOSE:     Registration and Resolution Support
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAAddressToStringA(IN LPSOCKADDR lpsaAddress,
//                     IN DWORD dwAddressLength,
//                     IN LPWSAPROTOCOL_INFOA lpProtocolInfo,
//                     OUT LPSTR lpszAddressString,
//                     IN OUT  LPDWORD lpdwAddressStringLength)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode, Status;
//     DWORD CatalogEntryId;
//     PTCATALOG Catalog;
//     PTCATALOG_ENTRY CatalogEntry;
//     LPWSTR UnicodeString;
//     DWORD Length = *lpdwAddressStringLength;
//
//     DPRINT("WSAAddressToStringA: %p\n", lpsaAddress);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Allocate the unicode string */
//     UnicodeString = HeapAlloc(WsSockHeap, 0, Length * 2);
//     if (!UnicodeString)
//     {
//         /* No memory; fail */
//         SetLastError(WSAENOBUFS);
//         return SOCKET_ERROR;
//     }
//
//     /* Get the catalog */
//     Catalog = WsProcGetTCatalog(Process);
//
//     /* Check if we got custom protocol info */
//     if (lpProtocolInfo)
//     {
//         /* Get the entry ID */
//         CatalogEntryId = lpProtocolInfo->dwCatalogEntryId;
//
//         /* Get the entry associated with it */
//         ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                    CatalogEntryId,
//                                                    &CatalogEntry);
//     }
//     else
//     {
//         /* Get it from the address family */
//         ErrorCode = WsTcGetEntryFromAf(Catalog,
//                                        lpsaAddress->sa_family,
//                                        &CatalogEntry);
//     }
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Call the provider */
//         Status = CatalogEntry->Provider->Service.lpWSPAddressToString(lpsaAddress,
//                                                                       dwAddressLength,
//                                                                       &CatalogEntry->
//                                                                       ProtocolInfo,
//                                                                       UnicodeString,
//                                                                       lpdwAddressStringLength,
//                                                                       &ErrorCode);
//         if (Status == ERROR_SUCCESS)
//         {
//             /* Convert the string */
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 UnicodeString,
//                                 -1,
//                                 lpszAddressString,
//                                 Length,
//                                 NULL,
//                                 NULL);
//         }
//
//         /* Dereference the entry */
//         WsTcEntryDereference(CatalogEntry);
//
//         /* Free the unicode string */
//         HeapFree(WsSockHeap, 0, UnicodeString);
//
//         /* Check for success and return */
//         if (Status == ERROR_SUCCESS) return ERROR_SUCCESS;
//     }
//     else
//     {
//         /* Free the unicode string */
//         HeapFree(WsSockHeap, 0, UnicodeString);
//     }
//
//     /* Set the error and return */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAAddressToStringW(IN LPSOCKADDR lpsaAddress,
//                     IN DWORD dwAddressLength,
//                     IN LPWSAPROTOCOL_INFOW lpProtocolInfo,
//                     OUT LPWSTR lpszAddressString,
//                     IN OUT LPDWORD lpdwAddressStringLength)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode, Status;
//     DWORD CatalogEntryId;
//     PTCATALOG Catalog;
//     PTCATALOG_ENTRY CatalogEntry;
//
//     DPRINT("WSAAddressToStringW: %p\n", lpsaAddress);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Get the catalog */
//     Catalog = WsProcGetTCatalog(Process);
//
//     /* Check if we got custom protocol info */
//     if (lpProtocolInfo)
//     {
//         /* Get the entry ID */
//         CatalogEntryId = lpProtocolInfo->dwCatalogEntryId;
//
//         /* Get the entry associated with it */
//         ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                    CatalogEntryId,
//                                                    &CatalogEntry);
//     }
//     else
//     {
//         /* Get it from the address family */
//         ErrorCode = WsTcGetEntryFromAf(Catalog,
//                                        lpsaAddress->sa_family,
//                                        &CatalogEntry);
//     }
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Call the provider */
//         Status = CatalogEntry->Provider->Service.lpWSPAddressToString(lpsaAddress,
//                                                                       dwAddressLength,
//                                                                       &CatalogEntry->
//                                                                       ProtocolInfo,
//                                                                       lpszAddressString,
//                                                                       lpdwAddressStringLength,
//                                                                       &ErrorCode);
//
//         /* Dereference the entry */
//         WsTcEntryDereference(CatalogEntry);
//
//         /* Check for success and return */
//         if (Status == ERROR_SUCCESS) return ERROR_SUCCESS;
//     }
//
//     /* Set the error and return */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSALookupServiceEnd(IN HANDLE hLookup)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     PNSQUERY Query = hLookup;
//
//     DPRINT("WSALookupServiceEnd: %lx\n", hLookup);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Check for a valid handle, then validate and reference it */
//     if (IsBadReadPtr(Query, sizeof(*Query)) || !WsNqValidateAndReference(Query))
//     {
//         /* Fail */
//         SetLastError(WSA_INVALID_HANDLE);
//         return SOCKET_ERROR;
//     }
//
//     /* Do the lookup */
//     ErrorCode = WsNqLookupServiceEnd(Query);
//
//     /* Remove the validation reference */
//     WsNqDereference(Query);
//
//     /* Remove the keep-alive */
//     WsNqDereference(Query);
//
//     /* Return */
//     return ERROR_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSALookupServiceBeginA(IN LPWSAQUERYSETA lpqsRestrictions,
//                        IN DWORD dwControlFlags,
//                        OUT LPHANDLE lphLookup)
// {
//     INT ErrorCode;
//     LPWSAQUERYSETW UnicodeQuerySet = NULL;
//     SIZE_T UnicodeQuerySetSize = 0;
//
//     DPRINT("WSALookupServiceBeginA: %p\n", lpqsRestrictions);
//
//     /* Verify pointer */
//     if (IsBadReadPtr(lpqsRestrictions, sizeof(*lpqsRestrictions)) ||
//         IsBadReadPtr(lpqsRestrictions->lpServiceClassId, sizeof(*lpqsRestrictions->lpServiceClassId)))
//     {
//         /* Invalid */
//         SetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     /* Clear the reserved fields */
//     lpqsRestrictions->dwOutputFlags = 0;
//     lpqsRestrictions->lpszComment = NULL;
//     lpqsRestrictions->dwNumberOfCsAddrs = 0;
//
//     /* Find out the side we'll need */
//     ErrorCode = MapAnsiQuerySetToUnicode(lpqsRestrictions,
//                                          &UnicodeQuerySetSize,
//                                          UnicodeQuerySet);
//
//     /* We should've failed */
//     if (ErrorCode == WSAEFAULT)
//     {
//         /* Allocate the buffer we'll need */
//         UnicodeQuerySet = HeapAlloc(WsSockHeap, 0, UnicodeQuerySetSize);
//         if (UnicodeQuerySet)
//         {
//             /* Do the conversion for real */
//             ErrorCode = MapAnsiQuerySetToUnicode(lpqsRestrictions,
//                                                  &UnicodeQuerySetSize,
//                                                  UnicodeQuerySet);
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Now call the Unicode function */
//                 ErrorCode = WSALookupServiceBeginW(UnicodeQuerySet,
//                                                    dwControlFlags,
//                                                    lphLookup);
//             }
//
//             /* Free our buffer */
//             HeapFree(WsSockHeap, 0, UnicodeQuerySet);
//         }
//         else
//         {
//             /* No memory to allocate */
//             ErrorCode = WSAEFAULT;
//         }
//     }
//
//     /* Set the error in case of failure */
//     if (ErrorCode != ERROR_SUCCESS)
//         SetLastError(ErrorCode);
//
//     /* Return to caller */
//     return ErrorCode == ERROR_SUCCESS ? ErrorCode : SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// WINAPI
// WSALookupServiceBeginW(IN LPWSAQUERYSETW lpqsRestrictions,
//                        IN DWORD dwControlFlags,
//                        OUT LPHANDLE lphLookup)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     PNSQUERY Query;
//
//     DPRINT("WSALookupServiceBeginW: %p\n", lpqsRestrictions);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Verify pointers */
//     if (IsBadWritePtr(lphLookup, sizeof(*lphLookup)) ||
//         IsBadReadPtr(lpqsRestrictions, sizeof(*lpqsRestrictions)) ||
//         IsBadReadPtr(lpqsRestrictions->lpServiceClassId, sizeof(*lpqsRestrictions->lpServiceClassId)))
//     {
//         /* They are invalid; fail */
//         SetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     /* Create a new query object */
//     if ((Query = WsNqAllocate()))
//     {
//         /* Initialize it */
//         WsNqInitialize(Query);
//
//         /* Do the lookup */
//         ErrorCode = WsNqLookupServiceBegin(Query,
//                                            lpqsRestrictions,
//                                            dwControlFlags,
//                                            WsProcGetNsCatalog(Process));
//
//         /* Check for success */
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             /* Return the handle */
//             *lphLookup = Query;
//         }
//         else
//         {
//             /* Fail */
//             *lphLookup = NULL;
//             WsNqDelete(Query);
//         }
//     }
//     else
//     {
//         /* No memory */
//         ErrorCode = SOCKET_ERROR;
//         SetLastError(WSAENOBUFS);
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// /*
//  * @implemented
//  */
// INT
// WINAPI
// WSALookupServiceNextW(IN HANDLE hLookup,
//                       IN DWORD dwControlFlags,
//                       IN OUT LPDWORD lpdwBufferLength,
//                       OUT LPWSAQUERYSETW lpqsResults)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     PNSQUERY Query = hLookup;
//
//     DPRINT("WSALookupServiceNextW: %lx\n", hLookup);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /*
//      * Verify pointers. Note that the size of the buffer
//      * pointed by lpqsResults is given by *lpdwBufferLength.
//      */
//     if (IsBadReadPtr(lpdwBufferLength, sizeof(*lpdwBufferLength)) ||
//         IsBadWritePtr(lpqsResults, *lpdwBufferLength))
//     {
//         /* It is invalid; fail */
//         SetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     /* Check for a valid handle, then validate and reference it */
//     if (IsBadReadPtr(Query, sizeof(*Query)) || !WsNqValidateAndReference(Query))
//     {
//         /* Fail */
//         SetLastError(WSA_INVALID_HANDLE);
//         return SOCKET_ERROR;
//     }
//
//     /* Do the lookup */
//     ErrorCode = WsNqLookupServiceNext(Query,
//                                       dwControlFlags,
//                                       lpdwBufferLength,
//                                       lpqsResults);
//
//     /* Remove the validation reference */
//     WsNqDereference(Query);
//
//     /* Return */
//     return ErrorCode;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSALookupServiceNextA(IN HANDLE hLookup,
//                       IN DWORD dwControlFlags,
//                       IN OUT LPDWORD lpdwBufferLength,
//                       OUT LPWSAQUERYSETA lpqsResults)
// {
//     LPWSAQUERYSETW UnicodeQuerySet;
//     DWORD UnicodeQuerySetSize;
//     INT ErrorCode;
//
//     DPRINT("WSALookupServiceNextA: %lx\n", hLookup);
//
//     /*
//      * Verify pointers. Note that the size of the buffer
//      * pointed by lpqsResults is given by *lpdwBufferLength.
//      */
//     if (IsBadReadPtr(lpdwBufferLength, sizeof(*lpdwBufferLength)) ||
//         IsBadWritePtr(lpqsResults, *lpdwBufferLength))
//     {
//         /* It is invalid; fail */
//         SetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     UnicodeQuerySetSize = *lpdwBufferLength;
//
//     /* Check how much the user is giving */
//     if (UnicodeQuerySetSize >= sizeof(WSAQUERYSETW))
//     {
//         /* Allocate the buffer we'll use */
//         UnicodeQuerySet = HeapAlloc(WsSockHeap, 0, UnicodeQuerySetSize);
//         if (!UnicodeQuerySet)
//         {
//             /*
//              * We failed, possibly because the specified size was too large?
//              * Retrieve the needed buffer size with the WSALookupServiceNextW
//              * call and retry again a second time.
//              */
//             UnicodeQuerySetSize = 0;
//         }
//     }
//     else
//     {
//         /*
//          * The buffer is too small. Retrieve the needed buffer size with
//          * the WSALookupServiceNextW call and return it to the caller.
//          */
//         UnicodeQuerySet = NULL;
//         UnicodeQuerySetSize = 0;
//     }
//
//     /* Call the Unicode Function */
//     ErrorCode = WSALookupServiceNextW(hLookup,
//                                       dwControlFlags,
//                                       &UnicodeQuerySetSize,
//                                       UnicodeQuerySet);
//
//     /*
//      * Check whether we actually just retrieved the needed buffer size
//      * because our previous local allocation did fail. If so, allocate
//      * a new buffer and retry again.
//      */
//     if ( (!UnicodeQuerySet) && (*lpdwBufferLength >= sizeof(WSAQUERYSETW)) &&
//          (ErrorCode == SOCKET_ERROR) && (GetLastError() == WSAEFAULT) )
//     {
//         /* Allocate the buffer we'll use */
//         UnicodeQuerySet = HeapAlloc(WsSockHeap, 0, UnicodeQuerySetSize);
//         if (UnicodeQuerySet)
//         {
//             /* Call the Unicode Function */
//             ErrorCode = WSALookupServiceNextW(hLookup,
//                                               dwControlFlags,
//                                               &UnicodeQuerySetSize,
//                                               UnicodeQuerySet);
//         }
//         /*
//          * Otherwise the allocation failed and we
//          * fall back into the error checks below.
//          */
//     }
//
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         SIZE_T SetSize = *lpdwBufferLength;
//
//         /* Now convert back to ANSI */
//         ErrorCode = MapUnicodeQuerySetToAnsi(UnicodeQuerySet,
//                                              &SetSize,
//                                              lpqsResults);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             SetLastError(ErrorCode);
//         }
//         else if (SetSize > MAXDWORD)
//         {
//             ErrorCode = ERROR_ARITHMETIC_OVERFLOW;
//             SetLastError(ErrorCode);
//         }
//         else
//         {
//             *lpdwBufferLength = SetSize;
//         }
//     }
//     else
//     {
//         /* Check if we ran out of space */
//         if (GetLastError() == WSAEFAULT)
//         {
//             /* Return how much space we'll need, including padding */
//             *lpdwBufferLength = UnicodeQuerySetSize +
//                                 ((sizeof(ULONG) * 6) - (6 * 1));
//         }
//     }
//
//     /* If we had a local buffer, free it */
//     if (UnicodeQuerySet)
//         HeapFree(WsSockHeap, 0, UnicodeQuerySet);
//
//     /* Return to caller */
//     return (ErrorCode == ERROR_SUCCESS) ? ErrorCode : SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSANSPIoctl(HANDLE hLookup,
//             DWORD dwControlCode,
//             LPVOID lpvInBuffer,
//             DWORD cbInBuffer,
//             LPVOID lpvOutBuffer,
//             DWORD cbOutBuffer,
//             LPDWORD lpcbBytesReturned,
//             LPWSACOMPLETION lpCompletion)
// {
//     DPRINT("WSANSPIoctl: %lx\n", hLookup);
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSARemoveServiceClass(IN LPGUID lpServiceClassId)
// {
//     DPRINT("WSARemoveServiceClass: %lx\n", lpServiceClassId);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSASetServiceA(IN LPWSAQUERYSETA lpqsRegInfo,
//                IN WSAESETSERVICEOP essOperation,
//                IN DWORD dwControlFlags)
// {
//     DPRINT("WSASetServiceA: %lx\n", lpqsRegInfo);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSASetServiceW(IN LPWSAQUERYSETW lpqsRegInfo,
//                IN WSAESETSERVICEOP essOperation,
//                IN DWORD dwControlFlags)
// {
//     DPRINT("WSASetServiceW: %lx\n", lpqsRegInfo);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSAGetServiceClassInfoA(IN LPGUID lpProviderId,
//                         IN LPGUID lpServiceClassId,
//                         IN OUT LPDWORD lpdwBufferLength,
//                         OUT LPWSASERVICECLASSINFOA lpServiceClassInfo)
// {
//     DPRINT("WSAGetServiceClassInfoA: %lx\n", lpProviderId);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSAGetServiceClassInfoW(IN LPGUID lpProviderId,
//                         IN LPGUID lpServiceClassId,
//                         IN OUT LPDWORD lpdwBufferLength,
//                         OUT LPWSASERVICECLASSINFOW lpServiceClassInfo)
// {
//     DPRINT("WSAGetServiceClassInfoW: %lx\n", lpProviderId);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSAGetServiceClassNameByClassIdA(IN LPGUID lpServiceClassId,
//                                  OUT LPSTR lpszServiceClassName,
//                                  IN OUT LPDWORD lpdwBufferLength)
// {
//     DPRINT("WSAGetServiceClassNameByClassIdA: %lx\n", lpServiceClassId);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSAGetServiceClassNameByClassIdW(IN LPGUID lpServiceClassId,
//                                  OUT LPWSTR lpszServiceClassName,
//                                  IN OUT LPDWORD lpdwBufferLength)
// {
//     DPRINT("WSAGetServiceClassNameByClassIdW: %lx\n", lpServiceClassId);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WSAInstallServiceClassA(IN LPWSASERVICECLASSINFOA lpServiceClassInfo)
// {
//     DPRINT("WSAInstallServiceClassA: %lx\n", lpServiceClassInfo);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// /*
// * @unimplemented
// */
// INT
// __stdcall
// WSAInstallServiceClassW(IN LPWSASERVICECLASSINFOW lpServiceClassInfo)
// {
//     DPRINT("WSAInstallServiceClassW: %lx\n", lpServiceClassInfo);
//     SetLastError(WSAEINVAL);
//     return SOCKET_ERROR;
// }
//
// VOID
// __stdcall
// NSProviderInfoFromContext(IN PNSCATALOG_ENTRY Entry,
//     IN PNSPROVIDER_ENUM_CONTEXT Context)
// {
//     INT size = Context->Unicode ? sizeof(WSANAMESPACE_INFOW) : sizeof(WSANAMESPACE_INFOA);
//     /* Calculate ProviderName string size */
//     INT size1 = Entry->ProviderName ? wcslen(Entry->ProviderName) + 1 : 0;
//     INT size2 = Context->Unicode ? size1 * sizeof(WCHAR) : size1 * sizeof(CHAR);
//     WSANAMESPACE_INFOW infoW;
//     /* Fill NS Provider data */
//     infoW.dwNameSpace = Entry->NamespaceId;
//     infoW.dwVersion = Entry->Version;
//     infoW.fActive = Entry->Enabled;
//     RtlMoveMemory(&infoW.NSProviderId,
//         &Entry->ProviderId,
//         sizeof(infoW.NSProviderId));
//     if (size2)
//     {
//         /* Calculate ProviderName string pointer */
//         infoW.lpszIdentifier = (LPWSTR)((ULONG_PTR)Context->ProtocolBuffer +
//             Context->BufferUsed + size);
//     }
//     else
//     {
//         infoW.lpszIdentifier = NULL;
//     }
//
//     /* Check if we'll have space */
//     if ((Context->BufferUsed + size + size2) <=
//         (Context->BufferLength))
//     {
//         /* Copy the data */
//         RtlMoveMemory((PVOID)((ULONG_PTR)Context->ProtocolBuffer +
//             Context->BufferUsed),
//             &infoW,
//             size);
//         if (size2)
//         {
//             /* Entry->ProviderName is LPWSTR */
//             if (Context->Unicode)
//             {
//                 RtlMoveMemory((PVOID)((ULONG_PTR)Context->ProtocolBuffer +
//                     Context->BufferUsed + size),
//                     Entry->ProviderName,
//                     size2);
//             }
//             else
//             {
//                 /* Call the conversion function */
//                 WideCharToMultiByte(CP_ACP,
//                     0,
//                     Entry->ProviderName,
//                     -1,
//                     (LPSTR)((ULONG_PTR)Context->ProtocolBuffer +
//                         Context->BufferUsed + size),
//                     size2,
//                     NULL,
//                     NULL);
//
//             }
//         }
//
//         /* Increase the count */
//         Context->Count++;
//     }
// }
//
// BOOL
// __stdcall
// NSProvidersEnumerationProc(PVOID EnumContext,
//     PNSCATALOG_ENTRY Entry)
// {
//     PNSPROVIDER_ENUM_CONTEXT Context = (PNSPROVIDER_ENUM_CONTEXT)EnumContext;
//
//     /* Calculate ProviderName string size */
//     INT size1 = Entry->ProviderName ? wcslen(Entry->ProviderName) + 1 : 0;
//     INT size2 = Context->Unicode ? size1 * sizeof(WCHAR) : size1 * sizeof(CHAR);
//
//     /* Copy the information */
//     NSProviderInfoFromContext(Entry, Context);
//     Context->BufferUsed += Context->Unicode ? (sizeof(WSANAMESPACE_INFOW)+size2) : (sizeof(WSANAMESPACE_INFOA)+size2);
//
//     /* Continue enumeration */
//     return TRUE;
// }
//
// INT
// __stdcall
// WSAEnumNameSpaceProvidersInternal(IN OUT LPDWORD lpdwBufferLength,
//     OUT LPWSANAMESPACE_INFOA lpnspBuffer, BOOLEAN Unicode)
// {
//     INT Status;
//     PWSPROCESS WsProcess;
//     PNSCATALOG Catalog;
//     NSPROVIDER_ENUM_CONTEXT Context;
//
//     DPRINT("WSAEnumNameSpaceProvidersInternal: %lx\n", lpnspBuffer);
//
//     if (!lpdwBufferLength)
//     {
//         WSASetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//     WsProcess = WsGetProcess();
//     /* Create a catalog object from the current one */
//     Catalog = WsProcGetNsCatalog(WsProcess);
//     if (!Catalog)
//     {
//         /* Fail if we couldn't */
//         WSASetLastError(WSA_NOT_ENOUGH_MEMORY);
//         return SOCKET_ERROR;
//     }
//
//     Context.ProtocolBuffer = lpnspBuffer;
//     Context.BufferLength = lpnspBuffer ? *lpdwBufferLength : 0;
//     Context.BufferUsed = 0;
//     Context.Count = 0;
//     Context.Unicode = Unicode;
//     Context.ErrorCode = ERROR_SUCCESS;
//
//     WsNcEnumerateCatalogItems(Catalog, NSProvidersEnumerationProc, &Context);
//
//     /* Get status */
//     Status = Context.Count;
//
//     /* Check the error code */
//     if (Context.ErrorCode == ERROR_SUCCESS)
//     {
//         /* Check if enough space was available */
//         if (Context.BufferLength < Context.BufferUsed)
//         {
//             /* Fail and tell them how much we need */
//             *lpdwBufferLength = Context.BufferUsed;
//             WSASetLastError(WSAEFAULT);
//             Status = SOCKET_ERROR;
//         }
//     }
//     else
//     {
//         /* Failure, normalize error */
//         Status = SOCKET_ERROR;
//         WSASetLastError(Context.ErrorCode);
//     }
//
//     /* Return */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAEnumNameSpaceProvidersA(IN OUT LPDWORD lpdwBufferLength,
//                            OUT LPWSANAMESPACE_INFOA lpnspBuffer)
// {
//     DPRINT("WSAEnumNameSpaceProvidersA: %lx\n", lpnspBuffer);
//     return WSAEnumNameSpaceProvidersInternal(lpdwBufferLength, (LPWSANAMESPACE_INFOA)lpnspBuffer, FALSE);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAEnumNameSpaceProvidersW(IN OUT LPDWORD lpdwBufferLength,
//                            OUT LPWSANAMESPACE_INFOW lpnspBuffer)
// {
//     DPRINT("WSAEnumNameSpaceProvidersW: %lx\n", lpnspBuffer);
//     return WSAEnumNameSpaceProvidersInternal(lpdwBufferLength, (LPWSANAMESPACE_INFOA)lpnspBuffer, TRUE);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAStringToAddressA(IN LPSTR AddressString,
//                     IN INT AddressFamily,
//                     IN LPWSAPROTOCOL_INFOA lpProtocolInfo,
//                     OUT LPSOCKADDR lpAddress,
//                     IN OUT  LPINT lpAddressLength)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode, Status;
//     DWORD CatalogEntryId;
//     PTCATALOG Catalog;
//     PTCATALOG_ENTRY CatalogEntry;
//     LPWSTR UnicodeString;
//     DWORD Length = (DWORD)strlen(AddressString) + 1;
//
//     DPRINT("WSAStringToAddressA: %s\n", AddressString);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Allocate the unicode string */
//     UnicodeString = HeapAlloc(WsSockHeap, 0, Length * 2);
//     if (!UnicodeString)
//     {
//         /* No memory; fail */
//         SetLastError(WSAENOBUFS);
//         return SOCKET_ERROR;
//     }
//
//     /* Convert the string */
//     MultiByteToWideChar(CP_ACP, 0, AddressString, -1, UnicodeString, Length);
//
//     /* Get the catalog */
//     Catalog = WsProcGetTCatalog(Process);
//
//     /* Check if we got custom protocol info */
//     if (lpProtocolInfo)
//     {
//         /* Get the entry ID */
//         CatalogEntryId = lpProtocolInfo->dwCatalogEntryId;
//
//         /* Get the entry associated with it */
//         ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                    CatalogEntryId,
//                                                    &CatalogEntry);
//     }
//     else
//     {
//         /* Get it from the address family */
//         ErrorCode = WsTcGetEntryFromAf(Catalog, AddressFamily, &CatalogEntry);
//     }
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Call the provider */
//         Status = CatalogEntry->Provider->Service.lpWSPStringToAddress(UnicodeString,
//                                                               AddressFamily,
//                                                               &CatalogEntry->
//                                                               ProtocolInfo,
//                                                               lpAddress,
//                                                               lpAddressLength,
//                                                               &ErrorCode);
//
//         /* Dereference the entry */
//         WsTcEntryDereference(CatalogEntry);
//
//         /* Free the unicode string */
//         HeapFree(WsSockHeap, 0, UnicodeString);
//
//         /* Check for success and return */
//         if (Status == ERROR_SUCCESS) return ERROR_SUCCESS;
//     }
//     else
//     {
//         /* Free the unicode string */
//         HeapFree(WsSockHeap, 0, UnicodeString);
//     }
//
//     /* Set the error and return */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAStringToAddressW(IN LPWSTR AddressString,
//                     IN INT AddressFamily,
//                     IN LPWSAPROTOCOL_INFOW lpProtocolInfo,
//                     OUT LPSOCKADDR lpAddress,
//                     IN OUT LPINT lpAddressLength)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode, Status;
//     DWORD CatalogEntryId;
//     PTCATALOG Catalog;
//     PTCATALOG_ENTRY CatalogEntry;
//
//     DPRINT("WSAStringToAddressW: %S\n", AddressString);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         /* Leave now */
//         SetLastError(ErrorCode);
//         return SOCKET_ERROR;
//     }
//
//     /* Get the catalog */
//     Catalog = WsProcGetTCatalog(Process);
//
//     /* Check if we got custom protocol info */
//     if (lpProtocolInfo)
//     {
//         /* Get the entry ID */
//         CatalogEntryId = lpProtocolInfo->dwCatalogEntryId;
//
//         /* Get the entry associated with it */
//         ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                    CatalogEntryId,
//                                                    &CatalogEntry);
//     }
//     else
//     {
//         /* Get it from the address family */
//         ErrorCode = WsTcGetEntryFromAf(Catalog, AddressFamily, &CatalogEntry);
//     }
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Call the provider */
//         Status = CatalogEntry->Provider->Service.lpWSPStringToAddress(AddressString,
//                                                                       AddressFamily,
//                                                                       &CatalogEntry->
//                                                                       ProtocolInfo,
//                                                                       lpAddress,
//                                                                       lpAddressLength,
//                                                                       &ErrorCode);
//
//         /* Dereference the entry */
//         WsTcEntryDereference(CatalogEntry);
//
//         /* Check for success and return */
//         if (Status == ERROR_SUCCESS) return ERROR_SUCCESS;
//     }
//
//     /* Set the error and return */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

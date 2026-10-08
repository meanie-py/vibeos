use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::addrconv::htonl;
use crate::src::getxbyxx::gethostbyname;
use crate::src::rnr::WSAAddressToStringW;
use crate::src::rnr::WSALookupServiceBeginW;
use crate::src::rnr::WSALookupServiceNextW;


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

/// `#define FreeAddrInfoW(a) freeaddrinfo((LPADDRINFO)a)` (addrinfo.c:21) — a macro,
/// so it stays a macro. The C casts to the ANSI LPADDRINFO; preserve that.
macro_rules! FreeAddrInfoW {
    ($a:expr) => {
        freeaddrinfo($a as LPADDRINFO)
    };
}

const EAI_MEMORY: i32 = -10;
const EAI_NONAME: i32 = -11;
const EAI_AGAIN: i32 = -12;
const EAI_FAIL: i32 = -13;
const EAI_NODATA: i32 = -14;
#[no_mangle]
pub unsafe extern "system" fn Dns_Ip4AddressToReverseName_W(
    AddressBuffer: LPWSTR,
    Address: IN_ADDR,
) {
    wsprintfW(
        AddressBuffer,
        win32macros_rust::wide_bytes!(b"%\0u\0.\0%\0u\0.\0%\0u\0.\0%\0u\0.\0i\0n\0-\0a\0d\0d\0r\0.\0a\0r\0p\0a\0.\0\0\0") as PCWSTR,
        Address.S_un.S_un_b.s_b4 as i32,
        Address.S_un.S_un_b.s_b3 as i32,
        Address.S_un.S_un_b.s_b2 as i32,
        Address.S_un.S_un_b.s_b1 as i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn Dns_SplitHostFromDomainNameW(DomainName: LPWSTR) {
}
unsafe extern "system" fn ConvertAddrinfoFromUnicodeToAnsi(mut Addrinfo: PADDRINFOW) -> i32 {
    let mut AnsiName: LPSTR;
    let mut UnicodeName: *mut LPWSTR;

    if !Addrinfo.is_null() {
        loop {
            UnicodeName = core::ptr::addr_of_mut!((*Addrinfo).ai_canonname);

            if !(*UnicodeName).is_null() {
                AnsiName = AnsiDupFromUnicode(*UnicodeName);
                if !AnsiName.is_null() {
                    HeapFree(WsSockHeap, 0, *UnicodeName as *mut core::ffi::c_void);
                    *UnicodeName = AnsiName as LPWSTR;
                } else {
                    return GetLastError() as i32;
                }
            }

            Addrinfo = (*Addrinfo).ai_next;
            if Addrinfo.is_null() {
                break;
            }
        }
    }

    ERROR_SUCCESS as i32
}
unsafe extern "system" fn ParseV4Address(AddressString: PCWSTR, pAddress: PDWORD) -> u8 {
    let mut AnsiAddressString: [i8; 128] = [0; 128];
    let mut cp: *mut i8 = AnsiAddressString.as_mut_ptr();
    let mut val: u32;
    let mut base: u32;
    let mut c: u8;
    let mut parts: [u32; 4] = [0; 4];
    let mut pp: *mut u32 = parts.as_mut_ptr();

    if AddressString.is_null() {
        return 0u8;
    }

    WideCharToMultiByte(
        0,
        0,
        (AddressString as *mut u16),
        -1,
        AnsiAddressString.as_mut_ptr(),
        core::mem::size_of::<[i8; 128]>() as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    if isdigit(*cp as i32) == 0 {
        return 0u8;
    }

    loop {
        val = 0;
        base = 10;

        if *cp as i32 == '0' as i32 {
            cp = cp.offset(1);
            if *cp as i32 == 'x' as i32 || *cp as i32 == 'X' as i32 {
                base = 16;
                cp = cp.offset(1);
            } else {
                base = 8;
            }
        }

        loop {
            c = *cp as u8;
            if c == 0 {
                break;
            }

            if isdigit(c as i32) != 0 {
                val = val.wrapping_mul(base).wrapping_add((c as i32 - '0' as i32) as u32);
                cp = cp.offset(1);
            } else {
                if !(base == 16 && isxdigit(c as i32) != 0) {
                    break;
                }
                val = (val << 4).wrapping_add(
                    (c as i32 + 10 - (if islower(c as i32) != 0 {
                        'a' as i32
                    } else {
                        'A' as i32
                    })) as u32
                );
                cp = cp.offset(1);
            }
        }

        if *cp as i32 != '.' as i32 {
            break;
        }

        if pp >= parts.as_mut_ptr().offset(4) {
            return 0u8;
        }
        *pp = val;
        pp = pp.offset(1);
        cp = cp.offset(1);
    }

    if *cp != 0 {
        return 0u8;
    }

    if pp >= parts.as_mut_ptr().offset(4) {
        return 0u8;
    }
    *pp = val;
    pp = pp.offset(1);

    if pp.offset_from(parts.as_mut_ptr()) as u32 != 4 {
        return 0u8;
    }

    if parts[0] > 0xff || parts[1] > 0xff || parts[2] > 0xff || parts[3] > 0xff {
        return 0u8;
    }

    val = (parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3];

    if !pAddress.is_null() {
        *pAddress = htonl(val);
    }

    1u8
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn NewAddrInfo(
    SocketType: i32,
    Protocol: i32,
    Port: u16,
    Address: u32,
) -> PADDRINFOW {
    let AddrInfo: PADDRINFOW;
    let SockAddress: PSOCKADDR_IN;

    AddrInfo = HeapAlloc(
        WsSockHeap,
        0,
        (core::mem::size_of::<ADDRINFOW>() as u32),
    ) as PADDRINFOW;
    if AddrInfo.is_null() {
        return core::ptr::null_mut();
    }

    SockAddress = HeapAlloc(
        WsSockHeap,
        0,
        (core::mem::size_of::<SOCKADDR_IN>() as u32),
    ) as PSOCKADDR_IN;
    if SockAddress.is_null() {
        HeapFree(WsSockHeap, 0, AddrInfo as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*SockAddress).sin_family = AF_INET as i16;
    (*SockAddress).sin_port = Port;
    (*SockAddress).sin_addr.S_un.S_addr = Address;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*SockAddress).sin_zero) as *mut u8,
        0,
        core::mem::size_of_val(&(*SockAddress).sin_zero),
    );

    (*AddrInfo).ai_family = (PF_INET as i32);
    (*AddrInfo).ai_socktype = SocketType;
    (*AddrInfo).ai_protocol = Protocol;
    (*AddrInfo).ai_flags = 0;
    (*AddrInfo).ai_next = core::ptr::null_mut();
    (*AddrInfo).ai_canonname = core::ptr::null_mut();
    (*AddrInfo).ai_addrlen = core::mem::size_of::<SOCKADDR_IN>();
    (*AddrInfo).ai_addr = SockAddress as PSOCKADDR;

    return AddrInfo;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn NewAddrInfo(
    SocketType: i32,
    Protocol: i32,
    Port: u16,
    Address: u32,
) -> PADDRINFOW {
    let AddrInfo: PADDRINFOW;
    let SockAddress: PSOCKADDR_IN;

    AddrInfo = HeapAlloc(
        WsSockHeap,
        0,
        (core::mem::size_of::<ADDRINFOW>() as SIZE_T),
    ) as PADDRINFOW;
    if AddrInfo.is_null() {
        return core::ptr::null_mut();
    }

    SockAddress = HeapAlloc(
        WsSockHeap,
        0,
        (core::mem::size_of::<SOCKADDR_IN>() as SIZE_T),
    ) as PSOCKADDR_IN;
    if SockAddress.is_null() {
        HeapFree(WsSockHeap, 0, AddrInfo as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*SockAddress).sin_family = AF_INET as i16;
    (*SockAddress).sin_port = Port;
    (*SockAddress).sin_addr.S_un.S_addr = Address;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*SockAddress).sin_zero) as *mut u8,
        0,
        core::mem::size_of_val(&(*SockAddress).sin_zero),
    );

    (*AddrInfo).ai_family = (PF_INET as i32);
    (*AddrInfo).ai_socktype = SocketType;
    (*AddrInfo).ai_protocol = Protocol;
    (*AddrInfo).ai_flags = 0;
    (*AddrInfo).ai_next = core::ptr::null_mut();
    (*AddrInfo).ai_canonname = core::ptr::null_mut();
    (*AddrInfo).ai_addrlen = core::mem::size_of::<SOCKADDR_IN>();
    (*AddrInfo).ai_addr = SockAddress as PSOCKADDR;

    return AddrInfo;
}
unsafe extern "system" fn CloneAddrInfo(Port: u16, ptResult: PADDRINFOW) -> i32 {
    let mut Next: PADDRINFOW = core::ptr::null_mut::<ADDRINFOW>();
    let mut New: PADDRINFOW = core::ptr::null_mut::<ADDRINFOW>();

    Next = ptResult;
    while !Next.is_null() {
        New = NewAddrInfo(
            (SOCK_DGRAM as i32),
            (*Next).ai_protocol,
            Port,
            (*((*Next).ai_addr as PSOCKADDR_IN)).sin_addr.S_un.S_addr,
        );
        if New.is_null() {
            break;
        }
        (*New).ai_next = (*Next).ai_next;
        (*Next).ai_next = New;
        Next = (*New).ai_next;
    }

    if !Next.is_null() {
        return EAI_MEMORY;
    }

    return 0;
}
unsafe extern "system" fn QueryDNS(
    NodeName: LPCSTR,
    SocketType: i32,
    Protocol: i32,
    Port: u16,
    Alias: *mut i8,
    pptResult: *mut PADDRINFOW,
) -> i32 {
    let mut Next: *mut PADDRINFOW = pptResult;
    let mut Hostent: PHOSTENT = core::ptr::null_mut();
    let mut Addresses: *mut *mut i8;

    *Next = core::ptr::null_mut();
    *Alias.offset(0) = 0;

    Hostent = gethostbyname(NodeName);
    if !Hostent.is_null() {
        if (*Hostent).h_addrtype as i32 == AF_INET as i32
            && (*Hostent).h_length as usize == core::mem::size_of::<IN_ADDR>()
        {
            Addresses = (*Hostent).h_addr_list as *mut *mut i8;
            while !(*Addresses).is_null() {
                *Next = NewAddrInfo(
                    SocketType,
                    Protocol,
                    Port,
                    (*(*Addresses as PIN_ADDR)).S_un.S_addr,
                );
                if (*Next).is_null() {
                    return EAI_MEMORY;
                }
                Next = core::ptr::addr_of_mut!((**Next).ai_next);
                Addresses = Addresses.offset(1);
            }
        }

        strcpy(Alias, (*Hostent).h_name);

        return 0;
    }

    match GetLastError() {
        WSAHOST_NOT_FOUND => EAI_NONAME,
        WSATRY_AGAIN => EAI_AGAIN,
        WSANO_RECOVERY => EAI_FAIL,
        WSANO_DATA => EAI_NODATA,
        _ => EAI_NONAME,
    }
}
unsafe extern "system" fn LookupNodeByAddr(
    pNodeBuffer: LPWSTR,
    NodeBufferSize: DWORD,
    OnlyNodeName: u8,
    Addr: *mut core::ffi::c_void,
    AddrSize: DWORD,
    AddressFamily: i32,
) -> i32 {
    let mut LookupGuid: GUID = GUID {
        Data1: ((0x9 << 16) | 53) as u32,
        Data2: 0,
        Data3: 0xc,
        Data4: [0xc0, 0, 0, 0, 0, 0, 0, 0x46],
    };
    let mut Ip4Addr: PIN_ADDR = Addr as PIN_ADDR;
    let mut ReverseBuffer: [u16; 76] = [0; 76];
    let mut Restrictions: WSAQUERYSETW = core::mem::zeroed();
    let mut Reply: WSAQUERYSETW;
    let mut BufferLength: DWORD;
    let mut ErrorCode: i32;
    let mut LookupHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    if Addr.is_null() {
        return WSAEFAULT as i32;
    }

    if AddressFamily == (AF_INET6 as i32) {
        if AddrSize as usize != core::mem::size_of::<IN6_ADDR>() {
            return WSAEFAULT as i32;
        }
        Ip4Addr = core::ptr::addr_of_mut!((*((Addr as PIN6_ADDR))).u.Byte).offset(12) as PIN_ADDR;
    } else if AddressFamily == (AF_INET as i32) {
        if AddrSize as usize != core::mem::size_of::<IN_ADDR>() {
            return WSAEFAULT as i32;
        }
    } else {
        return WSAEAFNOSUPPORT as i32;
    }

    if ((AddressFamily == (AF_INET6 as i32)) && (IN6_IS_ADDR_V4MAPPED(Addr as *const IN6_ADDR) != 0))
        || (AddressFamily == (AF_INET as i32))
    {
        Dns_Ip4AddressToReverseName_W(ReverseBuffer.as_mut_ptr(), *Ip4Addr);
    }

    RtlZeroMemory(
        core::ptr::addr_of_mut!(Restrictions) as *mut core::ffi::c_void,
        core::mem::size_of::<WSAQUERYSETW>() as usize,
    );
    Restrictions.dwSize = core::mem::size_of::<WSAQUERYSETW>() as DWORD;
    Restrictions.lpszServiceInstanceName = ReverseBuffer.as_mut_ptr();
    Restrictions.lpServiceClassId = core::ptr::addr_of_mut!(LookupGuid);
    Restrictions.dwNameSpace = NS_DNS as DWORD;

    ErrorCode = WSALookupServiceBeginW(
        core::ptr::addr_of_mut!(Restrictions),
        LUP_RETURN_NAME as DWORD,
        core::ptr::addr_of_mut!(LookupHandle),
    );

    if ErrorCode == ERROR_SUCCESS as i32 {
        BufferLength = ((NI_MAXHOST - 1) as usize * core::mem::size_of::<u16>()
            + core::mem::size_of::<WSAQUERYSETW>()) as DWORD;
        ErrorCode = WSALookupServiceNextW(
            LookupHandle,
            0,
            core::ptr::addr_of_mut!(BufferLength),
            core::ptr::addr_of_mut!(Restrictions),
        );
        if ErrorCode == ERROR_SUCCESS as i32 {
            Reply = Restrictions;
            if Reply.lpszServiceInstanceName.is_null() {
                ErrorCode = WSAHOST_NOT_FOUND as i32;
            } else {
                if OnlyNodeName != 0 {
                    Dns_SplitHostFromDomainNameW(Reply.lpszServiceInstanceName);
                }
                if wcslen(Reply.lpszServiceInstanceName).wrapping_add(1) > NodeBufferSize as usize
                {
                    ErrorCode = WSAEFAULT as i32;
                } else {
                    wcscpy(pNodeBuffer, Reply.lpszServiceInstanceName);
                }
            }
        }
    } else if ErrorCode == WSASERVICE_NOT_FOUND as i32 {
        ErrorCode = WSAHOST_NOT_FOUND as i32;
    }

    if !LookupHandle.is_null() {
        WSALookupServiceEnd(LookupHandle);
    }

    ErrorCode
}
unsafe extern "system" fn GetServiceNameForPort(
    pServiceBuffer: LPWSTR,
    ServiceBufferSize: DWORD,
    Port: WORD,
    Flags: DWORD,
) -> INT {
    return (ERROR_SUCCESS as i32);
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn LookupAddressForName(
    NodeName: LPCSTR,
    SocketType: INT,
    Protocol: INT,
    Port: WORD,
    bAI_CANONNAME: BOOL,
    pptResult: *mut PADDRINFOW,
) -> INT {
    let mut iError: INT = 0;
    let mut AliasCount: INT = 0;
    let mut szFQDN1: [CHAR; 1025] = [0; 1025];
    let mut szFQDN2: [CHAR; 1025] = [0; 1025];
    let mut Name: PCHAR = szFQDN1.as_mut_ptr();
    let mut Alias: PCHAR = szFQDN2.as_mut_ptr();
    let mut Scratch: PCHAR = core::ptr::null_mut();

    strcpy(Name, NodeName);

    loop {
        iError = QueryDNS(NodeName, SocketType, Protocol, Port, Alias, pptResult);
        if iError != 0 {
            break;
        }

        if !(*pptResult).is_null() {
            break;
        }

        if strlen(Alias) == 0 || strcmp(Name, Alias) == 0 || {
            AliasCount += 1;
            AliasCount == 16
        } {
            iError = EAI_FAIL;
            break;
        }

        Scratch = Name;
        Name = Alias;
        Alias = Scratch;
    }

    if iError == 0 && bAI_CANONNAME != 0 {
        (**pptResult).ai_canonname = HeapAlloc(WsSockHeap, 0, 512) as PWSTR;

        if (**pptResult).ai_canonname.is_null() {
            iError = EAI_MEMORY;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                Alias,
                -1,
                (**pptResult).ai_canonname,
                256,
            );
        }
    }

    iError
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn LookupAddressForName(
    NodeName: LPCSTR,
    SocketType: INT,
    Protocol: INT,
    Port: WORD,
    bAI_CANONNAME: BOOL,
    pptResult: *mut PADDRINFOW,
) -> INT {
    let mut iError: INT = 0;
    let mut AliasCount: INT = 0;
    let mut szFQDN1: [CHAR; 1025] = [0; 1025];
    let mut szFQDN2: [CHAR; 1025] = [0; 1025];
    let mut Name: PCHAR = szFQDN1.as_mut_ptr();
    let mut Alias: PCHAR = szFQDN2.as_mut_ptr();
    let mut Scratch: PCHAR = core::ptr::null_mut();

    strcpy(Name, (NodeName as *mut i8));

    loop {
        iError = QueryDNS(NodeName, SocketType, Protocol, Port, Alias, pptResult);
        if iError != 0 {
            break;
        }

        if !(*pptResult).is_null() {
            break;
        }

        if strlen(Alias) == 0 || strcmp(Name, Alias) == 0 || {
            AliasCount += 1;
            AliasCount == 16
        } {
            iError = EAI_FAIL;
            break;
        }

        Scratch = Name;
        Name = Alias;
        Alias = Scratch;
    }

    if iError == 0 && bAI_CANONNAME != 0 {
        (**pptResult).ai_canonname = HeapAlloc(WsSockHeap, 0, 512) as PWSTR;

        if (**pptResult).ai_canonname.is_null() {
            iError = EAI_MEMORY;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                Alias,
                -1,
                (**pptResult).ai_canonname,
                256,
            );
        }
    }

    iError
}

#[no_mangle]
pub unsafe extern "system" fn GetAddrInfoW(
    pszNodeName: PCWSTR,
    pszServiceName: PCWSTR,
    ptHints: *const ADDRINFOW,
    pptResult: *mut PADDRINFOW,
) -> i32 {
    let mut iError: i32 = 0;
    let mut iFlags: i32 = 0;
    let mut iFamily: i32 = (PF_UNSPEC as i32);
    let mut iSocketType: i32 = 0;
    let mut iProtocol: i32 = 0;
    let mut wPort: u16 = 0;
    let mut dwAddress: u32 = 0;
    let mut ptService: PSERVENT = core::ptr::null_mut();
    let mut pc: LPSTR = core::ptr::null_mut();
    let mut bClone: i32 = 0;
    let mut wTcpPort: u16 = 0;
    let mut wUdpPort: u16 = 0;
    let mut CanonicalName: [u16; 66] = [0; 66];
    let mut AnsiServiceName: [i8; 256] = [0; 256];
    let mut AnsiNodeName: [i8; 256] = [0; 256];

    *pptResult = core::ptr::null_mut();

    if pszNodeName.is_null() && pszServiceName.is_null() {
        SetLastError(EAI_NONAME as u32);
        return EAI_NONAME;
    }

    if !ptHints.is_null() {
        if (*ptHints).ai_addrlen != 0
            || !(*ptHints).ai_canonname.is_null()
            || !(*ptHints).ai_addr.is_null()
            || !(*ptHints).ai_next.is_null()
        {
            SetLastError(EAI_FAIL as u32);
            return EAI_FAIL;
        }

        iFlags = (*ptHints).ai_flags;
        if (iFlags & (AI_CANONNAME as i32)) != 0 && pszNodeName.is_null() {
            SetLastError(EAI_BADFLAGS as u32);
            return (EAI_BADFLAGS as i32);
        }

        iFamily = (*ptHints).ai_family;
        if iFamily != (PF_UNSPEC as i32) && iFamily != (PF_INET as i32) {
            SetLastError(EAI_FAMILY as u32);
            return (EAI_FAMILY as i32);
        }

        iSocketType = (*ptHints).ai_socktype;
        if iSocketType != 0
            && iSocketType != (SOCK_STREAM as i32)
            && iSocketType != (SOCK_DGRAM as i32)
            && iSocketType != (SOCK_RAW as i32)
        {
            SetLastError(EAI_SOCKTYPE as u32);
            return (EAI_SOCKTYPE as i32);
        }

        iProtocol = (*ptHints).ai_protocol;
    }

    if !pszServiceName.is_null() {
        WideCharToMultiByte(
            CP_ACP as u32,
            0,
            (pszServiceName as *mut u16),
            -1,
            AnsiServiceName.as_mut_ptr(),
            core::mem::size_of::<[i8; 256]>() as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );

        wPort = strtoul(AnsiServiceName.as_mut_ptr(), core::ptr::addr_of_mut!(pc), 10) as u16;

        if *pc == 0 {
            wPort = htons(wPort);
            wTcpPort = wPort;
            wUdpPort = wPort;
        } else {
            wPort = 0;
            if iSocketType == 0 || iSocketType == (SOCK_DGRAM as i32) {
                ptService = getservbyname(
                    AnsiServiceName.as_mut_ptr(),
                    b"udp\0".as_ptr() as *const i8,
                );
                if !ptService.is_null() {
                    wPort = (*ptService).s_port as u16;
                    wUdpPort = wPort;
                }
            }

            if iSocketType == 0 || iSocketType == (SOCK_STREAM as i32) {
                ptService = getservbyname(
                    AnsiServiceName.as_mut_ptr(),
                    b"tcp\0".as_ptr() as *const i8,
                );
                if !ptService.is_null() {
                    wPort = (*ptService).s_port as u16;
                    wTcpPort = wPort;
                }
            }

            if wPort == 0 {
                SetLastError(EAI_SERVICE as u32);
                return (EAI_SERVICE as i32);
            }

            if iSocketType == 0 {
                if wTcpPort != 0 && wUdpPort == 0 {
                    iSocketType = (SOCK_STREAM as i32);
                }
                if wTcpPort == 0 && wUdpPort != 0 {
                    iSocketType = (SOCK_DGRAM as i32);
                }
            }
        }
    }

    if pszNodeName.is_null() || ParseV4Address(pszNodeName, core::ptr::addr_of_mut!(dwAddress)) != 0 {
        if pszNodeName.is_null() {
            dwAddress = htonl(if (iFlags & (AI_PASSIVE as i32)) != 0 {
                INADDR_ANY
            } else {
                INADDR_LOOPBACK as u32
            });
        }

        *pptResult = NewAddrInfo(iSocketType, iProtocol, wPort, dwAddress);

        if (*pptResult).is_null() {
            iError = EAI_MEMORY;
        }

        if iError == 0 && !pszNodeName.is_null() {
            (**pptResult).ai_flags |= (AI_NUMERICHOST as i32);

            if (iFlags & (AI_CANONNAME as i32)) != 0 {
                GetNameInfoW(
                    (**pptResult).ai_addr,
                    (**pptResult).ai_addrlen as socklen_t,
                    CanonicalName.as_mut_ptr(),
                    0x41,
                    core::ptr::null_mut(),
                    0,
                    2,
                );

                (**pptResult).ai_canonname = HeapAlloc(
                    WsSockHeap,
                    0,
                    ((wcslen(CanonicalName.as_mut_ptr()) as usize as u64)) as SIZE_T,
                ) as *mut u16;

                if (**pptResult).ai_canonname.is_null() {
                    iError = EAI_MEMORY;
                } else {
                    core::ptr::copy_nonoverlapping(
                        CanonicalName.as_ptr(),
                        (**pptResult).ai_canonname,
                        wcslen(CanonicalName.as_mut_ptr()) as usize,
                    );
                }
            }
        }
    } else if (iFlags & (AI_NUMERICHOST as i32)) != 0 {
        iError = EAI_NONAME;
    } else {
        WideCharToMultiByte(
            CP_ACP as u32,
            0,
            (pszNodeName as *mut u16),
            -1,
            AnsiNodeName.as_mut_ptr(),
            core::mem::size_of::<[i8; 256]>() as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );

        iError = LookupAddressForName(
            AnsiNodeName.as_mut_ptr() as LPCSTR,
            iSocketType,
            iProtocol,
            wPort,
            iFlags & (AI_CANONNAME as i32),
            pptResult,
        );
    }

    if iError == 0 && bClone != 0 {
        iError = CloneAddrInfo(wUdpPort, *pptResult);
    }

    if iError != 0 {
        FreeAddrInfoW!(*pptResult);
        *pptResult = core::ptr::null_mut();
    }

    SetLastError(iError as u32);
    iError
}

#[no_mangle]
pub unsafe extern "system" fn freeaddrinfo(mut AddrInfo: PADDRINFOA) {
    let mut NextInfo: PADDRINFOA;

    NextInfo = AddrInfo;
    while !NextInfo.is_null() {
        if !(*NextInfo).ai_canonname.is_null() {
            HeapFree(WsSockHeap, 0, (*NextInfo).ai_canonname as PVOID);
        }

        if !(*NextInfo).ai_addr.is_null() {
            HeapFree(WsSockHeap, 0, (*NextInfo).ai_addr as PVOID);
        }

        AddrInfo = (*NextInfo).ai_next as PADDRINFOA;

        HeapFree(WsSockHeap, 0, NextInfo as PVOID);

        NextInfo = AddrInfo;
    }
}

#[no_mangle]
pub unsafe extern "system" fn getaddrinfo(
    nodename: *const i8,
    servname: *const i8,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> i32 {
    let mut ErrorCode: i32;
    let mut UnicodeNodeName: LPWSTR = core::ptr::null_mut();
    let mut UnicodeServName: LPWSTR = core::ptr::null_mut();

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode != (ERROR_SUCCESS as i32) {
        return ErrorCode;
    }

    if !nodename.is_null() {
        UnicodeNodeName = UnicodeDupFromAnsi(nodename as LPCSTR);
        if UnicodeNodeName.is_null() {
            ErrorCode = GetLastError() as i32;
            // goto Quickie
            if !UnicodeNodeName.is_null() {
                HeapFree(WsSockHeap, 0, UnicodeNodeName as *mut core::ffi::c_void);
            }
            if !UnicodeServName.is_null() {
                HeapFree(WsSockHeap, 0, UnicodeServName as *mut core::ffi::c_void);
            }
            if ErrorCode != (ERROR_SUCCESS as i32) {
                freeaddrinfo(*res);
                *res = core::ptr::null_mut();
            }
            SetLastError(ErrorCode as u32);
            return ErrorCode;
        }
    }

    if !servname.is_null() {
        UnicodeServName = UnicodeDupFromAnsi(servname as LPCSTR);
        if UnicodeServName.is_null() {
            ErrorCode = GetLastError() as i32;
            // goto Quickie
            if !UnicodeNodeName.is_null() {
                HeapFree(WsSockHeap, 0, UnicodeNodeName as *mut core::ffi::c_void);
            }
            if !UnicodeServName.is_null() {
                HeapFree(WsSockHeap, 0, UnicodeServName as *mut core::ffi::c_void);
            }
            if ErrorCode != (ERROR_SUCCESS as i32) {
                freeaddrinfo(*res);
                *res = core::ptr::null_mut();
            }
            SetLastError(ErrorCode as u32);
            return ErrorCode;
        }
    }

    ErrorCode = GetAddrInfoW(
        UnicodeNodeName as PCWSTR,
        UnicodeServName as PCWSTR,
        hints as PADDRINFOW,
        res as *mut PADDRINFOW,
    );

    if ErrorCode == (ERROR_SUCCESS as i32) {
        ConvertAddrinfoFromUnicodeToAnsi(*res as PADDRINFOW);
    }

    if !UnicodeNodeName.is_null() {
        HeapFree(WsSockHeap, 0, UnicodeNodeName as *mut core::ffi::c_void);
    }
    if !UnicodeServName.is_null() {
        HeapFree(WsSockHeap, 0, UnicodeServName as *mut core::ffi::c_void);
    }

    if ErrorCode != (ERROR_SUCCESS as i32) {
        freeaddrinfo(*res);
        *res = core::ptr::null_mut();
    }

    SetLastError(ErrorCode as u32);
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn GetNameInfoW(
    pSockaddr: *const SOCKADDR,
    SockaddrLength: socklen_t,
    pNodeBuffer: PWCHAR,
    NodeBufferSize: DWORD,
    pServiceBuffer: PWCHAR,
    ServiceBufferSize: DWORD,
    Flags: INT,
) -> INT {
    let mut AddressLength: DWORD;
    let mut AddrSize: DWORD;
    let mut Addr: *mut core::ffi::c_void;
    let mut Address: SOCKADDR_IN6 = core::mem::zeroed();
    let mut ErrorCode: INT = ERROR_SUCCESS as INT;

    if pSockaddr.is_null() {
        return WSAEFAULT as INT;
    }

    if (*pSockaddr).sa_family as i32 == AF_INET as i32 {
        AddressLength = core::mem::size_of::<SOCKADDR_IN>() as DWORD;
        Addr = core::ptr::addr_of_mut!((*(pSockaddr as PSOCKADDR_IN)).sin_addr) as *mut core::ffi::c_void;
        AddrSize = core::mem::size_of::<IN_ADDR>() as DWORD;
    } else if (*pSockaddr).sa_family as i32 == AF_INET6 as i32 {
        AddressLength = core::mem::size_of::<SOCKADDR_IN6>() as DWORD;
        Addr = core::ptr::addr_of_mut!((*(pSockaddr as *mut SOCKADDR_IN6)).sin6_addr) as *mut core::ffi::c_void;
        AddrSize = core::mem::size_of::<IN6_ADDR>() as DWORD;
    } else {
        SetLastError(EAI_FAMILY as DWORD);
        return EAI_FAMILY as INT;
    }

    if (SockaddrLength as DWORD) < AddressLength {
        return WSAEFAULT as INT;
    }

    'quickie: {
        if !pNodeBuffer.is_null() {
            if (Flags as i32 & NI_NUMERICHOST as i32) == 0 {
                ErrorCode = LookupNodeByAddr(
                    pNodeBuffer,
                    NodeBufferSize,
                    (Flags as i32 & NI_NOFQDN as i32) as u8,
                    Addr,
                    AddrSize,
                    (*pSockaddr).sa_family as i32,
                );
                if ErrorCode != ERROR_SUCCESS as INT {
                    if (Flags as i32 & NI_NAMEREQD as i32) != 0 {
                        break 'quickie;
                    }
                } else {
                    // Skip numeric conversion - jump to SkipNumeric label
                    if !pServiceBuffer.is_null() {
                        ErrorCode = GetServiceNameForPort(
                            pServiceBuffer,
                            ServiceBufferSize,
                            (*(pSockaddr as PSOCKADDR_IN)).sin_port as WORD,
                            Flags as DWORD,
                        );
                    }
                    SetLastError(ErrorCode as DWORD);
                    return ErrorCode;
                }
            }

            core::ptr::copy_nonoverlapping(
                pSockaddr as *const u8,
                core::ptr::addr_of_mut!(Address) as *mut u8,
                AddressLength as usize,
            );

            if (*pSockaddr).sa_family as i32 == AF_INET as i32 {
                (*(core::ptr::addr_of_mut!(Address) as PSOCKADDR_IN)).sin_port = 0;
            } else if (*pSockaddr).sa_family as i32 == AF_INET6 as i32 {
                (*(core::ptr::addr_of_mut!(Address) as *mut SOCKADDR_IN6)).sin6_port = 0;
            }

            let mut NodeBufferSizeMut = NodeBufferSize;
            ErrorCode = WSAAddressToStringW(
                core::ptr::addr_of_mut!(Address) as LPSOCKADDR,
                AddressLength,
                core::ptr::null_mut(),
                pNodeBuffer,
                core::ptr::addr_of_mut!(NodeBufferSizeMut),
            );
            if ErrorCode == SOCKET_ERROR as INT {
                ErrorCode = GetLastError() as INT;
                break 'quickie;
            }
        }

        if !pServiceBuffer.is_null() {
            ErrorCode = GetServiceNameForPort(
                pServiceBuffer,
                ServiceBufferSize,
                (*(pSockaddr as PSOCKADDR_IN)).sin_port as WORD,
                Flags as DWORD,
            );
        }
    }

    SetLastError(ErrorCode as DWORD);
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn getnameinfo(
    sa: *const sockaddr,
    salen: socklen_t,
    host: *mut i8,
    hostlen: DWORD,
    serv: *mut i8,
    servlen: DWORD,
    flags: INT,
) -> INT {
    let mut ErrorCode: INT;
    let mut Buffer: [WCHAR; 256] = [0; 256];
    let mut ServiceBuffer: [WCHAR; 17] = [0; 17];
    let mut HostLength: DWORD = 0;
    let mut ServLength: DWORD = 0;
    let mut ServiceString: PWCHAR = core::ptr::null_mut();
    let mut HostString: PWCHAR = core::ptr::null_mut();

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode != (ERROR_SUCCESS as i32) {
        return ErrorCode;
    }

    if !host.is_null() {
        HostString = Buffer.as_mut_ptr();
        HostLength = (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as DWORD;
    }

    if !serv.is_null() {
        ServiceString = ServiceBuffer.as_mut_ptr();
        ServLength = (core::mem::size_of::<[WCHAR; 17]>() / core::mem::size_of::<WCHAR>()) as DWORD;
    }

    ErrorCode = GetNameInfoW(
        sa as *const SOCKADDR,
        salen,
        HostString,
        HostLength,
        ServiceString,
        ServLength,
        flags,
    );

    if ErrorCode == (ERROR_SUCCESS as i32) {
        if !HostString.is_null() {
            ErrorCode = WideCharToMultiByte(
                CP_ACP,
                0,
                HostString,
                -1,
                host,
                hostlen as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            if ErrorCode == 0 {
                SetLastError(ErrorCode as DWORD);
                return ErrorCode;
            }
        }

        if !ServiceString.is_null() {
            ErrorCode = WideCharToMultiByte(
                CP_ACP,
                0,
                ServiceString,
                -1,
                serv,
                servlen as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            if ErrorCode == 0 {
                SetLastError(ErrorCode as DWORD);
                return ErrorCode;
            }
        }

        return (ERROR_SUCCESS as i32);
    }

    SetLastError(ErrorCode as DWORD);
    return ErrorCode;
}

type INT = i32;
type WCHAR = u16;
type PWCHAR = *mut WCHAR;
type LPCWSTR = *const WCHAR;
type LPBOOL = *mut i32;
type UINT = u32;
type CHAR = i8;
type LPCSTR = *const CHAR;
type socklen_t = i32;

const CP_ACP: UINT = 0;
// enforce_import_abi: moved 7 import decl(s) into the extern block their
// exporting DLL's .spec requires: GetLastError->system, SetLastError->system, WSAAddressToStringW->system, WSALookupServiceBeginW->system, WSALookupServiceEnd->system, WSALookupServiceNextW->system, WideCharToMultiByte->system.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/addrinfo.c
//  * PURPOSE:     Protocol-Independent Address Resolution
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// #include <ws2tcpip.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DEFINES *******************************************************************/
//
// #define Swap(a, b, c)       { (c) = (a); (a) = (b); (b) = (c); }
// #define FreeAddrInfoW(a)    freeaddrinfo((LPADDRINFO)a)
//
// /* FUNCTIONS *****************************************************************/
//
// /* FIXME: put into dnslib */
// VOID
// __stdcall
// Dns_Ip4AddressToReverseName_W(IN LPWSTR AddressBuffer,
//                               IN IN_ADDR Address)
// {
//     /* Convert the address into IPv4 format */
//     wsprintfW(AddressBuffer, L"%u.%u.%u.%u.in-addr.arpa.",
//               Address.S_un.S_un_b.s_b4,
//               Address.S_un.S_un_b.s_b3,
//               Address.S_un.S_un_b.s_b2,
//               Address.S_un.S_un_b.s_b1);
// }
//
// VOID
// WINAPI
// Dns_SplitHostFromDomainNameW(IN LPWSTR DomainName)
// {
//     /* FIXME */
// }
//
// static
// INT
// WINAPI
// ConvertAddrinfoFromUnicodeToAnsi(IN PADDRINFOW Addrinfo)
// {
//     LPSTR AnsiName;
//     LPWSTR *UnicodeName;
//
//     /* Make sure we have a valid pointer */
//     if (Addrinfo)
//     {
//         do
//         {
//             /* Get the name */
//             UnicodeName = &Addrinfo->ai_canonname;
//
//             /* Check if it exists */
//             if (*UnicodeName)
//             {
//                 /* Convert it */
//                 AnsiName = AnsiDupFromUnicode(*UnicodeName);
//                 if (AnsiName)
//                 {
//                     /* Free the old one */
//                     HeapFree(WsSockHeap, 0, *UnicodeName);
//
//                     /* Set the new one */
//                     *UnicodeName = (LPWSTR)AnsiName;
//                 }
//                 else
//                 {
//                     return GetLastError();
//                 }
//             }
//         } while ((Addrinfo = Addrinfo->ai_next));
//     }
//
//     /* All done */
//     return ERROR_SUCCESS;
// }
//
// static
// BOOL
// WINAPI
// ParseV4Address(IN PCWSTR AddressString,
//                OUT PDWORD pAddress)
// {
//     CHAR AnsiAddressString[MAX_HOSTNAME_LEN];
//     CHAR * cp = AnsiAddressString;
//     DWORD val, base;
//     unsigned char c;
//     DWORD parts[4], *pp = parts;
//     if (!AddressString)
//         return FALSE;
//     WideCharToMultiByte(CP_ACP,
//                         0,
//                         AddressString,
//                         -1,
//                         AnsiAddressString,
//                         sizeof(AnsiAddressString),
//                         NULL,
//                         0);
//     if (!isdigit(*cp)) return FALSE;
//
// again:
//     /*
//     * Collect number up to ``.''.
//     * Values are specified as for C:
//     * 0x=hex, 0=octal, other=decimal.
//     */
//     val = 0; base = 10;
//     if (*cp == '0') {
//         if (*++cp == 'x' || *cp == 'X')
//             base = 16, cp++;
//         else
//             base = 8;
//     }
//     while ((c = *cp)) {
//         if (isdigit(c)) {
//             val = (val * base) + (c - '0');
//             cp++;
//             continue;
//         }
//         if (base == 16 && isxdigit(c)) {
//             val = (val << 4) + (c + 10 - (islower(c) ? 'a' : 'A'));
//             cp++;
//             continue;
//         }
//         break;
//     }
//     if (*cp == '.') {
//         /*
//         * Internet format:
//         *    a.b.c.d
//         */
//         if (pp >= parts + 4) return FALSE;
//         *pp++ = val;
//         cp++;
//         goto again;
//     }
//     /*
//     * Check for trailing characters.
//     */
//     if (*cp) return FALSE;
//
//     if (pp >= parts + 4) return FALSE;
//     *pp++ = val;
//     /*
//     * Concoct the address according to
//     * the number of parts specified.
//     */
//     if ((DWORD)(pp - parts) != 4) return FALSE;
//     if (parts[0] > 0xff || parts[1] > 0xff || parts[2] > 0xff || parts[3] > 0xff) return FALSE;
//     val = (parts[0] << 24) | (parts[1] << 16) | (parts[2] << 8) | parts[3];
//
//     if (pAddress)
//         *pAddress = htonl(val);
//
//     return TRUE;
// }
//
// static
// PADDRINFOW
// WINAPI
// NewAddrInfo(IN INT SocketType,
//             IN INT Protocol,
//             IN WORD Port,
//             IN DWORD Address)
// {
//     PADDRINFOW AddrInfo;
//     PSOCKADDR_IN SockAddress;
//
//     /* Allocate a structure */
//     AddrInfo = HeapAlloc(WsSockHeap, 0, sizeof(ADDRINFOW));
//     if (!AddrInfo) return NULL;
//
//     /* Allocate a sockaddr */
//     SockAddress = HeapAlloc(WsSockHeap, 0, sizeof(SOCKADDR_IN));
//     if (!SockAddress)
//     {
//         /* Free the addrinfo and fail */
//         HeapFree(WsSockHeap, 0, AddrInfo);
//         return NULL;
//     }
//
//     /* Write data for socket address */
//     SockAddress->sin_family = AF_INET;
//     SockAddress->sin_port = Port;
//     SockAddress->sin_addr.s_addr = Address;
//     ZeroMemory(SockAddress->sin_zero, sizeof(SockAddress->sin_zero));
//
//     /* Fill out the addrinfo */
//     AddrInfo->ai_family = PF_INET;
//     AddrInfo->ai_socktype = SocketType;
//     AddrInfo->ai_protocol = Protocol;
//     AddrInfo->ai_flags = 0;
//     AddrInfo->ai_next = 0;
//     AddrInfo->ai_canonname = NULL;
//     AddrInfo->ai_addrlen = sizeof(SOCKADDR_IN);
//     AddrInfo->ai_addr = (PSOCKADDR)SockAddress;
//
//     /* Return it */
//     return AddrInfo;
// }
//
// static
// INT
// WINAPI
// CloneAddrInfo(IN WORD Port,
//               IN PADDRINFOW ptResult)
// {
//     PADDRINFOW Next = NULL;
//     PADDRINFOW New  = NULL;
//
//     /* Loop the chain */
//     for (Next = ptResult; Next;)
//     {
//         /* Create a new structure */
//         New = NewAddrInfo(SOCK_DGRAM,
//                           Next->ai_protocol,
//                           Port,
//                           ((PSOCKADDR_IN)Next->ai_addr)->sin_addr.s_addr);
//         if (!New) break;
//
//         /* Link them */
//         New->ai_next = Next->ai_next;
//         Next->ai_next = New;
//         Next = New->ai_next;
//     }
//
//     /* Check if we ran out of memory */
//     if (Next) return EAI_MEMORY;
//
//     /* Return success */
//     return 0;
// }
//
// static
// INT
// WINAPI
// QueryDNS(IN LPCSTR NodeName,
//          IN INT SocketType,
//          IN INT Protocol,
//          IN WORD Port,
//          OUT CHAR Alias[NI_MAXHOST],
//          OUT PADDRINFOW *pptResult)
// {
//     PADDRINFOW *Next = pptResult;
//     PHOSTENT Hostent = NULL;
//     PCHAR *Addresses;
//
//     /* Assume nothing found */
//     *Next = NULL;
//     Alias[0] = '\0';
//
//     /* Get the hostent */
//     Hostent = gethostbyname(NodeName);
//     if (Hostent)
//     {
//         /* Check for valid addresses */
//         if ((Hostent->h_addrtype == AF_INET) &&
//             (Hostent->h_length == sizeof(IN_ADDR)))
//         {
//             /* Loop every address in it */
//             for (Addresses = Hostent->h_addr_list; *Addresses; Addresses++)
//             {
//                 /* Create an addrinfo structure for it*/
//                 *Next = NewAddrInfo(SocketType,
//                                     Protocol,
//                                     Port,
//                                     ((PIN_ADDR)*Addresses)->s_addr);
//                 if (!*Next) return EAI_MEMORY;
//
//                 /* Move to the next entry */
//                 Next = &((*Next)->ai_next);
//             }
//         }
//
//         /* Copy the canonical name */
//         strcpy(Alias, Hostent->h_name);
//
//         /* Return success */
//         return 0;
//     }
//
//     /* Find out what the error was */
//     switch (GetLastError())
//     {
//         /* Convert the Winsock Error to an EAI error */
//         case WSAHOST_NOT_FOUND: return EAI_NONAME;
//         case WSATRY_AGAIN: return EAI_AGAIN;
//         case WSANO_RECOVERY: return EAI_FAIL;
//         case WSANO_DATA: return EAI_NODATA;
//         default: return EAI_NONAME;
//     }
// }
//
// static
// INT
// WINAPI
// LookupNodeByAddr(IN LPWSTR pNodeBuffer,
//                  IN DWORD NodeBufferSize,
//                  IN BOOLEAN OnlyNodeName,
//                  IN PVOID Addr,
//                  IN DWORD AddrSize,
//                  IN INT AddressFamily)
// {
//     GUID LookupGuid = SVCID_DNS_TYPE_PTR;
//     PIN_ADDR Ip4Addr = Addr;
//     WCHAR ReverseBuffer[76];
//     WSAQUERYSETW Restrictions, Reply;
//     DWORD BufferLength;
//     INT ErrorCode;
//     HANDLE LookupHandle;
//
//     /* Validate the address */
//     if (!Addr) return WSAEFAULT;
//
//     /* Make sure the family and address size match */
//     if (AddressFamily == AF_INET6)
//     {
//         /* Check the address size for this type */
//         if (AddrSize != sizeof(IN6_ADDR)) return WSAEFAULT;
//         Ip4Addr = (PIN_ADDR)&((PIN6_ADDR)Addr)->u.Byte[12];
//     }
//     else if (AddressFamily == AF_INET)
//     {
//         /* Check the address size for this type */
//         if (AddrSize != sizeof(IN_ADDR)) return WSAEFAULT;
//     }
//     else
//     {
//         /* Address family not supported */
//         return WSAEAFNOSUPPORT;
//     }
//
//     /* Check if this is a mapped V4 IPv6 or pure IPv4 */
//     if (((AddressFamily == AF_INET6) && (IN6_IS_ADDR_V4MAPPED(Addr))) ||
//         (AddressFamily == AF_INET))
//     {
//         /* Get the reverse name */
//         Dns_Ip4AddressToReverseName_W(ReverseBuffer, *Ip4Addr);
//     }
//     /* FIXME: Not implemented for now
//     else if ( */
//
//     /* By this point we have the Reverse Name, so prepare for lookup */
//     RtlZeroMemory(&Restrictions, sizeof(Restrictions));
//     Restrictions.dwSize = sizeof(Restrictions);
//     Restrictions.lpszServiceInstanceName = ReverseBuffer;
//     Restrictions.lpServiceClassId = &LookupGuid;
//     Restrictions.dwNameSpace = NS_DNS;
//
//     /* Now do the lookup */
//     ErrorCode = WSALookupServiceBeginW(&Restrictions,
//                                        LUP_RETURN_NAME,
//                                        &LookupHandle);
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Lookup successful, now get the data */
//         BufferLength = (NI_MAXHOST - 1) * sizeof(WCHAR) + sizeof(Restrictions);
//         ErrorCode = WSALookupServiceNextW(LookupHandle,
//                                           0,
//                                           &BufferLength,
//                                           &Restrictions);
//         if (ErrorCode == ERROR_SUCCESS)
//         {
//             /* Now check if we have a name back */
//             Reply = Restrictions;
//             if (!Reply.lpszServiceInstanceName)
//             {
//                 /* Fail */
//                 ErrorCode = WSAHOST_NOT_FOUND;
//             }
//             else
//             {
//                 /* Check if the caller only wants the node name */
//                 if (OnlyNodeName)
//                 {
//                     /* Split it and get only the partial host name */
//                     Dns_SplitHostFromDomainNameW(Reply.lpszServiceInstanceName);
//                 }
//
//                 /* Check the length and see if it's within our limit */
//                 if (wcslen(Reply.lpszServiceInstanceName) + 1 >
//                     NodeBufferSize)
//                 {
//                     /* It's not, so fail */
//                     ErrorCode = WSAEFAULT;
//                 }
//                 else
//                 {
//                     /* It will fit, let's copy it*/
//                     wcscpy(pNodeBuffer, Reply.lpszServiceInstanceName);
//                 }
//             }
//         }
//     }
//     else if (ErrorCode == WSASERVICE_NOT_FOUND)
//     {
//         /* Normalize the error code */
//         ErrorCode = WSAHOST_NOT_FOUND;
//     }
//
//     /* Finish the lookup if one was in progress */
//     if (LookupHandle) WSALookupServiceEnd(LookupHandle);
//
//     /* Return the error code */
//     return ErrorCode;
// }
//
// static
// INT
// WINAPI
// GetServiceNameForPort(IN LPWSTR pServiceBuffer,
//                       IN DWORD ServiceBufferSize,
//                       IN WORD Port,
//                       IN DWORD Flags)
// {
//     return ERROR_SUCCESS;
// }
//
// static
// INT
// WINAPI
// LookupAddressForName(IN LPCSTR NodeName,
//                      IN INT SocketType,
//                      IN INT Protocol,
//                      IN WORD Port,
//                      IN BOOL bAI_CANONNAME,
//                      OUT PADDRINFOW *pptResult)
// {
//     INT iError = 0;
//     INT AliasCount = 0;
//     CHAR szFQDN1[NI_MAXHOST] = "";
//     CHAR szFQDN2[NI_MAXHOST] = "";
//     PCHAR Name = szFQDN1;
//     PCHAR Alias = szFQDN2;
//     PCHAR Scratch = NULL;
//
//     /* Make a copy of the name */
//     strcpy(Name, NodeName);
//
//     /* Loop */
//     while (TRUE)
//     {
//         /* Do a DNS Query for the name */
//         iError = QueryDNS(NodeName,
//                           SocketType,
//                           Protocol,
//                           Port,
//                           Alias,
//                           pptResult);
//         if (iError) break;
//
//         /* Exit if we have a result */
//         if (*pptResult) break;
//
//         /* Don't loop continuously if this is a DNS misconfiguration */
//         if ((!strlen(Alias)) || (!strcmp(Name, Alias)) || (++AliasCount == 16))
//         {
//             /* Exit the loop with a failure */
//             iError = EAI_FAIL;
//             break;
//         }
//
//         /* Restart loopup if we got a CNAME */
//         Swap(Name, Alias, Scratch);
//     }
//
//     /* Check if we succeeded and the canonical name is requested */
//     if (!iError && bAI_CANONNAME)
//     {
//         /* Allocate memory for a copy */
//         (*pptResult)->ai_canonname = HeapAlloc(WsSockHeap, 0, 512);
//
//         /* Check if we had enough memory */
//         if (!(*pptResult)->ai_canonname)
//         {
//             /* Set failure code */
//             iError = EAI_MEMORY;
//         }
//         else
//         {
//             /* Convert the alias to UNICODE */
//             MultiByteToWideChar(CP_ACP,
//                                 0,
//                                 Alias,
//                                 -1,
//                                 (*pptResult)->ai_canonname,
//                                 256);
//         }
//     }
//
//     /* Return to caller */
//     return iError;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// GetAddrInfoW(IN PCWSTR pszNodeName,
//              IN PCWSTR pszServiceName,
//              IN const ADDRINFOW *ptHints,
//              OUT PADDRINFOW *pptResult)
// {
//     INT iError = 0;
//     INT iFlags = 0;
//     INT iFamily = PF_UNSPEC;
//     INT iSocketType = 0;
//     INT iProtocol = 0;
//     WORD wPort = 0;
//     DWORD dwAddress = 0;
//     PSERVENT ptService = NULL;
//     PCHAR pc = NULL;
//     BOOL bClone = FALSE;
//     WORD wTcpPort = 0;
//     WORD wUdpPort = 0;
//     WCHAR CanonicalName[0x42];
//     CHAR AnsiServiceName[256];
//     CHAR AnsiNodeName[256];
//     DPRINT("GetAddrInfoW: %S, %S, %p, %p\n", pszNodeName, pszServiceName, ptHints, pptResult);
//
//     /* Assume error */
//     *pptResult  = NULL;
//
//     /* We must have at least one name to work with */
//     if (!(pszNodeName) && !(pszServiceName))
//     {
//         /* Fail */
//         SetLastError(EAI_NONAME);
//         return EAI_NONAME;
//     }
//
//     /* Check if we got hints */
//     if (ptHints)
//     {
//         /* Make sure these are empty */
//         if ((ptHints->ai_addrlen) ||
//             (ptHints->ai_canonname) ||
//             (ptHints->ai_addr) ||
//             (ptHints->ai_next))
//         {
//             /* Fail if they aren't */
//             SetLastError(EAI_FAIL);
//             return EAI_FAIL;
//         }
//
//         /* Save the flags and validate them */
//         iFlags = ptHints->ai_flags;
//         if ((iFlags & AI_CANONNAME) && !pszNodeName)
//         {
//             SetLastError(EAI_BADFLAGS);
//             return EAI_BADFLAGS;
//         }
//
//         /* Save family and validate it */
//         iFamily = ptHints->ai_family;
//         if ((iFamily != PF_UNSPEC) && (iFamily != PF_INET))
//         {
//             SetLastError(EAI_FAMILY);
//             return EAI_FAMILY;
//         }
//
//         /* Save socket type and validate it */
//         iSocketType = ptHints->ai_socktype;
//         if ((iSocketType != 0) &&
//             (iSocketType != SOCK_STREAM) &&
//             (iSocketType != SOCK_DGRAM) &&
//             (iSocketType != SOCK_RAW))
//         {
//             SetLastError(EAI_SOCKTYPE);
//             return EAI_SOCKTYPE;
//         }
//
//         /* Save the protocol */
//         iProtocol = ptHints->ai_protocol;
//     }
//
//     /* Check if we have a service name */
//     if (pszServiceName)
//     {
//         /* We need to convert it to ANSI */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             pszServiceName,
//                             -1,
//                             AnsiServiceName,
//                             sizeof(AnsiServiceName),
//                             NULL,
//                             0);
//
//         /* Get the port */
//         wPort = (WORD)strtoul(AnsiServiceName, &pc, 10);
//
//         /* Check if the port string is numeric */
//         if (*pc == '\0')
//         {
//             /* Get the port directly */
//             wPort = wTcpPort = wUdpPort = htons(wPort);
//
//         }
//         else
//         {
//             wPort = 0;
//             /* The port name was a string. Check if this is a UDP socket */
//             if ((iSocketType == 0) || (iSocketType == SOCK_DGRAM))
//             {
//                 /* It's UDP, do a getservbyname */
//                 ptService = getservbyname(AnsiServiceName, "udp");
//
//                 /* If we got a servent, return the port from it */
//                 if (ptService) wPort = wUdpPort = ptService->s_port;
//             }
//
//             /* Check if this is a TCP socket */
//             if ((iSocketType == 0) || (iSocketType == SOCK_STREAM))
//             {
//                 /* It's TCP, do a getserbyname */
//                 ptService = getservbyname(AnsiServiceName, "tcp");
//
//                 /* Return the port from the servent */
//                 if (ptService) wPort = wTcpPort = ptService->s_port;
//             }
//
//             /* If we got 0, then fail */
//             if (wPort == 0)
//             {
//                 SetLastError(EAI_SERVICE);
//                 return EAI_SERVICE;
//             }
//
//             /* Check if this was for both */
//             if (iSocketType == 0)
//             {
//                 /* Do the TCP case right now */
//                 if (wTcpPort && !wUdpPort)
//                     iSocketType = SOCK_STREAM;
//                 if (!wTcpPort && wUdpPort)
//                     iSocketType = SOCK_DGRAM;
//                 //bClone = (wTcpPort && wUdpPort);
//             }
//         }
//     }
//
//     /* Check if no node was given or if this is is a valid IPv4 address */
//     if ((!pszNodeName) || (ParseV4Address(pszNodeName, &dwAddress)))
//     {
//         /* Check if we don't have a node name */
//         if (!pszNodeName)
//         {
//             /* Make one up based on the flags */
//             dwAddress = htonl((iFlags & AI_PASSIVE) ?
//                               INADDR_ANY : INADDR_LOOPBACK);
//         }
//
//         /* Create the Addr Info */
//         *pptResult = NewAddrInfo(iSocketType, iProtocol, wPort, dwAddress);
//
//         /* If we didn't get one back, assume out of memory */
//         if (!(*pptResult)) iError = EAI_MEMORY;
//
//         /* Check if we have success and a nodename */
//         if (!iError && pszNodeName)
//         {
//             /* Set AI_NUMERICHOST since this is a numeric string */
//             (*pptResult)->ai_flags |= AI_NUMERICHOST;
//
//             /* Check if the canonical name was requested */
//             if (iFlags & AI_CANONNAME)
//             {
//                 /* Get the canonical name */
//                 GetNameInfoW((*pptResult)->ai_addr,
//                              (socklen_t)(*pptResult)->ai_addrlen,
//                              CanonicalName,
//                              0x41,
//                              NULL,
//                              0,
//                              2);
//
//                 /* Allocate memory for a copy */
//                 (*pptResult)->ai_canonname = HeapAlloc(WsSockHeap,
//                                                        0,
//                                                        wcslen(CanonicalName));
//
//                 if (!(*pptResult)->ai_canonname)
//                 {
//                     /* No memory for the copy */
//                     iError = EAI_MEMORY;
//                 }
//                 else
//                 {
//                     /* Duplicate the string */
//                     RtlMoveMemory((*pptResult)->ai_canonname,
//                                   CanonicalName,
//                                   wcslen(CanonicalName));
//                 }
//             }
//         }
//     }
//     else if (iFlags & AI_NUMERICHOST)
//     {
//         /* No name for this request (we have a non-numeric name) */
//         iError = EAI_NONAME;
//     }
//     else
//     {
//         /* We need to convert it to ANSI */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             pszNodeName,
//                             -1,
//                             AnsiNodeName,
//                             sizeof(AnsiNodeName),
//                             NULL,
//                             0);
//
//         /* Non-numeric name, do DNS lookup */
//         iError = LookupAddressForName(AnsiNodeName,
//                                       iSocketType,
//                                       iProtocol,
//                                       wPort,
//                                       (iFlags & AI_CANONNAME),
//                                       pptResult);
//     }
//
//     /* If all was good and the caller requested UDP and TCP */
//     if (!iError && bClone)
//     {
//         /* Process UDP now, we already did TCP */
//         iError = CloneAddrInfo(wUdpPort, *pptResult);
//     }
//
//     /* If we've hit an error till here */
//     if (iError)
//     {
//         /* Free the address info and return nothing */
//         FreeAddrInfoW!(*pptResult);
//         *pptResult = NULL;
//     }
//
//     /* Return to caller */
//     SetLastError(iError);
//     return iError;
// }
//
// #undef freeaddrinfo
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// freeaddrinfo(PADDRINFOA AddrInfo)
// {
//     PADDRINFOA NextInfo;
//
//     /* Loop the chain of structures */
//     for (NextInfo = AddrInfo; NextInfo; NextInfo = AddrInfo)
//     {
//         /* Check if there is a canonical name */
//         if (NextInfo->ai_canonname)
//         {
//             /* Free it */
//             HeapFree(WsSockHeap, 0, NextInfo->ai_canonname);
//         }
//
//         /* Check if there is an address */
//         if (NextInfo->ai_addr)
//         {
//             /* Free it */
//             HeapFree(WsSockHeap, 0, NextInfo->ai_addr);
//         }
//
//         /* Move to the next entry */
//         AddrInfo = NextInfo->ai_next;
//
//         /* Free this entry */
//         HeapFree(WsSockHeap, 0, NextInfo);
//     }
// }
//
// #undef getaddrinfo
// /*
//  * @implemented
//  */
// INT
// WINAPI
// getaddrinfo(const char  *nodename,
//             const char  *servname,
//             const struct addrinfo  *hints,
//             struct addrinfo  *  *res)
// {
//     INT ErrorCode;
//     LPWSTR UnicodeNodeName = NULL;
//     LPWSTR UnicodeServName = NULL;
//     DPRINT("getaddrinfo: %s, %s, %p, %p\n", nodename, servname, hints, res);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) != ERROR_SUCCESS) return ErrorCode;
//
//     /* Convert the node name */
//     if (nodename)
//     {
//         UnicodeNodeName = UnicodeDupFromAnsi(nodename);
//         if (!UnicodeNodeName)
//         {
//             /* Prepare to fail */
//             ErrorCode = GetLastError();
//             goto Quickie;
//         }
//     }
//
//     /* Convert the servname too, if we have one */
//     if (servname)
//     {
//         UnicodeServName = UnicodeDupFromAnsi(servname);
//         if (!UnicodeServName)
//         {
//             /* Prepare to fail */
//             ErrorCode = GetLastError();
//             goto Quickie;
//         }
//     }
//
//     /* Now call the unicode function */
//     ErrorCode = GetAddrInfoW(UnicodeNodeName,
//                              UnicodeServName,
//                              (PADDRINFOW)hints,
//                              (PADDRINFOW*)res);
//
//     /* Convert it to ANSI if we succeeded */
//     if (ErrorCode == ERROR_SUCCESS) ConvertAddrinfoFromUnicodeToAnsi((PADDRINFOW)*res);
//
// Quickie:
//     /* Check if we have a unicode node name and serv name */
//     if (UnicodeNodeName) HeapFree(WsSockHeap, 0, UnicodeNodeName);
//     if (UnicodeServName) HeapFree(WsSockHeap, 0, UnicodeServName);
//
//     /* Check if we are in error */
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         /* Free the structure and return nothing */
//         freeaddrinfo(*res);
//         *res = NULL;
//     }
//
//     /* Set the last error and return */
//     SetLastError(ErrorCode);
//     return ErrorCode;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// GetNameInfoW(IN CONST SOCKADDR *pSockaddr,
//              IN socklen_t SockaddrLength,
//              OUT PWCHAR pNodeBuffer,
//              IN DWORD NodeBufferSize,
//              OUT PWCHAR pServiceBuffer,
//              IN DWORD ServiceBufferSize,
//              IN INT Flags)
// {
//     DWORD AddressLength, AddrSize;
//     PVOID Addr;
//     SOCKADDR_IN6 Address;
//     INT ErrorCode = ERROR_SUCCESS;
//
//     DPRINT("GetNameInfoW: %p, %d, %p, %ld, %p, %ld, %d\n",
//            pSockaddr,
//            SockaddrLength,
//            pNodeBuffer,
//            NodeBufferSize,
//            pServiceBuffer,
//            ServiceBufferSize,
//            Flags);
//
//     /* Check for valid socket */
//     if (!pSockaddr)
//         return WSAEFAULT;
//
//     /* Check which family this is */
//     if (pSockaddr->sa_family == AF_INET)
//     {
//         /* IPv4 */
//         AddressLength = sizeof(SOCKADDR_IN);
//         Addr = &((PSOCKADDR_IN)pSockaddr)->sin_addr;
//         AddrSize = sizeof(IN_ADDR);
//     }
//     else if (pSockaddr->sa_family == AF_INET6)
//     {
//         /* IPv6 */
//         AddressLength = sizeof(SOCKADDR_IN6);
//         Addr = &((PSOCKADDR_IN6)pSockaddr)->sin6_addr;
//         AddrSize = sizeof(IN6_ADDR);
//     }
//     else
//     {
//         /* Unsupported family */
//         SetLastError(EAI_FAMILY);
//         return EAI_FAMILY;
//     }
//
//     /* Check for valid socket address length */
//     if ((DWORD)SockaddrLength < AddressLength)
//         return WSAEFAULT;
//
//     /* Check if we have a node name */
//     if (pNodeBuffer)
//     {
//         /* Check if only the numeric host is wanted */
//         if (!(Flags & NI_NUMERICHOST))
//         {
//             /* Do the lookup by addr */
//             ErrorCode = LookupNodeByAddr(pNodeBuffer,
//                                          NodeBufferSize,
//                                          Flags & NI_NOFQDN,
//                                          Addr,
//                                          AddrSize,
//                                          pSockaddr->sa_family);
//             /* Check if we failed */
//             if (ErrorCode != ERROR_SUCCESS)
//             {
//                 /* Fail if the caller REALLY wants the NAME itself? */
//                 if (Flags & NI_NAMEREQD) goto quickie;
//             }
//             else
//             {
//                 /* We succeeded, no need to get the numeric address */
//                 goto SkipNumeric;
//             }
//         }
//
//         /* Copy the address */
//         RtlMoveMemory(&Address, pSockaddr, AddressLength);
//
//         /* Get the numeric address */
//         if (pSockaddr->sa_family == AF_INET)
//         {
//             /* IPv4 */
//             ((PSOCKADDR_IN)&Address)->sin_port = 0;
//         }
//         else if (pSockaddr->sa_family == AF_INET6)
//         {
//             /* IPv6 */
//             ((PSOCKADDR_IN6)&Address)->sin6_port = 0;
//         }
//         ErrorCode = WSAAddressToStringW((LPSOCKADDR)&Address,
//                                         AddressLength,
//                                         NULL,
//                                         pNodeBuffer,
//                                         &NodeBufferSize);
//         if (ErrorCode == SOCKET_ERROR)
//         {
//             /* Get the error code and exit with it */
//             ErrorCode = GetLastError();
//             goto quickie;
//         }
//     }
//
// SkipNumeric:
//     /* Check if we got a service name */
//     if (pServiceBuffer)
//     {
//         /* Handle this request */
//         ErrorCode = GetServiceNameForPort(pServiceBuffer,
//                                           ServiceBufferSize,
//                                           ((PSOCKADDR_IN)pSockaddr)->sin_port,
//                                           Flags);
//     }
//
//     /* Set the error and return it (or success) */
// quickie:
//     SetLastError(ErrorCode);
//     return ErrorCode;
// }
//
// #undef getnameinfo
// /*
//  * @implemented
//  */
// INT
// WINAPI
// getnameinfo(const struct sockaddr  *sa,
//             socklen_t salen,
//             char  *host,
//             DWORD hostlen,
//             char  *serv,
//             DWORD servlen,
//             INT flags)
// {
//     INT ErrorCode;
//     WCHAR Buffer[256];
//     WCHAR ServiceBuffer[17];
//     DWORD HostLength = 0, ServLength = 0;
//     PWCHAR ServiceString = NULL, HostString = NULL;
//     DPRINT("getnameinfo: %p, %p, %p, %lx\n", host, serv, sa, salen);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) != ERROR_SUCCESS) return ErrorCode;
//
//     /* Check if we have a host pointer */
//     if (host)
//     {
//         /* Setup the data for it */
//         HostString = Buffer;
//         HostLength = sizeof(Buffer) / sizeof(WCHAR);
//     }
//
//     /* Check if we have a service pointer */
//     if (serv)
//     {
//         /* Setup the data for it */
//         ServiceString = ServiceBuffer;
//         ServLength = sizeof(ServiceBuffer) / sizeof(WCHAR);
//     }
//
//     /* Now call the unicode function */
//     ErrorCode = GetNameInfoW(sa,
//                              salen,
//                              HostString,
//                              HostLength,
//                              ServiceString,
//                              ServLength,
//                              flags);
//
//     /* Check for success */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Check if we had a host pointer */
//         if (HostString)
//         {
//             /* Convert it back to ANSI */
//             ErrorCode = WideCharToMultiByte(CP_ACP,
//                                             0,
//                                             HostString,
//                                             -1,
//                                             host,
//                                             hostlen,
//                                             NULL,
//                                             NULL);
//             if (!ErrorCode) goto Quickie;
//         }
//
//         /* Check if we have a service pointer */
//         if (ServiceString)
//         {
//             /* Convert it back to ANSI */
//             ErrorCode = WideCharToMultiByte(CP_ACP,
//                                             0,
//                                             ServiceString,
//                                             -1,
//                                             serv,
//                                             servlen,
//                                             NULL,
//                                             NULL);
//             if (!ErrorCode) goto Quickie;
//         }
//
//         /* Return success */
//         return ERROR_SUCCESS;
//     }
//
//     /* Set the last error and return */
// Quickie:
//     SetLastError(ErrorCode);
//     return ErrorCode;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

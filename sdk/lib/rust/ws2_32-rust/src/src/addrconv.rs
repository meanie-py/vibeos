use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::startup::WSACleanup;


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

#[cfg(target_endian = "little")]
#[inline]
pub const fn DN2H(dw: u32) -> u32 {
    ((dw & 0xFF000000u32) >> 24) |
    ((dw & 0x00FF0000u32) >> 8) |
    ((dw & 0x0000FF00u32) << 8) |
    ((dw & 0x000000FFu32) << 24)
}

#[cfg(target_endian = "little")]
#[inline]
pub const fn DH2N(dw: u32) -> u32 {
    ((dw & 0xFF000000u32) >> 24) |
    ((dw & 0x00FF0000u32) >> 8) |
    ((dw & 0x0000FF00u32) << 8) |
    ((dw & 0x000000FFu32) << 24)
}

#[cfg(target_endian = "little")]
#[inline]
pub const fn WN2H(w: u16) -> u16 {
    ((w & 0xFF00u16) >> 8) |
    ((w & 0x00FFu16) << 8)
}

#[cfg(target_endian = "little")]
#[inline]
pub const fn WH2N(w: u16) -> u16 {
    ((w & 0xFF00u16) >> 8) |
    ((w & 0x00FFu16) << 8)
}

#[cfg(target_endian = "big")]
#[inline]
pub const fn DN2H(dw: u32) -> u32 {
    dw
}

#[cfg(target_endian = "big")]
#[inline]
pub const fn DH2N(dw: u32) -> u32 {
    dw
}

#[cfg(target_endian = "big")]
#[inline]
pub const fn WN2H(w: u16) -> u16 {
    w
}

#[cfg(target_endian = "big")]
#[inline]
pub const fn WH2N(w: u16) -> u16 {
    w
}

#[no_mangle]
pub unsafe extern "system" fn inet_addr(cp: *const i8) -> u_long {
    let mut val: u_long;
    let mut base: u_long;
    let mut n: u_long;
    let mut c: u8;
    let mut parts: [u_long; 4] = [0; 4];
    let mut pp: *mut u_long = parts.as_mut_ptr();

    if cp.is_null() {
        return 0;
    }
    if isdigit(*cp as i32) == 0 {
        return INADDR_NONE;
    }

    let mut cp = cp;
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
                val = val.wrapping_mul(base).wrapping_add((c as i32 - '0' as i32) as u_long);
                cp = cp.offset(1);
            } else {
                if !(base == 16 && isxdigit(c as i32) != 0) {
                    break;
                }
                val = (val << 4).wrapping_add(
                    (c as i32 + 10 - if islower(c as i32) != 0 { 'a' as i32 } else { 'A' as i32 }) as u_long
                );
                cp = cp.offset(1);
            }
        }
        if *cp as i32 != '.' as i32 {
            break;
        }
        if pp >= parts.as_mut_ptr().offset(4) {
            return INADDR_NONE;
        }
        *pp = val;
        pp = pp.offset(1);
        cp = cp.offset(1);
    }
    if *cp as i32 != 0 && isspace(*cp as i32) == 0 {
        return INADDR_NONE;
    }

    // The in-loop store is bounds-checked; THIS one was not. For an input with five
    // or more dot-separated groups ("1.2.3.4.5") pp is already parts+4 here, so this
    // writes a u_long one element past a 4-element STACK array. The `n` match below
    // then returns INADDR_NONE -- but only after the stack is already corrupted, and
    // inet_addr is exported from ws2_32 and reachable by any Winsock caller.
    // Upstream replaced the whole routine with Wine's; the minimal equivalent here is
    // to reject before storing, which is behaviourally identical (n would be >= 5 and
    // fall to INADDR_NONE anyway) minus the corruption.
    // Upstream-Commit: 6679825f21e
    if pp >= parts.as_mut_ptr().offset(4) {
        return INADDR_NONE;
    }

    *pp = val;
    pp = pp.offset(1);

    n = pp.offset_from(parts.as_mut_ptr()) as u_long;
    match n {
        1 => {
            val = parts[0];
        }
        2 => {
            val = (parts[0] << 24) | (parts[1] & 0xffffff);
        }
        3 => {
            val = (parts[0] << 24) | ((parts[1] & 0xff) << 16) | (parts[2] & 0xffff);
        }
        4 => {
            val = (parts[0] << 24) | ((parts[1] & 0xff) << 16) | ((parts[2] & 0xff) << 8) | (parts[3] & 0xff);
        }
        _ => return INADDR_NONE,
    }
    val = htonl(val);
    val
}

#[no_mangle]
pub unsafe extern "system" fn inet_ntoa(in_0: IN_ADDR) -> PSTR {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: i32;
    let mut WsaData: WSADATA = core::mem::zeroed();
    let mut ManualLoad: i32 = 0;
    let mut b: [i8; 10] = [0; 10];
    let mut p: PSTR;

    ErrorCode = WsApiProlog(
        core::ptr::addr_of_mut!(Process),
        core::ptr::addr_of_mut!(Thread)
    );

    if ErrorCode != (ERROR_SUCCESS as i32) {
        if ErrorCode != (WSANOTINITIALISED as i32) {
            SetLastError(ErrorCode as u32);
            return core::ptr::null_mut();
        }

        ErrorCode = WSAStartup(
            ((2u64 & 0xffu64) as u8 as i32 |
             ((2u64 & 0xffu64) as u8 as u16 as i32) << 8) as u16,
            core::ptr::addr_of_mut!(WsaData)
        ) as i32;

        if ErrorCode != (ERROR_SUCCESS as i32) {
            SetLastError(ErrorCode as u32);
            return core::ptr::null_mut();
        }

        ManualLoad = 1;
        ErrorCode = WsApiProlog(
            core::ptr::addr_of_mut!(Process),
            core::ptr::addr_of_mut!(Thread)
        );

        if ErrorCode != (ERROR_SUCCESS as i32) {
            WSACleanup();
            SetLastError(ErrorCode as u32);
            return core::ptr::null_mut();
        }
    }

    p = core::ptr::addr_of_mut!((*Thread).Buffer) as PSTR;

    _itoa(
        (in_0.S_un.S_addr & 0xff) as i32,
        b.as_mut_ptr(),
        10
    );
    strcpy(p, b.as_mut_ptr());

    _itoa(
        ((in_0.S_un.S_addr >> 8) & 0xff) as i32,
        b.as_mut_ptr(),
        10
    );
    strcat(p, b".\0".as_ptr() as PSTR);
    strcat(p, b.as_mut_ptr());

    _itoa(
        ((in_0.S_un.S_addr >> 16) & 0xff) as i32,
        b.as_mut_ptr(),
        10
    );
    strcat(p, b".\0".as_ptr() as PSTR);
    strcat(p, b.as_mut_ptr());

    _itoa(
        ((in_0.S_un.S_addr >> 24) & 0xff) as i32,
        b.as_mut_ptr(),
        10
    );
    strcat(p, b".\0".as_ptr() as PSTR);
    strcat(p, b.as_mut_ptr());

    if ManualLoad != 0 {
        WSACleanup();
    }

    p
}

#[no_mangle]
pub unsafe extern "system" fn htonl(hostlong: u_long) -> u_long {
    (hostlong & 0xFF000000) >> 24
        | (hostlong & 0x00FF0000) >> 8
        | (hostlong & 0x0000FF00) << 8
        | (hostlong & 0x000000FF) << 24
}

#[no_mangle]
pub unsafe extern "system" fn htons(hostshort: u16) -> u16 {
    ((hostshort as i32 & 0xff00) >> 8 | (hostshort as i32 & 0x00ff) << 8) as u16
}

#[no_mangle]
pub unsafe extern "system" fn ntohl(netlong: u_long) -> u_long {
    ((netlong & 0xFF000000) >> 24) |
    ((netlong & 0x00FF0000) >> 8) |
    ((netlong & 0x0000FF00) << 8) |
    ((netlong & 0x000000FF) << 24)
}

#[no_mangle]
pub unsafe extern "system" fn ntohs(netshort: u16) -> u16 {
    ((netshort as i32 & 0xff00) >> 8 | (netshort as i32 & 0xff) << 8) as u16
}

#[no_mangle]
pub unsafe extern "system" fn WSAHtonl(
    s: SOCKET,
    hostlong: u_long,
    lpnetlong: *mut u_long,
) -> i32 {

    let mut ErrorCode: i32;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == ERROR_SUCCESS as i32 {
        if lpnetlong.is_null() {
            SetLastError(WSAEFAULT);
            return SOCKET_ERROR;
        }

        let Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if (*(*Socket).CatalogEntry).ProtocolInfo.iNetworkByteOrder == LITTLEENDIAN as i32 {
                *lpnetlong = hostlong;
            } else {
                *lpnetlong = ((hostlong & 0xFF000000) >> 24)
                    | ((hostlong & 0x00FF0000) >> 8)
                    | ((hostlong & 0x0000FF00) << 8)
                    | ((hostlong & 0x000000FF) << 24);
            }

            WsSockDereference(Socket);
            return ERROR_SUCCESS as i32;
        } else {
            ErrorCode = WSAENOTSOCK as i32;
        }
    }

    SetLastError(ErrorCode as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSAHtons(
    s: SOCKET,
    hostshort: u16,
    lpnetshort: *mut u16,
) -> i32 {
    let mut ErrorCode: i32;
    let mut Socket: PWSSOCKET;

    ErrorCode = WsQuickProlog() as i32;
    if ErrorCode == ERROR_SUCCESS as i32 {
        if lpnetshort.is_null() {
            SetLastError(WSAEFAULT);
            return SOCKET_ERROR;
        }

        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if (*(*Socket).CatalogEntry).ProtocolInfo.iNetworkByteOrder == (LITTLEENDIAN as i32) {
                *lpnetshort = hostshort;
            } else {
                *lpnetshort = (((hostshort as i32 & 0xFF00) >> 8) |
                               ((hostshort as i32 & 0x00FF) << 8)) as u16;
            }

            WsSockDereference(Socket);

            return ERROR_SUCCESS as i32;
        } else {
            ErrorCode = WSAENOTSOCK as i32;
        }
    }

    SetLastError(ErrorCode as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSANtohl(
    s: SOCKET,
    netlong: u_long,
    lphostlong: *mut u_long,
) -> i32 {
    let mut ErrorCode: i32;
    let mut Socket: PWSSOCKET;

    ErrorCode = (WsQuickProlog() as i32);
    if ErrorCode == (ERROR_SUCCESS as i32) {
        if lphostlong.is_null() {
            SetLastError(WSAEFAULT);
            return SOCKET_ERROR;
        }

        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if (*(*Socket).CatalogEntry).ProtocolInfo.iNetworkByteOrder == (LITTLEENDIAN as i32) {
                *lphostlong = netlong;
            } else {
                *lphostlong = ((netlong & 0xFF000000u32) >> 24)
                    | ((netlong & 0x00FF0000u32) >> 8)
                    | ((netlong & 0x0000FF00u32) << 8)
                    | ((netlong & 0x000000FFu32) << 24);
            }

            WsSockDereference(Socket);
            return (ERROR_SUCCESS as i32);
        } else {
            ErrorCode = (WSAENOTSOCK as i32);
        }
    }

    SetLastError(ErrorCode as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSANtohs(
    s: SOCKET,
    netshort: u16,
    lphostshort: *mut u16,
) -> i32 {
    let mut ErrorCode: i32;
    let mut Socket: PWSSOCKET;

    ErrorCode = WsQuickProlog() as i32;
    if ErrorCode == ERROR_SUCCESS as i32 {
        if lphostshort.is_null() {
            SetLastError(WSAEFAULT);
            return SOCKET_ERROR;
        }

        Socket = WsSockGetSocket(s);
        if !Socket.is_null() {
            if (*(*Socket).CatalogEntry).ProtocolInfo.iNetworkByteOrder == LITTLEENDIAN as i32 {
                *lphostshort = netshort;
            } else {
                *lphostshort = (((netshort as i32 & 0xFF00) >> 8) |
                                ((netshort as i32 & 0x00FF) << 8)) as u16;
            }

            WsSockDereference(Socket);
            return ERROR_SUCCESS as i32;
        } else {
            ErrorCode = WSAENOTSOCK as i32;
        }
    }

    SetLastError(ErrorCode as u32);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn inet_ntop(
    Family: i32,
    pAddr: *const core::ffi::c_void,
    pStringBuf: PSTR,
    StringBufSize: usize,
) -> PCSTR {
    let mut Status: i32;
    let mut BufSize: u32 = StringBufSize as u32;

    match Family as u32 {
        AF_INET => {
            Status = RtlIpv4AddressToStringExA(
                pAddr as *const IN_ADDR,
                0,
                pStringBuf,
                &mut BufSize,
            );
        }
        AF_INET6 => {
            Status = RtlIpv6AddressToStringExA(
                (pAddr as *const core::ffi::c_void as *const in6_addr),
                0,
                0,
                pStringBuf,
                &mut BufSize,
            );
        }
        _ => {
            SetLastError(WSAEAFNOSUPPORT as u32);
            return core::ptr::null();
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastError(WSAEINVAL as u32);
        return core::ptr::null();
    }

    return pStringBuf as PCSTR;
}

#[no_mangle]
pub unsafe extern "system" fn InetNtopW(
    Family: i32,
    pAddr: *const core::ffi::c_void,
    pStringBuf: PWSTR,
    StringBufSize: usize,
) -> PCWSTR {

    let mut Status: i32;
    let mut BufSize: u32 = StringBufSize as u32;

    match Family as u32 {
        AF_INET => {
            Status = RtlIpv4AddressToStringExW(
                pAddr as *const IN_ADDR,
                0u16,
                pStringBuf,
                core::ptr::addr_of_mut!(BufSize),
            );
        }
        AF_INET6 => {
            Status = RtlIpv6AddressToStringExW(
                (pAddr as *const core::ffi::c_void as *const in6_addr),
                0u32,
                0u16,
                pStringBuf,
                core::ptr::addr_of_mut!(BufSize),
            );
        }
        _ => {
            SetLastError(WSAEAFNOSUPPORT as u32);
            return core::ptr::null();
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastError(WSAEINVAL as u32);
        return core::ptr::null();
    }

    pStringBuf as PCWSTR
}

#[no_mangle]
pub unsafe extern "system" fn inet_pton(
    Family: INT,
    pszAddrString: PCSTR,
    pAddrBuf: PVOID,
) -> INT {
    let mut Status: NTSTATUS;
    let mut ch: PCSTR = core::ptr::null::<CHAR>();

    if pszAddrString.is_null() || pAddrBuf.is_null() {
        SetLastError(WSAEFAULT);
        return -1;
    }

    match Family as u32 {
        AF_INET => {
            Status = RtlIpv4StringToAddressA(
                pszAddrString,
                1u8,
                core::ptr::addr_of_mut!(ch),
                pAddrBuf as *mut IN_ADDR,
            );
        }
        AF_INET6 => {
            Status = RtlIpv6StringToAddressA(
                pszAddrString,
                core::ptr::addr_of_mut!(ch),
                pAddrBuf as *mut IN6_ADDR,
            );
        }
        _ => {
            SetLastError(WSAEAFNOSUPPORT);
            return -1;
        }
    }

    if !NT_SUCCESS(Status) || (*ch as i32 != 0) {
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn InetPtonW(
    Family: i32,
    pszAddrString: PCWSTR,
    pAddrBuf: *mut core::ffi::c_void,
) -> i32 {

    let mut Status: i32;
    let mut ch: PCWSTR = core::ptr::null();

    if pszAddrString.is_null() || pAddrBuf.is_null() {
        SetLastError(WSAEFAULT);
        return -1;
    }

    match Family as u32 {
        AF_INET => {
            Status = RtlIpv4StringToAddressW(
                pszAddrString,
                1u8,
                core::ptr::addr_of_mut!(ch),
                pAddrBuf as *mut IN_ADDR,
            );
        }
        AF_INET6 => {
            Status = RtlIpv6StringToAddressW(
                pszAddrString,
                core::ptr::addr_of_mut!(ch),
                (pAddrBuf as *mut in6_addr),
            );
        }
        _ => {
            SetLastError(WSAEAFNOSUPPORT);
            return -1;
        }
    }

    if !NT_SUCCESS(Status) || (*ch as i32 != 0) {
        SetLastError(WSAEINVAL);
        return 0;
    }

    return 1;
}
// enforce_import_abi: moved 3 import decl(s) into the extern block their
// exporting DLL's .spec requires: _itoa->C, strcat->C, strcpy->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/addrconv.c
//  * PURPOSE:     Address and Port Conversion Support
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
// /* DEFINES *******************************************************************/
//
// #ifndef BE
//
// /* DWORD network to host byte order conversion for little endian machines */
// #define DN2H(dw) \
//   ((((dw) & 0xFF000000L) >> 24) | \
//    (((dw) & 0x00FF0000L) >> 8) | \
//      (((dw) & 0x0000FF00L) << 8) | \
//      (((dw) & 0x000000FFL) << 24))
//
// /* DWORD host to network byte order conversion for little endian machines */
// #define DH2N(dw) \
//     ((((dw) & 0xFF000000L) >> 24) | \
//      (((dw) & 0x00FF0000L) >> 8) | \
//      (((dw) & 0x0000FF00L) << 8) | \
//      (((dw) & 0x000000FFL) << 24))
//
// /* WORD network to host order conversion for little endian machines */
// #define WN2H(w) \
//     ((((w) & 0xFF00) >> 8) | \
//      (((w) & 0x00FF) << 8))
//
// /* WORD host to network byte order conversion for little endian machines */
// #define WH2N(w) \
//     ((((w) & 0xFF00) >> 8) | \
//      (((w) & 0x00FF) << 8))
//
// #else /* BE */
//
// /* DWORD network to host byte order conversion for big endian machines */
// #define DN2H(dw) \
//     (dw)
//
// /* DWORD host to network byte order conversion big endian machines */
// #define DH2N(dw) \
//     (dw)
//
// /* WORD network to host order conversion for big endian machines */
// #define WN2H(w) \
//     (w)
//
// /* WORD host to network byte order conversion for big endian machines */
// #define WH2N(w) \
//     (w)
//
// #endif /* BE */
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// ULONG
// __stdcall
// inet_addr(IN  CONST CHAR * cp)
// {
//     register u_long val, base, n;
//     register unsigned char c;
//     u_long parts[4], *pp = parts;
//     if (!cp) return INADDR_ANY;
//     if (!isdigit(*cp)) return INADDR_NONE;
//
// again:
//     /*
//      * Collect number up to ``.''.
//      * Values are specified as for C:
//      * 0x=hex, 0=octal, other=decimal.
//      */
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
//          * Internet format:
//          *    a.b.c.d
//          *    a.b.c    (with c treated as 16-bits)
//          *    a.b    (with b treated as 24 bits)
//          */
//         if (pp >= parts + 4) return (INADDR_NONE);
//         *pp++ = val;
//         cp++;
//         goto again;
//     }
//     /*
//      * Check for trailing characters.
//      */
//     if (*cp && !isspace((UCHAR)*cp)) return (INADDR_NONE);
//
//     *pp++ = val;
//     /*
//      * Concoct the address according to
//      * the number of parts specified.
//      */
//     n = (u_long)(pp - parts);
//     switch (n) {
//
//     case 1:                /* a -- 32 bits */
//         val = parts[0];
//         break;
//
//     case 2:                /* a.b -- 8.24 bits */
//         val = (parts[0] << 24) | (parts[1] & 0xffffff);
//         break;
//
//     case 3:                /* a.b.c -- 8.8.16 bits */
//         val = (parts[0] << 24) | ((parts[1] & 0xff) << 16) |
//             (parts[2] & 0xffff);
//         break;
//
//     case 4:                /* a.b.c.d -- 8.8.8.8 bits */
//         val = (parts[0] << 24) | ((parts[1] & 0xff) << 16) |
//               ((parts[2] & 0xff) << 8) | (parts[3] & 0xff);
//         break;
//
//     default:
//         return (INADDR_NONE);
//     }
//     val = htonl(val);
//     return (val);
// }
//
// /*
//  * @implemented
//  */
// CHAR *
//
// inet_ntoa(IN IN_ADDR in)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     WSADATA WsaData;
//     BOOL ManualLoad = FALSE;
//     CHAR b[10];
//     PCHAR p;
//     DPRINT("inet_ntoa: %lx\n", in);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//     {
//         DPRINT("MANUAL LOAD\n");
//
//         /* Only fail if the error wasn't related to a missing WSAStartup */
//         if (ErrorCode != WSANOTINITIALISED)
//         {
//             /* Fail */
//             SetLastError(ErrorCode);
//             return NULL;
//         }
//
//         /* Apps aren't expected to call WSAStartup for this API, so we will */
//         if ((ErrorCode = WSAStartup(MAKEWORD(2,2), &WsaData)) != ERROR_SUCCESS)
//         {
//             /* We failed */
//             SetLastError(ErrorCode);
//             return NULL;
//         }
//
//         /* Try the prolog again */
//         ManualLoad = TRUE;
//         if ((ErrorCode = WsApiProlog(&Process, &Thread)) != ERROR_SUCCESS)
//         {
//             /* Failed again... */
//             WSACleanup();
//             SetLastError(ErrorCode);
//             return NULL;
//         }
//     }
//
//     p = Thread->Buffer;
//     _itoa(in.S_un.S_addr & 0xFF, b, 10);
//     strcpy(p, b);
//     _itoa((in.S_un.S_addr >> 8) & 0xFF, b, 10);
//     strcat(p, ".");
//     strcat(p, b);
//     _itoa((in.S_un.S_addr >> 16) & 0xFF, b, 10);
//     strcat(p, ".");
//     strcat(p, b);
//     _itoa((in.S_un.S_addr >> 24) & 0xFF, b, 10);
//     strcat(p, ".");
//     strcat(p, b);
//
//     /* Cleanup the manual load */
//     if (ManualLoad) WSACleanup();
//
//     /* Return the buffer */
//     return p;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// __stdcall
// htonl(IN ULONG hostlong)
// {
//     return DH2N(hostlong);
// }
//
// /*
//  * @implemented
//  */
// USHORT
// __stdcall
// htons(IN USHORT hostshort)
// {
//     return WH2N(hostshort);
// }
//
// /*
//  * @implemented
//  */
// ULONG
// __stdcall
// ntohl(IN ULONG netlong)
// {
//     return DN2H(netlong);
// }
//
// /*
//  * @implemented
//  */
// USHORT
// __stdcall
// ntohs(IN USHORT netshort)
// {
//     return WN2H(netshort);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAHtonl(IN SOCKET s,
//          IN ULONG hostlong,
//          OUT ULONG * lpnetlong)
// {
//     INT ErrorCode;
//     PWSSOCKET Socket;
//     DPRINT("WSAHtonl: %p, %lx, %p\n", s, hostlong, lpnetlong);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Make sure we got a parameter */
//         if (!lpnetlong)
//         {
//             /* Fail */
//             SetLastError(WSAEFAULT);
//             return SOCKET_ERROR;
//         }
//
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Check which byte order to use */
//             if (Socket->CatalogEntry->ProtocolInfo.iNetworkByteOrder ==
//                 LITTLEENDIAN)
//             {
//                 /* No conversion needed */
//                 *lpnetlong = hostlong;
//             }
//             else
//             {
//                 /* Use a swap */
//                 *lpnetlong = DN2H(hostlong);
//             }
//
//             /* Dereference the socket */
//             WsSockDereference(Socket);
//
//             /* Return success */
//             return ERROR_SUCCESS;
//         }
//         else
//         {
//             /* Set the error code */
//             ErrorCode = WSAENOTSOCK;
//         }
//     }
//
//     /* Return with error */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAHtons(IN SOCKET s,
//          IN USHORT hostshort,
//          OUT USHORT * lpnetshort)
// {
//     INT ErrorCode;
//     PWSSOCKET Socket;
//     DPRINT("WSAHtons: %p, %lx, %p\n", s, hostshort, lpnetshort);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Make sure we got a parameter */
//         if (!lpnetshort)
//         {
//             /* Fail */
//             SetLastError(WSAEFAULT);
//             return SOCKET_ERROR;
//         }
//
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Check which byte order to use */
//             if (Socket->CatalogEntry->ProtocolInfo.iNetworkByteOrder ==
//                 LITTLEENDIAN)
//             {
//                 /* No conversion needed */
//                 *lpnetshort = hostshort;
//             }
//             else
//             {
//                 /* Use a swap */
//                 *lpnetshort = WN2H(hostshort);
//             }
//
//             /* Dereference the socket */
//             WsSockDereference(Socket);
//
//             /* Return success */
//             return ERROR_SUCCESS;
//         }
//         else
//         {
//             /* Set the error code */
//             ErrorCode = WSAENOTSOCK;
//         }
//     }
//
//     /* Return with error */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSANtohl(IN SOCKET s,
//          IN ULONG netlong,
//          OUT ULONG * lphostlong)
// {
//     INT ErrorCode;
//     PWSSOCKET Socket;
//     DPRINT("WSANtohl: %p, %lx, %p\n", s, netlong, lphostlong);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Make sure we got a parameter */
//         if (!lphostlong)
//         {
//             /* Fail */
//             SetLastError(WSAEFAULT);
//             return SOCKET_ERROR;
//         }
//
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Check which byte order to use */
//             if (Socket->CatalogEntry->ProtocolInfo.iNetworkByteOrder ==
//                 LITTLEENDIAN)
//             {
//                 /* No conversion needed */
//                 *lphostlong = netlong;
//             }
//             else
//             {
//                 /* Use a swap */
//                 *lphostlong = DN2H(netlong);
//             }
//
//             /* Dereference the socket */
//             WsSockDereference(Socket);
//
//             /* Return success */
//             return ERROR_SUCCESS;
//         }
//         else
//         {
//             /* Set the error code */
//             ErrorCode = WSAENOTSOCK;
//         }
//     }
//
//     /* Return with error */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSANtohs(IN SOCKET s,
//          IN USHORT netshort,
//          OUT USHORT * lphostshort)
// {
//     INT ErrorCode;
//     PWSSOCKET Socket;
//     DPRINT("WSANtohs: %p, %lx, %p\n", s, netshort, lphostshort);
//
//     /* Check for WSAStartup */
//     if ((ErrorCode = WsQuickProlog()) == ERROR_SUCCESS)
//     {
//         /* Make sure we got a parameter */
//         if (!lphostshort)
//         {
//             /* Fail */
//             SetLastError(WSAEFAULT);
//             return SOCKET_ERROR;
//         }
//
//         /* Get the Socket Context */
//         if ((Socket = WsSockGetSocket(s)))
//         {
//             /* Check which byte order to use */
//             if (Socket->CatalogEntry->ProtocolInfo.iNetworkByteOrder ==
//                 LITTLEENDIAN)
//             {
//                 /* No conversion needed */
//                 *lphostshort = netshort;
//             }
//             else
//             {
//                 /* Use a swap */
//                 *lphostshort = WN2H(netshort);
//             }
//
//             /* Dereference the socket */
//             WsSockDereference(Socket);
//
//             /* Return success */
//             return ERROR_SUCCESS;
//         }
//         else
//         {
//             /* Set the error code */
//             ErrorCode = WSAENOTSOCK;
//         }
//     }
//
//     /* Return with error */
//     SetLastError(ErrorCode);
//     return SOCKET_ERROR;
// }
//
// PCSTR
// __stdcall
// inet_ntop(
//      INT Family,
//      const VOID *pAddr,
//      PSTR pStringBuf,
//      size_t StringBufSize)
// {
//     NTSTATUS Status;
//     ULONG BufSize = StringBufSize;
//
//     switch (Family)
//     {
//         case AF_INET:
//             Status = RtlIpv4AddressToStringExA(pAddr, 0, pStringBuf, &BufSize);
//             break;
//         case AF_INET6:
//             Status = RtlIpv6AddressToStringExA(pAddr, 0, 0, pStringBuf, &BufSize);
//             break;
//         default:
//             SetLastError(WSAEAFNOSUPPORT);
//             return NULL;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(WSAEINVAL);
//         return NULL;
//     }
//
//     return pStringBuf;
// }
//
// PCWSTR
// __stdcall
// InetNtopW(
//      INT Family,
//      const VOID *pAddr,
//      PWSTR pStringBuf,
//      size_t StringBufSize)
// {
//     NTSTATUS Status;
//     ULONG BufSize = StringBufSize;
//
//     switch (Family)
//     {
//         case AF_INET:
//             Status = RtlIpv4AddressToStringExW(pAddr, 0, pStringBuf, &BufSize);
//             break;
//         case AF_INET6:
//             Status = RtlIpv6AddressToStringExW(pAddr, 0, 0, pStringBuf, &BufSize);
//             break;
//         default:
//             SetLastError(WSAEAFNOSUPPORT);
//             return NULL;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(WSAEINVAL);
//         return NULL;
//     }
//
//     return pStringBuf;
// }
//
// INT
// __stdcall
// inet_pton(
//      INT Family,
//      PCSTR pszAddrString,
//     (sizeof(IN_ADDR6)) PVOID pAddrBuf)
// {
//     NTSTATUS Status;
//     PCSTR ch;
//
//     if (!pszAddrString || !pAddrBuf)
//     {
//         SetLastError(WSAEFAULT);
//         return -1;
//     }
//
//     switch (Family)
//     {
//         case AF_INET:
//             Status = RtlIpv4StringToAddressA(pszAddrString, TRUE, &ch, pAddrBuf);
//             break;
//         case AF_INET6:
//             Status = RtlIpv6StringToAddressA(pszAddrString, &ch, pAddrBuf);
//             break;
//         default:
//             SetLastError(WSAEAFNOSUPPORT);
//             return -1;
//     }
//
//     if (!NT_SUCCESS(Status) || (*ch != 0))
//     {
//         return 0;
//     }
//
//     return 1;
// }
//
// INT
// __stdcall
// InetPtonW(
//      INT Family,
//      PCWSTR pszAddrString,
//     (sizeof(IN_ADDR6)) PVOID pAddrBuf)
// {
//     NTSTATUS Status;
//     PCWSTR ch;
//
//     if (!pszAddrString || !pAddrBuf)
//     {
//         SetLastError(WSAEFAULT);
//         return -1;
//     }
//
//     switch (Family)
//     {
//         case AF_INET:
//             Status = RtlIpv4StringToAddressW(pszAddrString, TRUE, &ch, pAddrBuf);
//             break;
//         case AF_INET6:
//             Status = RtlIpv6StringToAddressW(pszAddrString, &ch, pAddrBuf);
//             break;
//         default:
//             SetLastError(WSAEAFNOSUPPORT);
//             return -1;
//     }
//
//     if (!NT_SUCCESS(Status) || (*ch != 0))
//     {
//         SetLastError(WSAEINVAL); /* Only unicode version sets this error */
//         return 0;
//     }
//
//     return 1;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

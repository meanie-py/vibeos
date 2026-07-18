use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn OutputDebugStringW(lpOutputString: LPCWSTR);
}

// ---- file-scope preamble (translated) ----
pub type LPFN_NSPAPI = Option<unsafe extern "system" fn() -> DWORD>;

#[repr(C)]
pub struct _NS_ROUTINE {
    pub dwFunctionCount: DWORD,
    pub alpfnFunctions: *mut LPFN_NSPAPI,
    pub dwNameSpace: DWORD,
    pub dwPriority: DWORD,
}

pub type NS_ROUTINE = _NS_ROUTINE;
pub type PNS_ROUTINE = *mut _NS_ROUTINE;
pub type LPNS_ROUTINE = *mut _NS_ROUTINE;

#[no_mangle]
pub unsafe extern "system" fn EnumProtocolsA(
    ProtocolCount: LPINT,
    ProtocolBuffer: LPVOID,
    BufferLength: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0E\0n\0u\0m\0P\0r\0o\0t\0o\0c\0o\0l\0s\0A\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR,
    );
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn EnumProtocolsW(
    ProtocolCount: LPINT,
    ProtocolBuffer: LPVOID,
    BufferLength: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0E\0n\0u\0m\0P\0r\0o\0t\0o\0c\0o\0l\0s\0W\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16
    );

    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn GetAddressByNameA(
    NameSpace: DWORD,
    ServiceType: LPGUID,
    ServiceName: LPSTR,
    Protocols: LPINT,
    Resolution: DWORD,
    ServiceAsyncInfo: LPSERVICE_ASYNC_INFO,
    CsaddrBuffer: LPVOID,
    BufferLength: LPDWORD,
    AliasBuffer: LPSTR,
    AliasBufferLength: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0A\0d\0d\0r\0e\0s\0s\0B\0y\0N\0a\0m\0e\0A\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR,
    );

    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn GetAddressByNameW(
    NameSpace: DWORD,
    ServiceType: LPGUID,
    ServiceName: LPWSTR,
    Protocols: LPINT,
    Resolution: DWORD,
    ServiceAsyncInfo: LPSERVICE_ASYNC_INFO,
    CsaddrBuffer: LPVOID,
    BufferLength: LPDWORD,
    AliasBuffer: LPWSTR,
    AliasBufferLength: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0A\0d\0d\0r\0e\0s\0s\0B\0y\0N\0a\0m\0e\0W\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR
    );

    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn GetServiceA(
    NameSpace: DWORD,
    Guid: LPGUID,
    ServiceName: LPSTR,
    Properties: DWORD,
    Buffer: LPVOID,
    BufferSize: LPDWORD,
    ServiceAsyncInfo: LPSERVICE_ASYNC_INFO,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0S\0e\0r\0v\0i\0c\0e\0A\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR
    );
    
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn GetServiceW(
    _NameSpace: DWORD,
    _Guid: LPGUID,
    _ServiceName: LPWSTR,
    _Properties: DWORD,
    _Buffer: LPVOID,
    _BufferSize: LPDWORD,
    _ServiceAsyncInfo: LPSERVICE_ASYNC_INFO,
) -> INT {

    OutputDebugStringW(b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0S\0e\0r\0v\0i\0c\0e\0W\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR);

    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn GetTypeByNameA(
    ServiceName: LPSTR,
    ServiceType: LPGUID,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0T\0y\0p\0e\0B\0y\0N\0a\0m\0e\0A\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR
    );

    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn GetTypeByNameW(
    ServiceName: LPWSTR,
    ServiceType: LPGUID,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0T\0y\0p\0e\0B\0y\0N\0a\0m\0e\0W\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR,
    );
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn MigrateWinsockConfiguration(
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0M\0i\0g\0r\0a\0t\0e\0W\0i\0n\0s\0o\0c\0k\0C\0o\0n\0f\0i\0g\0u\0r\0a\0t\0i\0o\0n\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR,
    );
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn SetServiceA(
    NameSpace: DWORD,
    Operation: DWORD,
    Flags: DWORD,
    ServiceInfo: LPSERVICE_INFOA,
    ServiceAsyncInfo: LPSERVICE_ASYNC_INFO,
    dwStatusFlags: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0S\0e\0t\0S\0e\0r\0v\0i\0c\0e\0A\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0"
            .as_ptr() as LPCWSTR
    );
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn SetServiceW(
    NameSpace: DWORD,
    Operation: DWORD,
    Flags: DWORD,
    ServiceInfo: LPSERVICE_INFOW,
    ServiceAsyncInfo: LPSERVICE_ASYNC_INFO,
    dwStatusFlags: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0S\0e\0t\0S\0e\0r\0v\0i\0c\0e\0W\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0"
            .as_ptr() as LPCWSTR
    );
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn WSARecvEx(
    Sock: SOCKET,
    Buf: *mut i8,
    Len: i32,
    Flags: *mut i32,
) -> i32 {

    OutputDebugStringW(
        b"w\0\0\03\0\0\02\0\0\0s\0\0\0o\0\0\0c\0\0\0k\0\0\0 \0\0\0W\0\0\0S\0\0\0A\0\0\0R\0\0\0e\0\0\0c\0\0\0v\0\0\0E\0\0\0x\0\0\0 \0\0\0s\0\0\0t\0\0\0u\0\0\0b\0\0\0 \0\0\0c\0\0\0a\0\0\0l\0\0\0l\0\0\0e\0\0\0d\0\0\0\n\0\0\0\0\0\0\0".as_ptr() as *const u16,
    );
    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn dn_expand(
    MessagePtr: *mut u8,
    EndofMesOrig: *mut u8,
    CompDomNam: *mut u8,
    ExpandDomNam: *mut u8,
    Length: i32,
) -> i32 {

    OutputDebugStringW(b"w\x003\x002\0s\0o\0c\0k\0 \0d\0n\0_\0e\0x\0p\0a\0n\0d\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16);
    return SOCKET_ERROR;
}

#[no_mangle]
pub unsafe extern "system" fn getnetbyname(name: *const i8) -> *mut netent {
    
    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0g\0e\0t\0n\0e\0t\0b\0y\0n\0a\0m\0e\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR
    );
    
    core::ptr::null_mut::<netent>()
}

#[no_mangle]
pub unsafe extern "system" fn inet_network(cp: *const i8) -> UINT {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0i\0n\0e\0t\0_\0n\0e\0t\0w\0o\0r\0k\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0"
            .as_ptr() as *const u16
    );
    
    INADDR_NONE
}

#[no_mangle]
pub unsafe extern "system" fn rcmd(
    AHost: *mut *mut i8,
    InPort: u16,
    LocUser: *mut i8,
    RemUser: *mut i8,
    Cmd: *mut i8,
    Fd2p: *mut i32,
) -> usize {

    OutputDebugStringW(b"w\0\x003\0\x002\0\x00s\0\x00o\0\x00c\0\x00k\0\x00 \0\x00r\0\x00c\0\x00m\0\x00d\0\x00 \0\x00s\0\x00t\0\x00u\0\x00b\0\x00 \0\x00c\0\x00a\0\x00l\0\x00l\0\x00e\0\x00d\0\x00\n\0\x00\0\0\x00".as_ptr() as *const u16);

    return INVALID_SOCKET as usize;
}

#[no_mangle]
pub unsafe extern "system" fn rexec(
    AHost: *mut *mut i8,
    InPort: i32,
    User: *mut i8,
    Passwd: *mut i8,
    Cmd: *mut i8,
    Fd2p: *mut i32,
) -> SOCKET {

    OutputDebugStringW(
        b"w\0003\x002\0s\0o\0c\0k\0 \0r\0e\0x\0e\0c\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16
    );
    return INVALID_SOCKET;
}

#[no_mangle]
pub unsafe extern "system" fn rresvport(port: *mut i32) -> SOCKET {
    
    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0r\0r\0e\0s\0v\0p\0o\0r\0t\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR
    );
    
    INVALID_SOCKET
}

#[no_mangle]
pub unsafe extern "system" fn s_perror(str: *const i8) {
    
    OutputDebugStringW(
        &core::mem::transmute::<[u8; 60], [u16; 30]>(
            *b"w\x003\x002\0s\0o\0c\0k\0 \0s\0_\0p\0e\0r\0r\0o\0r\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0"
        ) as *const u16
    );
}

#[no_mangle]
pub unsafe extern "system" fn sethostname(
    Name: *mut i8,
    NameLen: i32,
) -> i32 {

    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0s\0e\0t\0h\0o\0s\0t\0n\0a\0m\0e\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16
    );

    SOCKET_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    InstDLL: HINSTANCE,
    Reason: DWORD,
    Reserved: LPVOID,
) -> BOOL {
    return TRUE as INT;
}

#[no_mangle]
pub unsafe extern "system" fn GetNameByTypeA(
    lpServiceType: LPGUID,
    lpServiceName: LPSTR,
    dwNameLength: DWORD,
) -> INT {
    
    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0N\0a\0m\0e\0B\0y\0T\0y\0p\0e\0A\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16
    );
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetNameByTypeW(
    lpServiceType: LPGUID,
    lpServiceName: LPWSTR,
    dwNameLength: DWORD,
) -> INT {
    
    OutputDebugStringW(
        b"w\x003\x002\0s\0o\0c\0k\0 \0G\0e\0t\0N\0a\0m\0e\0B\0y\0T\0y\0p\0e\0W\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16
    );
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn WSPStartup(
    wVersionRequested: WORD,
    lpWSPData: LPWSPDATA,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    UpcallTable: WSPUPCALLTABLE,
    lpProcTable: LPWSPPROC_TABLE,
) -> ::core::ffi::c_int {
    return TRUE as INT;
}

#[no_mangle]
pub unsafe extern "system" fn NPLoadNameSpaces(
    lpdwVersion: LPDWORD,
    nsrBuffer: LPNS_ROUTINE,
    lpdwBufferLength: LPDWORD,
) -> INT {

    OutputDebugStringW(
        b"m\0s\0w\0s\0o\0c\0k\0 \0N\0P\0L\0o\0a\0d\0N\0a\0m\0e\0S\0p\0a\0c\0e\0s\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR,
    );

    *lpdwVersion = 1;

    return TRUE as INT;
}

#[no_mangle]
pub unsafe extern "system" fn StartWsdpService() {
    
    OutputDebugStringW(
        b"m\0s\0w\0s\0o\0c\0k\0 \0S\0t\0a\0r\0t\0W\0s\0d\0p\0S\0e\0r\0v\0i\0c\0e\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16
    );
}

#[no_mangle]
pub unsafe extern "system" fn StopWsdpService() {
    
    OutputDebugStringW(b"m\0s\0w\0s\0o\0c\0k\0 \0S\0t\0o\0p\0W\0s\0d\0p\0S\0e\0r\0v\0i\0c\0e\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16);
}

#[no_mangle]
pub unsafe extern "system" fn SvchostPushServiceGlobals(lpGlobals: PSVCHOST_GLOBAL_DATA) {
    
    OutputDebugStringW(b"m\0s\0w\0s\0o\0c\0k\0 \0S\0v\0c\0h\0o\0s\0t\0P\0u\0s\0h\0S\0e\0r\0v\0i\0c\0e\0G\0l\0o\0b\0a\0l\0s\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn ServiceMain(Unknown1: DWORD, Unknown2: DWORD) {
    
    OutputDebugStringW(b"m\0s\0w\0s\0o\0c\0k\0 \0S\0e\0r\0v\0i\0c\0e\0M\0a\0i\0n\0 \0s\0t\0u\0b\0 \0c\0a\0l\0l\0e\0d\0\n\0\0\0".as_ptr() as *const u16);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS WinSock DLL
//  * FILE:            stubs.c
//  * PURPOSE:         Stub functions
//  * PROGRAMMERS:     Ge van Geldorp (ge@gse.nl)
//  * REVISIONS:
//  */
//
// #include "precomp.h"
//
// #include <ws2spi.h>
// #include <nspapi.h>
// #include <svc.h>
//
// typedef DWORD (* LPFN_NSPAPI)(VOID);
// typedef struct _NS_ROUTINE {
//     DWORD        dwFunctionCount;
//     LPFN_NSPAPI *alpfnFunctions;
//     DWORD        dwNameSpace;
//     DWORD        dwPriority;
// } NS_ROUTINE, *PNS_ROUTINE, *  LPNS_ROUTINE;
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// EnumProtocolsA(LPINT ProtocolCount,
//                LPVOID ProtocolBuffer,
//                LPDWORD BufferLength)
// {
//   OutputDebugStringW(L"w32sock EnumProtocolsA stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// EnumProtocolsW(LPINT ProtocolCount,
//                LPVOID ProtocolBuffer,
//                LPDWORD BufferLength)
// {
//   OutputDebugStringW(L"w32sock EnumProtocolsW stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetAddressByNameA(DWORD NameSpace,
//                   LPGUID ServiceType,
//                   LPSTR ServiceName,
//                   LPINT Protocols,
//                   DWORD Resolution,
//                   LPSERVICE_ASYNC_INFO ServiceAsyncInfo,
//                   LPVOID CsaddrBuffer,
//                   LPDWORD BufferLength,
//                   LPSTR AliasBuffer,
//                   LPDWORD AliasBufferLength)
// {
//   OutputDebugStringW(L"w32sock GetAddressByNameA stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetAddressByNameW(DWORD NameSpace,
//                   LPGUID ServiceType,
//                   LPWSTR ServiceName,
//                   LPINT Protocols,
//                   DWORD Resolution,
//                   LPSERVICE_ASYNC_INFO ServiceAsyncInfo,
//                   LPVOID CsaddrBuffer,
//                   LPDWORD BufferLength,
//                   LPWSTR AliasBuffer,
//                   LPDWORD AliasBufferLength)
// {
//   OutputDebugStringW(L"w32sock GetAddressByNameW stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetServiceA(DWORD NameSpace,
//             LPGUID Guid,
//             LPSTR ServiceName,
//             DWORD Properties,
//             LPVOID Buffer,
//             LPDWORD BufferSize,
//             LPSERVICE_ASYNC_INFO ServiceAsyncInfo)
// {
//   OutputDebugStringW(L"w32sock GetServiceA stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetServiceW(DWORD NameSpace,
//             LPGUID Guid,
//             LPWSTR ServiceName,
//             DWORD Properties,
//             LPVOID Buffer,
//             LPDWORD BufferSize,
//             LPSERVICE_ASYNC_INFO ServiceAsyncInfo)
// {
//   OutputDebugStringW(L"w32sock GetServiceW stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetTypeByNameA(LPSTR ServiceName,
//                LPGUID ServiceType)
// {
//   OutputDebugStringW(L"w32sock GetTypeByNameA stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetTypeByNameW(LPWSTR ServiceName,
//                LPGUID ServiceType)
// {
//   OutputDebugStringW(L"w32sock GetTypeByNameW stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// MigrateWinsockConfiguration(DWORD Unknown1,
//                             DWORD Unknown2,
//                             DWORD Unknown3)
// {
//   OutputDebugStringW(L"w32sock MigrateWinsockConfiguration stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// SetServiceA(DWORD NameSpace,
//             DWORD Operation,
//             DWORD Flags,
//             LPSERVICE_INFOA ServiceInfo,
//             LPSERVICE_ASYNC_INFO ServiceAsyncInfo,
//             LPDWORD dwStatusFlags)
// {
//   OutputDebugStringW(L"w32sock SetServiceA stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// SetServiceW(DWORD NameSpace,
//             DWORD Operation,
//             DWORD Flags,
//             LPSERVICE_INFOW ServiceInfo,
//             LPSERVICE_ASYNC_INFO ServiceAsyncInfo,
//             LPDWORD dwStatusFlags)
// {
//   OutputDebugStringW(L"w32sock SetServiceW stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// int
// WINAPI
// WSARecvEx(SOCKET Sock,
//           char *Buf,
//           int Len,
//           int *Flags)
// {
//   OutputDebugStringW(L"w32sock WSARecvEx stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// int
// WINAPI
// dn_expand(unsigned char *MessagePtr,
//           unsigned char *EndofMesOrig,
//           unsigned char *CompDomNam,
//           unsigned char *ExpandDomNam,
//           int Length)
// {
//   OutputDebugStringW(L"w32sock dn_expand stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// struct netent *
// WINAPI
// getnetbyname(const char *name)
// {
//   OutputDebugStringW(L"w32sock getnetbyname stub called\n");
//
//   return NULL;
// }
//
//
// /*
//  * @unimplemented
//  */
// UINT
// WINAPI
// inet_network(const char *cp)
// {
//   OutputDebugStringW(L"w32sock inet_network stub called\n");
//
//   return INADDR_NONE;
// }
//
//
// /*
//  * @unimplemented
//  */
// SOCKET
// WINAPI
// rcmd(char **AHost,
//      USHORT InPort,
//      char *LocUser,
//      char *RemUser,
//      char *Cmd,
//      int *Fd2p)
// {
//   OutputDebugStringW(L"w32sock rcmd stub called\n");
//
//   return INVALID_SOCKET;
// }
//
//
// /*
//  * @unimplemented
//  */
// SOCKET
// WINAPI
// rexec(char **AHost,
//       int InPort,
//       char *User,
//       char *Passwd,
//       char *Cmd,
//       int *Fd2p)
// {
//   OutputDebugStringW(L"w32sock rexec stub called\n");
//
//   return INVALID_SOCKET;
// }
//
//
// /*
//  * @unimplemented
//  */
// SOCKET
// WINAPI
// rresvport(int *port)
// {
//   OutputDebugStringW(L"w32sock rresvport stub called\n");
//
//   return INVALID_SOCKET;
// }
//
//
// /*
//  * @unimplemented
//  */
// void
// WINAPI
// s_perror(const char *str)
// {
//   OutputDebugStringW(L"w32sock s_perror stub called\n");
// }
//
//
// /*
//  * @unimplemented
//  */
// int
// WINAPI
// sethostname(char *Name, int NameLen)
// {
//   OutputDebugStringW(L"w32sock sethostname stub called\n");
//
//   return SOCKET_ERROR;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// DllMain(HINSTANCE InstDLL,
//         DWORD Reason,
//         LPVOID Reserved)
// {
//   return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetNameByTypeA(LPGUID lpServiceType,LPSTR lpServiceName,DWORD dwNameLength)
// {
//   OutputDebugStringW(L"w32sock GetNameByTypeA stub called\n");
//   return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// GetNameByTypeW(LPGUID lpServiceType,LPWSTR lpServiceName,DWORD dwNameLength)
// {
//   OutputDebugStringW(L"w32sock GetNameByTypeW stub called\n");
//   return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// int
// WINAPI
// WSPStartup(
//     IN WORD wVersionRequested,
//     OUT LPWSPDATA lpWSPData,
//     IN LPWSAPROTOCOL_INFOW lpProtocolInfo,
//     IN WSPUPCALLTABLE UpcallTable,
//     OUT LPWSPPROC_TABLE lpProcTable
//     )
// {
//   return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// INT
// WINAPI
// NPLoadNameSpaces(
//     IN OUT LPDWORD lpdwVersion,
//     IN OUT LPNS_ROUTINE nsrBuffer,
//     IN OUT LPDWORD lpdwBufferLength
//     )
// {
//   OutputDebugStringW(L"mswsock NPLoadNameSpaces stub called\n");
//
//   *lpdwVersion = 1;
//
//   return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// StartWsdpService()
// {
//   OutputDebugStringW(L"mswsock StartWsdpService stub called\n");
// }
//
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// StopWsdpService()
// {
//   OutputDebugStringW(L"mswsock StopWsdpService stub called\n");
// }
//
//
// /*
//  * @unimplemented
//  *
//  * See https://www.geoffchappell.com/studies/windows/win32/services/svchost/dll/svchostpushserviceglobals.htm
//  */
// VOID
// WINAPI
// SvchostPushServiceGlobals(PSVCHOST_GLOBAL_DATA lpGlobals)
// {
//   OutputDebugStringW(L"mswsock SvchostPushServiceGlobals stub called\n");
// }
//
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// ServiceMain(DWORD Unknown1, DWORD Unknown2)
// {
//   OutputDebugStringW(L"mswsock ServiceMain stub called\n");
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

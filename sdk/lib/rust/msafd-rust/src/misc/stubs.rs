use crate::types::*;
use crate::glue::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn WSPCancelBlockingCall(
    lpErrno: LPINT,
) -> INT {
    // C: `UNIMPLEMENTED; return 0;` -- a debug print, never the panic `unimplemented!()` is.
    0
}
#[no_mangle]
pub unsafe extern "system" fn WSPGetQOSByName(
    s: SOCKET,
    lpQOSName: LPWSABUF,
    lpQOS: LPQOS,
    lpErrno: LPINT,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn WSPJoinLeaf(
    s: SOCKET,
    name: *const SOCKADDR,
    namelen: INT,
    lpCallerData: LPWSABUF,
    lpCalleeData: LPWSABUF,
    lpSQOS: LPQOS,
    lpGQOS: LPQOS,
    dwFlags: DWORD,
    lpErrno: LPINT,
) -> SOCKET {
    0 as SOCKET
}
#[no_mangle]
pub unsafe extern "system" fn WSPAcceptEx(
    sListenSocket: SOCKET,
    sAcceptSocket: SOCKET,
    lpOutputBuffer: PVOID,
    dwReceiveDataLength: DWORD,
    dwLocalAddressLength: DWORD,
    dwRemoteAddressLength: DWORD,
    lpdwBytesReceived: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn WSPDisconnectEx(
    hSocket: SOCKET,
    lpOverlapped: LPOVERLAPPED,
    dwFlags: DWORD,
    reserved: DWORD,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn WSPGetAcceptExSockaddrs(
    lpOutputBuffer: PVOID,
    dwReceiveDataLength: DWORD,
    dwLocalAddressLength: DWORD,
    dwRemoteAddressLength: DWORD,
    LocalSockaddr: *mut *mut sockaddr,
    LocalSockaddrLength: LPINT,
    RemoteSockaddr: *mut *mut sockaddr,
    RemoteSockaddrLength: LPINT,
) {
    unimplemented!();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Ancillary Function Driver DLL
//  * FILE:        dll/win32/msafd/misc/stubs.c
//  * PURPOSE:     Stubs
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  * REVISIONS:
//  *   CSH 01/09-2000 Created
//  */
//
// #include <msafd.h>
//
// INT
// WSPAPI
// WSPCancelBlockingCall(
//     OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//
//     return 0;
// }
//
//
// BOOL
// WSPAPI
// WSPGetQOSByName(
//     IN      SOCKET s,
//     IN OUT  LPWSABUF lpQOSName,
//     OUT     LPQOS lpQOS,
//     OUT     LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//
//     return FALSE;
// }
//
//
// SOCKET
// WSPAPI
// WSPJoinLeaf(
//     IN  SOCKET s,
//     IN  CONST SOCKADDR *name,
//     IN  INT namelen,
//     IN  LPWSABUF lpCallerData,
//     OUT LPWSABUF lpCalleeData,
//     IN  LPQOS lpSQOS,
//     IN  LPQOS lpGQOS,
//     IN  DWORD dwFlags,
//     OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//
//     return (SOCKET)0;
// }
//
// BOOL
// WSPAPI
// WSPAcceptEx(
//     IN SOCKET sListenSocket,
//     IN SOCKET sAcceptSocket,
//     OUT PVOID lpOutputBuffer,
//     IN DWORD dwReceiveDataLength,
//     IN DWORD dwLocalAddressLength,
//     IN DWORD dwRemoteAddressLength,
//     OUT LPDWORD lpdwBytesReceived,
//     IN OUT LPOVERLAPPED lpOverlapped)
// {
//     UNIMPLEMENTED;
//
//     return FALSE;
// }
//
// BOOL
// WSPAPI
// WSPDisconnectEx(
//   IN SOCKET hSocket,
//   IN LPOVERLAPPED lpOverlapped,
//   IN DWORD dwFlags,
//   IN DWORD reserved)
// {
//     UNIMPLEMENTED;
//
//     return FALSE;
// }
//
// VOID
// WSPAPI
// WSPGetAcceptExSockaddrs(
//     IN PVOID lpOutputBuffer,
//     IN DWORD dwReceiveDataLength,
//     IN DWORD dwLocalAddressLength,
//     IN DWORD dwRemoteAddressLength,
//     OUT struct sockaddr **LocalSockaddr,
//     OUT LPINT LocalSockaddrLength,
//     OUT struct sockaddr **RemoteSockaddr,
//     OUT LPINT RemoteSockaddrLength)
// {
//     UNIMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

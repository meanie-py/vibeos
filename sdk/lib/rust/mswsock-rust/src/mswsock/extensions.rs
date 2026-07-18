use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn WSAIoctl( s: SOCKET, dwIoControlCode: DWORD, lpvInBuffer: PVOID, cbInBuffer: DWORD, lpvOutBuffer: PVOID, cbOutBuffer: DWORD, lpcbBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, lpCompletionRoutine: PVOID, ) -> INT;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut pfnTransmitFile: LPFN_TRANSMITFILE = None;

pub static mut pfnGetAcceptExSockaddrs: LPFN_GETACCEPTEXSOCKADDRS = None;

pub static mut pfnAcceptEx: LPFN_ACCEPTEX = None;

#[no_mangle]
pub unsafe extern "system" fn TransmitFile(
    Socket: SOCKET,
    File: HANDLE,
    NumberOfBytesToWrite: DWORD,
    NumberOfBytesPerSend: DWORD,
    Overlapped: LPOVERLAPPED,
    TransmitBuffers: LPTRANSMIT_FILE_BUFFERS,
    Flags: DWORD,
) -> BOOL {
    let mut TransmitFileGUID: GUID = GUID {
        Data1: 0xb5367df0,
        Data2: 0xcbac,
        Data3: 0x11cf,
        Data4: [0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92],
    };
    let mut cbBytesReturned: DWORD = 0;

    if WSAIoctl(
        Socket,
        SIO_GET_EXTENSION_FUNCTION_POINTER,
        core::ptr::addr_of_mut!(TransmitFileGUID) as LPVOID,
        core::mem::size_of::<GUID>() as DWORD,
        core::ptr::addr_of_mut!(pfnTransmitFile) as LPVOID,
        core::mem::size_of::<LPFN_TRANSMITFILE>() as DWORD,
        core::ptr::addr_of_mut!(cbBytesReturned),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == SOCKET_ERROR
    {
        return FALSE as INT;
    }

    if let Some(pfn) = pfnTransmitFile {
        pfn(
            Socket,
            File,
            NumberOfBytesToWrite,
            NumberOfBytesPerSend,
            Overlapped,
            TransmitBuffers,
            Flags,
        )
    } else {
        FALSE as INT
    }
}

#[no_mangle]
pub unsafe extern "system" fn AcceptEx(
    ListenSocket: SOCKET,
    AcceptSocket: SOCKET,
    OutputBuffer: PVOID,
    ReceiveDataLength: DWORD,
    LocalAddressLength: DWORD,
    RemoteAddressLength: DWORD,
    BytesReceived: LPDWORD,
    Overlapped: LPOVERLAPPED,
) -> BOOL {
    let mut AcceptExGUID: GUID = GUID {
        Data1: 0xb5367df1,
        Data2: 0xcbac,
        Data3: 0x11cf,
        Data4: [0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92],
    };
    let mut GetAcceptExSockaddrsGUID: GUID = GUID {
        Data1: 0xb5367df2,
        Data2: 0xcbac,
        Data3: 0x11cf,
        Data4: [0x95, 0xca, 0x00, 0x80, 0x5f, 0x48, 0xa1, 0x92],
    };
    let mut cbBytesReturned: DWORD = 0;

    if WSAIoctl(
        ListenSocket,
        SIO_GET_EXTENSION_FUNCTION_POINTER,
        core::ptr::addr_of_mut!(AcceptExGUID) as PVOID,
        core::mem::size_of::<GUID>() as DWORD,
        core::ptr::addr_of_mut!(pfnAcceptEx) as PVOID,
        core::mem::size_of::<LPFN_ACCEPTEX>() as DWORD,
        core::ptr::addr_of_mut!(cbBytesReturned),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == SOCKET_ERROR
    {
        return FALSE as INT;
    }

    if WSAIoctl(
        ListenSocket,
        SIO_GET_EXTENSION_FUNCTION_POINTER,
        core::ptr::addr_of_mut!(GetAcceptExSockaddrsGUID) as PVOID,
        core::mem::size_of::<GUID>() as DWORD,
        core::ptr::addr_of_mut!(pfnGetAcceptExSockaddrs) as PVOID,
        core::mem::size_of::<LPFN_GETACCEPTEXSOCKADDRS>() as DWORD,
        core::ptr::addr_of_mut!(cbBytesReturned),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == SOCKET_ERROR
    {
        pfnAcceptEx = None;
        return FALSE as INT;
    }

    if let Some(pfn) = pfnAcceptEx {
        pfn(
            ListenSocket,
            AcceptSocket,
            OutputBuffer,
            ReceiveDataLength,
            LocalAddressLength,
            RemoteAddressLength,
            BytesReceived,
            Overlapped,
        )
    } else {
        FALSE as INT
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetAcceptExSockaddrs(
    OutputBuffer: PVOID,
    ReceiveDataLength: DWORD,
    LocalAddressLength: DWORD,
    RemoteAddressLength: DWORD,
    LocalSockaddr: *mut LPSOCKADDR,
    LocalSockaddrLength: LPINT,
    RemoteSockaddr: *mut LPSOCKADDR,
    RemoteSockaddrLength: LPINT,
) {
    if let Some(pfn) = pfnGetAcceptExSockaddrs {
        pfn(
            OutputBuffer,
            ReceiveDataLength,
            LocalAddressLength,
            RemoteAddressLength,
            LocalSockaddr,
            LocalSockaddrLength,
            RemoteSockaddr,
            RemoteSockaddrLength,
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS WinSock DLL
//  * FILE:            stubs.c
//  * PURPOSE:         WSAIoctl wrappers for Microsoft extensions to Winsock
//  * PROGRAMMERS:     KJK::Hyperion <hackbunny@reactos.com>
//  * REVISIONS:
//  */
//
// #include "precomp.h"
//
// #include <winsock2.h>
// #include <mswsock.h>
//
// LPFN_TRANSMITFILE pfnTransmitFile = NULL;
// LPFN_GETACCEPTEXSOCKADDRS pfnGetAcceptExSockaddrs = NULL;
// LPFN_ACCEPTEX pfnAcceptEx = NULL;
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// TransmitFile(SOCKET Socket,
//              HANDLE File,
//              DWORD NumberOfBytesToWrite,
//              DWORD NumberOfBytesPerSend,
//              LPOVERLAPPED Overlapped,
//              LPTRANSMIT_FILE_BUFFERS TransmitBuffers,
//              DWORD Flags)
// {
//     GUID TransmitFileGUID = WSAID_TRANSMITFILE;
//     DWORD cbBytesReturned;
//
//     if (WSAIoctl(Socket,
//                  SIO_GET_EXTENSION_FUNCTION_POINTER,
//                  &TransmitFileGUID,
//                  sizeof(TransmitFileGUID),
//                  &pfnTransmitFile,
//                  sizeof(pfnTransmitFile),
//                  &cbBytesReturned,
//                  NULL,
//                  NULL) == SOCKET_ERROR)
//     {
//         return FALSE;
//     }
//
//     return pfnTransmitFile(Socket,
//                            File,
//                            NumberOfBytesToWrite,
//                            NumberOfBytesPerSend,
//                            Overlapped,
//                            TransmitBuffers,
//                            Flags);
// }
//
// /*
// * @implemented
// */
// BOOL
// WINAPI
// AcceptEx(SOCKET ListenSocket,
//          SOCKET AcceptSocket,
//          PVOID OutputBuffer,
//          DWORD ReceiveDataLength,
//          DWORD LocalAddressLength,
//          DWORD RemoteAddressLength,
//          LPDWORD BytesReceived,
//          LPOVERLAPPED Overlapped)
// {
//     GUID AcceptExGUID = WSAID_ACCEPTEX;
//     GUID GetAcceptExSockaddrsGUID = WSAID_GETACCEPTEXSOCKADDRS;
//     DWORD cbBytesReturned;
//
//     if (WSAIoctl(ListenSocket,
//                  SIO_GET_EXTENSION_FUNCTION_POINTER,
//                  &AcceptExGUID,
//                  sizeof(AcceptExGUID),
//                  &pfnAcceptEx,
//                  sizeof(pfnAcceptEx),
//                  &cbBytesReturned,
//                  NULL,
//                  NULL) == SOCKET_ERROR)
//     {
//         return FALSE;
//     }
//
//     if (WSAIoctl(ListenSocket,
//                  SIO_GET_EXTENSION_FUNCTION_POINTER,
//                  &GetAcceptExSockaddrsGUID,
//                  sizeof(GetAcceptExSockaddrsGUID),
//                  &pfnGetAcceptExSockaddrs,
//                  sizeof(pfnGetAcceptExSockaddrs),
//                  &cbBytesReturned,
//                  NULL,
//                  NULL) == SOCKET_ERROR)
//     {
//         pfnAcceptEx = NULL;
//         return FALSE;
//     }
//
//     return pfnAcceptEx(ListenSocket,
//                        AcceptSocket,
//                        OutputBuffer,
//                        ReceiveDataLength,
//                        LocalAddressLength,
//                        RemoteAddressLength,
//                        BytesReceived,
//                        Overlapped);
// }
//
//
// /*
// * @implemented
// */
// VOID
// WINAPI
// GetAcceptExSockaddrs(PVOID OutputBuffer,
//                      DWORD ReceiveDataLength,
//                      DWORD LocalAddressLength,
//                      DWORD RemoteAddressLength,
//                      LPSOCKADDR* LocalSockaddr,
//                      LPINT LocalSockaddrLength,
//                      LPSOCKADDR* RemoteSockaddr,
//                      LPINT RemoteSockaddrLength)
// {
//     if (pfnGetAcceptExSockaddrs)
//     {
//         pfnGetAcceptExSockaddrs(OutputBuffer,
//                                 ReceiveDataLength,
//                                 LocalAddressLength,
//                                 RemoteAddressLength,
//                                 LocalSockaddr,
//                                 LocalSockaddrLength,
//                                 RemoteSockaddr,
//                                 RemoteSockaddrLength);
//     }
// }
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

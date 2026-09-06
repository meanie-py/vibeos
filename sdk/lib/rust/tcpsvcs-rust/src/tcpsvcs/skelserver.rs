use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const BUF: i32 = 1024;

/// skelserver.c `static SOCKET SetUpListener(USHORT Port)`.
/// NOTE this function was MISSING from the machine translation (its only caller, StartServer,
/// referenced it); translated here from the C preserved verbatim at the foot of this file.
unsafe extern "C" fn SetUpListener(Port: USHORT) -> SOCKET {
    let mut bSetup: BOOL = 0;

    let sock: SOCKET = socket(AF_INET as c_int, SOCK_STREAM as c_int, 0);
    if sock != INVALID_SOCKET {
        let mut server: SOCKADDR_IN = core::mem::zeroed();
        server.sin_family = AF_INET as c_short;
        server.sin_addr.S_un.S_addr = htonl(INADDR_ANY);
        server.sin_port = Port;

        if bind(
            sock,
            core::ptr::addr_of!(server) as *const SOCKADDR,
            core::mem::size_of::<SOCKADDR_IN>() as c_int,
        ) != SOCKET_ERROR
        {
            if listen(sock, SOMAXCONN as c_int) != SOCKET_ERROR {
                bSetup = 1;
            } else {
                LogEvent(
                    wide!("listen() failed").as_ptr(),
                    WSAGetLastError() as DWORD,
                    0,
                    LOG_ERROR,
                );
            }
        } else {
            LogEvent(
                wide!("bind() failed").as_ptr(),
                WSAGetLastError() as DWORD,
                0,
                LOG_ERROR,
            );
        }
    } else {
        LogEvent(
            wide!("socket() failed").as_ptr(),
            WSAGetLastError() as DWORD,
            0,
            LOG_ERROR,
        );
    }

    if bSetup != 0 {
        sock
    } else {
        INVALID_SOCKET
    }
}

unsafe extern "C" fn AcceptConnections(
    listeningSocket: SOCKET,
    lpService: LPTHREAD_START_ROUTINE_SYS,
    lpName: LPWSTR,
) {
    const FMT_ACCEPTED: &[u16] = &[
        0x0041, 0x0063, 0x0063, 0x0065, 0x0070, 0x0074, 0x0065, 0x0064, 0x0020,
        0x0063, 0x006f, 0x006e, 0x006e, 0x0065, 0x0063, 0x0074, 0x0069, 0x006f, 0x006e, 0x0020,
        0x0074, 0x006f, 0x0020,
        0x0025, 0x0073, 0x0020,
        0x0073, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0020,
        0x0066, 0x0072, 0x006f, 0x006d, 0x0020,
        0x0025, 0x0053, 0x003a, 0x0025, 0x0064, 0x0000,
    ];
    const FMT_CREATING: &[u16] = &[
        0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0069, 0x006e, 0x0067, 0x0020,
        0x0077, 0x006f, 0x0072, 0x006b, 0x0065, 0x0072, 0x0020, 0x0074, 0x0068, 0x0072,
        0x0065, 0x0061, 0x0064, 0x0020, 0x0066, 0x006f, 0x0072, 0x0020, 0x0025, 0x0073, 0x0000,
    ];
    const FMT_FAILED: &[u16] = &[
        0x0046, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x0020, 0x0074, 0x006f, 0x0020,
        0x0073, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x0077, 0x006f, 0x0072, 0x006b,
        0x0065, 0x0072, 0x0020, 0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020,
        0x0066, 0x006f, 0x0072, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0025, 0x0073,
        0x0020, 0x0073, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0000,
    ];
    const FMT_ACCEPT_FAILED: &[u16] = &[
        0x0061, 0x0063, 0x0063, 0x0065, 0x0070, 0x0074, 0x0020, 0x0066, 0x0061, 0x0069,
        0x006c, 0x0065, 0x0064, 0x0000,
    ];
    const FMT_SELECT_FAILED: &[u16] = &[
        0x0073, 0x0065, 0x006c, 0x0065, 0x0063, 0x0074, 0x0020, 0x0066, 0x0061, 0x0069,
        0x006c, 0x0065, 0x0064, 0x0000,
    ];

    let mut client: SOCKADDR_IN = core::mem::zeroed();
    let mut logBuf: [WCHAR; 256] = [0; 256];
    let timeOut: INT = 2000;

    let mut timeVal: TIMEVAL = core::mem::zeroed();
    timeVal.tv_sec = (timeOut / 1000) as _;
    timeVal.tv_usec = (timeOut % 1000) as _;

    while bShutdown!() == 0 {
        let mut readFD: FD_SET = core::mem::zeroed();
        FD_ZERO(&raw mut readFD);
        FD_SET(listeningSocket, &raw mut readFD);

        let selRet: INT = select(
            0i32,
            &raw mut readFD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw const timeVal,
        );

        if selRet > 0 {
            if bShutdown!() == 0 || FD_ISSET(listeningSocket, &raw const readFD) != 0 {
                let mut addrSize: INT = core::mem::size_of::<SOCKADDR_IN>() as INT;
                let sock: SOCKET = accept(
                    listeningSocket,
                    core::ptr::addr_of_mut!(client) as *mut SOCKADDR,
                    &raw mut addrSize,
                );
                if sock != INVALID_SOCKET {
                    swprintf(
                        logBuf.as_mut_ptr(),
                        FMT_ACCEPTED.as_ptr(),
                        lpName,
                        inet_ntoa(client.sin_addr),
                        ntohs(client.sin_port) as i32,
                    );
                    LogEvent(logBuf.as_ptr(), 0, 0, LOG_FILE);

                    swprintf(
                        logBuf.as_mut_ptr(),
                        FMT_CREATING.as_ptr(),
                        lpName,
                    );
                    LogEvent(logBuf.as_ptr(), 0, 0, LOG_FILE);

                    if bShutdown!() == 0 {
                        let hThread: HANDLE = CreateThread(
                            core::ptr::null_mut(),
                            0,
                            lpService,
                            sock as LPVOID,
                            0,
                            core::ptr::null_mut(),
                        );
                        if !hThread.is_null() {
                            CloseHandle(hThread);
                        } else {
                            swprintf(
                                logBuf.as_mut_ptr(),
                                FMT_FAILED.as_ptr(),
                                lpName,
                            );
                            LogEvent(logBuf.as_ptr(), 0, 0, LOG_FILE);
                        }
                    }
                } else {
                    LogEvent(
                        FMT_ACCEPT_FAILED.as_ptr(),
                        WSAGetLastError() as u32,
                        0,
                        LOG_ERROR,
                    );
                }
            }
        } else if selRet == SOCKET_ERROR {
            LogEvent(
                FMT_SELECT_FAILED.as_ptr(),
                WSAGetLastError() as u32,
                0,
                LOG_ERROR,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ShutdownConnection(sock: SOCKET, bRec: BOOL) -> BOOL {
    let mut log_buf: [u16; 256] = [0u16; 256];
    let err_msg = wide!("Error in shutdown()").as_ptr();

    if shutdown(sock, SD_SEND as c_int) == SOCKET_ERROR {
        LogEvent(err_msg, WSAGetLastError() as DWORD, 0, LOG_ERROR);
        return 0;
    }

    if bRec != 0 {
        let mut read_buffer: [CHAR; BUF as usize] = [0 as CHAR; BUF as usize];
        let fmt = wide!("FYI, received %d unexpected bytes during shutdown").as_ptr();

        loop {
            let ret = recv(sock, read_buffer.as_mut_ptr(), BUF as i32, 0);
            if ret >= 0 {
                swprintf(log_buf.as_mut_ptr(), fmt, ret);
                LogEvent(log_buf.as_mut_ptr(), 0, 0, LOG_FILE);
            }
            if ret <= 0 {
                break;
            }
        }
    }

    closesocket(sock);
    1
}

#[no_mangle]
pub unsafe extern "system" fn StartServer(lpParam: LPVOID) -> u32 {
    let p_services = lpParam as *mut SERVICES;
    let mut log_buf: [u16; 256] = [0u16; 256];

    // L"Starting %s server"
    static STARTING_FMT: [u16; 19] = [
        'S' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16, 'i' as u16, 'n' as u16,
        'g' as u16, ' ' as u16, '%' as u16, 's' as u16, ' ' as u16, 's' as u16, 'e' as u16,
        'r' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 0u16,
    ];
    swprintf(
        log_buf.as_mut_ptr(),
        STARTING_FMT.as_ptr(),
        (*p_services).lpName,
    );
    LogEvent(log_buf.as_ptr(), 0, 0, LOG_FILE);

    if bShutdown!() == 0 {
        let listening_socket = SetUpListener(htons((*p_services).Port));
        if bShutdown!() == 0 && listening_socket != INVALID_SOCKET {
            // L"%s is waiting for connections on port %d"
            static WAITING_FMT: [u16; 41] = [
                '%' as u16, 's' as u16, ' ' as u16, 'i' as u16, 's' as u16, ' ' as u16,
                'w' as u16, 'a' as u16, 'i' as u16, 't' as u16, 'i' as u16, 'n' as u16,
                'g' as u16, ' ' as u16, 'f' as u16, 'o' as u16, 'r' as u16, ' ' as u16,
                'c' as u16, 'o' as u16, 'n' as u16, 'n' as u16, 'e' as u16, 'c' as u16,
                't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 's' as u16, ' ' as u16,
                'o' as u16, 'n' as u16, ' ' as u16, 'p' as u16, 'o' as u16, 'r' as u16,
                't' as u16, ' ' as u16, '%' as u16, 'd' as u16, 0u16,
            ];
            swprintf(
                log_buf.as_mut_ptr(),
                WAITING_FMT.as_ptr(),
                (*p_services).lpName,
                (*p_services).Port as u32,
            );
            LogEvent(log_buf.as_ptr(), 0, 0, LOG_FILE);

            AcceptConnections(listening_socket, (*p_services).lpService, (*p_services).lpName);
        } else {
            // L"Socket error when setting up listener"
            static ERR_FMT: [u16; 38] = [
                'S' as u16, 'o' as u16, 'c' as u16, 'k' as u16, 'e' as u16, 't' as u16,
                ' ' as u16, 'e' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16,
                ' ' as u16, 'w' as u16, 'h' as u16, 'e' as u16, 'n' as u16, ' ' as u16,
                's' as u16, 'e' as u16, 't' as u16, 't' as u16, 'i' as u16, 'n' as u16,
                'g' as u16, ' ' as u16, 'u' as u16, 'p' as u16, ' ' as u16, 'l' as u16,
                'i' as u16, 's' as u16, 't' as u16, 'e' as u16, 'n' as u16, 'e' as u16,
                'r' as u16, 0u16,
            ];
            LogEvent(ERR_FMT.as_ptr(), 0, 0, LOG_FILE);
        }
    }

    // L"Exiting %s thread"
    static EXITING_FMT: [u16; 18] = [
        'E' as u16, 'x' as u16, 'i' as u16, 't' as u16, 'i' as u16, 'n' as u16, 'g' as u16,
        ' ' as u16, '%' as u16, 's' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'r' as u16,
        'e' as u16, 'a' as u16, 'd' as u16, 0u16,
    ];
    swprintf(
        log_buf.as_mut_ptr(),
        EXITING_FMT.as_ptr(),
        (*p_services).lpName,
    );
    LogEvent(log_buf.as_ptr(), 0, 0, LOG_FILE);
    ExitThread(0);
    0
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/skelserver.c
//  * PURPOSE:     Sets up a server and listens for connections
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@gmail.com>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #define BUF 1024
//
// static SOCKET
// SetUpListener(USHORT Port)
// {
//     SOCKET sock;
//     SOCKADDR_IN server;
//     BOOL bSetup = FALSE;
//
//     sock = socket(AF_INET, SOCK_STREAM, 0);
//     if (sock != INVALID_SOCKET)
//     {
//         ZeroMemory(&server, sizeof(SOCKADDR_IN));
//         server.sin_family = AF_INET;
//         server.sin_addr.s_addr = htonl(INADDR_ANY);
//         server.sin_port = Port;
//
//         if (bind(sock, (SOCKADDR*)&server, sizeof(SOCKADDR_IN)) != SOCKET_ERROR)
//         {
//             if (listen(sock, SOMAXCONN) != SOCKET_ERROR)
//             {
//                 bSetup = TRUE;
//             }
//             else
//             {
//                 LogEvent(L"listen() failed", WSAGetLastError(), 0, LOG_ERROR);
//             }
//         }
//         else
//         {
//             LogEvent(L"bind() failed", WSAGetLastError(), 0, LOG_ERROR);
//         }
//     }
//     else
//     {
//         LogEvent(L"socket() failed", WSAGetLastError(), 0, LOG_ERROR);
//     }
//
//     return bSetup ? sock : INVALID_SOCKET;
// }
//
//
// static VOID
// AcceptConnections(SOCKET listeningSocket,
//                   LPTHREAD_START_ROUTINE lpService,
//                   LPWSTR lpName)
// {
//     SOCKADDR_IN client;
//     SOCKET sock;
//     HANDLE hThread;
//     TIMEVAL timeVal;
//     FD_SET readFD;
//     WCHAR logBuf[256];
//     INT timeOut = 2000;
//
//     timeVal.tv_sec  = timeOut / 1000;
//     timeVal.tv_usec = timeOut % 1000;
//
//     while (!bShutdown)
//     {
//         INT selRet = 0;
//
//         FD_ZERO(&readFD);
//         FD_SET(listeningSocket, &readFD);
//
//         selRet = select(0, &readFD, NULL, NULL, &timeVal);
//         if (selRet > 0)
//         {
//             if (!bShutdown || FD_ISSET(listeningSocket, &readFD))
//             {
//                 INT addrSize = sizeof(SOCKADDR_IN);
//
//                 sock = accept(listeningSocket, (SOCKADDR*)&client, &addrSize);
//                 if (sock != INVALID_SOCKET)
//                 {
//                     swprintf(logBuf,
//                              L"Accepted connection to %s server from %S:%d",
//                              lpName,
//                              inet_ntoa(client.sin_addr),
//                              ntohs(client.sin_port));
//                     LogEvent(logBuf, 0, 0, LOG_FILE);
//
//                     swprintf(logBuf, L"Creating worker thread for %s", lpName);
//                     LogEvent(logBuf, 0, 0, LOG_FILE);
//
//                     if (!bShutdown)
//                     {
//                         hThread = CreateThread(0, 0, lpService, (void*)sock, 0, NULL);
//                         if (hThread != NULL)
//                         {
//                             CloseHandle(hThread);
//                         }
//                         else
//                         {
//                             swprintf(logBuf, L"Failed to start worker thread for the %s server",
//                                      lpName);
//                             LogEvent(logBuf, 0, 0, LOG_FILE);
//                         }
//                     }
//                 }
//                 else
//                 {
//                     LogEvent(L"accept failed", WSAGetLastError(), 0, LOG_ERROR);
//                 }
//             }
//         }
//         else if (selRet == SOCKET_ERROR)
//         {
//             LogEvent(L"select failed", WSAGetLastError(), 0, LOG_ERROR);
//         }
//     }
// }
//
// BOOL
// ShutdownConnection(SOCKET sock,
//                    BOOL bRec)
// {
//     WCHAR logBuf[256];
//
//     /* Disallow any further data sends.  This will tell the other side
//        that we want to go away now.  If we skip this step, we don't
//        shut the connection down nicely. */
//     if (shutdown(sock, SD_SEND) == SOCKET_ERROR)
//     {
//         LogEvent(L"Error in shutdown()", WSAGetLastError(), 0, LOG_ERROR);
//         return FALSE;
//     }
//
//       /* Receive any extra data still sitting on the socket
//          before we close it */
//     if (bRec)
//     {
//         CHAR readBuffer[BUF];
//         INT ret;
//
//         do
//         {
//             ret = recv(sock, readBuffer, BUF, 0);
//             if (ret >= 0)
//             {
//                 swprintf(logBuf, L"FYI, received %d unexpected bytes during shutdown", ret);
//                 LogEvent(logBuf, 0, 0, LOG_FILE);
//             }
//         } while (ret > 0);
//     }
//
//     closesocket(sock);
//
//     return TRUE;
// }
//
//
// DWORD WINAPI
// StartServer(LPVOID lpParam)
// {
//     SOCKET listeningSocket;
//     PSERVICES pServices;
//     TCHAR logBuf[256];
//
//     pServices = (PSERVICES)lpParam;
//
//     swprintf(logBuf, L"Starting %s server", pServices->lpName);
//     LogEvent(logBuf, 0, 0, LOG_FILE);
//
//     if (!bShutdown)
//     {
//         listeningSocket = SetUpListener(htons(pServices->Port));
//         if (!bShutdown && listeningSocket != INVALID_SOCKET)
//         {
//             swprintf(logBuf,
//                      L"%s is waiting for connections on port %d",
//                      pServices->lpName,
//                      pServices->Port);
//             LogEvent(logBuf, 0, 0, LOG_FILE);
//
//             AcceptConnections(listeningSocket, pServices->lpService, pServices->lpName);
//         }
//         else
//         {
//             LogEvent(L"Socket error when setting up listener", 0, 0, LOG_FILE);
//         }
//     }
//
//     swprintf(logBuf, L"Exiting %s thread", pServices->lpName);
//     LogEvent(logBuf, 0, 0, LOG_FILE);
//     ExitThread(0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static ServiceName: [u16; 8] = [
    0x0074, 0x0063, 0x0070, 0x0073, 0x0076, 0x0063, 0x0073, 0x0000,
];
pub static mut bShutdown: BOOL = 0;
pub static mut bPause: BOOL = 0;

#[repr(C)]
pub struct _ServiceInfo {
    pub servStatus: SERVICE_STATUS,
    pub hStatus: SERVICE_STATUS_HANDLE,
}
pub type SERVICEINFO = _ServiceInfo;
pub type PSERVICEINFO = *mut _ServiceInfo;

const _ECHO_SVC_NAME: [u16; 5] = [0x0045, 0x0063, 0x0068, 0x006F, 0x0000];
const _DISCARD_SVC_NAME: [u16; 8] = [
    0x0044, 0x0069, 0x0073, 0x0063, 0x0061, 0x0072, 0x0064, 0x0000,
];
const _DAYTIME_SVC_NAME: [u16; 8] = [
    0x0044, 0x0061, 0x0079, 0x0074, 0x0069, 0x006D, 0x0065, 0x0000,
];
const _QOTD_SVC_NAME: [u16; 5] = [0x0051, 0x004F, 0x0054, 0x0044, 0x0000];
const _CHARGEN_SVC_NAME: [u16; 8] = [
    0x0043, 0x0068, 0x0061, 0x0072, 0x0067, 0x0065, 0x006E, 0x0000,
];

pub static mut Services: [SERVICES; NUM_SERVICES as usize] = [
    SERVICES {
        Port: ECHO_PORT as USHORT,
        lpName: _ECHO_SVC_NAME.as_ptr() as LPWSTR,
        lpService: Some(EchoHandler),
    },
    SERVICES {
        Port: DISCARD_PORT as USHORT,
        lpName: _DISCARD_SVC_NAME.as_ptr() as LPWSTR,
        lpService: Some(DiscardHandler),
    },
    SERVICES {
        Port: DAYTIME_PORT as USHORT,
        lpName: _DAYTIME_SVC_NAME.as_ptr() as LPWSTR,
        lpService: Some(DaytimeHandler),
    },
    SERVICES {
        Port: QOTD_PORT as USHORT,
        lpName: _QOTD_SVC_NAME.as_ptr() as LPWSTR,
        lpService: Some(QotdHandler),
    },
    SERVICES {
        Port: CHARGEN_PORT as USHORT,
        lpName: _CHARGEN_SVC_NAME.as_ptr() as LPWSTR,
        lpService: Some(ChargenHandler),
    },
];

const FMT: [u16; 36] = [
    83u16, 101, 114, 118, 105, 99, 101, 32, 115, 116, 97, 116, 101, 32, 48, 120, 37, 108, 120, 44,
    32, 67, 104, 101, 99, 107, 80, 111, 105, 110, 116, 32, 37, 108, 117, 0u16,
];

const ERR_STR: [u16; 26] = [
    67u16, 97, 110, 110, 111, 116, 32, 115, 101, 116, 32, 115, 101, 114, 118, 105, 99, 101, 32,
    115, 116, 97, 116, 117, 115, 0u16,
];
unsafe extern "C" fn UpdateStatus(pServInfo: PSERVICEINFO, NewStatus: u32, Check: u32) {
    if Check > 0 {
        (*pServInfo).servStatus.dwCheckPoint += Check;
    } else {
        (*pServInfo).servStatus.dwCheckPoint = Check;
    }
    if NewStatus > 0 {
        (*pServInfo).servStatus.dwCurrentState = NewStatus;
    }
    let mut sz_set: [u16; 50] = [0u16; 50];
    _snwprintf(
        sz_set.as_mut_ptr(),
        49usize,
        FMT.as_ptr() as *const u16,
        (*pServInfo).servStatus.dwCurrentState,
        (*pServInfo).servStatus.dwCheckPoint,
    );
    LogEvent(sz_set.as_ptr() as *const u16, 0u32, 0u32, LOG_FILE);
    if SetServiceStatus(
        (*pServInfo).hStatus,
        core::ptr::addr_of_mut!((*pServInfo).servStatus),
    ) == 0
    {
        LogEvent(
            ERR_STR.as_ptr() as *const u16,
            GetLastError(),
            0u32,
            LOG_ALL,
        );
    }
}
unsafe extern "C" fn CreateServers(pServInfo: PSERVICEINFO) -> u8 {
    const FMT_WSA_FAILED: [u16; 27] = [
        87, 83, 65, 83, 116, 97, 114, 116, 117, 112, 40, 41, 32, 102, 97, 105, 108, 101, 100, 32,
        58, 32, 37, 108, 117, 10, 0,
    ];
    const MSG_CREATE_SERVERS: [u16; 25] = [
        10, 67, 114, 101, 97, 116, 105, 110, 103, 32, 115, 101, 114, 118, 101, 114, 32, 84, 104,
        114, 101, 97, 100, 115, 0,
    ];
    const FMT_CREATE_THREAD: [u16; 30] = [
        67, 114, 101, 97, 116, 105, 110, 103, 32, 116, 104, 114, 101, 97, 100, 32, 102, 111, 114,
        32, 37, 115, 32, 115, 101, 114, 118, 101, 114, 0,
    ];
    const FMT_ERROR_CREATE: [u16; 34] = [
        10, 69, 114, 114, 111, 114, 32, 99, 114, 101, 97, 116, 105, 110, 103, 32, 37, 115, 32,
        115, 101, 114, 118, 101, 114, 32, 116, 104, 114, 101, 97, 100, 10, 0,
    ];
    const MSG_SET_RUNNING: [u16; 34] = [
        83, 101, 116, 116, 105, 110, 103, 32, 115, 101, 114, 118, 105, 99, 101, 32, 115, 116, 97,
        116, 117, 115, 32, 116, 111, 32, 114, 117, 110, 110, 105, 110, 103, 0,
    ];
    const MSG_DETACH_WINSOCK: [u16; 19] = [
        68, 101, 116, 97, 99, 104, 105, 110, 103, 32, 87, 105, 110, 115, 111, 99, 107, 50, 0,
    ];

    let mut wsa_data: WSADATA = core::mem::zeroed();
    let mut buf = [0u16; 256];
    let w_version_requested: u16 = MAKEWORD(2, 2);
    let ret_val = WSAStartup(w_version_requested, &mut wsa_data as *mut WSADATA);

    if ret_val != 0 {
        swprintf(
            buf.as_mut_ptr(),
            FMT_WSA_FAILED.as_ptr(),
            ret_val,
        );
        LogEvent(buf.as_ptr(), 0, 100, LOG_ALL);
        return 0u8;
    }

    UpdateStatus(pServInfo, 0, 1);

    LogEvent(MSG_CREATE_SERVERS.as_ptr(), 0, 0, LOG_FILE);

    let mut h_thread: [HANDLE; NUM_SERVICES as usize] = [0 as HANDLE; NUM_SERVICES as usize];
    let mut dw_thread_id: [u32; NUM_SERVICES as usize] = [0u32; NUM_SERVICES as usize];

    for i in 0..NUM_SERVICES as usize {
        swprintf(
            buf.as_mut_ptr(),
            FMT_CREATE_THREAD.as_ptr(),
            Services[i].lpName,
        );
        LogEvent(buf.as_ptr(), 0, 0, LOG_FILE);

        h_thread[i] = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(StartServer),
            &Services[i] as *const SERVICES as *mut core::ffi::c_void,
            0,
            &mut dw_thread_id[i],
        );

        if h_thread[i] == 0 as HANDLE {
            swprintf(
                buf.as_mut_ptr(),
                FMT_ERROR_CREATE.as_ptr(),
                Services[i].lpName,
            );
            LogEvent(buf.as_ptr(), GetLastError(), 0, LOG_ALL);
            return 0u8;
        }

        UpdateStatus(pServInfo, 0, 1);
    }

    LogEvent(MSG_SET_RUNNING.as_ptr(), 0, 0, LOG_FILE);
    UpdateStatus(pServInfo, SERVICE_RUNNING, 0);

    WaitForMultipleObjects(
        NUM_SERVICES as u32,
        h_thread.as_mut_ptr(),
        1,
        INFINITE,
    );

    for i in 0..NUM_SERVICES as usize {
        if h_thread[i] != 0 as HANDLE {
            CloseHandle(h_thread[i]);
        }
    }

    LogEvent(MSG_DETACH_WINSOCK.as_ptr(), 0, 0, LOG_FILE);
    WSACleanup();

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn ServerCtrlHandler(
    dwControl: u32,
    _dwEventType: u32,
    _lpEventData: LPVOID,
    lpContext: LPVOID,
) -> DWORD {
    let pServInfo: PSERVICEINFO = lpContext as PSERVICEINFO;

    match dwControl {
        SERVICE_CONTROL_SHUTDOWN | SERVICE_CONTROL_STOP => {
            LogEvent(
                [
                    10u16, 83, 101, 116, 116, 105, 110, 103, 32, 116, 104, 101, 32, 115, 101, 114,
                    118, 105, 99, 101, 32, 116, 111, 32, 83, 69, 82, 86, 73, 67, 69, 95, 83, 84,
                    79, 80, 95, 80, 69, 78, 68, 73, 78, 71, 0u16,
                ]
                .as_ptr(),
                0,
                0,
                LOG_FILE,
            );
            InterlockedExchange(
                core::ptr::addr_of_mut!(bShutdown) as *mut i32,
                1,
            );
            (*pServInfo).servStatus.dwWin32ExitCode = 0;
            (*pServInfo).servStatus.dwWaitHint = 5000;
            UpdateStatus(pServInfo, SERVICE_STOP_PENDING, 1);
        }
        SERVICE_CONTROL_PAUSE => {
            LogEvent(
                [
                    83u16, 101, 116, 116, 105, 110, 103, 32, 116, 104, 101, 32, 115, 101, 114, 118,
                    105, 99, 101, 32, 116, 111, 32, 83, 69, 82, 86, 73, 67, 69, 95, 80, 65, 85, 83,
                    69, 68, 0u16,
                ]
                .as_ptr(),
                0,
                0,
                LOG_FILE,
            );
            InterlockedExchange(
                core::ptr::addr_of_mut!(bPause) as *mut i32,
                1,
            );
            UpdateStatus(pServInfo, SERVICE_PAUSED, 0);
        }
        SERVICE_CONTROL_CONTINUE => {
            LogEvent(
                [
                    83u16, 101, 116, 116, 105, 110, 103, 32, 116, 104, 101, 32, 115, 101, 114, 118,
                    105, 99, 101, 32, 116, 111, 32, 83, 69, 82, 86, 73, 67, 69, 95, 82, 85, 78, 78,
                    73, 78, 71, 0u16,
                ]
                .as_ptr(),
                0,
                0,
                LOG_FILE,
            );
            InterlockedExchange(
                core::ptr::addr_of_mut!(bPause) as *mut i32,
                0,
            );
            UpdateStatus(pServInfo, SERVICE_RUNNING, 0);
        }
        SERVICE_CONTROL_INTERROGATE => {
            SetServiceStatus(
                (*pServInfo).hStatus,
                core::ptr::addr_of_mut!((*pServInfo).servStatus),
            );
        }
        other => {
            if other > 127 && other < 256 {
                LogEvent(
                    [
                        85u16, 115, 101, 114, 32, 100, 101, 102, 105, 110, 101, 100, 32, 99, 111,
                        110, 116, 114, 111, 108, 32, 99, 111, 100, 101, 0u16,
                    ]
                    .as_ptr(),
                    0,
                    0,
                    LOG_FILE,
                );
            } else {
                LogEvent(
                    [
                        69u16, 82, 82, 79, 82, 58, 32, 66, 97, 100, 32, 99, 111, 110, 116, 114,
                        111, 108, 32, 99, 111, 100, 101, 0u16,
                    ]
                    .as_ptr(),
                    0,
                    0,
                    LOG_FILE,
                );
            }
        }
    }
    NO_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn ServiceMain(argc: u32, argv: *mut *mut u16) {
    let mut servInfo: SERVICEINFO = core::mem::zeroed();

    // "Entering ServiceMain."
    let enter_str: [u16; 22] = [
        69, 110, 116, 101, 114, 105, 110, 103, 32, 83, 101, 114, 118, 105, 99, 101, 77, 97, 105, 110,
        46, 0,
    ];
    LogEvent(enter_str.as_ptr(), 0, 0, LOG_FILE);

    servInfo.servStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    servInfo.servStatus.dwCurrentState = SERVICE_STOPPED;
    servInfo.servStatus.dwControlsAccepted =
        SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN | SERVICE_ACCEPT_PAUSE_CONTINUE;
    servInfo.servStatus.dwWin32ExitCode = ERROR_SERVICE_SPECIFIC_ERROR;
    servInfo.servStatus.dwServiceSpecificExitCode = 0;
    servInfo.servStatus.dwCheckPoint = 0;
    servInfo.servStatus.dwWaitHint = 2 * CS_TIMEOUT;

    // "Registering service control handler"
    let reg_str: [u16; 36] = [
        82, 101, 103, 105, 115, 116, 101, 114, 105, 110, 103, 32, 115, 101, 114, 118, 105, 99, 101,
        32, 99, 111, 110, 116, 114, 111, 108, 32, 104, 97, 110, 100, 108, 101, 114, 0,
    ];
    LogEvent(reg_str.as_ptr(), 0, 0, LOG_FILE);

    servInfo.hStatus = RegisterServiceCtrlHandlerExW(
        ServiceName.as_ptr(),
        Some(ServerCtrlHandler),
        &mut servInfo as *mut SERVICEINFO as LPVOID,
    );

    if servInfo.hStatus.is_null() {
                LogEvent(wide!("Failed to register service").as_ptr(), GetLastError(), 100, LOG_ALL);
    }

    UpdateStatus(&mut servInfo, SERVICE_START_PENDING, 1);

    if CreateServers(&mut servInfo) == 0 {
        let err_str: [u16; 23] = [
            69, 114, 114, 111, 114, 32, 99, 114, 101, 97, 116, 105, 110, 103, 32, 115, 101, 114,
            118, 101, 114, 115, 0,
        ];
        LogEvent(err_str.as_ptr(), GetLastError(), 1, LOG_ALL);
        servInfo.servStatus.dwServiceSpecificExitCode = 1;
        UpdateStatus(&mut servInfo, SERVICE_STOPPED, 0);
        return;
    }

    LogEvent(
        wide!("Service threads shut down. Set SERVICE_STOPPED status").as_ptr(),
        0,
        0,
        LOG_FILE,
    );
    UpdateStatus(&mut servInfo, SERVICE_STOPPED, 0);

    // "Leaving ServiceMain\n"
    let leave_str: [u16; 21] = [
        76, 101, 97, 118, 105, 110, 103, 32, 83, 101, 114, 118, 105, 99, 101, 77, 97, 105, 110, 10,
        0,
    ];
    LogEvent(leave_str.as_ptr(), 0, 0, LOG_FILE);
}
#[no_mangle]
// `_tmain` is a tchar.h MACRO: with UNICODE (this module) it IS `wmain`. Exporting
// the literal macro name left the CRT startup with no entry point to call.
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut u16) -> i32 {
    let msg = &[
        'f' as u16, 'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16,
        ' ' as u16, 't' as u16, 'o' as u16, ' ' as u16,
        's' as u16, 't' as u16, 'a' as u16, 'r' as u16, 't' as u16,
        ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16,
        ' ' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
        ' ' as u16, 'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
        ' ' as u16, 'd' as u16, 'i' as u16, 's' as u16, 'p' as u16, 'a' as u16, 't' as u16, 'c' as u16, 'h' as u16, 'e' as u16, 'r' as u16,
        0u16
    ];

    let mut service_table = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::addr_of!(ServiceName) as *mut u16,
            lpServiceProc: Some(ServiceMain as unsafe extern "system" fn(u32, *mut *mut u16)),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];

    if InitLogging() != 0 {
        if StartServiceCtrlDispatcherW(service_table.as_mut_ptr()) == 0 {
            LogEvent(msg.as_ptr() as *mut u16, GetLastError(), 101u32, LOG_ALL);
        }
        UninitLogging();
    }

    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS simple TCP/IP services
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/tcpsvcs/tcpsvcs.c
//  * PURPOSE:     Provide CharGen, Daytime, Discard, Echo, and Qotd services
//  * COPYRIGHT:   Copyright 2005 - 2008 Ged Murphy <gedmurphy@reactos.org>
//  *
//  */
//
// #include "tcpsvcs.h"
//
// #include <winsvc.h>
//
// static WCHAR ServiceName[] = L"tcpsvcs";
//
// volatile BOOL bShutdown = FALSE;
// volatile BOOL bPause = FALSE;
//
// typedef struct _ServiceInfo
// {
//     SERVICE_STATUS servStatus;
//     SERVICE_STATUS_HANDLE hStatus;
// } SERVICEINFO, *PSERVICEINFO;
//
// static SERVICES
// Services[NUM_SERVICES] =
// {
//     {ECHO_PORT,    L"Echo",    EchoHandler},
//     {DISCARD_PORT, L"Discard", DiscardHandler},
//     {DAYTIME_PORT, L"Daytime", DaytimeHandler},
//     {QOTD_PORT,    L"QOTD",    QotdHandler},
//     {CHARGEN_PORT, L"Chargen", ChargenHandler}
// };
//
//
// static VOID
// UpdateStatus(PSERVICEINFO pServInfo,
//              DWORD NewStatus,
//              DWORD Check)
// {
//     WCHAR szSet[50];
//
//     if (Check > 0)
//         pServInfo->servStatus.dwCheckPoint += Check;
//     else
//         pServInfo->servStatus.dwCheckPoint = Check;
//
//     if (NewStatus > 0)
//         pServInfo->servStatus.dwCurrentState = NewStatus;
//
//     _snwprintf(szSet,
//                49,
//                L"Service state 0x%lx, CheckPoint %lu",
//                pServInfo->servStatus.dwCurrentState,
//                pServInfo->servStatus.dwCheckPoint);
//     LogEvent(szSet, 0, 0, LOG_FILE);
//
//     if (!SetServiceStatus(pServInfo->hStatus, &pServInfo->servStatus))
//         LogEvent(L"Cannot set service status", GetLastError(), 0, LOG_ALL);
// }
//
//
// static BOOL
// CreateServers(PSERVICEINFO pServInfo)
// {
//     DWORD dwThreadId[NUM_SERVICES];
//     HANDLE hThread[NUM_SERVICES];
//     WSADATA wsaData;
//     WCHAR buf[256];
//     INT i;
//     DWORD RetVal;
//
//     if ((RetVal = WSAStartup(MAKEWORD(2, 2), &wsaData)) != 0)
//     {
//         swprintf(buf, L"WSAStartup() failed : %lu\n", RetVal);
//         LogEvent(buf, 0, 100, LOG_ALL);
//         return FALSE;
//     }
//
//     UpdateStatus(pServInfo, 0, 1);
//
//     LogEvent(L"\nCreating server Threads", 0, 0, LOG_FILE);
//
//     /* Create worker threads. */
//     for (i = 0; i < NUM_SERVICES; i++)
//     {
//         swprintf(buf, L"Creating thread for %s server", Services[i].lpName);
//         LogEvent(buf, 0, 0, LOG_FILE);
//
//         hThread[i] = CreateThread(NULL,
//                                   0,
//                                   StartServer,
//                                   &Services[i],
//                                   0,
//                                   &dwThreadId[i]);
//
//         if (hThread[i] == NULL)
//         {
//             swprintf(buf, L"\nError creating %s server thread\n", Services[i].lpName);
//             LogEvent(buf, GetLastError(), 0, LOG_ALL);
//             return FALSE;
//         }
//
//         UpdateStatus(pServInfo, 0, 1);
//     }
//
//     LogEvent(L"Setting service status to running", 0, 0, LOG_FILE);
//     UpdateStatus(pServInfo, SERVICE_RUNNING, 0);
//
//     /* Wait until all threads have terminated. */
//     WaitForMultipleObjects(NUM_SERVICES, hThread, TRUE, INFINITE);
//
//     for (i = 0; i < NUM_SERVICES; i++)
//     {
//         if (hThread[i] != NULL)
//             CloseHandle(hThread[i]);
//     }
//
//     LogEvent(L"Detaching Winsock2", 0, 0, LOG_FILE);
//     WSACleanup();
//
//     return TRUE;
// }
//
// VOID WINAPI
// ServerCtrlHandler(DWORD dwControl,
//                   DWORD dwEventType,
//                   LPVOID lpEventData,
//                   LPVOID lpContext)
// {
//     PSERVICEINFO pServInfo = (PSERVICEINFO)lpContext;
//
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_SHUTDOWN:
//         case SERVICE_CONTROL_STOP:
//             LogEvent(L"\nSetting the service to SERVICE_STOP_PENDING", 0, 0, LOG_FILE);
//             InterlockedExchange((LONG *)&bShutdown, TRUE);
//             pServInfo->servStatus.dwWin32ExitCode = 0;
//             pServInfo->servStatus.dwWaitHint = 5000;
//             UpdateStatus(pServInfo, SERVICE_STOP_PENDING, 1);
//             break;
//
//         case SERVICE_CONTROL_PAUSE: /* not yet implemented */
//             LogEvent(L"Setting the service to SERVICE_PAUSED", 0, 0, LOG_FILE);
//             InterlockedExchange((LONG *)&bPause, TRUE);
//             UpdateStatus(pServInfo, SERVICE_PAUSED, 0);
//             break;
//
//         case SERVICE_CONTROL_CONTINUE:
//             LogEvent(L"Setting the service to SERVICE_RUNNING", 0, 0, LOG_FILE);
//             InterlockedExchange((LONG *)&bPause, FALSE);
//             UpdateStatus(pServInfo, SERVICE_RUNNING, 0);
//             break;
//
//         case SERVICE_CONTROL_INTERROGATE:
//             SetServiceStatus(pServInfo->hStatus, &pServInfo->servStatus);
//             break;
//
//         default:
//             if (dwControl > 127 && dwControl < 256) /* user defined */
//                 LogEvent(L"User defined control code", 0, 0, LOG_FILE);
//             else
//                 LogEvent(L"ERROR: Bad control code", 0, 0, LOG_FILE);
//             break;
//     }
// }
//
// VOID WINAPI
// ServiceMain(DWORD argc, LPWSTR argv[])
// {
//     SERVICEINFO servInfo;
//
//     LogEvent(L"Entering ServiceMain.", 0, 0, LOG_FILE);
//
//     servInfo.servStatus.dwServiceType      = SERVICE_WIN32_OWN_PROCESS;
//     servInfo.servStatus.dwCurrentState     = SERVICE_STOPPED;
//     servInfo.servStatus.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN | SERVICE_ACCEPT_PAUSE_CONTINUE;
//     servInfo.servStatus.dwWin32ExitCode    = ERROR_SERVICE_SPECIFIC_ERROR;
//     servInfo.servStatus.dwServiceSpecificExitCode = 0;
//     servInfo.servStatus.dwCheckPoint       = 0;
//     servInfo.servStatus.dwWaitHint         = 2 * CS_TIMEOUT;
//
//     LogEvent(L"Registering service control handler", 0, 0, LOG_FILE);
//     servInfo.hStatus = RegisterServiceCtrlHandlerExW(ServiceName,
//                                                      (LPHANDLER_FUNCTION_EX)ServerCtrlHandler,
//                                                      &servInfo);
//     if (!servInfo.hStatus)
//         LogEvent(L"Failed to register service", GetLastError(), 100, LOG_ALL);
//
//     UpdateStatus(&servInfo, SERVICE_START_PENDING, 1);
//
//     if (!CreateServers(&servInfo))
//     {
//         LogEvent(L"Error creating servers", GetLastError(), 1, LOG_ALL);
//         servInfo.servStatus.dwServiceSpecificExitCode = 1;
//         UpdateStatus(&servInfo, SERVICE_STOPPED, 0);
//         return;
//     }
//
//     LogEvent(L"Service threads shut down. Set SERVICE_STOPPED status", 0, 0, LOG_FILE);
//     UpdateStatus(&servInfo, SERVICE_STOPPED, 0);
//
//     LogEvent(L"Leaving ServiceMain\n", 0, 0, LOG_FILE);
// }
//
//
// int _tmain (int argc, LPTSTR argv [])
// {
//     SERVICE_TABLE_ENTRYW ServiceTable[] =
//     {
//         {ServiceName, ServiceMain},
//         {NULL,        NULL }
//     };
//
//     if (InitLogging())
//     {
//         if (!StartServiceCtrlDispatcherW(ServiceTable))
//             LogEvent(L"failed to start the service control dispatcher", GetLastError(), 101, LOG_ALL);
//
//         UninitLogging();
//     }
//
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

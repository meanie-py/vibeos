use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dprocess::WsProcSetVersion;
use crate::src::dprocess::WsProcStartup;
use crate::src::dsocket::WsSockStartup;
use crate::src::dthread::WsThreadStartup;
pub use crate::src::dllmain::WsDllHandle;


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

pub static mut WsSockPostRoutine: PWS_SOCK_POST_ROUTINE = None;

pub static mut WsStartupLock: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};

#[no_mangle]
pub unsafe extern "system" fn WsCreateStartupSynchronization() {
    InitializeCriticalSection((core::ptr::addr_of_mut!(WsStartupLock) as *mut c_void));
}

#[no_mangle]
pub unsafe extern "system" fn WsDestroyStartupSynchronization() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(WsStartupLock));
}

#[no_mangle]
pub unsafe extern "system" fn WSApSetPostRoutine(Routine: PVOID) -> BOOL {
    WsSockPostRoutine = core::mem::transmute::<PVOID, PWS_SOCK_POST_ROUTINE>(Routine);
    return (ERROR_SUCCESS as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WSACleanup() -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: i32;
    let mut RefCount: i32;

    EnterCriticalSection(core::ptr::addr_of_mut!(WsStartupLock));

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode == ERROR_SUCCESS as i32 {
        RefCount = InterlockedDecrement(core::ptr::addr_of_mut!((*Process).RefCount));
        if RefCount == 0 {
            WsProcDelete(Process);
        } else if RefCount == 1 && WsAsyncThreadInitialized != 0 {
            WsAsyncTerminateThread();
        }

        ErrorCode = ERROR_SUCCESS as i32;
        SetLastError(ERROR_SUCCESS);
    } else {
        SetLastError(ErrorCode as u32);
        ErrorCode = SOCKET_ERROR;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsStartupLock));

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn WSAStartup(
    wVersionRequested: u16,
    lpWSAData: LPWSADATA,
) -> i32 {

    let mut VersionReturned: u16 = 0;
    let mut ErrorCode: u32 = ERROR_SUCCESS;
    let mut CurrentProcess: PWSPROCESS;

    if WsDllHandle.is_null() {
        return WSASYSNOTREADY as i32;
    }

    match LOBYTE(wVersionRequested) {
        0 => {
            ErrorCode = WSAVERNOTSUPPORTED;
            VersionReturned = MAKEWORD(2, 2);
        }
        1 => {
            if HIBYTE(wVersionRequested) <= 1 {
                VersionReturned = MAKEWORD(1, HIBYTE(wVersionRequested));
            } else {
                VersionReturned = MAKEWORD(1, 1);
            }
        }
        2 => {
            if HIBYTE(wVersionRequested) <= 2 {
                VersionReturned = MAKEWORD(2, HIBYTE(wVersionRequested));
            } else {
                VersionReturned = MAKEWORD(2, 2);
            }
        }
        _ => {
            VersionReturned = MAKEWORD(2, 2);
        }
    }

    if lpWSAData.is_null() {
        SetLastError(WSANOTINITIALISED);
        return if ErrorCode == ERROR_SUCCESS {
            WSAEFAULT as i32
        } else {
            ErrorCode as i32
        };
    }

    (*lpWSAData).wVersion = VersionReturned;
    (*lpWSAData).wHighVersion = MAKEWORD(2, 2);
    lstrcpy(
        core::ptr::addr_of_mut!((*lpWSAData).szDescription) as *mut i8,
        b"WinSock 2.0\0".as_ptr() as *const i8,
    );
    lstrcpy(
        core::ptr::addr_of_mut!((*lpWSAData).szSystemStatus) as *mut i8,
        b"Running\0".as_ptr() as *const i8,
    );

    if LOBYTE(wVersionRequested) == 1 {
        (*lpWSAData).iMaxSockets = 32767;
        (*lpWSAData).iMaxUdpDg = 65467;
    } else {
        (*lpWSAData).iMaxSockets = 0;
        (*lpWSAData).iMaxUdpDg = 0;
    }

    if ErrorCode != ERROR_SUCCESS {
        SetLastError(WSANOTINITIALISED);
        return ErrorCode as i32;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(WsStartupLock));

    loop {
        CurrentProcess = WsGetProcess();
        if !CurrentProcess.is_null() {
            break;
        }

        ErrorCode = (WsProcStartup() as u32);
        if ErrorCode != ERROR_SUCCESS {
            break;
        }

        ErrorCode = (WsSockStartup() as u32);
        if ErrorCode != ERROR_SUCCESS {
            break;
        }

        ErrorCode = WsThreadStartup();
        if ErrorCode != ERROR_SUCCESS {
            break;
        }

        CurrentProcess = WsGetProcess();
        if !CurrentProcess.is_null() {
            continue;
        }

        ErrorCode = WSASYSNOTREADY;
        break;
    }

    if ErrorCode == ERROR_SUCCESS {
        WsProcSetVersion(CurrentProcess, wVersionRequested);
        InterlockedIncrement(core::ptr::addr_of_mut!((*CurrentProcess).RefCount));
        SetLastError(ERROR_SUCCESS);
    } else {
        SetLastError(WSANOTINITIALISED);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(WsStartupLock));

    ErrorCode as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/startup.c
//  * PURPOSE:     Startup/Cleanup Support
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
// /* DATA **********************************************************************/
//
// PWS_SOCK_POST_ROUTINE WsSockPostRoutine = NULL;
// CRITICAL_SECTION WsStartupLock;
//
// #define WsStartupLock()     EnterCriticalSection(&WsStartupLock)
// #define WsStartupUnlock()   LeaveCriticalSection(&WsStartupLock)
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// __stdcall
// WsCreateStartupSynchronization(VOID)
// {
//     /* Initialize the startup lock */
//     InitializeCriticalSection(&WsStartupLock);
// }
//
// VOID
// __stdcall
// WsDestroyStartupSynchronization(VOID)
// {
//     /* Destroy the startup lock */
//     DeleteCriticalSection(&WsStartupLock);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// WSApSetPostRoutine(PVOID Routine)
// {
//     /* Set the post routine */
//     DPRINT("WSApSetPostRoutine: %p\n", Routine);
//     WsSockPostRoutine = (PWS_SOCK_POST_ROUTINE)Routine;
//     return ERROR_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSACleanup(VOID)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     INT ErrorCode;
//     LONG RefCount;
//     DPRINT("WSACleanup\n");
//
//     /* Enter startup lock */
//     WsStartupLock();
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Decrement process reference count and check if it's zero */
//         if (!(RefCount = InterlockedDecrement(&Process->RefCount)))
//         {
//             /* It's zero, destroy the process structure */
//             WsProcDelete(Process);
//         }
//         else if (RefCount == 1 && WsAsyncThreadInitialized)
//         {
//             /* Kill async thread */
//             WsAsyncTerminateThread();
//         }
//
//         DPRINT("WSACleanup RefCount = %ld\n", RefCount);
//         /* Return success */
//         ErrorCode = ERROR_SUCCESS;
//
//         /* Clear last error */
//         SetLastError(ERROR_SUCCESS);
//     }
//     else
//     {
//         DPRINT("WSACleanup uninitialized\n");
//         /* Weren't initialized */
//         SetLastError(ErrorCode);
//         ErrorCode = SOCKET_ERROR;
//     }
//
//     /* Release startup lock */
//     WsStartupUnlock();
//
//     /* Done */
//     return ErrorCode;
// }
//
// /*
//  * @implemented
//  */
// INT
// WINAPI
// WSAStartup(IN WORD wVersionRequested,
//            OUT LPWSADATA lpWSAData)
// {
//     WORD VersionReturned = 0;
//     DWORD ErrorCode = ERROR_SUCCESS;
//     PWSPROCESS CurrentProcess;
//     DPRINT("WSAStartup: %wx %d.%d\n", wVersionRequested, LOBYTE(wVersionRequested), HIBYTE(wVersionRequested));
//
//     /* Make sure that we went through DLL Init */
//     if (!WsDllHandle) return WSASYSNOTREADY;
//
//     /* Check which version is being requested */
//     switch (LOBYTE(wVersionRequested))
//     {
//         case 0:
//
//             /* We don't support this unknown version */
//             ErrorCode = WSAVERNOTSUPPORTED;
//             VersionReturned = MAKEWORD(2, 2);
//             break;
//
//         case 1:
//             /* We support only 1.0 and 1.1 */
//             if (HIBYTE(wVersionRequested) <= 1)
//             {
//                 /* Caller wants 1.0, return it */
//                 VersionReturned = MAKEWORD(1, HIBYTE(wVersionRequested));
//             }
//             else
//             {
//                 /* The only other version we support is 1.1 */
//                 VersionReturned = MAKEWORD(1, 1);
//             }
//             break;
//
//         case 2:
//             /* We support only 2.0, 2.1 and 2.2 */
//             if (HIBYTE(wVersionRequested) <= 2)
//             {
//                 /* Caller wants 2.0-2.2, return it */
//                 VersionReturned = MAKEWORD(2, HIBYTE(wVersionRequested));
//             }
//             else
//             {
//                 /* The highest version we support is 2.2 */
//                 VersionReturned = MAKEWORD(2, 2);
//             }
//             break;
//
//         default:
//
//             /* Return 2.2 */
//             VersionReturned = MAKEWORD(2, 2);
//             break;
//     }
//
//     if (lpWSAData == NULL)
//     {
//         SetLastError(WSANOTINITIALISED);
//         return ErrorCode == ERROR_SUCCESS ? WSAEFAULT : ErrorCode;
//     }
//
//     /* Return the Version Requested, unless error */
//     lpWSAData->wVersion = VersionReturned;
//
//     /* We support Winsock 2.2 */
//     lpWSAData->wHighVersion = MAKEWORD(2,2);
//     lstrcpy(lpWSAData->szDescription, "WinSock 2.0");
//     lstrcpy(lpWSAData->szSystemStatus, "Running");
//
//     /*
//      * On Winsock 1, the following values are returned.
//      * Taken straight from a Winsock Test app on Windows.
//      */
//     if (LOBYTE(wVersionRequested) == 1)
//     {
//         lpWSAData->iMaxSockets = 32767;
//         lpWSAData->iMaxUdpDg = 65467;
//     }
//     else
//     {
//         lpWSAData->iMaxSockets = 0;
//         lpWSAData->iMaxUdpDg = 0;
//     }
//
//     /* Requested invalid version (0) */
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         SetLastError(WSANOTINITIALISED);
//         return ErrorCode;
//     }
//
//     /* Enter the startup synchronization lock */
//     WsStartupLock();
//
//     /* Now setup all our objects */
//     while (TRUE)
//     {
//         /* Make sure we don't already have a process */
//         CurrentProcess = WsGetProcess();
//         if (CurrentProcess) break;
//
//         /* Setup the process object support */
//         ErrorCode = WsProcStartup();
//         if (ErrorCode != ERROR_SUCCESS) break;
//
//         /* Setup the process object support */
//         ErrorCode = WsSockStartup();
//         if (ErrorCode != ERROR_SUCCESS) break;
//
//         /* Setup the process object support */
//         ErrorCode = WsThreadStartup();
//         if (ErrorCode != ERROR_SUCCESS) break;
//
//         /* Try getting the process now */
//         CurrentProcess = WsGetProcess();
//         if (!CurrentProcess)
//         {
//             /* Something is weird... */
//             ErrorCode = WSASYSNOTREADY;
//             break;
//         }
//     }
//
//     /* Check if all worked */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Set the requested version */
//         WsProcSetVersion(CurrentProcess, wVersionRequested);
//
//         /* Increase the reference count */
//         InterlockedIncrement(&CurrentProcess->RefCount);
//         DPRINT("WSAStartup RefCount = %ld\n", CurrentProcess->RefCount);
//
//         /* Clear last error */
//         SetLastError(ERROR_SUCCESS);
//     }
//     else
//     {
//         SetLastError(WSANOTINITIALISED);
//     }
//
//     /* Leave the startup lock */
//     WsStartupUnlock();
//
//     /* Return any Error */
//     return ErrorCode;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

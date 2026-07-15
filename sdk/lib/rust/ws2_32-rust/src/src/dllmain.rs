use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::r#async::WsAsyncGlobalInitialize;
use crate::src::r#async::WsAsyncGlobalTerminate;
use crate::src::dsocket::WsSockCleanup;
use crate::src::dthread::WsThreadCleanup;
use crate::src::dthread::WsThreadDestroyCurrentThread;
use crate::src::rasdial::WsRasInitializeAutodial;
use crate::src::rasdial::WsRasUninitializeAutodial;
use crate::src::startup::WsCreateStartupSynchronization;
use crate::src::startup::WsDestroyStartupSynchronization;


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

pub static mut WsSockHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut WsDllHandle: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

pub static mut GlobalTlsIndex: DWORD = TLS_OUT_OF_INDEXES;

const TLS_OUT_OF_INDEXES: DWORD = 0xFFFFFFFF;
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hModule: HANDLE,
    dwReason: DWORD,
    lpReserved: LPVOID,
) -> BOOL {
    let mut WsProcess: PWSPROCESS;

    #[cfg(feature = "vista_or_later")]
    {
        if Ws2HelpDllMain(hModule, dwReason, lpReserved) == 0 {
            return FALSE;
        }
    }

    match dwReason {
        DLL_PROCESS_ATTACH => {
            WsDllHandle = hModule as HINSTANCE;
            WsSockHeap = GetProcessHeap();

            if GlobalTlsIndex == TLS_OUT_OF_INDEXES {
                GlobalTlsIndex = TlsAlloc();
                if GlobalTlsIndex == TLS_OUT_OF_INDEXES {
                    return FALSE;
                }
            }

            WsCreateStartupSynchronization();
            WsAsyncGlobalInitialize();
            WsRasInitializeAutodial();
        }

        DLL_THREAD_ATTACH => {}

        DLL_THREAD_DETACH => {
            WsThreadDestroyCurrentThread();
        }

        DLL_PROCESS_DETACH => {
            if WsDllHandle.is_null() {
                return TRUE;
            }

            if !lpReserved.is_null() {
                WsThreadDestroyCurrentThread();

                WsProcess = WsGetProcess();
                if !WsProcess.is_null() {
                    WsProcDelete(WsProcess);
                }

                WsThreadCleanup();
                WsSockCleanup();

                WsDestroyStartupSynchronization();
                WsAsyncGlobalTerminate();

                TlsFree(GlobalTlsIndex);
            }

            WsRasUninitializeAutodial();
            WsDllHandle = core::ptr::null_mut();
        }

        _ => {}
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn WSAGetLastError() -> ::core::ffi::c_int {
    return GetLastError() as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "system" fn WSASetLastError(iError: INT) {
    SetLastError(iError as DWORD);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dllmain.c
//  * PURPOSE:     DLL Entrypoint
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /*
//  * BUGS:
//  *       - GetAddrInfoW is completely IPv4 hacked and has some dirty code.
//  *       - LookupNodeByAddr is missing some IPv6 support.
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* DATA **********************************************************************/
//
// HANDLE WsSockHeap;
// HINSTANCE WsDllHandle;
// DWORD GlobalTlsIndex = TLS_OUT_OF_INDEXES;
//
// /* FUNCTIONS *****************************************************************/
//
// #if (DLL_EXPORT_VERSION >= _WIN32_WINNT_VISTA)
// BOOL
// __stdcall
// Ws2HelpDllMain(
//     HANDLE hModule,
//     DWORD  dwReason,
//     LPVOID lpReserved);
// #endif
//
// BOOL
// __stdcall
// DllMain(HANDLE hModule,
//         DWORD dwReason,
//         LPVOID lpReserved)
// {
//     PWSPROCESS WsProcess;
//
// #if (DLL_EXPORT_VERSION >= _WIN32_WINNT_VISTA)
//     /* Initialize ws2help */
//     if (!Ws2HelpDllMain(hModule, dwReason, lpReserved))
//     {
//         /* Should never happen */
//         ASSERT(FALSE);
//         return FALSE;
//     }
// #endif
//
//     /* Main Entrypoint */
//     switch (dwReason)
//     {
//         case DLL_PROCESS_ATTACH:
//             /* Save DLL Handle */
//             WsDllHandle = hModule;
//
//             /* Get Global Heap */
//             WsSockHeap = GetProcessHeap();
//
//             /* TLS Allocation */
//             if (GlobalTlsIndex == TLS_OUT_OF_INDEXES)
//             {
//                 GlobalTlsIndex = TlsAlloc();
//                 if (GlobalTlsIndex == TLS_OUT_OF_INDEXES)
//                 {
//                     return FALSE;
//                 }
//             }
//
//             /* Initialize some critical sections */
//             WsCreateStartupSynchronization();
//             WsAsyncGlobalInitialize();
//             WsRasInitializeAutodial();
//             break;
//
//         case DLL_THREAD_ATTACH:
//             break;
//
//         case DLL_THREAD_DETACH:
//             /* Destroy the attached Winsock Thread */
//             WsThreadDestroyCurrentThread();
//             break;
//
//         case DLL_PROCESS_DETACH:
//             /* Make sure we were initialized */
//             if (!WsDllHandle) break;
//
//             /* Check if this was a FreeLibrary call (ie: not process cleanup) */
//             if (lpReserved)
//             {
//                 /* Destroy the thread which is exiting */
//                 WsThreadDestroyCurrentThread();
//
//                 /* Check if we have a process and destroy it */
//                 WsProcess = WsGetProcess();
//                 if (WsProcess) WsProcDelete(WsProcess);
//
//                 /* Cleanup the Thread and Socket managers */
//                 WsThreadCleanup();
//                 WsSockCleanup();
//
//                 /* Cleanup critical sections */
//                 WsDestroyStartupSynchronization();
//                 WsAsyncGlobalTerminate();
//
//                 /* Free the TLS Index */
//                 TlsFree(GlobalTlsIndex);
//             }
//
//             /* Cleanup RAS auto-dial helper */
//             WsRasUninitializeAutodial();
//
//             /* Clear our handle */
//             WsDllHandle = NULL;
//             break;
//     }
//
//     /* Return to OS */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WSAGetLastError(VOID)
// {
//     /* Let the Windows Function do the work */
//     return GetLastError();
// }
//
// /*
//  * @implemented
//  */
// VOID
// __stdcall
// WSASetLastError(IN INT iError)
// {
//     /* Let the Windows Function do the work */
//     SetLastError(iError);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

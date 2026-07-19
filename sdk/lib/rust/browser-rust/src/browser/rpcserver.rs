use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(
    lpParameter: *mut core::ffi::c_void,
) -> u32 {
    // C: `RPC_STATUS Status;` — reassigned three times, so it needs `mut`. RPC_STATUS is `long`,
    // not DWORD; RPC_S_OK is 0.
    let mut Status: RPC_STATUS;

    // Static wide strings, null-terminated
    const PROTOCOL: &[u16] = &[
        0x006E, // n
        0x0063, // c
        0x0061, // a
        0x0063, // c
        0x006E, // n
        0x005F, // _
        0x006E, // n
        0x0070, // p
        0x0000,
    ];
    const ENDPOINT: &[u16] = &[
        0x005C, // '\'
        0x005C, // '\'
        0x0070, // p
        0x0069, // i
        0x0070, // p
        0x0065, // e
        0x005C, // '\'
        0x0062, // b
        0x0072, // r
        0x006F, // o
        0x0077, // w
        0x0073, // s
        0x0065, // e
        0x0072, // r
        0x0000,
    ];

    Status = RpcServerUseProtseqEpW(
        PROTOCOL.as_ptr() as *mut u16,
        20u32,
        ENDPOINT.as_ptr() as *mut u16,
        core::ptr::null_mut(),
    );
    if Status != 0 /* RPC_S_OK */ {
        return 0u32;
    }

    Status = RpcServerRegisterIf(
        browser_v0_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status != 0 {
        return 0u32;
    }

    Status = RpcServerListen(1u32, RPC_C_LISTEN_MAX_CALLS_DEFAULT, 0u32 /* FALSE */);
    if Status != 0 {
    }

    0u32
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0u32, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn I_BrowserrServerEnum(
    ServerName: BROWSER_IDENTIFY_HANDLE,
    Transport: LPWSTR,
    ClientName: LPWSTR,
    EnumStruct: LPSERVER_ENUM_STRUCT,
    PreferedMaximumLength: u32,
    TotalEntries: LPDWORD,
    ServerType: u32,
    Domain: LPWSTR,
    ResumeHandle: LPDWORD,
) -> u32 {
    // UNIMPLEMENTED
    0
}

#[no_mangle]
pub unsafe extern "system" fn BrowserOpnum1NotUsedOnWire() -> u32 {
    UNIMPLEMENTED!();
    0
}

#[no_mangle]
pub unsafe extern "system" fn I_BrowserrQueryOtherDomains(
    ServerName: *mut core::ffi::c_void,
    EnumStruct: *mut core::ffi::c_void,
    TotalEntries: *mut u32,
) -> u32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn I_BrowserrResetNetlogonState(
    ServerName: BROWSER_IDENTIFY_HANDLE,
) -> NET_API_STATUS {
    // UNIMPLEMENTED
    0
}

#[no_mangle]
pub unsafe extern "system" fn I_BrowserrDebugTrace(
    ServerName: *mut core::ffi::c_void,
    String: *mut i8,
) -> u32 {
    // UNIMPLEMENTED
    0
}
#[no_mangle]
pub unsafe extern "system" fn I_BrowserrQueryStatistics(
    ServerName: BROWSER_IDENTIFY_HANDLE,
    Statistics: *mut LPBROWSER_STATISTICS,
) -> NET_API_STATUS {
    UNIMPLEMENTED!();
    0
}
#[no_mangle]
pub unsafe extern "system" fn I_BrowserrResetStatistics(
    ServerName: BROWSER_IDENTIFY_HANDLE,
) -> NET_API_STATUS {
    // UNIMPLEMENTED
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrBrowserStatisticsClear(
    ServerName: BROWSER_IDENTIFY_HANDLE,
) -> NET_API_STATUS {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NetrBrowserStatisticsGet(
    ServerName: BROWSER_IDENTIFY_HANDLE,
    Level: u32,
    StatisticsStruct: *mut BROWSER_STATISTICS_STRUCT,
) -> u32 {
    // UNIMPLEMENTED;
    0u32
}
#[no_mangle]
pub unsafe extern "system" fn I_BrowserrSetNetlogonState(
    ServerName: BROWSER_IDENTIFY_HANDLE,
    DomainName: LPWSTR,
    EmulatedComputerName: LPWSTR,
    Role: DWORD,
) -> NET_API_STATUS {
    // UNIMPLEMENTED
    0
}

#[no_mangle]
pub unsafe extern "system" fn I_BrowserrQueryEmulatedDomains(
    ServerName: BROWSER_IDENTIFY_HANDLE,
    EmulatedDomains: *mut BROWSER_EMULATED_DOMAIN_CONTAINER,
) -> NET_API_STATUS {
    0
}
#[no_mangle]
pub unsafe extern "system" fn BrowserOpnum11NotUsedOnWire() -> u32 {
    // UNIMPLEMENTED
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Browser Service
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Browser service RPC server
//  * COPYRIGHT:   Eric Kohl 2020 <eric.kohl@reactos.org>
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// //#include "lmerr.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(browser);
//
// /* FUNCTIONS *****************************************************************/
//
// DWORD
// WINAPI
// RpcThreadRoutine(
//     LPVOID lpParameter)
// {
//     RPC_STATUS Status;
//
//     Status = RpcServerUseProtseqEpW(L"ncacn_np", 20, L"\\pipe\\browser", NULL);
//     if (Status != RPC_S_OK)
//     {
//         ERR("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerRegisterIf(browser_v0_0_s_ifspec, NULL, NULL);
//     if (Status != RPC_S_OK)
//     {
//         ERR("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, FALSE);
//     if (Status != RPC_S_OK)
//     {
//         ERR("RpcServerListen() failed (Status %lx)\n", Status);
//     }
//
//     return 0;
// }
//
//
// void __RPC_FAR * __RPC_USER midl_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER midl_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// /* Function 0 */
// NET_API_STATUS
// __stdcall
// I_BrowserrServerEnum(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     LPWSTR Transport,
//     LPWSTR ClientName,
//     LPSERVER_ENUM_STRUCT EnumStruct,
//     DWORD PreferedMaximumLength,
//     LPDWORD TotalEntries,
//     DWORD ServerType,
//     LPWSTR Domain,
//     LPDWORD ResumeHandle)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 1 (BrowserrDebugCall) */
// NET_API_STATUS
// __stdcall
// BrowserOpnum1NotUsedOnWire(VOID)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 2 */
// NET_API_STATUS
// __stdcall
// I_BrowserrQueryOtherDomains(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     LPSERVER_ENUM_STRUCT EnumStruct,
//     LPDWORD TotalEntries)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 3 */
// NET_API_STATUS
// __stdcall
// I_BrowserrResetNetlogonState(
//     BROWSER_IDENTIFY_HANDLE ServerName)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 4 */
// NET_API_STATUS
// __stdcall
// I_BrowserrDebugTrace(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     LPSTR String)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 5 */
// NET_API_STATUS
// __stdcall
// I_BrowserrQueryStatistics(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     LPBROWSER_STATISTICS *Statistics)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 6 */
// NET_API_STATUS
// __stdcall
// I_BrowserrResetStatistics(
//     BROWSER_IDENTIFY_HANDLE ServerName)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 7 - Not used on wire */
// NET_API_STATUS
// __stdcall
// NetrBrowserStatisticsClear(
//     BROWSER_IDENTIFY_HANDLE ServerName)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 8 */
// NET_API_STATUS
// __stdcall
// NetrBrowserStatisticsGet(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     DWORD Level,
//     LPBROWSER_STATISTICS_STRUCT StatisticsStruct)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 9 */
// NET_API_STATUS
// __stdcall
// I_BrowserrSetNetlogonState(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     LPWSTR DomainName,
//     LPWSTR EmulatedComputerName,
//     DWORD Role)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 10 */
// NET_API_STATUS
// __stdcall
// I_BrowserrQueryEmulatedDomains(
//     BROWSER_IDENTIFY_HANDLE ServerName,
//     PBROWSER_EMULATED_DOMAIN_CONTAINER EmulatedDomains)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
// /* Function 11 (BrowserrServerEnumEx) */
// NET_API_STATUS
// __stdcall
// BrowserOpnum11NotUsedOnWire(void)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// rpcndr.h:94 `#define midl_user_allocate MIDL_user_allocate` — the C source spells these
// lowercase but the preprocessor emits the CAPITALISED symbol, which is what the generated
// browser_s.c marshaller binds to. The #[no_mangle] definitions above therefore carry the
// capitalised name; these aliases keep the C's source spelling working for in-crate callers.
pub use self::{MIDL_user_allocate as midl_user_allocate, MIDL_user_free as midl_user_free};

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;
#[no_mangle]
pub unsafe extern "system" fn RpcThreadRoutine(lpParameter: LPVOID) -> u32 {
    let mut Status: u32;
    let ncalrpc: [u16; 8] = [0x006e, 0x0063, 0x0061, 0x006c, 0x0072, 0x0070, 0x0063, 0x0];
    let dnsresolver: [u16; 12] = [0x0044, 0x004e, 0x0053, 0x0052, 0x0065, 0x0073, 0x006f, 0x006c, 0x0076, 0x0065, 0x0072, 0x0];

    Status = (RpcServerUseProtseqEpW(ncalrpc.as_ptr() as *mut c_ushort, 20, dnsresolver.as_ptr() as *mut c_ushort, core::ptr::null_mut())) as u32;
    if Status != 0 {
        DPRINT!("RpcServerUseProtseqEpW() failed (Status {:x})", Status);
        return 0;
    }

    Status = (RpcServerRegisterIf(DnsResolver_v2_0_s_ifspec, core::ptr::null_mut(), core::ptr::null_mut())) as u32;
    if Status != 0 {
        DPRINT!("RpcServerRegisterIf() failed (Status {:x})", Status);
        return 0;
    }

    Status = (RpcServerListen(1, 1234, 0)) as u32;
    if Status != 0 {
        DPRINT!("RpcServerListen() failed (Status {:x})", Status);
    }

    DPRINT!("RpcServerListen finished");
    0
}
#[no_mangle]
pub unsafe extern "system" fn CRrReadCache(
    pwszServerName: DNSRSLVR_HANDLE,
    ppCacheEntries: *mut *mut DNS_CACHE_ENTRY,
) -> u32 {
    // DPRINT removed
    DnsIntCacheGetEntries(ppCacheEntries)
}

#[no_mangle]
pub unsafe extern "system" fn R_ResolverFlushCache(pszServerName: DNSRSLVR_HANDLE) -> u32 {
    DnsIntCacheFlush(CACHE_FLUSH_NON_HOSTS_FILE_ENTRIES)
}
#[no_mangle]
pub unsafe extern "system" fn R_ResolverFlushCacheEntry(
    pszServerName: DNSRSLVR_HANDLE,
    pszName: LPCWSTR,
    wType: WORD,
) -> DWORD {
    if pszName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    DnsIntFlushCacheEntry(pszName, wType)
}
#[no_mangle]
pub unsafe extern "system" fn R_ResolverQuery(
    pszServerName: *mut core::ffi::c_void,
    pszName: *const u16,
    wType: u16,
    dwFlags: u32,
    dwRecords: *mut u32,
    ppResultRecords: *mut *mut _DNS_RECORDW
) -> u32 {
    DPRINT!(
        "R_ResolverQuery({:p} {:p} {:#x} {:#x} {:p} {:p})\n",
        pszServerName, pszName, wType, dwFlags, dwRecords, ppResultRecords
    );
    if pszName.is_null() || wType == 0 || ppResultRecords.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if (dwFlags & DNS_QUERY_WIRE_ONLY) != 0 && (dwFlags & DNS_QUERY_NO_WIRE_QUERY) != 0 {
        return ERROR_INVALID_PARAMETER;
    }
    let mut Status: u32 = ERROR_SUCCESS;
    if dwFlags & DNS_QUERY_WIRE_ONLY != 0 {
        DPRINT!("DNS query!\n");
        Status = Query_Main(pszName, wType, dwFlags, ppResultRecords);
    } else if dwFlags & DNS_QUERY_NO_WIRE_QUERY != 0 {
        DPRINT!("DNS cache query!\n");
        Status = DnsIntCacheGetEntryByName(pszName, wType, dwFlags, ppResultRecords);
    } else {
        DPRINT!("DNS cache query!\n");
        Status = DnsIntCacheGetEntryByName(pszName, wType, dwFlags, ppResultRecords);
        if Status == DNS_INFO_NO_RECORDS {
            DPRINT!("DNS query!\n");
            Status = Query_Main(pszName, wType, dwFlags, ppResultRecords);
            if Status == ERROR_SUCCESS {
                DPRINT!("DNS query successful!\n");
                DnsIntCacheAddEntry(*ppResultRecords, 0u8);
            }
        }
    }
    if !dwRecords.is_null() {
        *dwRecords = 0;
        if Status == ERROR_SUCCESS {
            let mut Record: *mut _DNS_RECORDW = *ppResultRecords;
            while !Record.is_null() {
                DPRINT!("Record: {:p}\n", (*Record).pName);
                *dwRecords += 1;
                Record = (*Record).pNext;
            }
        }
    }
    DPRINT!("R_ResolverQuery result {} {}\n", Status, *dwRecords);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len) as SIZE_T)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0u32, ptr);
}

#[no_mangle]
pub unsafe extern "system" fn DNSRSLVR_RPC_HANDLE_rundown(hClientHandle: DNSRSLVR_HANDLE) {
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS DNS Resolver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/services/dnsrslvr/rpcserver.c
//  * PURPOSE:     RPC server interface
//  * COPYRIGHT:   Copyright 2016 Christoph von Wittich
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// DWORD
// WINAPI
// RpcThreadRoutine(LPVOID lpParameter)
// {
//     RPC_STATUS Status;
//
//     Status = RpcServerUseProtseqEpW(L"ncalrpc", 20, L"DNSResolver", NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerRegisterIf(DnsResolver_v2_0_s_ifspec, NULL, NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, 0);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT("RpcServerListen() failed (Status %lx)\n", Status);
//     }
//
//     DPRINT("RpcServerListen finished\n");
//     return 0;
// }
//
//
// /* Function: 0x00 */
// DWORD
// __stdcall
// CRrReadCache(
//      DNSRSLVR_HANDLE pwszServerName,
//      DNS_CACHE_ENTRY **ppCacheEntries)
// {
//     DPRINT("CRrReadCache(%S %p)\n",
//            pwszServerName, ppCacheEntries);
//
//     return DnsIntCacheGetEntries(ppCacheEntries);
// }
//
//
// /* Function: 0x04 */
// DWORD
// __stdcall
// R_ResolverFlushCache(
//      DNSRSLVR_HANDLE pszServerName)
// {
//     DPRINT("R_ResolverFlushCache(%S)\n",
//            pszServerName);
//
//     return DnsIntCacheFlush(CACHE_FLUSH_NON_HOSTS_FILE_ENTRIES);
// }
//
//
// /* Function: 0x05 */
// DWORD
// __stdcall
// R_ResolverFlushCacheEntry(
//      DNSRSLVR_HANDLE pszServerName,
//      LPCWSTR pszName,
//      WORD wType)
// {
//     DPRINT("R_ResolverFlushCacheEntry(%S %S %x)\n",
//            pszServerName, pszName, wType);
//
//     if (pszName == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     return DnsIntFlushCacheEntry(pszName, wType);
// }
//
//
// /* Function: 0x07 */
// DWORD
// __stdcall
// R_ResolverQuery(
//      DNSRSLVR_HANDLE pszServerName,
//      LPCWSTR pszName,
//      WORD wType,
//      DWORD dwFlags,
//      DWORD *dwRecords,
//      DNS_RECORDW **ppResultRecords)
// {
//     PDNS_RECORDW Record;
//     DNS_STATUS Status = ERROR_SUCCESS;
//
//     DPRINT("R_ResolverQuery(%S %S %x %lx %p %p)\n",
//            pszServerName, pszName, wType, dwFlags, dwRecords, ppResultRecords);
//
//     if (pszName == NULL || wType == 0 || ppResultRecords == NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     if ((dwFlags & DNS_QUERY_WIRE_ONLY) != 0 && (dwFlags & DNS_QUERY_NO_WIRE_QUERY) != 0)
//         return ERROR_INVALID_PARAMETER;
//
//     if (dwFlags & DNS_QUERY_WIRE_ONLY)
//     {
//         DPRINT("DNS query!\n");
//         Status = Query_Main(pszName,
//                             wType,
//                             dwFlags,
//                             ppResultRecords);
//     }
//     else if (dwFlags & DNS_QUERY_NO_WIRE_QUERY)
//     {
//         DPRINT("DNS cache query!\n");
//         Status = DnsIntCacheGetEntryByName(pszName,
//                                            wType,
//                                            dwFlags,
//                                            ppResultRecords);
//     }
//     else
//     {
//         DPRINT("DNS cache query!\n");
//         Status = DnsIntCacheGetEntryByName(pszName,
//                                            wType,
//                                            dwFlags,
//                                            ppResultRecords);
//         if (Status == DNS_INFO_NO_RECORDS)
//         {
//             DPRINT("DNS query!\n");
//             Status = Query_Main(pszName,
//                                 wType,
//                                 dwFlags,
//                                 ppResultRecords);
//             if (Status == ERROR_SUCCESS)
//             {
//                 DPRINT("DNS query successful!\n");
//                 DnsIntCacheAddEntry(*ppResultRecords, FALSE);
//             }
//         }
//     }
//
//     if (dwRecords)
//     {
//         *dwRecords = 0;
//
//         if (Status == ERROR_SUCCESS)
//         {
//             Record = *ppResultRecords;
//             while (Record)
//             {
//                 DPRINT("Record: %S\n", Record->pName);
//                 (*dwRecords)++;
//                 Record = Record->pNext;
//             }
//         }
//     }
//
//     DPRINT("R_ResolverQuery result %ld %ld\n", Status, *dwRecords);
//
//     return Status;
// }
//
// void __RPC_FAR * __RPC_USER midl_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
// void __RPC_USER midl_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
// void __RPC_USER DNSRSLVR_RPC_HANDLE_rundown(DNSRSLVR_HANDLE hClientHandle)
// {
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// rpcndr.h:94 `#define midl_user_allocate MIDL_user_allocate` — the C source spells these
// lowercase but the preprocessor emits the CAPITALISED symbol, which is what the generated
// dnsrslvr_s.c marshaller binds to. The #[no_mangle] definitions above therefore carry the
// capitalised name; these aliases keep the C's source spelling working for in-crate callers.
pub use self::{MIDL_user_allocate as midl_user_allocate, MIDL_user_free as midl_user_free};

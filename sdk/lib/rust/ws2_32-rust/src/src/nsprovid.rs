use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;

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

#[no_mangle]
pub unsafe extern "system" fn WsNqProvAllocate() -> PNSQUERY_PROVIDER {

    let mut Provider: PNSQUERY_PROVIDER = core::ptr::null_mut();

    Provider = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_NSQUERY_PROVIDER>() as SIZE_T),
    ) as PNSQUERY_PROVIDER;

    return Provider;
}

#[no_mangle]
pub unsafe extern "system" fn WsNqProvInitialize(
    QueryProvider: PNSQUERY_PROVIDER,
    Provider: PNS_PROVIDER,
) -> DWORD {
    InterlockedIncrement(core::ptr::addr_of_mut!((*Provider).RefCount));
    (*QueryProvider).Provider = Provider;
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WsNqProvDelete(QueryProvider: PNSQUERY_PROVIDER) {
    if !(*QueryProvider).Provider.is_null() {
        WsNpDereference((*QueryProvider).Provider);
        (*QueryProvider).Provider = core::ptr::null_mut();
    }
    HeapFree(WsSockHeap, 0, QueryProvider as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn WsNpAllocate() -> PNS_PROVIDER {
    let mut Provider: PNS_PROVIDER = core::ptr::null_mut::<_NSPROVIDER>();


    Provider = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_NSPROVIDER>() as SIZE_T),
    ) as PNS_PROVIDER;

    if !Provider.is_null() {
        (*Provider).RefCount = 1;
        (*Provider).Service.cbSize = core::mem::size_of::<NSP_ROUTINE>() as DWORD;
    }

    return Provider;
}

#[no_mangle]
pub unsafe extern "system" fn WsNpInitialize(
    Provider: PNS_PROVIDER,
    DllName: LPWSTR,
    ProviderId: LPGUID,
) -> DWORD {
    let mut ErrorCode: i32 = ERROR_SUCCESS as i32;
    let mut NSPStartupProc: LPNSPSTARTUP;
    let mut AnsiPath: [i8; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut ExpandedDllPath: [i8; MAX_PATH as usize] = [0; MAX_PATH as usize];

    WideCharToMultiByte(
        CP_ACP,
        0,
        DllName,
        -1,
        AnsiPath.as_mut_ptr(),
        MAX_PATH as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );

    ExpandEnvironmentStringsA(
        AnsiPath.as_mut_ptr(),
        ExpandedDllPath.as_mut_ptr(),
        MAX_PATH,
    );

    (*Provider).DllHandle = LoadLibraryA(ExpandedDllPath.as_mut_ptr());
    if (*Provider).DllHandle.is_null() {
        ErrorCode = WSAEPROVIDERFAILEDINIT as i32;
        if !(*Provider).DllHandle.is_null() {
            FreeLibrary(((*Provider).DllHandle as *mut c_void));
        }
        (*Provider).DllHandle = core::ptr::null_mut();
        return ErrorCode as DWORD;
    }

    NSPStartupProc = core::mem::transmute(GetProcAddress(
        (*Provider).DllHandle,
        b"NSPStartup\0".as_ptr() as *const i8,
    ));
    if NSPStartupProc.is_none() {
        ErrorCode = WSAEPROVIDERFAILEDINIT as i32;
        if !(*Provider).DllHandle.is_null() {
            FreeLibrary(((*Provider).DllHandle as *mut c_void));
        }
        (*Provider).DllHandle = core::ptr::null_mut();
        return ErrorCode as DWORD;
    }

    NSPStartupProc.unwrap_unchecked()(ProviderId, core::ptr::addr_of_mut!((*Provider).Service));

    (*Provider).ProviderId = *ProviderId;
    return ErrorCode as DWORD;
}

const CP_ACP: u32 = 0;

#[no_mangle]
pub unsafe extern "system" fn WsNpNSPCleanup(Provider: PNS_PROVIDER) -> DWORD {
    let mut ErrorCode: i32 = ERROR_SUCCESS as i32;
    let mut lpNSPCleanup: LPNSPCLEANUP = None;

    if !(*Provider).DllHandle.is_null() {
        lpNSPCleanup = core::mem::transmute::<*mut core::ffi::c_void, LPNSPCLEANUP>(
            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*Provider).Service.NSPCleanup) as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            ),
        );
        if lpNSPCleanup.is_some() {
            ErrorCode = lpNSPCleanup.unwrap_unchecked()(
                core::ptr::addr_of_mut!((*Provider).ProviderId)
            );
        }
    }

    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsNpDelete(Provider: PNS_PROVIDER) {
    if !(*Provider).DllHandle.is_null() {
        WsNpNSPCleanup(Provider);
        FreeLibrary(((*Provider).DllHandle as *mut c_void));
        (*Provider).DllHandle = core::ptr::null_mut();
    }
    HeapFree(WsSockHeap, 0, Provider as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn WsNpDereference(Provider: PNS_PROVIDER) {
    if InterlockedDecrement(&mut (*Provider).RefCount) == 0 {
        WsNpDelete(Provider);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsNqProvLookupServiceEnd(QueryProvider: PNSQUERY_PROVIDER) -> DWORD {
    return WsNpLookupServiceEnd((*QueryProvider).Provider, (*QueryProvider).LookupHandle);
}

#[no_mangle]
pub unsafe extern "system" fn WsNqProvLookupServiceNext(
    QueryProvider: PNSQUERY_PROVIDER,
    ControlFlags: DWORD,
    BufferLength: PDWORD,
    Results: LPWSAQUERYSETW,
) -> DWORD {
    return WsNpLookupServiceNext(
        (*QueryProvider).Provider,
        (*QueryProvider).LookupHandle,
        ControlFlags,
        BufferLength,
        Results,
    );
}

#[no_mangle]
pub unsafe extern "system" fn WsNqProvLookupServiceBegin(
    QueryProvider: PNSQUERY_PROVIDER,
    Restrictions: LPWSAQUERYSETW,
    ServiceClassInfo: LPWSASERVICECLASSINFOW,
    ControlFlags: DWORD,
) -> DWORD {
    return WsNpLookupServiceBegin(
        (*QueryProvider).Provider,
        Restrictions,
        ServiceClassInfo,
        ControlFlags,
        core::ptr::addr_of_mut!((*QueryProvider).LookupHandle),
    );
}

#[no_mangle]
pub unsafe extern "system" fn WsNpLookupServiceEnd(
    Provider: PNS_PROVIDER,
    LookupHandle: HANDLE,
) -> DWORD {
    return (*Provider)
        .Service
        .NSPLookupServiceEnd
        .unwrap_unchecked()(LookupHandle) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn WsNpLookupServiceNext(
    Provider: PNS_PROVIDER,
    LookupHandle: HANDLE,
    ControlFlags: DWORD,
    BufferLength: PDWORD,
    Results: LPWSAQUERYSETW,
) -> DWORD {
    return ((*Provider)
        .Service
        .NSPLookupServiceNext
        .unwrap_unchecked()(
        LookupHandle,
        ControlFlags,
        BufferLength,
        Results,
    ) as u32);
}

#[no_mangle]
pub unsafe extern "system" fn WsNpLookupServiceBegin(
    Provider: PNS_PROVIDER,
    Restrictions: LPWSAQUERYSETW,
    ServiceClassInfo: LPWSASERVICECLASSINFOW,
    ControlFlags: DWORD,
    LookupHandle: PHANDLE,
) -> DWORD {
    return ((*Provider)
        .Service
        .NSPLookupServiceBegin
        .unwrap_unchecked()(
        core::ptr::addr_of_mut!((*Provider).ProviderId),
        Restrictions,
        ServiceClassInfo,
        ControlFlags,
        LookupHandle as *mut HANDLE,
    ) as u32);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/nsprovid.c
//  * PURPOSE:     Namespace Provider Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* FUNCTIONS *****************************************************************/
//
// PNSQUERY_PROVIDER
// __stdcall
// WsNqProvAllocate(VOID)
// {
//     PNSQUERY_PROVIDER Provider;
//
//     /* Allocate the object */
//     Provider = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Provider));
//
//     /* Return it */
//     return Provider;
// }
//
// DWORD
// __stdcall
// WsNqProvInitialize(IN PNSQUERY_PROVIDER QueryProvider,
//                    IN PNS_PROVIDER Provider)
// {
//     /* Reference the provider */
//     InterlockedIncrement(&Provider->RefCount);
//
//     /* Set it as our own */
//     QueryProvider->Provider = Provider;
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// VOID
// __stdcall
// WsNqProvDelete(IN PNSQUERY_PROVIDER QueryProvider)
// {
//     /* Check if we have a provider */
//     if (QueryProvider->Provider)
//     {
//         /* Dereference it */
//         WsNpDereference(QueryProvider->Provider);
//
//         /* Clear it */
//         QueryProvider->Provider = NULL;
//     }
//
//     /* Delete us */
//     HeapFree(WsSockHeap, 0, QueryProvider);
// }
//
// PNS_PROVIDER
// __stdcall
// WsNpAllocate(VOID)
// {
//     PNS_PROVIDER Provider;
//
//     /* Allocate the object */
//     Provider = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Provider));
//     if (Provider)
//     {
//         /* Set non-null data */
//         Provider->RefCount = 1;
//         Provider->Service.cbSize = sizeof(NSP_ROUTINE);
//     }
//
//     /* Return it */
//     return Provider;
// }
//
// DWORD
// __stdcall
// WsNpInitialize(IN PNS_PROVIDER Provider,
//                IN LPWSTR DllName,
//                IN LPGUID ProviderId)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     LPNSPSTARTUP NSPStartupProc;
//     CHAR AnsiPath[MAX_PATH], ExpandedDllPath[MAX_PATH];
//
//     /* Convert the path to ANSI */
//     WideCharToMultiByte(CP_ACP,
//                         0,
//                         DllName,
//                         -1,
//                         AnsiPath,
//                         MAX_PATH,
//                         NULL,
//                         NULL);
//
//     /* Expand the DLL Path */
//     ExpandEnvironmentStringsA(AnsiPath,
//                               ExpandedDllPath,
//                               MAX_PATH);
//
//     /* Load the DLL */
//     Provider->DllHandle = LoadLibraryA(ExpandedDllPath);
//     if (!Provider->DllHandle)
//     {
//         /* Fail */
//         ErrorCode = WSAEPROVIDERFAILEDINIT;
//         goto Fail;
//     }
//
//     /* Get the pointer to NSPStartup */
//     NSPStartupProc = (LPNSPSTARTUP)GetProcAddress(Provider->DllHandle,
//                                                   "NSPStartup");
//     if (!NSPStartupProc)
//     {
//         /* Fail */
//         ErrorCode = WSAEPROVIDERFAILEDINIT;
//         goto Fail;
//     }
//
//     /* Call it */
//     (*NSPStartupProc)(ProviderId, &Provider->Service);
//
//     /* Save the provider ID */
//     Provider->ProviderId = *ProviderId;
//     return ErrorCode;
//
// Fail:
//     /* Bail out */
//     if (Provider->DllHandle) FreeLibrary(Provider->DllHandle);
//     Provider->DllHandle = NULL;
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsNpNSPCleanup(IN PNS_PROVIDER Provider)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     LPNSPCLEANUP lpNSPCleanup = NULL;
//
//     /* Make sure we have a loaded handle */
//     if (Provider->DllHandle)
//     {
//         /* Get the pointer and clear it */
//         lpNSPCleanup = InterlockedExchangePointer((PVOID*)&Provider->Service.NSPCleanup,
//                                                   NULL);
//         /* If it's not NULL, call it */
//         if (lpNSPCleanup) ErrorCode = lpNSPCleanup(&Provider->ProviderId);
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsNpDelete(IN PNS_PROVIDER Provider)
// {
//     /* Make sure we have a loaded handle */
//     if (Provider->DllHandle)
//     {
//         /* Clean us up */
//         WsNpNSPCleanup(Provider);
//
//         /* Unload the library */
//         FreeLibrary(Provider->DllHandle);
//         Provider->DllHandle = NULL;
//     }
//
//     /* Delete us */
//     HeapFree(WsSockHeap, 0, Provider);
// }
//
// VOID
// __stdcall
// WsNpDereference(IN PNS_PROVIDER Provider)
// {
//     /* Decrease the reference count and check if it's zero */
//     if (!InterlockedDecrement(&Provider->RefCount))
//     {
//         /* Delete us*/
//         WsNpDelete(Provider);
//     }
// }
//
// DWORD
// __stdcall
// WsNqProvLookupServiceEnd(IN PNSQUERY_PROVIDER QueryProvider)
// {
//     /* Simply call the provider */
//     return WsNpLookupServiceEnd(QueryProvider->Provider,
//                                 QueryProvider->LookupHandle);
// }
//
// DWORD
// __stdcall
// WsNqProvLookupServiceNext(IN PNSQUERY_PROVIDER QueryProvider,
//                           IN DWORD ControlFlags,
//                           IN PDWORD BufferLength,
//                           LPWSAQUERYSETW Results)
// {
//     /* Simply call the provider */
//     return WsNpLookupServiceNext(QueryProvider->Provider,
//                                  QueryProvider->LookupHandle,
//                                  ControlFlags,
//                                  BufferLength,
//                                  Results);
// }
//
// DWORD
// __stdcall
// WsNqProvLookupServiceBegin(IN PNSQUERY_PROVIDER QueryProvider,
//                            IN LPWSAQUERYSETW Restrictions,
//                            IN LPWSASERVICECLASSINFOW ServiceClassInfo,
//                            IN DWORD ControlFlags)
// {
//     /* Simply call the provider */
//     return WsNpLookupServiceBegin(QueryProvider->Provider,
//                                   Restrictions,
//                                   ServiceClassInfo,
//                                   ControlFlags,
//                                   &QueryProvider->LookupHandle);
// }
//
// DWORD
// __stdcall
// WsNpLookupServiceEnd(IN PNS_PROVIDER Provider,
//                      IN HANDLE LookupHandle)
// {
//     /* Call the NSP */
//     return Provider->Service.NSPLookupServiceEnd(LookupHandle);
// }
//
// DWORD
// __stdcall
// WsNpLookupServiceNext(IN PNS_PROVIDER Provider,
//                       IN HANDLE LookupHandle,
//                       IN DWORD ControlFlags,
//                       OUT PDWORD BufferLength,
//                       OUT LPWSAQUERYSETW Results)
// {
//     /* Call the NSP */
//     return Provider->Service.NSPLookupServiceNext(LookupHandle,
//                                                   ControlFlags,
//                                                   BufferLength,
//                                                   Results);
// }
//
// DWORD
// __stdcall
// WsNpLookupServiceBegin(IN PNS_PROVIDER Provider,
//                        IN LPWSAQUERYSETW Restrictions,
//                        IN LPWSASERVICECLASSINFOW ServiceClassInfo,
//                        IN DWORD ControlFlags,
//                        OUT PHANDLE LookupHandle)
// {
//     /* Call the NSP */
//     return Provider->Service.NSPLookupServiceBegin(&Provider->ProviderId,
//                                                    Restrictions,
//                                                    ServiceClassInfo,
//                                                    ControlFlags,
//                                                    LookupHandle);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

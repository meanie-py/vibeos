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

pub const NDEBUG: i32 = 1;

#[no_mangle]
pub unsafe extern "system" fn WsTpAllocate() -> PTPROVIDER {

    let mut Provider: PTPROVIDER = core::ptr::null_mut();

    Provider = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_TPROVIDER>() as SIZE_T),
    ) as PTPROVIDER;

    if !Provider.is_null() {
        (*Provider).RefCount = 1;
    }

    Provider
}
#[no_mangle]
pub unsafe extern "system" fn WsTpInitialize(
    Provider: PTPROVIDER,
    DllName: LPSTR,
    ProtocolInfo: LPWSAPROTOCOL_INFOW,
) -> DWORD {
    let VersionRequested: WORD = ((2 as u64 & 0xff) as u8 as i32
        | ((2 as u64 & 0xff) as u8 as u16 as i32) << 8) as u16;
    let mut WSPStartupProc: LPWSPSTARTUP = None;
    let mut WspData: WSPDATA = WSPDATA {
        wVersion: 0,
        wHighVersion: 0,
        szDescription: [0; 256],
    };
    let mut ExpandedDllPath: [i8; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let ErrorCode: DWORD;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Provider).UpcallTable),
        0,
        1,
    );
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Provider).Service),
        0,
        1,
    );

    (*Provider).UpcallTable.lpWPUCloseEvent =
        Some(WPUCloseEvent);
    (*Provider).UpcallTable.lpWPUCloseSocketHandle =
        Some(WPUCloseSocketHandle);
    (*Provider).UpcallTable.lpWPUCreateEvent =
        Some(WPUCreateEvent);
    (*Provider).UpcallTable.lpWPUCreateSocketHandle =
        Some(WPUCreateSocketHandle);
    (*Provider).UpcallTable.lpWPUFDIsSet =
        Some(WPUFDIsSet);
    (*Provider).UpcallTable.lpWPUGetProviderPath =
        Some(WPUGetProviderPath);
    (*Provider).UpcallTable.lpWPUModifyIFSHandle =
        Some(WPUModifyIFSHandle);
    (*Provider).UpcallTable.lpWPUPostMessage =
        Some(WPUPostMessage);
    (*Provider).UpcallTable.lpWPUQueryBlockingCallback =
        Some(WPUQueryBlockingCallback);
    (*Provider).UpcallTable.lpWPUQuerySocketHandleContext =
        Some(WPUQuerySocketHandleContext);
    (*Provider).UpcallTable.lpWPUQueueApc =
        Some(WPUQueueApc);
    (*Provider).UpcallTable.lpWPUResetEvent =
        Some(WPUResetEvent);
    (*Provider).UpcallTable.lpWPUSetEvent =
        Some(WPUSetEvent);
    (*Provider).UpcallTable.lpWPUOpenCurrentThread =
        Some(WPUOpenCurrentThread);
    (*Provider).UpcallTable.lpWPUCloseThread =
        Some(WPUCloseThread);

    ExpandEnvironmentStringsA(
        DllName as *const i8,
        ExpandedDllPath.as_mut_ptr(),
        MAX_PATH as u32,
    );

    (*Provider).DllHandle = LoadLibraryA(ExpandedDllPath.as_ptr() as *const i8);

    if (*Provider).DllHandle.is_null() {
        return SOCKET_ERROR as u32;
    }

    WSPStartupProc = core::mem::transmute::<*mut core::ffi::c_void, LPWSPSTARTUP>(GetProcAddress(
        (*Provider).DllHandle,
        b"WSPStartup\0".as_ptr() as *const i8,
    ));

    if WSPStartupProc.is_none() {
        return SOCKET_ERROR as u32;
    }

    ErrorCode = WSPStartupProc.unwrap_unchecked()(
        VersionRequested,
        core::ptr::addr_of_mut!(WspData),
        ProtocolInfo,
        (*Provider).UpcallTable,
        core::ptr::addr_of_mut!((*Provider).Service),
    ) as u32;

    ErrorCode
}
#[no_mangle]
pub unsafe extern "system" fn WsTpWSPCleanup(
    Provider: PTPROVIDER,
    lpErrNo: LPINT,
) -> DWORD {
    let mut WSPCleanup: LPWSPCLEANUP = None;
    let mut ErrorCode: INT = (ERROR_SUCCESS as i32);

    if !(*Provider).DllHandle.is_null() {
        WSPCleanup = core::mem::transmute::<*mut core::ffi::c_void, LPWSPCLEANUP>(
            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*Provider).Service.lpWSPCleanup) as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            ),
        );
        if WSPCleanup.is_some() {
            ErrorCode = WSPCleanup.unwrap_unchecked()(lpErrNo) as INT;
        }
    }

    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsTpDelete(Provider: PTPROVIDER) {
    let mut ErrorCode: INT = 0;

    if !(*Provider).DllHandle.is_null() {
        WsTpWSPCleanup(Provider, &mut ErrorCode);
        FreeLibrary(((*Provider).DllHandle as *mut c_void));
        (*Provider).DllHandle = core::ptr::null_mut();
    }

    HeapFree(WsSockHeap, 0, Provider as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn WsTpDereference(Provider: PTPROVIDER) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*Provider).RefCount)) == 0 {
        WsTpDelete(Provider);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dprovide.c
//  * PURPOSE:     Transport Provider Object
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
// /* FUNCTIONS *****************************************************************/
//
// PTPROVIDER
// __stdcall
// WsTpAllocate(VOID)
// {
//     PTPROVIDER Provider;
//
//     /* Allocate the object */
//     Provider = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Provider));
//     if (Provider)
//     {
//         /* Setup non-zero data */
//         Provider->RefCount = 1;
//     }
//
//     /* Return it */
//     return Provider;
// }
//
// DWORD
// __stdcall
// WsTpInitialize(IN PTPROVIDER Provider,
//                IN LPSTR DllName,
//                IN LPWSAPROTOCOL_INFOW ProtocolInfo)
// {
//     WORD VersionRequested = MAKEWORD(2,2);
//     LPWSPSTARTUP WSPStartupProc;
//     WSPDATA WspData;
//     CHAR ExpandedDllPath[MAX_PATH];
//     DWORD ErrorCode;
//     DPRINT("WsTpInitialize: %p, %p, %p\n", Provider, DllName, ProtocolInfo);
//
//     /* Clear the tables */
//     RtlZeroMemory(&Provider->UpcallTable, sizeof(Provider->UpcallTable));
//     RtlZeroMemory(&Provider->Service, sizeof(Provider->Service));
//
//     /* Set up the Upcall Table */
//     Provider->UpcallTable.lpWPUCloseEvent = WPUCloseEvent;
//     Provider->UpcallTable.lpWPUCloseSocketHandle = WPUCloseSocketHandle;
//     Provider->UpcallTable.lpWPUCreateEvent = WPUCreateEvent;
//     Provider->UpcallTable.lpWPUCreateSocketHandle = WPUCreateSocketHandle;
//     Provider->UpcallTable.lpWPUFDIsSet = WPUFDIsSet;
//     Provider->UpcallTable.lpWPUGetProviderPath = WPUGetProviderPath;
//     Provider->UpcallTable.lpWPUModifyIFSHandle = WPUModifyIFSHandle;
//     Provider->UpcallTable.lpWPUPostMessage = WPUPostMessage;
//     Provider->UpcallTable.lpWPUQueryBlockingCallback = WPUQueryBlockingCallback;
//     Provider->UpcallTable.lpWPUQuerySocketHandleContext = WPUQuerySocketHandleContext;
//     Provider->UpcallTable.lpWPUQueueApc = WPUQueueApc;
//     Provider->UpcallTable.lpWPUResetEvent = WPUResetEvent;
//     Provider->UpcallTable.lpWPUSetEvent = WPUSetEvent;
//     Provider->UpcallTable.lpWPUOpenCurrentThread = WPUOpenCurrentThread;
//     Provider->UpcallTable.lpWPUCloseThread = WPUCloseThread;
//
//     /* Expand the DLL Path */
//     ExpandEnvironmentStrings(DllName, ExpandedDllPath, MAX_PATH);
//
//     /* Load the DLL */
//     Provider->DllHandle = LoadLibrary(ExpandedDllPath);
//
//     if (!Provider->DllHandle)
//     {
//         return SOCKET_ERROR;
//     }
//     /* Get the pointer to WSPStartup */
//     WSPStartupProc = (LPWSPSTARTUP)GetProcAddress(Provider->DllHandle, "WSPStartup");
//
//     if (!WSPStartupProc)
//     {
//         return SOCKET_ERROR;
//     }
//     /* Call it */
//     ErrorCode = (*WSPStartupProc)(VersionRequested,
//                       &WspData,
//                       ProtocolInfo,
//                       Provider->UpcallTable,
//                       &Provider->Service);
//
//     /* Return */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsTpWSPCleanup(IN PTPROVIDER Provider,
//                IN LPINT lpErrNo)
// {
//     LPWSPCLEANUP WSPCleanup = NULL;
//     INT ErrorCode = ERROR_SUCCESS;
//
//     /* Make sure we have a loaded handle */
//     if (Provider->DllHandle)
//     {
//         /* Get the pointer and clear it */
//         WSPCleanup = InterlockedExchangePointer((PVOID*)&Provider->Service.lpWSPCleanup,
//                                                 NULL);
//         /* If it's not NULL, call it */
//         if (WSPCleanup) ErrorCode = WSPCleanup(lpErrNo);
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsTpDelete(IN PTPROVIDER Provider)
// {
//     INT ErrorCode;
//
//     /* Make sure we have a loaded handle */
//     if (Provider->DllHandle)
//     {
//         /* Clean us up */
//         WsTpWSPCleanup(Provider, &ErrorCode);
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
// WsTpDereference(IN PTPROVIDER Provider)
// {
//     /* Decrease the reference count and check if it's zero */
//     if (!InterlockedDecrement(&Provider->RefCount))
//     {
//         /* Delete us*/
//         WsTpDelete(Provider);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

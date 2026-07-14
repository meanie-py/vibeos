use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dcatalog::WsTcAllocate;
use crate::src::dcatalog::WsTcInitializeFromRegistry;
use crate::src::nscatalo::WsNcAllocate;
use crate::src::nscatalo::WsNcInitializeFromRegistry;
use crate::src::nscatalo::WsNcRefreshFromRegistry;
use crate::src::nsprovid::WsNpNSPCleanup;
use crate::src::wsautil::WsOpenRegistryRoot;

extern "system" {
}

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

pub static mut CurrentWsProcess: PWSPROCESS = ::core::ptr::null::<_WSPROCESS>() as *mut _WSPROCESS;

#[no_mangle]
pub unsafe extern "system" fn WsProcInitialize(Process: PWSPROCESS) -> INT {
    let mut ErrorCode: INT = (WSAEFAULT as i32);
    let mut RootKey: HKEY = core::ptr::null_mut();

    InitializeCriticalSection((core::ptr::addr_of_mut!((*Process).ThreadLock) as *mut c_void));
    (*Process).LockReady = 1u8;

    RootKey = WsOpenRegistryRoot();

    (*Process).ProtocolCatalogEvent = CreateEvent(
        core::ptr::null_mut(),
        1,
        0,
        core::ptr::null_mut(),
    );
    (*Process).ProtocolCatalog = WsTcAllocate();

    WsTcInitializeFromRegistry(
        (*Process).ProtocolCatalog,
        RootKey,
        (*Process).ProtocolCatalogEvent,
    );

    (*Process).NamespaceCatalogEvent = CreateEvent(
        core::ptr::null_mut(),
        1,
        0,
        core::ptr::null_mut(),
    );
    (*Process).NamespaceCatalog = WsNcAllocate();

    ErrorCode = WsNcInitializeFromRegistry(
        (*Process).NamespaceCatalog,
        RootKey,
        (*Process).NamespaceCatalogEvent,
    );

    RegCloseKey(RootKey);
    ErrorCode
}
#[no_mangle]
pub unsafe extern "system" fn WsProcAllocate() -> PWSPROCESS {

    let Process: PWSPROCESS = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_WSPROCESS>() as SIZE_T),
    ) as PWSPROCESS;

    if !Process.is_null() {
        (*Process).Version = ((2 & 0xff) | ((2 & 0xff) << 8)) as u32;
    }

    Process
}

#[no_mangle]
pub unsafe extern "system" fn WsProcOpenAsyncHelperDevice(
    Process: PWSPROCESS,
    Handle: PHANDLE,
) -> INT {
    let mut ErrorCode: INT = (WSASYSCALLFAILURE as i32);

    EnterCriticalSection(&mut (*Process).ThreadLock);

    if !(*Process).ApcHelper.is_null()
        || WahOpenApcHelper(core::ptr::addr_of_mut!((*Process).ApcHelper)) == (ERROR_SUCCESS as u32 as i32)
    {
        *Handle = (*Process).ApcHelper as *mut core::ffi::c_void;
        ErrorCode = (ERROR_SUCCESS as i32);
    }

    LeaveCriticalSection(&mut (*Process).ThreadLock);
    return ErrorCode;
}

#[no_mangle]
pub unsafe extern "system" fn WsProcGetAsyncHelper(
    Process: PWSPROCESS,
    Handle: PHANDLE,
) -> INT {
    if !(*Process).ApcHelper.is_null() {
        *Handle = (*Process).ApcHelper;
        return (ERROR_SUCCESS as i32);
    } else {
        return WsProcOpenAsyncHelperDevice(Process, Handle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsProcStartup() -> INT {
    let mut ErrorCode: INT = (WSAEFAULT as i32);

    CurrentWsProcess = WsProcAllocate();

    if !CurrentWsProcess.is_null() {
        ErrorCode = WsProcInitialize(CurrentWsProcess);
    } else {
        ErrorCode = (WSA_NOT_ENOUGH_MEMORY as i32);
    }

    return ErrorCode;
}
#[no_mangle]
pub unsafe extern "system" fn WsProcGetTCatalog(Process: PWSPROCESS) -> PTCATALOG {
    if WsCheckCatalogState((*Process).ProtocolCatalogEvent) != 0 {
        WsTcRefreshFromRegistry(
            (*Process).ProtocolCatalog,
            (*Process).ProtocolCatalogEvent,
        );
    }
    (*Process).ProtocolCatalog
}

#[no_mangle]
pub unsafe extern "system" fn WsProcGetNsCatalog(Process: PWSPROCESS) -> PNSCATALOG {

    if WsCheckCatalogState((*Process).NamespaceCatalogEvent) != 0 {
        WsNcRefreshFromRegistry(
            (*Process).NamespaceCatalog,
            (*Process).NamespaceCatalogEvent,
        );
    }

    (*Process).NamespaceCatalog
}

#[no_mangle]
pub unsafe extern "system" fn WsProcDetachSocket(
    Process: PWSPROCESS,
    Handle: PWAH_HANDLE,
) -> BOOL {
    let Socket: PWSSOCKET = Handle as PWSSOCKET;

    WahRemoveHandleContext(WsSockHandleTable, Handle);

    if (*Socket).Provider.is_null() {
        if !(*Process).HandleHelper.is_null() {
            WahCloseSocketHandle(((*Process).HandleHelper as *mut _WSH_HANDLE_TABLE), (*Socket).Handle as SOCKET);
        }
    }

    WsSockDereference(Socket);
    return (TRUE as i32);
}
#[no_mangle]
pub unsafe extern "system" fn CleanupNamespaceProviders(
    _Callback: *mut core::ffi::c_void,
    Entry: PNSCATALOG_ENTRY,
) -> i32 {

    let Provider: PNS_PROVIDER = (*Entry).Provider;
    if !Provider.is_null() {
        WsNpNSPCleanup(Provider);
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CleanupProtocolProviders(
    Callback: PVOID,
    Entry: PTCATALOG_ENTRY,
) -> BOOL {
    let mut ErrorCode: INT = 0;

    let Provider: PTPROVIDER = (*Entry).Provider;
    if !Provider.is_null() {
        WsTpWSPCleanup(Provider, core::ptr::addr_of_mut!(ErrorCode));
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn WsProcDelete(Process: PWSPROCESS) {
    if (*Process).LockReady == 0 {
        return;
    }
    CurrentWsProcess = core::ptr::null_mut();
    if !WsSockHandleTable.is_null() {
        WahEnumerateHandleContexts(
            WsSockHandleTable,
            Some(WsSockDeleteSockets),
            Process as PVOID,
        );
    }
    if !(*Process).ApcHelper.is_null() {
        WahCloseApcHelper((*Process).ApcHelper);
    }
    if !(*Process).HandleHelper.is_null() {
        WahCloseHandleHelper((*Process).HandleHelper);
    }
    if !(*Process).NotificationHelper.is_null() {
        WahCloseNotificationHandleHelper((*Process).NotificationHelper);
    }
    if !(*Process).ProtocolCatalog.is_null() {
        WsTcEnumerateCatalogItems(
            (*Process).ProtocolCatalog,
            Some(CleanupProtocolProviders),
            core::ptr::null_mut(),
        );
        WsTcDelete((*Process).ProtocolCatalog);
        (*Process).ProtocolCatalog = core::ptr::null_mut();
    }
    if !(*Process).NamespaceCatalog.is_null() {
        WsNcEnumerateCatalogItems(
            (*Process).NamespaceCatalog,
            Some(CleanupNamespaceProviders),
            core::ptr::null_mut(),
        );
        WsNcDelete((*Process).NamespaceCatalog);
        (*Process).NamespaceCatalog = core::ptr::null_mut();
    }
    DeleteCriticalSection(core::ptr::addr_of_mut!((*Process).ThreadLock));
    HeapFree(WsSockHeap, 0, Process as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn WsProcSetVersion(
    Process: PWSPROCESS,
    VersionRequested: WORD,
) {
    let Major: WORD;
    let Minor: WORD;
    let OldMajor: WORD;
    let OldMinor: WORD;

    Major = (VersionRequested as u64 & 0xff) as BYTE as WORD;
    Minor = ((VersionRequested as u64 >> 8) & 0xff) as BYTE as WORD;
    OldMajor = ((*Process).Version as u64 & 0xff) as BYTE as WORD;
    OldMinor = (((*Process).Version as u64 >> 8) & 0xff) as BYTE as WORD;

    if (Major as i32) < (OldMajor as i32)
        || ((Major as i32) == (OldMajor as i32) && (Minor as i32) < (OldMinor as i32))
    {
        (*Process).Version = VersionRequested as DWORD;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dprocess.c
//  * PURPOSE:     Process Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* DATA **********************************************************************/
//
// PWSPROCESS CurrentWsProcess;
//
// #define WsProcLock()        EnterCriticalSection(&Process->ThreadLock)
// #define WsProcUnlock()      LeaveCriticalSection(&Process->ThreadLock)
//
// /* FUNCTIONS *****************************************************************/
//
// INT
// __stdcall
// WsProcInitialize(IN PWSPROCESS Process)
// {
//     INT ErrorCode = WSAEFAULT;
//     HKEY RootKey = NULL;
//
//     /* Initialize the thread list lock */
//     InitializeCriticalSection(&Process->ThreadLock);
//     Process->LockReady = TRUE;
//
//     /* Open the Winsock Key */
//     RootKey = WsOpenRegistryRoot();
//
//     /* Create the LP Catalog change event and catalog */
//     Process->ProtocolCatalogEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
//     Process->ProtocolCatalog = WsTcAllocate();
//
//     // FIXME: Check for Process->ProtocolCatalog == NULL
//
//     /* Initialize it */
//     WsTcInitializeFromRegistry(Process->ProtocolCatalog,
//                                RootKey,
//                                Process->ProtocolCatalogEvent);
//
//     /* Create the NS Catalog change event and catalog */
//     Process->NamespaceCatalogEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
//     Process->NamespaceCatalog = WsNcAllocate();
//
//     // FIXME: Check for Process->NamespaceCatalog == NULL
//
//     /* Initialize it */
//     ErrorCode = WsNcInitializeFromRegistry(Process->NamespaceCatalog,
//                                            RootKey,
//                                            Process->NamespaceCatalogEvent);
//
//     /* Close the root key */
//     RegCloseKey(RootKey);
//     return ErrorCode;
// }
//
// PWSPROCESS
// __stdcall
// WsProcAllocate(VOID)
// {
//     PWSPROCESS Process;
//
//     /* Allocate the structure */
//     Process = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Process));
//     if (Process)
//     {
//         /* Set default non-zero values */
//         Process->Version = MAKEWORD(2,2);
//     }
//
//     /* Return it */
//     return Process;
// }
//
// INT
// __stdcall
// WsProcOpenAsyncHelperDevice(IN PWSPROCESS Process,
//                             OUT PHANDLE Handle)
// {
//     INT ErrorCode = WSASYSCALLFAILURE;
//
//     /* Lock the process */
//     WsProcLock();
//
//     /* Check if we have a handle, and if not, create one */
//     if ((Process->ApcHelper) ||
//         (WahOpenApcHelper(&Process->ApcHelper) == ERROR_SUCCESS))
//     {
//         /* Return the handle */
//         *Handle = Process->ApcHelper;
//         ErrorCode = ERROR_SUCCESS;
//     }
//
//     /* Unload the process and return */
//     WsProcUnlock();
//     return ErrorCode;
// }
//
// INT
// __stdcall
// WsProcGetAsyncHelper(IN PWSPROCESS Process,
//                      OUT PHANDLE Handle)
// {
//     /* Check if we have it already set up */
//     if (Process->ApcHelper)
//     {
//         /* Just return it */
//         *Handle = Process->ApcHelper;
//         return ERROR_SUCCESS;
//     }
//     else
//     {
//         /* Open it for the first time */
//         return WsProcOpenAsyncHelperDevice(Process, Handle);
//     }
// }
//
// INT
// __stdcall
// WsProcStartup(VOID)
// {
//     INT ErrorCode = WSAEFAULT;
//
//     /* Create a new process */
//     CurrentWsProcess = WsProcAllocate();
//
//     /* Initialize it */
//     if (CurrentWsProcess)
//     {
//         /* Initialize the process */
//         ErrorCode = WsProcInitialize(CurrentWsProcess);
//     }
//     else
//     {
//         /* No memory for the process object */
//         ErrorCode = WSA_NOT_ENOUGH_MEMORY;
//     }
//
//     return ErrorCode;
// }
//
// PTCATALOG
// __stdcall
// WsProcGetTCatalog(IN PWSPROCESS Process)
// {
//     /* Check if the catalogs have been modified */
//     if (WsCheckCatalogState(Process->ProtocolCatalogEvent))
//     {
//         /* Modification happened, reload them */
//         WsTcRefreshFromRegistry(Process->ProtocolCatalog,
//                                 Process->ProtocolCatalogEvent);
//     }
//
//     /* Return it */
//     return Process->ProtocolCatalog;
// }
//
// PNSCATALOG
// __stdcall
// WsProcGetNsCatalog(IN PWSPROCESS Process)
// {
//     /* Check if the catalogs have been modified */
//     if (WsCheckCatalogState(Process->NamespaceCatalogEvent))
//     {
//         /* Modification happened, reload them */
//         WsNcRefreshFromRegistry(Process->NamespaceCatalog,
//                                 Process->NamespaceCatalogEvent);
//     }
//
//     /* Return it */
//     return Process->NamespaceCatalog;
// }
//
// BOOL
// __stdcall
// WsProcDetachSocket(IN PWSPROCESS Process,
//                    IN PWAH_HANDLE Handle)
// {
//     PWSSOCKET Socket = (PWSSOCKET)Handle;
//
//     /* Disassociate this socket from the table */
//     WahRemoveHandleContext(WsSockHandleTable, Handle);
//
//     /* If this is isn't an IFS socket */
//     if (!Socket->Provider)
//     {
//         /* Check if we have an active handle helper */
//         if (Process->HandleHelper)
//         {
//             /* Close it */
//             WahCloseSocketHandle(Process->HandleHelper, (SOCKET)Socket->Handle);
//         }
//     }
//
//     /* Remove a reference and return */
//     WsSockDereference(Socket);
//     return TRUE;
// }
//
// BOOL
// __stdcall
// CleanupNamespaceProviders(IN PVOID Callback,
//                           IN PNSCATALOG_ENTRY Entry)
// {
//     PNS_PROVIDER Provider;
//
//     /* Get the provider */
//     Provider = Entry->Provider;
//     if (Provider)
//     {
//         /* Do cleanup */
//         WsNpNSPCleanup(Provider);
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// BOOL
// __stdcall
// CleanupProtocolProviders(IN PVOID Callback,
//                          IN PTCATALOG_ENTRY Entry)
// {
//     PTPROVIDER Provider;
//     INT ErrorCode;
//
//     /* Get the provider */
//     Provider = Entry->Provider;
//     if (Provider)
//     {
//         /* Do cleanup */
//         WsTpWSPCleanup(Provider, &ErrorCode);
//     }
//
//     /* Return success */
//     return TRUE;
// }
//
// VOID
// __stdcall
// WsProcDelete(IN PWSPROCESS Process)
// {
//     /* Check if we didn't even initialize yet */
//     if (!Process->LockReady) return;
//
//     /* No more current process */
//     CurrentWsProcess = NULL;
//
//     /* If we have a socket table */
//     if (WsSockHandleTable)
//     {
//         /* Enumerate the sockets with a delete callback */
//         WahEnumerateHandleContexts(WsSockHandleTable,
//                                    WsSockDeleteSockets,
//                                    Process);
//     }
//
//     /* Close APC Helper */
//     if (Process->ApcHelper) WahCloseApcHelper(Process->ApcHelper);
//
//     /* Close handle helper */
//     if (Process->HandleHelper) WahCloseHandleHelper(Process->HandleHelper);
//
//     /* Check for notification helper */
//     if (Process->NotificationHelper)
//     {
//         /* Close notification helper */
//         WahCloseNotificationHandleHelper(Process->NotificationHelper);
//     }
//
//     /* Check if we have a protocol catalog*/
//     if (Process->ProtocolCatalog)
//     {
//         /* Enumerate it to clean it up */
//         WsTcEnumerateCatalogItems(Process->ProtocolCatalog,
//                                   CleanupProtocolProviders,
//                                   NULL);
//
//         /* Delete it */
//         WsTcDelete(Process->ProtocolCatalog);
//         Process->ProtocolCatalog = NULL;
//     }
//
//     /* Check if we have a namespace catalog*/
//     if (Process->NamespaceCatalog)
//     {
//         /* Enumerate it to clean it up */
//         WsNcEnumerateCatalogItems(Process->NamespaceCatalog,
//                                   CleanupNamespaceProviders,
//                                   NULL);
//
//         /* Delete it */
//         WsNcDelete(Process->NamespaceCatalog);
//         Process->NamespaceCatalog = NULL;
//     }
//
//     /* Delete the thread lock */
//     DeleteCriticalSection(&Process->ThreadLock);
//
//     /* Delete us */
//     HeapFree(WsSockHeap, 0, Process);
// }
//
// VOID
// __stdcall
// WsProcSetVersion(IN PWSPROCESS Process,
//                  IN WORD VersionRequested)
// {
//     WORD Major, Minor;
//     WORD OldMajor, OldMinor;
//
//     /* Get the version data */
//     Major = LOBYTE(VersionRequested);
//     Minor = HIBYTE(VersionRequested);
//     OldMajor = LOBYTE(Process->Version);
//     OldMinor = HIBYTE(Process->Version);
//
//     /* Check if we're going lower */
//     if ((Major < OldMajor) || ((Major == OldMajor) && (Minor < OldMinor)))
//     {
//         /* Set the new version */
//         Process->Version = VersionRequested;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

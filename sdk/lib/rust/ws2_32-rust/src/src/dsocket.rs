use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dcatalog::WsTcFindIfsProviderForSocket;
use crate::src::dprocess::WsProcDetachSocket;


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

pub static mut WsSockHandleTable: PWAH_HANDLE_TABLE =
    ::core::ptr::null::<_WSH_HANDLE_TABLE>() as *mut _WSH_HANDLE_TABLE;

#[no_mangle]
pub unsafe extern "system" fn WsSockStartup() -> INT {
    if WsSockHandleTable.is_null() {
        return WahCreateHandleContextTable(core::ptr::addr_of_mut!(WsSockHandleTable));
    }
    return (NO_ERROR as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WsSockCleanup() {
    if !WsSockHandleTable.is_null() {
        WahDestroyHandleContextTable(WsSockHandleTable);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsSockAllocate() -> PWSSOCKET {

    let Socket: PWSSOCKET = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_WSSOCKET>() as SIZE_T),
    ) as PWSSOCKET;

    if !Socket.is_null() {
        (*Socket).RefCount = 2;
        (*Socket).Overlapped = 1u8;
    }

    Socket
}

#[no_mangle]
pub unsafe extern "system" fn WsSockInitialize(
    Socket: PWSSOCKET,
    CatalogEntry: PTCATALOG_ENTRY,
) -> i32 {
    let CurrentThread: PWSTHREAD;

    (*Socket).CatalogEntry = CatalogEntry;
    InterlockedIncrement(&mut (*CatalogEntry).RefCount as *mut i32);

    (*Socket).Provider = (*CatalogEntry).Provider;

    CurrentThread = TlsGetValue(TlsIndex) as PWSTHREAD;
    if !CurrentThread.is_null() {
        (*Socket).Overlapped = ((*CurrentThread).OpenType == 0) as u8;
    }

    (ERROR_SUCCESS as i32)
}
#[no_mangle]
pub unsafe extern "system" fn WsSockGetSocketNoExport(Handle: SOCKET) -> PWSSOCKET {

    WahReferenceContextByHandle(WsSockHandleTable, Handle as LPVOID) as PWSSOCKET
}

#[no_mangle]
pub unsafe extern "system" fn WsSockFindIfsSocket(Handle: SOCKET) -> PWSSOCKET {
    let mut ErrorCode: i32;
    let mut Flags: u32 = 0;
    let mut Socket: PWSSOCKET = core::ptr::null_mut();
    let mut Process: PWSPROCESS;
    let mut Catalog: PTCATALOG;

    if Handle != !0 as SOCKET && GetHandleInformation(Handle as *mut core::ffi::c_void, &mut Flags) != 0 {
        Process = WsGetProcess();
        if !Process.is_null() {
            Catalog = WsProcGetTCatalog(Process);
            ErrorCode = (WsTcFindIfsProviderForSocket(Catalog, Handle) as i32);
            if ErrorCode == ERROR_SUCCESS as i32 {
                Socket = WsSockGetSocketNoExport(Handle);
                if !Socket.is_null() {
                    (*Socket).ApiSocket = 1u8;
                }
            }
        }
    }

    Socket
}

#[no_mangle]
pub unsafe extern "system" fn WsSockGetSocket(Handle: SOCKET) -> PWSSOCKET {
    let mut Socket: PWSSOCKET = core::ptr::null_mut();

    if !WsSockHandleTable.is_null() && {
        Socket = WahReferenceContextByHandle(WsSockHandleTable, Handle as HANDLE) as PWSSOCKET;
        !Socket.is_null()
    } {
        return Socket;
    } else {
        return WsSockFindIfsSocket(Handle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsSockAddApiReference(Handle: SOCKET) -> INT {
    let Socket: PWSSOCKET = WsSockGetSocketNoExport(Handle);
    if !Socket.is_null() {
        if !Socket.is_null() {
            (*Socket).ApiSocket = 1u8;
        }
        WsSockDereference(Socket);
        return (ERROR_SUCCESS as i32);
    }
    return WSASYSCALLFAILURE as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WsSockDeleteSockets(
    Context: LPVOID,
    Handle: PWAH_HANDLE,
) -> BOOL {

    return WsProcDetachSocket(Context as PWSPROCESS, Handle);
}
#[no_mangle]
pub unsafe extern "system" fn WsSockDelete(Socket: PWSSOCKET) {
    if !(*Socket).CatalogEntry.is_null() {
        WsTcEntryDereference((*Socket).CatalogEntry);
        (*Socket).CatalogEntry = core::ptr::null_mut();
    }
    HeapFree(WsSockHeap, 0, Socket as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn WsSockDereference(Socket: PWSSOCKET) {
    if InterlockedDecrement(&mut (*Socket).RefCount) == 0 {
        WsSockDelete(Socket);
    }
}
#[no_mangle]
pub unsafe extern "system" fn WsSockDisassociateHandle(Socket: PWSSOCKET) -> INT {

    WahRemoveHandleContext(WsSockHandleTable, Socket as PWAH_HANDLE)
}

#[no_mangle]
pub unsafe extern "system" fn WsSockAssociateHandle(
    Socket: PWSSOCKET,
    Handle: SOCKET,
    IsProvider: BOOLEAN,
) -> INT {
    let mut ErrorCode: INT = ERROR_SUCCESS as INT;
    let mut OldSocket: PWSSOCKET;

    (*Socket).IsProvider = IsProvider;
    (*Socket).Handle = Handle as HANDLE;

    OldSocket = WahInsertHandleContext(WsSockHandleTable, Socket as PWAH_HANDLE) as PWSSOCKET;

    if OldSocket != Socket {
        WsSockDereference(OldSocket);
    } else if OldSocket.is_null() {
        ErrorCode = WSAENOBUFS as INT;
    }

    return ErrorCode;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dsocket.c
//  * PURPOSE:     Socket Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* DATA **********************************************************************/
//
// PWAH_HANDLE_TABLE WsSockHandleTable;
//
// /* FUNCTIONS *****************************************************************/
//
// INT
// __stdcall
// WsSockStartup(VOID)
// {
//     /* Check if we have a socket table */
//     if (!WsSockHandleTable)
//     {
//         /* Create it */
//         return WahCreateHandleContextTable(&WsSockHandleTable);
//     }
//
//     /* Nothing to do */
//     return NO_ERROR;
// }
//
// VOID
// __stdcall
// WsSockCleanup(VOID)
// {
//     /* Check if we have a socket table */
//     if (WsSockHandleTable)
//     {
//         /* Destroy it */
//         WahDestroyHandleContextTable(WsSockHandleTable);
//     }
// }
//
// PWSSOCKET
// __stdcall
// WsSockAllocate(VOID)
// {
//     PWSSOCKET Socket;
//
//     /* Allocate the socket object */
//     Socket = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Socket));
//     if (Socket)
//     {
//         /* Setup default non-zero values */
//         Socket->RefCount = 2;
//         Socket->Overlapped = TRUE;
//     }
//
//     /* Return it */
//     return Socket;
// }
//
// INT
// __stdcall
// WsSockInitialize(IN PWSSOCKET Socket,
//                  IN PTCATALOG_ENTRY CatalogEntry)
// {
//     PWSTHREAD CurrentThread;
//
//     /* Associate this catalog and reference it */
//     Socket->CatalogEntry = CatalogEntry;
//     InterlockedIncrement(&CatalogEntry->RefCount);
//
//     /* Associate the Provider and Process Objects */
//     Socket->Provider = CatalogEntry->Provider;
//
//     /* Get the current Thread Object */
//     if ((CurrentThread = TlsGetValue(TlsIndex)))
//     {
//         /* Set the overlapped mode */
//         Socket->Overlapped = (CurrentThread->OpenType == 0);
//     }
//
//     /* Return status */
//     return ERROR_SUCCESS;
// }
//
// PWSSOCKET
// __stdcall
// WsSockGetSocketNoExport(IN SOCKET Handle)
// {
//     /* Let WAH do the translation */
//     return (PWSSOCKET)WahReferenceContextByHandle(WsSockHandleTable,
//                                                   (HANDLE)Handle);
// }
//
// PWSSOCKET
// __stdcall
// WsSockFindIfsSocket(IN SOCKET Handle)
// {
//     INT ErrorCode;
//     DWORD Flags;
//     PWSSOCKET Socket = NULL;
//     PWSPROCESS Process = NULL;
//     PTCATALOG Catalog = NULL;
//
//     /* Validate the socket and get handle info */
//     if ((Handle != INVALID_SOCKET) &&
//         (GetHandleInformation((HANDLE)Handle, &Flags)))
//     {
//         /* Get the process */
//         if ((Process = WsGetProcess()))
//         {
//             /* Get the catalog */
//             Catalog = WsProcGetTCatalog(Process);
//
//             /* Get the IFS Provider */
//             ErrorCode = WsTcFindIfsProviderForSocket(Catalog, Handle);
//
//             /* Check for success */
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Get the Socket now */
//                 Socket = WsSockGetSocketNoExport(Handle);
//
//                 /* Mark it as an API Socket */
//                 if (Socket) Socket->ApiSocket = TRUE;
//             }
//         }
//     }
//
//     /* Return the socket */
//     return Socket;
// }
//
// PWSSOCKET
// __stdcall
// WsSockGetSocket(IN SOCKET Handle)
// {
//     PWSSOCKET Socket;
//
//     /* Let WAH do the translation */
//     if ((WsSockHandleTable != NULL) &&
//         (Socket = (PWSSOCKET)WahReferenceContextByHandle(WsSockHandleTable, (HANDLE)Handle)))
//     {
//         return Socket;
//     }
//     else
//     {
//         /* WAH didn't find it, use IFS */
//         return WsSockFindIfsSocket(Handle);
//     }
// }
//
// INT
// __stdcall
// WsSockAddApiReference(IN SOCKET Handle)
// {
//     PWSSOCKET Socket;
//
//     /* Get the Socket now */
//     if ((Socket = WsSockGetSocketNoExport(Handle)))
//     {
//         /* Mark it as an API Socket */
//         if (Socket) Socket->ApiSocket = TRUE;
//
//         /* Remove a reference and return */
//         WsSockDereference(Socket);
//         return ERROR_SUCCESS;
//     }
//
//     /* Return error */
//     return WSASYSCALLFAILURE;
// }
//
// BOOL
// __stdcall
// WsSockDeleteSockets(IN LPVOID Context,
//                     IN PWAH_HANDLE Handle)
// {
//     /* Call the detach routine */
//     return WsProcDetachSocket((PWSPROCESS)Context, Handle);
// }
//
// VOID
// __stdcall
// WsSockDelete(IN PWSSOCKET Socket)
// {
//     /* Check if we have a catalog entry */
//     if (Socket->CatalogEntry)
//     {
//         /* Dereference it */
//         WsTcEntryDereference(Socket->CatalogEntry);
//         Socket->CatalogEntry = NULL;
//     }
//
//     /* Delete us */
//     HeapFree(WsSockHeap, 0, Socket);
// }
//
// VOID
// __stdcall
// WsSockDereference(IN PWSSOCKET Socket)
// {
//     /* Dereference and check if it's now 0 */
//     if (!(InterlockedDecrement(&Socket->RefCount)))
//     {
//         /* We can delete the Provider now */
//         WsSockDelete(Socket);
//     }
// }
//
// INT
// __stdcall
// WsSockDisassociateHandle(IN PWSSOCKET Socket)
// {
//     /* Remove it from the list */
//     return WahRemoveHandleContext(WsSockHandleTable, (PWAH_HANDLE)Socket);
// }
//
// INT
// __stdcall
// WsSockAssociateHandle(IN PWSSOCKET Socket,
//                       IN SOCKET Handle,
//                       IN BOOLEAN IsProvider)
// {
//     INT ErrorCode = ERROR_SUCCESS;
//     PWSSOCKET OldSocket;
//
//     /* Save the socket and provider */
//     Socket->IsProvider = IsProvider;
//     Socket->Handle = (HANDLE)Handle;
//
//     /* Insert it into the handle table */
//     OldSocket = (PWSSOCKET)WahInsertHandleContext(WsSockHandleTable,
//                                                   (PWAH_HANDLE)Socket);
//
//     /* Check if a socket already existed */
//     if (OldSocket != Socket)
//     {
//         /* We'll dereference it */
//         WsSockDereference(OldSocket);
//     }
//     else if (!OldSocket)
//     {
//         /* No memory to allocate it */
//         ErrorCode = WSAENOBUFS;
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

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

// No file-scope declarations to translate from this preamble.
// The #define NDEBUG is a preprocessor directive that affects compilation
// but does not produce any runtime declarations.

#[no_mangle]
pub unsafe extern "system" fn WPUCompleteOverlappedRequest(
    s: SOCKET,
    lpOverlapped: LPWSAOVERLAPPED,
    dwError: DWORD,
    cbTransferred: DWORD,
    lpErrno: LPINT,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUCompleteOverlappedRequest\0".as_ptr() as *const i8,
        b"dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const i8,
        29i32,
    );
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn WPUCloseEvent(
    hEvent: HANDLE,
    lpErrno: LPINT,
) -> BOOL {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUCloseEvent\0".as_ptr() as *const i8,
        b"dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const i8,
        41 as i32,
    );
    return (FALSE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WPUCloseThread(
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUCloseThread\0".as_ptr() as *const i8,
        b"dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const i8,
        53i32,
    );
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn WPUCreateEvent(lpErrno: LPINT) -> HANDLE {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUCreateEvent\0".as_ptr() as *const i8,
        b"reactos/dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const i8,
        64i32,
    );
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn WPUOpenCurrentThread(
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUOpenCurrentThread\0".as_ptr() as *const i8,
        b"dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const i8,
        76 as INT,
    );
    return 0 as INT;
}

#[no_mangle]
pub unsafe extern "system" fn WPUPostMessage(
    hWnd: HWND,
    Msg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> i32 {
    if WsSockPostRoutine.is_none() {
        WsSockPostRoutine = Some(PostMessage);
    }
    return WsSockPostRoutine.unwrap_unchecked()(hWnd, Msg, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn WPUQueryBlockingCallback(
    dwCatalogEntryId: u32,
    lplpfnCallback: *mut LPBLOCKINGCALLBACK,
    lpdwContext: PDWORD_PTR,
    lpErrno: LPINT,
) -> i32 {
    let mut Process: PWSPROCESS = core::ptr::null_mut();
    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut Catalog: PTCATALOG = core::ptr::null_mut();
    let mut ErrorCode: i32;
    let mut Status: i32;
    let mut Callback: LPBLOCKINGCALLBACK = None;
    let mut Entry: PTCATALOG_ENTRY = core::ptr::null_mut();
    let mut Context: DWORD_PTR = 0;

    ErrorCode = WsApiProlog(core::ptr::addr_of_mut!(Process), core::ptr::addr_of_mut!(Thread));
    if ErrorCode == ERROR_SUCCESS as i32 {
        Callback = (*Thread).BlockingCallback;
        if Callback.is_some() {
            Catalog = WsProcGetTCatalog(Process);
            ErrorCode = (WsTcGetEntryFromCatalogEntryId(
                Catalog,
                dwCatalogEntryId,
                core::ptr::addr_of_mut!(Entry),
            ) as i32);
            if ErrorCode == ERROR_SUCCESS as i32 {
                Context = (*(*Entry).Provider).Service.lpWSPCancelBlockingCall
                    .map_or(0, |f| f as usize) as DWORD_PTR;
                WsTcEntryDereference(Entry);
            }
        }
    }

    if ErrorCode == ERROR_SUCCESS as i32 {
        Status = ERROR_SUCCESS as i32;
    } else {
        Status = SOCKET_ERROR;
        Callback = None;
    }

    *lpdwContext = Context;
    *lpErrno = ErrorCode;
    *lplpfnCallback = Callback;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn WPUQueueApc(
    lpThreadId: LPWSATHREADID,
    lpfnUserApc: LPWSAUSERAPC,
    dwContext: DWORD_PTR,
    lpErrno: LPINT,
) -> ::core::ffi::c_int {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUQueueApc\0".as_ptr() as *const ::core::ffi::c_char,
        b"dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const ::core::ffi::c_char,
        178 as ::core::ffi::c_int,
    );
    return 0 as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "system" fn WPUResetEvent(
    hEvent: HANDLE,
    lpErrno: LPINT,
) -> BOOL {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUResetEvent\0".as_ptr() as *const ::core::ffi::c_char,
        b"reactos/dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const ::core::ffi::c_char,
        190 as ::core::ffi::c_int,
    );
    return (FALSE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn WPUSetEvent(
    hEvent: HANDLE,
    lpErrno: LPINT,
) -> BOOL {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WPUSetEvent\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/ws2_32/src/sputil.c\0".as_ptr() as *const core::ffi::c_char,
        202 as core::ffi::c_int,
    );
    return (FALSE as i32);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/sputil.c
//  * PURPOSE:     Transport Service Provider Utility Functions
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
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUCompleteOverlappedRequest(IN SOCKET s,
//                              IN LPWSAOVERLAPPED lpOverlapped,
//                              IN DWORD dwError,
//                              IN DWORD cbTransferred,
//                              OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// __stdcall
// WPUCloseEvent(IN WSAEVENT hEvent,
//               OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUCloseThread(IN LPWSATHREADID lpThreadId,
//                OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// WSAEVENT
// __stdcall
// WPUCreateEvent(OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return (WSAEVENT)0;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUOpenCurrentThread(OUT LPWSATHREADID lpThreadId,
//                      OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// WPUPostMessage(IN HWND hWnd,
//                IN UINT Msg,
//                IN WPARAM wParam,
//                IN LPARAM lParam)
// {
//     /* Make sure we have a post routine */
//     if (!WsSockPostRoutine) WsSockPostRoutine = PostMessage;
//
//     /* Call it */
//     return WsSockPostRoutine(hWnd, Msg, wParam, lParam);
// }
//
// /*
//  * @implemented
//  */
// INT
// __stdcall
// WPUQueryBlockingCallback(IN DWORD dwCatalogEntryId,
//                          OUT LPBLOCKINGCALLBACK * lplpfnCallback,
//                          OUT PDWORD_PTR lpdwContext,
//                          OUT LPINT lpErrno)
// {
//     PWSPROCESS Process;
//     PWSTHREAD Thread;
//     PTCATALOG Catalog;
//     INT ErrorCode;
//     INT Status;
//     LPBLOCKINGCALLBACK Callback = NULL;
//     PTCATALOG_ENTRY Entry;
//     DWORD_PTR Context = 0;
//     DPRINT("WPUQueryBlockingCallback: %lx \n", dwCatalogEntryId);
//
//     /* Enter prolog */
//     if ((ErrorCode = WsApiProlog(&Process, &Thread)) == ERROR_SUCCESS)
//     {
//         /* Get the callback function */
//         Callback = Thread->BlockingCallback;
//
//         /* Check if there is one */
//         if (Callback)
//         {
//             /* Get the catalog */
//             Catalog = WsProcGetTCatalog(Process);
//
//             /* Find the entry for this ID */
//             ErrorCode = WsTcGetEntryFromCatalogEntryId(Catalog,
//                                                        dwCatalogEntryId,
//                                                        &Entry);
//
//             /* Check for success */
//             if (ErrorCode == ERROR_SUCCESS)
//             {
//                 /* Get the context */
//                 Context = (DWORD_PTR)Entry->Provider->Service.lpWSPCancelBlockingCall;
//
//                 /* Dereference the entry */
//                 WsTcEntryDereference(Entry);
//             }
//         }
//     }
//
//     /* Check error code */
//     if (ErrorCode == ERROR_SUCCESS)
//     {
//         /* Return success as well */
//         Status = ERROR_SUCCESS;
//     }
//     else
//     {
//         /* Return expected value and no callback */
//         Status = SOCKET_ERROR;
//         Callback = NULL;
//     }
//
//     /* Return the settings */
//     *lpdwContext = Context;
//     *lpErrno = ErrorCode;
//     *lplpfnCallback = Callback;
//
//     /* Return to caller */
//     return Status;
// }
//
// /*
//  * @unimplemented
//  */
// INT
// __stdcall
// WPUQueueApc(IN LPWSATHREADID lpThreadId,
//             IN LPWSAUSERAPC lpfnUserApc,
//             IN DWORD_PTR dwContext,
//             OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// __stdcall
// WPUResetEvent(IN WSAEVENT hEvent,
//               OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// __stdcall
// WPUSetEvent(IN WSAEVENT hEvent,
//             OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

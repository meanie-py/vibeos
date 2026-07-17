use crate::prelude::*;
use crate::types::*;
use core::ffi::c_void;
use crate::src::dprocess::WsProcGetAsyncHelper;


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
pub unsafe extern "system" fn WsThreadDefaultBlockingHook() -> DWORD {

    let mut Message: MSG = core::mem::zeroed();
    let mut GotMessage: i32 = 0;

    GotMessage = PeekMessageA(
        &mut Message,
        core::ptr::null_mut(),
        0,
        0,
        PM_REMOVE,
    );

    if GotMessage != 0 {
        TranslateMessage(&Message);
        DispatchMessageA(&Message);
    }

    GotMessage as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadBlockingCallback(Context: DWORD_PTR) -> BOOL {

    let Thread: PWSTHREAD = TlsGetValue(TlsIndex) as PWSTHREAD;

    (*Thread).Blocking = 1u8;
    (*Thread).CancelBlockingCall = core::mem::transmute::<DWORD_PTR, LPWSPCANCELBLOCKINGCALL>(Context);
    (*Thread).Cancelled = 0u8;

    while core::mem::transmute::<_, fn() -> INT_PTR>(
        (*Thread).BlockingHook.unwrap_unchecked()
    )() != 0 {}

    (*Thread).Blocking = 0u8;

    return ((*Thread).Cancelled == 0) as BOOL;
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn WsThreadSetBlockingHook(
    Thread: PWSTHREAD,
    BlockingHook: FARPROC,
) -> FARPROC {
    let OldHook: FARPROC = (*Thread).BlockingHook;

    if BlockingHook == as_farproc(WsThreadDefaultBlockingHook as *const ()) {
        (*Thread).BlockingCallback = None;
    } else {
        (*Thread).BlockingCallback = as_blocking_callback(WsThreadBlockingCallback as *const ());
    }

    (*Thread).BlockingHook = BlockingHook;
    OldHook
}
#[no_mangle]
pub unsafe extern "system" fn WsThreadUnhookBlockingHook(Thread: PWSTHREAD) -> DWORD {
    (*Thread).BlockingHook = as_farproc(WsThreadDefaultBlockingHook as *const ());
    (*Thread).BlockingCallback = None;
    ERROR_SUCCESS as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadCancelBlockingCall(Thread: PWSTHREAD) -> DWORD {
    let mut ErrorCode: INT = 0;
    let mut ReturnValue: INT = 0;

    if (*Thread).Blocking == 0 {
        return WSAEINVAL as DWORD;
    }

    if (*Thread).Cancelled == 0 {
        ReturnValue = (*Thread)
            .CancelBlockingCall
            .unwrap_unchecked()(core::ptr::addr_of_mut!(ErrorCode)) as INT;
        if ReturnValue != (ERROR_SUCCESS as i32) {
            return ErrorCode as DWORD;
        }
        (*Thread).Cancelled = 1u8;
    }

    return ERROR_SUCCESS as DWORD;
}
#[no_mangle]
pub unsafe extern "system" fn WsThreadGetProtoBuffer(Thread: PWSTHREAD) -> PWSPROTO_BUFFER {
    if (*Thread).ProtocolInfo.is_null() {
        (*Thread).ProtocolInfo = HeapAlloc(
            WsSockHeap,
            0,
            (core::mem::size_of::<WSPROTO_BUFFER>() as SIZE_T),
        );
    }
    (*Thread).ProtocolInfo as PWSPROTO_BUFFER
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadAllocate() -> PWSTHREAD {
    let mut Thread: PWSTHREAD;

    Thread = HeapAlloc(
        WsSockHeap,
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<_WSTHREAD>() as SIZE_T),
    ) as PWSTHREAD;

    if !Thread.is_null() {
        (*Thread).BlockingHook = as_farproc(WsThreadDefaultBlockingHook as *const ());
    }

    return Thread;
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadStartup() -> DWORD {
    let mut ErrorCode: INT = WSASYSCALLFAILURE as INT;
    if GlobalTlsIndex != TLS_OUT_OF_INDEXES {
        ErrorCode = ERROR_SUCCESS as INT;
    }
    return ErrorCode as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadCleanup() {
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadInitialize(
    Thread: PWSTHREAD,
    Process: PWSPROCESS,
) -> DWORD {
    let mut ErrorCode: i32 = (WSASYSCALLFAILURE as i32);

    (*Thread).Process = Process;

    if WsProcGetAsyncHelper(Process, core::ptr::addr_of_mut!((*Thread).AsyncHelper)) == (ERROR_SUCCESS as i32) {
        if WahOpenCurrentThread((*Thread).AsyncHelper, core::ptr::addr_of_mut!((*Thread).WahThreadId)) == ERROR_SUCCESS {
            ErrorCode = ERROR_SUCCESS as i32;
        }
    }

    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadDelete(Thread: PWSTHREAD) {
    (*Thread).BlockingHook = None;

    if !(*Thread).Hostent.is_null() {
        HeapFree(WsSockHeap, 0, (*Thread).Hostent as *mut core::ffi::c_void);
    }
    if !(*Thread).Servent.is_null() {
        HeapFree(WsSockHeap, 0, (*Thread).Servent as *mut core::ffi::c_void);
    }
    if !(*Thread).ProtocolInfo.is_null() {
        HeapFree(WsSockHeap, 0, (*Thread).ProtocolInfo as *mut core::ffi::c_void);
    }

    TlsSetValue(TlsIndex, core::ptr::null_mut());

    WahCloseThread((*Thread).AsyncHelper, core::ptr::addr_of_mut!((*Thread).WahThreadId));

    (*Thread).Process = core::ptr::null_mut();
    HeapFree(WsSockHeap, 0, Thread as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadDestroyCurrentThread() {
    let mut Thread: PWSTHREAD = core::ptr::null_mut();

    if TlsIndex != TLS_OUT_OF_INDEXES {
        Thread = TlsGetValue(TlsIndex) as PWSTHREAD;
        if !Thread.is_null() {
            WsThreadDelete(Thread);
            TlsSetValue(TlsIndex, 0 as *mut core::ffi::c_void);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadCreate(
    Process: PWSPROCESS,
    CurrentThread: *mut PWSTHREAD,
) -> DWORD {

    let mut Thread: PWSTHREAD = core::ptr::null_mut();
    let mut ErrorCode: i32 = WSASYSCALLFAILURE as i32;

    if TlsIndex != TLS_OUT_OF_INDEXES {
        Thread = WsThreadAllocate();
        if !Thread.is_null() {
            if WsThreadInitialize(Thread, Process) == ERROR_SUCCESS {
                if TlsSetValue(TlsIndex, Thread as *mut core::ffi::c_void) != 0 {
                    *CurrentThread = Thread;
                    ErrorCode = ERROR_SUCCESS as i32;
                }
            }

            if ErrorCode != (ERROR_SUCCESS as i32) {
                WsThreadDelete(Thread);
            }
        }
    }

    ErrorCode as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadGetCurrentThread(
    Process: PWSPROCESS,
    Thread: *mut PWSTHREAD,
) -> DWORD {
    *Thread = TlsGetValue(GlobalTlsIndex) as PWSTHREAD;
    if !(*Thread).is_null() {
        return ERROR_SUCCESS;
    } else {
        return WsThreadCreate(Process, Thread);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadGetThreadId(Process: PWSPROCESS) -> LPWSATHREADID {

    let Thread: PWSTHREAD = TlsGetValue(TlsIndex) as PWSTHREAD;
    if !Thread.is_null() {
        core::ptr::addr_of_mut!((*Thread).WahThreadId)
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn WsThreadBlobToHostent(
    Thread: PWSTHREAD,
    Blob: LPBLOB,
) -> PHOSTENT {
    if (*Thread).HostentSize < (*Blob).cbSize {
        HeapFree(WsSockHeap, 0, (*Thread).Hostent as *mut core::ffi::c_void);
        (*Thread).Hostent = HeapAlloc(WsSockHeap, 0, (((*Blob).cbSize as usize as u64)) as SIZE_T) as PCHAR;
        (*Thread).HostentSize = (*Blob).cbSize;
    }

    if !(*Thread).Hostent.is_null() {
        core::ptr::copy_nonoverlapping(
            (*Blob).pBlobData as *const u8,
            (*Thread).Hostent as *mut u8,
            (*Blob).cbSize as usize,
        );
    } else {
        (*Thread).HostentSize = 0;
        SetLastError(WSA_NOT_ENOUGH_MEMORY);
    }

    (*Thread).Hostent as PHOSTENT
}
#[no_mangle]
pub unsafe extern "system" fn WsThreadBlobToServent(
    Thread: PWSTHREAD,
    Blob: LPBLOB,
) -> PSERVENT {
    if (*Thread).ServentSize < (*Blob).cbSize {
        HeapFree(WsSockHeap, 0, (*Thread).Servent as PVOID);
        (*Thread).Servent = HeapAlloc(WsSockHeap, 0, (*Blob).cbSize as SIZE_T) as PCHAR;
        (*Thread).ServentSize = (*Blob).cbSize;
    }

    if !(*Thread).Servent.is_null() {
        core::ptr::copy_nonoverlapping(
            (*Blob).pBlobData as *const u8,
            (*Thread).Servent as *mut u8,
            (*Blob).cbSize as usize,
        );
    } else {
        (*Thread).ServentSize = 0;
        SetLastError(WSA_NOT_ENOUGH_MEMORY);
    }

    (*Thread).Servent as PSERVENT
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock 2 API
//  * FILE:        dll/win32/ws2_32/src/dthread.c
//  * PURPOSE:     Thread Object
//  * PROGRAMMER:  Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ws2_32.h>
//
// /* FUNCTIONS *****************************************************************/
//
// DWORD
// __stdcall
// WsThreadDefaultBlockingHook(VOID)
// {
//     MSG Message;
//     BOOL GotMessage = FALSE;
//
//     /* Get the message */
//     GotMessage = PeekMessage(&Message, NULL, 0, 0, PM_REMOVE);
//
//     /* Check if we got one */
//     if (GotMessage)
//     {
//         /* Process it */
//         TranslateMessage(&Message);
//         DispatchMessage(&Message);
//     }
//
//     /* return */
//     return GotMessage;
// }
//
// BOOL
// __stdcall
// WsThreadBlockingCallback(IN DWORD_PTR Context)
// {
//     PWSTHREAD Thread = TlsGetValue(TlsIndex);
//
//     /* Set thread as blocking, set cancel callback and the clear cancel flag */
//     Thread->Blocking = TRUE;
//     Thread->CancelBlockingCall = (LPWSPCANCELBLOCKINGCALL)Context;
//     Thread->Cancelled = FALSE;
//
//     /* Call the blocking hook */
//     while (Thread->BlockingHook());
//
//     /* We're not blocking anymore */
//     Thread->Blocking = FALSE;
//
//     /* Return whether or not we were cancelled */
//     return !Thread->Cancelled;
// }
//
// FARPROC
// __stdcall
// WsThreadSetBlockingHook(IN PWSTHREAD Thread,
//                         IN FARPROC BlockingHook)
// {
//     FARPROC OldHook = Thread->BlockingHook;
//
//     /* Check if we're resetting to our default hook */
//     if (BlockingHook == (FARPROC)WsThreadDefaultBlockingHook)
//     {
//         /* Clear out the blocking callback */
//         Thread->BlockingCallback = NULL;
//     }
//     else
//     {
//         /* Set the blocking callback */
//         Thread->BlockingCallback = WsThreadBlockingCallback;
//     }
//
//     /* Set the new blocking hook and return the previous */
//     Thread->BlockingHook = BlockingHook;
//     return OldHook;
// }
//
// DWORD
// __stdcall
// WsThreadUnhookBlockingHook(IN PWSTHREAD Thread)
// {
//     /* Reset the hook to the default, and remove the callback */
//     Thread->BlockingHook = (FARPROC)WsThreadDefaultBlockingHook;
//     Thread->BlockingCallback = NULL;
//
//     /* Return success */
//     return ERROR_SUCCESS;
// }
//
// DWORD
// __stdcall
// WsThreadCancelBlockingCall(IN PWSTHREAD Thread)
// {
//     INT ErrorCode, ReturnValue;
//
//     /* Make sure that the Thread is really in a blocking call */
//     if (!Thread->Blocking) return WSAEINVAL;
//
//     /* Make sure we haven't already been cancelled */
//     if (!Thread->Cancelled)
//     {
//         /* Call the cancel procedure */
//         ReturnValue = Thread->CancelBlockingCall(&ErrorCode);
//         if (ReturnValue != ERROR_SUCCESS) return ErrorCode;
//
//         /* Set us as cancelled */
//         Thread->Cancelled = TRUE;
//     }
//
//     /* Success */
//     return ERROR_SUCCESS;
// }
//
// PWSPROTO_BUFFER
// __stdcall
// WsThreadGetProtoBuffer(IN PWSTHREAD Thread)
// {
//     /* See if it already exists */
//     if (!Thread->ProtocolInfo)
//     {
//         /* We don't have a buffer; allocate it */
//         Thread->ProtocolInfo = HeapAlloc(WsSockHeap, 0, sizeof(WSPROTO_BUFFER));
//     }
//
//     /* Return it */
//     return Thread->ProtocolInfo;
// }
//
// PWSTHREAD
// __stdcall
// WsThreadAllocate(VOID)
// {
//     PWSTHREAD Thread;
//
//     /* Allocate the object */
//     Thread = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, sizeof(*Thread));
//     if (Thread)
//     {
//         /* Set non-zero data */
//         Thread->BlockingHook = (FARPROC)WsThreadDefaultBlockingHook;
//     }
//
//     /* Return it */
//     return Thread;
// }
//
// DWORD
// __stdcall
// WsThreadStartup(VOID)
// {
//     INT ErrorCode = WSASYSCALLFAILURE;
//
//     /* Check if we have a valid TLS */
//     if (TlsIndex != TLS_OUT_OF_INDEXES)
//     {
//         /* TLS was already OK */
//         ErrorCode = ERROR_SUCCESS;
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsThreadCleanup(VOID)
// {
// }
//
// DWORD
// __stdcall
// WsThreadInitialize(IN PWSTHREAD Thread,
//                    IN PWSPROCESS Process)
// {
//     INT ErrorCode = WSASYSCALLFAILURE;
//
//     /* Set the process */
//     Thread->Process = Process;
//
//     /* Get the helper device */
//     if ((WsProcGetAsyncHelper(Process, &Thread->AsyncHelper)) == ERROR_SUCCESS)
//     {
//         /* Initialize a WAH Thread ID */
//         if ((WahOpenCurrentThread(Thread->AsyncHelper,
//                                   &Thread->WahThreadId)) == ERROR_SUCCESS)
//         {
//             /* Success */
//             ErrorCode = ERROR_SUCCESS;
//         }
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// VOID
// __stdcall
// WsThreadDelete(IN PWSTHREAD Thread)
// {
//     /* Remove the blocking hook */
//     Thread->BlockingHook = NULL;
//
//     /* Free our buffers */
//     if (Thread->Hostent) HeapFree(WsSockHeap, 0, Thread->Hostent);
//     if (Thread->Servent) HeapFree(WsSockHeap, 0, Thread->Servent);
//     if (Thread->ProtocolInfo) HeapFree(WsSockHeap, 0, Thread->ProtocolInfo);
//
//     /* Clear the TLS */
//     TlsSetValue(TlsIndex, NULL);
//
//     /* Close the WAH Handle */
//     WahCloseThread(Thread->AsyncHelper, &Thread->WahThreadId);
//
//     /* Unlink the process and free us */
//     Thread->Process = NULL;
//     HeapFree(WsSockHeap, 0, Thread);
// }
//
// VOID
// __stdcall
// WsThreadDestroyCurrentThread(VOID)
// {
//     PWSTHREAD Thread;
//
//     /* Make sure we have TLS */
//     if (TlsIndex != TLS_OUT_OF_INDEXES)
//     {
//         /* Get the thread */
//         if ((Thread = TlsGetValue(TlsIndex)))
//         {
//             /* Delete it */
//             WsThreadDelete(Thread);
// 			TlsSetValue(TlsIndex, 0);
//         }
//     }
// }
//
// DWORD
// __stdcall
// WsThreadCreate(IN PWSPROCESS Process,
//                IN PWSTHREAD *CurrentThread)
// {
//     PWSTHREAD Thread = NULL;
//     INT ErrorCode = WSASYSCALLFAILURE;
//
//     /* Make sure we have TLS */
//     if (TlsIndex != TLS_OUT_OF_INDEXES)
//     {
//         /* Allocate the thread */
//         if ((Thread = WsThreadAllocate()))
//         {
//             /* Initialize it */
//             if (WsThreadInitialize(Thread, Process) == ERROR_SUCCESS)
//             {
//                 /* Set the TLS */
//                 if (TlsSetValue(TlsIndex, Thread))
//                 {
//                     /* Return it and success */
//                     *CurrentThread = Thread;
//                     ErrorCode = ERROR_SUCCESS;
//                 }
//             }
//
//             /* Check for any failures */
//             if (ErrorCode != ERROR_SUCCESS) WsThreadDelete(Thread);
//         }
//     }
//
//     /* Return */
//     return ErrorCode;
// }
//
// DWORD
// __stdcall
// WsThreadGetCurrentThread(IN PWSPROCESS Process,
//                          IN PWSTHREAD *Thread)
// {
//     /* Get the thread */
//     if ((*Thread = TlsGetValue(TlsIndex)))
//     {
//         /* Success */
//         return ERROR_SUCCESS;
//     }
//     else
//     {
//         /* We failed, initialize it */
//         return WsThreadCreate(Process, Thread);
//     }
// }
//
// LPWSATHREADID
// __stdcall
// WsThreadGetThreadId(IN PWSPROCESS Process)
// {
//     PWSTHREAD Thread;
//
//     /* Get the thread */
//     if ((Thread = TlsGetValue(TlsIndex)))
//     {
//         /* Return the ID */
//         return &Thread->WahThreadId;
//     }
//     else
//     {
//         /* Not a valid thread */
//         return NULL;
//     }
// }
//
// PHOSTENT
// __stdcall
// WsThreadBlobToHostent(IN PWSTHREAD Thread,
//                       IN LPBLOB Blob)
// {
//     /* Check if our buffer is too small */
//     if (Thread->HostentSize < Blob->cbSize)
//     {
//         /* Delete the current buffer and allocate a new one */
//         HeapFree(WsSockHeap, 0, Thread->Hostent);
//         Thread->Hostent = HeapAlloc(WsSockHeap, 0, Blob->cbSize);
//
//         /* Set the new size */
//         Thread->HostentSize = Blob->cbSize;
//     }
//
//     /* Do we have a buffer? */
//     if (Thread->Hostent)
//     {
//         /* Copy the data inside */
//         RtlMoveMemory(Thread->Hostent, Blob->pBlobData, Blob->cbSize);
//     }
//     else
//     {
//         /* No buffer space! */
//         Thread->HostentSize = 0;
//         SetLastError(WSA_NOT_ENOUGH_MEMORY);
//     }
//
//     /* Return the buffer */
//     return (PHOSTENT)Thread->Hostent;
// }
//
// PSERVENT
// __stdcall
// WsThreadBlobToServent(IN PWSTHREAD Thread,
//                       IN LPBLOB Blob)
// {
//     /* Check if our buffer is too small */
//     if (Thread->ServentSize < Blob->cbSize)
//     {
//         /* Delete the current buffer and allocate a new one */
//         HeapFree(WsSockHeap, 0, Thread->Servent);
//         Thread->Servent = HeapAlloc(WsSockHeap, 0, Blob->cbSize);
//
//         /* Set the new size */
//         Thread->ServentSize = Blob->cbSize;
//     }
//
//     /* Do we have a buffer? */
//     if (Thread->Servent)
//     {
//         /* Copy the data inside */
//         RtlMoveMemory(Thread->Servent, Blob->pBlobData, Blob->cbSize);
//     }
//     else
//     {
//         /* No buffer space! */
//         Thread->ServentSize = 0;
//         SetLastError(WSA_NOT_ENOUGH_MEMORY);
//     }
//
//     /* Return the buffer */
//     return (PSERVENT)Thread->Servent;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

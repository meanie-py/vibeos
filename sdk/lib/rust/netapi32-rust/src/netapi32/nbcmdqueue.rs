use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_netbios: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netbios\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netbios as *mut __wine_debug_channel };

#[repr(C)]
pub struct NBCmdQueue {
    pub heap: HANDLE,
    pub cs: CRITICAL_SECTION,
    pub head: PNCB,
}
pub type _NBCmdQueue = NBCmdQueue; // the C used the tag name `struct NBCmdQueue`; c2rust also refers to `_NBCmdQueue`

#[no_mangle]
pub unsafe extern "C" fn NBCmdQueueCreate(mut heap: HANDLE) -> *mut _NBCmdQueue {
    let mut queue: *mut _NBCmdQueue = core::ptr::null_mut();

    if heap.is_null() {
        heap = GetProcessHeap();
    }
    queue = HeapAlloc(
        heap,
        0 as DWORD,
        core::mem::size_of::<_NBCmdQueue>() as SIZE_T,
    ) as *mut _NBCmdQueue;

    if !queue.is_null() {
        (*queue).heap = heap;
        InitializeCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
        (*(*queue).cs.DebugInfo).__bindgen_anon_1.Spare[0usize] =
            b"dll/win32/netapi32/nbcmdqueue.c: NBCmdQueue.cs\0"
                .as_ptr() as DWORD_PTR;
        (*queue).head = core::ptr::null_mut::<_NCB>();
    }

    queue
}
#[no_mangle]
pub unsafe extern "C" fn NBCmdQueueAdd(queue: *mut NBCmdQueue, ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR;

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbcmdqueue.c\0".as_ptr() as *const i8,
            b"NBCmdQueueAdd\0".as_ptr() as *const i8,
            74,
            b": queue %p, ncb %p\n\0".as_ptr() as *const i8,
            queue as isize,
            ncb as isize,
        );
    }

    if queue.is_null() {
        return NRC_BADDR as UCHAR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS as UCHAR;
    }

    let cancel_event_ptr: PHANDLE = &raw mut (*ncb).ncb_reserve as *mut u8 as PHANDLE;
    *cancel_event_ptr = core::ptr::null_mut();

    EnterCriticalSection(&raw mut (*queue).cs);

    let next_ptr: *mut PNCB = (&raw mut (*ncb).ncb_reserve as *mut u8)
        .add(core::mem::size_of::<HANDLE>()) as *mut PNCB;
    *next_ptr = (*queue).head;
    (*queue).head = ncb;

    ret = NRC_GOODRET as UCHAR;

    LeaveCriticalSection(&raw mut (*queue).cs);

    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbcmdqueue.c\0".as_ptr() as *const i8,
            b"NBCmdQueueAdd\0".as_ptr() as *const i8,
            87,
            b"returning 0x%02x\n\0".as_ptr() as *const i8,
            ret as i32,
        );
    }

    ret
}
unsafe extern "C" fn NBCmdQueueFindNBC(queue: *mut NBCmdQueue, ncb: PNCB) -> *mut PNCB {
    if queue.is_null() || ncb.is_null() {
        core::ptr::null_mut()
    } else {
        let mut ret = core::ptr::addr_of_mut!((*queue).head);
        while !ret.is_null() && *ret != ncb {
            ret = (core::ptr::addr_of_mut!((**ret).ncb_reserve) as *mut u8)
                .add(core::mem::size_of::<HANDLE>()) as *mut PNCB;
        }
        ret
    }
}
#[no_mangle]
pub unsafe extern "C" fn NBCmdQueueCancel(queue: *mut NBCmdQueue, ncb: PNCB) -> UCHAR {
    let mut ret: UCHAR;
    let spot: *mut PNCB;

    if queue.is_null() {
        return NRC_BADDR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS;
    }

    EnterCriticalSection(&raw mut (*queue).cs);
    spot = NBCmdQueueFindNBC(queue, ncb);
    if !spot.is_null() {
        let cancel_event_ptr = &raw mut (*(*spot)).ncb_reserve as *mut HANDLE;
        *cancel_event_ptr = CreateEventW(
            core::ptr::null_mut(),
            0i32,
            0i32,
            core::ptr::null_mut(),
        );
        WaitForSingleObject(*cancel_event_ptr, INFINITE);
        CloseHandle(*cancel_event_ptr);

        let next_ptr = (&raw mut (*(*spot)).ncb_reserve as *mut u8)
            .add(core::mem::size_of::<HANDLE>())
            as *mut PNCB;
        *spot = *next_ptr;

        if (*ncb).ncb_retcode == NRC_CMDCAN {
            ret = NRC_CMDCAN;
        } else {
            ret = NRC_CANOCCR;
        }
    } else {
        ret = NRC_INVADDRESS;
    }
    LeaveCriticalSection(&raw mut (*queue).cs);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NBCmdQueueComplete(
    queue: *mut NBCmdQueue,
    ncb: PNCB,
    retcode: UCHAR,
) -> UCHAR {
    if queue.is_null() {
        return NRC_BADDR;
    }
    if ncb.is_null() {
        return NRC_INVADDRESS;
    }
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbcmdqueue.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NBCmdQueueComplete\0".as_ptr() as *const core::ffi::c_char,
            143i32,
            b": queue %p, ncb %p\n\0".as_ptr() as *const core::ffi::c_char,
            queue,
            ncb,
        );
    }
    EnterCriticalSection(&raw mut (*queue).cs);
    let spot: *mut PNCB = NBCmdQueueFindNBC(queue, ncb);
    let ret: UCHAR;
    if !spot.is_null() {
        let cancel_handle: PHANDLE = &raw mut (**spot).ncb_reserve as *mut UCHAR as PHANDLE;
        if !(*cancel_handle).is_null() {
            SetEvent(*cancel_handle);
        } else {
            let next_ptr: *mut PNCB = (&raw mut (**spot).ncb_reserve as *mut UCHAR)
                .offset(core::mem::size_of::<HANDLE>() as isize) as *mut PNCB;
            *spot = *next_ptr;
        }
        ret = NRC_GOODRET;
    } else {
        ret = NRC_INVADDRESS;
    }
    LeaveCriticalSection(&raw mut (*queue).cs);
    if (*__wine_dbch___default).flags as u32 & (1u32 << (__WINE_DBCL_TRACE as u32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbcmdqueue.c\0"
                .as_ptr() as *const core::ffi::c_char,
            b"NBCmdQueueComplete\0".as_ptr() as *const core::ffi::c_char,
            163i32,
            b"returning 0x%02x\n\0".as_ptr() as *const core::ffi::c_char,
            ret as core::ffi::c_int,
        );
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn NBCmdQueueCancelAll(queue: *mut NBCmdQueue) -> UCHAR {
    if queue.is_null() {
        return NRC_BADDR as UCHAR;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    while !(*queue).head.is_null() {
        NBCmdQueueCancel(queue, (*queue).head);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*queue).cs));
    NRC_GOODRET as UCHAR
}
#[no_mangle]
pub unsafe extern "C" fn NBCmdQueueDestroy(queue: *mut NBCmdQueue) {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/netapi32/nbcmdqueue.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            b"NBCmdQueueDestroy\0".as_ptr() as *const ::core::ffi::c_char,
            191 as ::core::ffi::c_int,
            b": queue %p\n\0".as_ptr() as *const ::core::ffi::c_char,
            queue,
        );
    }
    if !queue.is_null() {
        NBCmdQueueCancelAll(queue);
        (*(*queue).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = 0 as DWORD_PTR;
        DeleteCriticalSection(&raw mut (*queue).cs);
        HeapFree((*queue).heap, 0u32, queue as *mut ::core::ffi::c_void);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /* Copyright (c) 2003 Juan Lang
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include "netapi32.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(netbios);
//
// struct NBCmdQueue
// {
//     HANDLE           heap;
//     CRITICAL_SECTION cs;
//     PNCB             head;
// };
//
// #define CANCEL_EVENT_PTR(ncb) (PHANDLE)((ncb)->ncb_reserve)
// #define NEXT_PTR(ncb) (PNCB *)((ncb)->ncb_reserve + sizeof(HANDLE))
//
// /* The reserved area of an ncb will be used for the following data:
//  * - a cancelled flag (BOOL, 4 bytes??)
//  * - a handle to an event that's set by a cancelled command on completion
//  *   (HANDLE, 4 bytes)
//  * These members are used in the following way
//  * - on cancel, set the event member of the reserved field (with create event)
//  * - NBCmdComplete will delete the ncb from the queue of there's no event;
//  *   otherwise it will set the event and not delete the ncb
//  * - cancel must lock the queue before finding the ncb in it, and can unlock it
//  *   once it's set the event (and the cancelled flag)
//  * - NBCmdComplete must lock the queue before attempting to remove the ncb or
//  *   check the event
//  * - NBCmdQueueCancelAll will lock the queue, and cancel all ncb's in the queue.
//  *   It'll then unlock the queue, and wait on the event in the head of the queue
//  *   until there's no more ncb's in the queue.
//  * Space optimization: use the handle as a boolean.  NULL == 0 => not cancelled.
//  * Non-NULL == valid handle => cancelled.  This allows storing a next pointer
//  * in the ncb's reserved field as well, avoiding a memory alloc for a new
//  * command (cool).
//  */
//
// struct NBCmdQueue *NBCmdQueueCreate(HANDLE heap)
// {
//     struct NBCmdQueue *queue;
//
//     if (heap == NULL)
//         heap = GetProcessHeap();
//     queue = HeapAlloc(heap, 0, sizeof(struct NBCmdQueue));
//     if (queue)
//     {
//         queue->heap = heap;
//         InitializeCriticalSection(&queue->cs);
//         queue->cs.DebugInfo->Spare[0] = (DWORD_PTR)(__FILE__ ": NBCmdQueue.cs");
//         queue->head = NULL;
//     }
//     return queue;
// }
//
// UCHAR NBCmdQueueAdd(struct NBCmdQueue *queue, PNCB ncb)
// {
//     UCHAR ret;
//
//     TRACE(": queue %p, ncb %p\n", queue, ncb);
//
//     if (!queue)
//         return NRC_BADDR;
//     if (!ncb)
//         return NRC_INVADDRESS;
//
//     *CANCEL_EVENT_PTR(ncb) = NULL;
//     EnterCriticalSection(&queue->cs);
//     *NEXT_PTR(ncb) = queue->head;
//     queue->head = ncb;
//     ret = NRC_GOODRET;
//     LeaveCriticalSection(&queue->cs);
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// static PNCB *NBCmdQueueFindNBC(struct NBCmdQueue *queue, PNCB ncb)
// {
//     PNCB *ret;
//
//     if (!queue || !ncb)
//         ret = NULL;
//     else
//     {
//         ret = &queue->head;
//         while (ret && *ret != ncb)
//             ret = NEXT_PTR(*ret);
//     }
//     return ret;
// }
//
// UCHAR NBCmdQueueCancel(struct NBCmdQueue *queue, PNCB ncb)
// {
//     UCHAR ret;
//     PNCB *spot;
//
//     TRACE(": queue %p, ncb %p\n", queue, ncb);
//
//     if (!queue)
//         return NRC_BADDR;
//     if (!ncb)
//         return NRC_INVADDRESS;
//
//     EnterCriticalSection(&queue->cs);
//     spot = NBCmdQueueFindNBC(queue, ncb);
//     if (spot)
//     {
//         *CANCEL_EVENT_PTR(*spot) = CreateEventW(NULL, FALSE, FALSE, NULL);
//         WaitForSingleObject(*CANCEL_EVENT_PTR(*spot), INFINITE);
//         CloseHandle(*CANCEL_EVENT_PTR(*spot));
//         *spot = *NEXT_PTR(*spot);
//         if (ncb->ncb_retcode == NRC_CMDCAN)
//             ret = NRC_CMDCAN;
//         else
//             ret = NRC_CANOCCR;
//     }
//     else
//         ret = NRC_INVADDRESS;
//     LeaveCriticalSection(&queue->cs);
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// UCHAR NBCmdQueueComplete(struct NBCmdQueue *queue, PNCB ncb, UCHAR retcode)
// {
//     UCHAR ret;
//     PNCB *spot;
//
//     TRACE(": queue %p, ncb %p\n", queue, ncb);
//
//     if (!queue)
//         return NRC_BADDR;
//     if (!ncb)
//         return NRC_INVADDRESS;
//
//     EnterCriticalSection(&queue->cs);
//     spot = NBCmdQueueFindNBC(queue, ncb);
//     if (spot)
//     {
//         if (*CANCEL_EVENT_PTR(*spot))
//             SetEvent(*CANCEL_EVENT_PTR(*spot));
//         else
//             *spot = *NEXT_PTR(*spot);
//         ret = NRC_GOODRET;
//     }
//     else
//         ret = NRC_INVADDRESS;
//     LeaveCriticalSection(&queue->cs);
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// UCHAR NBCmdQueueCancelAll(struct NBCmdQueue *queue)
// {
//     UCHAR ret;
//
//     TRACE(": queue %p\n", queue);
//
//     if (!queue)
//         return NRC_BADDR;
//
//     EnterCriticalSection(&queue->cs);
//     while (queue->head)
//     {
//         TRACE(": waiting for ncb %p (command 0x%02x)\n", queue->head,
//          queue->head->ncb_command);
//         NBCmdQueueCancel(queue, queue->head);
//     }
//     LeaveCriticalSection(&queue->cs);
//     ret = NRC_GOODRET;
//     TRACE("returning 0x%02x\n", ret);
//     return ret;
// }
//
// void NBCmdQueueDestroy(struct NBCmdQueue *queue)
// {
//     TRACE(": queue %p\n", queue);
//
//     if (queue)
//     {
//         NBCmdQueueCancelAll(queue);
//         queue->cs.DebugInfo->Spare[0] = 0;
//         DeleteCriticalSection(&queue->cs);
//         HeapFree(queue->heap, 0, queue);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

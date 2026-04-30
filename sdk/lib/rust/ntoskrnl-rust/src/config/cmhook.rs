use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{
    _EX_CALLBACK, _EX_RUNDOWN_REF, _FAST_MUTEX, _LARGE_INTEGER, _LIST_ENTRY,
    LARGE_INTEGER,
    PEX_CALLBACK_FUNCTION, PLARGE_INTEGER, REG_NOTIFY_CLASS,
    ExAcquireFastMutex, ExAcquireRundownProtection, ExInitializeCallBack,
    ExInitializeFastMutex, ExInitializeRundownProtection, ExReleaseFastMutex,
    ExReleaseRundownProtection, ExWaitForRundownProtectionRelease,
};

use ntkrnl_sys::opaque_ptr;
// CMP_MAX_CALLBACKS — from ntoskrnl/include/internal/cm.h:72
const CMP_MAX_CALLBACKS: ULONG = 100;

// REGISTRY_CALLBACK — local struct from cmhook.c (not in any public header).
#[repr(C)]
pub struct _REGISTRY_CALLBACK {
    pub ListEntry: _LIST_ENTRY,
    pub RundownRef: _EX_RUNDOWN_REF,
    pub Function: PEX_CALLBACK_FUNCTION,
    pub Context: PVOID,
    pub Cookie: _LARGE_INTEGER,
    pub PendingDelete: BOOLEAN,
}
pub type REGISTRY_CALLBACK = _REGISTRY_CALLBACK;
pub type PREGISTRY_CALLBACK = *mut _REGISTRY_CALLBACK;

// Globals from cmhook.c (cmhook.c is excluded from build — owned here now).
#[no_mangle]
pub static mut CmpCallBackCount: ULONG = 0;
#[no_mangle]
pub static mut CmpCallBackVector: [_EX_CALLBACK; 100] = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CmiCallbackHead: _LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CmiCallbackLock: _FAST_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn CmpInitCallback() {
    let mut i: ULONG;

    CmpCallBackCount = 0;

    i = 0;
    while i < CMP_MAX_CALLBACKS {
        ExInitializeCallBack(&mut CmpCallBackVector[i as usize]);
        i += 1;
    }

    InitializeListHead(&mut CmiCallbackHead);
    ExInitializeFastMutex(&mut CmiCallbackLock);
}

// C source has no NTAPI — function is cdecl. Match with extern "C".
#[no_mangle]
pub unsafe extern "C" fn CmiCallRegisteredCallbacks(
    Argument1: REG_NOTIFY_CLASS,
    Argument2: PVOID,
) -> NTSTATUS {
    let Argument2 = opaque_ptr(Argument2);
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut CurrentCallback: PREGISTRY_CALLBACK;

    ExAcquireFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

    CurrentEntry = CmiCallbackHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(CmiCallbackHead) {
        CurrentCallback = (CurrentEntry as *mut u8).offset(
            -(core::mem::offset_of!(_REGISTRY_CALLBACK, ListEntry) as isize)
        ) as PREGISTRY_CALLBACK;

        if (*CurrentCallback).PendingDelete == 0 &&
            ExAcquireRundownProtection(core::ptr::addr_of_mut!((*CurrentCallback).RundownRef)) != 0
        {
            ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

            Status = ((*CurrentCallback).Function.unwrap())(
                (*CurrentCallback).Context,
                Argument1 as PVOID,
                Argument2,
            );

            ExAcquireFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

            ExReleaseRundownProtection(core::ptr::addr_of_mut!((*CurrentCallback).RundownRef));

            if !NT_SUCCESS(Status) {
                break;
            }
        }

        CurrentEntry = (*CurrentEntry).Flink;
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmRegisterCallback(
    Function: PEX_CALLBACK_FUNCTION,
    Context: PVOID,
    Cookie: PLARGE_INTEGER,
) -> NTSTATUS {
    let Context = opaque_ptr(Context);
    let Cookie = opaque_ptr(Cookie);
    let Callback: PREGISTRY_CALLBACK = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<REGISTRY_CALLBACK>() as SIZE_T,
        0x62634D43u32,
    ) as PREGISTRY_CALLBACK;
    
    if Callback == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ExInitializeRundownProtection(core::ptr::addr_of_mut!((*Callback).RundownRef));
    (*Callback).Function = Function;
    (*Callback).Context = Context;
    (*Callback).PendingDelete = 0u8;

    ExAcquireFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

    (*Callback).Cookie.QuadPart = Callback as ULONG_PTR as i64;
    InsertTailList(
        core::ptr::addr_of_mut!(CmiCallbackHead),
        core::ptr::addr_of_mut!((*Callback).ListEntry),
    );

    ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

    *Cookie = (*Callback).Cookie;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmUnRegisterCallback(Cookie: LARGE_INTEGER) -> NTSTATUS {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut CurrentCallback: PREGISTRY_CALLBACK;

    ExAcquireFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

    CurrentEntry = CmiCallbackHead.Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(CmiCallbackHead) {
        CurrentCallback = (CurrentEntry as *mut u8).offset(-(core::mem::offset_of!(_REGISTRY_CALLBACK, ListEntry) as isize)) as PREGISTRY_CALLBACK;
        
        if (*CurrentCallback).Cookie.QuadPart == Cookie.QuadPart {
            if (*CurrentCallback).PendingDelete == 0 {
                (*CurrentCallback).PendingDelete = 1u8;
                ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

                ExWaitForRundownProtectionRelease(core::ptr::addr_of_mut!((*CurrentCallback).RundownRef));

                ExAcquireFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));
                RemoveEntryList(core::ptr::addr_of_mut!((*CurrentCallback).ListEntry));
                ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

                ExFreePoolWithTag(CurrentCallback as PVOID, 0x62634D43u32);
                return STATUS_SUCCESS;
            } else {
                ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));
                return STATUS_UNSUCCESSFUL;
            }
        }
        
        CurrentEntry = (*CurrentEntry).Flink;
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!(CmiCallbackLock));

    return STATUS_UNSUCCESSFUL;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmhook.c
//  * PURPOSE:         Configuration Manager - Registry Notifications/Callbacks
//  * PROGRAMMERS:     Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* GLOBALS *******************************************************************/
//
// ULONG CmpCallBackCount = 0;
// EX_CALLBACK CmpCallBackVector[100];
//
// LIST_ENTRY CmiCallbackHead;
// FAST_MUTEX CmiCallbackLock;
//
// typedef struct _REGISTRY_CALLBACK
// {
//     LIST_ENTRY ListEntry;
//     EX_RUNDOWN_REF RundownRef;
//     PEX_CALLBACK_FUNCTION Function;
//     PVOID Context;
//     LARGE_INTEGER Cookie;
//     BOOLEAN PendingDelete;
// } REGISTRY_CALLBACK, *PREGISTRY_CALLBACK;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpInitCallback(VOID)
// {
//     ULONG i;
//     PAGED_CODE();
//
//     /* Reset counter */
//     CmpCallBackCount = 0;
//
//     /* Loop all the callbacks */
//     for (i = 0; i < CMP_MAX_CALLBACKS; i++)
//     {
//         /* Initialize this one */
//         ExInitializeCallBack(&CmpCallBackVector[i]);
//     }
//
//     /* ROS: Initialize old-style callbacks for now */
//     InitializeListHead(&CmiCallbackHead);
//     ExInitializeFastMutex(&CmiCallbackLock);
// }
//
// NTSTATUS
// CmiCallRegisteredCallbacks(IN REG_NOTIFY_CLASS Argument1,
//                            IN PVOID Argument2)
// {
//     PLIST_ENTRY CurrentEntry;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PREGISTRY_CALLBACK CurrentCallback;
//     PAGED_CODE();
//
//     ExAcquireFastMutex(&CmiCallbackLock);
//
//     for (CurrentEntry = CmiCallbackHead.Flink;
//          CurrentEntry != &CmiCallbackHead;
//          CurrentEntry = CurrentEntry->Flink)
//     {
//         CurrentCallback = CONTAINING_RECORD(CurrentEntry, REGISTRY_CALLBACK, ListEntry);
//         if (!CurrentCallback->PendingDelete &&
//             ExAcquireRundownProtection(&CurrentCallback->RundownRef))
//         {
//             /* don't hold locks during the callbacks! */
//             ExReleaseFastMutex(&CmiCallbackLock);
//
//             Status = CurrentCallback->Function(CurrentCallback->Context,
//                                          (PVOID)Argument1,
//                                          Argument2);
//
//             ExAcquireFastMutex(&CmiCallbackLock);
//
//             /* don't release the rundown protection before holding the callback lock
//             so the pointer to the next callback isn't cleared in case this callback
//             get's deleted */
//             ExReleaseRundownProtection(&CurrentCallback->RundownRef);
//             if(!NT_SUCCESS(Status))
//             {
//                 /* one callback returned failure, don't call any more callbacks */
//                 break;
//             }
//         }
//     }
//
//     ExReleaseFastMutex(&CmiCallbackLock);
//
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CmRegisterCallback(IN PEX_CALLBACK_FUNCTION Function,
//                    IN PVOID Context,
//                    IN OUT PLARGE_INTEGER Cookie)
// {
//     PREGISTRY_CALLBACK Callback;
//     PAGED_CODE();
//     ASSERT(Function && Cookie);
//
//     Callback = ExAllocatePoolWithTag(PagedPool,
//                                      sizeof(REGISTRY_CALLBACK),
//                                      'bcMC');
//     if (Callback == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize the callback */
//     ExInitializeRundownProtection(&Callback->RundownRef);
//     Callback->Function = Function;
//     Callback->Context = Context;
//     Callback->PendingDelete = FALSE;
//
//     /* add it to the callback list and receive a cookie for the callback */
//     ExAcquireFastMutex(&CmiCallbackLock);
//
//     /* FIXME - to receive a unique cookie we'll just return the pointer to the
//        callback object */
//     Callback->Cookie.QuadPart = (ULONG_PTR)Callback;
//     InsertTailList(&CmiCallbackHead, &Callback->ListEntry);
//
//     ExReleaseFastMutex(&CmiCallbackLock);
//
//     *Cookie = Callback->Cookie;
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// CmUnRegisterCallback(IN LARGE_INTEGER Cookie)
// {
//     PLIST_ENTRY CurrentEntry;
//     PREGISTRY_CALLBACK CurrentCallback;
//     PAGED_CODE();
//
//     ExAcquireFastMutex(&CmiCallbackLock);
//
//     for (CurrentEntry = CmiCallbackHead.Flink;
//          CurrentEntry != &CmiCallbackHead;
//          CurrentEntry = CurrentEntry->Flink)
//     {
//         CurrentCallback = CONTAINING_RECORD(CurrentEntry, REGISTRY_CALLBACK, ListEntry);
//         if (CurrentCallback->Cookie.QuadPart == Cookie.QuadPart)
//         {
//             if (!CurrentCallback->PendingDelete)
//             {
//                 /* found the callback, don't unlink it from the list yet so we don't screw
//                 the calling loop */
//                 CurrentCallback->PendingDelete = TRUE;
//                 ExReleaseFastMutex(&CmiCallbackLock);
//
//                 /* if the callback is currently executing, wait until it finished */
//                 ExWaitForRundownProtectionRelease(&CurrentCallback->RundownRef);
//
//                 /* time to unlink it. It's now safe because every attempt to acquire a
//                 runtime protection on this callback will fail */
//                 ExAcquireFastMutex(&CmiCallbackLock);
//                 RemoveEntryList(&CurrentCallback->ListEntry);
//                 ExReleaseFastMutex(&CmiCallbackLock);
//
//                 /* free the callback */
//                 ExFreePoolWithTag(CurrentCallback, 'bcMC');
//                 return STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* pending delete, pretend like it already is deleted */
//                 ExReleaseFastMutex(&CmiCallbackLock);
//                 return STATUS_UNSUCCESSFUL;
//             }
//         }
//     }
//
//     ExReleaseFastMutex(&CmiCallbackLock);
//
//     return STATUS_UNSUCCESSFUL;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use crate::prelude::*;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut NotificationListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut NotificationListLock: RTL_RESOURCE = RTL_RESOURCE {
    Lock: RTL_CRITICAL_SECTION {
        DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
            as *mut _RTL_CRITICAL_SECTION_DEBUG,
        LockCount: 0,
        RecursionCount: 0,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0,
    },
    SharedSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SharedWaiters: 0,
    ExclusiveSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    ExclusiveWaiters: 0,
    NumberActive: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    TimeoutBoost: 0,
    DebugInfo: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
};

#[repr(C)]
pub struct LSA_NOTIFICATION_ENTRY {
    pub Entry: LIST_ENTRY,
    pub InformationClass: POLICY_NOTIFICATION_INFORMATION_CLASS,
    pub ClientId: _CLIENT_ID,
    pub EventHandle: HANDLE,
    pub MappedEventHandle: HANDLE,
}
pub type PLSA_NOTIFICATION_ENTRY = *mut LSA_NOTIFICATION_ENTRY;
#[no_mangle]
pub unsafe extern "C" fn LsapInitNotificationList() {
    InitializeListHead(&raw mut NotificationListHead);
    RtlInitializeResource(&raw mut NotificationListLock);
}

unsafe fn LsapGetNotificationEntry(
    pRequestMsg: PLSA_API_MSG,
) -> PLSA_NOTIFICATION_ENTRY {
    let mut NotificationEntry: PLIST_ENTRY = NotificationListHead.Flink;
    while NotificationEntry != core::ptr::addr_of_mut!(NotificationListHead) {
        let CurrentNotification: PLSA_NOTIFICATION_ENTRY = (NotificationEntry as usize)
            .wrapping_sub(
                core::mem::offset_of!(LSA_NOTIFICATION_ENTRY, Entry),
            ) as PLSA_NOTIFICATION_ENTRY;

        if (*CurrentNotification).ClientId.UniqueProcess
            == (*pRequestMsg).h.__bindgen_anon_1.ClientId.UniqueProcess
            && (*CurrentNotification).ClientId.UniqueThread
                == (*pRequestMsg).h.__bindgen_anon_1.ClientId.UniqueThread
            && (*CurrentNotification).InformationClass as u32
                == (*pRequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .PolicyChangeNotify
                    .__bindgen_anon_1
                    .Request
                    .InformationClass as u32
            && (*CurrentNotification).EventHandle
                == (*pRequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .PolicyChangeNotify
                    .__bindgen_anon_1
                    .Request
                    .NotificationEventHandle
        {
            return CurrentNotification;
        }
        NotificationEntry = (*NotificationEntry).Flink;
    }
    core::ptr::null_mut::<LSA_NOTIFICATION_ENTRY>()
}

unsafe fn LsapAddNotification(pRequestMsg: PLSA_API_MSG) -> NTSTATUS {
    let pEntry: PLSA_NOTIFICATION_ENTRY;
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_TRACE)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/notify.c\0"
                .as_ptr() as *const i8,
            b"LsapAddNotification\0".as_ptr() as *const i8,
            70i32,
            b"LsapAddNotification(%p)\n\0".as_ptr() as *const i8,
            pRequestMsg,
        );
    }

    pEntry = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        core::mem::size_of::<LSA_NOTIFICATION_ENTRY>() as SIZE_T,
    ) as PLSA_NOTIFICATION_ENTRY;
    if pEntry.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        (*pEntry).InformationClass = ((*pRequestMsg)
            .__bindgen_anon_1
            .__bindgen_anon_1
            .__bindgen_anon_1
            .PolicyChangeNotify
            .__bindgen_anon_1
            .Request)
            .InformationClass;
        (*pEntry).EventHandle = ((*pRequestMsg)
            .__bindgen_anon_1
            .__bindgen_anon_1
            .__bindgen_anon_1
            .PolicyChangeNotify
            .__bindgen_anon_1
            .Request)
            .NotificationEventHandle;
        (*pEntry).ClientId = core::mem::transmute((*pRequestMsg).h.__bindgen_anon_1.ClientId);

        Status = NtOpenProcess(
            core::ptr::addr_of_mut!(hProcess),
            PROCESS_DUP_HANDLE as ACCESS_MASK,
            core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
            (*pEntry).ClientId.UniqueProcess as PCLIENT_ID,
        );
        if Status < 0 {
            if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR)) != 0 {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/notify.c\0"
                        .as_ptr() as *const i8,
                    b"LsapAddNotification\0".as_ptr() as *const i8,
                    94i32,
                    b"NtOpenProcess() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
        } else {
            Status = NtDuplicateObject(
                hProcess,
                (*pEntry).EventHandle,
                (-1isize) as HANDLE,
                core::ptr::addr_of_mut!((*pEntry).MappedEventHandle),
                0 as ACCESS_MASK,
                0 as ULONG,
                DUPLICATE_SAME_ACCESS as ULONG,
            );
            if Status < 0 {
                if ((*__wine_dbch___default).flags as i32 & (1 << __WINE_DBCL_ERR)) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_ERR,
                        __wine_dbch___default,
                        b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/notify.c\0"
                            .as_ptr() as *const i8,
                        b"LsapAddNotification\0".as_ptr() as *const i8,
                        108i32,
                        b"NtDuplicateObject() failed (Status 0x%08lx)\n\0".as_ptr()
                            as *const i8,
                        Status,
                    );
                }
            } else {
                InsertHeadList(
                    core::ptr::addr_of_mut!(NotificationListHead),
                    core::ptr::addr_of_mut!((*pEntry).Entry),
                );
            }
        }
    }

    if !hProcess.is_null() {
        NtClose(hProcess);
    }
    if Status < 0 {
        if !pEntry.is_null() {
            RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, pEntry as PVOID);
        }
    }

    Status
}

unsafe fn LsapRemoveNotification(pRequestMsg: PLSA_API_MSG) -> NTSTATUS {
    if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE as i32) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/notify.c\0"
                .as_ptr() as *const i8,
            b"LsapRemoveNotification\0".as_ptr() as *const i8,
            137i32,
            b"LsapRemoveNotification(%p)\n\0".as_ptr() as *const i8,
            pRequestMsg,
        );
    }
    let pEntry = LsapGetNotificationEntry(pRequestMsg);
    if pEntry.is_null() {
        return STATUS_INVALID_HANDLE;
    }
    RemoveEntryList(&raw mut (*pEntry).Entry);
    NtClose((*pEntry).MappedEventHandle);
    RtlFreeHeap(RtlGetProcessHeap(), 0u32, pEntry as PVOID);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapRegisterNotification(pRequestMsg: PLSA_API_MSG) -> NTSTATUS {
    let Status: NTSTATUS;
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(NotificationListLock), 1u8);
    if (*pRequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.PolicyChangeNotify.__bindgen_anon_1.Request.Register != 0 {
        Status = LsapAddNotification(pRequestMsg);
    } else {
        Status = LsapRemoveNotification(pRequestMsg);
    }
    RtlReleaseResource(core::ptr::addr_of_mut!(NotificationListLock));
    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapNotifyPolicyChange(
    InformationClass: POLICY_NOTIFICATION_INFORMATION_CLASS,
) {
    let mut NotificationEntry: PLIST_ENTRY;
    let mut CurrentNotification: PLSA_NOTIFICATION_ENTRY;

    TRACE!(
        b"LsapNotifyPolicyChange(%lu)\n\0".as_ptr() as *const i8,
        InformationClass,
    );

    RtlAcquireResourceShared(
        core::ptr::addr_of!(NotificationListLock) as *mut RTL_RESOURCE,
        1u8,
    );

    NotificationEntry = NotificationListHead.Flink as PLIST_ENTRY;
    while NotificationEntry != core::ptr::addr_of_mut!(NotificationListHead) as PLIST_ENTRY {
        CurrentNotification = (NotificationEntry as *mut u8)
            .byte_offset(-(core::mem::offset_of!(LSA_NOTIFICATION_ENTRY, Entry) as isize))
            as PLSA_NOTIFICATION_ENTRY;

        if (*CurrentNotification).InformationClass as u32 == InformationClass as u32 {
            TRACE!(
                b"Notify event %p\n\0".as_ptr() as *const i8,
                (*CurrentNotification).MappedEventHandle,
            );
            NtSetEvent(
                (*CurrentNotification).MappedEventHandle,
                core::ptr::null_mut(),
            );
        }

        NotificationEntry = (*NotificationEntry).Flink as PLIST_ENTRY;
    }

    RtlReleaseResource(core::ptr::addr_of!(NotificationListLock) as *mut RTL_RESOURCE);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Local Security Authority Server DLL
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     LSA policy change notifications
//  * COPYRIGHT:   Eric Kohl 2018
//  */
//
// #include "lsasrv.h"
//
// typedef struct _LSA_NOTIFICATION_ENTRY
// {
//     LIST_ENTRY Entry;
//     POLICY_NOTIFICATION_INFORMATION_CLASS InformationClass;
//     CLIENT_ID ClientId;
//     HANDLE EventHandle;
//     HANDLE MappedEventHandle;
// } LSA_NOTIFICATION_ENTRY, *PLSA_NOTIFICATION_ENTRY;
//
// /* GLOBALS *****************************************************************/
//
// static LIST_ENTRY NotificationListHead;
// static RTL_RESOURCE NotificationListLock;
//
//
// /* FUNCTIONS ***************************************************************/
//
// VOID
// LsapInitNotificationList(VOID)
// {
//     InitializeListHead(&NotificationListHead);
//     RtlInitializeResource(&NotificationListLock);
// }
//
//
// static
// PLSA_NOTIFICATION_ENTRY
// LsapGetNotificationEntry(
//     PLSA_API_MSG pRequestMsg)
// {
//     PLIST_ENTRY NotificationEntry;
//     PLSA_NOTIFICATION_ENTRY CurrentNotification;
//
//     NotificationEntry = NotificationListHead.Flink;
//     while (NotificationEntry != &NotificationListHead)
//     {
//         CurrentNotification = CONTAINING_RECORD(NotificationEntry, LSA_NOTIFICATION_ENTRY, Entry);
//
//         if ((CurrentNotification->ClientId.UniqueProcess == pRequestMsg->h.ClientId.UniqueProcess) &&
//             (CurrentNotification->ClientId.UniqueThread == pRequestMsg->h.ClientId.UniqueThread) &&
//             (CurrentNotification->InformationClass == pRequestMsg->PolicyChangeNotify.Request.InformationClass) &&
//             (CurrentNotification->EventHandle == pRequestMsg->PolicyChangeNotify.Request.NotificationEventHandle))
//             return CurrentNotification;
//
//         NotificationEntry = NotificationEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// static
// NTSTATUS
// LsapAddNotification(
//     PLSA_API_MSG pRequestMsg)
// {
//     PLSA_NOTIFICATION_ENTRY pEntry;
//     HANDLE hProcess = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("LsapAddNotification(%p)\n", pRequestMsg);
//
//     /* Allocate a new notification list entry */
//     pEntry = RtlAllocateHeap(RtlGetProcessHeap(),
//                              HEAP_ZERO_MEMORY,
//                              sizeof(LSA_NOTIFICATION_ENTRY));
//     if (pEntry == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* Copy the notification data */
//     pEntry->InformationClass = pRequestMsg->PolicyChangeNotify.Request.InformationClass;
//     pEntry->EventHandle = pRequestMsg->PolicyChangeNotify.Request.NotificationEventHandle;
//     pEntry->ClientId = pRequestMsg->h.ClientId;
//
//     /* Open the client process */
//     Status = NtOpenProcess(&hProcess,
//                            PROCESS_DUP_HANDLE,
//                            NULL,
//                            pEntry->ClientId.UniqueProcess);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtOpenProcess() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Duplicate the event handle into the current process */
//     Status = NtDuplicateObject(hProcess,
//                                pEntry->EventHandle,
//                                NtCurrentProcess(),
//                                &pEntry->MappedEventHandle,
//                                0,
//                                0,
//                                DUPLICATE_SAME_ACCESS);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtDuplicateObject() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     /* Insert the new entry into the notification list */
//     InsertHeadList(&NotificationListHead,
//                    &pEntry->Entry);
//
// done:
//     if (hProcess != NULL)
//         NtClose(hProcess);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (pEntry != NULL)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, pEntry);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// LsapRemoveNotification(
//     PLSA_API_MSG pRequestMsg)
// {
//     PLSA_NOTIFICATION_ENTRY pEntry;
//
//     TRACE("LsapRemoveNotification(%p)\n", pRequestMsg);
//
//     pEntry = LsapGetNotificationEntry(pRequestMsg);
//     if (pEntry == NULL)
//     {
//         return STATUS_INVALID_HANDLE;
//     }
//
//     /* Remove the  notification entry from the notification list */
//     RemoveEntryList(&pEntry->Entry);
//
//     /* Close the mapped event handle */
//     NtClose(pEntry->MappedEventHandle);
//
//     /* Release the notification entry */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, pEntry);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// LsapRegisterNotification(
//     PLSA_API_MSG pRequestMsg)
// {
//     NTSTATUS Status;
//
//     TRACE("LsapRegisterNotification(%p)\n", pRequestMsg);
//
//     /* Acquire the notification list lock exclusively */
//     RtlAcquireResourceExclusive(&NotificationListLock, TRUE);
//
//     if (pRequestMsg->PolicyChangeNotify.Request.Register)
//     {
//         /* Register the notification event */
//         Status = LsapAddNotification(pRequestMsg);
//     }
//     else
//     {
//         /* Unregister the notification event */
//         Status = LsapRemoveNotification(pRequestMsg);
//     }
//
//     /* Release the notification list lock */
//     RtlReleaseResource(&NotificationListLock);
//
//     return Status;
// }
//
//
// VOID
// LsapNotifyPolicyChange(
//     POLICY_NOTIFICATION_INFORMATION_CLASS InformationClass)
// {
//     PLIST_ENTRY NotificationEntry;
//     PLSA_NOTIFICATION_ENTRY CurrentNotification;
//
//     TRACE("LsapNotifyPolicyChange(%lu)\n", InformationClass);
//
//     /* Acquire the notification list lock shared */
//     RtlAcquireResourceShared(&NotificationListLock, TRUE);
//
//     NotificationEntry = NotificationListHead.Flink;
//     while (NotificationEntry != &NotificationListHead)
//     {
//         CurrentNotification = CONTAINING_RECORD(NotificationEntry, LSA_NOTIFICATION_ENTRY, Entry);
//
//         if (CurrentNotification->InformationClass == InformationClass)
//         {
//             TRACE("Notify event %p\n", CurrentNotification->MappedEventHandle);
//             NtSetEvent(CurrentNotification->MappedEventHandle, NULL);
//         }
//
//         NotificationEntry = NotificationEntry->Flink;
//     }
//
//     /* Release the notification list lock */
//     RtlReleaseResource(&NotificationListLock);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

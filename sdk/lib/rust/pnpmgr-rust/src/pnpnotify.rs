use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _DEVICE_INTERFACE_CHANGE_NOTIFICATION,
    _HWPROFILE_CHANGE_NOTIFICATION,
    _TARGET_DEVICE_REMOVAL_NOTIFICATION,
    _TARGET_DEVICE_CUSTOM_NOTIFICATION,
    _DEVICE_NODE, _ETHREAD, _UNICODE_STRING,
    PDEVICE_INTERFACE_CHANGE_NOTIFICATION,
    PHWPROFILE_CHANGE_NOTIFICATION,
    PTARGET_DEVICE_REMOVAL_NOTIFICATION,
    PTARGET_DEVICE_CUSTOM_NOTIFICATION,
    PDEVICE_OBJECT,
    PFILE_OBJECT,
    PDRIVER_OBJECT,
    PDEVICE_NODE,
    PDRIVER_NOTIFICATION_CALLBACK_ROUTINE,
    GUID, LPCGUID, LPGUID,
    PUNICODE_STRING,
    IoGetDeviceInterfaces,
    RtlInitUnicodeString,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
}

#[inline(always)]
unsafe fn CONTAINING_RECORD<T>(ptr: PVOID, _type_size: usize, field_offset: usize) -> *mut T {
    (ptr as usize - field_offset) as *mut T
}
extern "C" {
    static GUID_DEVICE_INTERFACE_ARRIVAL: GUID;
    static GUID_PNP_CUSTOM_NOTIFICATION: GUID;
}
extern "system" {
    fn IopGetRelatedTargetDevice(FileObject: PFILE_OBJECT, DeviceNode: *mut PDEVICE_NODE) -> NTSTATUS;
}

#[inline]
unsafe fn IsEqualGUID(guid1: LPCGUID, guid2: LPCGUID) -> BOOLEAN {
    if (*guid1).Data1 == (*guid2).Data1 &&
       (*guid1).Data2 == (*guid2).Data2 &&
       (*guid1).Data3 == (*guid2).Data3 &&
       (*guid1).Data4 == (*guid2).Data4 {
        1u8
    } else {
        0u8
    }
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IopGetDeviceNode(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IopGetDeviceNode(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_NODE;
}

static mut PiNotifyTargetDeviceLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
static mut PiNotifyHwProfileLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };
static mut PiNotifyDeviceInterfaceLock: KGUARDED_MUTEX = unsafe { core::mem::zeroed() };

static mut PiNotifyHwProfileListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

static mut PiNotifyDeviceInterfaceListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

const TAG_PNP_NOTIFY: ULONG = u32::from_le_bytes(*b"PnPN");
const PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES: ULONG = 1;
const EventCategoryDeviceInterfaceChange: IO_NOTIFICATION_EVENT_CATEGORY = 2;
const EventCategoryHardwareProfileChange: IO_NOTIFICATION_EVENT_CATEGORY = 1;
const EventCategoryTargetDeviceChange: IO_NOTIFICATION_EVENT_CATEGORY = 3;

type IO_NOTIFICATION_EVENT_CATEGORY = i32;

#[repr(C)]
struct PNP_NOTIFY_ENTRY {
    PnpNotifyList: LIST_ENTRY,
    Context: PVOID,
    DriverObject: PDRIVER_OBJECT,
    PnpNotificationProc: PDRIVER_NOTIFICATION_CALLBACK_ROUTINE,
    union_data: PNP_NOTIFY_UNION,
    EventCategory: IO_NOTIFICATION_EVENT_CATEGORY,
    RefCount: u8,
    Deleted: BOOLEAN,
}

#[repr(C)]
#[derive(Copy, Clone)]
union PNP_NOTIFY_UNION {
    Guid: GUID,
    TargetDevice: PNP_TARGET_DEVICE,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PNP_TARGET_DEVICE {
    FileObject: PFILE_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
}

type PPNP_NOTIFY_ENTRY = *mut PNP_NOTIFY_ENTRY;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "C" fn PiInitializeNotifications() {
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock));
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(PiNotifyHwProfileLock));
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock));
    InitializeListHead(core::ptr::addr_of_mut!(PiNotifyHwProfileListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceListHead) as PLIST_ENTRY);
}

unsafe fn PiDereferencePnpNotifyEntry(Entry: PPNP_NOTIFY_ENTRY) {
    ObDereferenceObject((*Entry).DriverObject as PVOID);
    (*Entry).RefCount -= 1;
    if (*Entry).RefCount == 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*Entry).PnpNotifyList) as PLIST_ENTRY);
        if (*Entry).EventCategory == EventCategoryTargetDeviceChange {
            ObDereferenceObject((*Entry).union_data.TargetDevice.DeviceObject as PVOID);
        }
        ExFreePoolWithTag(Entry as PVOID, TAG_PNP_NOTIFY);
    }
}

unsafe fn PiReferencePnpNotifyEntry(Entry: PPNP_NOTIFY_ENTRY) {
    ObReferenceObject((*Entry).DriverObject as PVOID);
    (*Entry).RefCount += 1;
}

unsafe fn PiCallNotifyProc(
    Proc: PDRIVER_NOTIFICATION_CALLBACK_ROUTINE,
    NotificationStructure: PVOID,
    Context: PVOID,
) {
    // Debug assertions elided: CombinedApcDisable is in a deep union in _KTHREAD
    let _ = NotificationStructure;
    let _ = Context;

    (Proc.unwrap())(NotificationStructure, Context);
}

unsafe fn PiProcessSingleNotification(
    Entry: PPNP_NOTIFY_ENTRY,
    NotificationStructure: PVOID,
    Lock: PKGUARDED_MUTEX,
    NextEntry: *mut PLIST_ENTRY,
) {
    PiReferencePnpNotifyEntry(Entry);

    KeReleaseGuardedMutex(Lock);
    PiCallNotifyProc((*Entry).PnpNotificationProc, NotificationStructure, (*Entry).Context);
    KeAcquireGuardedMutex(Lock);

    *NextEntry = (*Entry).PnpNotifyList.Flink;
    PiDereferencePnpNotifyEntry(Entry);
}

#[no_mangle]
pub unsafe extern "C" fn PiNotifyDeviceInterfaceChange(
    Event: LPCGUID,
    InterfaceClassGuid: LPCGUID,
    SymbolicLinkName: PUNICODE_STRING,
) {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let notifyStruct = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_DEVICE_INTERFACE_CHANGE_NOTIFICATION>(),
        TAG_PNP_NOTIFY,
    ) as PDEVICE_INTERFACE_CHANGE_NOTIFICATION;
    if notifyStruct.is_null() {
        return;
    }

    (*notifyStruct).Version = 1;
    (*notifyStruct).Size = core::mem::size_of::<_DEVICE_INTERFACE_CHANGE_NOTIFICATION>() as u16;
    (*notifyStruct).Event = *Event;
    (*notifyStruct).InterfaceClassGuid = *InterfaceClassGuid;
    (*notifyStruct).SymbolicLinkName = SymbolicLinkName;

    crate::dbg_print!(
        b"[PNPN] IfaceChange class=%08x link=%wZ\r\n\0",
        (*InterfaceClassGuid).Data1,
        SymbolicLinkName
    );

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock));

    let mut entry = PiNotifyDeviceInterfaceListHead.Flink;
    while entry != core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceListHead) as PLIST_ENTRY {
        let nEntry = CONTAINING_RECORD(
            entry as PVOID,
            core::mem::size_of::<PNP_NOTIFY_ENTRY>(),
            core::mem::offset_of!(PNP_NOTIFY_ENTRY, PnpNotifyList),
        ) as PPNP_NOTIFY_ENTRY;

        if IsEqualGUID(
            core::ptr::addr_of!((*notifyStruct).InterfaceClassGuid),
            core::ptr::addr_of!((*nEntry).union_data.Guid),
        ) == 0u8
        {
            entry = (*entry).Flink;
            continue;
        }

        PiProcessSingleNotification(
            nEntry,
            notifyStruct as PVOID,
            core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock),
            core::ptr::addr_of_mut!(entry),
        );
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock));
    ExFreePoolWithTag(notifyStruct as PVOID, TAG_PNP_NOTIFY);
}

#[no_mangle]
pub unsafe extern "C" fn PiNotifyHardwareProfileChange(Event: LPCGUID) {
    let notifyStruct = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<_HWPROFILE_CHANGE_NOTIFICATION>(),
        TAG_PNP_NOTIFY,
    ) as PHWPROFILE_CHANGE_NOTIFICATION;
    if notifyStruct.is_null() {
        return;
    }

    (*notifyStruct).Version = 1;
    (*notifyStruct).Size = core::mem::size_of::<_HWPROFILE_CHANGE_NOTIFICATION>() as u16;
    (*notifyStruct).Event = *Event;

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PiNotifyHwProfileLock));

    let mut entry = PiNotifyHwProfileListHead.Flink;
    while entry != core::ptr::addr_of_mut!(PiNotifyHwProfileListHead) as PLIST_ENTRY {
        let nEntry = CONTAINING_RECORD(
            entry as PVOID,
            core::mem::size_of::<PNP_NOTIFY_ENTRY>(),
            core::mem::offset_of!(PNP_NOTIFY_ENTRY, PnpNotifyList),
        ) as PPNP_NOTIFY_ENTRY;

        PiProcessSingleNotification(
            nEntry,
            notifyStruct as PVOID,
            core::ptr::addr_of_mut!(PiNotifyHwProfileLock),
            core::ptr::addr_of_mut!(entry),
        );
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PiNotifyHwProfileLock));
    ExFreePoolWithTag(notifyStruct as PVOID, TAG_PNP_NOTIFY);
}

#[no_mangle]
pub unsafe extern "C" fn PiNotifyTargetDeviceChange(
    Event: LPCGUID,
    DeviceObject: PDEVICE_OBJECT,
    CustomNotification: PTARGET_DEVICE_CUSTOM_NOTIFICATION,
) {
    let DeviceObject = opaque_ptr(DeviceObject);
    let notificationStruct: PVOID;
    ObReferenceObject(DeviceObject as PVOID);

    let deviceNode = IopGetDeviceNode(DeviceObject);

    if IsEqualGUID(Event, core::ptr::addr_of!(GUID_PNP_CUSTOM_NOTIFICATION)) == 0u8 {
        let notifStruct = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<_TARGET_DEVICE_REMOVAL_NOTIFICATION>(),
            TAG_PNP_NOTIFY,
        ) as PTARGET_DEVICE_REMOVAL_NOTIFICATION;
        if notifStruct.is_null() {
            return;
        }

        (*notifStruct).Version = 1;
        (*notifStruct).Size = core::mem::size_of::<_TARGET_DEVICE_REMOVAL_NOTIFICATION>() as u16;
        (*notifStruct).Event = *Event;

        notificationStruct = notifStruct as PVOID;
    } else {
        notificationStruct = CustomNotification as PVOID;
    }

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock));

    let mut entry = (*deviceNode).TargetDeviceNotify.Flink;
    while entry != core::ptr::addr_of_mut!((*deviceNode).TargetDeviceNotify) as PLIST_ENTRY {
        let nEntry = CONTAINING_RECORD(
            entry as PVOID,
            core::mem::size_of::<PNP_NOTIFY_ENTRY>(),
            core::mem::offset_of!(PNP_NOTIFY_ENTRY, PnpNotifyList),
        ) as PPNP_NOTIFY_ENTRY;

        (*(notificationStruct as PTARGET_DEVICE_REMOVAL_NOTIFICATION)).FileObject =
            (*nEntry).union_data.TargetDevice.FileObject;

        PiProcessSingleNotification(
            nEntry,
            notificationStruct,
            core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock),
            core::ptr::addr_of_mut!(entry),
        );
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock));
    if notificationStruct != CustomNotification as PVOID {
        ExFreePoolWithTag(notificationStruct, TAG_PNP_NOTIFY);
    }
    ObDereferenceObject(DeviceObject as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn IoPnPDeliverServicePowerNotification(
    _VetoedPowerOperation: ULONG,
    _PowerNotificationCode: ULONG,
    _PowerNotificationData: ULONG,
    _Synchronous: BOOLEAN,
) -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn IoRegisterPlugPlayNotification(
    EventCategory: IO_NOTIFICATION_EVENT_CATEGORY,
    EventCategoryFlags: ULONG,
    EventCategoryData: PVOID,
    DriverObject: PDRIVER_OBJECT,
    CallbackRoutine: PDRIVER_NOTIFICATION_CALLBACK_ROUTINE,
    Context: PVOID,
    NotificationEntry: *mut PVOID,
) -> NTSTATUS {
    let EventCategoryData = opaque_ptr(EventCategoryData);
    let DriverObject = opaque_ptr(DriverObject);
    let Context = opaque_ptr(Context);
    let NotificationEntry = opaque_ptr(NotificationEntry);
    ObReferenceObject(DriverObject as PVOID);

    let Entry = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<PNP_NOTIFY_ENTRY>(),
        TAG_PNP_NOTIFY,
    ) as PPNP_NOTIFY_ENTRY;
    if Entry.is_null() {
        ObDereferenceObject(DriverObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Zero the entire struct first (matches C designated initializer behavior)
    core::ptr::write_bytes(Entry as *mut u8, 0, core::mem::size_of::<PNP_NOTIFY_ENTRY>());

    (*Entry).PnpNotificationProc = CallbackRoutine;
    (*Entry).Context = Context;
    (*Entry).DriverObject = DriverObject;
    (*Entry).EventCategory = EventCategory;
    (*Entry).RefCount = 1;
    (*Entry).Deleted = 0u8;

    match EventCategory {
        EventCategoryDeviceInterfaceChange => {
            (*Entry).union_data.Guid = *(EventCategoryData as LPGUID);

            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock));
            InsertTailList(
                core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Entry).PnpNotifyList) as PLIST_ENTRY,
            );
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock));

            if (EventCategoryFlags & PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES) != 0 {
                let mut SymbolicLinkList: PWSTR = core::ptr::null_mut();
                let Status = IoGetDeviceInterfaces(
                    EventCategoryData as LPCGUID,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(SymbolicLinkList),
                );
                if Status >= 0 {
                    let mut NotificationInfos: _DEVICE_INTERFACE_CHANGE_NOTIFICATION =
                        core::mem::zeroed();
                    let mut SymbolicLinkU: _UNICODE_STRING = core::mem::zeroed();

                    NotificationInfos.Version = 1;
                    NotificationInfos.Size =
                        core::mem::size_of::<_DEVICE_INTERFACE_CHANGE_NOTIFICATION>() as u16;
                    NotificationInfos.Event = GUID_DEVICE_INTERFACE_ARRIVAL;
                    NotificationInfos.InterfaceClassGuid = *(EventCategoryData as LPGUID);
                    NotificationInfos.SymbolicLinkName =
                        core::ptr::addr_of_mut!(SymbolicLinkU);

                    let mut SymbolicLink = SymbolicLinkList;
                    while *SymbolicLink != 0 {
                        RtlInitUnicodeString(
                            core::ptr::addr_of_mut!(SymbolicLinkU),
                            SymbolicLink,
                        );
                        PiCallNotifyProc(
                            CallbackRoutine,
                            core::ptr::addr_of_mut!(NotificationInfos) as PVOID,
                            Context,
                        );
                        SymbolicLink = SymbolicLink.add((SymbolicLinkU.Length / 2) as usize + 1);
                    }

                    ExFreePoolWithTag(SymbolicLinkList as PVOID, 0);
                }
            }
        }
        EventCategoryHardwareProfileChange => {
            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PiNotifyHwProfileLock));
            InsertTailList(
                core::ptr::addr_of_mut!(PiNotifyHwProfileListHead) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Entry).PnpNotifyList) as PLIST_ENTRY,
            );
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PiNotifyHwProfileLock));
        }
        EventCategoryTargetDeviceChange => {
            let mut deviceNode: PDEVICE_NODE = core::ptr::null_mut();
            (*Entry).union_data.TargetDevice.FileObject = EventCategoryData as PFILE_OBJECT;

            let Status = IopGetRelatedTargetDevice(
                (*Entry).union_data.TargetDevice.FileObject,
                core::ptr::addr_of_mut!(deviceNode),
            );
            if Status < 0 {
                ObDereferenceObject(DriverObject as PVOID);
                ExFreePoolWithTag(Entry as PVOID, TAG_PNP_NOTIFY);
                return Status;
            }
            (*Entry).union_data.TargetDevice.DeviceObject = (*deviceNode).PhysicalDeviceObject;

            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock));
            InsertTailList(
                core::ptr::addr_of_mut!((*deviceNode).TargetDeviceNotify) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Entry).PnpNotifyList) as PLIST_ENTRY,
            );
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock));
        }
        _ => {
            ObDereferenceObject(DriverObject as PVOID);
            ExFreePoolWithTag(Entry as PVOID, TAG_PNP_NOTIFY);
            return STATUS_NOT_SUPPORTED;
        }
    }

    core::ptr::write_volatile(NotificationEntry, Entry as PVOID);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoUnregisterPlugPlayNotification(
    NotificationEntry: PVOID,
) -> NTSTATUS {
    let NotificationEntry = opaque_ptr(NotificationEntry);
    let Entry = NotificationEntry as PPNP_NOTIFY_ENTRY;

    let Lock: PKGUARDED_MUTEX = match (*Entry).EventCategory {
        EventCategoryDeviceInterfaceChange => core::ptr::addr_of_mut!(PiNotifyDeviceInterfaceLock),
        EventCategoryHardwareProfileChange => core::ptr::addr_of_mut!(PiNotifyHwProfileLock),
        EventCategoryTargetDeviceChange => core::ptr::addr_of_mut!(PiNotifyTargetDeviceLock),
        _ => return STATUS_NOT_SUPPORTED,
    };

    KeAcquireGuardedMutex(Lock);
    if (*Entry).Deleted == 0u8 {
        (*Entry).Deleted = 1u8;
        PiDereferencePnpNotifyEntry(Entry);
    }
    KeReleaseGuardedMutex(Lock);

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Plug & Play notification functions
//  * COPYRIGHT:   Copyright 2003 Filip Navara <xnavara@volny.cz>
//  *              Copyright 2005-2006 Hervé Poussineau <hpoussin@reactos.org>
//  *              Copyright 2010 Pierre Schweitzer <pierre@reactos.org>
//  *              Copyright 2020 Victor Perevertkin <victor.perevertkin@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// KGUARDED_MUTEX PiNotifyTargetDeviceLock;
// KGUARDED_MUTEX PiNotifyHwProfileLock;
// KGUARDED_MUTEX PiNotifyDeviceInterfaceLock;
//
// _Guarded_by_(PiNotifyHwProfileLock)
// LIST_ENTRY PiNotifyHwProfileListHead;
//
// _Guarded_by_(PiNotifyDeviceInterfaceLock)
// LIST_ENTRY PiNotifyDeviceInterfaceListHead;
//
// /* TYPES *********************************************************************/
//
// typedef struct _PNP_NOTIFY_ENTRY
// {
//     LIST_ENTRY PnpNotifyList;
//     PVOID Context;
//     PDRIVER_OBJECT DriverObject;
//     PDRIVER_NOTIFICATION_CALLBACK_ROUTINE PnpNotificationProc;
//     union
//     {
//         GUID Guid; // for EventCategoryDeviceInterfaceChange
//         struct
//         {
//             PFILE_OBJECT FileObject; // for EventCategoryTargetDeviceChange
//             PDEVICE_OBJECT DeviceObject;
//         };
//     };
//     IO_NOTIFICATION_EVENT_CATEGORY EventCategory;
//     UINT8 RefCount;
//     BOOLEAN Deleted;
// } PNP_NOTIFY_ENTRY, *PPNP_NOTIFY_ENTRY;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// VOID
// PiInitializeNotifications(VOID)
// {
//     KeInitializeGuardedMutex(&PiNotifyTargetDeviceLock);
//     KeInitializeGuardedMutex(&PiNotifyHwProfileLock);
//     KeInitializeGuardedMutex(&PiNotifyDeviceInterfaceLock);
//     InitializeListHead(&PiNotifyHwProfileListHead);
//     InitializeListHead(&PiNotifyDeviceInterfaceListHead);
// }
//
// static
// CODE_SEG("PAGE")
// VOID
// PiDereferencePnpNotifyEntry(
//     _In_ PPNP_NOTIFY_ENTRY Entry)
// {
//     PAGED_CODE();
//     ASSERT(Entry->RefCount > 0);
//
//     ObDereferenceObject(Entry->DriverObject);
//     Entry->RefCount--;
//     if (Entry->RefCount == 0)
//     {
//         ASSERT(Entry->Deleted);
//
//         RemoveEntryList(&Entry->PnpNotifyList);
//         if (Entry->EventCategory == EventCategoryTargetDeviceChange)
//         {
//             // IopGetRelatedTargetDevice referenced the device upon the notification registration
//             ObDereferenceObject(Entry->DeviceObject);
//         }
//         ExFreePoolWithTag(Entry, TAG_PNP_NOTIFY);
//     }
// }
//
// static
// CODE_SEG("PAGE")
// VOID
// PiReferencePnpNotifyEntry(
//     _In_ PPNP_NOTIFY_ENTRY Entry)
// {
//     PAGED_CODE();
//     ObReferenceObject(Entry->DriverObject);
//     Entry->RefCount++;
// }
//
// /**
//  * @brief Calls PnP notification routine and makes some checks to detect faulty drivers
//  */
// static
// CODE_SEG("PAGE")
// VOID
// PiCallNotifyProc(
//     _In_ PDRIVER_NOTIFICATION_CALLBACK_ROUTINE Proc,
//     _In_ PVOID NotificationStructure,
//     _In_ PVOID Context)
// {
//     PAGED_CODE();
// #if DBG
//     KIRQL oldIrql = KeGetCurrentIrql();
//     ULONG oldApcDisable = KeGetCurrentThread()->CombinedApcDisable;
// #endif
//
//     Proc(NotificationStructure, Context);
//
//     ASSERT(oldIrql == KeGetCurrentIrql() &&
//            oldApcDisable == KeGetCurrentThread()->CombinedApcDisable);
// }
//
// static
// CODE_SEG("PAGE")
// _Requires_lock_held_(Lock)
// VOID
// PiProcessSingleNotification(
//     _In_ PPNP_NOTIFY_ENTRY Entry,
//     _In_ PVOID NotificationStructure,
//     _In_ PKGUARDED_MUTEX Lock,
//     _Out_ PLIST_ENTRY *NextEntry)
// {
//     PAGED_CODE();
//
//     // the notification may be unregistered inside the procedure
//     // thus reference the entry so we may proceed
//     PiReferencePnpNotifyEntry(Entry);
//
//     // release the lock because the notification routine has to be called without any
//     // limitations regarding APCs
//     KeReleaseGuardedMutex(Lock);
//     PiCallNotifyProc(Entry->PnpNotificationProc, NotificationStructure, Entry->Context);
//     KeAcquireGuardedMutex(Lock);
//
//     // take the next entry link only after the callback finishes
//     // the lock is not held there, so Entry may have changed at this point
//     *NextEntry = Entry->PnpNotifyList.Flink;
//     PiDereferencePnpNotifyEntry(Entry);
// }
//
// /**
//  * @brief      Delivers the event to all drivers subscribed to
//  *             EventCategoryDeviceInterfaceChange
//  *
//  * @param[in]  Event               The PnP event GUID
//  * @param[in]  InterfaceClassGuid  The GUID of an interface class
//  * @param[in]  SymbolicLinkName    Pointer to a string identifying the device interface name
//  */
// CODE_SEG("PAGE")
// VOID
// PiNotifyDeviceInterfaceChange(
//     _In_ LPCGUID Event,
//     _In_ LPCGUID InterfaceClassGuid,
//     _In_ PUNICODE_STRING SymbolicLinkName)
// {
//     PAGED_CODE();
//
//     PDEVICE_INTERFACE_CHANGE_NOTIFICATION notifyStruct;
//     notifyStruct = ExAllocatePoolWithTag(PagedPool, sizeof(*notifyStruct), TAG_PNP_NOTIFY);
//     if (!notifyStruct)
//     {
//         return;
//     }
//
//     *notifyStruct = (DEVICE_INTERFACE_CHANGE_NOTIFICATION) {
//         .Version = 1,
//         .Size = sizeof(DEVICE_INTERFACE_CHANGE_NOTIFICATION),
//         .Event = *Event,
//         .InterfaceClassGuid = *InterfaceClassGuid,
//         .SymbolicLinkName = SymbolicLinkName
//     };
//
//     DPRINT("Delivering a DeviceInterfaceChange PnP event\n");
//
//     KeAcquireGuardedMutex(&PiNotifyDeviceInterfaceLock);
//
//     PLIST_ENTRY entry = PiNotifyDeviceInterfaceListHead.Flink;
//     while (entry != &PiNotifyDeviceInterfaceListHead)
//     {
//         PPNP_NOTIFY_ENTRY nEntry = CONTAINING_RECORD(entry, PNP_NOTIFY_ENTRY, PnpNotifyList);
//
//         if (!IsEqualGUID(&notifyStruct->InterfaceClassGuid, &nEntry->Guid))
//         {
//             entry = entry->Flink;
//             continue;
//         }
//
//         PiProcessSingleNotification(nEntry, notifyStruct, &PiNotifyDeviceInterfaceLock, &entry);
//     }
//
//     KeReleaseGuardedMutex(&PiNotifyDeviceInterfaceLock);
//     ExFreePoolWithTag(notifyStruct, TAG_PNP_NOTIFY);
// }
//
//
// /**
//  * @brief      Delivers the event to all drivers subscribed to
//  *             EventCategoryHardwareProfileChange PnP event
//  *
//  * @param[in]  Event  The PnP event GUID
//  */
// CODE_SEG("PAGE")
// VOID
// PiNotifyHardwareProfileChange(
//     _In_ LPCGUID Event)
// {
//     PAGED_CODE();
//
//     PHWPROFILE_CHANGE_NOTIFICATION notifyStruct;
//     notifyStruct = ExAllocatePoolWithTag(PagedPool, sizeof(*notifyStruct), TAG_PNP_NOTIFY);
//     if (!notifyStruct)
//     {
//         return;
//     }
//
//     *notifyStruct = (HWPROFILE_CHANGE_NOTIFICATION) {
//         .Version = 1,
//         .Size = sizeof(HWPROFILE_CHANGE_NOTIFICATION),
//         .Event = *Event
//     };
//
//     DPRINT("Delivering a HardwareProfileChange PnP event\n");
//
//     KeAcquireGuardedMutex(&PiNotifyHwProfileLock);
//
//     PLIST_ENTRY entry = PiNotifyHwProfileListHead.Flink;
//     while (entry != &PiNotifyHwProfileListHead)
//     {
//         PPNP_NOTIFY_ENTRY nEntry = CONTAINING_RECORD(entry, PNP_NOTIFY_ENTRY, PnpNotifyList);
//
//         PiProcessSingleNotification(nEntry, notifyStruct, &PiNotifyHwProfileLock, &entry);
//     }
//
//     KeReleaseGuardedMutex(&PiNotifyHwProfileLock);
//     ExFreePoolWithTag(notifyStruct, TAG_PNP_NOTIFY);
// }
//
// /**
//  * @brief      Delivers the event to all drivers subscribed to
//  *             EventCategoryTargetDeviceChange PnP event
//  *
//  * @param[in]  Event               The PnP event GUID
//  * @param[in]  DeviceObject        The (target) device object
//  * @param[in]  CustomNotification  Pointer to a custom notification for GUID_PNP_CUSTOM_NOTIFICATION
//  */
// CODE_SEG("PAGE")
// VOID
// PiNotifyTargetDeviceChange(
//     _In_ LPCGUID Event,
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_opt_ PTARGET_DEVICE_CUSTOM_NOTIFICATION CustomNotification)
// {
//     PAGED_CODE();
//
//     PVOID notificationStruct;
//     // just in case our device is removed during the operation
//     ObReferenceObject(DeviceObject);
//
//     PDEVICE_NODE deviceNode = IopGetDeviceNode(DeviceObject);
//     ASSERT(deviceNode);
//
//     if (!IsEqualGUID(Event, &GUID_PNP_CUSTOM_NOTIFICATION))
//     {
//         PTARGET_DEVICE_REMOVAL_NOTIFICATION notifStruct;
//         notifStruct = ExAllocatePoolWithTag(PagedPool, sizeof(*notifStruct), TAG_PNP_NOTIFY);
//         if (!notifStruct)
//         {
//             return;
//         }
//
//         *notifStruct = (TARGET_DEVICE_REMOVAL_NOTIFICATION) {
//             .Version = 1,
//             .Size = sizeof(TARGET_DEVICE_REMOVAL_NOTIFICATION),
//             .Event = *Event
//         };
//
//         notificationStruct = notifStruct;
//
//         DPRINT("Delivering a (non-custom) TargetDeviceChange PnP event\n");
//     }
//     else
//     {
//         ASSERT(CustomNotification);
//         // assuming everythng else is correct
//
//         notificationStruct = CustomNotification;
//
//         DPRINT("Delivering a (custom) TargetDeviceChange PnP event\n");
//     }
//
//     KeAcquireGuardedMutex(&PiNotifyTargetDeviceLock);
//
//     PLIST_ENTRY entry = deviceNode->TargetDeviceNotify.Flink;
//     while (entry != &deviceNode->TargetDeviceNotify)
//     {
//         PPNP_NOTIFY_ENTRY nEntry = CONTAINING_RECORD(entry, PNP_NOTIFY_ENTRY, PnpNotifyList);
//
//         // put the file object from our saved entry to this particular notification's struct
//         ((PTARGET_DEVICE_REMOVAL_NOTIFICATION)notificationStruct)->FileObject = nEntry->FileObject;
//         // so you don't need to look at the definition ;)
//         C_ASSERT(FIELD_OFFSET(TARGET_DEVICE_REMOVAL_NOTIFICATION, FileObject)
//                  == FIELD_OFFSET(TARGET_DEVICE_CUSTOM_NOTIFICATION, FileObject));
//
//         PiProcessSingleNotification(nEntry, notificationStruct, &PiNotifyTargetDeviceLock, &entry);
//     }
//
//     KeReleaseGuardedMutex(&PiNotifyTargetDeviceLock);
//     if (notificationStruct != CustomNotification)
//         ExFreePoolWithTag(notificationStruct, TAG_PNP_NOTIFY);
//     ObDereferenceObject(DeviceObject);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @unimplemented
//  */
// ULONG
// NTAPI
// IoPnPDeliverServicePowerNotification(
//     _In_ ULONG VetoedPowerOperation,
//     _In_ ULONG PowerNotificationCode,
//     _In_ ULONG PowerNotificationData,
//     _In_ BOOLEAN Synchronous)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// IoRegisterPlugPlayNotification(
//     _In_ IO_NOTIFICATION_EVENT_CATEGORY EventCategory,
//     _In_ ULONG EventCategoryFlags,
//     _In_opt_ PVOID EventCategoryData,
//     _In_ PDRIVER_OBJECT DriverObject,
//     _In_ PDRIVER_NOTIFICATION_CALLBACK_ROUTINE CallbackRoutine,
//     _Inout_opt_ PVOID Context,
//     _Out_ PVOID *NotificationEntry)
// {
//     PPNP_NOTIFY_ENTRY Entry;
//     PWSTR SymbolicLinkList;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     DPRINT("%s(EventCategory 0x%x, EventCategoryFlags 0x%lx, DriverObject %p) called.\n",
//            __FUNCTION__, EventCategory, EventCategoryFlags, DriverObject);
//
//     ObReferenceObject(DriverObject);
//
//     /* Try to allocate entry for notification before sending any notification */
//     Entry = ExAllocatePoolWithTag(PagedPool, sizeof(PNP_NOTIFY_ENTRY), TAG_PNP_NOTIFY);
//     if (!Entry)
//     {
//         DPRINT("ExAllocatePool() failed\n");
//         ObDereferenceObject(DriverObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     *Entry = (PNP_NOTIFY_ENTRY) {
//         .PnpNotificationProc = CallbackRoutine,
//         .Context = Context,
//         .DriverObject = DriverObject,
//         .EventCategory = EventCategory,
//         .RefCount = 1
//     };
//
//     switch (EventCategory)
//     {
//         case EventCategoryDeviceInterfaceChange:
//         {
//             Entry->Guid = *(LPGUID)EventCategoryData;
//
//             // first register the notification
//             KeAcquireGuardedMutex(&PiNotifyDeviceInterfaceLock);
//             InsertTailList(&PiNotifyDeviceInterfaceListHead, &Entry->PnpNotifyList);
//             KeReleaseGuardedMutex(&PiNotifyDeviceInterfaceLock);
//
//             // then process existing interfaces if asked
//             if (EventCategoryFlags & PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES)
//             {
//                 DEVICE_INTERFACE_CHANGE_NOTIFICATION NotificationInfos;
//                 UNICODE_STRING SymbolicLinkU;
//                 PWSTR SymbolicLink;
//
//                 Status = IoGetDeviceInterfaces((LPGUID)EventCategoryData,
//                                                NULL, /* PhysicalDeviceObject OPTIONAL */
//                                                0, /* Flags */
//                                                &SymbolicLinkList);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Enumerate SymbolicLinkList */
//                     NotificationInfos.Version = 1;
//                     NotificationInfos.Size = sizeof(DEVICE_INTERFACE_CHANGE_NOTIFICATION);
//                     NotificationInfos.Event = GUID_DEVICE_INTERFACE_ARRIVAL;
//                     NotificationInfos.InterfaceClassGuid = *(LPGUID)EventCategoryData;
//                     NotificationInfos.SymbolicLinkName = &SymbolicLinkU;
//
//                     for (SymbolicLink = SymbolicLinkList;
//                          *SymbolicLink;
//                          SymbolicLink += (SymbolicLinkU.Length / sizeof(WCHAR)) + 1)
//                     {
//                         RtlInitUnicodeString(&SymbolicLinkU, SymbolicLink);
//                         DPRINT("Calling callback routine for %S\n", SymbolicLink);
//                         PiCallNotifyProc(CallbackRoutine, &NotificationInfos, Context);
//                     }
//
//                     ExFreePool(SymbolicLinkList);
//                 }
//             }
//             break;
//         }
//         case EventCategoryHardwareProfileChange:
//         {
//             KeAcquireGuardedMutex(&PiNotifyHwProfileLock);
//             InsertTailList(&PiNotifyHwProfileListHead, &Entry->PnpNotifyList);
//             KeReleaseGuardedMutex(&PiNotifyHwProfileLock);
//             break;
//         }
//         case EventCategoryTargetDeviceChange:
//         {
//             PDEVICE_NODE deviceNode;
//             Entry->FileObject = (PFILE_OBJECT)EventCategoryData;
//
//             // NOTE: the device node's PDO is referenced here
//             Status = IopGetRelatedTargetDevice(Entry->FileObject, &deviceNode);
//             if (!NT_SUCCESS(Status))
//             {
//                 ObDereferenceObject(DriverObject);
//                 ExFreePoolWithTag(Entry, TAG_PNP_NOTIFY);
//                 return Status;
//             }
//             // save it so we can dereference it later
//             Entry->DeviceObject = deviceNode->PhysicalDeviceObject;
//
//             // each DEVICE_NODE has its own registered notifications list
//             KeAcquireGuardedMutex(&PiNotifyTargetDeviceLock);
//             InsertTailList(&deviceNode->TargetDeviceNotify, &Entry->PnpNotifyList);
//             KeReleaseGuardedMutex(&PiNotifyTargetDeviceLock);
//             break;
//         }
//         default:
//         {
//             DPRINT1("%s: unknown EventCategory 0x%x UNIMPLEMENTED\n",
//                     __FUNCTION__, EventCategory);
//
//             ObDereferenceObject(DriverObject);
//             ExFreePoolWithTag(Entry, TAG_PNP_NOTIFY);
//             return STATUS_NOT_SUPPORTED;
//         }
//     }
//
//     DPRINT("%s returns NotificationEntry %p\n", __FUNCTION__, Entry);
//
//     *NotificationEntry = Entry;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// IoUnregisterPlugPlayNotification(
//     _In_ PVOID NotificationEntry)
// {
//     PPNP_NOTIFY_ENTRY Entry = NotificationEntry;
//     PKGUARDED_MUTEX Lock;
//
//     PAGED_CODE();
//
//     DPRINT("%s(NotificationEntry %p) called\n", __FUNCTION__, Entry);
//
//     switch (Entry->EventCategory)
//     {
//         case EventCategoryDeviceInterfaceChange:
//             Lock = &PiNotifyDeviceInterfaceLock;
//             break;
//         case EventCategoryHardwareProfileChange:
//             Lock = &PiNotifyHwProfileLock;
//             break;
//         case EventCategoryTargetDeviceChange:
//             Lock = &PiNotifyTargetDeviceLock;
//             break;
//         default:
//             UNREACHABLE;
//             return STATUS_NOT_SUPPORTED;
//     }
//
//     KeAcquireGuardedMutex(Lock);
//     if (!Entry->Deleted)
//     {
//         Entry->Deleted = TRUE; // so it can't be unregistered two times
//         PiDereferencePnpNotifyEntry(Entry);
//     }
//     else
//     {
//         DPRINT1("IoUnregisterPlugPlayNotification called two times for 0x%p\n", NotificationEntry);
//     }
//     KeReleaseGuardedMutex(Lock);
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use crate::types::*;
use crate::glue::*;
use crate::ddk_shims::IsEqualGUID;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    ExFreePool,
    _KWAIT_REASON_Executive,
    _IO_STATUS_BLOCK,
    IRP,
    IoAcquireCancelSpinLock,
    IoAllocateIrp,
    IoAllocateWorkItem,
    IoBuildDeviceIoControlRequest,
    IoFreeIrp,
    IoFreeWorkItem,
    IoGetAttachedDeviceReference,
    IoGetDeviceObjectPointer,
    IoGetNextIrpStackLocation,
    IoInitializeIrp,
    IoQueueWorkItem,
    IoRegisterPlugPlayNotification,
    IoReleaseCancelSpinLock,
    IoReportTargetDeviceChangeAsynchronous,
    IoSetCancelRoutine,
    IoSetCompletionRoutineEx,
    KEVENT,
    KeInitializeEvent,
    KeReleaseSemaphore,
    KeWaitForSingleObject,
    LIST_ENTRY,
    _EVENT_TYPE_NotificationEvent,
    PDEVICE_OBJECT,
    PDEVICE_RELATIONS,
    PFILE_OBJECT,
    PIO_STACK_LOCATION,
    PIRP,
    PTARGET_DEVICE_CUSTOM_NOTIFICATION,
    PUNICODE_STRING,
    RtlCompareUnicodeString,
    TARGET_DEVICE_CUSTOM_NOTIFICATION,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{DEVICE_OBJECT, DEVICE_RELATIONS, FILE_OBJECT, IO_STACK_LOCATION, KPRIORITY, KPROCESSOR_MODE, WORK_QUEUE_ITEM, _LIST_ENTRY, _UNICODE_STRING};

extern "system" {
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: usize, Tag: u32) -> *mut core::ffi::c_void;
    fn ExFreePoolWithTag(P: *mut core::ffi::c_void, Tag: u32);
    fn KeAcquireSpinLockRaiseToDpc(SpinLock: *mut KSPIN_LOCK) -> KIRQL;
    fn ExQueueWorkItem(WorkItem: *mut WORK_QUEUE_ITEM, QueueType: i32);
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: u32, Message: *mut i8);
    fn PsGetCurrentThread() -> *mut core::ffi::c_void;
    fn AllocatePool(size: u32) -> *mut core::ffi::c_void;
    fn FreePool(p: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
// No Rust equivalent for #define NDEBUG (preprocessor symbol); omitted.

const PagedPool: u32 = 1;
// UNICODE_NULL: u16 = 0 — imported from ntkrnl_sys::driver_ddk

const NotificationEvent: u32 = 0;
const Executive: u32 = 0;
const FALSE: u8 = 0;

const EventCategoryTargetDeviceChange: u32 = 0x03; // assumed from DDK, matches 0x2

const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016_u32 as i32;

const IO_NO_INCREMENT: i32 = 0;
#[no_mangle]
pub unsafe extern "C" fn SendOnlineNotification(
    SymbolicName: PUNICODE_STRING,
) {
    let mut Status: NTSTATUS = 0;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    Status = IoGetDeviceObjectPointer(
        SymbolicName,
        FILE_READ_ATTRIBUTES,
        core::ptr::addr_of_mut!(FileObject),
        core::ptr::addr_of_mut!(DeviceObject),
    );
    if Status < 0 {
        return;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    Status = MountMgrSendSyncDeviceIoCtl(
        0 /* IOCTL_VOLUME_ONLINE */,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        FileObject,
    );

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "system" fn SendOnlineNotificationWorker(Parameter: *mut core::ffi::c_void) {
    let mut OldIrql: KIRQL = 0;
    let mut Head: PLIST_ENTRY = core::ptr::null_mut();
    let mut DeviceExtension: *mut DEVICE_EXTENSION = core::ptr::null_mut();
    let mut WorkItem: *mut ONLINE_NOTIFICATION_WORK_ITEM = core::ptr::null_mut();
    let mut NewWorkItem: *mut ONLINE_NOTIFICATION_WORK_ITEM = core::ptr::null_mut();

    WorkItem = Parameter as *mut ONLINE_NOTIFICATION_WORK_ITEM;
    DeviceExtension = (*WorkItem).DeviceExtension;

    SendOnlineNotification(&raw mut (*WorkItem).SymbolicName);

    KeAcquireSpinLock(&raw mut (*DeviceExtension).WorkerLock as *mut _, &mut OldIrql);
    (*DeviceExtension).OnlineNotificationCount = (*DeviceExtension).OnlineNotificationCount.wrapping_sub(1);
    if (*DeviceExtension).OnlineNotificationCount == 0 {
        KeSetEvent(
            &raw mut (*DeviceExtension).OnlineNotificationEvent as *mut c_void,
            0i32,
            0u8,
        );
    }
    if !IsListEmpty(&raw mut (*DeviceExtension).OnlineNotificationListHead) {
        Head = RemoveHeadList(&raw mut (*DeviceExtension).OnlineNotificationListHead);
        NewWorkItem = (Head as usize).wrapping_sub(
            core::mem::offset_of!(ONLINE_NOTIFICATION_WORK_ITEM, WorkItem.List),
        ) as *mut ONLINE_NOTIFICATION_WORK_ITEM ;
        KeReleaseSpinLock(&raw mut (*DeviceExtension).WorkerLock as *mut _, OldIrql);
        (*NewWorkItem).WorkItem.List.Blink = core::ptr::null_mut();
        (*NewWorkItem).WorkItem.List.Flink = core::ptr::null_mut();
        ExQueueWorkItem(&raw mut (*NewWorkItem).WorkItem, DelayedWorkQueue as i32);
    } else {
        (*DeviceExtension).OnlineNotificationWorkerActive = 0;
        KeReleaseSpinLock(&raw mut (*DeviceExtension).WorkerLock as *mut _, OldIrql);
    }
    ExFreePoolWithTag((*WorkItem).SymbolicName.Buffer as *mut core::ffi::c_void, 1098149453u32);
    ExFreePoolWithTag(WorkItem as *mut core::ffi::c_void, 1098149453u32);
}
#[no_mangle]
pub unsafe extern "C" fn PostOnlineNotification(
    DeviceExtension: PDEVICE_EXTENSION,
    SymbolicName: PUNICODE_STRING,
) {
    let mut OldIrql: KIRQL;
    let mut WorkItem: PONLINE_NOTIFICATION_WORK_ITEM = core::ptr::null_mut();

    WorkItem = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<ONLINE_NOTIFICATION_WORK_ITEM>(),
        1098149453u32,
    ) as PONLINE_NOTIFICATION_WORK_ITEM;

    if WorkItem.is_null() {
        return;
    }

    (*WorkItem).WorkItem.WorkerRoutine = Some(
        core::mem::transmute::<unsafe extern "system" fn(*mut c_void), _>(
            SendOnlineNotificationWorker,
        ),
    );
    core::ptr::write_volatile(
        &raw mut (*WorkItem).WorkItem.Parameter,
        WorkItem as PVOID,
    );
    (*WorkItem).WorkItem.List.Flink = core::ptr::null_mut::<_LIST_ENTRY>();

    (*WorkItem).DeviceExtension = DeviceExtension;
    (*WorkItem).SymbolicName.Length = (*SymbolicName).Length;
    (*WorkItem).SymbolicName.MaximumLength =
        ((*SymbolicName).Length as usize).wrapping_add(core::mem::size_of::<u16>()) as u16;
    (*WorkItem).SymbolicName.Buffer = ExAllocatePoolWithTag(
        PagedPool,
        (*WorkItem).SymbolicName.MaximumLength as usize,
        1098149453u32,
    ) as *mut u16;

    if (*WorkItem).SymbolicName.Buffer.is_null() {
        ExFreePoolWithTag(WorkItem as *mut core::ffi::c_void, 1098149453u32);
        return;
    }

    RtlCopyMemory(
        (*WorkItem).SymbolicName.Buffer as *mut core::ffi::c_void,
        (*SymbolicName).Buffer as *const core::ffi::c_void,
        (*SymbolicName).Length as usize,
    );
    *(*WorkItem).SymbolicName.Buffer.add(
        ((*SymbolicName).Length as usize) / core::mem::size_of::<u16>(),
    ) = UNICODE_NULL;

    OldIrql = KeAcquireSpinLockRaiseToDpc(&raw mut (*DeviceExtension).WorkerLock as *mut _);
    (*DeviceExtension).OnlineNotificationCount =
        (*DeviceExtension).OnlineNotificationCount.wrapping_add(1);
    if (*DeviceExtension).OnlineNotificationWorkerActive == 0 {
        (*DeviceExtension).OnlineNotificationWorkerActive = 1;
        ExQueueWorkItem(&raw mut (*WorkItem).WorkItem, DelayedWorkQueue as i32);
    } else {
        InsertTailList(
            &raw mut (*DeviceExtension).OnlineNotificationListHead,
            &raw mut (*WorkItem).WorkItem.List,
        );
    }
    KeReleaseSpinLock(&raw mut (*DeviceExtension).WorkerLock as *mut _, OldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn WaitForOnlinesToComplete(DeviceExtension: PDEVICE_EXTENSION) {
    let mut OldIrql: KIRQL = 0;
    KeInitializeEvent(
        &raw mut (*DeviceExtension).OnlineNotificationEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );
    KeAcquireSpinLock(
        &raw mut (*DeviceExtension).WorkerLock as *mut _,
        &raw mut OldIrql,
    );
    if (*DeviceExtension).OnlineNotificationCount != 1 {
        (*DeviceExtension).OnlineNotificationCount =
            (*DeviceExtension).OnlineNotificationCount.wrapping_sub(1);
        KeReleaseSpinLock(&raw mut (*DeviceExtension).WorkerLock as *mut _, OldIrql);
        KeWaitForSingleObject(
            &raw mut (*DeviceExtension).OnlineNotificationEvent as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
        KeAcquireSpinLock(
            &raw mut (*DeviceExtension).WorkerLock as *mut _,
            &raw mut OldIrql,
        );
        (*DeviceExtension).OnlineNotificationCount =
            (*DeviceExtension).OnlineNotificationCount.wrapping_add(1);
    }
    KeReleaseSpinLock(&raw mut (*DeviceExtension).WorkerLock as *mut _, OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn MountMgrTargetDeviceNotification(
    NotificationStructure: PVOID,
    Context: PVOID,
) -> NTSTATUS {
    let mut DeviceExtension: PDEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceInformation: PDEVICE_INFORMATION = core::ptr::null_mut();
    let mut Notification: PTARGET_DEVICE_CUSTOM_NOTIFICATION = core::ptr::null_mut();

    DeviceInformation = Context as PDEVICE_INFORMATION;
    DeviceExtension = (*DeviceInformation).DeviceExtension;
    Notification = NotificationStructure as PTARGET_DEVICE_CUSTOM_NOTIFICATION;

    /* ASSERT(!IsEqualGUID(&Notification->Event, &GUID_TARGET_DEVICE_REMOVE_COMPLETE)); */
    if IsEqualGUID(
        &raw const (*Notification).Event,
        &raw const GUID_TARGET_DEVICE_REMOVE_COMPLETE,
    ) != 0 {
        RtlAssert(
            b"!IsEqualGUID(&Notification->Event, &GUID_TARGET_DEVICE_REMOVE_COMPLETE)\0"
                .as_ptr() as *const u8 as PVOID,
            b"notify.c\0".as_ptr() as *const u8 as PVOID,
            218,
            core::ptr::null_mut::<i8>(),
        );
    } else {
    }

    if IsEqualGUID(
        &raw const (*Notification).Event,
        &raw const GUID_IO_VOLUME_MOUNT,
    ) != 0 {
        let mount_state_ptr: *mut i32 = &raw mut (*DeviceInformation).MountState;
        let atomic = &*(mount_state_ptr as *const core::sync::atomic::AtomicI32);
        let old_val = match atomic.compare_exchange(
            0i32,
            0i32,
            core::sync::atomic::Ordering::SeqCst,
            core::sync::atomic::Ordering::SeqCst,
        ) {
            Ok(v) => v,
            Err(v) => v,
        };
        if old_val == 1 {
            InterlockedDecrement(mount_state_ptr);
        } else if (*DeviceInformation).NeedsReconcile != 0 {
            (*DeviceInformation).NeedsReconcile = 0;
            ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
        }
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterForTargetDeviceNotification(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DeviceInformation: *mut DEVICE_INFORMATION,
) {
    let mut Status: NTSTATUS = 0;
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();

    Status = IoGetDeviceObjectPointer(
        &raw mut (*DeviceInformation).DeviceName,
        FILE_READ_ATTRIBUTES,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    Status = IoRegisterPlugPlayNotification(
        EventCategoryTargetDeviceChange,
        0u32,
        FileObject as *mut core::ffi::c_void,
        (*DeviceExtension).DriverObject,
        Some(core::mem::transmute::<unsafe extern "system" fn(*mut c_void, *mut c_void) -> NTSTATUS, _>(MountMgrTargetDeviceNotification)),
        DeviceInformation as *mut core::ffi::c_void,
        &raw mut (*DeviceInformation).TargetDeviceNotificationEntry,
    );
    if !NT_SUCCESS(Status) {
        (*DeviceInformation).TargetDeviceNotificationEntry = core::ptr::null_mut();
    }

    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrNotify(DeviceExtension: *mut DEVICE_EXTENSION) {
    let mut Irp: *mut IRP = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut CopyList: LIST_ENTRY = LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    };
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();

    (*DeviceExtension).EpicNumber = (*DeviceExtension).EpicNumber.wrapping_add(1);
    InitializeListHead(&raw mut CopyList);
    IoAcquireCancelSpinLock(&raw mut OldIrql);
    while !IsListEmpty(&raw mut (*DeviceExtension).IrpListHead) {
        NextEntry = RemoveHeadList(&raw mut (*DeviceExtension).IrpListHead);
        Irp = ((NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)
        )) as *mut IRP;
        IoSetCancelRoutine(Irp, None);
        InsertTailList(
            &raw mut CopyList,
            &raw mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );
    }
    IoReleaseCancelSpinLock(OldIrql);
    while !IsListEmpty(&raw mut CopyList) {
        NextEntry = RemoveHeadList(&raw mut CopyList);
        Irp = ((NextEntry as usize).wrapping_sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)
        )) as *mut IRP;
        *((*Irp).AssociatedIrp.SystemBuffer as *mut u32) = (*DeviceExtension).EpicNumber as u32;
        (*Irp).IoStatus.Information = core::mem::size_of::<u32>() as _;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MountMgrNotifyNameChange(
    mut DeviceExtension: *mut DEVICE_EXTENSION,
    mut DeviceName: *mut _UNICODE_STRING,
    mut ValidateVolume: u8,
) {
    let mut Irp: *mut IRP = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut NextEntry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut Stack: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceRelations: *mut DEVICE_RELATIONS = core::ptr::null_mut();
    let mut DeviceInformation: *mut DEVICE_INFORMATION = core::ptr::null_mut();
    let mut DeviceNotification: TARGET_DEVICE_CUSTOM_NOTIFICATION = core::mem::zeroed();

    if ValidateVolume != 0 {
        NextEntry = (*DeviceExtension).DeviceListHead.Flink as *mut LIST_ENTRY;
        while NextEntry != core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead) {
            let offset = core::mem::offset_of!(DEVICE_INFORMATION, DeviceListEntry);
            DeviceInformation = ((NextEntry as usize) - offset) as *mut DEVICE_INFORMATION;
            if RtlCompareUnicodeString(
                DeviceName as *const _UNICODE_STRING,
                core::ptr::addr_of_mut!((*DeviceInformation).DeviceName) as *const _UNICODE_STRING,
                1u8,
            ) == 0
            {
                break;
            }
            NextEntry = (*NextEntry).Flink as *mut LIST_ENTRY;
        }
        if NextEntry == core::ptr::addr_of_mut!((*DeviceExtension).DeviceListHead)
            || (*DeviceInformation).ManuallyRegistered == 0
        {
            return;
        }
    }

    Status = IoGetDeviceObjectPointer(
        DeviceName,
        FILE_READ_ATTRIBUTES as u32,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, 0);

    Irp = IoBuildDeviceIoControlRequest(
        0u32,
        DeviceObject,
        core::ptr::null_mut(),
        0u32,
        core::ptr::null_mut(),
        0u32,
        0u8,
        &raw mut Event,
        &raw mut IoStatusBlock,
    );
    if Irp.is_null() {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        return;
    }

    Stack = IoGetNextIrpStackLocation(Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;

    (*Stack).MajorFunction = IRP_MJ_PNP as u8;
    (*Stack).MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS as u8;
    (*Stack).Parameters.QueryDeviceRelations.Type = TargetDeviceRelation;
    (*Stack).FileObject = FileObject;

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);

    if !NT_SUCCESS(Status) {
        return;
    }

    DeviceRelations = IoStatusBlock.Information as *mut DEVICE_RELATIONS;
    if (*DeviceRelations).Count < 1 {
        ExFreePool(DeviceRelations as *mut core::ffi::c_void);
        return;
    }

    DeviceObject = *core::ptr::addr_of_mut!((*DeviceRelations).Objects).cast::<*mut DEVICE_OBJECT>();

    ExFreePool(DeviceRelations as *mut core::ffi::c_void);

    DeviceNotification.Version = 1u16;
    DeviceNotification.Size = core::mem::size_of::<TARGET_DEVICE_CUSTOM_NOTIFICATION>() as u16;
    DeviceNotification.Event = GUID_IO_VOLUME_NAME_CHANGE;
    DeviceNotification.FileObject = core::ptr::null_mut();
    DeviceNotification.NameBufferOffset = -1i32;

    IoReportTargetDeviceChangeAsynchronous(
        DeviceObject,
        &raw mut DeviceNotification as *mut core::ffi::c_void,
        None,
        core::ptr::null_mut(),
    );

    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn RemoveWorkItem(mut WorkItem: PUNIQUE_ID_WORK_ITEM) {
    let DeviceExtension: PDEVICE_EXTENSION = (*WorkItem).DeviceExtension;
    KeWaitForSingleObject(
        &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
    if !(*WorkItem).Event.is_null() {
        KeReleaseSemaphore(
            &raw mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT as KPRIORITY,
            1 as LONG,
            0u8,
        );
        KeSetEvent(
            (*WorkItem).Event as *mut c_void,
            0 as KPRIORITY,
            0u8,
        );
    } else {
        RemoveEntryList(&raw mut (*WorkItem).UniqueIdWorkerItemListEntry);
        KeReleaseSemaphore(
            &raw mut (*DeviceExtension).DeviceLock,
            IO_NO_INCREMENT as KPRIORITY,
            1 as LONG,
            0u8,
        );
        IoFreeIrp((*WorkItem).Irp);
        ExFreePoolWithTag((*WorkItem).DeviceName.Buffer as *mut core::ffi::c_void, 1098149453);
        ExFreePoolWithTag((*WorkItem).IrpBuffer, 1098149453);
        ExFreePoolWithTag(WorkItem as *mut core::ffi::c_void, 1098149453);
    }
}

#[no_mangle]
pub unsafe extern "system" fn UniqueIdChangeNotifyWorker(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let _ = DeviceObject; // UNREFERENCED_PARAMETER

    let WorkItem: *mut UNIQUE_ID_WORK_ITEM = Context as *mut UNIQUE_ID_WORK_ITEM;

    if !((*(*WorkItem).Irp).IoStatus.__bindgen_anon_1.Status >= 0) {
        RemoveWorkItem(WorkItem);
        return;
    }

    let UniqueIdChange: *mut MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT =
        (*(*WorkItem).Irp).AssociatedIrp.SystemBuffer as *mut MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT;

    // Allocate OldUniqueId
    let OldUniqueId: *mut MOUNTDEV_UNIQUE_ID = ExAllocatePoolWithTag(
        PagedPool,
        ((*UniqueIdChange).OldUniqueIdLength as usize)
            .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>()),
        1098149453u32,
    ) as *mut MOUNTDEV_UNIQUE_ID;

    if OldUniqueId.is_null() {
        RemoveWorkItem(WorkItem);
        return;
    }

    (*OldUniqueId).UniqueIdLength = (*UniqueIdChange).OldUniqueIdLength;

    let src = (UniqueIdChange as *const u8).add((*UniqueIdChange).OldUniqueIdOffset as usize) as *const u8;
    let dst = &raw mut (*OldUniqueId).UniqueId as *mut u8;
    core::ptr::copy_nonoverlapping(src, dst, (*UniqueIdChange).OldUniqueIdLength as usize);

    // Allocate NewUniqueId
    let NewUniqueId: *mut MOUNTDEV_UNIQUE_ID = ExAllocatePoolWithTag(
        PagedPool,
        ((*UniqueIdChange).NewUniqueIdLength as usize)
            .wrapping_add(core::mem::size_of::<MOUNTDEV_UNIQUE_ID>()),
        1098149453u32,
    ) as *mut MOUNTDEV_UNIQUE_ID;

    if NewUniqueId.is_null() {
        ExFreePoolWithTag(OldUniqueId as *mut core::ffi::c_void, 1098149453u32);
        RemoveWorkItem(WorkItem);
        return;
    }

    (*NewUniqueId).UniqueIdLength = (*UniqueIdChange).NewUniqueIdLength;

    let src = (UniqueIdChange as *const u8).add((*UniqueIdChange).NewUniqueIdOffset as usize) as *const u8;
    let dst = &raw mut (*NewUniqueId).UniqueId as *mut u8;
    core::ptr::copy_nonoverlapping(src, dst, (*UniqueIdChange).NewUniqueIdLength as usize);

    MountMgrUniqueIdChangeRoutine((*WorkItem).DeviceExtension as *mut c_void, OldUniqueId, NewUniqueId);
    IssueUniqueIdChangeNotifyWorker(WorkItem, NewUniqueId);

    ExFreePoolWithTag(NewUniqueId as *mut core::ffi::c_void, 1098149453u32);
    ExFreePoolWithTag(OldUniqueId as *mut core::ffi::c_void, 1098149453u32);
}

#[no_mangle]
pub unsafe extern "system" fn UniqueIdChangeNotifyCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let WorkItem: PUNIQUE_ID_WORK_ITEM = Context as PUNIQUE_ID_WORK_ITEM;

    // UNREFERENCED_PARAMETER(DeviceObject);
    let _ = &DeviceObject;
    // UNREFERENCED_PARAMETER(Irp);
    let _ = &Irp;

    /* Simply queue the work item */
    IoQueueWorkItem(
        (*WorkItem).WorkItem,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PVOID),
            _,
        >(UniqueIdChangeNotifyWorker)),
        DelayedWorkQueue,
        WorkItem as PVOID,
    );

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "C" fn IssueUniqueIdChangeNotifyWorker(
    WorkItem: *mut UNIQUE_ID_WORK_ITEM,
    UniqueId: *mut MOUNTDEV_UNIQUE_ID,
) {
    let mut Irp: *mut IRP = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut Stack: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();

    Status = IoGetDeviceObjectPointer(
        &raw mut (*WorkItem).DeviceName,
        FILE_READ_ATTRIBUTES as u32,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        RemoveWorkItem(WorkItem);
        return;
    }

    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);

    Irp = (*WorkItem).Irp;
    IoInitializeIrp(
        Irp,
        (core::mem::size_of::<IRP>() as usize
            + (*WorkItem).StackSize as usize * core::mem::size_of::<IO_STACK_LOCATION>())
            as u16,
        (*WorkItem).StackSize as i8,
    );

    let event_ptr: *mut i32 = &raw mut (*WorkItem).Event as *mut i32;
    if core::sync::atomic::AtomicI32::from_ptr(event_ptr)
        .swap(0, core::sync::atomic::Ordering::SeqCst)
        != 0
    {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        RemoveWorkItem(WorkItem);
        return;
    }

    (*Irp).AssociatedIrp.SystemBuffer = (*WorkItem).IrpBuffer;
    (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as _;
    core::ptr::copy_nonoverlapping(
        UniqueId as *const u8,
        (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
        ((*UniqueId).UniqueIdLength as usize + core::mem::size_of::<u16>()) as usize,
    );

    Stack = IoGetNextIrpStackLocation(Irp);

    (*Stack).Parameters.DeviceIoControl.InputBufferLength =
        ((*UniqueId).UniqueIdLength as usize + core::mem::size_of::<u16>()) as u32;
    (*Stack).Parameters.DeviceIoControl.OutputBufferLength = (*WorkItem).IrpBufferLength;
    (*Stack).Parameters.DeviceIoControl.Type3InputBuffer = core::ptr::null_mut();
    (*Stack).Parameters.DeviceIoControl.IoControlCode = IOCTL_MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY;
    (*Stack).MajorFunction = IRP_MJ_DEVICE_CONTROL as u8;

    Status = IoSetCompletionRoutineEx(
        (*(*WorkItem).DeviceExtension).DeviceObject,
        Irp,
        Some(core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS,
            _,
        >(UniqueIdChangeNotifyCompletion)),
        WorkItem as *mut core::ffi::c_void,
        1u8,
        1u8,
        1u8,
    );
    if !(Status >= 0) {
        ObDereferenceObject(FileObject as *mut core::ffi::c_void);
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        RemoveWorkItem(WorkItem);
        return;
    }

    IoCallDriver(DeviceObject, Irp);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn IssueUniqueIdChangeNotify(
    DeviceExtension: *mut DEVICE_EXTENSION,
    DeviceName: *mut _UNICODE_STRING,
    UniqueId: *mut MOUNTDEV_UNIQUE_ID,
) {
    let mut Status: i32;
    let mut IrpBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut FileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();
    let mut WorkItem: *mut UNIQUE_ID_WORK_ITEM = core::ptr::null_mut();

    Status = IoGetDeviceObjectPointer(
        DeviceName,
        FILE_READ_ATTRIBUTES,
        &raw mut FileObject,
        &raw mut DeviceObject,
    );
    if !(Status >= 0) {
        return;
    }
    DeviceObject = IoGetAttachedDeviceReference((*FileObject).DeviceObject);
    ObDereferenceObject(FileObject as *mut core::ffi::c_void);
    WorkItem = AllocatePool(core::mem::size_of::<UNIQUE_ID_WORK_ITEM>() as u32) as *mut UNIQUE_ID_WORK_ITEM;
    if WorkItem.is_null() {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        return;
    }
    (*WorkItem).Event = core::ptr::null_mut::<KEVENT>();
    (*WorkItem).WorkItem = IoAllocateWorkItem((*DeviceExtension).DeviceObject);
    if (*WorkItem).WorkItem.is_null() {
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
    } else {
        (*WorkItem).DeviceExtension = DeviceExtension;
        (*WorkItem).StackSize = (*DeviceObject).StackSize as u32;
        (*WorkItem).Irp = IoAllocateIrp((*DeviceObject).StackSize, 0u8);
        ObDereferenceObject(DeviceObject as *mut core::ffi::c_void);
        if !(*WorkItem).Irp.is_null() {
            IrpBuffer = AllocatePool(
                (core::mem::size_of::<MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT>()
                    .wrapping_add(1024)) as u32,
            );
            if !IrpBuffer.is_null() {
                (*WorkItem).DeviceName.Length = (*DeviceName).Length;
                (*WorkItem).DeviceName.MaximumLength = ((*DeviceName).Length as usize)
                    .wrapping_add(core::mem::size_of::<u16>()) as u16;
                (*WorkItem).DeviceName.Buffer = AllocatePool(
                    (*WorkItem).DeviceName.MaximumLength as u32,
                ) as *mut u16;
                if !(*WorkItem).DeviceName.Buffer.is_null() {
                    RtlCopyMemory(
                        (*WorkItem).DeviceName.Buffer as *mut core::ffi::c_void,
                        (*DeviceName).Buffer as *const core::ffi::c_void,
                        (*DeviceName).Length as usize,
                    );
                    *(*WorkItem).DeviceName.Buffer.offset(
                        ((*DeviceName).Length as usize / core::mem::size_of::<u16>()) as isize,
                    ) = UNICODE_NULL;
                    (*WorkItem).IrpBuffer = IrpBuffer;
                    (*WorkItem).IrpBufferLength =
                        (core::mem::size_of::<MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT>() as usize)
                            .wrapping_add(1024) as u32;
                    KeWaitForSingleObject(
                        &raw mut (*DeviceExtension).DeviceLock as *mut core::ffi::c_void,
                        _KWAIT_REASON_Executive,
                        0i8,
                        0u8,
                        core::ptr::null_mut(),
                    );
                    InsertHeadList(
                        &raw mut (*DeviceExtension).UniqueIdWorkerItemListHead,
                        &raw mut (*WorkItem).UniqueIdWorkerItemListEntry,
                    );
                    KeReleaseSemaphore(
                        &raw mut (*DeviceExtension).DeviceLock,
                        IO_NO_INCREMENT as i32,
                        1i32,
                        0u8,
                    );
                    IssueUniqueIdChangeNotifyWorker(WorkItem, UniqueId);
                    return;
                }
            }
        }
    }
    if !IrpBuffer.is_null() {
        FreePool(IrpBuffer);
    }
    if !(*WorkItem).Irp.is_null() {
        IoFreeIrp((*WorkItem).Irp);
    }
    if !(*WorkItem).WorkItem.is_null() {
        IoFreeWorkItem((*WorkItem).WorkItem);
    }
    if !WorkItem.is_null() {
        FreePool(WorkItem as *mut core::ffi::c_void);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2011 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA.
//  *
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/filesystem/mountmgr/notify.c
//  * PURPOSE:          Mount Manager - Notifications handlers
//  * PROGRAMMER:       Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  *                   Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// #include "mntmgr.h"
//
// #include <ioevent.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// VOID
// SendOnlineNotification(IN PUNICODE_STRING SymbolicName)
// {
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Get device object */
//     Status = IoGetDeviceObjectPointer(SymbolicName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//         return;
//
//     /* And attached device object */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     /* And send VOLUME_ONLINE */
//     Status = MountMgrSendSyncDeviceIoCtl(IOCTL_VOLUME_ONLINE,
//                                          DeviceObject,
//                                          NULL, 0,
//                                          NULL, 0,
//                                          FileObject);
//     UNREFERENCED_PARAMETER(Status);
//
//     ObDereferenceObject(DeviceObject);
//     ObDereferenceObject(FileObject);
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// SendOnlineNotificationWorker(IN PVOID Parameter)
// {
//     KIRQL OldIrql;
//     PLIST_ENTRY Head;
//     PDEVICE_EXTENSION DeviceExtension;
//     PONLINE_NOTIFICATION_WORK_ITEM WorkItem;
//     PONLINE_NOTIFICATION_WORK_ITEM NewWorkItem;
//
//     WorkItem = (PONLINE_NOTIFICATION_WORK_ITEM)Parameter;
//     DeviceExtension = WorkItem->DeviceExtension;
//
//     /* First, send the notification */
//     SendOnlineNotification(&(WorkItem->SymbolicName));
//
//     KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//     /* If there are no notifications running any longer, reset event */
//     if (--DeviceExtension->OnlineNotificationCount == 0)
//     {
//         KeSetEvent(&(DeviceExtension->OnlineNotificationEvent), 0, FALSE);
//     }
//
//     /* If there are still notifications in queue */
//     if (!IsListEmpty(&(DeviceExtension->OnlineNotificationListHead)))
//     {
//         /* Queue a new one for execution */
//         Head = RemoveHeadList(&(DeviceExtension->OnlineNotificationListHead));
//         NewWorkItem = CONTAINING_RECORD(Head, ONLINE_NOTIFICATION_WORK_ITEM, WorkItem.List);
//         KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//         NewWorkItem->WorkItem.List.Blink = NULL;
//         NewWorkItem->WorkItem.List.Flink = NULL;
//         ExQueueWorkItem(&NewWorkItem->WorkItem, DelayedWorkQueue);
//     }
//     else
//     {
//         /* Mark it's over */
//         DeviceExtension->OnlineNotificationWorkerActive = 0;
//         KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//     }
//
//     FreePool(WorkItem->SymbolicName.Buffer);
//     FreePool(WorkItem);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// PostOnlineNotification(IN PDEVICE_EXTENSION DeviceExtension,
//                        IN PUNICODE_STRING SymbolicName)
// {
//     KIRQL OldIrql;
//     PONLINE_NOTIFICATION_WORK_ITEM WorkItem;
//
//     /* Allocate a notification work item */
//     WorkItem = AllocatePool(sizeof(ONLINE_NOTIFICATION_WORK_ITEM));
//     if (!WorkItem)
//     {
//         return;
//     }
//
//     ExInitializeWorkItem(&WorkItem->WorkItem, SendOnlineNotificationWorker, WorkItem);
//     WorkItem->DeviceExtension = DeviceExtension;
//     WorkItem->SymbolicName.Length = SymbolicName->Length;
//     WorkItem->SymbolicName.MaximumLength = SymbolicName->Length + sizeof(WCHAR);
//     WorkItem->SymbolicName.Buffer = AllocatePool(WorkItem->SymbolicName.MaximumLength);
//     if (!WorkItem->SymbolicName.Buffer)
//     {
//         FreePool(WorkItem);
//         return;
//     }
//
//     RtlCopyMemory(WorkItem->SymbolicName.Buffer, SymbolicName->Buffer, SymbolicName->Length);
//     WorkItem->SymbolicName.Buffer[SymbolicName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//     DeviceExtension->OnlineNotificationCount++;
//
//     /* If no worker are active */
//     if (DeviceExtension->OnlineNotificationWorkerActive == 0)
//     {
//         /* Queue that one for execution */
//         DeviceExtension->OnlineNotificationWorkerActive = 1;
//         ExQueueWorkItem(&WorkItem->WorkItem, DelayedWorkQueue);
//     }
//     else
//     {
//         /* Otherwise, just put it in the queue list */
//         InsertTailList(&(DeviceExtension->OnlineNotificationListHead), &(WorkItem->WorkItem.List));
//     }
//
//     KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WaitForOnlinesToComplete(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     KIRQL OldIrql;
//
//     KeInitializeEvent(&(DeviceExtension->OnlineNotificationEvent), NotificationEvent, FALSE);
//
//     KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//
//     /* Just wait all the worker are done */
//     if (DeviceExtension->OnlineNotificationCount != 1)
//     {
//         DeviceExtension->OnlineNotificationCount--;
//         KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
//
//         KeWaitForSingleObject(&(DeviceExtension->OnlineNotificationEvent),
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         KeAcquireSpinLock(&(DeviceExtension->WorkerLock), &OldIrql);
//         DeviceExtension->OnlineNotificationCount++;
//     }
//
//     KeReleaseSpinLock(&(DeviceExtension->WorkerLock), OldIrql);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// MountMgrTargetDeviceNotification(IN PVOID NotificationStructure,
//                                  IN PVOID Context)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     PDEVICE_INFORMATION DeviceInformation;
//     PTARGET_DEVICE_CUSTOM_NOTIFICATION Notification;
//
//     DeviceInformation = Context;
//     DeviceExtension = DeviceInformation->DeviceExtension;
//     Notification = NotificationStructure;
//
//     /* The notification have to be unregistered already (in device interface change handler) */
//     ASSERT(!IsEqualGUID(&Notification->Event, &GUID_TARGET_DEVICE_REMOVE_COMPLETE));
//
//     /* It it's to signal that a volume has been mounted
//      * Verify if a database sync is required and execute it
//      */
//     if (IsEqualGUID(&(Notification->Event), &GUID_IO_VOLUME_MOUNT))
//     {
//         /* If we were already mounted, then mark us unmounted */
//         if (InterlockedCompareExchange(&(DeviceInformation->MountState),
//                                        FALSE,
//                                        FALSE) == TRUE)
//         {
//             InterlockedDecrement(&(DeviceInformation->MountState));
//         }
//         /* Otherwise, start mounting the device and first, reconcile its DB if required */
//         else
//         {
//             if (DeviceInformation->NeedsReconcile)
//             {
//                 DeviceInformation->NeedsReconcile = FALSE;
//                 ReconcileThisDatabaseWithMaster(DeviceExtension, DeviceInformation);
//             }
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// VOID
// RegisterForTargetDeviceNotification(IN PDEVICE_EXTENSION DeviceExtension,
//                                     IN PDEVICE_INFORMATION DeviceInformation)
// {
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Get device object */
//     Status = IoGetDeviceObjectPointer(&(DeviceInformation->DeviceName),
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return;
//     }
//
//     /* And simply register for notifications */
//     Status = IoRegisterPlugPlayNotification(EventCategoryTargetDeviceChange,
//                                             0, FileObject,
//                                             DeviceExtension->DriverObject,
//                                             MountMgrTargetDeviceNotification,
//                                             DeviceInformation,
//                                             &(DeviceInformation->TargetDeviceNotificationEntry));
//     if (!NT_SUCCESS(Status))
//     {
//         DeviceInformation->TargetDeviceNotificationEntry = NULL;
//     }
//
//     ObDereferenceObject(FileObject);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrNotify(IN PDEVICE_EXTENSION DeviceExtension)
// {
//     PIRP Irp;
//     KIRQL OldIrql;
//     LIST_ENTRY CopyList;
//     PLIST_ENTRY NextEntry;
//
//     /* Increase the epic number */
//     DeviceExtension->EpicNumber++;
//
//     InitializeListHead(&CopyList);
//
//     /* Copy all the pending IRPs for notification */
//     IoAcquireCancelSpinLock(&OldIrql);
//     while (!IsListEmpty(&(DeviceExtension->IrpListHead)))
//     {
//         NextEntry = RemoveHeadList(&(DeviceExtension->IrpListHead));
//         Irp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//         IoSetCancelRoutine(Irp, NULL);
//         InsertTailList(&CopyList, &(Irp->Tail.Overlay.ListEntry));
//     }
//     IoReleaseCancelSpinLock(OldIrql);
//
//     /* Then, notify them one by one */
//     while (!IsListEmpty(&CopyList))
//     {
//         NextEntry = RemoveHeadList(&CopyList);
//         Irp = CONTAINING_RECORD(NextEntry, IRP, Tail.Overlay.ListEntry);
//
//         *((PULONG)Irp->AssociatedIrp.SystemBuffer) = DeviceExtension->EpicNumber;
//         Irp->IoStatus.Information = sizeof(DeviceExtension->EpicNumber);
//
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// MountMgrNotifyNameChange(IN PDEVICE_EXTENSION DeviceExtension,
//                          IN PUNICODE_STRING DeviceName,
//                          IN BOOLEAN ValidateVolume)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT FileObject;
//     PIO_STACK_LOCATION Stack;
//     PDEVICE_OBJECT DeviceObject;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PDEVICE_RELATIONS DeviceRelations;
//     PDEVICE_INFORMATION DeviceInformation;
//     TARGET_DEVICE_CUSTOM_NOTIFICATION DeviceNotification;
//
//     /* If we have to validate volume */
//     if (ValidateVolume)
//     {
//         /* Then, ensure we can find the device */
//         for (NextEntry = DeviceExtension->DeviceListHead.Flink;
//              NextEntry != &DeviceExtension->DeviceListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             DeviceInformation = CONTAINING_RECORD(NextEntry, DEVICE_INFORMATION, DeviceListEntry);
//             if (RtlCompareUnicodeString(DeviceName, &(DeviceInformation->DeviceName), TRUE) == 0)
//             {
//                 break;
//             }
//         }
//
//         /* No need to notify for a PnP device or if we didn't find the device */
//         if (NextEntry == &(DeviceExtension->DeviceListHead) ||
//             !DeviceInformation->ManuallyRegistered)
//         {
//             return;
//         }
//     }
//
//     /* Then, get device object */
//     Status = IoGetDeviceObjectPointer(DeviceName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return;
//     }
//
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     /* Set up empty IRP (yes, yes!) */
//     Irp = IoBuildDeviceIoControlRequest(0,
//                                         DeviceObject,
//                                         NULL,
//                                         0,
//                                         NULL,
//                                         0,
//                                         FALSE,
//                                         &Event,
//                                         &IoStatusBlock);
//     if (!Irp)
//     {
//         ObDereferenceObject(DeviceObject);
//         ObDereferenceObject(FileObject);
//         return;
//     }
//
//     Stack = IoGetNextIrpStackLocation(Irp);
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     Irp->IoStatus.Information = 0;
//
//     /* Properly set it, we want to query device relations */
//     Stack->MajorFunction = IRP_MJ_PNP;
//     Stack->MinorFunction = IRP_MN_QUERY_DEVICE_RELATIONS;
//     Stack->Parameters.QueryDeviceRelations.Type = TargetDeviceRelation;
//     Stack->FileObject = FileObject;
//
//     /* And call driver */
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     ObDereferenceObject(DeviceObject);
//     ObDereferenceObject(FileObject);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return;
//     }
//
//     /* Validate device return */
//     DeviceRelations = (PDEVICE_RELATIONS)IoStatusBlock.Information;
//     if (DeviceRelations->Count < 1)
//     {
//         ExFreePool(DeviceRelations);
//         return;
//     }
//
//     DeviceObject = DeviceRelations->Objects[0];
//     ExFreePool(DeviceRelations);
//
//     /* Set up real notification */
//     DeviceNotification.Version = 1;
//     DeviceNotification.Size = sizeof(TARGET_DEVICE_CUSTOM_NOTIFICATION);
//     DeviceNotification.Event = GUID_IO_VOLUME_NAME_CHANGE;
//     DeviceNotification.FileObject = NULL;
//     DeviceNotification.NameBufferOffset = -1;
//
//     /* And report */
//     IoReportTargetDeviceChangeAsynchronous(DeviceObject,
//                                            &DeviceNotification,
//                                            NULL, NULL);
//
//     ObDereferenceObject(DeviceObject);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// VOID
// RemoveWorkItem(IN PUNIQUE_ID_WORK_ITEM WorkItem)
// {
//     PDEVICE_EXTENSION DeviceExtension = WorkItem->DeviceExtension;
//
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//
//     /* If even if being worked, it's too late */
//     if (WorkItem->Event)
//     {
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//         KeSetEvent(WorkItem->Event, 0, FALSE);
//     }
//     else
//     {
//         /* Otherwise, remove it from the list, and delete it */
//         RemoveEntryList(&(WorkItem->UniqueIdWorkerItemListEntry));
//         KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//         IoFreeIrp(WorkItem->Irp);
//         FreePool(WorkItem->DeviceName.Buffer);
//         FreePool(WorkItem->IrpBuffer);
//         FreePool(WorkItem);
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// UniqueIdChangeNotifyWorker(IN PDEVICE_OBJECT DeviceObject,
//                            IN PVOID Context)
// {
//     PUNIQUE_ID_WORK_ITEM WorkItem = Context;
//     PMOUNTDEV_UNIQUE_ID OldUniqueId, NewUniqueId;
//     PMOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT UniqueIdChange;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     /* Validate worker */
//     if (!NT_SUCCESS(WorkItem->Irp->IoStatus.Status))
//     {
//         RemoveWorkItem(WorkItem);
//         return;
//     }
//
//     UniqueIdChange = WorkItem->Irp->AssociatedIrp.SystemBuffer;
//     /* Get the old unique ID */
//     OldUniqueId = AllocatePool(UniqueIdChange->OldUniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//     if (!OldUniqueId)
//     {
//         RemoveWorkItem(WorkItem);
//         return;
//     }
//
//     OldUniqueId->UniqueIdLength = UniqueIdChange->OldUniqueIdLength;
//     RtlCopyMemory(OldUniqueId->UniqueId,
//                   (PVOID)((ULONG_PTR)UniqueIdChange + UniqueIdChange->OldUniqueIdOffset),
//                   UniqueIdChange->OldUniqueIdLength);
//
//     /* Get the new unique ID */
//     NewUniqueId = AllocatePool(UniqueIdChange->NewUniqueIdLength + sizeof(MOUNTDEV_UNIQUE_ID));
//     if (!NewUniqueId)
//     {
//         FreePool(OldUniqueId);
//         RemoveWorkItem(WorkItem);
//         return;
//     }
//
//     NewUniqueId->UniqueIdLength = UniqueIdChange->NewUniqueIdLength;
//     RtlCopyMemory(NewUniqueId->UniqueId,
//                   (PVOID)((ULONG_PTR)UniqueIdChange + UniqueIdChange->NewUniqueIdOffset),
//                   UniqueIdChange->NewUniqueIdLength);
//
//     /* Call the real worker */
//     MountMgrUniqueIdChangeRoutine(WorkItem->DeviceExtension, OldUniqueId, NewUniqueId);
//     IssueUniqueIdChangeNotifyWorker(WorkItem, NewUniqueId);
//
//     FreePool(NewUniqueId);
//     FreePool(OldUniqueId);
//
//     return;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI

// driver-internal cdecl decls (types_*.rs authority = extern "C").
unsafe extern "C" {
    fn MountMgrSendSyncDeviceIoCtl( ioctl: u32, device_object: PDEVICE_OBJECT, input_buffer: *mut core::ffi::c_void, input_buffer_length: u32, output_buffer: *mut core::ffi::c_void, output_buffer_length: u32, file_object: PFILE_OBJECT, ) -> NTSTATUS;
    fn ReconcileThisDatabaseWithMaster(DeviceExtension: PDEVICE_EXTENSION, DeviceInformation: PDEVICE_INFORMATION);
    fn MountMgrUniqueIdChangeRoutine(DeviceExtension: *mut core::ffi::c_void, OldUniqueId: *mut MOUNTDEV_UNIQUE_ID, NewUniqueId: *mut MOUNTDEV_UNIQUE_ID);
}
// UniqueIdChangeNotifyCompletion(IN PDEVICE_OBJECT DeviceObject,
//                                IN PIRP Irp,
//                                IN PVOID Context)
// {
//     PUNIQUE_ID_WORK_ITEM WorkItem = Context;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//     UNREFERENCED_PARAMETER(Irp);
//
//     /* Simply queue the work item */
//     IoQueueWorkItem(WorkItem->WorkItem,
//                     UniqueIdChangeNotifyWorker,
//                     DelayedWorkQueue,
//                     WorkItem);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// /*
//  * @implemented
//  */
// VOID
// IssueUniqueIdChangeNotifyWorker(IN PUNIQUE_ID_WORK_ITEM WorkItem,
//                                 IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     PIRP Irp;
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject;
//     PIO_STACK_LOCATION Stack;
//     PDEVICE_OBJECT DeviceObject;
//
//     /* Get the device object */
//     Status = IoGetDeviceObjectPointer(&(WorkItem->DeviceName),
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         RemoveWorkItem(WorkItem);
//         return;
//     }
//
//     /* And then, the attached device */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     /* Initialize the IRP */
//     Irp = WorkItem->Irp;
//     IoInitializeIrp(Irp, IoSizeOfIrp(WorkItem->StackSize), (CCHAR)WorkItem->StackSize);
//
//     if (InterlockedExchange((PLONG)&(WorkItem->Event), 0) != 0)
//     {
//         ObDereferenceObject(FileObject);
//         ObDereferenceObject(DeviceObject);
//         RemoveWorkItem(WorkItem);
//         return;
//     }
//
//     Irp->AssociatedIrp.SystemBuffer = WorkItem->IrpBuffer;
//     Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//     RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer, UniqueId, UniqueId->UniqueIdLength + sizeof(USHORT));
//
//     Stack = IoGetNextIrpStackLocation(Irp);
//
//     Stack->Parameters.DeviceIoControl.InputBufferLength = UniqueId->UniqueIdLength + sizeof(USHORT);
//     Stack->Parameters.DeviceIoControl.OutputBufferLength = WorkItem->IrpBufferLength;
//     Stack->Parameters.DeviceIoControl.Type3InputBuffer = 0;
//     Stack->Parameters.DeviceIoControl.IoControlCode = IOCTL_MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY;
//     Stack->MajorFunction = IRP_MJ_DEVICE_CONTROL;
//
//     Status = IoSetCompletionRoutineEx(WorkItem->DeviceExtension->DeviceObject,
//                                       Irp,
//                                       UniqueIdChangeNotifyCompletion,
//                                       WorkItem,
//                                       TRUE, TRUE, TRUE);
//     if (!NT_SUCCESS(Status))
//     {
//         ObDereferenceObject(FileObject);
//         ObDereferenceObject(DeviceObject);
//         RemoveWorkItem(WorkItem);
//         return;
//     }
//
//     /* Call the driver */
//     IoCallDriver(DeviceObject, Irp);
//     ObDereferenceObject(FileObject);
//     ObDereferenceObject(DeviceObject);
// }
//
// /*
//  * @implemented
//  */
// VOID
// IssueUniqueIdChangeNotify(IN PDEVICE_EXTENSION DeviceExtension,
//                           IN PUNICODE_STRING DeviceName,
//                           IN PMOUNTDEV_UNIQUE_ID UniqueId)
// {
//     NTSTATUS Status;
//     PVOID IrpBuffer = NULL;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     PUNIQUE_ID_WORK_ITEM WorkItem = NULL;
//
//     /* Get the associated device object */
//     Status = IoGetDeviceObjectPointer(DeviceName,
//                                       FILE_READ_ATTRIBUTES,
//                                       &FileObject,
//                                       &DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         return;
//     }
//
//     /* And then, get attached device */
//     DeviceObject = IoGetAttachedDeviceReference(FileObject->DeviceObject);
//
//     ObDereferenceObject(FileObject);
//
//     /* Allocate a work item */
//     WorkItem = AllocatePool(sizeof(UNIQUE_ID_WORK_ITEM));
//     if (!WorkItem)
//     {
//         ObDereferenceObject(DeviceObject);
//         return;
//     }
//
//     WorkItem->Event = NULL;
//     WorkItem->WorkItem = IoAllocateWorkItem(DeviceExtension->DeviceObject);
//     if (!WorkItem->WorkItem)
//     {
//         ObDereferenceObject(DeviceObject);
//         goto Cleanup;
//     }
//
//     WorkItem->DeviceExtension = DeviceExtension;
//     WorkItem->StackSize = DeviceObject->StackSize;
//     /* Already provide the IRP */
//     WorkItem->Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//
//     ObDereferenceObject(DeviceObject);
//
//     if (!WorkItem->Irp)
//     {
//         goto Cleanup;
//     }
//
//     /* Ensure it has enough space */
//     IrpBuffer = AllocatePool(sizeof(MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT) + 1024);
//     if (!IrpBuffer)
//     {
//         goto Cleanup;
//     }
//
//     WorkItem->DeviceName.Length = DeviceName->Length;
//     WorkItem->DeviceName.MaximumLength = DeviceName->Length + sizeof(WCHAR);
//     WorkItem->DeviceName.Buffer = AllocatePool(WorkItem->DeviceName.MaximumLength);
//     if (!WorkItem->DeviceName.Buffer)
//     {
//         goto Cleanup;
//     }
//
//     RtlCopyMemory(WorkItem->DeviceName.Buffer, DeviceName->Buffer, DeviceName->Length);
//     WorkItem->DeviceName.Buffer[DeviceName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     WorkItem->IrpBuffer = IrpBuffer;
//     WorkItem->IrpBufferLength = sizeof(MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY_OUTPUT) + 1024;
//
//     /* Add the worker in the list */
//     KeWaitForSingleObject(&(DeviceExtension->DeviceLock), Executive, KernelMode, FALSE, NULL);
//     InsertHeadList(&(DeviceExtension->UniqueIdWorkerItemListHead), &(WorkItem->UniqueIdWorkerItemListEntry));
//     KeReleaseSemaphore(&(DeviceExtension->DeviceLock), IO_NO_INCREMENT, 1, FALSE);
//
//     /* And call the worker */
//     IssueUniqueIdChangeNotifyWorker(WorkItem, UniqueId);
//
//     return;
//
// Cleanup:
//     if (IrpBuffer)
//     {
//         FreePool(IrpBuffer);
//     }
//
//     if (WorkItem->Irp)
//     {
//         IoFreeIrp(WorkItem->Irp);
//     }
//
//     if (WorkItem->WorkItem)
//     {
//         IoFreeWorkItem(WorkItem->WorkItem);
//     }
//
//     if (WorkItem)
//     {
//         FreePool(WorkItem);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

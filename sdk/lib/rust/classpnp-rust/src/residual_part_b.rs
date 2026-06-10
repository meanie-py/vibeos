use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BusRelations, BusTypeFibre, BusTypeRAID, BusTypeiScsi, DEVICE_DATA_SET_RANGE, DEVICE_DSM_OFFLOAD_READ_PARAMETERS, DEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, DEVICE_MANAGE_DATA_SET_ATTRIBUTES, DEVICE_RELATIONS, DPFLTR_CLASSPNP_ID, DeviceNumber, DevicePropertyRemovalPolicy, ExFreePool, ExInitializeRundownProtectionCacheAware, ExReInitializeRundownProtectionCacheAware, ExSizeOfRundownProtectionCacheAware, ExWaitForRundownProtectionReleaseCacheAware, _KWAIT_REASON_Executive, GUID, IO_COMPLETION_ROUTINE, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, IRP, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateMdl, IoBuildDeviceIoControlRequest, IoCopyCurrentIrpStackLocationToNext, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoFreeIrp, IoFreeMdl, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoGetDeviceProperty, IoGetDriverObjectExtension, IoGetNextIrpStackLocation, IoInitializeIrp, IoInvalidateDeviceRelations, IoIs32bitProcess, IoMarkIrpPending, IoOpenDeviceRegistryKey, IoReadAccess, IoRegisterDeviceInterface, IoSetCompletionRoutine, IoSetDeviceInterfaceState, IoSetNextIrpStackLocation, IoSetStartIoAttributes, IoStartNextPacket, IoWMIRegistrationControl, IoWriteAccess, IoWriteErrorLogEntry, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeDelayExecutionThread, KeEnterCriticalRegion, KeGetRecommendedSharedDataAlignment, KeInitializeEvent, KeInitializeSpinLock, KeLeaveCriticalRegion, KeReleaseInStackQueuedSpinLock, KeSetTimerEx, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, MOUNTDEV_NAME, MmBuildMdlForNonPagedPool, MmProbeAndLockPages, MmUnlockPages, NTSTRSAFE_PSTR, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByPointer, PCCHAR, PCSZ, PDEVICE_CAPABILITIES, PDEVICE_DATA_SET_RANGE, PDEVICE_DSM_OFFLOAD_READ_PARAMETERS, PDEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, PDEVICE_MANAGE_DATA_SET_ATTRIBUTES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDRIVER_OBJECT, PEX_RUNDOWN_REF_CACHE_AWARE, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIO_WORKITEM, PIRP, PKEVENT, PMDL, PMOUNTDEV_NAME, PPREVENT_MEDIA_REMOVAL, PREVENT_MEDIA_REMOVAL, PRTL_QUERY_REGISTRY_TABLE, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_NUMBER, PSTORAGE_HOTPLUG_INFO, PSTORAGE_OFFLOAD_READ_OUTPUT, PSTORAGE_OFFLOAD_WRITE_OUTPUT, PSTORAGE_PROPERTY_ID, PSTORAGE_PROPERTY_QUERY, PSTORAGE_READ_CAPACITY, PULONGLONG, PUNICODE_STRING, PVPB, PowerDeviceD0, PropertyStandardQuery, PsGetCurrentThread, RTL_QUERY_REGISTRY_TABLE, RemovalPolicyExpectOrderlyRemoval, RemovalPolicyExpectSurpriseRemoval, RtlAnsiStringToUnicodeString, RtlDeleteRegistryValue, RtlFreeUnicodeString, RtlInitString, RtlInitUnicodeString, RtlQueryRegistryValues, STORAGE_DEVICE_NUMBER, STORAGE_HOTPLUG_INFO, STORAGE_OFFLOAD_READ_OUTPUT, STORAGE_OFFLOAD_WRITE_OUTPUT, STORAGE_PROPERTY_QUERY, STORAGE_READ_CAPACITY, STRING, StorageAccessAlignmentProperty, StorageDeviceCopyOffloadProperty, StorageDeviceLBProvisioningProperty, StorageDeviceMediumProductType, StorageDeviceSeekPenaltyProperty, StorageDeviceTrimProperty, StorageDeviceUniqueIdProperty, StorageDeviceWriteCacheProperty, _EVENT_TYPE_SynchronizationEvent, UINT32, _KWAIT_REASON_UserRequest, ZwClose, ZwOpenKey, ZwSetValueKey};
use core::ffi::c_void;




// ==== lock.c + obsolete.c residual exports ====

#[no_mangle]
pub unsafe extern "system" fn ClassAcquireRemoveLockEx(
    DeviceObject: PDEVICE_OBJECT,
    Tag: PVOID,
    File: PCSTR,
    Line: ULONG
) -> ULONG {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let rundownAcquired: BOOLEAN;
    let removeLockRundown: PEX_RUNDOWN_REF_CACHE_AWARE;

    #[cfg(debug_assertions)]
    {
        let lockValue: LONG;

        lockValue = InterlockedIncrement(&mut (*commonExtension).RemoveLock);

        TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_LOCK, 
                    b"ClassAcquireRemoveLock: Acquired for Object %p & irp %p - count is %d\n\0".as_ptr() as PCSTR,
                    DeviceObject, Tag, lockValue));

        NT_ASSERTMSG(b"ClassAcquireRemoveLock - lock value was negative : \0".as_ptr() as PCSTR,
                     (lockValue > 0));

        NT_ASSERTMSG(b"RemoveLock increased to meet LockHighWatermark\0".as_ptr() as PCSTR,
                     ((LockHighWatermark == 0) || (lockValue != LockHighWatermark)));

        if (*commonExtension).IsRemoved != REMOVE_COMPLETE {
            let removeTrackingList: PRTL_GENERIC_TABLE;
            let mut trackingBlock: REMOVE_TRACKING_BLOCK = core::mem::zeroed();
            let mut insertedTrackingBlock: PREMOVE_TRACKING_BLOCK = core::ptr::null_mut();
            let mut newElement: BOOLEAN = 0;

            let mut oldIrql: KIRQL = 0;

            trackingBlock.Tag = Tag;
            trackingBlock.File = File;
            trackingBlock.Line = Line;

            KeQueryTickCount(&mut trackingBlock.TimeLocked);

            KeAcquireSpinLock(&mut (*commonExtension).RemoveTrackingSpinlock, &mut oldIrql);

            removeTrackingList = (*commonExtension).RemoveTrackingList;

            if !removeTrackingList.is_null() {
                insertedTrackingBlock = RtlInsertElementGenericTable(
                    removeTrackingList,
                    &trackingBlock as *const _ as PVOID,
                    core::mem::size_of::<REMOVE_TRACKING_BLOCK>() as ULONG,
                    &mut newElement
                ) as PREMOVE_TRACKING_BLOCK;
            }

            if !insertedTrackingBlock.is_null() {
                if newElement == 0 {
                    TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_LOCK, 
                                b">>>>>ClassAcquireRemoveLock: already tracking Tag %p\n\0".as_ptr() as PCSTR,
                                Tag));
                    TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_LOCK, 
                                b">>>>>ClassAcquireRemoveLock: acquired in file %s on line %d\n\0".as_ptr() as PCSTR,
                                (*insertedTrackingBlock).File, (*insertedTrackingBlock).Line));
                }
            } else {
                (*commonExtension).RemoveTrackingUntrackedCount += 1;

                TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_LOCK, 
                            b">>>>>ClassAcquireRemoveLock: Cannot track Tag %p - currently %d untracked requsts\n\0".as_ptr() as PCSTR,
                            Tag, (*commonExtension).RemoveTrackingUntrackedCount));
            }

            KeReleaseSpinLock(&mut (*commonExtension).RemoveTrackingSpinlock, oldIrql);
        }
    }

    #[cfg(not(debug_assertions))]
    {
        let _ = Tag;
        let _ = File;
        let _ = Line;
    }

    removeLockRundown = (((*commonExtension).PrivateCommonData as PCHAR)
        .offset(core::mem::size_of::<CLASS_PRIVATE_COMMON_DATA>() as isize))
        as PEX_RUNDOWN_REF_CACHE_AWARE;
    rundownAcquired = ExAcquireRundownProtectionCacheAware(removeLockRundown);
    if rundownAcquired == 0 {
        InterlockedIncrement(&mut (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire as *mut LONG as *mut i32);
        TracePrint((TRACE_LEVEL_VERBOSE,
                    TRACE_FLAG_LOCK,
                    b"ClassAcquireRemoveLockEx: RemoveLockRundown acquisition failed RemoveLockFailAcquire = %d\n\0".as_ptr() as PCSTR,
                    (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire));
    }

    (*commonExtension).IsRemoved
}

#[no_mangle]
pub unsafe extern "system" fn ClassReleaseRemoveLock(
    DeviceObject: PDEVICE_OBJECT,
    Tag: PIRP
) {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let mut lockValue: LONG;
    let mut oldValue: LONG;
    let mut removeLockRundown: PEX_RUNDOWN_REF_CACHE_AWARE = core::ptr::null_mut();

    #[cfg(debug_assertions)]
    {
        let mut removeTrackingList: PRTL_GENERIC_TABLE = core::ptr::null_mut();
        let mut searchDataBlock: REMOVE_TRACKING_BLOCK = core::mem::zeroed();

        let mut found: BOOLEAN = 0u8;

        let isRemoved: BOOLEAN = if (*commonExtension).IsRemoved == REMOVE_COMPLETE { 1u8 } else { 0u8 };

        let mut oldIrql: KIRQL = core::mem::zeroed();

        if isRemoved != 0 {
            InterlockedDecrement(&mut (*commonExtension).RemoveLock);
            return;
        }

        KeAcquireSpinLock(&mut (*commonExtension).RemoveTrackingSpinlock,
                          &mut oldIrql);

        removeTrackingList = (*commonExtension).RemoveTrackingList;

        if !removeTrackingList.is_null() {
            searchDataBlock.Tag = Tag as PVOID;
            found = RtlDeleteElementGenericTable(removeTrackingList, &mut searchDataBlock as *mut _ as PVOID);
        }

        if found == 0 {
            if (*commonExtension).RemoveTrackingUntrackedCount == 0 {
            } else {
                (*commonExtension).RemoveTrackingUntrackedCount -= 1;
            }
        }

        KeReleaseSpinLock(&mut (*commonExtension).RemoveTrackingSpinlock,
                          oldIrql);

        lockValue = InterlockedDecrement(&mut (*commonExtension).RemoveLock);

        if lockValue == 0 {
        }
    }

    #[cfg(not(debug_assertions))]
    {
    }

    oldValue = 1;
    lockValue = (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire;
    while lockValue != 0 {
        oldValue = {
            let destination = &mut (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire as *mut LONG;
            let exchange = lockValue - 1;
            let comparand = lockValue;
            let result: LONG;
            core::arch::asm!(
                "lock cmpxchg dword ptr [{0}], {2}",
                in(reg) destination,
                inout("eax") comparand => result,
                in(reg) exchange,
                options(nostack)
            );
            result
        };

        if oldValue == lockValue {
            break;
        }

        lockValue = oldValue;
    }

    if lockValue == 0 {
        removeLockRundown = ((*commonExtension).PrivateCommonData as *mut u8)
            .add(core::mem::size_of::<CLASS_PRIVATE_COMMON_DATA>()) as PEX_RUNDOWN_REF_CACHE_AWARE;
        ExReleaseRundownProtectionCacheAware(removeLockRundown);
    }

    return;
}

#[no_mangle]
pub unsafe extern "system" fn ClassDeleteSrbLookasideList(CommonExtension: PCOMMON_DEVICE_EXTENSION) {
    // obsolete, still called by some code
    if (*CommonExtension).IsSrbLookasideListInitialized != 0 {
        (*CommonExtension).IsSrbLookasideListInitialized = 0;
        ExDeleteNPagedLookasideList(&raw mut (*CommonExtension).SrbLookasideList);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassInitializeSrbLookasideList(
    CommonExtension: PCOMMON_DEVICE_EXTENSION,
    NumberElements: ULONG,
) {
    let mut sizeNeeded: usize;
    // obsolete, still called by DISK.SYS
    if (*CommonExtension).IsSrbLookasideListInitialized == 0 {
        if (*CommonExtension).IsFdo != 0 {
            let fdo = CommonExtension as PFUNCTIONAL_DEVICE_EXTENSION;
            if (*(*fdo).AdapterDescriptor).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                sizeNeeded = crate::glue::CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
            } else {
                sizeNeeded = core::mem::size_of::<SCSI_REQUEST_BLOCK>();
            }
        } else {
            sizeNeeded = core::cmp::max(
                core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                crate::glue::CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
            );
        }
        ExInitializeNPagedLookasideList(
            &raw mut (*CommonExtension).SrbLookasideList,
            None,
            None,
            POOL_NX_ALLOCATION,
            sizeNeeded,
            u32::from_ne_bytes(*b"$scS"),
            NumberElements as USHORT,
        );
        (*CommonExtension).IsSrbLookasideListInitialized = 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassBuildRequest(Fdo: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    // obsolete, still called by CDROM.SYS
    let srb = ClasspAllocateSrb(fdoExtension);
    if srb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    ClasspBuildRequestEx(fdoExtension, Irp, srb);
    STATUS_SUCCESS
}

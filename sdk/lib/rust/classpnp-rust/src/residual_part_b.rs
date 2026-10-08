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
use ntkrnl_sys::ddk::{DEVICE_DATA_SET_RANGE, DEVICE_DSM_OFFLOAD_READ_PARAMETERS, DEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, DEVICE_MANAGE_DATA_SET_ATTRIBUTES, DEVICE_RELATIONS, DISK_GEOMETRY, ExFreePool, ExInitializeRundownProtectionCacheAware, ExReInitializeRundownProtectionCacheAware, ExSizeOfRundownProtectionCacheAware, ExWaitForRundownProtectionReleaseCacheAware, _KWAIT_REASON_Executive, GUID, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, IRP, IoAllocateDriverObjectExtension, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoCopyCurrentIrpStackLocationToNext, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoFreeIrp, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetDeviceProperty, IoGetDriverObjectExtension, IoGetNextIrpStackLocation, IoGetPagingIoPriority, IoInitializeIrp, IoInvalidateDeviceRelations, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoRegisterDeviceInterface, IoSetCompletionRoutine, IoSetDeviceInterfaceState, IoSetNextIrpStackLocation, IoSetStartIoAttributes, IoStartPacket, IoWMIRegistrationControl, IoWriteErrorLogEntry, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeBugCheck, KeDelayExecutionThread, KeEnterCriticalRegion, KeGetRecommendedSharedDataAlignment, KeInitializeDpc, KeInitializeEvent, KeInitializeSpinLock, KeInitializeTimer, KeLeaveCriticalRegion, KeQueryTimeIncrement, KeReleaseInStackQueuedSpinLock, KeSetTimerEx, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, LPGUID, MOUNTDEV_NAME, MmBuildMdlForNonPagedPool, MmProbeAndLockPages, MmUnlockPages, NTSTRSAFE_PSTR, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByPointer, PCCHAR, PCSZ, PDEVICE_CAPABILITIES, PDEVICE_DATA_SET_RANGE, PDEVICE_DSM_OFFLOAD_READ_PARAMETERS, PDEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, PDEVICE_MANAGE_DATA_SET_ATTRIBUTES, PDEVICE_OBJECT, PDEVICE_POWER_DESCRIPTOR, PDEVICE_RELATIONS, PDRIVER_OBJECT, PEX_RUNDOWN_REF_CACHE_AWARE, PIO_COMPLETION_ROUTINE, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIO_WORKITEM, PIRP, PKEVENT, PMDL, PMOUNTDEV_NAME, PPREVENT_MEDIA_REMOVAL, PREVENT_MEDIA_REMOVAL, PRTL_QUERY_REGISTRY_TABLE, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_NUMBER, PSTORAGE_HOTPLUG_INFO, PSTORAGE_OFFLOAD_READ_OUTPUT, PSTORAGE_OFFLOAD_WRITE_OUTPUT, PSTORAGE_PROPERTY_ID, PSTORAGE_PROPERTY_QUERY, PSTORAGE_READ_CAPACITY, PULONGLONG, PUNICODE_STRING, PVPB, PsGetCurrentThread, RTL_QUERY_REGISTRY_TABLE, RtlAnsiStringToUnicodeString, RtlCopyUnicodeString, RtlDeleteRegistryValue, RtlFreeUnicodeString, RtlInitString, RtlInitUnicodeString, RtlQueryRegistryValues, STORAGE_DEVICE_NUMBER, STORAGE_HOTPLUG_INFO, STORAGE_OFFLOAD_READ_OUTPUT, STORAGE_OFFLOAD_WRITE_OUTPUT, STORAGE_PROPERTY_QUERY, STORAGE_READ_CAPACITY, STRING, UINT32, _KWAIT_REASON_UserRequest, ZwClose, ZwOpenKey, ZwSetValueKey};
// Local aliases for enum variants not exported by ntkrnl_sys::ddk under short names
use ntkrnl_sys::ddk::{
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
    _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _MEDIA_TYPE_RemovableMedia as RemovableMedia,
    _MEDIA_TYPE_FixedMedia as FixedMedia,
    _DEVICE_REMOVAL_POLICY_RemovalPolicyExpectOrderlyRemoval as RemovalPolicyExpectOrderlyRemoval,
    _DEVICE_REMOVAL_POLICY_RemovalPolicyExpectSurpriseRemoval as RemovalPolicyExpectSurpriseRemoval,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypePaging as DeviceUsageTypePaging,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeDumpFile as DeviceUsageTypeDumpFile,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeBoot as DeviceUsageTypeBoot,
    _IO_PAGING_PRIORITY_IoPagingPriorityHigh as IoPagingPriorityHigh,
    _IO_PAGING_PRIORITY_IoPagingPriorityInvalid as IoPagingPriorityInvalid,
    _LOCK_OPERATION_IoReadAccess as IoReadAccess,
    _LOCK_OPERATION_IoWriteAccess as IoWriteAccess,
    _STORAGE_QUERY_TYPE_PropertyStandardQuery as PropertyStandardQuery,
    _STORAGE_PROPERTY_ID_StorageAdapterProperty as StorageAdapterProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceUniqueIdProperty as StorageDeviceUniqueIdProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceWriteCacheProperty as StorageDeviceWriteCacheProperty,
    _STORAGE_PROPERTY_ID_StorageMiniportProperty as StorageMiniportProperty,
    _STORAGE_PROPERTY_ID_StorageAccessAlignmentProperty as StorageAccessAlignmentProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceSeekPenaltyProperty as StorageDeviceSeekPenaltyProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceTrimProperty as StorageDeviceTrimProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceLBProvisioningProperty as StorageDeviceLBProvisioningProperty,
    _STORAGE_PROPERTY_ID_StorageDevicePowerProperty as StorageDevicePowerProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceCopyOffloadProperty as StorageDeviceCopyOffloadProperty,
    _STORAGE_PROPERTY_ID_StorageDeviceMediumProductType as StorageDeviceMediumProductType,
    _STORAGE_PORT_CODE_SET_StoragePortCodeSetStorport as StoragePortCodeSetStorport,
    _STORAGE_PORT_CODE_SET_StoragePortCodeSetSDport as StoragePortCodeSetSDport,
    _STORAGE_PORT_CODE_SET_StoragePortCodeSetUSBport as StoragePortCodeSetUSBport,
    _STORAGE_BUS_TYPE_BusType1394 as BusType1394,
    _STORAGE_BUS_TYPE_BusTypeFibre as BusTypeFibre,
    _STORAGE_BUS_TYPE_BusTypeUsb as BusTypeUsb,
    _STORAGE_BUS_TYPE_BusTypeRAID as BusTypeRAID,
    _STORAGE_BUS_TYPE_BusTypeiScsi as BusTypeiScsi,
    _STORAGE_BUS_TYPE_BusTypeFileBackedVirtual as BusTypeFileBackedVirtual,
    DEVICE_REGISTRY_PROPERTY_DevicePropertyRemovalPolicy as DevicePropertyRemovalPolicy,
    _DPFLTR_TYPE_DPFLTR_CLASSPNP_ID as DPFLTR_CLASSPNP_ID,
    _EVENT_TYPE_SynchronizationEvent,
};
// IO_COMPLETION_ROUTINE is a type alias for PIO_COMPLETION_ROUTINE in ntkrnl_sys::ddk
use ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE as IO_COMPLETION_ROUTINE;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DO_VERIFY_VOLUME, FILE_DEVICE_DISK, FILE_PORTABLE_DEVICE, IRP_MJ_SCSI, SL_OVERRIDE_VERIFY_VOLUME, SL_WRITE_THROUGH, STATUS_DEVICE_BUSY, STATUS_DEVICE_DOES_NOT_EXIST, STATUS_VERIFY_REQUIRED, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING};
use crate::types::{CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED, CLASSP_REG_DISBALE_IDLE_POWER_NAME, CLASSP_REG_PERF_RESTORE_VALUE_NAME, CLASSP_REG_QERR_OVERRIDE_MODE, CLASSP_REG_REMOVAL_POLICY_VALUE_NAME, CLASSP_REG_SUBKEY_NAME, CLASSP_VOLUME_VERIFY_CHECKED, CLASS_DRIVER_EXTENSION, CLASS_INIT_DATA, CLASS_INTERPRET_SENSE_INFO2, CLASS_INTERPRET_SENSE_INFO2_MAXIMUM_HISTORY_COUNT, CLASS_MAX_INTERLEAVE_PER_CRITICAL_IO, CLASS_PERF_RESTORE_MINIMUM, CLASS_QUERY_WMI_REGINFO_EX_LIST, CLASS_SRB_SCSI_REQUEST_BLOCK, CLASS_SRB_STORAGE_REQUEST_BLOCK, CLASS_WORKING_SET, CLASS_WORKING_SET_MAXIMUM, COMMON_DEVICE_EXTENSION, COMPLETION_CONTEXT, ClassGlobalDispatch, ClassMinimalPowerHandler, ClasspPowerSettingCallback, ClasspRetryRequestDpc, ClasspScanForClassHacks, ClasspStartIo, ClasspUpdateDiskProperties, DEV_USE_16BYTE_CDB, FDO_HACK_NO_SYNC_CACHE, FUNCTIONAL_DEVICE_EXTENSION, MAX_TOKEN_LIST_IDENTIFIERS, NUM_IO_RETRIES, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PREAD_CAPACITY_DATA_EX, PSCSI_REQUEST_BLOCK, PTRANSFER_PACKET, QERR_SET_ZERO_ALWAYS, QERR_SET_ZERO_ODX_OR_TP_ONLY, READ_CAPACITY_DATA, READ_CAPACITY_DATA_EX, REG_DISK_CLASS_CONTROL, REMOVE_PENDING, SCSIOP_START_STOP_UNIT, SCSI_REQUEST_BLOCK, SRB_FLAGS_DISABLE_AUTOSENSE, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_FREE_SENSE_BUFFER, SRB_FLAGS_NO_DATA_TRANSFER, SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_STATUS_QUEUE_FROZEN, SRB_TYPE_STORAGE_REQUEST_BLOCK, START_UNIT_TIMEOUT, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK, TRANSFER_PACKET, _CLASS_DEV_INFO, _CLASS_DRIVER_EXTENSION, _CLASS_FUNCTION_SUPPORT_INFO, _CLASS_INIT_DATA, _CLASS_PRIVATE_FDO_DATA, _COMMON_DEVICE_EXTENSION, _COMPLETION_CONTEXT, _FUNCTIONAL_DEVICE_EXTENSION, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK};
use ntkrnl_sys::ddk::{DRIVER_OBJECT, STORAGE_PROPERTY_ID, _DEVICE_OBJECT, _DEVICE_POWER_DESCRIPTOR, _DRIVER_OBJECT, _IO_PAGING_PRIORITY, _IO_PAGING_PRIORITY_IoPagingPriorityHigh, _IO_PAGING_PRIORITY_IoPagingPriorityInvalid, _IRP, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DPFLTR_INFO_LEVEL, FILE_DEVICE_CD_ROM, FILE_FLOPPY_DISKETTE, IO_WARNING_DISK_SURPRISE_REMOVED, IRP_PAGING_IO, SL_KEY_SPECIFIED, STATUS_DEVICE_NOT_READY, STATUS_FT_READ_FROM_COPY, STATUS_INTERNAL_ERROR, STATUS_MORE_PROCESSING_REQUIRED, WMIREG_ACTION_REGISTER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_INIT, TRACE_FLAG_PNP, TRACE_FLAG_RW};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_PRIVATE_DATA, CLASS_TAG_SENSE2, CLASS_TAG_WORKING_SET, FailurePredictionNone, IOCTL_STORAGE_PERSISTENT_RESERVE_IN, IOCTL_STORAGE_PERSISTENT_RESERVE_OUT, SCSI_DISK_DRIVER_INTERNAL};
use ntkrnl_sys::ddk::{_LARGE_INTEGER__bindgen_ty_1 as LARGE_INTEGER__bindgen_ty_1};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::debug::{ClasspInitializeDebugGlobals, SnapDiskStartup};
use crate::types::{BuildDeviceInputMdl, ClassDetermineTokenOperationCommandSupport, ClassForwardIrpSynchronous, ClassGetDescriptor, ClassGetDeviceParameter, ClassGetPdoId, ClassInitializeDispatchTables, ClassPnpQueryFdoRelations, ClassQueryPnpCapabilities, ClassRemoveDevice, ClassScanForSpecial, ClassSetDeviceParameter, ClasspAllocatePowerProcessIrp, ClasspAllocateReleaseQueueIrp, ClasspCalculateLogicalSectorSize, ClasspDeleteTimer, ClasspDisableTimer, ClasspEjectionControl, ClasspEnableIdlePower, ClasspEnableTimer, ClasspEnqueueIdleRequest, ClasspGetCopyOffloadMaxDuration, ClasspGetInquiryVpdSupportInfo, ClasspGetLBProvisioningInfo, ClasspInitializeHotplugInfo, ClasspInitializeIdleTimer, ClasspInitializeTimer, ClasspIsPortable, ClasspRegisterMountedDeviceInterface, ClasspScanForSpecialInRegistry, ClasspValidateOffloadSupported, ClasspZeroQERR, DequeueFreeTransferPacket, EnqueueDeferredClientIrp, EnqueueFreeTransferPacket, FreeDeviceInputMdl, InitLowMemRetry, InitializeStorageRequestBlock, InitializeTransferPackets, SetupDriveCapacityTransferPacket, SetupReadWriteTransferPacket, StepLowMemRetry, SubmitTransferPacket, TransferPacketQueueRetryDpc};
// KeQuerySystemTime: only in ntkrnl_sys::ddk on i386; not used in this file
// resolve_imports.py: explicit kernel/private-type imports
use crate::guid::{GUID_CONSOLE_DISPLAY_STATE};
use crate::types::{AllFdosList, ClassBadItems, ClassGuidQueryRegInfoEx, ClassGuidSenseInfo2, ClassGuidSrbSupport, ClassGuidWorkingSet, MEDIA_LOCK_TYPE_InternalMediaLock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK, _CDB__START_STOP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{min};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_LOCK, TRACE_LEVEL_VERBOSE};
use crate::types::{CLASS_PRIVATE_COMMON_DATA, ClasspBuildRequestEx, REMOVE_COMPLETE};
use ntkrnl_sys::ddk::{ExAcquireRundownProtectionCacheAware, ExDeleteNPagedLookasideList, ExInitializeNPagedLookasideList, ExReleaseRundownProtectionCacheAware, PCSTR, PRTL_GENERIC_TABLE, RtlDeleteElementGenericTable, RtlInsertElementGenericTable};
use crate::lock::{LockHighWatermark, REMOVE_TRACKING_BLOCK, PREMOVE_TRACKING_BLOCK};
use crate::dictlib::POOL_NX_ALLOCATION;
use ntkrnl_sys::inline::common::InterlockedCompareExchange;

extern "C" {
}

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

        TracePrint(TRACE_LEVEL_VERBOSE, TRACE_FLAG_LOCK, 
                    b"ClassAcquireRemoveLock: Acquired for Object %p & irp %p - count is %d\n\0".as_ptr() as PCSTR,
                    DeviceObject, Tag, lockValue);

        NT_ASSERTMSG!(b"ClassAcquireRemoveLock - lock value was negative : \0".as_ptr() as PCSTR,
                     (lockValue > 0));

        NT_ASSERTMSG!(b"RemoveLock increased to meet LockHighWatermark\0".as_ptr() as PCSTR,
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

            KeQueryTickCount(&mut trackingBlock.TimeLocked.QuadPart);

            KeAcquireSpinLock((&raw mut (*commonExtension).RemoveTrackingSpinlock).cast(), &mut oldIrql);

            removeTrackingList = ((*commonExtension).RemoveTrackingList).cast();

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
                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_LOCK, 
                                b">>>>>ClassAcquireRemoveLock: already tracking Tag %p\n\0".as_ptr() as PCSTR,
                                Tag);
                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_LOCK, 
                                b">>>>>ClassAcquireRemoveLock: acquired in file %s on line %d\n\0".as_ptr() as PCSTR,
                                (*insertedTrackingBlock).File, (*insertedTrackingBlock).Line);
                }
            } else {
                (*commonExtension).RemoveTrackingUntrackedCount += 1;

                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_LOCK, 
                            b">>>>>ClassAcquireRemoveLock: Cannot track Tag %p - currently %d untracked requsts\n\0".as_ptr() as PCSTR,
                            Tag, (*commonExtension).RemoveTrackingUntrackedCount);
            }

            KeReleaseSpinLock((&raw mut (*commonExtension).RemoveTrackingSpinlock).cast(), oldIrql);
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
    {
        // Debug tripwire: a bogus rundown area means we're about to AV inside
        // ExAcquireRundownProtectionCacheAware with no context — identify the
        // device object and pointers first.
        unsafe extern "C" { fn DbgPrint(Format: *const i8, ...) -> u32; }
        let pcd = (*commonExtension).PrivateCommonData;
        if pcd.is_null() {
            DbgPrint(
                c"[CLASSPNP] AcquireRemoveLock BAD: dev=%p NULL PrivateCommonData\n".as_ptr(),
                DeviceObject,
            );
        } else if (*removeLockRundown).Number == 0
            || (*removeLockRundown).RunRefSize == 0
            || (*removeLockRundown).RunRefs.is_null()
        {
            DbgPrint(
                c"[CLASSPNP] AcquireRemoveLock BAD rundown: dev=%p pcd=%p rr=%p RunRefs=%p size=%u num=%u\n".as_ptr(),
                DeviceObject,
                pcd,
                removeLockRundown,
                (*removeLockRundown).RunRefs,
                (*removeLockRundown).RunRefSize,
                (*removeLockRundown).Number,
            );
        }
    }
    rundownAcquired = ExAcquireRundownProtectionCacheAware(removeLockRundown);
    if rundownAcquired == 0 {
        InterlockedIncrement(&mut (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire as *mut LONG as *mut i32);
        TracePrint(TRACE_LEVEL_VERBOSE,
                    TRACE_FLAG_LOCK,
                    b"ClassAcquireRemoveLockEx: RemoveLockRundown acquisition failed RemoveLockFailAcquire = %d\n\0".as_ptr() as PCSTR,
                    (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire);
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

        KeAcquireSpinLock((&raw mut (*commonExtension).RemoveTrackingSpinlock).cast(),
                          &mut oldIrql);

        removeTrackingList = ((*commonExtension).RemoveTrackingList).cast();

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

        KeReleaseSpinLock((&raw mut (*commonExtension).RemoveTrackingSpinlock).cast(),
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
        oldValue = InterlockedCompareExchange(
            &mut (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire as *mut LONG as *mut i32,
            lockValue - 1,
            lockValue,
        ) as LONG;

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
    if (*CommonExtension).__bindgen_anon_1.IsSrbLookasideListInitialized() != 0 {
        (*CommonExtension).__bindgen_anon_1.set_IsSrbLookasideListInitialized(0);
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
    if (*CommonExtension).__bindgen_anon_1.IsSrbLookasideListInitialized() == 0 {
        if (*CommonExtension).__bindgen_anon_1.IsFdo() != 0 {
            let fdo = CommonExtension as PFUNCTIONAL_DEVICE_EXTENSION;
            let adapter_desc = (*fdo).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR;
            if (*adapter_desc).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
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
            sizeNeeded as ntkrnl_sys::ddk::SIZE_T,
            u32::from_ne_bytes(*b"$scS"),
            NumberElements as USHORT,
        );
        (*CommonExtension).__bindgen_anon_1.set_IsSrbLookasideListInitialized(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassBuildRequest(Fdo: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    // obsolete, still called by CDROM.SYS
    // ClasspAllocateSrb macro expansion: ExAllocateFromNPagedLookasideList(&ext->CommonExtension.SrbLookasideList)
    // CommonExtension is the first field of FUNCTIONAL_DEVICE_EXTENSION (at offset 0 via __bindgen_anon_1)
    let srb = ntkrnl_sys::inline::ex::ExAllocateFromNPagedLookasideList(
        &raw mut (*(fdoExtension as PCOMMON_DEVICE_EXTENSION)).SrbLookasideList,
    ) as PSCSI_REQUEST_BLOCK;
    if srb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    ClasspBuildRequestEx(fdoExtension, Irp, srb);
    STATUS_SUCCESS
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn KeQueryTickCount(TickCount: *mut i64);
}

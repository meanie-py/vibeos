//! class_part_c.rs — translated class.c/obsolete.c gap functions (rustconv 2026-06-10)
#![allow(non_snake_case, unused_imports, unused_variables, dead_code, unused_mut)]
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
use crate::types::{CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED, CLASSP_REG_DISBALE_IDLE_POWER_NAME, CLASSP_REG_PERF_RESTORE_VALUE_NAME, CLASSP_REG_QERR_OVERRIDE_MODE, CLASSP_REG_REMOVAL_POLICY_VALUE_NAME, CLASSP_REG_SUBKEY_NAME, CLASSP_VOLUME_VERIFY_CHECKED, CLASS_DRIVER_EXTENSION, CLASS_INIT_DATA, CLASS_INTERPRET_SENSE_INFO2, CLASS_INTERPRET_SENSE_INFO2_MAXIMUM_HISTORY_COUNT, CLASS_MAX_INTERLEAVE_PER_CRITICAL_IO, CLASS_PERF_RESTORE_MINIMUM, CLASS_QUERY_WMI_REGINFO_EX_LIST, CLASS_SRB_SCSI_REQUEST_BLOCK, CLASS_SRB_STORAGE_REQUEST_BLOCK, CLASS_WORKING_SET, CLASS_WORKING_SET_MAXIMUM, COMMON_DEVICE_EXTENSION, COMPLETION_CONTEXT, ClassGlobalDispatch, ClassMinimalPowerHandler, ClasspPowerSettingCallback, ClasspScanForClassHacks, DEV_USE_16BYTE_CDB, FDO_HACK_NO_SYNC_CACHE, FUNCTIONAL_DEVICE_EXTENSION, MAX_TOKEN_LIST_IDENTIFIERS, NUM_IO_RETRIES, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PREAD_CAPACITY_DATA_EX, PSCSI_REQUEST_BLOCK, PTRANSFER_PACKET, QERR_SET_ZERO_ALWAYS, QERR_SET_ZERO_ODX_OR_TP_ONLY, READ_CAPACITY_DATA, READ_CAPACITY_DATA_EX, REG_DISK_CLASS_CONTROL, REMOVE_PENDING, SCSIOP_START_STOP_UNIT, SCSI_REQUEST_BLOCK, SRB_FLAGS_DISABLE_AUTOSENSE, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_FREE_SENSE_BUFFER, SRB_FLAGS_NO_DATA_TRANSFER, SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_STATUS_QUEUE_FROZEN, SRB_TYPE_STORAGE_REQUEST_BLOCK, START_UNIT_TIMEOUT, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK, TRANSFER_PACKET, _CLASS_DEV_INFO, _CLASS_DRIVER_EXTENSION, _CLASS_FUNCTION_SUPPORT_INFO, _CLASS_INIT_DATA, _CLASS_PRIVATE_FDO_DATA, _COMMON_DEVICE_EXTENSION, _COMPLETION_CONTEXT, _FUNCTIONAL_DEVICE_EXTENSION, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK};
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
use ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE as BUS_QUERY_ID_TYPE;
use crate::ddk_shims::{ULongToPtr, LongToPtr, MmGetMdlVirtualAddress, Int64ShrlMod32, GET_FDO_EXTENSON_SENSE_DATA_LENGTH, ASSERT_PDO};
use crate::srblib::SrbSetQueueSortKey;

use crate::shims::{ClassCompleteRequest, ClassReleaseRemoveLock, ClassAcquireRemoveLock};
use crate::types::*;
use crate::glue::*;
use crate::debug::*;
use crate::guid::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::debug::{ClasspInitializeDebugGlobals, SnapDiskStartup};
use crate::types::{BuildDeviceInputMdl, ClassGetDeviceParameter, ClassInitializeDispatchTables, ClassQueryPnpCapabilities, ClassScanForSpecial, ClassSetDeviceParameter, ClasspAllocatePowerProcessIrp, ClasspAllocateReleaseQueueIrp, ClasspCalculateLogicalSectorSize, ClasspDeleteTimer, ClasspDisableTimer, ClasspEjectionControl, ClasspEnableIdlePower, ClasspEnableTimer, ClasspEnqueueIdleRequest, ClasspGetCopyOffloadMaxDuration, ClasspInitializeHotplugInfo, ClasspInitializeIdleTimer, ClasspInitializeTimer, ClasspIsPortable, ClasspScanForSpecialInRegistry, ClasspValidateOffloadSupported, ClasspZeroQERR, DequeueFreeTransferPacket, EnqueueDeferredClientIrp, EnqueueFreeTransferPacket, FreeDeviceInputMdl, InitLowMemRetry, InitializeStorageRequestBlock, InitializeTransferPackets, SetupDriveCapacityTransferPacket, SetupReadWriteTransferPacket, StepLowMemRetry, SubmitTransferPacket, TransferPacketQueueRetryDpc};
// KeQuerySystemTime: only in ntkrnl_sys::ddk on i386; not used in this file
// resolve_imports.py: explicit kernel/private-type imports
use crate::guid::{GUID_CONSOLE_DISPLAY_STATE};
use crate::types::{AllFdosList, ClassBadItems, ClassGuidQueryRegInfoEx, ClassGuidSenseInfo2, ClassGuidSrbSupport, ClassGuidWorkingSet, MEDIA_LOCK_TYPE_InternalMediaLock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK, _CDB__START_STOP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{min};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE, IOCTL_SCSI_GET_ADDRESS, IO_WRN_FAILURE_PREDICTED, KEY_WRITE, NonPagedPoolNxCacheAligned, RTL_QUERY_REGISTRY_TYPECHECK, RTL_QUERY_REGISTRY_TYPECHECK_SHIFT, SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED, SCSI_SENSE_OPTIONS_NONE, SENSE_BUFFER_SIZE_EX, STATUS_IO_DEVICE_ERROR, STATUS_IO_TIMEOUT, STATUS_NONEXISTENT_SECTOR, TRACE_FLAG_GENERAL, TRACE_FLAG_IOCTL, TRACE_FLAG_MCN, TRACE_FLAG_WMI, TRACE_LEVEL_VERBOSE, VPB_MOUNTED, _MEDIA_LOCK_TYPE_SecureMediaLock as SecureMediaLock, max};
use crate::guid::{GUID_IO_DEVICE_BECOMING_READY};
use crate::srblib::{SrbGetDataTransferLength, SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength, SrbGetSrbFlags, SrbGetSystemStatus, SrbSetScsiStatus};
use crate::types::{CLASSP_REG_WRITE_CACHE_VALUE_NAME, CLASS_ERROR_LOG_DATA, CLASS_PRIVATE_COMMON_DATA, CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK, ClassCheckVerifyComplete, ClassCleanupMediaChangeDetection, ClassDeviceGetLBProvisioningResources, ClassDeviceProcessOffloadRead, ClassDeviceProcessOffloadWrite, ClassIoComplete, ClassNotifyFailurePredicted, ClassQueueCapacityChangedEventWorker, ClassQueueProvisioningTypeChangedEventWorker, ClassQueueResourceExhaustionEventWorker, ClassQueueThresholdEventWorker, ClassReadDriveCapacity, ClassSendEjectionNotification, ClassSendNotification, ClassSendStartUnit, ClassSetMediaChangeState, ClasspAccessAlignmentProperty, ClasspConvertToScsiRequestBlock, ClasspDeviceCopyOffloadProperty, ClasspDeviceGetLBAStatus, ClasspDeviceLBProvisioningProperty, ClasspDeviceMediaTypeProperty, ClasspDeviceSeekPenaltyProperty, ClasspDeviceTrimProcess, ClasspDeviceTrimProperty, ClasspDuidQueryProperty, ClasspInitializeRemoveTracking, ClasspMcnControl, ClasspPerfIncrementErrorCount, ClasspPersistentReserve, ClasspPriorityHint, ClasspQueueLogIOEventWithContextWorker, ClasspStorageEventNotification, ClasspUninitializeRemoveTracking, ClasspWriteCacheProperty, DEV_SAFE_START_UNIT, DestroyAllTransferPackets, FDO_HACK_NO_RESERVE6, IOCTL_SCSI_EXECUTE_NONE, IOCTL_STORAGE_BASE, IdlePowerFDOList, IdlePowerFDOListMutex, InitializeDictionary, MAXIMUM_RETRIES, MODE_DISCONNECT_PAGE, MODE_PARAMETER_HEADER, MODE_PARAMETER_HEADER10, MODE_SENSE_CURRENT_VALUES, _MEDIA_CHANGE_DETECTION_STATE_MediaPresent as MediaPresent, NOT_READY_RETRY_INTERVAL, NO_REMOVE, NUM_ERROR_LOG_ENTRIES, PCDB, PCLASS_DEV_INFO, PCLASS_DRIVER_EXTENSION, PCLASS_ERROR_LOG_DATA, PCLASS_PRIVATE_COMMON_DATA, PFIXED_SENSE_DATA, PHYSICAL_DEVICE_EXTENSION, PIDLE_POWER_FDO_LIST_ENTRY, PINQUIRYDATA, PMODE_DISCONNECT_PAGE, PMODE_PARAMETER_HEADER, PMODE_PARAMETER_HEADER10, PPHYSICAL_DEVICE_EXTENSION, PSCSI_SENSE_DESCRIPTOR_BLOCK_COMMAND, PSCSI_SENSE_DESCRIPTOR_INFORMATION, PSTORAGE_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK_HEADER, SCSIOP_INQUIRY, SCSIOP_MODE_SENSE, SCSIOP_MODE_SENSE10, SCSIOP_READ, SCSIOP_READ12, SCSIOP_READ16, SCSIOP_READ6, SCSIOP_RELEASE_UNIT, SCSIOP_RELEASE_UNIT10, SCSIOP_RESERVE_UNIT, SCSIOP_RESERVE_UNIT10, SCSIOP_TEST_UNIT_READY, SCSIOP_VERIFY, SCSIOP_VERIFY16, SCSIOP_WRITE, SCSIOP_WRITE12, SCSIOP_WRITE16, SCSIOP_WRITE6, SCSISTAT_BUSY, SCSISTAT_GOOD, SCSISTAT_RESERVATION_CONFLICT, SCSI_ADDRESS, SCSI_ADSENSE_DATA_TRANSFER_ERROR, SCSI_ADSENSE_INVALID_MEDIA, SCSI_ADSENSE_LOGICAL_UNIT_ERROR, SCSI_ADSENSE_LUN_COMMUNICATION, SCSI_ADSENSE_LUN_NOT_READY, SCSI_ADSENSE_REC_DATA_ECC, SCSI_ADSENSE_REC_DATA_NOECC, SCSI_ADSENSE_SEEK_ERROR, SCSI_ADSENSE_TRACK_ERROR, SCSI_ADSENSE_WRITE_PROTECT, SCSI_PASS_THROUGH, SCSI_SENSEQ_CAPACITY_DATA_CHANGED, SCSI_SENSEQ_INITIATOR_RESPONSE_TIMEOUT, SCSI_SENSEQ_INIT_COMMAND_REQUIRED, SCSI_SENSEQ_INQUIRY_DATA_CHANGED, SCSI_SENSEQ_MICROCODE_CHANGED, SCSI_SENSEQ_OPERATING_DEFINITION_CHANGED, SCSI_SENSEQ_TIMEOUT_ON_LOGICAL_UNIT, SCSI_SENSE_DESCRIPTOR_TYPE_BLOCK_COMMAND, SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION, SCSI_SESNEQ_COMM_CRC_ERROR, SENSE_DATA, SRB_CLASS_FLAGS_FREE_MDL, SRB_CLASS_FLAGS_LOW_PRIORITY, SRB_CLASS_FLAGS_PAGING, SRB_FLAGS_CLASS_DRIVER_RESERVED, SRB_FLAGS_DATA_IN, SRB_FLAGS_DATA_OUT, SRB_FLAGS_DONT_START_NEXT_PACKET, SRB_FLAGS_NO_QUEUE_FREEZE, SRB_FLAGS_QUEUE_ACTION_ENABLE, SRB_FUNCTION_CLAIM_DEVICE, SRB_FUNCTION_RELEASE_DEVICE, SRB_HEAD_OF_QUEUE_TAG_REQUEST, SRB_ORDERED_QUEUE_TAG_REQUEST, SRB_SIMPLE_TAG_REQUEST, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_COMMAND_TIMEOUT, SRB_STATUS_INTERNAL_ERROR, SRB_STATUS_INVALID_LUN, SRB_STATUS_INVALID_PATH_ID, SRB_STATUS_INVALID_TARGET_ID, SRB_STATUS_NO_DEVICE, SRB_STATUS_NO_HBA, SRB_STATUS_PARITY_ERROR, SRB_STATUS_SELECTION_TIMEOUT, SRB_STATUS_SUCCESS, SRB_STATUS_TIMEOUT, SRB_STATUS_UNEXPECTED_BUS_FREE, SRB_TYPE_SCSI_REQUEST_BLOCK, _MEDIA_CHANGE_DETECTION_STATE_MediaNotPresent as MediaNotPresent, _MEDIA_CHANGE_DETECTION_STATE_MediaUnavailable as MediaUnavailable};
use ntkrnl_sys::ddk::{CCHAR, DEVICE_OBJECT, IO_STATUS_BLOCK, KPRIORITY, KPROCESSOR_MODE, OBJECT_ATTRIBUTES, PCSTR, PETHREAD, _EVENT_TYPE_NotificationEvent as NotificationEvent, _EVENT_TYPE_SynchronizationEvent as SynchronizationEvent, _KWAIT_REASON_Executive as Executive};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DEVICE_EVENT_BECOMING_READY, MOUNTDEV_SUGGESTED_LINK_NAME, MOUNTDEV_UNIQUE_ID, PMOUNTDEV_SUGGESTED_LINK_NAME, PMOUNTDEV_UNIQUE_ID};
// SimpleMediaLock
use crate::types::MEDIA_LOCK_TYPE_SimpleMediaLock as SimpleMediaLock;
// Drive letter helpers from class.rs
use crate::class::{ClassInitialize, FirstDriveLetter, LastDriveLetter};
// ClasspnpGlobals for debug builds
use crate::debug::ClasspnpGlobals;
// SCSI_PASS_THROUGH32 and IoIs32bitProcess only exist on 64-bit (WoW64 check)
#[cfg(target_pointer_width = "64")]
use crate::types::{SCSI_PASS_THROUGH32};

extern "C" {
    fn DbgPrintEx(ComponentId: ULONG, Level: ULONG, Format: PCSTR, ...) -> NTSTATUS;
    
}
unsafe extern "system" {
    fn PoUnregisterPowerSettingCallback(Handle: PVOID) -> NTSTATUS;
}

// gap-function constants (from classpnp C headers)
const SCSI_REQUEST_BLOCK_SIZE: u32 = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32;
const MINIMUM_RETRY_UNITS: i64 = 32;
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000Eu32 as i32;
const IOCTL_DISK_UPDATE_PROPERTIES: u32 = 0x00070140;
#[allow(non_upper_case_globals)]
static MOUNTDEV_MOUNTED_DEVICE_GUID: GUID = GUID {
    Data1: 0x53F5630D, Data2: 0xB6BF, Data3: 0x11D0,
    Data4: [0x94,0xF2,0x00,0xA0,0xC9,0x1E,0xFB,0x8B],
};


#[no_mangle]
pub unsafe extern "C" fn ClasspModeSelect(
    Fdo: PDEVICE_OBJECT,
    ModeSelectBuffer: PCHAR,
    Length: ULONG,
    SavePages: BOOLEAN
) -> NTSTATUS {

    #[repr(C)]
    struct TRANSFER_PACKET {
        _opaque: [u8; 0],
    }

    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;

    let senseBufferMdl = BuildDeviceInputMdl((ModeSelectBuffer).cast(), Length);
    if !senseBufferMdl.is_null() {
        let pkt = DequeueFreeTransferPacket(Fdo, 1u8);
        if !pkt.is_null() {
            let mut event: KEVENT = core::mem::zeroed();
            let mut pseudoIrp: IRP = core::mem::zeroed();

            pseudoIrp.Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = LongToPtr(1);
            pseudoIrp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            pseudoIrp.IoStatus.Information = 0;
            pseudoIrp.MdlAddress = senseBufferMdl;

            KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0);
            SetupModeSelectTransferPacket(
                pkt,
                &mut event,
                (ModeSelectBuffer).cast(),
                Length as UCHAR,
                SavePages,
                &mut pseudoIrp
            );
            SubmitTransferPacket(pkt);
            KeWaitForSingleObject(
                &mut event as *mut _ as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0,
                core::ptr::null_mut()
            );

            if !NT_SUCCESS(pseudoIrp.IoStatus.__bindgen_anon_1.Status) {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_GENERAL,
                    b"ClassModeSelect on Fdo %ph failed with status %xh.\0".as_ptr() as PCSTR,
                    Fdo,
                    pseudoIrp.IoStatus.__bindgen_anon_1.Status
                );
            }

            status = pseudoIrp.IoStatus.__bindgen_anon_1.Status;
        }

        FreeDeviceInputMdl(senseBufferMdl);
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}

// --- ClasspModeSense ---
#[no_mangle]
pub unsafe extern "C" fn ClasspModeSense(
    Fdo: PDEVICE_OBJECT,
    ModeSenseBuffer: PCHAR,
    Length: ULONG,
    PageMode: UCHAR,
    PageControl: UCHAR
) -> ULONG {
    let mut lengthTransferred: ULONG = 0;
    let senseBufferMdl: PMDL;

    senseBufferMdl = BuildDeviceInputMdl((ModeSenseBuffer).cast(), Length);
    if !senseBufferMdl.is_null() {
        let pkt = DequeueFreeTransferPacket(Fdo, 1u8);
        if !pkt.is_null() {
            let mut event: KEVENT = core::mem::zeroed();
            let mut pseudoIrp: IRP = core::mem::zeroed();

            pseudoIrp.Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = LongToPtr(1);
            pseudoIrp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            pseudoIrp.IoStatus.Information = 0;
            pseudoIrp.MdlAddress = senseBufferMdl;

            NT_ASSERT!(Length <= 0x0ff);
            KeInitializeEvent(
                core::ptr::addr_of_mut!(event),
                _EVENT_TYPE_SynchronizationEvent,
                (0i32) as u8
            );
            SetupModeSenseTransferPacket(
                pkt,
                core::ptr::addr_of_mut!(event),
                (ModeSenseBuffer).cast(),
                Length as UCHAR,
                PageMode,
                0,
                core::ptr::addr_of_mut!(pseudoIrp),
                PageControl
            );
            SubmitTransferPacket(pkt);
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(event) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0u8,
                core::ptr::null_mut()
            );

            if NT_SUCCESS(pseudoIrp.IoStatus.__bindgen_anon_1.Status) {
                lengthTransferred = pseudoIrp.IoStatus.Information as ULONG;
            } else {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_GENERAL,
                    b"ClasspModeSense on Fdo %ph failed with status %xh.\0".as_ptr() as PCHAR,
                    Fdo,
                    pseudoIrp.IoStatus.__bindgen_anon_1.Status
                );
            }
        }

        FreeDeviceInputMdl(senseBufferMdl);
    }

    lengthTransferred
}

// --- ClassAddChild ---
#[no_mangle]
pub unsafe extern "C" fn ClassAddChild(
    Parent: PFUNCTIONAL_DEVICE_EXTENSION,
    Child: PPHYSICAL_DEVICE_EXTENSION,
    AcquireLock: BOOLEAN
) {
    
    if AcquireLock != 0 {
        ClassAcquireChildLock(Parent);
    }

    #[cfg(debug_assertions)]
    {
        let mut testChild = (*(Parent as PCOMMON_DEVICE_EXTENSION)).ChildList;
        while !testChild.is_null() {
            debug_assert!(testChild != Child);
            testChild = (*(testChild as PCOMMON_DEVICE_EXTENSION)).ChildList;
        }
    }

    (*(Child as PCOMMON_DEVICE_EXTENSION)).ChildList = (*(Parent as PCOMMON_DEVICE_EXTENSION)).ChildList;
    (*(Parent as PCOMMON_DEVICE_EXTENSION)).ChildList = Child;

    if AcquireLock != 0 {
        ClassReleaseChildLock(Parent);
    }
}

// --- ClassRemoveChild ---
#[no_mangle]
pub unsafe extern "system" fn ClassRemoveChild(
    Parent: PFUNCTIONAL_DEVICE_EXTENSION,
    Child: PPHYSICAL_DEVICE_EXTENSION,
    AcquireLock: BOOLEAN,
) -> PPHYSICAL_DEVICE_EXTENSION {
    let mut Child = Child;
    
    if AcquireLock != 0 {
        ClassAcquireChildLock(Parent);
    }

    'try_block: {
        let mut previousChild: PCOMMON_DEVICE_EXTENSION = (Parent) as PCOMMON_DEVICE_EXTENSION;

        if (*(Parent as PCOMMON_DEVICE_EXTENSION)).ChildList.is_null() {
            Child = core::ptr::null_mut();
            break 'try_block;
        }

        if !Child.is_null() {
            loop {
                NT_ASSERT!(previousChild != (Child) as PCOMMON_DEVICE_EXTENSION as *mut _);

                if (*previousChild).ChildList == Child {
                    break;
                }

                previousChild = ((*previousChild).ChildList) as PCOMMON_DEVICE_EXTENSION;
                
                if previousChild.is_null() {
                    break;
                }
            }

            if previousChild.is_null() {
                Child = core::ptr::null_mut();
                break 'try_block;
            }
        }

        Child = (*previousChild).ChildList;
        (*previousChild).ChildList = (*(Child as PCOMMON_DEVICE_EXTENSION)).ChildList;
        (*(Child as PCOMMON_DEVICE_EXTENSION)).ChildList = core::ptr::null_mut();
    }

    if AcquireLock != 0 {
        ClassReleaseChildLock(Parent);
    }
    
    Child
}

// --- ClassPnpQueryFdoRelations ---
#[no_mangle]
pub unsafe extern "C" fn ClassPnpQueryFdoRelations(
    Fdo: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let driverExtension = IoGetDriverObjectExtension(
        (*Fdo).DriverObject,
        (ClassInitialize as PVOID) as PVOID
    ) as PCLASS_DRIVER_EXTENSION;

    if InterlockedIncrement(core::ptr::addr_of_mut!((*fdoExtension).EnumerationInterlock) as *mut LONG) == 1 {
        ((*driverExtension).InitData.ClassEnumerateDevice.unwrap())(Fdo);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = ClassRetrieveDeviceRelations(
        Fdo,
        BusRelations,
        core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as *mut PDEVICE_RELATIONS
    );
    
    InterlockedDecrement(core::ptr::addr_of_mut!((*fdoExtension).EnumerationInterlock) as *mut LONG);

    (*Irp).IoStatus.__bindgen_anon_1.Status
}

// --- ClassGetPdoId ---
#[no_mangle]
pub unsafe extern "C" fn ClassGetPdoId(
    Pdo: PDEVICE_OBJECT,
    IdType: BUS_QUERY_ID_TYPE,
    IdString: PUNICODE_STRING
) -> NTSTATUS {

    let driverExtension = IoGetDriverObjectExtension(
        (*Pdo).DriverObject,
        (ClassInitialize as PVOID) as *mut core::ffi::c_void
    ) as PCLASS_DRIVER_EXTENSION;

    ASSERT_PDO((Pdo).cast());
    NT_ASSERT!((*driverExtension).InitData.ClassQueryId.is_some());

    (*driverExtension).InitData.ClassQueryId.unwrap()(Pdo, IdType, IdString)
}

// --- ClassRetryRequest ---
#[no_mangle]
pub unsafe extern "C" fn ClassRetryRequest(
    SelfDeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    TimeDelta100ns: i64
) {
    let fdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA;
    let retryInfo: *mut CLASS_RETRY_INFO;
    let mut delta: i64;
    let mut irql: KIRQL = 0;
    let mut TimeDelta100ns = TimeDelta100ns;

    NT_ASSERT!(((core::mem::size_of::<CLASS_RETRY_INFO>() <= (4 * core::mem::size_of::<*mut core::ffi::c_void>())) as u32) as u8);

    fdoExtension = (*SelfDeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;

    if (*(fdoExtension as PCOMMON_DEVICE_EXTENSION)).__bindgen_anon_1.IsFdo() == 0 {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, (b"ClassRetryRequestEx: LOST IRP %p\n\0".as_ptr()).cast(), Irp);
        NT_ASSERT!(0);
        return;
    }

    fdoData = (*fdoExtension).PrivateFdoData;

    if TimeDelta100ns < 0 {
        NT_ASSERT!(0);
        TimeDelta100ns *= -1;
    }

    IoMarkIrpPending(Irp);

    retryInfo = core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext) as *mut CLASS_RETRY_INFO;
    core::ptr::write_bytes(retryInfo, 0, 1);

    delta = TimeDelta100ns / ((*fdoData).Retry.Granularity) as i64;
    if TimeDelta100ns % ((*fdoData).Retry.Granularity) as i64 != 0 {
        delta += 1;
    }
    if delta == 0 {
        delta = MINIMUM_RETRY_UNITS;
    }

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).Retry.Lock)).cast(), core::ptr::addr_of_mut!(irql));

    (*retryInfo).Next = (*fdoData).Retry.ListHead;
    (*fdoData).Retry.ListHead = retryInfo;

    if (*fdoData).Retry.Delta.QuadPart == 0 {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, (b"ClassRetry: +++ %p\n\0".as_ptr()).cast(), Irp);

        NT_ASSERT!((((*fdoData).Retry.ListHead != core::ptr::null_mut()) as u32) as u8);
        NT_ASSERT!((((*(*fdoData).Retry.ListHead).Next == core::ptr::null_mut()) as u32) as u8);

        KeQueryTickCount((core::ptr::addr_of_mut!((*fdoData).Retry.Tick)).cast());
        (*fdoData).Retry.Tick.QuadPart += delta;
        (*fdoData).Retry.Delta.QuadPart = delta;
        ClasspRetryDpcTimer(fdoData);

    } else if delta > (*fdoData).Retry.Delta.QuadPart {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, (b"ClassRetry:  ++ %p\n\0".as_ptr()).cast(), Irp);

        NT_ASSERT!((((*fdoData).Retry.ListHead != core::ptr::null_mut()) as u32) as u8);
        NT_ASSERT!((((*(*fdoData).Retry.ListHead).Next != core::ptr::null_mut()) as u32) as u8);

        (*fdoData).Retry.Tick.QuadPart -= (*fdoData).Retry.Delta.QuadPart;
        (*fdoData).Retry.Tick.QuadPart += delta;

        (*fdoData).Retry.Delta.QuadPart = delta;

    } else {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, (b"ClassRetry:  ++ %p\n\0".as_ptr()).cast(), Irp);
    }

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).Retry.Lock)).cast(), irql);
}

// --- ClasspReleaseQueue ---
#[no_mangle]
pub unsafe extern "C" fn ClasspReleaseQueue(
    Fdo: PDEVICE_OBJECT,
    ReleaseQueueIrp: PIRP
) {
    let mut irpStack: PIO_STACK_LOCATION;
    let mut irp: PIRP;
    let fdoExtension = Fdo as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let lowerDevice: PDEVICE_OBJECT;
    let mut srb: PSTORAGE_REQUEST_BLOCK_HEADER;
    let mut currentIrql: KIRQL = 0;
    let function: ULONG;

    lowerDevice = (*(fdoExtension as PCOMMON_DEVICE_EXTENSION)).LowerDeviceObject;

    KeRaiseIrql(DISPATCH_LEVEL, &mut currentIrql);

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock)).cast());

    NT_ASSERT!(ReleaseQueueIrp.is_null() || 
           ReleaseQueueIrp == (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrp);

    NT_ASSERT!((*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated != 0);
    if (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated == 0 {
        ClasspAllocateReleaseQueueIrp(fdoExtension);
    }
    if (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated == 0 {
        KeBugCheckEx(0x52, 0x12, Fdo as ULONG_PTR, 0x0, 0x0);
    }

    if ((*fdoExtension).ReleaseQueueInProgress != 0) && ReleaseQueueIrp.is_null() {

        (*fdoExtension).ReleaseQueueNeeded = 1u8;
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock)).cast());
        KeLowerIrql(currentIrql);
        return;

    }

    (*fdoExtension).ReleaseQueueInProgress = 1u8;
    if !ReleaseQueueIrp.is_null() {
        irp = ReleaseQueueIrp;
    } else {
        irp = (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrp;
    }

    if (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        srb = core::ptr::addr_of_mut!((*(*fdoExtension).PrivateFdoData).ReleaseQueueSrb.SrbEx) as PSTORAGE_REQUEST_BLOCK_HEADER;
    } else {
        srb = core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSrb) as PSTORAGE_REQUEST_BLOCK_HEADER;
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock)).cast());

    NT_ASSERT!(!irp.is_null());

    irpStack = IoGetNextIrpStackLocation(irp);

    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;

    SrbSetOriginalRequest((srb).cast(), irp);

    (*irpStack).Parameters.Scsi.Srb = srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;

    if TEST_FLAG((*Fdo).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
       function = SRB_FUNCTION_FLUSH_QUEUE;
    } else {
       function = SRB_FUNCTION_RELEASE_QUEUE;
    }

    if (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        (*(srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = function;
    } else {
        (*srb).Function = function as UCHAR;
    }

    ClassAcquireRemoveLock(Fdo, irp);

    IoSetCompletionRoutine(irp,
                           Some(ClassReleaseQueueCompletion),
                           Fdo as PVOID,
                           1u8,
                           1u8,
                           1u8);

    IoCallDriver(lowerDevice, irp);

    KeLowerIrql(currentIrql);

    return;
}

// --- ClasspAllocateReleaseRequest ---
#[no_mangle]
pub unsafe extern "C" fn ClasspAllocateReleaseRequest(
    Fdo: PDEVICE_OBJECT
) -> NTSTATUS {
    let fdoExtension = Fdo as PFUNCTIONAL_DEVICE_EXTENSION;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*fdoExtension).ReleaseQueueSpinLock));

    (*fdoExtension).ReleaseQueueNeeded = 0u8;
    (*fdoExtension).ReleaseQueueInProgress = 0u8;
    (*fdoExtension).ReleaseQueueIrpFromPool = 0u8;

    (*fdoExtension).ReleaseQueueIrp = core::ptr::null_mut();

    (*fdoExtension).ReleaseQueueSrb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;

    STATUS_SUCCESS
}

// --- ClasspFreeReleaseRequest ---
#[no_mangle]
pub unsafe extern "C" fn ClasspFreeReleaseRequest(
    Fdo: PDEVICE_OBJECT
) {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    NT_ASSERT!(if (*(fdoExtension as PCOMMON_DEVICE_EXTENSION)).IsRemoved != NO_REMOVE { 1 } else { 0 });

    if !(*fdoExtension).ReleaseQueueIrp.is_null() {
        if (*fdoExtension).ReleaseQueueIrpFromPool != 0 {
            FREE_POOL((*fdoExtension).ReleaseQueueIrp as PVOID);
        } else {
            IoFreeIrp((*fdoExtension).ReleaseQueueIrp);
        }
        (*fdoExtension).ReleaseQueueIrp = core::ptr::null_mut();
    }

    if !(*fdoExtension).PrivateFdoData.is_null() &&
       (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated != 0 {

        FREE_POOL((*(*fdoExtension).PrivateFdoData).ReleaseQueueIrp as PVOID);
        (*(*fdoExtension).PrivateFdoData).ReleaseQueueIrpAllocated = 0;
    }
}

// --- ClasspGetInquiryVpdSupportInfo ---
#[no_mangle]
pub unsafe extern "C" fn ClasspGetInquiryVpdSupportInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION
) {
    let mut status: NTSTATUS;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut cdb: PCDB;
    let mut supportedPages: PVPD_SUPPORTED_PAGES_PAGE = core::ptr::null_mut();
    let bufferLength: UCHAR = (VPD_MAX_BUFFER_SIZE) as u8;
    let mut allocationBufferLength: ULONG = bufferLength as ULONG;
    let mut srbExBuffer: [UCHAR; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE] = [0; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE];
    let mut srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER;

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        allocationBufferLength = ALIGN_UP_BY(allocationBufferLength as _, KeGetRecommendedSharedDataAlignment() as _) as _;
        supportedPages = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            allocationBufferLength as SIZE_T,
            0x33436353
        ) as PVPD_SUPPORTED_PAGES_PAGE;
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        supportedPages = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            bufferLength as SIZE_T,
            0x33436353
        ) as PVPD_SUPPORTED_PAGES_PAGE;
    }

    if supportedPages.is_null() {
        return;
    }

    RtlZeroMemory(supportedPages as PVOID, allocationBufferLength as SIZE_T);

    if (*((*FdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        status = InitializeStorageRequestBlock(
            srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK,
            (STORAGE_ADDRESS_TYPE_BTL8) as u16,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
            1,
            SrbExDataTypeScsiCdb16
        );

        if !NT_SUCCESS(status) {
            NT_ASSERT!(0);
            srb.Length = SCSI_REQUEST_BLOCK_SIZE as USHORT;
            srb.Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
            srbHeader = core::ptr::addr_of_mut!(srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
        } else {
            (*(srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
            srbHeader = srbExBuffer.as_mut_ptr() as PSTORAGE_REQUEST_BLOCK_HEADER;
        }
    } else {
        srb.Length = SCSI_REQUEST_BLOCK_SIZE as USHORT;
        srb.Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
        srbHeader = core::ptr::addr_of_mut!(srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
    }

    SrbSetTimeOutValue((srbHeader).cast(), (*FdoExtension).TimeOutValue);
    SrbSetRequestTag((srbHeader).cast(), SP_UNTAGGED);
    SrbSetRequestAttribute((srbHeader).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
    SrbAssignSrbFlags((srbHeader).cast(), (*FdoExtension).SrbFlags);
    SrbSetCdbLength((srbHeader).cast(), 6);

    cdb = SrbGetCdb((srbHeader).cast());
    (*cdb).CDB6INQUIRY3.as_mut().OperationCode = (SCSIOP_INQUIRY) as u8;
    (*cdb).CDB6INQUIRY3.as_mut().set_EnableVitalProductData(1);
    (*cdb).CDB6INQUIRY3.as_mut().PageCode = (VPD_SUPPORTED_PAGES) as u8;
    (*cdb).CDB6INQUIRY3.as_mut().AllocationLength = bufferLength;

    status = ClassSendSrbSynchronous(
        (*commonExtension).DeviceObject,
        srbHeader as PSCSI_REQUEST_BLOCK,
        supportedPages as PVOID,
        allocationBufferLength,
        0u8
    );

    if status == STATUS_DATA_OVERRUN && SrbGetDataTransferLength((srbHeader).cast()) < bufferLength as ULONG {
        status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(status) &&
       (*supportedPages).PageLength > 0 &&
       (*(*supportedPages).SupportedPageList.as_ptr().add((0) as usize)) > 0 {
        status = STATUS_NOT_SUPPORTED;
    }

    if NT_SUCCESS(status) {
        let mut i: i32 = 0;

        while i < (*supportedPages).PageLength as i32 {
            if i > 0 && (*(*supportedPages).SupportedPageList.as_ptr().add((i as usize) as usize)) <= (*(*supportedPages).SupportedPageList.as_ptr().add(((i - 1) as usize) as usize)) {
                (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceRODLimits((0u8) as u32);
                (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockLimits((0u8) as u32);
                (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceCharacteristics((0u8) as u32);
                (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_LBProvisioning((0u8) as u32);
                break;
            }

            match (*(*supportedPages).SupportedPageList.as_ptr().add((i as usize) as usize)) as u32 {
                VPD_THIRD_PARTY_COPY => {
                    (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceRODLimits((1u8) as u32);
                }
                VPD_BLOCK_LIMITS => {
                    (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockLimits((1u8) as u32);
                }
                VPD_BLOCK_DEVICE_CHARACTERISTICS => {
                    (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceCharacteristics((1u8) as u32);
                }
                VPD_LOGICAL_BLOCK_PROVISIONING => {
                    (*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.set_LBProvisioning((1u8) as u32);
                }
                _ => {}
            }

            i += 1;
        }
    }

    ExFreePool(supportedPages as PVOID);
}

// --- ClasspGetLBProvisioningInfo ---
#[no_mangle]
pub unsafe extern "C" fn ClasspGetLBProvisioningInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION
) -> NTSTATUS {
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbSize: ULONG = 0;

    if FdoExtension.is_null() ||
        (*FdoExtension).FunctionSupportInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.LBProvisioning() == (1u8) as u32) ||
        ((*(*FdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockLimits() == (1u8) as u32) {

        if (!(*FdoExtension).AdapterDescriptor.is_null()) &&
            ((*((*FdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8) {
            srbSize = (CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE) as u32;
        } else {
            srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
        }

        srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize as usize, u32::from_le_bytes(*b"0DcS")) as PSCSI_REQUEST_BLOCK;

        if srb.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    ClasspDeviceGetLBProvisioningVPDPage((*(FdoExtension as PCOMMON_DEVICE_EXTENSION)).DeviceObject, srb);

    ClasspDeviceGetBlockLimitsVPDPage(FdoExtension,
                                      srb,
                                      srbSize,
                                      core::ptr::addr_of_mut!((*(*FdoExtension).FunctionSupportInfo).BlockLimitsData));

    FREE_POOL((srb).cast());

    return STATUS_SUCCESS;
}

// --- ClasspBlockLimitsDataSnapshot ---
#[no_mangle]
pub unsafe extern "C" fn ClasspBlockLimitsDataSnapshot(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    ForceQuery: BOOLEAN,
    BlockLimitsData: PCLASS_VPD_B0_DATA,
    GenerationCount: *mut ULONG
) -> NTSTATUS {
    let status: NTSTATUS;
    let mut lockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    status = ClasspRefreshFunctionSupportInfo(FdoExtension, ForceQuery);

    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*(*FdoExtension).FunctionSupportInfo).SyncLock),
        &mut lockHandle
    );
    *BlockLimitsData = core::ptr::read(core::ptr::addr_of!((*(*FdoExtension).FunctionSupportInfo).BlockLimitsData));
    *GenerationCount = (*(*FdoExtension).FunctionSupportInfo).GenerationCount;
    KeReleaseInStackQueuedSpinLock(&mut lockHandle);

    status
}

// --- ClasspRegisterMountedDeviceInterface ---
#[no_mangle]
pub unsafe extern "C" fn ClasspRegisterMountedDeviceInterface(
    DeviceObject: PDEVICE_OBJECT
) {
    let commonExtension: *mut COMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let isFdo: u8 = (*commonExtension).__bindgen_anon_1.IsFdo();
    let pdo: PDEVICE_OBJECT;
    let mut interfaceName: _UNICODE_STRING = core::mem::zeroed();
    let status: NTSTATUS;

    if isFdo != 0 {
        let functionalExtension: *mut FUNCTIONAL_DEVICE_EXTENSION = commonExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
        pdo = (*functionalExtension).LowerPdo;
    } else {
        pdo = DeviceObject;
    }

    status = IoRegisterDeviceInterface(
        pdo,
        &MOUNTDEV_MOUNTED_DEVICE_GUID,
        core::ptr::null_mut(),
        &mut interfaceName
    );

    if NT_SUCCESS(status) {
        (*commonExtension).MountedDeviceInterfaceName = interfaceName;
        let status2 = IoSetDeviceInterfaceState(&mut interfaceName, 1u8);

        if !NT_SUCCESS(status2) {
            RtlFreeUnicodeString(&mut interfaceName);
        }
    }

    if !NT_SUCCESS(status) {
        RtlInitUnicodeString(
            &mut (*commonExtension).MountedDeviceInterfaceName,
            core::ptr::null_mut()
        );
    }
}

// --- ClassDetermineTokenOperationCommandSupport ---
#[no_mangle]
pub unsafe extern "C" fn ClassDetermineTokenOperationCommandSupport(
    DeviceObject: PDEVICE_OBJECT
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    TracePrint(TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClassDetermineTokenOperationCommandSupport (%p): Entering function.\n\0".as_ptr() as PCHAR,
                DeviceObject);

    status = ClasspGetBlockDeviceTokenLimitsInfo(DeviceObject);

    if NT_SUCCESS(status) {

        let mut maxListIdentifier: ULONG = MaxTokenOperationListIdentifier;

        if NT_SUCCESS(ClasspGetMaximumTokenListIdentifier(DeviceObject, REG_DISK_CLASS_CONTROL.as_ptr() as *mut u16, &mut maxListIdentifier)) {
            if maxListIdentifier >= MIN_TOKEN_LIST_IDENTIFIERS {

                NT_ASSERT!(maxListIdentifier <= MAX_TOKEN_LIST_IDENTIFIERS);
                MaxTokenOperationListIdentifier = maxListIdentifier;
            }
        }

    }

    TracePrint(TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClassDetermineTokenOperationCommandSupport (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                DeviceObject,
                status);

    return status;
}

// --- ClasspRetryRequestDpc ---
#[no_mangle]
pub unsafe extern "system" fn ClasspRetryRequestDpc(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    Arg1: PVOID,
    Arg2: PVOID,
) {

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: PCOMMON_DEVICE_EXTENSION;
    let fdoData: *mut CLASS_PRIVATE_FDO_DATA;
    let mut retryList: *mut CLASS_RETRY_INFO;
    let mut irql: KIRQL = 0;
    let DeviceObject = DeferredContext as PDEVICE_OBJECT;

    let _ = Dpc as PVOID;
    let _ = Arg1;
    let _ = Arg2;

    if DeferredContext.is_null() {
        NT_ASSERT!(0);
        return;
    }

    commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    NT_ASSERT!((*commonExtension).__bindgen_anon_1.IsFdo());
    fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    fdoData = (*fdoExtension).PrivateFdoData;

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).Retry.Lock)).cast(), core::ptr::addr_of_mut!(irql));
    {
        let mut now: LARGE_INTEGER = core::mem::zeroed();
        KeQueryTickCount(core::ptr::addr_of_mut!(now) as *mut i64);

        if (*core::ptr::addr_of!(now.QuadPart)) < (*core::ptr::addr_of!((*fdoData).Retry.Tick.QuadPart)) {
            ClasspRetryDpcTimer(fdoData);
            retryList = core::ptr::null_mut();
        } else {
            retryList = (*fdoData).Retry.ListHead;
            (*fdoData).Retry.ListHead = core::ptr::null_mut();
            (*fdoData).Retry.Delta.QuadPart = 0i64;
            (*fdoData).Retry.Tick.QuadPart = 0i64;
        }
    }
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).Retry.Lock)).cast(), irql);

    while !retryList.is_null() {
        let irp: PIRP;

        irp = (retryList as *const u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext) as isize)) as PIRP;
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassRetry:  -- %p\n\0".as_ptr() as PCHAR, irp);
        retryList = (*retryList).Next;

#[no_mangle]
pub unsafe extern "system" fn ClasspStartIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {

    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    if (*commonExtension).IsRemoved != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;

        ClassAcquireRemoveLock(DeviceObject, ClasspStartIo as PIRP);

        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_DISK_INCREMENT) as _);
        IoStartNextPacket(DeviceObject, 1u8);

        ClassReleaseRemoveLock(DeviceObject, ClasspStartIo as PIRP);

        return;
    }

    ((*(*commonExtension).DriverExtension).InitData.ClassStartIo.unwrap())(DeviceObject, Irp);
}

#[cfg(debug_assertions)]
        {
            (*irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = ULongToPtr(0xdddddddd);
            (*irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = ULongToPtr(0xdddddddd);
            (*irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[2] = ULongToPtr(0xdddddddd);
            (*irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] = ULongToPtr(0xdddddddd);
        }

        if NO_REMOVE == (crate::class_part_c::__cmpxchg(
            core::ptr::addr_of_mut!((*commonExtension).IsRemoved) as *mut i32,
            REMOVE_PENDING as i32,
            REMOVE_PENDING as i32,
        ).0) as u32 {
            IoCallDriver((*commonExtension).LowerDeviceObject, irp);
        } else {
            let irpStack: PIO_STACK_LOCATION;

            IoSetNextIrpStackLocation(irp);

            irpStack = IoGetCurrentIrpStackLocation(irp);

            if (*irpStack).MajorFunction == (IRP_MJ_SCSI) as u8 {
                let srb = (*irpStack).Parameters.Scsi.Srb;

                if !srb.is_null() {
                    (*(srb as *mut crate::types::_SCSI_REQUEST_BLOCK)).SrbStatus = SRB_STATUS_NO_DEVICE as u8;
                }
            }

            (*irpStack).Parameters.Others.Argument4 = core::ptr::null_mut();

            (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_SUCH_DEVICE;
            (*irp).IoStatus.Information = 0;
            IoCompleteRequest(irp, IO_NO_INCREMENT);
        }
    }
}

// --- ClasspUpdateDiskProperties ---
#[no_mangle]
pub unsafe extern "system" fn ClasspUpdateDiskProperties(
    Fdo: PDEVICE_OBJECT,
    Context: PVOID
) {
    let mut topOfStack: PDEVICE_OBJECT;
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut event: KEVENT = core::mem::zeroed();
    let mut irp: PIRP = core::ptr::null_mut();
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let WorkItem: PIO_WORKITEM = Context as PIO_WORKITEM;

    KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);

    topOfStack = IoGetAttachedDeviceReference(Fdo);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_DISK_UPDATE_PROPERTIES,
        topOfStack,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        &mut event,
        &mut ioStatus
    );

    if !irp.is_null() {
        status = IoCallDriver(topOfStack, irp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut event as *mut _ as *mut _,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0u8,
                core::ptr::null_mut()
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    // InterlockedExchange(&UpdateDiskPropertiesWorkItemActive, 0) — portable atomic swap.
    (*(core::ptr::addr_of_mut!((*fdoData).UpdateDiskPropertiesWorkItemActive)
        as *const core::sync::atomic::AtomicI32))
        .swap(0, core::sync::atomic::Ordering::SeqCst);

    if !NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"ClasspUpdateDiskProperties: Disk property update for fdo %p failed with status 0x%X.\n\0".as_ptr() as PCHAR,
            Fdo,
            status
        );
    } else {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClasspUpdateDiskProperties: Drive capacity has changed for %p.\n\0".as_ptr() as PCHAR,
            Fdo
        );
    }
    
    ObDereferenceObject(topOfStack as *mut _);

    if !WorkItem.is_null() {
        IoFreeWorkItem(WorkItem);
    }
}

// --- ClassDeviceControlDispatch ---
#[no_mangle]
pub unsafe extern "system" fn ClassDeviceControlDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let isRemoved: ULONG;

    isRemoved = ClassAcquireRemoveLockEx(DeviceObject, Irp as PVOID, b"class.c\0".as_ptr(), 0);
    ();
    if isRemoved != 0 {
        ClassReleaseRemoveLock(DeviceObject, Irp);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as i8) as _);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    NT_ASSERT!(((*(*commonExtension).DevInfo).ClassDeviceControl.is_some()) as BOOLEAN);

    return (*(*commonExtension).DevInfo).ClassDeviceControl.unwrap()(DeviceObject, Irp);
}

// --- ClasspBuildRequestEx ---
#[no_mangle]
pub unsafe extern "C" fn ClasspBuildRequestEx(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK
) {
    let currentIrpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let nextIrpStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);

    let startingOffset: LARGE_INTEGER = (*currentIrpStack).Parameters.Read.ByteOffset;

    let mut cdb: PCDB;
    let mut logicalBlockAddress: ULONG;
    let mut transferBlocks: USHORT;
    let mut status: NTSTATUS;
    let srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER = Srb as PSTORAGE_REQUEST_BLOCK_HEADER;

    if Srb.is_null() {
        NT_ASSERT!(0);
        return;
    }

    logicalBlockAddress = Int64ShrlMod32(
        startingOffset.QuadPart as u64,
        ((*FdoExtension).SectorShift) as u32
    ) as ULONG;

    if (*((*FdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        status = InitializeStorageRequestBlock(
            Srb as PSTORAGE_REQUEST_BLOCK,
            STORAGE_ADDRESS_TYPE_BTL8 as u16,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
            1,
            SrbExDataTypeScsiCdb16 as u32
        );
        if !NT_SUCCESS(status) {
            NT_ASSERT!(0);
            return;
        }

        (*(Srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
    } else {
        RtlZeroMemory(Srb as *mut core::ffi::c_void, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
    }

    SrbSetOriginalRequest(srbHeader as *mut core::ffi::c_void, Irp);

    SrbSetDataBuffer(
        (srbHeader).cast(),
        MmGetMdlVirtualAddress((*Irp).MdlAddress)
    );

    SrbSetDataTransferLength(
        (srbHeader).cast(),
        (*currentIrpStack).Parameters.Read.Length
    );

    SrbSetRequestAttribute((srbHeader).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);

    SrbSetQueueSortKey((srbHeader).cast(), logicalBlockAddress);

    SrbSetSenseInfoBuffer(srbHeader as *mut core::ffi::c_void, (*FdoExtension).SenseData as *mut core::ffi::c_void);
    SrbSetSenseInfoBufferLength((srbHeader).cast(), ((GET_FDO_EXTENSON_SENSE_DATA_LENGTH((FdoExtension).cast())) as u32) as _);

    SrbSetTimeOutValue(
        (srbHeader).cast(),
        ((SrbGetDataTransferLength((srbHeader).cast()) + 0xFFFF) >> 16) * (*FdoExtension).TimeOutValue
    );

    SrbSetCdbLength((srbHeader).cast(), 10);

    cdb = SrbGetCdb((srbHeader).cast());
    NT_ASSERT!(!cdb.is_null());

    transferBlocks = ((*currentIrpStack).Parameters.Read.Length >> (*FdoExtension).SectorShift) as USHORT;

    (*cdb).CDB10.as_mut().LogicalBlockByte0 = (&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(3).read();
    (*cdb).CDB10.as_mut().LogicalBlockByte1 = (&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(2).read();
    (*cdb).CDB10.as_mut().LogicalBlockByte2 = (&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(1).read();
    (*cdb).CDB10.as_mut().LogicalBlockByte3 = (&logicalBlockAddress as *const ULONG as *const FOUR_BYTE).cast::<u8>().offset(0).read();

    (*cdb).CDB10.as_mut().TransferBlocksMsb = (&transferBlocks as *const USHORT as *const FOUR_BYTE).cast::<u8>().offset(1).read();
    (*cdb).CDB10.as_mut().TransferBlocksLsb = (&transferBlocks as *const USHORT as *const FOUR_BYTE).cast::<u8>().offset(0).read();

    if (*currentIrpStack).MajorFunction == IRP_MJ_READ {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, b"ClassBuildRequest: Read Command\n\0".as_ptr() as PCHAR);

        SrbSetSrbFlags((srbHeader).cast(), SRB_FLAGS_DATA_IN);
        (*cdb).CDB10.as_mut().OperationCode = (SCSIOP_READ) as u8;
    } else {
        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_RW, b"ClassBuildRequest: Write Command\n\0".as_ptr() as PCHAR);

        SrbSetSrbFlags((srbHeader).cast(), SRB_FLAGS_DATA_OUT);
        (*cdb).CDB10.as_mut().OperationCode = (SCSIOP_WRITE) as u8;
    }

    if ((*currentIrpStack).Flags & (SL_WRITE_THROUGH) as u8) == 0 {
        SrbSetSrbFlags((srbHeader).cast(), SRB_FLAGS_ADAPTER_CACHE_ENABLE);
    } else {
        (*cdb).CDB10.as_mut().set_ForceUnitAccess((*FdoExtension).CdbForceUnitAccess);
    }

    if TEST_FLAG((*Irp).Flags, IRP_PAGING_IO | IRP_SYNCHRONOUS_PAGING_IO) != 0 {
        SrbSetSrbFlags((srbHeader).cast(), SRB_CLASS_FLAGS_PAGING);
    }

    SrbSetSrbFlags((srbHeader).cast(), (*FdoExtension).SrbFlags);

    (*nextIrpStack).MajorFunction = (IRP_MJ_SCSI) as u8;

    (*nextIrpStack).Parameters.Scsi.Srb = Srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;

    (*currentIrpStack).Parameters.Others.Argument4 = MAXIMUM_RETRIES as PVOID;

    IoSetCompletionRoutine(Irp, Some(ClassIoComplete), Srb as PVOID, 1u8, 1u8, 1u8);
}


#[inline(always)]
pub unsafe fn __cmpxchg(ptr: *mut i32, new: i32, cmp: i32) -> (i32, bool) {
    let a = &*(ptr as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(cmp, new, core::sync::atomic::Ordering::AcqRel, core::sync::atomic::Ordering::Acquire) {
        Ok(v) => (v, true),
        Err(v) => (v, false),
    }
}


// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "system" {
    fn ClassAcquireRemoveLockEx(DeviceObject: PDEVICE_OBJECT, Tag: PVOID, File: *const u8, Line: ULONG) -> ULONG;
}

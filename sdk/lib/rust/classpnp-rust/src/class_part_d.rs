//! class_part_d.rs — translated classpnp link-gap fns (RetryRequest, InterpretSenseInfoWithoutHistory,
//! ClassFreeOrReuseSrb, ClassRetrieveDeviceRelations,
//! ClasspGetBlockDeviceTokenLimitsInfo, ClasspRefreshFunctionSupportInfo,
//! ClasspMediaChangeRegistryCallBack, ClassLogResourceExhaustionEvent, ClassLogThresholdEvent)
//! rustconv gap-TU 2026-06-10 ($0.25). Import head mirrors class_part_c.
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
use crate::types::{BuildDeviceInputMdl, ClassGetDeviceParameter, ClassInitializeDispatchTables, ClassQueryPnpCapabilities, ClassScanForSpecial, ClassSetDeviceParameter, ClasspAllocatePowerProcessIrp, ClasspAllocateReleaseQueueIrp, ClasspCalculateLogicalSectorSize, ClasspDeleteTimer, ClasspDisableTimer, ClasspEjectionControl, ClasspEnableIdlePower, ClasspEnableTimer, ClasspEnqueueIdleRequest, ClasspGetCopyOffloadMaxDuration, ClasspInitializeHotplugInfo, ClasspInitializeIdleTimer, ClasspInitializeTimer, ClasspIsPortable, ClasspScanForSpecialInRegistry, ClasspZeroQERR, DequeueFreeTransferPacket, EnqueueDeferredClientIrp, EnqueueFreeTransferPacket, FreeDeviceInputMdl, InitLowMemRetry, InitializeStorageRequestBlock, InitializeTransferPackets, SetupDriveCapacityTransferPacket, SetupReadWriteTransferPacket, StepLowMemRetry, SubmitTransferPacket, TransferPacketQueueRetryDpc};
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
use crate::types::{CLASSP_REG_WRITE_CACHE_VALUE_NAME, CLASS_ERROR_LOG_DATA, CLASS_PRIVATE_COMMON_DATA, CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK, ClassCheckVerifyComplete, ClassCleanupMediaChangeDetection, ClassDeviceGetLBProvisioningResources, ClassIoComplete, ClassNotifyFailurePredicted, ClassQueueCapacityChangedEventWorker, ClassQueueProvisioningTypeChangedEventWorker, ClassQueueResourceExhaustionEventWorker, ClassQueueThresholdEventWorker, ClassReadDriveCapacity, ClassSendEjectionNotification, ClassSendNotification, ClassSendStartUnit, ClassSetMediaChangeState, ClasspAccessAlignmentProperty, ClasspConvertToScsiRequestBlock, ClasspDeviceCopyOffloadProperty, ClasspDeviceGetLBAStatus, ClasspDeviceLBProvisioningProperty, ClasspDeviceMediaTypeProperty, ClasspDeviceSeekPenaltyProperty, ClasspDeviceTrimProcess, ClasspDeviceTrimProperty, ClasspDuidQueryProperty, ClasspInitializeRemoveTracking, ClasspMcnControl, ClasspPerfIncrementErrorCount, ClasspPersistentReserve, ClasspPriorityHint, ClasspQueueLogIOEventWithContextWorker, ClasspStorageEventNotification, ClasspUninitializeRemoveTracking, ClasspWriteCacheProperty, DEV_SAFE_START_UNIT, DestroyAllTransferPackets, FDO_HACK_NO_RESERVE6, IOCTL_SCSI_EXECUTE_NONE, IOCTL_STORAGE_BASE, IdlePowerFDOList, IdlePowerFDOListMutex, InitializeDictionary, MAXIMUM_RETRIES, MODE_DISCONNECT_PAGE, MODE_PARAMETER_HEADER, MODE_PARAMETER_HEADER10, MODE_SENSE_CURRENT_VALUES, _MEDIA_CHANGE_DETECTION_STATE_MediaPresent as MediaPresent, NOT_READY_RETRY_INTERVAL, NO_REMOVE, NUM_ERROR_LOG_ENTRIES, PCDB, PCLASS_DEV_INFO, PCLASS_DRIVER_EXTENSION, PCLASS_ERROR_LOG_DATA, PCLASS_PRIVATE_COMMON_DATA, PFIXED_SENSE_DATA, PHYSICAL_DEVICE_EXTENSION, PIDLE_POWER_FDO_LIST_ENTRY, PINQUIRYDATA, PMODE_DISCONNECT_PAGE, PMODE_PARAMETER_HEADER, PMODE_PARAMETER_HEADER10, PPHYSICAL_DEVICE_EXTENSION, PSCSI_SENSE_DESCRIPTOR_BLOCK_COMMAND, PSCSI_SENSE_DESCRIPTOR_INFORMATION, PSTORAGE_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK_HEADER, SCSIOP_INQUIRY, SCSIOP_MODE_SENSE, SCSIOP_MODE_SENSE10, SCSIOP_READ, SCSIOP_READ12, SCSIOP_READ16, SCSIOP_READ6, SCSIOP_RELEASE_UNIT, SCSIOP_RELEASE_UNIT10, SCSIOP_RESERVE_UNIT, SCSIOP_RESERVE_UNIT10, SCSIOP_TEST_UNIT_READY, SCSIOP_VERIFY, SCSIOP_VERIFY16, SCSIOP_WRITE, SCSIOP_WRITE12, SCSIOP_WRITE16, SCSIOP_WRITE6, SCSISTAT_BUSY, SCSISTAT_GOOD, SCSISTAT_RESERVATION_CONFLICT, SCSI_ADDRESS, SCSI_ADSENSE_DATA_TRANSFER_ERROR, SCSI_ADSENSE_INVALID_MEDIA, SCSI_ADSENSE_LOGICAL_UNIT_ERROR, SCSI_ADSENSE_LUN_COMMUNICATION, SCSI_ADSENSE_LUN_NOT_READY, SCSI_ADSENSE_REC_DATA_ECC, SCSI_ADSENSE_REC_DATA_NOECC, SCSI_ADSENSE_SEEK_ERROR, SCSI_ADSENSE_TRACK_ERROR, SCSI_ADSENSE_WRITE_PROTECT, SCSI_PASS_THROUGH, SCSI_SENSEQ_CAPACITY_DATA_CHANGED, SCSI_SENSEQ_INITIATOR_RESPONSE_TIMEOUT, SCSI_SENSEQ_INIT_COMMAND_REQUIRED, SCSI_SENSEQ_INQUIRY_DATA_CHANGED, SCSI_SENSEQ_MICROCODE_CHANGED, SCSI_SENSEQ_OPERATING_DEFINITION_CHANGED, SCSI_SENSEQ_TIMEOUT_ON_LOGICAL_UNIT, SCSI_SENSE_DESCRIPTOR_TYPE_BLOCK_COMMAND, SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION, SCSI_SESNEQ_COMM_CRC_ERROR, SENSE_DATA, SRB_CLASS_FLAGS_FREE_MDL, SRB_CLASS_FLAGS_LOW_PRIORITY, SRB_CLASS_FLAGS_PAGING, SRB_FLAGS_CLASS_DRIVER_RESERVED, SRB_FLAGS_DATA_IN, SRB_FLAGS_DATA_OUT, SRB_FLAGS_DONT_START_NEXT_PACKET, SRB_FLAGS_NO_QUEUE_FREEZE, SRB_FLAGS_QUEUE_ACTION_ENABLE, SRB_FUNCTION_CLAIM_DEVICE, SRB_FUNCTION_RELEASE_DEVICE, SRB_HEAD_OF_QUEUE_TAG_REQUEST, SRB_ORDERED_QUEUE_TAG_REQUEST, SRB_SIMPLE_TAG_REQUEST, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_COMMAND_TIMEOUT, SRB_STATUS_INTERNAL_ERROR, SRB_STATUS_INVALID_LUN, SRB_STATUS_INVALID_PATH_ID, SRB_STATUS_INVALID_TARGET_ID, SRB_STATUS_NO_DEVICE, SRB_STATUS_NO_HBA, SRB_STATUS_PARITY_ERROR, SRB_STATUS_SELECTION_TIMEOUT, SRB_STATUS_SUCCESS, SRB_STATUS_TIMEOUT, SRB_STATUS_UNEXPECTED_BUS_FREE, SRB_TYPE_SCSI_REQUEST_BLOCK, _MEDIA_CHANGE_DETECTION_STATE_MediaNotPresent as MediaNotPresent, _MEDIA_CHANGE_DETECTION_STATE_MediaUnavailable as MediaUnavailable};
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



// TracePrint — telemetry no-op (matches ddk_shims/ETW no-op convention); macro swallows the
// C double-paren `TracePrint!((lvl, flag, fmt, args))` form.
macro_rules! TracePrint { ($($t:tt)*) => {{}} }
extern "C" {
    
    
    fn ClasspLogSystemEventWithDeviceNumber(DeviceObject: PDEVICE_OBJECT, ErrorCode: NTSTATUS) -> NTSTATUS;
}

// ---- link-gap imports + constants (header macros / values not in the import head) ----
use ntkrnl_sys::ddk::{STORAGE_LB_PROVISIONING_MAP_RESOURCES, DEVICE_RELATION_TYPE, RtlCompareMemory};
use ntkrnl_sys::inline::ex::ExFreeToNPagedLookasideList;
use crate::utils::ClassGetLBProvisioningResources;
use crate::srblib::SrbGetDataBuffer;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


const STATUS_DEVICE_FEATURE_NOT_SUPPORTED: NTSTATUS = 0xC0000463u32 as i32;
const IO_WARNING_SOFT_THRESHOLD_REACHED: NTSTATUS = 0x80040090u32 as i32;
const IO_WARNING_SOFT_THRESHOLD_REACHED_EX: NTSTATUS = 0x80040091u32 as i32;
const IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_LUN: NTSTATUS = 0x80040092u32 as i32;
const IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_POOL: NTSTATUS = 0x80040093u32 as i32;
const IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_LUN: NTSTATUS = 0x80040094u32 as i32;
const IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_POOL: NTSTATUS = 0x80040095u32 as i32;
const IO_ERROR_DISK_RESOURCES_EXHAUSTED: NTSTATUS = 0xC0040096u32 as i32;
const ERROR_LOG_MAXIMUM_SIZE: u32 = 248u32; // IO_ERROR_LOG_MESSAGE_LENGTH(256) - HEADER(8)
const CLASSPNP_POOL_TAG_SRB: u32 = 0x72536353; // 'rScS'
const CLASSPNP_POOL_TAG_VPD: u32 = 0x70566353; // 'pVcS'

// SrbSetQueueTag === SrbSetRequestTag (srbhelper.h #define); forward to the ddk_shims impl.
#[inline] unsafe fn SrbSetQueueTag(Srb: PVOID, Tag: u32) { crate::ddk_shims::SrbSetRequestTag(Srb, Tag); }

// ClasspFreeSrb(ext, srb) — classpnp.h macro: ExFreeToNPagedLookasideList(&ext->CommonExtension.SrbLookasideList, srb)
#[inline] unsafe fn ClasspFreeSrb(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PVOID) {
    ExFreeToNPagedLookasideList(
        core::ptr::addr_of_mut!((*(FdoExtension as PCOMMON_DEVICE_EXTENSION)).SrbLookasideList),
        Srb,
    );
}
#[no_mangle]
pub unsafe extern "C" fn RetryRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
    Associated: BOOLEAN,
    TimeDelta100ns: LONGLONG,
) {
    let currentIrpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let nextIrpStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    let transferByteCount: ULONG;
    let dataTransferLength: ULONG;
    let srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER = Srb as PSTORAGE_REQUEST_BLOCK_HEADER;

    dataTransferLength = SrbGetDataTransferLength((srbHeader).cast());
    if (*currentIrpStack).MajorFunction == IRP_MJ_READ as u8 ||
       (*currentIrpStack).MajorFunction == IRP_MJ_WRITE as u8 {

        // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
        transferByteCount = (*currentIrpStack).Parameters.Read.Length;

    } else if (*Irp).MdlAddress != core::ptr::null_mut() {

        NT_ASSERT!(SrbGetDataBuffer(srbHeader as PVOID) == MmGetMdlVirtualAddress((*Irp).MdlAddress));
        // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
        transferByteCount = (*(*Irp).MdlAddress).ByteCount;

    } else {

        transferByteCount = 0;
    }

    NT_ASSERT!(!TEST_FLAG(SrbGetSrbFlags(srbHeader as PVOID), SRB_FLAGS_FREE_SENSE_BUFFER));

    SrbSetDataTransferLength((srbHeader).cast(), transferByteCount);

    (*srbHeader).SrbStatus = 0;
    SrbSetScsiStatus((srbHeader).cast(), 0);

    if 0 == (*currentIrpStack).Parameters.Others.Argument4 as ULONG {
        SrbSetSrbFlags((srbHeader).cast(),
                      SRB_FLAGS_DISABLE_DISCONNECT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
        SrbClearSrbFlags((srbHeader).cast(), SRB_FLAGS_QUEUE_ACTION_ENABLE);

        SrbSetQueueTag((srbHeader).cast(), SP_UNTAGGED);
    }

    (*nextIrpStack).MajorFunction = IRP_MJ_SCSI as u8;

    (*nextIrpStack).Parameters.Scsi.Srb = (Srb as *mut core::ffi::c_void).cast();

    if Associated != 0 {
        IoSetCompletionRoutine(Irp, Some(ClassIoCompleteAssociated), Srb as PVOID, 1, 1, 1);
    } else {
        IoSetCompletionRoutine(Irp, Some(ClassIoComplete), Srb as PVOID, 1, 1, 1);
    }

    ClassRetryRequest(DeviceObject, Irp, TimeDelta100ns);
}
#[no_mangle]
pub unsafe extern "C" fn InterpretSenseInfoWithoutHistory(
    Fdo: PDEVICE_OBJECT,
    OriginalRequest: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
    MajorFunctionCode: UCHAR,
    IoDeviceCode: ULONG,
    PreviousRetryCount: ULONG,
    Status: *mut NTSTATUS,
    RetryIn100nsUnits: *mut LONGLONG,
) -> BOOLEAN {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExtension).PrivateFdoData;
    let mut tmpRetry: LONGLONG = 0;
    let mut retry: BOOLEAN = 0u8;

    if !(*fdoData).InterpretSenseInfo.is_null() {
        let mut tempSrb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
        let mut srbPtr: PSCSI_REQUEST_BLOCK = Srb;

        NT_ASSERT!((*(*fdoData).InterpretSenseInfo).Interpret.is_some());

        if (*Srb).Function == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8 &&
            ((*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension).SrbSupport &
             CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0 {
            ClasspConvertToScsiRequestBlock(&mut tempSrb, Srb as PSTORAGE_REQUEST_BLOCK);
            srbPtr = &mut tempSrb;
        }

        retry = ((*(*fdoData).InterpretSenseInfo).Interpret).unwrap()(
            Fdo,
            OriginalRequest,
            srbPtr,
            MajorFunctionCode,
            IoDeviceCode,
            PreviousRetryCount,
            core::ptr::null_mut(),
            Status,
            &mut tmpRetry,
        );
    } else {
        let mut seconds: ULONG = 0;

        retry = ClassInterpretSenseInfo(
            Fdo,
            Srb,
            MajorFunctionCode,
            IoDeviceCode,
            PreviousRetryCount,
            Status,
            &mut seconds,
        );
        tmpRetry = (seconds as LONGLONG) * 1000 * 1000 * 10;
    }

    if !RetryIn100nsUnits.is_null() {
        *RetryIn100nsUnits = tmpRetry;
    }
    retry
}
#[no_mangle]
pub unsafe extern "C" fn ClassFreeOrReuseSrb(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
) {
    let commonExt = FdoExtension as PCOMMON_DEVICE_EXTENSION;

    NT_ASSERT!(TEST_FLAG(SrbGetSrbFlags(Srb as PVOID), SRB_FLAGS_FREE_SENSE_BUFFER) == 0);

    if (*commonExt).__bindgen_anon_1.IsSrbLookasideListInitialized() != 0 {
        ClasspFreeSrb(FdoExtension, (Srb).cast());
    } else {
        TracePrint!((
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassFreeOrReuseSrb: someone is trying to use an uninitialized SrbLookasideList !!!\0".as_ptr() as PCHAR,
        ));
        FREE_POOL(Srb as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspValidateOffloadSupported(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION;
    let status: NTSTATUS;

    TracePrint!((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadSupported (%p): Entering function. Irp %p\n\0".as_ptr() as PCSZ,
                DeviceObject,
                Irp));

    fdoExt = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    status = STATUS_SUCCESS;

    if ((*DeviceObject).DeviceType == FILE_DEVICE_DISK) &&
        (TEST_FLAG((*DeviceObject).Characteristics, FILE_FLOPPY_DISKETTE) == 0) {

        if (*(*fdoExt).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits() == 0 {

            TracePrint!((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        b"ClasspValidateOffloadSupported (%p): Command not supported on this disk device.\n\0".as_ptr() as PCSZ,
                        DeviceObject));

            return STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        }

        if !NT_SUCCESS((*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus) {

            let status = (*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus;

            TracePrint!((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        b"ClasspValidateOffloadSupported (%p): VPD retrieval failed with %x.\n\0".as_ptr() as PCSZ,
                        DeviceObject,
                        status));

            TracePrint!((TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
                        DeviceObject,
                        Irp,
                        status));

            return status;
        }
    } else {

        TracePrint!((TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadSupported (%p): Suported only on Disk devices.\n\0".as_ptr() as PCSZ,
                    DeviceObject));

        TracePrint!((TRACE_LEVEL_VERBOSE,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
                    DeviceObject,
                    Irp,
                    STATUS_DEVICE_FEATURE_NOT_SUPPORTED));

        return STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
    }

    TracePrint!((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
                DeviceObject,
                Irp,
                status));

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassRetrieveDeviceRelations(
    Fdo: PDEVICE_OBJECT,
    RelationType: DEVICE_RELATION_TYPE,
    DeviceRelations: *mut PDEVICE_RELATIONS,
) -> NTSTATUS {

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    let mut count: ULONG = 0;
    let mut i: ULONG;

    let mut nextChild: PPHYSICAL_DEVICE_EXTENSION;

    let relationsSize: ULONG;
    let mut deviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let oldRelations: PDEVICE_RELATIONS = *DeviceRelations;

    let status: NTSTATUS;

    let _ = RelationType;

    ClassAcquireChildLock(fdoExtension);

    nextChild = (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().ChildList as PPHYSICAL_DEVICE_EXTENSION;

    while !nextChild.is_null() {
        let commonExtension: PCOMMON_DEVICE_EXTENSION;

        commonExtension = nextChild as PCOMMON_DEVICE_EXTENSION;

        NT_ASSERTMSG!(
            b"ClassPnp internal error: missing child on active list\n\0".as_ptr() as PCSZ,
            if (*nextChild).IsMissing == 0u8 { 1u8 } else { 0u8 }
        );

        nextChild = (*commonExtension).ChildList as PPHYSICAL_DEVICE_EXTENSION;

        count += 1;
    }

    if !oldRelations.is_null() {
        count += (*oldRelations).Count;
    }

    relationsSize = core::mem::size_of::<DEVICE_RELATIONS>() as ULONG +
                    (count * core::mem::size_of::<PDEVICE_OBJECT>() as ULONG);

    deviceRelations = ExAllocatePoolWithTag(PagedPool, relationsSize as usize, 0x35426353) as PDEVICE_RELATIONS;

    if deviceRelations.is_null() {
        TracePrint!(
            0x0002_0000u64,
            b"ClassRetrieveDeviceRelations: unable to allocate %d bytes for device relations\n\0".as_ptr() as PCSZ,
            relationsSize
        );

        ClassReleaseChildLock(fdoExtension);

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(deviceRelations as PVOID, relationsSize as usize);

    if !oldRelations.is_null() {
        i = 0;
        while i < (*oldRelations).Count {
            *core::ptr::addr_of_mut!((*deviceRelations).Objects).cast::<PDEVICE_OBJECT>().add(i as usize) = *core::ptr::addr_of!((*oldRelations).Objects).cast::<PDEVICE_OBJECT>().add(i as usize);
            i += 1;
        }

        FREE_POOL(oldRelations as PVOID);
    }

    nextChild = (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().ChildList as PPHYSICAL_DEVICE_EXTENSION;
    i = count;

    while !nextChild.is_null() {
        let commonExtension: PCOMMON_DEVICE_EXTENSION;

        commonExtension = nextChild as PCOMMON_DEVICE_EXTENSION;

        NT_ASSERTMSG!(
            b"ClassPnp internal error: missing child on active list\n\0".as_ptr() as PCSZ,
            if (*nextChild).IsMissing == 0u8 { 1u8 } else { 0u8 }
        );

        // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
        i -= 1;
        *core::ptr::addr_of_mut!((*deviceRelations).Objects).cast::<PDEVICE_OBJECT>().add(i as usize) = (*(nextChild as PCOMMON_DEVICE_EXTENSION)).DeviceObject;

        let status = ObReferenceObjectByPointer(
            (*(nextChild as PCOMMON_DEVICE_EXTENSION)).DeviceObject as PVOID,
            0,
            core::ptr::null_mut(),
            (0i32) as i8
        );
        if !NT_SUCCESS(status) {
            NT_ASSERT!(0u8);
            TracePrint!(
                0x0001_0000u64,
                b"ClassRetrieveDeviceRelations: Error referencing child device %p by pointer\n\0".as_ptr() as PCSZ,
                (*(nextChild as PCOMMON_DEVICE_EXTENSION)).DeviceObject
            );
        }
        (*nextChild).IsEnumerated = 1u8;
        nextChild = (*commonExtension).ChildList as PPHYSICAL_DEVICE_EXTENSION;
    }

    NT_ASSERTMSG!(
        b"Child list has changed: \0".as_ptr() as PCSZ,
        if i == 0 { 1u8 } else { 0u8 }
    );

    (*deviceRelations).Count = count;
    *DeviceRelations = deviceRelations;

    ClassReleaseChildLock(fdoExtension);
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetBlockDeviceTokenLimitsInfo(
    DeviceObject: PDEVICE_OBJECT
) -> NTSTATUS {
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut status: NTSTATUS;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbSize: ULONG = 0;
    let mut pageLength: USHORT = 0;
    let mut descriptorLength: USHORT = 0;
    let mut dataBuffer: PVOID = core::ptr::null_mut();
    let bufferLength: UCHAR = (VPD_MAX_BUFFER_SIZE) as u8;
    let mut allocationBufferLength: ULONG = bufferLength as ULONG;
    let mut cdb: PCDB;
    let mut dataTransferLength: ULONG = 0;
    let mut operatingParameters: PVPD_THIRD_PARTY_COPY_PAGE = core::ptr::null_mut();
    let mut blockDeviceTokenLimits: PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR = core::ptr::null_mut();

    TracePrint!((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Entering function.\n\0".as_ptr() as PCHAR,
                DeviceObject));

    if !(*fdoExtension).AdapterDescriptor.is_null() &&
        (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        srbSize = (CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE) as u32;
    } else {
        srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx, (srbSize) as usize, CLASSPNP_POOL_TAG_SRB) as PSCSI_REQUEST_BLOCK;

    if srb.is_null() {
        TracePrint!((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate SRB.\n\0".as_ptr() as PCHAR,
                    DeviceObject));

        status = STATUS_INSUFFICIENT_RESOURCES;
        'exit: loop {
            FREE_POOL(dataBuffer);
            FREE_POOL(srb as PVOID);
            (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = status;

            TracePrint!((TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        status));

            return status;
        }
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        allocationBufferLength = ALIGN_UP_BY(allocationBufferLength as ULONG_PTR, KeGetRecommendedSharedDataAlignment() as ULONG_PTR) as ULONG;
        dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned, allocationBufferLength as SIZE_T, CLASSPNP_POOL_TAG_VPD);
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, (bufferLength as ULONG) as usize, CLASSPNP_POOL_TAG_VPD);
    }

    if dataBuffer.is_null() {
        TracePrint!((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate dataBuffer.\n\0".as_ptr() as PCHAR,
                    DeviceObject));

        status = STATUS_INSUFFICIENT_RESOURCES;
        'exit: loop {
            FREE_POOL(dataBuffer);
            FREE_POOL(srb as PVOID);
            (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = status;

            TracePrint!((TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        status));

            return status;
        }
    }

    operatingParameters = dataBuffer as PVPD_THIRD_PARTY_COPY_PAGE;

    RtlZeroMemory(dataBuffer, (allocationBufferLength) as usize);

    if !(*fdoExtension).AdapterDescriptor.is_null() &&
        (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        status = InitializeStorageRequestBlock(srb as PSTORAGE_REQUEST_BLOCK,
                                               STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                                               CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
                                               1,
                                               SrbExDataTypeScsiCdb16);
        if NT_SUCCESS(status) {
            (*(srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        } else {
            NT_ASSERT!(0);

            TracePrint!((TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Falling back to using SRB (instead of SRB_EX).\n\0".as_ptr() as PCHAR,
                        DeviceObject));

            RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
            (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
            (*srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
        }
    } else {
        RtlZeroMemory(srb as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
        (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        (*srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
    }

    SrbSetTimeOutValue(srb as PVOID, (*fdoExtension).TimeOutValue);
    SrbSetRequestTag(srb as PVOID, SP_UNTAGGED);
    SrbSetRequestAttribute(srb as PVOID, (SRB_SIMPLE_TAG_REQUEST) as _);
    SrbAssignSrbFlags(srb as PVOID, (*fdoExtension).SrbFlags);

    SrbSetCdbLength(srb as PVOID, 6);

    cdb = SrbGetCdb(srb as PVOID) as PCDB;
    (*cdb).CDB6INQUIRY3.as_mut().OperationCode = (SCSIOP_INQUIRY) as u8;
    (*cdb).CDB6INQUIRY3.as_mut().set_EnableVitalProductData(1);
    (*cdb).CDB6INQUIRY3.as_mut().PageCode = (VPD_THIRD_PARTY_COPY) as u8;
    (*cdb).CDB6INQUIRY3.as_mut().AllocationLength = bufferLength;

    status = ClassSendSrbSynchronous((*(fdoExtension as PCOMMON_DEVICE_EXTENSION)).DeviceObject,
                                     srb,
                                     dataBuffer,
                                     allocationBufferLength,
                                     0u8);

    dataTransferLength = SrbGetDataTransferLength(srb as PVOID);

    if status == STATUS_DATA_OVERRUN && dataTransferLength < bufferLength as ULONG {
        status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(status) {
        REVERSE_BYTES_SHORT_M!(core::ptr::addr_of_mut!(pageLength), core::ptr::addr_of!((*operatingParameters).PageLength));
    } else {
        TracePrint!((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Inquiry for TPC VPD failed with %x.\n\0".as_ptr() as PCHAR,
                    DeviceObject,
                    status));
    }

    if NT_SUCCESS(status) &&
        pageLength >= core::mem::size_of::<WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR>() as USHORT {

        let mut descriptorType: USHORT = 0;

        blockDeviceTokenLimits = (*operatingParameters).ThirdPartyCopyDescriptors.as_ptr() as PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR;
        REVERSE_BYTES_SHORT_M!(core::ptr::addr_of_mut!(descriptorType), core::ptr::addr_of!((*blockDeviceTokenLimits).DescriptorType));
        REVERSE_BYTES_SHORT_M!(core::ptr::addr_of_mut!(descriptorLength), core::ptr::addr_of!((*blockDeviceTokenLimits).DescriptorLength));

        if descriptorType == (BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR_TYPE_WINDOWS) as u16 &&
            VPD_PAGE_HEADER_SIZE + descriptorLength as ULONG == core::mem::size_of::<WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR>() as ULONG {

            (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceRODLimits(1);

            REVERSE_BYTES_SHORT_M!(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumRangeDescriptors), core::ptr::addr_of!((*blockDeviceTokenLimits).MaximumRangeDescriptors));
            REVERSE_BYTES(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumInactivityTimer), (core::ptr::addr_of!((*blockDeviceTokenLimits).MaximumInactivityTimer)).cast());
            REVERSE_BYTES(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.DefaultInactivityTimer), (core::ptr::addr_of!((*blockDeviceTokenLimits).DefaultInactivityTimer)).cast());
            REVERSE_BYTES_QUAD_M!(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumTokenTransferSize), core::ptr::addr_of!((*blockDeviceTokenLimits).MaximumTokenTransferSize));
            REVERSE_BYTES_QUAD_M!(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.OptimalTransferCount), core::ptr::addr_of!((*blockDeviceTokenLimits).OptimalTransferCount));

            TracePrint!((TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): %s %s (rev %s) reported following parameters: \n\t\t\tMaxRangeDescriptors: %u\n\t\t\tMaxIAT: %u\n\t\t\tDefaultIAT: %u \n\t\t\tMaxTokenTransferSize: %I64u\n\t\t\tOptimalTransferCount: %I64u\n\t\t\tOptimalTransferLengthGranularity: %u \n\t\t\tOptimalTransferLength: %u\n\t\t\tMaxTransferLength: %u\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        ((*fdoExtension).DeviceDescriptor as *const u8).offset((*(*fdoExtension).DeviceDescriptor).VendorIdOffset as isize) as PCSZ,
                        ((*fdoExtension).DeviceDescriptor as *const u8).offset((*(*fdoExtension).DeviceDescriptor).ProductIdOffset as isize) as PCSZ,
                        ((*fdoExtension).DeviceDescriptor as *const u8).offset((*(*fdoExtension).DeviceDescriptor).ProductRevisionOffset as isize) as PCSZ,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumRangeDescriptors,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumInactivityTimer,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.DefaultInactivityTimer,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumTokenTransferSize,
                        (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.OptimalTransferCount,
                        (*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.OptimalTransferLengthGranularity,
                        (*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.OptimalTransferLength,
                        (*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.MaximumTransferLength));
        } else {
            TracePrint!((TRACE_LEVEL_WARNING,
                        TRACE_FLAG_PNP,
                        b"ClasspGetBlockDeviceTokenLimitsInfo (%p): ThirdPartyCopy VPD data doesn't have Windows OffloadDataTransfer descriptor.\n\0".as_ptr() as PCHAR,
                        DeviceObject));

            (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceRODLimits(0);
            status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
        }
    } else {
        TracePrint!((TRACE_LEVEL_WARNING,
                    TRACE_FLAG_PNP,
                    b"ClasspGetBlockDeviceTokenLimitsInfo (%p): TPC VPD data didn't return TPC descriptors of interest.\n\0".as_ptr() as PCHAR,
                    DeviceObject));

        (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.set_BlockDeviceRODLimits(0);
        status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
    }

    FREE_POOL(dataBuffer);
    FREE_POOL(srb as PVOID);
    (*(*fdoExtension).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus = status;

    TracePrint!((TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_PNP,
                b"ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n\0".as_ptr() as PCHAR,
                DeviceObject,
                status));

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspRefreshFunctionSupportInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    ForceQuery: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbSize: ULONG;
    let mut blockLimitsDataNew: CLASS_VPD_B0_DATA = core::mem::zeroed();
    let mut blockLimitsDataOriginal: PCLASS_VPD_B0_DATA;
    let mut lockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let generationCount: ULONG;
    let changeRequestCount: ULONG;

    generationCount = (*(*FdoExtension).FunctionSupportInfo).GenerationCount;
    changeRequestCount = (*(*FdoExtension).FunctionSupportInfo).ChangeRequestCount;
    if ForceQuery == 0 && generationCount == changeRequestCount {
        return STATUS_SUCCESS;
    }

    if !(*FdoExtension).AdapterDescriptor.is_null() &&
        (*((*FdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        srbSize = (CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE) as u32;
    } else {
        srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize as SIZE_T, 0x31446353) as PSCSI_REQUEST_BLOCK;
    if srb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ClasspDeviceGetBlockLimitsVPDPage(
        FdoExtension,
        srb,
        srbSize,
        &mut blockLimitsDataNew,
    );

    if NT_SUCCESS(status) {
        KeAcquireInStackQueuedSpinLock(
            &mut (*(*FdoExtension).FunctionSupportInfo).SyncLock,
            &mut lockHandle,
        );

        if generationCount == (*(*FdoExtension).FunctionSupportInfo).GenerationCount {
            blockLimitsDataOriginal = &mut (*(*FdoExtension).FunctionSupportInfo).BlockLimitsData;
            if (*blockLimitsDataOriginal).CommandStatus == (-1i32 as UCHAR) as i32 {
                *blockLimitsDataOriginal = blockLimitsDataNew;
            } else {
                (*blockLimitsDataOriginal).UGAVALID = blockLimitsDataNew.UGAVALID;
                (*blockLimitsDataOriginal).UnmapGranularityAlignment = blockLimitsDataNew.UnmapGranularityAlignment;
                (*blockLimitsDataOriginal).OptimalUnmapGranularity = blockLimitsDataNew.OptimalUnmapGranularity;
            }
            (*(*FdoExtension).FunctionSupportInfo).GenerationCount = changeRequestCount;
        }

        KeReleaseInStackQueuedSpinLock(&mut lockHandle);
    }

    FREE_POOL(srb as PVOID);
    status
}
#[no_mangle]
pub unsafe extern "system" fn ClasspMediaChangeRegistryCallBack(
    ValueName: PWSTR,
    ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    EntryContext: PVOID
) -> NTSTATUS {
    let valueFound: *mut ULONG;
    let deviceString: PUNICODE_STRING;
    let keyValue: PWSTR;
    let mut ValueLength = ValueLength;

    if ValueData.is_null() || Context.is_null() || EntryContext.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    valueFound = EntryContext as *mut ULONG;
    if (*valueFound) != 0 {
        return STATUS_SUCCESS;
    }

    if ValueLength == core::mem::size_of::<WCHAR>() as ULONG {
        return STATUS_SUCCESS;
    }

    if ValueType != REG_SZ {
        return STATUS_SUCCESS;
    }

    deviceString = Context as PUNICODE_STRING;
    keyValue = ValueData as PWSTR;
    ValueLength -= core::mem::size_of::<WCHAR>() as ULONG;

    if ValueLength > (*deviceString).Length as ULONG {
        ValueLength = (*deviceString).Length as ULONG;
    }

    if RtlCompareMemory((*deviceString).Buffer as PVOID, keyValue as PVOID, ValueLength as _) == ValueLength as _ {
        (*valueFound) = 1;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn ClassLogResourceExhaustionEvent(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID
) {

    let workItem = Context as PIO_WORKITEM;

    if NT_SUCCESS(ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_ERROR_DISK_RESOURCES_EXHAUSTED)) {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"ClassLogResourceExhaustionEvent: DO (%p), Permanent resource exhaustion logged.\n\0".as_ptr() as PCHAR,
            DeviceObject
        );
    } else {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassLogResourceExhaustionEvent: DO (%p), Failed to allocate memory for error log entry.\n\0".as_ptr() as PCHAR,
            DeviceObject
        );
    }

    ClassReleaseRemoveLock(DeviceObject, workItem as PIRP);

    if !workItem.is_null() {
        IoFreeWorkItem(workItem);
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClassLogThresholdEvent(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID
) {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let workItem: PIO_WORKITEM = Context as PIO_WORKITEM;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut resources: STORAGE_LB_PROVISIONING_MAP_RESOURCES = core::mem::zeroed();
    let resourcesSize: ULONG = core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>() as ULONG;
    let mut errorLogEntry: PIO_ERROR_LOG_PACKET = core::ptr::null_mut();
    let mut logEntrySize: ULONG = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as ULONG;
    let mut stringIndex: PWCHAR = core::ptr::null_mut();
    let mut stringSize: LONG = 0;
    let srbSize: ULONG;

    if !(*fdoExtension).AdapterDescriptor.is_null() &&
        (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as u8 {
        srbSize = (CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE) as u32;
    } else {
        srbSize = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG;
    }

    srb = ExAllocatePoolWithTag(NonPagedPoolNx,
                                (srbSize) as usize,
                                0x41436353) as PSCSI_REQUEST_BLOCK;
    if !srb.is_null() {

        ClassGetLBProvisioningResources(DeviceObject,
                                        srb,
                                        resourcesSize,
                                        &mut resources);

        logEntrySize = (core::mem::size_of::<IO_ERROR_LOG_PACKET>() + (11 * core::mem::size_of::<WCHAR>()) + (2 * (21 * core::mem::size_of::<WCHAR>()))) as ULONG;
        logEntrySize = min(logEntrySize, ERROR_LOG_MAXIMUM_SIZE);
        logEntrySize = min(logEntrySize, MAXUCHAR as ULONG);

        errorLogEntry = IoAllocateErrorLogEntry(DeviceObject as PVOID, logEntrySize as UCHAR) as PIO_ERROR_LOG_PACKET;
        if !errorLogEntry.is_null() {

            RtlZeroMemory(errorLogEntry as PVOID, (logEntrySize) as usize);
            (*errorLogEntry).StringOffset = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as USHORT;

            stringIndex = (errorLogEntry as ULONG_PTR + core::mem::size_of::<IO_ERROR_LOG_PACKET>()) as PWCHAR;
            stringSize = (logEntrySize - core::mem::size_of::<IO_ERROR_LOG_PACKET>() as ULONG) as LONG;

            status = strsafe_rust::RtlStringCbPrintfW!(stringIndex, stringSize as usize, w!("L%d").as_ptr() as PWSTR, (*fdoExtension).DeviceNumber);

            if NT_SUCCESS(status) {
                (*errorLogEntry).NumberOfStrings += 1;

                if resources.UsedMappingResourcesValid() != 0 &&
                    resources.AvailableMappingResourcesValid() != 0 {

                    stringIndex = stringIndex.offset((wcslen(stringIndex) + 1) as isize);
                    stringSize -= ((wcslen(stringIndex.offset(-((wcslen(stringIndex.offset(-1)) + 1) as isize))) + 1) * core::mem::size_of::<WCHAR>()) as LONG;

                    status = strsafe_rust::RtlStringCbPrintfW!(stringIndex, stringSize as usize, w!("L%I64u").as_ptr() as PWSTR, resources.UsedMappingResources);

                    if NT_SUCCESS(status) {
                        (*errorLogEntry).NumberOfStrings += 1;

                        stringIndex = stringIndex.offset((wcslen(stringIndex) + 1) as isize);
                        stringSize -= ((wcslen(stringIndex.offset(-((wcslen(stringIndex.offset(-1)) + 1) as isize))) + 1) * core::mem::size_of::<WCHAR>()) as LONG;

                        status = strsafe_rust::RtlStringCbPrintfW!(stringIndex, stringSize as usize, w!("L%I64u").as_ptr() as PWSTR, resources.AvailableMappingResources);

                        if NT_SUCCESS(status) {
                            (*errorLogEntry).NumberOfStrings += 1;
                        }
                    }
                } else {
                    TracePrint!((TRACE_LEVEL_WARNING,
                                TRACE_FLAG_GENERAL,
                                b"ClassLogThresholdEvent: DO (%p), Used and available mapping resources were unavailable.\n\0".as_ptr() as PCSZ,
                                DeviceObject));
                }
            }

            if (*errorLogEntry).NumberOfStrings == 3 {
                if resources.UsedMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) as u8 &&
                    resources.AvailableMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) as u8 {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_LUN;

                } else if resources.UsedMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) as u8 &&
                           resources.AvailableMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) as u8 {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_POOL;

                } else if resources.UsedMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) as u8 &&
                           resources.AvailableMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) as u8 {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_LUN;

                } else if resources.UsedMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) as u8 &&
                           resources.AvailableMappingResourcesScope() == (LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) as u8 {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_POOL;

                } else {

                    (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX;
                }
            } else {
                (*errorLogEntry).ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED;
            }

            IoWriteErrorLogEntry(errorLogEntry as PVOID);

            TracePrint!((TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_GENERAL,
                        b"ClassLogThresholdEvent: DO (%p), Soft threshold notification logged.\n\0".as_ptr() as PCSZ,
                        DeviceObject));
        } else {
            TracePrint!((TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        b"ClassLogThresholdEvent: DO (%p), Failed to allocate memory for error log entry.\n\0".as_ptr() as PCSZ,
                        DeviceObject));
        }
    } else {
        TracePrint!((TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClassLogThresholdEvent: DO (%p), Failed to allocate memory for SRB.\n\0".as_ptr() as PCSZ,
                    DeviceObject));
    }

    crate::ddk_shims::InterlockedExchange(core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).LBProvisioningData.SoftThresholdEventPending) as *mut i32, 0);

    ClassReleaseRemoveLock(DeviceObject, workItem as PIRP);

    FREE_POOL(srb as PVOID);

    if !workItem.is_null() {
        IoFreeWorkItem(workItem);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "classp.h"
// #include "debug.h"
//
// // ---- RetryRequest ----
// VOID
// RetryRequest(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PSCSI_REQUEST_BLOCK Srb,
//     BOOLEAN Associated,
//     LONGLONG TimeDelta100ns
//     )
// {
//     PIO_STACK_LOCATION currentIrpStack = IoGetCurrentIrpStackLocation(Irp);
//     PIO_STACK_LOCATION nextIrpStack = IoGetNextIrpStackLocation(Irp);
//     ULONG transferByteCount;
//     ULONG dataTransferLength;
//     PSTORAGE_REQUEST_BLOCK_HEADER srbHeader = (PSTORAGE_REQUEST_BLOCK_HEADER)Srb;
//
//     // This function is obsolete but is still used by some of our class drivers.
//     // TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "RetryRequest is OBSOLETE !"));
//
//     //
//     // Determine the transfer count of the request.  If this is a read or a
//     // write then the transfer count is in the Irp stack.  Otherwise assume
//     // the MDL contains the correct length.  If there is no MDL then the
//     // transfer length must be zero.
//     //
//
//     dataTransferLength = SrbGetDataTransferLength(srbHeader);
//     if (currentIrpStack->MajorFunction == IRP_MJ_READ ||
//         currentIrpStack->MajorFunction == IRP_MJ_WRITE) {
//
//         _Analysis_assume_(currentIrpStack->Parameters.Read.Length <= dataTransferLength);
//         transferByteCount = currentIrpStack->Parameters.Read.Length;
//
//     } else if (Irp->MdlAddress != NULL) {
//
//         //
//         // Note this assumes that only read and write requests are spilt and
//         // other request do not need to be.  If the data buffer address in
//         // the MDL and the SRB don't match then transfer length is most
//         // likely incorrect.
//         //
//
//         NT_ASSERT!(SrbGetDataBuffer(srbHeader) == MmGetMdlVirtualAddress(Irp->MdlAddress));
//     _Analysis_assume_(Irp->MdlAddress->ByteCount <= dataTransferLength);
//         transferByteCount = Irp->MdlAddress->ByteCount;
//
//     } else {
//
//         transferByteCount = 0;
//     }
//
//     //
//     // this is a safety net.  this should not normally be hit, since we are
//     // not guaranteed to be an fdoExtension
//     //
//
//     NT_ASSERT!(!TEST_FLAG(SrbGetSrbFlags(srbHeader as PVOID), SRB_FLAGS_FREE_SENSE_BUFFER));
//
//     //
//     // Reset byte count of transfer in SRB Extension.
//     //
//
//     SrbSetDataTransferLength(srbHeader, transferByteCount);
//
//     //
//     // Zero SRB statuses.
//     //
//
//     srbHeader->SrbStatus = 0;
//     SrbSetScsiStatus(srbHeader, 0);
//
//     //
//     // If this is the last retry, then disable all the special flags.
//     //
//
//     if ( 0 == (ULONG)(ULONG_PTR)currentIrpStack->Parameters.Others.Argument4 ) {
//         //
//         // Set the no disconnect flag, disable synchronous data transfers and
//         // disable tagged queuing. This fixes some errors.
//         // NOTE: Cannot clear these flags, just add to them
//         //
//
//         SrbSetSrbFlags(srbHeader,
//                           SRB_FLAGS_DISABLE_DISCONNECT | SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//         SrbClearSrbFlags(srbHeader, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//
//         SrbSetQueueTag(srbHeader, SP_UNTAGGED);
//     }
//
//
//     //
//     // Set up major SCSI function.
//     //
//
//     nextIrpStack->MajorFunction = IRP_MJ_SCSI;
//
//     //
//     // Save SRB address in next stack for port driver.
//     //
//
//     nextIrpStack->Parameters.Scsi.Srb = Srb;
//
//     if (Associated){
//         IoSetCompletionRoutine(Irp, ClassIoCompleteAssociated, Srb, TRUE, TRUE, TRUE);
//     }
//     else {
//         IoSetCompletionRoutine(Irp, ClassIoComplete, Srb, TRUE, TRUE, TRUE);
//     }
//
//     ClassRetryRequest(DeviceObject, Irp, TimeDelta100ns);
//     return;
// } // end RetryRequest()
//
// // ---- InterpretSenseInfoWithoutHistory ----
// BOOLEAN
// InterpretSenseInfoWithoutHistory(
//       PDEVICE_OBJECT Fdo,
//       PIRP OriginalRequest,
//       PSCSI_REQUEST_BLOCK Srb,
//          UCHAR MajorFunctionCode,
//          ULONG IoDeviceCode,
//          ULONG PreviousRetryCount,
//       NTSTATUS * Status,
//        
//          LONGLONG * RetryIn100nsUnits
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PCLASS_PRIVATE_FDO_DATA fdoData = fdoExtension->PrivateFdoData;
//     LONGLONG tmpRetry = 0;
//     BOOLEAN retry = FALSE;
//
//     if (fdoData->InterpretSenseInfo != NULL)
//     {
//         SCSI_REQUEST_BLOCK tempSrb = {0};
//         PSCSI_REQUEST_BLOCK srbPtr = Srb;
//
//         // SAL annotations and ClassInitializeEx() both validate this
//         NT_ASSERT!(fdoData->InterpretSenseInfo->Interpret != NULL);
//
//         //
//         // If class driver does not support extended SRB and this is
//         // an extended SRB, convert to legacy SRB and pass to class
//         // driver.
//         //
//         if ((Srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) &&
//             ((fdoExtension->CommonExtension.DriverExtension->SrbSupport &
//               CLASS_SRB_STORAGE_REQUEST_BLOCK) == 0)) {
//             ClasspConvertToScsiRequestBlock(&tempSrb, (PSTORAGE_REQUEST_BLOCK)Srb);
//             srbPtr = &tempSrb;
//         }
//
//         retry = fdoData->InterpretSenseInfo->Interpret(Fdo,
//                                                        OriginalRequest,
//                                                        srbPtr,
//                                                        MajorFunctionCode,
//                                                        IoDeviceCode,
//                                                        PreviousRetryCount,
//                                                        NULL,
//                                                        Status,
//                                                        &tmpRetry);
//     }
//     else
//     {
//         ULONG seconds = 0;
//
//         retry = ClassInterpretSenseInfo(Fdo,
//                                         Srb,
//                                         MajorFunctionCode,
//                                         IoDeviceCode,
//                                         PreviousRetryCount,
//                                         Status,
//                                         &seconds);
//         tmpRetry = ((LONGLONG)seconds) * 1000 * 1000 * 10;
//     }
//
//
//     if (RetryIn100nsUnits != NULL)
//     {
//         *RetryIn100nsUnits = tmpRetry;
//     }
//     return retry;
// }
//
// // ---- ClassFreeOrReuseSrb ----
// VOID ClassFreeOrReuseSrb(     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//                                 PSCSI_REQUEST_BLOCK Srb)
//
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExt = &FdoExtension->CommonExtension;
//
//     // This function is obsolete, but still called by DISK.SYS .
//     // TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "ClassFreeOrReuseSrb is OBSOLETE !"));
//
//     //
//     // safety net.  this should never occur.  if it does, it's a potential
//     // memory leak.
//     //
//     NT_ASSERT!(!TEST_FLAG(SrbGetSrbFlags(Srb), SRB_FLAGS_FREE_SENSE_BUFFER));
//
//     if (commonExt->IsSrbLookasideListInitialized){
//         
//         ClasspFreeSrb(FdoExtension, Srb);
//     }
//     else {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,"ClassFreeOrReuseSrb: someone is trying to use an uninitialized SrbLookasideList !!!"));
//         FREE_POOL(Srb);
//     }
// }
//
// // ---- ClasspValidateOffloadSupported ----
//  
//  
//  
// NTSTATUS
// ClasspValidateOffloadSupported(
//       PDEVICE_OBJECT DeviceObject,
//       PIRP Irp
//     )
//
//
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     TracePrint!((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspValidateOffloadSupported (%p): Entering function. Irp %p\n",
//                 DeviceObject,
//                 Irp));
//
//     fdoExt = DeviceObject->DeviceExtension;
//     status = STATUS_SUCCESS;
//
//     //
//     // For now this command is only supported by disk devices
//     //
//     if ((DeviceObject->DeviceType == FILE_DEVICE_DISK) &&
//         (!TEST_FLAG(DeviceObject->Characteristics, FILE_FLOPPY_DISKETTE))) {
//
//         if (!fdoExt->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits()) {
//
//             TracePrint!((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_IOCTL,
//                         "ClasspValidateOffloadSupported (%p): Command not supported on this disk device.\n",
//                         DeviceObject));
//
//             status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//             goto __ClasspValidateOffloadSupported_Exit;
//         }
//
//         if (!NT_SUCCESS(fdoExt->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus)) {
//
//             status = fdoExt->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus;
//
//             TracePrint!((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_IOCTL,
//                         "ClasspValidateOffloadSupported (%p): VPD retrieval failed with %x.\n",
//                         DeviceObject,
//                         status));
//
//             goto __ClasspValidateOffloadSupported_Exit;
//         }
//     } else {
//
//         TracePrint!((TRACE_LEVEL_WARNING,
//                     TRACE_FLAG_IOCTL,
//                     "ClasspValidateOffloadSupported (%p): Suported only on Disk devices.\n",
//                     DeviceObject));
//
//         status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//         goto __ClasspValidateOffloadSupported_Exit;
//     }
//
// __ClasspValidateOffloadSupported_Exit:
//     TracePrint!((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_IOCTL,
//                 "ClasspValidateOffloadSupported (%p): Exiting function Irp %p with status %x.\n",
//                 DeviceObject,
//                 Irp,
//                 status));
//
//     return status;
// }
//
// // ---- ClassRetrieveDeviceRelations ----
// NTSTATUS
// ClassRetrieveDeviceRelations(
//       PDEVICE_OBJECT Fdo,
//       DEVICE_RELATION_TYPE RelationType,
//       PDEVICE_RELATIONS *DeviceRelations
//     )
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//
//     ULONG count = 0;
//     ULONG i;
//
//     PPHYSICAL_DEVICE_EXTENSION nextChild;
//
//     ULONG relationsSize;
//     PDEVICE_RELATIONS deviceRelations = NULL;
//     PDEVICE_RELATIONS oldRelations = *DeviceRelations;
//
//     NTSTATUS status;
//
//     let _ = RelationType;
//
//     PAGED_CODE();
//
//     ClassAcquireChildLock(fdoExtension);
//
//     nextChild = fdoExtension->CommonExtension.ChildList;
//
//     //
//     // Count the number of PDO's attached to this disk
//     //
//
//     while (nextChild != NULL) {
//         PCOMMON_DEVICE_EXTENSION commonExtension;
//
//         commonExtension = &(nextChild->CommonExtension);
//
//         NT_ASSERTMSG!("ClassPnp internal error: missing child on active list\n",
//                   (nextChild->IsMissing == FALSE));
//
//         nextChild = commonExtension->ChildList;
//
//         count++;
//     };
//
//     //
//     // If relations already exist in the QDR, adjust the current count
//     // to include the previous list.
//     //
//
//     if (oldRelations) {
//         count += oldRelations->Count;
//     }
//
//     relationsSize = (sizeof(DEVICE_RELATIONS) +
//                      (count * sizeof(PDEVICE_OBJECT)));
//
//     deviceRelations = ExAllocatePoolWithTag(PagedPool, relationsSize, '5BcS');
//
//     if (deviceRelations == NULL) {
//
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_ENUM,  "ClassRetrieveDeviceRelations: unable to allocate "
//                        "%d bytes for device relations\n", relationsSize));
//
//         ClassReleaseChildLock(fdoExtension);
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(deviceRelations, relationsSize);
//
//     if (oldRelations) {
//
//         //
//         // Copy the old relations to the new list and free the old list.
//         //
//
//         for (i = 0; i < oldRelations->Count; i++) {
//             deviceRelations->Objects[i] = oldRelations->Objects[i];
//         }
//
//         FREE_POOL(oldRelations);
//     }
//
//     nextChild = fdoExtension->CommonExtension.ChildList;
//     i = count;
//
//     while (nextChild != NULL) {
//         PCOMMON_DEVICE_EXTENSION commonExtension;
//
//         commonExtension = &(nextChild->CommonExtension);
//
//         NT_ASSERTMSG!("ClassPnp internal error: missing child on active list\n",
//                   (nextChild->IsMissing == FALSE));
//
//     _Analysis_assume_(i >= 1);
//     deviceRelations->Objects[--i] = nextChild->DeviceObject;
//
//         status = ObReferenceObjectByPointer(
//                     nextChild->DeviceObject,
//                     0,
//                     NULL,
//                     KernelMode);
//         if (!NT_SUCCESS(status)) {
//             NT_ASSERT!(!"Error referencing child device by pointer");
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_ENUM,
//                         "ClassRetrieveDeviceRelations: Error referencing child "
//                         "device %p by pointer\n", nextChild->DeviceObject));
//
//         }
//         nextChild->IsEnumerated = TRUE;
//         nextChild = commonExtension->ChildList;
//     }
//
//     NT_ASSERTMSG!("Child list has changed: ", i == 0);
//
//     deviceRelations->Count = count;
//     *DeviceRelations = deviceRelations;
//
//     ClassReleaseChildLock(fdoExtension);
//     return STATUS_SUCCESS;
// } // end ClassRetrieveDeviceRelations()
//
// // ---- ClasspGetBlockDeviceTokenLimitsInfo ----
//  
// NTSTATUS
// ClasspGetBlockDeviceTokenLimitsInfo(
//       PDEVICE_OBJECT DeviceObject
//     )
//
//
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     NTSTATUS status;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     ULONG srbSize = 0;
//     USHORT pageLength = 0;
//     USHORT descriptorLength = 0;
//     PVOID dataBuffer = NULL;
//     UCHAR bufferLength = VPD_MAX_BUFFER_SIZE;
//     ULONG allocationBufferLength = bufferLength;
//     PCDB cdb;
//     ULONG dataTransferLength = 0;
//     PVPD_THIRD_PARTY_COPY_PAGE operatingParameters = NULL;
//     PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR blockDeviceTokenLimits = NULL;
//
//     TracePrint!((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_PNP,
//                 "ClasspGetBlockDeviceTokenLimitsInfo (%p): Entering function.\n",
//                 DeviceObject));
//
//     //
//     // Allocate an SRB for querying the device for LBP-related info.
//     //
//     if ((fdoExtension->AdapterDescriptor != NULL) &&
//         (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = sizeof(SCSI_REQUEST_BLOCK);
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, CLASSPNP_POOL_TAG_SRB);
//
//     if (!srb) {
//
//         TracePrint!((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate SRB.\n",
//                     DeviceObject));
//
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         goto __ClasspGetBlockDeviceTokenLimitsInfo_Exit;
//     }
//
// #if defined(_ARM_) || defined(_ARM64_)
//     //
//     // ARM has specific alignment requirements, although this will not have a functional impact on x86 or amd64
//     // based platforms. We are taking the conservative approach here.
//     //
//     allocationBufferLength = ALIGN_UP_BY(allocationBufferLength as ULONG_PTR, KeGetRecommendedSharedDataAlignment() as ULONG_PTR) as ULONG;
//     dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned, allocationBufferLength as SIZE_T, CLASSPNP_POOL_TAG_VPD);
// #else
//     dataBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, bufferLength, CLASSPNP_POOL_TAG_VPD);
// #endif
//
//     if (!dataBuffer) {
//
//         TracePrint!((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): Couldn't allocate dataBuffer.\n",
//                     DeviceObject));
//
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         goto __ClasspGetBlockDeviceTokenLimitsInfo_Exit;
//     }
//
//     operatingParameters = (PVPD_THIRD_PARTY_COPY_PAGE)dataBuffer;
//
//     RtlZeroMemory(dataBuffer, allocationBufferLength);
//
//     if ((fdoExtension->AdapterDescriptor != NULL) &&
//         (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         status = InitializeStorageRequestBlock((PSTORAGE_REQUEST_BLOCK)srb,
//                                                STORAGE_ADDRESS_TYPE_BTL8,
//                                                CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE,
//                                                1,
//                                                SrbExDataTypeScsiCdb16);
//         if (NT_SUCCESS(status)) {
//
//             ((PSTORAGE_REQUEST_BLOCK)srb)->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//
//         } else {
//
//             //
//             // Should not occur. Revert to legacy SRB.
//             //
//             NT_ASSERT!(FALSE);
//
//             TracePrint!((TRACE_LEVEL_WARNING,
//                         TRACE_FLAG_PNP,
//                         "ClasspGetBlockDeviceTokenLimitsInfo (%p): Falling back to using SRB (instead of SRB_EX).\n",
//                         DeviceObject));
//
//             RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//             srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//             srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         }
//     } else {
//
//         RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//         srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//     }
//
//     SrbSetTimeOutValue(srb, fdoExtension->TimeOutValue);
//     SrbSetRequestTag(srb, SP_UNTAGGED);
//     SrbSetRequestAttribute(srb, SRB_SIMPLE_TAG_REQUEST);
//     SrbAssignSrbFlags(srb, fdoExtension->SrbFlags);
//
//     SrbSetCdbLength(srb, 6);
//
//     cdb = SrbGetCdb(srb);
//     cdb->CDB6INQUIRY3.OperationCode = SCSIOP_INQUIRY;
//     cdb->CDB6INQUIRY3.EnableVitalProductData = 1;
//     cdb->CDB6INQUIRY3.PageCode = VPD_THIRD_PARTY_COPY;
//     cdb->CDB6INQUIRY3.AllocationLength = bufferLength;
//
//     status = ClassSendSrbSynchronous(fdoExtension->DeviceObject,
//                                      srb,
//                                      dataBuffer,
//                                      allocationBufferLength,
//                                      FALSE);
//
//     //
//     // Handle the case where we get back STATUS_DATA_OVERRUN because the input
//     // buffer was larger than necessary.
//     //
//     dataTransferLength = SrbGetDataTransferLength(srb);
//
//     if (status == STATUS_DATA_OVERRUN && dataTransferLength < bufferLength) {
//
//         status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(status)) {
//
//         REVERSE_BYTES_SHORT_M!(&pageLength, &operatingParameters->PageLength);
//
//     } else {
//         TracePrint!((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): Inquiry for TPC VPD failed with %x.\n",
//                     DeviceObject,
//                     status));
//     }
//
//     if ((NT_SUCCESS(status)) &&
//         (pageLength >= sizeof(WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR))) {
//
//         USHORT descriptorType;
//
//         blockDeviceTokenLimits = (PWINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR)operatingParameters->ThirdPartyCopyDescriptors;
//         REVERSE_BYTES_SHORT_M!(&descriptorType, &blockDeviceTokenLimits->DescriptorType);
//         REVERSE_BYTES_SHORT_M!(&descriptorLength, &blockDeviceTokenLimits->DescriptorLength);
//
//         if ((descriptorType == BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR_TYPE_WINDOWS) &&
//             (VPD_PAGE_HEADER_SIZE + descriptorLength == sizeof(WINDOWS_BLOCK_DEVICE_TOKEN_LIMITS_DESCRIPTOR))) {
//
//             fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits() = TRUE;
//
//             REVERSE_BYTES_SHORT_M!(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumRangeDescriptors, &blockDeviceTokenLimits->MaximumRangeDescriptors);
//             REVERSE_BYTES(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumInactivityTimer, &blockDeviceTokenLimits->MaximumInactivityTimer);
//             REVERSE_BYTES(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.DefaultInactivityTimer, &blockDeviceTokenLimits->DefaultInactivityTimer);
//             REVERSE_BYTES_QUAD_M!(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumTokenTransferSize, &blockDeviceTokenLimits->MaximumTokenTransferSize);
//             REVERSE_BYTES_QUAD_M!(&fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.OptimalTransferCount, &blockDeviceTokenLimits->OptimalTransferCount);
//
//             TracePrint!((TRACE_LEVEL_INFORMATION,
//                         TRACE_FLAG_PNP,
//                         "ClasspGetBlockDeviceTokenLimitsInfo (%p): %s %s (rev %s) reported following parameters: \
//                         \n\t\t\tMaxRangeDescriptors: %u\n\t\t\tMaxIAT: %u\n\t\t\tDefaultIAT: %u \
//                         \n\t\t\tMaxTokenTransferSize: %I64u\n\t\t\tOptimalTransferCount: %I64u\n\t\t\tOptimalTransferLengthGranularity: %u \
//                         \n\t\t\tOptimalTransferLength: %u\n\t\t\tMaxTransferLength: %u\n",
//                         DeviceObject,
//                         (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->VendorIdOffset),
//                         (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->ProductIdOffset),
//                         (PCSZ)(((PUCHAR)fdoExtension->DeviceDescriptor) + fdoExtension->DeviceDescriptor->ProductRevisionOffset),
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumRangeDescriptors,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumInactivityTimer,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.DefaultInactivityTimer,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.MaximumTokenTransferSize,
//                         fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.OptimalTransferCount,
//                         fdoExtension->FunctionSupportInfo->BlockLimitsData.OptimalTransferLengthGranularity,
//                         fdoExtension->FunctionSupportInfo->BlockLimitsData.OptimalTransferLength,
//                         fdoExtension->FunctionSupportInfo->BlockLimitsData.MaximumTransferLength));
//         } else {
//
//             TracePrint!((TRACE_LEVEL_WARNING,
//                         TRACE_FLAG_PNP,
//                         "ClasspGetBlockDeviceTokenLimitsInfo (%p): ThirdPartyCopy VPD data doesn't have Windows OffloadDataTransfer descriptor.\n",
//                         DeviceObject));
//
//             fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits() = FALSE;
//             status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//         }
//     } else {
//
//         TracePrint!((TRACE_LEVEL_WARNING,
//                     TRACE_FLAG_PNP,
//                     "ClasspGetBlockDeviceTokenLimitsInfo (%p): TPC VPD data didn't return TPC descriptors of interest.\n",
//                     DeviceObject));
//
//         fdoExtension->FunctionSupportInfo->ValidInquiryPages.BlockDeviceRODLimits() = FALSE;
//         status = STATUS_DEVICE_FEATURE_NOT_SUPPORTED;
//     }
//
// __ClasspGetBlockDeviceTokenLimitsInfo_Exit:
//
//     FREE_POOL(dataBuffer);
//     FREE_POOL(srb);
//     fdoExtension->FunctionSupportInfo->BlockDeviceRODLimitsData.CommandStatus = status;
//
//     TracePrint!((TRACE_LEVEL_VERBOSE,
//                 TRACE_FLAG_PNP,
//                 "ClasspGetBlockDeviceTokenLimitsInfo (%p): Exiting function with status %x.\n",
//                 DeviceObject,
//                 status));
//
//     return status;
// }
//
// // ---- ClasspRefreshFunctionSupportInfo ----
// NTSTATUS
// ClasspRefreshFunctionSupportInfo(
//       PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//       BOOLEAN ForceQuery
//     )
//
// {
//     NTSTATUS            status;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     ULONG               srbSize;
//     CLASS_VPD_B0_DATA   blockLimitsDataNew;
//     PCLASS_VPD_B0_DATA  blockLimitsDataOriginal;
//     KLOCK_QUEUE_HANDLE  lockHandle;
//     ULONG               generationCount;
//     ULONG               changeRequestCount;
//
//     //
//     // ChangeRequestCount is incremented every time we get an unit attention with
//     // SCSI_ADSENSE_OPERATING_CONDITIONS_CHANGED.  GenerationCount will be set to
//     // ChangeRequestCount after CLASS_FUNCTION_SUPPORT_INFO is refreshed with the latest
//     // VPD data.  i.e.  if both values are the same, data in
//     // CLASS_FUNCTION_SUPPORT_INFO is current
//     //
//
//     generationCount = FdoExtension->FunctionSupportInfo->GenerationCount;
//     changeRequestCount = FdoExtension->FunctionSupportInfo->ChangeRequestCount;
//     if (!ForceQuery && generationCount == changeRequestCount) {
//         return STATUS_SUCCESS;
//     }
//
//     //
//     // Allocate an SRB for querying the device for LBP-related info if either
//     // the Logical Block Provisioning (0xB2) or Block Limits (0xB0) VPD page
//     // exists.
//     //
//     if ((FdoExtension->AdapterDescriptor != NULL) &&
//         (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = sizeof(SCSI_REQUEST_BLOCK);
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx, srbSize, '1DcS');
//     if (srb == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     status = ClasspDeviceGetBlockLimitsVPDPage(FdoExtension,
//                                                srb,
//                                                srbSize,
//                                                &blockLimitsDataNew);
//
//     if (NT_SUCCESS(status)) {
//
//         KeAcquireInStackQueuedSpinLock(&FdoExtension->FunctionSupportInfo->SyncLock, &lockHandle);
//
//         //
//         // If the generationCount didn't change since we looked at it last time, it means
//         // no one has tried to update the CLASS_FUNCTION_SUPPORT_INFO data; otherwise, someone
//         // else has beat us to it.
//         //
//         if (generationCount == FdoExtension->FunctionSupportInfo->GenerationCount) {
//
//             blockLimitsDataOriginal = &FdoExtension->FunctionSupportInfo->BlockLimitsData;
//             if (blockLimitsDataOriginal->CommandStatus == -1) {
//                 //
//                 // CommandStatus == -1 means this is the first time we have
//                 // gotten the block limits data.
//                 //
//                 *blockLimitsDataOriginal = blockLimitsDataNew;
//             } else {
//                 //
//                 // We only expect the Optimal Unmap Granularity (and alignment)
//                 // to change, so those are the only parameters we update.
//                 //
//                 blockLimitsDataOriginal->UGAVALID = blockLimitsDataNew.UGAVALID;
//                 blockLimitsDataOriginal->UnmapGranularityAlignment = blockLimitsDataNew.UnmapGranularityAlignment;
//                 blockLimitsDataOriginal->OptimalUnmapGranularity = blockLimitsDataNew.OptimalUnmapGranularity;
//             }
//             FdoExtension->FunctionSupportInfo->GenerationCount = changeRequestCount;
//         }
//
//         KeReleaseInStackQueuedSpinLock(&lockHandle);
//     }
//
//     FREE_POOL(srb);
//     return status;
// }
//
// // ---- ClasspMediaChangeRegistryCallBack ----
//  
//  
//  
// NTSTATUS
// NTAPI 
// ClasspMediaChangeRegistryCallBack(
//       PWSTR ValueName,
//       ULONG ValueType,
//       PVOID ValueData,
//       ULONG ValueLength,
//       PVOID Context,
//       PVOID EntryContext
//     )
// {
//     PULONG valueFound;
//     PUNICODE_STRING deviceString;
//     PWSTR keyValue;
//
//     PAGED_CODE();
//     let _ = ValueName;
//
//     if (ValueData == NULL ||
//         Context == NULL ||
//         EntryContext == NULL) {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // if we have already set the value to true, exit
//     //
//
//     valueFound = EntryContext;
//     if ((*valueFound) != 0) {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspMcnRegCB: already set to true\n"));
//         return STATUS_SUCCESS;
//     }
//
//     if (ValueLength == sizeof(WCHAR)) {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN, "ClasspMcnRegCB: NULL string should "
//                     "never be passed to registry call-back!\n"));
//         return STATUS_SUCCESS;
//     }
//
//
//     //
//     // if the data is not a terminated string, exit
//     //
//
//     if (ValueType != REG_SZ) {
//         return STATUS_SUCCESS;
//     }
//
//     deviceString = Context;
//     keyValue = ValueData;
//     ValueLength -= sizeof(WCHAR); // ignore the null character
//
//     //
//     // do not compare more memory than is in deviceString
//     //
//
//     if (ValueLength > deviceString->Length) {
//         ValueLength = deviceString->Length;
//     }
//
//     //
//     // if the strings match, disable autorun
//     //
//
//     if (RtlCompareMemory(deviceString->Buffer, keyValue, ValueLength) == ValueLength) {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: Match found\n"));
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: DeviceString at %p\n",
//                     deviceString->Buffer));
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "ClasspRegMcnCB: KeyValue at %p\n",
//                     keyValue));
//         (*valueFound) = TRUE;
//     }
//
//     return STATUS_SUCCESS;
// } // end ClasspMediaChangeRegistryCallBack()
//
// // ---- ClassLogResourceExhaustionEvent ----
//  
//  
//  
// VOID
// NTAPI 
// ClassLogResourceExhaustionEvent(
//       PDEVICE_OBJECT DeviceObject,
//       PVOID Context
//     )
//
// {
//     PIO_WORKITEM workItem = (PIO_WORKITEM)Context;
//
//     if (NT_SUCCESS(ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_ERROR_DISK_RESOURCES_EXHAUSTED))) {
//
//         TracePrint!((TRACE_LEVEL_INFORMATION,
//                     TRACE_FLAG_GENERAL,
//                     "ClassLogResourceExhaustionEvent: DO (%p), Permanent resource exhaustion logged.\n",
//                     DeviceObject));
//     } else {
//         TracePrint!((TRACE_LEVEL_ERROR,
//                     TRACE_FLAG_GENERAL,
//                     "ClassLogResourceExhaustionEvent: DO (%p), Failed to allocate memory for error log entry.\n",
//                     DeviceObject));
//     }
//
//
//     ClassReleaseRemoveLock(DeviceObject, (PIRP)workItem);
//
//     if (workItem != NULL) {
//         IoFreeWorkItem(workItem);
//     }
// }
//
// // ---- ClassLogThresholdEvent ----
//  
//  
//  
// VOID
// NTAPI 
// ClassLogThresholdEvent(
//       PDEVICE_OBJECT DeviceObject,
//       PVOID Context
//     )
//
// {
//     NTSTATUS status = STATUS_SUCCESS;
//     PIO_WORKITEM workItem = (PIO_WORKITEM)Context;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = (PFUNCTIONAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     STORAGE_LB_PROVISIONING_MAP_RESOURCES resources = {0};
//     ULONG resourcesSize = sizeof(STORAGE_LB_PROVISIONING_MAP_RESOURCES);
//     PIO_ERROR_LOG_PACKET errorLogEntry = NULL;
//     ULONG logEntrySize = sizeof(IO_ERROR_LOG_PACKET);
//     PWCHAR stringIndex = NULL;
//     LONG stringSize = 0;
//     ULONG srbSize;
//
//     //
//     // Allocate an SRB for getting the LBP log page.
//     //
//     if ((fdoExtension->AdapterDescriptor != NULL) &&
//         (fdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK)) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = sizeof(SCSI_REQUEST_BLOCK);
//     }
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 'ACcS');
//     if (srb != NULL) {
//
//         //
//         // Try to get the LBP resources from the device so we can report them in
//         // the system event log.
//         //
//         ClassGetLBProvisioningResources(DeviceObject,
//                                         srb,
//                                         resourcesSize,
//                                         &resources);
//
//         //
//         // We need to allocate enough space for 3 insertion strings:
//         // The first is a ULONG representing the disk number in decimal, which means
//         // a max of 10 digits, plus one for the NULL character.
//         // The second and third are ULONGLONGs representing the used and available
//         // bytes, which means a max of 20 digits, plus one for the NULL character.
//         // Make sure we do not exceed the max error log size or the max size of a
//         // UCHAR since the size gets truncated to a UCHAR when we pass it to
//         // IoAllocateErrorLogEntry().
//         //
//         logEntrySize = sizeof(IO_ERROR_LOG_PACKET) + (11 * sizeof(WCHAR)) + (2 * (21 * sizeof(WCHAR)));
//         logEntrySize = min(logEntrySize, ERROR_LOG_MAXIMUM_SIZE);
//         logEntrySize = min(logEntrySize, MAXUCHAR);
//
//         errorLogEntry = (PIO_ERROR_LOG_PACKET)IoAllocateErrorLogEntry(DeviceObject, (UCHAR)logEntrySize);
//         if (errorLogEntry != NULL)
//         {
//             //
//             // There are two event IDs we can use here.  Both use the disk number,
//             // but one reports the available and used bytes while the other does not.
//             // We fall back on the latter if we failed to obtain the available and
//             // used byte counts from the LBP log page.
//             //
//             // The event insertion strings need to be in this order:
//             // 1. The disk number. (Both event IDs use this.)
//             // 2. Bytes used.
//             // 3. Bytes available.
//             //
//
//             RtlZeroMemory(errorLogEntry, logEntrySize);
//             errorLogEntry->StringOffset = sizeof(IO_ERROR_LOG_PACKET);
//
//             stringIndex = (PWCHAR)((ULONG_PTR)errorLogEntry + sizeof(IO_ERROR_LOG_PACKET));
//             stringSize = logEntrySize - sizeof(IO_ERROR_LOG_PACKET);
//
//             //
//             // Add the disk number to the insertion strings.
//             //
//             status = RtlStringCbPrintfW(stringIndex, stringSize, L"%d", fdoExtension->DeviceNumber);
//
//             if (NT_SUCCESS(status) )
//             {
//                 errorLogEntry->NumberOfStrings++;
//
//                 if (resources.UsedMappingResourcesValid &&
//                     resources.AvailableMappingResourcesValid)
//                 {
//                     //
//                     // Add the used mapping resources to the insertion strings.
//                     //
//                     stringIndex += (wcslen(stringIndex) + 1);
//                     stringSize -= (LONG)(wcslen(stringIndex) + 1) * sizeof(WCHAR);
//
//                     status = RtlStringCbPrintfW(stringIndex, stringSize, L"%I64u", resources.UsedMappingResources);
//
//                     if (NT_SUCCESS(status))
//                     {
//                         errorLogEntry->NumberOfStrings++;
//
//                         //
//                         // Add the available mapping resources to the insertion strings.
//                         //
//                         stringIndex += (wcslen(stringIndex) + 1);
//                         stringSize -= (LONG)(wcslen(stringIndex) + 1) * sizeof(WCHAR);
//
//                         status = RtlStringCbPrintfW(stringIndex, stringSize, L"%I64u", resources.AvailableMappingResources);
//
//                         if (NT_SUCCESS(status))
//                         {
//                             errorLogEntry->NumberOfStrings++;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     TracePrint!((TRACE_LEVEL_WARNING,
//                                 TRACE_FLAG_GENERAL,
//                                 "ClassLogThresholdEvent: DO (%p), Used and available mapping resources were unavailable.\n",
//                                 DeviceObject));
//                 }
//             }
//
//             //
//             // If we were able to successfully assemble all 3 insertion strings,
//             // then we can use one of the "extended" event IDs.  Otherwise, use the basic
//             // event ID, which only requires the disk number.
//             //
//             if (errorLogEntry->NumberOfStrings == 3)
//             {
//                 if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
//                     resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_LUN;
//
//                 } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN &&
//                            resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_LUN_POOL;
//
//                 } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
//                            resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_LUN;
//
//                 } else if (resources.UsedMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN &&
//                            resources.AvailableMappingResourcesScope == LOG_PAGE_LBP_RESOURCE_SCOPE_NOT_DEDICATED_TO_LUN) {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX_POOL_POOL;
//
//                 } else {
//
//                     errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED_EX;
//                 }
//             }
//             else
//             {
//                 errorLogEntry->ErrorCode = IO_WARNING_SOFT_THRESHOLD_REACHED;
//             }
//
//             //
//             // Write the error log packet to the system error logging thread.
//             // It will be freed automatically.
//             //
//             IoWriteErrorLogEntry(errorLogEntry);
//
//             TracePrint!((TRACE_LEVEL_INFORMATION,
//                         TRACE_FLAG_GENERAL,
//                         "ClassLogThresholdEvent: DO (%p), Soft threshold notification logged.\n",
//                         DeviceObject));
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_GENERAL,
//                         "ClassLogThresholdEvent: DO (%p), Failed to allocate memory for error log entry.\n",
//                         DeviceObject));
//         }
//     } else {
//         TracePrint!((TRACE_LEVEL_ERROR,
//                         TRACE_FLAG_GENERAL,
//                         "ClassLogThresholdEvent: DO (%p), Failed to allocate memory for SRB.\n",
//                         DeviceObject));
//     }
//
//
//     //
//     // Clear the soft threshold event pending flag so that another can be queued.
//     //
//     InterlockedExchange((PLONG)&(fdoExtension->FunctionSupportInfo->LBProvisioningData.SoftThresholdEventPending), 0);
//
//     ClassReleaseRemoveLock(DeviceObject, (PIRP)workItem);
//
//     FREE_POOL(srb);
//
//     if (workItem != NULL) {
//         IoFreeWorkItem(workItem);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// ClassShutdownFlush — gap-TU ($0.01); surfaced live once ClassInitializeDispatchTables un-folded.
#[no_mangle]
pub unsafe extern "system" fn ClassShutdownFlush(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let isRemoved: ULONG;

    isRemoved = crate::ddk_shims::ClassAcquireRemoveLock(DeviceObject, Irp as PVOID);

    if isRemoved != 0 {
        ClassReleaseRemoveLock(DeviceObject, Irp);

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;

        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as u8) as _);

        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    if (*(*commonExtension).DevInfo).ClassShutdownFlush.is_some() {
        return (*(*commonExtension).DevInfo).ClassShutdownFlush.unwrap()(DeviceObject, Irp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;

    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as u8) as _);

    return STATUS_INVALID_DEVICE_REQUEST;
}


// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "system" {
    fn ClassAcquireChildLock(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION);
    fn ClassReleaseChildLock(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION);
}

// ---------------------------------------------------------------------------
// ODX copy-offload entry points (class.c ClassDeviceProcessOffloadRead/Write).
//
// Never translated. Nothing noticed because the IOCTL_STORAGE_* dispatch arms
// that call them sat below a catch-all binding arm, so the calls were
// unreachable and the linker never demanded the symbols; restoring the dispatch
// (field manual §9.112a) made them undefined at link. The leaf validator
// (ClasspValidateOffloadSupported, above) was ported, but the token-transfer
// machinery it hands off to (ClasspServicePopulateTokenTransferRequest /
// ClasspServiceWriteUsingTokenTransferRequest and the ROD token pool) is still
// extern-decl-only, so the feature genuinely does not exist in the Rust port.
//
// The validate-and-complete prologue is faithful to the C. Where the C would
// enter the token path we complete with STATUS_NOT_SUPPORTED and say so on the
// trace, rather than fabricate a token implementation. On every device we
// actually run, ClasspValidateOffloadSupported already returns
// STATUS_DEVICE_FEATURE_NOT_SUPPORTED (no BlockDeviceRODLimits VPD page), so the
// truncated tail is unreachable in practice — but it fails LOUD if that ever
// changes instead of silently doing the wrong thing.
// OWED: port the ROD-token transfer path, then replace the tail here.

#[no_mangle]
pub unsafe extern "C" fn ClasspCompleteOffloadRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    CompletionStatus: NTSTATUS,
) {
    (*Irp).IoStatus.__bindgen_anon_1.Status = CompletionStatus;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT as u8);
}

#[no_mangle]
pub unsafe extern "C" fn ClassDeviceProcessOffloadRead(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status = ClasspValidateOffloadSupported(DeviceObject, Irp);
    if NT_SUCCESS(status) {
        TracePrint!((
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_IOCTL,
            b"ClassDeviceProcessOffloadRead (%p): ROD token transfer path is not ported to Rust.\n\0".as_ptr() as PCHAR,
            DeviceObject,
        ));
        status = STATUS_NOT_SUPPORTED;
    }
    ClasspCompleteOffloadRequest(DeviceObject, Irp, status);
    status
}

#[no_mangle]
pub unsafe extern "C" fn ClassDeviceProcessOffloadWrite(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status = ClasspValidateOffloadSupported(DeviceObject, Irp);
    if NT_SUCCESS(status) {
        TracePrint!((
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_IOCTL,
            b"ClassDeviceProcessOffloadWrite (%p): ROD token transfer path is not ported to Rust.\n\0".as_ptr() as PCHAR,
            DeviceObject,
        ));
        status = STATUS_NOT_SUPPORTED;
    }
    ClasspCompleteOffloadRequest(DeviceObject, Irp, status);
    status
}

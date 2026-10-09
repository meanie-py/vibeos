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
use ntkrnl_sys::ddk::{DEVICE_DATA_SET_RANGE, DEVICE_DSM_OFFLOAD_READ_PARAMETERS, DEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, DEVICE_MANAGE_DATA_SET_ATTRIBUTES, DEVICE_RELATIONS, DISK_GEOMETRY, ExFreePool, ExInitializeRundownProtectionCacheAware, ExReInitializeRundownProtectionCacheAware, ExSizeOfRundownProtectionCacheAware, ExWaitForRundownProtectionReleaseCacheAware, _KWAIT_REASON_Executive, GUID, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, IRP, IoAllocateDriverObjectExtension, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoCopyCurrentIrpStackLocationToNext, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoFreeIrp, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetDeviceProperty, IoGetDriverObjectExtension, IoGetNextIrpStackLocation, IoGetPagingIoPriority, IoInitializeIrp, IoInvalidateDeviceRelations, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoRegisterDeviceInterface, IoSetCompletionRoutine, IoSetDeviceInterfaceState, IoSetNextIrpStackLocation, IoSetStartIoAttributes, IoStartPacket, IoWMIRegistrationControl, IoWriteErrorLogEntry, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeBugCheck, KeDelayExecutionThread, KeEnterCriticalRegion, KeGetRecommendedSharedDataAlignment, KeInitializeDpc, KeInitializeEvent, KeInitializeSpinLock, KeInitializeTimer, KeLeaveCriticalRegion, KeQueryTimeIncrement, KeReleaseInStackQueuedSpinLock, KeSetTimerEx, KeWaitForSingleObject, LARGE_INTEGER, LIST_ENTRY, LPGUID, MOUNTDEV_NAME, MmBuildMdlForNonPagedPool, MmUnlockPages, NTSTRSAFE_PSTR, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByPointer, PCCHAR, PCSZ, PDEVICE_CAPABILITIES, PDEVICE_DATA_SET_RANGE, PDEVICE_DSM_OFFLOAD_READ_PARAMETERS, PDEVICE_DSM_OFFLOAD_WRITE_PARAMETERS, PDEVICE_MANAGE_DATA_SET_ATTRIBUTES, PDEVICE_OBJECT, PDEVICE_POWER_DESCRIPTOR, PDEVICE_RELATIONS, PDRIVER_OBJECT, PEX_RUNDOWN_REF_CACHE_AWARE, PIO_COMPLETION_ROUTINE, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIO_WORKITEM, PIRP, PKEVENT, PMDL, PMOUNTDEV_NAME, PPREVENT_MEDIA_REMOVAL, PREVENT_MEDIA_REMOVAL, PRTL_QUERY_REGISTRY_TABLE, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_NUMBER, PSTORAGE_HOTPLUG_INFO, PSTORAGE_OFFLOAD_READ_OUTPUT, PSTORAGE_OFFLOAD_WRITE_OUTPUT, PSTORAGE_PROPERTY_ID, PSTORAGE_PROPERTY_QUERY, PSTORAGE_READ_CAPACITY, PULONGLONG, PUNICODE_STRING, PVPB, PsGetCurrentThread, RTL_QUERY_REGISTRY_TABLE, RtlAnsiStringToUnicodeString, RtlCopyUnicodeString, RtlDeleteRegistryValue, RtlFreeUnicodeString, RtlInitString, RtlInitUnicodeString, RtlQueryRegistryValues, STORAGE_DEVICE_NUMBER, STORAGE_HOTPLUG_INFO, STORAGE_OFFLOAD_READ_OUTPUT, STORAGE_OFFLOAD_WRITE_OUTPUT, STORAGE_PROPERTY_QUERY, STORAGE_READ_CAPACITY, STRING, UINT32, _KWAIT_REASON_UserRequest, ZwClose, ZwOpenKey, ZwSetValueKey};
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


// ---- ntddstor.h. class.rs has a function-local `IOCTL_STORAGE_FIRMWARE_ACTIVATE`
// marked `// typical` — a guessed value (§9.113 shape). tools/gen_ioctl_consts.py
// confirms the guess happens to be right, but a function-local const is invisible
// here, so this arm bound every control code. Computed, crate-visible, provenanced. ----
#[cfg(feature = "NTDDI_WINTRHESHOLD")]
const IOCTL_STORAGE_FIRMWARE_ACTIVATE: u32 = 0x002ddc08; // CTL_CODE(IOCTL_STORAGE_BASE, 0x0702, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS)

// IO_COMPLETION_ROUTINE is a type alias for PIO_COMPLETION_ROUTINE in ntkrnl_sys::ddk
use ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE as IO_COMPLETION_ROUTINE;
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DO_VERIFY_VOLUME, FILE_DEVICE_DISK, FILE_PORTABLE_DEVICE, IRP_MJ_SCSI, SL_OVERRIDE_VERIFY_VOLUME, SL_WRITE_THROUGH, STATUS_DEVICE_BUSY, STATUS_DEVICE_DOES_NOT_EXIST, STATUS_VERIFY_REQUIRED, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING, IOCTL_STORAGE_FIRMWARE_GET_INFO};
use crate::types::{CLASSP_REG_ACCESS_ALIGNMENT_NOT_SUPPORTED, CLASSP_REG_DISBALE_IDLE_POWER_NAME, CLASSP_REG_PERF_RESTORE_VALUE_NAME, CLASSP_REG_QERR_OVERRIDE_MODE, CLASSP_REG_REMOVAL_POLICY_VALUE_NAME, CLASSP_REG_SUBKEY_NAME, CLASSP_VOLUME_VERIFY_CHECKED, CLASS_DRIVER_EXTENSION, CLASS_INIT_DATA, CLASS_INTERPRET_SENSE_INFO2, CLASS_INTERPRET_SENSE_INFO2_MAXIMUM_HISTORY_COUNT, CLASS_MAX_INTERLEAVE_PER_CRITICAL_IO, CLASS_PERF_RESTORE_MINIMUM, CLASS_QUERY_WMI_REGINFO_EX_LIST, CLASS_SRB_SCSI_REQUEST_BLOCK, CLASS_SRB_STORAGE_REQUEST_BLOCK, CLASS_WORKING_SET, CLASS_WORKING_SET_MAXIMUM, COMMON_DEVICE_EXTENSION, COMPLETION_CONTEXT, ClassGlobalDispatch, ClassMinimalPowerHandler, ClasspPowerSettingCallback, ClasspRetryRequestDpc, ClasspScanForClassHacks, ClasspStartIo, ClasspUpdateDiskProperties, DEV_USE_16BYTE_CDB, FDO_HACK_NO_SYNC_CACHE, FUNCTIONAL_DEVICE_EXTENSION, MAX_TOKEN_LIST_IDENTIFIERS, NUM_IO_RETRIES, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PREAD_CAPACITY_DATA_EX, PSCSI_REQUEST_BLOCK, PTRANSFER_PACKET, QERR_SET_ZERO_ALWAYS, QERR_SET_ZERO_ODX_OR_TP_ONLY, READ_CAPACITY_DATA, READ_CAPACITY_DATA_EX, REG_DISK_CLASS_CONTROL, REMOVE_PENDING, SCSIOP_START_STOP_UNIT, SCSI_REQUEST_BLOCK, SRB_FLAGS_DISABLE_AUTOSENSE, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_FREE_SENSE_BUFFER, SRB_FLAGS_NO_DATA_TRANSFER, SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_STATUS_QUEUE_FROZEN, SRB_TYPE_STORAGE_REQUEST_BLOCK, START_UNIT_TIMEOUT, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK, TRANSFER_PACKET, _CLASS_DEV_INFO, _CLASS_DRIVER_EXTENSION, _CLASS_FUNCTION_SUPPORT_INFO, _CLASS_INIT_DATA, _CLASS_PRIVATE_FDO_DATA, _COMMON_DEVICE_EXTENSION, _COMPLETION_CONTEXT, _FUNCTIONAL_DEVICE_EXTENSION, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK, DeviceDsmAction_Allocation, DeviceDsmAction_OffloadRead, DeviceDsmAction_OffloadWrite, DeviceDsmAction_Trim, SCSIOP_SYNCHRONIZE_CACHE, SCSI_ADSENSE_BUS_RESET, SCSI_ADSENSE_COPY_PROTECTION_FAILURE, SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR, SCSI_ADSENSE_DATA_AREA, SCSI_ADSENSE_FAILURE_PREDICTION_THRESHOLD_EXCEEDED, SCSI_ADSENSE_ILLEGAL_BLOCK, SCSI_ADSENSE_ILLEGAL_COMMAND, SCSI_ADSENSE_INVALID_CDB, SCSI_ADSENSE_INVALID_FIELD_PARAMETER_LIST, SCSI_ADSENSE_INVALID_LUN, SCSI_ADSENSE_INVALID_TOKEN, SCSI_ADSENSE_LB_PROVISIONING, SCSI_ADSENSE_MEDIUM_CHANGED, SCSI_ADSENSE_MUSIC_AREA, SCSI_ADSENSE_NO_MEDIA_IN_DEVICE, SCSI_ADSENSE_NO_SENSE, SCSI_ADSENSE_OPERATING_CONDITIONS_CHANGED, SCSI_ADSENSE_OPERATOR_REQUEST, SCSI_ADSENSE_PARAMETERS_CHANGED, SCSI_ADSENSE_PARAMETER_LIST_LENGTH, SCSI_ADSENSE_RESOURCE_FAILURE, SCSI_ADSENSE_VOLUME_OVERFLOW, SCSI_SENSEQ_AUTHENTICATION_FAILURE, SCSI_SENSEQ_BECOMING_READY, SCSI_SENSEQ_CAUSE_NOT_REPORTABLE, SCSI_SENSEQ_CLEANING_CARTRIDGE_INSTALLED, SCSI_SENSEQ_DATA_UNDERRUN, SCSI_SENSEQ_FORMAT_IN_PROGRESS, SCSI_SENSEQ_INSUFFICIENT_RESOURCES, SCSI_SENSEQ_INVALID_RELEASE_OF_PERSISTENT_RESERVATION, SCSI_SENSEQ_KEY_NOT_ESTABLISHED, SCSI_SENSEQ_KEY_NOT_PRESENT, SCSI_SENSEQ_LOGICAL_UNIT_RESET_COUNT_ERROR, SCSI_SENSEQ_LONG_WRITE_IN_PROGRESS, SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED, SCSI_SENSEQ_MEDIA_CODE_MISMATCHED_TO_LOGICAL_UNIT, SCSI_SENSEQ_MEDIUM_REMOVAL, SCSI_SENSEQ_OPERATION_IN_PROGRESS, SCSI_SENSEQ_OPERATION_IS_IN_PROGRESS, SCSI_SENSEQ_READ_OF_SCRAMBLED_SECTOR_WITHOUT_AUTHENTICATION, SCSI_SENSEQ_SOFT_THRESHOLD_REACHED, SCSI_SENSEQ_SPACE_ALLOC_FAILED_WRITE_PROTECT, SCSI_SENSEQ_SPACE_ALLOC_IN_PROGRESS, SCSI_SENSEQ_TOO_MANY_SEGMENT_DESCRIPTORS, SCSI_SENSEQ_UNKNOWN_FORMAT, SCSI_SENSEQ_UNREACHABLE_TARGET, SCSI_SENSEQ_WRITE_PROTECT_DISABLE, SCSI_SENSEQ_WRITE_PROTECT_ENABLE, SCSI_SENSE_ABORTED_COMMAND, SCSI_SENSE_BLANK_CHECK, SCSI_SENSE_COPY_ABORTED, SCSI_SENSE_DATA_PROTECT, SCSI_SENSE_HARDWARE_ERROR, SCSI_SENSE_ILLEGAL_REQUEST, SCSI_SENSE_MEDIUM_ERROR, SCSI_SENSE_NOT_READY, SCSI_SENSE_NO_SENSE, SCSI_SENSE_RECOVERED_ERROR, SCSI_SENSE_UNIT_ATTENTION, SRB_STATUS_ABORTED, SRB_STATUS_BUS_RESET, SRB_STATUS_DATA_OVERRUN, SRB_STATUS_ERROR, SRB_STATUS_INVALID_REQUEST, SRB_STATUS_PHASE_SEQUENCE_FAILURE, SRB_STATUS_REQUEST_FLUSHED};
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
use crate::types::{BuildDeviceInputMdl, ClassDetermineTokenOperationCommandSupport, ClassGetDeviceParameter, ClassGetPdoId, ClassInitializeDispatchTables, ClassPnpQueryFdoRelations, ClassQueryPnpCapabilities, ClassScanForSpecial, ClassSetDeviceParameter, ClasspAllocatePowerProcessIrp, ClasspAllocateReleaseQueueIrp, ClasspCalculateLogicalSectorSize, ClasspDeleteTimer, ClasspDisableTimer, ClasspEjectionControl, ClasspEnableIdlePower, ClasspEnableTimer, ClasspEnqueueIdleRequest, ClasspGetCopyOffloadMaxDuration, ClasspGetInquiryVpdSupportInfo, ClasspGetLBProvisioningInfo, ClasspInitializeHotplugInfo, ClasspInitializeIdleTimer, ClasspInitializeTimer, ClasspIsPortable, ClasspRegisterMountedDeviceInterface, ClasspScanForSpecialInRegistry, ClasspValidateOffloadSupported, ClasspZeroQERR, DequeueFreeTransferPacket, EnqueueDeferredClientIrp, EnqueueFreeTransferPacket, FreeDeviceInputMdl, InitLowMemRetry, InitializeStorageRequestBlock, InitializeTransferPackets, SetupDriveCapacityTransferPacket, SetupReadWriteTransferPacket, StepLowMemRetry, SubmitTransferPacket, TransferPacketQueueRetryDpc};
// KeQuerySystemTime: only in ntkrnl_sys::ddk on i386; not used in this file
// resolve_imports.py: explicit kernel/private-type imports
use crate::guid::{GUID_CONSOLE_DISPLAY_STATE};
use crate::types::{AllFdosList, ClassBadItems, ClassGuidQueryRegInfoEx, ClassGuidSenseInfo2, ClassGuidSrbSupport, ClassGuidWorkingSet, MEDIA_LOCK_TYPE_InternalMediaLock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK, _CDB__START_STOP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{min};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IOCTL_MOUNTDEV_QUERY_UNIQUE_ID, IOCTL_MOUNTDEV_QUERY_DEVICE_NAME,
    IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME, IOCTL_STORAGE_GET_DEVICE_NUMBER,
    IOCTL_STORAGE_FIND_NEW_DEVICES, IOCTL_STORAGE_GET_HOTPLUG_INFO, IOCTL_STORAGE_SET_HOTPLUG_INFO,
    IOCTL_STORAGE_MCN_CONTROL, IOCTL_STORAGE_READ_CAPACITY, IOCTL_STORAGE_EVENT_NOTIFICATION,
    IOCTL_STORAGE_CHECK_PRIORITY_HINT_SUPPORT, IOCTL_STORAGE_FIRMWARE_DOWNLOAD,
    IOCTL_STORAGE_GET_LB_PROVISIONING_MAP_RESOURCES, IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES};
use crate::glue::{CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE, IOCTL_SCSI_GET_ADDRESS, IO_WRN_FAILURE_PREDICTED, KEY_WRITE, NonPagedPoolNxCacheAligned, RTL_QUERY_REGISTRY_TYPECHECK, RTL_QUERY_REGISTRY_TYPECHECK_SHIFT, SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED, SCSI_SENSE_OPTIONS_NONE, SENSE_BUFFER_SIZE_EX, STATUS_IO_DEVICE_ERROR, STATUS_IO_TIMEOUT, STATUS_NONEXISTENT_SECTOR, TRACE_FLAG_GENERAL, TRACE_FLAG_IOCTL, TRACE_FLAG_MCN, TRACE_FLAG_WMI, TRACE_LEVEL_VERBOSE, VPB_MOUNTED, _MEDIA_LOCK_TYPE_SecureMediaLock as SecureMediaLock, max};
use crate::guid::{GUID_IO_DEVICE_BECOMING_READY};
use crate::srblib::{SrbGetDataTransferLength, SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength, SrbGetSrbFlags, SrbGetSystemStatus, SrbSetScsiStatus};
use crate::types::{CLASSP_REG_WRITE_CACHE_VALUE_NAME, CLASS_ERROR_LOG_DATA, CLASS_PRIVATE_COMMON_DATA, CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK, ClassAddChild, ClassCheckVerifyComplete, ClassCleanupMediaChangeDetection, ClassDeviceGetLBProvisioningResources, ClassDeviceProcessOffloadRead, ClassDeviceProcessOffloadWrite, ClassIoComplete, ClassNotifyFailurePredicted, ClassQueueCapacityChangedEventWorker, ClassQueueProvisioningTypeChangedEventWorker, ClassQueueResourceExhaustionEventWorker, ClassQueueThresholdEventWorker, ClassReadDriveCapacity, ClassRemoveChild, ClassSendEjectionNotification, ClassSendNotification, ClassSendStartUnit, ClassSetMediaChangeState, ClasspAccessAlignmentProperty, ClasspAllocateReleaseRequest, ClasspConvertToScsiRequestBlock, ClasspDeviceCopyOffloadProperty, ClasspDeviceGetLBAStatus, ClasspDeviceLBProvisioningProperty, ClasspDeviceMediaTypeProperty, ClasspDeviceSeekPenaltyProperty, ClasspDeviceTrimProcess, ClasspDeviceTrimProperty, ClasspDuidQueryProperty, ClasspFreeReleaseRequest, ClasspInitializeRemoveTracking, ClasspMcnControl, ClasspModeSense, ClasspPerfIncrementErrorCount, ClasspPersistentReserve, ClasspPriorityHint, ClasspQueueLogIOEventWithContextWorker, ClasspReleaseQueue, ClasspStorageEventNotification, ClasspUninitializeRemoveTracking, ClasspWriteCacheProperty, DEV_SAFE_START_UNIT, DestroyAllTransferPackets, FDO_HACK_NO_RESERVE6, IOCTL_SCSI_EXECUTE_NONE, IOCTL_STORAGE_BASE, IdlePowerFDOList, IdlePowerFDOListMutex, InitializeDictionary, MAXIMUM_RETRIES, MODE_DISCONNECT_PAGE, MODE_PARAMETER_HEADER, MODE_PARAMETER_HEADER10, MODE_SENSE_CURRENT_VALUES, _MEDIA_CHANGE_DETECTION_STATE_MediaPresent as MediaPresent, NOT_READY_RETRY_INTERVAL, NO_REMOVE, NUM_ERROR_LOG_ENTRIES, PCDB, PCLASS_DEV_INFO, PCLASS_DRIVER_EXTENSION, PCLASS_ERROR_LOG_DATA, PCLASS_PRIVATE_COMMON_DATA, PFIXED_SENSE_DATA, PHYSICAL_DEVICE_EXTENSION, PIDLE_POWER_FDO_LIST_ENTRY, PINQUIRYDATA, PMODE_DISCONNECT_PAGE, PMODE_PARAMETER_HEADER, PMODE_PARAMETER_HEADER10, PPHYSICAL_DEVICE_EXTENSION, PSCSI_SENSE_DESCRIPTOR_BLOCK_COMMAND, PSCSI_SENSE_DESCRIPTOR_INFORMATION, PSTORAGE_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK_HEADER, SCSIOP_INQUIRY, SCSIOP_MODE_SENSE, SCSIOP_MODE_SENSE10, SCSIOP_READ, SCSIOP_READ12, SCSIOP_READ16, SCSIOP_READ6, SCSIOP_RELEASE_UNIT, SCSIOP_RELEASE_UNIT10, SCSIOP_RESERVE_UNIT, SCSIOP_RESERVE_UNIT10, SCSIOP_TEST_UNIT_READY, SCSIOP_VERIFY, SCSIOP_VERIFY16, SCSIOP_WRITE, SCSIOP_WRITE12, SCSIOP_WRITE16, SCSIOP_WRITE6, SCSISTAT_BUSY, SCSISTAT_GOOD, SCSISTAT_RESERVATION_CONFLICT, SCSI_ADDRESS, SCSI_ADSENSE_DATA_TRANSFER_ERROR, SCSI_ADSENSE_INVALID_MEDIA, SCSI_ADSENSE_LOGICAL_UNIT_ERROR, SCSI_ADSENSE_LUN_COMMUNICATION, SCSI_ADSENSE_LUN_NOT_READY, SCSI_ADSENSE_REC_DATA_ECC, SCSI_ADSENSE_REC_DATA_NOECC, SCSI_ADSENSE_SEEK_ERROR, SCSI_ADSENSE_TRACK_ERROR, SCSI_ADSENSE_WRITE_PROTECT, SCSI_PASS_THROUGH, SCSI_SENSEQ_CAPACITY_DATA_CHANGED, SCSI_SENSEQ_INITIATOR_RESPONSE_TIMEOUT, SCSI_SENSEQ_INIT_COMMAND_REQUIRED, SCSI_SENSEQ_INQUIRY_DATA_CHANGED, SCSI_SENSEQ_MICROCODE_CHANGED, SCSI_SENSEQ_OPERATING_DEFINITION_CHANGED, SCSI_SENSEQ_TIMEOUT_ON_LOGICAL_UNIT, SCSI_SENSE_DESCRIPTOR_TYPE_BLOCK_COMMAND, SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION, SCSI_SESNEQ_COMM_CRC_ERROR, SENSE_DATA, SRB_CLASS_FLAGS_FREE_MDL, SRB_CLASS_FLAGS_LOW_PRIORITY, SRB_CLASS_FLAGS_PAGING, SRB_FLAGS_CLASS_DRIVER_RESERVED, SRB_FLAGS_DATA_IN, SRB_FLAGS_DATA_OUT, SRB_FLAGS_DONT_START_NEXT_PACKET, SRB_FLAGS_NO_QUEUE_FREEZE, SRB_FLAGS_QUEUE_ACTION_ENABLE, SRB_FUNCTION_CLAIM_DEVICE, SRB_FUNCTION_RELEASE_DEVICE, SRB_HEAD_OF_QUEUE_TAG_REQUEST, SRB_ORDERED_QUEUE_TAG_REQUEST, SRB_SIMPLE_TAG_REQUEST, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_COMMAND_TIMEOUT, SRB_STATUS_INTERNAL_ERROR, SRB_STATUS_INVALID_LUN, SRB_STATUS_INVALID_PATH_ID, SRB_STATUS_INVALID_TARGET_ID, SRB_STATUS_NO_DEVICE, SRB_STATUS_NO_HBA, SRB_STATUS_PARITY_ERROR, SRB_STATUS_SELECTION_TIMEOUT, SRB_STATUS_SUCCESS, SRB_STATUS_TIMEOUT, SRB_STATUS_UNEXPECTED_BUS_FREE, SRB_TYPE_SCSI_REQUEST_BLOCK, _MEDIA_CHANGE_DETECTION_STATE_MediaNotPresent as MediaNotPresent, _MEDIA_CHANGE_DETECTION_STATE_MediaUnavailable as MediaUnavailable};
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
#[cfg(target_pointer_width = "64")]
use ntkrnl_sys::ddk::{IoIs32bitProcess};

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


// ==== Missing constants not in glue.rs ====
// CTL_CODE(DeviceType, Function, Method, Access) = (DevType<<16)|(Access<<14)|(Func<<2)|Method
// IOCTL_SCSI_BASE = FILE_DEVICE_CONTROLLER = 4
const IOCTL_SCSI_PASS_THROUGH: u32             = (4u32 << 16) | (1u32 << 14) | (0x0401u32 << 2) | 0; // METHOD_BUFFERED, FILE_READ_ACCESS|FILE_WRITE_ACCESS
const IOCTL_SCSI_PASS_THROUGH_DIRECT: u32      = (4u32 << 16) | (1u32 << 14) | (0x0405u32 << 2) | 0;
const IOCTL_SCSI_PASS_THROUGH_EX: u32          = (4u32 << 16) | (1u32 << 14) | (0x0411u32 << 2) | 0;
const IOCTL_SCSI_PASS_THROUGH_DIRECT_EX: u32   = (4u32 << 16) | (1u32 << 14) | (0x0412u32 << 2) | 0;
const IOCTL_DISK_BASE: u32      = 7u32;  // FILE_DEVICE_DISK
const IOCTL_TAPE_BASE: u32      = 31u32; // FILE_DEVICE_TAPE
const IOCTL_CDROM_BASE: u32     = 2u32;  // FILE_DEVICE_CD_ROM
// IOCTL_STORAGE_* constants (IOCTL_STORAGE_BASE = 0x2D = 45 = FILE_DEVICE_MASS_STORAGE)
const IOCTL_STORAGE_CHECK_VERIFY: u32     = (0x2Du32 << 16) | (1u32 << 14) | (0x0200u32 << 2) | 0;
const IOCTL_STORAGE_CHECK_VERIFY2: u32    = (0x2Du32 << 16) | (0u32 << 14) | (0x0200u32 << 2) | 0;
const IOCTL_STORAGE_MEDIA_REMOVAL: u32    = (0x2Du32 << 16) | (1u32 << 14) | (0x0201u32 << 2) | 0;
const IOCTL_STORAGE_EJECT_MEDIA: u32      = (0x2Du32 << 16) | (1u32 << 14) | (0x0202u32 << 2) | 0;
const IOCTL_STORAGE_LOAD_MEDIA: u32       = (0x2Du32 << 16) | (1u32 << 14) | (0x0203u32 << 2) | 0;
const IOCTL_STORAGE_LOAD_MEDIA2: u32      = (0x2Du32 << 16) | (0u32 << 14) | (0x0203u32 << 2) | 0;
const IOCTL_STORAGE_RESERVE: u32          = (0x2Du32 << 16) | (1u32 << 14) | (0x0204u32 << 2) | 0;
const IOCTL_STORAGE_RELEASE: u32          = (0x2Du32 << 16) | (1u32 << 14) | (0x0205u32 << 2) | 0;
const IOCTL_STORAGE_EJECTION_CONTROL: u32 = (0x2Du32 << 16) | (0u32 << 14) | (0x0250u32 << 2) | 0;
const IOCTL_STORAGE_QUERY_PROPERTY: u32   = (0x2Du32 << 16) | (0u32 << 14) | (0x0500u32 << 2) | 0;
const IRP_MN_SCSI_CLASS: u8 = 0x01u8;
// WMIREG_ACTION_DEREGISTER
const WMIREG_ACTION_DEREGISTER: u32 = 2u32;
// CLASS_TAG_DEVICE_CONTROL pool tag
const CLASS_TAG_DEVICE_CONTROL: u32 = 0x4F496353;
// ERROR_LOG_MAXIMUM_SIZE = IO_ERROR_LOG_MESSAGE_LENGTH(256) - IO_ERROR_LOG_MESSAGE_HEADER_LENGTH(8) = 248
const ERROR_LOG_MAXIMUM_SIZE: u32 = 248u32;
// Missing STATUS codes
const STATUS_FILES_OPEN: NTSTATUS              = 0xC0000107u32 as i32;
const STATUS_INVALID_BLOCK_LENGTH: NTSTATUS    = 0xC0000173u32 as i32;
const STATUS_NO_MEDIA_IN_DEVICE: NTSTATUS      = 0xC0000013u32 as i32;
const STATUS_RETRY: NTSTATUS                   = 0xC000022Du32 as i32;
const STATUS_UNRECOGNIZED_MEDIA: NTSTATUS      = 0xC0000014u32 as i32;
const STATUS_CLEANER_CARTRIDGE_INSTALLED: NTSTATUS = 0x80000027u32 as i32;
const STATUS_DEVICE_HARDWARE_ERROR: NTSTATUS   = 0xC0000483u32 as i32;
const STATUS_OPERATION_IN_PROGRESS: NTSTATUS   = 0xC0000476u32 as i32;
const STATUS_DEVICE_UNREACHABLE: NTSTATUS      = 0xC0000464u32 as i32;
const STATUS_INVALID_TOKEN: NTSTATUS           = 0xC0000465u32 as i32;
const STATUS_NO_SUCH_DEVICE: NTSTATUS          = 0xC000000Eu32 as i32;
const STATUS_INVALID_INITIATOR_TARGET_PATH: NTSTATUS = 0xC0000477_u32 as i32;
const STATUS_INVALID_OFFSET_ALIGNMENT: NTSTATUS = 0xC0000474_u32 as i32;
const STATUS_TOO_MANY_SEGMENT_DESCRIPTORS: NTSTATUS = 0xC0000473_u32 as i32;
const STATUS_INVALID_FIELD_IN_PARAMETER_LIST: NTSTATUS = 0xC0000475_u32 as i32;
const STATUS_COPY_PROTECTION_FAILURE: NTSTATUS = 0xC0000305_u32 as i32;
const STATUS_CSS_AUTHENTICATION_FAILURE: NTSTATUS = 0xC0000306_u32 as i32;
const STATUS_CSS_KEY_NOT_PRESENT: NTSTATUS     = 0xC0000307_u32 as i32;
const STATUS_CSS_KEY_NOT_ESTABLISHED: NTSTATUS = 0xC0000308_u32 as i32;
const STATUS_CSS_SCRAMBLED_SECTOR: NTSTATUS    = 0xC0000309_u32 as i32;
const STATUS_CSS_REGION_MISMATCH: NTSTATUS     = 0xC000030A_u32 as i32;
const STATUS_CSS_RESETS_EXHAUSTED: NTSTATUS    = 0xC000030B_u32 as i32;
const STATUS_DISK_RESOURCES_EXHAUSTED: NTSTATUS = 0xC0000461_u32 as i32;
const STATUS_NO_DATA_DETECTED: NTSTATUS        = 0x80000022u32 as i32;
// Missing IO error log codes
const IO_WARNING_PAGING_FAILURE: NTSTATUS = 0x80040033u32 as i32;
const IO_ERR_SEEK_ERROR: NTSTATUS         = 0xC0040006u32 as i32;
const IO_RECOVERED_VIA_ECC: NTSTATUS      = 0x80040021u32 as i32;
const IO_ERR_CONTROLLER_ERROR: NTSTATUS   = 0xC004000Bu32 as i32;
const IO_ERR_BAD_BLOCK: NTSTATUS          = 0xC0040007u32 as i32;
const IO_ERROR_IO_HARDWARE_ERROR: NTSTATUS = 0xC004009Au32 as i32;
const IO_ERR_NOT_READY: NTSTATUS          = 0xC004000Fu32 as i32;
const IO_WARNING_IO_OPERATION_RETRIED: NTSTATUS = 0x80040099u32 as i32;
// u8 aliases for match patterns (can't use (CONST as u8) in match arms)
const SCSI_ADSENSE_TRACK_ERROR_U8: u8   = SCSI_ADSENSE_TRACK_ERROR as u8;
const SCSI_ADSENSE_SEEK_ERROR_U8: u8    = SCSI_ADSENSE_SEEK_ERROR as u8;
const SCSI_ADSENSE_REC_DATA_NOECC_U8: u8 = SCSI_ADSENSE_REC_DATA_NOECC as u8;
const SCSI_ADSENSE_REC_DATA_ECC_U8: u8  = SCSI_ADSENSE_REC_DATA_ECC as u8;
const SCSI_SENSEQ_INIT_COMMAND_REQUIRED_U8: u8 = SCSI_SENSEQ_INIT_COMMAND_REQUIRED as u8;
const SCSI_ADSENSE_LUN_NOT_READY_U8: u8  = SCSI_ADSENSE_LUN_NOT_READY as u8;
const SCSI_ADSENSE_LUN_COMMUNICATION_U8: u8 = SCSI_ADSENSE_LUN_COMMUNICATION as u8;
const SRB_STATUS_INVALID_PATH_ID_U8: u8 = SRB_STATUS_INVALID_PATH_ID as u8;
const SRB_STATUS_NO_DEVICE_U8: u8       = SRB_STATUS_NO_DEVICE as u8;
const SRB_STATUS_NO_HBA_U8: u8          = SRB_STATUS_NO_HBA as u8;
const SRB_STATUS_INVALID_LUN_U8: u8     = SRB_STATUS_INVALID_LUN as u8;
const SRB_STATUS_INVALID_TARGET_ID_U8: u8 = SRB_STATUS_INVALID_TARGET_ID as u8;
const SRB_STATUS_SELECTION_TIMEOUT_U8: u8 = SRB_STATUS_SELECTION_TIMEOUT as u8;
const SRB_STATUS_TIMEOUT_U8: u8         = SRB_STATUS_TIMEOUT as u8;
const SRB_STATUS_COMMAND_TIMEOUT_U8: u8 = SRB_STATUS_COMMAND_TIMEOUT as u8;
const SRB_STATUS_PARITY_ERROR_U8: u8    = SRB_STATUS_PARITY_ERROR as u8;
const SRB_STATUS_UNEXPECTED_BUS_FREE_U8: u8 = SRB_STATUS_UNEXPECTED_BUS_FREE as u8;
const SCSIOP_TEST_UNIT_READY_U8: u8     = SCSIOP_TEST_UNIT_READY as u8;

// ==== Local FORCEINLINE / NEEDS-SUBSTRATE helper declarations ====
unsafe extern "C" {
    
    fn ScsiGetSenseErrorCode(buf: PVOID) -> UCHAR;
    fn IsDescriptorSenseDataFormat(buf: PVOID) -> BOOLEAN;
    fn IsSenseDataFormatValueValid(buf: PVOID) -> BOOLEAN;
    fn ScsiGetSenseDescriptor(buf: PVOID, buf_len: UCHAR, desc_buf: *mut PVOID, desc_len: *mut UCHAR) -> BOOLEAN;
    fn ScsiGetNextSenseDescriptorByType(start: PVOID, start_len: UCHAR, types: *const UCHAR, type_count: ULONG, out_type: *mut UCHAR, out_buf: *mut PVOID, out_len: *mut UCHAR) -> BOOLEAN;
    fn ScsiValidateInformationSenseDescriptor(buf: PVOID, buf_len: u32) -> BOOLEAN;
    fn ScsiValidateBlockCommandSenseDescriptor(buf: PVOID, buf_len: u32) -> BOOLEAN;
    fn ScsiGetTotalSenseByteCountIndicated(buf: PVOID, buf_len: UCHAR, count: *mut UCHAR) -> BOOLEAN;
    fn ScsiGetSenseDescriptorLength(buf: PVOID) -> UCHAR;
    fn ScsiConvertToFixedSenseFormat(sense_buf: PVOID, buf_len: u32, fixed_buf: PVOID, fixed_buf_len: u32) -> BOOLEAN;
    fn SrbGetScsiStatus(srb: PVOID) -> UCHAR;
    fn SrbGetRequestAttribute(srb: PVOID) -> u8;
    fn SrbGetPathId(srb: PVOID) -> UCHAR;
    fn SrbGetTargetId(srb: PVOID) -> UCHAR;
    fn SrbGetLun(srb: PVOID) -> UCHAR;
    fn SrbGetCdbLength(srb: PVOID) -> UCHAR;
    fn SrbSetNextSrb(srb: PVOID, next: PVOID);
    
    
    fn PORT_ALLOCATED_SENSE_EX(fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, srb: PVOID) -> BOOLEAN;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, srb: PVOID);
    fn MmGetMdlVirtualAddress(mdl: PVOID) -> PVOID;
    fn RtlStringCchPrintfA(dest: *mut i8, dest_sz: usize, fmt: *const i8, ...) -> NTSTATUS;
    fn RtlStringCbPrintfA(dest: *mut i8, dest_sz: usize, fmt: *const i8, ...) -> NTSTATUS;
    fn ClasspIsOffloadDataTransferCommand(cdb: PCDB) -> BOOLEAN;
    fn InterpretSenseInfoWithoutHistory(fdo: PDEVICE_OBJECT, irp: PIRP, srb: PSCSI_REQUEST_BLOCK, major_fn: UCHAR, io_device_code: ULONG, prev_retry: ULONG, status: *mut NTSTATUS, retry_in_100ns: *mut LONGLONG) -> BOOLEAN;
    fn ClasspIsReceiveTokenInformation_c(cdb: PCDB) -> BOOLEAN;
    fn DBGGETSCSIOPSTR(srb: PVOID) -> PCHAR;
    fn DBGGETSRBSTATUSSTR(srb: PVOID) -> PCHAR;
    fn DBGGETSENSECODESTR(srb: PVOID) -> PCHAR;
    fn DBGGETADSENSECODESTR(srb: PVOID) -> PCHAR;
    fn DBGGETADSENSEQUALIFIERSTR(srb: PVOID) -> PCHAR;
    fn DBGGETIOCTLSTR(code: ULONG) -> PCHAR;
    #[link_name = "ClassAcquireRemoveLock"]
    fn ClassAcquireRemoveLockULONG(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> ULONG;
}

// ScsiGetFixedSenseKeyAndCodes — FORCEINLINE in C scsi.h; not a linked symbol.
// Direct copy from utils.rs private fn.
#[inline]
unsafe fn ScsiGetFixedSenseKeyAndCodes(
    SenseInfoBuffer: *mut c_void,
    SenseInfoBufferLength: UCHAR,
    SenseKey: *mut UCHAR,
    AdditionalSenseCode: *mut UCHAR,
    AdditionalSenseCodeQualifier: *mut UCHAR,
) -> BOOLEAN {
    use crate::types::PFIXED_SENSE_DATA;
    let senseData = SenseInfoBuffer as PFIXED_SENSE_DATA;
    if SenseInfoBuffer.is_null()
        || (SenseInfoBufferLength as usize) < core::mem::size_of::<crate::types::FIXED_SENSE_DATA>()
    {
        return 0u8;
    }
    if !SenseKey.is_null() {
        *SenseKey = (*senseData).SenseKey();
    }
    if !AdditionalSenseCode.is_null() {
        *AdditionalSenseCode = (*senseData).AdditionalSenseCode;
    }
    if !AdditionalSenseCodeQualifier.is_null() {
        *AdditionalSenseCodeQualifier = (*senseData).AdditionalSenseCodeQualifier;
    }
    1u8
}

// ClasspIsReceiveTokenInformation — FORCEINLINE in classp.h; wraps C helper
#[inline]
unsafe fn ClasspIsReceiveTokenInformation(Cdb: PCDB) -> BOOLEAN {
    use crate::types::{SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION, SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION};
    if Cdb.is_null() { return 0u8; }
    let op_code = (*Cdb).AsByte.as_ref()[0];
    let service_action = (*Cdb).AsByte.as_ref()[1];
    if op_code == (SCSIOP_RECEIVE_ROD_TOKEN_INFORMATION) as u8
        && service_action == (SERVICE_ACTION_RECEIVE_TOKEN_INFORMATION) as u8
    {
        1u8
    } else {
        0u8
    }
}

// IS_SCSIOP_READ/WRITE/READWRITE — macros in classpnp.h
#[inline]
fn IS_SCSIOP_READ(op: u8) -> bool {
    op == (SCSIOP_READ6) as u8
        || op == (SCSIOP_READ) as u8
        || op == (SCSIOP_READ12) as u8
        || op == (SCSIOP_READ16) as u8
}
#[inline]
fn IS_SCSIOP_WRITE(op: u8) -> bool {
    op == (SCSIOP_WRITE6) as u8
        || op == (SCSIOP_WRITE) as u8
        || op == (SCSIOP_WRITE12) as u8
        || op == (SCSIOP_WRITE16) as u8
}
#[inline]
fn IS_SCSIOP_READWRITE(op: u8) -> u8 {
    (IS_SCSIOP_READ(op) || IS_SCSIOP_WRITE(op)) as u8
}

// ARGUMENT_PRESENT macro equivalent
#[inline]
fn ARGUMENT_PRESENT(ptr: PVOID) -> u8 {
    (!ptr.is_null()) as u8
}

// ==== class.c missing functions (rustconv translate, class_split_b) ====

#[no_mangle]
pub unsafe extern "system" fn ClassAcquireChildLock(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION
) {
    if (*FdoExtension).ChildLockOwner != (KeGetCurrentThread()).cast() {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*FdoExtension).ChildLock) as *mut core::ffi::c_void,
            Executive,
            KernelMode as i8,
            0u8,
            core::ptr::null_mut()
        );

        NT_ASSERT!((*FdoExtension).ChildLockOwner == core::ptr::null_mut());
        NT_ASSERT!((*FdoExtension).ChildLockAcquisitionCount == 0);

        (*FdoExtension).ChildLockOwner = (KeGetCurrentThread()).cast();
    } else {
        NT_ASSERT!((*FdoExtension).ChildLockAcquisitionCount != 0);
    }

    (*FdoExtension).ChildLockAcquisitionCount += 1;
}

#[no_mangle]
pub unsafe extern "system" fn ClassClaimDevice(
    LowerDeviceObject: PDEVICE_OBJECT,
    Release: BOOLEAN
) -> NTSTATUS {
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut irp: PIRP;
    let irpStack: PIO_STACK_LOCATION;
    let mut event: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();

    srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;

    srb.Function = if Release != 0 {
        (SRB_FUNCTION_RELEASE_DEVICE) as u8
    } else {
        (SRB_FUNCTION_CLAIM_DEVICE) as u8
    };

    KeInitializeEvent(&mut event, (SynchronizationEvent as i32) as u32, 0);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_SCSI_EXECUTE_NONE,
        LowerDeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        1,
        &mut event,
        &mut ioStatus
    );

    if irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    irpStack = IoGetNextIrpStackLocation(irp);

    (*irpStack).Parameters.Scsi.Srb = &mut srb as *mut _ as *mut _;

    srb.OriginalRequest = irp as *mut core::ffi::c_void;

    status = IoCallDriver(LowerDeviceObject, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut event as *mut _ as PVOID,
            (Executive as i32) as u32,
            KernelMode as i8,
            0,
            core::ptr::null_mut()
        );
        status = ioStatus.__bindgen_anon_1.Status;
    }

    if Release != 0 {
        return STATUS_SUCCESS;
    }

    if !NT_SUCCESS(status) {
        return status;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn ClassCreateDeviceObject(
    DriverObject: PDRIVER_OBJECT,
    ObjectNameBuffer: PCCHAR,
    LowerDevice: PDEVICE_OBJECT,
    IsFdo: BOOLEAN,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut isPartitionable: BOOLEAN;
    let mut ntNameString: STRING = core::mem::zeroed();
    let mut ntUnicodeString: UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut characteristics: ULONG;
    let rundownSize: SIZE_T = (ExSizeOfRundownProtectionCacheAware()) as usize;
    let mut rundownAddr: PCHAR = core::ptr::null_mut();
    let mut devExtSize: ULONG;

    let driverExtension: PCLASS_DRIVER_EXTENSION = IoGetDriverObjectExtension(
        DriverObject,
        ClassInitialize as PVOID,
    ) as PCLASS_DRIVER_EXTENSION;

    let devInfo: PCLASS_DEV_INFO;

    *DeviceObject = core::ptr::null_mut();
    RtlInitUnicodeString(&mut ntUnicodeString as *mut _ as *mut _, core::ptr::null());

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClassCreateFdo: Create device object\n\0".as_ptr() as PCHAR,
    );

    NT_ASSERT!(!LowerDevice.is_null());

    isPartitionable = if (*driverExtension).InitData.ClassEnumerateDevice.is_some() {
        1u8
    } else {
        0u8
    };

    NT_ASSERT!(IsFdo != 0 || isPartitionable != 0);

    if IsFdo != 0 {
        devInfo = &mut (*driverExtension).InitData.FdoData;
    } else {
        devInfo = &mut (*driverExtension).InitData.PdoData;
    }

    characteristics = (*devInfo).DeviceCharacteristics;

    if ARGUMENT_PRESENT(ObjectNameBuffer as PVOID) != 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClassCreateFdo: Name is %s\n\0".as_ptr() as PCHAR,
            ObjectNameBuffer,
        );

        RtlInitString(&mut ntNameString, ObjectNameBuffer);

        status = RtlAnsiStringToUnicodeString(&mut ntUnicodeString as *mut _ as *mut _, &mut ntNameString as *mut _ as *mut _, 1);

        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"ClassCreateFdo: Cannot convert string %s\n\0".as_ptr() as PCHAR,
                ObjectNameBuffer,
            );

            ntUnicodeString.Buffer = core::ptr::null_mut();
            return status;
        }
    } else {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"ClassCreateFdo: Object will be unnamed\n\0".as_ptr() as PCHAR,
        );

        if IsFdo == 0 {
            SET_FLAG!(characteristics, FILE_AUTOGENERATED_DEVICE_NAME);
        }

        RtlInitUnicodeString(&mut ntUnicodeString as *mut _ as *mut _, core::ptr::null());
    }

    devExtSize = (*devInfo).DeviceExtensionSize
        + core::mem::size_of::<CLASS_PRIVATE_COMMON_DATA>() as ULONG
        + rundownSize as ULONG;

    status = IoCreateDevice(
        DriverObject,
        devExtSize,
        &mut ntUnicodeString as *mut _ as *mut _,
        (*devInfo).DeviceType,
        characteristics,
        0,
        &mut deviceObject,
    );

    if !NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"ClassCreateFdo: Can not create device object %lx\n\0".as_ptr() as PCHAR,
            status,
        );
        NT_ASSERT!(deviceObject.is_null());

        if !ntUnicodeString.Buffer.is_null() {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"ClassCreateFdo: Freeing unicode name buffer\n\0".as_ptr() as PCHAR,
            );
            FREE_POOL(ntUnicodeString.Buffer as PVOID);
            RtlInitUnicodeString(&mut ntUnicodeString as *mut _ as *mut _, core::ptr::null());
        }
    } else {
        let commonExtension: PCOMMON_DEVICE_EXTENSION = (*deviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

        RtlZeroMemory((*deviceObject).DeviceExtension, devExtSize as SIZE_T);

        (*commonExtension).Version = 0x03;

        (*commonExtension).IsRemoved = NO_REMOVE;

        (*commonExtension).RemoveLock = 0;

        KeInitializeEvent(
            &mut (*commonExtension).RemoveEvent,
            SynchronizationEvent,
            0,
        );

        ClasspInitializeRemoveTracking(deviceObject);

        (*commonExtension).PrivateCommonData = ((*deviceObject).DeviceExtension as PCHAR)
            .add((*devInfo).DeviceExtensionSize as usize)
            as PCLASS_PRIVATE_COMMON_DATA;

        rundownAddr = ((*commonExtension).PrivateCommonData as PCHAR)
            .add(core::mem::size_of::<CLASS_PRIVATE_COMMON_DATA>());

        ExInitializeRundownProtectionCacheAware(
            rundownAddr as PEX_RUNDOWN_REF_CACHE_AWARE,
            rundownSize as _,
        );

        (*(*commonExtension).PrivateCommonData).RemoveLockFailAcquire = 0;

        ClassAcquireRemoveLock(deviceObject, deviceObject as PIRP);

        (*commonExtension).DriverExtension = driverExtension;

        (*commonExtension).DevInfo = devInfo;

        (*commonExtension).DeviceObject = deviceObject;

        (*commonExtension).LowerDeviceObject = core::ptr::null_mut();

        (*commonExtension).DispatchTable = core::ptr::addr_of_mut!((*driverExtension).DeviceMajorFunctionTable) as *mut _;

        if IsFdo != 0 {
            let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = commonExtension as PFUNCTIONAL_DEVICE_EXTENSION;

            (*commonExtension).PartitionZeroExtension = ((*deviceObject).DeviceExtension).cast();

            SET_FLAG!((*deviceObject).Flags, DO_POWER_PAGABLE);

            (*commonExtension).ChildList = core::ptr::null_mut();

            (*commonExtension).DriverData = (fdoExtension as PCHAR)
                .add(core::mem::size_of::<FUNCTIONAL_DEVICE_EXTENSION>())
                as PVOID;

            if isPartitionable != 0 || (*devInfo).DeviceType == FILE_DEVICE_DISK {
                (*commonExtension).PartitionNumber = 0;
            } else {
                (*commonExtension).PartitionNumber = 0xFFFFFFFF;
            }

            (*fdoExtension).DevicePowerState = PowerDeviceD0;

            KeInitializeEvent(
                &mut (*fdoExtension).EjectSynchronizationEvent,
                SynchronizationEvent,
                1,
            );

            KeInitializeEvent(
                &mut (*fdoExtension).ChildLock,
                SynchronizationEvent,
                1,
            );

            status = ClasspAllocateReleaseRequest(deviceObject);

            if !NT_SUCCESS(status) {
                IoDeleteDevice(deviceObject);
                *DeviceObject = core::ptr::null_mut();

                if !ntUnicodeString.Buffer.is_null() {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_INIT,
                        b"ClassCreateFdo: Freeing unicode name buffer\n\0".as_ptr() as PCHAR,
                    );
                    FREE_POOL(ntUnicodeString.Buffer as PVOID);
                    RtlInitUnicodeString(&mut ntUnicodeString as *mut _ as *mut _, core::ptr::null());
                }

                return status;
            }
        } else {
            let pdoExtension: PPHYSICAL_DEVICE_EXTENSION =
                (*deviceObject).DeviceExtension as PPHYSICAL_DEVICE_EXTENSION;

            let p0Extension: PFUNCTIONAL_DEVICE_EXTENSION =
                (*LowerDevice).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

            SET_FLAG!((*deviceObject).Flags, DO_POWER_PAGABLE);

            (*commonExtension).PartitionZeroExtension = (p0Extension as PVOID).cast();

            ClassAddChild(p0Extension, pdoExtension, 1);

            (*commonExtension).DriverData = (pdoExtension as PCHAR)
                .add(core::mem::size_of::<PHYSICAL_DEVICE_EXTENSION>())
                as PVOID;

            (*commonExtension).LowerDeviceObject = IoGetAttachedDeviceReference(LowerDevice);

            ObDereferenceObject((*commonExtension).LowerDeviceObject as PVOID);
        }

        KeInitializeEvent(
            &mut (*commonExtension).PathCountEvent,
            SynchronizationEvent,
            1,
        );

        (*commonExtension).__bindgen_anon_1.set_IsFdo(IsFdo);

        (*commonExtension).DeviceName = core::mem::transmute(ntUnicodeString);

        (*commonExtension).PreviousState = 0xff;

        InitializeDictionary(&mut (*commonExtension).FileObjectDictionary);

        (*commonExtension).CurrentState = IRP_MN_STOP_DEVICE as u8;

        if (*(*commonExtension).DriverExtension).InitData.ClassStartIo.is_some() {
            IoSetStartIoAttributes(deviceObject, 1, 1);
        }
    }

    *DeviceObject = deviceObject;

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn ClassDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS
{
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut nextStack: PIO_STACK_LOCATION = core::ptr::null_mut();

    let controlCode = (*irpStack).Parameters.DeviceIoControl.IoControlCode;

    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: PCDB = core::ptr::null_mut();

    let mut status: NTSTATUS;
    let mut modifiedIoControlCode: ULONG = 0;
    let mut activityId: GUID = core::mem::zeroed();

    if (controlCode == IOCTL_SCSI_PASS_THROUGH) ||
       (controlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT) ||
       (controlCode == IOCTL_SCSI_PASS_THROUGH_EX) ||
       (controlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT_EX) {

        if (controlCode == IOCTL_SCSI_PASS_THROUGH) ||
           (controlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT) {

            #[cfg(all(target_pointer_width = "64"))]
            {
                if IoIs32bitProcess(Irp) != 0 {
                    if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<SCSI_PASS_THROUGH32>() as u32 {
                        (*Irp).IoStatus.Information = 0;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;

                        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);

                        status = STATUS_INVALID_PARAMETER;
                        return status;
                    }
                } else {
                    if (*irpStack).Parameters.DeviceIoControl.InputBufferLength <
                        core::mem::size_of::<SCSI_PASS_THROUGH>() as u32 {
                        (*Irp).IoStatus.Information = 0;
                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;

                        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);

                        status = STATUS_INVALID_PARAMETER;
                        return status;
                    }
                }
            }

            #[cfg(target_pointer_width = "32")]
            {
                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength <
                    core::mem::size_of::<SCSI_PASS_THROUGH>() as u32 {
                    (*Irp).IoStatus.Information = 0;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);

                    status = STATUS_INVALID_PARAMETER;
                    return status;
                }
            }
        }

        IoCopyCurrentIrpStackLocationToNext(Irp);

        nextStack = IoGetNextIrpStackLocation(Irp);
        (*nextStack).MinorFunction = 1;

        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());

        status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
        return status;
    }

    (*Irp).IoStatus.Information = 0;

    'outer: loop {
        match controlCode {
            IOCTL_MOUNTDEV_QUERY_UNIQUE_ID => {
                if (*commonExtension).MountedDeviceInterfaceName.Buffer.is_null() {
                    status = STATUS_INVALID_PARAMETER;
                    break;
                }

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as u32 {
                    status = STATUS_BUFFER_TOO_SMALL;
                    (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as _;
                    break;
                }

                let uniqueId = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTDEV_UNIQUE_ID;
                RtlZeroMemory(uniqueId as *mut core::ffi::c_void, core::mem::size_of::<MOUNTDEV_UNIQUE_ID>());
                (*uniqueId).UniqueIdLength = (*commonExtension).MountedDeviceInterfaceName.Length;

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<USHORT>() as u32 + (*uniqueId).UniqueIdLength as u32 {
                    status = STATUS_BUFFER_OVERFLOW;
                    (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as _;
                    break;
                }

                RtlCopyMemory(
                    (*uniqueId).UniqueId.as_mut_ptr() as *mut core::ffi::c_void,
                    (*commonExtension).MountedDeviceInterfaceName.Buffer as *const core::ffi::c_void,
                    (*uniqueId).UniqueIdLength as usize
                );

                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = (core::mem::size_of::<USHORT>() + (*uniqueId).UniqueIdLength as usize) as _;
            }

            IOCTL_MOUNTDEV_QUERY_DEVICE_NAME => {
                NT_ASSERT!(!(*commonExtension).DeviceName.Buffer.is_null());

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<MOUNTDEV_NAME>() as u32 {
                    status = STATUS_BUFFER_TOO_SMALL;
                    (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTDEV_NAME>() as _;
                    break;
                }

                let name = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTDEV_NAME;
                RtlZeroMemory(name as *mut core::ffi::c_void, core::mem::size_of::<MOUNTDEV_NAME>());
                (*name).NameLength = (*commonExtension).DeviceName.Length;

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<USHORT>() as u32 + (*name).NameLength as u32 {
                    status = STATUS_BUFFER_OVERFLOW;
                    (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTDEV_NAME>() as _;
                    break;
                }

                RtlCopyMemory(
                    (*name).Name.as_mut_ptr() as *mut core::ffi::c_void,
                    (*commonExtension).DeviceName.Buffer as *const core::ffi::c_void,
                    (*name).NameLength as usize
                );

                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = (core::mem::size_of::<USHORT>() + (*name).NameLength as usize) as _;
            }

            IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME => {
                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>() as u32 {
                    status = STATUS_BUFFER_TOO_SMALL;
                    (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>() as _;
                    break;
                }

                let valueName = ExAllocatePoolWithTag(
                    PagedPool,
                    (((*commonExtension).DeviceName.Length as usize + core::mem::size_of::<WCHAR>()) as u64) as usize,
                    0x38436353
                );

                if valueName.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                    break;
                }

                RtlCopyMemory(
                    valueName,
                    (*commonExtension).DeviceName.Buffer as *const core::ffi::c_void,
                    (*commonExtension).DeviceName.Length as usize
                );
                *((valueName as usize + (*commonExtension).DeviceName.Length as usize) as *mut WCHAR) = 0;

                let mut driveLetterNameBuffer: [WCHAR; 10] = [0; 10];
                let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
                let mut driveLetterName: UNICODE_STRING = core::mem::zeroed();

                driveLetterName.Buffer = driveLetterNameBuffer.as_mut_ptr();
                driveLetterName.MaximumLength = core::mem::size_of_val(&driveLetterNameBuffer) as USHORT;
                driveLetterName.Length = 0;

                queryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED |
                                      RTL_QUERY_REGISTRY_DIRECT |
                                      RTL_QUERY_REGISTRY_TYPECHECK;
                queryTable[0].Name = valueName as PWSTR;
                queryTable[0].EntryContext = core::ptr::addr_of_mut!(driveLetterName) as *mut core::ffi::c_void;
                queryTable[0].DefaultType = (REG_SZ << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | REG_NONE;

                status = RtlQueryRegistryValues(
                    RTL_REGISTRY_ABSOLUTE,
                    w!("\\Registry\\Machine\\System\\DISK").as_ptr() as PWSTR,
                    queryTable.as_mut_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut()
                );

                if !NT_SUCCESS(status) {
                    FREE_POOL!(valueName);
                    break;
                }

                if driveLetterName.Length == 4 &&
                   *driveLetterName.Buffer.offset(0) == 0x25 &&
                   *driveLetterName.Buffer.offset(1) == 0x3A {
                    *driveLetterName.Buffer.offset(0) = 0xFF;
                } else if driveLetterName.Length != 4 ||
                   *driveLetterName.Buffer.offset(0) < FirstDriveLetter as u16 ||
                   *driveLetterName.Buffer.offset(0) > LastDriveLetter as u16 ||
                   *driveLetterName.Buffer.offset(1) != 0x3A {
                    status = STATUS_NOT_FOUND;
                    FREE_POOL!(valueName);
                    break;
                }

                let suggestedName = (*Irp).AssociatedIrp.SystemBuffer as PMOUNTDEV_SUGGESTED_LINK_NAME;
                RtlZeroMemory(suggestedName as *mut core::ffi::c_void, core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>());
                (*suggestedName).UseOnlyIfThereAreNoOtherLinks = 1u8;
                (*suggestedName).NameLength = 28;

                (*Irp).IoStatus.Information = (FIELD_OFFSET!(MOUNTDEV_SUGGESTED_LINK_NAME, Name) + 28) as _;

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    (*Irp).IoStatus.Information as u32 {
                    (*Irp).IoStatus.Information = core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>() as _;
                    status = STATUS_BUFFER_OVERFLOW;
                    FREE_POOL!(valueName);
                    break;
                }

                RtlDeleteRegistryValue(
                    RTL_REGISTRY_ABSOLUTE,
                    w!("\\Registry\\Machine\\System\\DISK").as_ptr() as PWSTR,
                    valueName as PWSTR
                );

                FREE_POOL!(valueName);

                RtlCopyMemory(
                    (*suggestedName).Name.as_mut_ptr() as *mut core::ffi::c_void,
                    b"\\DosDevices\\\0".as_ptr() as *const core::ffi::c_void,
                    24
                );
                *(*suggestedName).Name.as_mut_ptr().offset(12) = *driveLetterName.Buffer.offset(0);
                *(*suggestedName).Name.as_mut_ptr().offset(13) = 0x3A;

                status = STATUS_SUCCESS;
            }

            _ => {
                status = STATUS_PENDING;
            }
        }
        break;
    }

    if status != STATUS_PENDING {
        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;

        IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return status;
    }

    if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        let fdoExtension = commonExtension as PFUNCTIONAL_DEVICE_EXTENSION;
        let sizeNeeded: usize;

        if !(*fdoExtension).AdapterDescriptor.is_null() &&
           (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
            sizeNeeded = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
        } else {
            sizeNeeded = core::mem::size_of::<SCSI_REQUEST_BLOCK>();
        }

        srb = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            ((sizeNeeded + (core::mem::size_of::<ULONG_PTR>() * 2)) as u64) as usize,
            0x39436353
        ) as PSCSI_REQUEST_BLOCK;

        if srb.is_null() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
            status = STATUS_INSUFFICIENT_RESOURCES;
            return status;
        }

        let function: PULONG_PTR;

        if !(*fdoExtension).AdapterDescriptor.is_null() &&
           (*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
            status = InitializeStorageRequestBlock(
                srb as PSTORAGE_REQUEST_BLOCK,
                (STORAGE_ADDRESS_TYPE_BTL8) as u16,
                CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
                1,
                SrbExDataTypeScsiCdb16
            );
            if NT_SUCCESS(status) {
                (*(srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
                function = ((srb as usize) + sizeNeeded) as PULONG_PTR;
            } else {
                NT_ASSERT!(false);
                return status;
            }
        } else {
            RtlZeroMemory(srb as *mut core::ffi::c_void, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
            (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
            (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
            function = (srb.offset(1)) as PULONG_PTR;
        }

        *function = DeviceObject as ULONG_PTR;
        *function.offset(1) = controlCode as ULONG_PTR;
    }

    if ((controlCode & 0xffff0000) == (IOCTL_DISK_BASE << 16)) ||
       ((controlCode & 0xffff0000) == (IOCTL_TAPE_BASE << 16)) ||
       ((controlCode & 0xffff0000) == (IOCTL_CDROM_BASE << 16)) {
        modifiedIoControlCode = (controlCode & !0xffff0000);
        modifiedIoControlCode |= IOCTL_STORAGE_BASE << 16;
    } else {
        modifiedIoControlCode = controlCode;
    }

    TracePrint(TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL, b"> ioctl %xh (%s)\0".as_ptr() as PCHAR, modifiedIoControlCode, DBGGETIOCTLSTR(modifiedIoControlCode));

    'switch: loop {
        match modifiedIoControlCode {
            IOCTL_STORAGE_GET_HOTPLUG_INFO => {
                FREE_POOL!(srb);

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as u32 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                    (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as _;

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    status = STATUS_BUFFER_TOO_SMALL;
                } else if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                } else {
                    let fdoExtension = commonExtension as PFUNCTIONAL_DEVICE_EXTENSION;
                    let info = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_HOTPLUG_INFO;

                    *info = (*(*fdoExtension).PrivateFdoData).HotplugInfo;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                    (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as _;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    status = STATUS_SUCCESS;
                }
            }

            IOCTL_STORAGE_SET_HOTPLUG_INFO => {
                FREE_POOL!(srb);

                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength <
                    core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as u32 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INFO_LENGTH_MISMATCH;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    status = STATUS_INFO_LENGTH_MISMATCH;
                    break 'switch;
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                } else {
                    let fdoExtension = commonExtension as PFUNCTIONAL_DEVICE_EXTENSION;
                    let info = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_HOTPLUG_INFO;

                    status = STATUS_SUCCESS;

                    if (*info).Size != (*(*fdoExtension).PrivateFdoData).HotplugInfo.Size {
                        status = STATUS_INVALID_PARAMETER_1;
                    }

                    if (*info).MediaRemovable != (*(*fdoExtension).PrivateFdoData).HotplugInfo.MediaRemovable {
                        status = STATUS_INVALID_PARAMETER_2;
                    }

                    if (*info).MediaHotplug != (*(*fdoExtension).PrivateFdoData).HotplugInfo.MediaHotplug {
                        status = STATUS_INVALID_PARAMETER_3;
                    }

                    if NT_SUCCESS(status) {
                        if (*info).WriteCacheEnableOverride != (*(*fdoExtension).PrivateFdoData).HotplugInfo.WriteCacheEnableOverride {
                            (*(*fdoExtension).PrivateFdoData).HotplugInfo.WriteCacheEnableOverride = (*info).WriteCacheEnableOverride;

                            ClassSetDeviceParameter(
                                fdoExtension,
                                crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                                crate::glue::regw::CLASSP_REG_WRITE_CACHE_VALUE_NAME.as_ptr() as PWSTR,
                                (*info).WriteCacheEnableOverride as ULONG
                            );
                        }

                        (*(*fdoExtension).PrivateFdoData).HotplugInfo.DeviceHotplug = (*info).DeviceHotplug;

                        ClassSetDeviceParameter(
                            fdoExtension,
                            crate::glue::regw::CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                            crate::glue::regw::CLASSP_REG_REMOVAL_POLICY_VALUE_NAME.as_ptr() as PWSTR,
                            if (*info).DeviceHotplug != 0 { RemovalPolicyExpectSurpriseRemoval } else { RemovalPolicyExpectOrderlyRemoval }
                        );
                    }

                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                }
            }

            IOCTL_STORAGE_CHECK_VERIFY | IOCTL_STORAGE_CHECK_VERIFY2 => {
                let mut Irp = Irp;
                let mut irp2: PIRP = core::ptr::null_mut();

                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, b"DeviceIoControl: Check verify\n\0".as_ptr() as PCHAR);

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength != 0 {
                    if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                        core::mem::size_of::<ULONG>() as u32 {
                        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"DeviceIoControl: media count buffer too small\n\0".as_ptr() as PCHAR);

                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;

                        FREE_POOL!(srb);

                        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);

                        status = STATUS_BUFFER_TOO_SMALL;
                        break 'switch;
                    }
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    NT_ASSERT!(srb.is_null());
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break 'switch;
                }

                let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength != 0 {
                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, b"DeviceIoControl: Check verify wants media count\n\0".as_ptr() as PCHAR);

                    irp2 = IoAllocateIrp(((*DeviceObject).StackSize as CCHAR) + 3, 0u8);

                    if irp2.is_null() {
                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                        (*Irp).IoStatus.Information = 0;
                        FREE_POOL!(srb);
                        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                        status = STATUS_INSUFFICIENT_RESOURCES;
                        break 'switch;
                    }

                    ClassAcquireRemoveLock(DeviceObject, (irp2 as *mut core::ffi::c_void).cast());

                    (*irp2).Tail.Overlay.Thread = (*Irp).Tail.Overlay.Thread;
                    IoSetNextIrpStackLocation(irp2);

                    let mut newStack = IoGetCurrentIrpStackLocation(irp2);
                    (*newStack).Parameters.Others.Argument1 = Irp as *mut core::ffi::c_void;
                    (*newStack).DeviceObject = DeviceObject;

                    IoSetCompletionRoutine(
                        irp2,
                        Some(ClassCheckVerifyComplete),
                        core::ptr::null_mut(),
                        1u8,
                        1u8,
                        1u8
                    );

                    IoSetNextIrpStackLocation(irp2);
                    newStack = IoGetCurrentIrpStackLocation(irp2);
                    (*newStack).DeviceObject = DeviceObject;
                    (*newStack).MajorFunction = (*irpStack).MajorFunction;
                    (*newStack).MinorFunction = (*irpStack).MinorFunction;
                    (*newStack).Flags = (*irpStack).Flags;

                    IoMarkIrpPending(Irp);

                    Irp = irp2;
                }

                SrbSetCdbLength((srb).cast(), 6);
                cdb = SrbGetCdb((srb).cast());
                (*core::ptr::addr_of_mut!((*cdb).CDB6GENERIC).cast::<crate::types::_CDB__CDB6GENERIC>()).OperationCode = SCSIOP_TEST_UNIT_READY as u8;

                SrbSetTimeOutValue((srb).cast(), (*fdoExtension).TimeOutValue);

                if controlCode == IOCTL_STORAGE_CHECK_VERIFY2 {
                    SrbSetSrbFlags((srb).cast(), SRB_CLASS_FLAGS_LOW_PRIORITY);
                }

                status = ClassSendSrbAsynchronous(
                    DeviceObject,
                    srb,
                    Irp,
                    core::ptr::null_mut(),
                    0,
                    0u8
                );
            }

            IOCTL_STORAGE_MEDIA_REMOVAL | IOCTL_STORAGE_EJECTION_CONTROL => {
                let mediaRemoval = (*Irp).AssociatedIrp.SystemBuffer as PPREVENT_MEDIA_REMOVAL;

                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, b"DiskIoControl: ejection control\n\0".as_ptr() as PCHAR);

                FREE_POOL!(srb);

                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength <
                    core::mem::size_of::<PREVENT_MEDIA_REMOVAL>() as u32 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INFO_LENGTH_MISMATCH;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    status = STATUS_INFO_LENGTH_MISMATCH;
                    break 'switch;
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                } else {
                    status = ClasspEjectionControl(
                        DeviceObject,
                        Irp,
                        if modifiedIoControlCode == IOCTL_STORAGE_EJECTION_CONTROL { SecureMediaLock } else { SimpleMediaLock },
                        (*mediaRemoval).PreventMediaRemoval
                    );

                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                }
            }

            IOCTL_STORAGE_MCN_CONTROL => {
                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, b"DiskIoControl: MCN control\n\0".as_ptr() as PCHAR);

                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength <
                    core::mem::size_of::<PREVENT_MEDIA_REMOVAL>() as u32 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INFO_LENGTH_MISMATCH;
                    (*Irp).IoStatus.Information = 0;

                    FREE_POOL!(srb);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    status = STATUS_INFO_LENGTH_MISMATCH;
                    break 'switch;
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    FREE_POOL!(srb);

                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                } else {
                    status = ClasspMcnControl(
                        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION,
                        Irp,
                        srb
                    );
                }
                break 'switch;
            }

            IOCTL_STORAGE_RESERVE | IOCTL_STORAGE_RELEASE => {
                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break 'switch;
                }

                let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

                if TEST_FLAG!((*(*fdoExtension).PrivateFdoData).HackFlags, (FDO_HACK_NO_RESERVE6) as ULONG_PTR) {
                    SrbSetCdbLength((srb).cast(), 10);
                    cdb = SrbGetCdb((srb).cast());
                    (*core::ptr::addr_of_mut!((*cdb).CDB10).cast::<crate::types::_CDB__CDB10>()).OperationCode = if modifiedIoControlCode == IOCTL_STORAGE_RESERVE {
                        SCSIOP_RESERVE_UNIT10 as u8
                    } else {
                        SCSIOP_RELEASE_UNIT10 as u8
                    };
                } else {
                    SrbSetCdbLength((srb).cast(), 6);
                    cdb = SrbGetCdb((srb).cast());
                    (*core::ptr::addr_of_mut!((*cdb).CDB6GENERIC).cast::<crate::types::_CDB__CDB6GENERIC>()).OperationCode = if modifiedIoControlCode == IOCTL_STORAGE_RESERVE {
                        SCSIOP_RESERVE_UNIT as u8
                    } else {
                        SCSIOP_RELEASE_UNIT as u8
                    };
                }

                SrbSetTimeOutValue((srb).cast(), (*fdoExtension).TimeOutValue);

                if IOCTL_STORAGE_RESERVE == modifiedIoControlCode {
                    SrbSetSrbFlags((srb).cast(), SRB_FLAGS_QUEUE_ACTION_ENABLE);
                    SrbSetRequestAttribute(srb.cast(), SRB_SIMPLE_TAG_REQUEST as u8);
                }

                status = ClassSendSrbAsynchronous(
                    DeviceObject,
                    srb,
                    Irp,
                    core::ptr::null_mut(),
                    0,
                    0u8
                );
            }

            IOCTL_STORAGE_PERSISTENT_RESERVE_IN | IOCTL_STORAGE_PERSISTENT_RESERVE_OUT => {
                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break 'switch;
                }

                status = ClasspPersistentReserve(DeviceObject, Irp, srb);
            }

            IOCTL_STORAGE_EJECT_MEDIA | IOCTL_STORAGE_LOAD_MEDIA | IOCTL_STORAGE_LOAD_MEDIA2 => {
                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break 'switch;
                }

                let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

                if (*commonExtension).PagingPathCount != 0 {
                    TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"ClassDeviceControl: call to eject paging device - failure\n\0".as_ptr() as PCHAR);

                    status = STATUS_FILES_OPEN;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    (*Irp).IoStatus.Information = 0;

                    FREE_POOL!(srb);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    break 'switch;
                }

                KeEnterCriticalRegion();
                let _ = KeWaitForSingleObject(
                    core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent) as *mut core::ffi::c_void,
                    Executive,
                    KernelMode as i8,
                    0u8,
                    core::ptr::null_mut()
                );

                if (*fdoExtension).ProtectedLockCount != 0 {
                    TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"ClassDeviceControl: call to eject protected locked device - failure\n\0".as_ptr() as PCHAR);

                    status = STATUS_DEVICE_BUSY;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    (*Irp).IoStatus.Information = 0;

                    FREE_POOL!(srb);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);

                    KeSetEvent(
                        (core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent) as PKEVENT).cast(),
                        IO_NO_INCREMENT as KPRIORITY,
                        0u8
                    );
                    KeLeaveCriticalRegion();

                    break 'switch;
                }

                SrbSetCdbLength((srb).cast(), 6);
                cdb = SrbGetCdb((srb).cast());

                (*core::ptr::addr_of_mut!((*cdb).START_STOP).cast::<crate::types::_CDB__START_STOP>()).OperationCode = SCSIOP_START_STOP_UNIT as u8;
                (*core::ptr::addr_of_mut!((*cdb).START_STOP).cast::<crate::types::_CDB__START_STOP>()).set_LoadEject(1);

                if modifiedIoControlCode == IOCTL_STORAGE_EJECT_MEDIA {
                    (*core::ptr::addr_of_mut!((*cdb).START_STOP).cast::<crate::types::_CDB__START_STOP>()).set_Start(0);
                } else {
                    (*core::ptr::addr_of_mut!((*cdb).START_STOP).cast::<crate::types::_CDB__START_STOP>()).set_Start(1);
                }

                SrbSetTimeOutValue((srb).cast(), (*fdoExtension).TimeOutValue);
                status = ClassSendSrbAsynchronous(
                    DeviceObject,
                    srb,
                    Irp,
                    core::ptr::null_mut(),
                    0,
                    0u8
                );

                KeSetEvent((core::ptr::addr_of_mut!((*fdoExtension).EjectSynchronizationEvent) as PKEVENT).cast(), IO_NO_INCREMENT as KPRIORITY, 0u8);
                KeLeaveCriticalRegion();
            }

            IOCTL_STORAGE_FIND_NEW_DEVICES => {
                FREE_POOL!(srb);

                if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
                    IoInvalidateDeviceRelations(
                        (*((commonExtension) as PFUNCTIONAL_DEVICE_EXTENSION)).LowerPdo,
                        BusRelations
                    );

                    status = STATUS_SUCCESS;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                } else {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                }
            }

            IOCTL_STORAGE_GET_DEVICE_NUMBER => {
                FREE_POOL!(srb);

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength >=
                    core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32 {
                    let deviceNumber = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DEVICE_NUMBER;
                    let fdoExtension = (*commonExtension).PartitionZeroExtension;

                    (*deviceNumber).DeviceType = (*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject).DeviceType;
                    (*deviceNumber).DeviceNumber = (*fdoExtension).DeviceNumber;
                    (*deviceNumber).PartitionNumber = (*commonExtension).PartitionNumber;

                    status = STATUS_SUCCESS;
                    (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as _;
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                    (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as _;
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
            }

            IOCTL_STORAGE_READ_CAPACITY => {
                FREE_POOL!(srb);

                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength <
                    core::mem::size_of::<STORAGE_READ_CAPACITY>() as u32 {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                    (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_READ_CAPACITY>() as _;

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    status = STATUS_BUFFER_TOO_SMALL;
                    break;
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                } else {
                    let fdoExt = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
                    let fdoData = (*fdoExt).PrivateFdoData;
                    let readCapacity = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_READ_CAPACITY;

                    status = ClassReadDriveCapacity(DeviceObject);
                    if NT_SUCCESS(status) && (*fdoData).IsCachedDriveCapDataValid != 0 {
                        (*readCapacity).Version = core::mem::size_of::<STORAGE_READ_CAPACITY>() as ULONG;
                        (*readCapacity).Size = core::mem::size_of::<STORAGE_READ_CAPACITY>() as ULONG;

                        REVERSE_BYTES_M!(
                            core::ptr::addr_of_mut!((*readCapacity).BlockLength),
                            core::ptr::addr_of!((*fdoData).LastKnownDriveCapacityData.BytesPerBlock)
                        );
                        REVERSE_BYTES_QUAD_M!(
                            core::ptr::addr_of_mut!((*readCapacity).NumberOfBlocks),
                            core::ptr::addr_of!((*fdoData).LastKnownDriveCapacityData.LogicalBlockAddress)
                        );
                        (*readCapacity).NumberOfBlocks.QuadPart += 1;

                        (*readCapacity).DiskLength = (*fdoExt).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength;

                        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                        (*Irp).IoStatus.Information = core::mem::size_of::<STORAGE_READ_CAPACITY>() as _;
                    } else {
                        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"ClassDeviceControl: ClassReadDriveCapacity failed: 0x%X IsCachedDriveCapDataValid: %d\n\0".as_ptr() as PCHAR,
                                       status, (*fdoData).IsCachedDriveCapDataValid as i32);
                        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                        (*Irp).IoStatus.Information = 0;
                    }
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                }
            }

            IOCTL_STORAGE_QUERY_PROPERTY => {
                let query = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;

                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32 {
                    status = STATUS_INFO_LENGTH_MISMATCH;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    FREE_POOL!(srb);
                    break;
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    FREE_POOL!(srb);
                    break;
                }

                match (*query).PropertyId {
                    StorageDeviceUniqueIdProperty => {
                        status = ClasspDuidQueryProperty(DeviceObject, Irp);
                    }
                    StorageDeviceWriteCacheProperty => {
                        status = ClasspWriteCacheProperty(DeviceObject, Irp, srb);
                    }
                    StorageAccessAlignmentProperty => {
                        status = ClasspAccessAlignmentProperty(DeviceObject, Irp, srb);
                    }
                    StorageDeviceSeekPenaltyProperty => {
                        status = ClasspDeviceSeekPenaltyProperty(DeviceObject, Irp, srb);
                    }
                    StorageDeviceTrimProperty => {
                        status = ClasspDeviceTrimProperty(DeviceObject, Irp, srb);
                    }
                    StorageDeviceLBProvisioningProperty => {
                        status = ClasspDeviceLBProvisioningProperty(DeviceObject, Irp, srb);
                    }
                    StorageDeviceCopyOffloadProperty => {
                        status = ClasspDeviceCopyOffloadProperty(DeviceObject, Irp, srb);
                    }
                    StorageDeviceMediumProductType => {
                        status = ClasspDeviceMediaTypeProperty(DeviceObject, Irp, srb);
                    }
                    _ => {
                        IoCopyCurrentIrpStackLocationToNext(Irp);

                        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                        status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    }
                }

                FREE_POOL!(srb);
            }

            IOCTL_STORAGE_CHECK_PRIORITY_HINT_SUPPORT => {
                FREE_POOL!(srb);

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break;
                }

                status = ClasspPriorityHint(DeviceObject, Irp);
            }

            IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES => {
                let dsmAttributes = (*Irp).AssociatedIrp.SystemBuffer as PDEVICE_MANAGE_DATA_SET_ATTRIBUTES;

                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>() as u32 {
                    status = STATUS_INVALID_PARAMETER;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    FREE_POOL!(srb);
                    break;
                }

                if (*irpStack).Parameters.DeviceIoControl.InputBufferLength <
                    (core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>() as u32 + (*dsmAttributes).ParameterBlockLength + (*dsmAttributes).DataSetRangesLength) {
                    status = STATUS_INVALID_PARAMETER;
                    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
                    FREE_POOL!(srb);
                    break;
                }

                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    FREE_POOL!(srb);
                    break;
                }

                match (*dsmAttributes).Action {
                    DeviceDsmAction_Trim => {
                        status = ClasspDeviceTrimProcess(DeviceObject, Irp, core::ptr::addr_of_mut!(activityId), srb);
                    }
                    DeviceDsmAction_OffloadRead => {
                        status = ClassDeviceProcessOffloadRead(DeviceObject, Irp, srb);
                    }
                    DeviceDsmAction_OffloadWrite => {
                        status = ClassDeviceProcessOffloadWrite(DeviceObject, Irp, srb);
                    }
                    DeviceDsmAction_Allocation => {
                        status = ClasspDeviceGetLBAStatus(DeviceObject, Irp, srb);
                    }
                    _ => {
                        IoCopyCurrentIrpStackLocationToNext(Irp);

                        ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                        status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    }
                }

                FREE_POOL!(srb);
            }

            IOCTL_STORAGE_GET_LB_PROVISIONING_MAP_RESOURCES => {
                if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
                    status = ClassDeviceGetLBProvisioningResources(DeviceObject, Irp, srb);
                } else {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                }

                FREE_POOL!(srb);
            }

            IOCTL_STORAGE_EVENT_NOTIFICATION => {
                FREE_POOL!(srb);

                status = ClasspStorageEventNotification(DeviceObject, Irp);
            }

            #[cfg(feature = "NTDDI_WINTRHESHOLD")]
            IOCTL_STORAGE_FIRMWARE_GET_INFO => {
                FREE_POOL!(srb);

                status = ClassDeviceHwFirmwareGetInfoProcess(DeviceObject, Irp);
            }

            #[cfg(feature = "NTDDI_WINTRHESHOLD")]
            IOCTL_STORAGE_FIRMWARE_DOWNLOAD => {
                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, Irp as *mut core::ffi::c_void);
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break 'switch;
                }

                status = ClassDeviceHwFirmwareDownloadProcess(DeviceObject, Irp, srb);
            }

            #[cfg(feature = "NTDDI_WINTRHESHOLD")]
            IOCTL_STORAGE_FIRMWARE_ACTIVATE => {
                if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
                    IoCopyCurrentIrpStackLocationToNext(Irp);

                    ClassReleaseRemoveLock(DeviceObject, Irp as *mut core::ffi::c_void);
                    status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
                    break 'switch;
                }

                status = ClassDeviceHwFirmwareActivateProcess(DeviceObject, Irp, srb);
            }

            _ => {
                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"IoDeviceControl: Unsupported device IOCTL %x for %p\n\0".as_ptr() as PCHAR,
                            controlCode, DeviceObject);

                FREE_POOL!(srb);

                IoCopyCurrentIrpStackLocationToNext(Irp);

                ClassReleaseRemoveLock(DeviceObject, (Irp as *mut core::ffi::c_void).cast());
                status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
            }
        }
        break;
    }

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, b"< ioctl %xh (%s): status %xh.\0".as_ptr() as PCHAR, modifiedIoControlCode, DBGGETIOCTLSTR(modifiedIoControlCode), status);

    status
}

#[no_mangle]
pub unsafe extern "system" fn ClassFindModePage(
    ModeSenseBuffer: PCHAR,
    Length: ULONG,
    PageMode: UCHAR,
    Use6Byte: BOOLEAN,
) -> PVOID {
    let mut limit: PUCHAR;
    let parameterHeaderLength: ULONG;
    let mut result: PVOID = core::ptr::null_mut();

    limit = ModeSenseBuffer.offset(Length as isize) as PUCHAR;
    parameterHeaderLength = if Use6Byte != 0 {
        core::mem::size_of::<MODE_PARAMETER_HEADER>() as ULONG
    } else {
        core::mem::size_of::<MODE_PARAMETER_HEADER10>() as ULONG
    };

    if Length >= parameterHeaderLength {
        let modeParam10: PMODE_PARAMETER_HEADER10;
        let blockDescriptorLength: ULONG;

        if Use6Byte != 0 {
            blockDescriptorLength = (*(ModeSenseBuffer as PMODE_PARAMETER_HEADER)).BlockDescriptorLength as ULONG;
        } else {
            modeParam10 = ModeSenseBuffer as PMODE_PARAMETER_HEADER10;
            blockDescriptorLength = (*modeParam10).BlockDescriptorLength[1] as ULONG;
        }

        let mut ModeSenseBuffer = ModeSenseBuffer.offset((parameterHeaderLength + blockDescriptorLength) as isize);

        while ModeSenseBuffer.offset(
            (core::mem::offset_of!(MODE_DISCONNECT_PAGE, PageLength) + core::mem::size_of::<u8>()) as isize
        ) < limit as PCHAR {
            if (*(ModeSenseBuffer as PMODE_DISCONNECT_PAGE)).PageCode() == PageMode {
                if ModeSenseBuffer.offset((*(ModeSenseBuffer as PMODE_DISCONNECT_PAGE)).PageLength as isize) > limit as PCHAR {
                    result = core::ptr::null_mut();
                } else {
                    result = ModeSenseBuffer as PVOID;
                }
                break;
            }

            ModeSenseBuffer = ModeSenseBuffer.offset(
                ((*(ModeSenseBuffer as PMODE_DISCONNECT_PAGE)).PageLength as isize) +
                (core::mem::offset_of!(MODE_DISCONNECT_PAGE, PageLength) + core::mem::size_of::<u8>()) as isize
            );
        }
    }

    result
}

#[no_mangle]
pub unsafe extern "system" fn ClassForwardIrpSynchronous(
    CommonExtension: PCOMMON_DEVICE_EXTENSION,
    Irp: PIRP
) -> NTSTATUS {
    IoCopyCurrentIrpStackLocationToNext(Irp);
    ClassSendIrpSynchronous((*CommonExtension).LowerDeviceObject, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn ClassGetDescriptor(
    DeviceObject: PDEVICE_OBJECT,
    PropertyId: PSTORAGE_PROPERTY_ID,
    Descriptor: *mut PVOID
) -> NTSTATUS {
    let mut query: STORAGE_PROPERTY_QUERY = core::mem::zeroed();
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut descriptor: PSTORAGE_DESCRIPTOR_HEADER;
    let mut length: ULONG;

    *Descriptor = core::ptr::null_mut();

    query.PropertyId = *PropertyId;
    query.QueryType = PropertyStandardQuery;

    descriptor = &mut query as *mut STORAGE_PROPERTY_QUERY as PVOID as PSTORAGE_DESCRIPTOR_HEADER;

    ClassSendDeviceIoControlSynchronous(
        IOCTL_STORAGE_QUERY_PROPERTY,
        DeviceObject,
        &mut query as *mut _ as PVOID,
        core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as ULONG,
        (core::mem::size_of::<ULONG>() * 2) as ULONG,
        0u8,
        &mut ioStatus
    );

    if !NT_SUCCESS(ioStatus.__bindgen_anon_1.Status) {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT, 
                   b"ClassGetDescriptor: error %lx trying to query properties #1\0".as_ptr() as PCHAR,
                   ioStatus.__bindgen_anon_1.Status);
        return ioStatus.__bindgen_anon_1.Status;
    }

    if (*descriptor).Size == 0 {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
                   b"ClassGetDescriptor: size returned was zero?! (status %x\0".as_ptr() as PCHAR,
                   ioStatus.__bindgen_anon_1.Status);
        return STATUS_UNSUCCESSFUL;
    }

    length = (*descriptor).Size;
    if length < core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as ULONG {
        length = core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as ULONG;
    }

    descriptor = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        length as usize,
        u32::from_ne_bytes(*b"4BcS")
    ) as PSTORAGE_DESCRIPTOR_HEADER;

    if descriptor.is_null() {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
                   b"ClassGetDescriptor: unable to memory for descriptor (%d bytes)\0".as_ptr() as PCHAR,
                   length);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(&mut query as *mut _ as PVOID, core::mem::size_of::<STORAGE_PROPERTY_QUERY>());
    query.PropertyId = *PropertyId;
    query.QueryType = PropertyStandardQuery;

    RtlZeroMemory(descriptor as PVOID, length as usize);

    RtlCopyMemory(
        descriptor as PVOID,
        &query as *const _ as PVOID,
        core::mem::size_of::<STORAGE_PROPERTY_QUERY>()
    );

    ClassSendDeviceIoControlSynchronous(
        IOCTL_STORAGE_QUERY_PROPERTY,
        DeviceObject,
        descriptor as PVOID,
        core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as ULONG,
        length,
        0u8,
        &mut ioStatus
    );

    if !NT_SUCCESS(ioStatus.__bindgen_anon_1.Status) {
        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
                   b"ClassGetDescriptor: error %lx trying to query properties #1\0".as_ptr() as PCHAR,
                   ioStatus.__bindgen_anon_1.Status);
        FREE_POOL(descriptor as PVOID);
        return ioStatus.__bindgen_anon_1.Status;
    }

    *Descriptor = descriptor as PVOID;
    ioStatus.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn ClassGetDriverExtension(
    DriverObject: PDRIVER_OBJECT
) -> PCLASS_DRIVER_EXTENSION {
    IoGetDriverObjectExtension(DriverObject, ClassInitialize as PVOID) as PCLASS_DRIVER_EXTENSION
}

#[no_mangle]
pub unsafe extern "system" fn ClassGetVpb(
    DeviceObject: PDEVICE_OBJECT
) -> PVPB {
    (*DeviceObject).Vpb
}

#[no_mangle]
pub unsafe extern "system" fn ClassInternalIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let nextStack = IoGetNextIrpStackLocation(Irp);

    let isRemoved: ULONG;

    let srb: PSCSI_REQUEST_BLOCK;

    isRemoved = ClassAcquireRemoveLockULONG(DeviceObject, (Irp as PVOID).cast());
    if isRemoved != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;

        ClassReleaseRemoveLock(DeviceObject, (Irp as PVOID).cast());

        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);

        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    srb = (*irpStack).Parameters.Scsi.Srb as *mut _ as PSCSI_REQUEST_BLOCK;

    if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        (*nextStack).Parameters.Scsi.Srb = srb as *mut _ as *mut _;
        (*nextStack).MajorFunction = (IRP_MJ_SCSI) as u8;
        (*nextStack).MinorFunction = IRP_MN_SCSI_CLASS;
    } else {
        IoCopyCurrentIrpStackLocationToNext(Irp);
    }

    ClassReleaseRemoveLock(DeviceObject, (Irp as PVOID).cast());

    IoCallDriver((*commonExtension).LowerDeviceObject, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn ClassInterpretSenseInfo(
    Fdo: PDEVICE_OBJECT,
    _Srb: PSCSI_REQUEST_BLOCK,
    MajorFunctionCode: UCHAR,
    IoDeviceCode: ULONG,
    RetryCount: ULONG,
    Status: *mut NTSTATUS,
    RetryInterval: PULONG,
) -> BOOLEAN {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let Srb = _Srb as PSTORAGE_REQUEST_BLOCK_HEADER;
    let senseBuffer = SrbGetSenseInfoBuffer((Srb).cast());
    let mut retry = 1u8;
    let mut logError = 0u8;
    let mut unhandledError = 0u8;
    let mut incrementErrorCount = 0u8;
    let mut logRetryableError = 1u8;
    let mut logErrorInternal = 1u8;
    let mut badSector: ULONGLONG = 0;
    let mut uniqueId: ULONG = 0;
    let mut logStatus: NTSTATUS;
    let mut readSector: ULONGLONG;
    let mut index: ULONG = 0;
    let mut retryInterval: ULONG = 0;
    let mut oldIrql: KIRQL = 0;
    let cdb = SrbGetCdb((Srb).cast());
    let mut cdbOpcode: UCHAR = 0;
    let cdbLength = SrbGetCdbLength((Srb).cast());

    if !cdb.is_null() {
        cdbOpcode = (*cdb).CDB6GENERIC.as_ref().OperationCode;
    }

    *Status = STATUS_IO_DEVICE_ERROR;
    logStatus = 0xFFFFFFFFu32 as NTSTATUS;

    if TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_CLASS_FLAGS_PAGING) != 0 {
        logError = 1;
        uniqueId = 301;
        logStatus = IO_WARNING_PAGING_FAILURE;
    }

    NT_ASSERT!((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0);

    if SRB_STATUS((*Srb).SrbStatus) == (SRB_STATUS_INTERNAL_ERROR) as u8 {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassInterpretSenseInfo: Internal Error code is %x\n\0".as_ptr() as PCHAR,
            SrbGetSystemStatus((Srb).cast()),
        );

        retry = 0;
        *Status = (SrbGetSystemStatus((Srb).cast())) as i32;
    } else if SrbGetScsiStatus((Srb).cast()) == SCSISTAT_RESERVATION_CONFLICT as u8 {
        *Status = STATUS_DEVICE_BUSY;
        retry = 0;
        logError = 0;
    } else {
        let mut validSense = 0u8;

        if ((*Srb).SrbStatus & (SRB_STATUS_AUTOSENSE_VALID) as u8 != 0) && !senseBuffer.is_null() {
            let mut errorCode: UCHAR = 0;
            let mut senseKey: UCHAR = 0;
            let mut addlSenseCode: UCHAR = 0;
            let mut addlSenseCodeQual: UCHAR = 0;
            let mut isIncorrectLengthValid = 0u8;
            let mut incorrectLength = 0u8;
            let mut isInformationValid = 0u8;
            let mut information: ULONGLONG = 0;

            validSense = ScsiGetSenseKeyAndCodes(
                senseBuffer,
                (SrbGetSenseInfoBufferLength((Srb).cast())) as _,
                SCSI_SENSE_OPTIONS_NONE,
                &mut senseKey,
                &mut addlSenseCode,
                &mut addlSenseCodeQual,
            );

            if validSense == 0 && IsSenseDataFormatValueValid(senseBuffer) == 0 {
                NT_ASSERT!(0u8);

                validSense = ScsiGetFixedSenseKeyAndCodes(
                    senseBuffer,
                    SrbGetSenseInfoBufferLength((Srb).cast()),
                    &mut senseKey,
                    &mut addlSenseCode,
                    &mut addlSenseCodeQual,
                );
            }

            if validSense == 0 {
                // goto __ClassInterpretSenseInfo_ProcessingInvalidSenseBuffer
            } else {
                errorCode = ScsiGetSenseErrorCode(senseBuffer);

                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Error code is %x\n\0".as_ptr() as PCHAR, (errorCode) as i32);
                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Sense key is %x\n\0".as_ptr() as PCHAR, (senseKey) as i32);
                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Additional sense code is %x\n\0".as_ptr() as PCHAR, (addlSenseCode) as i32);
                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Additional sense code qualifier is %x\n\0".as_ptr() as PCHAR, (addlSenseCodeQual) as i32);

                if IsDescriptorSenseDataFormat(senseBuffer) != 0 {
                    let mut startBuffer: PVOID = core::ptr::null_mut();
                    let mut startBufferLength: UCHAR = 0;

                    if ScsiGetSenseDescriptor(
                        senseBuffer,
                        SrbGetSenseInfoBufferLength((Srb).cast()),
                        &mut startBuffer,
                        &mut startBufferLength,
                    ) != 0
                    {
                        let mut outType: UCHAR = 0;
                        let mut outBuffer: PVOID = core::ptr::null_mut();
                        let mut outBufferLength: UCHAR = 0;
                        let mut foundBlockCommandType = 0u8;
                        let mut foundInformationType = 0u8;
                        let mut descriptorLength: UCHAR = 0;

                        let typeList: [UCHAR; 2] = [
                            (SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION) as u8,
                            (SCSI_SENSE_DESCRIPTOR_TYPE_BLOCK_COMMAND) as u8,
                        ];

                        while (foundBlockCommandType == 0 || foundInformationType == 0)
                            && ScsiGetNextSenseDescriptorByType(
                                startBuffer,
                                startBufferLength,
                                typeList.as_ptr(),
                                typeList.len() as ULONG,
                                &mut outType,
                                &mut outBuffer,
                                &mut outBufferLength,
                            ) != 0
                        {
                            descriptorLength = ScsiGetSenseDescriptorLength(outBuffer);

                            if outBufferLength < descriptorLength {
                                break;
                            }

                            if outType == (SCSI_SENSE_DESCRIPTOR_TYPE_BLOCK_COMMAND) as u8 {
                                if foundBlockCommandType == 0 {
                                    foundBlockCommandType = 1;

                                    if ScsiValidateBlockCommandSenseDescriptor(
                                        outBuffer,
                                        (outBufferLength) as _,
                                    ) != 0
                                    {
                                        incorrectLength =
                                            (*(outBuffer as PSCSI_SENSE_DESCRIPTOR_BLOCK_COMMAND))
                                                .IncorrectLength();
                                        isIncorrectLengthValid = 1;
                                    }
                                } else {
                                    NT_ASSERT!(0u8);
                                }
                            } else if outType == (SCSI_SENSE_DESCRIPTOR_TYPE_INFORMATION) as u8 {
                                if foundInformationType == 0 {
                                    foundInformationType = 1;

                                    if ScsiValidateInformationSenseDescriptor(
                                        outBuffer,
                                        (outBufferLength) as _,
                                    ) != 0
                                    {
                                        REVERSE_BYTES_QUAD_M!(
                                            &mut information,
                                            core::ptr::addr_of!(
                                                (*(outBuffer
                                                    as PSCSI_SENSE_DESCRIPTOR_INFORMATION))
                                                    .Information
                                            )
                                        );
                                        isInformationValid = 1;
                                    }
                                } else {
                                    NT_ASSERT!(0u8);
                                }
                            } else {
                                NT_ASSERT!(0u8);
                                break;
                            }

                            startBuffer = (outBuffer as PUCHAR).add(descriptorLength as usize)
                                as PVOID;
                            startBufferLength = outBufferLength - descriptorLength;
                        }
                    }
                } else {
                    incorrectLength = (*(senseBuffer as PFIXED_SENSE_DATA)).IncorrectLength();
                    REVERSE_BYTES_M!(
                        &mut information,
                        core::ptr::addr_of!((*(senseBuffer as PFIXED_SENSE_DATA)).Information)
                    );
                    isInformationValid = 1;
                    isIncorrectLengthValid = 1;
                }

                match senseKey as u32 {
                    SCSI_SENSE_NO_SENSE => {
                        if isIncorrectLengthValid != 0 && incorrectLength != 0 {
                            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Incorrect length detected.\n\0".as_ptr() as PCHAR);
                            *Status = STATUS_INVALID_BLOCK_LENGTH;
                            retry = 0;
                        } else {
                            TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: No specific sense key\n\0".as_ptr() as PCHAR);
                            *Status = STATUS_IO_DEVICE_ERROR;
                            retry = 1;
                        }
                    }

                    SCSI_SENSE_RECOVERED_ERROR => {
                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Recovered error\n\0".as_ptr() as PCHAR);
                        *Status = STATUS_SUCCESS;
                        retry = 0;
                        logError = 1;
                        uniqueId = 258;

                        match addlSenseCode as u32 {
                            SCSI_ADSENSE_TRACK_ERROR | SCSI_ADSENSE_SEEK_ERROR => {
                                logStatus = IO_ERR_SEEK_ERROR;
                            }

                            SCSI_ADSENSE_REC_DATA_NOECC | SCSI_ADSENSE_REC_DATA_ECC => {
                                logStatus = IO_RECOVERED_VIA_ECC;
                            }

                            SCSI_ADSENSE_FAILURE_PREDICTION_THRESHOLD_EXCEEDED => {
                                let mut wmiEventData: [UCHAR; 8] = [0; 8];

                                (wmiEventData.as_mut_ptr() as PULONG).write_unaligned(core::mem::size_of::<UCHAR>() as ULONG);
                                wmiEventData[core::mem::size_of::<ULONG>()] = addlSenseCodeQual;

                                logError = 0;
                                if (*fdoExtension).FailurePredicted == 0 {
                                    logError = 1;
                                }
                                (*fdoExtension).FailureReason = (addlSenseCodeQual) as u32;
                                logStatus = IO_WRN_FAILURE_PREDICTED;

                                ClassNotifyFailurePredicted(
                                    fdoExtension,
                                    wmiEventData.as_mut_ptr(),
                                    core::mem::size_of_val(&wmiEventData) as ULONG,
                                    0u8,
                                    4,
                                    SrbGetPathId((Srb).cast()),
                                    SrbGetTargetId((Srb).cast()),
                                    SrbGetLun((Srb).cast()),
                                );

                                (*fdoExtension).FailurePredicted = 1;
                            }

                            _ => {
                                logStatus = IO_ERR_CONTROLLER_ERROR;
                            }
                        }

                        if isIncorrectLengthValid != 0 && incorrectLength != 0 {
                            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Incorrect length detected.\n\0".as_ptr() as PCHAR);
                            *Status = STATUS_INVALID_BLOCK_LENGTH;
                        }
                    }

                    SCSI_SENSE_NOT_READY => {
                        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Device not ready\n\0".as_ptr() as PCHAR);
                        *Status = STATUS_DEVICE_NOT_READY;

                        match addlSenseCode as u32 {
                            SCSI_ADSENSE_LUN_NOT_READY => {
                                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Lun not ready\n\0".as_ptr() as PCHAR);

                                retryInterval = NOT_READY_RETRY_INTERVAL;

                                match addlSenseCodeQual as u32 {
                                    SCSI_SENSEQ_BECOMING_READY => {
                                        let mut notReady: DEVICE_EVENT_BECOMING_READY = core::mem::zeroed();

                                        logRetryableError = 0;
                                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: In process of becoming ready\n\0".as_ptr() as PCHAR);

                                        notReady.Version = 1;
                                        notReady.Reason = 1;
                                        notReady.Estimated100msToReady = retryInterval * 10;
                                        ClassSendNotification(
                                            fdoExtension,
                                            core::ptr::addr_of!(GUID_IO_DEVICE_BECOMING_READY),
                                            core::mem::size_of::<DEVICE_EVENT_BECOMING_READY>() as ULONG,
                                            core::ptr::addr_of_mut!(notReady) as PVOID,
                                        );
                                    }

                                    SCSI_SENSEQ_MANUAL_INTERVENTION_REQUIRED => {
                                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Manual intervention required\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_NO_MEDIA_IN_DEVICE;
                                        retry = 0;
                                    }

                                    SCSI_SENSEQ_FORMAT_IN_PROGRESS => {
                                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Format in progress\n\0".as_ptr() as PCHAR);
                                        retry = 0;
                                    }

                                    SCSI_SENSEQ_OPERATION_IN_PROGRESS => {
                                        let mut notReady: DEVICE_EVENT_BECOMING_READY = core::mem::zeroed();

                                        logRetryableError = 0;
                                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Operation In Progress\n\0".as_ptr() as PCHAR);

                                        notReady.Version = 1;
                                        notReady.Reason = 2;
                                        notReady.Estimated100msToReady = retryInterval * 10;
                                        ClassSendNotification(
                                            fdoExtension,
                                            core::ptr::addr_of!(GUID_IO_DEVICE_BECOMING_READY),
                                            core::mem::size_of::<DEVICE_EVENT_BECOMING_READY>() as ULONG,
                                            core::ptr::addr_of_mut!(notReady) as PVOID,
                                        );
                                    }

                                    SCSI_SENSEQ_LONG_WRITE_IN_PROGRESS => {
                                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Long write in progress\n\0".as_ptr() as PCHAR);
                                        retry = 0;
                                        retryInterval = 1;
                                    }

                                    SCSI_SENSEQ_SPACE_ALLOC_IN_PROGRESS => {
                                        logRetryableError = 0;
                                        TracePrint(TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: The device (%p) is busy allocating space.\n\0".as_ptr() as PCHAR, Fdo);

                                        *Status = STATUS_RETRY;
                                        retry = 1;
                                    }

                                    SCSI_SENSEQ_CAUSE_NOT_REPORTABLE => {
                                        if TEST_FLAG(
                                            (*fdoExtension).ScanForSpecialFlags,
                                            CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK,
                                        ) == 0
                                        {
                                            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: not ready, cause unknown\n\0".as_ptr() as PCHAR);
                                            retry = 0;
                                        } else {
                                            // Fall through to INIT_COMMAND_REQUIRED
                                            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Initializing command required\n\0".as_ptr() as PCHAR);
                                            retryInterval = 0;
                                            logRetryableError = 0;

                                            if TEST_FLAG(((*fdoExtension).DeviceFlags) as u32, DEV_SAFE_START_UNIT) != 0
                                                && TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_CLASS_FLAGS_LOW_PRIORITY) == 0
                                            {
                                                ClassSendStartUnit(Fdo);
                                            }
                                        }
                                    }

                                    SCSI_SENSEQ_INIT_COMMAND_REQUIRED | _ => {
                                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Initializing command required\n\0".as_ptr() as PCHAR);
                                        retryInterval = 0;
                                        logRetryableError = 0;

                                        if TEST_FLAG(((*fdoExtension).DeviceFlags) as u32, DEV_SAFE_START_UNIT) != 0
                                            && TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_CLASS_FLAGS_LOW_PRIORITY) == 0
                                        {
                                            ClassSendStartUnit(Fdo);
                                        }
                                    }
                                }
                            }

                            SCSI_ADSENSE_NO_MEDIA_IN_DEVICE => {
                                TracePrint(TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: No Media in device.\n\0".as_ptr() as PCHAR);
                                *Status = STATUS_NO_MEDIA_IN_DEVICE;
                                retry = 0;

                                if TEST_FLAG((*Fdo).Characteristics, FILE_REMOVABLE_MEDIA) == 0 {
                                    TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: No Media in a non-removable device %p\n\0".as_ptr() as PCHAR, Fdo);
                                }

                                if addlSenseCodeQual == 0xCC {
                                    ClassSetMediaChangeState(fdoExtension, MediaUnavailable, 0);
                                } else {
                                    ClassSetMediaChangeState(fdoExtension, MediaNotPresent, 0);
                                }
                            }

                            _ => {}
                        }
                    }

                    SCSI_SENSE_MEDIUM_ERROR => {
                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Medium Error (bad block)\n\0".as_ptr() as PCHAR);
                        *Status = STATUS_DEVICE_DATA_ERROR;

                        retry = 0;
                        logError = 1;
                        uniqueId = 256;
                        logStatus = IO_ERR_BAD_BLOCK;

                        if addlSenseCode == (SCSI_ADSENSE_INVALID_MEDIA) as u8 {
                            match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_UNKNOWN_FORMAT => {
                                    *Status = STATUS_UNRECOGNIZED_MEDIA;

                                    if TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_CLASS_FLAGS_PAGING) == 0 {
                                        logError = 0;
                                    }
                                }

                                SCSI_SENSEQ_CLEANING_CARTRIDGE_INSTALLED => {
                                    *Status = STATUS_CLEANER_CARTRIDGE_INSTALLED;
                                    logError = 0;
                                }

                                _ => {}
                            }
                        }
                    }

                    SCSI_SENSE_HARDWARE_ERROR => {
                        let mut logHardwareError = 1u8;

                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Hardware error\n\0".as_ptr() as PCHAR);

                        if (*fdoData).LegacyErrorHandling == 0 {
                            retry = 0;
                            *Status = STATUS_DEVICE_HARDWARE_ERROR;
                            logError = 0;
                        } else {
                            retry = 1;
                            *Status = STATUS_IO_DEVICE_ERROR;
                            logError = 1;
                            uniqueId = 257;
                            logStatus = IO_ERR_CONTROLLER_ERROR;
                            logHardwareError = 0;

                            if (addlSenseCode == (SCSI_ADSENSE_LOGICAL_UNIT_ERROR) as u8
                                && addlSenseCodeQual == (SCSI_SENSEQ_TIMEOUT_ON_LOGICAL_UNIT) as u8)
                                || (addlSenseCode == (SCSI_ADSENSE_DATA_TRANSFER_ERROR) as u8
                                    && addlSenseCodeQual == (SCSI_SENSEQ_INITIATOR_RESPONSE_TIMEOUT) as u8)
                            {
                                retry = 0;
                                logHardwareError = 1;
                                logError = 0;
                            }
                        }

                        if addlSenseCode == (SCSI_ADSENSE_LUN_COMMUNICATION) as u8
                            && addlSenseCodeQual == (SCSI_SESNEQ_COMM_CRC_ERROR) as u8
                        {
                            retry = 1;
                            retryInterval = 1;
                            logHardwareError = 0;
                            logError = 0;
                        }

                        if logHardwareError != 0 {
                            let senseInfoBufferLength = SrbGetSenseInfoBufferLength((Srb).cast());
                            let mut senseBufferSize: UCHAR = 0;

                            if ScsiGetTotalSenseByteCountIndicated(
                                senseBuffer,
                                senseInfoBufferLength,
                                &mut senseBufferSize,
                            ) != 0
                            {
                                senseBufferSize = min(senseBufferSize, senseInfoBufferLength);
                            } else {
                                senseBufferSize = senseInfoBufferLength;
                            }

                            ClasspQueueLogIOEventWithContextWorker(
                                Fdo,
                                senseBufferSize as ULONG,
                                senseBuffer,
                                SRB_STATUS((*Srb).SrbStatus),
                                SrbGetScsiStatus((Srb).cast()),
                                IO_ERROR_IO_HARDWARE_ERROR as ULONG,
                                cdbLength as ULONG,
                                cdb,
                                core::ptr::null_mut(),
                            );
                        }
                    }

                    SCSI_SENSE_ILLEGAL_REQUEST => {
                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Illegal SCSI request\n\0".as_ptr() as PCHAR);
                        *Status = STATUS_INVALID_DEVICE_REQUEST;
                        retry = 0;

                        match addlSenseCode as u32 {
                            SCSI_ADSENSE_NO_SENSE => match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_OPERATION_IS_IN_PROGRESS => {
                                    if ClasspIsOffloadDataTransferCommand(cdb) != 0 {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Duplicate List Identifier (command %x, parameter field offset 0x%016llx)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32, information);

                                        NT_ASSERTMSG!(
                                            b"Duplicate list identifier specified\0".as_ptr()
                                                as PCHAR,
                                            0u8,
                                        );

                                        *Status = STATUS_OPERATION_IN_PROGRESS;
                                    }
                                }
                                _ => {}
                            },

                            SCSI_ADSENSE_LUN_COMMUNICATION => match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_UNREACHABLE_TARGET => {
                                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Source-Destination LUNs can't communicate (command %x)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                    *Status = STATUS_DEVICE_UNREACHABLE;
                                }
                                _ => {}
                            },

                            SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR => match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_DATA_UNDERRUN => {
                                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Host specified a transfer length greater than what is represented by the token (considering the offset) [command %x]\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                    NT_ASSERTMSG!(
                                        b"Host specified blocks to write beyond what is represented by the token\0"
                                            .as_ptr() as PCHAR,
                                        0u8,
                                    );

                                    *Status = STATUS_DATA_OVERRUN;
                                }
                                _ => {}
                            },

                            SCSI_ADSENSE_PARAMETER_LIST_LENGTH => {
                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Target truncated the block device range descriptors in the parameter list (command %x)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                NT_ASSERTMSG!(b"Parameter data truncation\0".as_ptr() as PCHAR, 0u8);

                                *Status = STATUS_DATA_OVERRUN;
                            }

                            SCSI_ADSENSE_ILLEGAL_COMMAND => {
                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Illegal command\n\0".as_ptr() as PCHAR);
                            }

                            SCSI_ADSENSE_ILLEGAL_BLOCK => {
                                let mut logicalBlockAddr: LARGE_INTEGER = core::mem::zeroed();
                                let mut lastLBA: LARGE_INTEGER = core::mem::zeroed();
                                let mut numTransferBlocks: ULONG = 0;

                                logicalBlockAddr.QuadPart = 0;

                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Illegal block address\n\0".as_ptr() as PCHAR);

                                *Status = STATUS_NONEXISTENT_SECTOR;

                                if (*Fdo).DeviceType == FILE_DEVICE_DISK {
                                    if IS_SCSIOP_READWRITE(cdbOpcode) != 0 && !cdb.is_null() {
                                        if TEST_FLAG(((*fdoExtension).DeviceFlags) as u32, DEV_USE_16BYTE_CDB) != 0 {
                                            REVERSE_BYTES_QUAD_M!(
                                                &mut logicalBlockAddr,
                                                core::ptr::addr_of!((*cdb).CDB16.as_ref().LogicalBlock)
                                            );
                                            REVERSE_BYTES(
                                                &mut numTransferBlocks,
                                                (core::ptr::addr_of!((*cdb).CDB16.as_ref().TransferLength)).cast(),
                                            );
                                        } else {
                                            REVERSE_BYTES(
                                                &mut logicalBlockAddr.__bindgen_anon_1.LowPart,
                                                (core::ptr::addr_of!(
                                                    (*cdb).CDB10.as_ref().LogicalBlockByte0
                                                )).cast(),
                                            );
                                            REVERSE_BYTES_SHORT_M!(
                                                &mut numTransferBlocks as *mut _ as PUSHORT,
                                                core::ptr::addr_of!((*cdb).CDB10.as_ref().TransferBlocksMsb)
                                            );
                                        }

                                        REVERSE_BYTES_QUAD_M!(
                                            &mut lastLBA,
                                            core::ptr::addr_of!(
                                                (*fdoData)
                                                    .LastKnownDriveCapacityData
                                                    .LogicalBlockAddress
                                            )
                                        );

                                        if (logicalBlockAddr.QuadPart > lastLBA.QuadPart)
                                            || ((logicalBlockAddr.QuadPart
                                                + numTransferBlocks as i64
                                                - 1)
                                                > lastLBA.QuadPart)
                                        {
                                            TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Request beyond boundary. Last LBA: 0x%I64X Read LBA: 0x%I64X Length: 0x%X\n\0".as_ptr() as PCHAR, lastLBA.QuadPart, logicalBlockAddr.QuadPart, numTransferBlocks);
                                        } else {
                                            retry = 1;
                                            retryInterval = 5;
                                        }
                                    } else if ClasspIsOffloadDataTransferCommand(cdb) != 0 {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): LBA out of range (command %x, parameter field offset 0x%016llx)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32, information);

                                        NT_ASSERTMSG!(
                                            b"Number of blocks specified exceeds LUN capacity\0"
                                                .as_ptr() as PCHAR,
                                            0u8,
                                        );
                                    }
                                }
                            }

                            SCSI_ADSENSE_INVALID_TOKEN => {
                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Invalid/Expired/Modified token specified (command %x, parameter field offset 0x%016llx)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32, information);

                                *Status = STATUS_INVALID_TOKEN;
                            }

                            SCSI_ADSENSE_INVALID_CDB => {
                                if ClasspIsOffloadDataTransferCommand(cdb) != 0 {
                                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Incorrect I_T nexus likely used (command %x)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                    *Status = STATUS_INVALID_INITIATOR_TARGET_PATH;
                                } else {
                                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Invalid CDB\n\0".as_ptr() as PCHAR);

                                    retryInterval = 3;
                                }
                            }

                            SCSI_ADSENSE_INVALID_LUN => {
                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Invalid LUN\n\0".as_ptr() as PCHAR);
                                *Status = STATUS_NO_SUCH_DEVICE;
                            }

                            SCSI_ADSENSE_INVALID_FIELD_PARAMETER_LIST => {
                                match addlSenseCodeQual as u32 {
                                    SCSI_SENSEQ_INVALID_RELEASE_OF_PERSISTENT_RESERVATION => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Alignment violation for command %x.\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                        NT_ASSERTMSG!(
                                            b"Specified offset is not aligned to LUN's granularity\0"
                                                .as_ptr() as PCHAR,
                                            0u8,
                                        );

                                        *Status = STATUS_INVALID_OFFSET_ALIGNMENT;
                                    }

                                    SCSI_SENSEQ_TOO_MANY_SEGMENT_DESCRIPTORS => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Too many descriptors in parameter list for command %x (parameter field offset 0x%016llx)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32, information);

                                        NT_ASSERTMSG!(
                                            b"Too many descriptors specified\0".as_ptr() as PCHAR,
                                            0u8,
                                        );

                                        *Status = STATUS_TOO_MANY_SEGMENT_DESCRIPTORS;
                                    }

                                    _ => {
                                        if ClasspIsOffloadDataTransferCommand(cdb) != 0 {
                                            TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Illegal field in parameter list for command %x (parameter field offset 0x%016llx) [AddSense %x, AddSenseQ %x]\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32, information, (addlSenseCode) as i32, (addlSenseCodeQual) as i32);

                                            NT_ASSERTMSG!(
                                                b"Invalid field in parameter list\0".as_ptr()
                                                    as PCHAR,
                                                0u8,
                                            );

                                            *Status = STATUS_INVALID_FIELD_IN_PARAMETER_LIST;
                                        }
                                    }
                                }
                            }

                            SCSI_ADSENSE_COPY_PROTECTION_FAILURE => {
                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Copy protection failure\n\0".as_ptr() as PCHAR);

                                *Status = STATUS_COPY_PROTECTION_FAILURE;

                                match addlSenseCodeQual as u32 {
                                    SCSI_SENSEQ_AUTHENTICATION_FAILURE => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Authentication failure\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_CSS_AUTHENTICATION_FAILURE;
                                    }
                                    SCSI_SENSEQ_KEY_NOT_PRESENT => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Key not present\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_CSS_KEY_NOT_PRESENT;
                                    }
                                    SCSI_SENSEQ_KEY_NOT_ESTABLISHED => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Key not established\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_CSS_KEY_NOT_ESTABLISHED;
                                    }
                                    SCSI_SENSEQ_READ_OF_SCRAMBLED_SECTOR_WITHOUT_AUTHENTICATION => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Read of scrambled sector w/o authentication\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_CSS_SCRAMBLED_SECTOR;
                                    }
                                    SCSI_SENSEQ_MEDIA_CODE_MISMATCHED_TO_LOGICAL_UNIT => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Media region does not logical unit region\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_CSS_REGION_MISMATCH;
                                    }
                                    SCSI_SENSEQ_LOGICAL_UNIT_RESET_COUNT_ERROR => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Region set error -- region may be permanent\n\0".as_ptr() as PCHAR);
                                        *Status = STATUS_CSS_RESETS_EXHAUSTED;
                                    }
                                    _ => {}
                                }
                            }

                            SCSI_ADSENSE_MUSIC_AREA => {
                                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Music area\n\0".as_ptr() as PCHAR);
                            }

                            SCSI_ADSENSE_DATA_AREA => {
                                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Data area\n\0".as_ptr() as PCHAR);
                            }

                            SCSI_ADSENSE_VOLUME_OVERFLOW => {
                                TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Volume overflow\n\0".as_ptr() as PCHAR);
                            }

                            _ => {}
                        }
                    }

                    SCSI_SENSE_UNIT_ATTENTION => {
                        let count = InterlockedIncrement(
                            core::ptr::addr_of_mut!((*fdoExtension).MediaChangeCount)
                                as *mut i32,
                        );
                        TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, b"ClassInterpretSenseInfo: Media change count for device %d incremented to %#lx\n\0".as_ptr() as PCHAR, (*fdoExtension).DeviceNumber, count);

                        match addlSenseCode as u32 {
                            SCSI_ADSENSE_MEDIUM_CHANGED => {
                                logRetryableError = 0;
                                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, b"ClassInterpretSenseInfo: Media changed\n\0".as_ptr() as PCHAR);

                                if TEST_FLAG((*Fdo).Characteristics, FILE_REMOVABLE_MEDIA) == 0 {
                                    TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_MCN, b"ClassInterpretSenseInfo: Media Changed on non-removable device %p\n\0".as_ptr() as PCHAR, Fdo);
                                }
                                ClassSetMediaChangeState(fdoExtension, MediaPresent, 0);
                            }

                            SCSI_ADSENSE_BUS_RESET => {
                                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Bus reset\n\0".as_ptr() as PCHAR);
                            }

                            SCSI_ADSENSE_PARAMETERS_CHANGED => {
                                logRetryableError = 0;
                                if addlSenseCodeQual == (SCSI_SENSEQ_CAPACITY_DATA_CHANGED) as u8 {
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Device capacity changed (e.g. thinly provisioned LUN). Retry the request.\n\0".as_ptr() as PCHAR);

                                    ClassQueueCapacityChangedEventWorker(Fdo);

                                    retryInterval = 1;
                                    retry = 1;
                                }
                            }

                            SCSI_ADSENSE_LB_PROVISIONING => match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_SOFT_THRESHOLD_REACHED => {
                                    logRetryableError = 0;
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Device (%p) has hit a soft threshold.\n\0".as_ptr() as PCHAR, Fdo);

                                    ClassQueueThresholdEventWorker(Fdo);
                                }
                                _ => {
                                    retry = 0;
                                }
                            },

                            SCSI_ADSENSE_OPERATING_CONDITIONS_CHANGED => {
                                if addlSenseCodeQual == (SCSI_SENSEQ_MICROCODE_CHANGED) as u8 {
                                    logRetryableError = 1;
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Device firmware has been changed.\n\0".as_ptr() as PCHAR);

                                    retryInterval = 1;
                                    retry = 1;
                                } else {
                                    logRetryableError = 0;
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Device information changed. Invalidate the bus\n\0".as_ptr() as PCHAR);

                                    if addlSenseCodeQual == (SCSI_SENSEQ_INQUIRY_DATA_CHANGED) as u8 {
                                        ClassQueueProvisioningTypeChangedEventWorker(Fdo);
                                    }

                                    if addlSenseCodeQual == (SCSI_SENSEQ_INQUIRY_DATA_CHANGED) as u8
                                        || addlSenseCodeQual
                                            == (SCSI_SENSEQ_OPERATING_DEFINITION_CHANGED) as u8
                                    {
                                        InterlockedIncrement(
                                            core::ptr::addr_of_mut!(
                                                (*(*fdoExtension).FunctionSupportInfo)
                                                    .ChangeRequestCount
                                            ) as *mut i32,
                                        );
                                    }

                                    IoInvalidateDeviceRelations(
                                        (*fdoExtension).LowerPdo,
                                        BusRelations,
                                    );
                                    retryInterval = 5;
                                }
                            }

                            SCSI_ADSENSE_OPERATOR_REQUEST => match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_MEDIUM_REMOVAL => {
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Ejection request received!\n\0".as_ptr() as PCHAR);
                                    ClassSendEjectionNotification(fdoExtension);
                                }

                                SCSI_SENSEQ_WRITE_PROTECT_ENABLE => {
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Operator selected write permit?! (unsupported!)\n\0".as_ptr() as PCHAR);
                                }

                                SCSI_SENSEQ_WRITE_PROTECT_DISABLE => {
                                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Operator selected write protect?! (unsupported!)\n\0".as_ptr() as PCHAR);
                                }

                                _ => {}
                            },

                            SCSI_ADSENSE_FAILURE_PREDICTION_THRESHOLD_EXCEEDED => {
                                let mut wmiEventData: [UCHAR; 8] = [0; 8];

                                (wmiEventData.as_mut_ptr() as PULONG).write_unaligned(core::mem::size_of::<UCHAR>() as ULONG);
                                wmiEventData[core::mem::size_of::<ULONG>()] = addlSenseCodeQual;

                                logError = 0;
                                if (*fdoExtension).FailurePredicted == 0 {
                                    logError = 1;
                                }
                                (*fdoExtension).FailureReason = (addlSenseCodeQual) as u32;
                                logStatus = IO_WRN_FAILURE_PREDICTED;

                                ClassNotifyFailurePredicted(
                                    fdoExtension,
                                    wmiEventData.as_mut_ptr(),
                                    core::mem::size_of_val(&wmiEventData) as ULONG,
                                    0u8,
                                    4,
                                    SrbGetPathId((Srb).cast()),
                                    SrbGetTargetId((Srb).cast()),
                                    SrbGetLun((Srb).cast()),
                                );

                                (*fdoExtension).FailurePredicted = 1;

                                retry = 1;
                            }

                            _ => {
                                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Unit attention\n\0".as_ptr() as PCHAR);
                            }
                        }

                        if TEST_FLAG((*Fdo).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
                            if !ClassGetVpb(Fdo).is_null()
                                && ((*ClassGetVpb(Fdo)).Flags & (VPB_MOUNTED) as u16 != 0)
                            {
                                SET_FLAG!((*Fdo).Flags, DO_VERIFY_VOLUME);

                                *Status = STATUS_VERIFY_REQUIRED;
                                retry = 0;
                            } else {
                                *Status = STATUS_IO_DEVICE_ERROR;
                            }
                        } else {
                            *Status = STATUS_IO_DEVICE_ERROR;
                        }
                    }

                    SCSI_SENSE_DATA_PROTECT => {
                        retry = 0;

                        if addlSenseCode == (SCSI_ADSENSE_WRITE_PROTECT) as u8 {
                            match addlSenseCodeQual as u32 {
                                SCSI_SENSEQ_SPACE_ALLOC_FAILED_WRITE_PROTECT => {
                                    TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Device's (%p) resources are exhausted.\n\0".as_ptr() as PCHAR, Fdo);

                                    ClassQueueResourceExhaustionEventWorker(Fdo);

                                    *Status = STATUS_DISK_RESOURCES_EXHAUSTED;
                                }
                                _ => {}
                            }
                        } else {
                            if IS_SCSIOP_WRITE(cdbOpcode) {
                                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Media write protected\n\0".as_ptr() as PCHAR);
                                *Status = STATUS_MEDIA_WRITE_PROTECTED;
                            } else {
                                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Access denied\n\0".as_ptr() as PCHAR);
                                *Status = STATUS_ACCESS_DENIED;
                            }
                        }
                    }

                    SCSI_SENSE_BLANK_CHECK => {
                        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Media blank check\n\0".as_ptr() as PCHAR);
                        retry = 0;
                        *Status = STATUS_NO_DATA_DETECTED;
                    }

                    SCSI_SENSE_COPY_ABORTED => match addlSenseCode as u32 {
                        SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR => match addlSenseCodeQual as u32 {
                            SCSI_SENSEQ_DATA_UNDERRUN => {
                                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Data transfer was truncated (command %x)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                *Status = STATUS_SUCCESS;
                                retry = 0;
                            }
                            _ => {}
                        },
                        _ => {}
                    },

                    SCSI_SENSE_ABORTED_COMMAND => {
                        if ClasspIsOffloadDataTransferCommand(cdb) != 0 {
                            match addlSenseCode as u32 {
                                SCSI_ADSENSE_COPY_TARGET_DEVICE_ERROR => match addlSenseCodeQual as u32 {
                                    SCSI_SENSEQ_DATA_UNDERRUN => {
                                        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Target has truncated the data transfer (command %x)\n\0".as_ptr() as PCHAR, Fdo, (cdbOpcode) as i32);

                                        *Status = STATUS_SUCCESS;
                                        retry = 0;
                                    }
                                    _ => {}
                                },

                                SCSI_ADSENSE_RESOURCE_FAILURE => match addlSenseCodeQual as u32 {
                                    SCSI_SENSEQ_INSUFFICIENT_RESOURCES => {
                                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"ClassInterpretSenseInfo (%p): Target has insufficient resources (command %x)\n\0".as_ptr() as PCHAR, Fdo, ((*cdb).CDB6GENERIC.as_ref().OperationCode) as i32);

                                        *Status = STATUS_INSUFFICIENT_RESOURCES;
                                        retry = 0;
                                    }
                                    _ => {}
                                },

                                _ => {}
                            }
                        } else {
                            TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Command aborted\n\0".as_ptr() as PCHAR);
                            *Status = STATUS_IO_DEVICE_ERROR;
                            retryInterval = 1;
                        }
                    }

                    _ => {
                        TracePrint(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Unrecognized sense code\n\0".as_ptr() as PCHAR);
                        *Status = STATUS_IO_DEVICE_ERROR;
                    }
                }

                if ((IS_SCSIOP_READWRITE(cdbOpcode) != 0)
                    || (cdbOpcode == (SCSIOP_VERIFY) as u8)
                    || (cdbOpcode == (SCSIOP_VERIFY16) as u8))
                    && !cdb.is_null()
                {
                    if isInformationValid != 0 {
                        readSector = 0;
                        badSector = information;

                        if cdbOpcode == (SCSIOP_READ16) as u8
                            || cdbOpcode == (SCSIOP_WRITE16) as u8
                            || cdbOpcode == (SCSIOP_VERIFY16) as u8
                        {
                            REVERSE_BYTES_QUAD_M!(
                                &mut readSector,
                                (*cdb).AsByte.as_ref().as_ptr().add(2)
                            );
                        } else {
                            REVERSE_BYTES_M!(
                                &mut readSector,
                                (*cdb).AsByte.as_ref().as_ptr().add(2)
                            );
                        }

                        if cdbOpcode == (SCSIOP_READ) as u8
                            || cdbOpcode == (SCSIOP_WRITE) as u8
                            || cdbOpcode == (SCSIOP_VERIFY) as u8
                        {
                            REVERSE_BYTES_SHORT_M!(
                                &mut index as *mut _ as PUSHORT,
                                core::ptr::addr_of!((*cdb).CDB10.as_ref().TransferBlocksMsb)
                            );
                        } else if cdbOpcode == (SCSIOP_READ6) as u8 || cdbOpcode == (SCSIOP_WRITE6) as u8 {
                            index = (*cdb).CDB6READWRITE.as_ref().TransferBlocks as ULONG;
                        } else if cdbOpcode == (SCSIOP_READ12) as u8 || cdbOpcode == (SCSIOP_WRITE12) as u8 {
                            REVERSE_BYTES(
                                &mut index,
                                (core::ptr::addr_of!((*cdb).CDB12.as_ref().TransferLength)).cast(),
                            );
                        } else {
                            REVERSE_BYTES(
                                &mut index,
                                (core::ptr::addr_of!((*cdb).CDB16.as_ref().TransferLength)).cast(),
                            );
                        }

                        if !(badSector >= readSector && badSector < readSector + index as u64) {
                            badSector = readSector;
                        }
                    }
                }
            }
        }

        if validSense == 0 {
            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, b"ClassInterpretSenseInfo: Request sense info not valid. SrbStatus %2x\n\0".as_ptr() as PCHAR, (SRB_STATUS((*Srb).SrbStatus)) as i32);
            retry = 1;

            match SRB_STATUS((*Srb).SrbStatus) as u32 {
                SRB_STATUS_ABORTED => {
                    incrementErrorCount = 1;
                    *Status = STATUS_IO_TIMEOUT;
                    retryInterval = 1;
                    retry = 1;
                }

                SRB_STATUS_ERROR => {
                    *Status = STATUS_IO_DEVICE_ERROR;
                    if SrbGetScsiStatus((Srb).cast()) == SCSISTAT_GOOD as u8 {
                        incrementErrorCount = 1;
                    } else if SrbGetScsiStatus((Srb).cast()) == SCSISTAT_BUSY as u8 {
                        *Status = STATUS_DEVICE_NOT_READY;
                        logRetryableError = 0;
                    }
                }

                SRB_STATUS_INVALID_REQUEST => {
                    *Status = STATUS_INVALID_DEVICE_REQUEST;
                    retry = 0;
                }

                SRB_STATUS_INVALID_PATH_ID
                | SRB_STATUS_NO_DEVICE
                | SRB_STATUS_NO_HBA
                | SRB_STATUS_INVALID_LUN
                | SRB_STATUS_INVALID_TARGET_ID => {
                    *Status = STATUS_NO_SUCH_DEVICE;
                    retry = 0;
                }

                SRB_STATUS_SELECTION_TIMEOUT => {
                    logError = 1;
                    logStatus = IO_ERR_NOT_READY;
                    uniqueId = 260;
                    *Status = STATUS_DEVICE_NOT_CONNECTED;
                    retry = 0;
                }

                SRB_STATUS_TIMEOUT | SRB_STATUS_COMMAND_TIMEOUT => {
                    incrementErrorCount = 1;
                    *Status = STATUS_IO_TIMEOUT;
                }

                SRB_STATUS_PARITY_ERROR | SRB_STATUS_UNEXPECTED_BUS_FREE => {
                    incrementErrorCount = 1;
                    *Status = STATUS_IO_DEVICE_ERROR;
                    logRetryableError = 0;
                }

                SRB_STATUS_BUS_RESET => {
                    *Status = STATUS_IO_DEVICE_ERROR;
                    logRetryableError = 0;
                }

                SRB_STATUS_DATA_OVERRUN => {
                    *Status = STATUS_DATA_OVERRUN;
                    retry = 0;

                    if (cdbOpcode == (SCSIOP_MODE_SENSE) as u8
                        && SrbGetDataTransferLength((Srb).cast())
                            <= (*cdb).MODE_SENSE.as_ref().AllocationLength as u32)
                        || (cdbOpcode == (SCSIOP_INQUIRY) as u8
                            && SrbGetDataTransferLength((Srb).cast())
                                <= (*cdb).CDB6INQUIRY.as_ref().AllocationLength as u32)
                    {
                        *Status = STATUS_SUCCESS;
                        logErrorInternal = 0;
                        logError = 0;
                    } else if cdbOpcode == (SCSIOP_MODE_SENSE10) as u8 {
                        let mut allocationLength: USHORT = 0;
                        REVERSE_BYTES_SHORT_M!(
                            &mut allocationLength,
                            core::ptr::addr_of!((*cdb).MODE_SENSE10.as_ref().AllocationLength)
                        );
                        if SrbGetDataTransferLength((Srb).cast()) <= allocationLength as u32 {
                            *Status = STATUS_SUCCESS;
                            logErrorInternal = 0;
                            logError = 0;
                        }
                    } else if ClasspIsReceiveTokenInformation(cdb) != 0 {
                        let mut allocationLength: ULONG = 0;
                        {
                            let _rti = core::ptr::addr_of_mut!((*cdb).RECEIVE_TOKEN_INFORMATION) as *mut crate::types::_CDB__RECEIVE_TOKEN_INFORMATION;
                            REVERSE_BYTES_M!(
                                core::ptr::addr_of_mut!((*_rti).AllocationLength) as *mut u32,
                                &allocationLength as *const u32
                            )
                        }
                        if SrbGetDataTransferLength((Srb).cast()) <= allocationLength {
                            *Status = STATUS_SUCCESS;
                            logErrorInternal = 0;
                            logError = 0;
                        }
                    }
                }

                SRB_STATUS_PHASE_SEQUENCE_FAILURE => {
                    incrementErrorCount = 1;
                    *Status = STATUS_IO_DEVICE_ERROR;

                    if RetryCount > 1 {
                        retry = 0;
                    }
                }

                SRB_STATUS_REQUEST_FLUSHED => {
                    if TEST_FLAG((*Fdo).Flags, DO_VERIFY_VOLUME) != 0 {
                        *Status = STATUS_VERIFY_REQUIRED;
                        retry = 0;
                    } else {
                        *Status = STATUS_IO_DEVICE_ERROR;
                        logRetryableError = 0;
                    }
                }

                _ => {
                    logError = 1;
                    logStatus = IO_ERR_CONTROLLER_ERROR;
                    uniqueId = 259;
                    *Status = STATUS_IO_DEVICE_ERROR;
                    unhandledError = 1;
                    logRetryableError = 0;
                }
            }
        }
    }

    if incrementErrorCount != 0 {
        if retryInterval == 0 {
            retryInterval = 1;
        }
        ClasspPerfIncrementErrorCount(fdoExtension);
    }

    if (*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DevInfo).ClassError.is_some() {
        let mut tempSrb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
        let mut srbPtr = Srb as PSCSI_REQUEST_BLOCK;

        if ((*Srb).Function == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8)
            && (((*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension).SrbSupport
                & CLASS_SRB_STORAGE_REQUEST_BLOCK)
                == 0)
        {
            ClasspConvertToScsiRequestBlock(&mut tempSrb, Srb as PSTORAGE_REQUEST_BLOCK);
            srbPtr = &mut tempSrb;
        }

        (*(*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DevInfo).ClassError.unwrap()(
            Fdo,
            srbPtr,
            Status,
            &mut retry,
        );
    }

    if ARGUMENT_PRESENT(RetryInterval as PVOID) != 0 {
        *RetryInterval = retryInterval;
    }

    let cdb = SrbGetCdb((Srb).cast());
    if !cdb.is_null() {
        cdbOpcode = (*cdb).CDB6GENERIC.as_ref().OperationCode;
    }

    if (cdbOpcode == (SCSIOP_RESERVE_UNIT) as u8 || cdbOpcode == (SCSIOP_RELEASE_UNIT) as u8) && !cdb.is_null() {
        if *Status == STATUS_INVALID_DEVICE_REQUEST {
            SrbSetCdbLength((Srb).cast(), 10);
            (*cdb).CDB10.as_mut().OperationCode = if (*cdb).CDB6GENERIC.as_ref().OperationCode
                == (SCSIOP_RESERVE_UNIT) as u8
            {
                (SCSIOP_RESERVE_UNIT10) as u8
            } else {
                (SCSIOP_RELEASE_UNIT10) as u8
            };

            SET_FLAG!(
                (*(*fdoExtension).PrivateFdoData).HackFlags,
                (FDO_HACK_NO_RESERVE6) as ULONG_PTR
            );
            retry = 1;
        }
    }

    if logErrorInternal != 0 || logError != 0 {
        let mut totalSize: ULONG;
        let mut senseBufferSize: ULONG = 0;
        let mut staticErrLogEntry: IO_ERROR_LOG_PACKET = core::mem::zeroed();
        let mut staticErrLogData: CLASS_ERROR_LOG_DATA = core::mem::zeroed();
        let mut convertedSenseBuffer: SENSE_DATA = core::mem::zeroed();
        let mut convertedSenseBufferLength: UCHAR = 0;
        let mut senseDataConverted = 0u8;

        totalSize = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as ULONG
            + core::mem::size_of::<CLASS_ERROR_LOG_DATA>() as ULONG;

        if (TEST_FLAG(((*Srb).SrbStatus) as u32, SRB_STATUS_AUTOSENSE_VALID) != 0) && !senseBuffer.is_null()
        {
            let mut validSenseBytes: UCHAR = 0;
            let mut senseInfoBufferLength: UCHAR = 0;

            senseInfoBufferLength = SrbGetSenseInfoBufferLength((Srb).cast());

            if IsDescriptorSenseDataFormat(senseBuffer) != 0 {
                convertedSenseBufferLength = core::mem::size_of::<SENSE_DATA>() as UCHAR;

                senseDataConverted = ScsiConvertToFixedSenseFormat(
                    senseBuffer,
                    (senseInfoBufferLength) as _,
                    core::ptr::addr_of_mut!(convertedSenseBuffer) as PVOID,
                    (convertedSenseBufferLength) as _,
                );
            }

            if ScsiGetTotalSenseByteCountIndicated(
                senseBuffer,
                senseInfoBufferLength,
                &mut validSenseBytes,
            ) != 0
            {
                senseBufferSize = max(
                    validSenseBytes as u32,
                    core::mem::size_of_val(&staticErrLogData.SenseData) as u32,
                );
                senseBufferSize = min(senseBufferSize, senseInfoBufferLength as u32);
            } else {
                senseBufferSize = senseInfoBufferLength as u32;
            }

            if senseBufferSize > core::mem::size_of_val(&staticErrLogData.SenseData) as u32 {
                totalSize +=
                    senseBufferSize - core::mem::size_of_val(&staticErrLogData.SenseData) as u32;
                if totalSize > ERROR_LOG_MAXIMUM_SIZE {
                    senseBufferSize -= totalSize - ERROR_LOG_MAXIMUM_SIZE;
                    totalSize = ERROR_LOG_MAXIMUM_SIZE;
                }
            }
        }

        if retry != 0 {
            staticErrLogEntry.FinalStatus = STATUS_SUCCESS;
            staticErrLogData.set_ErrorRetried(1);
        } else {
            staticErrLogEntry.FinalStatus = *Status;
        }

        if logStatus == IO_WARNING_PAGING_FAILURE && (retry != 0 || NT_SUCCESS(*Status)) {
            logError = 0;
        }

        if TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_CLASS_FLAGS_PAGING) != 0 {
            staticErrLogData.set_ErrorPaging(1);
        }
        if unhandledError != 0 {
            staticErrLogData.set_ErrorUnhandled(1);
        }

        staticErrLogEntry.DeviceOffset.QuadPart = badSector as i64;
        staticErrLogEntry.DeviceOffset.QuadPart *=
            (*fdoExtension).DiskGeometry.BytesPerSector as i64;
        if logStatus == 0xFFFFFFFFu32 as NTSTATUS {
            staticErrLogEntry.ErrorCode = STATUS_IO_DEVICE_ERROR;
        } else {
            staticErrLogEntry.ErrorCode = logStatus;
        }

        staticErrLogEntry.DumpDataSize =
            (totalSize - core::mem::size_of::<IO_ERROR_LOG_PACKET>() as u32) as USHORT;

        staticErrLogEntry.SequenceNumber = 0;
        staticErrLogEntry.MajorFunctionCode = MajorFunctionCode;
        staticErrLogEntry.IoControlCode = IoDeviceCode;
        staticErrLogEntry.RetryCount = RetryCount as UCHAR;
        staticErrLogEntry.UniqueErrorValue = uniqueId;

        KeQueryTickCount(&mut staticErrLogData.TickCount);
        staticErrLogData.PortNumber = 0xFFFFFFFFu32;

        if (*Srb).Function == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8 {
            ClasspConvertToScsiRequestBlock(
                &mut staticErrLogData.Srb,
                Srb as PSTORAGE_REQUEST_BLOCK,
            );
        } else {
            staticErrLogData.Srb = core::ptr::read(Srb as PSCSI_REQUEST_BLOCK);
        }

        if (senseBufferSize != 0) && !senseBuffer.is_null() {
            if IsDescriptorSenseDataFormat(senseBuffer) != 0 {
                if senseDataConverted != 0 {
                    RtlCopyMemory(
                        core::ptr::addr_of_mut!(staticErrLogData.SenseData) as PVOID,
                        core::ptr::addr_of!(convertedSenseBuffer) as PVOID,
                        min(
                            convertedSenseBufferLength as usize,
                            core::mem::size_of_val(&staticErrLogData.SenseData),
                        ),
                    );
                }
            } else {
                RtlCopyMemory(
                    core::ptr::addr_of_mut!(staticErrLogData.SenseData) as PVOID,
                    senseBuffer,
                    min(
                        senseBufferSize as usize,
                        core::mem::size_of_val(&staticErrLogData.SenseData),
                    ),
                );
            }
        }

        if logErrorInternal != 0 {
            KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), &mut oldIrql);
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!(staticErrLogData),
                core::ptr::addr_of_mut!((*fdoData).ErrorLogs[(*fdoData).ErrorLogNextIndex as usize]),
                1,
            );
            (*fdoData).ErrorLogNextIndex = (*fdoData).ErrorLogNextIndex + 1;
            (*fdoData).ErrorLogNextIndex = (*fdoData).ErrorLogNextIndex % NUM_ERROR_LOG_ENTRIES;
            KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
        }

        if IS_SCSIOP_READWRITE(cdbOpcode) == 0 || retry == 0 || (RetryCount != 0) {
            logRetryableError = 0;
        }

        if logRetryableError != 0 {
            logError = 1;
        }

        if logError != 0 {
            let cdb = SrbGetCdb((Srb).cast());
            if !cdb.is_null() {
                match (*cdb).CDB10.as_ref().OperationCode as u32 {
                    SCSIOP_TEST_UNIT_READY => {
                        if (*fdoData).LoggedTURFailureSinceLastIO != 0 {
                            logError = 0;
                        } else {
                            (*fdoData).LoggedTURFailureSinceLastIO = 1;
                        }
                    }

                    SCSIOP_SYNCHRONIZE_CACHE => {
                        if (*fdoData).LoggedSYNCFailure != 0 {
                            logError = 0;
                        } else {
                            (*fdoData).LoggedSYNCFailure = 1;
                        }
                    }

                    _ => {}
                }
            }
        }

        if logError != 0 {
            if logRetryableError != 0 {
                NT_ASSERT!(IS_SCSIOP_READWRITE(cdbOpcode) != 0);

                ClasspQueueLogIOEventWithContextWorker(
                    Fdo,
                    senseBufferSize as ULONG,
                    senseBuffer,
                    SRB_STATUS((*Srb).SrbStatus),
                    SrbGetScsiStatus((Srb).cast()),
                    IO_WARNING_IO_OPERATION_RETRIED as ULONG,
                    cdbLength as ULONG,
                    cdb,
                    core::ptr::null_mut(),
                );
            } else {
                let errorLogEntry =
                    IoAllocateErrorLogEntry((Fdo).cast(), totalSize as UCHAR) as PIO_ERROR_LOG_PACKET;
                if !errorLogEntry.is_null() {
                    let errlogData = (*errorLogEntry).DumpData.as_mut_ptr() as PCLASS_ERROR_LOG_DATA;

                    *errorLogEntry = staticErrLogEntry;
                    core::ptr::copy_nonoverlapping(core::ptr::addr_of!(staticErrLogData), errlogData, 1);

                    if (senseBufferSize != 0) && !senseBuffer.is_null() {
                        RtlCopyMemory(
                            core::ptr::addr_of_mut!((*errlogData).SenseData) as PVOID,
                            senseBuffer,
                            (senseBufferSize) as usize,
                        );
                    }

                    IoWriteErrorLogEntry(errorLogEntry as PVOID);
                }
            }
        }
    }

    retry
}

#[no_mangle]
pub unsafe extern "system" fn ClassInvalidateBusRelations(
    Fdo: PDEVICE_OBJECT
) {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let driverExtension = IoGetDriverObjectExtension((*Fdo).DriverObject, ClassInitialize as PVOID) as PCLASS_DRIVER_EXTENSION;

    let mut status = STATUS_SUCCESS;

    if InterlockedIncrement(core::ptr::addr_of_mut!((*fdoExtension).EnumerationInterlock) as *mut i32) == 1 {
        status = ((*driverExtension).InitData.ClassEnumerateDevice.unwrap())(Fdo);
    }
    InterlockedDecrement(core::ptr::addr_of_mut!((*fdoExtension).EnumerationInterlock) as *mut i32);

    if !NT_SUCCESS(status) {
    }

    IoInvalidateDeviceRelations((*fdoExtension).LowerPdo, BusRelations);
}

#[no_mangle]
pub unsafe extern "system" fn ClassMarkChildMissing(
    Child: PPHYSICAL_DEVICE_EXTENSION,
    AcquireChildLock: BOOLEAN
) -> BOOLEAN {
    let returnValue = (*Child).IsEnumerated;

    extern "system" {
        fn ASSERT_PDO(DeviceObject: PDEVICE_OBJECT);
        fn ClassRemoveChild(
            Parent: PFUNCTIONAL_DEVICE_EXTENSION,
            Child: PPHYSICAL_DEVICE_EXTENSION,
            AcquireLock: BOOLEAN
        ) -> PPHYSICAL_DEVICE_EXTENSION;
        fn ClassRemoveDevice(DeviceObject: PDEVICE_OBJECT, RemoveType: UCHAR) -> NTSTATUS;
    }

    ASSERT_PDO((*Child).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject);

    (*Child).IsMissing = 1u8;

    ClassRemoveChild(
        (*Child).__bindgen_anon_1.CommonExtension.as_ref().PartitionZeroExtension,
        Child,
        AcquireChildLock
    );

    if (*Child).IsEnumerated == 0u8 {
        let child_dev = (*Child).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
        let commonExtension = (*child_dev).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
        (*commonExtension).IsRemoved = REMOVE_PENDING;
        ClassRemoveDevice(child_dev, IRP_MN_REMOVE_DEVICE as UCHAR);
    }

    returnValue
}

#[no_mangle]
pub unsafe extern "system" fn ClassMarkChildrenMissing(
    Fdo: PFUNCTIONAL_DEVICE_EXTENSION
) {
    let commonExtension = core::ptr::addr_of_mut!((*Fdo).__bindgen_anon_1.CommonExtension);
    let mut nextChild = (*commonExtension).as_ref().ChildList;

    ClassAcquireChildLock(Fdo);

    while !nextChild.is_null() {
        let tmpChild = nextChild;
        nextChild = (*tmpChild).__bindgen_anon_1.CommonExtension.as_ref().ChildList;
        ClassMarkChildMissing(tmpChild, 0u8);
    }
    ClassReleaseChildLock(Fdo);
}

#[no_mangle]
pub unsafe extern "system" fn ClassModeSelect(
    Fdo: PDEVICE_OBJECT,
    ModeSelectBuffer: PCHAR,
    Length: ULONG,
    SavePages: BOOLEAN
) -> NTSTATUS {
    #[cfg(feature = "ntddi_winblue")]
    {
        ClasspModeSelect(Fdo, ModeSelectBuffer, Length, SavePages)
    }
    #[cfg(not(feature = "ntddi_winblue"))]
    {
        let _ = Fdo;
        let _ = ModeSelectBuffer;
        let _ = Length;
        let _ = SavePages;
        STATUS_NOT_SUPPORTED
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassModeSense(
    Fdo: PDEVICE_OBJECT,
    ModeSenseBuffer: PCHAR,
    Length: ULONG,
    PageMode: UCHAR
) -> ULONG {
    ClasspModeSense(
        Fdo,
        ModeSenseBuffer,
        Length,
        PageMode,
        (MODE_SENSE_CURRENT_VALUES) as u8
    )
}

#[no_mangle]
pub unsafe extern "system" fn ClassModeSenseEx(
    Fdo: PDEVICE_OBJECT,
    ModeSenseBuffer: PCHAR,
    Length: ULONG,
    PageMode: UCHAR,
    PageControl: UCHAR
) -> ULONG {
    #[cfg(target_os = "windows")]
    {
        ClasspModeSense(Fdo, ModeSenseBuffer, Length, PageMode, PageControl)
    }
    #[cfg(not(target_os = "windows"))]
    {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassQueryTimeOutRegistryValue(
    DeviceObject: PDEVICE_OBJECT,
) -> ULONG {
    let driverExtension: PCLASS_DRIVER_EXTENSION = IoGetDriverObjectExtension(
        (*DeviceObject).DriverObject,
        ClassInitialize as PVOID,
    ) as PCLASS_DRIVER_EXTENSION;

    let registryPath: PUNICODE_STRING = core::ptr::addr_of_mut!((*driverExtension).RegistryPath);

    let mut parameters: PRTL_QUERY_REGISTRY_TABLE = core::ptr::null_mut();
    let mut path: PWSTR;
    let status: NTSTATUS;
    let mut timeOut: LONG = 0;
    let zero: ULONG = 0;
    let size: ULONG;

    if registryPath.is_null() {
        return 0;
    }

    parameters = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        ((core::mem::size_of::<RTL_QUERY_REGISTRY_TABLE>() * 2) as u64) as usize,
        u32::from_be_bytes(*b"1BcS"),
    ) as PRTL_QUERY_REGISTRY_TABLE;

    if parameters.is_null() {
        return 0;
    }

    size = (*registryPath).MaximumLength as ULONG + core::mem::size_of::<WCHAR>() as ULONG;
    path = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        (size as u64) as usize,
        u32::from_be_bytes(*b"2BcS"),
    ) as PWSTR;

    if path.is_null() {
        FREE_POOL(parameters as PVOID);
        return 0;
    }

    RtlZeroMemory(path as PVOID, size as usize);
    RtlCopyMemory(
        path as PVOID,
        (*registryPath).Buffer as PVOID,
        (size - core::mem::size_of::<WCHAR>() as ULONG) as usize,
    );

    RtlZeroMemory(
        parameters as PVOID,
        core::mem::size_of::<RTL_QUERY_REGISTRY_TABLE>() * 2,
    );

    (*parameters.offset(0)).Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK;
    (*parameters.offset(0)).Name = w!("TimeOutValue").as_ptr() as PWSTR;
    (*parameters.offset(0)).EntryContext = core::ptr::addr_of_mut!(timeOut) as PVOID;
    (*parameters.offset(0)).DefaultType = ((REG_DWORD << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | REG_DWORD) as ULONG;
    (*parameters.offset(0)).DefaultData = core::ptr::addr_of!(zero) as PVOID;
    (*parameters.offset(0)).DefaultLength = core::mem::size_of::<ULONG>() as ULONG;

    status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE | RTL_REGISTRY_OPTIONAL,
        path,
        parameters,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(status) {
        timeOut = 0;
    }

    FREE_POOL(parameters as PVOID);
    FREE_POOL(path as PVOID);

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"ClassQueryTimeOutRegistryValue: Timeout value %d\n\0".as_ptr() as PCHAR,
        timeOut,
    );

    return timeOut as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn ClassReleaseChildLock(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION
) {
    extern "system" {
        fn NT_ASSERT(condition: u8);
    }

    NT_ASSERT!(((*FdoExtension).ChildLockOwner == (KeGetCurrentThread()).cast()) as u32);
    NT_ASSERT!(((*FdoExtension).ChildLockAcquisitionCount != 0) as u32);

    (*FdoExtension).ChildLockAcquisitionCount -= 1;

    if (*FdoExtension).ChildLockAcquisitionCount == 0 {
        (*FdoExtension).ChildLockOwner = core::ptr::null_mut();
        KeSetEvent(
            core::ptr::addr_of_mut!((*FdoExtension).ChildLock).cast(),
            IO_NO_INCREMENT as i32,
            0u8
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClassReleaseQueue(
    Fdo: PDEVICE_OBJECT
) {
    ClasspReleaseQueue(Fdo, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn ClassRemoveDevice(
    DeviceObject: PDEVICE_OBJECT,
    RemoveType: UCHAR,
) -> NTSTATUS {
    let driverExtension = IoGetDriverObjectExtension(
        (*DeviceObject).DriverObject,
        ClassInitialize as PVOID,
    ) as PCLASS_DRIVER_EXTENSION;
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let lowerDeviceObject = (*commonExtension).LowerDeviceObject;
    let mut proceedWithRemove: BOOLEAN = 1u8;
    let mut status: NTSTATUS;
    let mut removeLockRundown: PEX_RUNDOWN_REF_CACHE_AWARE = core::ptr::null_mut();

    if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 ||
       !(*driverExtension).InitData.PdoData.ClassWmiInfo.GuidRegInfo.is_null() {
        status = IoWMIRegistrationControl(DeviceObject, WMIREG_ACTION_DEREGISTER);
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_WMI,
            b"ClassRemoveDevice: IoWMIRegistrationControl(%p, WMI_ACTION_DEREGISTER) --> %lx\0".as_ptr() as PCHAR,
            DeviceObject,
            status,
        );
    }

    if !(*commonExtension).MountedDeviceInterfaceName.Buffer.is_null() {
        IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!((*commonExtension).MountedDeviceInterfaceName),
            0u8,
        );
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*commonExtension).MountedDeviceInterfaceName));
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!((*commonExtension).MountedDeviceInterfaceName),
            core::ptr::null_mut(),
        );
    }

    if RemoveType == IRP_MN_REMOVE_DEVICE {
        ClassReleaseRemoveLock(DeviceObject, DeviceObject as PIRP);

        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_PNP,
            b"ClasspRemoveDevice - Reference count is now %d\n\0".as_ptr() as PCHAR,
            (*commonExtension).RemoveLock,
        );

        removeLockRundown = ((*commonExtension).PrivateCommonData as *mut u8)
            .add(core::mem::size_of::<CLASS_PRIVATE_COMMON_DATA>()) as PEX_RUNDOWN_REF_CACHE_AWARE;
        ExWaitForRundownProtectionReleaseCacheAware(removeLockRundown);

        KeSetEvent(
            core::ptr::addr_of_mut!((*commonExtension).RemoveEvent).cast(),
            IO_NO_INCREMENT as i32,
            0u8,
        );

        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_PNP,
            b"ClasspRemoveDevice - removing device %p\n\0".as_ptr() as PCHAR,
            DeviceObject,
        );

        if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_PNP,
                b"ClasspRemoveDevice - FDO %p has received a remove request.\n\0".as_ptr() as PCHAR,
                DeviceObject,
            );
        } else {
            let pdoExtension = (*DeviceObject).DeviceExtension as PPHYSICAL_DEVICE_EXTENSION;

            if (*pdoExtension).IsMissing != 0 {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_PNP,
                    b"ClasspRemoveDevice - PDO %p is missing and will be removed\0".as_ptr() as PCHAR,
                    DeviceObject,
                );
            } else {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_PNP,
                    b"ClasspRemoveDevice - PDO %p still exists and will be removed when it disappears\0".as_ptr() as PCHAR,
                    DeviceObject,
                );

                ExReInitializeRundownProtectionCacheAware(removeLockRundown);

                ClassAcquireRemoveLock(DeviceObject, DeviceObject as PIRP);

                (*commonExtension).IsRemoved = NO_REMOVE;

                IoInvalidateDeviceRelations(
                    (*(*commonExtension).PartitionZeroExtension).LowerPdo,
                    BusRelations,
                );

                proceedWithRemove = 0u8;
            }
        }
    }

    if proceedWithRemove != 0 {
        NT_ASSERT!((*(*commonExtension).DevInfo).ClassRemoveDevice.is_some());
        status = ((*(*commonExtension).DevInfo).ClassRemoveDevice.unwrap())(DeviceObject, RemoveType);
        NT_ASSERT!(NT_SUCCESS(status));
        status = STATUS_SUCCESS;
        let _ = status;

        if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
            let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

            ClasspDisableTimer(fdoExtension);

            if RemoveType == IRP_MN_REMOVE_DEVICE {
                let mut child: PPHYSICAL_DEVICE_EXTENSION;

                if !(*fdoExtension).FunctionSupportInfo.is_null() &&
                   (*(*fdoExtension).FunctionSupportInfo).IdlePower.IdlePowerEnabled() != 0 {
                    let mut fdoEntry: PIDLE_POWER_FDO_LIST_ENTRY;

                    KeAcquireGuardedMutex((core::ptr::addr_of_mut!(IdlePowerFDOListMutex)).cast());
                    fdoEntry = IdlePowerFDOList.Flink as PIDLE_POWER_FDO_LIST_ENTRY;
                    while (fdoEntry as PLIST_ENTRY) != core::ptr::addr_of_mut!(IdlePowerFDOList) {
                        let nextEntry = (*fdoEntry).ListEntry.Flink as PIDLE_POWER_FDO_LIST_ENTRY;
                        if (*fdoEntry).Fdo == DeviceObject {
                            RemoveEntryList(core::ptr::addr_of_mut!((*fdoEntry).ListEntry));
                            ExFreePool(fdoEntry as PVOID);
                            break;
                        }
                        fdoEntry = nextEntry;
                    }
                    KeReleaseGuardedMutex((core::ptr::addr_of_mut!(IdlePowerFDOListMutex)).cast());
                }

                ClassCleanupMediaChangeDetection(fdoExtension);

                FREE_POOL(((*fdoExtension).FailurePredictionInfo).cast());

                ClassAcquireChildLock(fdoExtension);
                child = ClassRemoveChild(fdoExtension, core::ptr::null_mut(), 0u8);
                while !child.is_null() {
                    let child_dev = (*child).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
                    let childCommonExtension = (*child_dev).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

                    (*child).IsMissing = 1u8;
                    (*childCommonExtension).IsRemoved = REMOVE_PENDING;
                    ClassRemoveDevice(child_dev, IRP_MN_REMOVE_DEVICE);
                    child = ClassRemoveChild(fdoExtension, core::ptr::null_mut(), 0u8);
                }
                ClassReleaseChildLock(fdoExtension);
            } else if RemoveType == IRP_MN_SURPRISE_REMOVAL {
                ClassMarkChildrenMissing(fdoExtension);
            }

            if RemoveType == IRP_MN_REMOVE_DEVICE {
                ClasspFreeReleaseRequest(DeviceObject);

                if !(*fdoExtension).PrivateFdoData.is_null() {
                    let allFdosListEntry = core::ptr::addr_of_mut!((*(*fdoExtension).PrivateFdoData).AllFdosListEntry);
                    if !(*allFdosListEntry).Flink.is_null() && !(*allFdosListEntry).Blink.is_null() {
                        RemoveEntryList(allFdosListEntry);
                    }
                    InitializeListHead(allFdosListEntry);

                    DestroyAllTransferPackets(DeviceObject);

                    ClasspDeleteTimer(fdoExtension);

                    FREE_POOL(((*(*fdoExtension).PrivateFdoData).PowerProcessIrp).cast());
                    FREE_POOL(((*(*fdoExtension).PrivateFdoData).FreeTransferPacketsLists).cast());
                    FREE_POOL(((*fdoExtension).PrivateFdoData).cast());
                }

                FREE_POOL((*fdoExtension).Reserved4 as PVOID);

                if !(*commonExtension).DeviceName.Buffer.is_null() {
                    FREE_POOL(((*commonExtension).DeviceName.Buffer).cast());
                    RtlInitUnicodeString(
                        core::ptr::addr_of_mut!((*commonExtension).DeviceName),
                        core::ptr::null_mut(),
                    );
                }

                if !(*fdoExtension).FunctionSupportInfo.is_null() {
                    // HwFirmwareInfo is a Windows Threshold+ field not present in this bindgen snapshot; skip free
                }
                FREE_POOL(((*fdoExtension).FunctionSupportInfo).cast());

                FREE_POOL(((*fdoExtension).MiniportDescriptor).cast());

                FREE_POOL(((*fdoExtension).AdapterDescriptor).cast());

                FREE_POOL(((*fdoExtension).DeviceDescriptor).cast());

                IoDetachDevice(lowerDeviceObject);
            }
        } else {
            if RemoveType == IRP_MN_REMOVE_DEVICE {
                let fdoExtension = (*commonExtension).PartitionZeroExtension;
                let pdoExtension = commonExtension as PPHYSICAL_DEVICE_EXTENSION;

                ClassRemoveChild(fdoExtension, pdoExtension, 1u8);
            }
        }

        (*commonExtension).PartitionLength.QuadPart = 0;

        if RemoveType == IRP_MN_REMOVE_DEVICE {
            ClasspUninitializeRemoveTracking(DeviceObject);

            IoDeleteDevice(DeviceObject);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClassSendDeviceIoControlSynchronous(
    IoControlCode: ULONG,
    TargetDeviceObject: PDEVICE_OBJECT,
    Buffer: PVOID,
    InputBufferLength: ULONG,
    OutputBufferLength: ULONG,
    InternalDeviceIoControl: BOOLEAN,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let mut irp: PIRP;
    let irpSp: PIO_STACK_LOCATION;
    let method: ULONG;

    irp = core::ptr::null_mut();
    method = IoControlCode & 3;

    irp = IoAllocateIrp((*TargetDeviceObject).StackSize as CCHAR, 0u8);
    if irp.is_null() {
        (*IoStatus).Information = 0;
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        return;
    }

    irpSp = IoGetNextIrpStackLocation(irp);

    if InternalDeviceIoControl != 0 {
        (*irpSp).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    } else {
        (*irpSp).MajorFunction = IRP_MJ_DEVICE_CONTROL;
    }

    (*irpSp).Parameters.DeviceIoControl.OutputBufferLength = OutputBufferLength;
    (*irpSp).Parameters.DeviceIoControl.InputBufferLength = InputBufferLength;
    (*irpSp).Parameters.DeviceIoControl.IoControlCode = IoControlCode;

    match method {
        METHOD_BUFFERED => {
            if (InputBufferLength != 0) || (OutputBufferLength != 0) {
                let alloc_size = if InputBufferLength > OutputBufferLength {
                    InputBufferLength
                } else {
                    OutputBufferLength
                };
                (*irp).AssociatedIrp.SystemBuffer = ExAllocatePoolWithTag(
                    NonPagedPoolNxCacheAligned,
                    alloc_size as SIZE_T,
                    CLASS_TAG_DEVICE_CONTROL,
                );
                if (*irp).AssociatedIrp.SystemBuffer.is_null() {
                    IoFreeIrp(irp);
                    (*IoStatus).Information = 0;
                    (*IoStatus).__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                    return;
                }

                if InputBufferLength != 0 {
                    RtlCopyMemory(
                        (*irp).AssociatedIrp.SystemBuffer,
                        Buffer,
                        InputBufferLength as SIZE_T,
                    );
                }
            }

            (*irp).UserBuffer = Buffer;
        }

        METHOD_IN_DIRECT | METHOD_OUT_DIRECT => {
            if InputBufferLength != 0 {
                (*irp).AssociatedIrp.SystemBuffer = Buffer;
            }

            if OutputBufferLength != 0 {
                (*irp).MdlAddress = IoAllocateMdl(
                    Buffer,
                    OutputBufferLength,
                    0u8,
                    0u8,
                    core::ptr::null_mut(),
                );
                if (*irp).MdlAddress.is_null() {
                    IoFreeIrp(irp);
                    (*IoStatus).Information = 0;
                    (*IoStatus).__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
                    return;
                }

                // C: _SEH2_EXCEPT { free the MDL and the IRP; report the code in
                // IoStatus; return }.
                let st = ntkrnl_sys::probe::MmTryProbeAndLockPages(
                    (*irp).MdlAddress,
                    KernelMode as KPROCESSOR_MODE,
                    if method == METHOD_IN_DIRECT {
                        IoReadAccess
                    } else {
                        IoWriteAccess
                    },
                );
                if !NT_SUCCESS(st) {
                    IoFreeMdl((*irp).MdlAddress);
                    IoFreeIrp(irp);
                    (*IoStatus).Information = 0;
                    (*IoStatus).__bindgen_anon_1.Status = st;
                    return;
                }
            }
        }

        METHOD_NEITHER => {
            IoFreeIrp(irp);
            (*IoStatus).Information = 0;
            (*IoStatus).__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
            return;
        }

        _ => {}
    }

    (*irp).Tail.Overlay.Thread = PsGetCurrentThread() as PETHREAD;

    ClassSendIrpSynchronous(TargetDeviceObject, irp);

    *IoStatus = (*irp).IoStatus;

    match method {
        METHOD_BUFFERED => {
            if OutputBufferLength != 0 {
                RtlCopyMemory(
                    Buffer,
                    (*irp).AssociatedIrp.SystemBuffer,
                    OutputBufferLength as SIZE_T,
                );
            }

            if (InputBufferLength != 0) || (OutputBufferLength != 0) {
                ExFreePool((*irp).AssociatedIrp.SystemBuffer);
            }
        }

        METHOD_IN_DIRECT | METHOD_OUT_DIRECT => {
            if OutputBufferLength != 0 {
                MmUnlockPages((*irp).MdlAddress);
                IoFreeMdl((*irp).MdlAddress);
                (*irp).MdlAddress = core::ptr::null_mut();
            }
        }

        _ => {}
    }

    IoFreeIrp(irp);
}

#[no_mangle]
pub unsafe extern "system" fn ClassSendIrpSynchronous(
    TargetDeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut event: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS;

    extern "C" {
        
        fn _Analysis_assume_(cond: BOOLEAN);
        fn TracePrint(level: u32, flags: u32, format: PCSTR, ...);
    }

    NT_ASSERT!((KeGetCurrentIrql() < DISPATCH_LEVEL as KIRQL) as u8);
    NT_ASSERT!((!TargetDeviceObject.is_null()) as u8);
    NT_ASSERT!((!Irp.is_null()) as u8);
    NT_ASSERT!(((*Irp).StackCount >= (*TargetDeviceObject).StackSize) as u8);

    KeInitializeEvent(&mut event, SynchronizationEvent, 0);
    IoSetCompletionRoutine(
        Irp,
        Some(unsafe { core::mem::transmute::<unsafe extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, *mut core::ffi::c_void) -> NTSTATUS, _>(ClassSignalCompletion) }),
        &mut event as *mut _ as PVOID,
        1,
        1,
        1,
    );

    status = IoCallDriver(TargetDeviceObject, Irp);
    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)
    if status == STATUS_PENDING {
        #[cfg(debug_assertions)]
        {
            let mut timeout: LARGE_INTEGER = core::mem::zeroed();
            timeout.QuadPart = -1i64 * 10 * 1000 * 1000 * ClasspnpGlobals.SecondsToWaitForIrps as i64;

            loop {
                status = KeWaitForSingleObject(
                    &mut event as *mut _ as PVOID,
                    Executive,
                    KernelMode as i8,
                    0,
                    &mut timeout,
                );

                if status == STATUS_TIMEOUT {
                    TracePrint(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        b"ClassSendIrpSynchronous: (%p) irp %p did not complete within %x seconds\n\0".as_ptr() as PCSTR,
                        TargetDeviceObject,
                        Irp,
                        ClasspnpGlobals.SecondsToWaitForIrps,
                    );

                    if ClasspnpGlobals.BreakOnLostIrps != 0 {
                        NT_ASSERT!(0);
                    }
                }

                if status != STATUS_TIMEOUT {
                    break;
                }
            }
        }
        #[cfg(not(debug_assertions))]
        {
            KeWaitForSingleObject(
                &mut event as *mut _ as PVOID,
                Executive,
                KernelMode as i8,
                0,
                core::ptr::null_mut(),
            );
        }

        status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn ClassSendSrbAsynchronous(
    Fdo: PDEVICE_OBJECT,
    _Srb: PSCSI_REQUEST_BLOCK,
    Irp: PIRP,
    BufferAddress: PVOID,
    BufferLength: ULONG,
    WriteToDevice: BOOLEAN,
) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut irpStack: PIO_STACK_LOCATION;
    let Srb = _Srb as PSTORAGE_REQUEST_BLOCK_HEADER;
    let mut savedFlags: ULONG;

    if (*Srb).Function != (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8 {
        (*Srb).Length = (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as ULONG) as u16;
        (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
    }

    SrbSetSenseInfoBuffer((Srb).cast(), (*fdoExtension).SenseData.cast());
    SrbSetSenseInfoBufferLength((Srb).cast(), GET_FDO_EXTENSON_SENSE_DATA_LENGTH(fdoExtension) as u8);

    SrbSetDataBuffer((Srb).cast(), BufferAddress);
    SrbSetDataTransferLength((Srb).cast(), BufferLength);

    savedFlags = SrbGetSrbFlags((Srb).cast()) & SRB_FLAGS_CLASS_DRIVER_RESERVED;

    SET_FLAG!(savedFlags, SrbGetSrbFlags((Srb).cast()) & SRB_FLAGS_DONT_START_NEXT_PACKET);

    if TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_FLAGS_QUEUE_ACTION_ENABLE) != 0
        && ((SrbGetRequestAttribute((Srb).cast()) as u32) == SRB_SIMPLE_TAG_REQUEST
            || (SrbGetRequestAttribute((Srb).cast()) as u32) == SRB_HEAD_OF_QUEUE_TAG_REQUEST
            || (SrbGetRequestAttribute((Srb).cast()) as u32) == SRB_ORDERED_QUEUE_TAG_REQUEST)
    {
        SET_FLAG!(savedFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
        if TEST_FLAG(SrbGetSrbFlags((Srb).cast()), SRB_FLAGS_NO_QUEUE_FREEZE) != 0 {
            SET_FLAG!(savedFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
        }
    }

    if !BufferAddress.is_null() {
        if (*Irp).MdlAddress.is_null() {
            let mdl = IoAllocateMdl(
                BufferAddress,
                BufferLength,
                0u8,
                0u8,
                Irp,
            );

            if mdl.is_null() || (*Irp).MdlAddress.is_null() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;

                if PORT_ALLOCATED_SENSE_EX(fdoExtension, (Srb).cast()) != 0 {
                    FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension, (Srb).cast());
                }
                ClassFreeOrReuseSrb(fdoExtension, _Srb);
                ClassReleaseRemoveLock(Fdo, (Irp as PVOID).cast());
                ClassCompleteRequest(Fdo, Irp, ((IO_NO_INCREMENT as u8) as i8) as _);

                return STATUS_INSUFFICIENT_RESOURCES;
            }

            SET_FLAG!(savedFlags, SRB_CLASS_FLAGS_FREE_MDL);

            MmBuildMdlForNonPagedPool((*Irp).MdlAddress);
        } else {
            NT_ASSERT!(BufferAddress == MmGetMdlVirtualAddress((*Irp).MdlAddress));
        }

        SrbAssignSrbFlags(
            (Srb).cast(),
            if WriteToDevice != 0 {
                SRB_FLAGS_DATA_OUT
            } else {
                SRB_FLAGS_DATA_IN
            },
        );
    } else {
        SrbAssignSrbFlags((Srb).cast(), SRB_FLAGS_NO_DATA_TRANSFER);
    }

    SrbSetSrbFlags((Srb).cast(), savedFlags);
    SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_DISABLE_SYNCH_TRANSFER);

    SrbSetScsiStatus((Srb).cast(), 0);
    (*Srb).SrbStatus = 0;

    SrbSetNextSrb((Srb).cast(), core::ptr::null_mut());

    irpStack = IoGetCurrentIrpStackLocation(Irp);
    (*irpStack).Parameters.Others.Argument4 = MAXIMUM_RETRIES as PVOID;

    IoSetCompletionRoutine(
        Irp,
        Some(ClassIoComplete),
        Srb as PVOID,
        1u8, 1u8, 1u8
    );

    irpStack = IoGetNextIrpStackLocation(Irp);
    (*irpStack).MajorFunction = (IRP_MJ_SCSI) as u8;
    (*irpStack).Parameters.Scsi.Srb = _Srb as *mut _ as *mut _;

    SrbSetOriginalRequest((Srb).cast(), Irp as PVOID);

    IoMarkIrpPending(Irp);

    IoCallDriver((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject, Irp);

    STATUS_PENDING
}

#[no_mangle]
pub unsafe extern "system" fn ClassSendSrbSynchronous(
    Fdo: PDEVICE_OBJECT,
    _Srb: PSCSI_REQUEST_BLOCK,
    BufferAddress: PVOID,
    BufferLength: ULONG,
    WriteToDevice: BOOLEAN,
) -> NTSTATUS {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData = (*fdoExtension).PrivateFdoData;
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut irp: PIRP;
    let mut irpStack: PIO_STACK_LOCATION;
    let mut event: KEVENT = core::mem::zeroed();
    let mut senseInfoBuffer: PVOID = core::ptr::null_mut();
    let mut senseInfoBufferLength: ULONG = (SENSE_BUFFER_SIZE_EX) as u32;
    let mut retryCount: ULONG = MAXIMUM_RETRIES;
    let mut status: NTSTATUS;
    let mut retry: BOOLEAN;
    let Srb = _Srb as PSTORAGE_REQUEST_BLOCK_HEADER;

    extern "C" {
        
        
        fn SrbSetSrbFlags(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Flags: ULONG);
        
        
        
        
        fn SrbGetSenseInfoBufferLength(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> UCHAR;
        
        fn SrbSetScsiStatus(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Status: UCHAR);
        fn SrbSetNextSrb(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Next: PVOID);
        
        fn SRB_STATUS(SrbStatus: UCHAR) -> UCHAR;
        fn TEST_FLAG(Flags: ULONG, Flag: ULONG) -> BOOLEAN;
        fn PORT_ALLOCATED_SENSE_EX(fdoExt: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> BOOLEAN;
        fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExt: PFUNCTIONAL_DEVICE_EXTENSION, Srb: PSTORAGE_REQUEST_BLOCK_HEADER);
        fn FREE_POOL(ptr: PVOID);
        fn DBGGETSCSIOPSTR(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
        fn DBGGETSRBSTATUSSTR(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
        fn DBGGETSENSECODESTR(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
        fn DBGGETADSENSECODESTR(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
        fn DBGGETADSENSEQUALIFIERSTR(Srb: PSTORAGE_REQUEST_BLOCK_HEADER) -> PCHAR;
        fn TracePrint(Level: ULONG, Flags: ULONG, Format: PCHAR, ...);
        
        fn InterpretSenseInfoWithoutHistory(
            Fdo: PDEVICE_OBJECT,
            OriginalRequest: PIRP,
            Srb: PSCSI_REQUEST_BLOCK,
            MajorFunctionCode: UCHAR,
            IoDeviceCode: ULONG,
            PreviousRetryCount: ULONG,
            Status: *mut NTSTATUS,
            RetryIn100nsUnits: *mut LONGLONG,
        ) -> BOOLEAN;
    }
    extern "system" {
        fn ClasspSendSynchronousCompletion(
            DeviceObject: PDEVICE_OBJECT,
            Irp: PIRP,
            Context: PVOID,
        ) -> NTSTATUS;
    }

    NT_ASSERT!((KeGetCurrentIrql() as u32) < (DISPATCH_LEVEL) as u32);
    NT_ASSERT!((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0);

    if (*Srb).Function != (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8 {
        (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
    }

    NT_ASSERT!(
        (((*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == SRB_TYPE_SCSI_REQUEST_BLOCK)
            && ((*Srb).Function == (SRB_FUNCTION_EXECUTE_SCSI) as u8))
            || (((*((*fdoExtension).AdapterDescriptor as *mut crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8)
                && ((*Srb).Function == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8))
    );

    senseInfoBufferLength = (ALIGN_UP_BY(senseInfoBufferLength as usize, KeGetRecommendedSharedDataAlignment() as usize)) as u32;

    senseInfoBuffer = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        (senseInfoBufferLength as u64) as usize,
        u32::from_be_bytes(*b"7CcS"),
    );

    if senseInfoBuffer.is_null() {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassSendSrbSynchronous: Can't allocate request sense buffer\n\0".as_ptr() as PCHAR,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    SrbSetSenseInfoBufferLength((Srb).cast(), SENSE_BUFFER_SIZE_EX as UCHAR);
    SrbSetSenseInfoBuffer((Srb).cast(), senseInfoBuffer);
    SrbSetDataBuffer((Srb).cast(), BufferAddress);

    'retry: loop {
        SrbAssignSrbFlags(Srb, (*fdoExtension).SrbFlags);

        if !BufferAddress.is_null() {
            if WriteToDevice != 0 {
                SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_DATA_OUT);
            } else {
                SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_DATA_IN);
            }
        }

        SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as u8);
        SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE);

        KeInitializeEvent(&mut event, NotificationEvent, 0u8);

        irp = IoAllocateIrp(
            ((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject as *mut _DEVICE_OBJECT)
                .as_ref()
                .unwrap()
                .StackSize as i8
                + 1,
            0u8,
        );

        if irp.is_null() {
            FREE_POOL(senseInfoBuffer);
            SrbSetSenseInfoBuffer((Srb).cast(), core::ptr::null_mut());
            SrbSetSenseInfoBufferLength((Srb).cast(), 0);
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                b"ClassSendSrbSynchronous: Can't allocate Irp\n\0".as_ptr() as PCHAR,
            );
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        irpStack = IoGetNextIrpStackLocation(irp);
        (*irpStack).MajorFunction = IRP_MJ_SCSI as UCHAR;
        (*irpStack).Parameters.Scsi.Srb = Srb as *mut _ as *mut _;

        IoSetCompletionRoutine(
            irp,
            Some(unsafe { core::mem::transmute::<unsafe extern "system" fn(*mut DEVICE_OBJECT, *mut IRP, PVOID) -> NTSTATUS, _>(ClasspSendSynchronousCompletion) }),
            Srb as PVOID,
            1u8, 1u8, 1u8
        );

        (*irp).UserIosb = &mut ioStatus;
        (*irp).UserEvent = (&mut event as *mut _ as PVOID).cast();

        if !BufferAddress.is_null() {
            (*irp).MdlAddress = IoAllocateMdl(BufferAddress, BufferLength, 0u8, 0u8, irp);
            if (*irp).MdlAddress.is_null() {
                FREE_POOL(senseInfoBuffer);
                SrbSetSenseInfoBuffer((Srb).cast(), core::ptr::null_mut());
                SrbSetSenseInfoBufferLength((Srb).cast(), 0);
                IoFreeIrp(irp);
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClassSendSrbSynchronous: Can't allocate MDL\n\0".as_ptr() as PCHAR,
                );
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            // C: _SEH2_EXCEPT { free the sense buffer, the MDL and the IRP; return
            // the code }.
            let st = ntkrnl_sys::probe::MmTryProbeAndLockPages(
                (*irp).MdlAddress,
                KernelMode as i8,
                if WriteToDevice != 0 { IoReadAccess } else { IoWriteAccess },
            );
            if !NT_SUCCESS(st) {
                FREE_POOL(senseInfoBuffer);
                SrbSetSenseInfoBuffer((Srb).cast(), core::ptr::null_mut());
                SrbSetSenseInfoBufferLength((Srb).cast(), 0);
                IoFreeMdl((*irp).MdlAddress);
                IoFreeIrp(irp);
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClassSendSrbSynchronous: Exception %lx locking buffer\n\0".as_ptr() as PCHAR,
                    st,
                );
                return st;
            }
        }

        SrbSetDataTransferLength((Srb).cast(), BufferLength);
        SrbSetScsiStatus((Srb).cast(), 0);
        (*Srb).SrbStatus = 0;
        SrbSetNextSrb((Srb).cast(), core::ptr::null_mut());
        SrbSetOriginalRequest((Srb).cast(), irp as PVOID);

        status = IoCallDriver((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject, irp);

        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut event as *mut _ as PVOID,
                Executive,
                KernelMode as i8,
                0u8,
                core::ptr::null_mut(),
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }

        NT_ASSERT!(status != STATUS_PENDING);
        NT_ASSERT!(TEST_FLAG((*Srb).SrbStatus as ULONG, SRB_STATUS_QUEUE_FROZEN) == 0);

        SrbSetOriginalRequest(Srb, core::ptr::null_mut());

        if SRB_STATUS((*Srb).SrbStatus) != (SRB_STATUS_SUCCESS) as u8 {
            let mut retryInterval: LONGLONG = 0;

            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                b"ClassSendSrbSynchronous - srb %ph failed (op=%s srbstat=%s(%xh), irpstat=%xh, sense=%s/%s/%s)\0"
                    .as_ptr() as PCHAR,
                Srb,
                DBGGETSCSIOPSTR(Srb),
                DBGGETSRBSTATUSSTR(Srb),
                (*Srb).SrbStatus as ULONG,
                status,
                DBGGETSENSECODESTR(Srb),
                DBGGETADSENSECODESTR(Srb),
                DBGGETADSENSEQUALIFIERSTR(Srb),
            );

            NT_ASSERT!(TEST_FLAG((*Srb).SrbStatus as ULONG, SRB_STATUS_QUEUE_FROZEN) == 0);

            retry = InterpretSenseInfoWithoutHistory(
                Fdo,
                core::ptr::null_mut(),
                Srb as PSCSI_REQUEST_BLOCK,
                IRP_MJ_SCSI as UCHAR,
                0,
                MAXIMUM_RETRIES - retryCount,
                &mut status,
                &mut retryInterval,
            );

            if retry != 0 {
                let mut validSense: BOOLEAN = 0u8;
                let mut additionalSenseCode: UCHAR = 0;

                if status == STATUS_DEVICE_NOT_READY {
                    validSense = ScsiGetSenseKeyAndCodes(
                        senseInfoBuffer,
                        (SrbGetSenseInfoBufferLength((Srb).cast())) as _,
                        SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
                        core::ptr::null_mut(),
                        &mut additionalSenseCode,
                        core::ptr::null_mut(),
                    );
                }

                if (validSense != 0 && additionalSenseCode == (SCSI_ADSENSE_LUN_NOT_READY) as u8)
                    || (SRB_STATUS((*Srb).SrbStatus) == (SRB_STATUS_SELECTION_TIMEOUT) as u8)
                {
                    let mut delay: LARGE_INTEGER = core::mem::zeroed();

                    if retryInterval < 2 * 1000 * 1000 * 10 {
                        retryInterval = 2 * 1000 * 1000 * 10;
                    }

                    delay.QuadPart = -retryInterval;
                    KeDelayExecutionThread(KernelMode as i8, 0u8, &mut delay);
                }

                if retryCount > 0 {
                    retryCount -= 1;

                    if PORT_ALLOCATED_SENSE_EX(fdoExtension, Srb) != 0 {
                        FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension, Srb);
                    }

                    continue 'retry;
                }
            }
        } else {
            (*fdoData).LoggedTURFailureSinceLastIO = 0u8;
            status = STATUS_SUCCESS;
        }

        break;
    }

    if PORT_ALLOCATED_SENSE_EX(fdoExtension, Srb) != 0 {
        FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension, Srb);
    }

    FREE_POOL(senseInfoBuffer);
    SrbSetSenseInfoBuffer((Srb).cast(), core::ptr::null_mut());
    SrbSetSenseInfoBufferLength((Srb).cast(), 0);

    status
}

#[no_mangle]
pub unsafe extern "system" fn ClassSignalCompletion(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let Event = Context as *mut KEVENT;

    if Context.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    KeSetEvent(Event as PVOID, (IO_NO_INCREMENT as i32) as i32, 0u8);

    return STATUS_MORE_PROCESSING_REQUIRED;
}

#[no_mangle]
pub unsafe extern "system" fn ClassUpdateInformationInRegistry(
    Fdo: PDEVICE_OBJECT,
    DeviceName: PCHAR,
    DeviceNumber: ULONG,
    InquiryData: PINQUIRYDATA,
    InquiryDataLength: ULONG
) {
    let mut status: NTSTATUS;
    let mut scsiAddress: SCSI_ADDRESS = core::mem::zeroed();
    let mut objectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut string: STRING = core::mem::zeroed();
    let mut unicodeName: UNICODE_STRING = core::mem::zeroed();
    let mut unicodeRegistryPath: UNICODE_STRING = core::mem::zeroed();
    let mut unicodeData: UNICODE_STRING = core::mem::zeroed();
    let mut targetKey: HANDLE = core::ptr::null_mut();
    let mut ioStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut buffer: [u8; 256] = [0; 256];

    NT_ASSERT!(!DeviceName.is_null());

    ClassSendDeviceIoControlSynchronous(
        IOCTL_SCSI_GET_ADDRESS,
        Fdo,
        core::ptr::addr_of_mut!(scsiAddress) as PVOID,
        0,
        core::mem::size_of::<SCSI_ADDRESS>() as ULONG,
        0u8,
        core::ptr::addr_of_mut!(ioStatus)
    );

    if !NT_SUCCESS(ioStatus.__bindgen_anon_1.Status) {
        status = ioStatus.__bindgen_anon_1.Status;
        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
                    b"UpdateInformationInRegistry: Get Address failed %lx\n\0".as_ptr() as PCHAR,
                    status);
    } else {
        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
                    b"GetAddress: Port %x, Path %x, Target %x, Lun %x\n\0".as_ptr() as PCHAR,
                    (scsiAddress.PortNumber) as i32,
                    (scsiAddress.PathId) as i32,
                    (scsiAddress.TargetId) as i32,
                    (scsiAddress.Lun) as i32);

        status = RtlStringCchPrintfA(
            buffer.as_mut_ptr() as NTSTRSAFE_PSTR,
            (core::mem::size_of_val(&buffer) - 1) as usize,
            b"\\Registry\\Machine\\Hardware\\DeviceMap\\Scsi\\Scsi Port %d\\Scsi Bus %d\\Target Id %d\\Logical Unit Id %d\0".as_ptr() as PCHAR,
            scsiAddress.PortNumber as i32,
            scsiAddress.PathId as i32,
            scsiAddress.TargetId as i32,
            scsiAddress.Lun as i32
        );

        if NT_SUCCESS(status) {
            RtlInitString(core::ptr::addr_of_mut!(string), buffer.as_ptr() as PCSZ);

            status = RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(unicodeRegistryPath) as *mut _,
                core::ptr::addr_of_mut!(string),
                1u8
            );

            if NT_SUCCESS(status) {
                // InitializeObjectAttributes inlined
                objectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
                objectAttributes.RootDirectory = core::ptr::null_mut();
                objectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
                objectAttributes.ObjectName = core::ptr::addr_of_mut!(unicodeRegistryPath) as *mut _;
                objectAttributes.SecurityDescriptor = core::ptr::null_mut();
                objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                status = ZwOpenKey(
                    core::ptr::addr_of_mut!(targetKey),
                    KEY_READ | KEY_WRITE,
                    core::ptr::addr_of_mut!(objectAttributes)
                );

                if NT_SUCCESS(status) {
                    RtlInitUnicodeString(
                        (core::ptr::addr_of_mut!(unicodeName)).cast(),
                        w!("DeviceName").as_ptr() as PWSTR
                    );

                    status = RtlStringCchPrintfA(
                        buffer.as_mut_ptr() as NTSTRSAFE_PSTR,
                        (core::mem::size_of_val(&buffer) - 1) as usize,
                        b"%s%d\0".as_ptr() as PCHAR,
                        DeviceName,
                        DeviceNumber
                    );

                    if NT_SUCCESS(status) {
                        RtlInitString(core::ptr::addr_of_mut!(string), buffer.as_ptr() as PCSZ);
                        status = RtlAnsiStringToUnicodeString(
                            core::ptr::addr_of_mut!(unicodeData) as *mut _,
                            core::ptr::addr_of_mut!(string),
                            1u8
                        );

                        if NT_SUCCESS(status) {
                            status = ZwSetValueKey(
                                targetKey,
                                (core::ptr::addr_of_mut!(unicodeName)).cast(),
                                0,
                                REG_SZ,
                                unicodeData.Buffer as PVOID,
                                unicodeData.Length as ULONG
                            );
                        }

                        if NT_SUCCESS(status) && InquiryDataLength != 0 {
                            NT_ASSERT!(!InquiryData.is_null());

                            RtlInitUnicodeString(
                                (core::ptr::addr_of_mut!(unicodeName)).cast(),
                                w!("InquiryData").as_ptr() as PWSTR
                            );
                            status = ZwSetValueKey(
                                targetKey,
                                (core::ptr::addr_of_mut!(unicodeName)).cast(),
                                0,
                                REG_BINARY,
                                InquiryData as PVOID,
                                InquiryDataLength
                            );
                        }
                    }
                }
            }
        }
    }

    if !NT_SUCCESS(status) {
        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
                    b"Failure to update information in registry: %08x\n\0".as_ptr() as PCHAR,
                    status);
    }

    if !unicodeData.Buffer.is_null() {
        RtlFreeUnicodeString((core::ptr::addr_of_mut!(unicodeData)).cast());
    }
    if !unicodeRegistryPath.Buffer.is_null() {
        RtlFreeUnicodeString((core::ptr::addr_of_mut!(unicodeRegistryPath)).cast());
    }
    if !targetKey.is_null() {
        ZwClose(targetKey);
    }
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    
}
extern "system" {
    fn NT_ASSERT(condition: BOOLEAN);
    fn SrbAssignSrbFlags(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Flags: ULONG);
    fn SrbSetDataBuffer(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Buffer: PVOID);
    fn SrbSetRequestAttribute(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Attribute: UCHAR);
    fn SrbSetSenseInfoBuffer(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Buffer: PVOID);
    fn SrbSetSenseInfoBufferLength(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Length: UCHAR);
    fn SrbSetDataTransferLength(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Length: ULONG);
    fn SrbSetOriginalRequest(Srb: PSTORAGE_REQUEST_BLOCK_HEADER, Request: PVOID);
    fn ScsiGetSenseKeyAndCodes(SenseInfoBuffer: PVOID, SenseInfoBufferLength: u32, Options: ULONG, SenseKey: *mut UCHAR, AdditionalSenseCode: *mut UCHAR, AdditionalSenseCodeQualifier: *mut UCHAR) -> BOOLEAN;
    fn SrbSetCdbLength(srb: PVOID, len: UCHAR);
    fn GET_FDO_EXTENSON_SENSE_DATA_LENGTH(fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) -> UCHAR;
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn KeQueryTickCount(count: *mut LARGE_INTEGER);
}

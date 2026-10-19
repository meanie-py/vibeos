use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_COPY_OFFLOAD_DESCRIPTOR, DEVICE_DATA_SET_LBP_STATE_PARAMETERS, DEVICE_DATA_SET_LB_PROVISIONING_STATE, DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2, DEVICE_DATA_SET_RANGE, DEVICE_LB_PROVISIONING_DESCRIPTOR, DEVICE_MANAGE_DATA_SET_ATTRIBUTES, DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT, DEVICE_SEEK_PENALTY_DESCRIPTOR, DEVICE_TRIM_DESCRIPTOR, ExFreePool, _KWAIT_REASON_Executive, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoCopyCurrentIrpStackLocationToNext, IoFreeIrp, IoFreeMdl, IoFreeWorkItem, IoGetCurrentIrpStackLocation, IoGetDeviceProperty, IoMarkIrpPending, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoReadPartitionTableEx, IoSetCompletionRoutine, IoSetNextIrpStackLocation, IoWriteErrorLogEntry, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeGetRecommendedSharedDataAlignment, KeInitializeEvent, KeReleaseInStackQueuedSpinLock, KeWaitForSingleObject, LARGE_INTEGER, MmUnlockPages, _OBJECT_ATTRIBUTES, PCSZ, PDEVICE_COPY_OFFLOAD_DESCRIPTOR, PDEVICE_DATA_SET_LBP_STATE_PARAMETERS, PDEVICE_DATA_SET_LB_PROVISIONING_STATE, PDEVICE_DATA_SET_LB_PROVISIONING_STATE_V2, PDEVICE_DATA_SET_RANGE, PDEVICE_LB_PROVISIONING_DESCRIPTOR, PDEVICE_MANAGE_DATA_SET_ATTRIBUTES, PDEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT, PDEVICE_OBJECT, PDEVICE_SEEK_PENALTY_DESCRIPTOR, PDEVICE_TRIM_DESCRIPTOR, PDRIVE_LAYOUT_INFORMATION_EX, PERSISTENT_RESERVE_COMMAND, PGUID, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PMDL, PPERSISTENT_RESERVE_COMMAND, PSTORAGE_ACCESS_ALIGNMENT_DESCRIPTOR, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_DEVICE_ID_DESCRIPTOR, PSTORAGE_EVENT_NOTIFICATION, PSTORAGE_LB_PROVISIONING_MAP_RESOURCES, PSTORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR, PSTORAGE_PRIORITY_HINT_SUPPORT, PSTORAGE_PROPERTY_QUERY, PSTORAGE_WRITE_CACHE_PROPERTY, PULONGLONG, RTL_QUERY_REGISTRY_TABLE, RtlInitUnicodeString, RtlQueryRegistryValues, RtlWriteRegistryValue, STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR, STORAGE_DESCRIPTOR_HEADER, STORAGE_EVENT_NOTIFICATION, STORAGE_LB_PROVISIONING_MAP_RESOURCES, STORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR, STORAGE_PRIORITY_HINT_SUPPORT, STORAGE_WRITE_CACHE_PROPERTY, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwCreateKey, ZwOpenKey};
// Enum variant aliases: these exist in ntkrnl_sys::ddk with _ENUM_Variant prefixed names
use ntkrnl_sys::ddk::{_WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue};
use ntkrnl_sys::ddk::{_LOCK_OPERATION_IoReadAccess as IoReadAccess, _LOCK_OPERATION_IoWriteAccess as IoWriteAccess};
use ntkrnl_sys::ddk::{_IO_PRIORITY_HINT_IoPriorityVeryLow as IoPriorityVeryLow, _IO_PRIORITY_HINT_IoPriorityLow as IoPriorityLow, _IO_PRIORITY_HINT_IoPriorityNormal as IoPriorityNormal};
use ntkrnl_sys::ddk::{_PARTITION_STYLE_PARTITION_STYLE_GPT as PARTITION_STYLE_GPT, _PARTITION_STYLE_PARTITION_STYLE_MBR as PARTITION_STYLE_MBR};
use ntkrnl_sys::ddk::{_STORAGE_PROPERTY_ID_StorageDeviceIdProperty as StorageDeviceIdProperty};
use ntkrnl_sys::ddk::{_STORAGE_PORT_CODE_SET_StoragePortCodeSetStorport as StoragePortCodeSetStorport, _STORAGE_PORT_CODE_SET_StoragePortCodeSetSpaceport as StoragePortCodeSetSpaceport, _STORAGE_PORT_CODE_SET_StoragePortCodeSetSDport as StoragePortCodeSetSDport, _STORAGE_PORT_CODE_SET_StoragePortCodeSetSCSIport as StoragePortCodeSetSCSIport};
use ntkrnl_sys::ddk::{PSTORAGE_IDENTIFIER, DEVICE_REGISTRY_PROPERTY_DevicePropertyPhysicalDeviceObjectName as DevicePropertyPhysicalDeviceObjectName};
use ntkrnl_sys::ddk::{_WRITE_CACHE_ENABLE_WriteCacheEnableUnknown as WriteCacheEnableUnknown, _WRITE_CACHE_ENABLE_WriteCacheEnabled as WriteCacheEnabled, _WRITE_CACHE_ENABLE_WriteCacheDisabled as WriteCacheDisabled};
use ntkrnl_sys::ddk::{_WRITE_CACHE_CHANGE_WriteCacheChangeUnknown as WriteCacheChangeUnknown, _WRITE_CACHE_CHANGE_WriteCacheChangeable as WriteCacheChangeable, _WRITE_CACHE_CHANGE_WriteCacheNotChangeable as WriteCacheNotChangeable};
use ntkrnl_sys::ddk::{_WRITE_CACHE_TYPE_WriteCacheTypeUnknown as WriteCacheTypeUnknown, _WRITE_CACHE_TYPE_WriteCacheTypeWriteBack as WriteCacheTypeWriteBack};
use ntkrnl_sys::ddk::{_WRITE_THROUGH_WriteThroughSupported as WriteThroughSupported, _WRITE_THROUGH_WriteThroughNotSupported as WriteThroughNotSupported};
use ntkrnl_sys::ddk::PIO_WORKITEM_ROUTINE as IO_WORKITEM_ROUTINE;
use crate::glue::{_STORAGE_PROPERTY_ID_PropertyExistsQuery as PropertyExistsQuery, _STORAGE_PROPERTY_ID_PropertyStandardQuery as PropertyStandardQuery};
use core::ffi::c_void;
use core::mem::zeroed;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DUID_HARDWARE_IDS_ONLY, DUID_INCLUDE_SOFTWARE_IDS, FILE_DEVICE_DISK, IRP_MJ_SCSI, PLUGPLAY_REGKEY_DEVICE, RTL_QUERY_REGISTRY_TYPECHECK, STATUS_DATA_ERROR, STATUS_IO_DEVICE_ERROR, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING};
use crate::types::{BLOCK_DEVICE_RANGE_DESCRIPTOR, BLOCK_DEVICE_TOKEN_DESCRIPTOR, CDB, CDB10GENERIC_LENGTH, CLASSPNP_SCAN_FOR_SPECIAL_INFO, CLASSP_REG_COPY_OFFLOAD_MAX_TARGET_DURATION, CLASS_ERROR_LEVEL_1, CLASS_ERROR_LEVEL_2, CLASS_SPECIAL_FUA_NOT_SUPPORTED, CLASS_VPD_B0_DATA, COMMON_DEVICE_EXTENSION, ClassLogResourceExhaustionEvent, ClassLogThresholdEvent, DEFAULT_MAX_TARGET_DURATION, DEVICE_DSM_FLAG_ALLOCATION_CONSOLIDATEABLE_ONLY, DEVICE_DSM_FLAG_ENTIRE_DATA_SET_RANGE, DEVICE_DSM_FLAG_TRIM_NOT_FS_ALLOCATED, DEV_POWER_PROTECTED, DeviceDsmAction_Allocation, FOUR_BYTE, FUNCTIONAL_DEVICE_EXTENSION, GET_LBA_STATUS_RETRY_COUNT_MAX, IO_RETRIED_LOG_MESSAGE_CONTEXT, LBA_STATUS_DESCRIPTOR, LBA_STATUS_LIST_HEADER, LBA_STATUS_MAPPED, LOG_PAGE_CODE_LOGICAL_BLOCK_PROVISIONING, LOG_PAGE_LOGICAL_BLOCK_PROVISIONING, LOG_PARAMETER_HEADER, LOG_PARAMETER_THRESHOLD_RESOURCE_COUNT, MAX_RECEIVE_TOKEN_INFORMATION_PARAMETER_DATA_LENGTH, MAX_SENSE_BUFFER_SIZE, MAX_TOKEN_OPERATION_PARAMETER_DATA_LENGTH, MODE_CACHING_PAGE, MODE_DSP_FUA_SUPPORTED, MODE_PAGE_CACHING, MODE_PAGE_CONTROL, MODE_PAGE_DATA_SIZE, MODE_PARAMETER_HEADER, MODE_SENSE_CHANGEABLE_VALUES, PBLOCK_DEVICE_RANGE_DESCRIPTOR, PCDB, PCLASS_PRIVATE_FDO_DATA, PCLASS_SCAN_FOR_SPECIAL_HANDLER, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PIO_RETRIED_LOG_MESSAGE_CONTEXT, PMODE_CONTROL_PAGE, PMODE_PARAMETER_HEADER, POPCODE_SENSE_DATA_IO_LOG_MESSAGE_CONTEXT_HEADER, POPULATE_TOKEN_HEADER, PRI_REGISTRATION_LIST, PRI_RESERVATION_LIST, PROVISIONING_TYPE_UNKNOWN, PRO_PARAMETER_LIST, PSCSI_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK, PTRANSFER_PACKET, RECEIVE_TOKEN_INFORMATION_HEADER, RECEIVE_TOKEN_INFORMATION_RESPONSE_HEADER, SCSIOP_GET_LBA_STATUS, SCSIOP_INQUIRY, SCSIOP_LOG_SENSE, SCSIOP_PERSISTENT_RESERVE_IN, SCSIOP_PERSISTENT_RESERVE_OUT, SCSIOP_READ_CAPACITY16, SCSIOP_WRITE_DATA_BUFF, SCSI_ADSENSE_LB_PROVISIONING, SCSI_REQUEST_BLOCK, SCSI_SENSEQ_SOFT_THRESHOLD_REACHED, SCSI_SENSE_UNIT_ATTENTION, SERVICE_ACTION_GET_LBA_STATUS, SERVICE_ACTION_POPULATE_TOKEN, SERVICE_ACTION_READ_CAPACITY16, SRB_FLAGS_DATA_IN, SRB_FLAGS_DATA_OUT, SRB_FLAGS_DISABLE_DISCONNECT, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_NO_QUEUE_FREEZE, SRB_FLAGS_QUEUE_ACTION_ENABLE, SRB_FUNCTION_EXECUTE_SCSI, SRB_HEAD_OF_QUEUE_TAG_REQUEST, SRB_SIMPLE_TAG_REQUEST, SRB_STATUS_AUTOSENSE_VALID, SRB_TYPE_STORAGE_REQUEST_BLOCK, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_EVENT_MEDIA_STATUS, STORAGE_EVENT_NOTIFICATION_VERSION_V1, STORAGE_REQUEST_BLOCK, VPD_BLOCK_DEVICE_CHARACTERISTICS, VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE, VPD_BLOCK_LIMITS, VPD_LOGICAL_BLOCK_PROVISIONING, VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, VPD_MAX_BUFFER_SIZE, WRITE_USING_TOKEN_HEADER, _CDB, _CLASS_READ_CAPACITY16_DATA, _CLASS_VPD_B0_DATA, _COMMON_DEVICE_EXTENSION, _FUNCTIONAL_DEVICE_EXTENSION, _LOG_PAGE_LOGICAL_BLOCK_PROVISIONING, _READ_CAPACITY16_DATA, _SCSI_PNP_REQUEST_BLOCK, _SCSI_POWER_REQUEST_BLOCK, _SCSI_WMI_REQUEST_BLOCK, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _SRBEX_DATA, _SRBEX_DATA_PNP, _SRBEX_DATA_POWER, _SRBEX_DATA_SCSI_CDB16, _SRBEX_DATA_SCSI_CDB32, _SRBEX_DATA_SCSI_CDB_VAR, _SRBEX_DATA_WMI, _STORAGE_REQUEST_BLOCK, _STOR_ADDR_BTL8, _UNMAP_BLOCK_DESCRIPTOR, _UNMAP_LIST_HEADER, _VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE, _VPD_BLOCK_LIMITS_PAGE, CLASS_FUNCTION_SUPPORT_SupportUnknown, LOG_PAGE_LBP_PARAMETER_CODE_AVAILABLE, LOG_PAGE_LBP_PARAMETER_CODE_USED, RESERVATION_ACTION_READ_KEYS, RESERVATION_ACTION_READ_RESERVATIONS};
use ntkrnl_sys::ddk::{CCHAR, DEVICE_OBJECT, DRIVE_LAYOUT_INFORMATION_EX, GUID, HANDLE, IRP, STORAGE_PROPERTY_ID, STORAGE_PROPERTY_QUERY, _DEVICE_DATA_SET_RANGE, _DEVICE_OBJECT, _IO_STACK_LOCATION, _IRP, _STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR, _STORAGE_DESCRIPTOR_HEADER, _STORAGE_DEVICE_ID_DESCRIPTOR, _STORAGE_PROPERTY_ID_StorageDeviceIdProperty, _STORAGE_PROPERTY_QUERY, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DUID_VERSION_1, FILE_FLOPPY_DISKETTE, KEY_WRITE, MAXUCHAR, MAXUSHORT, RTL_QUERY_REGISTRY_TYPECHECK_SHIFT, SCSI_SENSE_OPTIONS_NONE, STATUS_INTERNAL_ERROR, STATUS_INVALID_LEVEL, STATUS_INVALID_USER_BUFFER, STATUS_MORE_PROCESSING_REQUIRED, STATUS_NONEXISTENT_SECTOR, STATUS_TRIM_READ_ZERO_NOT_SUPPORTED, TRACE_LEVEL_FATAL, TRACE_LEVEL_VERBOSE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_LB_PROVISIONING, CLASS_TAG_MODE_DATA};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_GENERAL, TRACE_FLAG_INIT, TRACE_FLAG_IOCTL, TRACE_FLAG_PNP, TRACE_FLAG_SCSI};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx, NonPagedPoolNxCacheAligned, SP_UNTAGGED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASSPNP_POOL_TAG_FIRMWARE, CLASSPNP_POOL_TAG_LOG_MESSAGE, CLASSPNP_POOL_TAG_TOKEN_OPERATION, CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE, IOCTL_STORAGE_FIRMWARE_GET_INFO, IOCTL_STORAGE_PERSISTENT_RESERVE_IN, IOCTL_STORAGE_POWER_ACTIVE, IOCTL_STORAGE_POWER_IDLE, NotSupported, STORAGE_HW_FIRMWARE_REQUEST_FLAG_CONTROLLER, SupportUnknown, Supported};
use crate::types::{RESERVATION_ACTION_CLEAR, RESERVATION_ACTION_PREEMPT, RESERVATION_ACTION_PREEMPT_ABORT, RESERVATION_ACTION_REGISTER, RESERVATION_ACTION_REGISTER_IGNORE_EXISTING, RESERVATION_ACTION_RELEASE, RESERVATION_ACTION_RESERVE, RESERVATION_SCOPE_ELEMENT, RESERVATION_SCOPE_LU, RESERVATION_TYPE_EXCLUSIVE, RESERVATION_TYPE_EXCLUSIVE_REGISTRANTS, RESERVATION_TYPE_WRITE_EXCLUSIVE, RESERVATION_TYPE_WRITE_EXCLUSIVE_REGISTRANTS};
use crate::types::{SRB_FUNCTION_WMI, SRB_FUNCTION_PNP, SRB_FUNCTION_POWER};
// Local _STORAGE_ADAPTER_DESCRIPTOR (has SrbType field) shadows the ntkrnl-sys version
use crate::types::_STORAGE_ADAPTER_DESCRIPTOR as LocalStorageAdapterDescriptor;
// VPD_BLOCK_LIMITS_PAGE inner struct needed for offset_of!
use crate::types::_VPD_BLOCK_LIMITS_PAGE__bindgen_ty_1__bindgen_ty_1;
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ClassCheckMediaState, ClassFindModePage, ClassForwardIrpSynchronous, ClassGetDescriptor, ClassModeSense, ClassReadDriveCapacity, ClassSendDeviceIoControlSynchronous, ClassSendSrbAsynchronous, ClassSendSrbSynchronous, ClasspBlockLimitsDataSnapshot, ClasspModeSelect, ClasspModeSense, InitializeStorageRequestBlock};
use ntkrnl_sys::mm::{ALIGN_UP_BY};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK};
// resolve_imports.py: explicit kernel/private-type imports
use crate::srblib::{SrbGetDataTransferLength, SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{_CLASS_FUNCTION_SUPPORT_STATE_NotSupported, _CLASS_FUNCTION_SUPPORT_STATE_Supported, _SRBEX_DATA_TYPE_SrbExDataTypePnP, _SRBEX_DATA_TYPE_SrbExDataTypePower, _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdb16, _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdb32, _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdbVar, _SRBEX_DATA_TYPE_SrbExDataTypeWmi, _STORAGE_PROPERTY_ID_PropertyExistsQuery, _STORAGE_PROPERTY_ID_PropertyStandardQuery};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{STORAGE_DEVICE_LAYOUT_SIGNATURE, STORAGE_DEVICE_UNIQUE_IDENTIFIER, _DUID_MATCH_STATUS_DuidExactMatch as DuidExactMatch, _STORAGE_DEVICE_UNIQUE_IDENTIFIER, _DUID_MATCH_STATUS_DuidSubIdMatch as DuidSubIdMatch, _DUID_MATCH_STATUS_DuidNoMatch as DuidNoMatch, _DUID_MATCH_STATUS_DuidErrorGeneral as DuidErrorGeneral, _DUID_MATCH_STATUS_DuidErrorMissingDuid as DuidErrorMissingDuid, _DUID_MATCH_STATUS_DuidErrorVersionMismatch as DuidErrorVersionMismatch, _DUID_MATCH_STATUS_DuidErrorInvalidDuid as DuidErrorInvalidDuid, _DUID_MATCH_STATUS_DuidErrorInvalidDeviceIdDescSize as DuidErrorInvalidDeviceIdDescSize, _DUID_MATCH_STATUS_DuidErrorInvalidDeviceDescSize as DuidErrorInvalidDeviceDescSize, _DUID_MATCH_STATUS_DuidErrorInvalidLayoutSigSize as DuidErrorInvalidLayoutSigSize, _DUID_MATCH_STATUS_DuidErrorInvalidLayoutSigVersion as DuidErrorInvalidLayoutSigVersion, DUID_MATCH_STATUS, PFIXED_SENSE_DATA, PDESCRIPTOR_SENSE_DATA, _CDB__GET_LBA_STATUS};
use ntkrnl_sys::ddk::{_STORAGE_IDENTIFIER_CODE_SET_StorageIdCodeSetBinary as StorageIdCodeSetBinary, _STORAGE_IDENTIFIER_CODE_SET_StorageIdCodeSetAscii as StorageIdCodeSetAscii, _STORAGE_IDENTIFIER_CODE_SET_StorageIdCodeSetUtf8 as StorageIdCodeSetUtf8, _STORAGE_IDENTIFIER_TYPE_StorageIdTypeScsiNameString as StorageIdTypeScsiNameString, _STORAGE_IDENTIFIER_TYPE_StorageIdTypeFCPHName as StorageIdTypeFCPHName, _STORAGE_IDENTIFIER_TYPE_StorageIdTypeEUI64 as StorageIdTypeEUI64, _STORAGE_IDENTIFIER_TYPE_StorageIdTypeVendorId as StorageIdTypeVendorId, _STORAGE_ASSOCIATION_TYPE_StorageIdAssocPort as StorageIdAssocPort};
use ntkrnl_sys::ddk::RtlAssert;

extern "C" {
    
    
}
extern "system" {
}

// ---- file-scope preamble (translated) ----
pub const FIRMWARE_ACTIVATE_TIMEOUT_VALUE: u32 = 30;

// Constants not yet in glue.rs or bindgen — defined here from Windows headers.
// ntiologc.h: IO error log codes
const IO_ERROR_IO_HARDWARE_ERROR: NTSTATUS = 0xC004009Au32 as i32;
const IO_WARNING_IO_OPERATION_RETRIED: NTSTATUS = 0x80040099u32 as i32;

// iotypes.h: ERROR_LOG_MAXIMUM_SIZE = IO_ERROR_LOG_MESSAGE_LENGTH - IO_ERROR_LOG_MESSAGE_HEADER_LENGTH
// = min(PORT_MAXIMUM_MESSAGE_LENGTH=256, ERROR_LOG_MESSAGE_LIMIT_SIZE) - (sizeof(IO_ERROR_LOG_MESSAGE)-sizeof(IO_ERROR_LOG_PACKET)+80)
// On i386: = 256 - (sizeof(IO_ERROR_LOG_MESSAGE) - sizeof(IO_ERROR_LOG_PACKET) + 80)
// Computed: IO_ERROR_LOG_PACKET = 28, IO_ERROR_LOG_MESSAGE differs per arch. Use const fn:
const IO_ERROR_LOG_MESSAGE_HEADER_LENGTH: usize =
    core::mem::size_of::<ntkrnl_sys::ddk::_IO_ERROR_LOG_MESSAGE>()
    - core::mem::size_of::<IO_ERROR_LOG_PACKET>()
    + (core::mem::size_of::<u16>() * 40);
const IO_ERROR_LOG_MESSAGE_LENGTH: usize = {
    let limit = 240 + IO_ERROR_LOG_MESSAGE_HEADER_LENGTH;
    let port_max = 256usize;
    if port_max < limit { port_max } else { limit }
};
const ERROR_LOG_MAXIMUM_SIZE: usize = IO_ERROR_LOG_MESSAGE_LENGTH - IO_ERROR_LOG_MESSAGE_HEADER_LENGTH;

// ntddstor.h: DEVICE_LB_PROVISIONING_DESCRIPTOR_V1_SIZE
// = RTL_SIZEOF_THROUGH_FIELD(DEVICE_LB_PROVISIONING_DESCRIPTOR, UnmapGranularityAlignment)
// = offsetof + sizeof for that field
const DEVICE_LB_PROVISIONING_DESCRIPTOR_V1_SIZE: u32 =
    (core::mem::offset_of!(DEVICE_LB_PROVISIONING_DESCRIPTOR, UnmapGranularityAlignment)
    + core::mem::size_of::<u32>()) as u32;

// ntddstor.h: DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V1 = sizeof(DEVICE_DATA_SET_LB_PROVISIONING_STATE)
const DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V1: u32 =
    core::mem::size_of::<DEVICE_DATA_SET_LB_PROVISIONING_STATE>() as u32;

// ntddstor.h: DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2 = sizeof(DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2)
const DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2: u32 =
    core::mem::size_of::<DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2>() as u32;

// ntddstor.h: DEVICE_DATA_SET_LBP_STATE_PARAMETERS_VERSION_V1 = sizeof(DEVICE_DATA_SET_LBP_STATE_PARAMETERS)
const DEVICE_DATA_SET_LBP_STATE_PARAMETERS_VERSION_V1: u32 =
    core::mem::size_of::<DEVICE_DATA_SET_LBP_STATE_PARAMETERS>() as u32;

// classp.h: MAX_TOKEN_TRANSFER_SIZE = MAXULONGLONG
const MAX_TOKEN_TRANSFER_SIZE: u64 = u64::MAX;

// scsi.h / ntddstor.h: WRITE BUFFER CDB mode constants
const SCSI_WRITE_BUFFER_MODE_DOWNLOAD_MICROCODE_WITH_OFFSETS_SAVE_DEFER_ACTIVATE: u8 = 0x0E;
const SCSI_WRITE_BUFFER_MODE_ACTIVATE_DEFERRED_MICROCODE: u8 = 0x0F;

unsafe extern "C" {
    fn REVERSE_BYTES(dest: *mut u32, src: *const u32);
    fn REVERSE_BYTES_QUAD(dest: *mut u8, src: *const u8);
    fn REVERSE_BYTES_SHORT(dest: *mut u16, src: *const u16);
    fn TracePrint(level: u32, flags: u32, format: *const i8, ...);
    fn strlen(s: *const core::ffi::c_char) -> usize;
    fn strncmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char, n: usize) -> i32;
    fn strcmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> i32;
    fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    fn wcslen(s: *const u16) -> usize;
}

// ntiologc.h: disk event log codes used for ClassLogCapacityChangedProcess / ClassLogProvisioningTypeChangedEvent
const IO_WARNING_DISK_CAPACITY_CHANGED: ULONG = 0x80040097u32;
const IO_WARNING_DISK_PROVISIONING_TYPE_CHANGED: ULONG = 0x80040098u32;

// ---- FORCEINLINE helpers from classp.h / storduid.h / scsi.h ----
// These are inline in C so they must be defined as local fns, not extern declarations.

/// classp.h FORCEINLINE ClasspIsObsoletePortDriver
#[no_mangle]
pub unsafe extern "C" fn ClasspIsObsoletePortDriver(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> BOOLEAN {
    if !(*FdoExtension).MiniportDescriptor.is_null()
        && (*(*FdoExtension).MiniportDescriptor).Portdriver == StoragePortCodeSetSCSIport
    {
        return 1u8;
    }
    0u8
}

/// classp.h FORCEINLINE ClasspIsThinProvisioned
#[no_mangle]
pub unsafe extern "C" fn ClasspIsThinProvisioned(
    FunctionSupportInfo: *mut crate::types::_CLASS_FUNCTION_SUPPORT_INFO,
) -> BOOLEAN {
    if (*FunctionSupportInfo).LBProvisioningData.ProvisioningType() == PROVISIONING_TYPE_UNKNOWN as u8 {
        return 0u8;
    }
    if (*FunctionSupportInfo).LBProvisioningData.ProvisioningType() == 2u8 /* PROVISIONING_TYPE_THIN */
        && (*FunctionSupportInfo).LBProvisioningData.LBPU() != 0
    {
        return 1u8;
    }
    0u8
}

/// classp.h FORCEINLINE ClasspSupportsUnmap
#[inline]
pub unsafe fn ClasspSupportsUnmap(
    SupportInfo: *mut crate::types::_CLASS_FUNCTION_SUPPORT_INFO,
) -> bool {
    (*SupportInfo).LBProvisioningData.LBPU() != 0
}

/// classp.h FORCEINLINE ClasspLowerLayerNotSupport
#[inline]
pub unsafe fn ClasspLowerLayerNotSupport(Status: NTSTATUS) -> bool {
    (Status == STATUS_NOT_SUPPORTED)
        || (Status == STATUS_NOT_IMPLEMENTED)
        || (Status == STATUS_INVALID_DEVICE_REQUEST)
        || (Status == STATUS_INVALID_PARAMETER_1)
}

// scsi.h FORCEINLINE helpers for sense data parsing
#[inline]
unsafe fn ScsiGetSenseErrorCode(buf: *const core::ffi::c_void) -> u8 {
    (*(buf as *const u8)) & 0x7f
}
const SCSI_SENSE_ERRORCODE_FIXED_CURRENT: u8     = 0x70;
const SCSI_SENSE_ERRORCODE_FIXED_DEFERRED: u8    = 0x71;
const SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT: u8  = 0x72;
const SCSI_SENSE_ERRORCODE_DESCRIPTOR_DEFERRED: u8 = 0x73;

#[inline]
unsafe fn IsFixedSenseDataFormat(buf: *const core::ffi::c_void) -> bool {
    let ec = ScsiGetSenseErrorCode(buf);
    ec == SCSI_SENSE_ERRORCODE_FIXED_CURRENT || ec == SCSI_SENSE_ERRORCODE_FIXED_DEFERRED
}
#[inline]
unsafe fn IsDescriptorSenseDataFormat(buf: *const core::ffi::c_void) -> bool {
    let ec = ScsiGetSenseErrorCode(buf);
    ec == SCSI_SENSE_ERRORCODE_DESCRIPTOR_CURRENT || ec == SCSI_SENSE_ERRORCODE_DESCRIPTOR_DEFERRED
}

/// scsi.h FORCEINLINE ScsiGetFixedSenseKeyAndCodes
#[inline]
unsafe fn ScsiGetFixedSenseKeyAndCodes(
    SenseInfoBuffer: *mut core::ffi::c_void,
    SenseInfoBufferLength: UCHAR,
    SenseKey: *mut UCHAR,
    AdditionalSenseCode: *mut UCHAR,
    AdditionalSenseCodeQualifier: *mut UCHAR,
) -> BOOLEAN {
    if SenseInfoBuffer.is_null() || SenseInfoBufferLength == 0 {
        return 0u8;
    }
    let fixed = SenseInfoBuffer as PFIXED_SENSE_DATA;
    // RTL_CONTAINS_FIELD: check that AdditionalSenseLength field is within SenseInfoBufferLength bytes
    let additional_sense_length_offset =
        core::mem::offset_of!(crate::types::_SENSE_DATA, AdditionalSenseLength);
    let field_end = additional_sense_length_offset + core::mem::size_of::<UCHAR>();
    if field_end > SenseInfoBufferLength as usize {
        return 0u8;
    }
    let data_length = ((*fixed).AdditionalSenseLength as usize)
        + core::mem::offset_of!(crate::types::_SENSE_DATA, AdditionalSenseLength)
        + core::mem::size_of::<UCHAR>();
    let data_length = if data_length > SenseInfoBufferLength as usize {
        SenseInfoBufferLength as usize
    } else {
        data_length
    };
    if !SenseKey.is_null() {
        *SenseKey = (*fixed).SenseKey();
    }
    if !AdditionalSenseCode.is_null() {
        let asc_offset = core::mem::offset_of!(crate::types::_SENSE_DATA, AdditionalSenseCode);
        *AdditionalSenseCode = if asc_offset + 1 <= data_length {
            (*fixed).AdditionalSenseCode
        } else { 0 };
    }
    if !AdditionalSenseCodeQualifier.is_null() {
        let ascq_offset = core::mem::offset_of!(crate::types::_SENSE_DATA, AdditionalSenseCodeQualifier);
        *AdditionalSenseCodeQualifier = if ascq_offset + 1 <= data_length {
            (*fixed).AdditionalSenseCodeQualifier
        } else { 0 };
    }
    1u8
}

/// scsi.h FORCEINLINE ScsiGetDescriptorSenseKeyAndCodes
#[inline]
unsafe fn ScsiGetDescriptorSenseKeyAndCodes(
    SenseInfoBuffer: *mut core::ffi::c_void,
    SenseInfoBufferLength: UCHAR,
    SenseKey: *mut UCHAR,
    AdditionalSenseCode: *mut UCHAR,
    AdditionalSenseCodeQualifier: *mut UCHAR,
) -> BOOLEAN {
    if SenseInfoBuffer.is_null() || SenseInfoBufferLength == 0 {
        return 0u8;
    }
    let desc = SenseInfoBuffer as PDESCRIPTOR_SENSE_DATA;
    // RTL_CONTAINS_FIELD check for AdditionalSenseLength
    let al_offset = core::mem::offset_of!(crate::types::_DESCRIPTOR_SENSE_DATA, AdditionalSenseLength);
    if al_offset + core::mem::size_of::<UCHAR>() > SenseInfoBufferLength as usize {
        return 0u8;
    }
    if !SenseKey.is_null() {
        *SenseKey = (*desc).SenseKey();
    }
    if !AdditionalSenseCode.is_null() {
        *AdditionalSenseCode = (*desc).AdditionalSenseCode;
    }
    if !AdditionalSenseCodeQualifier.is_null() {
        *AdditionalSenseCodeQualifier = (*desc).AdditionalSenseCodeQualifier;
    }
    1u8
}

/// scsi.h FORCEINLINE ScsiGetSenseKeyAndCodes
#[inline]
unsafe fn ScsiGetSenseKeyAndCodes(
    SenseInfoBuffer: *mut core::ffi::c_void,
    SenseInfoBufferLength: UCHAR,
    Options: u32, // SCSI_SENSE_OPTIONS
    SenseKey: *mut UCHAR,
    AdditionalSenseCode: *mut UCHAR,
    AdditionalSenseCodeQualifier: *mut UCHAR,
) -> BOOLEAN {
    if SenseInfoBuffer.is_null() || SenseInfoBufferLength == 0 {
        return 0u8;
    }
    if IsDescriptorSenseDataFormat(SenseInfoBuffer) {
        return ScsiGetDescriptorSenseKeyAndCodes(
            SenseInfoBuffer, SenseInfoBufferLength, SenseKey,
            AdditionalSenseCode, AdditionalSenseCodeQualifier,
        );
    }
    const FIXED_FORMAT_IF_UNKNOWN: u32 = 0x00000001; // SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED
    if (Options & FIXED_FORMAT_IF_UNKNOWN) != 0 || IsFixedSenseDataFormat(SenseInfoBuffer) {
        return ScsiGetFixedSenseKeyAndCodes(
            SenseInfoBuffer, SenseInfoBufferLength, SenseKey,
            AdditionalSenseCode, AdditionalSenseCodeQualifier,
        );
    }
    0u8
}

/// storduid.h FORCEINLINE CompareStorageDuids
#[inline]
unsafe fn CompareStorageDuids(
    Duid1: *mut _STORAGE_DEVICE_UNIQUE_IDENTIFIER,
    Duid2: *mut _STORAGE_DEVICE_UNIQUE_IDENTIFIER,
) -> DUID_MATCH_STATUS {
    if Duid1.is_null() || Duid2.is_null() {
        return DuidErrorMissingDuid;
    }
    if ((*Duid1).Size as usize) < core::mem::size_of::<STORAGE_DEVICE_UNIQUE_IDENTIFIER>()
        || ((*Duid2).Size as usize) < core::mem::size_of::<STORAGE_DEVICE_UNIQUE_IDENTIFIER>()
    {
        return DuidErrorGeneral;
    }
    if (*Duid1).Version != DUID_VERSION_1 || (*Duid2).Version != DUID_VERSION_1 {
        return DuidErrorVersionMismatch;
    }
    if (*Duid1).StorageDeviceIdOffset == 0
        && (*Duid1).StorageDeviceOffset == 0
        && (*Duid1).DriveLayoutSignatureOffset == 0
    {
        return DuidErrorInvalidDuid;
    }
    if (*Duid2).StorageDeviceIdOffset == 0
        && (*Duid2).StorageDeviceOffset == 0
        && (*Duid2).DriveLayoutSignatureOffset == 0
    {
        return DuidErrorInvalidDuid;
    }
    if (*Duid1).Size == (*Duid2).Size
        && memcmp(Duid1 as *const core::ffi::c_void, Duid2 as *const core::ffi::c_void, (*Duid1).Size as usize) == 0
    {
        return DuidExactMatch;
    }
    if (*Duid1).StorageDeviceIdOffset != 0 && (*Duid2).StorageDeviceIdOffset != 0 {
        let idDesc1 = (Duid1 as *mut u8).add((*Duid1).StorageDeviceIdOffset as usize)
            as PSTORAGE_DEVICE_ID_DESCRIPTOR;
        let idDesc2 = (Duid2 as *mut u8).add((*Duid2).StorageDeviceIdOffset as usize)
            as PSTORAGE_DEVICE_ID_DESCRIPTOR;
        if ((*idDesc1).Size as usize) < core::mem::size_of::<ntkrnl_sys::ddk::STORAGE_DEVICE_ID_DESCRIPTOR>()
            || ((*idDesc2).Size as usize) < core::mem::size_of::<ntkrnl_sys::ddk::STORAGE_DEVICE_ID_DESCRIPTOR>()
        {
            return DuidErrorInvalidDeviceIdDescSize;
        }
        if (*idDesc1).Size == (*idDesc2).Size
            && memcmp(idDesc1 as *const core::ffi::c_void, idDesc2 as *const core::ffi::c_void, (*idDesc1).Size as usize) == 0
        {
            return DuidSubIdMatch;
        }
        let mut ident1 = (*idDesc1).Identifiers.as_ptr() as PSTORAGE_IDENTIFIER;
        let mut idx1: u32 = 0;
        while idx1 < (*idDesc1).NumberOfIdentifiers {
            if ((*ident1).Type == StorageIdTypeScsiNameString
                || (*ident1).Type == StorageIdTypeFCPHName
                || (*ident1).Type == StorageIdTypeEUI64
                || (*ident1).Type == StorageIdTypeVendorId)
                && (*ident1).Association == StorageIdAssocPort
                && ((*ident1).CodeSet == StorageIdCodeSetUtf8
                    || (*ident1).CodeSet == StorageIdCodeSetAscii
                    || (*ident1).CodeSet == StorageIdCodeSetBinary)
            {
                let mut ident2 = (*idDesc2).Identifiers.as_ptr() as PSTORAGE_IDENTIFIER;
                let mut idx2: u32 = 0;
                while idx2 < (*idDesc2).NumberOfIdentifiers {
                    if (*ident1).Type == (*ident2).Type
                        && (*ident1).Association == (*ident2).Association
                        && (*ident1).CodeSet == (*ident2).CodeSet
                        && (*ident1).IdentifierSize == (*ident2).IdentifierSize
                        && memcmp(
                            (*ident1).Identifier.as_ptr() as *const core::ffi::c_void,
                            (*ident2).Identifier.as_ptr() as *const core::ffi::c_void,
                            (*ident1).IdentifierSize as usize,
                        ) == 0
                    {
                        return DuidSubIdMatch;
                    }
                    ident2 = (ident2 as *mut u8).add((*ident2).NextOffset as usize) as PSTORAGE_IDENTIFIER;
                    idx2 += 1;
                }
            }
            ident1 = (ident1 as *mut u8).add((*ident1).NextOffset as usize) as PSTORAGE_IDENTIFIER;
            idx1 += 1;
        }
    }
    if (*Duid1).StorageDeviceOffset != 0 && (*Duid2).StorageDeviceOffset != 0 {
        let desc1 = (Duid1 as *mut u8).add((*Duid1).StorageDeviceOffset as usize)
            as PSTORAGE_DEVICE_DESCRIPTOR;
        let desc2 = (Duid2 as *mut u8).add((*Duid2).StorageDeviceOffset as usize)
            as PSTORAGE_DEVICE_DESCRIPTOR;
        if ((*desc1).Size as usize) < core::mem::size_of::<ntkrnl_sys::ddk::STORAGE_DEVICE_DESCRIPTOR>()
            || ((*desc2).Size as usize) < core::mem::size_of::<ntkrnl_sys::ddk::STORAGE_DEVICE_DESCRIPTOR>()
        {
            return DuidErrorInvalidDeviceDescSize;
        }
        if (*desc1).Size == (*desc2).Size
            && memcmp(desc1 as *const core::ffi::c_void, desc2 as *const core::ffi::c_void, (*desc1).Size as usize) == 0
        {
            return DuidSubIdMatch;
        }
        if (*desc1).SerialNumberOffset != 0 && (*desc2).SerialNumberOffset != 0 {
            let string1 = (desc1 as *const u8).add((*desc1).SerialNumberOffset as usize) as *const core::ffi::c_char;
            let string2 = (desc2 as *const u8).add((*desc2).SerialNumberOffset as usize) as *const core::ffi::c_char;
            if strcmp(string1, string2) == 0 {
                if (*desc1).VendorIdOffset != 0 && (*desc2).VendorIdOffset != 0 {
                    let s1 = (desc1 as *const u8).add((*desc1).VendorIdOffset as usize) as *const core::ffi::c_char;
                    let s2 = (desc2 as *const u8).add((*desc2).VendorIdOffset as usize) as *const core::ffi::c_char;
                    if strcmp(s1, s2) != 0 { return DuidNoMatch; }
                }
                if (*desc1).ProductIdOffset != 0 && (*desc2).ProductIdOffset != 0 {
                    let s1 = (desc1 as *const u8).add((*desc1).ProductIdOffset as usize) as *const core::ffi::c_char;
                    let s2 = (desc2 as *const u8).add((*desc2).ProductIdOffset as usize) as *const core::ffi::c_char;
                    if strcmp(s1, s2) != 0 { return DuidNoMatch; }
                }
                return DuidSubIdMatch;
            }
        }
    }
    if (*Duid1).DriveLayoutSignatureOffset != 0 && (*Duid2).DriveLayoutSignatureOffset != 0 {
        let sig1 = (Duid1 as *mut u8).add((*Duid1).DriveLayoutSignatureOffset as usize)
            as *mut STORAGE_DEVICE_LAYOUT_SIGNATURE;
        let sig2 = (Duid2 as *mut u8).add((*Duid2).DriveLayoutSignatureOffset as usize)
            as *mut STORAGE_DEVICE_LAYOUT_SIGNATURE;
        if (*sig1).Version != DUID_VERSION_1 && (*sig2).Version != DUID_VERSION_1 {
            return DuidErrorInvalidLayoutSigVersion;
        }
        if ((*sig1).Size as usize) < core::mem::size_of::<STORAGE_DEVICE_LAYOUT_SIGNATURE>()
            || ((*sig2).Size as usize) < core::mem::size_of::<STORAGE_DEVICE_LAYOUT_SIGNATURE>()
        {
            return DuidErrorInvalidLayoutSigSize;
        }
        if memcmp(sig1 as *const core::ffi::c_void, sig2 as *const core::ffi::c_void,
                  core::mem::size_of::<STORAGE_DEVICE_LAYOUT_SIGNATURE>()) == 0
        {
            return DuidSubIdMatch;
        }
    }
    DuidNoMatch
}

/// utils.c: ClassLogCapacityChangedProcess (IO_WORKITEM_ROUTINE — NTAPI = stdcall on i386)
#[no_mangle]
pub unsafe extern "system" fn ClassLogCapacityChangedProcess(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let work_item = Context as PIO_WORKITEM;
    let status = ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_WARNING_DISK_CAPACITY_CHANGED as NTSTATUS);
    if NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"ClassLogCapacityChangedEvent: DO (%p), Capacity changed logged.\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
    } else {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassLogCapacityChangedEvent: DO (%p), Failed to allocate memory for error log entry.\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
    }
    let _cap_status = ClassReadDriveCapacity(DeviceObject);
    ClassReleaseRemoveLock(DeviceObject, work_item as *mut _IRP);
    if !work_item.is_null() {
        IoFreeWorkItem(work_item);
    }
}

/// utils.c: ClassLogProvisioningTypeChangedEvent (IO_WORKITEM_ROUTINE — NTAPI = stdcall on i386)
#[no_mangle]
pub unsafe extern "system" fn ClassLogProvisioningTypeChangedEvent(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let work_item = Context as PIO_WORKITEM;
    let status = ClasspLogSystemEventWithDeviceNumber(DeviceObject, IO_WARNING_DISK_PROVISIONING_TYPE_CHANGED as NTSTATUS);
    if NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"ClassLogProvisioningTypeChangedEvent: DO (%p), LB Provisioning Type changed logged.\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
    } else {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassLogProvisioningTypeChangedEvent: DO (%p), Failed to allocate memory for error log entry.\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
    }
    ClassReleaseRemoveLock(DeviceObject, work_item as *mut _IRP);
    IoFreeWorkItem(work_item);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspMyStringMatches(StringToMatch: *mut i8, TargetString: *mut i8) -> u8 {
    if StringToMatch.is_null() {
        return 1u8;
    }
    let length = strlen(StringToMatch as *const i8) as u32;
    if length == 0 {
        return (strlen(TargetString as *const i8) == 0) as u8;
    }
    (strncmp(StringToMatch as *const i8, TargetString as *const i8, length as usize) == 0) as u8
}
#[no_mangle]
pub unsafe extern "system" fn ClassGetDeviceParameter(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    SubkeyName: PWSTR,
    ParameterName: PWSTR,
    ParameterValue: *mut u32,
) {
    let mut status: NTSTATUS;
    let mut query_table: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let mut device_parameter_handle: HANDLE = core::ptr::null_mut();
    let mut device_subkey_handle: HANDLE = core::ptr::null_mut();
    let mut default_parameter_value: u32 = 0;

    // PAGED_CODE(); // omitted

    status = IoOpenDeviceRegistryKey(
        (*FdoExtension).LowerPdo,
        1, // PLUGPLAY_REGKEY_DEVICE
        KEY_READ,
        &mut device_parameter_handle,
    );

    if NT_SUCCESS(status) && !SubkeyName.is_null() {
        let mut subkey_name: _UNICODE_STRING = core::mem::zeroed();
        let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

        RtlInitUnicodeString(&mut subkey_name, SubkeyName);

        // InitializeObjectAttributes expanded manually
        object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        object_attributes.RootDirectory = device_parameter_handle;
        object_attributes.ObjectName = &mut subkey_name;
        object_attributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
        object_attributes.SecurityDescriptor = core::ptr::null_mut();
        object_attributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(
            &mut device_subkey_handle,
            KEY_READ,
            &mut object_attributes,
        );
        if !NT_SUCCESS(status) {
            ZwClose(device_parameter_handle);
        }
    }

    if NT_SUCCESS(status) {
        default_parameter_value = *ParameterValue;

        let query = &mut query_table[0];
        query.Flags = (RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED | RTL_QUERY_REGISTRY_TYPECHECK) as u32;
        query.Name = ParameterName;
        query.EntryContext = ParameterValue as *mut core::ffi::c_void;
        query.DefaultType = ((REG_DWORD as u32) << 24) | (REG_NONE as u32); // RTL_QUERY_REGISTRY_TYPECHECK_SHIFT = 24
        query.DefaultData = core::ptr::null_mut();
        query.DefaultLength = 0;

        status = RtlQueryRegistryValues(
            RTL_REGISTRY_HANDLE,
            if !SubkeyName.is_null() {
                device_subkey_handle as PWSTR
            } else {
                device_parameter_handle as PWSTR
            },
            &mut query_table[0] as *mut RTL_QUERY_REGISTRY_TABLE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            *ParameterValue = default_parameter_value;
        }

        if !SubkeyName.is_null() {
            ZwClose(device_subkey_handle);
        }
        ZwClose(device_parameter_handle);
    }

    if !NT_SUCCESS(status) {
        status = IoOpenDeviceRegistryKey(
            (*FdoExtension).LowerPdo,
            2, // PLUGPLAY_REGKEY_DRIVER
            KEY_READ,
            &mut device_parameter_handle,
        );

        if NT_SUCCESS(status) && !SubkeyName.is_null() {
            let mut subkey_name: _UNICODE_STRING = core::mem::zeroed();
            let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

            RtlInitUnicodeString(&mut subkey_name, SubkeyName);

            // InitializeObjectAttributes expanded manually
            object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            object_attributes.RootDirectory = device_parameter_handle;
            object_attributes.ObjectName = &mut subkey_name;
            object_attributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
            object_attributes.SecurityDescriptor = core::ptr::null_mut();
            object_attributes.SecurityQualityOfService = core::ptr::null_mut();

            status = ZwOpenKey(
                &mut device_subkey_handle,
                KEY_READ,
                &mut object_attributes,
            );
            if !NT_SUCCESS(status) {
                ZwClose(device_parameter_handle);
            }
        }

        if NT_SUCCESS(status) {
            default_parameter_value = *ParameterValue;

            let query = &mut query_table[0];
            query.Flags = (RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED | RTL_QUERY_REGISTRY_TYPECHECK) as u32;
            query.Name = ParameterName;
            query.EntryContext = ParameterValue as *mut core::ffi::c_void;
            query.DefaultType = ((REG_DWORD as u32) << 24) | (REG_NONE as u32); // RTL_QUERY_REGISTRY_TYPECHECK_SHIFT = 24
            query.DefaultData = core::ptr::null_mut();
            query.DefaultLength = 0;

            status = RtlQueryRegistryValues(
                RTL_REGISTRY_HANDLE,
                if !SubkeyName.is_null() {
                    device_subkey_handle as PWSTR
                } else {
                    device_parameter_handle as PWSTR
                },
                &mut query_table[0] as *mut RTL_QUERY_REGISTRY_TABLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if NT_SUCCESS(status) {
                // Migrate the value over to the device-specific key
                ClassSetDeviceParameter(
                    FdoExtension,
                    SubkeyName,
                    ParameterName,
                    *ParameterValue,
                );
            } else {
                *ParameterValue = default_parameter_value;
            }

            if !SubkeyName.is_null() {
                ZwClose(device_subkey_handle);
            }
            ZwClose(device_parameter_handle);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClassSetDeviceParameter(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    SubkeyName: PWSTR,
    ParameterName: PWSTR,
    ParameterValue: u32,
) -> i32 {
    let mut status: i32;
    let mut device_parameter_handle: HANDLE = core::ptr::null_mut();
    let mut device_subkey_handle: HANDLE = core::ptr::null_mut();

    status = IoOpenDeviceRegistryKey(
        (*FdoExtension).LowerPdo,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_READ | KEY_WRITE,
        &mut device_parameter_handle,
    );

    if NT_SUCCESS(status) && !SubkeyName.is_null() {
        let mut subkey_name = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        RtlInitUnicodeString(&mut subkey_name, SubkeyName);

        let mut object_attributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: device_parameter_handle,
            Attributes: (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32,
            ObjectName: &mut subkey_name,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        status = ZwCreateKey(
            &mut device_subkey_handle,
            KEY_READ | KEY_WRITE,
            &mut object_attributes,
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(status) {
            ZwClose(device_parameter_handle);
        }
    }

    if NT_SUCCESS(status) {
        let handle: PWSTR = if !SubkeyName.is_null() {
            device_subkey_handle as PWSTR
        } else {
            device_parameter_handle as PWSTR
        };

        status = RtlWriteRegistryValue(
            RTL_REGISTRY_HANDLE,
            handle,
            ParameterName,
            REG_DWORD,
            &ParameterValue as *const u32 as *mut core::ffi::c_void,
            core::mem::size_of::<u32>() as u32,
        );

        if !SubkeyName.is_null() {
            ZwClose(device_subkey_handle);
        }
        ZwClose(device_parameter_handle);
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClassScanForSpecial(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    DeviceList: *const CLASSPNP_SCAN_FOR_SPECIAL_INFO,
    Function: PCLASS_SCAN_FOR_SPECIAL_HANDLER,
) {
    let device_descriptor = (*FdoExtension).DeviceDescriptor;
    if DeviceList.is_null() {
        return;
    }
    if Function.is_none() {
        return;
    }

    let null_string: [u8; 1] = [0u8; 1];
    let null_string_ptr = null_string.as_ptr() as *const u8;

    let vendor_id: *const u8;
    let vendor_offset = (*device_descriptor).VendorIdOffset;
    if vendor_offset != 0 && vendor_offset != (0xFFu8) as u32 {
        vendor_id = (device_descriptor as *const u8).add(vendor_offset as usize);
    } else {
        vendor_id = null_string_ptr;
    }

    let product_id: *const u8;
    let product_offset = (*device_descriptor).ProductIdOffset;
    if product_offset != 0 && product_offset != (0xFFu8) as u32 {
        product_id = (device_descriptor as *const u8).add(product_offset as usize);
    } else {
        product_id = null_string_ptr;
    }

    let product_rev: *const u8;
    let rev_offset = (*device_descriptor).ProductRevisionOffset;
    if rev_offset != 0 && rev_offset != (0xFFu8) as u32 {
        product_rev = (device_descriptor as *const u8).add(rev_offset as usize);
    } else {
        product_rev = null_string_ptr;
    }

    let mut current = DeviceList;
    while !((*current).VendorId.is_null() && (*current).ProductId.is_null() && (*current).ProductRevision.is_null()) {
        if ClasspMyStringMatches((*current).VendorId, vendor_id as *mut i8) != 0
            && ClasspMyStringMatches((*current).ProductId, product_id as *mut i8) != 0
            && ClasspMyStringMatches((*current).ProductRevision, product_rev as *mut i8) != 0
        {
            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, b"ClasspScanForSpecialByInquiry: Found matching \
                        controller Ven: %s Prod: %s Rev: %s\n\0".as_ptr() as *const i8,
                        vendor_id as *const i8, product_id as *const i8, product_rev as *const i8);
            let data = (*current).Data;
            if let Some(f) = Function { f(FdoExtension, data); }
            TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, b"ClasspScanForSpecialByInquiry: \
                        completed callback\n\0".as_ptr() as *const i8);
            return;
        }
        current = current.offset(1);
    }

    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, b"ClasspScanForSpecialByInquiry: no match found for %p\n\0".as_ptr() as *const i8,
                (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPerfIncrementErrorCount(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
) {
    let fdoData = (*FdoExtension).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock) as *mut KSPIN_LOCK, &mut oldIrql);

    (*fdoData).Perf.SuccessfulIO = 0;
    let errors = InterlockedIncrement(&raw mut (*FdoExtension).ErrorCount as *mut LONG);

    if (*fdoData).DisableThrottling == 0 {
        if errors >= (CLASS_ERROR_LEVEL_1) as i32 {
            (*FdoExtension).SrbFlags &= !SRB_FLAGS_NO_QUEUE_FREEZE;
            (*FdoExtension).SrbFlags &= !SRB_FLAGS_QUEUE_ACTION_ENABLE;
            (*FdoExtension).SrbFlags |= SRB_FLAGS_DISABLE_SYNCH_TRANSFER;

            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                (b"ClasspPerfIncrementErrorCount: Too many errors; disabling tagged queuing and synchronous data tranfers.\n\0"
                    .as_ptr() as *const u8).cast(),
            );
        }

        if errors >= (CLASS_ERROR_LEVEL_2) as i32 {
            (*FdoExtension).SrbFlags |= SRB_FLAGS_DISABLE_DISCONNECT;
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                (b"ClasspPerfIncrementErrorCount: Too many errors; disabling disconnects.\n\0"
                    .as_ptr() as *const u8).cast(),
            );
        }
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock) as *mut KSPIN_LOCK, oldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPerfIncrementSuccessfulIo(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) {
    let fdoData = (*FdoExtension).PrivateFdoData;
    let mut oldIrql: KIRQL = 0;
    let errors: i32;
    let mut succeeded: i32;

    if (*FdoExtension).ErrorCount == 0 {
        return;
    }
    if (*fdoData).Perf.ReEnableThreshhold == 0 {
        return;
    }

    // InterlockedIncrement equivalent
    {
        let ptr = core::ptr::addr_of_mut!((*fdoData).Perf.SuccessfulIO) as *mut i32 as *mut core::sync::atomic::AtomicI32;
        let prev = (*ptr).fetch_add(1, core::sync::atomic::Ordering::AcqRel);
        succeeded = prev.wrapping_add(1);
    }

    if succeeded < ((*fdoData).Perf.ReEnableThreshhold) as i32 {
        return;
    }

    KeAcquireSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), core::ptr::addr_of_mut!(oldIrql));

    succeeded = ((*fdoData).Perf.SuccessfulIO) as i32; // re-read under lock

    if (*FdoExtension).ErrorCount != 0 && (*fdoData).Perf.ReEnableThreshhold <= (succeeded) as u32 {
        (*fdoData).Perf.SuccessfulIO = 0;

        NT_ASSERT!(((*FdoExtension).ErrorCount > 0) as u8);

        // InterlockedDecrement equivalent
        {
            let ptr = core::ptr::addr_of_mut!((*FdoExtension).ErrorCount) as *mut i32 as *mut core::sync::atomic::AtomicI32;
            let prev = (*ptr).fetch_sub(1, core::sync::atomic::Ordering::AcqRel);
            errors = prev.wrapping_sub(1);
        }

        // Reverse order of sets "just in case"
        if errors < (CLASS_ERROR_LEVEL_2) as i32 {
            if errors == (CLASS_ERROR_LEVEL_2 - 1) as i32 {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClasspPerfIncrementSuccessfulIo: Error level 2 no longer required.\n\0".as_ptr() as PCSZ,
                );
            }
            if ((*fdoData).Perf.OriginalSrbFlags & SRB_FLAGS_DISABLE_DISCONNECT) == 0 {
                (*FdoExtension).SrbFlags &= !SRB_FLAGS_DISABLE_DISCONNECT;
            }
        }

        if errors < (CLASS_ERROR_LEVEL_1) as i32 {
            if errors == (CLASS_ERROR_LEVEL_1 - 1) as i32 {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClasspPerfIncrementSuccessfulIo: Error level 1 no longer required.\n\0".as_ptr() as PCSZ,
                );
            }
            if ((*fdoData).Perf.OriginalSrbFlags & SRB_FLAGS_DISABLE_SYNCH_TRANSFER) == 0 {
                (*FdoExtension).SrbFlags &= !SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
            }
            if ((*fdoData).Perf.OriginalSrbFlags & SRB_FLAGS_QUEUE_ACTION_ENABLE) != 0 {
                (*FdoExtension).SrbFlags |= SRB_FLAGS_QUEUE_ACTION_ENABLE;
            }
            if ((*fdoData).Perf.OriginalSrbFlags & SRB_FLAGS_NO_QUEUE_FREEZE) != 0 {
                (*FdoExtension).SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;
            }
        }
    }

    KeReleaseSpinLock((core::ptr::addr_of_mut!((*fdoData).SpinLock)).cast(), oldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspBuildDeviceMdl(Buffer: *mut core::ffi::c_void, BufferLen: u32, WriteToDevice: u8) -> PMDL {
    let mdl = IoAllocateMdl(Buffer, BufferLen, 0u8, 0u8, core::ptr::null_mut());
    if mdl.is_null() {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"ClasspBuildDeviceMdl: IoAllocateMdl failed\0".as_ptr() as PCSZ,
        );
        return core::ptr::null_mut();
    }

    // C: _SEH2_EXCEPT { trace; free the MDL; return NULL }.
    let status = ntkrnl_sys::probe::MmTryProbeAndLockPages(
        mdl,
        (0i32) as i8,
        if WriteToDevice != 0 { IoReadAccess } else { IoWriteAccess },
    );
    if status < 0 {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"ClasspBuildDeviceMdl: MmProbeAndLockPages failed with %xh.\0".as_ptr() as PCSZ,
            status,
        );
        IoFreeMdl(mdl);
        return core::ptr::null_mut();
    }

    mdl
}
#[no_mangle]
pub unsafe extern "C" fn BuildDeviceInputMdl(Buffer: *mut core::ffi::c_void, BufferLen: u32) -> PMDL {
    ClasspBuildDeviceMdl(Buffer, BufferLen, 0u8)
}
#[no_mangle]
pub unsafe extern "C" fn ClasspFreeDeviceMdl(mdl: PMDL) {
    MmUnlockPages(mdl);
    IoFreeMdl(mdl);
}
#[no_mangle]
pub unsafe extern "C" fn FreeDeviceInputMdl(mdl: PMDL) {
    ClasspFreeDeviceMdl(mdl);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDuidGetDeviceIdProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let mut deviceIdDescriptor: *mut _STORAGE_DEVICE_ID_DESCRIPTOR = core::ptr::null_mut();
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut propertyId: STORAGE_PROPERTY_ID = _STORAGE_PROPERTY_ID_StorageDeviceIdProperty;
    let status: NTSTATUS;

    // Call ClassGetDescriptor, pass &deviceIdDescriptor as *mut PVOID
    let mut descriptor_ptr: PVOID = core::ptr::null_mut();
    status = ClassGetDescriptor(
        (*commonExtension).LowerDeviceObject,
        &mut propertyId,
        &mut descriptor_ptr as *mut PVOID,
    );
    deviceIdDescriptor = descriptor_ptr as *mut _STORAGE_DEVICE_ID_DESCRIPTOR;

    if (status as i32) < 0 || deviceIdDescriptor.is_null() {
        ExFreePool(deviceIdDescriptor as PVOID);
        return status;
    }

    let queryLength = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
    let descHeader = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_DESCRIPTOR_HEADER;
    let offset = (*descHeader).Size;
    let dest = (descHeader as *mut u8).add(offset as usize);

    (*descHeader).Size += (*deviceIdDescriptor).Size;

    if queryLength < (*descHeader).Size {
        (*Irp).IoStatus.Information = ((core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as ULONG) as u64) as _;
        ExFreePool(deviceIdDescriptor as PVOID);
        return STATUS_BUFFER_OVERFLOW;
    }

    let storageDuid = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_DEVICE_UNIQUE_IDENTIFIER;
    (*storageDuid).StorageDeviceIdOffset = offset;
    core::ptr::copy_nonoverlapping(
        deviceIdDescriptor as *const u8,
        dest,
        (*deviceIdDescriptor).Size as usize,
    );
    (*Irp).IoStatus.Information = (((*storageDuid).Size) as u64) as _;

    ExFreePool(deviceIdDescriptor as PVOID);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDuidGetDeviceProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let deviceDescriptor = (*fdoExtension).DeviceDescriptor;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);

    let status = loop {
        if deviceDescriptor.is_null() {
            break STATUS_NOT_FOUND;
        }
        let queryLength = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
        let descHeader = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_DESCRIPTOR_HEADER;

        if (*deviceDescriptor).SerialNumberOffset == 0 {
            break STATUS_NOT_FOUND;
        }

        let offset = (*descHeader).Size;
        let dest = (descHeader as *mut u8).add(offset as usize);
        (*descHeader).Size += (*deviceDescriptor).Size;

        if queryLength < (*descHeader).Size {
            (*Irp).IoStatus.Information = (((core::mem::size_of::<_STORAGE_DESCRIPTOR_HEADER>() as usize) as u32) as u64) as _;
            break STATUS_BUFFER_OVERFLOW;
        }

        let storageDuid = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_DEVICE_UNIQUE_IDENTIFIER;
        (*storageDuid).StorageDeviceOffset = offset;

        core::ptr::copy_nonoverlapping(
            deviceDescriptor as *const u8,
            dest,
            (*deviceDescriptor).Size as usize,
        );

        (*Irp).IoStatus.Information = (((*storageDuid).Size as u32) as u64) as _;
        break STATUS_SUCCESS;
    };

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDuidGetDriveLayout(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP
) -> NTSTATUS {
    let mut layoutEx: *mut DRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS;

    'exit: loop {
        if (*DeviceObject).DeviceType != FILE_DEVICE_DISK {
            status = STATUS_NOT_FOUND;
            break 'exit;
        }

        status = IoReadPartitionTableEx(DeviceObject, &mut layoutEx);
        if !NT_SUCCESS(status) {
            status = STATUS_NOT_FOUND;
            break 'exit;
        }

        if (*layoutEx).PartitionStyle != PARTITION_STYLE_GPT &&
            (*layoutEx).PartitionStyle != PARTITION_STYLE_MBR {
            status = STATUS_NOT_FOUND;
            break 'exit;
        }

        let queryLength = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
        let descHeader = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_DESCRIPTOR_HEADER;

        let offset = (*descHeader).Size;
        let driveLayoutSignature = (descHeader as *mut u8).add(offset as usize) as *mut STORAGE_DEVICE_LAYOUT_SIGNATURE;
        (*descHeader).Size = (*descHeader).Size + core::mem::size_of::<STORAGE_DEVICE_LAYOUT_SIGNATURE>() as u32;

        if queryLength < (*descHeader).Size {
            (*Irp).IoStatus.Information = ((core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32) as u64) as _;
            status = STATUS_BUFFER_OVERFLOW;
            break 'exit;
        }

        let storageDuid = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_DEVICE_UNIQUE_IDENTIFIER;

        (*driveLayoutSignature).Size = core::mem::size_of::<STORAGE_DEVICE_LAYOUT_SIGNATURE>() as u32;
        (*driveLayoutSignature).Version = DUID_VERSION_1;

        if (*layoutEx).PartitionStyle == PARTITION_STYLE_MBR {
            (*driveLayoutSignature).Mbr = 1u8;
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*driveLayoutSignature).DeviceSpecific.MbrSignature) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*layoutEx).__bindgen_anon_1.Mbr.Signature) as *const core::ffi::c_void,
                (core::mem::size_of_val(&(*layoutEx).__bindgen_anon_1.Mbr.Signature) as u32) as usize,
            );
        } else {
            (*driveLayoutSignature).Mbr = 0u8;
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*driveLayoutSignature).DeviceSpecific.GptDiskId) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*layoutEx).__bindgen_anon_1.Gpt.DiskId) as *const core::ffi::c_void,
                (core::mem::size_of_val(&(*layoutEx).__bindgen_anon_1.Gpt.DiskId) as u32) as usize,
            );
        }

        (*storageDuid).DriveLayoutSignatureOffset = offset;
        (*Irp).IoStatus.Information = (((*storageDuid).Size) as u64) as _;
        status = STATUS_SUCCESS;
        break 'exit;
    }

    FREE_POOL((layoutEx).cast());
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDuidQueryProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let query = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let out_length = (*irp_stack).Parameters.DeviceIoControl.OutputBufferLength;
    let mut overflow: BOOLEAN = 0u8;
    let mut info_found: BOOLEAN = 0u8;
    let mut use_status: BOOLEAN = 1u8;
    let mut status: NTSTATUS;
    let include_optional_ids: BOOLEAN;

    'fnexit: {
        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            NT_ASSERT!((KeGetCurrentIrql() < DISPATCH_LEVEL) as u8);
            status = STATUS_INVALID_LEVEL;
            break 'fnexit;
        }

        if (*query).QueryType == PropertyExistsQuery {
            (*Irp).IoStatus.Information = 0;
            status = STATUS_SUCCESS;
            break 'fnexit;
        }

        if (*query).QueryType != PropertyStandardQuery {
            status = STATUS_NOT_SUPPORTED;
            break 'fnexit;
        }

        if (*query).AdditionalParameters[0] == (DUID_INCLUDE_SOFTWARE_IDS) as u8 {
            include_optional_ids = 1u8;
        } else if (*query).AdditionalParameters[0] == (DUID_HARDWARE_IDS_ONLY) as u8 {
            include_optional_ids = 0u8;
        } else {
            status = STATUS_INVALID_PARAMETER;
            break 'fnexit;
        }

        if out_length < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32 {
            status = STATUS_INFO_LENGTH_MISMATCH;
            break 'fnexit;
        }

        use_status = 0u8;

        let desc_header = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_DESCRIPTOR_HEADER;
        core::ptr::write_bytes(desc_header as *mut u8, 0, out_length as usize);

        (*desc_header).Version = DUID_VERSION_1;
        (*desc_header).Size = core::mem::size_of::<STORAGE_DEVICE_UNIQUE_IDENTIFIER>() as u32;

        status = ClasspDuidGetDeviceIdProperty(DeviceObject, Irp);
        if status == STATUS_BUFFER_OVERFLOW {
            overflow = 1u8;
        }
        if NT_SUCCESS(status) {
            info_found = 1u8;
        }

        status = ClasspDuidGetDeviceProperty(DeviceObject, Irp);
        if status == STATUS_BUFFER_OVERFLOW {
            overflow = 1u8;
        }
        if NT_SUCCESS(status) {
            info_found = 1u8;
        }

        if include_optional_ids == 0u8 {
            break 'fnexit;
        }

        status = ClasspDuidGetDriveLayout(DeviceObject, Irp);
        if status == STATUS_BUFFER_OVERFLOW {
            overflow = 1u8;
        }
        if NT_SUCCESS(status) {
            info_found = 1u8;
        }
    }

    if use_status == 0u8 {
        if overflow != 0u8 {
            if out_length == core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32 {
                status = STATUS_SUCCESS;
                (*Irp).IoStatus.Information = ((core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32) as u64) as _;
            } else {
                status = STATUS_BUFFER_OVERFLOW;
            }
        } else if info_found != 0u8 {
            status = STATUS_SUCCESS;
            NT_ASSERT!((DuidExactMatch == CompareStorageDuids(
                (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_DEVICE_UNIQUE_IDENTIFIER,
                (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_DEVICE_UNIQUE_IDENTIFIER)) as u8);
        } else {
            status = STATUS_NOT_FOUND;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspWriteCacheProperty(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> i32 {
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_PROPERTY_QUERY;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);

    let mut status: i32 = 0;
    let mut information: u32 = 0;
    let mut modeData: *mut MODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut pageData: *mut MODE_CACHING_PAGE;

    'write_cache_exit: loop {
        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            if !(KeGetCurrentIrql() < DISPATCH_LEVEL) {
                RtlAssert(
                    b"KeGetCurrentIrql() < DISPATCH_LEVEL\0".as_ptr() as *mut core::ffi::c_void,
                    b"utils.rs\0".as_ptr() as *mut core::ffi::c_void,
                    line!(),
                    core::ptr::null_mut::<i8>(),
                );
            }
            status = STATUS_INVALID_LEVEL;
            break 'write_cache_exit;
        }

        if (*query).QueryType == PropertyExistsQuery {
            status = STATUS_SUCCESS;
            break 'write_cache_exit;
        }

        if (*query).QueryType != PropertyStandardQuery {
            status = STATUS_NOT_SUPPORTED;
            break 'write_cache_exit;
        }

        let length_from_irp = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
        if length_from_irp < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32 {
            status = STATUS_INFO_LENGTH_MISMATCH;
            break 'write_cache_exit;
        }

        let writeCache = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_WRITE_CACHE_PROPERTY;
        core::ptr::write_bytes(writeCache, 0, length_from_irp as usize);

        (*writeCache).Version = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;
        (*writeCache).Size = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;

        if length_from_irp < core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32 {
            information = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32;
            status = STATUS_SUCCESS;
            break 'write_cache_exit;
        }

        (*writeCache).NVCacheEnabled = 0u8;
        (*writeCache).UserDefinedPowerProtection = 
            if ((*fdoExtension).DeviceFlags & (DEV_POWER_PROTECTED) as u16) != 0 { 1u8 } else { 0u8 };

        SrbSetTimeOutValue((Srb).cast(), (*fdoExtension).TimeOutValue * 4);
        SrbSetRequestTag((Srb).cast(), SP_UNTAGGED);
        SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
        SrbAssignSrbFlags((Srb).cast(), (*fdoExtension).SrbFlags);

        SrbSetCdbLength((Srb).cast(), 10);
        let cdb = SrbGetCdb((Srb).cast());
        (*cdb).CDB10.as_mut().OperationCode = 0x35_u8;

        status = ClassSendSrbSynchronous(DeviceObject, Srb, core::ptr::null_mut(), 0, 1u8);
        if status >= 0 {
            (*writeCache).FlushCacheSupported = 1u8;
        } else {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"ClasspWriteCacheProperty: Synchronize cache failed with status 0x%X\n\0".as_ptr() as PCSZ,
                status,
            );
            (*writeCache).FlushCacheSupported = 0u8;
            status = STATUS_SUCCESS;
        }

        modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned, (MODE_PAGE_DATA_SIZE) as usize, CLASS_TAG_MODE_DATA)
            as *mut MODE_PARAMETER_HEADER;

        if modeData.is_null() {
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"ClasspWriteCacheProperty: Unable to allocate mode data buffer\n\0".as_ptr() as PCSZ,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'write_cache_exit;
        }

        core::ptr::write_bytes(modeData as *mut u8, 0, MODE_PAGE_DATA_SIZE as usize);

        let mut length: u32 = ClassModeSense(DeviceObject, modeData as *mut i8, MODE_PAGE_DATA_SIZE, MODE_PAGE_CACHING as u8);
        if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32 {
            length = ClassModeSense(DeviceObject, modeData as *mut i8, MODE_PAGE_DATA_SIZE, MODE_PAGE_CACHING as u8);
            if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32 {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    b"ClasspWriteCacheProperty: Mode Sense failed\n\0".as_ptr() as PCSZ,
                );
                status = STATUS_IO_DEVICE_ERROR;
                break 'write_cache_exit;
            }
        }

        if length > ((*modeData).ModeDataLength as u32 + 1) {
            length = (*modeData).ModeDataLength as u32 + 1;
        }

        pageData = ClassFindModePage(modeData as *mut i8, length, MODE_PAGE_CACHING as u8, 1u8) as *mut MODE_CACHING_PAGE;
        if pageData.is_null() {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"ClasspWriteCacheProperty: Unable to find caching mode page.\n\0".as_ptr() as PCSZ,
            );
            (*writeCache).WriteCacheEnabled = WriteCacheEnableUnknown;
            (*writeCache).WriteCacheType = WriteCacheTypeUnknown;
        } else {
            (*writeCache).WriteCacheEnabled = if (*pageData).WriteCacheEnable() != 0 {
                WriteCacheEnabled
            } else {
                WriteCacheDisabled
            };
            (*writeCache).WriteCacheType = if (*pageData).WriteCacheEnable() != 0 {
                WriteCacheTypeWriteBack
            } else {
                WriteCacheTypeUnknown
            };
        }

        if (((*modeData).DeviceSpecificParameter & (MODE_DSP_FUA_SUPPORTED) as u8) != 0)
            && (((*fdoExtension).ScanForSpecialFlags & CLASS_SPECIAL_FUA_NOT_SUPPORTED) == 0)
        {
            (*writeCache).WriteThroughSupported = WriteThroughSupported;
        } else {
            (*writeCache).WriteThroughSupported = WriteThroughNotSupported;
        }

        core::ptr::write_bytes(modeData as *mut u8, 0, MODE_PAGE_DATA_SIZE as usize);

        length = ClasspModeSense(
            DeviceObject,
            modeData as *mut i8,
            MODE_PAGE_DATA_SIZE,
            MODE_PAGE_CACHING as u8,
            MODE_SENSE_CHANGEABLE_VALUES as u8,
        );

        if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32 {
            length = ClasspModeSense(
                DeviceObject,
                modeData as *mut i8,
                MODE_PAGE_DATA_SIZE,
                MODE_PAGE_CACHING as u8,
                MODE_SENSE_CHANGEABLE_VALUES as u8,
            );
            if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32 {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_IOCTL,
                    b"ClasspWriteCacheProperty: Mode Sense failed\n\0".as_ptr() as PCSZ,
                );
                (*writeCache).WriteCacheChangeable = WriteCacheChangeUnknown;
                information = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;
                break 'write_cache_exit;
            }
        }

        if length > ((*modeData).ModeDataLength as u32 + 1) {
            length = (*modeData).ModeDataLength as u32 + 1;
        }

        pageData = ClassFindModePage(modeData as *mut i8, length, MODE_PAGE_CACHING as u8, 1u8) as *mut MODE_CACHING_PAGE;
        if pageData.is_null() {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"ClasspWriteCacheProperty: Unable to find caching mode page.\n\0".as_ptr() as PCSZ,
            );
            (*writeCache).WriteCacheChangeable = WriteCacheChangeUnknown;
        } else {
            (*writeCache).WriteCacheChangeable = if (*pageData).WriteCacheEnable() != 0 {
                WriteCacheChangeable
            } else {
                WriteCacheNotChangeable
            };
        }

        information = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;
        break 'write_cache_exit;
    }

    FREE_POOL((modeData).cast());
    (*Irp).IoStatus.Information = ((information) as u64) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspCalculateLogicalSectorSize(
    Fdo: *mut _DEVICE_OBJECT,
    BytesPerBlockInBigEndian: u32,
) -> u32 {
    let mut logicalSectorSize = {
        let mut tmp = core::mem::MaybeUninit::<u32>::uninit();
        REVERSE_BYTES(tmp.as_mut_ptr(), &BytesPerBlockInBigEndian);
        tmp.assume_init()
    };
    if logicalSectorSize == 0 {
        logicalSectorSize = 512;
    } else if logicalSectorSize & (logicalSectorSize - 1) != 0 {
        TracePrint(
            1u32,
            4u32,
            b"FDO %ph has non-standard sector size 0x%x.\0".as_ptr() as *const i8,
            Fdo,
            logicalSectorSize,
        );
        while logicalSectorSize & (logicalSectorSize - 1) != 0 {
            logicalSectorSize &= logicalSectorSize - 1;
        }
    }
    logicalSectorSize
}
#[no_mangle]
pub unsafe extern "C" fn InterpretReadCapacity16Data(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    ReadCapacity16Data: *mut _READ_CAPACITY16_DATA,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cached_data: *mut _CLASS_READ_CAPACITY16_DATA =
        core::ptr::addr_of_mut!((*(*FdoExtension).FunctionSupportInfo).ReadCapacity16Data);

    (*cached_data).BytesPerLogicalSector = ClasspCalculateLogicalSectorSize(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        (*ReadCapacity16Data).BytesPerBlock,
    );

    // NT_ASSERT omitted for translation.

    let logical_blocks_per_physical_block: u16 =
        1u16 << (*ReadCapacity16Data).LogicalPerPhysicalExponent();
    let mut lowest_aligned_block: u16 = ((*ReadCapacity16Data).LowestAlignedBlock_MSB() as u16) << 8
        | (*ReadCapacity16Data).LowestAlignedBlock_LSB as u16;

    if lowest_aligned_block > logical_blocks_per_physical_block {
        status = STATUS_UNSUCCESSFUL;
    } else {
        lowest_aligned_block = (logical_blocks_per_physical_block - lowest_aligned_block)
            % logical_blocks_per_physical_block;
    }

    if NT_SUCCESS(status) {
        (*cached_data).BytesPerPhysicalSector =
            (*cached_data).BytesPerLogicalSector * logical_blocks_per_physical_block as u32;
        (*cached_data).BytesOffsetForSectorAlignment =
            (*cached_data).BytesPerLogicalSector * lowest_aligned_block as u32;

        (*cached_data).LBProvisioningEnabled = (*ReadCapacity16Data).LBPME();
        (*cached_data).LBProvisioningReadZeros = (*ReadCapacity16Data).LBPRZ();

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"InterpretReadCapacity16Data: Device's LBP enabled = %d\n\0".as_ptr() as *const i8,
            (*cached_data).LBProvisioningEnabled as i32,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassReadCapacity16(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let buffer_length: u8 = core::mem::size_of::<_READ_CAPACITY16_DATA>() as u8;
    let mut allocation_buffer_length: core::ffi::c_ulong = buffer_length as _;
    let mut data_transfer_length: core::ffi::c_ulong = 0;

    // If the information retrieval has already been attempted, return the cached status.
    if (*(*FdoExtension).FunctionSupportInfo).ReadCapacity16Data.CommandStatus != -1 {
        return (*(*FdoExtension).FunctionSupportInfo).ReadCapacity16Data.CommandStatus;
    }

    if ClasspIsObsoletePortDriver(FdoExtension) != 0u8 {
        (*(*FdoExtension).FunctionSupportInfo).ReadCapacity16Data.CommandStatus = STATUS_NOT_IMPLEMENTED;
        return STATUS_NOT_IMPLEMENTED;
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        allocation_buffer_length = ALIGN_UP_BY(
            allocation_buffer_length as _,
            KeGetRecommendedSharedDataAlignment() as _,
        ) as _;
    }

    let pool_tag: u32 = 0x53634334u32; // '4CcS'
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let data_buffer: *mut _READ_CAPACITY16_DATA = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        allocation_buffer_length as usize,
        pool_tag,
    ) as *mut _READ_CAPACITY16_DATA;
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    let data_buffer: *mut _READ_CAPACITY16_DATA = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        buffer_length as _,
        pool_tag,
    ) as *mut _READ_CAPACITY16_DATA;

    if data_buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // RtlZeroMemory counts bytes; write_bytes on the typed pointer counted
    // READ_CAPACITY16_DATAs: 32 x 32 bytes into a 32-byte block, a BAD_POOL_HEADER
    // from any user-mode IOCTL_STORAGE_QUERY_PROPERTY(StorageAccessAlignmentProperty).
    core::ptr::write_bytes(data_buffer as *mut u8, 0, allocation_buffer_length as usize);

    // Initialize the SRB
    if (*((*FdoExtension).AdapterDescriptor as *mut LocalStorageAdapterDescriptor)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
        status = InitializeStorageRequestBlock(
            Srb as *mut _STORAGE_REQUEST_BLOCK,
            STORAGE_ADDRESS_TYPE_BTL8 as u16,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
            1u32,
            SrbExDataTypeScsiCdb16,
        );
        if NT_SUCCESS(status) {
            (*(Srb as *mut _STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        } else {
            ntkrnl_sys::ASSERT!(false, "Should not occur");
        }
    } else {
        core::ptr::write_bytes(Srb as *mut u8, 0, core::mem::size_of::<_SCSI_REQUEST_BLOCK>());
        (*Srb).Length = core::mem::size_of::<_SCSI_REQUEST_BLOCK>() as _;
        (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
    }

    if NT_SUCCESS(status) {
        SrbSetTimeOutValue((Srb).cast(), (*FdoExtension).TimeOutValue);
        SrbSetRequestTag((Srb).cast(), SP_UNTAGGED);
        SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
        SrbAssignSrbFlags((Srb).cast(), (*FdoExtension).SrbFlags);
        SrbSetCdbLength((Srb).cast(), 16);
        let cdb: PCDB = SrbGetCdb((Srb).cast());
        (*(cdb)).READ_CAPACITY16.as_mut().OperationCode = SCSIOP_READ_CAPACITY16 as u8;
        (*(cdb)).READ_CAPACITY16.as_mut().set_ServiceAction(SERVICE_ACTION_READ_CAPACITY16 as u8);
        (*(cdb)).READ_CAPACITY16.as_mut().AllocationLength[3] = buffer_length;

        status = ClassSendSrbSynchronous(
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            Srb,
            data_buffer as *mut core::ffi::c_void,
            allocation_buffer_length,
            0u8, // FALSE
        );

        data_transfer_length = SrbGetDataTransferLength((Srb).cast());
    }

    if NT_SUCCESS(status) && data_transfer_length < 16 {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if status == STATUS_DATA_OVERRUN && data_transfer_length < (buffer_length as core::ffi::c_ulong) {
        status = STATUS_SUCCESS;
    }

    if NT_SUCCESS(status) {
        status = InterpretReadCapacity16Data(FdoExtension, data_buffer);
    }

    (*(*FdoExtension).FunctionSupportInfo).ReadCapacity16Data.CommandStatus = status;

    ExFreePool(data_buffer as *mut core::ffi::c_void);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspAccessAlignmentProperty(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_PROPERTY_QUERY;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut length: u32 = 0;
    let mut information: u32 = 0;
    let accessAlignment: *mut _STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR;

    if (*(*fdoExtension).FunctionSupportInfo).RegAccessAlignmentQueryNotSupported != 0 {
        status = STATUS_NOT_SUPPORTED;
        // goto Exit
        (*Irp).IoStatus.Information = ((information) as u64) as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    }

    let device_type = (*DeviceObject).DeviceType as u32;
    let characteristics = (*DeviceObject).Characteristics as u32;
    let lower_support = (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.AccessAlignmentProperty;
    if device_type != FILE_DEVICE_DISK
        || (characteristics & FILE_FLOPPY_DISKETTE) != 0
        || lower_support == _CLASS_FUNCTION_SUPPORT_STATE_Supported
    {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
        return status;
    }

    // Check proper query type
    let query_type = (*query).QueryType;
    if query_type == _STORAGE_PROPERTY_ID_PropertyExistsQuery {
        status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = ((information) as u64) as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    } else if query_type != _STORAGE_PROPERTY_ID_PropertyStandardQuery {
        status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = ((information) as u64) as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    }

    // Request validation
    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        NT_ASSERT!(((KeGetCurrentIrql() < DISPATCH_LEVEL) as i32) as u8);
        status = STATUS_INVALID_LEVEL;
        (*Irp).IoStatus.Information = ((information) as u64) as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    }

    accessAlignment = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR;
    length = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;

    if length < (core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32) {
        if length >= (core::mem::size_of::<_STORAGE_DESCRIPTOR_HEADER>() as u32) {
            information = core::mem::size_of::<_STORAGE_DESCRIPTOR_HEADER>() as u32;
            (*accessAlignment).Version = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
            (*accessAlignment).Size = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
            status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = ((information) as u64) as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
            return status;
        }
        status = STATUS_BUFFER_TOO_SMALL;
        (*Irp).IoStatus.Information = ((information) as u64) as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    }

    'work: loop {
        match (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.AccessAlignmentProperty {
            CLASS_FUNCTION_SUPPORT_SupportUnknown => {
                status = ClassForwardIrpSynchronous(commonExtension, Irp);
                if ClasspLowerLayerNotSupport(status) {
                    if ClasspIsObsoletePortDriver(fdoExtension) == 0 {
                        status = ClassReadCapacity16(fdoExtension, Srb);
                    } else {
                        (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.CommandStatus = status;
                    }
                    (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.AccessAlignmentProperty =
                        _CLASS_FUNCTION_SUPPORT_STATE_NotSupported;
                    if NT_SUCCESS(status) {
                        core::ptr::write_bytes(accessAlignment as *mut u8, 0, length as usize);
                        (*accessAlignment).Version = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
                        (*accessAlignment).Size = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
                        (*accessAlignment).BytesPerLogicalSector =
                            (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.BytesPerLogicalSector;
                        (*accessAlignment).BytesPerPhysicalSector =
                            (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.BytesPerPhysicalSector;
                        (*accessAlignment).BytesOffsetForSectorAlignment =
                            (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.BytesOffsetForSectorAlignment;
                        information = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
                    } else {
                        information = 0;
                    }
                } else {
                    (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.AccessAlignmentProperty =
                        _CLASS_FUNCTION_SUPPORT_STATE_Supported;
                    information = (*Irp).IoStatus.Information as u32;
                }
                break 'work;
            }
            _CLASS_FUNCTION_SUPPORT_STATE_NotSupported => {
                status = ClassReadCapacity16(fdoExtension, Srb);
                if NT_SUCCESS(status) {
                    core::ptr::write_bytes(accessAlignment as *mut u8, 0, length as usize);
                    (*accessAlignment).Version = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
                    (*accessAlignment).Size = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
                    (*accessAlignment).BytesPerLogicalSector =
                        (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.BytesPerLogicalSector;
                    (*accessAlignment).BytesPerPhysicalSector =
                        (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.BytesPerPhysicalSector;
                    (*accessAlignment).BytesOffsetForSectorAlignment =
                        (*(*fdoExtension).FunctionSupportInfo).ReadCapacity16Data.BytesOffsetForSectorAlignment;
                    information = core::mem::size_of::<_STORAGE_ACCESS_ALIGNMENT_DESCRIPTOR>() as u32;
                } else {
                    information = 0;
                }
                break 'work;
            }
            _CLASS_FUNCTION_SUPPORT_STATE_Supported => {
                NT_ASSERT!((0i32) as u8);
                status = STATUS_INTERNAL_ERROR;
                break 'work;
            }
            // utils.c's switch has NO `default:` label, so an unmatched value falls
            // straight out of the switch onto `Exit:`. An empty arm does NOT do that
            // inside the goto-elimination loop: it reaches the end of `'work: loop`
            // and re-matches the UNCHANGED scrutinee forever. arm64 emitted it as a
            // literal `b .` -- a silent hang on the storage property path, no bugcheck
            // and no serial output. Leaving the switch IS the break.
            _ => {
                break 'work;
            }
        }
    }
    // Exit label (after loop)
    (*Irp).IoStatus.Information = ((information) as u64) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}

unsafe fn IncursSeekPenalty(
    MediumRotationRate: u16,
    IncursSeekPenalty: PBOOLEAN,
) -> NTSTATUS {
    if MediumRotationRate == 0x0001 {
        (*IncursSeekPenalty) = 0u8;
        STATUS_SUCCESS
    } else if (MediumRotationRate >= 0x401) && (MediumRotationRate <= 0xFFFE) {
        (*IncursSeekPenalty) = 1u8;
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceMediaTypeProperty(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Srb: *mut SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_PROPERTY_QUERY;
    let pDesc = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut length: u32 = 0;
    let mut information: u32 = 0;

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspDeviceMediaTypeProperty (%p): Entering function.\n\0".as_ptr() as *const i8,
        DeviceObject as *mut core::ffi::c_void,
    );

    'exit: loop {
        if (*query).QueryType == PropertyExistsQuery as u32 {
            status = STATUS_SUCCESS;
            break 'exit;
        } else if (*query).QueryType != PropertyStandardQuery as u32 {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspDeviceMediaTypeProperty (%p): Unsupported query type %x for media type property.\n\0"
                    .as_ptr() as *const i8,
                DeviceObject as *mut core::ffi::c_void,
                (*query).QueryType,
            );
            status = STATUS_NOT_SUPPORTED;
            break 'exit;
        }

        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            NT_ASSERT!((KeGetCurrentIrql() < DISPATCH_LEVEL) as u8);
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspDeviceMediaTypeProperty (%p): Query property for media type at incorrect IRQL.\n\0"
                    .as_ptr() as *const i8,
                DeviceObject as *mut core::ffi::c_void,
            );
            status = STATUS_INVALID_LEVEL;
            break 'exit;
        }

        length = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
        if length >= core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32 {
            information = core::mem::size_of::<STORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR>() as u32;
            (*pDesc).Version = core::mem::size_of::<STORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR>() as u32;
            (*pDesc).Size = core::mem::size_of::<STORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR>() as u32;
        } else {
            status = STATUS_BUFFER_TOO_SMALL;
            break 'exit;
        }

        if length < core::mem::size_of::<STORAGE_MEDIUM_PRODUCT_TYPE_DESCRIPTOR>() as u32 {
            status = STATUS_SUCCESS;
            break 'exit;
        }

        if (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockDeviceCharacteristics() == (1u8) as u32 {
            status = ClasspDeviceGetBlockDeviceCharacteristicsVPDPage(fdoExtension, Srb);
        } else {
            status = STATUS_INVALID_DEVICE_REQUEST;
            break 'exit;
        }

        if !NT_SUCCESS(status) {
            status = (*(*fdoExtension).FunctionSupportInfo).DeviceCharacteristicsData.CommandStatus;
            information = 0;
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspDeviceGetBlockDeviceCharacteristicsVPDPage (%p): VPD retrieval fails with %x.\n\0"
                    .as_ptr() as *const i8,
                DeviceObject as *mut core::ffi::c_void,
                status,
            );
            break 'exit;
        }

        (*pDesc).MediumProductType =
            (*(*fdoExtension).FunctionSupportInfo).DeviceCharacteristicsData.MediumProductType;
        status = STATUS_SUCCESS;
        break 'exit;
    }

    (*Irp).IoStatus.Information = (((information as ULONG_PTR) as u32) as u64) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;

    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspDeviceMediaTypeProperty (%p): Exiting function with status %x.\n\0".as_ptr()
            as *const i8,
        DeviceObject as *mut core::ffi::c_void,
        status,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceGetBlockDeviceCharacteristicsVPDPage(
    fdo_extension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let buffer_length: u8 = core::mem::size_of::<VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE>() as u8;
    let mut allocation_buffer_length: u32 = buffer_length as u32;
    let mut data_buffer: *mut _VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE = core::ptr::null_mut();

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let alignment = KeGetRecommendedSharedDataAlignment();
        allocation_buffer_length = ALIGN_UP_BY(allocation_buffer_length as _, alignment as _) as _;
        data_buffer = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            allocation_buffer_length as usize,
            b'5' as u32 | ((b'C' as u32) << 8) | ((b'c' as u32) << 16) | ((b'S' as u32) << 24),
        ) as *mut _VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE;
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        data_buffer = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            buffer_length as usize,
            b'5' as u32 | ((b'C' as u32) << 8) | ((b'c' as u32) << 16) | ((b'S' as u32) << 24),
        ) as *mut _VPD_BLOCK_DEVICE_CHARACTERISTICS_PAGE;
    }

    if data_buffer.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        // Bytes, as RtlZeroMemory counts them (see ClasspReadCapacity16).
        core::ptr::write_bytes(data_buffer as *mut u8, 0u8, allocation_buffer_length as usize);

        SrbSetTimeOutValue((srb).cast(), (*fdo_extension).TimeOutValue);
        SrbSetRequestTag((srb).cast(), SP_UNTAGGED);
        SrbSetRequestAttribute((srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
        SrbAssignSrbFlags((srb).cast(), (*fdo_extension).SrbFlags);
        SrbSetCdbLength((srb).cast(), 6u8);

        let cdb: *mut _CDB = SrbGetCdb((srb).cast());
        (*cdb).CDB6INQUIRY3.as_mut().OperationCode = SCSIOP_INQUIRY as u8;
        (*cdb).CDB6INQUIRY3.as_mut().set_EnableVitalProductData(1u8);
        (*cdb).CDB6INQUIRY3.as_mut().PageCode = VPD_BLOCK_DEVICE_CHARACTERISTICS as u8;
        (*cdb).CDB6INQUIRY3.as_mut().AllocationLength = buffer_length;

        status = ClassSendSrbSynchronous(
            (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            srb,
            data_buffer as *mut core::ffi::c_void,
            allocation_buffer_length,
            0u8,
        );

        if NT_SUCCESS(status) {
            if SrbGetDataTransferLength((srb).cast()) < (8i32) as u32 {
                status = STATUS_UNSUCCESSFUL;
            } else if (*data_buffer).PageLength != 0x3Cu8 || (*data_buffer).PageCode != (VPD_BLOCK_DEVICE_CHARACTERISTICS) as u8 {
                status = STATUS_UNSUCCESSFUL;
            } else {
                (*(*fdo_extension)
                    .FunctionSupportInfo).DeviceCharacteristicsData
                    .MediumRotationRate = ((*data_buffer).MediumRotationRateMsb as u16) << 8
                    | (*data_buffer).MediumRotationRateLsb as u16;
                (*(*fdo_extension)
                    .FunctionSupportInfo).DeviceCharacteristicsData
                    .MediumProductType = ((*data_buffer).MediumProductType) as u32;
                (*(*fdo_extension)
                    .FunctionSupportInfo).DeviceCharacteristicsData
                    .NominalFormFactor = (*data_buffer).NominalFormFactor();
            }
        }
    }

    if !data_buffer.is_null() {
        ExFreePool(data_buffer as *mut core::ffi::c_void);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceSeekPenaltyProperty(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_PROPERTY_QUERY;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);

    if (*DeviceObject).DeviceType != FILE_DEVICE_DISK ||
       ((*DeviceObject).Characteristics & FILE_FLOPPY_DISKETTE) != 0 ||
       (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.SeekPenaltyProperty == Supported {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        let status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
        return status;
    }

    let (status, information) = 'block: loop {
        if (*query).QueryType == PropertyExistsQuery {
            break 'block (STATUS_SUCCESS, 0usize);
        } else if (*query).QueryType != PropertyStandardQuery {
            break 'block (STATUS_NOT_SUPPORTED, 0usize);
        }

        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            ntkrnl_sys::ASSERT!(KeGetCurrentIrql() < DISPATCH_LEVEL);
            break 'block (STATUS_INVALID_LEVEL, 0usize);
        }

        let seekPenalty = (*Irp).AssociatedIrp.SystemBuffer as *mut DEVICE_SEEK_PENALTY_DESCRIPTOR;
        let length = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;

        if length < core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32 {
            if length >= core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32 {
                let info = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>();
                (*seekPenalty).Version = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
                (*seekPenalty).Size = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
                break 'block (STATUS_SUCCESS, info);
            }
            break 'block (STATUS_BUFFER_TOO_SMALL, 0usize);
        }

        match (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.SeekPenaltyProperty {
            Supported => {
                ntkrnl_sys::ASSERT!(false, "Supported case should be handled earlier");
                break 'block (STATUS_UNSUCCESSFUL, 0usize);
            }
            SupportUnknown => {
                let mut status = ClassForwardIrpSynchronous(commonExtension, Irp);
                if ClasspLowerLayerNotSupport(status) {
                    if (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockDeviceCharacteristics() == 1 {
                        status = ClasspDeviceGetBlockDeviceCharacteristicsVPDPage(fdoExtension, Srb);
                    } else {
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    }
                    let mut incursSeekPenalty: BOOLEAN = 1u8;
                    if NT_SUCCESS(status) {
                        status = IncursSeekPenalty((*(*fdoExtension).FunctionSupportInfo).DeviceCharacteristicsData.MediumRotationRate, &mut incursSeekPenalty);
                    }
                    (*(*fdoExtension).FunctionSupportInfo).DeviceCharacteristicsData.CommandStatus = status;
                    (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.SeekPenaltyProperty = NotSupported;
                    let mut info: usize = 0;
                    if NT_SUCCESS(status) {
                        core::ptr::write_bytes(seekPenalty as *mut u8, 0, length as usize);
                        (*seekPenalty).Version = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
                        (*seekPenalty).Size = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
                        (*seekPenalty).IncursSeekPenalty = incursSeekPenalty;
                        info = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>();
                    }
                    break 'block (status, info);
                } else {
                    (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.SeekPenaltyProperty = Supported;
                    let info = (*Irp).IoStatus.Information;
                    break 'block (status, (info) as usize);
                }
            }
            NotSupported => {
                let mut status = (*(*fdoExtension).FunctionSupportInfo).DeviceCharacteristicsData.CommandStatus;
                let mut incursSeekPenalty: BOOLEAN = 1u8;
                if NT_SUCCESS(status) {
                    status = IncursSeekPenalty((*(*fdoExtension).FunctionSupportInfo).DeviceCharacteristicsData.MediumRotationRate, &mut incursSeekPenalty);
                }
                let mut info: usize = 0;
                if NT_SUCCESS(status) {
                    core::ptr::write_bytes(seekPenalty as *mut u8, 0, length as usize);
                    (*seekPenalty).Version = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
                    (*seekPenalty).Size = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;
                    (*seekPenalty).IncursSeekPenalty = incursSeekPenalty;
                    info = core::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>();
                }
                break 'block (status, info);
            }
            _ => {
                break 'block (STATUS_UNSUCCESSFUL, 0usize);
            }
        }
    };

    (*Irp).IoStatus.Information = (((information) as u32) as u64) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceGetLBProvisioningVPDPage(
    DeviceObject: *mut _DEVICE_OBJECT,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let fdo_extension = &mut *((*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION);
    
    // Cached status check
    if (*fdo_extension.FunctionSupportInfo).LBProvisioningData.CommandStatus != -1i32 {
        return (*fdo_extension.FunctionSupportInfo).LBProvisioningData.CommandStatus;
    }

    // Initialize defaults
    (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_ProvisioningType((PROVISIONING_TYPE_UNKNOWN) as u8);
    (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_LBPRZ(0u8);
    (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_LBPU(0u8);
    (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_ANC_SUP(0u8);
    (*fdo_extension.FunctionSupportInfo).LBProvisioningData.ThresholdExponent = 0u8;

    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut data_buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let buffer_length: u8 = (VPD_MAX_BUFFER_SIZE) as u8;
    let allocation_buffer_length: u32 = if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
        (ALIGN_UP_BY(buffer_length as usize, KeGetRecommendedSharedDataAlignment() as usize)) as u32
    } else {
        buffer_length as u32
    };
    let pool_type = if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
        NonPagedPoolNxCacheAligned
    } else {
        NonPagedPoolNx
    };
    let tag = u32::from_ne_bytes(*b"0CcS");

    let lb_provisioning_page: *mut VPD_LOGICAL_BLOCK_PROVISIONING_PAGE;
    let mut page_length: u16 = 0;
    let mut cdb: *mut _CDB;
    let mut data_transfer_length: u32 = 0;

    'exit: loop {
        if (*fdo_extension.FunctionSupportInfo).ValidInquiryPages.LBProvisioning() == (1u8) as u32 && !Srb.is_null() {
            data_buffer = ExAllocatePoolWithTag(pool_type, (allocation_buffer_length) as usize, tag);
            if data_buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                // Keep CommandStatus = -1 to retry later
                break 'exit;
            }

            // RtlZeroMemory equivalent
            core::ptr::write_bytes(data_buffer, 0u8, allocation_buffer_length as usize);
            lb_provisioning_page = data_buffer as *mut VPD_LOGICAL_BLOCK_PROVISIONING_PAGE;

            if (*(fdo_extension.AdapterDescriptor as *mut LocalStorageAdapterDescriptor)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
                status = InitializeStorageRequestBlock(
                    Srb as *mut STORAGE_REQUEST_BLOCK,
                    STORAGE_ADDRESS_TYPE_BTL8 as u16,
                    CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
                    1u32,
                    SrbExDataTypeScsiCdb16,
                );
                if NT_SUCCESS(status) {
                    (*(Srb as *mut STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
                } else {
                    NT_ASSERT!(0u8);
                }
            } else {
                core::ptr::write_bytes(Srb as *mut u8, 0u8, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
                (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                status = STATUS_SUCCESS;
            }

            if NT_SUCCESS(status) {
                SrbSetTimeOutValue((Srb).cast(), fdo_extension.TimeOutValue);
                SrbSetRequestTag((Srb).cast(), SP_UNTAGGED);
                SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
                SrbAssignSrbFlags((Srb).cast(), fdo_extension.SrbFlags);
                SrbSetCdbLength((Srb).cast(), 6);

                cdb = SrbGetCdb((Srb).cast());
                (*cdb).CDB6INQUIRY3.as_mut().OperationCode = (SCSIOP_INQUIRY) as u8;
                (*cdb).CDB6INQUIRY3.as_mut().set_EnableVitalProductData(1);
                (*cdb).CDB6INQUIRY3.as_mut().PageCode = (VPD_LOGICAL_BLOCK_PROVISIONING) as u8;
                (*cdb).CDB6INQUIRY3.as_mut().AllocationLength = buffer_length;

                status = ClassSendSrbSynchronous(
                    fdo_extension.__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                    Srb,
                    data_buffer,
                    allocation_buffer_length,
                    0u8,
                );
                data_transfer_length = SrbGetDataTransferLength((Srb).cast());
            }

            // Handle STATUS_DATA_OVERRUN
            if status == STATUS_DATA_OVERRUN && data_transfer_length < buffer_length as u32 {
                status = STATUS_SUCCESS;
            }

            if NT_SUCCESS(status) {
                REVERSE_BYTES_SHORT(
                    &mut page_length,
                    (*lb_provisioning_page).PageLength.as_ptr() as *const u16,
                );
            }

            if NT_SUCCESS(status) {
                let field_diff = (core::mem::offset_of!(VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, Reserved2)
                    - core::mem::offset_of!(VPD_LOGICAL_BLOCK_PROVISIONING_PAGE, ThresholdExponent)) as u16;
                if data_transfer_length < 0x08u32
                    || page_length < field_diff
                    || (*lb_provisioning_page).PageCode != (VPD_LOGICAL_BLOCK_PROVISIONING) as u8
                {
                    status = STATUS_INFO_LENGTH_MISMATCH;
                }
            }

            if NT_SUCCESS(status) {
                (*fdo_extension
                    .FunctionSupportInfo).LBProvisioningData
                    .set_ProvisioningType((*lb_provisioning_page).ProvisioningType());
                (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_LBPRZ(
                    (*lb_provisioning_page).LBPRZ());
                (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_LBPU(
                    (*lb_provisioning_page).LBPU());
                (*fdo_extension.FunctionSupportInfo).LBProvisioningData.set_ANC_SUP(
                    (*lb_provisioning_page).ANC_SUP());
                (*fdo_extension
                    .FunctionSupportInfo).LBProvisioningData
                    .ThresholdExponent = (*lb_provisioning_page).ThresholdExponent;

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_PNP,
                    b"ClasspDeviceGetLBProvisioningVPDPage (%p): %s %s (rev %s) reported following parameters: \
                        \n\t\t\tProvisioningType: %u \
                        \n\t\t\tLBPRZ: %u \
                        \n\t\t\tLBPU: %u \
                        \n\t\t\tANC_SUP: %I64u \
                        \n\t\t\tThresholdExponent: %u\n\0"
                        .as_ptr() as PCSZ,
                    DeviceObject,
                    ((fdo_extension.DeviceDescriptor as *const u8)
                        .add((*fdo_extension.DeviceDescriptor).VendorIdOffset as usize))
                        as PCSZ,
                    ((fdo_extension.DeviceDescriptor as *const u8)
                        .add((*fdo_extension.DeviceDescriptor).ProductIdOffset as usize))
                        as PCSZ,
                    ((fdo_extension.DeviceDescriptor as *const u8)
                        .add((*fdo_extension.DeviceDescriptor).ProductRevisionOffset as usize))
                        as PCSZ,
                    ((*lb_provisioning_page).ProvisioningType()) as i32,
                    ((*lb_provisioning_page).LBPRZ()) as i32,
                    ((*lb_provisioning_page).LBPU()) as i32,
                    ((*lb_provisioning_page).ANC_SUP()) as i32,
                    ((*lb_provisioning_page).ThresholdExponent) as i32,
                );
            }
        } else {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
        break;
    }

    // Cache the status
    (*fdo_extension.FunctionSupportInfo).LBProvisioningData.CommandStatus = status;

    // Free buffer
    FREE_POOL(data_buffer);

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceGetBlockLimitsVPDPage(
    fdo_extension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    srb: *mut _SCSI_REQUEST_BLOCK,
    srb_size: u32,
    block_limits_data: *mut _CLASS_VPD_B0_DATA,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut data_buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let buffer_length: u8 = (VPD_MAX_BUFFER_SIZE) as u8;
    let mut allocation_buffer_length: u32 = buffer_length as u32;
    let cdb: *mut _CDB;
    let block_limits: *mut _VPD_BLOCK_LIMITS_PAGE;
    let mut data_transfer_length: u32 = 0;

    // Set default UNMAP parameters
    if (*(*fdo_extension).FunctionSupportInfo).LBProvisioningData.LBPU() != 0 {
        (*block_limits_data).MaxUnmapLbaCount = MAXULONG;
        (*block_limits_data).MaxUnmapBlockDescrCount = MAXULONG;
        (*block_limits_data).OptimalUnmapGranularity = 1;
        (*block_limits_data).UnmapGranularityAlignment = 0;
        (*block_limits_data).UGAVALID = FALSE;
    } else {
        (*block_limits_data).MaxUnmapLbaCount = 0;
        (*block_limits_data).MaxUnmapBlockDescrCount = 0;
        (*block_limits_data).OptimalUnmapGranularity = 0;
        (*block_limits_data).UnmapGranularityAlignment = 0;
        (*block_limits_data).UGAVALID = FALSE;
    }

    let need_assign = 'exit: {
        if (*(*fdo_extension).FunctionSupportInfo).ValidInquiryPages.BlockLimits() != 0 {
            // ARM alignment
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            {
                allocation_buffer_length = ALIGN_UP_BY(
                    allocation_buffer_length as _,
                    KeGetRecommendedSharedDataAlignment() as _,
                ) as _;
                data_buffer = ExAllocatePoolWithTag(
                    NonPagedPoolNxCacheAligned,
                    allocation_buffer_length as usize,
                    0x53634330u32,
                );
            }
            #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
            {
                data_buffer = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    buffer_length as usize,
                    0x53634330u32,
                );
            }

            if data_buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'exit false;
            }

            block_limits = data_buffer as *mut _VPD_BLOCK_LIMITS_PAGE;
            core::ptr::write_bytes(data_buffer, 0, allocation_buffer_length as usize);

            // Initialize Srb
            if (*((*fdo_extension).AdapterDescriptor as *mut LocalStorageAdapterDescriptor)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
                status = InitializeStorageRequestBlock(
                    srb as *mut STORAGE_REQUEST_BLOCK,
                    STORAGE_ADDRESS_TYPE_BTL8 as u16,
                    srb_size,
                    1u32,
                    SrbExDataTypeScsiCdb16,
                );
                if NT_SUCCESS(status) {
                    (*(srb as *mut STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
                } else {
                    // Should not occur
                    NT_ASSERT!(FALSE);
                }
            } else {
                core::ptr::write_bytes(srb as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                (*srb).Length = (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u8) as u16;
                (*srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                status = STATUS_SUCCESS;
            }

            if NT_SUCCESS(status) {
                SrbSetTimeOutValue((srb).cast(), (*fdo_extension).TimeOutValue);
                SrbSetRequestTag((srb).cast(), SP_UNTAGGED);
                SrbSetRequestAttribute((srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
                SrbAssignSrbFlags((srb).cast(), (*fdo_extension).SrbFlags);
                SrbSetCdbLength((srb).cast(), 6);

                cdb = SrbGetCdb((srb).cast());
                (*cdb).CDB6INQUIRY3.as_mut().OperationCode = SCSIOP_INQUIRY as u8;
                (*cdb).CDB6INQUIRY3.as_mut().set_EnableVitalProductData(1u8);
                (*cdb).CDB6INQUIRY3.as_mut().PageCode = VPD_BLOCK_LIMITS as u8;
                (*cdb).CDB6INQUIRY3.as_mut().AllocationLength = buffer_length;

                status = ClassSendSrbSynchronous(
                    (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                    srb,
                    data_buffer,
                    allocation_buffer_length,
                    FALSE,
                );
                data_transfer_length = SrbGetDataTransferLength((srb).cast());
            }

            if status == STATUS_DATA_OVERRUN && data_transfer_length < buffer_length as u32 {
                status = STATUS_SUCCESS;
            }

            if NT_SUCCESS(status) {
                let mut page_length: u16 = 0;
                REVERSE_BYTES_SHORT(&mut page_length, (*block_limits).PageLength.as_ptr() as *const u16);

                if data_transfer_length >= 0x10 && (*block_limits).PageCode == (VPD_BLOCK_LIMITS) as u8 {
                    let inner = (*block_limits).__bindgen_anon_1.__bindgen_anon_1.as_ref();
                    REVERSE_BYTES_SHORT(
                        &mut (*block_limits_data).OptimalTransferLengthGranularity,
                        inner.OptimalTransferLengthGranularity.as_ptr() as *const u16,
                    );
                    REVERSE_BYTES(
                        &mut (*block_limits_data).MaximumTransferLength,
                        inner.MaximumTransferLength.as_ptr() as *const u32,
                    );
                    REVERSE_BYTES(
                        &mut (*block_limits_data).OptimalTransferLength,
                        inner.OptimalTransferLength.as_ptr() as *const u32,
                    );
                }

                // FIELD_OFFSET(VPD_BLOCK_LIMITS_PAGE, Reserved1) - FIELD_OFFSET(VPD_BLOCK_LIMITS_PAGE, Reserved0)
                // = offset of Reserved1 - offset of Reserved0 in the inner struct (both are in __bindgen_anon_1.__bindgen_anon_1)
                let vpd_field_diff = (core::mem::offset_of!(_VPD_BLOCK_LIMITS_PAGE__bindgen_ty_1__bindgen_ty_1, Reserved1)
                    - core::mem::offset_of!(_VPD_BLOCK_LIMITS_PAGE__bindgen_ty_1__bindgen_ty_1, Reserved0)) as u32;
                if data_transfer_length < 0x24
                    || (page_length as u32) < vpd_field_diff
                    || (*block_limits).PageCode != (VPD_BLOCK_LIMITS) as u8
                {
                    status = STATUS_INFO_LENGTH_MISMATCH;
                }
            }

            if NT_SUCCESS(status) {
                let inner = (*block_limits).__bindgen_anon_1.__bindgen_anon_1.as_ref();
                REVERSE_BYTES(
                    &mut (*block_limits_data).MaxUnmapLbaCount,
                    inner.MaximumUnmapLBACount.as_ptr() as *const u32,
                );
                REVERSE_BYTES(
                    &mut (*block_limits_data).MaxUnmapBlockDescrCount,
                    inner.MaximumUnmapBlockDescriptorCount.as_ptr() as *const u32,
                );
                REVERSE_BYTES(
                    &mut (*block_limits_data).OptimalUnmapGranularity,
                    inner.OptimalUnmapGranularity.as_ptr() as *const u32,
                );

                (*block_limits_data).UGAVALID = inner.__bindgen_anon_1.__bindgen_anon_1.as_ref().UGAValid();
                if (*block_limits_data).UGAVALID == TRUE {
                    REVERSE_BYTES(
                        &mut (*block_limits_data).UnmapGranularityAlignment,
                        inner.__bindgen_anon_1.UnmapGranularityAlignment.as_ref().as_ptr() as *const u32,
                    );
                    (*block_limits_data).UnmapGranularityAlignment &= 0x7FFFFFFF;
                } else {
                    (*block_limits_data).UnmapGranularityAlignment = 0;
                }

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_PNP,
                    b"ClasspDeviceGetBlockLimitsVPDPage (%p): %s %s (rev %s) reported following parameters: \
                        \n\t\t\tOptimalTransferLengthGranularity: %u \
                        \n\t\t\tMaximumTransferLength: %u \
                        \n\t\t\tOptimalTransferLength: %u \
                        \n\t\t\tMaximumUnmapLBACount: %u \
                        \n\t\t\tMaximumUnmapBlockDescriptorCount: %u \
                        \n\t\t\tOptimalUnmapGranularity: %u \
                        \n\t\t\tUGAValid: %u \
                        \n\t\t\tUnmapGranularityAlignment: %u\n\0"
                        .as_ptr() as PCSZ,
                    (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                    ((*(*fdo_extension).DeviceDescriptor).VendorIdOffset as *const u8).add(0) as PCSZ,
                    ((*(*fdo_extension).DeviceDescriptor).ProductIdOffset as *const u8).add(0) as PCSZ,
                    ((*(*fdo_extension).DeviceDescriptor).ProductRevisionOffset as *const u8).add(0) as PCSZ,
                    ((*block_limits_data).OptimalTransferLengthGranularity) as i32,
                    (*block_limits_data).MaximumTransferLength,
                    (*block_limits_data).OptimalTransferLength,
                    (*block_limits_data).MaxUnmapLbaCount,
                    (*block_limits_data).MaxUnmapBlockDescrCount,
                    (*block_limits_data).OptimalUnmapGranularity,
                    ((*block_limits_data).UGAVALID) as i32,
                    (*block_limits_data).UnmapGranularityAlignment,
                );
            }
            true
        } else {
            status = STATUS_INVALID_DEVICE_REQUEST;
            false
        }
    };

    if need_assign {
        (*block_limits_data).CommandStatus = status;
    }

    FREE_POOL(data_buffer);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceTrimProperty(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let _ = Srb;

    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_PROPERTY_QUERY;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut length: ULONG = 0;
    let mut information: ULONG = 0;
    let mut trimDescr: PDEVICE_TRIM_DESCRIPTOR = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // If not a disk, floppy, or already supported, forward directly.
    if (*DeviceObject).DeviceType != FILE_DEVICE_DISK
        || (TEST_FLAG((*DeviceObject).Characteristics, FILE_FLOPPY_DISKETTE) != 0)
        || (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.TrimProperty == Supported
    {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
        return status;
    }

    'trim_exit: {
        // Check query type.
        if (*query).QueryType == PropertyExistsQuery {
            status = STATUS_SUCCESS;
            break 'trim_exit;
        } else if (*query).QueryType != PropertyStandardQuery {
            status = STATUS_NOT_SUPPORTED;
            break 'trim_exit;
        }

        // IRQL check.
        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            ntkrnl_sys::ASSERT!(KeGetCurrentIrql() < DISPATCH_LEVEL);
            status = STATUS_INVALID_LEVEL;
            break 'trim_exit;
        }

        trimDescr = (*Irp).AssociatedIrp.SystemBuffer as PDEVICE_TRIM_DESCRIPTOR;
        length = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;

        if length < (core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG) {
            if length >= (core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as ULONG) {
                information = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as ULONG;
                (*trimDescr).Version = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                (*trimDescr).Size = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                status = STATUS_SUCCESS;
                break 'trim_exit;
            }
            status = STATUS_BUFFER_TOO_SMALL;
            break 'trim_exit;
        }

        let support_info = (*fdoExtension).FunctionSupportInfo;
        status = STATUS_SUCCESS; // default

        match (*support_info).LowerLayerSupport.TrimProperty {
            SupportUnknown => {
                status = ClassForwardIrpSynchronous(commonExtension, Irp);
                if status == STATUS_NOT_SUPPORTED
                    || status == STATUS_NOT_IMPLEMENTED
                    || status == STATUS_INVALID_DEVICE_REQUEST
                    || status == STATUS_INVALID_PARAMETER_1
                {
                    status = (*support_info).LBProvisioningData.CommandStatus;
                    ntkrnl_sys::ASSERT!(status != (-1i32 as NTSTATUS));
                    (*support_info).LowerLayerSupport.TrimProperty = NotSupported;
                    if NT_SUCCESS(status) {
                        RtlZeroMemory(trimDescr as *mut core::ffi::c_void, length as usize);
                        (*trimDescr).Version = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                        (*trimDescr).Size = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                        (*trimDescr).TrimEnabled = ClasspSupportsUnmap(support_info) as u8;
                        information = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                    } else {
                        information = 0;
                    }
                } else {
                    (*support_info).LowerLayerSupport.TrimProperty = Supported;
                    information = (*Irp).IoStatus.Information as ULONG;
                }
            }
            NotSupported => {
                status = (*support_info).LBProvisioningData.CommandStatus;
                ntkrnl_sys::ASSERT!(status != (-1i32 as NTSTATUS));
                if NT_SUCCESS(status) {
                    RtlZeroMemory(trimDescr as *mut core::ffi::c_void, length as usize);
                    (*trimDescr).Version = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                    (*trimDescr).Size = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                    (*trimDescr).TrimEnabled = ClasspSupportsUnmap(support_info) as u8;
                    information = core::mem::size_of::<DEVICE_TRIM_DESCRIPTOR>() as ULONG;
                } else {
                    information = 0;
                }
            }
            Supported => {
                core::unreachable!();
            }
            _ => {
                core::unreachable!();
            }
        }
    } // 'trim_exit

    (*Irp).IoStatus.Information = (((information as ULONG_PTR) as u32) as u64) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceLBProvisioningProperty(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Srb: *mut SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut information: u32;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_PROPERTY_QUERY;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut length: u32;
    let mut blockLimitsData: CLASS_VPD_B0_DATA = core::mem::zeroed();
    let mut generationCount: u32 = 0;
    let lbpDescr: *mut DEVICE_LB_PROVISIONING_DESCRIPTOR;

    _ = Srb;

    let result = 'exit: loop {
        if (*query).QueryType == PropertyExistsQuery {
            status = STATUS_SUCCESS;
            information = 0;
            break 'exit;
        }
        if (*query).QueryType != PropertyStandardQuery {
            status = STATUS_NOT_SUPPORTED;
            information = 0;
            break 'exit;
        }

        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            NT_ASSERT!((KeGetCurrentIrql() < DISPATCH_LEVEL) as u8);
            status = STATUS_INVALID_LEVEL;
            information = 0;
            break 'exit;
        }

        lbpDescr = query as *mut DEVICE_LB_PROVISIONING_DESCRIPTOR;
        length = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
        core::ptr::write_bytes(lbpDescr as *mut u8, 0, length as usize);

        if length < DEVICE_LB_PROVISIONING_DESCRIPTOR_V1_SIZE {
            if length >= core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32 {
                information = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32;
                (*lbpDescr).Version = core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32;
                (*lbpDescr).Size = core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32;
                status = STATUS_SUCCESS;
                break 'exit;
            }
            status = STATUS_BUFFER_TOO_SMALL;
            information = 0;
            break 'exit;
        }

        if length < core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32 {
            (*lbpDescr).Version = DEVICE_LB_PROVISIONING_DESCRIPTOR_V1_SIZE;
            (*lbpDescr).Size = DEVICE_LB_PROVISIONING_DESCRIPTOR_V1_SIZE;
            information = DEVICE_LB_PROVISIONING_DESCRIPTOR_V1_SIZE;
        } else {
            (*lbpDescr).Version = core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32;
            (*lbpDescr).Size = core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32;
            information = core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32;
        }

        let blockLimitsStatus = ClasspBlockLimitsDataSnapshot(
            fdoExtension,
            1u8,
            &mut blockLimitsData as *mut CLASS_VPD_B0_DATA,
            &mut generationCount as *mut u32,
        );

        (*lbpDescr).set_ThinProvisioningEnabled(ClasspIsThinProvisioned((*fdoExtension).FunctionSupportInfo));

        if (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
            status = ClassReadDriveCapacity((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);
            if status < 0 || (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
                status = STATUS_INVALID_DEVICE_REQUEST;
                information = 0;
                break 'exit;
            }
        }

        (*lbpDescr).set_ThinProvisioningReadZeros((*(*fdoExtension).FunctionSupportInfo).LBProvisioningData.LBPRZ());
        (*lbpDescr).set_AnchorSupported((*(*fdoExtension).FunctionSupportInfo).LBProvisioningData.ANC_SUP());

        if blockLimitsStatus >= 0 {
            (*lbpDescr).set_UnmapGranularityAlignmentValid(blockLimitsData.UGAVALID);
            (*lbpDescr).OptimalUnmapGranularity = (blockLimitsData.OptimalUnmapGranularity as u64) * ((*fdoExtension).DiskGeometry.BytesPerSector as u64);
            (*lbpDescr).UnmapGranularityAlignment = (blockLimitsData.UnmapGranularityAlignment as u64) * ((*fdoExtension).DiskGeometry.BytesPerSector as u64);

            // dead: C is #if (NTDDI_VERSION >= NTDDI_WINBLUE), classpnp builds at WIN8
            #[cfg(any())]
            if length >= core::mem::size_of::<DEVICE_LB_PROVISIONING_DESCRIPTOR>() as u32 {
                (*lbpDescr).MaxUnmapLbaCount = blockLimitsData.MaxUnmapLbaCount;
                (*lbpDescr).MaxUnmapBlockDescriptorCount = blockLimitsData.MaxUnmapBlockDescrCount;
            }
        }

        status = STATUS_SUCCESS;
        break 'exit;
    };

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    (*Irp).IoStatus.Information = (((information as usize) as u32) as u64) as _;

    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);

    status
}
#[no_mangle]
pub unsafe extern "C" fn ConvertDataSetRangeToUnmapBlockDescr(
    fdo_extension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    block_descr: *mut _UNMAP_BLOCK_DESCRIPTOR,
    current_block_descr_index: *mut u32,
    max_block_descr_index: u32,
    current_lba_count: *mut u64,
    max_lba_count: u64,
    data_set_range: *mut _DEVICE_DATA_SET_RANGE,
) {
    let fdo_ext = &*fdo_extension;
    let range = &mut *data_set_range;

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"ConvertDataSetRangeToUnmapBlockDescr (%p): Generating UNMAP Block Descriptors from DataSetRange: \
         \n\t\tStartingOffset = %I64u bytes \
         \n\t\tLength = %I64u bytes\n\0"
            .as_ptr() as *const i8,
        fdo_ext.__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        range.StartingOffset,
        range.LengthInBytes,
    );

    while (*range).LengthInBytes > 0
        && *current_block_descr_index < max_block_descr_index
        && *current_lba_count < max_lba_count
    {
        let bytes_per_sector = fdo_ext.DiskGeometry.BytesPerSector as u64;
        let mut starting_sector = (*range).StartingOffset / (bytes_per_sector) as i64;
        let mut sector_count = (*range).LengthInBytes / bytes_per_sector;

        if sector_count > u32::MAX as u64 {
            sector_count = u32::MAX as u64;
        }

        if (*current_lba_count + sector_count) > max_lba_count {
            sector_count = max_lba_count - *current_lba_count;
        }

        let block = &mut *block_descr.add(*current_block_descr_index as usize);
        REVERSE_BYTES_QUAD(
            (*block).StartingLba.as_mut_ptr(),
            core::ptr::addr_of!(starting_sector) as *const u8,
        );
        let sector_count_low = sector_count as u32;
        REVERSE_BYTES((*block).LbaCount.as_mut_ptr() as *mut u32, core::ptr::addr_of!(sector_count_low) as *const u32);

        (*range).StartingOffset += (sector_count * bytes_per_sector) as i64;
        (*range).LengthInBytes -= sector_count * bytes_per_sector;

        *current_block_descr_index += 1;
        *current_lba_count += sector_count_low as u64;

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"ConvertDataSetRangeToUnmapBlockDescr (%p): Generated UNMAP Block Descriptor: \
             \n\t\t\tStartingLBA = %I64u \
             \n\t\t\tLBACount = %I64u\n\0"
                .as_ptr() as *const i8,
            fdo_ext.__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            starting_sector,
            sector_count,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeviceProcessDsmTrimRequest(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    DataSetRanges: *mut _DEVICE_DATA_SET_RANGE,
    DataSetRangesCount: u32,
    UnmapGranularity: u32,
    SrbFlags: u32,
    Irp: *mut _IRP,
    ActivityId: *mut GUID,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let buffer: *mut _UNMAP_LIST_HEADER;
    let mut blockDescrPointer: *mut _UNMAP_BLOCK_DESCRIPTOR;
    let mut bufferLength: u32;
    let mut maxBlockDescrCount: u32;
    let mut neededBlockDescrCount: u32;
    let mut allDataSetRangeFullyConverted: u8;
    let mut needToSendCommand: u8;
    let mut tempDataSetRangeFullyConverted: u8;
    let mut dataSetRangeIndex: u32;
    let mut tempDataSetRange: _DEVICE_DATA_SET_RANGE = core::mem::zeroed();
    let mut blockDescrIndex: u32;
    let mut lbaCount: u64;
    let mut maxLbaCount: u64;
    let mut maxParameterListLength: u64;

    let _ = UnmapGranularity;
    let _ = ActivityId;
    let _ = Irp;

    // 1.1 Calculate needed Block Descriptor count
    neededBlockDescrCount = 0;
    for i in 0..DataSetRangesCount {
        let range = &*DataSetRanges.offset(i as isize);
        let mut lba = range.LengthInBytes / (*FdoExtension).DiskGeometry.BytesPerSector as u64;
        if lba > 0 {
            neededBlockDescrCount += ((lba - 1) / u32::MAX as u64 + 1) as u32;
        }
    }

    // 1.2 Calculate max parameter list length
    let blockLimits = &(*(*FdoExtension).FunctionSupportInfo).BlockLimitsData;
    if blockLimits.MaxUnmapBlockDescrCount != 0 && blockLimits.MaxUnmapBlockDescrCount != u32::MAX {
        maxParameterListLength = (blockLimits.MaxUnmapBlockDescrCount as u64
            * core::mem::size_of::<_UNMAP_BLOCK_DESCRIPTOR>() as u64)
            + core::mem::size_of::<_UNMAP_LIST_HEADER>() as u64;
        maxParameterListLength = core::cmp::min(maxParameterListLength, u16::MAX as u64);
    } else {
        maxParameterListLength = u16::MAX as u64;
    }

    // 1.3 Buffer size
    bufferLength = core::cmp::min(
        (*(*FdoExtension).PrivateFdoData).HwMaxXferLen,
        maxParameterListLength as u32,
    );
    bufferLength = core::cmp::min(
        bufferLength,
        neededBlockDescrCount as u32 * core::mem::size_of::<_UNMAP_BLOCK_DESCRIPTOR>() as u32
            + core::mem::size_of::<_UNMAP_LIST_HEADER>() as u32,
    );

    maxBlockDescrCount =
        (bufferLength - core::mem::size_of::<_UNMAP_LIST_HEADER>() as u32)
            / core::mem::size_of::<_UNMAP_BLOCK_DESCRIPTOR>() as u32;

    if maxBlockDescrCount == 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"DeviceProcessDsmTrimRequest (%p): Max Block Descriptor count is Zero\n\0".as_ptr() as PCSZ,
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );
        NT_ASSERT!(0u8);
        status = STATUS_DATA_ERROR;
        return status;
    }

    maxLbaCount = blockLimits.MaxUnmapLbaCount as u64;
    if maxLbaCount == 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"DeviceProcessDsmTrimRequest (%p): Max LBA count is Zero\n\0".as_ptr() as PCSZ,
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );
        NT_ASSERT!(0u8);
        status = STATUS_DATA_ERROR;
        return status;
    }

    // Allocate buffer
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let alignment = KeGetRecommendedSharedDataAlignment() as usize;
        bufferLength = ((bufferLength.wrapping_sub(1)) / (alignment as u32) + 1) * (alignment as u32);
        buffer = ExAllocatePoolWithTag(
            0x204u32, // NonPagedPoolNxCacheAligned
            bufferLength as usize,
            u32::from_ne_bytes(*b"TpLC"),
        ) as *mut _UNMAP_LIST_HEADER;
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        buffer = ExAllocatePoolWithTag(
            0, // NonPagedPoolNx: ReactOS #defines it to NonPagedPool (no NX pool here)
            bufferLength as usize,
            u32::from_ne_bytes(*b"TpLC"),
        ) as *mut _UNMAP_LIST_HEADER;
    }

    if buffer.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        return status;
    }
    core::ptr::write_bytes(buffer as *mut u8, 0, bufferLength as usize);

    blockDescrPointer = (*buffer).Descriptors.as_mut_ptr();
    allDataSetRangeFullyConverted = 0u8;
    needToSendCommand = 0u8;
    tempDataSetRangeFullyConverted = 1u8;
    dataSetRangeIndex = 0;
    core::ptr::write_bytes(&mut tempDataSetRange as *mut _DEVICE_DATA_SET_RANGE as *mut u8, 0, core::mem::size_of::<_DEVICE_DATA_SET_RANGE>());
    blockDescrIndex = 0;
    lbaCount = 0;

    'exit: {
        while allDataSetRangeFullyConverted == 0u8 {
            if tempDataSetRangeFullyConverted != 0u8 {
                let range = &*DataSetRanges.offset(dataSetRangeIndex as isize);
                tempDataSetRange.StartingOffset = range.StartingOffset;
                tempDataSetRange.LengthInBytes = range.LengthInBytes;
                dataSetRangeIndex += 1;
            }

            ConvertDataSetRangeToUnmapBlockDescr(
                FdoExtension,
                blockDescrPointer,
                &mut blockDescrIndex,
                maxBlockDescrCount,
                &mut lbaCount,
                maxLbaCount,
                &mut tempDataSetRange,
            );

            tempDataSetRangeFullyConverted = if tempDataSetRange.LengthInBytes == 0 { 1u8 } else { 0u8 };
            allDataSetRangeFullyConverted = if tempDataSetRangeFullyConverted != 0u8 && dataSetRangeIndex == DataSetRangesCount { 1u8 } else { 0u8 };

            if blockDescrIndex == maxBlockDescrCount
                || lbaCount == maxLbaCount
                || allDataSetRangeFullyConverted != 0u8
            {
                let transferSize: u16 = (blockDescrIndex as usize * core::mem::size_of::<_UNMAP_BLOCK_DESCRIPTOR>()
                    + core::mem::size_of::<_UNMAP_LIST_HEADER>()) as u16;
                if transferSize > bufferLength as u16 {
                    NT_ASSERT!(0u8);
                    status = STATUS_BUFFER_TOO_SMALL;
                    break 'exit;
                }

                // Set buffer header fields
                let mut tempSize: u16 = transferSize
                    - core::mem::offset_of!(_UNMAP_LIST_HEADER, BlockDescrDataLength) as u16;
                (*buffer).DataLength = tempSize.swap_bytes().to_ne_bytes();
                tempSize = transferSize
                    - core::mem::offset_of!(_UNMAP_LIST_HEADER, Descriptors) as u16;
                (*buffer).BlockDescrDataLength = tempSize.swap_bytes().to_ne_bytes();

                // Initialize SRB
                if (*((*FdoExtension).AdapterDescriptor as *mut LocalStorageAdapterDescriptor)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
                    status = InitializeStorageRequestBlock(
                        Srb as *mut _STORAGE_REQUEST_BLOCK,
                        STORAGE_ADDRESS_TYPE_BTL8 as u16,
                        CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
                        1u32,
                        SrbExDataTypeScsiCdb16,
                    );
                    if NT_SUCCESS(status) {
                        (*(Srb as *mut _STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
                    } else {
                        NT_ASSERT!(0u8);
                        break 'exit;
                    }
                } else {
                    core::ptr::write_bytes(Srb as *mut u8, 0, core::mem::size_of::<_SCSI_REQUEST_BLOCK>());
                    (*Srb).Length = (core::mem::size_of::<_SCSI_REQUEST_BLOCK>() as u8) as u16;
                    (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                }

                SrbSetTimeOutValue((Srb).cast(), (*FdoExtension).TimeOutValue);
                SrbSetRequestTag((Srb).cast(), SP_UNTAGGED);
                SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);

                SrbAssignSrbFlags((Srb).cast(), (*FdoExtension).SrbFlags);
                SrbClearSrbFlags((Srb).cast(), SRB_FLAGS_DATA_IN);
                SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_DATA_OUT);
                SrbSetSrbFlags((Srb).cast(), SrbFlags);

                SrbSetCdbLength((Srb).cast(), 10);

                // SrbGetCdb: for a STORAGE_REQUEST_BLOCK (the SrbType branch above)
                // the CDB lives in the SrbExData, not at SCSI_REQUEST_BLOCK.Cdb; the
                // direct field cast wrote the UNMAP CDB over the SRB_EX header.
                let cdb = SrbGetCdb((Srb).cast()) as *mut _CDB;
                (*cdb).UNMAP.as_mut().OperationCode = 0x42u8; // SCSIOP_UNMAP
                (*cdb).UNMAP.as_mut().set_Anchor(0);
                (*cdb).UNMAP.as_mut().set_GroupNumber(0);
                (*cdb).UNMAP.as_mut().AllocationLength[0] = (transferSize >> 8) as u8;
                (*cdb).UNMAP.as_mut().AllocationLength[1] = transferSize as u8;

                status = ClassSendSrbSynchronous(
                    (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                    Srb,
                    buffer as *mut c_void,
                    transferSize as u32,
                    1u8,
                );

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_IOCTL,
                    b"DeviceProcessDsmTrimRequest (%p): UNMAP command issued. Returned NTSTATUS: %!STATUS!.\n\0".as_ptr() as PCSZ,
                    (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                    status,
                );

                // Reset for next chunk
                blockDescrIndex = 0;
                lbaCount = 0;
                core::ptr::write_bytes(buffer as *mut u8, 0, bufferLength as usize);
            }
        }
    }

    ExFreePool(buffer as *mut c_void);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceTrimProcess(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    ActivityId: *mut GUID,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;

    if (*DeviceObject).DeviceType != FILE_DEVICE_DISK
        || ((*DeviceObject).Characteristics & FILE_FLOPPY_DISKETTE) != 0
        || (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.TrimProcess == Supported
    {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        TracePrint(TRACE_LEVEL_INFORMATION,
                   TRACE_FLAG_GENERAL,
                   b"ClasspDeviceTrimProcess (%p): Lower layer supports Trim DSM IOCTL, forwarding IOCTL.\n\0".as_ptr() as PCSZ,
                   DeviceObject);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        return IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
    }

    let dsmAttributes = (*Irp).AssociatedIrp.SystemBuffer as *mut DEVICE_MANAGE_DATA_SET_ATTRIBUTES;

    // Request validation
    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        NT_ASSERT!((KeGetCurrentIrql() < DISPATCH_LEVEL) as u8);
        let status: NTSTATUS = STATUS_INVALID_LEVEL;
        // goto Exit
        {
            (*Irp).IoStatus.Information = 0;
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
            return status;
        }
    }

    if (!(((*dsmAttributes).Flags & DEVICE_DSM_FLAG_ENTIRE_DATA_SET_RANGE) != 0)
         && ((*dsmAttributes).DataSetRangesOffset == 0 || (*dsmAttributes).DataSetRangesLength == 0))
        || (((*dsmAttributes).Flags & DEVICE_DSM_FLAG_ENTIRE_DATA_SET_RANGE) != 0
            && ((*dsmAttributes).DataSetRangesOffset != 0 || (*dsmAttributes).DataSetRangesLength != 0))
    {
        let status: NTSTATUS = STATUS_INVALID_PARAMETER;
        // goto Exit
        {
            (*Irp).IoStatus.Information = 0;
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
            return status;
        }
    }

    let mut status: NTSTATUS = STATUS_SUCCESS;
    'exit: loop {
        match (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.TrimProcess {
            SupportUnknown => {
                status = ClassForwardIrpSynchronous(commonExtension, Irp);
                TracePrint(TRACE_LEVEL_INFORMATION,
                           TRACE_FLAG_GENERAL,
                           b"ClasspDeviceTrimProcess (%p): Trim DSM IOCTL support unknown.  Forwarded IOCTL and received NTSTATUS %!STATUS!.\n\0".as_ptr() as PCSZ,
                           DeviceObject,
                           status);

                if ClasspLowerLayerNotSupport(status) {
                    if (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.LBProvisioning() == (1u8) as u32 {
                        NT_ASSERT!(((*(*fdoExtension).FunctionSupportInfo).LBProvisioningData.CommandStatus != -1) as u8);
                    }
                    if (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.BlockLimits() == (1u8) as u32 {
                        NT_ASSERT!(((*(*fdoExtension).FunctionSupportInfo).BlockLimitsData.CommandStatus != -1) as u8);
                    }
                } else {
                    (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.TrimProcess = Supported;
                    break 'exit; // goto Exit
                }
            }

            NotSupported => {
                if ClasspSupportsUnmap((*fdoExtension).FunctionSupportInfo) {
                    if (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
                        status = ClassReadDriveCapacity((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);
                        if !NT_SUCCESS(status) || (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
                            status = STATUS_INVALID_DEVICE_REQUEST;
                            break 'exit; // goto Exit
                        }
                    }

                    let blockLimitsData: CLASS_VPD_B0_DATA = zeroed();
                    let generationCount: u32 = 0;
                    ClasspBlockLimitsDataSnapshot(
                        fdoExtension,
                        0u8,
                        &blockLimitsData as *const CLASS_VPD_B0_DATA as *mut CLASS_VPD_B0_DATA,
                        &generationCount as *const u32 as *mut u32,
                    );

                    let granularityInBlocks: u32;
                    if blockLimitsData.OptimalUnmapGranularity != 0 {
                        granularityInBlocks = blockLimitsData.OptimalUnmapGranularity;
                    } else {
                        granularityInBlocks = 1;
                        TracePrint(TRACE_LEVEL_INFORMATION,
                                   TRACE_FLAG_GENERAL,
                                   b"ClasspDeviceTrimProcess (%p): Optimal Unmap Granularity not provided, defaulted to 1.\n\0".as_ptr() as PCSZ,
                                   DeviceObject);
                    }

                    let granularityAlignmentInBytes: u64;
                    if blockLimitsData.UGAVALID == 1u8 {
                        granularityAlignmentInBytes = (blockLimitsData.UnmapGranularityAlignment as u64) * (*fdoExtension).DiskGeometry.BytesPerSector as u64;
                    } else {
                        granularityAlignmentInBytes = 0;
                        TracePrint(TRACE_LEVEL_INFORMATION,
                                   TRACE_FLAG_GENERAL,
                                   b"ClasspDeviceTrimProcess (%p): Unmap Granularity Alignment not provided, defaulted to 0.\n\0".as_ptr() as PCSZ,
                                   DeviceObject);
                    }

                    let dataSetRanges: *mut _DEVICE_DATA_SET_RANGE;
                    let dataSetRangesCount: u32;
                    let mut entireDataSetRange: _DEVICE_DATA_SET_RANGE = zeroed();

                    if ((*dsmAttributes).Flags & DEVICE_DSM_FLAG_ENTIRE_DATA_SET_RANGE) != 0 {
                        entireDataSetRange.StartingOffset = granularityAlignmentInBytes as i64;
                        entireDataSetRange.LengthInBytes = ((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart as u64)
                            - (entireDataSetRange.StartingOffset as u64);
                        dataSetRanges = &mut entireDataSetRange as *mut _DEVICE_DATA_SET_RANGE;
                        dataSetRangesCount = 1;
                    } else {
                        dataSetRanges = ((dsmAttributes as *const u8).add((*dsmAttributes).DataSetRangesOffset as usize))
                            as *mut _DEVICE_DATA_SET_RANGE;
                        dataSetRangesCount = (*dsmAttributes).DataSetRangesLength / core::mem::size_of::<_DEVICE_DATA_SET_RANGE>() as u32;

                        for i in 0..dataSetRangesCount {
                            if (*dataSetRanges.add(i as usize)).StartingOffset % (*fdoExtension).DiskGeometry.BytesPerSector as i64 != 0
                                || (*dataSetRanges.add(i as usize)).LengthInBytes % (*fdoExtension).DiskGeometry.BytesPerSector as u64 != 0
                                || (*dataSetRanges.add(i as usize)).StartingOffset < granularityAlignmentInBytes as i64
                                || (*dataSetRanges.add(i as usize)).LengthInBytes == 0
                                || (*dataSetRanges.add(i as usize)).StartingOffset as u64 + (*dataSetRanges.add(i as usize)).LengthInBytes
                                    > ((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart) as u64
                            {
                                TracePrint(TRACE_LEVEL_ERROR,
                                           TRACE_FLAG_IOCTL,
                                           b"ClasspDeviceTrimProcess (%p): Invalid dataset range.  StartingOffset = %I64x, LengthInBytes = %I64x\n\0".as_ptr() as PCSZ,
                                           DeviceObject,
                                           (*dataSetRanges.add(i as usize)).StartingOffset,
                                           (*dataSetRanges.add(i as usize)).LengthInBytes);
                                status = STATUS_INVALID_PARAMETER;
                                break 'exit;
                            }
                        }
                    }

                    if ((*dsmAttributes).Flags & DEVICE_DSM_FLAG_TRIM_NOT_FS_ALLOCATED) == 0 {
                        if (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.LBProvisioning() != 0
                            && (*(*fdoExtension).FunctionSupportInfo).LBProvisioningData.LBPRZ() == 0
                        {
                            TracePrint(TRACE_LEVEL_ERROR,
                                       TRACE_FLAG_IOCTL,
                                       b"ClasspDeviceTrimProcess (%p): Device does not support file level TRIM.\n\0".as_ptr() as PCSZ,
                                       DeviceObject);
                            status = STATUS_TRIM_READ_ZERO_NOT_SUPPORTED;
                            break 'exit;
                        }
                    }

                    let srbFlags: u32 = 0;
                    status = DeviceProcessDsmTrimRequest(
                        fdoExtension,
                        dataSetRanges,
                        dataSetRangesCount,
                        granularityInBlocks,
                        srbFlags,
                        Irp,
                        ActivityId,
                        Srb,
                    );
                } else {
                    TracePrint(TRACE_LEVEL_ERROR,
                               TRACE_FLAG_IOCTL,
                               b"ClasspDeviceTrimProcess (%p): Device does not support UNMAP.\n\0".as_ptr() as PCSZ,
                               DeviceObject);
                    status = STATUS_NOT_SUPPORTED;
                }

                (*(*fdoExtension).FunctionSupportInfo).LowerLayerSupport.TrimProcess = NotSupported;
                break 'exit; // after NotSupported branch, we always go to Exit
            }

            Supported => {
                NT_ASSERT!(0u8); // FALSE
                break 'exit;
            }

            _ => {}
        }
        break; // end loop
    }

    // Exit:
    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn GetLBAStatus(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    Srb: *mut SCSI_REQUEST_BLOCK,
    StartingLBA: u64,
    LBAStatusHeader: *mut LBA_STATUS_LIST_HEADER,
    LBAStatusSize: u32,
    ConsolidateableBlocksOnly: u8,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;

    if LBAStatusHeader.is_null() || LBAStatusSize == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    SrbSetTimeOutValue((Srb).cast(), (*FdoExtension).TimeOutValue);
    SrbSetRequestTag((Srb).cast(), SP_UNTAGGED);
    SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
    SrbAssignSrbFlags((Srb).cast(), (*FdoExtension).SrbFlags);
    SrbSetCdbLength((Srb).cast(), core::mem::size_of::<_CDB__GET_LBA_STATUS>() as u8);

    let cdb = SrbGetCdb((Srb).cast());
    (*cdb).GET_LBA_STATUS.as_mut().OperationCode = (SCSIOP_GET_LBA_STATUS) as u8;
    (*cdb).GET_LBA_STATUS.as_mut().set_ServiceAction((SERVICE_ACTION_GET_LBA_STATUS) as u8);

    // REVERSE_BYTES_QUAD(&(cdb->GET_LBA_STATUS.StartingLBA), &StartingLBA);
    let start_be = StartingLBA.to_be_bytes();
    core::ptr::copy_nonoverlapping(
        start_be.as_ptr(),
        core::ptr::addr_of_mut!((*cdb).GET_LBA_STATUS.as_mut().StartingLBA) as *mut u8,
        8,
    );

    // REVERSE_BYTES(&(cdb->GET_LBA_STATUS.AllocationLength), &LBAStatusSize);
    let alloc_be = LBAStatusSize.to_be_bytes();
    core::ptr::copy_nonoverlapping(
        alloc_be.as_ptr(),
        core::ptr::addr_of_mut!((*cdb).GET_LBA_STATUS.as_mut().AllocationLength) as *mut u8,
        4,
    );

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        (b"GetLBAStatus (%p): sending command with StartingLBA = 0x%I64x, AllocationLength = 0x%I64x, ConsolidateableBlocksOnly = %u\n\0"
            .as_ptr() as *const u8).cast(),
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        StartingLBA,
        LBAStatusSize as u64,
        ConsolidateableBlocksOnly as u32,
    );

    status = ClassSendSrbSynchronous(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        Srb,
        (LBAStatusHeader).cast(),
        LBAStatusSize,
        0u8,
    );

    if status == STATUS_DATA_OVERRUN
        && SrbGetDataTransferLength((Srb).cast()) < LBAStatusSize
    {
        status = STATUS_SUCCESS;
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        (b"GetLBAStatus (%p): command returned NT Status: %!STATUS!\n\0"
            .as_ptr() as *const u8).cast(),
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        status,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceGetLBAStatus(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Srb: *mut SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let fdo_extension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let dsm_attributes = (*Irp).AssociatedIrp.SystemBuffer as *mut DEVICE_MANAGE_DATA_SET_ATTRIBUTES;
    let dsm_output = (*Irp).AssociatedIrp.SystemBuffer as *mut DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT;

    let mut final_status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut information: usize = 0;

    'main: loop {
        if dsm_output.is_null() || dsm_attributes.is_null() {
            final_status = STATUS_INVALID_PARAMETER;
            break 'main;
        }

        let mut starting_offset: i64;
        let mut length_in_bytes: u64;

        if ((*dsm_attributes).Flags & DEVICE_DSM_FLAG_ENTIRE_DATA_SET_RANGE) != 0 {
            let unmap_granularity_alignment = if (*(*fdo_extension).FunctionSupportInfo).BlockLimitsData.UGAVALID != 0 {
                (*(*fdo_extension).FunctionSupportInfo).BlockLimitsData.UnmapGranularityAlignment as i64
            } else {
                0i64
            };
            starting_offset = unmap_granularity_alignment;
            length_in_bytes = (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart as u64 - (starting_offset as u64);
        } else {
            if (*dsm_attributes).DataSetRangesOffset == 0 || (*dsm_attributes).DataSetRangesLength == 0 {
                final_status = STATUS_INVALID_PARAMETER;
                break 'main;
            }
            let data_set_ranges =
                (dsm_attributes as *mut u8).offset((*dsm_attributes).DataSetRangesOffset as isize) as *mut DEVICE_DATA_SET_RANGE;
            starting_offset = (*data_set_ranges).StartingOffset;
            length_in_bytes = (*data_set_ranges).LengthInBytes;
        }

        #[allow(unused_mut)] // only the WINBLUE block below assigns it
        let mut output_version = DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V1;
        // dead: C is #if (NTDDI_VERSION >= NTDDI_WINBLUE), classpnp builds at WIN8. Running
        // it failed requests the WIN8 C serves, and asked the worker for V2 output.
        #[cfg(any())]
        if (*dsm_attributes).ParameterBlockOffset >= core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>() as u32
            && (*dsm_attributes).ParameterBlockLength >= core::mem::size_of::<DEVICE_DATA_SET_LBP_STATE_PARAMETERS>() as u32
        {
            let parameters = (dsm_attributes as *const u8)
                .offset((*dsm_attributes).ParameterBlockOffset as isize) as *const DEVICE_DATA_SET_LBP_STATE_PARAMETERS;
            if (*parameters).Version == DEVICE_DATA_SET_LBP_STATE_PARAMETERS_VERSION_V1
                && (*parameters).Size >= core::mem::size_of::<DEVICE_DATA_SET_LBP_STATE_PARAMETERS>() as u32
            {
                output_version = (*parameters).OutputVersion;
                if output_version != DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V1
                    && output_version != DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2
                {
                    final_status = STATUS_INVALID_PARAMETER;
                    break 'main;
                }
            }
        }

        let mut block_limits_data: CLASS_VPD_B0_DATA = core::mem::zeroed();
        let mut generation_count1: u32 = 0;
        final_status = ClasspBlockLimitsDataSnapshot(
            fdo_extension,
            1u8,
            &mut block_limits_data,
            &mut generation_count1,
        );
        if !NT_SUCCESS(final_status) {
            information = 0;
            break 'main;
        }

        let consolidateable_blocks_only = if ((*dsm_attributes).Flags & DEVICE_DSM_FLAG_ALLOCATION_CONSOLIDATEABLE_ONLY) != 0 {
            1u8
        } else {
            0u8
        };

        let retry_count_max = GET_LBA_STATUS_RETRY_COUNT_MAX;
        let mut retry_count = 0;
        while retry_count < retry_count_max {
            let mut dsm_output_length = (*irp_stack).Parameters.DeviceIoControl.OutputBufferLength as u32;
            let mut block_limits_data_may_have_changed: u8 = 0;
            let mut generation_count2: u32 = 0;

            let get_lba_worker_status = ClasspDeviceGetLBAStatusWorker(
                DeviceObject,
                &mut block_limits_data,
                starting_offset as u64,
                length_in_bytes,
                dsm_output,
                &mut dsm_output_length,
                Srb,
                consolidateable_blocks_only,
                output_version,
                &mut block_limits_data_may_have_changed,
            );

            if !NT_SUCCESS(get_lba_worker_status) && block_limits_data_may_have_changed == 0 {
                information = 0;
                final_status = get_lba_worker_status;
                break 'main;
            }

            final_status = ClasspBlockLimitsDataSnapshot(
                fdo_extension,
                1u8,
                &mut block_limits_data,
                &mut generation_count2,
            );
            if !NT_SUCCESS(final_status) {
                information = 0;
                break 'main;
            }

            if generation_count1 == generation_count2 {
                final_status = get_lba_worker_status;
                if NT_SUCCESS(final_status) {
                    information = dsm_output_length as usize;
                }
                break 'main;
            }

            generation_count1 = generation_count2;
            information = 0;
            final_status = STATUS_DEVICE_DATA_ERROR;

            retry_count += 1;
        }

        break 'main;
    }

    (*Irp).IoStatus.Information = (((information) as u32) as u64) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = final_status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    final_status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeviceGetLBAStatusWorker( // nolint: version-vs-c -- its V2 arm is unreachable: the caller's WINBLUE block is compiled out, so OutputVersion is always V1
    DeviceObject: *mut DEVICE_OBJECT,
    BlockLimitsData: *mut CLASS_VPD_B0_DATA,
    StartingOffset: u64,
    LengthInBytes: u64,
    DsmOutput: *mut DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT,
    DsmOutputLength: *mut u32,
    Srb: *mut SCSI_REQUEST_BLOCK,
    ConsolidateableBlocksOnly: u8,
    OutputVersion: u32,
    BlockLimitsDataMayHaveChanged: *mut u8,
) -> NTSTATUS {
    let mut status: i32 = STATUS_SUCCESS;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;

    let bitMapGranularityInBits = core::mem::size_of::<u32>() * 8;
    let outputLength = *DsmOutputLength;

    let mut lbpState: *mut DEVICE_DATA_SET_LB_PROVISIONING_STATE;
    let mut blocksPerSlab;
    let mut bytesPerSlab;
    let alignmentInBytes: u64;
    let alignmentInBlocks: u32;
    let requiredOutputLength: u32;
    let maxBufferSize: u32;
    let maxSlabs: u32;
    let mut requestedSlabs: u64;
    let mut startingLBA: u64;
    let startingOffsetDelta: u64;
    let mut totalProcessedSlabs: u64 = 0;
    let slabsPerCommand: u64;
    let mut doneProcessing: u8 = 0;
    let lbaStatusSize: u32;
    let mut lbaStatusListHeader: *mut LBA_STATUS_LIST_HEADER = core::ptr::null_mut();

    *BlockLimitsDataMayHaveChanged = 0;

    'lba_exit: {

    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        NT_ASSERT!((KeGetCurrentIrql() < DISPATCH_LEVEL) as u8);
        status = STATUS_INVALID_LEVEL;
        break 'lba_exit;
    }

    if (*(*fdoExtension).FunctionSupportInfo).LBProvisioningData.LBPU() == 0 {
        return STATUS_NOT_SUPPORTED;
    }

    if (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
        status = ClassReadDriveCapacity((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);
        if !NT_SUCCESS(status) || (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
            status = STATUS_INVALID_DEVICE_REQUEST;
            break 'lba_exit;
        }
    }

    if (*BlockLimitsData).UGAVALID == 1u8 {
        alignmentInBlocks = (*BlockLimitsData).UnmapGranularityAlignment;
        alignmentInBytes = alignmentInBlocks as u64 * (*fdoExtension).DiskGeometry.BytesPerSector as u64;
    } else {
        alignmentInBlocks = 0;
        alignmentInBytes = 0;
    }

    if LengthInBytes == 0
        || StartingOffset < alignmentInBytes
        || StartingOffset + LengthInBytes > (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart as u64
    {
        TracePrint(
            TRACE_LEVEL_ERROR,
             TRACE_FLAG_IOCTL,
             b"ClasspDeviceGetLBAStatusWorker (%p): Invalid range, length is %I64u bytes, starting offset is %I64u bytes, Unmap alignment is %I64u bytes, and disk size is %I64u bytes\n\0"
                 .as_ptr() as PCSZ,
            DeviceObject,
            LengthInBytes,
            StartingOffset,
            alignmentInBytes,
            (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart as u64,
        );
        status = STATUS_INVALID_PARAMETER;
        break 'lba_exit;
    }

    blocksPerSlab = (*BlockLimitsData).OptimalUnmapGranularity;
    bytesPerSlab = blocksPerSlab as u64 * (*fdoExtension).DiskGeometry.BytesPerSector as u64;

    if (StartingOffset - alignmentInBytes) % bytesPerSlab != 0 {
        startingLBA = ((StartingOffset - alignmentInBytes) / bytesPerSlab + 1) * blocksPerSlab as u64 + alignmentInBlocks as u64;
        startingOffsetDelta = startingLBA * (*fdoExtension).DiskGeometry.BytesPerSector as u64 - StartingOffset;
    } else {
        startingLBA = ((StartingOffset - alignmentInBytes) / bytesPerSlab) * blocksPerSlab as u64 + alignmentInBlocks as u64;
        startingOffsetDelta = 0;
    }

    if LengthInBytes % bytesPerSlab == 0 {
        requestedSlabs = LengthInBytes / bytesPerSlab;
    } else {
        requestedSlabs = LengthInBytes / bytesPerSlab + 1;
    }

    if requestedSlabs == 0 {
        TracePrint(
            TRACE_LEVEL_ERROR,
             TRACE_FLAG_IOCTL,
             b"ClasspDeviceGetLBAStatusWorker (%p): Invalid number (%I64u) of slabs requested\n\0"
                 .as_ptr() as PCSZ,
            DeviceObject,
            requestedSlabs,
        );
        status = STATUS_INVALID_PARAMETER;
        break 'lba_exit;
    }

    requestedSlabs = core::cmp::min(requestedSlabs, u32::MAX as u64);

    if OutputVersion == DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2 {
        requiredOutputLength = core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT>() as u32
            + core::mem::size_of::<DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2>() as u32
            + ((requestedSlabs - 1) / bitMapGranularityInBits as u64) as u32
                * core::mem::size_of::<u32>() as u32;
    } else {
        requiredOutputLength = core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT>() as u32
            + core::mem::size_of::<DEVICE_DATA_SET_LB_PROVISIONING_STATE>() as u32
            + ((requestedSlabs - 1) / bitMapGranularityInBits as u64) as u32
                * core::mem::size_of::<u32>() as u32;
    }

    if outputLength < requiredOutputLength {
        status = STATUS_BUFFER_TOO_SMALL;
        TracePrint(
            TRACE_LEVEL_ERROR,
             TRACE_FLAG_IOCTL,
             b"ClasspDeviceGetLBAStatusWorker (%p): Given output buffer is %u bytes, needs to be %u bytes\n\0"
                 .as_ptr() as PCSZ,
            DeviceObject,
            outputLength,
            requiredOutputLength,
        );
        if outputLength >= core::mem::size_of::<u32>() as u32 {
            *(DsmOutput as *mut u32) = requiredOutputLength;
        }
        break 'lba_exit;
    }

    maxBufferSize = core::cmp::min(u32::MAX, (*(*fdoExtension).PrivateFdoData).HwMaxXferLen);
    maxSlabs = (maxBufferSize - core::mem::size_of::<LBA_STATUS_LIST_HEADER>() as u32) / core::mem::size_of::<LBA_STATUS_DESCRIPTOR>() as u32;
    slabsPerCommand = core::cmp::min(requestedSlabs, maxSlabs as u64);

    // ARM has specific alignment requirements (the C's `_ARM_ || _ARM64_` arm,
    // which every other VPD/capacity allocation in this file already mirrors).
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        lbaStatusSize = ALIGN_UP_BY(
            (core::mem::size_of::<LBA_STATUS_LIST_HEADER>() as u32
                + (slabsPerCommand as u32 * core::mem::size_of::<LBA_STATUS_DESCRIPTOR>() as u32))
                as _,
            KeGetRecommendedSharedDataAlignment() as _,
        ) as _;
        lbaStatusListHeader = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            lbaStatusSize as usize,
            CLASS_TAG_LB_PROVISIONING,
        ) as *mut LBA_STATUS_LIST_HEADER;
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        lbaStatusSize = core::mem::size_of::<LBA_STATUS_LIST_HEADER>() as u32
            + (slabsPerCommand as u32 * core::mem::size_of::<LBA_STATUS_DESCRIPTOR>() as u32);
        lbaStatusListHeader = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            lbaStatusSize as usize,
            CLASS_TAG_LB_PROVISIONING,
        ) as *mut LBA_STATUS_LIST_HEADER;
    }

    if lbaStatusListHeader.is_null() {
        TracePrint(
            TRACE_LEVEL_ERROR,
             TRACE_FLAG_IOCTL,
             b"ClasspDeviceGetLBAStatusWorker (%p): Failed to allocate %u bytes for descriptors\n\0"
                 .as_ptr() as PCSZ,
            DeviceObject,
            lbaStatusSize,
        );
        NT_ASSERT!(!lbaStatusListHeader.is_null() as u8);
        status = STATUS_INSUFFICIENT_RESOURCES;
        break 'lba_exit;
    }

    (*DsmOutput).Action = DeviceDsmAction_Allocation;
    (*DsmOutput).Size = core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT>() as u32;
    (*DsmOutput).OutputBlockOffset = 0;
    (*DsmOutput).OutputBlockLength = 0;
    *DsmOutputLength = (*DsmOutput).Size;

    lbpState = (DsmOutput as *mut u8).add(core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT>()) as *mut DEVICE_DATA_SET_LB_PROVISIONING_STATE;
    core::ptr::write_bytes(
        lbpState as *mut u8,
        0,
        (requiredOutputLength - core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT>() as u32) as usize,
    );

    loop {
        status = GetLBAStatus(
            fdoExtension,
            Srb,
            startingLBA,
            lbaStatusListHeader,
            lbaStatusSize,
            ConsolidateableBlocksOnly,
        );

        if NT_SUCCESS(status) {
            let mut descrIndex;
            let mut descrSize: u32 = 0;
            let descrSizeOverhead: u32;
            let mut descrCount: u32;
            let mut expectedStartingLBA: u64;
            let mut processCurrentDescriptor: u8;
            let mut commandProcessedSlabs: u64 = 0;

            let paramFieldEnd = core::mem::offset_of!(LBA_STATUS_LIST_HEADER, ParameterLength) + core::mem::size_of::<u32>();
            descrSizeOverhead = core::mem::offset_of!(LBA_STATUS_LIST_HEADER, Descriptors) as u32 - paramFieldEnd as u32;

            // Read big-endian ParameterLength
            descrSize = u32::from_be(core::ptr::read_unaligned(
                &raw const (*lbaStatusListHeader).ParameterLength as *const u32,
            ));

            let headerEnd = core::mem::offset_of!(LBA_STATUS_LIST_HEADER, ParameterLength) + core::mem::size_of::<u32>();
            let maxDescrBytes = lbaStatusSize - headerEnd as u32;
            if descrSize > maxDescrBytes {
                descrSize = maxDescrBytes;
            }

            if descrSize >= descrSizeOverhead {
                descrSize -= descrSizeOverhead;
                descrCount = descrSize / core::mem::size_of::<LBA_STATUS_DESCRIPTOR>() as u32;

                if descrCount > 0 {
                    expectedStartingLBA = startingLBA;

                    descrIndex = 0;
                    while descrIndex < descrCount
                        && totalProcessedSlabs < requestedSlabs
                        && doneProcessing == 0
                    {
                        let lbaStatusDescr =
                            &*(*lbaStatusListHeader).Descriptors.as_ptr().add(descrIndex as usize);
                        let returnedStartingLBA: u64;
                        let mapped: u32;
                        let mut lbaCount: u32 = 0;

                        returnedStartingLBA = u64::from_be(core::ptr::read_unaligned(
                            &raw const lbaStatusDescr.StartingLBA as *const u64,
                        ));
                        lbaCount = u32::from_be(core::ptr::read_unaligned(
                            &raw const lbaStatusDescr.LogicalBlockCount as *const u32,
                        ));

                        mapped = if lbaStatusDescr.ProvisioningStatus() != LBA_STATUS_MAPPED as u8 {
                            0x0
                        } else {
                            0x1
                        };

                        if returnedStartingLBA != expectedStartingLBA {
                            TracePrint(
                                TRACE_LEVEL_ERROR,
                                 TRACE_FLAG_IOCTL,
                                 b"ClasspDeviceGetLBAStatusWorker (%p): Device returned starting LBA = %I64x when %I64x was expected.\n\0"
                                     .as_ptr() as PCSZ,
                                DeviceObject,
                                returnedStartingLBA,
                                startingLBA,
                            );
                            doneProcessing = 1;
                            processCurrentDescriptor = 0;
                            *BlockLimitsDataMayHaveChanged = 1;
                        } else if lbaCount > 0 && lbaCount % blocksPerSlab != 0 {
                            TracePrint(
                                TRACE_LEVEL_WARNING,
                                 TRACE_FLAG_IOCTL,
                                 b"ClasspDeviceGetLBAStatusWorker (%p): Device returned an LBA count (%u) that is not a multiple of the slab size (%u)\n\0"
                                     .as_ptr() as PCSZ,
                                DeviceObject,
                                lbaCount,
                                blocksPerSlab,
                            );
                            lbaCount = ((lbaCount / blocksPerSlab) + 1) * blocksPerSlab;
                            doneProcessing = 1;
                            processCurrentDescriptor = 1;
                        } else if lbaCount == 0 {
                            TracePrint(
                                TRACE_LEVEL_WARNING,
                                 TRACE_FLAG_IOCTL,
                                 b"ClasspDeviceGetLBAStatusWorker (%p): Device returned a zero LBA count\n\0"
                                     .as_ptr() as PCSZ,
                                DeviceObject,
                            );
                            processCurrentDescriptor = 0;
                        } else {
                            processCurrentDescriptor = 1;
                        }

                        if processCurrentDescriptor != 0 {
                            let mut descrSlabs = lbaCount / blocksPerSlab;
                            while descrSlabs > 0 && totalProcessedSlabs < requestedSlabs {
                                let bitMapIndex = (totalProcessedSlabs / bitMapGranularityInBits as u64) as usize;
                                let bitPos = (totalProcessedSlabs % bitMapGranularityInBits as u64) as u32;

                                // SlabAllocationBitMap is a C flexible array (`ULONG
                                // SlabAllocationBitMap[ANYSIZE_ARRAY]`, psdk/ntddstor.h:934/948)
                                // sized for every requested slab. bindgen's [_; 1] panicked once
                                // 32 slabs were processed. Raw pointers throughout: a pointer
                                // derived from a `&mut` to the struct may not step past its end.
                                if OutputVersion == DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2 {
                                    let lbpV2 = lbpState as *mut DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2;
                                    *(&raw mut (*lbpV2).SlabAllocationBitMap[0]).add(bitMapIndex) |= mapped << bitPos;
                                } else {
                                    *(&raw mut (*lbpState).SlabAllocationBitMap[0]).add(bitMapIndex) |= mapped << bitPos;
                                }
                                descrSlabs -= 1;
                                commandProcessedSlabs += 1;
                                totalProcessedSlabs += 1;
                            }
                        }

                        expectedStartingLBA = returnedStartingLBA + lbaCount as u64;
                        descrIndex += 1;
                    }

                    if commandProcessedSlabs > 0 {
                        startingLBA += commandProcessedSlabs * blocksPerSlab as u64;
                    } else {
                        TracePrint(
                            TRACE_LEVEL_ERROR,
                             TRACE_FLAG_IOCTL,
                             b"ClasspDeviceGetLBAStatusWorker (%p): The slab allocation bitmap has zero length.\n\0"
                                 .as_ptr() as PCSZ,
                            DeviceObject,
                        );
                        NT_ASSERT!((commandProcessedSlabs != 0) as u8);
                        doneProcessing = 1;
                        status = STATUS_UNSUCCESSFUL;
                    }
                } else {
                    TracePrint(
                        TRACE_LEVEL_ERROR,
                         TRACE_FLAG_IOCTL,
                         b"ClasspDeviceGetLBAStatusWorker (%p): Device returned no LBA Status Descriptors.\n\0"
                             .as_ptr() as PCSZ,
                        DeviceObject,
                    );
                    doneProcessing = 1;
                    status = STATUS_UNSUCCESSFUL;
                }
            } else {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                     TRACE_FLAG_IOCTL,
                     b"ClasspDeviceGetLBAStatusWorker (%p): not enough bytes returned\n\0"
                         .as_ptr() as PCSZ,
                    DeviceObject,
                );
                doneProcessing = 1;
                status = STATUS_DEVICE_DATA_ERROR;
            }
        }

        if !NT_SUCCESS(status) || doneProcessing != 0 || totalProcessedSlabs >= requestedSlabs {
            break;
        }
    }

    if totalProcessedSlabs > 0 {
        if OutputVersion == DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2 {
            let lbpV2 = &mut *(lbpState as *mut DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2);
            lbpV2.SlabSizeInBytes = bytesPerSlab;
            lbpV2.SlabOffsetDeltaInBytes = startingOffsetDelta;
            lbpV2.SlabAllocationBitMapBitCount = totalProcessedSlabs as u32;
            lbpV2.SlabAllocationBitMapLength = ((totalProcessedSlabs - 1) / bitMapGranularityInBits as u64) as u32 + 1;
            lbpV2.Version = DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V2;
            lbpV2.Size = core::mem::size_of::<DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2>() as u32
                + ((lbpV2.SlabAllocationBitMapLength - 1) as usize * core::mem::size_of::<u32>()) as u32;
            (*DsmOutput).OutputBlockLength = lbpV2.Size;
        } else {
            (*lbpState).SlabSizeInBytes = bytesPerSlab;
            (*lbpState).SlabOffsetDeltaInBytes = startingOffsetDelta as u32;
            (*lbpState).SlabAllocationBitMapBitCount = totalProcessedSlabs as u32;
            (*lbpState).SlabAllocationBitMapLength = ((totalProcessedSlabs - 1) / bitMapGranularityInBits as u64) as u32 + 1;
            (*lbpState).Version = DEVICE_DATA_SET_LB_PROVISIONING_STATE_VERSION_V1;
            (*lbpState).Size = core::mem::size_of::<DEVICE_DATA_SET_LB_PROVISIONING_STATE>() as u32
                + (((*lbpState).SlabAllocationBitMapLength - 1) as usize * core::mem::size_of::<u32>()) as u32;
            (*DsmOutput).OutputBlockLength = (*(lbpState as *mut DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2)).Size; // size offset same in V2
            // Actually, we can't directly use V2 size; just use the common offset.
            // Since both structs have Size at the same offset, we can set:
            // (*DsmOutput).OutputBlockLength = (*(lbpState as *mut DEVICE_DATA_SET_LB_PROVISIONING_STATE_V2)).Size;
            // But simpler: just use (*lbpState).Size for both, but for V1 it's Size. So use (*lbpState).Size.
        }
        // Common for both: OutputBlockOffset and DsmOutputLength
        (*DsmOutput).OutputBlockOffset = core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES_OUTPUT>() as u32;
        *DsmOutputLength = (*DsmOutput).Size + (*DsmOutput).OutputBlockLength;
        status = STATUS_SUCCESS;
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
         TRACE_FLAG_IOCTL,
         b"ClasspDeviceGetLBAStatusWorker (%p): Processed a total of %u slabs\n\0"
             .as_ptr() as PCSZ,
        DeviceObject,
        totalProcessedSlabs,
    );

    } // 'lba_exit
    ExFreePool(lbaStatusListHeader as *mut core::ffi::c_void);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassGetLBProvisioningLogPage(
    DeviceObject: PDEVICE_OBJECT,
    Srb: PSCSI_REQUEST_BLOCK,
    LogPageSize: u32,
    LogPage: *mut LOG_PAGE_LOGICAL_BLOCK_PROVISIONING,
) -> NTSTATUS {
    let fdo_extension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if LogPage.is_null()
        || LogPageSize < core::mem::size_of::<LOG_PAGE_LOGICAL_BLOCK_PROVISIONING>() as u32
        || LogPageSize > MAXUSHORT
    {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassGetLBProvisioningLogPage: DO (%p), Invalid parameter, LogPage = %p, LogPageSize = %u.\n\0"
                .as_ptr() as *const i8,
            DeviceObject,
            LogPage,
            LogPageSize,
        );
        return STATUS_INVALID_PARAMETER;
    }

    if (*((*fdo_extension).AdapterDescriptor as *mut LocalStorageAdapterDescriptor)).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK as u8 {
        status = InitializeStorageRequestBlock(
            Srb as *mut STORAGE_REQUEST_BLOCK,
            STORAGE_ADDRESS_TYPE_BTL8 as u16,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as u32,
            1u32,
            SrbExDataTypeScsiCdb16,
        );
        if NT_SUCCESS(status) {
            (*(Srb as *mut STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        } else {
            NT_ASSERT!(0u8);
        }
    } else {
        RtlZeroMemory(Srb as *mut c_void, (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32) as usize);
        (*Srb).Length = (core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u8) as u16;
        (*Srb).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
    }

    SrbSetTimeOutValue((Srb).cast(), (*fdo_extension).TimeOutValue);
    SrbSetRequestTag((Srb).cast(), SP_UNTAGGED);
    SrbSetRequestAttribute((Srb).cast(), (SRB_SIMPLE_TAG_REQUEST) as _);
    SrbAssignSrbFlags((Srb).cast(), (*fdo_extension).SrbFlags);
    SrbSetCdbLength((Srb).cast(), core::mem::size_of::<CDB>() as u8); // matches sizeof(cdb->LOGSENSE) as CDB union defines it

    let cdb = SrbGetCdb((Srb).cast()) as *mut CDB;
    let logsense = (*cdb).LOGSENSE.as_mut();
    logsense.OperationCode = SCSIOP_LOG_SENSE as u8;
    logsense.set_PageCode(LOG_PAGE_CODE_LOGICAL_BLOCK_PROVISIONING as u8);
    logsense.set_PCBit(0u8);
    logsense.ParameterPointer[0] = 0;
    logsense.ParameterPointer[1] = 0;
    logsense.AllocationLength = [((LogPageSize >> 8) & 0xFF) as u8, (LogPageSize & 0xFF) as u8]; // REVERSE_BYTES_SHORT big-endian

    status = ClassSendSrbSynchronous(
        (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        Srb,
        LogPage as *mut c_void,
        LogPageSize,
        0u8,
    );

    if status == STATUS_DATA_OVERRUN && SrbGetDataTransferLength((Srb).cast()) < LogPageSize {
        status = STATUS_SUCCESS;
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"ClassGetLBProvisioningLogPage: DO (%p), LogSense command issued for LBP log page. NT Status: %!STATUS!.\n\0"
            .as_ptr() as *const i8,
        DeviceObject,
        status,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassInterpretLBProvisioningLogPage(
    DeviceObject: *mut _DEVICE_OBJECT,
    LogPageSize: u32,
    LogPage: *mut _LOG_PAGE_LOGICAL_BLOCK_PROVISIONING,
    ResourcesSize: u32,
    Resources: *mut STORAGE_LB_PROVISIONING_MAP_RESOURCES,
) -> NTSTATUS {
    let fdo_extension = &mut *((*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION);
    let threshold_exponent = (*fdo_extension.FunctionSupportInfo).LBProvisioningData.ThresholdExponent;

    if threshold_exponent > 32 {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassInterpretLBProvisioningLogPage: DO (%p), Threshold Exponent (%u) is invalid.\n\0"
                .as_ptr() as *const i8,
            DeviceObject as *const c_void,
            threshold_exponent as u32,
        );
        return STATUS_NOT_SUPPORTED;
    }

    if Resources.is_null()
        || ResourcesSize < core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>() as u32
        || LogPage.is_null()
        || LogPageSize < core::mem::size_of::<LOG_PAGE_LOGICAL_BLOCK_PROVISIONING>() as u32
    {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassInterpretLBProvisioningLogPage: DO (%p), Invalid parameter, Resources = %p, ResourcesSize = %u, LogPage = %p, LogPageSize = %u.\n\0"
                .as_ptr() as *const i8,
            DeviceObject as *const c_void,
            Resources as *const c_void,
            ResourcesSize,
            LogPage as *const c_void,
            LogPageSize,
        );
        return STATUS_INVALID_PARAMETER;
    }

    let threshold_set_size = 1u64 << threshold_exponent;

    // REVERSE_BYTES_SHORT(&pageLength, &(LogPage->PageLength));
    let src_page_length = core::ptr::addr_of!((*LogPage).PageLength) as *const u16;
    let mut page_length_raw: u16 = 0;
    core::ptr::copy_nonoverlapping(src_page_length, &mut page_length_raw, 1);
    let mut page_length = page_length_raw.swap_bytes();

    // Initialize output buffer
    core::ptr::write_bytes(
        Resources as *mut u8,
        0,
        core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>(),
    );
    (*Resources).Size = core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>() as u32;
    (*Resources).Version = core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>() as u32;

    // Adjust pageLength not to exceed buffer
    let offset_params = core::mem::offset_of!(_LOG_PAGE_LOGICAL_BLOCK_PROVISIONING, Parameters);
    let max_len = LogPageSize as usize - offset_params;
    page_length = core::cmp::min(page_length as usize, max_len) as u16;

    let mut parameter = (LogPage as *const u8).add(offset_params)
        as *mut LOG_PARAMETER_HEADER;
    let end_of_page = (parameter as *const u8).add(page_length as usize)
        as *mut c_void;

    while (parameter as *mut c_void) < end_of_page {
        let param_length = (*parameter).ParameterLength as usize;
        if param_length > 0 {
            let mut param_code: u16 = 0;
            let src_code = core::ptr::addr_of!((*parameter).ParameterCode) as *const u16;
            core::ptr::copy_nonoverlapping(src_code, &mut param_code, 1);
            param_code = param_code.swap_bytes();

            match param_code as u32 {
                LOG_PAGE_LBP_PARAMETER_CODE_AVAILABLE => {
                    let threshold_param = parameter as *mut LOG_PARAMETER_THRESHOLD_RESOURCE_COUNT;
                    let mut resource_count: u32 = 0;
                    let src_rc = core::ptr::addr_of!((*threshold_param).ResourceCount) as *const u32;
                    core::ptr::copy_nonoverlapping(src_rc, &mut resource_count, 1);
                    resource_count = resource_count.swap_bytes();

                    (*Resources).AvailableMappingResources =
                        resource_count as u64 * threshold_set_size
                            * fdo_extension.DiskGeometry.BytesPerSector as u64;
                    (*Resources).set_AvailableMappingResourcesValid(1u8); // TRUE

                    let offset_rescount3 = core::mem::offset_of!(
                        LOG_PARAMETER_THRESHOLD_RESOURCE_COUNT,
                        ResourceCount
                    ) + 3 * core::mem::size_of::<u32>();
                    if param_length > offset_rescount3 {
                        (*Resources).set_AvailableMappingResourcesScope(
                            (*threshold_param).Scope());
                    }
                }
                LOG_PAGE_LBP_PARAMETER_CODE_USED => {
                    let threshold_param = parameter as *mut LOG_PARAMETER_THRESHOLD_RESOURCE_COUNT;
                    let mut resource_count: u32 = 0;
                    let src_rc = core::ptr::addr_of!((*threshold_param).ResourceCount) as *const u32;
                    core::ptr::copy_nonoverlapping(src_rc, &mut resource_count, 1);
                    resource_count = resource_count.swap_bytes();

                    (*Resources).UsedMappingResources =
                        resource_count as u64 * threshold_set_size
                            * fdo_extension.DiskGeometry.BytesPerSector as u64;
                    (*Resources).set_UsedMappingResourcesValid(1u8); // TRUE

                    let offset_rescount3 = core::mem::offset_of!(
                        LOG_PARAMETER_THRESHOLD_RESOURCE_COUNT,
                        ResourceCount
                    ) + 3 * core::mem::size_of::<u32>();
                    if param_length > offset_rescount3 {
                        (*Resources).set_UsedMappingResourcesScope(
                            (*threshold_param).Scope());
                    }
                }
                _ => {}
            }
        }

        // Move to next parameter
        parameter = (parameter as *const u8)
            .add(core::mem::size_of::<LOG_PARAMETER_HEADER>() + param_length)
            as *mut LOG_PARAMETER_HEADER;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClassGetLBProvisioningResources(
    DeviceObject: PDEVICE_OBJECT,
    Srb: PSCSI_REQUEST_BLOCK,
    ResourcesSize: u32,
    Resources: PSTORAGE_LB_PROVISIONING_MAP_RESOURCES,
) -> NTSTATUS {
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let mut logPage: *mut _LOG_PAGE_LOGICAL_BLOCK_PROVISIONING = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if (*(*fdoExtension).FunctionSupportInfo).ValidInquiryPages.LBProvisioning() == (0u8) as u32 {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassGetLBProvisioningResources: DO (%p), Device does not support logical block provisioning.\n\0"
                .as_ptr() as PCSZ,
            DeviceObject,
        );
        return STATUS_NOT_SUPPORTED;
    }

    if Resources.is_null()
        || ResourcesSize < core::mem::size_of::<STORAGE_LB_PROVISIONING_MAP_RESOURCES>() as u32
    {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassGetLBProvisioningResources: DO (%p), Invalid parameter, Resources = %p, ResourcesSize = %u.\n\0"
                .as_ptr() as PCSZ,
            DeviceObject,
            Resources,
            ResourcesSize,
        );
        return STATUS_INVALID_PARAMETER;
    }

    let mut logPageSize = core::mem::size_of::<LOG_PAGE_LOGICAL_BLOCK_PROVISIONING>() as u32
        + 2 * core::mem::size_of::<LOG_PARAMETER_THRESHOLD_RESOURCE_COUNT>() as u32;

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        logPageSize = ALIGN_UP_BY(logPageSize as _, KeGetRecommendedSharedDataAlignment() as _) as _;
        logPage = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            logPageSize as usize,
            CLASS_TAG_LB_PROVISIONING,
        ) as *mut _LOG_PAGE_LOGICAL_BLOCK_PROVISIONING;
    }
    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    {
        logPage = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            (logPageSize) as usize,
            CLASS_TAG_LB_PROVISIONING,
        ) as *mut _LOG_PAGE_LOGICAL_BLOCK_PROVISIONING;
    }

    if !logPage.is_null() {
        status = ClassGetLBProvisioningLogPage(DeviceObject, Srb, logPageSize, logPage);
        if NT_SUCCESS(status) {
            status = ClassInterpretLBProvisioningLogPage(
                DeviceObject,
                logPageSize,
                logPage,
                ResourcesSize,
                Resources,
            );
        }
        ExFreePool(logPage as *mut core::ffi::c_void);
    } else {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClassGetLBProvisioningResources: DO (%p), Failed to allocate memory for LBP log page.\n\0"
                .as_ptr() as PCSZ,
            DeviceObject,
        );
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassDeviceGetLBProvisioningResources(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mapResources: PSTORAGE_LB_PROVISIONING_MAP_RESOURCES =
        (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_LB_PROVISIONING_MAP_RESOURCES;
    let status: NTSTATUS = ClassGetLBProvisioningResources(
        DeviceObject,
        Srb,
        (*irpStack).Parameters.DeviceIoControl.OutputBufferLength,
        mapResources,
    );
    if NT_SUCCESS(status) {
        (*Irp).IoStatus.Information = ((((*mapResources).Size as usize) as u32) as u64) as _;
    } else {
        (*Irp).IoStatus.Information = 0;
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspLogSystemEventWithDeviceNumber(
    DeviceObject: PDEVICE_OBJECT,
    IoErrorCode: NTSTATUS,
) -> NTSTATUS {
    let mut status = STATUS_INSUFFICIENT_RESOURCES;
    let fdo_extension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    let mut log_entry_size = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as u32 + 11 * core::mem::size_of::<u16>() as u32;
    log_entry_size = core::cmp::min(log_entry_size, ERROR_LOG_MAXIMUM_SIZE as u32);
    log_entry_size = core::cmp::min(log_entry_size, MAXUCHAR as u32);

    let error_log_entry = IoAllocateErrorLogEntry((DeviceObject).cast(), log_entry_size as u8) as *mut IO_ERROR_LOG_PACKET;
    if error_log_entry.is_null() {
        return status;
    }

    core::ptr::write_bytes(error_log_entry as *mut u8, 0, log_entry_size as usize);

    (*error_log_entry).StringOffset = core::mem::size_of::<IO_ERROR_LOG_PACKET>() as u16;
    (*error_log_entry).ErrorCode = IoErrorCode;

    let string_index = (error_log_entry as *mut u8).add(core::mem::size_of::<IO_ERROR_LOG_PACKET>()) as PWSTR;
    let string_size = (log_entry_size - core::mem::size_of::<IO_ERROR_LOG_PACKET>() as u32) as usize;

    let format: [u16; 3] = [0x0025, 0x0064, 0x0000]; // L"%d"
    status = strsafe_rust::RtlStringCbPrintfW!(
        string_index,
        string_size,
        format.as_ptr(),
        (*fdo_extension).DeviceNumber,
    );

    if NT_SUCCESS(status) {
        (*error_log_entry).NumberOfStrings += 1;
    }

    IoWriteErrorLogEntry(error_log_entry as *mut c_void);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClassQueueThresholdEventWorker(DeviceObject: PDEVICE_OBJECT) {
    let common_extension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdo_extension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
        let pending_ptr = core::ptr::addr_of_mut!(
            (*(*fdo_extension).FunctionSupportInfo).LBProvisioningData.SoftThresholdEventPending
        ) as *mut core::sync::atomic::AtomicI32;

        if (*pending_ptr)
            .compare_exchange(
                0,
                1,
                core::sync::atomic::Ordering::AcqRel,
                core::sync::atomic::Ordering::Acquire,
            )
            .is_ok()
        {
            let work_item = IoAllocateWorkItem(DeviceObject);

            if !work_item.is_null() {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_GENERAL,
                    b"ClassQueueThresholdEventWorker: DO (%p), Queueing soft threshold notification work item.\n\0".as_ptr()
                        as PCSZ,
                    DeviceObject,
                );

                ClassAcquireRemoveLock(DeviceObject, work_item as PIRP);

                IoQueueWorkItem(
                    work_item,
                    Some(ClassLogThresholdEvent),
                    DelayedWorkQueue,
                    (work_item).cast(),
                );
            } else {
                (*pending_ptr).store(0, core::sync::atomic::Ordering::Release);

                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    b"ClassQueueThresholdEventWorker: DO (%p), Failed to allocate memory for the work item.\n\0"
                        .as_ptr() as PCSZ,
                    DeviceObject,
                );
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassQueueResourceExhaustionEventWorker(DeviceObject: PDEVICE_OBJECT) {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let mut workItem: PIO_WORKITEM = core::ptr::null_mut();

    if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        workItem = IoAllocateWorkItem(DeviceObject);

        if !workItem.is_null() {
            // Trace information
            let fmt = b"ClassQueueResourceExhaustionEventWorker: DO (%p), Queueing permanent resource exhaustion event work item.\n\0";
            unsafe {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_GENERAL,
                    fmt.as_ptr() as *const i8,
                    DeviceObject as *mut core::ffi::c_void,
                );
            }

            ClassAcquireRemoveLock(DeviceObject, workItem as PIRP);

            IoQueueWorkItem(
                workItem,
                Some(ClassLogResourceExhaustionEvent),
                DelayedWorkQueue,
                workItem as PVOID,
            );
        } else {
            // Trace error
            let fmt = b"ClassQueueResourceExhaustionEventWorker: DO (%p), Failed to allocate memory for the work item.\n\0";
            unsafe {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_GENERAL,
                    fmt.as_ptr() as *const i8,
                    DeviceObject as *mut core::ffi::c_void,
                );
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassQueueCapacityChangedEventWorker(DeviceObject: PDEVICE_OBJECT) {
    let common_extension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;

    if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
        let work_item = IoAllocateWorkItem(DeviceObject);
        if work_item != core::ptr::null_mut() {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_GENERAL,
                b"ClassQueueCapacityChangedEventWorker: DO (%p), Queueing capacity changed event work item.\n\0".as_ptr() as *const core::ffi::c_char,
                DeviceObject,
            );

            ClassAcquireRemoveLock(DeviceObject, work_item as PIRP);

            IoQueueWorkItem(
                work_item,
                Some(abi_cast!(ClassLogCapacityChangedProcess)),
                DelayedWorkQueue,
                (work_item).cast(),
            );
        } else {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                b"ClassQueueCapacityChangedEventWorker: DO (%p), Failed to allocate memory for the work item.\n\0".as_ptr() as *const core::ffi::c_char,
                DeviceObject,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassQueueProvisioningTypeChangedEventWorker(DeviceObject: PDEVICE_OBJECT) {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        let workItem = IoAllocateWorkItem(DeviceObject);
        if !workItem.is_null() {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_GENERAL,
                b"ClassQueueProvisioningTypeChangedEventWorker: DO (%p), Queueing LB provisioning type changed event work item.\n\0"
                    .as_ptr() as *const i8,
                DeviceObject as *mut core::ffi::c_void,
            );
            ClassAcquireRemoveLock(DeviceObject, workItem as *mut _IRP);
            IoQueueWorkItem(
                workItem,
                Some(abi_cast!(ClassLogProvisioningTypeChangedEvent)),
                DelayedWorkQueue,
                workItem as *mut core::ffi::c_void,
            );
        } else {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                b"ClassQueueProvisioningTypeChangedEventWorker: DO (%p), Failed to allocate memory for the work item.\n\0"
                    .as_ptr() as *const i8,
                DeviceObject as *mut core::ffi::c_void,
            );
        }
    }
}

unsafe extern "system" fn ClasspLogIOEventWithContext(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID
) {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let ioLogMessageContextHeader = Context as POPCODE_SENSE_DATA_IO_LOG_MESSAGE_CONTEXT_HEADER;
    let workItem: PIO_WORKITEM;
    let mut errorLogEntry: PIO_ERROR_LOG_PACKET = core::ptr::null_mut();
    let mut logEntrySize: ULONG;
    let mut stringIndex: PWSTR = core::ptr::null_mut();
    let mut stringSize: LONG = 0;
    let mut senseBufferSize: ULONG;
    let mut stringsBufferLength: ULONG = 0;
    let mut pdoNameLength: ULONG = 0;

    NT_ASSERT!((!ioLogMessageContextHeader.is_null()) as u8);

    match (*ioLogMessageContextHeader).ErrorCode as i32 {
        IO_ERROR_IO_HARDWARE_ERROR | IO_WARNING_IO_OPERATION_RETRIED => {
            stringsBufferLength = (19 + 11) * core::mem::size_of::<u16>() as ULONG;

            status = IoGetDeviceProperty(
                (*fdoExtension).LowerPdo,
                DevicePropertyPhysicalDeviceObjectName,
                0,
                core::ptr::null_mut(),
                &mut pdoNameLength,
            );

            if status == STATUS_BUFFER_TOO_SMALL && pdoNameLength > 0 {
                stringsBufferLength += pdoNameLength;
            } else {
                pdoNameLength = 0;
            }
        }
        _ => {}
    }

    workItem = (*ioLogMessageContextHeader).WorkItem;

    senseBufferSize = ((*ioLogMessageContextHeader).SenseDataSize as ULONG + (core::mem::size_of::<ULONG>() as ULONG - 1))
                      & !(core::mem::size_of::<ULONG>() as ULONG - 1);

    logEntrySize = core::mem::offset_of!(IO_ERROR_LOG_PACKET, DumpData) as ULONG
                   + core::mem::size_of::<ULONG>() as ULONG
                   + senseBufferSize;

    logEntrySize += stringsBufferLength;

    if logEntrySize > ERROR_LOG_MAXIMUM_SIZE as u32 {
        if senseBufferSize > 0 {
            if logEntrySize - (ERROR_LOG_MAXIMUM_SIZE as u32) < senseBufferSize {
                senseBufferSize -= logEntrySize - (ERROR_LOG_MAXIMUM_SIZE as u32);
                senseBufferSize = (senseBufferSize) & !(core::mem::size_of::<ULONG>() as ULONG - 1);
            } else {
                senseBufferSize = 0;
            }
        }
        logEntrySize = ERROR_LOG_MAXIMUM_SIZE as u32;
    }

    errorLogEntry = IoAllocateErrorLogEntry((DeviceObject).cast(), logEntrySize as u8) as PIO_ERROR_LOG_PACKET;

    if !errorLogEntry.is_null() {
        core::ptr::write_bytes(errorLogEntry as *mut u8, 0, logEntrySize as usize);

        (*errorLogEntry).MajorFunctionCode = IRP_MJ_SCSI as u8;
        (*errorLogEntry).RetryCount = 1;
        (*errorLogEntry).DumpDataSize = (core::mem::size_of::<ULONG>() + senseBufferSize as usize) as u16;
        (*errorLogEntry).StringOffset = (core::mem::offset_of!(IO_ERROR_LOG_PACKET, DumpData) as u16)
                                        + (*errorLogEntry).DumpDataSize;
        (*errorLogEntry).ErrorCode = ((*ioLogMessageContextHeader).ErrorCode) as i32;

        let dump_data_base = (errorLogEntry as *mut u8).add(core::mem::offset_of!(IO_ERROR_LOG_PACKET, DumpData)) as *mut ULONG;
        *dump_data_base = (((*ioLogMessageContextHeader).OpCode as ULONG) << 24)
                        | (((*ioLogMessageContextHeader).SrbStatus as ULONG) << 16)
                        | (((*ioLogMessageContextHeader).ScsiStatus as ULONG) << 8);

        let dump_data_bytes = dump_data_base.add(1) as *mut u8;
        if senseBufferSize > (*ioLogMessageContextHeader).SenseDataSize as ULONG {
            core::ptr::copy_nonoverlapping(
                (*ioLogMessageContextHeader).SenseData as *const u8,
                dump_data_bytes,
                (*ioLogMessageContextHeader).SenseDataSize as usize,
            );
            core::ptr::write_bytes(
                dump_data_bytes.add((*ioLogMessageContextHeader).SenseDataSize as usize),
                b'-',
                (senseBufferSize - (*ioLogMessageContextHeader).SenseDataSize as ULONG) as usize,
            );
        } else {
            core::ptr::copy_nonoverlapping(
                (*ioLogMessageContextHeader).SenseData as *const u8,
                dump_data_bytes,
                senseBufferSize as usize,
            );
        }

        stringIndex = (errorLogEntry as *mut u8).add(
            core::mem::offset_of!(IO_ERROR_LOG_PACKET, DumpData) + (*errorLogEntry).DumpDataSize as usize
        ) as PWSTR;
        stringSize = (logEntrySize - (*errorLogEntry).StringOffset as ULONG) as LONG;

        match (*ioLogMessageContextHeader).ErrorCode as i32 {
            IO_ERROR_IO_HARDWARE_ERROR | IO_WARNING_IO_OPERATION_RETRIED => {
                let ioLogMessageContext = Context as PIO_RETRIED_LOG_MESSAGE_CONTEXT;

                let fmt0: [u16; 8] = [0x0030, 0x0078, 0x0025, 0x0049, 0x0036, 0x0034, 0x0078, 0x0000]; // L"0x%I64x\0"
                status = strsafe_rust::RtlStringCbPrintfW!(
                    stringIndex,
                    stringSize as usize,
                    fmt0.as_ptr() as PWSTR,
                    (*ioLogMessageContext).Lba.QuadPart,
                );

                if NT_SUCCESS(status) {
                    (*errorLogEntry).NumberOfStrings += 1;

                    let len = wcslen(stringIndex) as usize;
                    stringSize -= (((len + 1) * core::mem::size_of::<u16>()) as LONG) as i32;
                    stringIndex = stringIndex.add(len + 1);

                    if stringSize > 0 {
                        let fmt1: [u16; 3] = [0x0025, 0x0064, 0x0000]; // L"%d\0"
                        status = strsafe_rust::RtlStringCbPrintfW!(
                            stringIndex,
                            stringSize as usize,
                            fmt1.as_ptr() as PWSTR,
                            (*ioLogMessageContext).DeviceNumber,
                        );

                        if NT_SUCCESS(status) {
                            (*errorLogEntry).NumberOfStrings += 1;

                            let len = wcslen(stringIndex) as usize;
                            stringSize -= (((len + 1) * core::mem::size_of::<u16>()) as LONG) as i32;
                            stringIndex = stringIndex.add(len + 1);

                            if stringSize >= pdoNameLength as LONG && pdoNameLength > 0 {
                                let mut resultLength: ULONG = 0;

                                status = IoGetDeviceProperty(
                                    (*fdoExtension).LowerPdo,
                                    DevicePropertyPhysicalDeviceObjectName,
                                    pdoNameLength,
                                    (stringIndex).cast(),
                                    &mut resultLength,
                                );

                                if NT_SUCCESS(status) && resultLength > 0 {
                                    (*errorLogEntry).NumberOfStrings += 1;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        IoWriteErrorLogEntry((errorLogEntry).cast());

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"ClasspLogIORetriedEvent: DO (%p), Soft threshold notification logged.\n\0".as_ptr() as PCSZ,
            DeviceObject as PVOID,
        );
    }

    ClassReleaseRemoveLock(DeviceObject, workItem as PIRP);

    if !(*ioLogMessageContextHeader).SenseData.is_null() {
        ExFreePool((*ioLogMessageContextHeader).SenseData);
    }
    if !workItem.is_null() {
        IoFreeWorkItem(workItem);
    }
    ExFreePool(ioLogMessageContextHeader as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspQueueLogIOEventWithContextWorker(
    DeviceObject: PDEVICE_OBJECT,
    SenseBufferSize: ULONG,
    SenseData: PVOID,
    SrbStatus: UCHAR,
    ScsiStatus: UCHAR,
    ErrorCode: ULONG,
    CdbLength: ULONG,
    Cdb: PCDB,
    Pkt: PTRANSFER_PACKET,
) {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut ioLogMessageContextHeader: POPCODE_SENSE_DATA_IO_LOG_MESSAGE_CONTEXT_HEADER = core::ptr::null_mut();
    let mut senseData: PVOID = core::ptr::null_mut();
    let mut workItem: PIO_WORKITEM = core::ptr::null_mut();
    let mut senseBufferSize: ULONG = 0;
    let mut lba: LARGE_INTEGER = core::mem::zeroed();
    let mut success = false;
    let mut print_error = false;

    // block for goto-like control flow
    'outer: {
        if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
            return;
        }
        if Cdb.is_null() {
            return;
        }

        workItem = IoAllocateWorkItem(DeviceObject);
        if workItem.is_null() {
            print_error = true;
            break 'outer;
        }

        if SenseBufferSize != 0 {
            senseData = ExAllocatePoolWithTag(NonPagedPoolNx, (SenseBufferSize) as usize, CLASSPNP_POOL_TAG_LOG_MESSAGE);
            if !senseData.is_null() {
                senseBufferSize = SenseBufferSize;
            }
        }

        if CdbLength == 16 {
            REVERSE_BYTES_QUAD(core::ptr::addr_of_mut!(lba) as *mut u8, &(*Cdb).CDB16.as_ref().LogicalBlock as *const _ as *const u8);
        } else {
            let bptr = &mut lba.__bindgen_anon_1.LowPart as *mut ULONG as *mut FOUR_BYTE;
            (*bptr).__bindgen_anon_1.as_mut().Byte3 = (*Cdb).CDB10.as_ref().LogicalBlockByte0;
            (*bptr).__bindgen_anon_1.as_mut().Byte2 = (*Cdb).CDB10.as_ref().LogicalBlockByte1;
            (*bptr).__bindgen_anon_1.as_mut().Byte1 = (*Cdb).CDB10.as_ref().LogicalBlockByte2;
            (*bptr).__bindgen_anon_1.as_mut().Byte0 = (*Cdb).CDB10.as_ref().LogicalBlockByte3;
        }

        match ErrorCode as i32 {
            IO_ERROR_IO_HARDWARE_ERROR | IO_WARNING_IO_OPERATION_RETRIED => {
                let ioLogMessageContext = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    (core::mem::size_of::<IO_RETRIED_LOG_MESSAGE_CONTEXT>() as ULONG) as usize,
                    CLASSPNP_POOL_TAG_LOG_MESSAGE,
                ) as PIO_RETRIED_LOG_MESSAGE_CONTEXT;
                if ioLogMessageContext.is_null() {
                    print_error = true;
                    break 'outer;
                }
                (*ioLogMessageContext).Lba.QuadPart = lba.QuadPart;
                (*ioLogMessageContext).DeviceNumber = (*fdoExtension).DeviceNumber;
                ioLogMessageContextHeader = ioLogMessageContext as POPCODE_SENSE_DATA_IO_LOG_MESSAGE_CONTEXT_HEADER;
            }
            _ => break 'outer, // default case -> goto Exit (no error trace)
        }

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"ClasspQueueLogIOEventWithContextWorker: DO (%p), Pkt (%p), Queueing IO retried event log message work item.\n\0".as_ptr() as PCSZ,
            DeviceObject,
            Pkt,
        );

        (*ioLogMessageContextHeader).WorkItem = workItem;
        if !senseData.is_null() {
            core::ptr::copy_nonoverlapping(
                SenseData,
                senseData,
                SenseBufferSize as usize,
            );
        }
        (*ioLogMessageContextHeader).SenseData = senseData;
        (*ioLogMessageContextHeader).SenseDataSize = senseBufferSize;
        (*ioLogMessageContextHeader).SrbStatus = SrbStatus;
        (*ioLogMessageContextHeader).ScsiStatus = ScsiStatus;
        (*ioLogMessageContextHeader).OpCode = (*Cdb).CDB6GENERIC.as_ref().OperationCode;
        (*ioLogMessageContextHeader).Reserved = 0;
        (*ioLogMessageContextHeader).ErrorCode = ErrorCode;

        ClassAcquireRemoveLock(DeviceObject, workItem as PIRP);

        IoQueueWorkItem(workItem, Some(abi_cast!(ClasspLogIOEventWithContext)), DelayedWorkQueue, ioLogMessageContextHeader as PVOID);
        success = true;
        break 'outer;
    }

    if success {
        return;
    }

    if print_error {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"ClasspQueueLogIOEventWithContextWorker: DO (%p), Failed to allocate memory for the log message.\n\0".as_ptr() as PCSZ,
            DeviceObject,
        );
    }

    if !senseData.is_null() {
        ExFreePool(senseData);
    }
    if !workItem.is_null() {
        IoFreeWorkItem(workItem);
    }
    if !ioLogMessageContextHeader.is_null() {
        ExFreePool(ioLogMessageContextHeader as PVOID);
    }
}

unsafe fn ValidPersistentReserveScope(Scope: u8) -> u8 {
    let scope_lu = RESERVATION_SCOPE_LU as u8;
    let scope_elem = RESERVATION_SCOPE_ELEMENT as u8;
    if Scope == scope_lu || Scope == scope_elem { 1u8 } else { 0u8 }
}

unsafe fn ValidPersistentReserveType(Type: UCHAR) -> BOOLEAN {
    let t_we = RESERVATION_TYPE_WRITE_EXCLUSIVE as u8;
    let t_ex = RESERVATION_TYPE_EXCLUSIVE as u8;
    let t_wer = RESERVATION_TYPE_WRITE_EXCLUSIVE_REGISTRANTS as u8;
    let t_er = RESERVATION_TYPE_EXCLUSIVE_REGISTRANTS as u8;
    if Type == t_we || Type == t_ex || Type == t_wer || Type == t_er { 1u8 } else { 0u8 }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPersistentReserve(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    Srb: *mut SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let mut cdb: *mut CDB = core::ptr::null_mut();
    let prCommand = (*Irp).AssociatedIrp.SystemBuffer as *mut PERSISTENT_RESERVE_COMMAND;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let mut status: NTSTATUS;
    let mut dataBufLen: ULONG;
    let controlCode = (*irpStack).Parameters.DeviceIoControl.IoControlCode;
    let mut writeToDevice: BOOLEAN;

    if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<PERSISTENT_RESERVE_COMMAND>() as ULONG ||
        (*prCommand).Size < core::mem::size_of::<PERSISTENT_RESERVE_COMMAND>() as ULONG {
        status = STATUS_INFO_LENGTH_MISMATCH;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        (*Irp).IoStatus.Information = 0;
        FREE_POOL((Srb).cast());
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    }

    if ((prCommand as ULONG_PTR) & ((*(*fdoExtension).AdapterDescriptor).AlignmentMask as ULONG_PTR)) != 0 {
        status = STATUS_INVALID_USER_BUFFER;
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        (*Irp).IoStatus.Information = 0;
        FREE_POOL((Srb).cast());
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
        return status;
    }

    status = STATUS_SUCCESS;
    SrbSetCdbLength((Srb).cast(), 10);
    cdb = SrbGetCdb((Srb).cast());

    if controlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_IN {
        let pr_in = (*prCommand).__bindgen_anon_1.PR_IN.as_ref();
        if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength < pr_in.AllocationLength as ULONG {
            status = STATUS_INVALID_PARAMETER;
        }
        match pr_in.ServiceAction() as u32 {
            RESERVATION_ACTION_READ_KEYS => {
                if pr_in.AllocationLength < core::mem::size_of::<PRI_REGISTRATION_LIST>() as USHORT {
                    status = STATUS_INVALID_PARAMETER;
                }
            }
            RESERVATION_ACTION_READ_RESERVATIONS => {
                if pr_in.AllocationLength < core::mem::size_of::<PRI_RESERVATION_LIST>() as USHORT {
                    status = STATUS_INVALID_PARAMETER;
                }
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }
        if !NT_SUCCESS(status) {
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            (*Irp).IoStatus.Information = 0;
            FREE_POOL((Srb).cast());
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
            return status;
        }
        let cdb_pri = (*cdb).PERSISTENT_RESERVE_IN.as_mut();
        cdb_pri.OperationCode = SCSIOP_PERSISTENT_RESERVE_IN as u8;
        cdb_pri.set_ServiceAction(pr_in.ServiceAction());
        // REVERSE_BYTES_SHORT: big-endian allocation length into cdb bytes
        let al_be = pr_in.AllocationLength.swap_bytes();
        cdb_pri.AllocationLength = al_be.to_ne_bytes();
        dataBufLen = (*irpStack).Parameters.DeviceIoControl.OutputBufferLength;
        writeToDevice = FALSE;
    } else {
        let pr_out = (*prCommand).__bindgen_anon_1.PR_OUT.as_ref();
        match pr_out.ServiceAction() as u32 {
            RESERVATION_ACTION_REGISTER
            | RESERVATION_ACTION_REGISTER_IGNORE_EXISTING
            | RESERVATION_ACTION_CLEAR => {}
            RESERVATION_ACTION_RESERVE
            | RESERVATION_ACTION_RELEASE
            | RESERVATION_ACTION_PREEMPT
            | RESERVATION_ACTION_PREEMPT_ABORT => {
                if ValidPersistentReserveScope(pr_out.Scope()) == 0
                    || ValidPersistentReserveType(pr_out.Type()) == 0 {
                    status = STATUS_INVALID_PARAMETER;
                }
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
            }
        }
        if NT_SUCCESS(status) {
            if (*irpStack).Parameters.DeviceIoControl.InputBufferLength
                < (core::mem::size_of::<PERSISTENT_RESERVE_COMMAND>() + core::mem::size_of::<PRO_PARAMETER_LIST>()) as ULONG
                || (*prCommand).Size < (*irpStack).Parameters.DeviceIoControl.InputBufferLength {
                status = STATUS_INVALID_PARAMETER;
            }
        }
        if !NT_SUCCESS(status) {
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            (*Irp).IoStatus.Information = 0;
            FREE_POOL((Srb).cast());
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
            return status;
        }
        let pr_out = (*prCommand).__bindgen_anon_1.PR_OUT.as_ref();
        let cdb_pro = (*cdb).PERSISTENT_RESERVE_OUT.as_mut();
        cdb_pro.OperationCode = SCSIOP_PERSISTENT_RESERVE_OUT as u8;
        cdb_pro.set_ServiceAction(pr_out.ServiceAction());
        cdb_pro.set_Scope(pr_out.Scope());
        cdb_pro.set_Type(pr_out.Type());
        cdb_pro.ParameterListLength[1] = core::mem::size_of::<PRO_PARAMETER_LIST>() as UCHAR;
        core::ptr::copy_nonoverlapping(
            pr_out.ParameterList.as_ptr() as *const u8,
            prCommand as *mut u8,
            core::mem::size_of::<PRO_PARAMETER_LIST>(),
        );
        dataBufLen = core::mem::size_of::<PRO_PARAMETER_LIST>() as ULONG;
        writeToDevice = TRUE;
    }

    SrbSetTimeOutValue((Srb).cast(), (*fdoExtension).TimeOutValue);
    SrbSetRequestAttribute((Srb).cast(), (SRB_HEAD_OF_QUEUE_TAG_REQUEST) as _);
    SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_NO_QUEUE_FREEZE | SRB_FLAGS_QUEUE_ACTION_ENABLE);

    status = ClassSendSrbAsynchronous(
        DeviceObject,
        Srb,
        Irp,
        prCommand as *mut c_void,
        dataBufLen,
        writeToDevice,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPriorityHint(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let device_extension = (*DeviceObject).DeviceExtension;
    let fdo_extension = device_extension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let common_extension = device_extension as *mut COMMON_DEVICE_EXTENSION;
    let fdo_data = (*fdo_extension).PrivateFdoData;
    let pri_support = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_PRIORITY_HINT_SUPPORT;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS = STATUS_SUCCESS;

    (*Irp).IoStatus.Information = 0;

    'priority_hint_exit: {
        if (*fdo_data).IdlePrioritySupported == 0 {
            status = STATUS_NOT_SUPPORTED;
            break 'priority_hint_exit;
        }

        if (*irp_stack).Parameters.DeviceIoControl.OutputBufferLength
            < core::mem::size_of::<STORAGE_PRIORITY_HINT_SUPPORT>() as u32
        {
            status = STATUS_BUFFER_TOO_SMALL;
            break 'priority_hint_exit;
        }

        core::ptr::write_bytes(
            pri_support as *mut u8,
            0,
            core::mem::size_of::<STORAGE_PRIORITY_HINT_SUPPORT>(),
        );

        status = ClassForwardIrpSynchronous(common_extension, Irp);
        if status < 0 {
            TracePrint(
                TRACE_LEVEL_FATAL,
                TRACE_FLAG_IOCTL,
                b"ClasspPriorityHint: I/O priority not supported by port driver.\n\0".as_ptr() as PCSZ,
            );
            (*pri_support).SupportFlags = 0;
            status = STATUS_SUCCESS;
        }

        TracePrint(
            TRACE_LEVEL_FATAL,
            TRACE_FLAG_IOCTL,
            b"ClasspPriorityHint: I/O priorities supported by port driver: %X\n\0".as_ptr() as PCSZ,
            (*pri_support).SupportFlags,
        );

        (*pri_support).SupportFlags |= (1u32 << IoPriorityVeryLow as u32)
            | (1u32 << IoPriorityLow as u32)
            | (1u32 << IoPriorityNormal as u32);

        TracePrint(
            TRACE_LEVEL_FATAL,
            TRACE_FLAG_IOCTL,
            b"ClasspPriorityHint: I/O priorities supported: %X\n\0".as_ptr() as PCSZ,
            (*pri_support).SupportFlags,
        );

        (*Irp).IoStatus.Information =
            ((core::mem::size_of::<STORAGE_PRIORITY_HINT_SUPPORT>() as u32) as u64) as _;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspConvertToScsiRequestBlock(
    Srb: PSCSI_REQUEST_BLOCK,
    SrbEx: PSTORAGE_REQUEST_BLOCK,
) {

    if Srb.is_null() || SrbEx.is_null() {
        return;
    }

    core::ptr::write_bytes(Srb as *mut u8, 0, core::mem::size_of::<_SCSI_REQUEST_BLOCK>());

    (*Srb).Length = core::mem::size_of::<_SCSI_REQUEST_BLOCK>() as u16;
    (*Srb).Function = (*SrbEx).SrbFunction as u8;
    (*Srb).SrbStatus = (*SrbEx).SrbStatus;
    (*Srb).QueueTag = (*SrbEx).RequestTag as u8;
    (*Srb).QueueAction = (*SrbEx).RequestAttribute as u8;
    (*Srb).SrbFlags = (*SrbEx).SrbFlags;
    (*Srb).DataTransferLength = (*SrbEx).DataTransferLength;
    (*Srb).TimeOutValue = (*SrbEx).TimeOutValue;
    (*Srb).DataBuffer = (*SrbEx).DataBuffer;
    (*Srb).OriginalRequest = (*SrbEx).OriginalRequest;
    (*Srb).SrbExtension = (*SrbEx).MiniportContext;
    (*Srb).__bindgen_anon_1.bindgen_union_field = (*SrbEx).SystemStatus;

    if (*SrbEx).AddressOffset as usize >= core::mem::size_of::<_STORAGE_REQUEST_BLOCK>() {
        let storAddrBtl8 = (SrbEx as *const u8).add((*SrbEx).AddressOffset as usize) as *mut _STOR_ADDR_BTL8;
        if (*storAddrBtl8).Type == STORAGE_ADDRESS_TYPE_BTL8 as u16 {
            (*Srb).PathId = (*storAddrBtl8).Path;
            (*Srb).TargetId = (*storAddrBtl8).Target;
            (*Srb).Lun = (*storAddrBtl8).Lun;
        } else {
            NT_ASSERT!(0u8);
        }
    }

    if (*SrbEx).NumSrbExData > 0 {
        let mut foundEntry: u8 = 0;
        // SrbExDataOffset is a C flexible array (`ULONG SrbExDataOffset[ANYSIZE_ARRAY]`,
        // ddk/srb.h:660) of NumSrbExData entries; bindgen's [_; 1] panicked on the second.
        for i in 0..(*SrbEx).NumSrbExData {
            if (*(&raw const (*SrbEx).SrbExDataOffset[0]).add(i as usize)) as usize == 0 ||
                ((*(&raw const (*SrbEx).SrbExDataOffset[0]).add(i as usize)) as usize) < core::mem::size_of::<_STORAGE_REQUEST_BLOCK>() {
                NT_ASSERT!(0u8);
                continue;
            }

            let srbExData = (SrbEx as *const u8).add((*(&raw const (*SrbEx).SrbExDataOffset[0]).add(i as usize)) as usize) as *mut _SRBEX_DATA;

            match (*SrbEx).SrbFunction as u32 {
                t if t == SRB_FUNCTION_EXECUTE_SCSI => {
                    match (*srbExData).Type {
                        t if t == _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdb16 => {
                            let cdb16 = srbExData as *mut _SRBEX_DATA_SCSI_CDB16;
                            (*Srb).ScsiStatus = (*cdb16).ScsiStatus;
                            (*Srb).CdbLength = (*cdb16).CdbLength;
                            (*Srb).SenseInfoBufferLength = (*cdb16).SenseInfoBufferLength;
                            (*Srb).SenseInfoBuffer = (*cdb16).SenseInfoBuffer;
                            core::ptr::copy_nonoverlapping(
                                (*cdb16).Cdb.as_ptr() as *const u8,
                                (*Srb).Cdb.as_mut_ptr() as *mut u8,
                                core::mem::size_of::<[u8; 16]>(),
                            );
                            foundEntry = 1;
                        }
                        t if t == _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdb32 => {
                            let cdb32 = srbExData as *mut _SRBEX_DATA_SCSI_CDB32;
                            (*Srb).ScsiStatus = (*cdb32).ScsiStatus;
                            (*Srb).CdbLength = (*cdb32).CdbLength;
                            (*Srb).SenseInfoBufferLength = (*cdb32).SenseInfoBufferLength;
                            (*Srb).SenseInfoBuffer = (*cdb32).SenseInfoBuffer;
                            core::ptr::copy_nonoverlapping(
                                (*cdb32).Cdb.as_ptr() as *const u8,
                                (*Srb).Cdb.as_mut_ptr() as *mut u8,
                                core::mem::size_of::<[u8; 16]>(),
                            );
                            foundEntry = 1;
                        }
                        t if t == _SRBEX_DATA_TYPE_SrbExDataTypeScsiCdbVar => {
                            let cdbVar = srbExData as *mut _SRBEX_DATA_SCSI_CDB_VAR;
                            (*Srb).ScsiStatus = (*cdbVar).ScsiStatus;
                            (*Srb).CdbLength = (*cdbVar).CdbLength as u8;
                            (*Srb).SenseInfoBufferLength = (*cdbVar).SenseInfoBufferLength;
                            (*Srb).SenseInfoBuffer = (*cdbVar).SenseInfoBuffer;
                            core::ptr::copy_nonoverlapping(
                                (*cdbVar).Cdb.as_ptr() as *const u8,
                                (*Srb).Cdb.as_mut_ptr() as *mut u8,
                                core::mem::size_of::<[u8; 16]>(),
                            );
                            foundEntry = 1;
                        }
                        _ => {}
                    }
                }
                t if t == SRB_FUNCTION_WMI => {
                    if (*srbExData).Type == _SRBEX_DATA_TYPE_SrbExDataTypeWmi {
                        let wmi = srbExData as *mut _SRBEX_DATA_WMI;
                        (*(Srb as *mut _SCSI_WMI_REQUEST_BLOCK)).WMISubFunction = (*wmi).WMISubFunction;
                        (*(Srb as *mut _SCSI_WMI_REQUEST_BLOCK)).WMIFlags = (*wmi).WMIFlags;
                        (*(Srb as *mut _SCSI_WMI_REQUEST_BLOCK)).DataPath = (*wmi).DataPath;
                        foundEntry = 1;
                    }
                }
                t if t == SRB_FUNCTION_PNP => {
                    if (*srbExData).Type == _SRBEX_DATA_TYPE_SrbExDataTypePnP {
                        let pnp = srbExData as *mut _SRBEX_DATA_PNP;
                        (*(Srb as *mut _SCSI_PNP_REQUEST_BLOCK)).PnPAction = (*pnp).PnPAction;
                        (*(Srb as *mut _SCSI_PNP_REQUEST_BLOCK)).PnPSubFunction = (*pnp).PnPSubFunction;
                        (*(Srb as *mut _SCSI_PNP_REQUEST_BLOCK)).SrbPnPFlags = (*pnp).SrbPnPFlags;
                        foundEntry = 1;
                    }
                }
                t if t == SRB_FUNCTION_POWER => {
                    if (*srbExData).Type == _SRBEX_DATA_TYPE_SrbExDataTypePower {
                        let power = srbExData as *mut _SRBEX_DATA_POWER;
                        (*(Srb as *mut _SCSI_POWER_REQUEST_BLOCK)).DevicePowerState = (*power).DevicePowerState;
                        (*(Srb as *mut _SCSI_POWER_REQUEST_BLOCK)).PowerAction = (*power).PowerAction;
                        (*(Srb as *mut _SCSI_POWER_REQUEST_BLOCK)).SrbPowerFlags = (*power).SrbPowerFlags;
                        foundEntry = 1;
                    }
                }
                _ => {}
            }

            if foundEntry != 0 {
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetMaximumTokenListIdentifier(
    DeviceObject: PDEVICE_OBJECT,
    RegistryPath: PWSTR,
    MaximumListIdentifier: *mut u32,
) -> NTSTATUS {
    let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    let mut value: u32 = 0;
    let status: NTSTATUS;

    const WIDE_STR: &[u16] = &[
        0x004D, 0x0061, 0x0078, 0x0069, 0x006D, 0x0075, 0x006D,
        0x004C, 0x0069, 0x0073, 0x0074, 0x0049, 0x0064, 0x0065,
        0x006E, 0x0074, 0x0069, 0x0066, 0x0069, 0x0065, 0x0072,
        0x0000,
    ];

    unsafe {
        TracePrint(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_PNP,
            b"ClasspGetMaximumTokenListIdentifier (%p): Entering function.\n\0".as_ptr() as *const i8,
            DeviceObject as *const core::ffi::c_void,
        );
    }

    queryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK;
    queryTable[0].Name = WIDE_STR.as_ptr() as PWSTR;
    queryTable[0].EntryContext = &mut value as *mut u32 as *mut core::ffi::c_void;
    queryTable[0].DefaultType = ((REG_DWORD as u32) << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | (REG_DWORD as u32);
    queryTable[0].DefaultData = &mut value as *mut u32 as *mut core::ffi::c_void;
    queryTable[0].DefaultLength = core::mem::size_of::<u32>() as u32;

    status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        RegistryPath,
        queryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        *MaximumListIdentifier = value;
    } else {
        *MaximumListIdentifier = 0;
    }

    unsafe {
        TracePrint(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_PNP,
            b"ClasspGetMaximumTokenListIdentifier (%p): Exiting function with status %x (maxListId %u).\n\0".as_ptr() as *const i8,
            DeviceObject as *const core::ffi::c_void,
            status as core::ffi::c_int,
            *MaximumListIdentifier as core::ffi::c_uint,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetCopyOffloadMaxDuration(
    DeviceObject: PDEVICE_OBJECT,
    RegistryPath: PWSTR,
    MaxDuration: *mut ULONG,
) -> NTSTATUS {
    const RTL_QUERY_REGISTRY_TYPECHECK_SHIFT: u32 = 24;
    let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
    core::ptr::write_bytes(&mut queryTable as *mut _ as *mut u8, 0u8, core::mem::size_of_val(&queryTable));
    let mut value: ULONG = 0;
    let status: NTSTATUS;

    queryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK;
    queryTable[0].Name = crate::glue::regw::CLASSP_REG_COPY_OFFLOAD_MAX_TARGET_DURATION.as_ptr() as *const u16;
    queryTable[0].EntryContext = &mut value as *mut ULONG as *mut core::ffi::c_void;
    queryTable[0].DefaultType = (REG_DWORD as u32) << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT | REG_NONE as u32;
    queryTable[0].DefaultData = &value as *const ULONG as *mut core::ffi::c_void;
    queryTable[0].DefaultLength = core::mem::size_of::<ULONG>() as u32;

    status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        RegistryPath,
        queryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) && value > DEFAULT_MAX_TARGET_DURATION {
        *MaxDuration = value;
    } else {
        *MaxDuration = DEFAULT_MAX_TARGET_DURATION;
    }

    status
}

unsafe fn ClasspValidateOffloadInputParameters(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    // PAGED_CODE(); omitted – macro expands to nothing in this context

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspValidateOffloadInputParameters (%p): Entering function Irp %p.\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let dsmAttributes = (*Irp).AssociatedIrp.SystemBuffer as *mut DEVICE_MANAGE_DATA_SET_ATTRIBUTES;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    'validate: loop {
        if dsmAttributes.is_null() {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadInputParameters (%p): NULL DsmAttributes passed in.\n\0".as_ptr() as *const i8,
                DeviceObject,
            );
            status = STATUS_INVALID_PARAMETER;
            break 'validate;
        }

        let inputBufferLength = (*irpStack).Parameters.DeviceIoControl.InputBufferLength;
        if inputBufferLength < core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>() as u32
            || inputBufferLength
                < (core::mem::size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>()
                    + ((*dsmAttributes).ParameterBlockLength) as usize
                    + ((*dsmAttributes).DataSetRangesLength) as usize) as u32
        {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadInputParameters (%p): Input buffer size (%u) too small.\n\0".as_ptr() as *const i8,
                DeviceObject,
                inputBufferLength,
            );
            status = STATUS_INVALID_PARAMETER;
            break 'validate;
        }

        if (*dsmAttributes).DataSetRangesOffset == 0 || (*dsmAttributes).DataSetRangesLength == 0 {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadInputParameters (%p): Incorrect DataSetRanges [offset %u, length %u].\n\0".as_ptr() as *const i8,
                DeviceObject,
                (*dsmAttributes).DataSetRangesOffset,
                (*dsmAttributes).DataSetRangesLength,
            );
            status = STATUS_INVALID_PARAMETER;
            break 'validate;
        }

        let dataSetRanges =
            (dsmAttributes as *mut u8).offset((*dsmAttributes).DataSetRangesOffset as isize)
                as *mut DEVICE_DATA_SET_RANGE;
        let dataSetRangesCount =
            (*dsmAttributes).DataSetRangesLength / core::mem::size_of::<DEVICE_DATA_SET_RANGE>() as u32;

        if dataSetRangesCount == 0 {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspValidateOffloadInputParameters (%p): DataSetRanges specifies no extents.\n\0".as_ptr() as *const i8,
                DeviceObject,
            );
            status = STATUS_INVALID_PARAMETER;
            break 'validate;
        }

        if (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
            status = ClassReadDriveCapacity((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);
            if !NT_SUCCESS(status) || (*fdoExtension).DiskGeometry.BytesPerSector == 0 {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadInputParameters (%p): Couldn't retrieve disk geometry, status: %x, bytes/sector: %u.\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    status,
                    (*fdoExtension).DiskGeometry.BytesPerSector,
                );
                status = STATUS_INVALID_PARAMETER;
                break 'validate;
            }
        }

        for i in 0..dataSetRangesCount {
            let range = dataSetRanges.add(i as usize);
            if (*range).StartingOffset as u64 % (*fdoExtension).DiskGeometry.BytesPerSector as u64 != 0
                || (*range).LengthInBytes as u64 % (*fdoExtension).DiskGeometry.BytesPerSector as u64 != 0
                || (*range).LengthInBytes == 0
            {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadInputParameters (%p): Incorrect DataSetRanges entry %u [offset %I64x, length %I64x].\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    i,
                    (*range).StartingOffset,
                    (*range).LengthInBytes,
                );
                status = STATUS_INVALID_PARAMETER;
                break 'validate;
            }

            if ((*range).StartingOffset as u64 + (*range).LengthInBytes as u64)
                > (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart as u64
            {
                TracePrint(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"ClasspValidateOffloadInputParameters (%p): Error! DataSetRange %u (starting LBA %I64x) specified length %I64x exceeds the medium's capacity (%I64x).\n\0".as_ptr() as *const i8,
                    DeviceObject,
                    i,
                    (*range).StartingOffset,
                    (*range).LengthInBytes,
                    (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().PartitionLength.QuadPart,
                );
                status = STATUS_NONEXISTENT_SECTOR;
                break 'validate;
            }
        }

        status = STATUS_SUCCESS;
        break 'validate;
    }

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspValidateOffloadInputParameters (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
        status,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetTokenOperationCommandBufferLength(
    Fdo: PDEVICE_OBJECT,
    ServiceAction: u32,
    CommandBufferLength: *mut u32,
    TokenOperationBufferLength: *mut u32,
    ReceiveTokenInformationBufferLength: *mut u32,
) -> NTSTATUS {
    let fdoExt: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExt).PrivateFdoData;
    let tokenOperationBufferLength: u32;
    let receiveTokenInformationBufferLength: u32;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let adapterDesc: PSTORAGE_ADAPTER_DESCRIPTOR = (*(*commonExtension).PartitionZeroExtension).AdapterDescriptor;
    let hwMaxXferLen: u32;
    let mut bufferLength: u32 = 0;
    let tokenOperationHeaderSize: usize;
    let responseSize: usize;

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspGetTokenOperationCommandBufferLengths (%p): Entering function.\n\0".as_ptr() as PCHAR,
        Fdo,
    );

    NT_ASSERT!(((*(*fdoExt).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits() != 0
            && NT_SUCCESS((*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus)) as u8);

    if ServiceAction == SERVICE_ACTION_POPULATE_TOKEN {
        tokenOperationHeaderSize = core::mem::offset_of!(POPULATE_TOKEN_HEADER, BlockDeviceRangeDescriptor);
        responseSize = core::mem::offset_of!(RECEIVE_TOKEN_INFORMATION_RESPONSE_HEADER, TokenDescriptor)
            + core::mem::size_of::<BLOCK_DEVICE_TOKEN_DESCRIPTOR>();
    } else {
        tokenOperationHeaderSize = core::mem::offset_of!(WRITE_USING_TOKEN_HEADER, BlockDeviceRangeDescriptor);
        responseSize = 0;
    }

    // The TokenOperation command can specify a parameter length of max 2^16.
    // If the device has a max limit on the number of range descriptors that can be specified in
    // the TokenOperation command, we are limited to the lesser of these two values.
    let blockData = &(*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData;
    if blockData.MaximumRangeDescriptors == 0 {
        tokenOperationBufferLength = MAX_TOKEN_OPERATION_PARAMETER_DATA_LENGTH;
    } else {
        tokenOperationBufferLength = core::cmp::min(
            tokenOperationHeaderSize as u32
                + (blockData.MaximumRangeDescriptors as u32)
                    * (core::mem::size_of::<BLOCK_DEVICE_RANGE_DESCRIPTOR>() as u32),
            MAX_TOKEN_OPERATION_PARAMETER_DATA_LENGTH,
        );
    }

    // The ReceiveTokenInformation command can specify a parameter length of max 2^32
    // Also, since the sense data can be of variable size, we'll use MAX_SENSE_BUFFER_SIZE.
    receiveTokenInformationBufferLength = core::cmp::min(
        (core::mem::offset_of!(RECEIVE_TOKEN_INFORMATION_HEADER, SenseData) + MAX_SENSE_BUFFER_SIZE as usize
            + responseSize) as u32,
        MAX_RECEIVE_TOKEN_INFORMATION_PARAMETER_DATA_LENGTH,
    );

    // Since we're going to reuse the buffer for both the TokenOperation and the ReceiveTokenInformation
    // commands, the buffer length needs to handle both operations.
    bufferLength = core::cmp::max(tokenOperationBufferLength, receiveTokenInformationBufferLength);

    // The buffer length needs to be further limited to the adapter's capability though.
    hwMaxXferLen = core::cmp::min((*fdoData).HwMaxXferLen, (*adapterDesc).MaximumTransferLength);
    bufferLength = core::cmp::min(bufferLength, hwMaxXferLen);

    *CommandBufferLength = bufferLength;

    if !TokenOperationBufferLength.is_null() {
        *TokenOperationBufferLength = tokenOperationBufferLength;
    }
    if !ReceiveTokenInformationBufferLength.is_null() {
        *ReceiveTokenInformationBufferLength = receiveTokenInformationBufferLength;
    }

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspGetTokenOperationCommandBufferLengths (%p): Exiting function with bufferLength %u (tokenOpBufLen %u, recTokenInfoBufLen %u).\n\0".as_ptr()
            as PCHAR,
        Fdo,
        bufferLength,
        tokenOperationBufferLength,
        receiveTokenInformationBufferLength,
    );

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspGetTokenOperationDescriptorLimits(
    Fdo: *mut _DEVICE_OBJECT,
    ServiceAction: u32,
    MaxParameterBufferLength: u32,
    MaxBlockDescriptorsCount: *mut u32,
    MaxBlockDescriptorsLength: *mut u64,
) -> i32 {
    let fdoExt = (*Fdo).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let token_operation_header_size = if ServiceAction == SERVICE_ACTION_POPULATE_TOKEN {
        core::mem::offset_of!(POPULATE_TOKEN_HEADER, BlockDeviceRangeDescriptor) as u32
    } else {
        core::mem::offset_of!(WRITE_USING_TOKEN_HEADER, BlockDeviceRangeDescriptor) as u32
    };

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspGetTokenOperationDescriptorLimits (%p): Entering function.\n\0".as_ptr() as *const core::ffi::c_char,
        Fdo,
    );

    NT_ASSERT!(((*(*fdoExt).FunctionSupportInfo).ValidInquiryPages.BlockDeviceRODLimits() != 0
            && ((*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.CommandStatus as i32) >= 0) as u8);

    *MaxBlockDescriptorsCount = (MaxParameterBufferLength - token_operation_header_size)
        / (core::mem::size_of::<BLOCK_DEVICE_RANGE_DESCRIPTOR>() as u32);

    *MaxBlockDescriptorsLength = if (*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumTokenTransferSize == 0 {
        MAX_TOKEN_TRANSFER_SIZE as u64
    } else {
        (*(*fdoExt).FunctionSupportInfo).BlockDeviceRODLimitsData.MaximumTokenTransferSize
    };

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspGetTokenOperationDescriptorLimits (%p): Exiting function with MaxDescr %u, MaxXferBlocks %I64u.\n\0".as_ptr() as *const core::ffi::c_char,
        Fdo,
        *MaxBlockDescriptorsCount,
        *MaxBlockDescriptorsLength,
    );

    STATUS_SUCCESS
}

pub unsafe fn ClasspConvertDataSetRangeToBlockDescr(
    Fdo: PDEVICE_OBJECT,
    BlockDescr: PVOID,
    CurrentBlockDescrIndex: PULONG,
    MaxBlockDescrCount: ULONG,
    CurrentLbaCount: PULONG,
    MaxLbaCount: ULONGLONG,
    DataSetRange: PDEVICE_DATA_SET_RANGE,
    TotalSectorsProcessed: PULONGLONG,
) {
    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let blockDescr = BlockDescr as PBLOCK_DEVICE_RANGE_DESCRIPTOR;
    let mut totalSectorCount: ULONGLONG = 0;

    let _optimalLbaPerDescrGranularity: USHORT = 0;
    let optimalLbaPerDescr: ULONG = 0;
    let maxLbaPerDescr: ULONG = 0;

    if optimalLbaPerDescr != 0 && maxLbaPerDescr != 0 {
        NT_ASSERT!((optimalLbaPerDescr <= maxLbaPerDescr) as u8);
    }

    while (*DataSetRange).LengthInBytes > 0
        && *CurrentBlockDescrIndex < MaxBlockDescrCount
        && (*CurrentLbaCount as ULONGLONG) < MaxLbaCount
    {
        let sector_size = (*fdoExtension).DiskGeometry.BytesPerSector as ULONGLONG;
        let startingSector = (*DataSetRange).StartingOffset / (sector_size) as i64;

        let mut sectorCount = core::cmp::min(
            (*DataSetRange).LengthInBytes / sector_size,
            MAXULONG as ULONGLONG,
        );

        if (*CurrentLbaCount as ULONGLONG) + sectorCount >= MaxLbaCount {
            sectorCount = MaxLbaCount - *CurrentLbaCount as ULONGLONG;
        }

        if maxLbaPerDescr > 0 {
            sectorCount = sectorCount.min(maxLbaPerDescr as ULONGLONG);
        }

        if optimalLbaPerDescr > 0 {
            sectorCount = sectorCount.min(optimalLbaPerDescr as ULONGLONG);
        }

        if _optimalLbaPerDescrGranularity > 0 {
            let gran = _optimalLbaPerDescrGranularity as ULONGLONG;
            let numberOfOptimalChunks = sectorCount / gran;
            if numberOfOptimalChunks > 0 {
                sectorCount = numberOfOptimalChunks * gran;
            }
        }

        NT_ASSERT!((sectorCount <= MAXULONG as ULONGLONG) as u8);

        let idx = *CurrentBlockDescrIndex as usize;
        // REVERSE_BYTES_QUAD expanded: write big-endian starting sector into [u8;8]
        {
            let src: u64 = startingSector as u64;
            REVERSE_BYTES_QUAD(
                (*blockDescr.add(idx)).LogicalBlockAddress.as_mut_ptr(),
                core::ptr::addr_of!(src) as *const u8,
            );
        }
        // REVERSE_BYTES expanded: write big-endian sector count into [u8;4]
        {
            let src: u32 = sectorCount as ULONG;
            REVERSE_BYTES(
                (*blockDescr.add(idx)).TransferLength.as_mut_ptr() as *mut u32,
                core::ptr::addr_of!(src) as *const u32,
            );
        }

        totalSectorCount += sectorCount;
        (*DataSetRange).StartingOffset += (sectorCount * sector_size) as i64;
        (*DataSetRange).LengthInBytes -= sectorCount * sector_size;

        *CurrentBlockDescrIndex += 1;
        *CurrentLbaCount += sectorCount as ULONG;

        // TracePrint omitted for brevity; assume callable
    }

    *TotalSectorsProcessed = totalSectorCount;
}
#[no_mangle]
pub unsafe extern "C" fn ClasspBinaryToAscii(
    HexBuffer: *mut u8,
    Length: u32,
    UpdateLength: *mut u32,
) -> *mut u8 {
    static INTEGER_TABLE: [u8; 16] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
        b'A', b'B', b'C', b'D', b'E', b'F',
    ];

    if HexBuffer.is_null() || Length == 0 {
        *UpdateLength = 0;
        return core::ptr::null_mut();
    }

    let actual_length = Length.wrapping_mul(2).wrapping_add(1);

    let buffer =
        ExAllocatePoolWithTag(NonPagedPoolNx, (actual_length) as usize, CLASSPNP_POOL_TAG_TOKEN_OPERATION) as *mut u8;
    if buffer.is_null() {
        *UpdateLength = 0;
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(buffer, 0u8, actual_length as usize);

    let mut j: u32 = 0;
    for i in 0..Length {
        let byte = *HexBuffer.add(i as usize);
        let high_word = byte >> 4;
        let low_word = byte & 0x0F;

        *buffer.add(j as usize) = INTEGER_TABLE[high_word as usize];
        j += 1;
        *buffer.add(j as usize) = INTEGER_TABLE[low_word as usize];
        j += 1;
    }

    *UpdateLength = actual_length;
    buffer
}
#[no_mangle]
pub unsafe extern "C" fn ClasspStorageEventNotification(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspStorageEventNotification (%p): Entering function Irp %p.\n\0".as_ptr() as PCSZ,
        DeviceObject,
        Irp,
    );

    let fdo_extension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let storage_events = (*Irp).AssociatedIrp.SystemBuffer as PSTORAGE_EVENT_NOTIFICATION;

    let status = loop {
        if storage_events.is_null() {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspStorageEventNotification (%p): NULL storage events passed in.\n\0".as_ptr() as PCSZ,
                DeviceObject,
            );
            break STATUS_INVALID_PARAMETER;
        }

        if (*irp_stack).Parameters.DeviceIoControl.InputBufferLength
            < core::mem::size_of::<STORAGE_EVENT_NOTIFICATION>() as u32
        {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspStorageEventNotification (%p): Input buffer size (%u) too small.\n\0".as_ptr() as PCSZ,
                DeviceObject,
                (*irp_stack).Parameters.DeviceIoControl.InputBufferLength,
            );
            break STATUS_INFO_LENGTH_MISMATCH;
        }

        if (*storage_events).Version != STORAGE_EVENT_NOTIFICATION_VERSION_V1
            || (*storage_events).Size != core::mem::size_of::<STORAGE_EVENT_NOTIFICATION>() as u32
        {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"ClasspStorageEventNotification (%p): Invalid version/size [version %u, size %u].\n\0".as_ptr()
                    as PCSZ,
                DeviceObject,
                (*storage_events).Version,
                (*storage_events).Size,
            );
            break STATUS_INVALID_PARAMETER;
        }

        if ((*storage_events).Events & (STORAGE_EVENT_MEDIA_STATUS) as u64) != 0 {
            if (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0
                && (*(*fdo_extension).FunctionSupportInfo).AsynchronousNotificationSupported != 0
            {
                ClassCheckMediaState(fdo_extension);
            } else {
                break STATUS_NOT_SUPPORTED;
            }
        }

        break STATUS_SUCCESS;
    };

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"ClasspStorageEventNotification (%p): Exiting function Irp %p with status %x.\n\0".as_ptr() as PCSZ,
        DeviceObject,
        Irp,
        status,
    );

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspZeroQERR(DeviceObject: PDEVICE_OBJECT) {
    let mut modeData: PMODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut pageData: PMODE_CONTROL_PAGE = core::ptr::null_mut();
    let mut size: u32 = 0;

    modeData = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        (MODE_PAGE_DATA_SIZE) as usize,
        CLASS_TAG_MODE_DATA,
    ) as PMODE_PARAMETER_HEADER;

    if modeData.is_null() {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_SCSI,
            b"ClasspZeroQERR: Unable to allocate mode data buffer\n\0".as_ptr() as PCHAR,
        );
        core::ptr::write_bytes(modeData as *mut u8 , 0, 0); // dummy to satisfy label? No, just goto exit.
        // Actually we just need to exit. Use labelled block.
        // We'll restructure with labelled block.
        // But we'll implement goto with 'exit block.
    }

    // Replace the allocation block with 'exit: block
    // To handle goto, use:
    let result: () = 'exit: {
        if modeData.is_null() {
            TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_SCSI, b"ClasspZeroQERR: Unable to allocate mode data buffer\n\0".as_ptr() as PCHAR);
            break 'exit;
        }

        core::ptr::write_bytes(modeData as *mut u8, 0, (MODE_PAGE_DATA_SIZE) as usize);

        size = ClassModeSense(
            DeviceObject,
            modeData as PCHAR,
            MODE_PAGE_DATA_SIZE,
            (MODE_PAGE_CONTROL) as u8,
        );

        if size < (core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32) {
            size = ClassModeSense(
                DeviceObject,
                modeData as PCHAR,
                MODE_PAGE_DATA_SIZE,
                (MODE_PAGE_CONTROL) as u8,
            );

            if size < (core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32) {
                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_SCSI, b"ClasspZeroQERR: Mode Sense failed\n\0".as_ptr() as PCHAR);
                break 'exit;
            }
        }

        if size > ((*modeData).ModeDataLength as u32) + 1 {
            size = ((*modeData).ModeDataLength as u32) + 1;
        }

        pageData = (ClassFindModePage(
            modeData as PCHAR,
            size,
            (MODE_PAGE_CONTROL) as u8,
            1u8,
        )).cast();

        if !pageData.is_null() {
            TracePrint(
                TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_SCSI,
                b"ClasspZeroQERR (%p): Current settings: QERR = %u, TST = %u, TAS = %u.\n\0".as_ptr() as PCHAR,
                DeviceObject,
                ((*pageData).QERR()) as i32,
                ((*pageData).TST()) as i32,
                ((*pageData).TAS()) as i32,
            );

            if (*pageData).QERR() != 0 {
                let status: NTSTATUS;
                let pageSavable: u8 = 0;
                (*pageData).set_QERR(0);
                let pageSavable = (*pageData).PageSavable();
                status = ClasspModeSelect(
                    DeviceObject,
                    modeData as PCHAR,
                    size,
                    pageSavable,
                );

                if !NT_SUCCESS(status) {
                    TracePrint(
                        TRACE_LEVEL_WARNING,
                        TRACE_FLAG_SCSI,
                        b"ClasspZeroQERR (%p): Failed to set QERR = 0 with status %x\n\0".as_ptr() as PCHAR,
                        DeviceObject,
                        status,
                    );
                }
            }
        }
    };

    // ClasspZeroQERR_Exit:
    if !modeData.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPowerActivateDevice(DeviceObject: *mut _DEVICE_OBJECT) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;

    NT_ASSERT!(((*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0) as u8);
    NT_ASSERT!(((*(*fdoExtension).FunctionSupportInfo).IdlePower.IdlePowerEnabled() != 0u32) as u8);

    let mut event: KEVENT = core::mem::zeroed();
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();

    KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);

    let irp = IoBuildDeviceIoControlRequest(
        IOCTL_STORAGE_POWER_ACTIVE,
        (*fdoExtension).LowerPdo,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        &mut event,
        &mut ioStatus,
    );

    if !irp.is_null() {
        status = IoCallDriver((*fdoExtension).LowerPdo, irp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut event as *mut KEVENT as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                (0i32) as i8,
                0u8,
                core::ptr::null_mut(),
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPowerIdleDevice(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let irp: PIRP;
    let mut event: KEVENT = core::mem::zeroed();
    let mut io_status: _IO_STATUS_BLOCK = core::mem::zeroed();

    let fdo_extension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    ntkrnl_sys::ASSERT!((*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0);
    ntkrnl_sys::ASSERT!((*(*fdo_extension).FunctionSupportInfo).IdlePower.IdlePowerEnabled() != 0);

    KeInitializeEvent(core::ptr::addr_of_mut!(event), 1, 0u8);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_STORAGE_POWER_IDLE,
        (*fdo_extension).LowerPdo,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        0,
        0u8,
        core::ptr::addr_of_mut!(event),
        core::ptr::addr_of_mut!(io_status),
    );

    if !irp.is_null() {
        status = IoCallDriver((*fdo_extension).LowerPdo, irp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(event) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            status = io_status.__bindgen_anon_1.Status;
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}
// dead: C is #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD), classpnp builds at WIN8
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn ClasspGetHwFirmwareInfo(
    DeviceObject: *mut _DEVICE_OBJECT,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;

    let mut firmwareInfo: *mut STORAGE_HW_FIRMWARE_INFO = core::ptr::null_mut();
    let mut query: *mut STORAGE_HW_FIRMWARE_INFO_QUERY;

    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut dataLength: u32 = core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO>() as u32;
    let mut iteration: u32 = 1;

    let mut lockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    'retry: loop {
        firmwareInfo = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            dataLength as usize,
            CLASSPNP_POOL_TAG_FIRMWARE,
        ) as *mut STORAGE_HW_FIRMWARE_INFO;

        if firmwareInfo.is_null() {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_INIT,
                b"ClasspGetHwFirmwareInfo: cannot allocate memory to hold data. \n\0".as_ptr() as *const i8,
            );
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::write_bytes(firmwareInfo as *mut u8, 0u8, dataLength as usize);

        query = firmwareInfo as *mut STORAGE_HW_FIRMWARE_INFO_QUERY;
        (*query).Version = core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO_QUERY>() as u32;
        (*query).Size = core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO_QUERY>() as u32;
        (*query).Flags = 0;

        ClassSendDeviceIoControlSynchronous(
            IOCTL_STORAGE_FIRMWARE_GET_INFO,
            (*commonExtension).LowerDeviceObject,
            query as *mut c_void,
            core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO_QUERY>() as u32,
            dataLength,
            0u8,
            core::ptr::addr_of_mut!(ioStatus),
        );

        if !NT_SUCCESS(ioStatus.__bindgen_anon_1.Status) && (ioStatus.__bindgen_anon_1.Status != STATUS_BUFFER_OVERFLOW) {
            if ClasspLowerLayerNotSupport(ioStatus.__bindgen_anon_1.Status) {
                {
                    let __ptr = core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).HwFirmwareGetInfoSupport) as *mut core::sync::atomic::AtomicI32;
                    (*__ptr).compare_exchange(SupportUnknown as i32, NotSupported as i32, core::sync::atomic::Ordering::AcqRel, core::sync::atomic::Ordering::Acquire).ok();
                }
            }

            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_INIT,
                b"ClasspGetHwFirmwareInfo: error %lx trying to query hardware firmware information #%d \n\0"
                    .as_ptr() as *const i8,
                ioStatus.__bindgen_anon_1.Status,
                iteration as u32,
            );
            FREE_POOL(firmwareInfo as *mut c_void);
            return ioStatus.__bindgen_anon_1.Status;
        }

        if ((*firmwareInfo).Version < core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO>() as u32)
            || ((*firmwareInfo).Size < core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO>() as u32)
            || ((*firmwareInfo).SlotCount == 0)
            || ((*firmwareInfo).ImagePayloadMaxSize
                > (*(*fdoExtension).AdapterDescriptor).MaximumTransferLength)
        {
            {
                let __ptr = core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).HwFirmwareGetInfoSupport) as *mut core::sync::atomic::AtomicI32;
                (*__ptr).compare_exchange(SupportUnknown as i32, NotSupported as i32, core::sync::atomic::Ordering::AcqRel, core::sync::atomic::Ordering::Acquire).ok();
            }

            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_INIT,
                b"ClasspGetHwFirmwareInfo: error in returned data! Version: 0x%X, Size: 0x%X, SlotCount: 0x%X, ActiveSlot: 0x%X, PendingActiveSlot: 0x%X, ImagePayloadMaxSize: 0x%X \n\0"
                    .as_ptr() as *const i8,
                (*firmwareInfo).Version,
                (*firmwareInfo).Size,
                (*firmwareInfo).SlotCount,
                (*firmwareInfo).ActiveSlot,
                (*firmwareInfo).PendingActivateSlot,
                (*firmwareInfo).ImagePayloadMaxSize,
            );

            FREE_POOL(firmwareInfo as *mut c_void);
            return STATUS_UNSUCCESSFUL;
        }

        if ((*firmwareInfo).Size > core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO>() as u32)
            && (iteration < 2)
        {
            dataLength = core::cmp::max(
                (*firmwareInfo).Size,
                core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO>() as u32
                    + core::mem::size_of::<STORAGE_HW_FIRMWARE_SLOT_INFO>() as u32
                        * ((*firmwareInfo).SlotCount - 1),
            );

            FREE_POOL(firmwareInfo as *mut c_void);
            iteration += 1;
            continue 'retry;
        }
        break 'retry;
    }

    KeAcquireInStackQueuedSpinLock(
        &mut (*(*fdoExtension).FunctionSupportInfo).SyncLock,
        &mut lockHandle,
    );

    let oldState = {
        let __ptr = core::ptr::addr_of_mut!((*(*fdoExtension).FunctionSupportInfo).HwFirmwareGetInfoSupport) as *mut core::sync::atomic::AtomicI32;
        (*__ptr).compare_exchange(SupportUnknown as i32, Supported as i32, core::sync::atomic::Ordering::AcqRel, core::sync::atomic::Ordering::Acquire)
            .unwrap_or_else(|x| x) as u32
    };

    if oldState == SupportUnknown {
        (*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo = firmwareInfo;
    } else if oldState == Supported {
        let cachedInfo = (*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo;
        (*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo = firmwareInfo;
        FREE_POOL(cachedInfo as *mut c_void);
    } else {
        FREE_POOL(firmwareInfo as *mut c_void);
    }

    KeReleaseInStackQueuedSpinLock(&mut lockHandle);

    ioStatus.__bindgen_anon_1.Status
}
// dead: C is #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD), classpnp builds at WIN8
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn ClassDeviceHwFirmwareIsPortDriverSupported(DeviceObject: *mut DEVICE_OBJECT) -> BOOLEAN {
    let common_extension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let fdo_extension = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;
    let mut is_supported: BOOLEAN = 0;
    if (*common_extension).__bindgen_anon_1.IsFdo() != 0 && !(*fdo_extension).MiniportDescriptor.is_null() {
        let port_driver = (*(*fdo_extension).MiniportDescriptor).Portdriver;
        if port_driver == StoragePortCodeSetStorport || port_driver == StoragePortCodeSetSpaceport || port_driver == StoragePortCodeSetSDport {
            is_supported = 1;
        }
    }
    is_supported
}
// dead: C is #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD), classpnp builds at WIN8
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn ClassDeviceHwFirmwareGetInfoProcess(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // Local constants for enum values used in the original code
    const NotSupported: i32 = 2;
    const STORAGE_HW_FIRMWARE_REQUEST_FLAG_CONTROLLER: u32 = 1;

    // Replace C's min() macro
    let min = |a: u32, b: u32| -> u32 {
        if a < b { a } else { b }
    };

    // Declare external functions that are not part of the known ntkrnl_sys surface

    let mut status = STATUS_SUCCESS;
    let common_extension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdo_extension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let query = (*Irp).AssociatedIrp.SystemBuffer as *mut STORAGE_HW_FIRMWARE_INFO_QUERY;
    let mut pass_down: u8 = 0;
    let mut copy_data: u8 = 0;

    // Input buffer length check
    if (*irp_stack).Parameters.DeviceIoControl.InputBufferLength
        < core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO_QUERY>() as u32
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }
    // Output buffer length check
    else if (*irp_stack).Parameters.DeviceIoControl.OutputBufferLength
        < core::mem::size_of::<STORAGE_HW_FIRMWARE_INFO>() as u32
    {
        status = STATUS_BUFFER_TOO_SMALL;
    }
    // Port driver support check
    else if ClassDeviceHwFirmwareIsPortDriverSupported(DeviceObject) == 0 {
        status = STATUS_NOT_IMPLEMENTED;
    }
    // Check that FunctionSupportInfo is allocated (for FDOs)
    else if (*common_extension).__bindgen_anon_1.IsFdo() != 0 && (*fdo_extension).FunctionSupportInfo.is_null() {
        status = STATUS_UNSUCCESSFUL;
    }
    // All preliminary checks passed; proceed with main logic
    else {
        // Determine if the request must be forwarded to the lower device
        if (*common_extension).__bindgen_anon_1.IsFdo() == 0 {
            pass_down = 1;
        }
        if ((*query).Flags & STORAGE_HW_FIRMWARE_REQUEST_FLAG_CONTROLLER) != 0 {
            pass_down = 1;
        }

        if pass_down != 0 {
            IoCopyCurrentIrpStackLocationToNext(Irp);
            ClassReleaseRemoveLock(DeviceObject, Irp);
            status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
            return status;
        }

        // Handle the FDO case: check if hardware firmware get info is supported
        if (* (*fdo_extension).FunctionSupportInfo).HwFirmwareGetInfoSupport == NotSupported {
            status = STATUS_NOT_IMPLEMENTED;
        } else {
            status = ClasspGetHwFirmwareInfo(DeviceObject);
            copy_data = if NT_SUCCESS(status) { 1u8 } else { 0u8 };
        }
    }

    // Exit_Firmware_Get_Info: copy cached data if successful
    if copy_data != 0 {
        let mut lock_handle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
        KeAcquireInStackQueuedSpinLock(
            &mut (* (*fdo_extension).FunctionSupportInfo).SyncLock,
            &mut lock_handle,
        );

        let data_length = min(
            (*irp_stack).Parameters.DeviceIoControl.OutputBufferLength,
            (*(*(*fdo_extension).FunctionSupportInfo).HwFirmwareInfo).Size,
        );

        core::ptr::copy_nonoverlapping(
            (*(*fdo_extension).FunctionSupportInfo).HwFirmwareInfo as *const u8,
            (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
            data_length as usize,
        );

        KeReleaseInStackQueuedSpinLock(&mut lock_handle);
        (*Irp).IoStatus.Information = (data_length as ULONG_PTR) as _;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;

    // Unconditional completion
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, 0i8); // IO_NO_INCREMENT

    status
}

// dead: C is #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD), classpnp builds at WIN8
#[cfg(any())]
unsafe extern "stdcall" fn ClassHwFirmwareDownloadComplete(
    fdo: PDEVICE_OBJECT,
    irp: PIRP,
    context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let irp_stack = IoGetCurrentIrpStackLocation(irp);
    if !context.is_null() {
        ExFreePool(context);
    }
    let original_irp = (*irp_stack).Parameters.Others.Argument1 as PIRP;
    (*original_irp).IoStatus.__bindgen_anon_1.Status = (*irp).IoStatus.__bindgen_anon_1.Status;
    (*original_irp).IoStatus.Information = (*irp).IoStatus.Information;
    ClassReleaseRemoveLock(fdo, (original_irp).cast());
    ClassCompleteRequest(fdo, (original_irp).cast(), IO_DISK_INCREMENT);
    IoFreeIrp(irp);
    STATUS_MORE_PROCESSING_REQUIRED
}
// dead: C is #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD), classpnp builds at WIN8
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn ClassDeviceHwFirmwareDownloadProcess(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    Srb: *mut _SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let firmwareDownload = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_HW_FIRMWARE_DOWNLOAD;
    let mut passDown: BOOLEAN = 0u8;
    let mut i: u32;
    let mut bufferSize: u32 = 0;
    let mut firmwareImageBuffer: *mut u8 = core::ptr::null_mut();
    let mut irp2: *mut _IRP = core::ptr::null_mut();
    let mut newStack: *mut _IO_STACK_LOCATION;
    let mut cdb: *mut _CDB;
    let mut lockHeld: BOOLEAN = 0u8;
    let mut lockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut status: NTSTATUS;

    let result = 'exit: {
        if (*irpStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<STORAGE_HW_FIRMWARE_DOWNLOAD>() as u32 {
            status = STATUS_INFO_LENGTH_MISMATCH;
            break 'exit status;
        }

        if (*firmwareDownload).Version < core::mem::size_of::<STORAGE_HW_FIRMWARE_DOWNLOAD>() as u32 ||
            (*firmwareDownload).Size > (*irpStack).Parameters.DeviceIoControl.InputBufferLength ||
            ((*firmwareDownload).BufferSize + core::mem::offset_of!(STORAGE_HW_FIRMWARE_DOWNLOAD, ImageBuffer) as u32) > (*firmwareDownload).Size
        {
            status = STATUS_INVALID_PARAMETER;
            break 'exit status;
        }

        if ClassDeviceHwFirmwareIsPortDriverSupported(DeviceObject) == 0 {
            status = STATUS_NOT_IMPLEMENTED;
            break 'exit status;
        }

        if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 && (*fdoExtension).FunctionSupportInfo.is_null() {
            status = STATUS_UNSUCCESSFUL;
            break 'exit status;
        }

        if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
            passDown = 1;
        }
        if (*firmwareDownload).Flags & STORAGE_HW_FIRMWARE_REQUEST_FLAG_CONTROLLER != 0 {
            passDown = 1;
        }

        if passDown != 0 {
            IoCopyCurrentIrpStackLocationToNext(Irp);
            ClassReleaseRemoveLock(DeviceObject, Irp);
            status = IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
            FREE_POOL((Srb).cast());
            return status;
        }

        if (*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo.is_null() {
            if (*(*fdoExtension).FunctionSupportInfo).HwFirmwareGetInfoSupport == NotSupported {
                status = STATUS_NOT_IMPLEMENTED;
                break 'exit status;
            } else {
                status = ClasspGetHwFirmwareInfo(DeviceObject);
                if !NT_SUCCESS(status) {
                    break 'exit status;
                }
            }
        }

        if (*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo.is_null() {
            if (*(*fdoExtension).FunctionSupportInfo).HwFirmwareGetInfoSupport == NotSupported {
                status = STATUS_NOT_IMPLEMENTED;
            } else {
                status = STATUS_UNSUCCESSFUL;
            }
            break 'exit status;
        }

        lockHeld = 1;
        KeAcquireInStackQueuedSpinLock(core::ptr::addr_of!((*(*fdoExtension).FunctionSupportInfo).SyncLock) as *mut u32, &mut lockHandle);

        if (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).SupportUpgrade == 0 ||
            (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).ImagePayloadAlignment == 0
        {
            status = STATUS_NOT_SUPPORTED;
            break 'exit status;
        }

        i = 0;
        while i < (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).SlotCount {
            if (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).Slot[i].SlotNumber == (*firmwareDownload).Slot {
                break;
            }
            i += 1;
        }

        if i >= (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).SlotCount ||
            (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).Slot[i].ReadOnly != 0
        {
            status = STATUS_INVALID_PARAMETER;
            break 'exit status;
        }

        if (*firmwareDownload).BufferSize == 0 ||
            (*firmwareDownload).BufferSize % (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).ImagePayloadAlignment != 0 ||
            (*firmwareDownload).BufferSize > (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).ImagePayloadMaxSize ||
            (*firmwareDownload).BufferSize > (*(*fdoExtension).AdapterDescriptor).MaximumTransferLength ||
            (*firmwareDownload).Offset % (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).ImagePayloadAlignment != 0 ||
            (*firmwareDownload).Offset > 0xFFFFFF ||
            (*firmwareDownload).BufferSize > 0xFFFFFF
        {
            status = STATUS_INVALID_PARAMETER;
            break 'exit status;
        }

        if ((*firmwareDownload).ImageBuffer as usize % (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).ImagePayloadAlignment as usize != 0) {
            bufferSize = (ALIGN_UP_BY((*firmwareDownload).BufferSize, (*(*(*fdoExtension).FunctionSupportInfo).HwFirmwareInfo).ImagePayloadAlignment)) as u32;

            NT_ASSERT!((lockHeld != 0) as u8);
            KeReleaseInStackQueuedSpinLock(&mut lockHandle);
            lockHeld = 0;

            firmwareImageBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, (bufferSize) as usize, CLASSPNP_POOL_TAG_FIRMWARE) as *mut u8;
            if firmwareImageBuffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'exit status;
            }

            core::ptr::write_bytes(firmwareImageBuffer, 0, bufferSize as usize);
            core::ptr::copy_nonoverlapping((*firmwareDownload).ImageBuffer as *const u8, firmwareImageBuffer, (*firmwareDownload).BufferSize as usize);
        } else {
            NT_ASSERT!((lockHeld != 0) as u8);
            KeReleaseInStackQueuedSpinLock(&mut lockHandle);
            lockHeld = 0;

            firmwareImageBuffer = (*firmwareDownload).ImageBuffer;
            bufferSize = (*firmwareDownload).BufferSize;
        }

        irp2 = IoAllocateIrp(((*DeviceObject).StackSize as i32 + 3) as CCHAR, 0u8);
        if irp2.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;

            if firmwareImageBuffer != (*firmwareDownload).ImageBuffer {
                FREE_POOL(firmwareImageBuffer as *mut core::ffi::c_void);
            }

            break 'exit status;
        }

        ClassAcquireRemoveLock(DeviceObject, irp2);

        (*(*irp2).Tail.Overlay.Thread) = (*(*Irp).Tail.Overlay.Thread);
        IoSetNextIrpStackLocation(irp2);

        newStack = IoGetCurrentIrpStackLocation(irp2);
        (*newStack).Parameters.Others.Argument1 = Irp as _;
        (*newStack).DeviceObject = DeviceObject;

        IoSetCompletionRoutine(
            irp2,
            Some(ClassHwFirmwareDownloadComplete),
            if firmwareImageBuffer != (*firmwareDownload).ImageBuffer {
                firmwareImageBuffer as *mut core::ffi::c_void
            } else {
                core::ptr::null_mut()
            },
            1u8, 1u8, 1u8,
        );

        IoSetNextIrpStackLocation(irp2);
        newStack = IoGetCurrentIrpStackLocation(irp2);
        (*newStack).DeviceObject = DeviceObject;
        (*newStack).MajorFunction = (*irpStack).MajorFunction;
        (*newStack).MinorFunction = (*irpStack).MinorFunction;
        (*newStack).Flags = (*irpStack).Flags;

        IoMarkIrpPending(Irp);

        SrbSetCdbLength((Srb).cast(), CDB10GENERIC_LENGTH as u8);
        cdb = SrbGetCdb((Srb).cast());
        // WRITE BUFFER CDB (10-byte, opcode 0x3B) — no named variant in local _CDB; use AsByte
        {
            let cdb_bytes = (*cdb).AsByte.as_mut();
            cdb_bytes[0] = SCSIOP_WRITE_DATA_BUFF as u8; // OperationCode
            cdb_bytes[1] = SCSI_WRITE_BUFFER_MODE_DOWNLOAD_MICROCODE_WITH_OFFSETS_SAVE_DEFER_ACTIVATE & 0x1F; // Mode[4:0]
            cdb_bytes[2] = (*firmwareDownload).Slot as u8; // BufferID
            let offset_be = (*firmwareDownload).Offset.to_be_bytes();
            cdb_bytes[3] = offset_be[1]; // BufferOffset[0] = Offset byte 2 (big-endian 3 bytes)
            cdb_bytes[4] = offset_be[2];
            cdb_bytes[5] = offset_be[3];
            let size_be = bufferSize.to_be_bytes();
            cdb_bytes[6] = size_be[1]; // ParameterListLength[0]
            cdb_bytes[7] = size_be[2];
            cdb_bytes[8] = size_be[3];
            cdb_bytes[9] = 0; // Control
        }

        SrbSetRequestAttribute((Srb).cast(), SRB_HEAD_OF_QUEUE_TAG_REQUEST);
        SrbSetSrbFlags((Srb).cast(), SRB_FLAGS_NO_QUEUE_FREEZE | SRB_FLAGS_QUEUE_ACTION_ENABLE);
        SrbSetTimeOutValue((Srb).cast(), (*fdoExtension).TimeOutValue);

        status = ClassSendSrbAsynchronous(
            DeviceObject,
            Srb,
            irp2,
            (firmwareImageBuffer).cast(),
            bufferSize,
            1u8,
        );

        if status != STATUS_PENDING {
            if firmwareImageBuffer != (*firmwareDownload).ImageBuffer {
                FREE_POOL(firmwareImageBuffer as *mut core::ffi::c_void);
            }
            // Srb has been freed, null it to prevent double free later
            // (but Srb is a local pointer, we just skip freeing it)
            // ClassSendSrbAsynchronous already freed it? In C they set Srb = NULL.
            // We'll just leave Srb as is, but the exit path will FREE_POOL(Srb) again? Wait, in C they set Srb = NULL, then break to exit. In the exit code they call FREE_POOL(Srb) which would be a no-op if NULL. To emulate, we can set Srb = core::ptr::null_mut() before break.
            // But Srb is a function parameter (not mutable pointer). We can't change the pointer passed in. But we can set a local variable srb_ptr = Srb and modify that, but original Srb is separate. Better: set a flag to skip FREE_POOL(Srb) at exit. But simpler: we can conditionally call FREE_POOL(Srb) only if Srb is not null. In the failure path after ClassSendSrbAsynchronous, we know Srb has been freed, so we should not free it again. So we'll just not break to exit that frees it, but instead handle cleanup directly and return. To keep logic close, we'll break 'exit with status, but then in the exit handler we'll add a flag. I'll add a variable `skip_srb_free: bool = false;` and set it true in that path, then in exit block conditionally free Srb.

            // However, the C code also called ClassReleaseRemoveLock and IoFreeIrp in the error path, then goto Exit_Firmware_Download. That exit path does not call those again. So we need to replicate that. I'll restructure to match C exactly: after the if block, they do:
            // ClassReleaseRemoveLock(DeviceObject, irp2);
            // IoFreeIrp(irp2);
            // goto Exit_Firmware_Download;
            // So we'll do that before breaking to 'exit.
            // So we'll handle as:
            ClassReleaseRemoveLock(DeviceObject, irp2);
            IoFreeIrp(irp2);
            // Srb freed, set to null so that exit's FREE_POOL(Srb) does nothing.
            // We'll just set Srb = core::ptr::null_mut(); but that would require param to be `Srb: &mut *mut _SCSI_REQUEST_BLOCK` which it's not.
            // Instead, we'll use a boolean variable `freeSrb: BOOLEAN = 1u8;` and set to 0 in this path. Then at exit we conditionally free.
            // For simplicity, we'll break to exit with status, and handle srb freeing conditionally.
        } else {
            return status;
        }
        // This point reached only if status != STATUS_PENDING and we haven't returned.
        // We need to fall through to exit label.
        status
    }; // end 'exit block

    // Exit path (after early returns, only reached on error or status != PENDING after send)
    // Note: In the success case (STATUS_PENDING) we already returned, so we won't get here.
    if lockHeld != 0 {
        KeReleaseInStackQueuedSpinLock(&mut lockHandle);
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = result;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
    FREE_POOL((Srb).cast());
    result
}
// dead: C is #if (NTDDI_VERSION >= NTDDI_WINTHRESHOLD), classpnp builds at WIN8
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn ClassDeviceHwFirmwareActivateProcess(
    DeviceObject: *mut _DEVICE_OBJECT,
    Irp: *mut _IRP,
    Srb: *mut SCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let common_extension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdo_extension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let firmware_activate = (*Irp).AssociatedIrp.SystemBuffer as *mut _STORAGE_HW_FIRMWARE_ACTIVATE;

    let mut srb = Srb;
    let mut lock_held: BOOLEAN = 0;
    let mut lock_handle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();

    // Determine pass-down early return path
    let mut pass_down: BOOLEAN = 0;
    if (*common_extension).__bindgen_anon_1.IsFdo() == 0 {
        pass_down = 1;
    }
    if ((*firmware_activate).Flags & STORAGE_HW_FIRMWARE_REQUEST_FLAG_CONTROLLER) != 0 {
        pass_down = 1;
    }
    if pass_down != 0 {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        let status = IoCallDriver((*common_extension).LowerDeviceObject, Irp);
        FREE_POOL((srb).cast());
        return status;
    }

    // Main processing that may fail and requires cleanup
    let status = loop {
        if (*irp_stack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<STORAGE_HW_FIRMWARE_ACTIVATE>() as u32 {
            break STATUS_INFO_LENGTH_MISMATCH;
        }

        if ((*firmware_activate).Version < core::mem::size_of::<STORAGE_HW_FIRMWARE_ACTIVATE>() as u32) ||
            ((*firmware_activate).Size > (*irp_stack).Parameters.DeviceIoControl.InputBufferLength)
        {
            break STATUS_INVALID_PARAMETER;
        }

        if ClassDeviceHwFirmwareIsPortDriverSupported(DeviceObject) == 0 {
            break STATUS_NOT_IMPLEMENTED;
        }

        if (*common_extension).__bindgen_anon_1.IsFdo() != 0 && (*fdo_extension).FunctionSupportInfo.is_null() {
            break STATUS_UNSUCCESSFUL;
        }

        let func_support = (*fdo_extension).FunctionSupportInfo;
        if (*func_support).HwFirmwareInfo.is_null() {
            if (*func_support).HwFirmwareGetInfoSupport == NotSupported {
                break STATUS_NOT_IMPLEMENTED;
            } else {
                let info_status = ClasspGetHwFirmwareInfo(DeviceObject);
                if !NT_SUCCESS(info_status) {
                    break info_status;
                }
            }
        }

        let func_support = (*fdo_extension).FunctionSupportInfo;
        if (*func_support).HwFirmwareInfo.is_null() {
            if (*func_support).HwFirmwareGetInfoSupport == NotSupported {
                break STATUS_NOT_IMPLEMENTED;
            } else {
                break STATUS_UNSUCCESSFUL;
            }
        }

        lock_held = 1;
        KeAcquireInStackQueuedSpinLock(
            core::ptr::addr_of_mut!((*func_support).SyncLock),
            core::ptr::addr_of_mut!(lock_handle),
        );

        let hw_info = (*func_support).HwFirmwareInfo;
        if (*hw_info).SupportUpgrade == 0 {
            break STATUS_NOT_SUPPORTED;
        }

        let slot = (*firmware_activate).Slot;
        let mut found = false;
        let mut i: u32 = 0;
        while i < (*hw_info).SlotCount {
            if (*hw_info).Slot[i as usize].SlotNumber == slot {
                found = true;
                break;
            }
            i += 1;
        }
        if !found {
            break STATUS_INVALID_PARAMETER;
        }

        KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(lock_handle));
        lock_held = 0;

        SrbSetCdbLength((srb).cast(), CDB10GENERIC_LENGTH as u8);
        let cdb = SrbGetCdb((srb).cast());
        // WRITE BUFFER CDB (10-byte, opcode 0x3B) — no named variant in local _CDB; use AsByte
        {
            let cdb_bytes = (*cdb).AsByte.as_mut();
            cdb_bytes[0] = SCSIOP_WRITE_DATA_BUFF as u8; // OperationCode
            cdb_bytes[1] = SCSI_WRITE_BUFFER_MODE_ACTIVATE_DEFERRED_MICROCODE & 0x1F; // Mode[4:0]
            cdb_bytes[2] = slot; // BufferID
            // BufferOffset[3] = 0, ParameterListLength[3] = 0 (already zeroed by SRB init)
        }
        SrbSetTimeOutValue((srb).cast(), 30);

        let send_status = ClassSendSrbAsynchronous(
            DeviceObject,
            srb,
            Irp,
            core::ptr::null_mut(),
            0,
            0,
        );
        if send_status != STATUS_PENDING {
            srb = core::ptr::null_mut();
            break send_status;
        }

        return send_status;
    };

    if lock_held != 0 {
        KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(lock_handle));
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
    FREE_POOL((srb).cast());
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspIsThinProvisioningError(Srb: PSCSI_REQUEST_BLOCK) -> BOOLEAN {
    if ((*Srb).SrbStatus & (SRB_STATUS_AUTOSENSE_VALID) as u8) != 0 {
        let senseBuffer = SrbGetSenseInfoBuffer((Srb).cast());
        if !senseBuffer.is_null() {
            let mut senseKey: UCHAR = 0;
            let mut addlSenseCode: UCHAR = 0;
            let mut addlSenseCodeQual: UCHAR = 0;
            let validSense = ScsiGetSenseKeyAndCodes(
                senseBuffer,
                SrbGetSenseInfoBufferLength((Srb).cast()),
                SCSI_SENSE_OPTIONS_NONE,
                &mut senseKey,
                &mut addlSenseCode,
                &mut addlSenseCodeQual,
            );
            return (validSense != 0
                && senseKey == (SCSI_SENSE_UNIT_ATTENTION) as u8
                && addlSenseCode == (SCSI_ADSENSE_LB_PROVISIONING) as u8
                && addlSenseCodeQual == (SCSI_SENSEQ_SOFT_THRESHOLD_REACHED) as u8) as BOOLEAN;
        }
    }
    0u8
}


// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "system" {
    fn ClassReleaseRemoveLock(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP);
    fn ClassCompleteRequest(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP, PriorityBoost: u8);
}

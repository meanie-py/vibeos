use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_MEDIA_INFO, DISK_GEOMETRY, DISK_GEOMETRY_EX, DRIVE_LAYOUT_INFORMATION, DRIVE_LAYOUT_INFORMATION_EX, GET_LENGTH_INFORMATION, GET_MEDIA_TYPES, IO_ERROR_LOG_PACKET, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, KeDelayExecutionThread, LARGE_INTEGER, LPCSTR, MOUNTDEV_NAME, PARTITION_INFORMATION, PARTITION_INFORMATION_EX, PDEVICE_MEDIA_INFO, PDISK_GEOMETRY, PDISK_GEOMETRY_EX, PDRIVE_LAYOUT_INFORMATION, PDRIVE_LAYOUT_INFORMATION_EX, PERSISTENT_RESERVE_COMMAND, PGET_LENGTH_INFORMATION, PGET_MEDIA_TYPES, PIO_ERROR_LOG_PACKET, PMOUNTDEV_NAME, PPARTITION_INFORMATION, PPARTITION_INFORMATION_EX, PPERSISTENT_RESERVE_COMMAND, PPREVENT_MEDIA_REMOVAL, PREVENT_MEDIA_REMOVAL, PSTORAGE_ADAPTER_DESCRIPTOR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_DEVICE_NUMBER, PSTORAGE_EVENT_NOTIFICATION, PSTORAGE_HOTPLUG_INFO, PSTORAGE_PROPERTY_QUERY, PSTORAGE_READ_CAPACITY, PSTORAGE_WRITE_CACHE_PROPERTY, PVERIFY_INFORMATION, RTL_QUERY_REGISTRY_TABLE, RtlDeleteRegistryValue, RtlQueryRegistryValues, STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, STORAGE_DEVICE_NUMBER, STORAGE_EVENT_NOTIFICATION, STORAGE_HOTPLUG_INFO, STORAGE_READ_CAPACITY, STORAGE_WRITE_CACHE_PROPERTY};
// IoIs32bitProcess is missing from the i386/arm64 ntkrnl-sys bindgen `ddk`
// module (present only on amd64); declare it locally as the real ntoskrnl
// export. PIRP/BOOLEAN are in scope via the ddk imports above.
extern "system" { fn IoIs32bitProcess(Irp: PIRP) -> BOOLEAN; }
// bindgen enum-variant constants (prefixed in ddk) used as plain names here:
use ntkrnl_sys::ddk::_STORAGE_MEDIA_TYPE_CD_ROM as CD_ROM;
use ntkrnl_sys::ddk::_PARTITION_STYLE_PARTITION_STYLE_MBR as PARTITION_STYLE_MBR;
use ntkrnl_sys::ddk::_STORAGE_QUERY_TYPE_PropertyExistsQuery as PropertyExistsQuery;
use ntkrnl_sys::ddk::_STORAGE_QUERY_TYPE_PropertyStandardQuery as PropertyStandardQuery;
use ntkrnl_sys::ddk::_MEDIA_TYPE_RemovableMedia as RemovableMedia;
use ntkrnl_sys::ddk::_DEVICE_REMOVAL_POLICY_RemovalPolicyExpectOrderlyRemoval as RemovalPolicyExpectOrderlyRemoval;
use ntkrnl_sys::ddk::_DEVICE_REMOVAL_POLICY_RemovalPolicyExpectSurpriseRemoval as RemovalPolicyExpectSurpriseRemoval;
use ntkrnl_sys::ddk::_WRITE_CACHE_CHANGE_WriteCacheChangeUnknown as WriteCacheChangeUnknown;
use ntkrnl_sys::ddk::_WRITE_CACHE_CHANGE_WriteCacheChangeable as WriteCacheChangeable;
use ntkrnl_sys::ddk::_WRITE_CACHE_ENABLE_WriteCacheDisabled as WriteCacheDisabled;
use ntkrnl_sys::ddk::_WRITE_CACHE_ENABLE_WriteCacheEnableUnknown as WriteCacheEnableUnknown;
use ntkrnl_sys::ddk::_WRITE_CACHE_ENABLE_WriteCacheEnabled as WriteCacheEnabled;
use ntkrnl_sys::ddk::_WRITE_CACHE_CHANGE_WriteCacheNotChangeable as WriteCacheNotChangeable;
use ntkrnl_sys::ddk::_WRITE_CACHE_TYPE_WriteCacheTypeUnknown as WriteCacheTypeUnknown;
use ntkrnl_sys::ddk::_WRITE_CACHE_TYPE_WriteCacheTypeWriteBack as WriteCacheTypeWriteBack;
use ntkrnl_sys::ddk::_WRITE_THROUGH_WriteThroughNotSupported as WriteThroughNotSupported;
use ntkrnl_sys::ddk::_WRITE_THROUGH_WriteThroughSupported as WriteThroughSupported;
// DVD_KEY_TYPE enum variants (used as bare names in match arms / comparisons):
use crate::types::_DVD_KEY_TYPE_DvdChallengeKey as DvdChallengeKey;
use crate::types::_DVD_KEY_TYPE_DvdBusKey1 as DvdBusKey1;
use crate::types::_DVD_KEY_TYPE_DvdBusKey2 as DvdBusKey2;
use crate::types::_DVD_KEY_TYPE_DvdTitleKey as DvdTitleKey;
use crate::types::_DVD_KEY_TYPE_DvdAsf as DvdAsf;
use crate::types::_DVD_KEY_TYPE_DvdSetRpcKey as DvdSetRpcKey;
use crate::types::_DVD_KEY_TYPE_DvdGetRpcKey as DvdGetRpcKey;
use crate::types::_DVD_KEY_TYPE_DvdDiskKey as DvdDiskKey;
use crate::types::_DVD_KEY_TYPE_DvdInvalidateAGID as DvdInvalidateAGID;
// bindgen enum-variant aliases used as bare names:
use crate::types::DVD_STRUCTURE_FORMAT_DvdMaxDescriptor as DvdMaxDescriptor;
use crate::types::_CDROM_OPC_INFO_TYPE_SimpleOpcInfo as SimpleOpcInfo;
use crate::types::_CDROM_PERFORMANCE_REQUEST_TYPE_CdromWriteSpeedRequest as CdromWriteSpeedRequest;
use crate::types::_CDROM_PERFORMANCE_REQUEST_TYPE_CdromPerformanceRequest as CdromPerformanceRequest;
use crate::types::_CDROM_PERFORMANCE_TYPE_CdromReadPerformance as CdromReadPerformance;
use crate::types::_CDROM_PERFORMANCE_TYPE_CdromWritePerformance as CdromWritePerformance;
use crate::types::_CDROM_PERFORMANCE_EXCEPTION_TYPE_CdromNominalPerformance as CdromNominalPerformance;
use crate::types::_CDROM_PERFORMANCE_EXCEPTION_TYPE_CdromEntirePerformanceList as CdromEntirePerformanceList;
use crate::types::_CDROM_PERFORMANCE_EXCEPTION_TYPE_CdromPerformanceExceptionsOnly as CdromPerformanceExceptionsOnly;
use crate::types::_CDROM_PERFORMANCE_TOLERANCE_TYPE_Cdrom10Nominal20Exceptions as Cdrom10Nominal20Exceptions;
// type aliases
use crate::types::_CDB__GET_PERFORMANCE as GetPerformanceCdb;
type size_t = usize;
use ntkrnl_sys::ddk::{STORAGE_PROPERTY_QUERY, VERIFY_INFORMATION, STORAGE_ADAPTER_DESCRIPTOR, _UNICODE_STRING};
// DVD key/structure lengths (from ntddcdvd.h)
const DVD_CHALLENGE_KEY_LENGTH: u32 = 16;
const DVD_BUS_KEY_LENGTH: u32 = 12;
const DVD_TITLE_KEY_LENGTH: u32 = 8;
const DVD_ASF_LENGTH: u32 = 8;
const DVD_END_ALL_SESSIONS: u8 = 0xFF;
// IOCTLs (CTL_CODE-computed values from ntdddisk.h / ntddcdrm.h)
const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x00074004;
const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x00070048;
const IOCTL_DISK_GET_DRIVE_LAYOUT: u32 = 0x0007400C;
const IOCTL_DISK_GET_DRIVE_LAYOUT_EX: u32 = 0x00070050;
const IOCTL_CDROM_GET_VOLUME: u32 = 0x00024014;
// misc pool tags / SCSI constants
const CDROM_TAG_MODE_DATA: u32 = 0x4D436353; // 'CROM'
const SP_UNTAGGED: u8 = 0xFF;
const SCSI_REQUEST_BLOCK_SIZE: usize = core::mem::size_of::<SCSI_REQUEST_BLOCK>();
const STATUS_INVALID_LEVEL: NTSTATUS = 0xC0000148u32 as i32;
use core::ffi::c_void;
use core::mem::offset_of;
use ntkrnl_sys::ddk::RtlAssert;

extern "C" {
    fn CdromDbgPrintForce2(format: *const i8, arg1: usize, arg2: usize) -> u32;
    fn CdromDbgPrint3(format: *const i8, arg1: usize, arg2: usize, arg3: usize) -> u32;
    fn NT_SUCCESS(status: NTSTATUS) -> bool;
    fn PerformEjectionControl(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: *mut core::ffi::c_void, LockType: i32, PreventMediaRemoval: u8) -> NTSTATUS;
    fn CdromDbgPrintReadWriteValidate(
        format: *const i8,
        request_type: usize,
        starting_offset: u64,
        transfer_byte_count: usize,
        bytes_per_sector: usize,
        partition_length: u64,
        validation_schema: usize,
        is_valid: usize,
        status: usize,
    ) -> u32;
    fn CdromGetReadWriteFields(
        request: WDFREQUEST,
        request_type: *mut ULONG,
        device_offset: *mut i64,
        transfer_length: *mut ULONG_PTR,
    );
}
extern "system" {
    static CLASSP_REG_SUBKEY_NAME: u16;
    static CLASSP_REG_REMOVAL_POLICY_VALUE_NAME: u16;
    fn RequestGetContext(request: WDFREQUEST) -> *mut CDROM_REQUEST_CONTEXT;
    fn ScratchBuffer_BeginUse(device_extension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_PerformNextReadWrite(device_extension: *mut CDROM_DEVICE_EXTENSION, first: u32) -> NTSTATUS;
    fn ScratchBuffer_EndUse(DeviceExtension: PCDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_ExecuteCdb(DeviceExtension: PCDROM_DEVICE_EXTENSION, OriginalRequest: *mut core::ffi::c_void, transferByteCount: u32, flag: u8, cdb: *mut CDB, cdbLen: u8) -> NTSTATUS;
    fn NT_ASSERT(condition: i32);
    fn DeviceGetExtension(Device: WDFDEVICE) -> *mut CDROM_DEVICE_EXTENSION;
    fn MmGetMdlVirtualAddress(Mdl: PMDL) -> *mut core::ffi::c_void;
    fn MmGetMdlByteCount(Mdl: PMDL) -> u32;
}
unsafe extern "C" {
    fn REVERSE_BYTES(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void);
    fn PLAY_ACTIVE(DeviceExtension: PCDROM_DEVICE_EXTENSION) -> *mut i32;
    fn REVERSE_SHORT(p: *mut u16);
    fn REVERSE_LONG(p: *mut u32);
    fn RtlULongAdd(a: u32, b: u32, result: *mut u32) -> i32;
    fn RtlULongSub(a: u32, b: u32, result: *mut u32) -> i32;
    fn MSF_TO_LBA(m: u8, s: u8, f: u8) -> u32;
}
unsafe extern "C" {
    fn PAGED_CODE();
}
unsafe extern "C" {
    fn MediaReadCapacity(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS;
}
unsafe extern "C" {
    fn RequestCompletion( device_extension: *mut CDROM_DEVICE_EXTENSION, request: WDFREQUEST, status: NTSTATUS, info: ULONG_PTR, );
}
unsafe extern "C" {
    fn UNREFERENCED_PARAMETER(param: usize);
}
extern "system" {
    fn RequestDummyCompletionRoutine( Request: WDFREQUEST, ioTarget: WDFIOTARGET, CompletionParams: *mut WDF_REQUEST_COMPLETION_PARAMS, CompletionContext: WDFCONTEXT, );
    fn DeviceSendSrbSynchronously(Device: *mut core::ffi::c_void, Srb: *mut SCSI_REQUEST_BLOCK, DataBuffer: *mut core::ffi::c_void, DataTransferLength: u32, AutoSense: u8, Request: *mut core::ffi::c_void) -> i32;
    fn IoWriteErrorLogEntry(ElEntry: *mut IO_ERROR_LOG_PACKET);
    fn IoAllocateErrorLogEntry(IoObject: *mut core::ffi::c_void, EntrySize: u32) -> *mut IO_ERROR_LOG_PACKET;
    fn WDF_REQUEST_PARAMETERS_INIT(Parameters: *mut WDF_REQUEST_PARAMETERS);
    fn RtlZeroMemory(Destination: *mut core::ffi::c_void, Length: usize);
    fn KeGetCurrentIrql() -> u8;
    fn ExFreePool(Pool: *mut core::ffi::c_void);
    fn ScratchBuffer_ExecuteCdbEx(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: *mut core::ffi::c_void, transferSize: u32, bFlag: u8, Cdb: *const CDB, CdbLength: u8, TimeoutValue: u32) -> NTSTATUS;
    fn DeviceSetParameter( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, SubkeyName: *const u16, ValueName: *const u16, Value: u32, ) -> NTSTATUS;
    fn FileObjectGetContext(FileObject: WDFFILEOBJECT) -> PFILE_OBJECT_CONTEXT;
    fn TEST_FLAG(flags: u32, bit: u32) -> bool;
    fn RequestSetupMcnRequest(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, supported: u8) -> i32;
    fn RequestGetCurrentStackLocationFlags(Request: WDFREQUEST) -> u8;
    fn ScratchBuffer_ResetItems(device_extension: *mut CDROM_DEVICE_EXTENSION, free_mdl: u8);
    fn DevicePickDvdRegion(Device: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn ValidChar(c: u8) -> i32;
    fn CompareStorageDuids( Duid1: *const STORAGE_DEVICE_UNIQUE_IDENTIFIER, Duid2: *const STORAGE_DEVICE_UNIQUE_IDENTIFIER, ) -> i32;
    fn TimeOutValueGetCapValue(timeout: u32, multiplier: u32) -> u32;
    fn InterlockedCompareExchange(Destination: *mut i32, Exchange: i32, Comperand: i32) -> i32;
    fn REVERSE_BYTES_SHORT(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void);
    fn Int64ShrlMod32(value: i64, shift: u32) -> i64;
}
unsafe extern "C" {
    fn RequestSend( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, ioTarget: WDFIOTARGET, sendOptionsFlags: ULONG, requestSent: *mut BOOLEAN, ) -> NTSTATUS;
}
unsafe extern "C" {
    fn EXCLUSIVE_MODE(cdData: *mut CDROM_DATA) -> u8;
}
unsafe extern "C" {
    fn EXCLUSIVE_OWNER(cdData: *mut CDROM_DATA, FileObject: *mut core::ffi::c_void) -> u8;
}
unsafe extern "C" {
    fn DeviceSendNotification(deviceExtension: PCDROM_DEVICE_EXTENSION, guid: *const GUID, param1: u32, param2: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn IsVolumeMounted(DeviceObject: *mut core::ffi::c_void) -> u8;
}
unsafe extern "C" {
    fn DeviceUnlockExclusive( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, FileObject: WDFFILEOBJECT, NoMediaNotifications: i32, ) -> NTSTATUS;
}
unsafe extern "C" {
    fn RequestDuidGetDeviceIdProperty( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, requestParameters: WDF_REQUEST_PARAMETERS, outLength: *mut usize, ) -> i32;
}
unsafe extern "C" {
    fn RequestDuidGetDeviceProperty( deviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: WDFREQUEST, requestParameters: WDF_REQUEST_PARAMETERS, outLength: *mut usize, ) -> i32;
}
unsafe extern "C" {
    #[link_name = "DeviceRetrieveModeSenseUsingScratch"]
    fn DeviceRetrieveModeSenseUsingScratch__c(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Buffer: *mut i8, BufferLength: usize, Page: u8, PageControl: u8) -> u32;
}

// common.rs (and the C) return ULONG; a usize decl read 32 bits nobody wrote on 64-bit.
#[allow(non_snake_case)]
unsafe fn DeviceRetrieveModeSenseUsingScratch(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Buffer: *mut i8, BufferLength: usize, Page: u8, PageControl: u8) -> usize {
    DeviceRetrieveModeSenseUsingScratch__c(DeviceExtension, Buffer, BufferLength, Page, PageControl) as usize
}
unsafe extern "C" {
    fn ModeSenseFindSpecificPage(Data: *mut i8, DataLength: usize, PageCode: u8, ReadWrite: i32) -> *mut MODE_CACHING_PAGE;
}
unsafe extern "C" {
    fn FREE_POOL(Pool: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn RequestSetupMcnSyncIrp(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
}
unsafe extern "C" {
    fn RequestSendMcnRequest( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, ) -> u8;
}
unsafe extern "C" {
    fn RequestPostWorkMcnRequest( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, ) -> u8;
}
unsafe extern "C" {
    fn DeviceSendDelayedMediaChangeNotifications( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, );
}
unsafe extern "C" {
    fn RequestSenseInfoInterpretForScratchBuffer( device_extension: *mut CDROM_DEVICE_EXTENSION, times_already_retried: u32, status: *mut i32, retry_in_100ns_units: *mut i64, ) -> i32;
}
unsafe extern "C" {
    fn ScratchBuffer_SetupSrb(device_extension: *mut CDROM_DEVICE_EXTENSION, request: *mut core::ffi::c_void, transfer_byte_count: u32, use_scratch: u8);
}
unsafe extern "C" {
    fn ScratchBuffer_SendSrb(device_extension: *mut CDROM_DEVICE_EXTENSION, wait: u8, SrbHistoryItem: *mut *mut SRB_HISTORY_ITEM) -> i32;
}
unsafe extern "C" {
    fn DeviceMarkActive(DeviceExtension: PCDROM_DEVICE_EXTENSION, b1: u8, b2: u8);
}

// ---- file-scope preamble (translated) ----
pub const FirstDriveLetter: char = 'C';
pub const LastDriveLetter: char = 'Z';

// READ_DVD_STRUCTURE_FORMAT_STRINGS[DvdMaxDescriptor+1]
pub const READ_DVD_STRUCTURE_FORMAT_STRINGS: [LPCSTR; 6] = [
    (b"Physical\0").as_ptr() as LPCSTR,
    (b"Copyright\0").as_ptr() as LPCSTR,
    (b"DiskKey\0").as_ptr() as LPCSTR,
    (b"BCA\0").as_ptr() as LPCSTR,
    (b"Manufacturer\0").as_ptr() as LPCSTR,
    (b"Unknown\0").as_ptr() as LPCSTR,
];

const WDF_REQUEST_SEND_OPTION_SYNCHRONOUS: ULONG = 2;

const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const MAXIMUM_RETRIES: u8 = 0x4;  // was 5 (LLM-invented); bindgen-correct
const RAW_SECTOR_SIZE: u32 = 2352;
const CD_RAW_SECTOR_WITH_SUBCODE_SIZE: u32 = 2448;
const CD_RAW_SECTOR_WITH_C2_SIZE: u32 = 0xa58;  // was 2960 (LLM-invented); bindgen-correct
const CD_RAW_SECTOR_WITH_C2_AND_SUBCODE_SIZE: u32 = 0xab8;  // was 3056 (LLM-invented); bindgen-correct
const COOKED_SECTOR_SIZE: u32 = 2048;
// TRACK_MODE_TYPE enum values (from crate::types bindgen enum)
const CDDA: u32 = _TRACK_MODE_TYPE_CDDA as u32;
const YellowMode2: u32 = _TRACK_MODE_TYPE_YellowMode2 as u32;
const XAForm2: u32 = _TRACK_MODE_TYPE_XAForm2 as u32;
const RawWithSubCode: u32 = _TRACK_MODE_TYPE_RawWithSubCode as u32;
const RawWithC2: u32 = _TRACK_MODE_TYPE_RawWithC2 as u32;
const RawWithC2AndSubCode: u32 = _TRACK_MODE_TYPE_RawWithC2AndSubCode as u32;

const IOCTL_CDROM_CURRENT_POSITION: u32 = 0x1;  // was 0x002D5000 (LLM-invented); bindgen-correct
const IOCTL_CDROM_MEDIA_CATALOG: u32 = 0x2;  // was 0x002D5008 (LLM-invented); bindgen-correct
const IOCTL_CDROM_TRACK_ISRC: u32 = 0x3;  // was 0x002D5010 (LLM-invented); bindgen-correct

const FILE_DEVICE_DVD: u32 = 0x33;

const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const IOCTL_DVD_SEND_KEY: u32 = 0x00335008; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const DVD_CHALLENGE_KEY: u32 = 1;
const DVD_BUS_KEY1: u32 = 2;
const DVD_BUS_KEY2: u32 = 3;
const DVD_TITLE_KEY: u32 = 4;
const DVD_ASF: u32 = 5;
const DVD_SET_RPC_KEY: u32 = 6;
const DVD_GET_RPC_KEY: u32 = 7;
const DVD_DISK_KEY: u32 = 8;
const DVD_INVALIDATE_AGID: u32 = 0x3f;

const CdromSetSpeed: u32 = 0;
const CdromSetStreaming: u32 = 1;

const STATUS_INVALID_BUFFER_SIZE: i32 = 0xC0000206u32 as i32;

// MAX_COPY_PROTECT_AGID is provided by crate::types (= 4)

unsafe extern "system" {
}

// Forward declarations for unknown symbols (caller dedups)

const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const CDROM_EXCLUSIVE_CALLER_LENGTH: u32 = 0x40;  // was 256 (LLM-invented); bindgen-correct
const CDROM_LOCK_IGNORE_VOLUME: u32 = 0x1;
const IO_CDROM_EXCLUSIVE_LOCK: u32 = 0x40040085;
const IOCTL_CDROM_EXCLUSIVE_ACCESS: u32 = 0x0002C05C;
const STATUS_INVALID_DEVICE_STATE: i32 = 0xC0000184u32 as i32;

// GUID is imported from ntkrnl_sys::ddk (same layout: Data1/2/3/4).
const GUID_IO_CDROM_EXCLUSIVE_LOCK: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0; 8],
};

/// Constants used by TracePrint – the caller must provide the actual definitions
/// if they are not already in scope.
// const TRACE_LEVEL_ERROR: u32 = 1;
// const TRACE_LEVEL_INFORMATION: u32 = 4;
// const TRACE_FLAG_IOCTL: u32 = 0x00000040;

const DUID_INCLUDE_SOFTWARE_IDS: u32 = 1;
const DUID_HARDWARE_IDS_ONLY: u32 = 0;
const DUID_VERSION_1: u32 = 1;
const DuidExactMatch: i32 = 10;

const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL: u32 = 0;
const SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT: u32 = 1;
const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE: u32 = 2;

// Extern forward declarations for unknown functions

// IOCTL code constants (computed from CTL_CODE macro values)
const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007405C;
const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000;
const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: u32 = 0x000700A0;
const IOCTL_CDROM_GET_DRIVE_GEOMETRY: u32 = 0x0002404C;
const IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX: u32 = 0x00024050;
const IOCTL_STORAGE_READ_CAPACITY: u32 = 0x002D5140;

const MAXUSHORT: u16 = 0xFFFF;
const MAXULONG: u32 = 0xFFFFFFFF;
const SCSIOP_VERIFY: u8 = 0x2F;
const STATUS_MEDIA_WRITE_PROTECTED: i32 = 0xC00000A2_u32 as i32;

const SCSIOP_TEST_UNIT_READY: u8 = 0x00;
const CDROM_TEST_UNIT_READY_TIMEOUT: u32 = 0x14;  // was 10 (LLM-invented); bindgen-correct
const IOCTL_STORAGE_CHECK_VERIFY: u32 = 0x002D4800;

const STORAGE_EVENT_NOTIFICATION_VERSION_V1: u32 = 1; // unverified
const STORAGE_EVENT_MEDIA_STATUS: u32 = 0x00000001; // unverified
const STORAGE_EVENT_DEVICE_STATUS: u32 = 0x00000002; // unverified
const STORAGE_EVENT_DEVICE_OPERATION: u32 = 0x00000004; // unverified
const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
const TRACE_FLAG_MCN: u32 = 12; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

const IOCTL_STORAGE_EJECTION_CONTROL: u32 = 0x002D0940; // TODO: value
const SecureMediaLock: i32 = 0;
const SimpleMediaLock: i32 = 1;

const STATUS_INVALID_HANDLE: NTSTATUS = 0xC0000008u32 as i32;
const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC0000010u32 as i32;
const FDO_HACK_NO_STREAMING: u32 = 0x20;  // was 0x00000040 (LLM-invented); bindgen-correct
const CdromStreamingDisable: u32 = 0;
const CdromStreamingEnableForReadOnly: u32 = 1;
const CdromStreamingEnableForWriteOnly: u32 = 2;
const CdromStreamingEnableForReadWrite: u32 = 3;

// External declarations for functions not in ntkrnl_sys

// SCSI operation code constant – actual value must be supplied by the caller
const SCSIOP_SEND_OPC_INFORMATION: u8 = 0x54; // TODO: define actual value  // was 0x00 (LLM-invented); bindgen-correct

// Minimal structure for CDB union used in this function
#[repr(C)]
struct CDB_SEND_OPC_INFORMATION {
    OperationCode: u8,
    DoOpc: u8,
    Exclude0: u8,
    Exclude1: u8,
}


// Minimal structure for the input buffer

/// Handles an IOCTL_CDROM_SEND_OPC_INFORMATION request

const SL_REALTIME_STREAM: u8 = 0x20;

const WdfRequestTypeRead: u32 = 3;
const WdfRequestTypeWrite: u32 = 4;

const SCSIOP_START_STOP_UNIT: u8 = 0x1B;
const IOCTL_STORAGE_EJECT_MEDIA: u32 = 0x002D4808;
const LOADING_MECHANISM_TRAY: u32 = 1; // assumed enum value
const STATUS_DEVICE_BUSY: NTSTATUS = 0x80000011_u32 as i32;

const IOCTL_STORAGE_RESERVE: u32 = 0x002D4810;          // actual value unknown
const IOCTL_STORAGE_RELEASE: u32 = 0x002D4814;          // actual value unknown
const SCSIOP_RESERVE_UNIT10: u8 = 0x56;           // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const SCSIOP_RELEASE_UNIT10: u8 = 0x57;           // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const SCSIOP_RESERVE_UNIT: u8 = 0x16;             // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const SCSIOP_RELEASE_UNIT: u8 = 0x17;             // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const FDO_HACK_NO_RESERVE6: u32 = 0x8;           // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const SRB_FLAGS_QUEUE_ACTION_ENABLE: u8 = 0x2;    // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const SRB_SIMPLE_TAG_REQUEST: u8 = 0x20;          // actual value unknown  // was 0 (LLM-invented); bindgen-correct
const CDROM_TAG_SRB: u32 = 0x53436353;                 // actual value unknown
// cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;

// PAGED_CODE();

const IOCTL_STORAGE_PERSISTENT_RESERVE_IN: u32 = 0x002D5018; // placeholder
const IOCTL_STORAGE_PERSISTENT_RESERVE_OUT: u32 = 0x002DD01C; // placeholder
const SCSIOP_PERSISTENT_RESERVE_IN: u8 = 0x5e; // placeholder
const SCSIOP_PERSISTENT_RESERVE_OUT: u8 = 0x5f; // placeholder

const IOCTL_DISK_VOLUMES_ARE_READY: u32 = 0x0007C220;

const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
const CDROM_VOLUME_VERIFY_CHECKED: u32 = 0x34;  // was 0x01 (LLM-invented); bindgen-correct
const STATUS_VERIFY_REQUIRED: i32 = 0x80000016_u32 as i32;
const DO_VERIFY_VOLUME: u32 = 0x02;
// RAW_SECTOR_SIZE / CD_RAW_SECTOR_WITH_* defined above (u32); CDDA family aliased above.
const SCSIOP_READ_CD: u8 = 0xBE;
const CD_DA_SECTOR: u8 = 1;
const YELLOW_MODE2_SECTOR: u8 = 3;
const FORM2_MODE2_SECTOR: u8 = 5;
#[no_mangle]
pub unsafe extern "C" fn ReadQChannel(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    OriginalRequest: WDFREQUEST,
    InputBuffer: *mut core::ffi::c_void,
    InputBufferLength: usize,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: usize,
    DataLength: *mut usize,
) -> i32 {
    const IOCTL_CDROM_CURRENT_POSITION: u32 = 0x1;  // was 0x2403C (LLM-invented); bindgen-correct
    const IOCTL_CDROM_MEDIA_CATALOG: u32 = 0x2;  // was 0x24024 (LLM-invented); bindgen-correct
    const IOCTL_CDROM_TRACK_ISRC: u32 = 0x3;  // was 0x24014 (LLM-invented); bindgen-correct
    const AUDIO_STATUS_IN_PROGRESS: u8 = 0x11;
    const CDB_USE_MSF: u8 = 0x1;  // was 0x02 (LLM-invented); bindgen-correct
    const CDB_SUBCHANNEL_BLOCK: u8 = 0x1;  // was 0x40 (LLM-invented); bindgen-correct
    const SCSIOP_READ_SUB_CHANNEL: u8 = 0x42;
    const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let mut status: i32 = STATUS_SUCCESS;
    let mut transferByteCount: u32 = 0;
    let mut cdb: CDB = core::mem::zeroed();
    let inputBuffer = InputBuffer as PCDROM_SUB_Q_DATA_FORMAT;
    let userChannelData = OutputBuffer as PSUB_Q_CHANNEL_DATA;

    PAGED_CODE();

    UNREFERENCED_PARAMETER(InputBufferLength);

    *DataLength = 0;

    // Set size of channel data -- however, this is dependent on
    // what information we are requesting (which Format)
    match (*inputBuffer).Format as u32 {
        IOCTL_CDROM_CURRENT_POSITION => {
            transferByteCount = core::mem::size_of::<SUB_Q_CURRENT_POSITION>() as u32;
        }
        IOCTL_CDROM_MEDIA_CATALOG => {
            transferByteCount = core::mem::size_of::<SUB_Q_MEDIA_CATALOG_NUMBER>() as u32;
        }
        IOCTL_CDROM_TRACK_ISRC => {
            transferByteCount = core::mem::size_of::<SUB_Q_TRACK_ISRC>() as u32;
        }
        _ => {}
    }

    ScratchBuffer_BeginUse(DeviceExtension);

    core::ptr::write_bytes(&mut cdb as *mut CDB as *mut u8, 0, core::mem::size_of::<CDB>());

    // Set up the CDB
    // Always logical unit 0, but only use MSF addressing
    // for IOCTL_CDROM_CURRENT_POSITION
    if (*inputBuffer).Format as u32 == IOCTL_CDROM_CURRENT_POSITION {
        cdb.SUBCHANNEL.as_mut().set_Msf(CDB_USE_MSF);
    }

    // Return subchannel data
    cdb.SUBCHANNEL.as_mut().set_SubQ(CDB_SUBCHANNEL_BLOCK);

    // Specify format of informatin to return
    cdb.SUBCHANNEL.as_mut().Format = (*inputBuffer).Format;

    // Specify which track to access (only used by Track ISRC reads)
    if (*inputBuffer).Format as u32 == IOCTL_CDROM_TRACK_ISRC {
        cdb.SUBCHANNEL.as_mut().TrackNumber = (*inputBuffer).Track;
    }

    cdb.SUBCHANNEL.as_mut().AllocationLength[0] = (transferByteCount >> 8) as u8;
    cdb.SUBCHANNEL.as_mut().AllocationLength[1] = (transferByteCount & 0xFF) as u8;
    cdb.SUBCHANNEL.as_mut().OperationCode = SCSIOP_READ_SUB_CHANNEL;

    status = ScratchBuffer_ExecuteCdb(
        DeviceExtension,
        (OriginalRequest) as _,
        transferByteCount,
        1, // TRUE
        &mut cdb,
        10,
    );

    if status >= 0 {
        let subQPtr = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer as PSUB_Q_CHANNEL_DATA;

        #[cfg(debug_assertions)]
        {
            match (*inputBuffer).Format as u32 {
                IOCTL_CDROM_CURRENT_POSITION => {
                    TracePrint!(
                        TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ReadQChannel: Audio Status is %u\n\0".as_ptr() as *const i8,
                        (*subQPtr).CurrentPosition.as_ref().Header.AudioStatus as u32,
                    );
                    TracePrint!(
                        TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ReadQChannel: Control = 0x%x\n\0".as_ptr() as *const i8,
                        (*subQPtr).CurrentPosition.as_ref().Control() as u32,
                    );
                    TracePrint!(
                        TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ReadQChannel: Track = %u\n\0".as_ptr() as *const i8,
                        (*subQPtr).CurrentPosition.as_ref().TrackNumber as u32,
                    );
                }
                IOCTL_CDROM_MEDIA_CATALOG => {
                    TracePrint!(
                        TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ReadQChannel: Mcval is %u\n\0".as_ptr() as *const i8,
                        (*subQPtr).MediaCatalog.as_ref().Mcval() as u32,
                    );
                }
                IOCTL_CDROM_TRACK_ISRC => {
                    TracePrint!(
                        TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_IOCTL,
                        b"ReadQChannel: Tcval is %u\n\0".as_ptr() as *const i8,
                        (*subQPtr).TrackIsrc.as_ref().Tcval() as u32,
                    );
                }
                _ => {}
            }
        }

        // Update the play active status.
        if (*subQPtr).CurrentPosition.as_ref().Header.AudioStatus == AUDIO_STATUS_IN_PROGRESS {
            *PLAY_ACTIVE(DeviceExtension) = 1;
        } else {
            *PLAY_ACTIVE(DeviceExtension) = 0;
        }

        // Check if output buffer is large enough to contain the data.
        if OutputBufferLength < (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize {
            (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength = OutputBufferLength as u32;
        }

        // Copy our buffer into users.
        core::ptr::copy_nonoverlapping(
            subQPtr as *const u8,
            userChannelData as *mut u8,
            (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize,
        );

        *DataLength = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize;
    }

    // nothing to do after the command finishes.
    ScratchBuffer_EndUse(DeviceExtension);

    return status;
}

const SCSIOP_SEEK: u8 = 0x2B;

#[no_mangle]
pub unsafe extern "C" fn ReadDvdStructure(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    OriginalRequest: WDFREQUEST,
    InputBuffer: *mut core::ffi::c_void,
    InputBufferLength: usize,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: usize,
    DataLength: *mut usize,
) -> i32 {

    let mut status: i32 = 0; // STATUS_SUCCESS
    let request: *mut DVD_READ_STRUCTURE = InputBuffer as *mut DVD_READ_STRUCTURE;
    let header: *mut DVD_DESCRIPTOR_HEADER = OutputBuffer as *mut DVD_DESCRIPTOR_HEADER;
    let mut cdb: CDB = core::mem::zeroed();
    let mut dataLength: u16;
    let mut blockNumber: u32;
    let fourByte: *const u8; // used as PFOUR_BYTE pointer

    // PAGED_CODE();

    // UNREFERENCED_PARAMETER(InputBufferLength);
    let _ = InputBufferLength;

    if (*DeviceExtension).DeviceAdditionalData.DriveDeviceType != 0x33 { // FILE_DEVICE_DVD
        *DataLength = 0;
        return 0xC0000010u32 as i32; // STATUS_INVALID_DEVICE_REQUEST
    }

    dataLength = OutputBufferLength as u16;
    blockNumber = (request.as_ref().unwrap().BlockByteOffset.QuadPart >> (*DeviceExtension).SectorShift as i64) as u32;
    let block_number_ptr: *const u32 = &blockNumber;
    fourByte = block_number_ptr as *const u8;

    ScratchBuffer_BeginUse(DeviceExtension);

    // RtlZeroMemory(&cdb, sizeof(CDB));
    core::ptr::write_bytes((&raw mut cdb) as *mut u8, 0u8, core::mem::size_of::<CDB>());
    // Set up the CDB
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().OperationCode = 0xAD; // SCSIOP_READ_DVD_STRUCTURE
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[0] = *fourByte.add(3); // Byte3 (MSB)
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[1] = *fourByte.add(2); // Byte2
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[2] = *fourByte.add(1); // Byte1
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[3] = *fourByte.add(0); // Byte0 (LSB)
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().LayerNumber   = request.as_ref().unwrap().LayerNumber;
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().Format        = request.as_ref().unwrap().Format as u8;

#[cfg(debug_assertions)] // DBG
    {
        let fmt = request.as_ref().unwrap().Format as u8;
        if fmt > DvdMaxDescriptor as u8 {
            TracePrint!(
                5, // TRACE_LEVEL_VERBOSE
                0x01, // TRACE_FLAG_IOCTL
                c"READ_DVD_STRUCTURE format %x = %s (%x bytes)\n".as_ptr() as *const i8,
                fmt as u32,
                READ_DVD_STRUCTURE_FORMAT_STRINGS[DvdMaxDescriptor as usize],
                dataLength as u32,
            );
        } else {
            TracePrint!(
                5, // TRACE_LEVEL_VERBOSE
                0x01, // TRACE_FLAG_IOCTL
                c"READ_DVD_STRUCTURE format %x = %s (%x bytes)\n".as_ptr() as *const i8,
                fmt as u32,
                READ_DVD_STRUCTURE_FORMAT_STRINGS[fmt as usize],
                dataLength as u32,
            );
        }
    }

    if request.as_ref().unwrap().Format == 0x01 { // DvdDiskKeyDescriptor
        (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().set_AGID(request.as_ref().unwrap().SessionId as u8);
    }

    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().AllocationLength[0] = (dataLength >> 8) as u8;
    (*((&raw mut cdb) as *mut CDB)).READ_DVD_STRUCTURE.as_mut().AllocationLength[1] = (dataLength & 0xff) as u8;

    status = ScratchBuffer_ExecuteCdb(DeviceExtension, (OriginalRequest) as _, dataLength as u32, (1i32) as _, &raw const cdb as *mut CDB, 12);

    if status >= 0 { // NT_SUCCESS(status)
        TracePrint!(
            4, // TRACE_LEVEL_INFORMATION
            0x01,
            c"DvdDCCompletion - READ_STRUCTURE: descriptor format of %d\n".as_ptr() as *const i8,
            request.as_ref().unwrap().Format,
        );

        // RtlMoveMemory(header, ..., ...) -> copy_nonoverlapping
        let src = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer;
        let len = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength;
        core::ptr::copy_nonoverlapping(src as *const u8, header as *mut u8, len as usize);

        TracePrint!(
            0x02,
            0x01,
            c"DvdDCCompletion - READ_STRUCTURE:\n\tHeader at %p\n\tDvdDCCompletion - READ_STRUCTURE: data at %p\n\tDataBuffer was at %p\n\tDataTransferLength was %lx\n".as_ptr() as *const i8,
            header,
            core::ptr::addr_of!((*header).Data) as *const c_void,
            (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer,
            (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength,
        );

        TracePrint!(
            0x02,
            0x01,
            c"READ_STRUCTURE: header->Length %lx -> ".as_ptr() as *const i8,
            (*header).Length as u32,
        );

        REVERSE_SHORT(&raw mut (*header).Length);

        TracePrint!(
            0x02,
            0x01,
            c"%lx\n".as_ptr() as *const i8,
            (*header).Length as u32,
        );

        if request.as_ref().unwrap().Format == 0x00 { // DvdPhysicalDescriptor
            let transfer_len = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength;
            let header_data_offset = core::mem::offset_of!(DVD_DESCRIPTOR_HEADER, Data);
            let tempLength: u32 = if transfer_len > header_data_offset as u32 {
                transfer_len - header_data_offset as u32
            } else {
                0
            };

            let layer: *mut DVD_LAYER_DESCRIPTOR = (*header).Data.as_mut_ptr() as *mut DVD_LAYER_DESCRIPTOR;

            if tempLength >= (core::mem::offset_of!(DVD_LAYER_DESCRIPTOR, StartingDataSector) + core::mem::size_of::<u32>()) as u32 {
                TracePrint!(
                    0x02,
                    0x01,
                    c"READ_STRUCTURE: StartingDataSector %lx -> ".as_ptr() as *const i8,
                    (*layer).StartingDataSector,
                );
                REVERSE_LONG(&raw mut (*layer).StartingDataSector);
                TracePrint!(
                    0x02,
                    0x01,
                    c"%lx\n".as_ptr() as *const i8,
                    (*layer).StartingDataSector,
                );
            }
            if tempLength >= (core::mem::offset_of!(DVD_LAYER_DESCRIPTOR, EndDataSector) + core::mem::size_of::<u32>()) as u32 {
                TracePrint!(
                    0x02,
                    0x01,
                    c"READ_STRUCTURE: EndDataSector %lx -> ".as_ptr() as *const i8,
                    (*layer).EndDataSector,
                );
                REVERSE_LONG(&raw mut (*layer).EndDataSector);
                TracePrint!(
                    0x02,
                    0x01,
                    c"%lx\n".as_ptr() as *const i8,
                    (*layer).EndDataSector,
                );
            }
            if tempLength >= (core::mem::offset_of!(DVD_LAYER_DESCRIPTOR, EndLayerZeroSector) + core::mem::size_of::<u32>()) as u32 {
                TracePrint!(
                    0x02,
                    0x01,
                    c"READ_STRUCTURE: EndLayerZeroSector %lx -> ".as_ptr() as *const i8,
                    (*layer).EndLayerZeroSector,
                );
                REVERSE_LONG(&raw mut (*layer).EndLayerZeroSector);
                TracePrint!(
                    0x02,
                    0x01,
                    c"%lx\n".as_ptr() as *const i8,
                    (*layer).EndLayerZeroSector,
                );
            }
        }

        if status < 0 { // never true but kept
            TracePrint!(
                0x02,
                0x01,
                c"Status is %lx\n".as_ptr() as *const i8,
                status,
            );
            TracePrint!(
                0x02,
                0x01,
                c"DvdDeviceControlCompletion - IOCTL_DVD_READ_STRUCTURE: data transfer length of %d\n".as_ptr() as *const i8,
                (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength,
            );
        }

        *DataLength = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize;
    }

    ScratchBuffer_EndUse(DeviceExtension);

    status
}


const SCSIOP_SET_CD_SPEED: u8 = 0xBB;    // assumed value
const SCSIOP_SET_STREAMING: u8 = 0xB6;   // assumed value

// Constants
const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// Forward declarations of extern functions not in available imports

const STATUS_INVALID_USER_BUFFER: i32 = (0xC00000E8u32 as i32); // FIXED 2026-07-05: was LLM-guessed "typical" value; verified vs C header (ntstatus.h; 0xC0000146 is STATUS_FLOAT_STACK_CHECK)
const RESERVATION_ACTION_READ_KEYS: u8 = 0;
const RESERVATION_ACTION_READ_RESERVATIONS: u8 = 1;
const RESERVATION_ACTION_REGISTER: u8 = 0;
const RESERVATION_ACTION_REGISTER_IGNORE_EXISTING: u8 = 0x6;  // was 1 (LLM-invented); bindgen-correct
const RESERVATION_ACTION_CLEAR: u8 = 0x3;  // was 4 (LLM-invented); bindgen-correct
const RESERVATION_ACTION_RESERVE: u8 = 0x1;  // was 2 (LLM-invented); bindgen-correct
const RESERVATION_ACTION_RELEASE: u8 = 0x2;  // was 3 (LLM-invented); bindgen-correct
const RESERVATION_ACTION_PREEMPT: u8 = 0x4;  // was 5 (LLM-invented); bindgen-correct
const RESERVATION_ACTION_PREEMPT_ABORT: u8 = 0x5;  // was 6 (LLM-invented); bindgen-correct

// cdrom.h:68: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNxCacheAligned: u32 = 4;
const CDROM_TAG_PLAY_ACTIVE: u32 = 0x70436353; // CDROM_TAG_PLAY_ACTIVE placeholder
const AUDIO_STATUS_IN_PROGRESS: u8 = 0x11; // AUDIO_STATUS_IN_PROGRESS

// Forward declarations for unknown externs / macros

// Constants (placeholders – actual values from DDK/ntifs)
const IOCTL_DVD_READ_KEY: u32 = 0x00335004; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)          // approximate
const IOCTL_DVD_START_SESSION: u32 = 0x00335000; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)     // approximate
const DvdDiskKeyDescriptor: u8 = 0x10;
const DVD_DISK_KEY_LENGTH: u32 = 2048;
const STATUS_INTERNAL_ERROR: i32 = (0xC00000E0u32 as i32);   // NTSTATUS
const STATUS_INSUFFICIENT_RESOURCES: i32 = (0xC000009Au32 as i32);
const STATUS_SUCCESS: i32 = 0;
const SCSIOP_REPORT_KEY: u8 = 0xA4;
const DVD_REPORT_AGID: u8 = 0x0;  // was 0xFE (LLM-invented); bindgen-correct
const DVD_TAG_READ_KEY: u32 = 0x75436353; // "RBM "?? placeholder

// Function translation

const IOCTL_CDROM_READ_TOC: u32 = 0x00024000; // TODO: define correct IOCTL value
const IOCTL_CDROM_GET_LAST_SESSION: u32 = 0x00024038; // TODO: define correct IOCTL value
const CDROM_READ_TOC_EX_FORMAT_SESSION: u8 = 0x1; // TODO: define correct value  // was 0 (LLM-invented); bindgen-correct
const SCSIOP_READ_TOC: u8 = 0x43; // READ TOC opcode

// Constants

// External function declarations for callees not yet translated.

const DvdCopyrightDescriptor: u8 = 0;
const DVD_RPC_KEY_LENGTH: usize = 8;

#[no_mangle]
pub unsafe extern "system" fn RequestHandleUnknownIoctl(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let requestContext: *mut CDROM_REQUEST_CONTEXT = RequestGetContext(Request);
    let syncRequired: BOOLEAN = (*requestContext).SyncRequired;

    let mut sendOptionsFlags: ULONG = 0;
    let mut requestSent: BOOLEAN = 0;

    WdfRequestFormatRequestUsingCurrentType(Request);

    if syncRequired != 0 {
        sendOptionsFlags = WDF_REQUEST_SEND_OPTION_SYNCHRONOUS;
    } else {
        WdfRequestSetCompletionRoutine(
            Request,
            // RequestDummyCompletionRoutine is declared `extern "system"` (stdcall
            // ABI on i386, C on amd64/arm64) — the field's ABI on each arch — but
            // rustc nominally distinguishes it from the field's `"stdcall"`/`"C"`
            // fn-ptr type on i386. Transmute to the inferred field type (zero-cost).
            Some(core::mem::transmute(
                RequestDummyCompletionRoutine as unsafe extern "system" fn(WDFREQUEST, WDFIOTARGET, *mut WDF_REQUEST_COMPLETION_PARAMS, WDFCONTEXT),
            )),
            core::ptr::null_mut(),
        );
    }

    status = RequestSend(
        deviceExtension,
        Request,
        (*deviceExtension).IoTarget,
        sendOptionsFlags,
        &mut requestSent as *mut BOOLEAN,
    );

    if requestSent != 0 {
        if syncRequired != 0 {
            // the request needs to be completed here.
            RequestCompletion(
                deviceExtension,
                Request,
                status,
                (WdfRequestGetInformation(Request) as u32) as usize,
            );
        }
    } else {
        // failed to send the request to IoTarget
        RequestCompletion(
            deviceExtension,
            Request,
            status,
            (WdfRequestGetInformation(Request) as u32) as usize,
        );
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceIsPlayActive(Device: WDFDEVICE) -> u8 {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let currentBuffer: *mut SUB_Q_CURRENT_POSITION;
    let mut bytesRead: usize = 0;

    // PAGED_CODE(); // macro

    // if we don't think it is playing audio, don't bother checking.
    if (*deviceExtension).DeviceAdditionalData.PlayActive == 0 {
        return 0;
    }

    // Allocate the required memory
    ntkrnl_sys::ASSERT!(core::mem::size_of::<SUB_Q_CURRENT_POSITION>() >= core::mem::size_of::<CDROM_SUB_Q_DATA_FORMAT>());
    currentBuffer = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        core::mem::size_of::<SUB_Q_CURRENT_POSITION>(),
        CDROM_TAG_PLAY_ACTIVE,
    ) as *mut SUB_Q_CURRENT_POSITION;
    if currentBuffer.is_null() {
        return 0;
    }

    // set the options in the output buffer format
    {
        let fmt_ptr = currentBuffer as *mut CDROM_SUB_Q_DATA_FORMAT;
        (*fmt_ptr).Format = IOCTL_CDROM_CURRENT_POSITION as u8;
        (*fmt_ptr).Track = 0;
    }

    // Send SCSI command to read Q Channel information.
    status = ReadQChannel(
        deviceExtension,
        core::ptr::null_mut(), // WDFREQUEST NULL
        currentBuffer as *mut core::ffi::c_void,
        core::mem::size_of::<CDROM_SUB_Q_DATA_FORMAT>(),
        currentBuffer as *mut core::ffi::c_void,
        core::mem::size_of::<SUB_Q_CURRENT_POSITION>(),
        core::ptr::addr_of_mut!(bytesRead),
    );

    if status < 0 { // !NT_SUCCESS
        ExFreePool(currentBuffer as *mut core::ffi::c_void);
        return 0;
    }

    // update the playactive flag appropriately
    if (*currentBuffer).Header.AudioStatus == AUDIO_STATUS_IN_PROGRESS {
        (*deviceExtension).DeviceAdditionalData.PlayActive = 1;
    } else {
        (*deviceExtension).DeviceAdditionalData.PlayActive = 0;
    }

    ExFreePool(currentBuffer as *mut core::ffi::c_void);

    return (*deviceExtension).DeviceAdditionalData.PlayActive;
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetInquiryData(
    device_extension: *mut CDROM_DEVICE_EXTENSION,
    request: WDFREQUEST,
    request_parameters: WDF_REQUEST_PARAMETERS,
    data_length: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*device_extension).DeviceAdditionalData);

    *data_length = 0;

    if request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength == 0 {
        status = STATUS_BUFFER_TOO_SMALL;
    } else {
        let mut output_buffer: *mut core::ffi::c_void = core::ptr::null_mut();

        *data_length = if ((*cdData).CachedInquiryDataByteCount as usize) < request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize {
            (*cdData).CachedInquiryDataByteCount as usize
        } else {
            request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize
        };

        status = WdfRequestRetrieveOutputBuffer(
            request,
            request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &mut output_buffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(status) && (!output_buffer.is_null()) {
            RtlCopyMemory(
                output_buffer,
                core::ptr::addr_of!((*cdData).CachedInquiryData) as *const core::ffi::c_void,
                *data_length,
            );
        }

        if request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < (*cdData).CachedInquiryDataByteCount as usize {
            status = STATUS_BUFFER_OVERFLOW;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetMediaTypeEx(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> i32 {
    const STATUS_INVALID_LEVEL: i32 = 0xC0000148u32 as i32;
    const MEDIA_READ_ONLY: u32 = 0x0004;
    const MEDIA_READ_WRITE: u32 = 0x8;  // was 0x0006 (LLM-invented); bindgen-correct
    const MEDIA_CURRENTLY_MOUNTED: u32 = 0x80000000;
    const SCSIOP_TEST_UNIT_READY: u8 = 0x00;
    const CDROM_TEST_UNIT_READY_TIMEOUT: u32 = 0x14;  // was 10 (LLM-invented); bindgen-correct
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let mut status: i32 = STATUS_SUCCESS;
    let cd_data: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut media_types: *mut GET_MEDIA_TYPES = core::ptr::null_mut();
    let mut media_info: *mut DEVICE_MEDIA_INFO = core::ptr::null_mut();
    let mut size_needed: u32 = 0;
    let zpodd_info: *mut ZERO_POWER_ODD_INFO = (*DeviceExtension).ZeroPowerODDInfo;

    *DataLength = 0;

    if KeGetCurrentIrql() >= DISPATCH_LEVEL {
        NT_ASSERT(0);
        return STATUS_INVALID_LEVEL;
    }

    size_needed = core::mem::size_of::<GET_MEDIA_TYPES>() as u32;
    if (*cd_data).Mmc.IsMmc != 0 {
        size_needed += core::mem::size_of::<DEVICE_MEDIA_INFO>() as u32;
    }

    status = WdfRequestRetrieveOutputBuffer(
        Request,
        size_needed as usize,
        &mut media_types as *mut *mut GET_MEDIA_TYPES as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) && !media_types.is_null() {
        media_info = core::ptr::addr_of_mut!((*media_types).MediaInfo[0]);
        RtlZeroMemory(media_types as *mut core::ffi::c_void, size_needed as usize);

        (*media_types).DeviceType = (*cd_data).DriveDeviceType;
        (*media_types).MediaInfoCount = 1;

        let info = &mut (*media_info).DeviceSpecific.RemovableDiskInfo;
        info.MediaType = CD_ROM;
        info.NumberMediaSides = 1;
        info.MediaCharacteristics = MEDIA_READ_ONLY;
        info.Cylinders.QuadPart = (*DeviceExtension).DiskGeometry.Cylinders.QuadPart;
        info.TracksPerCylinder = (*DeviceExtension).DiskGeometry.TracksPerCylinder;
        info.SectorsPerTrack = (*DeviceExtension).DiskGeometry.SectorsPerTrack;
        info.BytesPerSector = (*DeviceExtension).DiskGeometry.BytesPerSector;

        if (*cd_data).Mmc.IsMmc != 0 {
            (*media_types).MediaInfoCount += 1;
            media_info = media_info.add(1);

            let info2 = &mut (*media_info).DeviceSpecific.RemovableDiskInfo;
            info2.MediaType = RemovableMedia as u32;
            info2.NumberMediaSides = 1;
            info2.MediaCharacteristics = MEDIA_READ_WRITE;
            info2.Cylinders.QuadPart = (*DeviceExtension).DiskGeometry.Cylinders.QuadPart;
            info2.TracksPerCylinder = (*DeviceExtension).DiskGeometry.TracksPerCylinder;
            info2.SectorsPerTrack = (*DeviceExtension).DiskGeometry.SectorsPerTrack;
            info2.BytesPerSector = (*DeviceExtension).DiskGeometry.BytesPerSector;

            media_info = media_info.sub(1);
        }

        let cond = (EXCLUSIVE_MODE(cd_data) == 0
            || EXCLUSIVE_OWNER(cd_data, WdfRequestGetFileObject(Request) as *mut core::ffi::c_void) != 0)
            && (zpodd_info.is_null() || (*zpodd_info).InZeroPowerState == 0);
        if cond {
            let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
            RtlZeroMemory(
                &mut srb as *mut SCSI_REQUEST_BLOCK as *mut core::ffi::c_void,
                core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
            );
            let cdb = &mut srb.Cdb as *mut _ as *mut CDB;
            srb.CdbLength = 6;
            (*cdb).CDB6GENERIC.as_mut().OperationCode = SCSIOP_TEST_UNIT_READY;
            srb.TimeOutValue = CDROM_TEST_UNIT_READY_TIMEOUT;

            status = DeviceSendSrbSynchronously(
                (*DeviceExtension).Device as *mut core::ffi::c_void,
                &mut srb,
                core::ptr::null_mut(),
                0,
                0,
                (Request) as _,
            );

            if NT_SUCCESS(status) {
                if (*cd_data).Mmc.IsMmc != 0 && (*cd_data).Mmc.WriteAllowed != 0 {
                    media_info = media_info.add(1);
                    (*media_info).DeviceSpecific.RemovableDiskInfo.MediaCharacteristics |=
                        MEDIA_CURRENTLY_MOUNTED;
                    media_info = media_info.sub(1);
                } else {
                    (*media_info).DeviceSpecific.RemovableDiskInfo.MediaCharacteristics |=
                        MEDIA_CURRENTLY_MOUNTED;
                }
            } else {
                let format_str =
                    b"RequestHandleGetMediaTypeEx: GET_MEDIA_TYPES status of TUR - %lx\n\0"
                        .as_ptr() as *const i8;
                TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, format_str, status);
            }
        }

        status = STATUS_SUCCESS;
    }

    *DataLength = size_needed as usize;
    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetDvdRegion(
    device_extension: *mut CDROM_DEVICE_EXTENSION,
    request: WDFREQUEST,
    data_length: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut output_buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut bytes_returned: usize = 0;
    let mut copy_protect_key: *mut DVD_COPY_PROTECT_KEY = core::ptr::null_mut();
    let mut key_length: u32 = 0;
    let mut dvd_header: *mut DVD_DESCRIPTOR_HEADER;
    let mut copy_right_descriptor: *mut DVD_COPYRIGHT_DESCRIPTOR;
    let mut dvd_region: *mut DVD_REGION = core::ptr::null_mut();
    let mut read_structure: *mut DVD_READ_STRUCTURE = core::ptr::null_mut();
    let mut rpc_key: *mut DVD_RPC_KEY;

    *data_length = 0;

    if (*device_extension).DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD {
        status = STATUS_INVALID_DEVICE_REQUEST;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            request,
            core::mem::size_of::<DVD_REGION>(),
            &mut output_buffer as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        let mut tmp = core::cmp::max(
            core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() + core::mem::size_of::<DVD_COPYRIGHT_DESCRIPTOR>(),
            core::mem::size_of::<DVD_READ_STRUCTURE>(),
        );
        tmp = core::cmp::max(tmp, DVD_RPC_KEY_LENGTH);
        let align = core::mem::size_of::<u64>();
        tmp += align - (tmp & (align - 1));
        key_length = tmp as u32;

        read_structure = ExAllocatePoolWithTag(NonPagedPoolNx, key_length as usize, DVD_TAG_READ_KEY)
            as *mut DVD_READ_STRUCTURE;
        if read_structure.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if NT_SUCCESS(status) {
        RtlZeroMemory(read_structure as *mut core::ffi::c_void, key_length as usize);
        (*read_structure).Format = DvdCopyrightDescriptor as u32;
        status = ReadDvdStructure(
            device_extension,
            request,
            read_structure as *mut core::ffi::c_void,
            key_length as usize,
            read_structure as *mut core::ffi::c_void,
            core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() + core::mem::size_of::<DVD_COPYRIGHT_DESCRIPTOR>(),
            &mut bytes_returned,
        );
    }

    if NT_SUCCESS(status) {
        dvd_header = read_structure as *mut DVD_DESCRIPTOR_HEADER;
        copy_right_descriptor = (*dvd_header).Data.as_ptr() as *mut DVD_COPYRIGHT_DESCRIPTOR;
        dvd_region = output_buffer as *mut DVD_REGION;
        (*dvd_region).CopySystem = (*copy_right_descriptor).CopyrightProtectionType;
        (*dvd_region).RegionData = (*copy_right_descriptor).RegionManagementInformation;

        copy_protect_key = read_structure as *mut DVD_COPY_PROTECT_KEY;
        RtlZeroMemory(copy_protect_key as *mut core::ffi::c_void, DVD_RPC_KEY_LENGTH);
        (*copy_protect_key).KeyLength = DVD_RPC_KEY_LENGTH as u32;
        (*copy_protect_key).KeyType = DvdGetRpcKey;

        status = DvdStartSessionReadKey(
            device_extension,
            IOCTL_DVD_READ_KEY,
            request,
            copy_protect_key as *mut core::ffi::c_void,
            DVD_RPC_KEY_LENGTH,
            copy_protect_key as *mut core::ffi::c_void,
            DVD_RPC_KEY_LENGTH,
            &mut bytes_returned,
        );
    }

    if NT_SUCCESS(status) {
        rpc_key = (*copy_protect_key).KeyData.as_ptr() as *mut DVD_RPC_KEY;
        if (*rpc_key).RpcScheme == 1 {
            if (*rpc_key).TypeCode() != 0 {
                (*dvd_region).SystemRegion = !(*rpc_key).RegionMask;
                (*dvd_region).ResetCount = (*rpc_key).UserResetsAvailable();
            } else {
                (*dvd_region).SystemRegion = 0;
                (*dvd_region).ResetCount = (*rpc_key).UserResetsAvailable();
            }
            *data_length = core::mem::size_of::<DVD_REGION>();
        } else {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestHandleGetDvdRegion => rpcKey->RpcScheme != 1\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    if !read_structure.is_null() {
        ExFreePool(read_structure as *mut core::ffi::c_void);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateRawRead(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    mut RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);

    let mut inputBuffer: *mut c_void = core::ptr::null_mut();
    let irp: *mut IRP;
    let currentStack: *mut IO_STACK_LOCATION;

    let mut startingOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut transferBytes: u64 = 0u64;
    let mut endOffset: u64;
    let mut mdlBytes: u64;
    let mut rawReadInfo: RAW_READ_INFO = core::mem::zeroed();

    *DataLength = 0usize;

    irp = WdfRequestWdmGetIrp(Request);
    currentStack = IoGetCurrentIrpStackLocation(irp);

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &mut inputBuffer as *mut *mut c_void,
        core::ptr::null_mut(),
    );

    // Check that ending sector is on disc and buffers are there and of
    // correct size.
    if status >= 0 && RequestParameters.Parameters.DeviceIoControl.as_ref().Type3InputBuffer.is_null() {
        // This is a call from user space.
        if !inputBuffer.is_null() {
            (*currentStack).Parameters.DeviceIoControl.Type3InputBuffer = inputBuffer;
            RequestParameters.Parameters.DeviceIoControl.as_mut().Type3InputBuffer = inputBuffer;

            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < core::mem::size_of::<RAW_READ_INFO>() {
                *DataLength = core::mem::size_of::<RAW_READ_INFO>();
                status = STATUS_BUFFER_TOO_SMALL;
            }
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        // Copy away the input buffer.
        rawReadInfo = core::ptr::read(RequestParameters.Parameters.DeviceIoControl.as_ref().Type3InputBuffer as *const RAW_READ_INFO);

        startingOffset.QuadPart = rawReadInfo.DiskOffset.QuadPart;

        if rawReadInfo.TrackMode == CDDA
            || rawReadInfo.TrackMode == YellowMode2
            || rawReadInfo.TrackMode == XAForm2
        {
            transferBytes = (rawReadInfo.SectorCount as u64) * (RAW_SECTOR_SIZE as u64);
        } else if rawReadInfo.TrackMode == RawWithSubCode {
            transferBytes = (rawReadInfo.SectorCount as u64) * (CD_RAW_SECTOR_WITH_SUBCODE_SIZE as u64);
        } else if rawReadInfo.TrackMode == RawWithC2 {
            transferBytes = (rawReadInfo.SectorCount as u64) * (CD_RAW_SECTOR_WITH_C2_SIZE as u64);
        } else if rawReadInfo.TrackMode == RawWithC2AndSubCode {
            transferBytes = (rawReadInfo.SectorCount as u64) * (CD_RAW_SECTOR_WITH_C2_AND_SUBCODE_SIZE as u64);
        } else {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid TrackMode type %x for XA read\n\0".as_ptr() as *const i8,
                rawReadInfo.TrackMode,
            );
        }

        endOffset = (rawReadInfo.SectorCount as u64) * (COOKED_SECTOR_SIZE as u64);
        endOffset += startingOffset.QuadPart as u64;

        if rawReadInfo.SectorCount == 0 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid I/O parameters for XA Read (zero sectors requested)\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        } else if transferBytes < rawReadInfo.SectorCount as u64 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid I/O parameters for XA Read (TransferBytes Overflow)\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        } else if endOffset < startingOffset.QuadPart as u64 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid I/O parameters for XA Read (EndingOffset Overflow)\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < transferBytes as usize {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid I/O parameters for XA Read (Bad buffer size)\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        } else if endOffset > (*DeviceExtension).PartitionLength.QuadPart as u64 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid I/O parameters for XA Read (Request Out of Bounds)\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        mdlBytes = MmGetMdlByteCount((*irp).MdlAddress) as u64;
        if mdlBytes < transferBytes {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid MDL %s, Irp %p\n\0".as_ptr() as *const i8,
                b"size (5)\0".as_ptr() as *const i8,
                irp,
            );
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        if ((MmGetMdlVirtualAddress((*irp).MdlAddress) as usize) & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize) != 0 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: Invalid I/O parameters for XA Read (Buffer %p not aligned with mask %x\n\0".as_ptr() as *const i8,
                RequestParameters.Parameters.DeviceIoControl.as_ref().Type3InputBuffer,
                (*(*DeviceExtension).AdapterDescriptor).AlignmentMask,
            );
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        let mut bufferIsPageAligned: u8 = 0;
        let mut maxLength: ULONG = 0;

        if ((MmGetMdlVirtualAddress((*irp).MdlAddress) as usize) & (PAGE_SIZE - 1)) == 0 {
            bufferIsPageAligned = 1;
        }

        if bufferIsPageAligned != 0 {
            maxLength = (*cdData).MaxPageAlignedTransferBytes;
        } else {
            maxLength = (*cdData).MaxUnalignedTransferBytes;
        }

        if transferBytes > maxLength as u64 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestValidateRawRead: The XA Read (type %x) would require %I64x bytes, but the adapter can only handle %x bytes (for a%saligned buffer)\n\0".as_ptr() as *const i8,
                rawReadInfo.TrackMode,
                transferBytes,
                maxLength,
                if bufferIsPageAligned != 0 { b" \0".as_ptr() } else { b"n un\0".as_ptr() } as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        (*currentStack).MinorFunction = MAXIMUM_RETRIES; // HACKHACK - REF #0001
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateReadTocEx(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut inputBuffer: *mut CDROM_READ_TOC_EX = core::ptr::null_mut();
    *DataLength = 0;

    if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize) < core::mem::size_of::<CDROM_READ_TOC_EX>() {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) < MINIMUM_CDROM_READ_TOC_EX_SIZE as usize {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = MINIMUM_CDROM_READ_TOC_EX_SIZE as usize;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) > (u16::MAX as usize) {
        status = STATUS_INVALID_PARAMETER;
    } else if ((RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
               & ((*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize) != 0) {
        status = STATUS_INVALID_PARAMETER;
    }

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &raw mut inputBuffer as *mut *mut CDROM_READ_TOC_EX as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        let input = &*inputBuffer;
        if (input.Reserved1() != 0) || (input.Reserved2 != 0) || (input.Reserved3 != 0) {
            status = STATUS_INVALID_PARAMETER;
        } else if (input.Format() == 0x00 /* CDROM_READ_TOC_EX_FORMAT_TOC */)
               || (input.Format() == 0x02 /* CDROM_READ_TOC_EX_FORMAT_FULL_TOC */)
               || (input.Format() == 0x05 /* CDROM_READ_TOC_EX_FORMAT_CDTEXT */) {
            // SessionTrack field is used
        } else if (input.Format() == 0x01 /* CDROM_READ_TOC_EX_FORMAT_SESSION */)
                || (input.Format() == 0x03 /* CDROM_READ_TOC_EX_FORMAT_PMA */)
                || (input.Format() == 0x04 /* CDROM_READ_TOC_EX_FORMAT_ATIP */) {
            if input.SessionTrack != 0 {
                status = STATUS_INVALID_PARAMETER;
            }
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn RequestValidateReadToc(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    *DataLength = 0;

    if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
        < core::mem::size_of::<CDROM_TOC>()
    {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = core::mem::size_of::<CDROM_TOC>();
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
        & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize)
        != 0
    {
        status = STATUS_INVALID_PARAMETER;
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateGetLastSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    *DataLength = 0;

    let output_buffer_len = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;

    if output_buffer_len < core::mem::size_of::<CDROM_TOC_SESSION_DATA>() {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = core::mem::size_of::<CDROM_TOC_SESSION_DATA>();
    } else {
        let alignment_mask = (*(*DeviceExtension).AdapterDescriptor).AlignmentMask;
        if output_buffer_len & alignment_mask as usize != 0 {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateReadQChannel(
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut inputBuffer: *mut CDROM_SUB_Q_DATA_FORMAT = core::ptr::null_mut();
    let mut transferByteCount: u32 = 0;

    *DataLength = 0;

    if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize) < core::mem::size_of::<CDROM_SUB_Q_DATA_FORMAT>() {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            core::ptr::addr_of_mut!(inputBuffer) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if (*inputBuffer).Format as u32 == IOCTL_CDROM_CURRENT_POSITION {
            transferByteCount = core::mem::size_of::<SUB_Q_CURRENT_POSITION>() as u32;
        } else if (*inputBuffer).Format as u32 == IOCTL_CDROM_MEDIA_CATALOG {
            transferByteCount = core::mem::size_of::<SUB_Q_MEDIA_CATALOG_NUMBER>() as u32;
        } else if (*inputBuffer).Format as u32 == IOCTL_CDROM_TRACK_ISRC {
            transferByteCount = core::mem::size_of::<SUB_Q_TRACK_ISRC>() as u32;
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if NT_SUCCESS(status) {
        if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) < (transferByteCount as usize) {
            status = STATUS_BUFFER_TOO_SMALL;
            *DataLength = transferByteCount as usize;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateDvdReadStructure(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS; // 0
    *DataLength = 0;

    if status >= 0 {
        if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < core::mem::size_of::<DVD_READ_STRUCTURE>() {
            TracePrint!(
                2, // TRACE_LEVEL_ERROR
                0x80, // TRACE_FLAG_IOCTL
                b"RequestValidateDvdReadStructure - input buffer length too small (was %d should be %d)\n\0".as_ptr() as *const i8,
                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as i32,
                core::mem::size_of::<DVD_READ_STRUCTURE>() as i32,
            );
            status = STATUS_INVALID_PARAMETER;
        } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>() {
            TracePrint!(
                2, // TRACE_LEVEL_ERROR
                0x80, // TRACE_FLAG_IOCTL
                b"RequestValidateDvdReadStructure - output buffer cannot hold header information\n\0".as_ptr() as *const i8,
            );
            status = STATUS_BUFFER_TOO_SMALL;
            *DataLength = core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>();
        } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > 65535 {
            // MAXUSHORT
            TracePrint!(
                2, // TRACE_LEVEL_ERROR
                0x80, // TRACE_FLAG_IOCTL
                b"RequestValidateDvdReadStructure - output buffer too large\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize) != 0 {
            status = STATUS_INVALID_PARAMETER;
        } else if (*DeviceExtension).DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn RequestValidateDvdStartSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    *DataLength = 0;
    if NT_SUCCESS(status) {
        if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
            < core::mem::size_of::<DVD_SESSION_ID>()
        {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"RequestValidateDvdStartSession: DVD_START_SESSION - output buffer too small\n\0"
                    .as_ptr() as *const i8,
            );
            status = STATUS_BUFFER_TOO_SMALL;
            *DataLength = core::mem::size_of::<DVD_SESSION_ID>();
        } else if (*DeviceExtension).DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateDvdSendKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS;
    let mut key: *mut DVD_COPY_PROTECT_KEY = core::ptr::null_mut();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        core::ptr::addr_of_mut!(key) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        if ((RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize)
            < core::mem::size_of::<DVD_COPY_PROTECT_KEY>())
            || (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize
                != (*key).KeyLength as usize)
        {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"RequestValidateDvdSendKey: [%p] IOCTL_DVD_SEND_KEY - key is too small or does not match KeyLength\n\0"
                    .as_ptr() as *const i8,
                Request,
            );
            status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
        }
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_DVD_SEND_KEY {
            if ((*key).KeyType != DVD_CHALLENGE_KEY)
                && ((*key).KeyType != DVD_BUS_KEY2)
                && ((*key).KeyType != DVD_INVALIDATE_AGID)
            {
                status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
            }
        } else if ((*key).KeyType != DVD_CHALLENGE_KEY)
            && ((*key).KeyType != DVD_BUS_KEY1)
            && ((*key).KeyType != DVD_BUS_KEY2)
            && ((*key).KeyType != DVD_TITLE_KEY)
            && ((*key).KeyType != DVD_ASF)
            && ((*key).KeyType != DVD_SET_RPC_KEY)
            && ((*key).KeyType != DVD_GET_RPC_KEY)
            && ((*key).KeyType != DVD_DISK_KEY)
            && ((*key).KeyType != DVD_INVALIDATE_AGID)
        {
            status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
        } else if (*DeviceExtension).DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD {
            status = ntkrnl_sys::constants::STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateGetConfiguration(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    *DataLength = 0;
    if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < core::mem::size_of::<GET_CONFIGURATION_HEADER>() {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = core::mem::size_of::<GET_CONFIGURATION_HEADER>();
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > 0xffff {
        status = STATUS_INVALID_BUFFER_SIZE;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize) != 0 {
        status = STATUS_INVALID_PARAMETER;
    }

    if status >= 0 {
        #[cfg(all(windows, target_pointer_width = "64", feature = "wow64"))]
        {
            if WdfRequestIsFrom32BitProcess(Request) != 0 {
                let mut inputBuffer: *mut GET_CONFIGURATION_IOCTL_INPUT32 = core::ptr::null_mut();
                if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize) < core::mem::size_of::<GET_CONFIGURATION_IOCTL_INPUT32>() {
                    status = STATUS_INFO_LENGTH_MISMATCH;
                }
                if status >= 0 {
                    status = WdfRequestRetrieveInputBuffer(
                        Request,
                        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                        &mut inputBuffer as *mut *mut GET_CONFIGURATION_IOCTL_INPUT32 as *mut *mut core::ffi::c_void,
                        core::ptr::null_mut(),
                    );
                }
                if status >= 0 {
                    if (*inputBuffer).Feature > 0xffff {
                        status = STATUS_INVALID_PARAMETER;
                    } else if ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
                              ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT) &&
                              ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL) {
                        status = STATUS_INVALID_PARAMETER;
                    } else if (*inputBuffer).Reserved[0] != 0 || (*inputBuffer).Reserved[1] != 0 {
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
            } else {
                let mut inputBuffer: *mut GET_CONFIGURATION_IOCTL_INPUT = core::ptr::null_mut();
                if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize) < core::mem::size_of::<GET_CONFIGURATION_IOCTL_INPUT>() {
                    status = STATUS_INFO_LENGTH_MISMATCH;
                }
                if status >= 0 {
                    status = WdfRequestRetrieveInputBuffer(
                        Request,
                        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                        &mut inputBuffer as *mut *mut GET_CONFIGURATION_IOCTL_INPUT as *mut *mut core::ffi::c_void,
                        core::ptr::null_mut(),
                    );
                }
                if status >= 0 {
                    if (*inputBuffer).Feature > 0xffff {
                        status = STATUS_INVALID_PARAMETER;
                    } else if ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
                              ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT) &&
                              ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL) {
                        status = STATUS_INVALID_PARAMETER;
                    } else if (*inputBuffer).Reserved[0] != 0 || (*inputBuffer).Reserved[1] != 0 {
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
            }
        }
        #[cfg(not(all(windows, target_pointer_width = "64", feature = "wow64")))]
        {
            let mut inputBuffer: *mut GET_CONFIGURATION_IOCTL_INPUT = core::ptr::null_mut();
            if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize) < core::mem::size_of::<GET_CONFIGURATION_IOCTL_INPUT>() {
                status = STATUS_INFO_LENGTH_MISMATCH;
            }
            if status >= 0 {
                status = WdfRequestRetrieveInputBuffer(
                    Request,
                    RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                    &mut inputBuffer as *mut *mut GET_CONFIGURATION_IOCTL_INPUT as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );
            }
            if status >= 0 {
                if (*inputBuffer).Feature > 0xffff {
                    status = STATUS_INVALID_PARAMETER;
                } else if ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
                          ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT) &&
                          ((*inputBuffer).RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL) {
                    status = STATUS_INVALID_PARAMETER;
                } else if !(*inputBuffer).Reserved[0].is_null() || !(*inputBuffer).Reserved[1].is_null() {
                    status = STATUS_INVALID_PARAMETER;
                }
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateSetSpeed(
    device_extension: *mut CDROM_DEVICE_EXTENSION,
    request: WDFREQUEST,
    request_parameters: WDF_REQUEST_PARAMETERS,
    data_length: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let cd_data: *mut CDROM_DATA = &mut (*device_extension).DeviceAdditionalData as *mut CDROM_DATA;
    let mut input_buffer: *mut CDROM_SET_SPEED = core::ptr::null_mut();
    let mut required_length: u32 = 0;

    *data_length = 0;

    if request_parameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < core::mem::size_of::<CDROM_SET_SPEED>() {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }
    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            request,
            core::mem::size_of::<CDROM_SET_SPEED>(),
            &mut input_buffer as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }
    if status >= 0 {
        if (*input_buffer).RequestType > CdromSetStreaming {
            status = STATUS_INVALID_PARAMETER;
        } else if (*input_buffer).RequestType == CdromSetSpeed {
            required_length = core::mem::size_of::<CDROM_SET_SPEED>() as u32;
        } else {
            if (*cd_data).Mmc.IsMmc == 0 {
                status = STATUS_INVALID_DEVICE_REQUEST;
            }
            required_length = core::mem::size_of::<CDROM_SET_STREAMING>() as u32;
        }
    }
    if status >= 0 {
        if request_parameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < required_length as usize {
            status = STATUS_INFO_LENGTH_MISMATCH;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsReadMediaKeyBlock(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut layerNumber: *mut AACS_LAYER_NUMBER = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize
        != core::mem::size_of::<AACS_LAYER_NUMBER>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) < 8 {
        *DataLength = 8;
        status = STATUS_BUFFER_TOO_SMALL;
    }

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut layerNumber as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        if *layerNumber > 255 {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsStartSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        < core::mem::size_of::<DVD_SESSION_ID>()
    {
        *DataLength = core::mem::size_of::<DVD_SESSION_ID>();
        status = STATUS_BUFFER_TOO_SMALL;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        > core::mem::size_of::<DVD_SESSION_ID>()
    {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsSendCertificate(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let cdData = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut inputBuffer: *mut AACS_SEND_CERTIFICATE = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else {
        let input_len = RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength;
        if input_len as usize != core::mem::size_of::<AACS_SEND_CERTIFICATE>() {
            status = STATUS_INFO_LENGTH_MISMATCH;
        }
    }

    if NT_SUCCESS(status) {
        let mut raw_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &raw mut raw_ptr,
            core::ptr::null_mut(),
        );
        inputBuffer = raw_ptr as *mut AACS_SEND_CERTIFICATE;
    }

    if NT_SUCCESS(status) {
        if (*inputBuffer).SessionId as u32 > MAX_COPY_PROTECT_AGID {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsGetCertificate(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();
    let _ = &RequestParameters; // UNREFERENCED_PARAMETER

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize != core::mem::size_of::<DVD_SESSION_ID>() {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) < core::mem::size_of::<AACS_CERTIFICATE>() {
        *DataLength = core::mem::size_of::<AACS_CERTIFICATE>();
        status = STATUS_BUFFER_TOO_SMALL;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) > core::mem::size_of::<AACS_CERTIFICATE>() {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut sessionId as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if (*sessionId) as u32 > 3 /* MAX_COPY_PROTECT_AGID */ {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsGetChallengeKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize
        != core::mem::size_of::<DVD_SESSION_ID>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        < core::mem::size_of::<AACS_CHALLENGE_KEY>()
    {
        *DataLength = core::mem::size_of::<AACS_CHALLENGE_KEY>();
        status = STATUS_BUFFER_TOO_SMALL;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        > core::mem::size_of::<AACS_CHALLENGE_KEY>()
    {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut sessionId as *mut *mut DVD_SESSION_ID as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if *sessionId > MAX_COPY_PROTECT_AGID {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsSendChallengeKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    const MAX_COPY_PROTECT_AGID: u32 = 0x4;  // was 1 (LLM-invented); bindgen-correct

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut inputBuffer: *mut AACS_SEND_CHALLENGE_KEY = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
        != core::mem::size_of::<AACS_SEND_CHALLENGE_KEY>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut inputBuffer as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if (*inputBuffer).SessionId as u32 > MAX_COPY_PROTECT_AGID {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsReadVolumeId(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize
        != core::mem::size_of::<DVD_SESSION_ID>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        < core::mem::size_of::<AACS_VOLUME_ID>()
    {
        *DataLength = core::mem::size_of::<AACS_VOLUME_ID>();
        status = STATUS_BUFFER_TOO_SMALL;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        > core::mem::size_of::<AACS_VOLUME_ID>()
    {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            core::ptr::addr_of_mut!(sessionId) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if (*sessionId) as u32 > MAX_COPY_PROTECT_AGID {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsReadSerialNumber(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData) as *mut CDROM_DATA;
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize
        != core::mem::size_of::<DVD_SESSION_ID>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        < core::mem::size_of::<AACS_SERIAL_NUMBER>()
    {
        *DataLength = core::mem::size_of::<AACS_SERIAL_NUMBER>();
        status = STATUS_BUFFER_TOO_SMALL;
    } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
        > core::mem::size_of::<AACS_SERIAL_NUMBER>()
    {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            core::ptr::addr_of_mut!(sessionId) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        if *sessionId > 3 /* MAX_COPY_PROTECT_AGID */ {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsReadMediaId(
    device_extension: *mut CDROM_DEVICE_EXTENSION,
    request: WDFREQUEST,
    request_parameters: WDF_REQUEST_PARAMETERS,
    data_length: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cd_data: *mut CDROM_DATA = core::ptr::addr_of_mut!((*device_extension).DeviceAdditionalData);
    let mut session_id: *mut DVD_SESSION_ID = core::ptr::null_mut();

    *data_length = 0;

    if (*cd_data).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if request_parameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as u32 != core::mem::size_of::<DVD_SESSION_ID>() as u32 {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if (request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as u32) < (core::mem::size_of::<AACS_MEDIA_ID>() as u32) {
        *data_length = core::mem::size_of::<AACS_MEDIA_ID>() as usize;
        status = STATUS_BUFFER_TOO_SMALL;
    } else if (request_parameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as u32) > (core::mem::size_of::<AACS_MEDIA_ID>() as u32) {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            request,
            request_parameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut session_id as *mut *mut DVD_SESSION_ID as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if *session_id > MAX_COPY_PROTECT_AGID {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsBindingNonce(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA =
        core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut inputBuffer: *mut AACS_READ_BINDING_NONCE = core::ptr::null_mut();

    *DataLength = 0;

    if (*cdData).Mmc.IsAACS == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize
        != core::mem::size_of::<AACS_READ_BINDING_NONCE>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
        < core::mem::size_of::<AACS_BINDING_NONCE>()
    {
        *DataLength = core::mem::size_of::<AACS_BINDING_NONCE>();
        status = STATUS_BUFFER_TOO_SMALL;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
        > core::mem::size_of::<AACS_BINDING_NONCE>()
    {
        status = STATUS_INVALID_BUFFER_SIZE;
    }

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &raw mut inputBuffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        if (*inputBuffer).SessionId > MAX_COPY_PROTECT_AGID {
            status = STATUS_INVALID_PARAMETER;
        } else if (*inputBuffer).NumberOfSectors > 255 {
            status = STATUS_INVALID_PARAMETER;
        } else if (*inputBuffer).StartLba > MAXULONG as u64 {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestValidateExclusiveAccess(
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // UNREFERENCED_PARAMETER(RequestParameters); // not needed in Rust

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut exclusiveAccess: *mut CDROM_EXCLUSIVE_ACCESS = core::ptr::null_mut();

    *DataLength = 0;

    if KeGetCurrentIrql() != PASSIVE_LEVEL {
        TracePrint!(
            2, // TRACE_LEVEL_ERROR
            0x20, // TRACE_FLAG_IOCTL
            b"RequestValidateExclusiveAccess: IOCTL must be called at passive level.\n\0"
                .as_ptr()
                as *const i8,
        );
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
        < core::mem::size_of::<CDROM_EXCLUSIVE_ACCESS>()
    {
        TracePrint!(
            2, // TRACE_LEVEL_ERROR
            0x20, // TRACE_FLAG_IOCTL
            b"RequestValidateExclusiveAccess: Input buffer too small\n\0"
                .as_ptr()
                as *const i8,
        );
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if status >= 0 { // NT_SUCCESS
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            &mut exclusiveAccess as *mut *mut CDROM_EXCLUSIVE_ACCESS as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 { // NT_SUCCESS
        match (*exclusiveAccess).RequestType {
            0 => { // ExclusiveAccessQueryState
                if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                    < core::mem::size_of::<CDROM_EXCLUSIVE_LOCK_STATE>()
                {
                    // Output buffer too small.
                    TracePrint!(
                        2, // TRACE_LEVEL_ERROR
                        0x20, // TRACE_FLAG_IOCTL
                        b"RequestValidateExclusiveAccess: Output buffer too small\n\0"
                            .as_ptr()
                            as *const i8,
                    );
                    *DataLength = core::mem::size_of::<CDROM_EXCLUSIVE_LOCK_STATE>();
                    status = STATUS_BUFFER_TOO_SMALL;
                }
            }

            1 => { // ExclusiveAccessLockDevice
                if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                    < core::mem::size_of::<CDROM_EXCLUSIVE_LOCK>()
                {
                    // Input buffer too small
                    TracePrint!(
                        2, // TRACE_LEVEL_ERROR
                        0x20, // TRACE_FLAG_IOCTL
                        b"RequestValidateExclusiveAccess: Input buffer too small\n\0"
                            .as_ptr()
                            as *const i8,
                    );
                    status = STATUS_INFO_LENGTH_MISMATCH;
                }
            }

            2 => { // ExclusiveAccessUnlockDevice
                // Nothing to check
            }

            _ => {
                // Unknown request type.
                TracePrint!(
                    2, // TRACE_LEVEL_ERROR
                    0x20, // TRACE_FLAG_IOCTL
                    b"RequestValidateExclusiveAccess: Invalid request type\n\0"
                        .as_ptr()
                        as *const i8,
                );
                status = STATUS_INVALID_PARAMETER;
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleExclusiveAccessQueryLockState(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let deviceExtension = DeviceGetExtension(Device);
    let cdData = &mut (*deviceExtension).DeviceAdditionalData as *mut CDROM_DATA;
    let mut exclusiveLockState: *mut CDROM_EXCLUSIVE_LOCK_STATE = core::ptr::null_mut();

    // PAGED_CODE(); – skipped, no effect in Rust

    status = WdfRequestRetrieveOutputBuffer(
        Request,
        core::mem::size_of::<CDROM_EXCLUSIVE_LOCK_STATE>(),
        &mut exclusiveLockState as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );
    ntkrnl_sys::ASSERT!(status >= 0, "NT_ASSERT");

    core::ptr::write_bytes(
        exclusiveLockState as *mut u8,
        0,
        core::mem::size_of::<CDROM_EXCLUSIVE_LOCK_STATE>(),
    );

    if EXCLUSIVE_MODE(cdData) != 0 {
        (*exclusiveLockState).LockState = 1; // TRUE
        core::ptr::copy_nonoverlapping(
            &(*cdData).CallerName as *const _ as *const u8,
            &mut (*exclusiveLockState).CallerName as *mut _ as *mut u8,
            CDROM_EXCLUSIVE_CALLER_LENGTH as usize,
        );
    } else {
        (*exclusiveLockState).LockState = 0; // FALSE
    }

    RequestCompletion(
        deviceExtension,
        Request,
        status,
        (core::mem::size_of::<CDROM_EXCLUSIVE_LOCK_STATE>() as u32) as usize,
    );

    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleExclusiveAccessLockDevice(
    Device: WDFDEVICE,
    Request: WDFREQUEST
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let deviceExtension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*deviceExtension).DeviceAdditionalData);
    let mut exclusiveLock: *mut CDROM_EXCLUSIVE_LOCK = core::ptr::null_mut();
    let mut logEntry: *mut IO_ERROR_LOG_PACKET;

    let mut fileObject: WDFFILEOBJECT = core::ptr::null_mut();
    let mut idx: u32 = 0;
    let mut nameLength: u32 = 0;

    PAGED_CODE();

    fileObject = WdfRequestGetFileObject(Request);

    if fileObject.is_null() {
        status = STATUS_INVALID_HANDLE;
        TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
            b"RequestHandleExclusiveAccessLockDevice: FileObject is NULL, cannot grant exclusive access\n".as_ptr() as *const i8);
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            core::mem::size_of::<CDROM_EXCLUSIVE_LOCK>(),
            &mut exclusiveLock as *mut *mut CDROM_EXCLUSIVE_LOCK as *mut *mut core::ffi::c_void,
            core::ptr::null_mut()
        );
    }

    if NT_SUCCESS(status) {
        idx = 0;
        while idx < CDROM_EXCLUSIVE_CALLER_LENGTH && (*exclusiveLock).CallerName[idx as usize] != b'\0' {
            let ch: u8 = (*exclusiveLock).CallerName[idx as usize];
            if ValidChar(ch) == 0 {
                TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
                    b"RequestHandleExclusiveAccessLockDevice: Invalid characters in caller name\n".as_ptr() as *const i8);
                status = STATUS_INVALID_PARAMETER;
                break;
            }
            idx += 1;
        }
    }

    if NT_SUCCESS(status) {
        if idx == 0 || idx >= CDROM_EXCLUSIVE_CALLER_LENGTH {
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
                b"RequestHandleExclusiveAccessLockDevice: Not a valid null terminated string.\n".as_ptr() as *const i8);
            status = STATUS_INVALID_PARAMETER;
        } else {
            nameLength = idx + 1;
            NT_ASSERT((nameLength <= CDROM_EXCLUSIVE_CALLER_LENGTH) as i32);
        }
    }

    if NT_SUCCESS(status) {
        if ((*exclusiveLock).Access.Flags & CDROM_LOCK_IGNORE_VOLUME) == 0 &&
            IsVolumeMounted((*deviceExtension).DeviceObject as *mut core::ffi::c_void) != 0
        {
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
                b"RequestHandleExclusiveAccessLockDevice: Unable to lock device, file system mounted\n".as_ptr() as *const i8);
            status = STATUS_INVALID_DEVICE_STATE;
        }
    }

    if NT_SUCCESS(status) {
        let oldOwner = InterlockedCompareExchangePointer(
            &mut (*cdData).ExclusiveOwner as *mut _ as *mut *mut core::ffi::c_void,
            fileObject as *mut core::ffi::c_void,
            core::ptr::null_mut()
        );
        if oldOwner.is_null() {
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
                b"RequestHandleExclusiveAccessLockDevice: Entering exclusive mode! Device locked by file object %p\n".as_ptr() as *const i8,
                fileObject);

            // Zero out the CallerName
            core::ptr::write_bytes(&mut (*cdData).CallerName as *mut u8, 0, CDROM_EXCLUSIVE_CALLER_LENGTH as usize);
            core::ptr::copy_nonoverlapping(
                &(*exclusiveLock).CallerName as *const u8,
                &mut (*cdData).CallerName as *mut u8,
                nameLength as usize
            );

            DeviceSendNotification(deviceExtension, &GUID_IO_CDROM_EXCLUSIVE_LOCK, 0, core::ptr::null_mut());

            logEntry = IoAllocateErrorLogEntry(
                (*deviceExtension).DeviceObject as *mut core::ffi::c_void,
                (core::mem::size_of::<IO_ERROR_LOG_PACKET>() + CDROM_EXCLUSIVE_CALLER_LENGTH as usize) as u32
            );

            if !logEntry.is_null() {
                let dumpDataPtr: *mut u8 = (*logEntry).DumpData.as_mut_ptr() as *mut u8;

                (*logEntry).FinalStatus       = STATUS_SUCCESS;
                (*logEntry).ErrorCode         = IO_CDROM_EXCLUSIVE_LOCK as i32;
                (*logEntry).SequenceNumber    = 0;
                (*logEntry).MajorFunctionCode = IRP_MJ_DEVICE_CONTROL;
                (*logEntry).IoControlCode     = IOCTL_CDROM_EXCLUSIVE_ACCESS;
                (*logEntry).RetryCount        = 0;
                (*logEntry).UniqueErrorValue  = 0x1;
                (*logEntry).DumpDataSize      = CDROM_EXCLUSIVE_CALLER_LENGTH as u16;

                core::ptr::copy_nonoverlapping(
                    &(*cdData).CallerName as *const u8,
                    dumpDataPtr,
                    CDROM_EXCLUSIVE_CALLER_LENGTH as usize
                );

                IoWriteErrorLogEntry(logEntry);
            }
        } else {
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
                b"RequestHandleExclusiveAccessLockDevice: Unable to lock device, device already locked.\n".as_ptr() as *const i8);
            status = STATUS_ACCESS_DENIED;
        }
    }

    RequestCompletion(deviceExtension, Request, status, 0);
    return status;
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleExclusiveAccessUnlockDevice(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> NTSTATUS {
    // PAGED_CODE(); -- omitted; no equivalent in Rust, pageability is handled by the caller

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let deviceExtension = DeviceGetExtension(Device);
    let mut exclusiveAccess: *mut CDROM_EXCLUSIVE_ACCESS = core::ptr::null_mut();
    let fileObject = WdfRequestGetFileObject(Request);

    if fileObject.is_null() {
        status = STATUS_INVALID_HANDLE;
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_IOCTL,
            b"RequestHandleExclusiveAccessUnlockDevice: FileObject is NULL, cannot release exclusive access\n\0".as_ptr() as *const i8,
        );
    }

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            core::mem::size_of::<*mut CDROM_EXCLUSIVE_ACCESS>(),
            core::ptr::addr_of_mut!(exclusiveAccess) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        let noMediaFlag: i32 =
            if ((*exclusiveAccess).Flags & CDROM_NO_MEDIA_NOTIFICATIONS) != 0 { 1 } else { 0 };
        status = DeviceUnlockExclusive(deviceExtension, fileObject, noMediaFlag);
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"RequestHandleExclusiveAccessUnlockDevice: Device unlocked\n\0".as_ptr() as *const i8,
        );
    }

    RequestCompletion(deviceExtension, Request, status, 0);

    status
}
#[no_mangle]
// i386 ABI: declared extern "system" (stdcall) at cdrom.rs:91 block / called at cdrom.rs:2644.
// Definition must match the decl or i386 link-fails on the `@16` decoration (cdecl def emits no
// `@N`). Latent until arm64-love's CTL_CODE const fixes kept this call path alive past DCE.
pub unsafe extern "system" fn RequestHandleQueryPropertyRetrieveCachedData(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: *mut WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    const StorageDeviceProperty: u32 = 0;
    const StorageAdapterProperty: u32 = 1;
    const INVALID_OFFSET: u32 = 0xFFFFFFFF;

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut inputBuffer: *mut STORAGE_PROPERTY_QUERY = core::ptr::null_mut();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        (*RequestParameters).Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &mut inputBuffer as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        if (*inputBuffer).PropertyId == StorageDeviceProperty {
            let out_buf_len = (*RequestParameters).Parameters.DeviceIoControl.as_ref().OutputBufferLength;

            if out_buf_len == 0 {
                *DataLength = 0;
                status = STATUS_SUCCESS;
            } else if out_buf_len < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() {
                *DataLength = (*(*DeviceExtension).DeviceDescriptor).Size as usize;
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let mut outputDescriptor: *mut STORAGE_DEVICE_DESCRIPTOR = core::ptr::null_mut();
                let device_desc_ptr = (*DeviceExtension).DeviceDescriptor;

                status = WdfRequestRetrieveOutputBuffer(
                    Request,
                    out_buf_len,
                    &mut outputDescriptor as *mut _ as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(status) {
                    let dev_size = (*device_desc_ptr).Size as usize;
                    let copy_len = if (out_buf_len as usize) < dev_size { out_buf_len as usize } else { dev_size };
                    *DataLength = copy_len;

                    core::ptr::copy_nonoverlapping(
                        device_desc_ptr as *const u8,
                        outputDescriptor as *mut u8,
                        copy_len,
                    );

                    let local_desc_buf = device_desc_ptr as *mut u8;

                    // VendorIdOffset adjustment
                    if *DataLength >= core::mem::offset_of!(STORAGE_DEVICE_DESCRIPTOR, VendorIdOffset) + core::mem::size_of::<u32>()
                        && (*device_desc_ptr).VendorIdOffset != 0
                        && (*device_desc_ptr).VendorIdOffset != INVALID_OFFSET
                    {
                        let offset = (*device_desc_ptr).VendorIdOffset as usize;
                        let ptr = local_desc_buf.add(offset);
                        let mut len = 0usize;
                        while *ptr.add(len) != 0 {
                            len += 1;
                        }
                        if *DataLength < offset + len {
                            (*outputDescriptor).VendorIdOffset = 0;
                        }
                    }

                    // ProductIdOffset adjustment
                    if *DataLength >= core::mem::offset_of!(STORAGE_DEVICE_DESCRIPTOR, ProductIdOffset) + core::mem::size_of::<u32>()
                        && (*device_desc_ptr).ProductIdOffset != 0
                        && (*device_desc_ptr).ProductIdOffset != INVALID_OFFSET
                    {
                        let offset = (*device_desc_ptr).ProductIdOffset as usize;
                        let ptr = local_desc_buf.add(offset);
                        let mut len = 0usize;
                        while *ptr.add(len) != 0 {
                            len += 1;
                        }
                        if *DataLength < offset + len {
                            (*outputDescriptor).ProductIdOffset = 0;
                        }
                    }

                    // ProductRevisionOffset adjustment
                    if *DataLength >= core::mem::offset_of!(STORAGE_DEVICE_DESCRIPTOR, ProductRevisionOffset) + core::mem::size_of::<u32>()
                        && (*device_desc_ptr).ProductRevisionOffset != 0
                        && (*device_desc_ptr).ProductRevisionOffset != INVALID_OFFSET
                    {
                        let offset = (*device_desc_ptr).ProductRevisionOffset as usize;
                        let ptr = local_desc_buf.add(offset);
                        let mut len = 0usize;
                        while *ptr.add(len) != 0 {
                            len += 1;
                        }
                        if *DataLength < offset + len {
                            (*outputDescriptor).ProductRevisionOffset = 0;
                        }
                    }

                    // SerialNumberOffset adjustment
                    if *DataLength >= core::mem::offset_of!(STORAGE_DEVICE_DESCRIPTOR, SerialNumberOffset) + core::mem::size_of::<u32>()
                        && (*device_desc_ptr).SerialNumberOffset != 0
                        && (*device_desc_ptr).SerialNumberOffset != INVALID_OFFSET
                    {
                        let offset = (*device_desc_ptr).SerialNumberOffset as usize;
                        let ptr = local_desc_buf.add(offset);
                        let mut len = 0usize;
                        while *ptr.add(len) != 0 {
                            len += 1;
                        }
                        if *DataLength < offset + len {
                            (*outputDescriptor).SerialNumberOffset = 0;
                        }
                    }

                    status = STATUS_SUCCESS;
                }
            }
        } else if (*inputBuffer).PropertyId == StorageAdapterProperty {
            let out_buf_len = (*RequestParameters).Parameters.DeviceIoControl.as_ref().OutputBufferLength;

            if out_buf_len == 0 {
                *DataLength = 0;
                status = STATUS_SUCCESS;
            } else if out_buf_len < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() {
                *DataLength = (*(*DeviceExtension).AdapterDescriptor).Size as usize;
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let mut outputDescriptor: *mut STORAGE_ADAPTER_DESCRIPTOR = core::ptr::null_mut();

                status = WdfRequestRetrieveOutputBuffer(
                    Request,
                    out_buf_len,
                    &mut outputDescriptor as *mut _ as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(status) {
                    let adapter_desc_ptr = (*DeviceExtension).AdapterDescriptor;
                    let dev_size = (*adapter_desc_ptr).Size as usize;
                    let copy_len = if (out_buf_len as usize) < dev_size { out_buf_len as usize } else { dev_size };
                    *DataLength = copy_len;

                    core::ptr::copy_nonoverlapping(
                        adapter_desc_ptr as *const u8,
                        outputDescriptor as *mut u8,
                        copy_len,
                    );

                    status = STATUS_SUCCESS;
                }
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleQueryPropertyDeviceUniqueId(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> i32 {
    let deviceExtension = DeviceGetExtension(Device);
    let mut status: i32 = STATUS_SUCCESS;
    let mut inputBuffer: *mut STORAGE_PROPERTY_QUERY = core::ptr::null_mut();
    let mut descHeader: *mut STORAGE_DESCRIPTOR_HEADER = core::ptr::null_mut();
    let mut outLength: usize = 0;
    let mut requestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();

    WDF_REQUEST_PARAMETERS_INIT(&mut requestParameters);
    WdfRequestGetParameters(Request, &mut requestParameters);

    status = WdfRequestRetrieveInputBuffer(
        Request,
        requestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut inputBuffer as *mut *mut STORAGE_PROPERTY_QUERY as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        let mut overflow: u8 = 0;
        let mut infoFound: u8 = 0;

        if KeGetCurrentIrql() >= DISPATCH_LEVEL {
            NT_ASSERT(0i32);
            outLength = 0;
            status = STATUS_INVALID_LEVEL;
        } else if (*inputBuffer).QueryType == PropertyExistsQuery {
            outLength = 0;
            status = STATUS_SUCCESS;
        } else if (*inputBuffer).QueryType != PropertyStandardQuery {
            outLength = 0;
            status = STATUS_NOT_SUPPORTED;
        } else {
            if (*inputBuffer).AdditionalParameters[0] as u32 == DUID_INCLUDE_SOFTWARE_IDS {
                // Do nothing
            } else if (*inputBuffer).AdditionalParameters[0] as u32 == DUID_HARDWARE_IDS_ONLY {
                // Do nothing
            } else {
                outLength = 0;
                status = STATUS_INVALID_PARAMETER;
            }
        }

        if status >= 0 && outLength < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() {
            outLength = 0;
            status = STATUS_INFO_LENGTH_MISMATCH;
        }

        if status >= 0 {
            outLength = requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize;
            status = WdfRequestRetrieveOutputBuffer(
                Request,
                requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
                &mut descHeader as *mut *mut STORAGE_DESCRIPTOR_HEADER as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
        }

        if status >= 0 {
            RtlZeroMemory(descHeader as *mut core::ffi::c_void, outLength);

            (*descHeader).Version = DUID_VERSION_1;
            (*descHeader).Size = core::mem::size_of::<STORAGE_DEVICE_UNIQUE_IDENTIFIER>() as u32;

            status = RequestDuidGetDeviceIdProperty(
                deviceExtension,
                Request,
                core::ptr::read(&requestParameters),
                &mut outLength,
            );

            if status == STATUS_BUFFER_OVERFLOW {
                overflow = 1;
            }

            if status >= 0 {
                infoFound = 1;
            }

            status = RequestDuidGetDeviceProperty(
                deviceExtension,
                Request,
                requestParameters,
                &mut outLength,
            );

            if status == STATUS_BUFFER_OVERFLOW {
                overflow = 1;
            }

            if status >= 0 {
                infoFound = 1;
            }

            if overflow != 0 {
                if outLength == core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() {
                    status = STATUS_SUCCESS;
                } else {
                    outLength = WdfRequestGetInformation(Request) as usize;
                    status = STATUS_BUFFER_OVERFLOW;
                }
            } else if infoFound != 0 {
                status = STATUS_SUCCESS;

                NT_ASSERT(
                    (DuidExactMatch
                        == CompareStorageDuids(
                            descHeader as *const STORAGE_DEVICE_UNIQUE_IDENTIFIER,
                            descHeader as *const STORAGE_DEVICE_UNIQUE_IDENTIFIER,
                        )) as i32,
                );
            } else {
                status = STATUS_NOT_FOUND;
            }
        }
    }

    RequestCompletion(deviceExtension, Request, status, (outLength as u32) as usize);

    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleQueryPropertyWriteCache(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let mut query: *mut STORAGE_PROPERTY_QUERY = core::ptr::null_mut();
    let mut writeCache: *mut STORAGE_WRITE_CACHE_PROPERTY = core::ptr::null_mut();
    let mut modeData: *mut MODE_PARAMETER_HEADER = core::ptr::null_mut();
    let mut pageData: *mut MODE_CACHING_PAGE = core::ptr::null_mut();
    let mut length: usize = 0;
    let mut information: u32 = 0;
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut requestParameters: WDF_REQUEST_PARAMETERS = core::mem::zeroed();

    // WDF_REQUEST_PARAMETERS_INIT also sets Size. The "expanded" zero-fill this replaced
    // left Size at 0, and WdfRequestGetParameters returns without filling anything when
    // Size is wrong: the write-cache property query always saw zero-length buffers.
    WDF_REQUEST_PARAMETERS_INIT(&mut requestParameters);

    WdfRequestGetParameters(Request, &mut requestParameters);

    status = WdfRequestRetrieveInputBuffer(
        Request,
        requestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut query as *mut *mut STORAGE_PROPERTY_QUERY as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        if KeGetCurrentIrql() >= 2u8 /* DISPATCH_LEVEL */ {
            NT_ASSERT(0i32);
            status = STATUS_INVALID_LEVEL;
        } else if (*query).QueryType == 0 /* PropertyExistsQuery */ {
            information = 0;
            status = 0; // STATUS_SUCCESS
        } else if (*query).QueryType != 1 /* PropertyStandardQuery */ {
            status = STATUS_NOT_SUPPORTED;
        }
    }

    if status >= 0 {
        length = requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize;

        if length < core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() {
            status = (0xC0000004u32 as i32); // STATUS_INFO_LENGTH_MISMATCH
        }
    }

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            &mut writeCache as *mut *mut STORAGE_WRITE_CACHE_PROPERTY as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        core::ptr::write_bytes(writeCache as *mut u8, 0, length);

        (*writeCache).Version = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;
        (*writeCache).Size = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;

        if length < core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() {
            information = core::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32;
            status = 0; // STATUS_SUCCESS

            RequestCompletion(deviceExtension, Request, status, (information) as usize);
            return status;
        }
    }

    if status >= 0 {
        srb = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>() + (core::mem::size_of::<usize>() * 2),
            CDROM_TAG_SRB,
        ) as *mut SCSI_REQUEST_BLOCK;

        if srb.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if status >= 0 {
        core::ptr::write_bytes(srb as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        (*writeCache).NVCacheEnabled = 0; // FALSE
        (*writeCache).UserDefinedPowerProtection =
            if ((*deviceExtension).DeviceFlags & DEV_POWER_PROTECTED as u16) != 0 { 1 } else { 0 };

        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
        (*srb).TimeOutValue =
            TimeOutValueGetCapValue((*deviceExtension).TimeOutValue, 4);
        (*srb).QueueTag = SP_UNTAGGED;
        (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST;
        (*srb).SrbFlags = (*deviceExtension).SrbFlags;

        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*srb).CdbLength = 10;

        (*srb).Cdb[0] = SCSIOP_SYNCHRONIZE_CACHE as u8;

        status = DeviceSendSrbSynchronously(
            Device as *mut core::ffi::c_void,
            srb,
            core::ptr::null_mut(),
            0,
            1, // TRUE -> flush drive cache
            (Request) as _,
        );

        if status >= 0 {
            (*writeCache).FlushCacheSupported = 1; // TRUE
        } else {
            // Device does not support sync cache
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestHandleQueryPropertyWriteCache: Synchronize cache failed with status 0x%X\n\0".as_ptr() as *const i8,
                status,
            );
            (*writeCache).FlushCacheSupported = 0; // FALSE

            // Reset the status if there was any failure
            status = 0;
        }

        modeData = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            MODE_PAGE_DATA_SIZE as usize,
            CDROM_TAG_MODE_DATA,
        ) as *mut MODE_PARAMETER_HEADER;

        if modeData.is_null() {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestHandleQueryPropertyWriteCache: Unable to allocate mode data buffer\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if status >= 0 {
        core::ptr::write_bytes(modeData as *mut u8, 0, MODE_PAGE_DATA_SIZE as usize);

        length = DeviceRetrieveModeSenseUsingScratch(
            deviceExtension,
            modeData as *mut i8,
            MODE_PAGE_DATA_SIZE as usize,
            (MODE_PAGE_CACHING) as _,
            (MODE_SENSE_CURRENT_VALUES) as _,
        );

        if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
            // Retry
            length = DeviceRetrieveModeSenseUsingScratch(
                deviceExtension,
                modeData as *mut i8,
                MODE_PAGE_DATA_SIZE as usize,
                (MODE_PAGE_CACHING) as _,
                (MODE_SENSE_CURRENT_VALUES) as _,
            );

            if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_IOCTL,
                    b"RequestHandleQueryPropertyWriteCache: Mode Sense failed\n\0".as_ptr() as *const i8,
                );
                status = (0xC0000185u32 as i32); // STATUS_IO_DEVICE_ERROR
            }
        }
    }

    if status >= 0 {
        if length > (*modeData).ModeDataLength as usize + 1usize {
            length = (*modeData).ModeDataLength as usize + 1usize;
        }

        pageData = ModeSenseFindSpecificPage(
            modeData as *mut i8,
            length,
            (MODE_PAGE_CACHING) as _,
            1, // TRUE
        );

        if pageData.is_null() {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestHandleQueryPropertyWriteCache: Unable to find caching mode page.\n\0".as_ptr() as *const i8,
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

        if (*modeData).DeviceSpecificParameter & MODE_DSP_FUA_SUPPORTED as u8 != 0 {
            (*writeCache).WriteThroughSupported = WriteThroughSupported;
        } else {
            (*writeCache).WriteThroughSupported = WriteThroughNotSupported;
        }

        // Get changeable caching mode page
        core::ptr::write_bytes(modeData as *mut u8, 0, MODE_PAGE_DATA_SIZE as usize);

        length = DeviceRetrieveModeSenseUsingScratch(
            deviceExtension,
            modeData as *mut i8,
            MODE_PAGE_DATA_SIZE as usize,
            (MODE_PAGE_CACHING) as _,
            (MODE_SENSE_CHANGEABLE_VALUES) as _,
        );

        if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
            // Retry
            length = DeviceRetrieveModeSenseUsingScratch(
                deviceExtension,
                modeData as *mut i8,
                MODE_PAGE_DATA_SIZE as usize,
                (MODE_PAGE_CACHING) as _,
                (MODE_SENSE_CHANGEABLE_VALUES) as _,
            );

            if length < core::mem::size_of::<MODE_PARAMETER_HEADER>() {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_IOCTL,
                    b"RequestHandleQueryPropertyWriteCache: Mode Sense failed\n\0".as_ptr() as *const i8,
                );

                (*writeCache).WriteCacheChangeable = WriteCacheChangeUnknown;
                information = core::mem::size_of::<STORAGE_WRITE_CACHE_PROPERTY>() as u32;
            }
        }
    }

    if status >= 0 {
        if length > (*modeData).ModeDataLength as usize + 1usize {
            length = (*modeData).ModeDataLength as usize + 1usize;
        }

        pageData = ModeSenseFindSpecificPage(
            modeData as *mut i8,
            length,
            (MODE_PAGE_CACHING) as _,
            1, // TRUE
        );

        if pageData.is_null() {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"RequestHandleQueryPropertyWriteCache: Unable to find caching mode page.\n\0".as_ptr() as *const i8,
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
    }

    if !srb.is_null() { ExFreePool(srb as *mut core::ffi::c_void); }
    if !modeData.is_null() { ExFreePool(modeData as *mut core::ffi::c_void); }

    RequestCompletion(deviceExtension, Request, status, (information) as usize);

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn RequestValidateDvdReadKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut keyParameters: *mut DVD_COPY_PROTECT_KEY = core::ptr::null_mut();
    let mut keyLength: u32 = 0;

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        core::ptr::addr_of_mut!(keyParameters) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize)
            < core::mem::size_of::<DVD_COPY_PROTECT_KEY>()
        {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"DvdDeviceControl: EstablishDriveKey - challenge key buffer too small\n\0"
                    .as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if NT_SUCCESS(status) {
        match (*keyParameters).KeyType {
            DvdChallengeKey => {
                // C_ASSERT omitted (runtime safe)
                keyLength = DVD_CHALLENGE_KEY_LENGTH;
            }
            DvdBusKey1 | DvdBusKey2 => {
                keyLength = DVD_BUS_KEY_LENGTH;
            }
            DvdTitleKey => {
                keyLength = DVD_TITLE_KEY_LENGTH;
            }
            DvdAsf => {
                keyLength = DVD_ASF_LENGTH;
            }
            DvdDiskKey => {
                keyLength = DVD_DISK_KEY_LENGTH;
            }
            DvdGetRpcKey => {
                keyLength = DVD_RPC_KEY_LENGTH as u32;
            }
            _ => {
                keyLength = core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32;
            }
        }

        if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize)
            < (keyLength as usize)
        {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"DvdDeviceControl: EstablishDriveKey - output buffer too small\n\0"
                    .as_ptr() as *const i8,
            );
            status = STATUS_BUFFER_TOO_SMALL;
            *DataLength = keyLength as usize;
        } else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
            & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize)
            != 0
        {
            status = STATUS_INVALID_PARAMETER;
        } else if (*DeviceExtension).DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateDvdEndSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();

    // UNREFERENCED_PARAMETER(DeviceExtension);
    let _ = DeviceExtension;

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &mut sessionId as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if (status as i32) >= 0 {
        if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize)
            < core::mem::size_of::<DVD_SESSION_ID>()
        {
            const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
            const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_IOCTL,
                b"DvdDeviceControl: EndSession - input buffer too small\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateAacsEndSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();
    let cdData: *mut CDROM_DATA = &mut (*DeviceExtension).DeviceAdditionalData;

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut sessionId as *mut *mut DVD_SESSION_ID as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        if (*cdData).Mmc.IsAACS == 0 {
            status = STATUS_INVALID_DEVICE_REQUEST;
        } else if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize != core::mem::size_of::<DVD_SESSION_ID>() {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateEnableStreaming(
    request: WDFREQUEST,
    requestParameters: WDF_REQUEST_PARAMETERS,
    dataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut inputBuffer: *mut CDROM_STREAMING_CONTROL = core::ptr::null_mut();

    *dataLength = 0;

    if requestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
        < core::mem::size_of::<CDROM_STREAMING_CONTROL>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            request,
            core::mem::size_of::<CDROM_STREAMING_CONTROL>(),
            &raw mut inputBuffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        let req_type = (*inputBuffer).RequestType;
        if req_type != CdromStreamingDisable
            && req_type != CdromStreamingEnableForReadOnly
            && req_type != CdromStreamingEnableForWriteOnly
            && req_type != CdromStreamingEnableForReadWrite
        {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateSendOpcInformation(
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut inputBuffer: *mut CDROM_SIMPLE_OPC_INFO = core::ptr::null_mut();

    *DataLength = 0;

    if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize)
        < core::mem::size_of::<CDROM_SIMPLE_OPC_INFO>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            core::mem::size_of::<CDROM_SIMPLE_OPC_INFO>(),
            core::ptr::addr_of_mut!(inputBuffer) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if (*inputBuffer).RequestType != SimpleOpcInfo {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidateGetPerformance(
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut writeSpeedRequest: *mut CDROM_WRITE_SPEED_REQUEST = core::ptr::null_mut();
    let mut performanceRequest: *mut CDROM_PERFORMANCE_REQUEST = core::ptr::null_mut();

    *DataLength = 0;

    // CDROM_WRITE_SPEED_REQUEST is the smallest performance request that we support.
    // We use it to retrieve request type and then check input length more carefully
    // on a per request type basis.
    if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
        < core::mem::size_of::<CDROM_WRITE_SPEED_REQUEST>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            core::mem::size_of::<CDROM_WRITE_SPEED_REQUEST>(),
            core::ptr::addr_of_mut!(writeSpeedRequest) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        if (*writeSpeedRequest).RequestType == CdromPerformanceRequest {
            // CDROM_PERFORMANCE_REQUEST is bigger than CDROM_WRITE_SPEED_REQUEST,
            // so we perform more checks and retrieve more bytes through WDF.
            if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                < core::mem::size_of::<CDROM_PERFORMANCE_REQUEST>()
            {
                status = STATUS_INFO_LENGTH_MISMATCH;
            }
            if NT_SUCCESS(status) {
                status = WdfRequestRetrieveInputBuffer(
                    Request,
                    core::mem::size_of::<CDROM_PERFORMANCE_REQUEST>(),
                    core::ptr::addr_of_mut!(performanceRequest) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );
            }

            if (!NT_SUCCESS(status)) {
                // just pass the status code from above
            }
            // validate all enum-type fields of CDROM_PERFORMANCE_REQUEST
            else if (*performanceRequest).PerformanceType != CdromReadPerformance
                && (*performanceRequest).PerformanceType != CdromWritePerformance
            {
                status = STATUS_INVALID_PARAMETER;
            } else if (*performanceRequest).Exceptions != CdromNominalPerformance
                && (*performanceRequest).Exceptions != CdromEntirePerformanceList
                && (*performanceRequest).Exceptions != CdromPerformanceExceptionsOnly
            {
                status = STATUS_INVALID_PARAMETER;
            } else if (*performanceRequest).Tolerance != Cdrom10Nominal20Exceptions {
                status = STATUS_INVALID_PARAMETER;
            }
        } else if (*writeSpeedRequest).RequestType == CdromWriteSpeedRequest {
            // No additional checks here: all remaining fields are ignored
            // if RequestType == CdromWriteSpeedRequest.
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    // finally, check output buffer length
    if NT_SUCCESS(status) {
        if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
            < core::mem::size_of::<CDROM_PERFORMANCE_HEADER>()
        {
            status = STATUS_BUFFER_TOO_SMALL;
            *DataLength = core::mem::size_of::<CDROM_PERFORMANCE_HEADER>();
        } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
            > (u16::MAX as usize)
        {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn RequestGetScsiPassThroughCdb(Irp: PIRP) -> *mut CDB {

    const IOCTL_SCSI_PASS_THROUGH: u32 = 0x0004D004;
    const IOCTL_SCSI_PASS_THROUGH_DIRECT: u32 = 0x0004D014;
    const IOCTL_SCSI_PASS_THROUGH_EX: u32 = 0x0004D044;
    const IOCTL_SCSI_PASS_THROUGH_DIRECT_EX: u32 = 0x0004D048;

    let mut cdb: *mut CDB = core::ptr::null_mut();

    PAGED_CODE();

    let current_irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let io_control_code = (*current_irp_stack).Parameters.DeviceIoControl.IoControlCode;

    if (io_control_code == IOCTL_SCSI_PASS_THROUGH
        || io_control_code == IOCTL_SCSI_PASS_THROUGH_DIRECT
        || io_control_code == IOCTL_SCSI_PASS_THROUGH_EX
        || io_control_code == IOCTL_SCSI_PASS_THROUGH_DIRECT_EX)
        && !(*Irp).AssociatedIrp.SystemBuffer.is_null()
    {
        let input_buffer_length = (*current_irp_stack).Parameters.DeviceIoControl.InputBufferLength;
        let input_buffer = (*Irp).AssociatedIrp.SystemBuffer;
        let legacy_pass_through = if io_control_code == IOCTL_SCSI_PASS_THROUGH_EX
            || io_control_code == IOCTL_SCSI_PASS_THROUGH_DIRECT_EX
        {
            false
        } else {
            true
        };

        #[cfg(target_pointer_width = "64")]
        {
            if IoIs32bitProcess(Irp) != 0 {
                if legacy_pass_through {
                    if (input_buffer_length as usize) >= core::mem::size_of::<SCSI_PASS_THROUGH32>() {
                        cdb = &raw mut (*(input_buffer as *mut SCSI_PASS_THROUGH32)).Cdb as *mut CDB;
                    }
                } else {
                    if (input_buffer_length as usize) >= core::mem::size_of::<SCSI_PASS_THROUGH32_EX>() {
                        cdb = &raw mut (*(input_buffer as *mut SCSI_PASS_THROUGH32_EX)).Cdb as *mut CDB;
                    }
                }
            } else {
                if legacy_pass_through {
                    if (input_buffer_length as usize) >= core::mem::size_of::<SCSI_PASS_THROUGH>() {
                        cdb = &raw mut (*(input_buffer as *mut SCSI_PASS_THROUGH)).Cdb as *mut CDB;
                    }
                } else {
                    if (input_buffer_length as usize) >= core::mem::size_of::<SCSI_PASS_THROUGH_EX>() {
                        cdb = &raw mut (*(input_buffer as *mut SCSI_PASS_THROUGH_EX)).Cdb as *mut CDB;
                    }
                }
            }
        }

        #[cfg(not(target_pointer_width = "64"))]
        {
            if legacy_pass_through {
                if (input_buffer_length as usize) >= core::mem::size_of::<SCSI_PASS_THROUGH>() {
                    cdb = &raw mut (*(input_buffer as *mut SCSI_PASS_THROUGH)).Cdb as *mut CDB;
                }
            } else {
                if (input_buffer_length as usize) >= core::mem::size_of::<SCSI_PASS_THROUGH_EX>() {
                    cdb = &raw mut (*(input_buffer as *mut SCSI_PASS_THROUGH_EX)).Cdb as *mut CDB;
                }
            }
        }
    }

    cdb
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleScsiPassThrough(
    Device: WDFDEVICE,
    Request: WDFREQUEST,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let device_extension = DeviceGetExtension(Device);
    let irp = WdfRequestWdmGetIrp(Request);
    let zpodd_info = (*device_extension).ZeroPowerODDInfo;
    let mut cdb: PCDB = core::ptr::null_mut();
    let mut is_soft_eject: i32 = 0; // BOOLEAN as i32, 0 = FALSE

    #[cfg(debug_assertions)]
    {
        let request_context = RequestGetContext(Request);
        // NT_ASSERT(request_context->SyncRequired) in C
        // We expand NT_ASSERT as a plain assertion; SyncRequired is a bool field.
        ntkrnl_sys::ASSERT!((*request_context).SyncRequired != 0);
    }

    // PAGED_CODE() expansion: assert IRQL <= APC_LEVEL
    {
        let irql = KeGetCurrentIrql();
        if irql as u32 > APC_LEVEL as u32 {
            RtlAssert(
                b"KeGetCurrentIrql() <= APC_LEVEL\0".as_ptr() as *mut core::ffi::c_void,
                b"ioctl.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                b"PAGED_CODE: RequestHandleScsiPassThrough\0".as_ptr() as *mut i8,
            );
        }
    }

    if !zpodd_info.is_null()
        && (*zpodd_info).LoadingMechanism as u32 == LOADING_MECHANISM_TRAY
        && (*zpodd_info).Load == 0
    {
        cdb = RequestGetScsiPassThroughCdb(irp);
        if !cdb.is_null()
            && (*cdb).AsByte.as_ref()[0] == SCSIOP_START_STOP_UNIT
            && (*cdb).START_STOP.as_ref().LoadEject() == 1
            && (*cdb).START_STOP.as_ref().Start() == 0
        {
            is_soft_eject = 1;
        }
    }

    WdfRequestFormatRequestUsingCurrentType(Request);

    // Set MinorFunction
    {
        let next_stack = IoGetNextIrpStackLocation(irp);
        (*next_stack).MinorFunction = 1;
    }

    status = RequestSend(
        device_extension,
        Request,
        (*device_extension).IoTarget,
        WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
        core::ptr::null_mut(),
    );

    if status < 0 && is_soft_eject != 0 {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_POWER,
            b"RequestHandleScsiPassThrough: soft eject detected, device marked as active\n\0".as_ptr() as *const i8,
        );
        DeviceMarkActive(device_extension, 1, 0); // TRUE, FALSE as i32
    }

    RequestCompletion(
        device_extension,
        Request,
        status,
        (WdfRequestGetInformation(Request) as u32) as usize,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleMountQueryUniqueId(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut uniqueId: *mut MOUNTDEV_UNIQUE_ID = core::ptr::null_mut();

    *DataLength = 0;

    if (*DeviceExtension).MountedDeviceInterfaceName.Buffer.is_null() {
        status = STATUS_INVALID_PARAMETER;
    } else if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
        < core::mem::size_of::<MOUNTDEV_UNIQUE_ID>()
    {
        *DataLength = core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as usize;
        status = STATUS_BUFFER_TOO_SMALL;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &mut uniqueId as *mut *mut MOUNTDEV_UNIQUE_ID as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        RtlZeroMemory(
            uniqueId as *mut core::ffi::c_void,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
        );

        (*uniqueId).UniqueIdLength = (*DeviceExtension).MountedDeviceInterfaceName.Length;

        if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
            < (core::mem::size_of::<u16>()
                + (*DeviceExtension).MountedDeviceInterfaceName.Length as usize)
        {
            *DataLength = core::mem::size_of::<MOUNTDEV_UNIQUE_ID>() as usize;
            status = STATUS_BUFFER_OVERFLOW;
        }
    }

    if NT_SUCCESS(status) {
        RtlCopyMemory(
            (*uniqueId).UniqueId.as_mut_ptr() as *mut core::ffi::c_void,
            (*DeviceExtension).MountedDeviceInterfaceName.Buffer as *const core::ffi::c_void,
            (*uniqueId).UniqueIdLength as usize,
        );

        *DataLength = core::mem::size_of::<u16>() as usize + (*uniqueId).UniqueIdLength as usize;
        status = STATUS_SUCCESS;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleMountQueryDeviceName(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut name: *mut MOUNTDEV_NAME = core::ptr::null_mut();

    *DataLength = 0;

    NT_ASSERT((!(*DeviceExtension).DeviceName.Buffer.is_null()) as i32);

    if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < core::mem::size_of::<MOUNTDEV_NAME>() {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = core::mem::size_of::<MOUNTDEV_NAME>();
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            core::ptr::addr_of_mut!(name) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        core::ptr::write_bytes(
            name as *mut u8,
            0u8,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
        );
        (*name).NameLength = (*DeviceExtension).DeviceName.Length;

        if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
            < (core::mem::size_of::<u16>() + (*DeviceExtension).DeviceName.Length as usize)
        {
            status = STATUS_BUFFER_OVERFLOW;
            *DataLength = core::mem::size_of::<MOUNTDEV_NAME>();
        }
    }

    if NT_SUCCESS(status) {
        let count = (*name).NameLength as usize;
        core::ptr::copy_nonoverlapping(
            (*DeviceExtension).DeviceName.Buffer as *const u8,
            core::ptr::addr_of_mut!((*name).Name) as *mut u8,
            count,
        );

        status = STATUS_SUCCESS;
        *DataLength = core::mem::size_of::<u16>() + count;
    }

    status
}

const CDROM_TAG_STRINGS: u32 = 0x73436353; // placeholder
const RTL_QUERY_REGISTRY_TYPECHECK: u32 = 0x00000100; // xdk/rtltypes.h:24 (was 0x10000)
const RTL_QUERY_REGISTRY_TYPECHECK_SHIFT: u32 = 24; // xdk/rtltypes.h:26 (was 16)
#[no_mangle]
pub unsafe extern "C" fn RequestHandleMountQuerySuggestedLinkName(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_NOT_FOUND;
    let mut suggestedName: *mut MOUNTDEV_SUGGESTED_LINK_NAME = core::ptr::null_mut();
    let mut driveLetterNameBuffer: [u16; 10] = [0u16; 10];
    let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [
        core::mem::zeroed(),
        core::mem::zeroed(),
    ];
    let mut valueName: *mut u16 = core::ptr::null_mut();
    let driveLetterName = &mut core::mem::zeroed::<_UNICODE_STRING>();
    let registry_disk_path: [u16; 31] = [
        '\\' as u16, 'R' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16,
        't' as u16, 'r' as u16, 'y' as u16, '\\' as u16, 'M' as u16, 'a' as u16,
        'c' as u16, 'h' as u16, 'i' as u16, 'n' as u16, 'e' as u16, '\\' as u16,
        'S' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16,
        '\\' as u16, 'D' as u16, 'I' as u16, 'S' as u16, 'K' as u16, 0,
        0,
    ];

    *DataLength = 0;
    return status;

    // Check minimum output buffer size
    if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) <
        core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>()
    {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>();
    }

    let required_length = core::mem::offset_of!(MOUNTDEV_SUGGESTED_LINK_NAME, Name) + 28;
    if NT_SUCCESS(status)
        && (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) < required_length
    {
        *DataLength = required_length;
        status = STATUS_BUFFER_OVERFLOW;
    }

    // Allocate buffer for device name copy
    if NT_SUCCESS(status) {
        valueName = ExAllocatePoolWithTag(
            PagedPool,
            (*DeviceExtension).DeviceName.Length as usize + core::mem::size_of::<u16>(),
            CDROM_TAG_STRINGS, // placeholder
        ) as *mut u16;
        if valueName.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    // Copy device name and null-terminate
    if NT_SUCCESS(status) {
        RtlCopyMemory(
            valueName as *mut _,
            (*DeviceExtension).DeviceName.Buffer as *const _,
            (*DeviceExtension).DeviceName.Length as usize,
        );
        let idx = (*DeviceExtension).DeviceName.Length as usize / core::mem::size_of::<u16>();
        *valueName.add(idx) = 0;

        // Prepare query table
        (*driveLetterName).Buffer = driveLetterNameBuffer.as_mut_ptr();
        (*driveLetterName).MaximumLength = core::mem::size_of_val(&driveLetterNameBuffer) as u16;
        (*driveLetterName).Length = 0;

        queryTable[0].Flags =
            RTL_QUERY_REGISTRY_REQUIRED | RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK;
        queryTable[0].Name = valueName as *const u16; // PCWSTR
        queryTable[0].EntryContext = driveLetterName as *mut _UNICODE_STRING as *mut core::ffi::c_void;
        queryTable[0].DefaultType =
            ((REG_SZ as u32) << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | (REG_NONE as u32);

        status = RtlQueryRegistryValues(
            RTL_REGISTRY_ABSOLUTE,
            registry_disk_path.as_ptr(),
            queryTable.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    // Validate drive letter
    if NT_SUCCESS(status) {
        if ((*driveLetterName).Length == 4)
            && (*driveLetterName).Buffer.read() == '%' as u16
            && (*driveLetterName).Buffer.add(1).read() == ':' as u16
        {
            (*driveLetterName).Buffer.write(0xFF);
        } else if ((*driveLetterName).Length != 4)
            || ((*driveLetterName).Buffer.read() < 'C' as u16) // FirstDriveLetter
            || ((*driveLetterName).Buffer.read() > 'Z' as u16) // LastDriveLetter
            || ((*driveLetterName).Buffer.add(1).read() != ':' as u16)
        {
            status = STATUS_NOT_FOUND;
        }
    }

    // Retrieve output buffer
    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &mut suggestedName as *mut *mut MOUNTDEV_SUGGESTED_LINK_NAME as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    // Fill the output structure
    if NT_SUCCESS(status) {
        RtlZeroMemory(
            suggestedName as *mut _,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
        );
        (*suggestedName).UseOnlyIfThereAreNoOtherLinks = 1; // TRUE
        (*suggestedName).NameLength = 28;

        *DataLength = core::mem::offset_of!(MOUNTDEV_SUGGESTED_LINK_NAME, Name) + 28;

        if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize) < *DataLength {
            *DataLength = core::mem::size_of::<MOUNTDEV_SUGGESTED_LINK_NAME>();
            status = STATUS_BUFFER_OVERFLOW;
        }
    }

    // Finalize suggested name
    if NT_SUCCESS(status) {
        RtlDeleteRegistryValue(
            RTL_REGISTRY_ABSOLUTE,
            registry_disk_path.as_ptr(),
            valueName as *const u16,
        );

        let name_ptr: *mut u16 =
            core::ptr::addr_of_mut!((*suggestedName).Name) as *mut u16;
        let dos_devices: [u16; 12] = [
            '\\' as u16, 'D' as u16, 'o' as u16, 's' as u16,
            'D' as u16, 'e' as u16, 'v' as u16, 'i' as u16,
            'c' as u16, 'e' as u16, 's' as u16, '\\' as u16,
        ];
        core::ptr::copy_nonoverlapping(
            dos_devices.as_ptr(),
            name_ptr,
            12,
        );
        *name_ptr.add(12) = (*driveLetterName).Buffer.read();
        *name_ptr.add(13) = ':' as u16;
    }

    FREE_POOL(valueName as *mut core::ffi::c_void);

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleReadTOC(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    PAGED_CODE();

    *DataLength = 0;

    if DeviceIsPlayActive((*DeviceExtension).Device) != 0 {
        status = STATUS_DEVICE_BUSY;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            core::ptr::addr_of_mut!(outputBuffer),
            core::ptr::null_mut(),
        );
    }

    // handle the request
    if NT_SUCCESS(status) {
        let output_len: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as u32;
        let toc_size: usize = core::mem::size_of::<CDROM_TOC>();
        let transferSize: usize = if (output_len as usize) < toc_size {
            output_len as usize
        } else {
            toc_size
        };

        RtlZeroMemory(outputBuffer, transferSize as usize);

        ScratchBuffer_BeginUse(DeviceExtension);

        let mut cdb: CDB = core::mem::zeroed();
        // Set up the CDB
        if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_CDROM_GET_LAST_SESSION {
            // Set format to return first and last session numbers.
            cdb.READ_TOC.as_mut().set_Format2(CDROM_READ_TOC_EX_FORMAT_SESSION);
        } else {
            // Use MSF addressing
            cdb.READ_TOC.as_mut().set_Msf(1);
        }

        cdb.READ_TOC.as_mut().OperationCode = SCSIOP_READ_TOC;
        cdb.READ_TOC.as_mut().AllocationLength[0] = ((transferSize >> 8) & 0xFF) as u8;
        cdb.READ_TOC.as_mut().AllocationLength[1] = (transferSize & 0xFF) as u8;

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            transferSize as u32,
            1, // TRUE
            core::ptr::addr_of_mut!(cdb),
            10,
        );

        if NT_SUCCESS(status) {
            *DataLength = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize;
            RtlCopyMemory(
                outputBuffer,
                (*DeviceExtension).ScratchContext.ScratchBuffer,
                *DataLength,
            );
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleReadTocEx(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    // PAGED_CODE();
    let mut status: i32 = STATUS_SUCCESS; // STATUS_SUCCESS is i32 0
    let mut input_buffer: *mut CDROM_READ_TOC_EX = core::ptr::null_mut();
    let mut output_buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    *DataLength = 0;

    // Check if play is active
    if DeviceIsPlayActive((*DeviceExtension).Device) != 0 {
        status = STATUS_DEVICE_BUSY;
    }

    // Retrieve input buffer
    if status >= 0 {
        let input_length = RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength;
        status = WdfRequestRetrieveInputBuffer(
            Request,
            input_length,
            &mut input_buffer as *mut *mut CDROM_READ_TOC_EX as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    // Retrieve output buffer
    if status >= 0 {
        let output_length = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            output_length,
            &mut output_buffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    // Handle the request
    if status >= 0 {
        let output_length = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
        let transfer_size = if output_length < MAXUSHORT as usize {
            output_length
        } else {
            MAXUSHORT as usize
        };

        ScratchBuffer_BeginUse(DeviceExtension);

        // Zero out the CDB
        let mut cdb: CDB = core::mem::zeroed();
        core::ptr::write_bytes(&mut cdb as *mut CDB as *mut u8, 0, core::mem::size_of::<CDB>());

        // Fill CDB fields from input buffer
        cdb.READ_TOC.as_mut().OperationCode = SCSIOP_READ_TOC;
        cdb.READ_TOC.as_mut().set_Msf((*input_buffer).Msf());
        cdb.READ_TOC.as_mut().set_Format2((*input_buffer).Format());
        cdb.READ_TOC.as_mut().StartingTrack = (*input_buffer).SessionTrack;
        cdb.READ_TOC.as_mut().AllocationLength[0] = ((transfer_size >> 8) & 0xFF) as u8;
        cdb.READ_TOC.as_mut().AllocationLength[1] = (transfer_size & 0xFF) as u8;

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            transfer_size as u32,
            (1i32) as _, // TRUE
            &mut cdb,
            10u8,
        );

        if status >= 0 {
            let scratch_srb = (*DeviceExtension).ScratchContext.ScratchSrb;
            let data_transfer_length = (*scratch_srb).DataTransferLength;
            if data_transfer_length < 4u32 {
                // MINIMUM_CDROM_READ_TOC_EX_SIZE
                *DataLength = 0;
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                *DataLength = data_transfer_length as usize;
                core::ptr::copy_nonoverlapping(
                    (*DeviceExtension).ScratchContext.ScratchBuffer as *const u8,
                    output_buffer as *mut u8,
                    *DataLength,
                );
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn GetConfigurationDataConversionTypeAllToTypeOne(
    RequestedFeature: FEATURE_NUMBER,
    Srb: *mut SCSI_REQUEST_BLOCK,
    DataLength: *mut usize,
) {
    let mut featureHeader: *mut FEATURE_HEADER = core::ptr::null_mut();
    let mut thisFeature: FEATURE_NUMBER = 0;
    let mut totalLength: u32 = 0;
    let mut featureLength: u32 = 0;
    let mut headerLength: u32 = 0;
    let mut header: *mut GET_CONFIGURATION_HEADER = core::ptr::null_mut();

    // PAGED_CODE();

    *DataLength = 0;

    let size_through_data_length = (core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength) + core::mem::size_of::<u32>()) as u32;

    if (*Srb).DataTransferLength < size_through_data_length {
        return;
    }

    header = (*Srb).DataBuffer as *mut GET_CONFIGURATION_HEADER;

    // REVERSE_BYTES(&totalLength, header->DataLength)
    {
        totalLength = u32::from_be_bytes((*header).DataLength);
    }

    totalLength += size_through_data_length;

    totalLength = core::cmp::min(totalLength, (*Srb).DataTransferLength);

    if totalLength >= (core::mem::size_of::<GET_CONFIGURATION_HEADER>() + core::mem::size_of::<FEATURE_HEADER>()) as u32 {
        featureHeader = ((*Srb).DataBuffer as *mut u8).add(core::mem::size_of::<GET_CONFIGURATION_HEADER>()) as *mut FEATURE_HEADER;

        thisFeature = (((*featureHeader).FeatureCode[0] as u32) << 8) | ((*featureHeader).FeatureCode[1] as u32);

        if thisFeature == RequestedFeature {
            featureLength = core::mem::size_of::<FEATURE_HEADER>() as u32 + (*featureHeader).AdditionalLength as u32;
        }
    }

    totalLength = core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32 + featureLength;

    headerLength = totalLength - size_through_data_length;

    // REVERSE_BYTES(header->DataLength, &headerLength)
    {
        (*header).DataLength = headerLength.to_be_bytes();
    }

    *DataLength = totalLength as usize;
}
#[no_mangle]
pub unsafe extern "C" fn GetConfigurationDataSynthesize(
    InputBuffer: *mut core::ffi::c_void,
    InputBufferSize: u32,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferSize: usize,
    StartingFeature: FEATURE_NUMBER,
    RequestType: u32,
    DataLength: *mut usize,
) {
    let mut featureHeader: *mut FEATURE_HEADER = core::ptr::null_mut();
    let mut validLength: u32 = 0;
    let mut featureLength: u32 = 0;
    let mut headerLength: u32 = 0;
    let mut buffer: *mut u8 = core::ptr::null_mut();
    let mut bytesRemaining: u32 = 0;
    let mut featureCode: FEATURE_NUMBER = 0;
    let mut shouldCopy: i32 = 0;
    let mut copyLength: usize = 0;
    let mut transferedLength: usize = 0;
    let mut requiredLength: usize = 0;
    let mut header: *mut GET_CONFIGURATION_HEADER = core::ptr::null_mut();

    PAGED_CODE();

    if InputBufferSize < (core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32) {
        *DataLength = 0;
        return;
    }

    header = InputBuffer as *mut GET_CONFIGURATION_HEADER;
    REVERSE_BYTES(
        &mut validLength as *mut u32 as *mut core::ffi::c_void,
        (*header).DataLength.as_ptr() as *const core::ffi::c_void,
    );

    validLength += (core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength)
        + core::mem::size_of_val(&(*header).DataLength)) as u32;

    validLength = core::cmp::min(validLength, InputBufferSize);

    copyLength = core::cmp::min(OutputBufferSize, core::mem::size_of::<GET_CONFIGURATION_HEADER>() as usize);

    core::ptr::copy(InputBuffer as *const u8, OutputBuffer as *mut u8, copyLength);

    transferedLength = copyLength;
    requiredLength = core::mem::size_of::<GET_CONFIGURATION_HEADER>() as usize;

    if validLength > core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32 {
        buffer = (*header).Data.as_ptr() as *mut u8;
        bytesRemaining = validLength - (core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32);

        while bytesRemaining >= core::mem::size_of::<FEATURE_HEADER>() as u32 {
            featureHeader = buffer as *mut FEATURE_HEADER;
            shouldCopy = 0;

            featureCode = ((*featureHeader).FeatureCode[0] as u32) << 8
                | ((*featureHeader).FeatureCode[1] as u32);
            featureLength = (core::mem::size_of::<FEATURE_HEADER>() as u32)
                + (*featureHeader).AdditionalLength as u32;

            if featureCode >= StartingFeature {
                match RequestType {
                    SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL => {
                        shouldCopy = 1;
                    }
                    SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT => {
                        if (*featureHeader).Current() != 0 {
                            shouldCopy = 1;
                        }
                    }
                    SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE => {
                        if featureCode == StartingFeature {
                            shouldCopy = 1;
                        }
                    }
                    _ => {}
                }
            }

            if shouldCopy != 0 {
                copyLength = core::cmp::min(featureLength as usize, bytesRemaining as usize);
                copyLength = core::cmp::min(copyLength, OutputBufferSize - transferedLength);

                core::ptr::copy(
                    buffer as *const u8,
                    (OutputBuffer as *mut u8).add(transferedLength),
                    copyLength,
                );

                transferedLength += copyLength;
                requiredLength += featureLength as usize;
            }

            let min_step = core::cmp::min(featureLength, bytesRemaining);
            buffer = buffer.add(min_step as usize);
            bytesRemaining -= min_step;
        }
    }

    let rtl_size = (core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength)
        + core::mem::size_of_val(&(*header).DataLength)) as usize;

    if transferedLength >= rtl_size {
        headerLength = (requiredLength as u32) - (rtl_size as u32);
        header = OutputBuffer as *mut GET_CONFIGURATION_HEADER;
        REVERSE_BYTES(
            (*header).DataLength.as_ptr() as *mut core::ffi::c_void,
            &headerLength as *const u32 as *const core::ffi::c_void,
        );
    }

    *DataLength = transferedLength;
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetConfiguration(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut inputBuffer: *mut GET_CONFIGURATION_IOCTL_INPUT = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut transferByteCount: usize = 0;
    let zpoddInfo: *mut ZERO_POWER_ODD_INFO = (*DeviceExtension).ZeroPowerODDInfo;
    let mut inZeroPowerState: i32 = 0; // FALSE

    // PAGED_CODE();
    *DataLength = 0;

    if (*cdData).Mmc.IsMmc == 0 {
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            &raw mut inputBuffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &raw mut outputBuffer,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        if !zpoddInfo.is_null() && (*zpoddInfo).InZeroPowerState != 0 {
            inZeroPowerState = 1;
        }

        if inZeroPowerState == 0 || (*zpoddInfo).GetConfigurationBuffer.is_null() {
            let mut cdb: CDB = core::mem::zeroed();
            RtlZeroMemory(core::ptr::addr_of_mut!(cdb) as *mut core::ffi::c_void, core::mem::size_of::<CDB>());

            const MAXUSHORT: u32 = 65535;
            let output_len = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
            transferByteCount = if output_len < (MAXUSHORT - 1) as usize {
                output_len
            } else {
                (MAXUSHORT - 1) as usize
            };

            const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE: u32 = 0x2;  // was 1 (LLM-invented); bindgen-correct
            const CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG: u32 = 0x800;  // was 0x00000001 (LLM-invented); bindgen-correct

            let is_type_one = (*inputBuffer).RequestType == SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE;
            let hack_flag_set = ((*cdData).HackFlags & CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG as ntkrnl_sys::ddk::ULONG_PTR) != 0;

            if is_type_one && hack_flag_set {
                let needed_size = core::mem::size_of::<GET_CONFIGURATION_HEADER>() + core::mem::size_of::<FEATURE_HEADER>();
                transferByteCount = if transferByteCount >= needed_size {
                    transferByteCount
                } else {
                    needed_size
                };
            }

            ScratchBuffer_BeginUse(DeviceExtension);

            const SCSIOP_GET_CONFIGURATION: u8 = 0x46;
            cdb.GET_CONFIGURATION.as_mut().OperationCode = SCSIOP_GET_CONFIGURATION;
            cdb.GET_CONFIGURATION.as_mut().AllocationLength[0] = (transferByteCount >> 8) as u8;
            cdb.GET_CONFIGURATION.as_mut().AllocationLength[1] = (transferByteCount & 0xFF) as u8;
            cdb.GET_CONFIGURATION.as_mut().StartingFeature[0] = ((*inputBuffer).Feature >> 8) as u8;
            cdb.GET_CONFIGURATION.as_mut().StartingFeature[1] = ((*inputBuffer).Feature & 0xFF) as u8;
            cdb.GET_CONFIGURATION.as_mut().set_RequestType((*inputBuffer).RequestType as u8);

            if is_type_one && hack_flag_set {
                const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
                const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
                TracePrint!(TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, b"DeviceHandleGetConfiguration: Changing TYPE_ONE Get Config to TYPE_ALL\n\0".as_ptr() as *const i8);
                const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL: u8 = 0;
                cdb.GET_CONFIGURATION.as_mut().set_RequestType(SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL);
            }

            status = ScratchBuffer_ExecuteCdb(DeviceExtension, (Request) as _, transferByteCount as u32, 1, core::ptr::addr_of_mut!(cdb), 12);

            if status >= 0 {
                let scratch_srb = (*DeviceExtension).ScratchContext.ScratchSrb;
                let data_transfer_length = (*scratch_srb).DataTransferLength;

                if is_type_one && hack_flag_set {
                    if (data_transfer_length as usize) < core::mem::size_of::<GET_CONFIGURATION_HEADER>() {
                        const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
                        const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
                        TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, b"DeviceHandleGetConfiguration: No get config header!\n\0".as_ptr() as *const i8);
                        *DataLength = 0;
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    } else {
                        GetConfigurationDataConversionTypeAllToTypeOne(
                            (*inputBuffer).Feature,
                            scratch_srb,
                            DataLength,
                        );
                        *DataLength = if *DataLength < RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength {
                            *DataLength
                        } else {
                            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                        };
                    }
                } else {
                    *DataLength = data_transfer_length as usize;
                }

                if status >= 0 {
                    core::ptr::copy(
                        (*DeviceExtension).ScratchContext.ScratchBuffer as *const u8,
                        outputBuffer as *mut u8,
                        *DataLength,
                    );
                }
            }

            ScratchBuffer_EndUse(DeviceExtension);
        } else {
            GetConfigurationDataSynthesize(
                (*zpoddInfo).GetConfigurationBuffer as *mut core::ffi::c_void,
                (*zpoddInfo).GetConfigurationBufferSize,
                outputBuffer,
                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
                (*inputBuffer).Feature,
                (*inputBuffer).RequestType,
                DataLength,
            );
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetDriveGeometry(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // PAGED_CODE() expansion: assert IRQL <= APC_LEVEL
    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() <= APC_LEVEL);

    *DataLength = 0;

    status = WdfRequestRetrieveOutputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
        &mut outputBuffer,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        status = MediaReadCapacity((*DeviceExtension).Device as PDEVICE_OBJECT);
    }

    if NT_SUCCESS(status) {
        match RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode {
            IOCTL_DISK_GET_LENGTH_INFO => {
                let lengthInfo = outputBuffer as *mut GET_LENGTH_INFORMATION;
                (*lengthInfo).Length = (*DeviceExtension).PartitionLength;
                *DataLength = core::mem::size_of::<GET_LENGTH_INFORMATION>();
            }
            IOCTL_DISK_GET_DRIVE_GEOMETRY | IOCTL_CDROM_GET_DRIVE_GEOMETRY => {
                let geometry = outputBuffer as *mut DISK_GEOMETRY;
                *geometry = (*DeviceExtension).DiskGeometry;
                *DataLength = core::mem::size_of::<DISK_GEOMETRY>();
            }
            IOCTL_DISK_GET_DRIVE_GEOMETRY_EX | IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX => {
                let geometryEx = outputBuffer as *mut DISK_GEOMETRY_EX;
                (*geometryEx).DiskSize = (*DeviceExtension).PartitionLength;
                (*geometryEx).Geometry = (*DeviceExtension).DiskGeometry;
                *DataLength = core::mem::offset_of!(DISK_GEOMETRY_EX, Data);
            }
            IOCTL_STORAGE_READ_CAPACITY => {
                let readCapacity = outputBuffer as *mut STORAGE_READ_CAPACITY;
                (*readCapacity).Version = core::mem::size_of::<STORAGE_READ_CAPACITY>() as u32;
                (*readCapacity).Size = core::mem::size_of::<STORAGE_READ_CAPACITY>() as u32;
                (*readCapacity).BlockLength = (*DeviceExtension).DiskGeometry.BytesPerSector;
                if (*readCapacity).BlockLength > 0 {
                    (*readCapacity).NumberOfBlocks.QuadPart = (*DeviceExtension).PartitionLength.QuadPart / (*readCapacity).BlockLength as i64;
                } else {
                    (*readCapacity).NumberOfBlocks.QuadPart = 0;
                }
                (*readCapacity).DiskLength = (*DeviceExtension).PartitionLength;
                *DataLength = core::mem::size_of::<STORAGE_READ_CAPACITY>();
            }
            _ => {
                // NT_ASSERT(FALSE)
                ntkrnl_sys::ASSERT!(false);
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleDiskVerify(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let _ = RequestParameters; // UNREFERENCED_PARAMETER

    // PAGED_CODE equivalent
    if KeGetCurrentIrql() > APC_LEVEL {
        // debug check (no-op in release)
    }

    *DataLength = 0;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
    let mut status: i32 = STATUS_SUCCESS;

    if (*cdData).Mmc.WriteAllowed == 0 {
        status = STATUS_MEDIA_WRITE_PROTECTED;
    }

    if NT_SUCCESS(status) {
        let input_buffer_len = RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize;
        let mut verifyInfo_raw: *mut core::ffi::c_void = core::ptr::null_mut();
        status = WdfRequestRetrieveInputBuffer(
            Request,
            input_buffer_len,
            &mut verifyInfo_raw,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(status) {
            let verifyInfo = verifyInfo_raw as *mut VERIFY_INFORMATION;
            let mut byteOffset: LARGE_INTEGER = core::mem::zeroed();
            let sum = (*DeviceExtension)
                .StartingOffset
                .QuadPart
                .wrapping_add((*verifyInfo).StartingOffset.QuadPart);
            byteOffset.QuadPart = sum;

            if sum < (*DeviceExtension).StartingOffset.QuadPart
                || ((*verifyInfo).Length >> (*DeviceExtension).SectorShift) > MAXUSHORT as u32
                || (byteOffset.QuadPart >> (*DeviceExtension).SectorShift) > MAXULONG as i64
            {
                status = STATUS_INVALID_PARAMETER;
            } else {
                let transferSize: u32 = 0;
                let sectorOffset: u32;
                let sectorCount: u16;
                let mut timeoutValue: u32 = 0;
                let mut cdb: CDB = core::mem::zeroed();

                ScratchBuffer_BeginUse(DeviceExtension);

                sectorOffset =
                    (byteOffset.QuadPart >> (*DeviceExtension).SectorShift) as u32;
                sectorCount = ((*verifyInfo).Length >> (*DeviceExtension).SectorShift) as u16;

                RtlZeroMemory(
                    &mut cdb as *mut CDB as *mut core::ffi::c_void,
                    core::mem::size_of::<CDB>(),
                );

                cdb.CDB10.as_mut().OperationCode = SCSIOP_VERIFY;

                // Big-endian encoding of sectorOffset (u32) -> FOUR_BYTE cast
                let sector_offset_ptr: *const FOUR_BYTE =
                    core::ptr::addr_of!(sectorOffset) as *const u32 as *const FOUR_BYTE;
                cdb.CDB10.as_mut().LogicalBlockByte0 = (*sector_offset_ptr).__bindgen_anon_1.as_ref().Byte3;
                cdb.CDB10.as_mut().LogicalBlockByte1 = (*sector_offset_ptr).__bindgen_anon_1.as_ref().Byte2;
                cdb.CDB10.as_mut().LogicalBlockByte2 = (*sector_offset_ptr).__bindgen_anon_1.as_ref().Byte1;
                cdb.CDB10.as_mut().LogicalBlockByte3 = (*sector_offset_ptr).__bindgen_anon_1.as_ref().Byte0;

                // Big-endian encoding of sectorCount (u16) -> FOUR_BYTE cast
                let sector_count_ptr: *const FOUR_BYTE =
                    core::ptr::addr_of!(sectorCount) as *const u16 as *const FOUR_BYTE;
                cdb.CDB10.as_mut().TransferBlocksMsb = (*sector_count_ptr).__bindgen_anon_1.as_ref().Byte1;
                cdb.CDB10.as_mut().TransferBlocksLsb = (*sector_count_ptr).__bindgen_anon_1.as_ref().Byte0;

                if sectorCount != 0 {
                    timeoutValue = TimeOutValueGetCapValue(
                        (*DeviceExtension).TimeOutValue,
                        ((sectorCount as u32 + 128) / 128),
                    );
                }

                status = ScratchBuffer_ExecuteCdbEx(
                    DeviceExtension,
                    (Request) as _,
                    transferSize,
                    0u8, // FALSE
                    &cdb as *const CDB,
                    (10u32) as _,
                    timeoutValue,
                );

                ScratchBuffer_EndUse(DeviceExtension);
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleCheckVerify(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS

    // PAGED_CODE(); is a no-op macro, omitted here

    *DataLength = 0;

    if status >= 0 {
        let mut transfer_size: u32 = 0;
        // Stack-allocated CDB, zeroed via RtlZeroMemory to match the original.
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);
        RtlZeroMemory((&mut cdb) as *mut CDB as *mut core::ffi::c_void, core::mem::size_of::<CDB>());

        // Set up the CDB
        cdb.CDB6GENERIC.as_mut().OperationCode = SCSIOP_TEST_UNIT_READY;

        status = ScratchBuffer_ExecuteCdbEx(
            DeviceExtension,
            (Request) as _,
            transfer_size,
            0, // FALSE
            &mut cdb,
            6,
            CDROM_TEST_UNIT_READY_TIMEOUT,
        );

        CdromDbgPrintForce2(
            b"[CHKVER] mcc=%lx ioctl=%lx\n\0".as_ptr() as *const i8,
            (*DeviceExtension).MediaChangeCount as usize,
            RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode as usize,
        );
        if status >= 0 {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_STORAGE_CHECK_VERIFY
                && RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength != 0
            {
                let mut output_buffer: *mut u32 = core::ptr::null_mut();
                status = WdfRequestRetrieveOutputBuffer(
                    Request,
                    RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
                    &mut output_buffer as *mut _ as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );

                if !output_buffer.is_null() {
                    *output_buffer = (*DeviceExtension).MediaChangeCount;
                    *DataLength = core::mem::size_of::<u32>();
                }
            } else {
                *DataLength = 0;
            }
        }

        // nothing to do after the command finishes.
        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleFakePartitionInfo(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ioctl: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;

    PAGED_CODE();

    *DataLength = 0;

    if NT_SUCCESS(status) {
        if (ioctl != IOCTL_DISK_GET_DRIVE_LAYOUT) &&
            (ioctl != IOCTL_DISK_GET_DRIVE_LAYOUT_EX) &&
            (ioctl != IOCTL_DISK_GET_PARTITION_INFO) &&
            (ioctl != IOCTL_DISK_GET_PARTITION_INFO_EX)
        {
            status = STATUS_INTERNAL_ERROR;
        }
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            &mut outputBuffer,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        match ioctl {
            IOCTL_DISK_GET_DRIVE_LAYOUT => {
                *DataLength = (core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION, PartitionEntry)
                    + core::mem::size_of::<PARTITION_INFORMATION>()) as usize;
                RtlZeroMemory(outputBuffer, *DataLength);
            }
            IOCTL_DISK_GET_DRIVE_LAYOUT_EX => {
                *DataLength = (core::mem::offset_of!(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry)
                    + core::mem::size_of::<PARTITION_INFORMATION_EX>()) as usize;
                RtlZeroMemory(outputBuffer, *DataLength);
            }
            IOCTL_DISK_GET_PARTITION_INFO => {
                *DataLength = core::mem::size_of::<PARTITION_INFORMATION>();
                RtlZeroMemory(outputBuffer, *DataLength);
            }
            IOCTL_DISK_GET_PARTITION_INFO_EX => {
                *DataLength = core::mem::size_of::<PARTITION_INFORMATION_EX>();
                RtlZeroMemory(outputBuffer, *DataLength);
            }
            _ => {
                NT_ASSERT(0);
            }
        }

        if ioctl == IOCTL_DISK_GET_DRIVE_LAYOUT {
            let layout = outputBuffer as *mut DRIVE_LAYOUT_INFORMATION;
            (*layout).PartitionCount = 1;
            (*layout).Signature = 1;
            outputBuffer = (*layout).PartitionEntry.as_mut_ptr() as *mut core::ffi::c_void;
            ioctl = IOCTL_DISK_GET_PARTITION_INFO;
        } else if ioctl == IOCTL_DISK_GET_DRIVE_LAYOUT_EX {
            let layoutEx = outputBuffer as *mut DRIVE_LAYOUT_INFORMATION_EX;
            (*layoutEx).PartitionStyle = PARTITION_STYLE_MBR;
            (*layoutEx).PartitionCount = 1;
            (*layoutEx).__bindgen_anon_1.Mbr.Signature = 1;
            outputBuffer = (*layoutEx).PartitionEntry.as_mut_ptr() as *mut core::ffi::c_void;
            ioctl = IOCTL_DISK_GET_PARTITION_INFO_EX;
        }

        if ioctl == IOCTL_DISK_GET_PARTITION_INFO {
            let partitionInfo = outputBuffer as *mut PARTITION_INFORMATION;
            (*partitionInfo).RewritePartition = 0;
            (*partitionInfo).RecognizedPartition = 1;
            (*partitionInfo).PartitionType = PARTITION_FAT32 as u8;
            (*partitionInfo).BootIndicator = 0;
            (*partitionInfo).HiddenSectors = 0;
            (*partitionInfo).StartingOffset.QuadPart = 0;
            (*partitionInfo).PartitionLength = (*DeviceExtension).PartitionLength;
            (*partitionInfo).PartitionNumber = 0;
        } else {
            let partitionInfo = outputBuffer as *mut PARTITION_INFORMATION_EX;
            (*partitionInfo).PartitionStyle = PARTITION_STYLE_MBR;
            (*partitionInfo).RewritePartition = 0;
            (*partitionInfo).__bindgen_anon_1.Mbr.RecognizedPartition = 1;
            (*partitionInfo).__bindgen_anon_1.Mbr.PartitionType = PARTITION_FAT32 as u8;
            (*partitionInfo).__bindgen_anon_1.Mbr.BootIndicator = 0;
            (*partitionInfo).__bindgen_anon_1.Mbr.HiddenSectors = 0;
            (*partitionInfo).StartingOffset.QuadPart = 0;
            (*partitionInfo).PartitionLength = (*DeviceExtension).PartitionLength;
            (*partitionInfo).PartitionNumber = 0;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetDeviceNumber(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut size_t,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    *DataLength = 0;

    if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize
        >= core::mem::size_of::<STORAGE_DEVICE_NUMBER>()
    {
        let mut deviceNumber: *mut STORAGE_DEVICE_NUMBER = core::ptr::null_mut();
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            &mut deviceNumber as *mut _ as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(status) {
            (*deviceNumber).DeviceType = (*(*DeviceExtension).DeviceObject).DeviceType;
            (*deviceNumber).DeviceNumber = (*DeviceExtension).DeviceNumber;
            (*deviceNumber).PartitionNumber = MAXULONG; // legacy reason, return (-1) for this IOCTL.

            status = STATUS_SUCCESS;
            *DataLength = core::mem::size_of::<STORAGE_DEVICE_NUMBER>();
        }
    } else {
        status = STATUS_BUFFER_TOO_SMALL;
        *DataLength = core::mem::size_of::<STORAGE_DEVICE_NUMBER>();
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetHotPlugInfo(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    unsafe {
        let mut status: i32 = STATUS_SUCCESS;
        *DataLength = 0;
        if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize >= core::mem::size_of::<STORAGE_HOTPLUG_INFO>() {
            let mut info: *mut STORAGE_HOTPLUG_INFO = core::ptr::null_mut();
            status = WdfRequestRetrieveOutputBuffer(
                Request,
                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
                &mut info as *mut *mut STORAGE_HOTPLUG_INFO as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
            if status >= 0 {
                *info = (*((*DeviceExtension).PrivateFdoData)).HotplugInfo;
                status = STATUS_SUCCESS;
                *DataLength = core::mem::size_of::<STORAGE_HOTPLUG_INFO>();
            }
        } else {
            status = STATUS_BUFFER_TOO_SMALL;
            *DataLength = core::mem::size_of::<STORAGE_HOTPLUG_INFO>();
        }
        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleSetHotPlugInfo(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // Forward declarations for unknown externals
    // These constants are expected to be defined elsewhere (e.g. in a global scope)

    unsafe {
        // UNREFERENCED_PARAMETER(RequestParameters);
        let _ = &RequestParameters;

        let mut status: NTSTATUS = STATUS_SUCCESS;
        let mut info: *mut STORAGE_HOTPLUG_INFO = core::ptr::null_mut();

        *DataLength = 0;

        if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize)
            < core::mem::size_of::<STORAGE_HOTPLUG_INFO>()
        {
            status = STATUS_INFO_LENGTH_MISMATCH;
        }

        if status >= 0 {
            status = WdfRequestRetrieveInputBuffer(
                Request,
                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                &mut info as *mut *mut STORAGE_HOTPLUG_INFO as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
        }

        if status >= 0 {
            if (*info).Size != (*(*DeviceExtension).PrivateFdoData).HotplugInfo.Size {
                status = STATUS_INVALID_PARAMETER_1;
            }
            if (*info).MediaRemovable != (*(*DeviceExtension).PrivateFdoData).HotplugInfo.MediaRemovable {
                status = STATUS_INVALID_PARAMETER_2;
            }
            if (*info).MediaHotplug != (*(*DeviceExtension).PrivateFdoData).HotplugInfo.MediaHotplug {
                status = STATUS_INVALID_PARAMETER_3;
            }
            if (*info).WriteCacheEnableOverride
                != (*(*DeviceExtension).PrivateFdoData).HotplugInfo.WriteCacheEnableOverride
            {
                status = STATUS_INVALID_PARAMETER_5;
            }
        }

        if status >= 0 {
            (*(*DeviceExtension).PrivateFdoData).HotplugInfo.DeviceHotplug = (*info).DeviceHotplug;

            // Store the user-defined override in the registry
            DeviceSetParameter(
                DeviceExtension,
                core::ptr::addr_of!(CLASSP_REG_SUBKEY_NAME) as *const u16,
                core::ptr::addr_of!(CLASSP_REG_REMOVAL_POLICY_VALUE_NAME) as *const u16,
                if (*info).DeviceHotplug != 0 {
                    RemovalPolicyExpectSurpriseRemoval
                } else {
                    RemovalPolicyExpectOrderlyRemoval
                },
            );
        }

        status
    }
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleEventNotification(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: *mut WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut info: *mut MEDIA_CHANGE_DETECTION_INFO = core::ptr::null_mut();
    let mut requestInUse: i32 = 0;
    let mut eventBuffer: *mut STORAGE_EVENT_NOTIFICATION = core::ptr::null_mut();

    *DataLength = 0;

    info = (*DeviceExtension).MediaChangeDetectionInfo;

    if (*DeviceExtension).IsInitialized == 0 || info.is_null() {
        status = ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) && !Request.is_null() && !RequestParameters.is_null() {
        if ((*RequestParameters).Parameters.DeviceIoControl.as_ref().InputBufferLength as usize) < core::mem::size_of::<STORAGE_EVENT_NOTIFICATION>() {
            status = ntkrnl_sys::constants::STATUS_INFO_LENGTH_MISMATCH;
        }

        if ntkrnl_sys::base::NT_SUCCESS(status) {
            status = WdfRequestRetrieveInputBuffer(
                Request,
                (*RequestParameters).Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                &mut eventBuffer as *mut _ as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );

            if ntkrnl_sys::base::NT_SUCCESS(status) {
                if (*eventBuffer).Version != STORAGE_EVENT_NOTIFICATION_VERSION_V1
                    || (*eventBuffer).Size != core::mem::size_of::<STORAGE_EVENT_NOTIFICATION>() as u32
                {
                    status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
                } else if ((*eventBuffer).Events
                    & (STORAGE_EVENT_MEDIA_STATUS | STORAGE_EVENT_DEVICE_STATUS | STORAGE_EVENT_DEVICE_OPERATION) as u64)
                    == 0
                {
                    status = ntkrnl_sys::constants::STATUS_NOT_SUPPORTED;
                }
            }
        }
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        if (*info).MediaChangeDetectionDisableCount != 0 {
            let format_str = b"RequestHandleEventNotification: device %p has detection disabled \n\0".as_ptr() as *const i8;
            TracePrint!(
                TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_MCN,
                format_str,
                (*DeviceExtension).DeviceObject as *const core::ffi::c_void,
            );
            status = ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
        }
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        let dest = &mut (*info).MediaChangeRequestInUse as *mut i32;
        requestInUse = InterlockedCompareExchange(dest, 1, 0);

        if requestInUse != 0 {
            status = ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
        }
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        RequestSetupMcnSyncIrp(DeviceExtension);

        ntkrnl_sys::driver_ddk::IoCallDriver(
            (*DeviceExtension).DeviceObject,
            (*info).MediaChangeSyncIrp,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleEjectionControl(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    // PAGED_CODE();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut mediaRemoval: *mut PREVENT_MEDIA_REMOVAL = core::ptr::null_mut();

    *DataLength = 0;

    if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
        < core::mem::size_of::<PREVENT_MEDIA_REMOVAL>()
    {
        status = STATUS_INFO_LENGTH_MISMATCH;
    } else {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            &mut mediaRemoval as *mut *mut PREVENT_MEDIA_REMOVAL as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        let lock_type = if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode
            == IOCTL_STORAGE_EJECTION_CONTROL
        {
            SecureMediaLock
        } else {
            SimpleMediaLock
        };
        status = PerformEjectionControl(
            DeviceExtension,
            (Request) as _,
            lock_type,
            ((*mediaRemoval).PreventMediaRemoval as i32) as _,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleEnableStreaming(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> NTSTATUS {
    unsafe {
        PAGED_CODE();
        let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);
        let fdoData: *mut CDROM_PRIVATE_FDO_DATA = (*DeviceExtension).PrivateFdoData;
        let mut fileObject: WDFFILEOBJECT = core::ptr::null_mut();
        let mut fileObjectContext: PFILE_OBJECT_CONTEXT = core::ptr::null_mut();
        let mut status: NTSTATUS = STATUS_SUCCESS;
        let mut inputBuffer: *mut CDROM_STREAMING_CONTROL = core::ptr::null_mut();
        let mut enforceStreamingRead: u8 = 0;
        let mut enforceStreamingWrite: u8 = 0;
        let streamingReadSupported: u8;
        let streamingWriteSupported: u8;
        *DataLength = 0;
        status = WdfRequestRetrieveInputBuffer(
            Request,
            core::mem::size_of::<CDROM_STREAMING_CONTROL>() as usize,
            core::ptr::addr_of_mut!(inputBuffer) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(status) {
            fileObject = WdfRequestGetFileObject(Request);
            if !fileObject.is_null() {
                fileObjectContext = FileObjectGetContext(fileObject);
            }
            NT_ASSERT((!fileObjectContext.is_null()) as i32);
            if fileObjectContext.is_null() {
                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"RequestHandleEnableStreaming: cannot find file object context\n\0".as_ptr() as *const i8,
                );
                status = STATUS_INVALID_HANDLE;
            }
        }
        if NT_SUCCESS(status) {
            let requestType = (*inputBuffer).RequestType;
            if requestType == CdromStreamingDisable {
                enforceStreamingRead = 0;
                enforceStreamingWrite = 0;
            } else if requestType == CdromStreamingEnableForReadOnly {
                enforceStreamingRead = 1;
                enforceStreamingWrite = 0;
            } else if requestType == CdromStreamingEnableForWriteOnly {
                enforceStreamingRead = 0;
                enforceStreamingWrite = 1;
            } else if requestType == CdromStreamingEnableForReadWrite {
                enforceStreamingRead = 1;
                enforceStreamingWrite = 1;
            }
            let mmc = &(*cdData).Mmc;
            streamingReadSupported = if (mmc.StreamingReadSupported != 0) && !TEST_FLAG((*fdoData).HackFlags as u32, FDO_HACK_NO_STREAMING) { 1 } else { 0 };
            streamingWriteSupported = if (mmc.StreamingWriteSupported != 0) && !TEST_FLAG((*fdoData).HackFlags as u32, FDO_HACK_NO_STREAMING) { 1 } else { 0 };
            if (enforceStreamingRead != 0 && streamingReadSupported == 0) ||
                (enforceStreamingWrite != 0 && streamingWriteSupported == 0) {
                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_IOCTL,
                    b"RequestHandleEnableStreaming: requested Streaming mode is not supported\n\0".as_ptr() as *const i8,
                );
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                (*fileObjectContext).EnforceStreamingRead = enforceStreamingRead;
                (*fileObjectContext).EnforceStreamingWrite = enforceStreamingWrite;
            }
        }
        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleSendOpcInformation(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut inputBuffer: *mut CDROM_SIMPLE_OPC_INFO = core::ptr::null_mut();

    // PAGED_CODE(); omitted – equivalent debug check not transliterated

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        core::mem::size_of::<CDROM_SIMPLE_OPC_INFO>(),
        &mut inputBuffer as *mut *mut CDROM_SIMPLE_OPC_INFO as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if ::ntkrnl_sys::base::NT_SUCCESS(status) {
        let mut cdb: CDB = core::mem::zeroed();
        ScratchBuffer_BeginUse(DeviceExtension);

        core::ptr::write_bytes((core::ptr::addr_of_mut!(cdb)) as *mut u8, 0, core::mem::size_of::<CDB>());

        // we support only the simplest version for now
        cdb.SEND_OPC_INFORMATION.as_mut().OperationCode = SCSIOP_SEND_OPC_INFORMATION;
        cdb.SEND_OPC_INFORMATION.as_mut().set_DoOpc(1);
        cdb.SEND_OPC_INFORMATION.as_mut().set_Exclude0(if (*inputBuffer).Exclude0 != 0 { 1 } else { 0 });
        cdb.SEND_OPC_INFORMATION.as_mut().set_Exclude1(if (*inputBuffer).Exclude1 != 0 { 1 } else { 0 });

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            0,
            0, // FALSE
            core::ptr::addr_of_mut!(cdb),
            core::mem::size_of::<CDB_SEND_OPC_INFORMATION>() as u8,
        );

        // nothing to do after the command finishes
        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleGetPerformance(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut size_t,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let mut inputBuffer: *mut CDROM_PERFORMANCE_REQUEST = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    *DataLength = 0;

    if status >= 0 {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            &mut inputBuffer as *mut *mut CDROM_PERFORMANCE_REQUEST as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }
    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &mut outputBuffer as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        let mut descriptorSize: u16 = 0;
        let mut descriptorCount: u16 = 0;
        let mut transferSize: size_t = 0;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        RtlZeroMemory(
            &mut cdb as *mut CDB as *mut core::ffi::c_void,
            core::mem::size_of::<CDB>(),
        );

        if (*inputBuffer).RequestType == 0 /* CdromPerformanceRequest */ {
            cdb.GET_PERFORMANCE.as_mut().Type = 0;
            // 10b is the only defined tolerance in MMCr6
            cdb.GET_PERFORMANCE.as_mut().set_Tolerance(2);

            match (*inputBuffer).Exceptions {
                0 /* CdromNominalPerformance */ => {
                    cdb.GET_PERFORMANCE.as_mut().set_Except(0);
                    descriptorSize = core::mem::size_of::<CDROM_NOMINAL_PERFORMANCE_DESCRIPTOR>() as u16;
                }
                1 /* CdromEntirePerformanceList */ => {
                    cdb.GET_PERFORMANCE.as_mut().set_Except(1);
                    descriptorSize = core::mem::size_of::<CDROM_NOMINAL_PERFORMANCE_DESCRIPTOR>() as u16;
                }
                2 /* CdromPerformanceExceptionsOnly */ => {
                    cdb.GET_PERFORMANCE.as_mut().set_Except(2);
                    descriptorSize = core::mem::size_of::<CDROM_EXCEPTION_PERFORMANCE_DESCRIPTOR>() as u16;
                }
                _ => {} // keep existing values, no fallthrough
            }

            match (*inputBuffer).PerformanceType {
                0 /* CdromReadPerformance */ => cdb.GET_PERFORMANCE.as_mut().set_Write(0),
                1 /* CdromWritePerformance */ => cdb.GET_PERFORMANCE.as_mut().set_Write(1),
                _ => {}
            }

            REVERSE_BYTES(
                &mut cdb.GET_PERFORMANCE.as_mut().StartingLBA as *mut _ as *mut core::ffi::c_void,
                &(*inputBuffer).StaringLba as *const _ as *const core::ffi::c_void,
            );
        } else if (*inputBuffer).RequestType == 1 /* CdromWriteSpeedRequest */ {
            cdb.GET_PERFORMANCE.as_mut().Type = 3;
            descriptorSize = core::mem::size_of::<CDROM_WRITE_SPEED_DESCRIPTOR>() as u16;
        }

        cdb.GET_PERFORMANCE.as_mut().OperationCode = 0xAC /* SCSIOP_GET_PERFORMANCE */;

        // calculate how many descriptors can fit into the output buffer
        if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >=
            core::mem::size_of::<CDROM_PERFORMANCE_HEADER>()
            && RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <= core::u16::MAX as size_t
            && descriptorSize > 0
        {
            descriptorCount = (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                - core::mem::size_of::<CDROM_PERFORMANCE_HEADER>()) as u16;
            descriptorCount /= descriptorSize;
        } else {
            status = (0xC000000Du32 as i32) /* STATUS_INVALID_PARAMETER */;
        }

        REVERSE_BYTES_SHORT(
            &mut cdb.GET_PERFORMANCE.as_mut().MaximumNumberOfDescriptors as *mut _ as *mut core::ffi::c_void,
            &descriptorCount as *const _ as *const core::ffi::c_void,
        );

        // Calculate transfer size. We round it up to meet adapter requirements.
        // Extra bytes are discarded later, when we copy data to the output buffer.
        transferSize = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
        transferSize += (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as size_t;
        transferSize &= !((*(*DeviceExtension).AdapterDescriptor).AlignmentMask as size_t);

        if status >= 0 {
            status = ScratchBuffer_ExecuteCdbEx(
                DeviceExtension,
                (Request) as _,
                transferSize as u32,
                1, // TRUE
                &cdb,
                (core::mem::size_of::<GetPerformanceCdb>() as u32) as _,
                10 /* CDROM_GET_PERFORMANCE_TIMEOUT */,
            );
        }

        if status >= 0 {
            if (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength
                < core::mem::size_of::<CDROM_PERFORMANCE_HEADER>() as u32
            {
                *DataLength = 0;
                status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                let srb_len = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as size_t;
                let out_len = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
                *DataLength = if out_len < srb_len { out_len } else { srb_len };
                RtlCopyMemory(
                    outputBuffer,
                    (*DeviceExtension).ScratchContext.ScratchBuffer,
                    *DataLength,
                );
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleMcnSyncFakeIoctl(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let info: *mut MEDIA_CHANGE_DETECTION_INFO =
        (*DeviceExtension).MediaChangeDetectionInfo;
    let mut shouldRetry: i32 = 1; // TRUE
    let mut requestSent: i32 = 0; // FALSE

    // PAGED_CODE ();    // pageable code assertion, elided mechanically

    *DataLength = 0;

    while shouldRetry != 0 {
        status = RequestSetupMcnRequest(
            DeviceExtension,
            ((*info).Gesn.Supported as i32) as _,
        );

        if !(status >= 0) {
            // !NT_SUCCESS
            shouldRetry = 0; // FALSE
        }

        if status >= 0 {
            // NT_SUCCESS
            requestSent = RequestSendMcnRequest(DeviceExtension) as i32;

            if requestSent != 0 {
                shouldRetry = RequestPostWorkMcnRequest(DeviceExtension) as i32;
            } else {
                shouldRetry = 0; // FALSE
            }
        }
    }

    DeviceSendDelayedMediaChangeNotifications(DeviceExtension);

    status = 0; // STATUS_SUCCESS
    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestIsRealtimeStreaming(Request: WDFREQUEST, IsReadRequest: u8) -> u8 {
    let mut useStreaming: u8 = 0;

    if useStreaming == 0 {
        let mut currentStackLocationFlags: u8 = 0;
        currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);

        useStreaming = if (currentStackLocationFlags as i32 & SL_REALTIME_STREAM as i32) != 0 { 1 } else { 0 };
    }

    if useStreaming == 0 {
        let fileObject: WDFFILEOBJECT;
        let fileObjectContext: PFILE_OBJECT_CONTEXT;

        fileObject = WdfRequestGetFileObject(Request);

        if fileObject != core::ptr::null_mut() {
            fileObjectContext = FileObjectGetContext(fileObject);
            NT_ASSERT((fileObjectContext != core::ptr::null_mut()) as i32);

            if IsReadRequest != 0 && (*fileObjectContext).EnforceStreamingRead != 0 {
                useStreaming = 1;
            }

            if IsReadRequest == 0 && (*fileObjectContext).EnforceStreamingWrite != 0 {
                useStreaming = 1;
            }
        }
    }

    useStreaming
}

const TRACE_FLAG_RW: u32 = 3; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const FeatureRandomWritable: u32 = 0x0020;
const FeatureIncrementalStreamingWritable: u32 = 0x0021;
const FeatureRestrictedOverwrite: u32 = 0x0022;
const FeatureRigidRestrictedOverwrite: u32 = 0x0023;
const FeatureDefectManagement: u32 = 0x0008;
#[no_mangle]
pub unsafe extern "C" fn RequestValidateReadWrite(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: *mut WDF_REQUEST_PARAMETERS,
) -> NTSTATUS {
    let RequestParameters = &*RequestParameters;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = &raw mut (*DeviceExtension).DeviceAdditionalData;
    let mut isValid: BOOLEAN = 1;
    let mut startingOffset: i64 = 0;
    let mut transferByteCount: usize = 0;
    let mut requestType: ULONG = RequestParameters.Type;
    let irp: PIRP = WdfRequestWdmGetIrp(Request);
    let currentStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(irp);

    if (((*(*DeviceExtension).DeviceObject).Flags & DO_VERIFY_VOLUME) != 0)
        && ((*currentStack).MinorFunction as u32 != CDROM_VOLUME_VERIFY_CHECKED)
        && (((*currentStack).Flags as u32 & SL_OVERRIDE_VERIFY_VOLUME) == 0)
    {
        status = STATUS_VERIFY_REQUIRED;
    }

    if NT_SUCCESS(status) {
        if ((*DeviceExtension).DeviceAdditionalData.PlayActive != 0) {
            status = STATUS_DEVICE_BUSY;
        }
    }

    if NT_SUCCESS(status) {
        if (EXCLUSIVE_MODE(cdData) != 0) && ((*cdData).ExclusiveOwner != WdfRequestGetFileObject(Request)) {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_RW,
                b"RequestValidateReadWrite: Access Denied! Device in exclusive mode.\n\0".as_ptr() as *const i8,
            );
            status = STATUS_ACCESS_DENIED;
        }
    }

    CdromDbgPrint3(
        b"[RWV0] st=%x doFlags=%x minor=%x\n\0".as_ptr() as *const i8,
        status as usize,
        (*(*DeviceExtension).DeviceObject).Flags as usize,
        (*currentStack).MinorFunction as usize,
    );

    if NT_SUCCESS(status) {
        CdromGetReadWriteFields(
            Request,
            &mut requestType,
            &mut startingOffset,
            &mut transferByteCount,
        );
        CdromDbgPrint3(
            b"[RWVg] grwf_type=%x grwf_off=%zx grwf_len=%zx\n\0".as_ptr() as *const i8,
            requestType as usize,
            startingOffset as usize,
            transferByteCount,
        );
        CdromDbgPrint3(
            b"[RWVf] paramRdLen=%zx paramRdOff=%zx ptr=%p\n\0".as_ptr() as *const i8,
            RequestParameters.Parameters.Read.as_ref().Length,
            RequestParameters.Parameters.Read.as_ref().DeviceOffset as usize,
            RequestParameters as *const _ as usize,
        );

        if transferByteCount == 0 {
            if requestType == WdfRequestTypeRead {
                startingOffset = RequestParameters.Parameters.Read.as_ref().DeviceOffset;
                transferByteCount = RequestParameters.Parameters.Read.as_ref().Length;
            } else {
                startingOffset = RequestParameters.Parameters.Write.as_ref().DeviceOffset;
                transferByteCount = RequestParameters.Parameters.Write.as_ref().Length;
            }
        }

        if (*DeviceExtension).DiskGeometry.BytesPerSector == 0 {
            (*DeviceExtension).DiskGeometry.BytesPerSector = 2048;
        }
        if (*DeviceExtension).SectorShift == 0 {
            (*DeviceExtension).SectorShift = 11;
        }

        if ((startingOffset & ((*DeviceExtension).DiskGeometry.BytesPerSector as i64 - 1)) != 0)
            || ((transferByteCount as i64 & ((*DeviceExtension).DiskGeometry.BytesPerSector as i64 - 1)) != 0)
        {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }

        if NT_SUCCESS(status)
            && requestType == WdfRequestTypeRead
            && startingOffset == 0x8000
            && transferByteCount == 2048
            && (*DeviceExtension).PartitionLength.QuadPart > startingOffset
        {
            return STATUS_SUCCESS;
        }
    }

    if NT_SUCCESS(status) {
        let mut schema: u32 = (*cdData).Mmc.ValidationSchema;

        if requestType == WdfRequestTypeRead {
            if (*cdData).Mmc.WriteAllowed == 0 {
                schema = FeatureRandomWritable;
            } else if schema != FeatureIncrementalStreamingWritable {
                schema = FeatureRandomWritable;
            }
        }

        if (requestType == WdfRequestTypeWrite) && ((*cdData).Mmc.WriteAllowed == 0) {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_RW,
                b"RequestValidateReadWrite: Write request to read-only media\n\0".as_ptr() as *const i8,
            );
            isValid = 0;
        }

        if isValid != 0 {
            match schema {
                FeatureDefectManagement | FeatureRandomWritable => {
                    if (*cdData).Mmc.IsWriter == 0 {
                        if (startingOffset >= (*DeviceExtension).PartitionLength.QuadPart)
                            || startingOffset < 0
                        {
                            TracePrint!(
                                TRACE_LEVEL_ERROR,
                                TRACE_FLAG_RW,
                                b"RequestValidateReadWrite: Request is out of bounds\n\0".as_ptr() as *const i8,
                            );
                            isValid = 0;
                        } else {
                            let bytesRemaining: u64 = (*DeviceExtension).PartitionLength.QuadPart as u64 - startingOffset as u64;
                            if (transferByteCount as u64) > bytesRemaining {
                                TracePrint!(
                                    TRACE_LEVEL_ERROR,
                                    TRACE_FLAG_RW,
                                    b"RequestValidateReadWrite: Request is out of bounds\n\0".as_ptr() as *const i8,
                                );
                                isValid = 0;
                            }
                        }
                    }
                }
                FeatureIncrementalStreamingWritable => {}
                _ => {
                    if schema == FeatureRigidRestrictedOverwrite {
                        if ((transferByteCount >> (*DeviceExtension).SectorShift) as u32 % (*cdData).Mmc.Blocking) != 0 {
                            TracePrint!(
                                TRACE_LEVEL_ERROR,
                                TRACE_FLAG_RW,
                                b"RequestValidateReadWrite: Number of blocks is not a multiple of the blocking size (%x)\n\0".as_ptr() as *const i8,
                                (*cdData).Mmc.Blocking,
                            );
                            isValid = 0;
                        }
                        // fall through to boundary check
                    }
                    if (schema == FeatureRigidRestrictedOverwrite) || (schema == FeatureRestrictedOverwrite) {
                        if (Int64ShrlMod32(startingOffset, (*DeviceExtension).SectorShift as u32) as u32 % (*cdData).Mmc.Blocking) != 0 {
                            TracePrint!(
                                TRACE_LEVEL_ERROR,
                                TRACE_FLAG_RW,
                                b"RequestValidateReadWrite: Starting block is not a multiple of the blocking size (%x)\n\0".as_ptr() as *const i8,
                                (*cdData).Mmc.Blocking,
                            );
                            isValid = 0;
                        }
                    }
                    // unknown schema
                    if (schema != FeatureRigidRestrictedOverwrite) && (schema != FeatureRestrictedOverwrite) {
                        TracePrint!(
                            TRACE_LEVEL_ERROR,
                            TRACE_FLAG_RW,
                            b"RequestValidateReadWrite: Unknown validation schema (%x)\n\0".as_ptr() as *const i8,
                            schema,
                        );
                        isValid = 0;
                    }
                }
            }
        }

        if isValid == 0 {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    if NT_SUCCESS(status) {
        if RequestIsRealtimeStreaming(Request, (requestType == WdfRequestTypeRead) as BOOLEAN) != 0 {
            if requestType == WdfRequestTypeRead && ((*cdData).Mmc.StreamingReadSupported == 0) {
                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_RW,
                    b"RequestValidateReadWrite: Streaming reads are not supported.\n\0".as_ptr() as *const i8,
                );
                status = STATUS_INVALID_DEVICE_REQUEST;
            }
            if requestType == WdfRequestTypeWrite && ((*cdData).Mmc.StreamingWriteSupported == 0) {
                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_RW,
                    b"RequestValidateReadWrite: Streaming writes are not supported.\n\0".as_ptr() as *const i8,
                );
                status = STATUS_INVALID_DEVICE_REQUEST;
            }
        }
    }

    CdromDbgPrintReadWriteValidate(
        b"[RWV] type=%x off=%I64x len=%Ix bps=%x part=%I64x schema=%x valid=%x st=%x\n\0".as_ptr() as *const i8,
        requestType as usize,
        startingOffset as u64,
        transferByteCount,
        (*DeviceExtension).DiskGeometry.BytesPerSector as usize,
        (*DeviceExtension).PartitionLength.QuadPart as u64,
        (*cdData).Mmc.ValidationSchema as usize,
        isValid as usize,
        status as usize,
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleReadWrite(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: *mut WDF_REQUEST_PARAMETERS,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let cdData: *mut CDROM_DATA = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData);

    let mut transferByteCount: usize = 0;
    let mut requestType: ULONG = (*RequestParameters).Type;
    let mut requestOffset: i64 = 0;
    let irp: *mut IRP;
    let currentStack: *mut IO_STACK_LOCATION;
    let dataBuffer: *mut u8;

    irp = WdfRequestWdmGetIrp(Request);
    currentStack = IoGetCurrentIrpStackLocation(irp);
    dataBuffer = MmGetMdlVirtualAddress((*irp).MdlAddress) as *mut u8;

    if NT_SUCCESS(status) {
        CdromGetReadWriteFields(
            Request,
            &mut requestType,
            &mut requestOffset,
            &mut transferByteCount,
        );

        if transferByteCount == 0 && (*RequestParameters).Type == WdfRequestTypeRead {
            transferByteCount = (*RequestParameters).Parameters.Read.as_ref().Length as usize;
        } else if transferByteCount == 0 {
            transferByteCount = (*RequestParameters).Parameters.Write.as_ref().Length as usize;
        }

        if transferByteCount == 0 {
            status = STATUS_SUCCESS;
            RequestCompletion(DeviceExtension, Request, status, 0);
            return status;
        }

        (*currentStack).Parameters.Read.ByteOffset.QuadPart += (*DeviceExtension).StartingOffset.QuadPart;

        if requestType == WdfRequestTypeRead {
            (*RequestParameters).Parameters.Read.as_mut().DeviceOffset = (*currentStack).Parameters.Read.ByteOffset.QuadPart;
        } else {
            (*RequestParameters).Parameters.Write.as_mut().DeviceOffset = (*currentStack).Parameters.Write.ByteOffset.QuadPart;
        }
    }

    if NT_SUCCESS(status) {
        let entireXferLen: u32 = (*currentStack).Parameters.Read.Length;
        let mut maxLength: u32 = 0;
        let mut packetsCount: u32;

        let readWriteContext: *mut CDROM_SCRATCH_READ_WRITE_CONTEXT;
        let requestContext: *mut CDROM_REQUEST_CONTEXT;
        let originalRequestContext: *mut CDROM_REQUEST_CONTEXT;

        if ((dataBuffer as usize) & (PAGE_SIZE as usize - 1)) == 0 {
            maxLength = (*cdData).MaxPageAlignedTransferBytes;
        } else {
            maxLength = (*cdData).MaxUnalignedTransferBytes;
        }

        packetsCount = entireXferLen / maxLength;
        if entireXferLen % maxLength != 0 {
            packetsCount += 1;
        }

        originalRequestContext = RequestGetContext(Request);

        ScratchBuffer_BeginUse(DeviceExtension);

        readWriteContext = core::ptr::addr_of_mut!((*DeviceExtension).ScratchContext.ScratchReadWriteContext);
        requestContext = RequestGetContext((*DeviceExtension).ScratchContext.ScratchRequest);

        (*readWriteContext).PacketsCount = packetsCount;
        (*readWriteContext).EntireXferLen = entireXferLen;
        (*readWriteContext).MaxLength = maxLength;
        (*readWriteContext).StartingOffset = (*currentStack).Parameters.Read.ByteOffset;
        (*readWriteContext).DataBuffer = dataBuffer;
        (*readWriteContext).TransferedBytes = 0;
        (*readWriteContext).IsRead = if requestType == WdfRequestTypeRead { 1u8 } else { 0u8 };

        (*requestContext).OriginalRequest = Request;
        (*requestContext).DeviceExtension = DeviceExtension;

        (*originalRequestContext).ReadWriteIsCompleted = 0u8;
        (*originalRequestContext).ReadWriteRetryInitialized = 0u8;
        (*originalRequestContext).DeviceExtension = DeviceExtension;

        status = ScratchBuffer_PerformNextReadWrite(DeviceExtension, 1);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleLoadEjectMedia(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let zpoddInfo: *mut ZERO_POWER_ODD_INFO = (*DeviceExtension).ZeroPowerODDInfo;

    // PAGED_CODE(); equivalent: ensure IRQL <= APC_LEVEL
    ntkrnl_sys::ASSERT!(ntkrnl_sys::ke::KeGetCurrentIrql() <= ntkrnl_sys::base::APC_LEVEL);

    *DataLength = 0;

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        // Synchronize with ejection control and ejection cleanup code as
        // well as other eject/load requests.
        WdfWaitLockAcquire((*DeviceExtension).EjectSynchronizationLock, core::ptr::null_mut());

        if (*DeviceExtension).ProtectedLockCount != 0 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_IOCTL,
                b"RequestHandleLoadEjectMedia: call to eject protected locked device - failure\n\0"
                    .as_ptr() as *const i8,
            );
            status = STATUS_DEVICE_BUSY;
        }

        if ntkrnl_sys::base::NT_SUCCESS(status) {
            let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
            let cdb: *mut CDB = srb.Cdb.as_mut_ptr() as *mut CDB;

            ntkrnl_sys::rtl::RtlZeroMemory(
                core::ptr::addr_of_mut!(srb) as *mut core::ffi::c_void,
                core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
            );

            srb.CdbLength = 6;
            (*cdb).START_STOP.as_mut().OperationCode = SCSIOP_START_STOP_UNIT;
            (*cdb).START_STOP.as_mut().set_LoadEject(1);

            if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode
                == IOCTL_STORAGE_EJECT_MEDIA
            {
                (*cdb).START_STOP.as_mut().set_Start(0);

                // We are sending down a soft eject, and in this case we should take an active ref
                // if the command succeeds.
                if !zpoddInfo.is_null()
                    && (*zpoddInfo).LoadingMechanism as u32 == LOADING_MECHANISM_TRAY
                    && (*zpoddInfo).Load == 0
                {
                    (*zpoddInfo).MonitorStartStopUnit = 1; // TRUE
                }
            } else {
                (*cdb).START_STOP.as_mut().set_Start(1);
            }

            status = DeviceSendSrbSynchronously(
                (*DeviceExtension).Device as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(srb),
                core::ptr::null_mut(),
                0u32,
                0, // FALSE
                (Request) as _,
            );

            if !zpoddInfo.is_null() {
                (*zpoddInfo).MonitorStartStopUnit = 0; // FALSE
            }
        }

        WdfWaitLockRelease((*DeviceExtension).EjectSynchronizationLock);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleReserveRelease(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: *mut core::ffi::c_void,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: *mut CDB = core::ptr::null_mut();
    let mut ioctl_code: u32 = 0;

    // PAGED_CODE();
    *DataLength = 0;

    srb = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>() + (core::mem::size_of::<usize>() * 2),
        CDROM_TAG_SRB,
    ) as *mut SCSI_REQUEST_BLOCK;

    if srb.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    if status >= 0 {
        ioctl_code = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
        cdb = core::ptr::addr_of_mut!((*srb).Cdb) as *mut CDB;

        core::ptr::write_bytes(srb as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        // TEST_FLAG expanded inline
        if ((*(*DeviceExtension).PrivateFdoData).HackFlags & FDO_HACK_NO_RESERVE6 as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
            (*srb).CdbLength = 10;
            (*cdb).CDB10.as_mut().OperationCode = if ioctl_code == IOCTL_STORAGE_RESERVE {
                SCSIOP_RESERVE_UNIT10
            } else {
                SCSIOP_RELEASE_UNIT10
            };
        } else {
            (*srb).CdbLength = 6;
            (*cdb).CDB6GENERIC.as_mut().OperationCode = if ioctl_code == IOCTL_STORAGE_RESERVE {
                SCSIOP_RESERVE_UNIT
            } else {
                SCSIOP_RELEASE_UNIT
            };
        }

        (*srb).TimeOutValue = (*DeviceExtension).TimeOutValue;

        if ioctl_code == IOCTL_STORAGE_RESERVE {
            // SET_FLAG expanded inline
            (*srb).SrbFlags |= SRB_FLAGS_QUEUE_ACTION_ENABLE as u32;
            (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST;
        }

        status = DeviceSendSrbSynchronously(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            srb,
            core::ptr::null_mut(),
            0,
            0,
            (Request as WDFREQUEST) as _,
        );
        *DataLength = 0;

        // FREE_POOL expanded to ExFreePool
        ExFreePool(srb as *mut core::ffi::c_void);
    }

    status
}

unsafe fn ValidPersistentReserveScope(Scope: u8) -> u8 {
    match Scope {
        0x00 /* RESERVATION_SCOPE_LU */ | 0x01 /* RESERVATION_SCOPE_ELEMENT */ => {
            return 1;
        }
        _ => {
            return 0;
        }
    }
}

unsafe fn ValidPersistentReserveType(Type: u8) -> u8 {
    unsafe {
        match Type {
            /* RESERVATION_TYPE_WRITE_EXCLUSIVE */ 0x01u8
            | /* RESERVATION_TYPE_EXCLUSIVE */ 0x03u8
            | /* RESERVATION_TYPE_WRITE_EXCLUSIVE_REGISTRANTS */ 0x05u8
            | /* RESERVATION_TYPE_EXCLUSIVE_REGISTRANTS */ 0x06u8 => 1,
            _ => 0,
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RequestValidatePersistentReserve(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let ioctlCode: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
    let mut reserveCommand: *mut PERSISTENT_RESERVE_COMMAND = core::ptr::null_mut();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &mut reserveCommand as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
            < core::mem::size_of::<PERSISTENT_RESERVE_COMMAND>()
            || (*reserveCommand).Size < core::mem::size_of::<PERSISTENT_RESERVE_COMMAND>() as u32
        {
            *DataLength = 0;
            status = STATUS_INFO_LENGTH_MISMATCH;
        } else if (reserveCommand as usize
            & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize)
            != 0
        {
            *DataLength = 0;
            status = STATUS_INVALID_USER_BUFFER;
        }
    }

    if status >= 0 {
        if ioctlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_IN {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength
                < (*reserveCommand).__bindgen_anon_1.PR_IN.as_ref().AllocationLength as usize
            {
                *DataLength = 0;
                status = STATUS_INVALID_PARAMETER;
            } else {
                match (*reserveCommand).__bindgen_anon_1.PR_IN.as_ref().ServiceAction() {
                    RESERVATION_ACTION_READ_KEYS => {
                        if (usize::from((*reserveCommand).__bindgen_anon_1.PR_IN.as_ref().AllocationLength))
                            < core::mem::size_of::<PRI_REGISTRATION_LIST>()
                        {
                            *DataLength = 0;
                            status = STATUS_INVALID_PARAMETER;
                        }
                    }
                    RESERVATION_ACTION_READ_RESERVATIONS => {
                        if (usize::from((*reserveCommand).__bindgen_anon_1.PR_IN.as_ref().AllocationLength))
                            < core::mem::size_of::<PRI_RESERVATION_LIST>()
                        {
                            *DataLength = 0;
                            status = STATUS_INVALID_PARAMETER;
                        }
                    }
                    _ => {
                        *DataLength = 0;
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
            }
        } else {
            // case of IOCTL_STORAGE_PERSISTENT_RESERVE_OUT
            match (*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().ServiceAction() {
                RESERVATION_ACTION_REGISTER
                | RESERVATION_ACTION_REGISTER_IGNORE_EXISTING
                | RESERVATION_ACTION_CLEAR => {
                    // Scope and type ignored.
                }
                RESERVATION_ACTION_RESERVE
                | RESERVATION_ACTION_RELEASE
                | RESERVATION_ACTION_PREEMPT
                | RESERVATION_ACTION_PREEMPT_ABORT => {
                    if ValidPersistentReserveScope((*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().Scope()) == 0
                        || ValidPersistentReserveType((*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().Type()) == 0
                    {
                        *DataLength = 0;
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
                _ => {
                    *DataLength = 0;
                    status = STATUS_INVALID_PARAMETER;
                }
            }

            if status >= 0 {
                if RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                    < (core::mem::size_of::<PERSISTENT_RESERVE_COMMAND>()
                        + core::mem::size_of::<PRO_PARAMETER_LIST>())
                    || ((*reserveCommand).Size as usize)
                        < RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength
                {
                    *DataLength = 0;
                    status = STATUS_INVALID_PARAMETER;
                }
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandlePersistentReserve(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let ioctlCode: u32 = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: *mut CDB = core::ptr::null_mut();
    let mut writeToDevice: u8 = 0; // will be set later
    let mut reserveCommand: PPERSISTENT_RESERVE_COMMAND = core::ptr::null_mut();

    PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        &mut reserveCommand as *mut _ as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        srb = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>() + (core::mem::size_of::<ULONG_PTR>() * 2),
            CDROM_TAG_SRB,
        ) as *mut SCSI_REQUEST_BLOCK;

        if srb.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            cdb = &mut (*srb).Cdb as *mut _ as *mut CDB;
            RtlZeroMemory(srb as *mut core::ffi::c_void, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
        }
    }

    if NT_SUCCESS(status) {
        let mut dataBufLen: usize = 0;

        if ioctlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_IN {
            (*cdb).PERSISTENT_RESERVE_IN.as_mut().OperationCode = SCSIOP_PERSISTENT_RESERVE_IN;
            (*cdb).PERSISTENT_RESERVE_IN.as_mut().set_ServiceAction((*reserveCommand).__bindgen_anon_1.PR_IN.as_ref().ServiceAction());

            REVERSE_BYTES_SHORT(
                (*cdb).PERSISTENT_RESERVE_IN.as_mut().AllocationLength.as_mut_ptr() as *mut core::ffi::c_void,
                &(*reserveCommand).__bindgen_anon_1.PR_IN.as_ref().AllocationLength as *const u16 as *const core::ffi::c_void,
            );

            dataBufLen = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
            writeToDevice = 0; // FALSE
        } else {
            (*cdb).PERSISTENT_RESERVE_OUT.as_mut().OperationCode = SCSIOP_PERSISTENT_RESERVE_OUT;
            (*cdb).PERSISTENT_RESERVE_OUT.as_mut().set_ServiceAction((*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().ServiceAction());
            (*cdb).PERSISTENT_RESERVE_OUT.as_mut().set_Scope((*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().Scope());
            (*cdb).PERSISTENT_RESERVE_OUT.as_mut().set_Type((*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().Type());

            (*cdb).PERSISTENT_RESERVE_OUT.as_mut().ParameterListLength[1] =
                core::mem::size_of::<PRO_PARAMETER_LIST>() as u8;

            // Move the parameter list to the beginning of the data buffer.
            // src (field inside reserveCommand) overlaps dst (its base): C uses
            // RtlMoveMemory; RtlCopyMemory here = copy_nonoverlapping = UB on
            // overlap. Use ptr::copy (memmove). Found by rustconv semdiff.
            core::ptr::copy(
                (*reserveCommand).__bindgen_anon_1.PR_OUT.as_ref().ParameterList.as_ptr() as *const u8,
                reserveCommand as *mut u8,
                core::mem::size_of::<PRO_PARAMETER_LIST>(),
            );

            dataBufLen = core::mem::size_of::<PRO_PARAMETER_LIST>();
            writeToDevice = 1; // TRUE
        }

        (*srb).CdbLength = 10;
        (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST;

        status = DeviceSendSrbSynchronously(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            srb,
            reserveCommand as *mut core::ffi::c_void,
            dataBufLen as u32,
            writeToDevice,
            (Request) as _,
        );

        ExFreePool(srb as *mut core::ffi::c_void);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleAreVolumesReady(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut completeRequest: BOOLEAN = 1;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    let _ = &RequestParameters;

    *DataLength = 0;

    loop {
        if (*DeviceExtension).IsVolumeOnlinePending == 0 {
            status = STATUS_SUCCESS;
            break;
        }

        status = WdfRequestForwardToIoQueue(Request, (*DeviceExtension).ManualVolumeReadyQueue);

        if (!NT_SUCCESS(status)) {
            break;
        }

        status = STATUS_PENDING;
        completeRequest = 0;
        break;
    }

    if completeRequest != 0 {
        RequestCompletion(DeviceExtension, Request, status, (0u32) as usize);
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn RequestHandleVolumeOnline(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: *mut core::ffi::c_void,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut request: WDFREQUEST = core::ptr::null_mut();
    let mut irp: *mut IRP = core::ptr::null_mut();
    let mut nextStack: *mut IO_STACK_LOCATION = core::ptr::null_mut();

    let _ = RequestParameters;

    *DataLength = 0;

    (*DeviceExtension).IsVolumeOnlinePending = 0; // FALSE

    // Complete all parked volume ready requests.
    loop {
        status = WdfIoQueueRetrieveNextRequest(
            (*DeviceExtension).ManualVolumeReadyQueue,
            core::ptr::addr_of_mut!(request),
        );

        if (!NT_SUCCESS(status)) {
            break;
        }

        RequestCompletion(DeviceExtension, request, STATUS_SUCCESS, 0);
    }

    // Change the IOCTL code to IOCTL_DISK_VOLUMES_ARE_READY, and forward the request down,
    // so that if the underlying port driver will also know that volume is ready.
    WdfRequestFormatRequestUsingCurrentType(Request as WDFREQUEST);
    irp = WdfRequestWdmGetIrp(Request as WDFREQUEST);
    nextStack = IoGetNextIrpStackLocation(irp);

    (*irp).AssociatedIrp.SystemBuffer = core::ptr::addr_of!((*DeviceExtension).DeviceNumber) as *mut core::ffi::c_void;
    (*nextStack).Parameters.DeviceIoControl.OutputBufferLength = 0;
    (*nextStack).Parameters.DeviceIoControl.InputBufferLength = core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32;
    (*nextStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_DISK_VOLUMES_ARE_READY;

    // Send the request straight down (synchronously).
    RequestSend(
        DeviceExtension,
        Request as WDFREQUEST,
        (*DeviceExtension).IoTarget,
        WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
        core::ptr::null_mut(),
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleRawRead(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let _ = &RequestParameters;
    *DataLength = 0;

    let mut status: i32 = STATUS_SUCCESS;
    let cdData = &(*DeviceExtension).DeviceAdditionalData as *const CDROM_DATA;
    let mut rawReadInfo: RAW_READ_INFO = core::mem::zeroed();
    let mut outputVirtAddr: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut startingSector: u32 = 0;

    let irp = WdfRequestWdmGetIrp(Request);
    let currentIrpStack = IoGetCurrentIrpStackLocation(irp);

    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    *DataLength = 0;

    if ((*(*DeviceExtension).DeviceObject).Flags & DO_VERIFY_VOLUME) != 0 &&
        (*currentIrpStack).MinorFunction as u32 != CDROM_VOLUME_VERIFY_CHECKED &&
        ((*currentIrpStack).Flags as u32 & SL_OVERRIDE_VERIFY_VOLUME) == 0
    {
        MediaReadCapacity((*DeviceExtension).Device as PDEVICE_OBJECT);
        *DataLength = 0;
        status = STATUS_VERIFY_REQUIRED;
    }

    if status >= 0 {
        rawReadInfo = core::ptr::read(
            (*currentIrpStack).Parameters.DeviceIoControl.Type3InputBuffer as *const RAW_READ_INFO,
        );

        if (*currentIrpStack).Parameters.DeviceIoControl.OutputBufferLength > 0 {
            ntkrnl_sys::ASSERT!(!(*irp).MdlAddress.is_null());
            outputVirtAddr = MmGetMdlVirtualAddress((*irp).MdlAddress);
            if (outputVirtAddr as usize) & (*(*DeviceExtension).AdapterDescriptor).AlignmentMask as usize != 0 {
                ntkrnl_sys::ASSERT!(false);
                *DataLength = 0;
                status = STATUS_INVALID_PARAMETER;
            }
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &mut outputBuffer,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        match rawReadInfo.TrackMode {
            CDDA | YellowMode2 | XAForm2 => {}
            RawWithC2AndSubCode => {
                if (*cdData).Mmc.ReadCdC2Pointers == 0 || (*cdData).Mmc.ReadCdSubCode == 0 {
                    // TracePrint omitted
                    *DataLength = 0;
                    status = STATUS_INVALID_DEVICE_REQUEST;
                }
            }
            RawWithC2 => {
                if (*cdData).Mmc.ReadCdC2Pointers == 0 {
                    // TracePrint omitted
                    *DataLength = 0;
                    status = STATUS_INVALID_DEVICE_REQUEST;
                }
            }
            RawWithSubCode => {
                if (*cdData).Mmc.ReadCdSubCode == 0 {
                    // TracePrint omitted
                    *DataLength = 0;
                    status = STATUS_INVALID_DEVICE_REQUEST;
                }
            }
            _ => {
                *DataLength = 0;
                status = STATUS_INVALID_DEVICE_REQUEST;
            }
        }
    }

    if status >= 0 {
        let srb = (*DeviceExtension).ScratchContext.ScratchSrb;
        let cdb = (*srb).Cdb.as_mut_ptr() as *mut CDB;
        let mut transferByteCount: usize = 0;
        let mut shouldRetry: i32 = 1;
        let mut timesAlreadyRetried: u32 = 0;
        let mut retryIn100nsUnits: i64 = 0;

        transferByteCount = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize;

        ScratchBuffer_BeginUse(DeviceExtension);

        while shouldRetry != 0 {
            match rawReadInfo.TrackMode {
                CDDA => {
                    transferByteCount = rawReadInfo.SectorCount as usize * RAW_SECTOR_SIZE as usize;
                    (*cdb).READ_CD.as_mut().set_ExpectedSectorType(CD_DA_SECTOR as u8);
                    (*cdb).READ_CD.as_mut().set_IncludeUserData(1);
                    (*cdb).READ_CD.as_mut().set_HeaderCode(3);
                    (*cdb).READ_CD.as_mut().set_IncludeSyncData(1);
                }
                YellowMode2 => {
                    transferByteCount = rawReadInfo.SectorCount as usize * RAW_SECTOR_SIZE as usize;
                    (*cdb).READ_CD.as_mut().set_ExpectedSectorType(YELLOW_MODE2_SECTOR as u8);
                    (*cdb).READ_CD.as_mut().set_IncludeUserData(1);
                    (*cdb).READ_CD.as_mut().set_HeaderCode(1);
                    (*cdb).READ_CD.as_mut().set_IncludeSyncData(1);
                }
                XAForm2 => {
                    transferByteCount = rawReadInfo.SectorCount as usize * RAW_SECTOR_SIZE as usize;
                    (*cdb).READ_CD.as_mut().set_ExpectedSectorType(FORM2_MODE2_SECTOR as u8);
                    (*cdb).READ_CD.as_mut().set_IncludeUserData(1);
                    (*cdb).READ_CD.as_mut().set_HeaderCode(3);
                    (*cdb).READ_CD.as_mut().set_IncludeSyncData(1);
                }
                RawWithC2AndSubCode => {
                    transferByteCount = rawReadInfo.SectorCount as usize * CD_RAW_SECTOR_WITH_C2_AND_SUBCODE_SIZE as usize;
                    (*cdb).READ_CD.as_mut().set_ExpectedSectorType(0);
                    (*cdb).READ_CD.as_mut().set_IncludeUserData(1);
                    (*cdb).READ_CD.as_mut().set_HeaderCode(3);
                    (*cdb).READ_CD.as_mut().set_IncludeSyncData(1);
                    (*cdb).READ_CD.as_mut().set_ErrorFlags(2);
                    (*cdb).READ_CD.as_mut().set_SubChannelSelection(1);
                }
                RawWithC2 => {
                    transferByteCount = rawReadInfo.SectorCount as usize * CD_RAW_SECTOR_WITH_C2_SIZE as usize;
                    (*cdb).READ_CD.as_mut().set_ExpectedSectorType(0);
                    (*cdb).READ_CD.as_mut().set_IncludeUserData(1);
                    (*cdb).READ_CD.as_mut().set_HeaderCode(3);
                    (*cdb).READ_CD.as_mut().set_IncludeSyncData(1);
                    (*cdb).READ_CD.as_mut().set_ErrorFlags(2);
                }
                RawWithSubCode => {
                    transferByteCount = rawReadInfo.SectorCount as usize * CD_RAW_SECTOR_WITH_SUBCODE_SIZE as usize;
                    (*cdb).READ_CD.as_mut().set_ExpectedSectorType(0);
                    (*cdb).READ_CD.as_mut().set_IncludeUserData(1);
                    (*cdb).READ_CD.as_mut().set_HeaderCode(3);
                    (*cdb).READ_CD.as_mut().set_IncludeSyncData(1);
                    (*cdb).READ_CD.as_mut().set_SubChannelSelection(1);
                }
                _ => {
                    ntkrnl_sys::ASSERT!(false);
                }
            }

            ScratchBuffer_SetupSrb(DeviceExtension, (Request) as _, transferByteCount as u32, 1);
            {
                let scratchIrp = WdfRequestWdmGetIrp((*DeviceExtension).ScratchContext.ScratchRequest);
                (*scratchIrp).MdlAddress = (*irp).MdlAddress;
                (*srb).DataBuffer = outputVirtAddr;
                (*srb).DataTransferLength = transferByteCount as u32;
            }

            startingSector = (rawReadInfo.DiskOffset.QuadPart as usize >> (*DeviceExtension).SectorShift as usize) as u32;

            (*srb).CdbLength = 12;

            (*cdb).READ_CD.as_mut().OperationCode = SCSIOP_READ_CD as u8;
            (*cdb).READ_CD.as_mut().TransferBlocks[2] = (rawReadInfo.SectorCount & 0xFF) as u8;
            (*cdb).READ_CD.as_mut().TransferBlocks[1] = ((rawReadInfo.SectorCount >> 8) & 0xFF) as u8;
            (*cdb).READ_CD.as_mut().TransferBlocks[0] = ((rawReadInfo.SectorCount >> 16) & 0xFF) as u8;

            (*cdb).READ_CD.as_mut().StartingLBA[3] = (startingSector & 0xFF) as u8;
            (*cdb).READ_CD.as_mut().StartingLBA[2] = ((startingSector >> 8) & 0xFF) as u8;
            (*cdb).READ_CD.as_mut().StartingLBA[1] = ((startingSector >> 16) & 0xFF) as u8;
            (*cdb).READ_CD.as_mut().StartingLBA[0] = ((startingSector >> 24) & 0xFF) as u8;

            ScratchBuffer_SendSrb(DeviceExtension, 1, core::ptr::null_mut());

            if (*srb).SrbStatus as u32 == SRB_STATUS_ABORTED && *(*srb).__bindgen_anon_1.InternalStatus.as_ref() == STATUS_CANCELLED as u32 {
                shouldRetry = 0;
                status = STATUS_CANCELLED;
            } else {
                shouldRetry = RequestSenseInfoInterpretForScratchBuffer(
                    DeviceExtension,
                    timesAlreadyRetried,
                    &mut status,
                    &mut retryIn100nsUnits,
                );
                if shouldRetry != 0 {
                    let mut t: LARGE_INTEGER = core::mem::zeroed();
                    t.QuadPart = -retryIn100nsUnits;
                    timesAlreadyRetried += 1;
                    KeDelayExecutionThread(0i8, 0, &mut t);
                    ScratchBuffer_ResetItems(DeviceExtension, 0);
                }
            }
        }

        if status >= 0 {
            *DataLength = (*srb).DataTransferLength as usize;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandlePlayAudioMsf(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut input_buffer: *mut CDROM_PLAY_AUDIO_MSF = core::ptr::null_mut();

    // PAGED_CODE();
    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &raw mut input_buffer as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        ScratchBuffer_BeginUse(DeviceExtension);
        let mut transfer_size: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        // RtlZeroMemory(&cdb, sizeof(CDB));
        core::ptr::write_bytes((core::ptr::addr_of_mut!(cdb)) as *mut u8, 0u8, core::mem::size_of::<CDB>());

        // Set up the CDB
        cdb.PLAY_AUDIO_MSF.as_mut().OperationCode = 0x47u8; // SCSIOP_PLAY_AUDIO_MSF

        cdb.PLAY_AUDIO_MSF.as_mut().StartingM = (*input_buffer).StartingM;
        cdb.PLAY_AUDIO_MSF.as_mut().StartingS = (*input_buffer).StartingS;
        cdb.PLAY_AUDIO_MSF.as_mut().StartingF = (*input_buffer).StartingF;

        cdb.PLAY_AUDIO_MSF.as_mut().EndingM = (*input_buffer).EndingM;
        cdb.PLAY_AUDIO_MSF.as_mut().EndingS = (*input_buffer).EndingS;
        cdb.PLAY_AUDIO_MSF.as_mut().EndingF = (*input_buffer).EndingF;

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            transfer_size,
            0, // FALSE
            core::ptr::addr_of_mut!(cdb),
            10,
        );

        if status >= 0 {
            // PLAY_ACTIVE(DeviceExtension) = TRUE;
            (*DeviceExtension).DeviceAdditionalData.PlayActive = 1;
            *DataLength = 0;
        }

        // nothing to do after the command finishes.
        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleReadQChannel(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut inputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // PAGED_CODE ();
    if KeGetCurrentIrql() > APC_LEVEL {
        RtlAssert(
            b"KeGetCurrentIrql() <= APC_LEVEL\0".as_ptr() as *mut core::ffi::c_void,
            b"ioctl.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            b"PAGED_CODE\0".as_ptr() as *mut i8,
        );
    }

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        core::ptr::addr_of_mut!(inputBuffer),
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            core::ptr::addr_of_mut!(outputBuffer),
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        status = ReadQChannel(
            DeviceExtension,
            Request,
            inputBuffer,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            outputBuffer,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            DataLength,
        );
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DeviceHandlePauseAudio(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = ntkrnl_sys::base::STATUS_SUCCESS;
    // PAGED_CODE();
    *DataLength = 0;
    if ntkrnl_sys::base::NT_SUCCESS(status) {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        ntkrnl_sys::rtl::RtlZeroMemory(
            &mut cdb as *mut CDB as *mut core::ffi::c_void,
            core::mem::size_of::<CDB>(),
        );

        ScratchBuffer_BeginUse(DeviceExtension);

        cdb.PAUSE_RESUME.as_mut().OperationCode = 0x4B; // SCSIOP_PAUSE_RESUME
        cdb.PAUSE_RESUME.as_mut().Action = 0; // CDB_AUDIO_PAUSE

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            transferSize,
            (0i32) as _, // FALSE
            core::ptr::addr_of_mut!(cdb),
            10u8,
        );

        if ntkrnl_sys::base::NT_SUCCESS(status) {
            (*DeviceExtension).DeviceAdditionalData.PlayActive = 0;
            *DataLength = 0;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleResumeAudio(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> NTSTATUS {
    if ntkrnl_sys::ke::KeGetCurrentIrql() > ntkrnl_sys::base::APC_LEVEL {
        RtlAssert(
            b"KeGetCurrentIrql() <= APC_LEVEL\0".as_ptr() as *mut core::ffi::c_void,
            b"ioctl.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            b"PAGED_CODE\0".as_ptr() as *mut i8,
        );
    }
    let mut status: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS;
    *DataLength = 0;
    if ntkrnl_sys::base::NT_SUCCESS(status) {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        ScratchBuffer_BeginUse(DeviceExtension);
        ntkrnl_sys::rtl::RtlZeroMemory(
            &mut cdb as *mut CDB as *mut core::ffi::c_void,
            core::mem::size_of::<CDB>(),
        );
        cdb.PAUSE_RESUME.as_mut().OperationCode = SCSIOP_PAUSE_RESUME as u8;
        cdb.PAUSE_RESUME.as_mut().Action = CDB_AUDIO_RESUME as u8;
        status = ScratchBuffer_ExecuteCdb(DeviceExtension, (Request) as _, transferSize, 0, &mut cdb, 10);
        if ntkrnl_sys::base::NT_SUCCESS(status) {
            (*DeviceExtension).DeviceAdditionalData.PlayActive = 1;
            *DataLength = 0;
        }
        ScratchBuffer_EndUse(DeviceExtension);
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleSeekAudioMsf(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut inputBuffer: *mut CDROM_SEEK_AUDIO_MSF = core::ptr::null_mut();

    // PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters
            .Parameters
            .DeviceIoControl
            .as_ref()
            .InputBufferLength as usize,
        &mut inputBuffer as *mut *mut CDROM_SEEK_AUDIO_MSF as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        let logicalBlockAddress: u32 = MSF_TO_LBA(
            (*inputBuffer).M,
            (*inputBuffer).S,
            (*inputBuffer).F,
        );

        ScratchBuffer_BeginUse(DeviceExtension);

        core::ptr::write_bytes(
            &mut cdb as *mut CDB as *mut u8,
            0,
            core::mem::size_of::<CDB>(),
        );
        // Set up the CDB
        cdb.SEEK.as_mut().OperationCode = SCSIOP_SEEK;
        let addr_bytes = &logicalBlockAddress as *const u32 as *const FOUR_BYTE;
        cdb.SEEK.as_mut().LogicalBlockAddress[0] = (*addr_bytes).__bindgen_anon_1.as_ref().Byte3;
        cdb.SEEK.as_mut().LogicalBlockAddress[1] = (*addr_bytes).__bindgen_anon_1.as_ref().Byte2;
        cdb.SEEK.as_mut().LogicalBlockAddress[2] = (*addr_bytes).__bindgen_anon_1.as_ref().Byte1;
        cdb.SEEK.as_mut().LogicalBlockAddress[3] = (*addr_bytes).__bindgen_anon_1.as_ref().Byte0;

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            transferSize,
            (0_i32) as _, // FALSE
            &mut cdb,
            10_u8,
        );

        if status >= 0 {
            *DataLength = 0;
        }

        // nothing to do after the command finishes.

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleStopAudio(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;

    PAGED_CODE();

    *DataLength = 0;

    if NT_SUCCESS(status) {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        RtlZeroMemory(
            core::ptr::addr_of_mut!(cdb).cast(),
            core::mem::size_of::<CDB>(),
        );

        cdb.START_STOP.as_mut().OperationCode = SCSIOP_START_STOP_UNIT;
        cdb.START_STOP.as_mut().set_Immediate(1);
        cdb.START_STOP.as_mut().set_Start(0);
        cdb.START_STOP.as_mut().set_LoadEject(0);

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request) as _,
            transferSize,
            (0i32) as _,
            core::ptr::addr_of_mut!(cdb),
            6u8,
        );

        if NT_SUCCESS(status) {
            *PLAY_ACTIVE(DeviceExtension) = 0i32;
            *DataLength = 0;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}

#[no_mangle]
pub unsafe extern "C" fn DeviceHandleGetSetVolume(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    PAGED_CODE();
    *DataLength = 0;

    if NT_SUCCESS(status) {
        ScratchBuffer_BeginUse(DeviceExtension);
        let transferSize: u32 = MODE_DATA_SIZE;
        let mut cdb: CDB = core::mem::zeroed();
        core::ptr::write_bytes(core::ptr::addr_of_mut!(cdb) as *mut u8, 0, core::mem::size_of::<CDB>());
        cdb.MODE_SENSE10.as_mut().OperationCode = SCSIOP_MODE_SENSE10 as u8;
        cdb.MODE_SENSE10.as_mut().set_PageCode(CDROM_AUDIO_CONTROL_PAGE as u8);
        cdb.MODE_SENSE10.as_mut().AllocationLength[0] = (MODE_DATA_SIZE >> 8) as u8;
        cdb.MODE_SENSE10.as_mut().AllocationLength[1] = (MODE_DATA_SIZE & 0xFF) as u8;

        status = ScratchBuffer_ExecuteCdb(DeviceExtension, (Request) as _, transferSize, 1, core::ptr::addr_of_mut!(cdb), 10);

        if NT_SUCCESS(status) {
            if RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_CDROM_GET_VOLUME {
                let audioOutput: *mut AUDIO_OUTPUT;
                let mut volumeControl: *mut VOLUME_CONTROL = core::ptr::null_mut();
                let mut bytesTransferred: u32;

                status = WdfRequestRetrieveOutputBuffer(
                    Request,
                    RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
                    core::ptr::addr_of_mut!(volumeControl) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(status) {
                    audioOutput = ModeSenseFindSpecificPage(
                        (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer as *mut i8,
                        (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize,
                        (CDROM_AUDIO_CONTROL_PAGE as u32) as _,
                        0,
                    ) as *mut AUDIO_OUTPUT;

                    bytesTransferred = ((audioOutput as *const u8).offset_from((*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer as *const u8) as u32)
                        .wrapping_add(core::mem::size_of::<AUDIO_OUTPUT>() as u32);

                    if !audioOutput.is_null()
                        && (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength >= bytesTransferred
                    {
                        for i in 0u32..4u32 {
                            (*volumeControl).PortVolume[i as usize] = (*audioOutput).PortOutput[i as usize].Volume;
                        }
                        *DataLength = core::mem::size_of::<VOLUME_CONTROL>() as usize;
                    } else {
                        *DataLength = 0;
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    }
                }
            } else {
                // IOCTL_CDROM_SET_VOLUME
                let mut audioInput: *mut AUDIO_OUTPUT = core::ptr::null_mut();
                let audioOutput: *mut AUDIO_OUTPUT;
                let mut volumeControl: *mut VOLUME_CONTROL = core::ptr::null_mut();
                let mut i: u32;
                let mut bytesTransferred: u32;
                let headerLength: u32;

                status = WdfRequestRetrieveInputBuffer(
                    Request,
                    RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
                    core::ptr::addr_of_mut!(volumeControl) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(status) {
                    audioInput = ModeSenseFindSpecificPage(
                        (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer as *mut i8,
                        (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength as usize,
                        (CDROM_AUDIO_CONTROL_PAGE as u32) as _,
                        0,
                    ) as *mut AUDIO_OUTPUT;

                    if audioInput.is_null() {
                        TracePrint!(
                            3, // TRACE_LEVEL_WARNING
                            1, // TRACE_FLAG_IOCTL (assumed value)
                            b"Mode Sense Page %d not found\n\0".as_ptr() as *const i8,
                            CDROM_AUDIO_CONTROL_PAGE as i32,
                        );
                        *DataLength = 0;
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    }
                }

                if NT_SUCCESS(status) {
                    ScratchBuffer_ResetItems(DeviceExtension, 1);

                    headerLength = core::mem::size_of::<MODE_PARAMETER_HEADER10>() as u32;
                    bytesTransferred = core::mem::size_of::<AUDIO_OUTPUT>() as u32 + headerLength;

                    audioOutput = ((*DeviceExtension).ScratchContext.ScratchBuffer as *mut u8)
                        .add(headerLength as usize) as *mut AUDIO_OUTPUT;

                    for i in 0u32..4u32 {
                        (*audioOutput).PortOutput[i as usize].Volume = (*volumeControl).PortVolume[i as usize];
                        (*audioOutput).PortOutput[i as usize].ChannelSelection = (*audioInput).PortOutput[i as usize].ChannelSelection;
                    }

                    (*audioOutput).CodePage = CDROM_AUDIO_CONTROL_PAGE as u8;
                    (*audioOutput).ParameterLength = (core::mem::size_of::<AUDIO_OUTPUT>() as u32).wrapping_sub(2) as u8;
                    (*audioOutput).Immediate = MODE_SELECT_IMMEDIATE as u8;

                    core::ptr::write_bytes(core::ptr::addr_of_mut!(cdb) as *mut u8, 0, core::mem::size_of::<CDB>());
                    cdb.MODE_SELECT10.as_mut().OperationCode = SCSIOP_MODE_SELECT10 as u8;
                    cdb.MODE_SELECT10.as_mut().ParameterListLength[0] = (bytesTransferred >> 8) as u8;
                    cdb.MODE_SELECT10.as_mut().ParameterListLength[1] = (bytesTransferred & 0xFF) as u8;
                    cdb.MODE_SELECT10.as_mut().set_PFBit(1);

                    status = ScratchBuffer_ExecuteCdb(DeviceExtension, (Request) as _, bytesTransferred, 0, core::ptr::addr_of_mut!(cdb), 10);
                }
                *DataLength = 0;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleReadDvdStructure(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let mut input_buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut output_buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    // PAGED_CODE ();

    *DataLength = 0;

    let req_params = &RequestParameters;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        req_params.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut input_buffer,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            req_params.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            &mut output_buffer,
            core::ptr::null_mut(),
        );
    }

    if status >= 0 {
        status = ReadDvdStructure(
            DeviceExtension,
            Request,
            input_buffer,
            req_params.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            output_buffer,
            req_params.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            DataLength,
        );
    }

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceHandleDvdEndSession(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut sessionId: *mut DVD_SESSION_ID = core::ptr::null_mut();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        (&mut sessionId) as *mut *mut DVD_SESSION_ID as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(status) {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        let mut currentSession: DVD_SESSION_ID;
        let mut limitSession: DVD_SESSION_ID;

        if *sessionId == DVD_END_ALL_SESSIONS as u32 {
            currentSession = 0;
            limitSession = MAX_COPY_PROTECT_AGID - 1;
        } else {
            currentSession = *sessionId;
            limitSession = *sessionId;
        }

        ScratchBuffer_BeginUse(DeviceExtension);

        loop {
            core::ptr::write_bytes(&mut cdb as *mut CDB as *mut u8, 0, core::mem::size_of::<CDB>());
            cdb.SEND_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
            cdb.SEND_KEY.as_mut().set_AGID(currentSession as u8);
            cdb.SEND_KEY.as_mut().set_KeyFormat(DVD_INVALIDATE_AGID as u8);

            status = ScratchBuffer_ExecuteCdb(
                DeviceExtension,
                (Request) as _,
                transferSize,
                (0i32) as _, // FALSE
                core::ptr::addr_of_mut!(cdb),
                12,
            );

            currentSession += 1;

            if !(currentSession <= limitSession && NT_SUCCESS(status)) {
                break;
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleDvdStartSessionReadKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut keyParameters: *mut DVD_COPY_PROTECT_KEY = core::ptr::null_mut();
    let mut outputBuffer: *mut core::ffi::c_void = core::ptr::null_mut();

    PAGED_CODE();

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            &raw mut outputBuffer,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) && RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_DVD_READ_KEY {
        status = WdfRequestRetrieveInputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            (&raw mut keyParameters) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        status = DvdStartSessionReadKey(
            DeviceExtension,
            RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode,
            Request,
            keyParameters as *mut core::ffi::c_void,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
            outputBuffer,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
            DataLength,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DvdStartSessionReadKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    IoControlCode: u32,
    OriginalRequest: WDFREQUEST,
    InputBuffer: *mut core::ffi::c_void,
    InputBufferLength: usize,
    OutputBuffer: *mut core::ffi::c_void,
    OutputBufferLength: usize,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let mut keyLength: u32 = 0;
    let mut result: u32 = 0;
    let mut allocationLength: u32 = 0; // set to keyLength below (REPORT KEY alloc len)
    let mut fourByte: *mut FOUR_BYTE;
    let keyParameters: *mut DVD_COPY_PROTECT_KEY = InputBuffer as *mut DVD_COPY_PROTECT_KEY;

    // PAGED_CODE(); // ignored

    // UNREFERENCED_PARAMETER(InputBufferLength);
    let _ = InputBufferLength;

    *DataLength = 0;

    // fourByte = (PFOUR_BYTE)&allocationLength;
    fourByte = &allocationLength as *const u32 as *mut FOUR_BYTE;

    if IoControlCode == IOCTL_DVD_READ_KEY {
        if keyParameters.is_null() {
            status = STATUS_INTERNAL_ERROR;
        }

        if status >= 0 &&
            (*keyParameters).KeyType == DvdGetRpcKey &&
            (*DeviceExtension).DeviceAdditionalData.Mmc.IsCssDvd != 0
        {
            DevicePickDvdRegion((*DeviceExtension).Device as *mut core::ffi::c_void);
        }

        if status >= 0 &&
            (*keyParameters).KeyType == DvdDiskKey
        {
            let mut keyHeader: *mut DVD_COPY_PROTECT_KEY = core::ptr::null_mut();
            let readStructureRequest: *mut DVD_READ_STRUCTURE = keyParameters as *mut DVD_READ_STRUCTURE;

            // save the key header
            keyHeader = ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<DVD_COPY_PROTECT_KEY>(),
                DVD_TAG_READ_KEY,
            ) as *mut DVD_COPY_PROTECT_KEY;

            if keyHeader.is_null() {
                // TracePrint!(...) omitted (varargs unsupported)
                status = STATUS_INSUFFICIENT_RESOURCES;
            }

            if status >= 0 {
                let rawKey: *mut READ_DVD_STRUCTURES_HEADER = OutputBuffer as *mut READ_DVD_STRUCTURES_HEADER;
                let outputKey: *mut DVD_COPY_PROTECT_KEY = OutputBuffer as *mut DVD_COPY_PROTECT_KEY;

                core::ptr::copy_nonoverlapping(
                    InputBuffer as *const u8,
                    keyHeader as *mut u8,
                    core::mem::size_of::<DVD_COPY_PROTECT_KEY>(),
                );

                (*readStructureRequest).Format = DvdDiskKeyDescriptor as u32;
                (*readStructureRequest).BlockByteOffset.QuadPart = 0;
                (*readStructureRequest).LayerNumber = 0;
                (*readStructureRequest).SessionId = (*keyHeader).SessionId;

                status = ReadDvdStructure(
                    DeviceExtension,
                    OriginalRequest,
                    keyParameters as *mut core::ffi::c_void,
                    core::mem::size_of::<DVD_READ_STRUCTURE>(),
                    OutputBuffer,
                    core::mem::size_of::<READ_DVD_STRUCTURES_HEADER>() +
                        core::mem::size_of::<DVD_DISK_KEY_DESCRIPTOR>(),
                    DataLength,
                );

                if status >= 0 {
                    // RtlMoveMemory(outputKey->KeyData, rawKey->Data, sizeof(DVD_DISK_KEY_DESCRIPTOR));
                    core::ptr::copy(
                        (*rawKey).Data.as_ptr() as *const u8,
                        (*outputKey).KeyData.as_mut_ptr() as *mut u8,
                        core::mem::size_of::<DVD_DISK_KEY_DESCRIPTOR>(),
                    );

                    // RtlCopyMemory(outputKey, keyHeader, sizeof(DVD_COPY_PROTECT_KEY));
                    core::ptr::copy_nonoverlapping(
                        keyHeader as *const u8,
                        outputKey as *mut u8,
                        core::mem::size_of::<DVD_COPY_PROTECT_KEY>(),
                    );

                    (*outputKey).KeyLength = DVD_DISK_KEY_LENGTH;
                    *DataLength = DVD_DISK_KEY_LENGTH as usize;
                }
                // else TracePrint!(...) omitted
            }

            FREE_POOL(keyHeader as *mut core::ffi::c_void);
            return status;
        }

        if status >= 0 {
            status = RtlULongSub(OutputBufferLength as u32, core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32, &mut result);
        }
        if status >= 0 {
            status = RtlULongAdd(result, core::mem::size_of::<CDVD_KEY_HEADER>() as u32, &mut keyLength);
        }
        if status >= 0 {
            // min(keyLength, MAXUSHORT)
            if keyLength > MAXUSHORT as u32 {
                keyLength = MAXUSHORT as u32;
            }
        }
    } else { // IOCTL_DVD_START_SESSION
        keyLength = core::mem::size_of::<CDVD_KEY_HEADER>() as u32 + core::mem::size_of::<CDVD_REPORT_AGID_DATA>() as u32;
        status = STATUS_SUCCESS;
    }

    if status >= 0 {
        let mut cdb: CDB = core::mem::zeroed();

        allocationLength = keyLength; // fourByte points here → CDB AllocationLength
        core::ptr::write_bytes(&mut cdb as *mut CDB as *mut u8, 0, core::mem::size_of::<CDB>());

        cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_REPORT_KEY;
        cdb.REPORT_KEY.as_mut().AllocationLength[0] = (*fourByte).__bindgen_anon_1.as_ref().Byte1;
        cdb.REPORT_KEY.as_mut().AllocationLength[1] = (*fourByte).__bindgen_anon_1.as_ref().Byte0;

        ScratchBuffer_BeginUse(DeviceExtension);

        if IoControlCode == IOCTL_DVD_READ_KEY {
            if (*keyParameters).KeyType == DvdTitleKey {
                let logicalBlockAddress: u32 = ((*keyParameters).Parameters.TitleOffset.as_ref().QuadPart as u64 >> (*DeviceExtension).SectorShift) as u32;
                fourByte = &logicalBlockAddress as *const u32 as *mut FOUR_BYTE;
                cdb.REPORT_KEY.as_mut().LogicalBlockAddress[0] = (*fourByte).__bindgen_anon_1.as_ref().Byte3;
                cdb.REPORT_KEY.as_mut().LogicalBlockAddress[1] = (*fourByte).__bindgen_anon_1.as_ref().Byte2;
                cdb.REPORT_KEY.as_mut().LogicalBlockAddress[2] = (*fourByte).__bindgen_anon_1.as_ref().Byte1;
                cdb.REPORT_KEY.as_mut().LogicalBlockAddress[3] = (*fourByte).__bindgen_anon_1.as_ref().Byte0;
            }
            cdb.REPORT_KEY.as_mut().set_KeyFormat((*keyParameters).KeyType as u8);
            cdb.REPORT_KEY.as_mut().set_AGID((*keyParameters).SessionId as u8);
            // TracePrint omitted
        } else {
            cdb.REPORT_KEY.as_mut().set_KeyFormat(DVD_REPORT_AGID as u8);
            cdb.REPORT_KEY.as_mut().set_AGID(0);
            // TracePrint omitted
        }

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (OriginalRequest) as _,
            keyLength,
            1, // TRUE
            &mut cdb,
            12,
        );

        if status >= 0 {
            if IoControlCode == IOCTL_DVD_READ_KEY {
                let copyProtectKey: *mut DVD_COPY_PROTECT_KEY = OutputBuffer as *mut DVD_COPY_PROTECT_KEY;
                let keyHeader: *mut CDVD_KEY_HEADER = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer as *mut CDVD_KEY_HEADER;
                let mut transferLength: u32 = 0;
                let dataLength: u32;

                let tempStatus: i32 = RtlULongSub(
                    (*(*DeviceExtension).ScratchContext.ScratchSrb).DataTransferLength,
                    core::mem::offset_of!(CDVD_KEY_HEADER, Data) as u32,
                    &mut transferLength,
                );

                dataLength = ((*keyHeader).DataLength[0] as u32) << 8 | ((*keyHeader).DataLength[1] as u32);

                if tempStatus >= 0 && dataLength >= 2 {
                    let mut adjDataLength = dataLength - 2;
                    if adjDataLength < transferLength {
                        transferLength = adjDataLength;
                    }
                    // TracePrint omitted

                    if (*copyProtectKey).KeyType == DvdTitleKey {
                        core::ptr::copy(
                            (*keyHeader).Data.as_ptr().add(1) as *const u8,
                            (*copyProtectKey).KeyData.as_mut_ptr() as *mut u8,
                            (transferLength - 1) as usize,
                        );
                        *(*copyProtectKey).KeyData.as_mut_ptr().add((transferLength - 1) as usize) = 0;
                        (*copyProtectKey).KeyFlags = *(*keyHeader).Data.as_ptr() as u32;
                    } else {
                        core::ptr::copy(
                            (*keyHeader).Data.as_ptr() as *const u8,
                            (*copyProtectKey).KeyData.as_mut_ptr() as *mut u8,
                            transferLength as usize,
                        );
                    }

                    (*copyProtectKey).KeyLength = core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32;
                    (*copyProtectKey).KeyLength += transferLength;
                    *DataLength = (*copyProtectKey).KeyLength as usize;
                } else {
                    status = STATUS_SUCCESS;
                    *DataLength = 0;
                }
            } else {
                let sessionId: *mut DVD_SESSION_ID = OutputBuffer as *mut DVD_SESSION_ID;
                let keyHeader: *mut CDVD_KEY_HEADER = (*(*DeviceExtension).ScratchContext.ScratchSrb).DataBuffer as *mut CDVD_KEY_HEADER;
                let keyData: *mut CDVD_REPORT_AGID_DATA = (*keyHeader).Data.as_mut_ptr() as *mut CDVD_REPORT_AGID_DATA;

                *sessionId = (*keyData).AGID() as u32;
                *DataLength = core::mem::size_of::<DVD_SESSION_ID>();
            }
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleDvdSendKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: *mut core::ffi::c_void,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {

    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut input_buffer: *mut core::ffi::c_void = core::ptr::null_mut();

    PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request as WDFREQUEST,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        core::ptr::addr_of_mut!(input_buffer),
        core::ptr::null_mut(),
    );

    if status >= 0 {
        status = DvdSendKey(
            DeviceExtension,
            Request as WDFREQUEST,
            input_buffer,
            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
            DataLength,
        );
    }

    return status;
}

const SCSIOP_SEND_KEY: u8 = 0xA3;
const STATUS_INVALID_PARAMETER: i32 = (0xC000000Du32 as i32);
#[no_mangle]
pub unsafe extern "C" fn DvdSendKey(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    OriginalRequest: WDFREQUEST,
    InputBuffer: *mut core::ffi::c_void,
    InputBufferLength: usize,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0;
    let key: *mut DVD_COPY_PROTECT_KEY = InputBuffer as *mut DVD_COPY_PROTECT_KEY;
    let mut keyLength: u32 = 0;
    let mut result: u32 = 0;
    let mut four_byte: *mut FOUR_BYTE = core::ptr::null_mut();

    let _ = InputBufferLength;
    *DataLength = 0;

    if status >= 0 {
        if (*key).KeyLength < core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32
            || ((*key).KeyLength - core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32)
                > (*DeviceExtension).ScratchContext.ScratchBufferSize
        {
            NT_ASSERT(0);
            status = STATUS_INTERNAL_ERROR;
        }
    }

    if status >= 0 {
        status = RtlULongSub(
            (*key).KeyLength,
            core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32,
            &mut result,
        );
    }

    if status >= 0 {
        status = RtlULongAdd(
            result,
            core::mem::size_of::<CDVD_KEY_HEADER>() as u32,
            &mut keyLength,
        );
    }

    if status >= 0 {
        let scratch_size = (*DeviceExtension).ScratchContext.ScratchBufferSize;
        if keyLength > scratch_size {
            keyLength = scratch_size;
        }
        if keyLength < 2 {
            status = STATUS_INVALID_PARAMETER;
        }
    }

    if status >= 0 {
        let mut keyBuffer: *mut CDVD_KEY_HEADER = core::ptr::null_mut();
        let mut cdb: CDB = core::mem::zeroed();

        ScratchBuffer_BeginUse(DeviceExtension);

        keyBuffer =
            (*DeviceExtension).ScratchContext.ScratchBuffer as *mut CDVD_KEY_HEADER;

        keyLength -= 2;
        four_byte = &raw mut keyLength as *mut u32 as *mut FOUR_BYTE;
        (*keyBuffer).DataLength[0] = (*four_byte).__bindgen_anon_1.as_ref().Byte1;
        (*keyBuffer).DataLength[1] = (*four_byte).__bindgen_anon_1.as_ref().Byte0;
        keyLength += 2;

        let dst = (*keyBuffer).Data.as_mut_ptr();
        let src = (*key).KeyData.as_ptr();
        let len = (*key).KeyLength - core::mem::size_of::<DVD_COPY_PROTECT_KEY>() as u32;
        core::ptr::copy(src, dst, len as usize);

        cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
        cdb.SEND_KEY.as_mut().ParameterListLength[0] = (*four_byte).__bindgen_anon_1.as_ref().Byte1;
        cdb.SEND_KEY.as_mut().ParameterListLength[1] = (*four_byte).__bindgen_anon_1.as_ref().Byte0;
        cdb.SEND_KEY.as_mut().set_KeyFormat((*key).KeyType as u8);
        cdb.SEND_KEY.as_mut().set_AGID((*key).SessionId as u8);

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (OriginalRequest) as _,
            keyLength,
            0,
            &mut cdb,
            12,
        );

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleSetReadAhead(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    // PAGED_CODE macro
    PAGED_CODE();

    let mut status: i32 = STATUS_SUCCESS;
    let mut readAhead: *mut STORAGE_SET_READ_AHEAD = core::ptr::null_mut();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength as usize,
        &mut readAhead as *mut *mut STORAGE_SET_READ_AHEAD as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        let mut blockAddress: u32;

        ScratchBuffer_BeginUse(DeviceExtension);

        core::ptr::write_bytes(&mut cdb as *mut CDB as *mut u8, 0, core::mem::size_of::<CDB>());
        // Set up the CDB
        cdb.SET_READ_AHEAD.as_mut().OperationCode = SCSIOP_SET_READ_AHEAD as u8;

        blockAddress = ((*readAhead).TriggerAddress.QuadPart >> (*DeviceExtension).SectorShift) as u32;

        // Defensive coding. Prefix cannot recognize this usage.
        {
            let _ = blockAddress;
        }

        let fourByte: *mut FOUR_BYTE = core::ptr::addr_of_mut!(blockAddress) as *mut FOUR_BYTE;

        cdb.SET_READ_AHEAD.as_mut().TriggerLBA[0] = (*fourByte).__bindgen_anon_1.as_ref().Byte3;
        cdb.SET_READ_AHEAD.as_mut().TriggerLBA[1] = (*fourByte).__bindgen_anon_1.as_ref().Byte2;
        cdb.SET_READ_AHEAD.as_mut().TriggerLBA[2] = (*fourByte).__bindgen_anon_1.as_ref().Byte1;
        cdb.SET_READ_AHEAD.as_mut().TriggerLBA[3] = (*fourByte).__bindgen_anon_1.as_ref().Byte0;

        blockAddress = ((*readAhead).TargetAddress.QuadPart >> (*DeviceExtension).SectorShift) as u32;

        cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[0] = (*fourByte).__bindgen_anon_1.as_ref().Byte3;
        cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[1] = (*fourByte).__bindgen_anon_1.as_ref().Byte2;
        cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[2] = (*fourByte).__bindgen_anon_1.as_ref().Byte1;
        cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[3] = (*fourByte).__bindgen_anon_1.as_ref().Byte0;

        status = ScratchBuffer_ExecuteCdb(DeviceExtension, (Request) as _, transferSize, (0i32) as _, &mut cdb, 12);

        if status >= 0 {
            *DataLength = 0;
        }

        // nothing to do after the command finishes.
        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHandleSetSpeed(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: *mut core::ffi::c_void,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let cdData = core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData) as *mut CDROM_DATA;
    let mut inputBuffer: *mut CDROM_SET_SPEED = core::ptr::null_mut();

    PAGED_CODE();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request as WDFREQUEST,
        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
        core::ptr::addr_of_mut!(inputBuffer) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if status >= 0 {
        let mut transferSize: u32 = 0;
        let mut cdb: CDB = core::mem::zeroed();
        let requestType = (*inputBuffer).RequestType;

        ScratchBuffer_BeginUse(DeviceExtension);

        RtlZeroMemory(
            core::ptr::addr_of_mut!(cdb) as *mut core::ffi::c_void,
            core::mem::size_of::<CDB>(),
        );

        if requestType == CdromSetSpeed {
            let speed = inputBuffer;
            cdb.SET_CD_SPEED.as_mut().OperationCode = SCSIOP_SET_CD_SPEED;
            cdb.SET_CD_SPEED.as_mut().__bindgen_anon_1.__bindgen_anon_1.as_mut().set_RotationControl((*speed).RotationControl as u8);
            REVERSE_BYTES_SHORT(
                cdb.SET_CD_SPEED.as_mut().ReadSpeed.as_mut_ptr() as *mut core::ffi::c_void,
                core::ptr::addr_of!((*speed).ReadSpeed) as *const core::ffi::c_void,
            );
            REVERSE_BYTES_SHORT(
                cdb.SET_CD_SPEED.as_mut().WriteSpeed.as_mut_ptr() as *mut core::ffi::c_void,
                core::ptr::addr_of!((*speed).WriteSpeed) as *const core::ffi::c_void,
            );
        } else {
            let stream = inputBuffer as *mut CDROM_SET_STREAMING;
            let mut perfDescriptor: *mut PERFORMANCE_DESCRIPTOR;

            transferSize = core::mem::size_of::<PERFORMANCE_DESCRIPTOR>() as u32;

            perfDescriptor =
                (*DeviceExtension).ScratchContext.ScratchBuffer as *mut PERFORMANCE_DESCRIPTOR;
            RtlZeroMemory(perfDescriptor as *mut core::ffi::c_void, transferSize as usize);

            (*perfDescriptor).set_RandomAccess((*stream).RandomAccess);
            (*perfDescriptor).set_Exact((*stream).SetExact);
            (*perfDescriptor).set_RestoreDefaults((*stream).RestoreDefaults);
            (*perfDescriptor).set_WriteRotationControl((*stream).RotationControl as u8);

            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*perfDescriptor).StartLba) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*stream).StartLba) as *const core::ffi::c_void,
            );
            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*perfDescriptor).EndLba) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*stream).EndLba) as *const core::ffi::c_void,
            );
            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*perfDescriptor).ReadSize) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*stream).ReadSize) as *const core::ffi::c_void,
            );
            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*perfDescriptor).ReadTime) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*stream).ReadTime) as *const core::ffi::c_void,
            );
            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*perfDescriptor).WriteSize) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*stream).WriteSize) as *const core::ffi::c_void,
            );
            REVERSE_BYTES(
                core::ptr::addr_of_mut!((*perfDescriptor).WriteTime) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*stream).WriteTime) as *const core::ffi::c_void,
            );

            cdb.SET_STREAMING.as_mut().OperationCode = SCSIOP_SET_STREAMING;
            REVERSE_BYTES_SHORT(
                cdb.SET_STREAMING.as_mut().ParameterListLength.as_mut_ptr() as *mut core::ffi::c_void,
                &transferSize as *const u32 as *const core::ffi::c_void,
            );

            (*cdData).RestoreDefaults = if (*stream).Persistent != 0 { 0 } else { 1 };

            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"DeviceHandleSetSpeed: Restore default speed on media change set to %s\n\0"
                    .as_ptr() as *const i8,
                if (*cdData).RestoreDefaults != 0 {
                    b"true\0".as_ptr()
                } else {
                    b"false\0".as_ptr()
                } as *const i8,
            );
        }

        status = ScratchBuffer_ExecuteCdb(
            DeviceExtension,
            (Request as WDFREQUEST) as _,
            transferSize,
            0,
            &raw mut cdb,
            12,
        );

        if status >= 0 {
            *DataLength = 0;
        }

        ScratchBuffer_EndUse(DeviceExtension);
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*--
//
// Copyright (C) Microsoft Corporation. All rights reserved.
//
// Module Name:
//
//     ioctl.c
//
// Abstract:
//
//     Include all funtions for processing IOCTLs
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #include "stddef.h"
// #include "string.h"
//
// #include "ntddk.h"
// #include "ntddstor.h"
// #include "cdrom.h"
// #include "ioctl.h"
// #include "scratch.h"
// #include "mmc.h"
//
//
// #ifdef DEBUG_USE_WPP
// #include "ioctl.tmh"
// #endif
//
//
// #define FirstDriveLetter 'C'
// #define LastDriveLetter  'Z'
//
// #if DBG
//     LPCSTR READ_DVD_STRUCTURE_FORMAT_STRINGS[DvdMaxDescriptor+1] = {
//         "Physical",
//         "Copyright",
//         "DiskKey",
//         "BCA",
//         "Manufacturer",
//         "Unknown"
//     };
// #endif // DBG
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// GetConfigurationDataConversionTypeAllToTypeOne(
//         FEATURE_NUMBER       RequestedFeature,
//         PSCSI_REQUEST_BLOCK  Srb,
//        size_t *             DataLength
//     );
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// GetConfigurationDataSynthesize(
//         PVOID           InputBuffer,
//                                 ULONG           InputBufferSize,
//       PVOID           OutputBuffer,
//                                 size_t          OutputBufferSize,
//                                 FEATURE_NUMBER  StartingFeature,
//                                 ULONG           RequestType,
//                                size_t *        DataLength
//     );
//
// _IRQL_requires_max_(APC_LEVEL)
// PCDB
// RequestGetScsiPassThroughCdb(
//      PIRP Irp
//     );
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceIsPlayActive)
// #pragma alloc_text(PAGE, RequestHandleGetDvdRegion)
// #pragma alloc_text(PAGE, RequestHandleReadTOC)
// #pragma alloc_text(PAGE, RequestHandleReadTocEx)
// #pragma alloc_text(PAGE, RequestHandleGetConfiguration)
// #pragma alloc_text(PAGE, RequestHandleGetDriveGeometry)
// #pragma alloc_text(PAGE, RequestHandleDiskVerify)
// #pragma alloc_text(PAGE, RequestHandleCheckVerify)
// #pragma alloc_text(PAGE, RequestHandleFakePartitionInfo)
// #pragma alloc_text(PAGE, RequestHandleEjectionControl)
// #pragma alloc_text(PAGE, RequestHandleEnableStreaming)
// #pragma alloc_text(PAGE, RequestHandleSendOpcInformation)
// #pragma alloc_text(PAGE, RequestHandleGetPerformance)
// #pragma alloc_text(PAGE, RequestHandleMcnSyncFakeIoctl)
// #pragma alloc_text(PAGE, RequestHandleLoadEjectMedia)
// #pragma alloc_text(PAGE, RequestHandleReserveRelease)
// #pragma alloc_text(PAGE, RequestHandlePersistentReserve)
// #pragma alloc_text(PAGE, DeviceHandleRawRead)
// #pragma alloc_text(PAGE, DeviceHandlePlayAudioMsf)
// #pragma alloc_text(PAGE, DeviceHandleReadQChannel)
// #pragma alloc_text(PAGE, ReadQChannel)
// #pragma alloc_text(PAGE, DeviceHandlePauseAudio)
// #pragma alloc_text(PAGE, DeviceHandleResumeAudio)
// #pragma alloc_text(PAGE, DeviceHandleSeekAudioMsf)
// #pragma alloc_text(PAGE, DeviceHandleStopAudio)
// #pragma alloc_text(PAGE, DeviceHandleGetSetVolume)
// #pragma alloc_text(PAGE, DeviceHandleReadDvdStructure)
// #pragma alloc_text(PAGE, ReadDvdStructure)
// #pragma alloc_text(PAGE, DeviceHandleDvdEndSession)
// #pragma alloc_text(PAGE, DeviceHandleDvdStartSessionReadKey)
// #pragma alloc_text(PAGE, DvdStartSessionReadKey)
// #pragma alloc_text(PAGE, DeviceHandleDvdSendKey)
// #pragma alloc_text(PAGE, DvdSendKey)
// #pragma alloc_text(PAGE, DeviceHandleSetReadAhead)
// #pragma alloc_text(PAGE, DeviceHandleSetSpeed)
// #pragma alloc_text(PAGE, RequestHandleExclusiveAccessQueryLockState)
// #pragma alloc_text(PAGE, RequestHandleExclusiveAccessLockDevice)
// #pragma alloc_text(PAGE, RequestHandleExclusiveAccessUnlockDevice)
// #pragma alloc_text(PAGE, RequestHandleScsiPassThrough)
// #pragma alloc_text(PAGE, RequestGetScsiPassThroughCdb)
// #pragma alloc_text(PAGE, GetConfigurationDataConversionTypeAllToTypeOne)
// #pragma alloc_text(PAGE, GetConfigurationDataSynthesize)
//
// #endif
//
// NTSTATUS
// RequestHandleUnknownIoctl(
//      WDFDEVICE    Device,
//      WDFREQUEST   Request
//     )
// /*++
//
// Routine Description:
//
//     All unknown IOCTLs will be forward to lower level driver.
//
// Arguments:
//
//     Device - device object
//     Request - request to be handled
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_UNSUCCESSFUL;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//     PCDROM_REQUEST_CONTEXT      requestContext = RequestGetContext(Request);
//     BOOLEAN                     syncRequired = requestContext->SyncRequired;
//
//     ULONG                       sendOptionsFlags = 0;
//     BOOLEAN                     requestSent = FALSE;
//
//     WdfRequestFormatRequestUsingCurrentType(Request);
//
//     if (syncRequired)
//     {
//         sendOptionsFlags = WDF_REQUEST_SEND_OPTION_SYNCHRONOUS;
//     }
//     else
//     {
//         WdfRequestSetCompletionRoutine(Request, RequestDummyCompletionRoutine, NULL);
//     }
//
//     status = RequestSend(deviceExtension,
//                          Request,
//                          deviceExtension->IoTarget,
//                          sendOptionsFlags,
//                          &requestSent);
//
//     if (requestSent)
//     {
//         if (syncRequired)
//         {
//             // the request needs to be completed here.
//             RequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request));
//         }
//     }
//     else
//     {
//         // failed to send the request to IoTarget
//         RequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request));
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// BOOLEAN
// DeviceIsPlayActive(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//     This routine determines if the cd is currently playing music.
//
// Arguments:
//
//     Device - Device object.
//
// Return Value:
//
//     BOOLEAN - TRUE if the device is playing music.
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PSUB_Q_CURRENT_POSITION currentBuffer;
//     size_t                  bytesRead = 0;
//
//     PAGED_CODE ();
//
//     // if we don't think it is playing audio, don't bother checking.
//     if (!deviceExtension->DeviceAdditionalData.PlayActive)
//     {
//         return FALSE;
//     }
//
//     // Allocate the required memory
//     NT_ASSERT(sizeof(SUB_Q_CURRENT_POSITION) >= sizeof(CDROM_SUB_Q_DATA_FORMAT));
//     currentBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                           sizeof(SUB_Q_CURRENT_POSITION),
//                                           CDROM_TAG_PLAY_ACTIVE);
//     if (currentBuffer == NULL)
//     {
//         return FALSE;
//     }
//
//     // set the options in the output buffer format
//     ((PCDROM_SUB_Q_DATA_FORMAT) currentBuffer)->Format = IOCTL_CDROM_CURRENT_POSITION;
//     ((PCDROM_SUB_Q_DATA_FORMAT) currentBuffer)->Track = 0;
//
//     // Send SCSI command to read Q Channel information.
//     status = ReadQChannel(deviceExtension,
//                           NULL,
//                           currentBuffer,
//                           sizeof(CDROM_SUB_Q_DATA_FORMAT),
//                           currentBuffer,
//                           sizeof(SUB_Q_CURRENT_POSITION),
//                           &bytesRead);
//
//     if ((!NT_SUCCESS(status)))
//     {
//         ExFreePool(currentBuffer);
//         return FALSE;
//     }
//
//     // update the playactive flag appropriately
//     if (currentBuffer->Header.AudioStatus == AUDIO_STATUS_IN_PROGRESS)
//     {
//         deviceExtension->DeviceAdditionalData.PlayActive = TRUE;
//     }
//     else
//     {
//         deviceExtension->DeviceAdditionalData.PlayActive = FALSE;
//     }
//
//     ExFreePool(currentBuffer);
//
//     return deviceExtension->DeviceAdditionalData.PlayActive;
// }
//
// NTSTATUS
// RequestHandleGetInquiryData(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength)
// /*++
//
// Routine Description:
//
//    Handler for IOCTL_CDROM_GET_INQUIRY_DATA
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     PCDROM_DATA cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength == 0)
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else
//     {
//         PVOID outputBuffer = NULL;
//
//         *DataLength = min(cdData->CachedInquiryDataByteCount,
//                           RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength);
//
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//
//         if (NT_SUCCESS(status) &&
//             (outputBuffer != NULL))
//         {
//             // Always copy as much data as possible
//             RtlCopyMemory(outputBuffer,
//                           cdData->CachedInquiryData,
//                           *DataLength);
//         }
//
//         // and finally decide between two possible status values
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < cdData->CachedInquiryDataByteCount)
//         {
//             status = STATUS_BUFFER_OVERFLOW;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestHandleGetMediaTypeEx(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handler for IOCTL_STORAGE_GET_MEDIA_TYPES_EX
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_DATA         cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     PGET_MEDIA_TYPES        mediaTypes = NULL;
//     PDEVICE_MEDIA_INFO      mediaInfo = NULL; //&mediaTypes->MediaInfo[0];
//     ULONG                   sizeNeeded = 0;
//     PZERO_POWER_ODD_INFO    zpoddInfo = DeviceExtension->ZeroPowerODDInfo;
//
//     *DataLength = 0;
//
//     // Must run below dispatch level.
//     if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//     {
//         NT_ASSERT(FALSE);
//         return STATUS_INVALID_LEVEL;
//     }
//
//     sizeNeeded = sizeof(GET_MEDIA_TYPES);
//
//     // IsMmc is static...
//     if (cdData->Mmc.IsMmc)
//     {
//         sizeNeeded += sizeof(DEVICE_MEDIA_INFO) * 1; // return two media types
//     }
//
//     status = WdfRequestRetrieveOutputBuffer(Request,
//                                             sizeNeeded,
//                                             (PVOID*)&mediaTypes,
//                                             NULL);
//
//     if (NT_SUCCESS(status) &&
//         (mediaTypes != NULL))
//     {
//         mediaInfo = &mediaTypes->MediaInfo[0];
//
//         RtlZeroMemory(mediaTypes, sizeNeeded);
//
//         // ISSUE-2000/5/11-henrygab - need to update GET_MEDIA_TYPES_EX
//
//         mediaTypes->DeviceType = cdData->DriveDeviceType;
//
//         mediaTypes->MediaInfoCount = 1;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaType = CD_ROM;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.NumberMediaSides = 1;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics = MEDIA_READ_ONLY;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.Cylinders.QuadPart = DeviceExtension->DiskGeometry.Cylinders.QuadPart;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.TracksPerCylinder = DeviceExtension->DiskGeometry.TracksPerCylinder;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.SectorsPerTrack = DeviceExtension->DiskGeometry.SectorsPerTrack;
//         mediaInfo->DeviceSpecific.RemovableDiskInfo.BytesPerSector = DeviceExtension->DiskGeometry.BytesPerSector;
//
//         if (cdData->Mmc.IsMmc)
//         {
//             // also report a removable disk
//             mediaTypes->MediaInfoCount += 1;
//
//             mediaInfo++;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaType = (STORAGE_MEDIA_TYPE)RemovableMedia;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.NumberMediaSides = 1;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics = MEDIA_READ_WRITE;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.Cylinders.QuadPart = DeviceExtension->DiskGeometry.Cylinders.QuadPart;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.TracksPerCylinder = DeviceExtension->DiskGeometry.TracksPerCylinder;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.SectorsPerTrack = DeviceExtension->DiskGeometry.SectorsPerTrack;
//             mediaInfo->DeviceSpecific.RemovableDiskInfo.BytesPerSector = DeviceExtension->DiskGeometry.BytesPerSector;
//             mediaInfo--;
//
//         }
//
//         // Status will either be success, if media is present, or no media.
//         // It would be optimal to base from density code and medium type, but not all devices
//         // have values for these fields.
//
//         // Send a TUR to determine if media is present, only if the device is not in ZPODD mode.
//         if ((!EXCLUSIVE_MODE(cdData) ||
//              EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request))) &&
//             ((zpoddInfo == NULL) ||
//              (zpoddInfo->InZeroPowerState == FALSE)))
//         {
//             SCSI_REQUEST_BLOCK  srb;
//             PCDB                cdb = (PCDB)srb.Cdb;
//
//             RtlZeroMemory(&srb,sizeof(SCSI_REQUEST_BLOCK));
//
//             srb.CdbLength = 6;
//             cdb->CDB6GENERIC.OperationCode = SCSIOP_TEST_UNIT_READY;
//
//             srb.TimeOutValue = CDROM_TEST_UNIT_READY_TIMEOUT;
//
//             status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                                 &srb,
//                                                 NULL,
//                                                 0,
//                                                 FALSE,
//                                                 Request);
//
//             if (NT_SUCCESS(status))
//             {
//                 // set the disk's media as current if we can write to it.
//                 if (cdData->Mmc.IsMmc && cdData->Mmc.WriteAllowed)
//                 {
//                     mediaInfo++;
//                     SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics,
//                              MEDIA_CURRENTLY_MOUNTED);
//                     mediaInfo--;
//                 }
//                 else
//                 {
//                     SET_FLAG(mediaInfo->DeviceSpecific.RemovableDiskInfo.MediaCharacteristics,
//                              MEDIA_CURRENTLY_MOUNTED);
//                 }
//             }
//             else
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                            "RequestHandleGetMediaTypeEx: GET_MEDIA_TYPES status of TUR - %lx\n", status));
//             }
//         }
//
//         // per legacy cdrom behavior, always return success
//         status = STATUS_SUCCESS;
//     }
//
//     *DataLength = sizeNeeded;
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleGetDvdRegion(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handler for IOCTL_DVD_GET_REGION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//
//     PVOID               outputBuffer = NULL;
//     size_t              bytesReturned = 0;
//
//     PDVD_COPY_PROTECT_KEY       copyProtectKey = NULL;
//     ULONG                       keyLength = 0;
//     PDVD_DESCRIPTOR_HEADER      dvdHeader;
//     PDVD_COPYRIGHT_DESCRIPTOR   copyRightDescriptor;
//     PDVD_REGION                 dvdRegion = NULL;
//     PDVD_READ_STRUCTURE         readStructure = NULL;
//     PDVD_RPC_KEY                rpcKey;
//
//     PAGED_CODE ();
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "RequestHandleGetDvdRegion: [%p] IOCTL_DVD_GET_REGION\n", Request));
//
//     *DataLength = 0;
//
//     // reject the request if it's not a DVD device.
//     if (DeviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 sizeof(DVD_REGION),
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // figure out how much data buffer we need
//         keyLength = max((sizeof(DVD_DESCRIPTOR_HEADER) + sizeof(DVD_COPYRIGHT_DESCRIPTOR)),
//                         sizeof(DVD_READ_STRUCTURE));
//         keyLength = max(keyLength,
//                         DVD_RPC_KEY_LENGTH);
//
//         // round the size to nearest ULONGLONG -- why?
//         // could this be to deal with device alignment issues?
//         keyLength += sizeof(ULONGLONG) - (keyLength & (sizeof(ULONGLONG) - 1));
//
//         readStructure = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                               keyLength,
//                                               DVD_TAG_READ_KEY);
//         if (readStructure == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory (readStructure, keyLength);
//         readStructure->Format = DvdCopyrightDescriptor;
//
//         // use READ_STRUCTURE to read copyright descriptor
//         status = ReadDvdStructure(DeviceExtension,
//                                   Request,
//                                   readStructure,
//                                   keyLength,
//                                   readStructure,
//                                   sizeof(DVD_DESCRIPTOR_HEADER) + sizeof(DVD_COPYRIGHT_DESCRIPTOR),
//                                   &bytesReturned);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // we got the copyright descriptor, so now get the region if possible
//         dvdHeader = (PDVD_DESCRIPTOR_HEADER) readStructure;
//         copyRightDescriptor = (PDVD_COPYRIGHT_DESCRIPTOR) dvdHeader->Data;
//
//         // the original irp's systembuffer has a copy of the info that
//         // should be passed down in the request
//         dvdRegion = outputBuffer;
//
//         dvdRegion->CopySystem = copyRightDescriptor->CopyrightProtectionType;
//         dvdRegion->RegionData = copyRightDescriptor->RegionManagementInformation;
//
//         // now reuse the buffer to request the copy protection info
//         copyProtectKey = (PDVD_COPY_PROTECT_KEY) readStructure;
//         RtlZeroMemory (copyProtectKey, DVD_RPC_KEY_LENGTH);
//         copyProtectKey->KeyLength = DVD_RPC_KEY_LENGTH;
//         copyProtectKey->KeyType = DvdGetRpcKey;
//
//         // send a request for READ_KEY
//         status = DvdStartSessionReadKey(DeviceExtension,
//                                         IOCTL_DVD_READ_KEY,
//                                         Request,
//                                         copyProtectKey,
//                                         DVD_RPC_KEY_LENGTH,
//                                         copyProtectKey,
//                                         DVD_RPC_KEY_LENGTH,
//                                         &bytesReturned);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // the request succeeded.  if a supported scheme is returned,
//         // then return the information to the caller
//         rpcKey = (PDVD_RPC_KEY) copyProtectKey->KeyData;
//
//         if (rpcKey->RpcScheme == 1)
//         {
//             if (rpcKey->TypeCode)
//             {
//                 dvdRegion->SystemRegion = ~rpcKey->RegionMask;
//                 dvdRegion->ResetCount = rpcKey->UserResetsAvailable;
//             }
//             else
//             {
//                 // the drive has not been set for any region
//                 dvdRegion->SystemRegion = 0;
//                 dvdRegion->ResetCount = rpcKey->UserResetsAvailable;
//             }
//
//             *DataLength = sizeof(DVD_REGION);
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestHandleGetDvdRegion => rpcKey->RpcScheme != 1\n"));
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     // Clean up
//     if (readStructure != NULL)
//     {
//         ExFreePool(readStructure);
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateRawRead(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_RAW_READ
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_DATA         cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     PVOID               inputBuffer = NULL;
//     PIRP                irp = NULL;
//     PIO_STACK_LOCATION  currentStack = NULL;
//
//     LARGE_INTEGER       startingOffset = {0};
//     ULONGLONG           transferBytes = 0;
//     ULONGLONG           endOffset;
//     ULONGLONG           mdlBytes;
//     RAW_READ_INFO       rawReadInfo = {0};
//
//     *DataLength = 0;
//
//     irp = WdfRequestWdmGetIrp(Request);
//     currentStack = IoGetCurrentIrpStackLocation(irp);
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &inputBuffer,
//                                            NULL);
//
//     // Check that ending sector is on disc and buffers are there and of
//     // correct size.
//     if (NT_SUCCESS(status) &&
//         (RequestParameters.Parameters.DeviceIoControl.as_ref().Type3InputBuffer == NULL))
//     {
//         //  This is a call from user space.  This is the only time that we need to validate parameters.
//         //  Validate the input and get the input buffer into Type3InputBuffer
//         //  so the rest of the code will be uniform.
//         if (inputBuffer != NULL)
//         {
//             currentStack->Parameters.DeviceIoControl.as_mut().Type3InputBuffer = inputBuffer;
//             RequestParameters.Parameters.DeviceIoControl.as_mut().Type3InputBuffer = inputBuffer;
//
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < sizeof(RAW_READ_INFO))
//             {
//                 *DataLength = sizeof(RAW_READ_INFO);
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//         }
//         else
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         //  Since this ioctl is METHOD_OUT_DIRECT, we need to copy away the input buffer before interpreting it.
//         //  This prevents a malicious app from messing with the input buffer while we are interpreting it.
//         rawReadInfo = *(PRAW_READ_INFO)RequestParameters.Parameters.DeviceIoControl.as_ref().Type3InputBuffer;
//
//         startingOffset.QuadPart = rawReadInfo.DiskOffset.QuadPart;
//
//         if ((rawReadInfo.TrackMode == CDDA)        ||
//             (rawReadInfo.TrackMode == YellowMode2) ||
//             (rawReadInfo.TrackMode == XAForm2)     )
//         {
//             transferBytes = (ULONGLONG)rawReadInfo.SectorCount * RAW_SECTOR_SIZE;
//         }
//         else if (rawReadInfo.TrackMode == RawWithSubCode)
//         {
//             transferBytes = (ULONGLONG)rawReadInfo.SectorCount * CD_RAW_SECTOR_WITH_SUBCODE_SIZE;
//         }
//         else if (rawReadInfo.TrackMode == RawWithC2)
//         {
//             transferBytes = (ULONGLONG)rawReadInfo.SectorCount * CD_RAW_SECTOR_WITH_C2_SIZE;
//         }
//         else if (rawReadInfo.TrackMode == RawWithC2AndSubCode)
//         {
//             transferBytes = (ULONGLONG)rawReadInfo.SectorCount * CD_RAW_SECTOR_WITH_C2_AND_SUBCODE_SIZE;
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                       "RequestValidateRawRead: Invalid TrackMode type %x for XA read\n",
//                       rawReadInfo.TrackMode
//                       ));
//         }
//
//         endOffset = (ULONGLONG)rawReadInfo.SectorCount * COOKED_SECTOR_SIZE;
//         endOffset += startingOffset.QuadPart;
//
//         // check for overflows....
//         if (rawReadInfo.SectorCount == 0)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestValidateRawRead: Invalid I/O parameters for XA "
//                         "Read (zero sectors requested)\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (transferBytes < (ULONGLONG)(rawReadInfo.SectorCount))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestValidateRawRead: Invalid I/O parameters for XA "
//                         "Read (TransferBytes Overflow)\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (endOffset < (ULONGLONG)startingOffset.QuadPart)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestValidateRawRead: Invalid I/O parameters for XA "
//                         "Read (EndingOffset Overflow)\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < transferBytes)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestValidateRawRead: Invalid I/O parameters for XA "
//                         "Read (Bad buffer size)\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (endOffset > (ULONGLONG)DeviceExtension->PartitionLength.QuadPart)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestValidateRawRead: Invalid I/O parameters for XA "
//                         "Read (Request Out of Bounds)\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // cannot validate the MdlAddress, since it is not included in any
//         // other location per the DDK and file system calls.
//
//         // validate the mdl describes at least the number of bytes
//         // requested from us.
//         mdlBytes = (ULONGLONG)MmGetMdlByteCount(irp->MdlAddress);
//         if (mdlBytes < transferBytes)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestValidateRawRead: Invalid MDL %s, Irp %p\n",
//                         "size (5)", irp));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // check the buffer for alignment
//         // This is important for x86 as some busses (ie ATAPI)
//         // require word-aligned buffers.
//         if ( ((ULONG_PTR)MmGetMdlVirtualAddress(irp->MdlAddress)) &
//              DeviceExtension->AdapterDescriptor->AlignmentMask )
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                       "RequestValidateRawRead: Invalid I/O parameters for "
//                       "XA Read (Buffer %p not aligned with mask %x\n",
//                       RequestParameters.Parameters.DeviceIoControl.as_ref().Type3InputBuffer,
//                       DeviceExtension->AdapterDescriptor->AlignmentMask));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Validate the request is not too large for the adapter
//         BOOLEAN bufferIsPageAligned = FALSE;
//         ULONG   maxLength = 0;
//
//         // if buffer is not page-aligned, then subtract one as the
//         // transfer could cross a page boundary.
//         if ((((ULONG_PTR)MmGetMdlVirtualAddress(irp->MdlAddress)) & (PAGE_SIZE-1)) == 0)
//         {
//             bufferIsPageAligned = TRUE;
//         }
//
//         if (bufferIsPageAligned)
//         {
//             maxLength = cdData->MaxPageAlignedTransferBytes;
//         }
//         else
//         {
//             maxLength = cdData->MaxUnalignedTransferBytes;
//         }
//
//         if (transferBytes > maxLength)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                       "RequestValidateRawRead: The XA Read (type %x) would require %I64x bytes, "
//                       "but the adapter can only handle %x bytes (for a%saligned buffer)\n",
//                       rawReadInfo.TrackMode,
//                       transferBytes,
//                       maxLength,
//                       (bufferIsPageAligned ? " " : "n un")
//                       ));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         //
//         // HACKHACK - REF #0001
//         // The retry count will be in this irp's IRP_MN function,
//         // as the new irp was freed, and we therefore cannot use
//         // this irp's next stack location for this function.
//         // This may be a good location to store this info for
//         // when we remove RAW_READ (mode switching), as we will
//         // no longer have the nextIrpStackLocation to play with
//         // when that occurs
//         //
//         currentStack->MinorFunction = MAXIMUM_RETRIES; // HACKHACK - REF #0001
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateReadTocEx(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_READ_TOC_EX
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//
//     PCDROM_READ_TOC_EX  inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//         sizeof(CDROM_READ_TOC_EX))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//              MINIMUM_CDROM_READ_TOC_EX_SIZE)
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = MINIMUM_CDROM_READ_TOC_EX_SIZE;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >
//              ((USHORT)-1))
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength &
//              DeviceExtension->AdapterDescriptor->AlignmentMask)
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if ((inputBuffer->Reserved1 != 0) ||
//             (inputBuffer->Reserved2 != 0) ||
//             (inputBuffer->Reserved3 != 0))
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         // NOTE: when adding new formats, ensure that first two bytes
//         //       specify the amount of additional data available.
//         else if ((inputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_TOC     ) ||
//                  (inputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_FULL_TOC) ||
//                  (inputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_CDTEXT  ))
//         {
//             // SessionTrack field is used
//         }
//         else if ((inputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_SESSION) ||
//                  (inputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_PMA)     ||
//                  (inputBuffer->Format == CDROM_READ_TOC_EX_FORMAT_ATIP))
//         {
//             // SessionTrack field is reserved
//             if (inputBuffer->SessionTrack != 0)
//             {
//                 status = STATUS_INVALID_PARAMETER;
//             }
//         }
//         else
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateReadToc(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_READ_TOC
//
// Arguments:
//
//     DeviceExtension - device context
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//         sizeof(CDROM_TOC))
//     {
//         // they didn't request the entire TOC -- use _EX version
//         // for partial transfers and such.
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(CDROM_TOC);
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength &
//              DeviceExtension->AdapterDescriptor->AlignmentMask)
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateGetLastSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_GET_LAST_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//         sizeof(CDROM_TOC_SESSION_DATA))
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(CDROM_TOC_SESSION_DATA);
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength &
//              DeviceExtension->AdapterDescriptor->AlignmentMask)
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateReadQChannel(
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_READ_Q_CHANNEL
//
// Arguments:
//
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PCDROM_SUB_Q_DATA_FORMAT inputBuffer = NULL;
//     ULONG                    transferByteCount = 0;
//
//     *DataLength = 0;
//
//     if(RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//        sizeof(CDROM_SUB_Q_DATA_FORMAT))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // check for all valid types of request
//         if (inputBuffer->Format == IOCTL_CDROM_CURRENT_POSITION)
//         {
//             transferByteCount = sizeof(SUB_Q_CURRENT_POSITION);
//         }
//         else if (inputBuffer->Format == IOCTL_CDROM_MEDIA_CATALOG)
//         {
//             transferByteCount = sizeof(SUB_Q_MEDIA_CATALOG_NUMBER);
//         }
//         else if (inputBuffer->Format == IOCTL_CDROM_TRACK_ISRC)
//         {
//             transferByteCount = sizeof(SUB_Q_TRACK_ISRC);
//         }
//         else
//         {
//             // Format not valid
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//             transferByteCount)
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             *DataLength = transferByteCount;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateDvdReadStructure(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_DVD_READ_STRUCTURE
//
// Arguments:
//
//     DeviceExtension - device context
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         if(RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//            sizeof(DVD_READ_STRUCTURE))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestValidateDvdReadStructure - input buffer "
//                         "length too small (was %d should be %d)\n",
//                         (int)RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                         sizeof(DVD_READ_STRUCTURE)));
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//                 sizeof(READ_DVD_STRUCTURES_HEADER))
//         {
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestValidateDvdReadStructure - output buffer "
//                         "cannot hold header information\n"));
//             status = STATUS_BUFFER_TOO_SMALL;
//             *DataLength = sizeof(READ_DVD_STRUCTURES_HEADER);
//         }
//         else if(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >
//                 MAXUSHORT)
//         {
//             // key length must fit in two bytes
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestValidateDvdReadStructure - output buffer "
//                         "too large\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength &
//                  DeviceExtension->AdapterDescriptor->AlignmentMask)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (DeviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//         {
//             // reject the request if it's not a DVD device.
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateDvdStartSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Validate request of IOCTL_DVD_START_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         if(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//            sizeof(DVD_SESSION_ID))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestValidateDvdStartSession: DVD_START_SESSION - output "
//                         "buffer too small\n"));
//             status = STATUS_BUFFER_TOO_SMALL;
//             *DataLength = sizeof(DVD_SESSION_ID);
//         }
//         else if (DeviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//         {
//             // reject the request if it's not a DVD device.
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateDvdSendKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_DVD_SEND_KEY, IOCTL_DVD_SEND_KEY2
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PDVD_COPY_PROTECT_KEY   key = NULL;
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &key,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         if((RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < sizeof(DVD_COPY_PROTECT_KEY)) ||
//            (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != key->KeyLength))
//         {
//
//             //
//             // Key is too small to have a header or the key length doesn't
//             // match the input buffer length.  Key must be invalid
//             //
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestValidateDvdSendKey: [%p] IOCTL_DVD_SEND_KEY - "
//                         "key is too small or does not match KeyLength\n",
//                         Request));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // allow only certain key type (non-destructive) to go through
//         // IOCTL_DVD_SEND_KEY (which only requires READ access to the device)
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_DVD_SEND_KEY)
//         {
//             if ((key->KeyType != DvdChallengeKey) &&
//                 (key->KeyType != DvdBusKey2) &&
//                 (key->KeyType != DvdInvalidateAGID))
//             {
//                 status = STATUS_INVALID_PARAMETER;
//             }
//         }
//         else if ((key->KeyType != DvdChallengeKey) &&
//                 (key->KeyType != DvdBusKey1) &&
//                 (key->KeyType != DvdBusKey2) &&
//                 (key->KeyType != DvdTitleKey) &&
//                 (key->KeyType != DvdAsf) &&
//                 (key->KeyType != DvdSetRpcKey) &&
//                 (key->KeyType != DvdGetRpcKey) &&
//                 (key->KeyType != DvdDiskKey) &&
//                 (key->KeyType != DvdInvalidateAGID))
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (DeviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//         {
//             // reject the request if it's not a DVD device.
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateGetConfiguration(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_GET_CONFIGURATION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//         sizeof(GET_CONFIGURATION_HEADER))
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(GET_CONFIGURATION_HEADER);
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > 0xffff)
//     {
//         // output buffer is too large
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength &
//         DeviceExtension->AdapterDescriptor->AlignmentMask)
//     {
//         // buffer is not proper size multiple
//         status = STATUS_INVALID_PARAMETER;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//
// #if BUILD_WOW64_ENABLED && defined(_WIN64)
//
//         if (WdfRequestIsFrom32BitProcess(Request))
//         {
//             PGET_CONFIGURATION_IOCTL_INPUT32 inputBuffer = NULL;
//
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//                 sizeof(GET_CONFIGURATION_IOCTL_INPUT32))
//             {
//                 status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//
//             //
//             // also verify the arguments are reasonable.
//             //
//             if (NT_SUCCESS(status))
//             {
//                 status = WdfRequestRetrieveInputBuffer(Request,
//                                                        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                        &inputBuffer,
//                                                        NULL);
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 if (inputBuffer->Feature > 0xffff)
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//                 else if ((inputBuffer->RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
//                          (inputBuffer->RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT) &&
//                          (inputBuffer->RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL))
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//                 else if (inputBuffer->Reserved[0] || inputBuffer->Reserved[1])
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//         }
//         else
//
// #endif
//
//         {
//             PGET_CONFIGURATION_IOCTL_INPUT inputBuffer = NULL;
//
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//                 sizeof(GET_CONFIGURATION_IOCTL_INPUT))
//             {
//                 status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//
//             // also verify the arguments are reasonable.
//             if (NT_SUCCESS(status))
//             {
//                 status = WdfRequestRetrieveInputBuffer(Request,
//                                                        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                        &inputBuffer,
//                                                        NULL);
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 if (inputBuffer->Feature > 0xffff)
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//                 else if ((inputBuffer->RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
//                          (inputBuffer->RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT) &&
//                          (inputBuffer->RequestType != SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL))
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//                 else if (inputBuffer->Reserved[0] || inputBuffer->Reserved[1])
//                 {
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateSetSpeed(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_SET_SPEED
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PCDROM_SET_SPEED        inputBuffer = NULL;
//     ULONG                   requiredLength = 0;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < sizeof(CDROM_SET_SPEED))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Get the request type using CDROM_SET_SPEED structure
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                sizeof(CDROM_SET_SPEED),
//                                                &inputBuffer,
//                                                NULL);
//
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->RequestType > CdromSetStreaming)
//         {
//             // Unknown request type.
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (inputBuffer->RequestType == CdromSetSpeed)
//         {
//             requiredLength = sizeof(CDROM_SET_SPEED);
//         }
//         else
//         {
//             // Don't send SET STREAMING command if this is not a MMC compliant device
//             if (cdData->Mmc.IsMmc == FALSE)
//             {
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//
//             requiredLength = sizeof(CDROM_SET_STREAMING);
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < requiredLength)
//         {
//             // Input buffer too small
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsReadMediaKeyBlock(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_READ_MEDIA_KEY_BLOCK
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PAACS_LAYER_NUMBER      layerNumber = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(AACS_LAYER_NUMBER))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < 8)
//     {
//         // This is a variable-length structure, but we're pretty sure
//         // it can never be less than eight bytes...
//         *DataLength = 8;
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &layerNumber,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (*layerNumber > 255)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsStartSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_START_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(DVD_SESSION_ID))
//     {
//         *DataLength = sizeof(DVD_SESSION_ID);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(DVD_SESSION_ID))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsSendCertificate(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_SEND_CERTIFICATE
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PAACS_SEND_CERTIFICATE  inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(AACS_SEND_CERTIFICATE))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->SessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsGetCertificate(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_GET_CERTIFICATE
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PDVD_SESSION_ID         sessionId = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(DVD_SESSION_ID))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(AACS_CERTIFICATE))
//     {
//         *DataLength = sizeof(AACS_CERTIFICATE);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(AACS_CERTIFICATE))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &sessionId,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (*sessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsGetChallengeKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_GET_CHALLENGE_KEY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PDVD_SESSION_ID         sessionId = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(DVD_SESSION_ID))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(AACS_CHALLENGE_KEY))
//     {
//         *DataLength = sizeof(AACS_CHALLENGE_KEY);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(AACS_CHALLENGE_KEY))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &sessionId,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (*sessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsSendChallengeKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_SEND_CHALLENGE_KEY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_DATA                 cdData = &(DeviceExtension->DeviceAdditionalData);
//     PAACS_SEND_CHALLENGE_KEY    inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(AACS_SEND_CHALLENGE_KEY))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->SessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsReadVolumeId(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_READ_VOLUME_ID
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PDVD_SESSION_ID         sessionId = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(DVD_SESSION_ID))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(AACS_VOLUME_ID))
//     {
//         *DataLength = sizeof(AACS_VOLUME_ID);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(AACS_VOLUME_ID))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &sessionId,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (*sessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsReadSerialNumber(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_READ_SERIAL_NUMBER
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PDVD_SESSION_ID         sessionId = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(DVD_SESSION_ID))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(AACS_SERIAL_NUMBER))
//     {
//         *DataLength = sizeof(AACS_SERIAL_NUMBER);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(AACS_SERIAL_NUMBER))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &sessionId,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (*sessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsReadMediaId(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_READ_MEDIA_ID
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DATA             cdData = &(DeviceExtension->DeviceAdditionalData);
//     PDVD_SESSION_ID         sessionId = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(DVD_SESSION_ID))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(AACS_MEDIA_ID))
//     {
//         *DataLength = sizeof(AACS_MEDIA_ID);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(AACS_MEDIA_ID))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &sessionId,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (*sessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateAacsBindingNonce(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_READ_BINDING_NONCE
//                        IOCTL_AACS_GENERATE_BINDING_NONCE
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                 status = STATUS_SUCCESS;
//     PCDROM_DATA              cdData = &(DeviceExtension->DeviceAdditionalData);
//     PAACS_READ_BINDING_NONCE inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.IsAACS)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(AACS_READ_BINDING_NONCE))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(AACS_BINDING_NONCE))
//     {
//         *DataLength = sizeof(AACS_BINDING_NONCE);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength > sizeof(AACS_BINDING_NONCE))
//     {
//         status = STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->SessionId > MAX_COPY_PROTECT_AGID)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (inputBuffer->NumberOfSectors > 255)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (inputBuffer->StartLba > MAXULONG)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateExclusiveAccess(
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_CDROM_EXCLUSIVE_ACCESS
//
// Arguments:
//
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_EXCLUSIVE_ACCESS exclusiveAccess = NULL;
//
//     *DataLength = 0;
//
//     if (KeGetCurrentIrql() != PASSIVE_LEVEL)
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "RequestValidateExclusiveAccess: IOCTL must be called at passive level.\n"));
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < sizeof(CDROM_EXCLUSIVE_ACCESS))
//     {
//
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "RequestValidateExclusiveAccess: Input buffer too small\n"));
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &exclusiveAccess,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         switch (exclusiveAccess->RequestType)
//         {
//             case ExclusiveAccessQueryState:
//             {
//                 if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//                     sizeof(CDROM_EXCLUSIVE_LOCK_STATE))
//                 {
//                     //
//                     // Output buffer too small.
//                     //
//                     TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "RequestValidateExclusiveAccess: Output buffer too small\n"));
//                     *DataLength = sizeof(CDROM_EXCLUSIVE_LOCK_STATE);
//                     status = STATUS_BUFFER_TOO_SMALL;
//                 }
//                 break;
//             }
//
//             case ExclusiveAccessLockDevice:
//             {
//                 if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//                     sizeof(CDROM_EXCLUSIVE_LOCK))
//                 {
//                     //
//                     // Input buffer too small
//                     //
//                     TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "RequestValidateExclusiveAccess: Input buffer too small\n"));
//                     status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 break;
//             }
//             case ExclusiveAccessUnlockDevice:
//             {
//                 //
//                 // Nothing to check
//                 //
//                 break;
//             }
//
//             default:
//             {
//                 //
//                 // Unknown request type.
//                 //
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "RequestValidateExclusiveAccess: Invalid request type\n"));
//                 status = STATUS_INVALID_PARAMETER;
//             }
//         }
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleExclusiveAccessQueryLockState(
//      WDFDEVICE Device,
//      WDFREQUEST Request
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_EXCLUSIVE_ACCESS with ExclusiveAccessQueryState
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA                 cdData = &deviceExtension->DeviceAdditionalData;
//     PCDROM_EXCLUSIVE_LOCK_STATE exclusiveLockState = NULL;
//
//     PAGED_CODE();
//
//     status = WdfRequestRetrieveOutputBuffer(Request,
//                                             sizeof(CDROM_EXCLUSIVE_LOCK_STATE),
//                                             &exclusiveLockState,
//                                             NULL);
//     NT_ASSERT(NT_SUCCESS(status));
//
//     RtlZeroMemory(exclusiveLockState, sizeof(CDROM_EXCLUSIVE_LOCK_STATE));
//
//     if (EXCLUSIVE_MODE(cdData))
//     {
//         // Device is locked for exclusive use
//         exclusiveLockState->LockState = TRUE;
//
//         RtlCopyMemory(&exclusiveLockState->CallerName,
//                       &cdData->CallerName,
//                       CDROM_EXCLUSIVE_CALLER_LENGTH);
//
//     }
//     else
//     {
//         // Device is not locked
//         exclusiveLockState->LockState = FALSE;
//     }
//
//     RequestCompletion(deviceExtension, Request, status, sizeof(CDROM_EXCLUSIVE_LOCK_STATE));
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleExclusiveAccessLockDevice(
//      WDFDEVICE Device,
//      WDFREQUEST Request
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_EXCLUSIVE_ACCESS with ExclusiveAccessLockDevice
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_DATA             cdData = &deviceExtension->DeviceAdditionalData;
//     PCDROM_EXCLUSIVE_LOCK   exclusiveLock = NULL;
//     PIO_ERROR_LOG_PACKET    logEntry;
//
//     WDFFILEOBJECT           fileObject = NULL;
//     ULONG                   idx = 0;
//     ULONG                   nameLength = 0;
//
//     PAGED_CODE();
//
//     fileObject = WdfRequestGetFileObject(Request);
//
//     if (fileObject == NULL)
//     {
//         status = STATUS_INVALID_HANDLE;
//
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                     "RequestHandleExclusiveAccessLockDevice: FileObject is NULL, cannot grant exclusive access\n"));
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                sizeof(CDROM_EXCLUSIVE_LOCK),
//                                                &exclusiveLock,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Validate the caller name string
//         for (idx = 0; (idx < CDROM_EXCLUSIVE_CALLER_LENGTH) && (exclusiveLock->CallerName[idx] != '\0'); idx++)
//         {
//             if (!ValidChar(exclusiveLock->CallerName[idx]))
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "RequestHandleExclusiveAccessLockDevice: Invalid characters in caller name\n"));
//                 // error out
//                 status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if ((idx == 0) || (idx >= CDROM_EXCLUSIVE_CALLER_LENGTH))
//         {
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestHandleExclusiveAccessLockDevice: Not a valid null terminated string.\n"));
//             //error out
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             nameLength = idx+1; // Add 1 for the NULL character
//             NT_ASSERT(nameLength <= CDROM_EXCLUSIVE_CALLER_LENGTH);
//         }
//     }
//
//     // If the file system is still mounted on this device fail the request,
//     // unless the force lock flag is set.
//     if (NT_SUCCESS(status))
//     {
//         if ((TEST_FLAG(exclusiveLock->Access.Flags, CDROM_LOCK_IGNORE_VOLUME) == FALSE) &&
//             IsVolumeMounted(deviceExtension->DeviceObject))
//         {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "RequestHandleExclusiveAccessLockDevice: Unable to lock device, file system mounted\n"));
//                 status = STATUS_INVALID_DEVICE_STATE;
//         }
//     }
//
//     // Lock the device for exclusive access if the device is not already locked
//     if (NT_SUCCESS(status))
//     {
//         if (InterlockedCompareExchangePointer((PVOID)&cdData->ExclusiveOwner, (PVOID)fileObject, NULL) == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "RequestHandleExclusiveAccessLockDevice: Entering exclusive mode! Device locked by file object %p\n", fileObject));
//
//             // Zero out the CallerName before storing it in the extension
//             RtlZeroMemory(&cdData->CallerName, CDROM_EXCLUSIVE_CALLER_LENGTH);
//             RtlCopyMemory(&cdData->CallerName,
//                           &exclusiveLock->CallerName,
//                           nameLength);
//
//             // Send Exclusive Lock notification
//             DeviceSendNotification(deviceExtension,
//                                    &GUID_IO_CDROM_EXCLUSIVE_LOCK,
//                                    0,
//                                    NULL);
//
//             // Log an informational event with the caller name
//             logEntry = IoAllocateErrorLogEntry(
//                                 deviceExtension->DeviceObject,
//                                 sizeof(IO_ERROR_LOG_PACKET) + CDROM_EXCLUSIVE_CALLER_LENGTH);
//
//             if (logEntry != NULL)
//             {
//                 PUCHAR dumpDataPtr = (PUCHAR) logEntry->DumpData;
//
//                 logEntry->FinalStatus       = STATUS_SUCCESS;
//                 logEntry->ErrorCode         = IO_CDROM_EXCLUSIVE_LOCK;
//                 logEntry->SequenceNumber    = 0;
//                 logEntry->MajorFunctionCode = IRP_MJ_DEVICE_CONTROL;
//                 logEntry->IoControlCode     = IOCTL_CDROM_EXCLUSIVE_ACCESS;
//                 logEntry->RetryCount        = 0;
//                 logEntry->UniqueErrorValue  = 0x1;
//                 logEntry->DumpDataSize      = CDROM_EXCLUSIVE_CALLER_LENGTH;
//
//                 RtlCopyMemory(dumpDataPtr,
//                                 (PUCHAR)&cdData->CallerName,
//                                 CDROM_EXCLUSIVE_CALLER_LENGTH);
//
//                 // Write the error log packet.
//                 IoWriteErrorLogEntry(logEntry);
//             }
//
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestHandleExclusiveAccessLockDevice: Unable to lock device, device already locked.\n"));
//
//             status = STATUS_ACCESS_DENIED;
//         }
//     }
//
//     RequestCompletion(deviceExtension, Request, status, 0);
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleExclusiveAccessUnlockDevice(
//      WDFDEVICE Device,
//      WDFREQUEST Request
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_EXCLUSIVE_ACCESS with ExclusiveAccessUnlockDevice
//
// Arguments:
//
//     Device - device handle
//     Request - request to be handled
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_EXCLUSIVE_ACCESS exclusiveAccess = NULL;
//     WDFFILEOBJECT           fileObject = NULL;
//
//     PAGED_CODE();
//
//     fileObject = WdfRequestGetFileObject(Request);
//
//     if (fileObject == NULL)
//     {
//         // The device can be unlocked from exclusive mode only via the file object which locked it.
//         status = STATUS_INVALID_HANDLE;
//
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                     "RequestHandleExclusiveAccessUnlockDevice: FileObject is NULL, cannot release exclusive access\n"));
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                sizeof(PCDROM_EXCLUSIVE_ACCESS),
//                                                &exclusiveAccess,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceUnlockExclusive(deviceExtension, fileObject,
//                         TEST_FLAG(exclusiveAccess->Flags, CDROM_NO_MEDIA_NOTIFICATIONS));
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "RequestHandleExclusiveAccessUnlockDevice: Device unlocked\n"));
//     }
//
//     RequestCompletion(deviceExtension, Request, status, 0);
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleQueryPropertyRetrieveCachedData(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_QUERY_PROPERTY when the required data is cached.
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PSTORAGE_PROPERTY_QUERY inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->PropertyId == StorageDeviceProperty)
//         {
//             // check output buffer length
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength == 0)
//             {
//                 // According to MSDN, an output buffer of size 0 can be used to determine if a property exists
//                 // so this must be a success case with no data transferred
//                 *DataLength = 0;
//                 status = STATUS_SUCCESS;
//             }
//             else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(STORAGE_DESCRIPTOR_HEADER))
//             {
//                 // Buffer too small
//                 *DataLength = DeviceExtension->DeviceDescriptor->Size;
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//             else
//             {
//                 PSTORAGE_DEVICE_DESCRIPTOR  outputDescriptor = NULL;
//                 CHAR*                       localDescriptorBuffer = (CHAR*)DeviceExtension->DeviceDescriptor;
//
//                 status = WdfRequestRetrieveOutputBuffer(Request,
//                                                         RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                         &outputDescriptor,
//                                                         NULL);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     // transfer as much data out as the buffer will allow
//                     *DataLength = min(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                       DeviceExtension->DeviceDescriptor->Size);
//
//                     RtlCopyMemory(outputDescriptor,
//                                   DeviceExtension->DeviceDescriptor,
//                                   *DataLength);
//
//                     // walk through and update offset variables to reflect data that didn't make it into the output buffer
//                     if ((*DataLength >= RTL_SIZEOF_THROUGH_FIELD(STORAGE_DEVICE_DESCRIPTOR, VendorIdOffset)) &&
//                         (DeviceExtension->DeviceDescriptor->VendorIdOffset != 0) &&
//                         (DeviceExtension->DeviceDescriptor->VendorIdOffset != 0xFFFFFFFF))
//                     {
//                         // set VendorIdOffset appropriately
//                         if (*DataLength <
//                             (DeviceExtension->DeviceDescriptor->VendorIdOffset + strlen(localDescriptorBuffer + DeviceExtension->DeviceDescriptor->VendorIdOffset)))
//                         {
//                             outputDescriptor->VendorIdOffset = 0;
//                         }
//                     }
//
//                     if ((*DataLength >= RTL_SIZEOF_THROUGH_FIELD(STORAGE_DEVICE_DESCRIPTOR, ProductIdOffset)) &&
//                         (DeviceExtension->DeviceDescriptor->ProductIdOffset != 0) &&
//                         (DeviceExtension->DeviceDescriptor->ProductIdOffset != 0xFFFFFFFF))
//                     {
//                         // set ProductIdOffset appropriately
//                         if (*DataLength <
//                             (DeviceExtension->DeviceDescriptor->ProductIdOffset + strlen(localDescriptorBuffer + DeviceExtension->DeviceDescriptor->ProductIdOffset)))
//                         {
//                             outputDescriptor->ProductIdOffset = 0;
//                         }
//                     }
//
//                     if ((*DataLength >= RTL_SIZEOF_THROUGH_FIELD(STORAGE_DEVICE_DESCRIPTOR, ProductRevisionOffset)) &&
//                         (DeviceExtension->DeviceDescriptor->ProductRevisionOffset != 0) &&
//                         (DeviceExtension->DeviceDescriptor->ProductRevisionOffset != 0xFFFFFFFF))
//                     {
//                         // set ProductRevisionOffset appropriately
//                         if (*DataLength <
//                             (DeviceExtension->DeviceDescriptor->ProductRevisionOffset + strlen(localDescriptorBuffer + DeviceExtension->DeviceDescriptor->ProductRevisionOffset)))
//                         {
//                             outputDescriptor->ProductRevisionOffset = 0;
//                         }
//                     }
//
//                     if ((*DataLength >= RTL_SIZEOF_THROUGH_FIELD(STORAGE_DEVICE_DESCRIPTOR, SerialNumberOffset)) &&
//                         (DeviceExtension->DeviceDescriptor->SerialNumberOffset != 0) &&
//                         (DeviceExtension->DeviceDescriptor->SerialNumberOffset != 0xFFFFFFFF))
//                     {
//                         // set SerialNumberOffset appropriately
//                         if (*DataLength <
//                             (DeviceExtension->DeviceDescriptor->SerialNumberOffset + strlen(localDescriptorBuffer + DeviceExtension->DeviceDescriptor->SerialNumberOffset)))
//                         {
//                             // NOTE: setting this to 0 since that is what most port drivers do
//                             //       [this could cause issues with SCSI port devices whose clients expect -1 in this field]
//                             outputDescriptor->SerialNumberOffset = 0;
//                         }
//                     }
//                     status = STATUS_SUCCESS;
//                 }
//             }
//         }   //end of StorageDeviceProperty
//         else if (inputBuffer->PropertyId == StorageAdapterProperty)
//         {
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength == 0)
//             {
//                 // According to MSDN, an output buffer of size 0 can be used to determine if a property exists
//                 // so this must be a success case with no data transferred
//                 *DataLength = 0;
//                 status = STATUS_SUCCESS;
//             }
//             else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(STORAGE_DESCRIPTOR_HEADER))
//             {
//                 // Buffer too small
//                 *DataLength = DeviceExtension->AdapterDescriptor->Size;
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//             else
//             {
//                 PSTORAGE_ADAPTER_DESCRIPTOR outputDescriptor = NULL;
//
//                 status = WdfRequestRetrieveOutputBuffer(Request,
//                                                         RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                         &outputDescriptor,
//                                                         NULL);
//                 if (NT_SUCCESS(status))
//                 {
//                     // copy as much data out as the buffer will allow
//                     *DataLength = min(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                       DeviceExtension->AdapterDescriptor->Size);
//
//                     RtlCopyMemory(outputDescriptor,
//                                   DeviceExtension->AdapterDescriptor,
//                                   *DataLength);
//
//                     // set status
//                     status = STATUS_SUCCESS;
//                 }
//             }
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleQueryPropertyDeviceUniqueId(
//      WDFDEVICE        Device,
//      WDFREQUEST       Request
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_QUERY_PROPERTY with StorageDeviceUniqueIdProperty.
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//     PSTORAGE_PROPERTY_QUERY     inputBuffer = NULL;
//     PSTORAGE_DESCRIPTOR_HEADER  descHeader = NULL;
//     size_t                      outLength = 0;
//     WDF_REQUEST_PARAMETERS      requestParameters;
//
//     // Get the Request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            requestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         BOOLEAN overflow = FALSE;
//         BOOLEAN infoFound = FALSE;
//
//         // Must run at less then dispatch.
//         if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//         {
//             NT_ASSERT(FALSE);
//             outLength = 0;
//             status = STATUS_INVALID_LEVEL;
//         }
//         else if (inputBuffer->QueryType == PropertyExistsQuery)
//         {
//             outLength = 0;
//             status = STATUS_SUCCESS;
//         }
//         else if (inputBuffer->QueryType != PropertyStandardQuery)
//         {
//             outLength = 0;
//             status = STATUS_NOT_SUPPORTED;
//         }
//         else
//         {
//             // Check AdditionalParameters validity.
//             if (inputBuffer->AdditionalParameters[0] == DUID_INCLUDE_SOFTWARE_IDS)
//             {
//                 // Do nothing
//             }
//             else if (inputBuffer->AdditionalParameters[0] == DUID_HARDWARE_IDS_ONLY)
//             {
//                 // Do nothing
//             }
//             else
//             {
//                 outLength = 0;
//                 status = STATUS_INVALID_PARAMETER;
//             }
//
//             if (NT_SUCCESS(status) &&
//                 (outLength < sizeof(STORAGE_DESCRIPTOR_HEADER)))
//             {
//                 outLength = 0;
//                 status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//         }
//
//         // From this point forward the status depends on the overflow
//         // and infoFound flags.
//         if (NT_SUCCESS(status))
//         {
//             outLength = requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//             status = WdfRequestRetrieveOutputBuffer(Request,
//                                                     requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                     &descHeader,
//                                                     NULL);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             RtlZeroMemory(descHeader, outLength);
//
//             descHeader->Version = DUID_VERSION_1;
//             descHeader->Size = sizeof(STORAGE_DEVICE_UNIQUE_IDENTIFIER);
//
//             // Try to build device unique id from StorageDeviceIdProperty.
//             status = RequestDuidGetDeviceIdProperty(deviceExtension,
//                                                     Request,
//                                                     requestParameters,
//                                                     &outLength);
//
//             if (status == STATUS_BUFFER_OVERFLOW)
//             {
//                 overflow = TRUE;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 infoFound = TRUE;
//             }
//
//             // Try to build device unique id from StorageDeviceProperty.
//             status = RequestDuidGetDeviceProperty(deviceExtension,
//                                                   Request,
//                                                   requestParameters,
//                                                   &outLength);
//
//             if (status == STATUS_BUFFER_OVERFLOW)
//             {
//                 overflow = TRUE;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 infoFound = TRUE;
//             }
//
//             // Return overflow, success, or a generic error.
//             if (overflow)
//             {
//                 // If output buffer is STORAGE_DESCRIPTOR_HEADER, then return
//                 // success to the user.  Otherwise, send an error so the user
//                 // knows a larger buffer is required.
//                 if (outLength == sizeof(STORAGE_DESCRIPTOR_HEADER))
//                 {
//                     status = STATUS_SUCCESS;
//                 }
//                 else
//                 {
//                     outLength = (ULONG)WdfRequestGetInformation(Request);
//                     status = STATUS_BUFFER_OVERFLOW;
//                 }
//
//             }
//             else if (infoFound)
//             {
//                 status = STATUS_SUCCESS;
//
//                 // Exercise the compare routine.  This should always succeed.
//                 NT_ASSERT(DuidExactMatch == CompareStorageDuids((PSTORAGE_DEVICE_UNIQUE_IDENTIFIER)descHeader,
//                                                              (PSTORAGE_DEVICE_UNIQUE_IDENTIFIER)descHeader));
//
//             }
//             else
//             {
//                 status = STATUS_NOT_FOUND;
//             }
//         }
//     }
//
//     RequestCompletion(deviceExtension, Request, status, outLength);
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleQueryPropertyWriteCache(
//      WDFDEVICE    Device,
//      WDFREQUEST   Request
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_QUERY_PROPERTY with StorageDeviceWriteCacheProperty.
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                      status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION       deviceExtension = DeviceGetExtension(Device);
//     PSTORAGE_PROPERTY_QUERY       query = NULL;
//     PSTORAGE_WRITE_CACHE_PROPERTY writeCache = NULL;
//     PMODE_PARAMETER_HEADER        modeData = NULL;
//     PMODE_CACHING_PAGE            pageData = NULL;
//     size_t                        length = 0;
//     ULONG                         information = 0;
//     PSCSI_REQUEST_BLOCK           srb = NULL;
//     WDF_REQUEST_PARAMETERS        requestParameters;
//
//     // Get the Request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            requestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &query,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//
//         // Must run at less then dispatch.
//         if (KeGetCurrentIrql() >= DISPATCH_LEVEL)
//         {
//             NT_ASSERT(FALSE);
//             status = STATUS_INVALID_LEVEL;
//         }
//         else if (query->QueryType == PropertyExistsQuery)
//         {
//             information = 0;
//             status = STATUS_SUCCESS;
//         }
//         else if (query->QueryType != PropertyStandardQuery)
//         {
//             status = STATUS_NOT_SUPPORTED;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         length = requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//
//         if (length < sizeof(STORAGE_DESCRIPTOR_HEADER))
//         {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 requestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &writeCache,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory(writeCache, length);
//
//         // Set version and required size.
//         writeCache->Version = sizeof(STORAGE_WRITE_CACHE_PROPERTY);
//         writeCache->Size = sizeof(STORAGE_WRITE_CACHE_PROPERTY);
//
//         if (length < sizeof(STORAGE_WRITE_CACHE_PROPERTY))
//         {
//             // caller only wants header information, bail out.
//             information = sizeof(STORAGE_DESCRIPTOR_HEADER);
//             status = STATUS_SUCCESS;
//
//             RequestCompletion(deviceExtension, Request, status, information);
//             return status;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                     sizeof(SCSI_REQUEST_BLOCK) +
//                                     (sizeof(ULONG_PTR) * 2),
//                                     CDROM_TAG_SRB);
//
//         if (srb == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         // Set known values
//         writeCache->NVCacheEnabled = FALSE;
//         writeCache->UserDefinedPowerProtection = TEST_FLAG(deviceExtension->DeviceFlags, DEV_POWER_PROTECTED);
//
//         // Check for flush cache support by sending a sync cache command
//         // to the device.
//
//         // Set timeout value and mark the request as not being a tagged request.
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//         srb->TimeOutValue = TimeOutValueGetCapValue(deviceExtension->TimeOutValue, 4);
//         srb->QueueTag = SP_UNTAGGED;
//         srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//         srb->SrbFlags = deviceExtension->SrbFlags;
//
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->CdbLength = 10;
//
//         srb->Cdb[0] = SCSIOP_SYNCHRONIZE_CACHE;
//
//         status = DeviceSendSrbSynchronously(Device,
//                                             srb,
//                                             NULL,
//                                             0,
//                                             TRUE,   //flush drive cache
//                                             Request);
//
//         if (NT_SUCCESS(status))
//         {
//             writeCache->FlushCacheSupported = TRUE;
//         }
//         else
//         {
//             // Device does not support sync cache
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "RequestHandleQueryPropertyWriteCache: Synchronize cache failed with status 0x%X\n", status));
//             writeCache->FlushCacheSupported = FALSE;
//
//             // Reset the status if there was any failure
//             status = STATUS_SUCCESS;
//         }
//
//         modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                          MODE_PAGE_DATA_SIZE,
//                                          CDROM_TAG_MODE_DATA);
//
//         if (modeData == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                         "RequestHandleQueryPropertyWriteCache: Unable to allocate mode data buffer\n"));
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory(modeData, MODE_PAGE_DATA_SIZE);
//
//         length = DeviceRetrieveModeSenseUsingScratch(deviceExtension,
//                                                      (PCHAR)modeData,
//                                                      MODE_PAGE_DATA_SIZE,
//                                                      MODE_PAGE_CACHING,
//                                                      MODE_SENSE_CURRENT_VALUES);
//
//         if (length < sizeof(MODE_PARAMETER_HEADER))
//         {
//             // Retry the request in case of a check condition.
//             length = DeviceRetrieveModeSenseUsingScratch(deviceExtension,
//                                                          (PCHAR)modeData,
//                                                          MODE_PAGE_DATA_SIZE,
//                                                          MODE_PAGE_CACHING,
//                                                          MODE_SENSE_CURRENT_VALUES);
//
//             if (length < sizeof(MODE_PARAMETER_HEADER))
//             {
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, "RequestHandleQueryPropertyWriteCache: Mode Sense failed\n"));
//                 status = STATUS_IO_DEVICE_ERROR;
//             }
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // If the length is greater than length indicated by the mode data reset
//         // the data to the mode data.
//         if (length > (ULONG) (modeData->ModeDataLength + 1))
//         {
//             length = modeData->ModeDataLength + 1;
//         }
//
//         // Look for caching page in the returned mode page data.
//         pageData = ModeSenseFindSpecificPage((PCHAR)modeData,
//                                              length,
//                                              MODE_PAGE_CACHING,
//                                              TRUE);
//
//         // Check if valid caching page exists.
//         if (pageData == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "RequestHandleQueryPropertyWriteCache: Unable to find caching mode page.\n"));
//
//             // Set write cache value as unknown.
//             writeCache->WriteCacheEnabled = WriteCacheEnableUnknown;
//             writeCache->WriteCacheType = WriteCacheTypeUnknown;
//         }
//         else
//         {
//             writeCache->WriteCacheEnabled = pageData->WriteCacheEnable
//                                             ? WriteCacheEnabled
//                                             : WriteCacheDisabled;
//
//             writeCache->WriteCacheType = pageData->WriteCacheEnable
//                                          ? WriteCacheTypeWriteBack
//                                          : WriteCacheTypeUnknown;
//         }
//
//         // Check write through support.
//         if (modeData->DeviceSpecificParameter & MODE_DSP_FUA_SUPPORTED)
//         {
//             writeCache->WriteThroughSupported = WriteThroughSupported;
//         }
//         else
//         {
//             writeCache->WriteThroughSupported = WriteThroughNotSupported;
//         }
//
//         // Get the changeable caching mode page and check write cache is changeable.
//         RtlZeroMemory(modeData, MODE_PAGE_DATA_SIZE);
//
//         length = DeviceRetrieveModeSenseUsingScratch(deviceExtension,
//                                                      (PCHAR) modeData,
//                                                      MODE_PAGE_DATA_SIZE,
//                                                      MODE_PAGE_CACHING,
//                                                      MODE_SENSE_CHANGEABLE_VALUES);
//
//         if (length < sizeof(MODE_PARAMETER_HEADER))
//         {
//             // Retry the request in case of a check condition.
//             length = DeviceRetrieveModeSenseUsingScratch(deviceExtension,
//                                                          (PCHAR) modeData,
//                                                          MODE_PAGE_DATA_SIZE,
//                                                          MODE_PAGE_CACHING,
//                                                          MODE_SENSE_CHANGEABLE_VALUES);
//
//             if (length < sizeof(MODE_PARAMETER_HEADER))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "RequestHandleQueryPropertyWriteCache: Mode Sense failed\n"));
//
//                 // If the device fails to return changeable pages, then
//                 // set the write cache changeable value to unknown.
//                 writeCache->WriteCacheChangeable = WriteCacheChangeUnknown;
//                 information = sizeof(STORAGE_WRITE_CACHE_PROPERTY);
//             }
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // If the length is greater than length indicated by the mode data reset
//         // the data to the mode data.
//         if (length > (ULONG) (modeData->ModeDataLength + 1))
//         {
//             length = modeData->ModeDataLength + 1;
//         }
//
//         // Look for caching page in the returned mode page data.
//         pageData = ModeSenseFindSpecificPage((PCHAR)modeData,
//                                              length,
//                                              MODE_PAGE_CACHING,
//                                              TRUE);
//         // Check if valid caching page exists.
//         if (pageData == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "RequestHandleQueryPropertyWriteCache: Unable to find caching mode page.\n"));
//
//             // Set write cache changeable value to unknown.
//             writeCache->WriteCacheChangeable = WriteCacheChangeUnknown;
//         }
//         else
//         {
//             writeCache->WriteCacheChangeable = pageData->WriteCacheEnable
//                                                ? WriteCacheChangeable
//                                                : WriteCacheNotChangeable;
//         }
//
//         information = sizeof(STORAGE_WRITE_CACHE_PROPERTY);
//
//     }
//
//     FREE_POOL(srb);
//     FREE_POOL(modeData);
//
//     RequestCompletion(deviceExtension, Request, status, information);
//
//     return status;
// }
//
// NTSTATUS
// RequestValidateDvdReadKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_DVD_READ_KEY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PDVD_COPY_PROTECT_KEY   keyParameters = NULL;
//     ULONG                   keyLength = 0;
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &keyParameters,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         if(RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < sizeof(DVD_COPY_PROTECT_KEY))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "DvdDeviceControl: EstablishDriveKey - challenge "
//                         "key buffer too small\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         switch(keyParameters->KeyType)
//         {
//
//             case DvdChallengeKey:
//             {
//                 C_ASSERT(sizeof(DVD_COPY_PROTECT_KEY) <= DVD_CHALLENGE_KEY_LENGTH);
//                 keyLength = DVD_CHALLENGE_KEY_LENGTH;
//                 break;
//             }
//             case DvdBusKey1:
//             case DvdBusKey2:
//             {
//                 C_ASSERT(sizeof(DVD_COPY_PROTECT_KEY) <= DVD_BUS_KEY_LENGTH);
//                 keyLength = DVD_BUS_KEY_LENGTH;
//                 break;
//             }
//             case DvdTitleKey:
//             {
//                 C_ASSERT(sizeof(DVD_COPY_PROTECT_KEY) <= DVD_TITLE_KEY_LENGTH);
//                 keyLength = DVD_TITLE_KEY_LENGTH;
//                 break;
//             }
//             case DvdAsf:
//             {
//                 C_ASSERT(sizeof(DVD_COPY_PROTECT_KEY) <= DVD_ASF_LENGTH);
//                 keyLength = DVD_ASF_LENGTH;
//                 break;
//             }
//             case DvdDiskKey:
//             {
//                 C_ASSERT(sizeof(DVD_COPY_PROTECT_KEY) <= DVD_DISK_KEY_LENGTH);
//                 keyLength = DVD_DISK_KEY_LENGTH;
//                 break;
//             }
//             case DvdGetRpcKey:
//             {
//                 C_ASSERT(sizeof(DVD_COPY_PROTECT_KEY) <= DVD_RPC_KEY_LENGTH);
//                 keyLength = DVD_RPC_KEY_LENGTH;
//                 break;
//             }
//             default:
//             {
//                 keyLength = sizeof(DVD_COPY_PROTECT_KEY);
//                 break;
//             }
//         }
//
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < keyLength)
//         {
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "DvdDeviceControl: EstablishDriveKey - output "
//                         "buffer too small\n"));
//             status = STATUS_BUFFER_TOO_SMALL;
//             *DataLength = keyLength;
//         }
//         else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength &
//                  DeviceExtension->AdapterDescriptor->AlignmentMask)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else if (DeviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//         {
//             // reject the request if it's not a DVD device.
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestValidateDvdEndSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DVD_END_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID sessionId = NULL;
//
//     UNREFERENCED_PARAMETER(DeviceExtension);
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &sessionId,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         if(RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//             sizeof(DVD_SESSION_ID))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "DvdDeviceControl: EndSession - input buffer too "
//                         "small\n"));
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestValidateAacsEndSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_AACS_END_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID sessionId = NULL;
//     PCDROM_DATA     cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &sessionId,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         if (!cdData->Mmc.IsAACS)
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//         else if(RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength != sizeof(DVD_SESSION_ID))
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestValidateEnableStreaming(
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Validates an IOCTL_CDROM_ENABLE_STREAMING request
//
// Arguments:
//
//     Request - request to be handled
//     RequestParameters - request parameters
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//
//     PCDROM_STREAMING_CONTROL    inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//         sizeof(CDROM_STREAMING_CONTROL))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Get the request type using CDROM_STREAMING_CONTROL structure
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                sizeof(CDROM_STREAMING_CONTROL),
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->RequestType != CdromStreamingDisable &&
//             inputBuffer->RequestType != CdromStreamingEnableForReadOnly &&
//             inputBuffer->RequestType != CdromStreamingEnableForWriteOnly &&
//             inputBuffer->RequestType != CdromStreamingEnableForReadWrite)
//         {
//             // Unknown request type
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestValidateSendOpcInformation(
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Validates an IOCTL_CDROM_SEND_OPC_INFORMATION request
//
// Arguments:
//
//     Request - request to be handled
//     RequestParameters - request parameters
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//
//     PCDROM_SIMPLE_OPC_INFO      inputBuffer = NULL;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//         sizeof(CDROM_SIMPLE_OPC_INFO))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Get the request type using CDROM_SIMPLE_OPC_INFO structure
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                sizeof(CDROM_SIMPLE_OPC_INFO),
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->RequestType != SimpleOpcInfo)
//         {
//             // Unknown request type
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestValidateGetPerformance(
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Validates an IOCTL_CDROM_GET_PERFORMANCE request
//
// Arguments:
//
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_WRITE_SPEED_REQUEST  writeSpeedRequest = NULL;
//     PCDROM_PERFORMANCE_REQUEST  performanceRequest = NULL;
//
//     *DataLength = 0;
//
//     // CDROM_WRITE_SPEED_REQUEST is the smallest performance request that we support.
//     // We use it to retrieve request type and then check input length more carefully
//     // on a per request type basis.
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//         sizeof(CDROM_WRITE_SPEED_REQUEST))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                sizeof(CDROM_WRITE_SPEED_REQUEST),
//                                                (PVOID*)&writeSpeedRequest,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (writeSpeedRequest->RequestType == CdromPerformanceRequest)
//         {
//             // CDROM_PERFORMANCE_REQUEST is bigger than CDROM_WRITE_SPEED_REQUEST,
//             // so we perform more checks and retrieve more bytes through WDF.
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//                 sizeof(CDROM_PERFORMANCE_REQUEST))
//             {
//                 status = STATUS_INFO_LENGTH_MISMATCH;
//             }
//             if (NT_SUCCESS(status))
//             {
//                 status = WdfRequestRetrieveInputBuffer(Request,
//                                                        sizeof(CDROM_PERFORMANCE_REQUEST),
//                                                        &performanceRequest,
//                                                        NULL);
//             }
//
//             if ((!NT_SUCCESS(status)))
//             {
//                 // just pass the status code from above
//             }
//             // validate all enum-type fields of CDROM_PERFORMANCE_REQUEST
//             else if (performanceRequest->PerformanceType != CdromReadPerformance &&
//                      performanceRequest->PerformanceType != CdromWritePerformance)
//             {
//                 status = STATUS_INVALID_PARAMETER;
//             }
//             else if (performanceRequest->Exceptions != CdromNominalPerformance &&
//                      performanceRequest->Exceptions != CdromEntirePerformanceList &&
//                      performanceRequest->Exceptions != CdromPerformanceExceptionsOnly)
//             {
//                 status = STATUS_INVALID_PARAMETER;
//             }
//             else if (performanceRequest->Tolerance != Cdrom10Nominal20Exceptions)
//             {
//                 status = STATUS_INVALID_PARAMETER;
//             }
//         }
//         else if (writeSpeedRequest->RequestType == CdromWriteSpeedRequest)
//         {
//             // No additional checks here: all remaining fields are ignored
//             // if RequestType == CdromWriteSpeedRequest.
//         }
//         else
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     // finally, check output buffer length
//     if (NT_SUCCESS(status))
//     {
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//                  sizeof(CDROM_PERFORMANCE_HEADER))
//         {
//             status = STATUS_BUFFER_TOO_SMALL;
//             *DataLength = sizeof(CDROM_PERFORMANCE_HEADER);
//         }
//         else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >
//             ((USHORT)-1))
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// PCDB
// RequestGetScsiPassThroughCdb(
//      PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     Get the CDB structure from the SCSI pass through
//
// Arguments:
//
//     Irp - request to be handled
//
// Return Value:
//
//     PCDB
//
// --*/
// {
//     PCDB                cdb = NULL;
//     PIO_STACK_LOCATION  currentIrpStack = IoGetCurrentIrpStackLocation(Irp);
//     ULONG               inputBufferLength = 0;
//     PVOID               inputBuffer = NULL;
//     BOOLEAN             legacyPassThrough = FALSE;
//
//     PAGED_CODE();
//
//     if (((currentIrpStack->Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_SCSI_PASS_THROUGH) ||
//          (currentIrpStack->Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT) ||
//          (currentIrpStack->Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_SCSI_PASS_THROUGH_EX) ||
//          (currentIrpStack->Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT_EX)) &&
//         (Irp->AssociatedIrp.SystemBuffer != NULL))
//     {
//         inputBufferLength = currentIrpStack->Parameters.DeviceIoControl.as_ref().InputBufferLength;
//         inputBuffer = Irp->AssociatedIrp.SystemBuffer;
//         legacyPassThrough = TRUE;
//
//         if ((currentIrpStack->Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_SCSI_PASS_THROUGH_EX) ||
//             (currentIrpStack->Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_SCSI_PASS_THROUGH_DIRECT_EX))
//         {
//             legacyPassThrough = FALSE;
//         }
//
//         //
//         //  If this is a 32 bit application running on 64 bit then thunk the
//         //  input structures to grab the cdb.
//         //
//
// #if BUILD_WOW64_ENABLED && defined(_WIN64)
//
//         if (IoIs32bitProcess(Irp))
//         {
//             if (legacyPassThrough)
//             {
//                 if (inputBufferLength >= sizeof(SCSI_PASS_THROUGH32))
//                 {
//                     cdb = (PCDB)((PSCSI_PASS_THROUGH32)inputBuffer)->Cdb;
//                 }
//             }
//             else
//             {
//                 if (inputBufferLength >= sizeof(SCSI_PASS_THROUGH32_EX))
//                 {
//                     cdb = (PCDB)((PSCSI_PASS_THROUGH32_EX)inputBuffer)->Cdb;
//                 }
//             }
//
//         }
//         else
//
// #endif
//
//         {
//             if (legacyPassThrough)
//             {
//                 if (inputBufferLength >= sizeof(SCSI_PASS_THROUGH))
//                 {
//                     cdb = (PCDB)((PSCSI_PASS_THROUGH)inputBuffer)->Cdb;
//                 }
//             }
//             else
//             {
//                 if (inputBufferLength >= sizeof(SCSI_PASS_THROUGH_EX))
//                 {
//                     cdb = (PCDB)((PSCSI_PASS_THROUGH_EX)inputBuffer)->Cdb;
//                 }
//             }
//         }
//     }
//
//     return cdb;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleScsiPassThrough(
//      WDFDEVICE    Device,
//      WDFREQUEST   Request
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_SCSI_PASS_THROUGH
//                      IOCTL_SCSI_PASS_THROUGH_DIRECT
//
//    The function sets the MinorFunction field of irpStack,
//    and pass the request to lower level driver.
//
// Arguments:
//
//     Device - device object
//     Request - request to be handled
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_UNSUCCESSFUL;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//     PIRP                        irp = WdfRequestWdmGetIrp(Request);
//     PZERO_POWER_ODD_INFO        zpoddInfo = deviceExtension->ZeroPowerODDInfo;
//     PCDB                        cdb = NULL;
//     BOOLEAN                     isSoftEject = FALSE;
//
// #if DBG
//     PCDROM_REQUEST_CONTEXT      requestContext = RequestGetContext(Request);
// #endif
//
//
//     PAGED_CODE();
//
// #if DBG
//     // SPTI is always processed in sync manner.
//     NT_ASSERT(requestContext->SyncRequired);
// #endif
//
//     if ((zpoddInfo != NULL) &&
//         (zpoddInfo->LoadingMechanism == LOADING_MECHANISM_TRAY) &&
//         (zpoddInfo->Load == 0))                                         // Drawer
//     {
//         cdb = RequestGetScsiPassThroughCdb(irp);
//
//         if ((cdb != NULL) &&
//             (cdb->AsByte[0] == SCSIOP_START_STOP_UNIT) &&
//             (cdb->START_STOP.LoadEject == 1) &&
//             (cdb->START_STOP.Start == 0))
//         {
//             isSoftEject = TRUE;
//         }
//     }
//
//     WdfRequestFormatRequestUsingCurrentType(Request);
//
//     // Special for SPTI, set the MinorFunction.
//     {
//         PIO_STACK_LOCATION  nextStack = IoGetNextIrpStackLocation(irp);
//
//         nextStack->MinorFunction = 1;
//     }
//
//
//     status = RequestSend(deviceExtension,
//                          Request,
//                          deviceExtension->IoTarget,
//                          WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
//                          NULL);
//
//
//     if ((!NT_SUCCESS(status)) &&
//         (isSoftEject != FALSE))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                    "RequestHandleScsiPassThrough: soft eject detected, device marked as active\n"));
//
//         DeviceMarkActive(deviceExtension, TRUE, FALSE);
//     }
//
//     RequestCompletion(deviceExtension, Request, status, WdfRequestGetInformation(Request));
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleMountQueryUniqueId(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_MOUNTDEV_QUERY_UNIQUE_ID
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PMOUNTDEV_UNIQUE_ID uniqueId = NULL;
//
//     *DataLength = 0;
//
//     if (!DeviceExtension->MountedDeviceInterfaceName.Buffer)
//     {
//         status = STATUS_INVALID_PARAMETER;
//     }
//     else if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(MOUNTDEV_UNIQUE_ID))
//     {
//         *DataLength = sizeof(MOUNTDEV_UNIQUE_ID);
//         status = STATUS_BUFFER_TOO_SMALL;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &uniqueId,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory(uniqueId, RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength);
//
//         uniqueId->UniqueIdLength = DeviceExtension->MountedDeviceInterfaceName.Length;
//
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//             (sizeof(USHORT) + DeviceExtension->MountedDeviceInterfaceName.Length))
//         {
//             *DataLength = sizeof(MOUNTDEV_UNIQUE_ID);
//             status = STATUS_BUFFER_OVERFLOW;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlCopyMemory(uniqueId->UniqueId,
//                       DeviceExtension->MountedDeviceInterfaceName.Buffer,
//                       uniqueId->UniqueIdLength);
//
//         *DataLength = sizeof(USHORT) + uniqueId->UniqueIdLength;
//         status = STATUS_SUCCESS;
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleMountQueryDeviceName(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_MOUNTDEV_QUERY_DEVICE_NAME
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//     PMOUNTDEV_NAME  name = NULL;
//
//     *DataLength = 0;
//
//     NT_ASSERT(DeviceExtension->DeviceName.Buffer);
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < sizeof(MOUNTDEV_NAME))
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(MOUNTDEV_NAME);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &name,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory(name, RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength);
//         name->NameLength = DeviceExtension->DeviceName.Length;
//
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//             (sizeof(USHORT) + DeviceExtension->DeviceName.Length))
//         {
//             status = STATUS_BUFFER_OVERFLOW;
//             *DataLength = sizeof(MOUNTDEV_NAME);
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlCopyMemory(name->Name,
//                       DeviceExtension->DeviceName.Buffer,
//                       name->NameLength);
//
//         status = STATUS_SUCCESS;
//         *DataLength = sizeof(USHORT) + name->NameLength;
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleMountQuerySuggestedLinkName(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//
//     PMOUNTDEV_SUGGESTED_LINK_NAME suggestedName = NULL;
//
//     WCHAR                    driveLetterNameBuffer[10] = {0};
//     RTL_QUERY_REGISTRY_TABLE queryTable[2] = {0};
//     PWSTR                    valueName = NULL;
//     UNICODE_STRING           driveLetterName = {0};
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <
//         sizeof(MOUNTDEV_SUGGESTED_LINK_NAME))
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(MOUNTDEV_SUGGESTED_LINK_NAME);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         valueName = ExAllocatePoolWithTag(PagedPool,
//                                           DeviceExtension->DeviceName.Length + sizeof(WCHAR),
//                                           CDROM_TAG_STRINGS);
//         if (valueName == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlCopyMemory(valueName,
//                       DeviceExtension->DeviceName.Buffer,
//                       DeviceExtension->DeviceName.Length);
//         valueName[DeviceExtension->DeviceName.Length/sizeof(WCHAR)] = 0;
//
//         driveLetterName.Buffer = driveLetterNameBuffer;
//         driveLetterName.MaximumLength = sizeof(driveLetterNameBuffer);
//         driveLetterName.Length = 0;
//
//         queryTable[0].Flags = RTL_QUERY_REGISTRY_REQUIRED | RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK;
//         queryTable[0].Name = valueName;
//         queryTable[0].EntryContext = &driveLetterName;
//         queryTable[0].DefaultType = (REG_SZ << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | REG_NONE;
//
//         status = RtlQueryRegistryValues(RTL_REGISTRY_ABSOLUTE,
//                                         L"\\Registry\\Machine\\System\\DISK", // why hard coded?
//                                         queryTable, NULL, NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if ((driveLetterName.Length == 4) &&
//             (driveLetterName.Buffer[0] == '%') &&
//             (driveLetterName.Buffer[1] == ':'))
//         {
//             driveLetterName.Buffer[0] = 0xFF;
//         }
//         else if ((driveLetterName.Length != 4) ||
//                  (driveLetterName.Buffer[0] < FirstDriveLetter) ||
//                  (driveLetterName.Buffer[0] > LastDriveLetter) ||
//                  (driveLetterName.Buffer[1] != ':'))
//         {
//             status = STATUS_NOT_FOUND;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &suggestedName,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlZeroMemory(suggestedName, RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength);
//         suggestedName->UseOnlyIfThereAreNoOtherLinks = TRUE;
//         suggestedName->NameLength = 28;
//
//         *DataLength = FIELD_OFFSET(MOUNTDEV_SUGGESTED_LINK_NAME, Name) + 28;
//
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < *DataLength)
//         {
//             *DataLength = sizeof(MOUNTDEV_SUGGESTED_LINK_NAME);
//             status = STATUS_BUFFER_OVERFLOW;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RtlDeleteRegistryValue(RTL_REGISTRY_ABSOLUTE,
//                                L"\\Registry\\Machine\\System\\DISK",
//                                valueName);
//
//         RtlCopyMemory(suggestedName->Name, L"\\DosDevices\\", 24);
//         suggestedName->Name[12] = driveLetterName.Buffer[0];
//         suggestedName->Name[13] = ':';
//     }
//
//     FREE_POOL(valueName);
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleReadTOC(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_READ_TOC
//                      IOCTL_CDROM_GET_LAST_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS status = STATUS_SUCCESS;
//     VOID*    outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (DeviceIsPlayActive(DeviceExtension->Device))
//     {
//         status = STATUS_DEVICE_BUSY;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     // handle the request
//     if (NT_SUCCESS(status))
//     {
//         size_t  transferSize;
//         CDB     cdb;
//
//         transferSize = min(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength, sizeof(CDROM_TOC));
//
//         RtlZeroMemory(outputBuffer, transferSize);
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_CDROM_GET_LAST_SESSION)
//         {
//             // Set format to return first and last session numbers.
//             cdb.READ_TOC.as_mut().Format2 = CDROM_READ_TOC_EX_FORMAT_SESSION;
//         }
//         else
//         {
//             // Use MSF addressing
//             cdb.READ_TOC.as_mut().Msf = 1;
//         }
//
//         cdb.READ_TOC.as_mut().OperationCode = SCSIOP_READ_TOC;
//         cdb.READ_TOC.as_mut().AllocationLength[0] = (UCHAR)(transferSize >> 8);
//         cdb.READ_TOC.as_mut().AllocationLength[1] = (UCHAR)(transferSize & 0xFF);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, (ULONG)transferSize, TRUE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//             RtlCopyMemory(outputBuffer,
//                           DeviceExtension->ScratchContext.ScratchBuffer,
//                           *DataLength);
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleReadTocEx(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_READ_TOC_EX
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_READ_TOC_EX  inputBuffer = NULL;
//     VOID*               outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (DeviceIsPlayActive(DeviceExtension->Device))
//     {
//         status = STATUS_DEVICE_BUSY;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     // handle the request
//     if (NT_SUCCESS(status))
//     {
//         size_t  transferSize;
//         CDB     cdb;
//
//         transferSize = min(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength, MAXUSHORT);
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.READ_TOC.as_mut().OperationCode = SCSIOP_READ_TOC;
//         cdb.READ_TOC.as_mut().Msf = inputBuffer->Msf;
//         cdb.READ_TOC.as_mut().Format2 = inputBuffer->Format;
//         cdb.READ_TOC.as_mut().StartingTrack = inputBuffer->SessionTrack;
//         cdb.READ_TOC.as_mut().AllocationLength[0] = (UCHAR)(transferSize >> 8);
//         cdb.READ_TOC.as_mut().AllocationLength[1] = (UCHAR)(transferSize & 0xFF);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, (ULONG)transferSize, TRUE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             if (DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength < MINIMUM_CDROM_READ_TOC_EX_SIZE)
//             {
//                 *DataLength = 0;
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//             else
//             {
//                 *DataLength = DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//                 RtlCopyMemory(outputBuffer,
//                               DeviceExtension->ScratchContext.ScratchBuffer,
//                               *DataLength);
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// GetConfigurationDataConversionTypeAllToTypeOne(
//         FEATURE_NUMBER       RequestedFeature,
//         PSCSI_REQUEST_BLOCK  Srb,
//        size_t *             DataLength
//     )
// /*++
//
// Routine Description:
//
//     Some CDROM devices do not handle the GET CONFIGURATION commands with
//     TYPE ONE request. The command will time out causing a bus reset.
//     To avoid this problem we set a device flag during start device if the device
//     fails a TYPE ONE request. If this flag is set the TYPE ONE requests will be
//     tried as TYPE ALL request and the data will be converted to TYPE ONE format
//     in this routine.
//
// Arguments:
//
//     RequestedFeature - device context
//     Srb - request to be handled
//     DataLength - transfer data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     PFEATURE_HEADER     featureHeader = NULL;
//     FEATURE_NUMBER      thisFeature;
//     ULONG               totalLength = 0;
//     ULONG               featureLength = 0;
//     ULONG               headerLength = 0;
//
//     PGET_CONFIGURATION_HEADER   header = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (Srb->DataTransferLength < RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength))
//     {
//         // do not have valid data.
//         return;
//     }
//
//     // Calculate the length of valid data available in the
//     // capabilities buffer from the DataLength field
//     header = (PGET_CONFIGURATION_HEADER) Srb->DataBuffer;
//     REVERSE_BYTES(&totalLength, header->DataLength);
//
//     totalLength += RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//     // Make sure the we have enough data in the SRB
//     totalLength = min(totalLength, Srb->DataTransferLength);
//
//     // If we have received enough data from the device
//     // check for the given feature.
//     if (totalLength >= (sizeof(GET_CONFIGURATION_HEADER) + sizeof(FEATURE_HEADER)))
//     {
//         // Feature information is present. Verify the feature.
//         featureHeader = (PFEATURE_HEADER)((PUCHAR)Srb->DataBuffer + sizeof(GET_CONFIGURATION_HEADER));
//
//         thisFeature  = (featureHeader->FeatureCode[0] << 8) | (featureHeader->FeatureCode[1]);
//
//         if (thisFeature == RequestedFeature)
//         {
//             // Calculate the feature length
//             featureLength = sizeof(FEATURE_HEADER) + featureHeader->AdditionalLength;
//         }
//     }
//
//     // Calculate the total size
//     totalLength = sizeof(GET_CONFIGURATION_HEADER) + featureLength;
//
//     headerLength = totalLength -
//         RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//     REVERSE_BYTES(header->DataLength, &headerLength);
//
//     *DataLength = totalLength;
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// GetConfigurationDataSynthesize(
//         PVOID           InputBuffer,
//                                 ULONG           InputBufferSize,
//       PVOID           OutputBuffer,
//                                 size_t          OutputBufferSize,
//                                 FEATURE_NUMBER  StartingFeature,
//                                 ULONG           RequestType,
//                                size_t *        DataLength
//     )
// /*++
//
// Routine Description:
//
//     Get Configuration is a frequently used command, and we don't want it to wake
//     up the device in case it is in Zero Power state. Before entering Zero Power state,
//     the complete response of the command is saved in cache, and since the response
//     is always the same in case there is no media, we can synthesize the response
//     based on the user request.
//
// Arguments:
//
//     InputBuffer - buffer containing cached command response
//     InputBufferSize - size of above buffer
//     OutputBuffer - buffer to fill in result
//     OutputBufferSize - size of above buffer
//     StartingFeature - requested Starting Feature Number
//     RequestType - requested Request Type
//     DataLength - transfer data length
//
// Return Value:
//
// --*/
// {
//     PFEATURE_HEADER     featureHeader = NULL;
//     ULONG               validLength = 0;
//     ULONG               featureLength = 0;
//     ULONG               headerLength = 0;
//     PUCHAR              buffer = NULL;
//     ULONG               bytesRemaining = 0;
//     FEATURE_NUMBER      featureCode = 0;
//     BOOLEAN             shouldCopy = FALSE;
//     size_t              copyLength = 0;
//     size_t              transferedLength = 0;
//     size_t              requiredLength = 0;
//
//     PGET_CONFIGURATION_HEADER   header = NULL;
//
//     PAGED_CODE ();
//
//     if (InputBufferSize < sizeof (GET_CONFIGURATION_HEADER))
//     {
//         // do not have valid data.
//         *DataLength = 0;
//
//         return;
//     }
//
//     // Calculate the length of valid data available in the
//     // capabilities buffer from the DataLength field
//     header = (PGET_CONFIGURATION_HEADER) InputBuffer;
//     REVERSE_BYTES(&validLength, header->DataLength);
//
//     validLength += RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//     // Make sure we have enough data
//     validLength = min(validLength, InputBufferSize);
//
//     // Copy the header first
//     copyLength = min(OutputBufferSize, sizeof (GET_CONFIGURATION_HEADER));
//
//     RtlMoveMemory(OutputBuffer,
//                   InputBuffer,
//                   copyLength);
//
//     transferedLength = copyLength;
//     requiredLength = sizeof (GET_CONFIGURATION_HEADER);
//
//     if (validLength > sizeof (GET_CONFIGURATION_HEADER))
//     {
//         buffer = header->Data;
//         bytesRemaining = validLength - sizeof (GET_CONFIGURATION_HEADER);
//
//         // Ignore incomplete feature descriptor
//         while (bytesRemaining >= sizeof (FEATURE_HEADER))
//         {
//             featureHeader = (PFEATURE_HEADER) buffer;
//             shouldCopy = FALSE;
//
//             featureCode = (featureHeader->FeatureCode[0] << 8) | (featureHeader->FeatureCode[1]);
//             featureLength = sizeof (FEATURE_HEADER) + featureHeader->AdditionalLength;
//
//             if (featureCode >= StartingFeature)
//             {
//                 switch (RequestType) {
//
//                 case SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL:
//
//                     shouldCopy = TRUE;
//                     break;
//
//                 case SCSI_GET_CONFIGURATION_REQUEST_TYPE_CURRENT:
//
//                     if (featureHeader->Current)
//                     {
//                         shouldCopy = TRUE;
//                     }
//                     break;
//
//                 case SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE:
//
//                     if (featureCode == StartingFeature)
//                     {
//                         shouldCopy = TRUE;
//                     }
//                     break;
//
//                 default:
//
//                     break;
//                 }
//             }
//
//             if (shouldCopy != FALSE)
//             {
//                 copyLength = min(featureLength, bytesRemaining);
//                 copyLength = min(copyLength, OutputBufferSize - transferedLength);
//
//                 RtlMoveMemory((PUCHAR) OutputBuffer + transferedLength,
//                               buffer,
//                               copyLength);
//
//                 transferedLength += copyLength;
//                 requiredLength += featureLength;
//             }
//
//             buffer += min(featureLength, bytesRemaining);
//             bytesRemaining -= min(featureLength, bytesRemaining);
//         }
//     }
//
//     // Adjust Data Length field in header
//     if (transferedLength >= RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength))
//     {
//         headerLength = (ULONG) requiredLength - RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//         header = (PGET_CONFIGURATION_HEADER) OutputBuffer;
//         REVERSE_BYTES(header->DataLength, &headerLength);
//     }
//
//     *DataLength = transferedLength;
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleGetConfiguration(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_GET_CONFIGURATION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PCDROM_DATA                     cdData = &(DeviceExtension->DeviceAdditionalData);
//     PGET_CONFIGURATION_IOCTL_INPUT  inputBuffer = NULL;
//     VOID*                           outputBuffer = NULL;
//     size_t                          transferByteCount = 0;
//     PZERO_POWER_ODD_INFO            zpoddInfo = DeviceExtension->ZeroPowerODDInfo;
//     BOOLEAN                         inZeroPowerState = FALSE;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     //
//     if (!cdData->Mmc.IsMmc)
//     {
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &inputBuffer,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // If device is Zero Power state, there should be no media in device, thus we can synthesize the response
//         // from our cache. Avoid waking up the device in this case.
//         if ((zpoddInfo != NULL) &&
//             (zpoddInfo->InZeroPowerState != FALSE))
//         {
//             inZeroPowerState = TRUE;
//         }
//
//         if ((inZeroPowerState == FALSE) ||
//             (zpoddInfo->GetConfigurationBuffer == NULL))
//         {
//             CDB cdb;
//
//             //The maximum number of bytes that a Drive may return
//             //to describe its Features in one GET CONFIGURATION Command is 65,534
//             transferByteCount = min(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength, (MAXUSHORT - 1));
//
//             // If this is a TYPE ONE request and if this device can't handle this
//             // request, then we need to send TYPE ALL request to the device and
//             // convert the data in the completion routine. If required allocate a big
//             // buffer to get both configuration and feature header.
//             if ((inputBuffer->RequestType == SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
//                 TEST_FLAG(cdData->HackFlags, CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG))
//             {
//                 transferByteCount = max(transferByteCount,
//                                         sizeof(GET_CONFIGURATION_HEADER) + sizeof(FEATURE_HEADER));
//             }
//
//             ScratchBuffer_BeginUse(DeviceExtension);
//
//             RtlZeroMemory(&cdb, sizeof(CDB));
//             // Set up the CDB
//             cdb.GET_CONFIGURATION.OperationCode = SCSIOP_GET_CONFIGURATION;
//             cdb.GET_CONFIGURATION.AllocationLength[0] = (UCHAR)(transferByteCount >> 8);
//             cdb.GET_CONFIGURATION.AllocationLength[1] = (UCHAR)(transferByteCount & 0xff);
//
//             cdb.GET_CONFIGURATION.StartingFeature[0] = (UCHAR)(inputBuffer->Feature >> 8);
//             cdb.GET_CONFIGURATION.StartingFeature[1] = (UCHAR)(inputBuffer->Feature & 0xff);
//             cdb.GET_CONFIGURATION.RequestType        = (UCHAR)(inputBuffer->RequestType);
//
//             // If the device does not support TYPE ONE get configuration commands
//             // then change the request type to TYPE ALL. Convert the returned data to
//             // TYPE ONE format in the completion routine.
//             if ((inputBuffer->RequestType == SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
//                 TEST_FLAG(cdData->HackFlags, CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG))
//             {
//
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                            "DeviceHandleGetConfiguration: Changing TYPE_ONE Get Config to TYPE_ALL\n"));
//                 cdb.GET_CONFIGURATION.RequestType = SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL;
//             }
//
//             status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, (ULONG)transferByteCount, TRUE, &cdb, 12);
//
//             if (NT_SUCCESS(status))
//             {
//                 if ((inputBuffer->RequestType == SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE) &&
//                     TEST_FLAG(cdData->HackFlags, CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG))
//                 {
//
//                     if (DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength < sizeof(GET_CONFIGURATION_HEADER))
//                     {
//                         // Not enough data to calculate the data length.
//                         // So assume feature is not present
//                         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL, "DeviceHandleGetConfiguration: No get config header!\n"));
//                         *DataLength = 0;
//                         status = STATUS_INVALID_DEVICE_REQUEST;
//                     }
//                     else
//                     {
//                         //Some CDROM devices do not handle the GET CONFIGURATION commands with
//                         //TYPE ONE request. The command will time out causing a bus reset.
//                         //To avoid this problem we set a device flag during start device if the device
//                         //fails a TYPE ONE request. If this flag is set the TYPE ONE requests will be
//                         //tried as TYPE ALL request and the data will be converted to TYPE ONE format
//                         //in this routine.
//                         GetConfigurationDataConversionTypeAllToTypeOne(inputBuffer->Feature, DeviceExtension->ScratchContext.ScratchSrb, DataLength);
//                         *DataLength = min(*DataLength, RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength);
//                     }
//                 }
//                 else
//                 {
//                     *DataLength = DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//                 }
//
//                 // copy data to output buffer
//                 if (NT_SUCCESS(status))
//                 {
//                     RtlMoveMemory(outputBuffer,
//                                   DeviceExtension->ScratchContext.ScratchBuffer,
//                                   *DataLength);
//                 }
//             }
//
//             ScratchBuffer_EndUse(DeviceExtension);
//         }
//         else
//         {
//             // We are in Zero Power state, and our cached response is available.
//             // Synthesize the requested data.
//             GetConfigurationDataSynthesize(zpoddInfo->GetConfigurationBuffer,
//                                            zpoddInfo->GetConfigurationBufferSize,
//                                            outputBuffer,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                            inputBuffer->Feature,
//                                            inputBuffer->RequestType,
//                                            DataLength
//                                            );
//         }
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// RequestHandleGetDriveGeometry(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DISK_GET_LENGTH_INFO
//                      IOCTL_DISK_GET_DRIVE_GEOMETRY
//                      IOCTL_DISK_GET_DRIVE_GEOMETRY_EX
//                      IOCTL_CDROM_GET_DRIVE_GEOMETRY
//                      IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX
//                      IOCTL_STORAGE_READ_CAPACITY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     VOID*               outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveOutputBuffer(Request,
//                                             RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                             &outputBuffer,
//                                             NULL);
//
//     // Issue ReadCapacity to update device extension
//     // with information for current media.
//     if (NT_SUCCESS(status))
//     {
//         status = MediaReadCapacity(DeviceExtension->Device);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         switch(RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode)
//         {
//             case IOCTL_DISK_GET_LENGTH_INFO:
//             {
//                 PGET_LENGTH_INFORMATION lengthInfo = (PGET_LENGTH_INFORMATION)outputBuffer;
//
//                 lengthInfo->Length = DeviceExtension->PartitionLength;
//                 *DataLength = sizeof(GET_LENGTH_INFORMATION);
//                 break;
//             }
//             case IOCTL_DISK_GET_DRIVE_GEOMETRY:
//             case IOCTL_CDROM_GET_DRIVE_GEOMETRY:
//             {
//                 PDISK_GEOMETRY geometry = (PDISK_GEOMETRY)outputBuffer;
//
//                 *geometry = DeviceExtension->DiskGeometry;
//                 *DataLength = sizeof(DISK_GEOMETRY);
//                 break;
//             }
//             case IOCTL_DISK_GET_DRIVE_GEOMETRY_EX:
//             case IOCTL_CDROM_GET_DRIVE_GEOMETRY_EX:
//             {
//                 PDISK_GEOMETRY_EX geometryEx = (PDISK_GEOMETRY_EX)outputBuffer;
//
//                 geometryEx->DiskSize = DeviceExtension->PartitionLength;
//                 geometryEx->Geometry = DeviceExtension->DiskGeometry;
//                 *DataLength = FIELD_OFFSET(DISK_GEOMETRY_EX, Data);
//                 break;
//             }
//             case IOCTL_STORAGE_READ_CAPACITY:
//             {
//                 PSTORAGE_READ_CAPACITY readCapacity = (PSTORAGE_READ_CAPACITY)outputBuffer;
//
//                 readCapacity->Version = sizeof(STORAGE_READ_CAPACITY);
//                 readCapacity->Size = sizeof(STORAGE_READ_CAPACITY);
//
//                 readCapacity->BlockLength = DeviceExtension->DiskGeometry.BytesPerSector;
//                 if (readCapacity->BlockLength > 0)
//                 {
//                     readCapacity->NumberOfBlocks.QuadPart = DeviceExtension->PartitionLength.QuadPart/readCapacity->BlockLength;
//                 }
//                 else
//                 {
//                     readCapacity->NumberOfBlocks.QuadPart = 0;
//                 }
//
//                 readCapacity->DiskLength = DeviceExtension->PartitionLength;
//
//                 *DataLength = sizeof(STORAGE_READ_CAPACITY);
//                 break;
//             }
//             default:
//             {
//                 NT_ASSERT(FALSE);
//                 break;
//             }
//         } // end of switch()
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleDiskVerify(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DISK_VERIFY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_DATA         cdData = &(DeviceExtension->DeviceAdditionalData);
//     PVERIFY_INFORMATION verifyInfo = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (!cdData->Mmc.WriteAllowed)
//     {
//         status = STATUS_MEDIA_WRITE_PROTECTED;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &verifyInfo,
//                                                NULL);
//     }
//
//     // handle the request
//     if (NT_SUCCESS(status))
//     {
//         LARGE_INTEGER byteOffset = {0};
//
//         // Add disk offset to starting sector.
//         byteOffset.QuadPart = DeviceExtension->StartingOffset.QuadPart +
//                               verifyInfo->StartingOffset.QuadPart;
//
//         // prevent overflow returning success but only validating small area
//         if (((DeviceExtension->StartingOffset.QuadPart + verifyInfo->StartingOffset.QuadPart) < DeviceExtension->StartingOffset.QuadPart) ||
//             ((verifyInfo->Length >> DeviceExtension->SectorShift) > MAXUSHORT) ||
//             ((byteOffset.QuadPart >> DeviceExtension->SectorShift) > MAXULONG) )
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//         else
//         {
//             ULONG   transferSize = 0;
//             ULONG   timeoutValue = 0;
//             CDB     cdb;
//             ULONG   sectorOffset;
//             USHORT  sectorCount;
//
//             ScratchBuffer_BeginUse(DeviceExtension);
//
//             // Convert byte offset to sector offset.
//             sectorOffset = (ULONG)(byteOffset.QuadPart >> DeviceExtension->SectorShift);
//
//             // Convert ULONG byte count to USHORT sector count.
//             sectorCount = (USHORT)(verifyInfo->Length >> DeviceExtension->SectorShift);
//
//             RtlZeroMemory(&cdb, sizeof(CDB));
//             // Set up the CDB
//             cdb.CDB10.as_mut().OperationCode = SCSIOP_VERIFY;
//
//             // Move little endian values into CDB in big endian format.
//             cdb.CDB10.as_mut().LogicalBlockByte0 = ((PFOUR_BYTE)&sectorOffset)->Byte3;
//             cdb.CDB10.as_mut().LogicalBlockByte1 = ((PFOUR_BYTE)&sectorOffset)->Byte2;
//             cdb.CDB10.as_mut().LogicalBlockByte2 = ((PFOUR_BYTE)&sectorOffset)->Byte1;
//             cdb.CDB10.as_mut().LogicalBlockByte3 = ((PFOUR_BYTE)&sectorOffset)->Byte0;
//
//             cdb.CDB10.as_mut().TransferBlocksMsb = ((PFOUR_BYTE)&sectorCount)->Byte1;
//             cdb.CDB10.as_mut().TransferBlocksLsb = ((PFOUR_BYTE)&sectorCount)->Byte0;
//
//             // The verify command is used by the NT FORMAT utility and
//             // requests are sent down for 5% of the volume size. The
//             // request timeout value is calculated based on the number of
//             // sectors verified.
//             if (sectorCount != 0)
//             {
//                 // sectorCount is a USHORT, so no overflow here...
//                 timeoutValue = TimeOutValueGetCapValue(DeviceExtension->TimeOutValue, ((sectorCount + 128) / 128));
//             }
//
//             status = ScratchBuffer_ExecuteCdbEx(DeviceExtension, Request, transferSize, FALSE, &cdb, 10, timeoutValue);
//
//             // nothing to do after the command finishes.
//             ScratchBuffer_EndUse(DeviceExtension);
//         }
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleCheckVerify(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_CHECK_VERIFY2
//                      IOCTL_STORAGE_CHECK_VERIFY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = 0;
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.CDB6GENERIC.as_mut().OperationCode = SCSIOP_TEST_UNIT_READY;
//
//         status = ScratchBuffer_ExecuteCdbEx(DeviceExtension, Request, transferSize, FALSE, &cdb, 6, CDROM_TEST_UNIT_READY_TIMEOUT);
//
//         if (NT_SUCCESS(status))
//         {
//             if((RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_STORAGE_CHECK_VERIFY) &&
//                (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength))
//             {
//                 PULONG outputBuffer = NULL;
//                 status = WdfRequestRetrieveOutputBuffer(Request,
//                                                         RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                         &outputBuffer,
//                                                         NULL);
//
//                 if (outputBuffer != NULL)
//                 {
//                     *outputBuffer = DeviceExtension->MediaChangeCount;
//                     *DataLength = sizeof(ULONG);
//                 }
//             }
//             else
//             {
//                 *DataLength = 0;
//             }
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleFakePartitionInfo(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DISK_GET_DRIVE_LAYOUT
//                      IOCTL_DISK_GET_DRIVE_LAYOUT_EX
//                      IOCTL_DISK_GET_PARTITION_INFO
//                      IOCTL_DISK_GET_PARTITION_INFO_EX
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     VOID*       outputBuffer = NULL;
//     ULONG       ioctl = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         if ((ioctl != IOCTL_DISK_GET_DRIVE_LAYOUT) &&
//             (ioctl != IOCTL_DISK_GET_DRIVE_LAYOUT_EX) &&
//             (ioctl != IOCTL_DISK_GET_PARTITION_INFO) &&
//             (ioctl != IOCTL_DISK_GET_PARTITION_INFO_EX))
//         {
//             status = STATUS_INTERNAL_ERROR;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     // handle the request
//     if (NT_SUCCESS(status))
//     {
//         switch (ioctl)
//         {
//         case IOCTL_DISK_GET_DRIVE_LAYOUT:
//             *DataLength = FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION, PartitionEntry[1]);
//             RtlZeroMemory(outputBuffer, *DataLength);
//             break;
//         case IOCTL_DISK_GET_DRIVE_LAYOUT_EX:
//             *DataLength = FIELD_OFFSET(DRIVE_LAYOUT_INFORMATION_EX, PartitionEntry[1]);
//             RtlZeroMemory(outputBuffer, *DataLength);
//             break;
//         case IOCTL_DISK_GET_PARTITION_INFO:
//             *DataLength = sizeof(PARTITION_INFORMATION);
//             RtlZeroMemory(outputBuffer, *DataLength);
//             break;
//         case IOCTL_DISK_GET_PARTITION_INFO_EX:
//             *DataLength = sizeof(PARTITION_INFORMATION_EX);
//             RtlZeroMemory(outputBuffer, *DataLength);
//             break;
//         default:
//             NT_ASSERT(!"Invalid ioctl should not have reached this point\n");
//             break;
//         }
//
//         // if we are getting the drive layout, then we need to start by
//         // adding some of the non-partition stuff that says we have
//         // exactly one partition available.
//         if (ioctl == IOCTL_DISK_GET_DRIVE_LAYOUT)
//         {
//             PDRIVE_LAYOUT_INFORMATION layout;
//             layout = (PDRIVE_LAYOUT_INFORMATION)outputBuffer;
//             layout->PartitionCount = 1;
//             layout->Signature = 1;
//             outputBuffer = (PVOID)(layout->PartitionEntry);
//             ioctl = IOCTL_DISK_GET_PARTITION_INFO;
//         }
//         else if (ioctl == IOCTL_DISK_GET_DRIVE_LAYOUT_EX)
//         {
//             PDRIVE_LAYOUT_INFORMATION_EX layoutEx;
//             layoutEx = (PDRIVE_LAYOUT_INFORMATION_EX)outputBuffer;
//             layoutEx->PartitionStyle = PARTITION_STYLE_MBR;
//             layoutEx->PartitionCount = 1;
//             layoutEx->Mbr.Signature = 1;
//             outputBuffer = (PVOID)(layoutEx->PartitionEntry);
//             ioctl = IOCTL_DISK_GET_PARTITION_INFO_EX;
//         }
//
//         // NOTE: the local var 'ioctl' is now modified to either EX or
//         // non-EX version. the local var 'systemBuffer' is now pointing
//         // to the partition information structure.
//         if (ioctl == IOCTL_DISK_GET_PARTITION_INFO)
//         {
//             PPARTITION_INFORMATION partitionInfo;
//             partitionInfo = (PPARTITION_INFORMATION)outputBuffer;
//             partitionInfo->RewritePartition = FALSE;
//             partitionInfo->RecognizedPartition = TRUE;
//             partitionInfo->PartitionType = PARTITION_FAT32;
//             partitionInfo->BootIndicator = FALSE;
//             partitionInfo->HiddenSectors = 0;
//             partitionInfo->StartingOffset.QuadPart = 0;
//             partitionInfo->PartitionLength = DeviceExtension->PartitionLength;
//             partitionInfo->PartitionNumber = 0;
//         }
//         else
//         {
//             PPARTITION_INFORMATION_EX partitionInfo;
//             partitionInfo = (PPARTITION_INFORMATION_EX)outputBuffer;
//             partitionInfo->PartitionStyle = PARTITION_STYLE_MBR;
//             partitionInfo->RewritePartition = FALSE;
//             partitionInfo->Mbr.RecognizedPartition = TRUE;
//             partitionInfo->Mbr.PartitionType = PARTITION_FAT32;
//             partitionInfo->Mbr.BootIndicator = FALSE;
//             partitionInfo->Mbr.HiddenSectors = 0;
//             partitionInfo->StartingOffset.QuadPart = 0;
//             partitionInfo->PartitionLength = DeviceExtension->PartitionLength;
//             partitionInfo->PartitionNumber = 0;
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleGetDeviceNumber(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_GET_DEVICE_NUMBER
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >=
//        sizeof(STORAGE_DEVICE_NUMBER))
//     {
//         PSTORAGE_DEVICE_NUMBER deviceNumber = NULL;
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &deviceNumber,
//                                                 NULL);
//         if (NT_SUCCESS(status))
//         {
//             deviceNumber->DeviceType = DeviceExtension->DeviceObject->DeviceType;
//             deviceNumber->DeviceNumber = DeviceExtension->DeviceNumber;
//             deviceNumber->PartitionNumber = (ULONG)-1; // legacy reason, return (-1) for this IOCTL.
//
//             status = STATUS_SUCCESS;
//             *DataLength = sizeof(STORAGE_DEVICE_NUMBER);
//         }
//     }
//     else
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(STORAGE_DEVICE_NUMBER);
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleGetHotPlugInfo(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_GET_HOTPLUG_INFO
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     *DataLength = 0;
//
//     if(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >=
//        sizeof(STORAGE_HOTPLUG_INFO))
//     {
//         PSTORAGE_HOTPLUG_INFO info = NULL;
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &info,
//                                                 NULL);
//         if (NT_SUCCESS(status))
//         {
//             *info = DeviceExtension->PrivateFdoData->HotplugInfo;
//
//             status = STATUS_SUCCESS;
//             *DataLength = sizeof(STORAGE_HOTPLUG_INFO);
//         }
//     }
//     else
//     {
//         status = STATUS_BUFFER_TOO_SMALL;
//         *DataLength = sizeof(STORAGE_HOTPLUG_INFO);
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleSetHotPlugInfo(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_SET_HOTPLUG_INFO
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PSTORAGE_HOTPLUG_INFO   info = NULL;
//
//     *DataLength = 0;
//
//     if (RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//         sizeof(STORAGE_HOTPLUG_INFO))
//     {
//         // Indicate unsuccessful status and no data transferred.
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &info,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (info->Size != DeviceExtension->PrivateFdoData->HotplugInfo.Size)
//         {
//             status = STATUS_INVALID_PARAMETER_1;
//         }
//
//         if (info->MediaRemovable != DeviceExtension->PrivateFdoData->HotplugInfo.MediaRemovable)
//         {
//             status = STATUS_INVALID_PARAMETER_2;
//         }
//
//         if (info->MediaHotplug != DeviceExtension->PrivateFdoData->HotplugInfo.MediaHotplug)
//         {
//             status = STATUS_INVALID_PARAMETER_3;
//         }
//
//         if (info->WriteCacheEnableOverride != DeviceExtension->PrivateFdoData->HotplugInfo.WriteCacheEnableOverride)
//         {
//             status = STATUS_INVALID_PARAMETER_5;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         DeviceExtension->PrivateFdoData->HotplugInfo.DeviceHotplug = info->DeviceHotplug;
//
//         // Store the user-defined override in the registry
//         DeviceSetParameter(DeviceExtension,
//                            CLASSP_REG_SUBKEY_NAME,
//                            CLASSP_REG_REMOVAL_POLICY_VALUE_NAME,
//                            (info->DeviceHotplug) ? RemovalPolicyExpectSurpriseRemoval : RemovalPolicyExpectOrderlyRemoval);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleEventNotification(
//           PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       PWDF_REQUEST_PARAMETERS  RequestParameters,
//          size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     This routine handles the process of IOCTL_STORAGE_EVENT_NOTIFICATION
//
// Arguments:
//
//     DeviceExtension - device context
//
//     Request - request to be handled
//
//     RequestParameters - request parameter
//
//     DataLength - data transferred
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                     status = STATUS_SUCCESS;
//     PMEDIA_CHANGE_DETECTION_INFO info = NULL;
//     LONG                         requestInUse;
//     PSTORAGE_EVENT_NOTIFICATION  eventBuffer = NULL;
//
//     *DataLength = 0;
//
//     info = DeviceExtension->MediaChangeDetectionInfo;
//
//     // Since AN is ASYNCHRONOUS and can happen at any time,
//     // make certain not to do anything before properly initialized.
//     if ((!DeviceExtension->IsInitialized) || (info == NULL))
//     {
//         status = STATUS_UNSUCCESSFUL;
//     }
//
//     if (NT_SUCCESS(status) && (Request != NULL) && (RequestParameters != NULL)) {
//
//         //
//         // Validate IOCTL parameters
//         //
//         if (RequestParameters->Parameters.DeviceIoControl.as_ref().InputBufferLength <
//             sizeof(STORAGE_EVENT_NOTIFICATION)) {
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//
//         //
//         // Check for an supported event
//         //
//         if (NT_SUCCESS(status)) {
//             status = WdfRequestRetrieveInputBuffer(Request,
//                                                    RequestParameters->Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                    &eventBuffer,
//                                                    NULL);
//             if (NT_SUCCESS(status)) {
//                 if ((eventBuffer->Version != STORAGE_EVENT_NOTIFICATION_VERSION_V1) ||
//                     (eventBuffer->Size != sizeof(STORAGE_EVENT_NOTIFICATION))) {
//                     status = STATUS_INVALID_PARAMETER;
//                 } else if ((eventBuffer->Events &
//                            (STORAGE_EVENT_MEDIA_STATUS | STORAGE_EVENT_DEVICE_STATUS | STORAGE_EVENT_DEVICE_OPERATION)) == 0) {
//                     status = STATUS_NOT_SUPPORTED;
//                 }
//             }
//         }
//
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (info->MediaChangeDetectionDisableCount != 0)
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                        "RequestHandleEventNotification: device %p has detection disabled \n",
//                        DeviceExtension->DeviceObject));
//             status = STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // if the request is not in use, mark it as such.
//         requestInUse = InterlockedCompareExchange((PLONG)&info->MediaChangeRequestInUse, 1, 0);
//
//         if (requestInUse != 0)
//         {
//             status = STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // The last MCN finished. ok to issue the new one.
//         RequestSetupMcnSyncIrp(DeviceExtension);
//
//         // The irp will go into KMDF framework and a request will be created there to represent it.
//         IoCallDriver(DeviceExtension->DeviceObject, info->MediaChangeSyncIrp);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// RequestHandleEjectionControl(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_MEDIA_REMOVAL
//                      IOCTL_STORAGE_EJECTION_CONTROL
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PPREVENT_MEDIA_REMOVAL  mediaRemoval = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if(RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//        sizeof(PREVENT_MEDIA_REMOVAL))
//     {
//         status = STATUS_INFO_LENGTH_MISMATCH;
//     }
//     else
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                &mediaRemoval,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = PerformEjectionControl(DeviceExtension,
//                                         Request,
//                                         ((RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_STORAGE_EJECTION_CONTROL)
//                                           ? SecureMediaLock
//                                           : SimpleMediaLock),
//                                         mediaRemoval->PreventMediaRemoval);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleEnableStreaming(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Handles an IOCTL_CDROM_ENABLE_STREAMING request
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     DataLength - transferred data length
//
// Notes:
//
//     This IOCTL is serialized because it changes read/write
//     behavior and we want to make sure that all previous
//     reads/writes have been completed before we change the
//     behavior.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     PCDROM_DATA                 cdData = &(DeviceExtension->DeviceAdditionalData);
//     PCDROM_PRIVATE_FDO_DATA     fdoData = DeviceExtension->PrivateFdoData;
//
//     WDFFILEOBJECT               fileObject = NULL;
//     PFILE_OBJECT_CONTEXT        fileObjectContext = NULL;
//
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_STREAMING_CONTROL    inputBuffer = NULL;
//
//     BOOLEAN                     enforceStreamingRead = FALSE;
//     BOOLEAN                     enforceStreamingWrite = FALSE;
//     BOOLEAN                     streamingReadSupported, streamingWriteSupported;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            sizeof(CDROM_STREAMING_CONTROL),
//                                            &inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         // get file object context
//         fileObject = WdfRequestGetFileObject(Request);
//         if (fileObject != NULL) {
//             fileObjectContext = FileObjectGetContext(fileObject);
//         }
//         NT_ASSERT(fileObjectContext != NULL);
//
//         if (fileObjectContext == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestHandleEnableStreaming: cannot find file object context\n"));
//             status = STATUS_INVALID_HANDLE;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (inputBuffer->RequestType == CdromStreamingDisable)
//         {
//             enforceStreamingRead = FALSE;
//             enforceStreamingWrite = FALSE;
//         }
//         else if (inputBuffer->RequestType == CdromStreamingEnableForReadOnly)
//         {
//             enforceStreamingRead = TRUE;
//             enforceStreamingWrite = FALSE;
//         }
//         else if (inputBuffer->RequestType == CdromStreamingEnableForWriteOnly)
//         {
//             enforceStreamingRead = FALSE;
//             enforceStreamingWrite = TRUE;
//         }
//         else if (inputBuffer->RequestType == CdromStreamingEnableForReadWrite)
//         {
//             enforceStreamingRead = TRUE;
//             enforceStreamingWrite = TRUE;
//         }
//
//         streamingReadSupported = cdData->Mmc.StreamingReadSupported && !TEST_FLAG(fdoData->HackFlags, FDO_HACK_NO_STREAMING);
//         streamingWriteSupported = cdData->Mmc.StreamingWriteSupported && !TEST_FLAG(fdoData->HackFlags, FDO_HACK_NO_STREAMING);
//         if ((enforceStreamingRead && !streamingReadSupported) ||
//             (enforceStreamingWrite && !streamingWriteSupported))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                         "RequestHandleEnableStreaming: requested Streaming mode is not supported\n"));
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//         else
//         {
//             fileObjectContext->EnforceStreamingRead = enforceStreamingRead;
//             fileObjectContext->EnforceStreamingWrite = enforceStreamingWrite;
//         }
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleSendOpcInformation(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Handles an IOCTL_CDROM_SEND_OPC_INFORMATION request
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_SIMPLE_OPC_INFO  inputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            sizeof(CDROM_SIMPLE_OPC_INFO),
//                                            (PVOID*)&inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         CDB                         cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//
//         // we support only the simplest version for now
//         cdb.SEND_OPC_INFORMATION.as_mut().OperationCode = SCSIOP_SEND_OPC_INFORMATION;
//         cdb.SEND_OPC_INFORMATION.as_mut().DoOpc = 1;
//         cdb.SEND_OPC_INFORMATION.as_mut().Exclude0 = (inputBuffer->Exclude0 ? 1 : 0);
//         cdb.SEND_OPC_INFORMATION.as_mut().Exclude1 = (inputBuffer->Exclude1 ? 1 : 0);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, 0, FALSE, &cdb, sizeof(cdb.SEND_OPC_INFORMATION));
//
//         // nothing to do after the command finishes
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleGetPerformance(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Handles an IOCTL_CDROM_GET_PERFORMANCE request
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_PERFORMANCE_REQUEST  inputBuffer = NULL;
//     PVOID                       outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     // Retrieve pointers to input/output data. The size has been validated earlier
//     // in RequestValidateGetPerformance, so we do not check it again.
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                (PVOID*)&inputBuffer,
//                                                NULL);
//     }
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         USHORT                      descriptorSize = 0;
//         USHORT                      descriptorCount = 0;
//         size_t                      transferSize = 0;
//         CDB                         cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         // Set up the CDB
//         RtlZeroMemory(&cdb, sizeof(CDB));
//
//         if (inputBuffer->RequestType == CdromPerformanceRequest)
//         {
//             cdb.GET_PERFORMANCE.as_mut().Type = 0;
//
//             // 10b is the only defined tolerance in MMCr6
//             cdb.GET_PERFORMANCE.as_mut().Tolerance = 2;
//
//             switch (inputBuffer->Exceptions) {
//                 case CdromNominalPerformance:
//                     cdb.GET_PERFORMANCE.as_mut().Except = 0;
//                     descriptorSize = sizeof(CDROM_NOMINAL_PERFORMANCE_DESCRIPTOR);
//                     break;
//                 case CdromEntirePerformanceList:
//                     cdb.GET_PERFORMANCE.as_mut().Except = 1;
//                     descriptorSize = sizeof(CDROM_NOMINAL_PERFORMANCE_DESCRIPTOR);
//                     break;
//                 case CdromPerformanceExceptionsOnly:
//                     cdb.GET_PERFORMANCE.as_mut().Except = 2;
//                     descriptorSize = sizeof(CDROM_EXCEPTION_PERFORMANCE_DESCRIPTOR);
//                     break;
//             }
//
//             switch (inputBuffer->PerformanceType) {
//                 case CdromReadPerformance:
//                     cdb.GET_PERFORMANCE.as_mut().Write = 0; break;
//                 case CdromWritePerformance:
//                     cdb.GET_PERFORMANCE.as_mut().Write = 1; break;
//             }
//
//             REVERSE_BYTES(&cdb.GET_PERFORMANCE.as_ref().StartingLBA, &inputBuffer->StaringLba);
//         }
//         else if (inputBuffer->RequestType == CdromWriteSpeedRequest)
//         {
//             cdb.GET_PERFORMANCE.as_mut().Type = 3;
//             descriptorSize = sizeof(CDROM_WRITE_SPEED_DESCRIPTOR);
//         }
//
//         cdb.GET_PERFORMANCE.as_mut().OperationCode = SCSIOP_GET_PERFORMANCE;
//
//         // calculate how many descriptors can fit into the output buffer
//         if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength >=
//                 sizeof(CDROM_PERFORMANCE_HEADER) &&
//             RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength <=
//                 MAXUSHORT &&
//             descriptorSize > 0)
//         {
//             descriptorCount = (USHORT)(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength - sizeof(CDROM_PERFORMANCE_HEADER));
//             descriptorCount /= descriptorSize;
//         }
//         else
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//
//         REVERSE_BYTES_SHORT(&cdb.GET_PERFORMANCE.as_ref().MaximumNumberOfDescriptors, &descriptorCount);
//
//         // Calculate transfer size. We round it up to meet adapter requirements.
//         // Extra bytes are discarded later, when we copy data to the output buffer.
//         transferSize = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//         transferSize += DeviceExtension->AdapterDescriptor->AlignmentMask;
//         transferSize &= ~DeviceExtension->AdapterDescriptor->AlignmentMask;
//
//         if (NT_SUCCESS(status))
//         {
//             status = ScratchBuffer_ExecuteCdbEx(DeviceExtension, Request, (ULONG)transferSize, TRUE, &cdb, sizeof(cdb.GET_PERFORMANCE), CDROM_GET_PERFORMANCE_TIMEOUT);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             if (DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength < sizeof(CDROM_PERFORMANCE_HEADER))
//             {
//                 *DataLength = 0;
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//             else
//             {
//                 *DataLength = min(RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                  DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength);
//                 RtlCopyMemory(outputBuffer,
//                               DeviceExtension->ScratchContext.ScratchBuffer,
//                               *DataLength);
//             }
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleMcnSyncFakeIoctl(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     Handles an IOCTL_MCN_SYNC_FAKE_IOCTL request
//
// Arguments:
//
//     DeviceExtension - device context
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PMEDIA_CHANGE_DETECTION_INFO    info = DeviceExtension->MediaChangeDetectionInfo;
//     BOOLEAN                         shouldRetry = TRUE;
//     BOOLEAN                         requestSent = FALSE;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     //
//     // Try to acquire the media change event.  If we can't do it immediately
//     // then bail out and assume the caller will try again later.
//     //
//     while (shouldRetry)
//     {
//
//         status = RequestSetupMcnRequest(DeviceExtension,
//                                         info->Gesn.Supported);
//
//         if ((!NT_SUCCESS(status)))
//         {
//             shouldRetry = FALSE;
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             requestSent = RequestSendMcnRequest(DeviceExtension);
//
//             if (requestSent)
//             {
//                 shouldRetry = RequestPostWorkMcnRequest(DeviceExtension);
//             }
//             else
//             {
//                 shouldRetry = FALSE;
//             }
//         }
//     }
//
//     // If there were any media change notifications that were not delivered
//     // for some reason, make an attempt to do so at this time.
//     DeviceSendDelayedMediaChangeNotifications(DeviceExtension);
//
//     // Set the status and then complete the original request.
//     // The timer handler will be able to send the next request.
//     status = STATUS_SUCCESS;
//
//     return status;
// }
//
// BOOLEAN
// RequestIsRealtimeStreaming(
//       WDFREQUEST       Request,
//       BOOLEAN          IsReadRequest
//     )
// /*++
//
// Routine Description:
//
//     Checks whether a given read/write request should
//     be performed in Real-Time Streaming mode.
//
// Arguments:
//
//     Request - request to be checked
//     IsReadRequest - TRUE = read request; FALSE = write request
//
// Return Value:
//
//     TRUE  - a Real-Time Streaming operation has to be performed
//     FALSE - a normal (non-Streaming) operation has to be performed
//
// --*/
// {
//     BOOLEAN     useStreaming = FALSE;
//
//     if (!useStreaming) {
//         //
//         // Check if we're required to use Streaming via I/O Stack Location flags
//         //
//         UCHAR currentStackLocationFlags = 0;
//         currentStackLocationFlags = RequestGetCurrentStackLocationFlags(Request);
//
//         useStreaming = TEST_FLAG(currentStackLocationFlags, SL_REALTIME_STREAM);
//     }
//
//     if (!useStreaming) {
//         //
//         // Check if we were previously requested to enforce Streaming for
//         // the file handle through which this request was sent.
//         //
//
//         WDFFILEOBJECT           fileObject;
//         PFILE_OBJECT_CONTEXT    fileObjectContext;
//
//         fileObject = WdfRequestGetFileObject(Request);
//
//         if (fileObject != NULL) {
//             fileObjectContext = FileObjectGetContext(fileObject);
//             NT_ASSERT(fileObjectContext != NULL);
//
//             if (IsReadRequest && fileObjectContext->EnforceStreamingRead)
//             {
//                 useStreaming = TRUE;
//             }
//
//             if (!IsReadRequest && fileObjectContext->EnforceStreamingWrite)
//             {
//                 useStreaming = TRUE;
//             }
//         }
//     }
//
//     return useStreaming;
// }
//
//
// NTSTATUS
// RequestValidateReadWrite(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters
//     )
// /*++
//
// Routine Description:
//
//    Validate Read/Write request
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_DATA         cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     BOOLEAN             isValid = TRUE;
//     LONGLONG            startingOffset = 0;
//     size_t              transferByteCount = 0;
//     PIRP                irp = NULL;
//     PIO_STACK_LOCATION  currentStack = NULL;
//
//     irp = WdfRequestWdmGetIrp(Request);
//     currentStack = IoGetCurrentIrpStackLocation(irp);
//
//     if (TEST_FLAG(DeviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME) &&
//         (currentStack->MinorFunction != CDROM_VOLUME_VERIFY_CHECKED) &&
//         !TEST_FLAG(currentStack->Flags, SL_OVERRIDE_VERIFY_VOLUME))
//     {
//         //  DO_VERIFY_VOLUME is set for the device object,
//         //  but this request is not itself a verify request.
//         //  So fail this request.
//
//         //set the status for volume verification.
//         status = STATUS_VERIFY_REQUIRED;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (PLAY_ACTIVE(DeviceExtension))
//         {
//             status = STATUS_DEVICE_BUSY;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // If the device is in exclusive mode, check whether the request is from
//         // the handle that locked the device.
//         if (EXCLUSIVE_MODE(cdData) && !EXCLUSIVE_OWNER(cdData, WdfRequestGetFileObject(Request)))
//         {
//             // This request is not from the owner. We can't let the operation go.
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW, "RequestValidateReadWrite: Access Denied! Device in exclusive mode.\n"));
//
//             status = STATUS_ACCESS_DENIED;
//         }
//     }
//
//     // Validate the request alignment.
//     if (NT_SUCCESS(status))
//     {
//         if (RequestParameters.Type == WdfRequestTypeRead)
//         {
//             startingOffset = RequestParameters.Parameters.Read.as_ref().DeviceOffset;
//             transferByteCount = RequestParameters.Parameters.Read.as_ref().Length;
//         }
//         else
//         {
//             startingOffset = RequestParameters.Parameters.Write.as_ref().DeviceOffset;
//             transferByteCount = RequestParameters.Parameters.Write.as_ref().Length;
//         }
//
//         if (!DeviceExtension->DiskGeometry.BytesPerSector)
//         {
//             DeviceExtension->DiskGeometry.BytesPerSector = 2048;
//         }
//
//         if (!DeviceExtension->SectorShift)
//         {
//             DeviceExtension->SectorShift = 11;
//         }
//
//         // Perform some basic validation up front
//         if (TEST_FLAG(startingOffset, DeviceExtension->DiskGeometry.BytesPerSector - 1) ||
//             TEST_FLAG(transferByteCount,       DeviceExtension->DiskGeometry.BytesPerSector - 1))
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     // validate the request against the current mmc schema
//     if (NT_SUCCESS(status))
//     {
//         FEATURE_NUMBER schema = cdData->Mmc.ValidationSchema;
//
//         // We validate read requests according to the RandomWritable schema, except in the
//         // case of IncrementalStreamingWritable, wherein  the drive is responsible for all
//         // of the verification
//         if (RequestParameters.Type == WdfRequestTypeRead)
//         {
//             if (!cdData->Mmc.WriteAllowed)
//             {
//                 // standard legacy validation of read irps
//                 // if writing is not allowed on the media
//                 schema =  FeatureRandomWritable;
//             }
//             else if (schema != FeatureIncrementalStreamingWritable)
//             {
//                 // standard legacy validation of read irps
//                 // if not using streaming writes on writable media
//                 schema =  FeatureRandomWritable;
//             }
//         }
//
//         // Fail write requests to read-only media
//         if ((RequestParameters.Type == WdfRequestTypeWrite) &&
//             !(cdData->Mmc.WriteAllowed))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,  "RequestValidateReadWrite: Write request to read-only media\n"));
//             isValid = FALSE;
//         }
//
//         if (isValid)
//         {
//             switch (schema)
//             {
//                 case FeatureDefectManagement:
//                 case FeatureRandomWritable:
//                     // Ensure that the request is within bounds for ROM drives.
//                     // Writer drives do not need to have bounds as outbounds request should not damage the drive.
//                     if(!cdData->Mmc.IsWriter)
//                     {
//                         if ((startingOffset >= DeviceExtension->PartitionLength.QuadPart) ||
//                             startingOffset < 0)
//                         {
//                             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,  "RequestValidateReadWrite: Request is out of bounds\n"));
//                             isValid = FALSE;
//
//                         }
//                         else
//                         {
//                             ULONGLONG bytesRemaining = DeviceExtension->PartitionLength.QuadPart - startingOffset;
//
//                             if ((ULONGLONG)transferByteCount > bytesRemaining)
//                             {
//                                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,  "RequestValidateReadWrite: Request is out of bounds\n"));
//                                 isValid = FALSE;
//                             }
//                         }
//                     }
//                     break;
//
//                 case FeatureRigidRestrictedOverwrite:
//                     // Ensure that the number of blocks is a multiple of the blocking size
//                     if (((transferByteCount >> DeviceExtension->SectorShift) % cdData->Mmc.Blocking) != 0)
//                     {
//                         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,
//                                    "RequestValidateReadWrite: Number of blocks is not a multiple of the blocking size (%x)\n",
//                                    cdData->Mmc.Blocking));
//
//                         isValid = FALSE;
//                     }
//                     // Fall through
//                 case FeatureRestrictedOverwrite:
//                     // Ensure that the request begins on a blocking boundary
//                     if ((Int64ShrlMod32(startingOffset, DeviceExtension->SectorShift) % cdData->Mmc.Blocking) != 0)
//                     {
//                         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,
//                                    "RequestValidateReadWrite: Starting block is not a multiple of the blocking size (%x)\n",
//                                    cdData->Mmc.Blocking));
//
//                         isValid = FALSE;
//                     }
//                     break;
//
//                 case FeatureIncrementalStreamingWritable:
//                     // Let the drive handle the verification
//                     break;
//
//                 default:
//                     // Unknown schema. Fail the request
//                     TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,
//                                "RequestValidateReadWrite: Unknown validation schema (%x)\n",
//                                schema));
//
//                     isValid = FALSE;
//                     break;
//             } //end of switch (schema)
//         } // end of if (isValid)
//
//         if (!isValid)
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     } // end of mmc schema validation
//
//     // validate that the Real-Time Streaming requests meet device capabilties
//     if (NT_SUCCESS(status))
//     {
//         // We do not check for FDO_HACK_NO_STREAMING in DeviceExtension->PrivateFdoData->HackFlags here,
//         // because we're going to hide device failures related to streaming reads/writes from the sender
//         // of the request. FDO_HACK_NO_STREAMING is going to be taken into account later during actual
//         // processing of the request.
//         if (RequestIsRealtimeStreaming(Request, RequestParameters.Type == WdfRequestTypeRead))
//         {
//             if (RequestParameters.Type == WdfRequestTypeRead && !cdData->Mmc.StreamingReadSupported)
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,
//                            "RequestValidateReadWrite: Streaming reads are not supported.\n"));
//
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//             if (RequestParameters.Type == WdfRequestTypeWrite && !cdData->Mmc.StreamingWriteSupported)
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_RW,
//                            "RequestValidateReadWrite: Streaming writes are not supported.\n"));
//
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//         }
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestHandleReadWrite(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters
//     )
// /*++
//
// Routine Description:
//
//     Handle a read/write request
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_DATA         cdData = &(DeviceExtension->DeviceAdditionalData);
//
//     size_t              transferByteCount = 0;
//     PIRP                irp = NULL;
//     PIO_STACK_LOCATION  currentStack = NULL;
//
//     PUCHAR              dataBuffer;
//
//
//     irp = WdfRequestWdmGetIrp(Request);
//     currentStack = IoGetCurrentIrpStackLocation(irp);
//     dataBuffer = MmGetMdlVirtualAddress(irp->MdlAddress);
//
//     if (NT_SUCCESS(status))
//     {
//         if (RequestParameters.Type == WdfRequestTypeRead)
//         {
//             transferByteCount = RequestParameters.Parameters.Read.as_ref().Length;
//         }
//         else
//         {
//             transferByteCount = RequestParameters.Parameters.Write.as_ref().Length;
//         }
//
//         if (transferByteCount == 0)
//         {
//             //  Several parts of the code turn 0 into 0xffffffff,
//             //  so don't process a zero-length request any further.
//             status = STATUS_SUCCESS;
//             RequestCompletion(DeviceExtension, Request, status, 0);
//             return status;
//         }
//
//         // Add partition byte offset to make starting byte relative to
//         // beginning of disk.
//         currentStack->Parameters.Read.ByteOffset.QuadPart += (DeviceExtension->StartingOffset.QuadPart);
//
//         //not very necessary as the starting offset for CD/DVD device is always 0.
//         if (RequestParameters.Type == WdfRequestTypeRead)
//         {
//             RequestParameters.Parameters.Read.as_mut().DeviceOffset = currentStack->Parameters.Read.ByteOffset.QuadPart;
//         }
//         else
//         {
//             RequestParameters.Parameters.Write.as_mut().DeviceOffset = currentStack->Parameters.Write.ByteOffset.QuadPart;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   entireXferLen = currentStack->Parameters.Read.as_ref().Length;
//         ULONG   maxLength = 0;
//         ULONG   packetsCount = 0;
//
//         PCDROM_SCRATCH_READ_WRITE_CONTEXT readWriteContext;
//         PCDROM_REQUEST_CONTEXT            requestContext;
//         PCDROM_REQUEST_CONTEXT            originalRequestContext;
//
//         // get the count of packets we need to send.
//         if ((((ULONG_PTR)dataBuffer) & (PAGE_SIZE-1)) == 0)
//         {
//             maxLength = cdData->MaxPageAlignedTransferBytes;
//         }
//         else
//         {
//             maxLength = cdData->MaxUnalignedTransferBytes;
//         }
//
//         packetsCount = entireXferLen / maxLength;
//
//         if (entireXferLen % maxLength != 0)
//         {
//             packetsCount++;
//         }
//
//         originalRequestContext = RequestGetContext(Request);
//
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         readWriteContext = &DeviceExtension->ScratchContext.ScratchReadWriteContext;
//         requestContext = RequestGetContext(DeviceExtension->ScratchContext.ScratchRequest);
//
//         readWriteContext->PacketsCount = packetsCount;
//         readWriteContext->EntireXferLen = entireXferLen;
//         readWriteContext->MaxLength = maxLength;
//         readWriteContext->StartingOffset = currentStack->Parameters.Read.ByteOffset;
//         readWriteContext->DataBuffer = dataBuffer;
//         readWriteContext->TransferedBytes = 0;
//         readWriteContext->IsRead = (RequestParameters.Type == WdfRequestTypeRead);
//
//         requestContext->OriginalRequest = Request;
//         requestContext->DeviceExtension = DeviceExtension;
//
//         //
//         // Setup the READ/WRITE fields in the original request which is what
//         // we use to properly synchronize cancellation logic between the
//         // cancel callback and the timer routine.
//         //
//
//         originalRequestContext->ReadWriteIsCompleted = FALSE;
//         originalRequestContext->ReadWriteRetryInitialized = FALSE;
//         originalRequestContext->DeviceExtension = DeviceExtension;
//
//         status = ScratchBuffer_PerformNextReadWrite(DeviceExtension, TRUE);
//
//         // We do not call ScratchBuffer_EndUse here, because we're not releasing the scratch SRB.
//         // It will be released in the completion routine.
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// RequestHandleLoadEjectMedia(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_EJECT_MEDIA
//                      IOCTL_STORAGE_LOAD_MEDIA
//                      IOCTL_STORAGE_LOAD_MEDIA2
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PZERO_POWER_ODD_INFO        zpoddInfo = DeviceExtension->ZeroPowerODDInfo;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         // Synchronize with ejection control and ejection cleanup code as
//         // well as other eject/load requests.
//         WdfWaitLockAcquire(DeviceExtension->EjectSynchronizationLock, NULL);
//
//         if(DeviceExtension->ProtectedLockCount != 0)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,  "RequestHandleLoadEjectMedia: call to eject protected locked "
//                                                                 "device - failure\n"));
//             status = STATUS_DEVICE_BUSY;
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             SCSI_REQUEST_BLOCK  srb;
//             PCDB                cdb = (PCDB)srb.Cdb;
//
//             RtlZeroMemory(&srb, sizeof(SCSI_REQUEST_BLOCK));
//
//             srb.CdbLength = 6;
//
//             cdb->START_STOP.OperationCode = SCSIOP_START_STOP_UNIT;
//             cdb->START_STOP.LoadEject = 1;
//
//             if(RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_STORAGE_EJECT_MEDIA)
//             {
//                 cdb->START_STOP.Start = 0;
//
//                 // We are sending down a soft eject, and in this case we should take an active ref
//                 // if the command succeeds.
//                 if ((zpoddInfo != NULL) &&
//                     (zpoddInfo->LoadingMechanism == LOADING_MECHANISM_TRAY) &&
//                     (zpoddInfo->Load == 0))                                         // Drawer
//                 {
//                     zpoddInfo->MonitorStartStopUnit = TRUE;
//                 }
//             }
//             else
//             {
//                 cdb->START_STOP.Start = 1;
//             }
//
//             status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                                 &srb,
//                                                 NULL,
//                                                 0,
//                                                 FALSE,
//                                                 Request);
//
//             if (zpoddInfo != NULL)
//             {
//                 zpoddInfo->MonitorStartStopUnit = FALSE;
//             }
//         }
//
//         WdfWaitLockRelease(DeviceExtension->EjectSynchronizationLock);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// RequestHandleReserveRelease(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_RESERVE
//                      IOCTL_STORAGE_RELEASE
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     PCDB                cdb = NULL;
//     ULONG               ioctlCode = 0;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                          sizeof(SCSI_REQUEST_BLOCK) +
//                          (sizeof(ULONG_PTR) * 2),
//                          CDROM_TAG_SRB);
//
//     if (srb == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         ioctlCode = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
//         cdb = (PCDB)srb->Cdb;
//         RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         if (TEST_FLAG(DeviceExtension->PrivateFdoData->HackFlags, FDO_HACK_NO_RESERVE6))
//         {
//             srb->CdbLength = 10;
//             cdb->CDB10.OperationCode = (ioctlCode == IOCTL_STORAGE_RESERVE) ? SCSIOP_RESERVE_UNIT10 : SCSIOP_RELEASE_UNIT10;
//         }
//         else
//         {
//             srb->CdbLength = 6;
//             cdb->CDB6GENERIC.OperationCode = (ioctlCode == IOCTL_STORAGE_RESERVE) ? SCSIOP_RESERVE_UNIT : SCSIOP_RELEASE_UNIT;
//         }
//
//         // Set timeout value.
//         srb->TimeOutValue = DeviceExtension->TimeOutValue;
//
//         // Send reserves as tagged requests.
//         if (ioctlCode == IOCTL_STORAGE_RESERVE)
//         {
//             SET_FLAG(srb->SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//             srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//         }
//
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             srb,
//                                             NULL,
//                                             0,
//                                             FALSE,
//                                             Request);
//         // no data transfer.
//         *DataLength = 0;
//
//         FREE_POOL(srb);
//     }
//
//     return status;
// } // end RequestHandleReserveRelease()
//
// static // __REACTOS__
// BOOLEAN
// ValidPersistentReserveScope(
//     UCHAR Scope)
// {
//     switch (Scope) {
//     case RESERVATION_SCOPE_LU:
//     case RESERVATION_SCOPE_ELEMENT:
//
//         return TRUE;
//
//     default:
//
//         return FALSE;
//     }
// }
//
// static // __REACTOS__
// BOOLEAN
// ValidPersistentReserveType(
//     UCHAR Type)
// {
//     switch (Type) {
//     case RESERVATION_TYPE_WRITE_EXCLUSIVE:
//     case RESERVATION_TYPE_EXCLUSIVE:
//     case RESERVATION_TYPE_WRITE_EXCLUSIVE_REGISTRANTS:
//     case RESERVATION_TYPE_EXCLUSIVE_REGISTRANTS:
//
//         return TRUE;
//
//     default:
//
//         return FALSE;
//     }
// }
//
// NTSTATUS
// RequestValidatePersistentReserve(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Validate request of IOCTL_STORAGE_PERSISTENT_RESERVE_IN
//                      IOCTL_STORAGE_PERSISTENT_RESERVE_OUT
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     ULONG               ioctlCode = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
//
//     PPERSISTENT_RESERVE_COMMAND reserveCommand = NULL;
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&reserveCommand,
//                                            NULL);
//     if (NT_SUCCESS(status))
//     {
//         if ((RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength < sizeof(PERSISTENT_RESERVE_COMMAND)) ||
//             (reserveCommand->Size < sizeof(PERSISTENT_RESERVE_COMMAND)))
//         {
//             *DataLength = 0;
//             status = STATUS_INFO_LENGTH_MISMATCH;
//         }
//         else if ((ULONG_PTR)reserveCommand & DeviceExtension->AdapterDescriptor->AlignmentMask)
//         {
//             // Check buffer alignment. Only an issue if another kernel mode component
//             // (not the I/O manager) allocates the buffer.
//             *DataLength = 0;
//             status = STATUS_INVALID_USER_BUFFER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (ioctlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_IN)
//         {
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength < reserveCommand->PR_IN.AllocationLength)
//             {
//                 *DataLength = 0;
//                 status = STATUS_INVALID_PARAMETER;
//             }
//             else
//             {
//                 switch (reserveCommand->PR_IN.ServiceAction)
//                 {
//                 case RESERVATION_ACTION_READ_KEYS:
//                     if (reserveCommand->PR_IN.AllocationLength < sizeof(PRI_REGISTRATION_LIST))
//                     {
//                         *DataLength = 0;
//                         status = STATUS_INVALID_PARAMETER;
//                     }
//                     break;
//
//                 case RESERVATION_ACTION_READ_RESERVATIONS:
//                     if (reserveCommand->PR_IN.AllocationLength < sizeof(PRI_RESERVATION_LIST))
//                     {
//                         *DataLength = 0;
//                         status = STATUS_INVALID_PARAMETER;
//                     }
//                     break;
//
//                 default:
//                     *DataLength = 0;
//                     status = STATUS_INVALID_PARAMETER;
//                     break;
//                 }
//             }
//         }
//         else // case of IOCTL_STORAGE_PERSISTENT_RESERVE_OUT
//         {
//             // Verify ServiceAction, Scope, and Type
//             switch (reserveCommand->PR_OUT.ServiceAction)
//             {
//             case RESERVATION_ACTION_REGISTER:
//             case RESERVATION_ACTION_REGISTER_IGNORE_EXISTING:
//             case RESERVATION_ACTION_CLEAR:
//                 // Scope and type ignored.
//                 break;
//
//             case RESERVATION_ACTION_RESERVE:
//             case RESERVATION_ACTION_RELEASE:
//             case RESERVATION_ACTION_PREEMPT:
//             case RESERVATION_ACTION_PREEMPT_ABORT:
//                 if (!ValidPersistentReserveScope(reserveCommand->PR_OUT.Scope) ||
//                     !ValidPersistentReserveType(reserveCommand->PR_OUT.Type))
//                 {
//                     *DataLength = 0;
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//                 break;
//
//             default:
//                 *DataLength = 0;
//                 status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             // Check input buffer for PR Out.
//             // Caller must include the PR parameter list.
//             if (NT_SUCCESS(status))
//             {
//                 if ((RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength <
//                         (sizeof(PERSISTENT_RESERVE_COMMAND) + sizeof(PRO_PARAMETER_LIST))) ||
//                     (reserveCommand->Size < RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength))
//                 {
//                     *DataLength = 0;
//                     status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//         } // end of validation for IOCTL_STORAGE_PERSISTENT_RESERVE_OUT
//     }
//
//     return status;
// } // end RequestValidatePersistentReserve()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// RequestHandlePersistentReserve(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_PERSISTENT_RESERVE_IN
//                      IOCTL_STORAGE_PERSISTENT_RESERVE_OUT
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     ULONG               ioctlCode = RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
//     PSCSI_REQUEST_BLOCK srb = NULL;
//     PCDB                cdb = NULL;
//     BOOLEAN             writeToDevice;
//
//     PPERSISTENT_RESERVE_COMMAND reserveCommand = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&reserveCommand,
//                                            NULL);
//     if (NT_SUCCESS(status))
//     {
//         srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                              sizeof(SCSI_REQUEST_BLOCK) +
//                              (sizeof(ULONG_PTR) * 2),
//                              CDROM_TAG_SRB);
//
//         if (srb == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//         else
//         {
//             cdb = (PCDB)srb->Cdb;
//             RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         size_t dataBufLen = 0;
//
//         // Fill in the CDB.
//         if (ioctlCode == IOCTL_STORAGE_PERSISTENT_RESERVE_IN)
//         {
//             cdb->PERSISTENT_RESERVE_IN.OperationCode    = SCSIOP_PERSISTENT_RESERVE_IN;
//             cdb->PERSISTENT_RESERVE_IN.ServiceAction    = reserveCommand->PR_IN.ServiceAction;
//
//             REVERSE_BYTES_SHORT(&(cdb->PERSISTENT_RESERVE_IN.AllocationLength),
//                                 &(reserveCommand->PR_IN.AllocationLength));
//
//             dataBufLen = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//             writeToDevice = FALSE;
//         }
//         else
//         {
//             cdb->PERSISTENT_RESERVE_OUT.OperationCode   = SCSIOP_PERSISTENT_RESERVE_OUT;
//             cdb->PERSISTENT_RESERVE_OUT.ServiceAction   = reserveCommand->PR_OUT.ServiceAction;
//             cdb->PERSISTENT_RESERVE_OUT.Scope           = reserveCommand->PR_OUT.Scope;
//             cdb->PERSISTENT_RESERVE_OUT.Type            = reserveCommand->PR_OUT.Type;
//
//             cdb->PERSISTENT_RESERVE_OUT.ParameterListLength[1] = (UCHAR)sizeof(PRO_PARAMETER_LIST);
//
//             // Move the parameter list to the beginning of the data buffer (so it is aligned
//             // correctly and that the MDL describes it correctly).
//             RtlMoveMemory(reserveCommand,
//                           reserveCommand->PR_OUT.ParameterList,
//                           sizeof(PRO_PARAMETER_LIST));
//
//             dataBufLen = sizeof(PRO_PARAMETER_LIST);
//             writeToDevice = TRUE;
//         }
//
//         srb->CdbLength = 10;
//         srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             srb,
//                                             reserveCommand,
//                                             (ULONG) dataBufLen,
//                                             writeToDevice,
//                                             Request);
//
//         FREE_POOL(srb);
//     }
//
//     return status;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleAreVolumesReady(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DISK_ARE_VOLUMES_READY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     BOOLEAN                 completeRequest = TRUE;
//     NTSTATUS                status = STATUS_SUCCESS;
//
//     UNREFERENCED_PARAMETER(RequestParameters);
//
//     *DataLength = 0;
//
//     if (DeviceExtension->IsVolumeOnlinePending == FALSE)
//     {
//         status = STATUS_SUCCESS;
//         goto Cleanup;
//     }
//
//     //
//     // Add to the volume ready queue. No worries about request cancellation,
//     // since KMDF will automatically handle that while request is in queue.
//     //
//     status = WdfRequestForwardToIoQueue(Request,
//                                         DeviceExtension->ManualVolumeReadyQueue
//                                         );
//
//     if((!NT_SUCCESS(status)))
//     {
//         goto Cleanup;
//     }
//
//     status = STATUS_PENDING;
//     completeRequest = FALSE;
//
// Cleanup:
//
//     if (completeRequest)
//     {
//         RequestCompletion(DeviceExtension, Request, status, 0);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleVolumeOnline(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_VOLUME_ONLINE / IOCTL_VOLUME_POST_ONLINE
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     WDFREQUEST              request;
//     PIRP                    irp = NULL;
//     PIO_STACK_LOCATION      nextStack = NULL;
//
//     UNREFERENCED_PARAMETER(RequestParameters);
//
//     *DataLength = 0;
//
//     DeviceExtension->IsVolumeOnlinePending = FALSE;
//
//     // Complete all parked volume ready requests.
//     for (;;)
//     {
//         status = WdfIoQueueRetrieveNextRequest(DeviceExtension->ManualVolumeReadyQueue,
//                                                &request);
//
//         if ((!NT_SUCCESS(status)))
//         {
//             break;
//         }
//
//         RequestCompletion(DeviceExtension, request, STATUS_SUCCESS, 0);
//     }
//
//     // Change the IOCTL code to IOCTL_DISK_VOLUMES_ARE_READY, and forward the request down,
//     // so that if the underlying port driver will also know that volume is ready.
//     WdfRequestFormatRequestUsingCurrentType(Request);
//     irp = WdfRequestWdmGetIrp(Request);
//     nextStack = IoGetNextIrpStackLocation(irp);
//
//     irp->AssociatedIrp.SystemBuffer = &DeviceExtension->DeviceNumber;
//     nextStack->Parameters.DeviceIoControl.as_mut().OutputBufferLength = 0;
//     nextStack->Parameters.DeviceIoControl.as_mut().InputBufferLength = sizeof (DeviceExtension->DeviceNumber);
//     nextStack->Parameters.DeviceIoControl.as_mut().IoControlCode = IOCTL_DISK_VOLUMES_ARE_READY;
//
//     // Send the request straight down (synchronously).
//     RequestSend(DeviceExtension,
//                 Request,
//                 DeviceExtension->IoTarget,
//                 WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
//                 NULL);
//
//     return STATUS_SUCCESS;
// }
// #endif
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceHandleRawRead(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_RAW_READ
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     // Determine whether the drive is currently in raw or cooked mode,
//     // and which command to use to read the data.
//
//     NTSTATUS        status = STATUS_SUCCESS;
//     PCDROM_DATA     cdData = &(DeviceExtension->DeviceAdditionalData);
//     RAW_READ_INFO   rawReadInfo = {0};
//     PVOID           outputVirtAddr = NULL;
//     ULONG           startingSector;
//
//     PIRP                irp = WdfRequestWdmGetIrp(Request);
//     PIO_STACK_LOCATION  currentIrpStack = IoGetCurrentIrpStackLocation(irp);
//
//     VOID*               outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (TEST_FLAG(DeviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME) &&
//         (currentIrpStack->MinorFunction != CDROM_VOLUME_VERIFY_CHECKED) &&
//         !TEST_FLAG(currentIrpStack->Flags, SL_OVERRIDE_VERIFY_VOLUME))
//     {
//         //  DO_VERIFY_VOLUME is set for the device object,
//         //  but this request is not itself a verify request.
//         //  So fail this request.
//         (VOID) MediaReadCapacity(DeviceExtension->Device);
//
//         *DataLength = 0;
//         //set the status for volume verification.
//         status = STATUS_VERIFY_REQUIRED;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Since this ioctl is METHOD_OUT_DIRECT, we need to copy away
//         // the input buffer before interpreting it.  This prevents a
//         // malicious app from messing with the input buffer while we
//         // are interpreting it. This is done in dispacth.
//         //
//         // Here, we are going to get the input buffer out of saved place.
//         rawReadInfo = *(PRAW_READ_INFO)currentIrpStack->Parameters.DeviceIoControl.as_ref().Type3InputBuffer;
//
//         if (currentIrpStack->Parameters.DeviceIoControl.as_ref().OutputBufferLength > 0)
//         {
//             // Make sure that any user buffer that we pass down to
//             // the hardware is properly aligned
//             NT_ASSERT(irp->MdlAddress);
//             outputVirtAddr = MmGetMdlVirtualAddress(irp->MdlAddress);
//             if ((ULONG_PTR)outputVirtAddr & DeviceExtension->AdapterDescriptor->AlignmentMask)
//             {
//                 NT_ASSERT(!((ULONG_PTR)outputVirtAddr & DeviceExtension->AdapterDescriptor->AlignmentMask));
//                 *DataLength = 0;
//                 status = STATUS_INVALID_PARAMETER;
//             }
//         }
//         else
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         switch (rawReadInfo.TrackMode)
//         {
//         case CDDA:
//         case YellowMode2:
//         case XAForm2:
//             // no check needed.
//             break;
//
//         case RawWithC2AndSubCode:
//             if (!cdData->Mmc.ReadCdC2Pointers || !cdData->Mmc.ReadCdSubCode)
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                           "Request to read C2 & Subcode rejected.  "
//                           "Is C2 supported: %d   Is Subcode supported: %d\n",
//                           cdData->Mmc.ReadCdC2Pointers,
//                           cdData->Mmc.ReadCdSubCode
//                           ));
//                 *DataLength = 0;
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//             break;
//         case RawWithC2:
//             if (!cdData->Mmc.ReadCdC2Pointers)
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                           "Request to read C2 rejected because drive does not "
//                           "report support for C2 pointers\n"
//                           ));
//                 *DataLength = 0;
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//             break;
//         case RawWithSubCode:
//
//             if (!cdData->Mmc.ReadCdSubCode)
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                           "Request to read subcode rejected because drive does "
//                           "not report support for reading the subcode data\n"
//                           ));
//                 *DataLength = 0;
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//             }
//             break;
//
//         default:
//             *DataLength = 0;
//             status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         PSCSI_REQUEST_BLOCK srb = DeviceExtension->ScratchContext.ScratchSrb;
//         PCDB                cdb = (PCDB)(srb->Cdb);
//
//         size_t              transferByteCount;
//         BOOLEAN             shouldRetry = TRUE;
//         ULONG               timesAlreadyRetried = 0;
//         LONGLONG            retryIn100nsUnits = 0;
//
//         transferByteCount = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         while (shouldRetry)
//         {
//             // Setup cdb depending upon the sector type we want.
//             switch (rawReadInfo.TrackMode)
//             {
//             case CDDA:
//                 transferByteCount  = rawReadInfo.SectorCount * RAW_SECTOR_SIZE;
//                 cdb->READ_CD.ExpectedSectorType = CD_DA_SECTOR;
//                 cdb->READ_CD.IncludeUserData = 1;
//                 cdb->READ_CD.HeaderCode = 3;
//                 cdb->READ_CD.IncludeSyncData = 1;
//                 break;
//
//             case YellowMode2:
//                 transferByteCount  = rawReadInfo.SectorCount * RAW_SECTOR_SIZE;
//                 cdb->READ_CD.ExpectedSectorType = YELLOW_MODE2_SECTOR;
//                 cdb->READ_CD.IncludeUserData = 1;
//                 cdb->READ_CD.HeaderCode = 1;
//                 cdb->READ_CD.IncludeSyncData = 1;
//                 break;
//
//             case XAForm2:
//                 transferByteCount  = rawReadInfo.SectorCount * RAW_SECTOR_SIZE;
//                 cdb->READ_CD.ExpectedSectorType = FORM2_MODE2_SECTOR;
//                 cdb->READ_CD.IncludeUserData = 1;
//                 cdb->READ_CD.HeaderCode = 3;
//                 cdb->READ_CD.IncludeSyncData = 1;
//                 break;
//
//             case RawWithC2AndSubCode:
//                 transferByteCount  = rawReadInfo.SectorCount * CD_RAW_SECTOR_WITH_C2_AND_SUBCODE_SIZE;
//                 cdb->READ_CD.ExpectedSectorType = 0;    // Any sector type
//                 cdb->READ_CD.IncludeUserData = 1;
//                 cdb->READ_CD.HeaderCode = 3;            // Header and subheader returned
//                 cdb->READ_CD.IncludeSyncData = 1;
//                 cdb->READ_CD.ErrorFlags = 2;            // C2 and block error
//                 cdb->READ_CD.SubChannelSelection = 1;   // raw subchannel data
//                 break;
//
//             case RawWithC2:
//                 transferByteCount  = rawReadInfo.SectorCount * CD_RAW_SECTOR_WITH_C2_SIZE;
//                 cdb->READ_CD.ExpectedSectorType = 0;    // Any sector type
//                 cdb->READ_CD.IncludeUserData = 1;
//                 cdb->READ_CD.HeaderCode = 3;            // Header and subheader returned
//                 cdb->READ_CD.IncludeSyncData = 1;
//                 cdb->READ_CD.ErrorFlags = 2;            // C2 and block error
//                 break;
//
//             case RawWithSubCode:
//                 transferByteCount  = rawReadInfo.SectorCount * CD_RAW_SECTOR_WITH_SUBCODE_SIZE;
//                 cdb->READ_CD.ExpectedSectorType = 0;    // Any sector type
//                 cdb->READ_CD.IncludeUserData = 1;
//                 cdb->READ_CD.HeaderCode = 3;            // Header and subheader returned
//                 cdb->READ_CD.IncludeSyncData = 1;
//                 cdb->READ_CD.SubChannelSelection = 1;   // raw subchannel data
//                 break;
//
//             default:
//                 // should already checked before coming in loop.
//                 NT_ASSERT(FALSE);
//                 break;
//             }
//
//             ScratchBuffer_SetupSrb(DeviceExtension, Request, (ULONG)transferByteCount, TRUE);
//             // Restore the buffer, buffer size and MdlAddress. They got changed but we don't want to use the scratch buffer.
//             {
//                 PIRP    scratchIrp = WdfRequestWdmGetIrp(DeviceExtension->ScratchContext.ScratchRequest);
//                 scratchIrp->MdlAddress = irp->MdlAddress;
//                 srb->DataBuffer = outputVirtAddr;
//                 srb->DataTransferLength = (ULONG)transferByteCount;
//             }
//             // Calculate starting offset.
//             startingSector = (ULONG)(rawReadInfo.DiskOffset.QuadPart >> DeviceExtension->SectorShift);
//
//             // Fill in CDB fields.
//             srb->CdbLength = 12;
//
//             cdb->READ_CD.OperationCode = SCSIOP_READ_CD;
//             cdb->READ_CD.TransferBlocks[2]  = (UCHAR) (rawReadInfo.SectorCount & 0xFF);
//             cdb->READ_CD.TransferBlocks[1]  = (UCHAR) (rawReadInfo.SectorCount >> 8 );
//             cdb->READ_CD.TransferBlocks[0]  = (UCHAR) (rawReadInfo.SectorCount >> 16);
//
//             cdb->READ_CD.StartingLBA[3]  = (UCHAR) (startingSector & 0xFF);
//             cdb->READ_CD.StartingLBA[2]  = (UCHAR) ((startingSector >>  8));
//             cdb->READ_CD.StartingLBA[1]  = (UCHAR) ((startingSector >> 16));
//             cdb->READ_CD.StartingLBA[0]  = (UCHAR) ((startingSector >> 24));
//
//             ScratchBuffer_SendSrb(DeviceExtension, TRUE, NULL);
//
//             if ((DeviceExtension->ScratchContext.ScratchSrb->SrbStatus == SRB_STATUS_ABORTED) &&
//                 (DeviceExtension->ScratchContext.ScratchSrb->InternalStatus == STATUS_CANCELLED))
//             {
//                 shouldRetry = FALSE;
//                 status = STATUS_CANCELLED;
//             }
//             else
//             {
//                 shouldRetry = RequestSenseInfoInterpretForScratchBuffer(DeviceExtension,
//                                                                         timesAlreadyRetried,
//                                                                         &status,
//                                                                         &retryIn100nsUnits);
//                 if (shouldRetry)
//                 {
//                     LARGE_INTEGER t;
//                     t.QuadPart = -retryIn100nsUnits;
//                     timesAlreadyRetried++;
//                     KeDelayExecutionThread(KernelMode, FALSE, &t);
//                     // keep items clean
//                     ScratchBuffer_ResetItems(DeviceExtension, FALSE);
//                 }
//             }
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = srb->DataTransferLength;
//         }
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandlePlayAudioMsf(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_PLAY_AUDIO_MSF
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_PLAY_AUDIO_MSF   inputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&inputBuffer,
//                                            NULL);
//
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = 0;
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.PLAY_AUDIO_MSF.as_mut().OperationCode = SCSIOP_PLAY_AUDIO_MSF;
//
//         cdb.PLAY_AUDIO_MSF.as_mut().StartingM = inputBuffer->StartingM;
//         cdb.PLAY_AUDIO_MSF.as_mut().StartingS = inputBuffer->StartingS;
//         cdb.PLAY_AUDIO_MSF.as_mut().StartingF = inputBuffer->StartingF;
//
//         cdb.PLAY_AUDIO_MSF.as_mut().EndingM = inputBuffer->EndingM;
//         cdb.PLAY_AUDIO_MSF.as_mut().EndingS = inputBuffer->EndingS;
//         cdb.PLAY_AUDIO_MSF.as_mut().EndingF = inputBuffer->EndingF;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             PLAY_ACTIVE(DeviceExtension) = TRUE;
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleReadQChannel(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_READ_Q_CHANNEL
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     PVOID       inputBuffer = NULL;
//     PVOID       outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = ReadQChannel(DeviceExtension,
//                               Request,
//                               inputBuffer,
//                               RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                               outputBuffer,
//                               RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                               DataLength);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// ReadQChannel(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST           OriginalRequest,
//       PVOID                    InputBuffer,
//       size_t                   InputBufferLength,
//       PVOID                    OutputBuffer,
//       size_t                   OutputBufferLength,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    base function to handle request of IOCTL_CDROM_READ_Q_CHANNEL
//
// Arguments:
//
//     DeviceExtension - device context
//     OriginalRequest - original request to be handled
//     InputBuffer - input buffer
//     InputBufferLength - length of input buffer
//     OutputBuffer - output buffer
//     OutputBufferLength - length of output buffer
//
// Return Value:
//
//     NTSTATUS
//     DataLength - returned data length
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     ULONG                       transferByteCount = 0;
//     CDB                         cdb;
//     PCDROM_SUB_Q_DATA_FORMAT    inputBuffer = (PCDROM_SUB_Q_DATA_FORMAT)InputBuffer;
//     PSUB_Q_CHANNEL_DATA         userChannelData = (PSUB_Q_CHANNEL_DATA)OutputBuffer;
//
//     PAGED_CODE ();
//
//     UNREFERENCED_PARAMETER(InputBufferLength);
//
//     *DataLength = 0;
//
//     // Set size of channel data -- however, this is dependent on
//     // what information we are requesting (which Format)
//     switch( inputBuffer->Format )
//     {
//         case IOCTL_CDROM_CURRENT_POSITION:
//             transferByteCount = sizeof(SUB_Q_CURRENT_POSITION);
//             break;
//
//         case IOCTL_CDROM_MEDIA_CATALOG:
//             transferByteCount = sizeof(SUB_Q_MEDIA_CATALOG_NUMBER);
//             break;
//
//         case IOCTL_CDROM_TRACK_ISRC:
//             transferByteCount = sizeof(SUB_Q_TRACK_ISRC);
//             break;
//     }
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     // Always logical unit 0, but only use MSF addressing
//     // for IOCTL_CDROM_CURRENT_POSITION
//     if (inputBuffer->Format==IOCTL_CDROM_CURRENT_POSITION)
//     {
//         cdb.SUBCHANNEL.as_mut().Msf = CDB_USE_MSF;
//     }
//
//     // Return subchannel data
//     cdb.SUBCHANNEL.as_mut().SubQ = CDB_SUBCHANNEL_BLOCK;
//
//     // Specify format of informatin to return
//     cdb.SUBCHANNEL.as_mut().Format = inputBuffer->Format;
//
//     // Specify which track to access (only used by Track ISRC reads)
//     if (inputBuffer->Format==IOCTL_CDROM_TRACK_ISRC)
//     {
//         cdb.SUBCHANNEL.as_mut().TrackNumber = inputBuffer->Track;
//     }
//
//     cdb.SUBCHANNEL.as_mut().AllocationLength[0] = (UCHAR) (transferByteCount >> 8);
//     cdb.SUBCHANNEL.as_mut().AllocationLength[1] = (UCHAR) (transferByteCount & 0xFF);
//     cdb.SUBCHANNEL.as_mut().OperationCode = SCSIOP_READ_SUB_CHANNEL;
//
//     status = ScratchBuffer_ExecuteCdb(DeviceExtension, OriginalRequest, transferByteCount, TRUE, &cdb, 10);
//
//     if (NT_SUCCESS(status))
//     {
//         PSUB_Q_CHANNEL_DATA subQPtr = DeviceExtension->ScratchContext.ScratchSrb->DataBuffer;
//
// #if DBG
//         switch( inputBuffer->Format )
//         {
//         case IOCTL_CDROM_CURRENT_POSITION:
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Audio Status is %u\n", subQPtr->CurrentPosition.Header.AudioStatus ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: ADR = 0x%x\n", subQPtr->CurrentPosition.ADR ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Control = 0x%x\n", subQPtr->CurrentPosition.Control ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Track = %u\n", subQPtr->CurrentPosition.TrackNumber ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Index = %u\n", subQPtr->CurrentPosition.IndexNumber ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Absolute Address = %x\n", *((PULONG)subQPtr->CurrentPosition.AbsoluteAddress) ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Relative Address = %x\n", *((PULONG)subQPtr->CurrentPosition.TrackRelativeAddress) ));
//             break;
//
//         case IOCTL_CDROM_MEDIA_CATALOG:
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Audio Status is %u\n", subQPtr->MediaCatalog.Header.AudioStatus ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Mcval is %u\n", subQPtr->MediaCatalog.Mcval ));
//             break;
//
//         case IOCTL_CDROM_TRACK_ISRC:
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Audio Status is %u\n", subQPtr->TrackIsrc.Header.AudioStatus ));
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,"ReadQChannel: Tcval is %u\n", subQPtr->TrackIsrc.Tcval ));
//             break;
//         }
// #endif
//
//         // Update the play active status.
//         if (subQPtr->CurrentPosition.Header.AudioStatus == AUDIO_STATUS_IN_PROGRESS)
//         {
//             PLAY_ACTIVE(DeviceExtension) = TRUE;
//         }
//         else
//         {
//             PLAY_ACTIVE(DeviceExtension) = FALSE;
//         }
//
//         // Check if output buffer is large enough to contain
//         // the data.
//         if (OutputBufferLength < DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength)
//         {
//             DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength = (ULONG)OutputBufferLength;
//         }
//
//         // Copy our buffer into users.
//         RtlMoveMemory(userChannelData,
//                       subQPtr,
//                       DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength);
//
//         *DataLength = DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//     }
//
//     // nothing to do after the command finishes.
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandlePauseAudio(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_PAUSE_AUDIO
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = 0;
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.PAUSE_RESUME.as_mut().OperationCode = SCSIOP_PAUSE_RESUME;
//         cdb.PAUSE_RESUME.as_mut().Action = CDB_AUDIO_PAUSE;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             PLAY_ACTIVE(DeviceExtension) = FALSE;
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleResumeAudio(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_RESUME_AUDIO
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = 0;
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.PAUSE_RESUME.as_mut().OperationCode = SCSIOP_PAUSE_RESUME;
//         cdb.PAUSE_RESUME.as_mut().Action = CDB_AUDIO_RESUME;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             PLAY_ACTIVE(DeviceExtension) = TRUE;  //not in original code. But we should set it.
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleSeekAudioMsf(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_SEEK_AUDIO_MSF
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS              status = STATUS_SUCCESS;
//     PCDROM_SEEK_AUDIO_MSF inputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = 0;
//         CDB     cdb;
//         ULONG   logicalBlockAddress;
//
//         logicalBlockAddress = MSF_TO_LBA(inputBuffer->M, inputBuffer->S, inputBuffer->F);
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.SEEK.as_mut().OperationCode      = SCSIOP_SEEK;
//         cdb.SEEK.as_mut().LogicalBlockAddress[0] = ((PFOUR_BYTE)&logicalBlockAddress)->Byte3;
//         cdb.SEEK.as_mut().LogicalBlockAddress[1] = ((PFOUR_BYTE)&logicalBlockAddress)->Byte2;
//         cdb.SEEK.as_mut().LogicalBlockAddress[2] = ((PFOUR_BYTE)&logicalBlockAddress)->Byte1;
//         cdb.SEEK.as_mut().LogicalBlockAddress[3] = ((PFOUR_BYTE)&logicalBlockAddress)->Byte0;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleStopAudio(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = 0;
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.START_STOP.as_mut().OperationCode = SCSIOP_START_STOP_UNIT;
//         cdb.START_STOP.as_mut().Immediate = 1;
//         cdb.START_STOP.as_mut().Start = 0;
//         cdb.START_STOP.as_mut().LoadEject = 0;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 6);
//
//         if (NT_SUCCESS(status))
//         {
//             PLAY_ACTIVE(DeviceExtension) = FALSE;
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleGetSetVolume(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_GET_VOLUME
//                      IOCTL_CDROM_SET_VOLUME
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS              status = STATUS_SUCCESS;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG   transferSize = MODE_DATA_SIZE;
//         CDB     cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.MODE_SENSE10.as_mut().OperationCode = SCSIOP_MODE_SENSE10;
//         cdb.MODE_SENSE10.as_mut().PageCode = CDROM_AUDIO_CONTROL_PAGE;
//         cdb.MODE_SENSE10.as_mut().AllocationLength[0] = (UCHAR)(MODE_DATA_SIZE >> 8);
//         cdb.MODE_SENSE10.as_mut().AllocationLength[1] = (UCHAR)(MODE_DATA_SIZE & 0xFF);
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, TRUE, &cdb, 10);
//
//         if (NT_SUCCESS(status))
//         {
//             if (RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_CDROM_GET_VOLUME)
//             {
//                 PAUDIO_OUTPUT   audioOutput;
//                 PVOLUME_CONTROL volumeControl;
//                 ULONG           bytesTransferred;
//
//                 status = WdfRequestRetrieveOutputBuffer(Request,
//                                                         RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                         (PVOID*)&volumeControl,
//                                                         NULL);
//                 if (NT_SUCCESS(status))
//                 {
//                     audioOutput = ModeSenseFindSpecificPage((PCHAR)DeviceExtension->ScratchContext.ScratchSrb->DataBuffer,
//                                                             DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength,
//                                                             CDROM_AUDIO_CONTROL_PAGE,
//                                                             FALSE);
//
//                     // Verify the page is as big as expected.
//                     bytesTransferred = (ULONG)((PCHAR)audioOutput - (PCHAR)DeviceExtension->ScratchContext.ScratchSrb->DataBuffer) +
//                                         sizeof(AUDIO_OUTPUT);
//
//                     if ((audioOutput != NULL) &&
//                         (DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength >= bytesTransferred))
//                     {
//                         ULONG i;
//                         for (i=0; i<4; i++)
//                         {
//                             volumeControl->PortVolume[i] = audioOutput->PortOutput[i].Volume;
//                         }
//
//                         // Set bytes transferred in IRP.
//                         *DataLength = sizeof(VOLUME_CONTROL);
//
//                     }
//                     else
//                     {
//                         *DataLength = 0;
//                         status = STATUS_INVALID_DEVICE_REQUEST;
//                     }
//                 }
//             }
//             else    //IOCTL_CDROM_SET_VOLUME
//             {
//                 PAUDIO_OUTPUT   audioInput = NULL;
//                 PAUDIO_OUTPUT   audioOutput = NULL;
//                 PVOLUME_CONTROL volumeControl = NULL;
//                 ULONG           i,bytesTransferred,headerLength;
//
//                 status = WdfRequestRetrieveInputBuffer(Request,
//                                                        RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                        (PVOID*)&volumeControl,
//                                                        NULL);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     audioInput = ModeSenseFindSpecificPage((PCHAR)DeviceExtension->ScratchContext.ScratchSrb->DataBuffer,
//                                                            DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength,
//                                                            CDROM_AUDIO_CONTROL_PAGE,
//                                                            FALSE);
//
//                     // Check to make sure the mode sense data is valid before we go on
//                     if(audioInput == NULL)
//                     {
//                         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                                     "Mode Sense Page %d not found\n",
//                                     CDROM_AUDIO_CONTROL_PAGE));
//
//                         *DataLength = 0;
//                         status = STATUS_INVALID_DEVICE_REQUEST;
//                     }
//                 }
//
//                 if (NT_SUCCESS(status))
//                 {
//                     // keep items clean; clear the command history
//                     ScratchBuffer_ResetItems(DeviceExtension, TRUE);
//
//                     headerLength = sizeof(MODE_PARAMETER_HEADER10);
//                     bytesTransferred = sizeof(AUDIO_OUTPUT) + headerLength;
//
//                     // use the scratch buffer as input buffer.
//                     // the content of this buffer will not be changed in the following loop.
//                     audioOutput = (PAUDIO_OUTPUT)((PCHAR)DeviceExtension->ScratchContext.ScratchBuffer + headerLength);
//
//                     for (i=0; i<4; i++)
//                     {
//                         audioOutput->PortOutput[i].Volume = volumeControl->PortVolume[i];
//                         audioOutput->PortOutput[i].ChannelSelection = audioInput->PortOutput[i].ChannelSelection;
//                     }
//
//                     audioOutput->CodePage = CDROM_AUDIO_CONTROL_PAGE;
//                     audioOutput->ParameterLength = sizeof(AUDIO_OUTPUT) - 2;
//                     audioOutput->Immediate = MODE_SELECT_IMMEDIATE;
//
//                     RtlZeroMemory(&cdb, sizeof(CDB));
//                     // Set up the CDB
//                     cdb.MODE_SELECT10.as_mut().OperationCode = SCSIOP_MODE_SELECT10;
//                     cdb.MODE_SELECT10.as_mut().ParameterListLength[0] = (UCHAR) (bytesTransferred >> 8);
//                     cdb.MODE_SELECT10.as_mut().ParameterListLength[1] = (UCHAR) (bytesTransferred & 0xFF);
//                     cdb.MODE_SELECT10.as_mut().PFBit = 1;
//
//                     status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, bytesTransferred, FALSE, &cdb, 10);
//
//                 }
//                 *DataLength = 0;
//             }
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleReadDvdStructure(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DVD_READ_STRUCTURE
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     PVOID       inputBuffer = NULL;
//     PVOID       outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = ReadDvdStructure(DeviceExtension,
//                                   Request,
//                                   inputBuffer,
//                                   RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                   outputBuffer,
//                                   RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                   DataLength);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// ReadDvdStructure(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST           OriginalRequest,
//       PVOID                    InputBuffer,
//       size_t                   InputBufferLength,
//       PVOID                    OutputBuffer,
//       size_t                   OutputBufferLength,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    base function to handle request of IOCTL_DVD_START_SESSION
//                                       IOCTL_DVD_READ_KEY
//
// Arguments:
//
//     DeviceExtension - device context
//     OriginalRequest - original request to be handled
//     InputBuffer - input buffer
//     InputBufferLength - length of input buffer
//     OutputBuffer - output buffer
//     OutputBufferLength - length of output buffer
//
// Return Value:
//
//     NTSTATUS
//     DataLength - returned data length
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PDVD_READ_STRUCTURE     request = (PDVD_READ_STRUCTURE)InputBuffer;
//     PDVD_DESCRIPTOR_HEADER  header = (PDVD_DESCRIPTOR_HEADER)OutputBuffer;
//     CDB                     cdb;
//
//     USHORT                  dataLength;
//     ULONG                   blockNumber;
//     PFOUR_BYTE              fourByte;
//
//     PAGED_CODE ();
//
//     UNREFERENCED_PARAMETER(InputBufferLength);
//
//     if (DeviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//     {
//         *DataLength = 0;
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     dataLength = (USHORT)OutputBufferLength;
//     blockNumber = (ULONG)(request->BlockByteOffset.QuadPart >> DeviceExtension->SectorShift);
//     fourByte = (PFOUR_BYTE)&blockNumber;
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     cdb.READ_DVD_STRUCTURE.as_mut().OperationCode = SCSIOP_READ_DVD_STRUCTURE;
//     cdb.READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[0] = fourByte->Byte3;
//     cdb.READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[1] = fourByte->Byte2;
//     cdb.READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[2] = fourByte->Byte1;
//     cdb.READ_DVD_STRUCTURE.as_mut().RMDBlockNumber[3] = fourByte->Byte0;
//     cdb.READ_DVD_STRUCTURE.as_mut().LayerNumber   = request->LayerNumber;
//     cdb.READ_DVD_STRUCTURE.as_mut().Format        = (UCHAR)request->Format;
//
// #if DBG
//     {
//         if ((UCHAR)request->Format > DvdMaxDescriptor)
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                         "READ_DVD_STRUCTURE format %x = %s (%x bytes)\n",
//                         (UCHAR)request->Format,
//                         READ_DVD_STRUCTURE_FORMAT_STRINGS[DvdMaxDescriptor],
//                         dataLength
//                         ));
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                         "READ_DVD_STRUCTURE format %x = %s (%x bytes)\n",
//                         (UCHAR)request->Format,
//                         READ_DVD_STRUCTURE_FORMAT_STRINGS[(UCHAR)request->Format],
//                         dataLength
//                         ));
//         }
//     }
// #endif // DBG
//
//     if (request->Format == DvdDiskKeyDescriptor)
//     {
//         cdb.READ_DVD_STRUCTURE.as_mut().AGID = (UCHAR)request->SessionId;
//     }
//
//     cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[0] = (UCHAR)(dataLength >> 8);
//     cdb.READ_DVD_STRUCTURE.as_mut().AllocationLength[1] = (UCHAR)(dataLength & 0xff);
//
//     status = ScratchBuffer_ExecuteCdb(DeviceExtension, OriginalRequest, dataLength, TRUE, &cdb, 12);
//
//     if (NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "DvdDCCompletion - READ_STRUCTURE: descriptor format of %d\n", request->Format));
//
//         RtlMoveMemory(header,
//                       DeviceExtension->ScratchContext.ScratchSrb->DataBuffer,
//                       DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength);
//
//         // Cook the data.  There are a number of fields that really
//         // should be byte-swapped for the caller.
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                   "DvdDCCompletion - READ_STRUCTURE:\n"
//                   "\tHeader at %p\n"
//                   "\tDvdDCCompletion - READ_STRUCTURE: data at %p\n"
//                   "\tDataBuffer was at %p\n"
//                   "\tDataTransferLength was %lx\n",
//                   header,
//                   header->Data,
//                   DeviceExtension->ScratchContext.ScratchSrb->DataBuffer,
//                   DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength));
//
//         // First the fields in the header
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "READ_STRUCTURE: header->Length %lx -> ",
//                    header->Length));
//
//         REVERSE_SHORT(&header->Length);
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "%lx\n", header->Length));
//
//         // Now the fields in the descriptor
//         if(request->Format == DvdPhysicalDescriptor)
//         {
//             ULONG tempLength = (DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength > (ULONG)FIELD_OFFSET(DVD_DESCRIPTOR_HEADER, Data))
//                                 ? (DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength - FIELD_OFFSET(DVD_DESCRIPTOR_HEADER, Data))
//                                 : 0;
//
//             PDVD_LAYER_DESCRIPTOR layer = (PDVD_LAYER_DESCRIPTOR)&(header->Data[0]);
//
//             // Make sure the buffer size is good for swapping bytes.
//             if (tempLength >= RTL_SIZEOF_THROUGH_FIELD(DVD_LAYER_DESCRIPTOR, StartingDataSector))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "READ_STRUCTURE: StartingDataSector %lx -> ",
//                                layer->StartingDataSector));
//                 REVERSE_LONG(&(layer->StartingDataSector));
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "%lx\n", layer->StartingDataSector));
//             }
//             if (tempLength >= RTL_SIZEOF_THROUGH_FIELD(DVD_LAYER_DESCRIPTOR, EndDataSector))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "READ_STRUCTURE: EndDataSector %lx -> ",
//                                layer->EndDataSector));
//                 REVERSE_LONG(&(layer->EndDataSector));
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "%lx\n", layer->EndDataSector));
//             }
//             if (tempLength >= RTL_SIZEOF_THROUGH_FIELD(DVD_LAYER_DESCRIPTOR, EndLayerZeroSector))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "READ_STRUCTURE: EndLayerZeroSector %lx -> ",
//                                layer->EndLayerZeroSector));
//                 REVERSE_LONG(&(layer->EndLayerZeroSector));
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "%lx\n", layer->EndLayerZeroSector));
//             }
//         }
//
//         if ((!NT_SUCCESS(status)))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "Status is %lx\n", status));
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DvdDeviceControlCompletion - "
//                         "IOCTL_DVD_READ_STRUCTURE: data transfer length of %d\n",
//                         DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength));
//         }
//
//         *DataLength = DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength;
//     }
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleDvdEndSession(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DVD_END_SESSION
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//     PDVD_SESSION_ID sessionId = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&sessionId,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG           transferSize = 0;
//         CDB             cdb;
//         DVD_SESSION_ID  currentSession = 0;
//         DVD_SESSION_ID  limitSession = 0;
//
//         if(*sessionId == DVD_END_ALL_SESSIONS)
//         {
//             currentSession = 0;
//             limitSession = MAX_COPY_PROTECT_AGID - 1;
//         }
//         else
//         {
//             currentSession = *sessionId;
//             limitSession = *sessionId;
//         }
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         do
//         {
//             RtlZeroMemory(&cdb, sizeof(CDB));
//             // Set up the CDB
//             cdb.SEND_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
//             cdb.SEND_KEY.as_mut().AGID = (UCHAR)(currentSession);
//             cdb.SEND_KEY.as_mut().KeyFormat = DVD_INVALIDATE_AGID;
//
//             status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 12);
//
//             currentSession++;
//         } while ((currentSession <= limitSession) && NT_SUCCESS(status));
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleDvdStartSessionReadKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DVD_START_SESSION
//                      IOCTL_DVD_READ_KEY
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PDVD_COPY_PROTECT_KEY   keyParameters = NULL;
//     PVOID                   outputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &outputBuffer,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status) && RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode == IOCTL_DVD_READ_KEY)
//     {
//         status = WdfRequestRetrieveInputBuffer(Request,
//                                                RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                                (PVOID*)&keyParameters,
//                                                NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = DvdStartSessionReadKey(DeviceExtension,
//                                         RequestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode,
//                                         Request,
//                                         keyParameters,
//                                         RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                         outputBuffer,
//                                         RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                         DataLength);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DvdStartSessionReadKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       ULONG                    IoControlCode,
//       WDFREQUEST           OriginalRequest,
//       PVOID                InputBuffer,
//       size_t                   InputBufferLength,
//       PVOID                    OutputBuffer,
//       size_t                   OutputBufferLength,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    base function to handle request of IOCTL_DVD_START_SESSION
//                                       IOCTL_DVD_READ_KEY
//
// Arguments:
//
//     DeviceExtension - device context
//     IoControlCode - IOCTL_DVD_READ_KEY or IOCTL_DVD_START_SESSION
//     OriginalRequest - original request to be handled
//     InputBuffer - input buffer
//     InputBufferLength - length of input buffer
//     OutputBuffer - output buffer
//     OutputBufferLength - length of output buffer
//
// Return Value:
//
//     NTSTATUS
//     DataLength - returned data length
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     ULONG                   keyLength = 0;
//     ULONG                   result = 0;
//     ULONG                   allocationLength;
//     PFOUR_BYTE              fourByte;
//     PDVD_COPY_PROTECT_KEY   keyParameters = (PDVD_COPY_PROTECT_KEY)InputBuffer;
//
//     PAGED_CODE ();
//
//     UNREFERENCED_PARAMETER(InputBufferLength);
//
//     *DataLength = 0;
//
//     fourByte = (PFOUR_BYTE)&allocationLength;
//
//     if (IoControlCode == IOCTL_DVD_READ_KEY)
//     {
//         if (keyParameters == NULL)
//         {
//             status = STATUS_INTERNAL_ERROR;
//         }
//
//         // First of all, initialize the DVD region of the drive, if it has not been set yet
//         if (NT_SUCCESS(status) &&
//             (keyParameters->KeyType == DvdGetRpcKey) &&
//             DeviceExtension->DeviceAdditionalData.Mmc.IsCssDvd)
//         {
//             DevicePickDvdRegion(DeviceExtension->Device);
//         }
//
//         if (NT_SUCCESS(status) &&
//             (keyParameters->KeyType == DvdDiskKey))
//         {
//             // Special case - need to use READ DVD STRUCTURE command to get the disk key.
//             PDVD_COPY_PROTECT_KEY   keyHeader = NULL;
//             PDVD_READ_STRUCTURE     readStructureRequest = (PDVD_READ_STRUCTURE)keyParameters;
//
//             // save the key header so we can restore the interesting parts later
//             keyHeader = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                               sizeof(DVD_COPY_PROTECT_KEY),
//                                               DVD_TAG_READ_KEY);
//
//             if(keyHeader == NULL)
//             {
//                 // Can't save the context so return an error
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "DvdDeviceControl - READ_KEY: unable to allocate context\n"));
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 PREAD_DVD_STRUCTURES_HEADER rawKey = OutputBuffer;
//                 PDVD_COPY_PROTECT_KEY       outputKey = OutputBuffer;
//
//                 // save input parameters
//                 RtlCopyMemory(keyHeader,
//                               InputBuffer,
//                               sizeof(DVD_COPY_PROTECT_KEY));
//
//                 readStructureRequest->Format = DvdDiskKeyDescriptor;
//                 readStructureRequest->BlockByteOffset.QuadPart = 0;
//                 readStructureRequest->LayerNumber = 0;
//                 readStructureRequest->SessionId = keyHeader->SessionId;
//
//                 status = ReadDvdStructure(DeviceExtension,
//                                           OriginalRequest,
//                                           InputBuffer,
//                                           sizeof(DVD_READ_STRUCTURE),
//                                           OutputBuffer,
//                                           sizeof(READ_DVD_STRUCTURES_HEADER) + sizeof(DVD_DISK_KEY_DESCRIPTOR),
//                                           DataLength);
//
//                 // fill the output buffer, it's not touched in DeviceHandleReadDvdStructure()
//                 // for this specific request type: DvdDiskKeyDescriptor
//                 if (NT_SUCCESS(status))
//                 {
//                     // Shift the data down to its new position.
//                     RtlMoveMemory(outputKey->KeyData,
//                                   rawKey->Data,
//                                   sizeof(DVD_DISK_KEY_DESCRIPTOR));
//
//                     RtlCopyMemory(outputKey,
//                                   keyHeader,
//                                   sizeof(DVD_COPY_PROTECT_KEY));
//
//                     outputKey->KeyLength = DVD_DISK_KEY_LENGTH;
//
//                     *DataLength = DVD_DISK_KEY_LENGTH;
//                 }
//                 else
//                 {
//                     TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                                 "StartSessionReadKey Failed with status %x, %xI64 (%x) bytes\n",
//                                 status,
//                                 (unsigned int)*DataLength,
//                                 ((rawKey->Length[0] << 16) | rawKey->Length[1]) ));
//                 }
//
//                 FREE_POOL(keyHeader);
//             }
//
//             // special process finished. return from here.
//             return status;
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             status = RtlULongSub((ULONG)OutputBufferLength,
//                                  (ULONG)sizeof(DVD_COPY_PROTECT_KEY), &result);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             status = RtlULongAdd(result, sizeof(CDVD_KEY_HEADER), &keyLength);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             //The data length field of REPORT KEY Command occupies two bytes
//             keyLength = min(keyLength, MAXUSHORT);
//         }
//     }
//     else    //IOCTL_DVD_START_SESSION
//     {
//         keyParameters = NULL;
//         keyLength = sizeof(CDVD_KEY_HEADER) + sizeof(CDVD_REPORT_AGID_DATA);
//         status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         CDB cdb;
//
//         allocationLength = keyLength;
//
//         // Defensive coding. Prefix cannot recognize this usage.
//         UNREFERENCED_PARAMETER(allocationLength);
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_REPORT_KEY;
//         cdb.REPORT_KEY.as_mut().AllocationLength[0] = fourByte->Byte1;
//         cdb.REPORT_KEY.as_mut().AllocationLength[1] = fourByte->Byte0;
//
//         // set the specific parameters....
//         if(IoControlCode == IOCTL_DVD_READ_KEY)
//         {
//             if(keyParameters->KeyType == DvdTitleKey)
//             {
//                 ULONG logicalBlockAddress;
//
//                 logicalBlockAddress = (ULONG)(keyParameters->Parameters.TitleOffset.QuadPart >>
//                                               DeviceExtension->SectorShift);
//
//                 fourByte = (PFOUR_BYTE)&(logicalBlockAddress);
//
//                 cdb.REPORT_KEY.as_mut().LogicalBlockAddress[0] = fourByte->Byte3;
//                 cdb.REPORT_KEY.as_mut().LogicalBlockAddress[1] = fourByte->Byte2;
//                 cdb.REPORT_KEY.as_mut().LogicalBlockAddress[2] = fourByte->Byte1;
//                 cdb.REPORT_KEY.as_mut().LogicalBlockAddress[3] = fourByte->Byte0;
//             }
//
//             cdb.REPORT_KEY.as_mut().KeyFormat = (UCHAR)keyParameters->KeyType;
//             cdb.REPORT_KEY.as_mut().AGID = (UCHAR)keyParameters->SessionId;
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "DvdStartSessionReadKey => sending irp %p (%s)\n",
//                         OriginalRequest, "READ_KEY"));
//         }
//         else
//         {
//             cdb.REPORT_KEY.as_mut().KeyFormat = DVD_REPORT_AGID;
//             cdb.REPORT_KEY.as_mut().AGID = 0;
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "DvdStartSessionReadKey => sending irp %p (%s)\n",
//                         OriginalRequest, "START_SESSION"));
//         }
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, OriginalRequest, keyLength, TRUE, &cdb, 12);
//
//         if (NT_SUCCESS(status))
//         {
//             if(IoControlCode == IOCTL_DVD_READ_KEY)
//             {
//                 NTSTATUS                tempStatus;
//                 PDVD_COPY_PROTECT_KEY   copyProtectKey = (PDVD_COPY_PROTECT_KEY)OutputBuffer;
//                 PCDVD_KEY_HEADER        keyHeader = DeviceExtension->ScratchContext.ScratchSrb->DataBuffer;
//                 ULONG                   dataLength;
//                 ULONG                   transferLength;
//
//                 tempStatus = RtlULongSub(DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength,
//                                          FIELD_OFFSET(CDVD_KEY_HEADER, Data),
//                                          &transferLength);
//
//                 dataLength = (keyHeader->DataLength[0] << 8) + keyHeader->DataLength[1];
//
//                 if (NT_SUCCESS(tempStatus) && (dataLength >= 2))
//                 {
//                     // Adjust the data length to ignore the two reserved bytes in the
//                     // header.
//                     dataLength -= 2;
//
//                     // take the minimum of the transferred length and the
//                     // length as specified in the header.
//                     if(dataLength < transferLength)
//                     {
//                         transferLength = dataLength;
//                     }
//
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                                 "DvdDeviceControlCompletion: [%p] - READ_KEY with "
//                                 "transfer length of (%d or %d) bytes\n",
//                                 OriginalRequest,
//                                 dataLength,
//                                 DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength - 2));
//
//                     // Copy the key data into the return buffer
//                     if(copyProtectKey->KeyType == DvdTitleKey)
//                     {
//                         RtlMoveMemory(copyProtectKey->KeyData,
//                                       keyHeader->Data + 1,
//                                       transferLength - 1);
//
//                         copyProtectKey->KeyData[transferLength - 1] = 0;
//
//                         // If this is a title key then we need to copy the CGMS flags
//                         // as well.
//                         copyProtectKey->KeyFlags = *(keyHeader->Data);
//
//                     }
//                     else
//                     {
//                         RtlMoveMemory(copyProtectKey->KeyData,
//                                       keyHeader->Data,
//                                       transferLength);
//                     }
//
//                     copyProtectKey->KeyLength = sizeof(DVD_COPY_PROTECT_KEY);
//                     copyProtectKey->KeyLength += transferLength;
//
//                     *DataLength = copyProtectKey->KeyLength;
//                 }
//                 else
//                 {
//                     //There is no valid data from drive.
//                     //This may happen when Key Format = 0x3f that does not require data back from drive.
//                     status = STATUS_SUCCESS;
//                     *DataLength = 0;
//                 }
//             }
//             else
//             {
//                 PDVD_SESSION_ID         sessionId = (PDVD_SESSION_ID)OutputBuffer;
//                 PCDVD_KEY_HEADER        keyHeader = DeviceExtension->ScratchContext.ScratchSrb->DataBuffer;
//                 PCDVD_REPORT_AGID_DATA  keyData = (PCDVD_REPORT_AGID_DATA)keyHeader->Data;
//
//                 *sessionId = keyData->AGID;
//
//                 *DataLength = sizeof(DVD_SESSION_ID);
//             }
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleDvdSendKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_DVD_SEND_KEY
//                      IOCTL_DVD_SEND_KEY2
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PVOID                   inputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            &inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         status = DvdSendKey(DeviceExtension,
//                             Request,
//                             inputBuffer,
//                             RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                             DataLength);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DvdSendKey(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST           OriginalRequest,
//       PVOID                    InputBuffer,
//       size_t                   InputBufferLength,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    base function to handle request of IOCTL_DVD_SEND_KEY(2)
//    NOTE: cdrom does not process this IOCTL if the input buffer length is bigger than Port transfer length.
//
// Arguments:
//
//     DeviceExtension - device context
//     OriginalRequest - original request to be handled
//     InputBuffer - input buffer
//     InputBufferLength - length of input buffer
//
// Return Value:
//
//     NTSTATUS
//     DataLength - returned data length
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PDVD_COPY_PROTECT_KEY   key = (PDVD_COPY_PROTECT_KEY)InputBuffer;
//
//     ULONG                   keyLength = 0;
//     ULONG                   result = 0;
//     PFOUR_BYTE              fourByte;
//
//     PAGED_CODE ();
//
//     UNREFERENCED_PARAMETER(InputBufferLength);
//
//     *DataLength = 0;
//
//     if (NT_SUCCESS(status))
//     {
//         if ((key->KeyLength < sizeof(DVD_COPY_PROTECT_KEY)) ||
//             ((key->KeyLength - sizeof(DVD_COPY_PROTECT_KEY)) > DeviceExtension->ScratchContext.ScratchBufferSize))
//         {
//             NT_ASSERT(FALSE);
//             status = STATUS_INTERNAL_ERROR;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = RtlULongSub(key->KeyLength, sizeof(DVD_COPY_PROTECT_KEY), &result);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = RtlULongAdd(result, sizeof(CDVD_KEY_HEADER), &keyLength);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         keyLength = min(keyLength, DeviceExtension->ScratchContext.ScratchBufferSize);
//
//         if (keyLength < 2)
//         {
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         PCDVD_KEY_HEADER            keyBuffer = NULL;
//         CDB                         cdb;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         // prepare the input buffer
//         keyBuffer = (PCDVD_KEY_HEADER)DeviceExtension->ScratchContext.ScratchBuffer;
//
//         // keylength is decremented here by two because the
//         // datalength does not include the header, which is two
//         // bytes.  keylength is immediately incremented later
//         // by the same amount.
//         keyLength -= 2;
//         fourByte = (PFOUR_BYTE)&keyLength;
//         keyBuffer->DataLength[0] = fourByte->Byte1;
//         keyBuffer->DataLength[1] = fourByte->Byte0;
//         keyLength += 2;
//
//         RtlMoveMemory(keyBuffer->Data,
//                       key->KeyData,
//                       key->KeyLength - sizeof(DVD_COPY_PROTECT_KEY));
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.REPORT_KEY.as_mut().OperationCode = SCSIOP_SEND_KEY;
//
//         cdb.SEND_KEY.as_mut().ParameterListLength[0] = fourByte->Byte1;
//         cdb.SEND_KEY.as_mut().ParameterListLength[1] = fourByte->Byte0;
//         cdb.SEND_KEY.as_mut().KeyFormat = (UCHAR)key->KeyType;
//         cdb.SEND_KEY.as_mut().AGID = (UCHAR)key->SessionId;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, OriginalRequest, keyLength, FALSE, &cdb, 12);
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleSetReadAhead(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_STORAGE_SET_READ_AHEAD
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PSTORAGE_SET_READ_AHEAD readAhead = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&readAhead,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG       transferSize = 0;
//         CDB         cdb;
//         ULONG       blockAddress;
//         PFOUR_BYTE  fourByte = (PFOUR_BYTE)&blockAddress;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         cdb.SET_READ_AHEAD.as_mut().OperationCode = SCSIOP_SET_READ_AHEAD;
//
//         blockAddress = (ULONG)(readAhead->TriggerAddress.QuadPart >>
//                                DeviceExtension->SectorShift);
//
//         // Defensive coding. Prefix cannot recognize this usage.
//         UNREFERENCED_PARAMETER(blockAddress);
//
//         cdb.SET_READ_AHEAD.as_mut().TriggerLBA[0] = fourByte->Byte3;
//         cdb.SET_READ_AHEAD.as_mut().TriggerLBA[1] = fourByte->Byte2;
//         cdb.SET_READ_AHEAD.as_mut().TriggerLBA[2] = fourByte->Byte1;
//         cdb.SET_READ_AHEAD.as_mut().TriggerLBA[3] = fourByte->Byte0;
//
//         blockAddress = (ULONG)(readAhead->TargetAddress.QuadPart >>
//                                DeviceExtension->SectorShift);
//
//         cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[0] = fourByte->Byte3;
//         cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[1] = fourByte->Byte2;
//         cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[2] = fourByte->Byte1;
//         cdb.SET_READ_AHEAD.as_mut().ReadAheadLBA[3] = fourByte->Byte0;
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 12);
//
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceHandleSetSpeed(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//       WDF_REQUEST_PARAMETERS   RequestParameters,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//    Handle request of IOCTL_CDROM_SET_SPEED
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request to be handled
//     RequestParameters - request parameter
//     DataLength - transferred data length
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PCDROM_DATA         cdData = &(DeviceExtension->DeviceAdditionalData);
//     PCDROM_SET_SPEED    inputBuffer = NULL;
//
//     PAGED_CODE ();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            RequestParameters.Parameters.DeviceIoControl.as_ref().InputBufferLength,
//                                            (PVOID*)&inputBuffer,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         ULONG               transferSize = 0;
//         CDB                 cdb;
//         CDROM_SPEED_REQUEST requestType = inputBuffer->RequestType;
//
//         ScratchBuffer_BeginUse(DeviceExtension);
//
//         RtlZeroMemory(&cdb, sizeof(CDB));
//         // Set up the CDB
//         if (requestType == CdromSetSpeed)
//         {
//             PCDROM_SET_SPEED speed = inputBuffer;
//
//             cdb.SET_CD_SPEED.as_mut().OperationCode = SCSIOP_SET_CD_SPEED;
//             cdb.SET_CD_SPEED.as_mut().RotationControl = speed->RotationControl;
//             REVERSE_BYTES_SHORT(&cdb.SET_CD_SPEED.as_ref().ReadSpeed, &speed->ReadSpeed);
//             REVERSE_BYTES_SHORT(&cdb.SET_CD_SPEED.as_ref().WriteSpeed, &speed->WriteSpeed);
//         }
//         else
//         {
//             PCDROM_SET_STREAMING    stream = (PCDROM_SET_STREAMING)inputBuffer;
//             PPERFORMANCE_DESCRIPTOR perfDescriptor;
//
//             transferSize = sizeof(PERFORMANCE_DESCRIPTOR);
//
//             perfDescriptor = DeviceExtension->ScratchContext.ScratchBuffer;
//             RtlZeroMemory(perfDescriptor, transferSize);
//
//             perfDescriptor->RandomAccess = stream->RandomAccess;
//             perfDescriptor->Exact = stream->SetExact;
//             perfDescriptor->RestoreDefaults = stream->RestoreDefaults;
//             perfDescriptor->WriteRotationControl = stream->RotationControl;
//
//             REVERSE_BYTES(&perfDescriptor->StartLba,  &stream->StartLba);
//             REVERSE_BYTES(&perfDescriptor->EndLba,    &stream->EndLba);
//             REVERSE_BYTES(&perfDescriptor->ReadSize,  &stream->ReadSize);
//             REVERSE_BYTES(&perfDescriptor->ReadTime,  &stream->ReadTime);
//             REVERSE_BYTES(&perfDescriptor->WriteSize, &stream->WriteSize);
//             REVERSE_BYTES(&perfDescriptor->WriteTime, &stream->WriteTime);
//
//             cdb.SET_STREAMING.as_mut().OperationCode = SCSIOP_SET_STREAMING;
//             REVERSE_BYTES_SHORT(&cdb.SET_STREAMING.as_ref().ParameterListLength, &transferSize);
//
//             // set value in extension by user inputs.
//             cdData->RestoreDefaults = stream->Persistent ? FALSE : TRUE;
//
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DeviceHandleSetSpeed: Restore default speed on media change set to %s\n",
//                        cdData->RestoreDefaults ? "true" : "false"));
//         }
//
//         status = ScratchBuffer_ExecuteCdb(DeviceExtension, Request, transferSize, FALSE, &cdb, 12);
//
//         if (NT_SUCCESS(status))
//         {
//             *DataLength = 0;
//         }
//
//         // nothing to do after the command finishes.
//         ScratchBuffer_EndUse(DeviceExtension);
//     }
//
//     return status;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

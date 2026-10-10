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
use ntkrnl_sys::ddk::{IoAllocateIrp, IoAllocateMdl, IoFreeIrp, IoFreeMdl, IoGetNextIrpStackLocation, IoReportTargetDeviceChangeAsynchronous, IoSetCompletionRoutine, IoSetHardErrorOrVerifyDevice, KEVENT, KeDelayExecutionThread, LARGE_INTEGER, LPCSTR, MmProbeAndLockPages, MmUnlockPages, PCSTR, PKEVENT, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_DEVICE_ID_DESCRIPTOR, PTARGET_DEVICE_CUSTOM_NOTIFICATION, PUNICODE_STRING, TARGET_DEVICE_CUSTOM_NOTIFICATION, STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, UNICODE_STRING};
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID as STORAGE_PROPERTY_ID;
use ntkrnl_sys::ddk::ULONG_PTR;
use ntkrnl_sys::ddk::_MEDIA_TYPE_RemovableMedia as RemovableMedia;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDeviceIdProperty as StorageDeviceIdProperty;
// KeLowerIrql is missing from the i386/arm64 ntkrnl-sys bindgen `ddk` module
// (present only on amd64); declare it locally as the real HAL/ntoskrnl export.
// KIRQL is in scope via the ddk imports and is u8 on every arch.
extern "system" { fn KeLowerIrql(NewIrql: KIRQL); }
extern "C" {
    fn CdromDbgPrint3(format: *const i8, arg1: usize, arg2: usize, arg3: usize) -> u32;
    fn CdromDbgPrint4(format: *const i8, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> u32;
}
use core::ffi::c_void;
use crate::types::_FEATURE_NUMBER_FeatureAACS as FeatureAACS;
use crate::types::_FEATURE_NUMBER_FeatureCDAudioAnalogPlay as FeatureCDAudioAnalogPlay;
use crate::types::_FEATURE_NUMBER_FeatureCdMastering as FeatureCdMastering;
use crate::types::_FEATURE_NUMBER_FeatureCdRead as FeatureCdRead;
use crate::types::_FEATURE_NUMBER_FeatureCdTrackAtOnce as FeatureCdTrackAtOnce;
use crate::types::_FEATURE_NUMBER_FeatureCdrwCAVWrite as FeatureCdrwCAVWrite;
use crate::types::_FEATURE_NUMBER_FeatureCore as FeatureCore;
use crate::types::_FEATURE_NUMBER_FeatureDDCDRWWrite as FeatureDDCDRWWrite;
use crate::types::_FEATURE_NUMBER_FeatureDDCDRWrite as FeatureDDCDRWrite;
use crate::types::_FEATURE_NUMBER_FeatureDDCDRead as FeatureDDCDRead;
use crate::types::_FEATURE_NUMBER_FeatureDefectManagement as FeatureDefectManagement;
use crate::types::_FEATURE_NUMBER_FeatureDiscControlBlocks as FeatureDiscControlBlocks;
use crate::types::_FEATURE_NUMBER_FeatureDvdCPRM as FeatureDvdCPRM;
use crate::types::_FEATURE_NUMBER_FeatureDvdCSS as FeatureDvdCSS;
use crate::types::_FEATURE_NUMBER_FeatureDvdPlusRW as FeatureDvdPlusRW;
use crate::types::_FEATURE_NUMBER_FeatureDvdRead as FeatureDvdRead;
use crate::types::_FEATURE_NUMBER_FeatureDvdRecordableWrite as FeatureDvdRecordableWrite;
use crate::types::_FEATURE_NUMBER_FeatureEmbeddedChanger as FeatureEmbeddedChanger;
use crate::types::_FEATURE_NUMBER_FeatureEnhancedDefectReporting as FeatureEnhancedDefectReporting;
use crate::types::_FEATURE_NUMBER_FeatureFirmwareDate as FeatureFirmwareDate;
use crate::types::_FEATURE_NUMBER_FeatureFormattable as FeatureFormattable;
use crate::types::_FEATURE_NUMBER_FeatureHDDVDRead as FeatureHDDVDRead;
use crate::types::_FEATURE_NUMBER_FeatureHDDVDWrite as FeatureHDDVDWrite;
use crate::types::_FEATURE_NUMBER_FeatureIncrementalStreamingWritable as FeatureIncrementalStreamingWritable;
use crate::types::_FEATURE_NUMBER_FeatureLayerJumpRecording as FeatureLayerJumpRecording;
use crate::types::_FEATURE_NUMBER_FeatureLogicalUnitSerialNumber as FeatureLogicalUnitSerialNumber;
use crate::types::_FEATURE_NUMBER_FeatureMicrocodeUpgrade as FeatureMicrocodeUpgrade;
use crate::types::_FEATURE_NUMBER_FeatureMorphing as FeatureMorphing;
use crate::types::_FEATURE_NUMBER_FeatureMrw as FeatureMrw;
use crate::types::_FEATURE_NUMBER_FeatureMultiRead as FeatureMultiRead;
use crate::types::_FEATURE_NUMBER_FeaturePowerManagement as FeaturePowerManagement;
use crate::types::_FEATURE_NUMBER_FeatureProfileList as FeatureProfileList;
use crate::types::_FEATURE_NUMBER_FeatureRandomReadable as FeatureRandomReadable;
use crate::types::_FEATURE_NUMBER_FeatureRandomWritable as FeatureRandomWritable;
use crate::types::_FEATURE_NUMBER_FeatureRealTimeStreaming as FeatureRealTimeStreaming;
use crate::types::_FEATURE_NUMBER_FeatureRemovableMedium as FeatureRemovableMedium;
use crate::types::_FEATURE_NUMBER_FeatureRestrictedOverwrite as FeatureRestrictedOverwrite;
use crate::types::_FEATURE_NUMBER_FeatureRigidRestrictedOverwrite as FeatureRigidRestrictedOverwrite;
use crate::types::_FEATURE_NUMBER_FeatureSMART as FeatureSMART;
use crate::types::_FEATURE_NUMBER_FeatureSectorErasable as FeatureSectorErasable;
use crate::types::_FEATURE_NUMBER_FeatureTimeout as FeatureTimeout;
use crate::types::_FEATURE_NUMBER_FeatureWriteOnce as FeatureWriteOnce;
use crate::types::_FEATURE_NUMBER_FeatureWriteProtect as FeatureWriteProtect;

use crate::types::MEDIA_LOCK_TYPE_SimpleMediaLock as SimpleMediaLock;
use crate::types::MEDIA_LOCK_TYPE_SecureMediaLock as SecureMediaLock;
use crate::types::MEDIA_LOCK_TYPE_InternalMediaLock as InternalMediaLock;
use crate::types::CDROM_POWER_DOWN_STATE_PowerDownDeviceInitial as PowerDownDeviceInitial;
use crate::types::CDROM_POWER_DOWN_STATE_PowerDownDeviceLocked as PowerDownDeviceLocked;
use crate::types::CDROM_POWER_DOWN_STATE_PowerDownDeviceQuiesced as PowerDownDeviceQuiesced;
use crate::types::CDROM_POWER_DOWN_STATE_PowerDownDeviceFlushed as PowerDownDeviceFlushed;
use crate::types::CDROM_POWER_DOWN_STATE_PowerDownDeviceStopped as PowerDownDeviceStopped;
use crate::types::DVD_STRUCTURE_FORMAT_DvdCopyrightDescriptor as DvdCopyrightDescriptor;
use crate::types::_DVD_KEY_TYPE_DvdGetRpcKey as DvdGetRpcKey;
use crate::types::_DVD_KEY_TYPE_DvdSetRpcKey as DvdSetRpcKey;

// NTSTATUS codes not present in the kernel constants crate (NTSTATUS = i32).
const STATUS_INVALID_DEVICE_STATE: i32 = 0xC0000184u32 as i32;
const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32;
// KMDF WDF_REQUEST_REUSE_FLAGS::WDF_REQUEST_REUSE_NO_FLAGS == 0
const WDF_REQUEST_REUSE_NO_FLAGS: u32 = 0;
// WPP trace flag (gates tracing only; any distinct nonzero value is fine).
const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// SRB_STATUS(x) macro: mask out the status-modifier bits.
// Returns u32 so it compares directly against SRB_STATUS_SUCCESS/_PENDING (u32).
#[inline]
fn SRB_STATUS(status: u8) -> u32 {
    (status as u32) & SRB_STATUS_STATUS_MASK
}
const SRB_STATUS_STATUS_MASK: u32 = 0x3f;

extern "C" {
    fn RequestSenseInfoInterpret( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Request: *mut core::ffi::c_void, Srb: *mut SCSI_REQUEST_BLOCK, RetryAttempt: ULONG, Status: *mut NTSTATUS, RetryInterval: *mut LARGE_INTEGER, ) -> BOOLEAN;
    fn FREE_POOL(ptr: *mut core::ffi::c_void);
    fn DeviceMarkActive(DeviceExtension: PCDROM_DEVICE_EXTENSION, IsActive: BOOLEAN, SetIdleTimeout: BOOLEAN,);
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER(deviceExtension: *mut CDROM_DEVICE_EXTENSION, Srb: *mut SCSI_REQUEST_BLOCK);
    fn PORT_ALLOCATED_SENSE(deviceExtension: *mut CDROM_DEVICE_EXTENSION, Srb: *mut SCSI_REQUEST_BLOCK) -> BOOLEAN;
}
extern "system" {
    fn NT_ASSERT(expr: i32);
    fn PoQueryWatchdogTime(DeviceObject: *mut core::ffi::c_void, SecondsRemaining: *mut ULONG) -> u8;
    fn ScratchBuffer_ExecuteCdb(DeviceExtension: PCDROM_DEVICE_EXTENSION, OriginalRequest: *mut core::ffi::c_void, TransferSize: u32, GetDataFromDevice: u8, Cdb: PCDB, OprationLength: u8) -> NTSTATUS;
}
unsafe extern "C" {
    fn PowerContextReuseRequest(DeviceExtension: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS;
}
unsafe extern "system" {
    fn KeQueryTickCount(CurrentCount: *mut LARGE_INTEGER);
}
extern "system" {
    fn RequestGetContext(Request: WDFREQUEST) -> *mut CDROM_REQUEST_CONTEXT;
    fn DeviceGetExtension(Device: WDFDEVICE) -> *mut CDROM_DEVICE_EXTENSION;
    fn KeGetCurrentIrql() -> KIRQL;
    static GUID_DEVINTERFACE_CDROM: GUID;
    static MOUNTDEV_MOUNTED_DEVICE_GUID: GUID;
}
unsafe extern "C" {
    fn RtlULongAdd(ul1: u32, ul2: u32, pulRes: *mut u32) -> NTSTATUS;
    fn InterlockedIncrement(p: *mut i32) -> i32;
    fn WHICH_BIT(bps: u32, shift: *mut u8);
}
unsafe extern "C" {
    fn DvdSendKey( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, param2: *mut core::ffi::c_void, CopyProtectKey: *mut DVD_COPY_PROTECT_KEY, KeyLength: usize, BytesReturned: *mut usize, ) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceRetrieveDescriptor( Device: WDFDEVICE, PropertyId: *mut STORAGE_PROPERTY_ID, Descriptor: *mut PSTORAGE_DESCRIPTOR_HEADER, ) -> NTSTATUS;
}
// C min() is a macro; extern "system" decl = i386 stdcall/cdecl mismatch vs
// ddk_shims' extern "C" def → stack corruption. Local wrapper instead.
#[inline] fn min(a: u32, b: u32) -> u32 { core::cmp::min(a, b) }
extern "system" {
    fn ScratchBuffer_BeginUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn ScratchBuffer_EndUse(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
    fn WDF_REQUEST_PARAMETERS_INIT(params: *mut WDF_REQUEST_PARAMETERS);
    fn WDF_REQUEST_SEND_OPTIONS_INIT(Options: *mut WDF_REQUEST_SEND_OPTIONS, Flags: u32);
    fn RtlInitUnicodeString(DestinationString: *mut UNICODE_STRING, SourceString: *const u16);
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    static GUID_IO_CDROM_EXCLUSIVE_UNLOCK: GUID;
    static CDROM_REQUEST_CONTEXT_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
    fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(attributes: *mut WDF_OBJECT_ATTRIBUTES, context_type: *const core::ffi::c_void);
    fn WDF_REQUEST_REUSE_PARAMS_INIT(params: *mut WDF_REQUEST_REUSE_PARAMS, flags: u32, status: i32);
    fn PagedCode();
    fn __SEH2_ProbeAndLockPages(mdl: *mut MDL, mode: u32, access: u32) -> i32;
    fn FileObjectGetContext(FileObject: WDFFILEOBJECT) -> *mut FILE_OBJECT_CONTEXT;
}
unsafe extern "C" {
    fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN;
    fn InterlockedCompareExchange(Destination: *mut i32, Exchange: i32, Comperand: i32) -> i32;
    fn REVERSE_BYTES_SHORT(dest: *mut u16, src: *const u32);
}
unsafe extern "C" {
    fn EXCLUSIVE_MODE(cdData: *mut CDROM_DATA) -> i32;
}
unsafe extern "C" {
    fn EXCLUSIVE_OWNER(cdData: *mut CDROM_DATA, FileObject: *mut core::ffi::c_void) -> u8;
}
unsafe extern "C" {
    fn DeviceSetMediaChangeStateEx(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, newState: i32, previousState: *mut i32);
}
unsafe extern "C" {
    fn IsVolumeMounted(DeviceObject: *mut DEVICE_OBJECT) -> u8;
}
unsafe extern "C" {
    fn RequestSetupMcnSyncIrp(DeviceExtension: *mut CDROM_DEVICE_EXTENSION);
}
extern "C" {
    fn DvdStartSessionReadKey( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, IoControlCode: u32, param3: *mut core::ffi::c_void, CopyProtectKey: *mut DVD_COPY_PROTECT_KEY, InKeyLength: usize, OutCopyProtectKey: *mut DVD_COPY_PROTECT_KEY, OutKeyLength: usize, BytesReturned: *mut usize, ) -> NTSTATUS;
    fn ReadDvdStructure( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, param2: *mut core::ffi::c_void, ReadStructure: *mut DVD_READ_STRUCTURE, ReadLength: usize, ReadStructureOut: *mut DVD_READ_STRUCTURE, DescriptorLength: usize, BytesReturned: *mut usize, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// Wrapper so the static of raw C-string pointers can be `Sync` (raw pointers
// are not Sync by default). The pointers are to 'static read-only literals.
struct SyncStrTable([*const i8; 3]);
unsafe impl Sync for SyncStrTable {}
pub static LockTypeStrings: SyncStrTable = SyncStrTable([
    b"Simple\0".as_ptr() as *const i8,
    b"Secure\0".as_ptr() as *const i8,
    b"Internal\0".as_ptr() as *const i8,
]);

const KEY_READ: u32 = 0x20019;
const KEY_WRITE: u32 = 0x20006;
const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
const PLUGPLAY_REGKEY_DRIVER: u32 = 2;

const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32;

// Forward declarations for unknown functions and constants used by this function.

// ---------------------------------------------------------------------------
// Function translation
// ---------------------------------------------------------------------------

const WDF_NO_OBJECT_ATTRIBUTES: PWDF_OBJECT_ATTRIBUTES = core::ptr::null_mut(); // NULL for object attributes handle
const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

const CdRomDeviceInterface: i32 = 0;
const MountedDeviceInterface: i32 = 1;

// Forward declaration: opaque structs used by this function (caller provides definitions)

unsafe extern "C" {
    fn strlen(s: *const i8) -> usize;
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
}

// Extern declarations for functions not in available imports

// Constants used by this function
#[allow(dead_code)]
const IOCTL_MCN_SYNC_FAKE_IOCTL: u32 = 0x0007C0C8;  // = IOCTL_DISK_UPDATE_DRIVE_SIZE
// (cdrom.h:181, CTL_CODE(IOCTL_DISK_BASE, 0x32, METHOD_BUFFERED, READ|WRITE)). NOT a
// placeholder: autorun.rs WRITES this into IoControlCode and cdrom.rs matches on it,
// so an invented value means the MCN sync IRP is never recognised.
#[allow(dead_code)]
const WdfRequestTypeDeviceControl: u32 = 7; // Common KMDF value
const WdfRequestTypeRead: u32 = 3;
const WdfRequestTypeWrite: u32 = 4;
#[allow(dead_code)]
const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

type NTSTATUS = i32;
type BOOLEAN = u8;
type ULONG = u32;




const WDF_REQUEST_SEND_OPTION_SYNCHRONOUS: u32 = 0x02;

/// Routine Description:
///     retrieve device parameter from registry.
///
/// Arguments:
///     DeviceExtension - device context.
///     SubkeyName - name of subkey
///     ParameterName - the registry parameter to be retrieved
///
/// Return Value:
///     ParameterValue - registry value retrieved

const MediaUnknown: i32 = 0;
const MediaUnavailable: i32 = 1;
const DO_VERIFY_VOLUME: u32 = 0x02;
const CdromMmcUpdateRequired: i32 = 1;
const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// Forward declarations for unknown external functions

// Constants for trace flags – typical kernel values

// Helper to compute RTL_SIZEOF_THROUGH_FIELD – same as Windows kernel macro.
// This is inlined as a constant expression using offset_of! and size_of.

/// _IRQL_requires_max_(APC_LEVEL)

// Constants (approximate values; some are typical Windows DDK constants)
// cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;
const CDROM_TAG_COMPLETION_CONTEXT: u32 = 0x63436353; // placeholder, replace with actual tag

const START_UNIT_TIMEOUT: u32 = 0xf0; // typical timeout in seconds  // was 60 (LLM-invented); bindgen-correct

// SCSI constants
const SRB_FUNCTION_EXECUTE_SCSI: u8 = 0x00;
const SRB_FLAGS_NO_DATA_TRANSFER: u32 = 0x0;  // was 0x00000004 (LLM-invented); bindgen-correct
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x00000008;
const SCSIOP_START_STOP_UNIT: u8 = 0x1B;
const IOCTL_SCSI_EXECUTE_NONE: u32 = 0x001B0013;

// Tracing constants (typical WPP values)

// NTSTATUS
const STATUS_INTERNAL_ERROR: i32 = (0xC00000E5u32 as i32);

// Extern functions / data

// Translate function

const SRB_FUNCTION_FLUSH_QUEUE: u8 = 0x15;  // was 0x02 (LLM-invented); bindgen-correct
const WDF_REQUEST_SEND_OPTION_IGNORE_TARGET_STATE: u32 = 0x00000004;

const CDROM_READ_CAPACITY_TIMEOUT: u32 = 0x14; // typical value, if incorrect caller should override  // was 10 (LLM-invented); bindgen-correct
const SCSIOP_READ_CAPACITY: u8 = 0x25;

const CDROM_TAG_STREAM: u32 = 0x4F434353;
// cdrom.h:68: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNxCacheAligned: u32 = 4;
const SCSIOP_SET_STREAMING: u8 = 0xB6;
#[no_mangle]
pub unsafe extern "C" fn RequestSetReceivedTime(Request: WDFREQUEST) {
    let requestContext: *mut CDROM_REQUEST_CONTEXT = RequestGetContext(Request);
    let mut temp: LARGE_INTEGER = core::mem::zeroed();
    KeQueryTickCount(&mut temp);
    (*requestContext).TimeReceived = temp;
    return;
}
#[no_mangle]
pub unsafe extern "C" fn RequestSetSentTime(Request: WDFREQUEST) {
    let requestContext = RequestGetContext(Request);
    let mut temp: LARGE_INTEGER = core::mem::zeroed();

    KeQueryTickCount(&mut temp as *mut LARGE_INTEGER);

    if (*requestContext).TimeSentDownFirstTime.QuadPart == 0 {
        (*requestContext).TimeSentDownFirstTime = temp;
    }

    (*requestContext).TimeSentDownLasttTime = temp;

    if !(*requestContext).OriginalRequest.is_null() {
        let originalRequestContext = RequestGetContext((*requestContext).OriginalRequest);

        if (*originalRequestContext).TimeSentDownFirstTime.QuadPart == 0 {
            (*originalRequestContext).TimeSentDownFirstTime = temp;
        }

        (*originalRequestContext).TimeSentDownLasttTime = temp;
    }
}
#[no_mangle]
pub unsafe extern "C" fn RequestClearSendTime(Request: WDFREQUEST) {
    let requestContext = RequestGetContext(Request);
    (*requestContext).TimeSentDownFirstTime.QuadPart = 0;
    (*requestContext).TimeSentDownLasttTime.QuadPart = 0;
}
#[no_mangle]
pub unsafe extern "system" fn DeviceGetParameter(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    SubkeyName: *mut u16,
    ParameterName: *mut u16,
    ParameterValue: *mut u32,
) {
    // Forward declarations for missing externs

    const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
    const PLUGPLAY_REGKEY_DRIVER: u32 = 2;
    const WDF_NO_OBJECT_ATTRIBUTES: PWDF_OBJECT_ATTRIBUTES = core::ptr::null_mut();

    // PAGED_CODE();
    let mut status: NTSTATUS;
    let mut rootKey: WDFKEY = core::ptr::null_mut();
    let mut subKey: WDFKEY = core::ptr::null_mut();
    let mut registrySubKeyName: UNICODE_STRING = core::mem::zeroed();
    let mut registryValueName: UNICODE_STRING = core::mem::zeroed();
    let mut defaultParameterValue: u32;

    RtlInitUnicodeString(&raw mut registryValueName, ParameterName as *const u16);

    if !SubkeyName.is_null() {
        RtlInitUnicodeString(&raw mut registrySubKeyName, SubkeyName as *const u16);
    }

    // open the hardware key
    status = WdfDeviceOpenRegistryKey(
        (*DeviceExtension).Device,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_READ,
        WDF_NO_OBJECT_ATTRIBUTES,
        &raw mut rootKey,
    );

    // open the sub key
    if NT_SUCCESS(status) && !SubkeyName.is_null() {
        status = WdfRegistryOpenKey(
            rootKey,
            &registrySubKeyName as *const UNICODE_STRING,
            KEY_READ,
            WDF_NO_OBJECT_ATTRIBUTES,
            &raw mut subKey,
        );

        if !NT_SUCCESS(status) {
            WdfRegistryClose(rootKey);
            rootKey = core::ptr::null_mut();
        }
    }

    if NT_SUCCESS(status) && !rootKey.is_null() {
        defaultParameterValue = *ParameterValue;

        status = WdfRegistryQueryULong(
            if !subKey.is_null() { subKey } else { rootKey },
            &registryValueName as *const UNICODE_STRING,
            ParameterValue,
        );

        if !NT_SUCCESS(status) {
            *ParameterValue = defaultParameterValue; // use default value
        }
    }

    // close what we open
    if !subKey.is_null() {
        WdfRegistryClose(subKey);
        subKey = core::ptr::null_mut();
    }
    if !rootKey.is_null() {
        WdfRegistryClose(rootKey);
        rootKey = core::ptr::null_mut();
    }

    // Windows 2000 SP3 uses the driver-specific key, so look in there
    if !NT_SUCCESS(status) {
        // open the software key
        status = WdfDeviceOpenRegistryKey(
            (*DeviceExtension).Device,
            PLUGPLAY_REGKEY_DRIVER,
            KEY_READ,
            WDF_NO_OBJECT_ATTRIBUTES,
            &raw mut rootKey,
        );

        // open the sub key
        if NT_SUCCESS(status) && !SubkeyName.is_null() {
            status = WdfRegistryOpenKey(
                rootKey,
                &registrySubKeyName as *const UNICODE_STRING,
                KEY_READ,
                WDF_NO_OBJECT_ATTRIBUTES,
                &raw mut subKey,
            );

            if !NT_SUCCESS(status) {
                WdfRegistryClose(rootKey);
                rootKey = core::ptr::null_mut();
            }
        }

        if NT_SUCCESS(status) && !rootKey.is_null() {
            defaultParameterValue = *ParameterValue;

            status = WdfRegistryQueryULong(
                if !subKey.is_null() { subKey } else { rootKey },
                &registryValueName as *const UNICODE_STRING,
                ParameterValue,
            );

            if !NT_SUCCESS(status) {
                *ParameterValue = defaultParameterValue; // use default value
            } else {
                // Migrate the value over to the device-specific key
                DeviceSetParameter(DeviceExtension, SubkeyName, ParameterName, *ParameterValue);
            }
        }

        // close what we open
        if !subKey.is_null() {
            WdfRegistryClose(subKey);
        }
        if !rootKey.is_null() {
            WdfRegistryClose(rootKey);
        }
    }

    // implicitly returns ()
}
#[no_mangle]
pub unsafe extern "system" fn DeviceSetParameter(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    SubkeyName: *mut u16,
    ParameterName: *mut u16,
    ParameterValue: u32,
) -> i32 {
    let mut status: i32;
    let mut root_key: WDFKEY = core::ptr::null_mut();
    let mut sub_key: WDFKEY = core::ptr::null_mut();
    let mut registry_subkey_name: UNICODE_STRING = core::mem::zeroed();
    let mut registry_value_name: UNICODE_STRING = core::mem::zeroed();

    PAGED_CODE!();

    RtlInitUnicodeString(&mut registry_value_name, ParameterName);

    if !SubkeyName.is_null() {
        RtlInitUnicodeString(&mut registry_subkey_name, SubkeyName);
    }

    // open the hardware key
    status = WdfDeviceOpenRegistryKey(
        (*DeviceExtension).Device,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut::<_WDF_OBJECT_ATTRIBUTES>(), // WDF_NO_OBJECT_ATTRIBUTES
        &mut root_key,
    );

    // open the sub key
    if NT_SUCCESS(status) && !SubkeyName.is_null() {
        status = WdfRegistryOpenKey(
            root_key,
            &registry_subkey_name,
            KEY_READ | KEY_WRITE,
            core::ptr::null_mut::<_WDF_OBJECT_ATTRIBUTES>(),
            &mut sub_key,
        );

        if !NT_SUCCESS(status) {
            WdfRegistryClose(root_key);
            root_key = core::ptr::null_mut();
        }
    }

    if NT_SUCCESS(status) && !root_key.is_null() {
        status = WdfRegistryAssignULong(
            if !sub_key.is_null() { sub_key } else { root_key },
            &registry_value_name,
            ParameterValue,
        );
    }

    // close what we open
    if !sub_key.is_null() {
        WdfRegistryClose(sub_key);
    }

    if !root_key.is_null() {
        WdfRegistryClose(root_key);
    }

    status
}
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn DeviceSendRequestSynchronously(
    device: WDFDEVICE,
    request: WDFREQUEST,
    request_formated: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let device_extension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(device);
    let mut request_cancelled: BOOLEAN = 0;
    let request_context: PCDROM_REQUEST_CONTEXT = RequestGetContext(request);

    PAGED_CODE!();

    if request_formated == 0 {
        WdfRequestFormatRequestUsingCurrentType(request);
    }

    if !(*request_context).OriginalRequest.is_null() {
        request_cancelled = WdfRequestIsCanceled((*request_context).OriginalRequest);
    }

    if request_cancelled == 0 {
        status = RequestSend(
            device_extension,
            request,
            (*device_extension).IoTarget,
            WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
            core::ptr::null_mut(),
        );
    } else {
        status = STATUS_CANCELLED;
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn DeviceSendSrbSynchronously(
    Device: WDFDEVICE,
    Srb: *mut SCSI_REQUEST_BLOCK,
    BufferAddress: *mut core::ffi::c_void,
    BufferLength: u32,
    WriteToDevice: u8,
    OriginalRequest: WDFREQUEST,
) -> i32 {
    #[repr(C)]
    struct LARGE_INTEGER {
        QuadPart: i64,
    }

    const SENSE_BUFFER_SIZE: u32 = 18; // placeholder
    const CDROM_TAG_SENSE_INFO: u32 = 0x49436353; // 'SENS' placeholder
    // cdrom.h:68: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
    // `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
    // Windows value (0x200/0x204) is not what any TU here compiles.
    const NonPagedPoolNxCacheAligned: u32 = 4;
    const SRB_FUNCTION_EXECUTE_SCSI: u32 = 0x00;
    const SRB_SIMPLE_TAG_REQUEST: u32 = 0x20;
    const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x00000008;
    const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x100;  // was 0x00000010 (LLM-invented); bindgen-correct
    const SRB_FLAGS_DATA_OUT: u32 = 0x80;  // was 0x00000004 (LLM-invented); bindgen-correct
    const SRB_FLAGS_DATA_IN: u32 = 0x40;  // was 0x00000008 (LLM-invented); bindgen-correct
    const IOCTL_SCSI_EXECUTE_OUT: u32 = 0x1b0012;  // was 0x4D006 (LLM-invented); bindgen-correct
    const IOCTL_SCSI_EXECUTE_IN: u32 = 0x1b0011;  // was 0x4D008 (LLM-invented); bindgen-correct
    const IOCTL_SCSI_EXECUTE_NONE: u32 = 0x001B0013;
    const IRP_MJ_SCSI: u8 = 0x0F; // IRP_MJ_SCSI
    const SRB_STATUS_PENDING: u8 = 0x00;
    const SRB_STATUS_QUEUE_FROZEN: u8 = 0x40;  // was 0x01 (LLM-invented); bindgen-correct
    const SRB_STATUS_SUCCESS: u8 = 0x01;
    const STATUS_SUCCESS: i32 = 0;
    const STATUS_INSUFFICIENT_RESOURCES: i32 = (0xC000009Au32 as i32);
    const STATUS_CANCELLED: i32 = (0xC0000120u32 as i32);
    const STATUS_PENDING: i32 = 0x103;
    const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
    const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const WDF_REQUEST_REUSE_NO_FLAGS: u32 = 0;
    const IoReadAccess: u32 = 0;
    const IoWriteAccess: u32 = 1;

    let mut status: i32;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);
    let fdoData: *mut CDROM_PRIVATE_FDO_DATA = (*deviceExtension).PrivateFdoData;
    let mut senseInfoBuffer: *mut u8;
    let mut retryCount: u32 = 0;
    let mut retry: u8;
    let mut ioctlCode: u32;
    let mut request: WDFREQUEST = core::ptr::null_mut();
    let mut irp: *mut IRP = core::ptr::null_mut();
    let mut nextStack: *mut IO_STACK_LOCATION;
    let mut mdlAddress: *mut MDL = core::ptr::null_mut();
    let mut memoryLocked: u8 = 0;
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let zpoddInfo: *mut ZERO_POWER_ODD_INFO = (*deviceExtension).ZeroPowerODDInfo;

    PagedCode();

    // NOTE: asserts disabled for translation
    // NT_ASSERT(KeGetCurrentIrql() < DISPATCH_LEVEL);

    let cleanup = 'flow: {
        // 1. allocate SenseBuffer and initiate Srb common fields
        (*Srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
        (*Srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*Srb).SenseInfoBufferLength = SENSE_BUFFER_SIZE as u8;

        senseInfoBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
                                                SENSE_BUFFER_SIZE as usize,
                                                CDROM_TAG_SENSE_INFO) as *mut u8;
        if senseInfoBuffer.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
                       b"DeviceSendSrbSynchronously: Can't allocate MDL\n\0".as_ptr() as *const i8);
            break 'flow;
        }
        (*Srb).SenseInfoBuffer = senseInfoBuffer as *mut core::ffi::c_void;
        (*Srb).DataBuffer = BufferAddress;

        if (*Srb).TimeOutValue == 0 {
            (*Srb).TimeOutValue = (*deviceExtension).TimeOutValue;
        }

        // 2. Create Request object
        WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&mut attributes,
                                                core::ptr::addr_of!(crate::ddk_shims::CDROM_REQUEST_CONTEXT) as *const core::ffi::c_void); // CDROM_REQUEST_CONTEXT type-info (null → NULL deref 0x1E)
        status = WdfRequestCreate(&mut attributes,
                                  (*deviceExtension).IoTarget,
                                  &mut request);
        if status < 0 {
            TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
                       b"DeviceSendSrbSynchronously: Can't create request: %lx\n\0".as_ptr() as *const i8,
                       status);
            break 'flow;
        }
        irp = WdfRequestWdmGetIrp(request);

        // 3. Build an MDL for the data buffer
        if !BufferAddress.is_null() {
            mdlAddress = IoAllocateMdl(BufferAddress,
                                       BufferLength,
                                       0, 0,
                                       irp);
            if mdlAddress.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
                           b"DeviceSendSrbSynchronously: Can't allocate MDL\n\0".as_ptr() as *const i8);
                break 'flow;
            }

            // SEH2 replacement: external wrapper that catches exceptions
            status = __SEH2_ProbeAndLockPages(mdlAddress,
                                              0u32,
                                              if WriteToDevice != 0 { IoReadAccess } else { IoWriteAccess });
            if status != 0 {
                TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
                           b"DeviceSendSrbSynchronously: Exception %lx locking buffer\n\0".as_ptr() as *const i8,
                           status);
                break 'flow;
            }
            memoryLocked = 1;
        }

        // ioctlCode is loop-invariant (depends only on BufferAddress/WriteToDevice) and MUST be set
        // before WdfIoTargetFormatRequestForInternalIoctlOthers below. The translation originally
        // assigned it in step 4.2 (AFTER the format call), so every send formatted with ioctlCode=0
        // → the I/O target never executed the SRB (SrbStatus stayed PENDING, DataTransferLength 0).
        ioctlCode = if BufferAddress.is_null() {
            IOCTL_SCSI_EXECUTE_NONE
        } else if WriteToDevice != 0 {
            IOCTL_SCSI_EXECUTE_OUT
        } else {
            IOCTL_SCSI_EXECUTE_IN
        };

        // 4. Format Srb, Irp; Send request and retry when necessary
        loop {
            retry = 0;

            // 4.1 reuse request object; set originalRequest field
            {
                let mut params: WDF_REQUEST_REUSE_PARAMS = core::mem::zeroed();
                let requestContext: *mut CDROM_REQUEST_CONTEXT;

                if !(*irp).MdlAddress.is_null() {
                    (*irp).MdlAddress = core::ptr::null_mut();
                }

                WDF_REQUEST_REUSE_PARAMS_INIT(&mut params,
                                              WDF_REQUEST_REUSE_NO_FLAGS,
                                              STATUS_SUCCESS);
                status = WdfRequestReuse(request, &mut params);
                if status < 0 {
                    TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
                               b"DeviceSendSrbSynchronously: WdfRequestReuse failed, %!STATUS!\n\0".as_ptr() as *const i8,
                               status);
                    break; // exit the loop
                }

                status = WdfIoTargetFormatRequestForInternalIoctlOthers(
                    (*deviceExtension).IoTarget,
                    request,
                    ioctlCode,
                    core::ptr::null_mut(), core::ptr::null_mut(),
                    core::ptr::null_mut(), core::ptr::null_mut(),
                    core::ptr::null_mut(), core::ptr::null_mut());
                if status < 0 {
                    TracePrint!(TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
                               b"DeviceSendSrbSynchronously: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n\0".as_ptr() as *const i8,
                               status);
                    break; // exit the loop
                }

                requestContext = RequestGetContext(request);
                (*requestContext).OriginalRequest = OriginalRequest;
            }

            // 4.2 Format Srb and Irp
            {
                (*Srb).OriginalRequest = irp as *mut _;
                (*Srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
                (*Srb).DataTransferLength = BufferLength;
                (*Srb).SrbFlags = (*deviceExtension).SrbFlags;

                (*Srb).SrbFlags |= SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
                (*Srb).SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;

                if !BufferAddress.is_null() {
                    if WriteToDevice != 0 {
                        (*Srb).SrbFlags |= SRB_FLAGS_DATA_OUT;
                        ioctlCode = IOCTL_SCSI_EXECUTE_OUT;
                    } else {
                        (*Srb).SrbFlags |= SRB_FLAGS_DATA_IN;
                        ioctlCode = IOCTL_SCSI_EXECUTE_IN;
                    }
                } else {
                    ioctlCode = IOCTL_SCSI_EXECUTE_NONE;
                }

                (*Srb).ScsiStatus = 0;
                (*Srb).SrbStatus = 0;
                (*Srb).NextSrb = core::ptr::null_mut();

                (*irp).MdlAddress = mdlAddress;
                nextStack = IoGetNextIrpStackLocation(irp);
                (*nextStack).MajorFunction = IRP_MJ_SCSI;

                // union access: DeviceIoControl part
                let params_ioctl = &mut (*nextStack).Parameters.DeviceIoControl;
                params_ioctl.IoControlCode = ioctlCode;

                // union access: Scsi part
                let params_scsi = &mut (*nextStack).Parameters.Scsi;
                params_scsi.Srb = Srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;
            }

            // 4.3 send Request to lower driver
            status = DeviceSendRequestSynchronously(Device, request, 1);

            if status != STATUS_CANCELLED {
                // NT_ASSERTs skipped for translation
                // NT_ASSERT(SRB_STATUS((*Srb).SrbStatus) != SRB_STATUS_PENDING);
                // NT_ASSERT(status != STATUS_PENDING);
                // NT_ASSERT(!((*Srb).SrbStatus & SRB_STATUS_QUEUE_FROZEN != 0));

                // 4.4 error process
                let srb_status = core::ptr::read_volatile(core::ptr::addr_of!((*Srb).SrbStatus));
                if (srb_status & 0x3f) != SRB_STATUS_SUCCESS {
                    let mut retryIntervalIn100ns: i64 = 0;

                    retry = RequestSenseInfoInterpret(deviceExtension,
                                                      request as *mut core::ffi::c_void,
                                                      Srb,
                                                      retryCount,
                                                      &mut status,
                                                      &mut retryIntervalIn100ns as *mut i64 as *mut ntkrnl_sys::ddk::LARGE_INTEGER);
                    if retry != 0 {
                        let mut t = LARGE_INTEGER { QuadPart: -retryIntervalIn100ns };
                        retryCount += 1;
                        KeDelayExecutionThread(0i8, 0, &mut t as *mut _ as *mut ntkrnl_sys::ddk::LARGE_INTEGER);
                    }
                } else {
                    // Request succeeded
                    (*fdoData).LoggedTURFailureSinceLastIO = 0; // FALSE
                    status = STATUS_SUCCESS;
                    retry = 0;
                }
            }

            if retry == 0 {
                break;
            }
        } // end retry loop

        // zpoddInfo check
        let srb_status = core::ptr::read_volatile(core::ptr::addr_of!((*Srb).SrbStatus));
        if !zpoddInfo.is_null() && ((*zpoddInfo).MonitorStartStopUnit != 0) && ((srb_status & 0x3f) == SRB_STATUS_SUCCESS) {
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                       b"DeviceSendSrbSynchronously: soft eject detected, device marked as active\n\0".as_ptr() as *const i8);
            DeviceMarkActive(deviceExtension, 1, 0);
        }

        // port allocated sense check (runs only if no early goto)
        if PORT_ALLOCATED_SENSE(deviceExtension, Srb) != 0 {
            FREE_PORT_ALLOCATED_SENSE_BUFFER(deviceExtension, Srb);
        }
    }; // 'flow block end

    // 5. Release all allocated resources (Executes always)
    if !senseInfoBuffer.is_null() {
        FREE_POOL(senseInfoBuffer as *mut core::ffi::c_void);
    }

    (*Srb).SenseInfoBuffer = core::ptr::null_mut();
    (*Srb).SenseInfoBufferLength = 0;

    if !mdlAddress.is_null() {
        if memoryLocked != 0 {
            MmUnlockPages(mdlAddress);
        }
        IoFreeMdl(mdlAddress);
        (*irp).MdlAddress = core::ptr::null_mut();
    }

    if !request.is_null() {
        WdfObjectDelete(request as WDFOBJECT);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSendNotification(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Guid: *const GUID,
    ExtraDataSize: u32,
    ExtraData: *mut core::ffi::c_void,
) {
    const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
    const TRACE_FLAG_MCN: u32 = 12; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    // cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
    // `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
    // Windows value (0x200/0x204) is not what any TU here compiles.
    const NonPagedPoolNx: u32 = 0;
    const CDROM_TAG_NOTIFICATION: u32 = 0x6F436353;

    let mut notification: *mut TARGET_DEVICE_CUSTOM_NOTIFICATION;
    let mut requiredSize: u32 = 0;
    let status: NTSTATUS;

    status = RtlULongAdd(
        (core::mem::size_of::<TARGET_DEVICE_CUSTOM_NOTIFICATION>() - 1u32 as usize) as u32,
        ExtraDataSize,
        &mut requiredSize,
    );

    if !NT_SUCCESS(status) || requiredSize > 0x0000ffff {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"Error sending event: size too large! (%x)\n\0".as_ptr() as *const i8,
            requiredSize,
        );
        return;
    }

    notification = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        requiredSize as usize,
        CDROM_TAG_NOTIFICATION,
    ) as *mut TARGET_DEVICE_CUSTOM_NOTIFICATION;

    if notification.is_null() {
        return;
    }

    core::ptr::write_bytes(notification as *mut u8, 0, requiredSize as usize);
    (*notification).Version = 1;
    (*notification).Size = requiredSize as u16;
    (*notification).FileObject = core::ptr::null_mut();
    (*notification).NameBufferOffset = -1i32;
    core::ptr::copy_nonoverlapping(Guid, &raw mut (*notification).Event, 1);

    if !ExtraData.is_null() {
        core::ptr::copy_nonoverlapping(
            ExtraData as *const u8,
            &raw mut (*notification).CustomDataBuffer as *mut u8,
            ExtraDataSize as usize,
        );
    }

    IoReportTargetDeviceChangeAsynchronous(
        (*DeviceExtension).LowerPdo,
        notification as *mut core::ffi::c_void,
        None,
        core::ptr::null_mut(),
    );

    FREE_POOL(notification as *mut core::ffi::c_void);

    return;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSendStartUnit(Device: WDFDEVICE) {
    // local variables (direct transliteration of C)
    let mut status: NTSTATUS = STATUS_SUCCESS; // STATUS_SUCCESS = 0
    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION;
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut startUnitRequest: WDFREQUEST = core::ptr::null_mut();
    let mut inputMemory: WDFMEMORY = core::ptr::null_mut();
    let mut context: *mut COMPLETION_CONTEXT = core::ptr::null_mut();
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: *mut CDB;

    deviceExtension = DeviceGetExtension(Device);

    if NT_SUCCESS(status) {
        context = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<COMPLETION_CONTEXT>(),
            CDROM_TAG_COMPLETION_CONTEXT) as *mut COMPLETION_CONTEXT;

        if context.is_null() {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                b"DeviceSendStartUnit: Failed to allocate completion context\n\0".as_ptr() as *const i8,
            );
            status = STATUS_INTERNAL_ERROR;
        }
    }

    if NT_SUCCESS(status) {
        (*context).Device = Device;
        srb = &mut (*context).Srb; // Srb field is inside COMPLETION_CONTEXT (by value)

        // Zero out srb
        core::ptr::write_bytes(srb as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*srb).TimeOutValue = START_UNIT_TIMEOUT;
        (*srb).SrbFlags = SRB_FLAGS_NO_DATA_TRANSFER | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;

        (*srb).CdbLength = 6;
        cdb = &mut (*srb).Cdb as *mut _ as *mut CDB;

        (*cdb).START_STOP.as_mut().OperationCode = SCSIOP_START_STOP_UNIT;
        (*cdb).START_STOP.as_mut().set_Start(1);
        (*cdb).START_STOP.as_mut().set_Immediate(0);
        (*cdb).START_STOP.as_mut().set_LogicalUnitNumber((*srb).Lun);

        // WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes, CDROM_REQUEST_CONTEXT);
        // Expand macro: initialize attributes as zeroed, then set ContextTypeInfo
        core::ptr::write_bytes(
            (core::ptr::addr_of_mut!(attributes)) as *mut u8,
            0,
            core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>());
        attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
        // The INIT macro also sets both of these to InheritFromParent (1); left at 0
        // (Invalid), FxValidateObjectAttributes rejected the request and START UNIT was
        // never sent.
        attributes.ExecutionLevel = 1; // WdfExecutionLevelInheritFromParent
        attributes.SynchronizationScope = 1; // WdfSynchronizationScopeInheritFromParent
        attributes.ContextTypeInfo = &CDROM_REQUEST_CONTEXT_TYPE_INFO as *const _ as *const WDF_OBJECT_CONTEXT_TYPE_INFO;

        attributes.ParentObject = (*deviceExtension).IoTarget as WDFOBJECT;

        status = WdfRequestCreate(
            core::ptr::addr_of_mut!(attributes),
            (*deviceExtension).IoTarget,
            core::ptr::addr_of_mut!(startUnitRequest),
        );
    }

    if NT_SUCCESS(status) {
        (*srb).OriginalRequest = WdfRequestWdmGetIrp(startUnitRequest) as *mut core::ffi::c_void;
        NT_ASSERT((!(*srb).OriginalRequest.is_null()) as i32);

        // WDF_OBJECT_ATTRIBUTES_INIT(&attributes) – expand: zero + set size
        core::ptr::write_bytes(
            (core::ptr::addr_of_mut!(attributes)) as *mut u8,
            0,
            core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>());
        attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
        attributes.ExecutionLevel = 1; // WdfExecutionLevelInheritFromParent
        attributes.SynchronizationScope = 1; // WdfSynchronizationScopeInheritFromParent
        attributes.ParentObject = startUnitRequest as WDFOBJECT;

        status = WdfMemoryCreatePreallocated(
            core::ptr::addr_of_mut!(attributes),
            srb as *mut core::ffi::c_void,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
            core::ptr::addr_of_mut!(inputMemory),
        );
    }

    if NT_SUCCESS(status) {
        status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            (*deviceExtension).IoTarget,
            startUnitRequest,
            IOCTL_SCSI_EXECUTE_NONE,
            inputMemory,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        WdfRequestSetCompletionRoutine(
            startUnitRequest,
            Some(core::mem::transmute(DeviceAsynchronousCompletion as unsafe extern "system" fn(_, _, _, _))),
            context as WDFCONTEXT,
        );

        status = RequestSend(
            deviceExtension,
            startUnitRequest,
            (*deviceExtension).IoTarget,
            0,
            core::ptr::null_mut(),
        );
    }

    if !NT_SUCCESS(status) {
        FREE_POOL(context as *mut core::ffi::c_void);
        if !startUnitRequest.is_null() {
            WdfObjectDelete(startUnitRequest as WDFOBJECT);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSendIoctlAsynchronously(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    IoControlCode: ULONG,
    TargetDeviceObject: *mut DEVICE_OBJECT,
) {
    let mut irp: *mut IRP;
    let mut nextIrpStack: *mut IO_STACK_LOCATION;

    irp = IoAllocateIrp((*(*DeviceExtension).DeviceObject).StackSize as i8, 0u8);

    if !irp.is_null() {
        nextIrpStack = IoGetNextIrpStackLocation(irp);

        (*nextIrpStack).MajorFunction = IRP_MJ_DEVICE_CONTROL;
        (*nextIrpStack).Parameters.DeviceIoControl.OutputBufferLength = 0;
        (*nextIrpStack).Parameters.DeviceIoControl.InputBufferLength = 0;
        (*nextIrpStack).Parameters.DeviceIoControl.IoControlCode = IoControlCode;
        (*nextIrpStack).Parameters.DeviceIoControl.Type3InputBuffer = core::ptr::null_mut();

        IoSetCompletionRoutine(
            irp,
            Some(core::mem::transmute(RequestAsynchronousIrpCompletion as unsafe extern "system" fn(_, _, _) -> _)),
            DeviceExtension as *mut core::ffi::c_void,
            1u8,
            1u8,
            1u8,
        );

        let _ = IoCallDriver(TargetDeviceObject, irp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RequestAsynchronousIrpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let _ = DeviceObject;
    let _ = Context;
    IoFreeIrp(Irp);
    return STATUS_MORE_PROCESSING_REQUIRED;
}
#[no_mangle]
pub unsafe extern "system" fn DeviceAsynchronousCompletion(
    Request: WDFREQUEST,
    Target: WDFIOTARGET,
    Params: *mut WDF_REQUEST_COMPLETION_PARAMS,
    Context: WDFCONTEXT,
) {
    // Extern declarations for symbols not in the provided import list.

    // Constants for SCSI SRB flags.
    const SRB_FUNCTION_EXECUTE_SCSI: u8 = 0x00;
    const SRB_STATUS_QUEUE_FROZEN: u8 = 0x40;

    // Cast Context to the expected completion context pointer.
    let context = Context as *mut COMPLETION_CONTEXT;

    // Obtain device extension from the device stored in the context.
    let deviceExtension = DeviceGetExtension((*context).Device);

    // Suppress unused parameter warnings (mechanical preservation of C code).
    UNREFERENCED_PARAMETER!(&Target as *const _ as usize);
    UNREFERENCED_PARAMETER!(&Params as *const _ as usize);

    // If this is an execute SRB, check for a frozen queue.
    if (*context).Srb.Function == SRB_FUNCTION_EXECUTE_SCSI {
        // Check for SRB_STATUS_QUEUE_FROZEN.
        if ((*context).Srb.SrbStatus & SRB_STATUS_QUEUE_FROZEN) != 0 {
            DeviceReleaseQueue((*context).Device);
        }
    }

    // Free port-allocated sense buffer if present.
    if PORT_ALLOCATED_SENSE(deviceExtension, &mut (*context).Srb) != 0 {
        FREE_PORT_ALLOCATED_SENSE_BUFFER(deviceExtension, &mut (*context).Srb);
    }

    // Release the context memory.
    FREE_POOL(context as *mut core::ffi::c_void);

    // Delete the WDF request object.
    WdfObjectDelete(Request as WDFOBJECT);
}
#[no_mangle]
pub unsafe extern "C" fn DeviceReleaseQueue(Device: WDFDEVICE) {
    let deviceExtension = DeviceGetExtension(Device) as *mut CDROM_DEVICE_EXTENSION;
    let mut srb: *mut SCSI_REQUEST_BLOCK;
    let mut currentIrql: KIRQL = 0;

    KeRaiseIrql(DISPATCH_LEVEL, &mut currentIrql);

    WdfSpinLockAcquire((*deviceExtension).ReleaseQueueSpinLock);

    if (*deviceExtension).ReleaseQueueInProgress != 0 {
        (*deviceExtension).ReleaseQueueNeeded = 1;
        WdfSpinLockRelease((*deviceExtension).ReleaseQueueSpinLock);
        KeLowerIrql(currentIrql);
        return;
    }

    (*deviceExtension).ReleaseQueueInProgress = 1;

    WdfSpinLockRelease((*deviceExtension).ReleaseQueueSpinLock);

    srb = core::ptr::addr_of_mut!((*deviceExtension).ReleaseQueueSrb);

    (*srb).Function = SRB_FUNCTION_FLUSH_QUEUE as u8;

    (*srb).OriginalRequest = WdfRequestWdmGetIrp((*deviceExtension).ReleaseQueueRequest) as *mut core::ffi::c_void;

    WdfRequestSetCompletionRoutine(
        (*deviceExtension).ReleaseQueueRequest,
        Some(core::mem::transmute(DeviceReleaseQueueCompletion as unsafe extern "system" fn(_, _, _, _))),
        Device as WDFCONTEXT,
    );

    RequestSend(
        deviceExtension,
        (*deviceExtension).ReleaseQueueRequest,
        (*deviceExtension).IoTarget,
        WDF_REQUEST_SEND_OPTION_IGNORE_TARGET_STATE,
        core::ptr::null_mut(),
    );

    KeLowerIrql(currentIrql);
}
#[no_mangle]
pub unsafe extern "system" fn DeviceReleaseQueueCompletion(
    Request: WDFREQUEST,
    Target: WDFIOTARGET,
    Params: PWDF_REQUEST_COMPLETION_PARAMS,
    Context: WDFCONTEXT,
) {
    unsafe {
        let _ = Target;
        let _ = Params;

        let mut status: NTSTATUS;
        let device: WDFDEVICE = Context as WDFDEVICE;
        let deviceExtension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(device);
        let mut releaseQueueNeeded: BOOLEAN;
        let mut params: WDF_REQUEST_REUSE_PARAMS = core::mem::zeroed();

        WDF_REQUEST_REUSE_PARAMS_INIT(
            core::ptr::addr_of_mut!(params),
            WDF_REQUEST_REUSE_NO_FLAGS,
            STATUS_SUCCESS,
        );

        WdfSpinLockAcquire((*deviceExtension).ReleaseQueueSpinLock);

        releaseQueueNeeded = (*deviceExtension).ReleaseQueueNeeded;
        (*deviceExtension).ReleaseQueueNeeded = 0;
        (*deviceExtension).ReleaseQueueInProgress = 0;

        status = WdfRequestReuse(Request, core::ptr::addr_of_mut!(params));

        if NT_SUCCESS(status) {
            status = WdfIoTargetFormatRequestForInternalIoctlOthers(
                (*deviceExtension).IoTarget,
                Request,
                IOCTL_SCSI_EXECUTE_NONE,
                (*deviceExtension).ReleaseQueueInputMemory,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        if !NT_SUCCESS(status) {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                b"DeviceReleaseQueueCompletion: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n\0"
                    .as_ptr() as *const i8,
                status,
            );
        }

        RequestClearSendTime(Request);
        WdfSpinLockRelease((*deviceExtension).ReleaseQueueSpinLock);

        if releaseQueueNeeded != 0 {
            DeviceReleaseQueue(device);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DevicePerfIncrementErrorCount(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    // Declare unknown externs (the caller will deduplicate)

    let fdoData = (*DeviceExtension).PrivateFdoData;
    let mut oldIrql: ntkrnl_sys::base::KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock) as *mut usize, &mut oldIrql);

    (*fdoData).Perf.SuccessfulIO = 0; // implicit interlock
    let errors = InterlockedIncrement(
        core::ptr::addr_of_mut!((*DeviceExtension).ErrorCount) as *mut i32,
    );

    if errors >= CLASS_ERROR_LEVEL_1 as i32 {
        // If the error count has exceeded the error limit, then disable
        // any tagged queuing, multiple requests per lu queueing
        // and sychronous data transfers.
        //
        // Clearing the no queue freeze flag prevents the port driver
        // from sending multiple requests per logical unit.
        (*DeviceExtension).SrbFlags &= !SRB_FLAGS_NO_QUEUE_FREEZE;
        (*DeviceExtension).SrbFlags &= !SRB_FLAGS_QUEUE_ACTION_ENABLE;
        (*DeviceExtension).SrbFlags |= SRB_FLAGS_DISABLE_SYNCH_TRANSFER;

        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"PerfIncrementErrorCount: Too many errors; disabling tagged queuing and \
              synchronous data tranfers.\n\0"
                .as_ptr() as *const i8,
        );
    }

    if errors >= CLASS_ERROR_LEVEL_2 as i32 {
        // If a second threshold is reached, disable disconnects.
        (*DeviceExtension).SrbFlags |= SRB_FLAGS_DISABLE_DISCONNECT;
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"PerfIncrementErrorCount: Too many errors; disabling disconnects.\n\0"
                .as_ptr() as *const i8,
        );
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*fdoData).SpinLock) as *mut usize, oldIrql);
}
#[no_mangle]
pub unsafe extern "C" fn DeviceFindFeaturePage(
    FeatureBuffer: *mut GET_CONFIGURATION_HEADER,
    Length: u32,
    Feature: FEATURE_NUMBER,
) -> *mut FEATURE_HEADER {

    // DataLength is a 4-byte big-endian field. C: REVERSE_BYTES(&validLength,
    // FeatureBuffer->DataLength) byte-swaps the FULL 4 bytes, then adds
    // RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength) =
    // offset_of(DataLength) + sizeof(DataLength). The original Rust read only the
    // low 2 bytes and added size_of::<u16>() (off-by-2 + truncation; found by
    // rustconv semdiff).
    let validLength: u32 = ((((*FeatureBuffer).DataLength[0] as u32) << 24)
        | (((*FeatureBuffer).DataLength[1] as u32) << 16)
        | (((*FeatureBuffer).DataLength[2] as u32) << 8)
        | ((*FeatureBuffer).DataLength[3] as u32))
        + (core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength)
            + core::mem::size_of_val(&(*FeatureBuffer).DataLength)) as u32;

    if (Length as usize) < (core::mem::size_of::<GET_CONFIGURATION_HEADER>() + core::mem::size_of::<FEATURE_HEADER>()) {
        return core::ptr::null_mut();
    }

    let mut limit: *mut u8 = FeatureBuffer as *mut u8;
    limit = limit.add(core::cmp::min(Length as usize, validLength as usize));

    let mut buffer: *mut u8 = core::ptr::addr_of_mut!((*FeatureBuffer).Data) as *mut u8;

    while {
        let buffer_end = (buffer as usize) + core::mem::size_of::<FEATURE_HEADER>();
        buffer_end <= (limit as usize)
    } {
        let header: *mut FEATURE_HEADER = buffer as *mut FEATURE_HEADER;
        let thisFeature: u16 = (((*header).FeatureCode[0] as u16) << 8) | ((*header).FeatureCode[1] as u16);

        if thisFeature as u32 == Feature {
            let mut temp: *mut u8 = buffer;
            temp = temp.add(core::mem::size_of::<FEATURE_HEADER>());
            temp = temp.add((*header).AdditionalLength as usize);

            if temp > limit {
                TracePrint!(
                    3, // TRACE_LEVEL_WARNING
                    0x20, // TRACE_FLAG_GENERAL
                    b"Feature %x exists, but not safe to access all its data. returning NULL\n\0".as_ptr() as *const i8,
                    Feature as u32,
                );
                return core::ptr::null_mut();
            } else {
                return buffer as *mut FEATURE_HEADER;
            }
        }

        if (((*header).AdditionalLength as u32) % 4 != 0) && !(Feature >= 0xff00 && Feature <= 0xffff) {
            return core::ptr::null_mut();
        }

        buffer = buffer.add(core::mem::size_of::<FEATURE_HEADER>());
        buffer = buffer.add((*header).AdditionalLength as usize);
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn DevicePrintAllFeaturePages(
    Buffer: *mut GET_CONFIGURATION_HEADER,
    Usable: u32,
) {
    PAGED_CODE!();

    ////////////////////////////////////////////////////////////////////////////////
    // items expected to ALWAYS be current if they exist
    ////////////////////////////////////////////////////////////////////////////////

    let mut header: *mut FEATURE_HEADER;

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureProfileList);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: CurrentProfile %x with %x bytes of data at %p\n\0"
                .as_ptr() as *const i8,
            (((*(*Buffer).CurrentProfile.as_ptr().offset(0)) as u32) << 8)
                | ((*(*Buffer).CurrentProfile.as_ptr().offset(1)) as u32),
            Usable,
            Buffer as usize,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureCore);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"CORE Features\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureMorphing);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Morphing\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureRemovableMedium);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Removable Medium\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeaturePowerManagement);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Power Management\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureEmbeddedChanger);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Embedded Changer\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureMicrocodeUpgrade);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Microcode Update\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureTimeout);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Timeouts\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureLogicalUnitSerialNumber);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"LUN Serial Number\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureFirmwareDate);
    if !header.is_null() {
        let feature_size: u32 =
            (*header).AdditionalLength as u32
            + (core::mem::offset_of!(FEATURE_HEADER, AdditionalLength)
               + core::mem::size_of::<u8>()) as u32; // AdditionalLength is a UCHAR

        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Firmware Date\0".as_ptr() as *const i8,
        );

        if feature_size
            >= (core::mem::offset_of!(FEATURE_DATA_FIRMWARE_DATE, Minute)
                + core::mem::size_of::<[u8; 2]>()) as u32
        {
            let date: *mut FEATURE_DATA_FIRMWARE_DATE = header as *mut FEATURE_DATA_FIRMWARE_DATE;
            let mut date_string: [u8; 18] = [0u8; 18];
            date_string[0] = (*date).Year[0];
            date_string[1] = (*date).Year[1];
            date_string[2] = (*date).Year[2];
            date_string[3] = (*date).Year[3];
            date_string[4] = b'/';
            date_string[5] = (*date).Month[0];
            date_string[6] = (*date).Month[1];
            date_string[7] = b'/';
            date_string[8] = (*date).Day[0];
            date_string[9] = (*date).Day[1];
            date_string[10] = b' ';
            date_string[11] = b' ';
            date_string[12] = (*date).Hour[0];
            date_string[13] = (*date).Hour[1];
            date_string[14] = b':';
            date_string[15] = (*date).Minute[0];
            date_string[16] = (*date).Minute[1];
            date_string[17] = 0;
            // SECONDS IS NOT AVAILABLE ON EARLY IMPLEMENTATIONS -- ignore it
            //date_string[17] = ':';
            //date_string[18] = date->Seconds[0];
            //date_string[19] = date->Seconds[1];
            //date_string[20] = 0;
            TracePrint!(
                TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_INIT,
                b"CdromGetConfiguration: Firmware Date/Time %s (UTC)\n\0".as_ptr() as *const i8,
                date_string.as_ptr() as *const i8,
            );
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // items expected not to always be current
    ////////////////////////////////////////////////////////////////////////////////

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureWriteProtect);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_GENERAL,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Software Write Protect\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureRandomReadable);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Random Reads\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureMultiRead);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Multi-Read\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdRead);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"reading from CD-ROM/R/RW\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdRead);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DVD Structure Reads\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureRandomWritable);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Random Writes\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureIncrementalStreamingWritable);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Incremental Streaming Writing\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureSectorErasable);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Sector Erasable Media\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureFormattable);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Formatting\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDefectManagement);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"defect management\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureWriteOnce);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Write Once Media\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureRestrictedOverwrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Restricted Overwrites\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdrwCAVWrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"CD-RW CAV recording\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureMrw);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Mount Rainier media\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureEnhancedDefectReporting);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Enhanced Defect Reporting\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdPlusRW);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DVD+RW media\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureRigidRestrictedOverwrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Rigid Restricted Overwrite\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdTrackAtOnce);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"CD Recording (Track At Once)\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdMastering);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"CD Recording (Mastering)\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdRecordableWrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DVD Recording (Mastering)\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDDCDRead);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DD CD Reading\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDDCDRWrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DD CD-R Writing\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDDCDRWWrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DD CD-RW Writing\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureLayerJumpRecording);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Layer Jump Recording\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureHDDVDRead);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"HD-DVD Reading\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureHDDVDWrite);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"HD-DVD Writing\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureSMART);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"S.M.A.R.T.\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureCDAudioAnalogPlay);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Analogue CD Audio Operations\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdCSS);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DVD CSS\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureRealTimeStreaming);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"Real-time Streaming Reads\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDiscControlBlocks);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DVD Disc Control Blocks\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdCPRM);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"DVD CPRM\0".as_ptr() as *const i8,
        );
    }

    header = DeviceFindFeaturePage(Buffer, Usable, FeatureAACS);
    if !header.is_null() {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_INIT,
            b"CdromGetConfiguration: %s %s\n\0".as_ptr() as *const i8,
            if (*header).Current() != 0 {
                b"Currently supports\0".as_ptr() as *const i8
            } else {
                b"Is able to support\0".as_ptr() as *const i8
            },
            b"AACS\0".as_ptr() as *const i8,
        );
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn MediaReadCapacity(Device: WDFDEVICE) -> i32 {
    // PAGED_CODE() // debug check omitted
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut capacityData: READ_CAPACITY_DATA = core::mem::zeroed();
    RtlZeroMemory(
        &mut srb as *mut SCSI_REQUEST_BLOCK as *mut core::ffi::c_void,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
    );
    RtlZeroMemory(
        &mut capacityData as *mut READ_CAPACITY_DATA as *mut core::ffi::c_void,
        core::mem::size_of::<READ_CAPACITY_DATA>(),
    );

    let cdb: *mut CDB = core::ptr::addr_of_mut!(srb.Cdb) as *mut CDB;

    srb.CdbLength = 10;
    srb.TimeOutValue = CDROM_READ_CAPACITY_TIMEOUT;
    (*cdb).CDB10.as_mut().OperationCode = SCSIOP_READ_CAPACITY;

    let status = DeviceSendSrbSynchronously(
        Device,
        &mut srb,
        &mut capacityData as *mut READ_CAPACITY_DATA as *mut core::ffi::c_void,
        core::mem::size_of::<READ_CAPACITY_DATA>() as u32,
        0u8,                           // FALSE
        core::ptr::null_mut(),         // NULL
    );

    if !NT_SUCCESS(status) {
        capacityData.BytesPerBlock = 0;
    }

    MediaReadCapacityDataInterpret(Device, &mut capacityData);

    status
}

#[no_mangle]
pub unsafe extern "C" fn MediaReadCapacityDataInterpret(
    Device: WDFDEVICE,
    ReadCapacityBuffer: PREAD_CAPACITY_DATA,
) {
    let deviceExtension = DeviceGetExtension(Device);
    if deviceExtension.is_null() {
        return;
    }
    let mut lastSector: u32 = 0;
    let mut bps: u32 = 0;
    let mut bytesPerBlock: u32;
    let mut errorHappened: i32 = 0;

    // PAGED_CODE();

    NT_ASSERT(if !ReadCapacityBuffer.is_null() { 1 } else { 0 });

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_GENERAL,
        b"MediaReadCapacityDataInterpret: Entering\n\0".as_ptr() as *const i8,
    );

    bytesPerBlock = (*ReadCapacityBuffer).BytesPerBlock;

    {
        let bps_bytes = &mut bps as *mut u32 as *mut u8;
        let bp_bytes = &bytesPerBlock as *const u32 as *const u8;
        *bps_bytes.add(0) = *bp_bytes.add(3);
        *bps_bytes.add(1) = *bp_bytes.add(2);
        *bps_bytes.add(2) = *bp_bytes.add(1);
        *bps_bytes.add(3) = *bp_bytes.add(0);
    }

    if bps == 0 {
        bps = 2048;
        errorHappened = 1;
    } else {
        bps = 1u32 << (31 - bps.leading_zeros());
    }

    (*deviceExtension).DiskGeometry.BytesPerSector = bps;

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_GENERAL,
        b"MediaReadCapacityDataInterpret: Calculated bps %#x\n\0".as_ptr() as *const i8,
        bps,
    );

    bytesPerBlock = (*ReadCapacityBuffer).LogicalBlockAddress;

    {
        let ls_bytes = &mut lastSector as *mut u32 as *mut u8;
        let bp_bytes = &bytesPerBlock as *const u32 as *const u8;
        *ls_bytes.add(0) = *bp_bytes.add(3);
        *ls_bytes.add(1) = *bp_bytes.add(2);
        *ls_bytes.add(2) = *bp_bytes.add(1);
        *ls_bytes.add(3) = *bp_bytes.add(0);
    }

    WHICH_BIT(bps, &mut (*deviceExtension).SectorShift);

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_GENERAL,
        b"MediaReadCapacityDataInterpret: Sector size is %d\n\0".as_ptr() as *const i8,
        (*deviceExtension).DiskGeometry.BytesPerSector,
    );

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_GENERAL,
        b"MediaReadCapacityDataInterpret: Number of Sectors is %d\n\0".as_ptr() as *const i8,
        lastSector + 1,
    );

    if errorHappened != 0 {
        (*deviceExtension).PartitionLength.QuadPart = 0x7fffffffi64;
    } else {
        let total_sectors = (lastSector as i64 + 1) as i64;
        (*deviceExtension).PartitionLength.QuadPart = total_sectors;
        (*deviceExtension).PartitionLength.QuadPart <<= (*deviceExtension).SectorShift as u32;
    }

    (*deviceExtension).DiskGeometry.TracksPerCylinder = 0x40u32;
    (*deviceExtension).DiskGeometry.SectorsPerTrack = 0x20u32;

    let cylinders = ((lastSector as i64 + 1) / (32 * 64)) as i64;
    (*deviceExtension).DiskGeometry.Cylinders.QuadPart = cylinders;

    (*deviceExtension).DiskGeometry.MediaType = RemovableMedia;

    return;
}
#[no_mangle]
pub unsafe extern "system" fn DevicePickDvdRegion(Device: WDFDEVICE) {
    const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
    const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
    // CTL_CODE(IOCTL_DVD_BASE=FILE_DEVICE_DVD=0x33, 0x401, METHOD_BUFFERED, FILE_READ_ACCESS)
    const IOCTL_DVD_READ_KEY: u32 = 0x0033_5004;
    // pool tag 'vCcS' (displays "ScCv") — cdrom.h DVD_TAG_DVD_REGION
    const DVD_TAG_DVD_REGION: u32 = 0x7643_6353;
    const DVD_MAX_REGION: u32 = 8;
    // DvdCopyrightDescriptor (=1), DvdGetRpcKey (=0x8), DvdSetRpcKey (=0x6) come
    // from the real ntddcdvd.h enums imported at the top of this file — do NOT
    // shadow them with =0 placeholders (that disabled region selection entirely).
    // DVD_RPC_KEY_LENGTH / DVD_SET_RPC_KEY_LENGTH = sizeof(key) + sizeof(DVD_COPY_PROTECT_KEY)
    const DVD_RPC_KEY_LENGTH: usize =
        core::mem::size_of::<DVD_RPC_KEY>() + core::mem::size_of::<DVD_COPY_PROTECT_KEY>();
    const DVD_SET_RPC_KEY_LENGTH: usize =
        core::mem::size_of::<DVD_SET_RPC_KEY>() + core::mem::size_of::<DVD_COPY_PROTECT_KEY>();

    // DVD_DEFAULT_REGION wide string constant used for registry value name
    // We expand DECLARE_CONST_UNICODE_STRING manually using RtlInitUnicodeString.
    let dvd_default_region_wstr: [u16; 18] = [
        0x44, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74, 0x44, 0x76, 0x64,
        0x52, 0x65, 0x67, 0x69, 0x6f, 0x6e, 0x0, 0x0
    ];
    let mut registry_value_name = core::mem::zeroed::<UNICODE_STRING>();
    RtlInitUnicodeString(
        &mut registry_value_name,
        dvd_default_region_wstr.as_ptr(),
    );

    PAGED_CODE!();

    let device_extension = DeviceGetExtension(Device);

    // InterlockedExchange((PLONG)&deviceExtension->DeviceAdditionalData.PickDvdRegion, 0)
    let pick_dvd_region_ptr = core::ptr::addr_of!((*device_extension).DeviceAdditionalData.PickDvdRegion)
        as *const core::sync::atomic::AtomicI32;
    let mut pick_dvd_region = (*pick_dvd_region_ptr).swap(0, core::sync::atomic::Ordering::AcqRel);
    if pick_dvd_region == 0 {
        return;
    }

    let buffer_len = core::cmp::max(
        core::cmp::max(
            core::mem::size_of::<DVD_DESCRIPTOR_HEADER>()
                + core::mem::size_of::<DVD_COPYRIGHT_DESCRIPTOR>(),
            core::mem::size_of::<DVD_READ_STRUCTURE>(),
        ),
        core::cmp::max(DVD_RPC_KEY_LENGTH, DVD_SET_RPC_KEY_LENGTH),
    );

    let dvd_read_structure = ntkrnl_sys::ex::ExAllocatePoolWithTag(
        ntkrnl_sys::mm::PagedPool,
        buffer_len,
        DVD_TAG_DVD_REGION,
    ) as *mut DVD_READ_STRUCTURE;

    if dvd_read_structure.is_null() {
        (*pick_dvd_region_ptr).swap(pick_dvd_region, core::sync::atomic::Ordering::AcqRel);
        return;
    }

    // Single labeled block models the C `goto getout` cleanup jump: every
    // `break 'getout` exits here and falls through to the FREE_POOL + final
    // InterlockedExchange below. (Each goto was previously mis-lowered as a
    // no-op self-breaking 'getout block, so failure paths fell THROUGH
    // instead of cleaning up.)
    'getout: {
    let copy_protect_key = dvd_read_structure as *mut DVD_COPY_PROTECT_KEY;

    // dvdCopyRight = (PDVD_COPYRIGHT_DESCRIPTOR)((PDVD_DESCRIPTOR_HEADER)dvdReadStructure)->Data;
    let dvd_copy_right =
        (dvd_read_structure as *mut u8).add(core::mem::size_of::<DVD_DESCRIPTOR_HEADER>())
        as *mut DVD_COPYRIGHT_DESCRIPTOR;

    // Get the media region
    ntkrnl_sys::rtl::RtlZeroMemory(dvd_read_structure as *mut core::ffi::c_void, buffer_len);
    (*dvd_read_structure).Format = DvdCopyrightDescriptor as u32;

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"DevicePickDvdRegion (%p): Getting Copyright Descriptor\n\0".as_ptr() as *const i8,
    );

    let mut bytes_returned: usize = 0;
    let status = ReadDvdStructure(
        device_extension,
        core::ptr::null_mut(),
        dvd_read_structure,
        core::mem::size_of::<DVD_READ_STRUCTURE>(),
        dvd_read_structure,
        core::mem::size_of::<DVD_DESCRIPTOR_HEADER>() + core::mem::size_of::<DVD_COPYRIGHT_DESCRIPTOR>(),
        &mut bytes_returned,
    );

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"DevicePickDvdRegion (%p): Got Copyright Descriptor %x\n\0".as_ptr() as *const i8,
    );

    let mut media_region: u8;
    if ntkrnl_sys::base::NT_SUCCESS(status) && (*dvd_copy_right).CopyrightProtectionType == 0x01 {
        if (*dvd_copy_right).RegionManagementInformation == 0xff {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"DevicePickDvdRegion (%p): RegionManagementInformation \
                  is set to dis-allow playback for all regions.  This is \
                  most likely a poorly authored disc.  defaulting to all \
                  region disc for purpose of choosing initial region\n\0"
                    .as_ptr() as *const i8,
            );
            (*dvd_copy_right).RegionManagementInformation = 0;
        }
        media_region = !(*dvd_copy_right).RegionManagementInformation;
    } else {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): failed to auto-choose a region due to status %x getting copyright descriptor\n\0"
                .as_ptr() as *const i8,
        );
        break 'getout; // goto getout
    }

    // get the device region
    ntkrnl_sys::rtl::RtlZeroMemory(copy_protect_key as *mut core::ffi::c_void, buffer_len);
    (*copy_protect_key).KeyLength = DVD_RPC_KEY_LENGTH as u32;
    (*copy_protect_key).KeyType = DvdGetRpcKey as u32;

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"DevicePickDvdRegion (%p): Getting RpcKey\n\0".as_ptr() as *const i8,
    );

    let status = DvdStartSessionReadKey(
        device_extension,
        IOCTL_DVD_READ_KEY,
        core::ptr::null_mut(),
        copy_protect_key,
        DVD_RPC_KEY_LENGTH,
        copy_protect_key,
        DVD_RPC_KEY_LENGTH,
        &mut bytes_returned,
    );

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"DevicePickDvdRegion (%p): Got RpcKey %x\n\0".as_ptr() as *const i8,
    );

    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): failed to get RpcKey from a DVD Device\n\0".as_ptr() as *const i8,
        );
        break 'getout; // goto getout
    }

    // rpcKey = (PDVD_RPC_KEY)copyProtectKey->KeyData
    let rpc_key =
        (copy_protect_key as *mut u8).add(core::mem::size_of::<DVD_COPY_PROTECT_KEY>())
        as *mut DVD_RPC_KEY;

    if (*rpc_key).RegionMask != 0xff {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): not picking a region since \
              it is already chosen\n\0".as_ptr() as *const i8,
        );
        break 'getout; // goto getout
    }

    if (*rpc_key).UserResetsAvailable() <= 1 {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): not picking a region since \
              only one change remains\n\0".as_ptr() as *const i8,
        );
        break 'getout; // goto getout
    }

    // OOBE sets this key based upon the system locale
    let driver = WdfGetDriver();
    let mut registry_key: WDFKEY = core::ptr::null_mut();
    let mut status = WdfDriverOpenParametersRegistryKey(
        driver,
        ntkrnl_sys::constants::KEY_READ,
        core::ptr::null_mut(), // WDF_NO_OBJECT_ATTRIBUTES
        &mut registry_key,
    );

    let mut default_dvd_region: u32 = 0;
    if ntkrnl_sys::base::NT_SUCCESS(status) {
        status = WdfRegistryQueryULong(
            registry_key,
            &raw const registry_value_name,
            &mut default_dvd_region,
        );
        WdfRegistryClose(registry_key);
    }

    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): failed to read registry value due to status %x\n\0"
                .as_ptr() as *const i8,
        );
        default_dvd_region = 0;
    }

    if default_dvd_region > DVD_MAX_REGION {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): registry has a bogus default \
              region value of %x\n\0".as_ptr() as *const i8,
        );
        default_dvd_region = 0;
    }

    let mut dvd_region: u32;

    if default_dvd_region != 0 && (media_region & (1u8 << (default_dvd_region - 1))) != 0 {
        dvd_region = 1u32 << (default_dvd_region - 1);
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): Choice #1: media matches \
              drive's default, chose region %x\n\0".as_ptr() as *const i8,
        );
    } else if media_region != 0 {
        let mut mask: u8 = 1;
        dvd_region = 0;
        while media_region != 0 && dvd_region == 0 {
            dvd_region = (media_region & mask) as u32;
            mask <<= 1;
        }
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): Choice #2: choosing lowest \
              media region %x\n\0".as_ptr() as *const i8,
        );
    } else if default_dvd_region != 0 {
        dvd_region = 1u32 << (default_dvd_region - 1);
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): Choice #3: using default \
              region for this install %x\n\0".as_ptr() as *const i8,
        );
    } else {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): Choice #4: failed to choose \
              a media region\n\0".as_ptr() as *const i8,
        );
        break 'getout; // goto getout
    }

    // Now set the region
    ntkrnl_sys::rtl::RtlZeroMemory(copy_protect_key as *mut core::ffi::c_void, buffer_len);
    (*copy_protect_key).KeyLength = DVD_SET_RPC_KEY_LENGTH as u32;
    (*copy_protect_key).KeyType = DvdSetRpcKey as u32;

    let dvd_rpc_key =
        (copy_protect_key as *mut u8).add(core::mem::size_of::<DVD_COPY_PROTECT_KEY>())
        as *mut DVD_SET_RPC_KEY;
    (*dvd_rpc_key).PreferredDriveRegionCode = (!dvd_region) as u8;

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"DevicePickDvdRegion (%p): Sending new Rpc Key to region %x\n\0".as_ptr() as *const i8,
    );

    status = DvdSendKey(
        device_extension,
        core::ptr::null_mut(),
        copy_protect_key,
        DVD_SET_RPC_KEY_LENGTH,
        &mut bytes_returned,
    );

    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_IOCTL,
        b"DevicePickDvdRegion (%p): Sent new Rpc Key %x\n\0".as_ptr() as *const i8,
    );

    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): unable to set dvd initial \
              region code (%x)\n\0".as_ptr() as *const i8,
        );
    } else {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_IOCTL,
            b"DevicePickDvdRegion (%p): Successfully set dvd \
              initial region\n\0".as_ptr() as *const i8,
        );
        pick_dvd_region = 0;
    }

    } // end 'getout — fall through to cleanup

    // cleanup (from getout:)
    if !dvd_read_structure.is_null() {
        FREE_POOL(dvd_read_structure as *mut core::ffi::c_void);
    }

    // final InterlockedExchange
    let _ = (*pick_dvd_region_ptr).swap(pick_dvd_region, core::sync::atomic::Ordering::AcqRel);
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRegisterInterface(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    InterfaceType: i32,
) -> i32 {
    let mut status: i32;
    let mut string: WDFSTRING = core::ptr::null_mut();
    let mut interfaceGuid: *mut GUID;
    let mut savingString: *mut UNICODE_STRING;
    let mut setRestricted: bool = false;
    let mut localString: UNICODE_STRING = core::mem::zeroed();

    // PAGED_CODE() — DBG-only diagnostic in C (xdk/kefuncs.h: KdPrint + ASSERT
    // under DBG, NOP_FUNCTION in retail). It was translated to
    // unreachable_unchecked(), which is UB: an assertion became an optimizer
    // ASSUMPTION, so LLVM is free to delete the IRQL check and poison whatever
    // follows. Never lower an assert to UB — match the C diagnostic instead.
    if KeGetCurrentIrql() > APC_LEVEL {
        extern "C" {
            fn DbgPrint(fmt: *const i8, ...) -> u32;
        }
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as u32,
        );
    }

    // Switch on InterfaceType
    match InterfaceType {
        CdRomDeviceInterface => {
            interfaceGuid = core::ptr::addr_of!(GUID_DEVINTERFACE_CDROM) as *mut GUID;
            setRestricted = true;
            savingString = core::ptr::addr_of_mut!(localString);
        }
        MountedDeviceInterface => {
            interfaceGuid = core::ptr::addr_of!(MOUNTDEV_MOUNTED_DEVICE_GUID) as *mut GUID;
            savingString = core::ptr::addr_of_mut!((*DeviceExtension).MountedDeviceInterfaceName);
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    status = WdfDeviceCreateDeviceInterface(
        (*DeviceExtension).Device,
        interfaceGuid,
        core::ptr::null(),
    );

    if !NT_SUCCESS(status) {
        let s: *const i8 = if InterfaceType == CdRomDeviceInterface {
            b"CdRom Interface\0".as_ptr() as *const i8
        } else {
            b"Mounted Device Interface\0".as_ptr() as *const i8
        };
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"DeviceRegisterInterface: Unable to register cdrom DCA for fdo %p type: %s [%lx]\n\0".as_ptr() as *const i8,
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            s,
            status as u32,
        );
    }

    if NT_SUCCESS(status) {
        let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
        attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
        // WDF_OBJECT_ATTRIBUTES_INIT sets these to InheritFromParent(1); leaving them 0
        // (Invalid) makes WdfStringCreate reject with STATUS_WDF_OBJECT_ATTRIBUTES_INVALID.
        attributes.ExecutionLevel = 1;       // WdfExecutionLevelInheritFromParent
        attributes.SynchronizationScope = 1; // WdfSynchronizationScopeInheritFromParent
        attributes.ParentObject = (*DeviceExtension).Device as WDFOBJECT;

        status = WdfStringCreate(
            core::ptr::null(),
            &mut attributes,
            &mut string,
        );
    }

    if NT_SUCCESS(status) {
        status = WdfDeviceRetrieveDeviceInterfaceString(
            (*DeviceExtension).Device,
            interfaceGuid,
            core::ptr::null(),
            string,
        );
    }

    if NT_SUCCESS(status) {
        WdfStringGetUnicodeString(string, savingString);
        if setRestricted {
            WdfObjectDelete(string as WDFOBJECT);
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRestoreDefaultSpeed(WorkItem: WDFWORKITEM) {
    let mut status: i32;
    let device: WDFDEVICE = WdfWorkItemGetParentObject(WorkItem) as WDFDEVICE;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION = DeviceGetExtension(device);
    let transferLength: u32 = core::mem::size_of::<PERFORMANCE_DESCRIPTOR>() as u32;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let cdb: *mut CDB = core::ptr::addr_of_mut!(srb.Cdb) as *mut CDB;

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgBreakPoint();
    }

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"DeviceRestoreDefaultSpeed: Restore device speed for %p\n\0".as_ptr() as *const i8,
        device,
    );

    let perfDescriptor: *mut PERFORMANCE_DESCRIPTOR = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        transferLength as usize,
        CDROM_TAG_STREAM,
    ) as *mut PERFORMANCE_DESCRIPTOR;
    if perfDescriptor.is_null() {
        return;
    }

    RtlZeroMemory(perfDescriptor as *mut core::ffi::c_void, transferLength as usize);
    (*perfDescriptor).set_RestoreDefaults(1); // TRUE

    srb.TimeOutValue = (*deviceExtension).TimeOutValue;
    srb.CdbLength = 12;
    (*cdb).SET_STREAMING.as_mut().OperationCode = SCSIOP_SET_STREAMING;
    REVERSE_BYTES_SHORT(
        core::ptr::addr_of_mut!((*cdb).SET_STREAMING.as_mut().ParameterListLength) as *mut u16,
        core::ptr::addr_of!(transferLength) as *const u32,
    );

    status = DeviceSendSrbSynchronously(
        device,
        &mut srb,
        perfDescriptor as *mut core::ffi::c_void,
        transferLength,
        1, // TRUE (WriteToDevice)
        core::ptr::null_mut(),
    );
    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_IOCTL,
        b"DeviceRestoreDefaultSpeed: Set Streaming command completed with status: 0x%X\n\0".as_ptr() as *const i8,
        status,
    );

    FREE_POOL(perfDescriptor as *mut core::ffi::c_void);
    WdfObjectDelete(WorkItem as WDFOBJECT);
}
#[no_mangle]
pub unsafe extern "C" fn StringsAreMatched(StringToMatch: *mut i8, TargetString: *mut i8) -> i32 {
    let length: usize;
    PAGED_CODE!();
    NT_ASSERT((!TargetString.is_null()) as i32);
    if StringToMatch.is_null() {
        return 1;
    }
    length = strlen(StringToMatch);
    if length == 0 {
        return (strlen(TargetString) == 0) as i32;
    }
    (strncmp(StringToMatch, TargetString, length) == 0) as i32
}
#[no_mangle]
pub unsafe extern "system" fn RequestSetContextFields(
    Request: WDFREQUEST,
    Handler: PSYNC_HANDLER,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let requestContext: *mut CDROM_REQUEST_CONTEXT = RequestGetContext(Request);
    let mut syncEvent: *mut KEVENT;

    syncEvent = ExAllocatePoolWithTag(
        0, // NonPagedPoolNx: ReactOS #defines it to NonPagedPool (no NX pool here)
        core::mem::size_of::<KEVENT>(),
        0x74764553u32, // CDROM_TAG_SYNC_EVENT
    ) as *mut KEVENT;

    if syncEvent.is_null() {
        // memory allocation failed.
        status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        // now, put the special synchronization information into the context
        (*requestContext).SyncRequired = 1u8; // TRUE
        (*requestContext).SyncEvent = syncEvent;
        (*requestContext).SyncCallback = Handler;

        status = STATUS_SUCCESS;
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestDuidGetDeviceIdProperty(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut deviceIdDescriptor: PSTORAGE_DEVICE_ID_DESCRIPTOR = core::ptr::null_mut();
    let mut descHeader: PSTORAGE_DESCRIPTOR_HEADER = core::ptr::null_mut();
    let mut propertyId: STORAGE_PROPERTY_ID = StorageDeviceIdProperty;

    *DataLength = 0;

    status = DeviceRetrieveDescriptor(
        (*DeviceExtension).Device,
        core::ptr::addr_of_mut!(propertyId),
        core::ptr::addr_of_mut!(deviceIdDescriptor) as *mut *mut STORAGE_DESCRIPTOR_HEADER,
    );

    if NT_SUCCESS(status) && deviceIdDescriptor.is_null() {
        status = STATUS_NOT_FOUND;
    }

    if NT_SUCCESS(status) {
        status = WdfRequestRetrieveOutputBuffer(
            Request,
            RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
            core::ptr::addr_of_mut!(descHeader) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
    }

    if NT_SUCCESS(status) {
        let offset: ULONG = (*descHeader).Size;
        let dest: *mut u8 = (descHeader as *mut u8).add(offset as usize);
        let outputBufferSize: usize = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize;

        status = RtlULongAdd(
            (*descHeader).Size,
            (*deviceIdDescriptor).Size,
            core::ptr::addr_of_mut!((*descHeader).Size),
        );

        if NT_SUCCESS(status) && (outputBufferSize < (*descHeader).Size as usize) {
            *DataLength = (*descHeader).Size as usize;
            status = STATUS_BUFFER_OVERFLOW;
        }

        if NT_SUCCESS(status) {
            let storageDuid: PSTORAGE_DEVICE_UNIQUE_IDENTIFIER = descHeader as PSTORAGE_DEVICE_UNIQUE_IDENTIFIER;
            (*storageDuid).StorageDeviceIdOffset = offset;
            RtlCopyMemory(
                dest as *mut core::ffi::c_void,
                deviceIdDescriptor as *const core::ffi::c_void,
                (*deviceIdDescriptor).Size as usize,
            );
            *DataLength = (*storageDuid).Size as usize;
            status = STATUS_SUCCESS;
        }

        FREE_POOL(deviceIdDescriptor as *mut core::ffi::c_void);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestDuidGetDeviceProperty(
    DeviceExtension: *const CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    RequestParameters: WDF_REQUEST_PARAMETERS,
    DataLength: *mut usize,
) -> NTSTATUS {
    unsafe {
        let mut status: NTSTATUS = STATUS_SUCCESS;
        let deviceDescriptor: *const STORAGE_DEVICE_DESCRIPTOR =
            (*DeviceExtension).DeviceDescriptor;
        let mut descHeader: *mut STORAGE_DESCRIPTOR_HEADER = core::ptr::null_mut();
        let mut storageDuid: *mut STORAGE_DEVICE_UNIQUE_IDENTIFIER;
        let mut dest: *mut u8;

        if deviceDescriptor.is_null() {
            status = STATUS_NOT_FOUND;
        }

        if NT_SUCCESS(status) {
            status = WdfRequestRetrieveOutputBuffer(
                Request,
                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize,
                core::ptr::addr_of_mut!(descHeader) as *mut PVOID,
                core::ptr::null_mut(),
            );
        }

        if NT_SUCCESS(status) && ((*deviceDescriptor).SerialNumberOffset == 0) {
            status = STATUS_NOT_FOUND;
        }

        // Use this info only if serial number is available.
        if NT_SUCCESS(status) {
            let offset: ULONG = (*descHeader).Size;
            let outputBufferSize: usize =
                RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength as usize;

            // Adjust required size and potential destination location.
            dest = (descHeader as *mut u8).add(offset as usize);

            status = RtlULongAdd(
                (*descHeader).Size,
                (*deviceDescriptor).Size,
                core::ptr::addr_of_mut!((*descHeader).Size),
            );

            if NT_SUCCESS(status) && (outputBufferSize < (*descHeader).Size as usize) {
                // Output buffer is too small.  Return error and make sure
                // the caller get info about required buffer size.
                *DataLength = (*descHeader).Size as usize;
                status = STATUS_BUFFER_OVERFLOW;
            }

            if NT_SUCCESS(status) {
                storageDuid = descHeader as *mut STORAGE_DEVICE_UNIQUE_IDENTIFIER;
                (*storageDuid).StorageDeviceOffset = offset;

                core::ptr::copy_nonoverlapping(
                    deviceDescriptor as *const u8,
                    dest,
                    (*deviceDescriptor).Size as usize,
                );

                *DataLength = (*storageDuid).Size as usize;
                status = STATUS_SUCCESS;
            }
        }

        status
    }
}

const SCSIOP_MODE_SENSE: u8 = 0x1a;  // was 0x5A (LLM-invented); bindgen-correct
#[no_mangle]
pub unsafe extern "C" fn DeviceRetrieveModeSenseUsingScratch(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    ModeSenseBuffer: *mut u8,
    Length: u32,
    PageCode: u8,
    PageControl: u8,
) -> u32 {
    // PAGED_CODE(); -- not applicable in Rust
    let mut status: i32;
    let mut transferSize: u32 = min(
        Length,
        (*DeviceExtension)
            .ScratchContext
            .ScratchBufferSize,
    );
    let mut cdb: CDB = core::mem::zeroed();

    ScratchBuffer_BeginUse(DeviceExtension);

    RtlZeroMemory(
        core::ptr::addr_of_mut!(cdb) as *mut core::ffi::c_void,
        core::mem::size_of::<CDB>(),
    );

    cdb.MODE_SENSE.as_mut().OperationCode = SCSIOP_MODE_SENSE;
    cdb.MODE_SENSE.as_mut().set_PageCode(PageCode);
    cdb.MODE_SENSE.as_mut().set_Pc(PageControl);
    cdb.MODE_SENSE.as_mut().AllocationLength = transferSize as u8;

    status = ScratchBuffer_ExecuteCdb(
        DeviceExtension,
        core::ptr::null_mut(),
        transferSize,
        1, // TRUE
        core::ptr::addr_of_mut!(cdb),
        6u8,
    );

    if NT_SUCCESS(status) {
        transferSize = min(
            Length,
            (*(*DeviceExtension)
                .ScratchContext
                .ScratchSrb)
                .DataTransferLength,
        );
        RtlCopyMemory(
            ModeSenseBuffer as *mut core::ffi::c_void,
            (*DeviceExtension)
                .ScratchContext
                .ScratchBuffer as *const core::ffi::c_void,
            transferSize as usize,
        );
    }

    ScratchBuffer_EndUse(DeviceExtension);

    transferSize
}
#[no_mangle]
pub unsafe extern "C" fn ModeSenseFindSpecificPage(
    mut ModeSenseBuffer: *mut u8,
    Length: usize,
    PageMode: u8,
    Use6BytesCdb: i32,
) -> *mut core::ffi::c_void {
    let mut result: *mut core::ffi::c_void = core::ptr::null_mut();
    let limit: usize = (ModeSenseBuffer as usize) + Length;

    let parameterHeaderLength: usize = if Use6BytesCdb != 0 {
        core::mem::size_of::<MODE_PARAMETER_HEADER>()
    } else {
        core::mem::size_of::<MODE_PARAMETER_HEADER10>()
    };

    if Length >= parameterHeaderLength {
        let blockDescriptorLength: u32;
        if Use6BytesCdb != 0 {
            blockDescriptorLength =
                (*(ModeSenseBuffer as *const MODE_PARAMETER_HEADER)).BlockDescriptorLength as u32;
        } else {
            let modeParam10 = ModeSenseBuffer as *const MODE_PARAMETER_HEADER10;
            blockDescriptorLength = (*modeParam10).BlockDescriptorLength[1] as u32;
        }
        ModeSenseBuffer = ModeSenseBuffer.add(parameterHeaderLength + blockDescriptorLength as usize);

        let size_through_page_length = core::mem::offset_of!(MODE_DISCONNECT_PAGE, PageLength)
            + core::mem::size_of::<u8>();

        while (ModeSenseBuffer as usize) + size_through_page_length < limit {
            let page_code = (*(ModeSenseBuffer as *const MODE_DISCONNECT_PAGE)).PageCode();
            let page_len = (*(ModeSenseBuffer as *const MODE_DISCONNECT_PAGE)).PageLength as usize;

            if page_code == PageMode {
                if (ModeSenseBuffer as usize) + page_len > limit {
                    result = core::ptr::null_mut();
                } else {
                    result = ModeSenseBuffer as *mut core::ffi::c_void;
                }
                break;
            }

            ModeSenseBuffer = ModeSenseBuffer.add(page_len + size_through_page_length);
        }
    }

    result
}
#[no_mangle]
pub unsafe extern "C" fn PerformEjectionControl(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    LockType: MEDIA_LOCK_TYPE,
    Lock: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut fileObjectContext: *mut FILE_OBJECT_CONTEXT = core::ptr::null_mut();
    let mut srb = core::mem::MaybeUninit::<SCSI_REQUEST_BLOCK>::uninit();
    let srb_ptr = srb.as_mut_ptr();
    let cdb: *mut CDB;
    let mut newLockCount: i32 = 0;
    let mut newProtectedLockCount: i32 = 0;
    let mut newInternalLockCount: i32 = 0;
    let mut newFileLockCount: i32 = 0;
    let mut countChanged: BOOLEAN = 0;
    let mut previouslyLocked: BOOLEAN = 0;
    let mut nowLocked: BOOLEAN = 0;

    unsafe {
        PAGED_CODE!();

        status = WdfWaitLockAcquire((*DeviceExtension).EjectSynchronizationLock, core::ptr::null_mut());
        if !NT_SUCCESS(status) {
            NT_ASSERT(0);
        }

        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"PerformEjectionControl: Received request for %s lock type\n\0".as_ptr() as *const i8,
            LockTypeStrings.0[LockType as usize],
        );

        loop {
            if LockType == SecureMediaLock {
                let fileObject: WDFFILEOBJECT = WdfRequestGetFileObject(Request);
                if fileObject.is_null() {
                    status = STATUS_INVALID_HANDLE;
                    TracePrint!(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_IOCTL,
                        b"FileObject does not match to the one in IRP_MJ_CREATE, KMDF returns NULL\n\0".as_ptr() as *const i8,
                    );
                    break;
                }
                fileObjectContext = FileObjectGetContext(fileObject);
                NT_ASSERT(((!fileObjectContext.is_null())) as i32);
            }

            NT_ASSERT((((*DeviceExtension).LockCount >= 0)) as i32);
            NT_ASSERT((((*DeviceExtension).ProtectedLockCount >= 0)) as i32);
            NT_ASSERT((((*DeviceExtension).InternalLockCount >= 0)) as i32);

            newLockCount = (*DeviceExtension).LockCount;
            newProtectedLockCount = (*DeviceExtension).ProtectedLockCount;
            newInternalLockCount = (*DeviceExtension).InternalLockCount;
            if !fileObjectContext.is_null() {
                newFileLockCount = (*fileObjectContext).LockCount as i32;
            }

            if Lock != 0 && LockType == SimpleMediaLock {
                newLockCount += 1;
                countChanged = 1;
            } else if Lock != 0 && LockType == SecureMediaLock {
                newFileLockCount += 1;
                newProtectedLockCount += 1;
                countChanged = 1;
            } else if Lock != 0 && LockType == InternalMediaLock {
                newInternalLockCount += 1;
                countChanged = 1;
            } else if Lock == 0 && LockType == SimpleMediaLock {
                if newLockCount != 0 {
                    newLockCount -= 1;
                    countChanged = 1;
                }
            } else if Lock == 0 && LockType == SecureMediaLock {
                if newFileLockCount == 0 || newProtectedLockCount == 0 {
                    status = STATUS_INVALID_DEVICE_STATE;
                    break;
                }
                newFileLockCount -= 1;
                newProtectedLockCount -= 1;
                countChanged = 1;
            } else if Lock == 0 && LockType == InternalMediaLock {
                NT_ASSERT(((newInternalLockCount != 0)) as i32);
                newInternalLockCount -= 1;
                countChanged = 1;
            }

            if (*DeviceExtension).LockCount != 0
                || (*DeviceExtension).ProtectedLockCount != 0
                || (*DeviceExtension).InternalLockCount != 0
            {
                previouslyLocked = 1;
            }
            if newLockCount != 0 || newProtectedLockCount != 0 || newInternalLockCount != 0 {
                nowLocked = 1;
            }

            if previouslyLocked != nowLocked {
                core::ptr::write_bytes(srb_ptr as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                (*srb_ptr).CdbLength = 6;
                (*srb_ptr).TimeOutValue = (*DeviceExtension).TimeOutValue;
                cdb = core::ptr::addr_of_mut!((*srb_ptr).Cdb) as *mut CDB;
                (*cdb).MEDIA_REMOVAL.as_mut().OperationCode = SCSIOP_MEDIUM_REMOVAL as u8;
                (*cdb).MEDIA_REMOVAL.as_mut().set_Prevent(Lock);
                status = DeviceSendSrbSynchronously(
                    (*DeviceExtension).Device,
                    srb_ptr,
                    core::ptr::null_mut(),
                    0,
                    0,
                    Request,
                );
            }
            break;
        }

        // Exit label
        if countChanged != 0 && NT_SUCCESS(status) {
            (*DeviceExtension).LockCount = newLockCount;
            (*DeviceExtension).ProtectedLockCount = newProtectedLockCount;
            (*DeviceExtension).InternalLockCount = newInternalLockCount;
            if !fileObjectContext.is_null() {
                (*fileObjectContext).LockCount = newFileLockCount as u32;
            }
        }

        WdfWaitLockRelease((*DeviceExtension).EjectSynchronizationLock);

        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_IOCTL,
            b"PerformEjectionControl: %!STATUS!, Count Changed: %d, Command Sent: %d, Current Counts: Internal: %x  Secure: %x  Simple: %x\n\0".as_ptr() as *const i8,
            status,
            countChanged as i32,
            (previouslyLocked != nowLocked) as i32,
            (*DeviceExtension).InternalLockCount,
            (*DeviceExtension).ProtectedLockCount,
            (*DeviceExtension).LockCount,
        );

        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeviceUnlockExclusive(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    FileObject: *mut core::ffi::c_void,
    IgnorePreviousMediaChanges: i32,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let cdData = &raw mut (*DeviceExtension).DeviceAdditionalData as *mut CDROM_DATA;
    let info = (*DeviceExtension).MediaChangeDetectionInfo;
    let mut ANPending: i32;
    let mut requestInUse: i32;

    PAGED_CODE!();

    if EXCLUSIVE_MODE(cdData) == 0 {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_IOCTL,
            b"RequestHandleExclusiveAccessUnlockDevice: Device not locked for exclusive access, can't unlock device.\n\0"
                .as_ptr() as *const i8,
        );
        status = STATUS_INVALID_DEVICE_REQUEST;
    } else if EXCLUSIVE_OWNER(cdData, FileObject) == 0 {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_IOCTL,
            b"RequestHandleExclusiveAccessUnlockDevice: Unable to unlock device, invalid file object\n\0"
                .as_ptr() as *const i8,
        );
        status = STATUS_INVALID_HANDLE;
    }

    if status >= 0 {
        if IgnorePreviousMediaChanges == 0 {
            let mut previousMediaState: i32 = MediaUnknown;
            DeviceSetMediaChangeStateEx(DeviceExtension, MediaUnavailable, &raw mut previousMediaState);
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_IOCTL,
                b"DeviceUnlockExclusive: Changing the media state to MediaUnavailable\n\0"
                    .as_ptr() as *const i8,
            );
            DeviceSetMediaChangeStateEx(DeviceExtension, previousMediaState, core::ptr::null_mut());
        }

        if IsVolumeMounted((*DeviceExtension).DeviceObject) != 0 {
            (*(*DeviceExtension).DeviceObject).Flags |= DO_VERIFY_VOLUME;
        }

        (*cdData).Mmc.WriteAllowed = 0;
        (*cdData).Mmc.UpdateState = CdromMmcUpdateRequired;

        DeviceSendNotification(
            DeviceExtension,
            &raw const GUID_IO_CDROM_EXCLUSIVE_UNLOCK,
            0,
            core::ptr::null_mut(),
        );

        InterlockedExchangePointer(
            &raw mut (*cdData).ExclusiveOwner as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );

        if !info.is_null() && (*info).AsynchronousNotificationSupported != 0 {
            ANPending = (*info).ANSignalPendingDueToExclusiveLock as i32;
            (*info).ANSignalPendingDueToExclusiveLock = 0;

            if ANPending != 0 && (*info).MediaChangeDetectionDisableCount == 0 {
                requestInUse = InterlockedCompareExchange(
                    &raw mut (*info).MediaChangeRequestInUse,
                    1,
                    0,
                );

                if requestInUse == 0 {
                    RequestSetupMcnSyncIrp(DeviceExtension);
                    IoCallDriver(
                        (*DeviceExtension).DeviceObject,
                        (*info).MediaChangeSyncIrp,
                    );
                }
            }
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestCompletion(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    Status: NTSTATUS,
    Information: usize,
) {
    let mut status = Status;

    // IoIsErrorUserInduced(Status) — set the device to be verified / raise a
    // hard error so the FS / shell prompts for media. The C does this for every
    // completion; the original Rust dropped it (found by rustconv semdiff).
    // ntstatus.h values.
    const STATUS_WRONG_VOLUME: i32 = 0xC0000012u32 as i32;
    const STATUS_NO_MEDIA_IN_DEVICE: i32 = 0xC0000013u32 as i32;
    const STATUS_UNRECOGNIZED_MEDIA: i32 = 0xC0000014u32 as i32;
    const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32;
    const STATUS_MEDIA_WRITE_PROTECTED: i32 = 0xC00000A2u32 as i32;
    const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32;
    const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
    let is_error_user_induced = status == STATUS_DEVICE_NOT_READY
        || status == STATUS_IO_TIMEOUT
        || status == STATUS_MEDIA_WRITE_PROTECTED
        || status == STATUS_NO_MEDIA_IN_DEVICE
        || status == STATUS_VERIFY_REQUIRED
        || status == STATUS_UNRECOGNIZED_MEDIA
        || status == STATUS_WRONG_VOLUME;
    if is_error_user_induced {
        let irp = WdfRequestWdmGetIrp(Request);
        if !irp.is_null() && !(*irp).Tail.Overlay.Thread.is_null() {
            IoSetHardErrorOrVerifyDevice(irp, (*DeviceExtension).DeviceObject);
        }
    }

    // IMAPI expects ERROR_DEV_NOT_EXISTS on a surprise-removed recorder, else it
    // retries WRITE commands for up to 3 minutes.
    if !NT_SUCCESS(status) && (*DeviceExtension).SurpriseRemoved != 0 {
        status = STATUS_DEVICE_DOES_NOT_EXIST;
    }

    WdfRequestCompleteWithInformation(Request, status, Information as ULONG_PTR);
}
#[no_mangle]
pub unsafe extern "system" fn RequestDummyCompletionRoutine(
    Request: WDFREQUEST,
    Target: WDFIOTARGET,
    Params: PWDF_REQUEST_COMPLETION_PARAMS,
    Context: WDFCONTEXT,
) {
    let _ = Target;
    let _ = Params;
    let _ = Context;
    WdfRequestCompleteWithInformation(
        Request,
        WdfRequestGetStatus(Request),
        WdfRequestGetInformation(Request),
    );
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSendPowerDownProcessRequest(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    CompletionRoutine: *const core::ffi::c_void,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    // Forward declarations for unknown symbols

    let mut status: NTSTATUS;
    let mut requestSent: BOOLEAN = 0;
    let mut shouldRetry: BOOLEAN = 1;
    let cdb: *mut CDB = core::ptr::addr_of_mut!((*DeviceExtension).PowerContext.Srb.Cdb) as *mut CDB;
    let mut timeoutValue: ULONG = (*DeviceExtension).TimeOutValue;
    let mut retryCount: ULONG = 1;

    (*DeviceExtension).PowerContext.RetryIntervalIn100ns = 0;
    status = PowerContextReuseRequest(DeviceExtension);
    RequestClearSendTime((*DeviceExtension).PowerContext.PowerRequest);

    if !NT_SUCCESS(status) {
        return status;
    }

    match (*DeviceExtension).PowerContext.PowerChangeState.PowerDown {
        PowerDownDeviceInitial | PowerDownDeviceQuiesced | PowerDownDeviceStopped => {}
        PowerDownDeviceLocked => {
            retryCount = MAXIMUM_RETRIES;
            timeoutValue = (*DeviceExtension).TimeOutValue;
        }
        PowerDownDeviceFlushed => {
            let mut secondsRemaining: ULONG = 0;
            PoQueryWatchdogTime((*DeviceExtension).LowerPdo as *mut core::ffi::c_void, &mut secondsRemaining);
            if secondsRemaining == 0 {
                retryCount = MAXIMUM_RETRIES;
                timeoutValue = SCSI_CDROM_TIMEOUT;
            } else {
                if secondsRemaining >= 32 {
                    retryCount = (secondsRemaining - 30) / SCSI_CDROM_TIMEOUT + 1;
                    timeoutValue = SCSI_CDROM_TIMEOUT;
                    if retryCount > MAXIMUM_RETRIES {
                        retryCount = MAXIMUM_RETRIES;
                    }
                    if retryCount == 1 {
                        timeoutValue = secondsRemaining - 30;
                    }
                } else {
                    retryCount = 1;
                    timeoutValue = 2;
                }
            }
        }
        _ => {
            NT_ASSERT(0); // FALSE
            status = STATUS_NOT_IMPLEMENTED;
            return status;
        }
    }

    (*DeviceExtension).PowerContext.RetryCount = retryCount;

    while shouldRetry != 0 {
        (*DeviceExtension).PowerContext.Srb.SrbFlags = SRB_FLAGS_NO_DATA_TRANSFER
            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER
            | SRB_FLAGS_NO_QUEUE_FREEZE
            | SRB_FLAGS_BYPASS_LOCKED_QUEUE
            | SRB_FLAGS_D3_PROCESSING;

        (*DeviceExtension).PowerContext.Srb.Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*DeviceExtension).PowerContext.Srb.TimeOutValue = timeoutValue;

        if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown == PowerDownDeviceInitial {
            (*DeviceExtension).PowerContext.Srb.Function = SRB_FUNCTION_LOCK_QUEUE as u8;
        } else if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown == PowerDownDeviceLocked {
            (*DeviceExtension).PowerContext.Srb.Function = SRB_FUNCTION_QUIESCE_DEVICE as u8;
        } else if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown == PowerDownDeviceQuiesced {
            (*DeviceExtension).PowerContext.Srb.CdbLength = 10;
            (*cdb).SYNCHRONIZE_CACHE10.as_mut().OperationCode = SCSIOP_SYNCHRONIZE_CACHE as u8;
        } else if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown == PowerDownDeviceFlushed {
            (*DeviceExtension).PowerContext.Srb.CdbLength = 6;
            (*cdb).START_STOP.as_mut().OperationCode = SCSIOP_START_STOP_UNIT;
            (*cdb).START_STOP.as_mut().set_Start(0);
            (*cdb).START_STOP.as_mut().set_Immediate(1);
        } else if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown == PowerDownDeviceStopped {
            (*DeviceExtension).PowerContext.Srb.Function = SRB_FUNCTION_UNLOCK_QUEUE as u8;
        }

        if !CompletionRoutine.is_null() {
            WdfRequestSetCompletionRoutine(
                (*DeviceExtension).PowerContext.PowerRequest,
                core::mem::transmute::<*const core::ffi::c_void, PFN_WDF_REQUEST_COMPLETION_ROUTINE>(CompletionRoutine),
                Context,
            );
        }

        status = RequestSend(
            DeviceExtension,
            (*DeviceExtension).PowerContext.PowerRequest,
            (*DeviceExtension).IoTarget,
            if CompletionRoutine.is_null() { WDF_REQUEST_SEND_OPTION_SYNCHRONOUS } else { 0 },
            &mut requestSent,
        );

        if requestSent != 0 {
            if CompletionRoutine.is_null()
                && SRB_STATUS((*DeviceExtension).PowerContext.Srb.SrbStatus) != SRB_STATUS_SUCCESS
            {
                let command_name: *const i8 = if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown
                    == PowerDownDeviceQuiesced
                {
                    b"SYNC CACHE\0".as_ptr() as *const i8
                } else {
                    b"STOP UNIT\0".as_ptr() as *const i8
                };

                TracePrint!(
                    TRACE_LEVEL_ERROR,
                    TRACE_FLAG_POWER,
                    b"%p\tError occured when issuing %s command to device. Srb %p, Status %x\n\0"
                        .as_ptr() as *const i8,
                    (*DeviceExtension).PowerContext.PowerRequest,
                    command_name,
                    &raw const (*DeviceExtension).PowerContext.Srb as *const core::ffi::c_void,
                    (*DeviceExtension).PowerContext.Srb.SrbStatus as u32,
                );

                NT_ASSERT(
                    ((((*DeviceExtension).PowerContext.Srb.SrbStatus as u32) & SRB_STATUS_QUEUE_FROZEN)
                        == 0) as i32,
                );

                let retry_remaining = retryCount - (*DeviceExtension).PowerContext.RetryCount;
                shouldRetry = RequestSenseInfoInterpret(
                    DeviceExtension,
                    (*DeviceExtension).PowerContext.PowerRequest as *mut core::ffi::c_void,
                    &raw mut (*DeviceExtension).PowerContext.Srb,
                    retry_remaining,
                    &mut status,
                    &raw mut (*DeviceExtension).PowerContext.RetryIntervalIn100ns as *mut LARGE_INTEGER,
                );

                if shouldRetry != 0 {
                    let current = (*DeviceExtension).PowerContext.RetryCount;
                    (*DeviceExtension).PowerContext.RetryCount = current - 1;
                    if current == 0 {
                        shouldRetry = 0;
                    }
                }
            } else {
                shouldRetry = 0;
            }
        } else {
            shouldRetry = 0;
        }

        if shouldRetry != 0 {
            let mut interval: LARGE_INTEGER = core::mem::zeroed();
            interval.QuadPart = -((*DeviceExtension).PowerContext.RetryIntervalIn100ns);
            KeDelayExecutionThread(0i8, 0, &mut interval);

            status = PowerContextReuseRequest(DeviceExtension);
            if !NT_SUCCESS(status) {
                shouldRetry = 0;
            }
        }
    }

    if (*DeviceExtension).PowerContext.PowerChangeState.PowerDown == PowerDownDeviceQuiesced {
        KeQueryTickCount(&raw mut (*DeviceExtension).PowerContext.Step1CompleteTime);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn RequestSend(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    IoTarget: WDFIOTARGET,
    Flags: ULONG,
    RequestSent: *mut BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut requestSent: BOOLEAN;
    let mut options: WDF_REQUEST_SEND_OPTIONS = WDF_REQUEST_SEND_OPTIONS {
        Size: 0,
        Flags: 0,
        Timeout: 0,
    };

    let zpodd_info = core::ptr::read_volatile(core::ptr::addr_of!((*DeviceExtension).ZeroPowerODDInfo));
    if !zpodd_info.is_null() {
        let _ = core::ptr::read_volatile(core::ptr::addr_of!((*zpodd_info).InZeroPowerState));
    }

    if NT_SUCCESS(status) {
        WDF_REQUEST_SEND_OPTIONS_INIT(&mut options, Flags);
        RequestSetSentTime(Request);

        // _Analysis_assume_(options.Timeout != 0);
        requestSent = WdfRequestSend(Request, IoTarget, &mut options);
        // _Analysis_assume_(options.Timeout == 0);

        if requestSent == 0 || (Flags & WDF_REQUEST_SEND_OPTION_SYNCHRONOUS) != 0 {
            status = WdfRequestGetStatus(Request);
            if requestSent == 0 {
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_GENERAL,
                    b"WdfRequestSend failed: %lx\n\0".as_ptr() as *const i8,
                    status,
                );
            }
        } else {
            status = STATUS_SUCCESS;
        }

        if !RequestSent.is_null() {
            *RequestSent = requestSent;
        }
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation. All rights reserved.
//
// Module Name:
//
//     common.c
//
// Abstract:
//
//     shared private routines for cdrom.sys
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
//
// #include "ntddk.h"
// #include "ntddstor.h"
// #include "ntstrsafe.h"
//
// #include "cdrom.h"
// #include "scratch.h"
//
//
// #ifdef DEBUG_USE_WPP
// #include "common.tmh"
// #endif
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceGetParameter)
// #pragma alloc_text(PAGE, DeviceSetParameter)
// #pragma alloc_text(PAGE, DeviceSendSrbSynchronously)
// #pragma alloc_text(PAGE, DevicePickDvdRegion)
// #pragma alloc_text(PAGE, StringsAreMatched)
// #pragma alloc_text(PAGE, PerformEjectionControl)
// #pragma alloc_text(PAGE, DeviceFindFeaturePage)
// #pragma alloc_text(PAGE, DevicePrintAllFeaturePages)
// #pragma alloc_text(PAGE, DeviceRegisterInterface)
// #pragma alloc_text(PAGE, DeviceRestoreDefaultSpeed)
// #pragma alloc_text(PAGE, DeviceSendRequestSynchronously)
// #pragma alloc_text(PAGE, MediaReadCapacity)
// #pragma alloc_text(PAGE, MediaReadCapacityDataInterpret)
// #pragma alloc_text(PAGE, DeviceRetrieveModeSenseUsingScratch)
// #pragma alloc_text(PAGE, ModeSenseFindSpecificPage)
// #pragma alloc_text(PAGE, DeviceUnlockExclusive)
//
// #endif
//
// LPCSTR LockTypeStrings[] = {"Simple",
//                             "Secure",
//                             "Internal"
//                             };
//
// VOID
// RequestSetReceivedTime(
//      WDFREQUEST Request
//     )
// {
//     PCDROM_REQUEST_CONTEXT requestContext = RequestGetContext(Request);
//     LARGE_INTEGER          temp;
//
//     KeQueryTickCount(&temp);
//
//     requestContext->TimeReceived = temp;
//
//     return;
// }
//
// VOID
// RequestSetSentTime(
//      WDFREQUEST Request
//     )
// {
//     PCDROM_REQUEST_CONTEXT requestContext = RequestGetContext(Request);
//     LARGE_INTEGER          temp;
//
//     KeQueryTickCount(&temp);
//
//     if (requestContext->TimeSentDownFirstTime.QuadPart == 0)
//     {
//         requestContext->TimeSentDownFirstTime = temp;
//     }
//
//     requestContext->TimeSentDownLasttTime = temp;
//
//     if (requestContext->OriginalRequest != NULL)
//     {
//         PCDROM_REQUEST_CONTEXT originalRequestContext = RequestGetContext(requestContext->OriginalRequest);
//
//         if (originalRequestContext->TimeSentDownFirstTime.QuadPart == 0)
//         {
//             originalRequestContext->TimeSentDownFirstTime = temp;
//         }
//
//         originalRequestContext->TimeSentDownLasttTime = temp;
//     }
//
//     return;
// }
//
// VOID
// RequestClearSendTime(
//      WDFREQUEST Request
//     )
// /*
// Routine Description:
//
//     This function is used to clean SentTime fields in reusable request context.
//
// Arguments:
//     Request -
//
// Return Value:
//     N/A
//
// */
// {
//     PCDROM_REQUEST_CONTEXT requestContext = RequestGetContext(Request);
//
//     requestContext->TimeSentDownFirstTime.QuadPart = 0;
//     requestContext->TimeSentDownLasttTime.QuadPart = 0;
//
//     return;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// VOID
// DeviceGetParameter(
//      PCDROM_DEVICE_EXTENSION    DeviceExtension,
//      PWSTR                  SubkeyName,
//      PWSTR                      ParameterName,
//      PULONG                  ParameterValue  // also default value
//     )
// /*++
// Routine Description:
//
//     retrieve device parameter from registry.
//
// Arguments:
//
//     DeviceExtension - device context.
//
//     SubkeyName - name of subkey
//
//     ParameterName - the registry parameter to be retrieved
//
// Return Value:
//
//     ParameterValue - registry value retrieved
//
// --*/
// {
//     NTSTATUS        status;
//     WDFKEY          rootKey = NULL;
//     WDFKEY          subKey = NULL;
//     UNICODE_STRING  registrySubKeyName;
//     UNICODE_STRING  registryValueName;
//     ULONG           defaultParameterValue;
//
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&registryValueName, ParameterName);
//
//     if (SubkeyName != NULL)
//     {
//         RtlInitUnicodeString(&registrySubKeyName, SubkeyName);
//     }
//
//     // open the hardware key
//     status = WdfDeviceOpenRegistryKey(DeviceExtension->Device,
//                                       PLUGPLAY_REGKEY_DEVICE,
//                                       KEY_READ,
//                                       WDF_NO_OBJECT_ATTRIBUTES,
//                                       &rootKey);
//
//     // open the sub key
//     if (NT_SUCCESS(status) && (SubkeyName != NULL))
//     {
//         status = WdfRegistryOpenKey(rootKey,
//                                     &registrySubKeyName,
//                                     KEY_READ,
//                                     WDF_NO_OBJECT_ATTRIBUTES,
//                                     &subKey);
//
//         if (!NT_SUCCESS(status))
//         {
//             WdfRegistryClose(rootKey);
//             rootKey = NULL;
//         }
//     }
//
//     if (NT_SUCCESS(status) && (rootKey != NULL))
//     {
//         defaultParameterValue = *ParameterValue;
//
//         status = WdfRegistryQueryULong((subKey != NULL) ? subKey : rootKey,
//                                        &registryValueName,
//                                        ParameterValue);
//
//         if (!NT_SUCCESS(status))
//         {
//             *ParameterValue = defaultParameterValue; // use default value
//         }
//     }
//
//     // close what we open
//     if (subKey != NULL)
//     {
//         WdfRegistryClose(subKey);
//         subKey = NULL;
//     }
//
//     if (rootKey != NULL)
//     {
//         WdfRegistryClose(rootKey);
//         rootKey = NULL;
//     }
//
//     // Windows 2000 SP3 uses the driver-specific key, so look in there
//     if (!NT_SUCCESS(status))
//     {
//         // open the software key
//         status = WdfDeviceOpenRegistryKey(DeviceExtension->Device,
//                                           PLUGPLAY_REGKEY_DRIVER,
//                                           KEY_READ,
//                                           WDF_NO_OBJECT_ATTRIBUTES,
//                                           &rootKey);
//
//         // open the sub key
//         if (NT_SUCCESS(status) && (SubkeyName != NULL))
//         {
//             status = WdfRegistryOpenKey(rootKey,
//                                         &registrySubKeyName,
//                                         KEY_READ,
//                                         WDF_NO_OBJECT_ATTRIBUTES,
//                                         &subKey);
//
//             if (!NT_SUCCESS(status))
//             {
//                 WdfRegistryClose(rootKey);
//                 rootKey = NULL;
//             }
//         }
//
//         if (NT_SUCCESS(status) && (rootKey != NULL))
//         {
//             defaultParameterValue = *ParameterValue;
//
//             status = WdfRegistryQueryULong((subKey != NULL) ? subKey : rootKey,
//                                            &registryValueName,
//                                            ParameterValue);
//
//             if (!NT_SUCCESS(status))
//             {
//                 *ParameterValue = defaultParameterValue; // use default value
//             }
//             else
//             {
//                 // Migrate the value over to the device-specific key
//                 DeviceSetParameter(DeviceExtension, SubkeyName, ParameterName, *ParameterValue);
//             }
//         }
//
//         // close what we open
//         if (subKey != NULL)
//         {
//             WdfRegistryClose(subKey);
//             subKey = NULL;
//         }
//
//         if (rootKey != NULL)
//         {
//             WdfRegistryClose(rootKey);
//             rootKey = NULL;
//         }
//     }
//
//     return;
//
// } // end DeviceetParameter()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceSetParameter(
//      PCDROM_DEVICE_EXTENSION    DeviceExtension,
//      PWSTR                SubkeyName,
//      PWSTR                      ParameterName,
//      ULONG                      ParameterValue
//     )
// /*++
// Routine Description:
//
//     set parameter to registry.
//
// Arguments:
//
//     DeviceExtension - device context.
//
//     SubkeyName - name of subkey
//
//     ParameterName - the registry parameter to be retrieved
//
//     ParameterValue - registry value to be set
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status;
//     WDFKEY          rootKey = NULL;
//     WDFKEY          subKey = NULL;
//     UNICODE_STRING  registrySubKeyName;
//     UNICODE_STRING  registryValueName;
//
//     PAGED_CODE();
//
//     RtlInitUnicodeString(&registryValueName, ParameterName);
//
//     if (SubkeyName != NULL)
//     {
//         RtlInitUnicodeString(&registrySubKeyName, SubkeyName);
//     }
//
//     // open the hardware key
//     status = WdfDeviceOpenRegistryKey(DeviceExtension->Device,
//                                       PLUGPLAY_REGKEY_DEVICE,
//                                       KEY_READ | KEY_WRITE,
//                                       WDF_NO_OBJECT_ATTRIBUTES,
//                                       &rootKey);
//
//     // open the sub key
//     if (NT_SUCCESS(status) && (SubkeyName != NULL))
//     {
//         status = WdfRegistryOpenKey(rootKey,
//                                     &registrySubKeyName,
//                                     KEY_READ | KEY_WRITE,
//                                     WDF_NO_OBJECT_ATTRIBUTES,
//                                     &subKey);
//
//         if (!NT_SUCCESS(status))
//         {
//             WdfRegistryClose(rootKey);
//             rootKey = NULL;
//         }
//     }
//
//     if (NT_SUCCESS(status) && (rootKey != NULL))
//     {
//         status = WdfRegistryAssignULong((subKey != NULL) ? subKey : rootKey,
//                                         &registryValueName,
//                                         ParameterValue);
//     }
//
//     // close what we open
//     if (subKey != NULL)
//     {
//         WdfRegistryClose(subKey);
//         subKey = NULL;
//     }
//
//     if (rootKey != NULL)
//     {
//         WdfRegistryClose(rootKey);
//         rootKey = NULL;
//     }
//
//     return status;
//
// } // end DeviceSetParameter()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceSendRequestSynchronously(
//      WDFDEVICE    Device,
//      WDFREQUEST   Request,
//      BOOLEAN      RequestFormated
//     )
// /*++
// Routine Description:
//
//     send a request to lower driver synchronously.
//
// Arguments:
//
//     Device - device object.
//
//     Request - request object
//
//     RequestFormated - if the request is already formatted, will no do it in this function
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//     BOOLEAN                     requestCancelled = FALSE;
//     PCDROM_REQUEST_CONTEXT      requestContext = RequestGetContext(Request);
//
//     PAGED_CODE();
//
//     if (!RequestFormated)
//     {
//         // set request up for sending down
//         WdfRequestFormatRequestUsingCurrentType(Request);
//     }
//
//     // get cancellation status for the original request
//     if (requestContext->OriginalRequest != NULL)
//     {
//         requestCancelled = WdfRequestIsCanceled(requestContext->OriginalRequest);
//     }
//
//     if (!requestCancelled)
//     {
//         status = RequestSend(deviceExtension,
//                              Request,
//                              deviceExtension->IoTarget,
//                              WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
//                              NULL);
//     }
//     else
//     {
//         status = STATUS_CANCELLED;
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceSendSrbSynchronously(
//      WDFDEVICE           Device,
//      PSCSI_REQUEST_BLOCK Srb,
//      PVOID           BufferAddress,
//      ULONG               BufferLength,
//      BOOLEAN             WriteToDevice,
//      WDFREQUEST      OriginalRequest
//     )
// /*++
// Routine Description:
//
//     Send a SRB structure to lower driver synchronously.
//
//     Process of this function:
//     1. Allocate SenseBuffer; Create Request; Allocate MDL
//     2. Do following loop if necessary
//        2.1 Reuse Request
//        2.2 Format Srb, Irp
//        2.3 Send Request
//        2.4 Error Intepret and retry decision making.
//     3. Release all allocated resosurces.
//
// Arguments:
//
//     Device - device object.
//
//     Request - request object
//
//     RequestFormated - if the request is already formatted, will no do it in this function
//
// Return Value:
//     NTSTATUS
//
// NOTE:
// The caller needs to setup following fields before calling this routine.
//         srb.CdbLength
//         srb.TimeOutValue
//         cdb
//
// BufferLength and WriteToDevice to control the data direction of the device
//     BufferLength = 0: No data transfer
//     BufferLenth != 0 && !WriteToDevice: get data from device
//     BufferLenth != 0 && WriteToDevice: send data to device
// --*/
// {
//     NTSTATUS                status;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     PCDROM_PRIVATE_FDO_DATA fdoData = deviceExtension->PrivateFdoData;
//     PUCHAR                  senseInfoBuffer = NULL;
//     ULONG                   retryCount = 0;
//     BOOLEAN                 retry = FALSE;
//     ULONG                   ioctlCode = 0;
//     WDFREQUEST              request = NULL;
//     PIRP                    irp = NULL;
//     PIO_STACK_LOCATION      nextStack = NULL;
//     PMDL                    mdlAddress = NULL;
//     BOOLEAN                 memoryLocked = FALSE;
//     WDF_OBJECT_ATTRIBUTES   attributes;
//     PZERO_POWER_ODD_INFO    zpoddInfo = deviceExtension->ZeroPowerODDInfo;
//
//     PAGED_CODE();
//
//     // NOTE: This code is only pagable because we are not freezing
//     //       the queue.  Allowing the queue to be frozen from a pagable
//     //       routine could leave the queue frozen as we try to page in
//     //       the code to unfreeze the queue.  The result would be a nice
//     //       case of deadlock.  Therefore, since we are unfreezing the
//     //       queue regardless of the result, just set the NO_FREEZE_QUEUE
//     //       flag in the SRB.
//     NT_ASSERT(KeGetCurrentIrql() < DISPATCH_LEVEL);
//
//     //1. allocate SenseBuffer and initiate Srb common fields
//     //   these fields will not be changed by lower driver.
//     {
//         // Write length to SRB.
//         Srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//         Srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         Srb->SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//
//         // Sense buffer is in aligned nonpaged pool.
//         senseInfoBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                                 SENSE_BUFFER_SIZE,
//                                                 CDROM_TAG_SENSE_INFO);
//
//         if (senseInfoBuffer == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "DeviceSendSrbSynchronously: Can't allocate MDL\n"));
//
//             goto Exit;
//         }
//
//         Srb->SenseInfoBuffer = senseInfoBuffer;
//         Srb->DataBuffer = BufferAddress;
//
//         // set timeout value to default value if it's not specifically set by caller.
//         if (Srb->TimeOutValue == 0)
//         {
//             Srb->TimeOutValue = deviceExtension->TimeOutValue;
//         }
//     }
//
//     //2. Create Request object
//     {
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_REQUEST_CONTEXT);
//
//         status = WdfRequestCreate(&attributes,
//                                   deviceExtension->IoTarget,
//                                   &request);
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "DeviceSendSrbSynchronously: Can't create request: %lx\n",
//                        status));
//
//             goto Exit;
//         }
//
//         irp = WdfRequestWdmGetIrp(request);
//     }
//
//     // 3. Build an MDL for the data buffer and stick it into the irp.
//     if (BufferAddress != NULL)
//     {
//         mdlAddress = IoAllocateMdl( BufferAddress,
//                                     BufferLength,
//                                     FALSE,
//                                     FALSE,
//                                     irp );
//         if (mdlAddress == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "DeviceSendSrbSynchronously: Can't allocate MDL\n"));
//
//             goto Exit;
//         }
//
//         _SEH2_TRY
//         {
//             MmProbeAndLockPages(mdlAddress,
//                                 KernelMode,
//                                 (WriteToDevice ? IoReadAccess : IoWriteAccess));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             status = _SEH2_GetExceptionCode();
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "DeviceSendSrbSynchronously: Exception %lx locking buffer\n", status));
//
//             _SEH2_YIELD(goto Exit);
//         }
//         _SEH2_END;
//
//         memoryLocked = TRUE;
//     }
//
//     // 4. Format Srb, Irp; Send request and retry when necessary
//     do
//     {
//         // clear the control variable.
//         retry = FALSE;
//
//         // 4.1 reuse the request object; set originalRequest field.
//         {
//             WDF_REQUEST_REUSE_PARAMS    params;
//             PCDROM_REQUEST_CONTEXT      requestContext = NULL;
//
//             // deassign the MdlAddress, this is the value we assign explicitly.
//             // doing this can prevent WdfRequestReuse to release the Mdl unexpectly.
//             if (irp->MdlAddress)
//             {
//                 irp->MdlAddress = NULL;
//             }
//
//             WDF_REQUEST_REUSE_PARAMS_INIT(&params,
//                                           WDF_REQUEST_REUSE_NO_FLAGS,
//                                           STATUS_SUCCESS);
//
//             status = WdfRequestReuse(request, &params);
//
//             if (!NT_SUCCESS(status))
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                            "DeviceSendSrbSynchronously: WdfRequestReuse failed, %!STATUS!\n",
//                            status));
//                 // exit the loop.
//                 break;
//             }
//
//             // WDF requests to format the request befor sending it
//             status = WdfIoTargetFormatRequestForInternalIoctlOthers(deviceExtension->IoTarget,
//                                                                     request,
//                                                                     ioctlCode,
//                                                                     NULL, NULL,
//                                                                     NULL, NULL,
//                                                                     NULL, NULL);
//
//             if (!NT_SUCCESS(status))
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                            "DeviceSendSrbSynchronously: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n",
//                            status));
//                 // exit the loop.
//                 break;
//             }
//
//             requestContext = RequestGetContext(request);
//             requestContext->OriginalRequest = OriginalRequest;
//         }
//
//         // 4.2 Format Srb and Irp
//         {
//             Srb->OriginalRequest = irp;
//             Srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//             Srb->DataTransferLength = BufferLength;
//             Srb->SrbFlags = deviceExtension->SrbFlags;
//
//             // Disable synchronous transfer for these requests.
//             SET_FLAG(Srb->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//             SET_FLAG(Srb->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//
//             if (BufferAddress != NULL)
//             {
//                 if (WriteToDevice)
//                 {
//                     SET_FLAG(Srb->SrbFlags, SRB_FLAGS_DATA_OUT);
//                     ioctlCode = IOCTL_SCSI_EXECUTE_OUT;
//                 }
//                 else
//                 {
//                     SET_FLAG(Srb->SrbFlags, SRB_FLAGS_DATA_IN);
//                     ioctlCode = IOCTL_SCSI_EXECUTE_IN;
//                 }
//             }
//             else
//             {
//                 ioctlCode = IOCTL_SCSI_EXECUTE_NONE;
//             }
//
//
//             // Zero out status.
//             Srb->ScsiStatus = 0;
//             Srb->SrbStatus = 0;
//             Srb->NextSrb = NULL;
//
//             // irp related fields
//             irp->MdlAddress = mdlAddress;
//
//             nextStack = IoGetNextIrpStackLocation(irp);
//
//             nextStack->MajorFunction = IRP_MJ_SCSI;
//             nextStack->Parameters.DeviceIoControl.IoControlCode = ioctlCode;
//             nextStack->Parameters.Scsi.Srb = Srb;
//         }
//
//         // 4.3 send Request to lower driver.
//         status = DeviceSendRequestSynchronously(Device, request, TRUE);
//
//         if (status != STATUS_CANCELLED)
//         {
//             NT_ASSERT(SRB_STATUS(Srb->SrbStatus) != SRB_STATUS_PENDING);
//             NT_ASSERT(status != STATUS_PENDING);
//             NT_ASSERT(!(Srb->SrbStatus & SRB_STATUS_QUEUE_FROZEN));
//
//             // 4.4 error process.
//             if (SRB_STATUS(Srb->SrbStatus) != SRB_STATUS_SUCCESS)
//             {
//                 LONGLONG    retryIntervalIn100ns = 0;
//
//                 // Update status and determine if request should be retried.
//                 retry = RequestSenseInfoInterpret(deviceExtension,
//                                                   request,
//                                                   Srb,
//                                                   retryCount,
//                                                   &status,
//                                                   &retryIntervalIn100ns);
//
//                 if (retry)
//                 {
//                     LARGE_INTEGER t;
//                     t.QuadPart = -retryIntervalIn100ns;
//                     retryCount++;
//                     KeDelayExecutionThread(KernelMode, FALSE, &t);
//                 }
//             }
//             else
//             {
//                 // Request succeeded.
//                 fdoData->LoggedTURFailureSinceLastIO = FALSE;
//                 status = STATUS_SUCCESS;
//                 retry = FALSE;
//             }
//         }
//     } while(retry);
//
//     if ((zpoddInfo != NULL) &&
//         (zpoddInfo->MonitorStartStopUnit != FALSE) &&
//         (SRB_STATUS(Srb->SrbStatus) == SRB_STATUS_SUCCESS))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceSendSrbSynchronously: soft eject detected, device marked as active\n"));
//
//         DeviceMarkActive(deviceExtension, TRUE, FALSE);
//     }
//
//     // 5. Release all allocated resources.
//
//     // required even though we allocated our own, since the port driver may
//     // have allocated one also
//     if (PORT_ALLOCATED_SENSE(deviceExtension, Srb))
//     {
//         FREE_PORT_ALLOCATED_SENSE_BUFFER(deviceExtension, Srb);
//     }
//
// Exit:
//
//     if (senseInfoBuffer != NULL)
//     {
//         FREE_POOL(senseInfoBuffer);
//     }
//
//     Srb->SenseInfoBuffer = NULL;
//     Srb->SenseInfoBufferLength = 0;
//
//     if (mdlAddress)
//     {
//         if (memoryLocked)
//         {
//             MmUnlockPages(mdlAddress);
//             memoryLocked = FALSE;
//         }
//
//         IoFreeMdl(mdlAddress);
//         irp->MdlAddress = NULL;
//     }
//
//     if (request)
//     {
//         WdfObjectDelete(request as WDFOBJECT);
//     }
//
//     return status;
// }
//
//
// VOID
// DeviceSendNotification(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      const GUID*              Guid,
//      ULONG                    ExtraDataSize,
//      PVOID                ExtraData
//     )
// /*++
// Routine Description:
//
//     send notification to other components
//
// Arguments:
//
//     DeviceExtension - device context.
//
//     Guid - GUID for the notification
//
//     ExtraDataSize - data size along with notification
//
//     ExtraData - data buffer send with notification
//
// Return Value:
//     None
//
// --*/
// {
//     PTARGET_DEVICE_CUSTOM_NOTIFICATION  notification;
//     ULONG                               requiredSize;
//     NTSTATUS                            status;
//
//     status = RtlULongAdd((sizeof(TARGET_DEVICE_CUSTOM_NOTIFICATION) - sizeof(UCHAR)),
//                          ExtraDataSize,
//                          &requiredSize);
//
//     if (!(NT_SUCCESS(status)) || (requiredSize > 0x0000ffff))
//     {
//         // MAX_USHORT, max total size for these events!
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "Error sending event: size too large! (%x)\n",
//                    requiredSize));
//         return;
//     }
//
//     notification = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                          requiredSize,
//                                          CDROM_TAG_NOTIFICATION);
//
//     // if none allocated, exit
//     if (notification == NULL)
//     {
//         return;
//     }
//
//     // Prepare and send the request!
//     RtlZeroMemory(notification, requiredSize);
//     notification->Version = 1;
//     notification->Size = (USHORT)(requiredSize);
//     notification->FileObject = NULL;
//     notification->NameBufferOffset = -1;
//     notification->Event = *Guid;
//
//     if (ExtraData != NULL)
//     {
//         RtlCopyMemory(notification->CustomDataBuffer, ExtraData, ExtraDataSize);
//     }
//
//     IoReportTargetDeviceChangeAsynchronous(DeviceExtension->LowerPdo,
//                                            notification,
//                                            NULL,
//                                            NULL);
//
//     FREE_POOL(notification);
//
//     return;
// }
//
//
// VOID
// DeviceSendStartUnit(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//     Send command to SCSI unit to start or power up.
//     Because this command is issued asynchronounsly, that is, without
//     waiting on it to complete, the IMMEDIATE flag is not set. This
//     means that the CDB will not return until the drive has powered up.
//     This should keep subsequent requests from being submitted to the
//     device before it has completely spun up.
//
//     This routine is called from the InterpretSense routine, when a
//     request sense returns data indicating that a drive must be
//     powered up.
//
//     This routine may also be called from a class driver's error handler,
//     or anytime a non-critical start device should be sent to the device.
//
// Arguments:
//
//     Device - The device object.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = NULL;
//     WDF_OBJECT_ATTRIBUTES   attributes;
//     WDFREQUEST              startUnitRequest = NULL;
//     WDFMEMORY               inputMemory = NULL;
//
//     PCOMPLETION_CONTEXT     context = NULL;
//     PSCSI_REQUEST_BLOCK     srb = NULL;
//     PCDB                    cdb = NULL;
//
//     deviceExtension = DeviceGetExtension(Device);
//
//     if (NT_SUCCESS(status))
//     {
//         // Allocate Srb from nonpaged pool.
//         context = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                         sizeof(COMPLETION_CONTEXT),
//                                         CDROM_TAG_COMPLETION_CONTEXT);
//
//         if (context == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "DeviceSendStartUnit: Failed to allocate completion context\n"));
//
//             status = STATUS_INTERNAL_ERROR;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Save the device object in the context for use by the completion
//         // routine.
//         context->Device = Device;
//         srb = &context->Srb;
//
//         // Zero out srb.
//         RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         // setup SRB structure.
//         srb->Length = sizeof(SCSI_REQUEST_BLOCK);
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->TimeOutValue = START_UNIT_TIMEOUT;
//
//         srb->SrbFlags = SRB_FLAGS_NO_DATA_TRANSFER |
//                         SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
//
//         // setup CDB
//         srb->CdbLength = 6;
//         cdb = (PCDB)srb->Cdb;
//
//         cdb->START_STOP.OperationCode = SCSIOP_START_STOP_UNIT;
//         cdb->START_STOP.Start = 1;
//         cdb->START_STOP.Immediate = 0;
//         cdb->START_STOP.LogicalUnitNumber = srb->Lun;
//
//         //Create Request for sending down to port driver
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_REQUEST_CONTEXT);
//         attributes.ParentObject = deviceExtension->IoTarget;
//
//         status = WdfRequestCreate(&attributes,
//                                   deviceExtension->IoTarget,
//                                   &startUnitRequest);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         srb->OriginalRequest = WdfRequestWdmGetIrp(startUnitRequest);
//         NT_ASSERT(srb->OriginalRequest != NULL);
//
//         //Prepare the request
//         WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//         attributes.ParentObject = startUnitRequest as WDFOBJECT;
//
//         status = WdfMemoryCreatePreallocated(&attributes,
//                                              (PVOID)srb,
//                                              sizeof(SCSI_REQUEST_BLOCK),
//                                              &inputMemory);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(deviceExtension->IoTarget,
//                                                                 startUnitRequest,
//                                                                 IOCTL_SCSI_EXECUTE_NONE,
//                                                                 inputMemory,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Set a CompletionRoutine callback function.
//         WdfRequestSetCompletionRoutine(startUnitRequest,
//                                        DeviceAsynchronousCompletion,
//                                        context);
//
//         status = RequestSend(deviceExtension,
//                              startUnitRequest,
//                              deviceExtension->IoTarget,
//                              0,
//                              NULL);
//     }
//
//     // release resources when failed.
//     if (!NT_SUCCESS(status))
//     {
//         FREE_POOL(context);
//         if (startUnitRequest != NULL)
//         {
//             WdfObjectDelete(startUnitRequest as WDFOBJECT);
//         }
//     }
//
//     return;
// } // end StartUnit()
//
//
// VOID
// DeviceSendIoctlAsynchronously(
//      PCDROM_DEVICE_EXTENSION    DeviceExtension,
//      ULONG                      IoControlCode,
//      PDEVICE_OBJECT             TargetDeviceObject
//     )
// /*++
//
// Routine Description:
//
//     Send an IOCTL asynchronously
//
// Arguments:
//
//     DeviceExtension - device context.
//     IoControlCode - IOCTL code.
//     TargetDeviceObject - target device object.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     PIRP                irp = NULL;
//     PIO_STACK_LOCATION  nextIrpStack = NULL;
//
//     irp = IoAllocateIrp(DeviceExtension->DeviceObject->StackSize, FALSE);
//
//     if (irp != NULL)
//     {
//         nextIrpStack = IoGetNextIrpStackLocation(irp);
//
//         nextIrpStack->MajorFunction = IRP_MJ_DEVICE_CONTROL;
//
//         nextIrpStack->Parameters.DeviceIoControl.OutputBufferLength = 0;
//         nextIrpStack->Parameters.DeviceIoControl.InputBufferLength = 0;
//         nextIrpStack->Parameters.DeviceIoControl.IoControlCode = IoControlCode;
//         nextIrpStack->Parameters.DeviceIoControl.Type3InputBuffer = NULL;
//
//         IoSetCompletionRoutine(irp,
//                                RequestAsynchronousIrpCompletion,
//                                DeviceExtension,
//                                TRUE,
//                                TRUE,
//                                TRUE);
//
//         (VOID) IoCallDriver(TargetDeviceObject, irp);
//     }
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestAsynchronousIrpCompletion(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Context
//     )
// /*++
//
// Routine Description:
//
//     Free the Irp.
//
// Arguments:
//
//     DeviceObject - device that the completion routine fires on.
//
//     Irp - The irp to be completed.
//
//     Context - IRP context
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     UNREFERENCED_PARAMETER(DeviceObject);
//     UNREFERENCED_PARAMETER(Context);
//
//     IoFreeIrp(Irp);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceAsynchronousCompletion(
//      WDFREQUEST                       Request,
//      WDFIOTARGET                      Target,
//      PWDF_REQUEST_COMPLETION_PARAMS   Params,
//      WDFCONTEXT                       Context
//     )
// /*++
//
// Routine Description:
//
//     This routine is called when an asynchronous I/O request
//     which was issused by the class driver completes.  Examples of such requests
//     are release queue or START UNIT. This routine releases the queue if
//     necessary.  It then frees the context and the IRP.
//
// Arguments:
//
//     DeviceObject - The device object for the logical unit; however since this
//         is the top stack location the value is NULL.
//
//     Irp - Supplies a pointer to the Irp to be processed.
//
//     Context - Supplies the context to be used to process this request.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     PCOMPLETION_CONTEXT     context = (PCOMPLETION_CONTEXT)Context;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(context->Device);
//
//     UNREFERENCED_PARAMETER(Target);
//     UNREFERENCED_PARAMETER(Params);
//
//     // If this is an execute srb, then check the return status and make sure.
//     // the queue is not frozen.
//     if (context->Srb.Function == SRB_FUNCTION_EXECUTE_SCSI)
//     {
//         // Check for a frozen queue.
//         if (context->Srb.SrbStatus & SRB_STATUS_QUEUE_FROZEN)
//         {
//             // Unfreeze the queue getting the device object from the context.
//             DeviceReleaseQueue(context->Device);
//         }
//     }
//
//     // free port-allocated sense buffer if we can detect
//     //
//     if (PORT_ALLOCATED_SENSE(deviceExtension, &context->Srb))
//     {
//         FREE_PORT_ALLOCATED_SENSE_BUFFER(deviceExtension, &context->Srb);
//     }
//
//     FREE_POOL(context);
//
//     WdfObjectDelete(Request as WDFOBJECT);
//
// } // end DeviceAsynchronousCompletion()
//
//
// VOID
// DeviceReleaseQueue(
//      WDFDEVICE    Device
//     )
// /*++
//
// Routine Description:
//
//     This routine issues an internal device control command
//     to the port driver to release a frozen queue. The call
//     is issued asynchronously as DeviceReleaseQueue will be invoked
//     from the IO completion DPC (and will have no context to
//     wait for a synchronous call to complete).
//
//     This routine must be called with the remove lock held.
//
// Arguments:
//
//     Device - The functional device object for the device with the frozen queue.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//     PSCSI_REQUEST_BLOCK         srb = NULL;
//     KIRQL                       currentIrql;
//
//     // we raise irql seperately so we're not swapped out or suspended
//     // while holding the release queue irp in this routine.  this lets
//     // us release the spin lock before lowering irql.
//     KeRaiseIrql(DISPATCH_LEVEL, &currentIrql);
//
//     WdfSpinLockAcquire(deviceExtension->ReleaseQueueSpinLock);
//
//     if (deviceExtension->ReleaseQueueInProgress)
//     {
//         // Someone is already doing this work - just set the flag to indicate that
//         // we need to release the queue again.
//         deviceExtension->ReleaseQueueNeeded = TRUE;
//         WdfSpinLockRelease(deviceExtension->ReleaseQueueSpinLock);
//         KeLowerIrql(currentIrql);
//
//         return;
//     }
//
//     // Mark that there is a release queue in progress and drop the spinlock.
//     deviceExtension->ReleaseQueueInProgress = TRUE;
//
//     WdfSpinLockRelease(deviceExtension->ReleaseQueueSpinLock);
//
//     srb = &(deviceExtension->ReleaseQueueSrb);
//
//     // Optical media are removable, so we just flush the queue.  This will also release it.
//     srb->Function = SRB_FUNCTION_FLUSH_QUEUE;
//
//     srb->OriginalRequest = WdfRequestWdmGetIrp(deviceExtension->ReleaseQueueRequest);
//
//     // Set a CompletionRoutine callback function.
//     WdfRequestSetCompletionRoutine(deviceExtension->ReleaseQueueRequest,
//                                    DeviceReleaseQueueCompletion,
//                                    Device);
//     // Send the request. If an error occurs, complete the request.
//     RequestSend(deviceExtension,
//                 deviceExtension->ReleaseQueueRequest,
//                 deviceExtension->IoTarget,
//                 WDF_REQUEST_SEND_OPTION_IGNORE_TARGET_STATE,
//                 NULL);
//
//     KeLowerIrql(currentIrql);
//
//     return;
//
// } // end DeviceReleaseQueue()
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceReleaseQueueCompletion(
//      WDFREQUEST                       Request,
//      WDFIOTARGET                      Target,
//      PWDF_REQUEST_COMPLETION_PARAMS   Params,
//      WDFCONTEXT                       Context
//     )
// /*++
//
// Routine Description:
//
//     This routine is called when an asynchronous release queue request which
//     was issused in DeviceReleaseQueue completes. This routine prepares for
//     the next release queue request and resends it if necessary.
//
// Arguments:
//
//     Request - The completed request.
//
//     Target - IoTarget object
//
//     Params - Completion parameters
//
//     Context - WDFDEVICE object handle.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     NTSTATUS                    status;
//     WDFDEVICE                   device = Context;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(device);
//
//     BOOLEAN                     releaseQueueNeeded = FALSE;
//     WDF_REQUEST_REUSE_PARAMS    params = {0};
//
//     UNREFERENCED_PARAMETER(Target);
//     UNREFERENCED_PARAMETER(Params);
//
//     WDF_REQUEST_REUSE_PARAMS_INIT(&params,
//                                   WDF_REQUEST_REUSE_NO_FLAGS,
//                                   STATUS_SUCCESS);
//
//     // Grab the spinlock and clear the release queue in progress flag so others
//     // can run. Save (and clear) the state of the release queue needed flag
//     // so that we can issue a new release queue outside the spinlock.
//     WdfSpinLockAcquire(deviceExtension->ReleaseQueueSpinLock);
//
//     releaseQueueNeeded = deviceExtension->ReleaseQueueNeeded;
//
//     deviceExtension->ReleaseQueueNeeded = FALSE;
//     deviceExtension->ReleaseQueueInProgress = FALSE;
//
//     // Reuse the ReleaseQueueRequest for the next time.
//     status = WdfRequestReuse(Request,&params);
//
//     if (NT_SUCCESS(status))
//     {
//         // Preformat the ReleaseQueueRequest for the next time.
//         // This should always succeed because it was already preformatted once during device initialization
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(deviceExtension->IoTarget,
//                                                                 Request,
//                                                                 IOCTL_SCSI_EXECUTE_NONE,
//                                                                 deviceExtension->ReleaseQueueInputMemory,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL);
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                    "DeviceReleaseQueueCompletion: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n",
//                    status));
//     }
//
//     RequestClearSendTime(Request);
//
//     WdfSpinLockRelease(deviceExtension->ReleaseQueueSpinLock);
//
//     // If we need a release queue then issue one now.  Another processor may
//     // have already started one in which case we'll try to issue this one after
//     // it is done - but we should never recurse more than one deep.
//     if (releaseQueueNeeded)
//     {
//         DeviceReleaseQueue(device);
//     }
//
//     return;
//
// } // DeviceReleaseQueueCompletion()
//
//
// //
// // In order to provide better performance without the need to reboot,
// // we need to implement a self-adjusting method to set and clear the
// // srb flags based upon current performance.
// //
// // whenever there is an error, immediately grab the spin lock.  the
// // MP perf hit here is acceptable, since we're in an error path.  this
// // is also neccessary because we are guaranteed to be modifying the
// // SRB flags here, setting SuccessfulIO to zero, and incrementing the
// // actual error count (which is always done within this spinlock).
// //
// // whenever there is no error, increment a counter.  if there have been
// // errors on the device, and we've enabled dynamic perf, *and* we've
// // just crossed the perf threshhold, then grab the spin lock and
// // double check that the threshhold has, indeed been hit(*). then
// // decrement the error count, and if it's dropped sufficiently, undo
// // some of the safety changes made in the SRB flags due to the errors.
// //
// // * this works in all cases.  even if lots of ios occur after the
// //   previous guy went in and cleared the successfulio counter, that
// //   just means that we've hit the threshhold again, and so it's proper
// //   to run the inner loop again.
// //
//
// VOID
// DevicePerfIncrementErrorCount(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// {
//     PCDROM_PRIVATE_FDO_DATA fdoData = DeviceExtension->PrivateFdoData;
//     KIRQL                   oldIrql;
//     ULONG                   errors;
//
//     KeAcquireSpinLock(&fdoData->SpinLock, &oldIrql);
//
//     fdoData->Perf.SuccessfulIO = 0; // implicit interlock
//     errors = InterlockedIncrement((PLONG)&DeviceExtension->ErrorCount);
//
//     if (errors >= CLASS_ERROR_LEVEL_1)
//     {
//         // If the error count has exceeded the error limit, then disable
//         // any tagged queuing, multiple requests per lu queueing
//         // and sychronous data transfers.
//         //
//         // Clearing the no queue freeze flag prevents the port driver
//         // from sending multiple requests per logical unit.
//         CLEAR_FLAG(DeviceExtension->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//         CLEAR_FLAG(DeviceExtension->SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//
//         SET_FLAG(DeviceExtension->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                    "PerfIncrementErrorCount: Too many errors; disabling tagged queuing and "
//                    "synchronous data tranfers.\n"));
//     }
//
//     if (errors >= CLASS_ERROR_LEVEL_2)
//     {
//         // If a second threshold is reached, disable disconnects.
//         SET_FLAG(DeviceExtension->SrbFlags, SRB_FLAGS_DISABLE_DISCONNECT);
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                    "PerfIncrementErrorCount: Too many errors; disabling disconnects.\n"));
//     }
//
//     KeReleaseSpinLock(&fdoData->SpinLock, oldIrql);
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// PVOID
// DeviceFindFeaturePage(
//      PGET_CONFIGURATION_HEADER   FeatureBuffer,
//      ULONG const                                Length,
//      FEATURE_NUMBER const                       Feature
//     )
// /*++
// Routine Description:
//
//     find the specific feature page in the buffer
//
// Arguments:
//
//     FeatureBuffer - buffer contains the device feature set.
//
//     Length - buffer length
//
//     Feature - the feature number looking for.
//
// Return Value:
//
//     PVOID - pointer to the starting location of the specific feature in buffer.
//
// --*/
// {
//     PUCHAR buffer;
//     PUCHAR limit;
//     ULONG  validLength;
//
//     PAGED_CODE();
//
//     if (Length < sizeof(GET_CONFIGURATION_HEADER) + sizeof(FEATURE_HEADER))
//     {
//         return NULL;
//     }
//
//     // Calculate the length of valid data available in the
//     // capabilities buffer from the DataLength field
//     REVERSE_BYTES(&validLength, FeatureBuffer->DataLength);
//
//     validLength += RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//     // set limit to point to first illegal address
//     limit  = (PUCHAR)FeatureBuffer;
//     limit += min(Length, validLength);
//
//     // set buffer to point to first page
//     buffer = FeatureBuffer->Data;
//
//     // loop through each page until we find the requested one, or
//     // until it's not safe to access the entire feature header
//     // (if equal, have exactly enough for the feature header)
//     while (buffer + sizeof(FEATURE_HEADER) <= limit)
//     {
//         PFEATURE_HEADER header = (PFEATURE_HEADER)buffer;
//         FEATURE_NUMBER  thisFeature;
//
//         thisFeature  = (header->FeatureCode[0] << 8) |
//                        (header->FeatureCode[1]);
//
//         if (thisFeature == Feature)
//         {
//             PUCHAR temp;
//
//             // if don't have enough memory to safely access all the feature
//             // information, return NULL
//             temp = buffer;
//             temp += sizeof(FEATURE_HEADER);
//             temp += header->AdditionalLength;
//
//             if (temp > limit)
//             {
//                 // this means the transfer was cut-off, an insufficiently
//                 // small buffer was given, or other arbitrary error.  since
//                 // it's not safe to view the amount of data (even though
//                 // the header is safe) in this feature, pretend it wasn't
//                 // transferred at all...
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL,
//                            "Feature %x exists, but not safe to access all its data.  returning NULL\n",
//                            Feature));
//                 return NULL;
//             }
//             else
//             {
//                 return buffer;
//             }
//         }
//
//         if ((header->AdditionalLength % 4) &&
//             !(Feature >= 0xff00 && Feature <= 0xffff))
//         {
//             return NULL;
//         }
//
//         buffer += sizeof(FEATURE_HEADER);
//         buffer += header->AdditionalLength;
//     }
//
//     return NULL;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DevicePrintAllFeaturePages(
//      PGET_CONFIGURATION_HEADER   Buffer,
//      ULONG const                                Usable
//     )
// /*++
// Routine Description:
//
//     print out all feature pages in the buffer
//
// Arguments:
//
//     Buffer - buffer contains the device feature set.
//
//     Usable -
//
// Return Value:
//
//     none
//
// --*/
// {
// #if DBG
//     PFEATURE_HEADER header;
//
//     PAGED_CODE();
//
//     ////////////////////////////////////////////////////////////////////////////////
//     // items expected to ALWAYS be current if they exist
//     ////////////////////////////////////////////////////////////////////////////////
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureProfileList);
//     if (header != NULL) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: CurrentProfile %x "
//                    "with %x bytes of data at %p\n",
//                    Buffer->CurrentProfile[0] << 8 |
//                    Buffer->CurrentProfile[1],
//                    Usable, Buffer));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureCore);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "CORE Features"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureMorphing);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Morphing"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureRemovableMedium);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Removable Medium"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeaturePowerManagement);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Power Management"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureEmbeddedChanger);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Embedded Changer"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureMicrocodeUpgrade);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Microcode Update"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureTimeout);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Timeouts"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureLogicalUnitSerialNumber);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "LUN Serial Number"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureFirmwareDate);
//     if (header) {
//
//         ULONG featureSize = header->AdditionalLength;
//         featureSize += RTL_SIZEOF_THROUGH_FIELD(FEATURE_HEADER, AdditionalLength);
//
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                    "Currently supports" : "Is able to support"),
//                    "Firmware Date"
//                    ));
//
//         if (featureSize >= RTL_SIZEOF_THROUGH_FIELD(FEATURE_DATA_FIRMWARE_DATE, Minute))
//         {
//             PFEATURE_DATA_FIRMWARE_DATE date = (PFEATURE_DATA_FIRMWARE_DATE)header;
//             // show date as "YYYY/MM/DD hh:mm", which is 18 chars (17+NULL)
//             UCHAR dateString[18] = { 0 };
//             dateString[ 0] = date->Year[0];
//             dateString[ 1] = date->Year[1];
//             dateString[ 2] = date->Year[2];
//             dateString[ 3] = date->Year[3];
//             dateString[ 4] = '/';
//             dateString[ 5] = date->Month[0];
//             dateString[ 6] = date->Month[1];
//             dateString[ 7] = '/';
//             dateString[ 8] = date->Day[0];
//             dateString[ 9] = date->Day[1];
//             dateString[10] = ' ';
//             dateString[11] = ' ';
//             dateString[12] = date->Hour[0];
//             dateString[13] = date->Hour[1];
//             dateString[14] = ':';
//             dateString[15] = date->Minute[0];
//             dateString[16] = date->Minute[1];
//             dateString[17] = 0;
//             // SECONDS IS NOT AVAILABLE ON EARLY IMPLEMENTATIONS -- ignore it
//             //dateString[17] = ':';
//             //dateString[18] = date->Seconds[0];
//             //dateString[19] = date->Seconds[1];
//             //dateString[20] = 0;
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                         "CdromGetConfiguration: Firmware Date/Time %s (UTC)\n",
//                         (PCSTR)dateString
//                         ));
//         }
//     }
//
// ////////////////////////////////////////////////////////////////////////////////
// // items expected not to always be current
// ////////////////////////////////////////////////////////////////////////////////
//
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureWriteProtect);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Software Write Protect"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureRandomReadable);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Random Reads"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureMultiRead);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Multi-Read"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdRead);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "reading from CD-ROM/R/RW"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdRead);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DVD Structure Reads"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureRandomWritable);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Random Writes"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureIncrementalStreamingWritable);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Incremental Streaming Writing"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureSectorErasable);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Sector Erasable Media"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureFormattable);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Formatting"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDefectManagement);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "defect management"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureWriteOnce);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Write Once Media"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureRestrictedOverwrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Restricted Overwrites"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdrwCAVWrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "CD-RW CAV recording"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureMrw);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Mount Rainier media"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureEnhancedDefectReporting);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Enhanced Defect Reporting"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdPlusRW);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DVD+RW media"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureRigidRestrictedOverwrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Rigid Restricted Overwrite"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdTrackAtOnce);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "CD Recording (Track At Once)"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureCdMastering);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "CD Recording (Mastering)"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdRecordableWrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DVD Recording (Mastering)"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDDCDRead);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DD CD Reading"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDDCDRWrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DD CD-R Writing"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDDCDRWWrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DD CD-RW Writing"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureLayerJumpRecording);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Layer Jump Recording"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureHDDVDRead);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "HD-DVD Reading"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureHDDVDWrite);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "HD-DVD Writing"
//                    ));
//     }
//
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureSMART);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "S.M.A.R.T."
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureCDAudioAnalogPlay);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Analogue CD Audio Operations"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdCSS);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DVD CSS"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureRealTimeStreaming);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "Real-time Streaming Reads"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDiscControlBlocks);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DVD Disc Control Blocks"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureDvdCPRM);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "DVD CPRM"
//                    ));
//     }
//
//     header = DeviceFindFeaturePage(Buffer, Usable, FeatureAACS);
//     if (header) {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_INIT,
//                    "CdromGetConfiguration: %s %s\n",
//                    (header->Current ?
//                     "Currently supports" : "Is able to support"),
//                    "AACS"
//                    ));
//     }
//
// #else
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER(Usable);
//     UNREFERENCED_PARAMETER(Buffer);
//
// #endif // DBG
//     return;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// MediaReadCapacity(
//      WDFDEVICE Device
//     )
// /*++
// Routine Description:
//
//     Get media capacity
//
// Arguments:
//
//     Device - the device that owns the media
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status;
//     SCSI_REQUEST_BLOCK      srb;
//     PCDB                    cdb = NULL;
//     READ_CAPACITY_DATA      capacityData;
//
//     PAGED_CODE();
//
//     RtlZeroMemory(&srb, sizeof(srb));
//     RtlZeroMemory(&capacityData, sizeof(capacityData));
//
//     cdb = (PCDB)(&srb.Cdb);
//
//     //Prepare SCSI command fields
//     srb.CdbLength = 10;
//     srb.TimeOutValue = CDROM_READ_CAPACITY_TIMEOUT;
//     cdb->CDB10.OperationCode = SCSIOP_READ_CAPACITY;
//
//     status = DeviceSendSrbSynchronously(Device,
//                                         &srb,
//                                         &capacityData,
//                                         sizeof(READ_CAPACITY_DATA),
//                                         FALSE,
//                                         NULL);
//
//     //Remember the result
//     if (!NT_SUCCESS(status))
//     {
//         //Set the BytesPerBlock to zero, this is for safe as if error happens this field should stay zero (no change).
//         //it will be treated as error case in MediaReadCapacityDataInterpret()
//         capacityData.BytesPerBlock = 0;
//     }
//
//     MediaReadCapacityDataInterpret(Device, &capacityData);
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// MediaReadCapacityDataInterpret(
//      WDFDEVICE            Device,
//      PREAD_CAPACITY_DATA  ReadCapacityBuffer
//     )
// /*++
// Routine Description:
//
//     Interpret media capacity and set corresponding fields in device context
//
// Arguments:
//
//     Device - the device that owns the media
//
//     ReadCapacityBuffer - data buffer of capacity
//
// Return Value:
//
//     none
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//     ULONG                   lastSector = 0;
//     ULONG                   bps = 0;
//     ULONG                   lastBit = 0;
//     ULONG                   bytesPerBlock = 0;
//     BOOLEAN                 errorHappened = FALSE;
//
//     PAGED_CODE();
//
//     NT_ASSERT(ReadCapacityBuffer);
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL,
//                 "MediaReadCapacityDataInterpret: Entering\n"));
//
//     // Swizzle bytes from Read Capacity and translate into
//     // the necessary geometry information in the device extension.
//     bytesPerBlock = ReadCapacityBuffer->BytesPerBlock;
//
//     ((PFOUR_BYTE)&bps)->Byte0 = ((PFOUR_BYTE)&bytesPerBlock)->Byte3;
//     ((PFOUR_BYTE)&bps)->Byte1 = ((PFOUR_BYTE)&bytesPerBlock)->Byte2;
//     ((PFOUR_BYTE)&bps)->Byte2 = ((PFOUR_BYTE)&bytesPerBlock)->Byte1;
//     ((PFOUR_BYTE)&bps)->Byte3 = ((PFOUR_BYTE)&bytesPerBlock)->Byte0;
//
//     // Insure that bps is a power of 2.
//     // This corrects a problem with the HP 4020i CDR where it
//     // returns an incorrect number for bytes per sector.
//     if (!bps)
//     {
//         // Set disk geometry to default values (per ISO 9660).
//         bps = 2048;
//         errorHappened = TRUE;
//     }
//     else
//     {
//         lastBit = (ULONG)(-1);
//         while (bps)
//         {
//             lastBit++;
//             bps = (bps >> 1);
//         }
//         bps = (1 << lastBit);
//     }
//
//     deviceExtension->DiskGeometry.BytesPerSector = bps;
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL,
//                "MediaReadCapacityDataInterpret: Calculated bps %#x\n",
//                 deviceExtension->DiskGeometry.BytesPerSector));
//
//     // Copy last sector in reverse byte order.
//     bytesPerBlock = ReadCapacityBuffer->LogicalBlockAddress;
//
//     ((PFOUR_BYTE)&lastSector)->Byte0 = ((PFOUR_BYTE)&bytesPerBlock)->Byte3;
//     ((PFOUR_BYTE)&lastSector)->Byte1 = ((PFOUR_BYTE)&bytesPerBlock)->Byte2;
//     ((PFOUR_BYTE)&lastSector)->Byte2 = ((PFOUR_BYTE)&bytesPerBlock)->Byte1;
//     ((PFOUR_BYTE)&lastSector)->Byte3 = ((PFOUR_BYTE)&bytesPerBlock)->Byte0;
//
//     // Calculate sector to byte shift.
//     WHICH_BIT(bps, deviceExtension->SectorShift);
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL,
//                "MediaReadCapacityDataInterpret: Sector size is %d\n",
//                deviceExtension->DiskGeometry.BytesPerSector));
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL,
//                "MediaReadCapacityDataInterpret: Number of Sectors is %d\n",
//                lastSector + 1));
//
//     // Calculate media capacity in bytes.
//     if (errorHappened)
//     {
//         // Set disk geometry to default values (per ISO 9660).
//         deviceExtension->PartitionLength.QuadPart = (LONGLONG)(0x7fffffff);
//     }
//     else
//     {
//         deviceExtension->PartitionLength.QuadPart = (LONGLONG)(lastSector + 1);
//         deviceExtension->PartitionLength.QuadPart =
//                     (deviceExtension->PartitionLength.QuadPart << deviceExtension->SectorShift);
//     }
//
//     // we've defaulted to 32/64 forever.  don't want to change this now...
//     deviceExtension->DiskGeometry.TracksPerCylinder = 0x40;
//     deviceExtension->DiskGeometry.SectorsPerTrack = 0x20;
//
//     // Calculate number of cylinders.
//     deviceExtension->DiskGeometry.Cylinders.QuadPart = (LONGLONG)((lastSector + 1) / (32 * 64));
//
//     deviceExtension->DiskGeometry.MediaType = RemovableMedia;
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DevicePickDvdRegion(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//     pick a default dvd region
//
// Arguments:
//
//     Device - Device Object
//
// Return Value:
//
//     none
//
// --*/
// {
//     NTSTATUS                status;
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(Device);
//
//     // these five pointers all point to dvdReadStructure or part of
//     // its data, so don't deallocate them more than once!
//     PDVD_READ_STRUCTURE         dvdReadStructure;
//     PDVD_COPY_PROTECT_KEY       copyProtectKey;
//     PDVD_COPYRIGHT_DESCRIPTOR   dvdCopyRight;
//     PDVD_RPC_KEY                rpcKey;
//     PDVD_SET_RPC_KEY            dvdRpcKey;
//
//     size_t          bytesReturned = 0;
//     ULONG           bufferLen = 0;
//     UCHAR           mediaRegion = 0;
//     ULONG           pickDvdRegion = 0;
//     ULONG           defaultDvdRegion = 0;
//     ULONG           dvdRegion = 0;
//     WDFKEY          registryKey = NULL;
//
//     DECLARE_CONST_UNICODE_STRING(registryValueName, DVD_DEFAULT_REGION);
//
//     PAGED_CODE();
//
//     if ((pickDvdRegion = InterlockedExchange((PLONG)&deviceExtension->DeviceAdditionalData.PickDvdRegion, 0)) == 0)
//     {
//         // it was non-zero, so either another thread will do this, or
//         // we no longer need to pick a region
//         return;
//     }
//
//     bufferLen = max(
//                     max(sizeof(DVD_DESCRIPTOR_HEADER) +
//                             sizeof(DVD_COPYRIGHT_DESCRIPTOR),
//                         sizeof(DVD_READ_STRUCTURE)
//                         ),
//                     max(DVD_RPC_KEY_LENGTH,
//                         DVD_SET_RPC_KEY_LENGTH
//                         )
//                     );
//
//     dvdReadStructure = (PDVD_READ_STRUCTURE)
//                         ExAllocatePoolWithTag(PagedPool, bufferLen, DVD_TAG_DVD_REGION);
//
//     if (dvdReadStructure == NULL)
//     {
//         InterlockedExchange((PLONG)&deviceExtension->DeviceAdditionalData.PickDvdRegion, pickDvdRegion);
//         return;
//     }
//
//     copyProtectKey = (PDVD_COPY_PROTECT_KEY)dvdReadStructure;
//
//     dvdCopyRight = (PDVD_COPYRIGHT_DESCRIPTOR)
//                         ((PDVD_DESCRIPTOR_HEADER)dvdReadStructure)->Data;
//
//     // get the media region
//     RtlZeroMemory (dvdReadStructure, bufferLen);
//     dvdReadStructure->Format = DvdCopyrightDescriptor;
//
//     // Build and send a request for READ_KEY
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "DevicePickDvdRegion (%p): Getting Copyright Descriptor\n",
//                 Device));
//
//     status = ReadDvdStructure(deviceExtension,
//                               NULL,
//                               dvdReadStructure,
//                               sizeof(DVD_READ_STRUCTURE),
//                               dvdReadStructure,
//                               sizeof(DVD_DESCRIPTOR_HEADER) + sizeof(DVD_COPYRIGHT_DESCRIPTOR),
//                               &bytesReturned);
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "DevicePickDvdRegion (%p): Got Copyright Descriptor %x\n",
//                 Device, status));
//
//     if ((NT_SUCCESS(status)) &&
//         (dvdCopyRight->CopyrightProtectionType == 0x01))
//     {
//         // keep the media region bitmap around
//         // a 1 means ok to play
//         if (dvdCopyRight->RegionManagementInformation == 0xff)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                       "DevicePickDvdRegion (%p): RegionManagementInformation "
//                       "is set to dis-allow playback for all regions.  This is "
//                       "most likely a poorly authored disc.  defaulting to all "
//                       "region disc for purpose of choosing initial region\n",
//                       Device));
//             dvdCopyRight->RegionManagementInformation = 0;
//         }
//
//         mediaRegion = ~dvdCopyRight->RegionManagementInformation;
//     }
//     else
//     {
//         // can't automatically pick a default region on a drive without media, so just exit
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): failed to auto-choose a region due to status %x getting copyright descriptor\n",
//                     Device, status));
//         goto getout;
//     }
//
//     // get the device region
//     RtlZeroMemory (copyProtectKey, bufferLen);
//     copyProtectKey->KeyLength = DVD_RPC_KEY_LENGTH;
//     copyProtectKey->KeyType = DvdGetRpcKey;
//
//     // Build and send a request for READ_KEY for RPC key
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "DevicePickDvdRegion (%p): Getting RpcKey\n",
//                 Device));
//     status = DvdStartSessionReadKey(deviceExtension,
//                                     IOCTL_DVD_READ_KEY,
//                                     NULL,
//                                     copyProtectKey,
//                                     DVD_RPC_KEY_LENGTH,
//                                     copyProtectKey,
//                                     DVD_RPC_KEY_LENGTH,
//                                     &bytesReturned);
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "DevicePickDvdRegion (%p): Got RpcKey %x\n",
//                 Device, status));
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): failed to get RpcKey from "
//                     "a DVD Device\n", Device));
//         goto getout;
//     }
//
//     // so we now have what we can get for the media region and the
//     // drive region.  we will not set a region if the drive has one
//     // set already (mask is not all 1's), nor will we set a region
//     // if there are no more user resets available.
//     rpcKey = (PDVD_RPC_KEY)copyProtectKey->KeyData;
//
//     if (rpcKey->RegionMask != 0xff)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): not picking a region since "
//                     "it is already chosen\n", Device));
//         goto getout;
//     }
//
//     if (rpcKey->UserResetsAvailable <= 1)
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): not picking a region since "
//                     "only one change remains\n", Device));
//         goto getout;
//     }
//
//     // OOBE sets this key based upon the system locale
//     status = WdfDriverOpenParametersRegistryKey(WdfGetDriver(),
//                                                 KEY_READ,
//                                                 WDF_NO_OBJECT_ATTRIBUTES,
//                                                 &registryKey);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRegistryQueryULong(registryKey,
//                                        &registryValueName,
//                                        &defaultDvdRegion);
//
//         WdfRegistryClose(registryKey);
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): failed to read registry value due to status %x\n",
//                     Device, status));
//
//         // by default the default Dvd region is 0
//         defaultDvdRegion = 0;
//         status = STATUS_SUCCESS;
//     }
//
//     if (defaultDvdRegion > DVD_MAX_REGION)
//     {
//         // the registry has a bogus default
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): registry has a bogus default "
//                     "region value of %x\n", Device, defaultDvdRegion));
//
//         defaultDvdRegion = 0;
//     }
//
//     // if defaultDvdRegion == 0, it means no default.
//
//     // we will select the initial dvd region for the user
//
//     if ((defaultDvdRegion != 0) &&
//         (mediaRegion & (1 << (defaultDvdRegion - 1))))
//     {
//         // first choice:
//         // the media has region that matches
//         // the default dvd region.
//         dvdRegion = (1 << (defaultDvdRegion - 1));
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): Choice #1: media matches "
//                     "drive's default, chose region %x\n", Device, dvdRegion));
//     }
//     else if (mediaRegion)
//     {
//         // second choice:
//         // pick the lowest region number from the media
//         UCHAR mask = 1;
//         dvdRegion = 0;
//
//         while (mediaRegion && !dvdRegion)
//         {
//             // pick the lowest bit
//             dvdRegion = mediaRegion & mask;
//             mask <<= 1;
//         }
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): Choice #2: choosing lowest "
//                     "media region %x\n", Device, dvdRegion));
//     }
//     else if (defaultDvdRegion)
//     {
//         // third choice:
//         // default dvd region from the dvd class installer
//         dvdRegion = (1 << (defaultDvdRegion - 1));
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): Choice #3: using default "
//                     "region for this install %x\n", Device, dvdRegion));
//     }
//     else
//     {
//         // unable to pick one for the user -- this should rarely
//         // happen, since the proppage dvd class installer sets
//         // the key based upon the system locale
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                     "DevicePickDvdRegion (%p): Choice #4: failed to choose "
//                     "a media region\n", Device));
//         goto getout;
//     }
//
//     // now that we've chosen a region, set the region by sending the
//     // appropriate request to the drive
//     RtlZeroMemory (copyProtectKey, bufferLen);
//     copyProtectKey->KeyLength = DVD_SET_RPC_KEY_LENGTH;
//     copyProtectKey->KeyType = DvdSetRpcKey;
//     dvdRpcKey = (PDVD_SET_RPC_KEY)copyProtectKey->KeyData;
//     dvdRpcKey->PreferredDriveRegionCode = (UCHAR)~dvdRegion;
//
//     // Build and send request for SEND_KEY
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "DevicePickDvdRegion (%p): Sending new Rpc Key to region %x\n",
//                 Device, dvdRegion));
//
//     status = DvdSendKey(deviceExtension,
//                         NULL,
//                         copyProtectKey,
//                         DVD_SET_RPC_KEY_LENGTH,
//                         &bytesReturned);
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL,
//                 "DevicePickDvdRegion (%p): Sent new Rpc Key %x\n",
//                 Device, status));
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL, "DevicePickDvdRegion (%p): unable to set dvd initial "
//                      " region code (%x)\n", Device, status));
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_IOCTL, "DevicePickDvdRegion (%p): Successfully set dvd "
//                      "initial region\n", Device));
//
//         pickDvdRegion = 0;
//     }
//
// getout:
//     if (dvdReadStructure)
//     {
//         FREE_POOL(dvdReadStructure);
//     }
//
//     // update the new PickDvdRegion value
//     InterlockedExchange((PLONG)&deviceExtension->DeviceAdditionalData.PickDvdRegion, pickDvdRegion);
//
//     return;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceRegisterInterface(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      CDROM_DEVICE_INTERFACES InterfaceType
//     )
// /*++
// Routine Description:
//
//     used to register device class interface or mount device interface
//
// Arguments:
//
//     DeviceExtension - device context
//
//     InterfaceType - interface type to be registered.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status;
//     WDFSTRING       string = NULL;
//     GUID*           interfaceGuid = NULL;
//     PUNICODE_STRING savingString = NULL;
//     BOOLEAN         setRestricted = FALSE;
//     UNICODE_STRING  localString;
//
//     PAGED_CODE();
//
//     //Get parameters
//     switch(InterfaceType)
//     {
//     case CdRomDeviceInterface:
//         interfaceGuid = (LPGUID)&GUID_DEVINTERFACE_CDROM;
//         setRestricted = TRUE;
//         savingString = &localString;
//         break;
//     case MountedDeviceInterface:
//         interfaceGuid = (LPGUID)&MOUNTDEV_MOUNTED_DEVICE_GUID;
//         savingString = &(DeviceExtension->MountedDeviceInterfaceName);
//         break;
//     default:
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     status = WdfDeviceCreateDeviceInterface(DeviceExtension->Device,
//                                             interfaceGuid,
//                                             NULL);
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                     "DeviceRegisterInterface: Unable to register cdrom "
//                     "DCA for fdo %p type: %s [%lx]\n",
//                     DeviceExtension->Device,
//                     (InterfaceType == CdRomDeviceInterface)? "CdRom Interface" : "Mounted Device Interface",
//                     status));
//     }
//
//     // Retrieve interface string
//     if (NT_SUCCESS(status))
//     {
//         // The string object will be released when its parent object is released.
//         WDF_OBJECT_ATTRIBUTES  attributes;
//
//         WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//         attributes.ParentObject = DeviceExtension->Device;
//
//         status = WdfStringCreate(WDF_NO_OBJECT_ATTRIBUTES,
//                                  NULL,
//                                  &string);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfDeviceRetrieveDeviceInterfaceString(DeviceExtension->Device,
//                                                         interfaceGuid,
//                                                         NULL,
//                                                         string);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         WdfStringGetUnicodeString(string, savingString);
//
//         if (setRestricted) {
//
//
//             WdfObjectDelete(string as WDFOBJECT);
//         }
//     }
//
//     return status;
// } // end DeviceRegisterInterface()
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceRestoreDefaultSpeed(
//      WDFWORKITEM  WorkItem
//     )
// /*++
//
// Routine Description:
//
//     This workitem is called on a media change when the CDROM device
//     speed should be restored to the default value.
//
// Arguments:
//
//     Fdo      - Supplies the device object for the CDROM device.
//     WorkItem - Supplies the pointer to the workitem.
//
// Return Value:
//
//     None
//
// --*/
// {
//     NTSTATUS                status;
//     WDFDEVICE               device = WdfWorkItemGetParentObject(WorkItem);
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//     PPERFORMANCE_DESCRIPTOR perfDescriptor;
//     ULONG                   transferLength = sizeof(PERFORMANCE_DESCRIPTOR);
//     SCSI_REQUEST_BLOCK      srb = {0};
//     PCDB                    cdb = (PCDB)srb.Cdb;
//
//     PAGED_CODE();
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DeviceRestoreDefaultSpeed: Restore device speed for %p\n", device));
//
//     perfDescriptor = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                            transferLength,
//                                            CDROM_TAG_STREAM);
//     if (perfDescriptor == NULL)
//     {
//         return;
//     }
//
//     RtlZeroMemory(perfDescriptor, transferLength);
//
//     perfDescriptor->RestoreDefaults = TRUE;
//
//     srb.TimeOutValue = deviceExtension->TimeOutValue;
//
//     srb.CdbLength = 12;
//     cdb->SET_STREAMING.OperationCode = SCSIOP_SET_STREAMING;
//     REVERSE_BYTES_SHORT(&cdb->SET_STREAMING.ParameterListLength, &transferLength);
//
//     status = DeviceSendSrbSynchronously(device,
//                                         &srb,
//                                         perfDescriptor,
//                                         transferLength,
//                                         TRUE,
//                                         NULL);
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                "DeviceRestoreDefaultSpeed: Set Streaming command completed with status: 0x%X\n", status));
//
//     FREE_POOL(perfDescriptor);
//     WdfObjectDelete(WorkItem as WDFOBJECT);
//
//     return;
// }
//
// // custom string match -- careful!
// _IRQL_requires_max_(APC_LEVEL)
// BOOLEAN
// StringsAreMatched(
//      PCHAR StringToMatch,
//          PCHAR TargetString
//     )
// /*++
//
// Routine Description:
//
//     compares if two strings are identical
//
// Arguments:
//
//     StringToMatch - source string.
//     TargetString - target string.
//
// Return Value:
//
//     BOOLEAN - TRUE (identical); FALSE (not match)
//
// --*/
// {
//     size_t length;
//
//     PAGED_CODE();
//
//     NT_ASSERT(TargetString);
//
//     // if no match requested, return TRUE
//     if (StringToMatch == NULL)
//     {
//         return TRUE;
//     }
//
//     // cache the string length for efficiency
//     length = strlen(StringToMatch);
//
//     // ZERO-length strings may only match zero-length strings
//     if (length == 0)
//     {
//         return (strlen(TargetString) == 0);
//     }
//
//     // strncmp returns zero if the strings match
//     return (strncmp(StringToMatch, TargetString, length) == 0);
// }
//
//
// NTSTATUS
// RequestSetContextFields(
//      WDFREQUEST    Request,
//      PSYNC_HANDLER Handler
//     )
// /*++
//
// Routine Description:
//
//     set the request object context fields
//
// Arguments:
//
//     Request - request object.
//     Handler - the function that finally handles this request.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_REQUEST_CONTEXT  requestContext = RequestGetContext(Request);
//     PKEVENT                 syncEvent = NULL;
//
//     syncEvent = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                       sizeof(KEVENT),
//                                       CDROM_TAG_SYNC_EVENT);
//
//     if (syncEvent == NULL)
//     {
//         // memory allocation failed.
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         // now, put the special synchronization information into the context
//         requestContext->SyncRequired = TRUE;
//         requestContext->SyncEvent = syncEvent;
//         requestContext->SyncCallback = Handler;
//
//         status = STATUS_SUCCESS;
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// RequestDuidGetDeviceIdProperty(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      WDFREQUEST              Request,
//      WDF_REQUEST_PARAMETERS  RequestParameters,
//      size_t *               DataLength
//     )
// /*++
//
// Routine Description:
//
//
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request object.
//     RequestParameters - request parameter
//     DataLength - transferred data length.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PSTORAGE_DEVICE_ID_DESCRIPTOR   deviceIdDescriptor = NULL;
//     PSTORAGE_DESCRIPTOR_HEADER      descHeader = NULL;
//     STORAGE_PROPERTY_ID             propertyId = StorageDeviceIdProperty;
//
//     *DataLength = 0;
//
//     // Get the VPD page 83h data.
//     status = DeviceRetrieveDescriptor(DeviceExtension->Device,
//                                       &propertyId,
//                                       (PSTORAGE_DESCRIPTOR_HEADER*)&deviceIdDescriptor);
//
//     if (NT_SUCCESS(status) && (deviceIdDescriptor == NULL))
//     {
//         status = STATUS_NOT_FOUND;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &descHeader,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         PSTORAGE_DEVICE_UNIQUE_IDENTIFIER   storageDuid = NULL;
//         ULONG                               offset = descHeader->Size;
//         PUCHAR                              dest = (PUCHAR)descHeader + offset;
//         size_t                              outputBufferSize;
//
//         outputBufferSize = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//
//         // Adjust required size and potential destination location.
//         status = RtlULongAdd(descHeader->Size, deviceIdDescriptor->Size, &descHeader->Size);
//
//         if (NT_SUCCESS(status) &&
//             (outputBufferSize < descHeader->Size))
//         {
//             // Output buffer is too small.  Return error and make sure
//             // the caller gets info about required buffer size.
//             *DataLength = descHeader->Size;
//             status = STATUS_BUFFER_OVERFLOW;
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             storageDuid = (PSTORAGE_DEVICE_UNIQUE_IDENTIFIER)descHeader;
//             storageDuid->StorageDeviceIdOffset = offset;
//
//             RtlCopyMemory(dest,
//                           deviceIdDescriptor,
//                           deviceIdDescriptor->Size);
//
//             *DataLength = storageDuid->Size;
//             status = STATUS_SUCCESS;
//         }
//
//         FREE_POOL(deviceIdDescriptor);
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestDuidGetDeviceProperty(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      WDFREQUEST              Request,
//      WDF_REQUEST_PARAMETERS  RequestParameters,
//      size_t *               DataLength
//     )
// /*++
//
// Routine Description:
//
//
//
// Arguments:
//
//     DeviceExtension - device context
//     Request - request object.
//     RequestParameters - request parameter
//     DataLength - transferred data length.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                          status = STATUS_SUCCESS;
//     PSTORAGE_DEVICE_DESCRIPTOR        deviceDescriptor = DeviceExtension->DeviceDescriptor;
//     PSTORAGE_DESCRIPTOR_HEADER        descHeader = NULL;
//     PSTORAGE_DEVICE_UNIQUE_IDENTIFIER storageDuid;
//     PUCHAR                            dest = NULL;
//
//     if (deviceDescriptor == NULL)
//     {
//         status = STATUS_NOT_FOUND;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRequestRetrieveOutputBuffer(Request,
//                                                 RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength,
//                                                 &descHeader,
//                                                 NULL);
//     }
//
//     if (NT_SUCCESS(status) &&
//         (deviceDescriptor->SerialNumberOffset == 0))
//     {
//         status = STATUS_NOT_FOUND;
//     }
//
//     // Use this info only if serial number is available.
//     if (NT_SUCCESS(status))
//     {
//         ULONG offset = descHeader->Size;
//         size_t outputBufferSize = RequestParameters.Parameters.DeviceIoControl.as_ref().OutputBufferLength;
//
//         // Adjust required size and potential destination location.
//         dest = (PUCHAR)descHeader + offset;
//
//         status = RtlULongAdd(descHeader->Size, deviceDescriptor->Size, &descHeader->Size);
//
//         if (NT_SUCCESS(status) &&
//             (outputBufferSize < descHeader->Size))
//         {
//             // Output buffer is too small.  Return error and make sure
//             // the caller get info about required buffer size.
//             *DataLength = descHeader->Size;
//             status = STATUS_BUFFER_OVERFLOW;
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             storageDuid = (PSTORAGE_DEVICE_UNIQUE_IDENTIFIER)descHeader;
//             storageDuid->StorageDeviceOffset = offset;
//
//             RtlCopyMemory(dest,
//                           deviceDescriptor,
//                           deviceDescriptor->Size);
//
//             *DataLength = storageDuid->Size;
//             status = STATUS_SUCCESS;
//         }
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// ULONG
// DeviceRetrieveModeSenseUsingScratch(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      PCHAR     ModeSenseBuffer,
//      ULONG                    Length,
//      UCHAR                    PageCode,
//      UCHAR                    PageControl
//     )
// /*++
//
// Routine Description:
//
//     retrieve mode sense informaiton of the device
//
// Arguments:
//
//     DeviceExtension - device context
//     ModeSenseBuffer - buffer to savee the mode sense info.
//     Length - buffer length
//     PageCode - .
//     PageControl -
//
// Return Value:
//
//     ULONG - transferred data length
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     ULONG                       transferSize = min(Length, DeviceExtension->ScratchContext.ScratchBufferSize);
//     CDB                         cdb;
//
//     PAGED_CODE();
//
//     ScratchBuffer_BeginUse(DeviceExtension);
//
//     RtlZeroMemory(&cdb, sizeof(CDB));
//     // Set up the CDB
//     cdb.MODE_SENSE.OperationCode = SCSIOP_MODE_SENSE;
//     cdb.MODE_SENSE.PageCode = PageCode;
//     cdb.MODE_SENSE.Pc = PageControl;
//     cdb.MODE_SENSE.AllocationLength = (UCHAR)transferSize;
//
//     status = ScratchBuffer_ExecuteCdb(DeviceExtension, NULL, transferSize, TRUE, &cdb, 6);
//
//     if (NT_SUCCESS(status))
//     {
//         transferSize = min(Length, DeviceExtension->ScratchContext.ScratchSrb->DataTransferLength);
//         RtlCopyMemory(ModeSenseBuffer,
//                       DeviceExtension->ScratchContext.ScratchBuffer,
//                       transferSize);
//     }
//
//     ScratchBuffer_EndUse(DeviceExtension);
//
//     return transferSize;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// PVOID
// ModeSenseFindSpecificPage(
//      PCHAR     ModeSenseBuffer,
//      size_t                   Length,
//      UCHAR                    PageMode,
//      BOOLEAN                  Use6BytesCdb
//     )
// /*++
//
// Routine Description:
//
//     This routine scans through the mode sense data and finds the requested
//     mode sense page code.
//
// Arguments:
//     ModeSenseBuffer - Supplies a pointer to the mode sense data.
//
//     Length - Indicates the length of valid data.
//
//     PageMode - Supplies the page mode to be searched for.
//
//     Use6BytesCdb - Indicates whether 6 or 10 byte mode sense was used.
//
// Return Value:
//
//     A pointer to the the requested mode page.  If the mode page was not found
//     then NULL is return.
//
// --*/
// {
//     PCHAR limit;
//     ULONG  parameterHeaderLength;
//     PVOID  result = NULL;
//
//     PAGED_CODE();
//
//     limit = ModeSenseBuffer + Length;
//     parameterHeaderLength = (Use6BytesCdb)
//                             ? sizeof(MODE_PARAMETER_HEADER)
//                             : sizeof(MODE_PARAMETER_HEADER10);
//
//     if (Length >= parameterHeaderLength)
//     {
//         PMODE_PARAMETER_HEADER10 modeParam10;
//         ULONG                    blockDescriptorLength;
//
//         //  Skip the mode select header and block descriptors.
//         if (Use6BytesCdb)
//         {
//             blockDescriptorLength = ((PMODE_PARAMETER_HEADER)ModeSenseBuffer)->BlockDescriptorLength;
//         }
//         else
//         {
//             modeParam10 = (PMODE_PARAMETER_HEADER10) ModeSenseBuffer;
//             blockDescriptorLength = modeParam10->BlockDescriptorLength[1];
//         }
//
//         ModeSenseBuffer += parameterHeaderLength + blockDescriptorLength;
//
//         // ModeSenseBuffer now points at pages.  Walk the pages looking for the
//         // requested page until the limit is reached.
//         while (ModeSenseBuffer +
//                RTL_SIZEOF_THROUGH_FIELD(MODE_DISCONNECT_PAGE, PageLength) < limit)
//         {
//             if (((PMODE_DISCONNECT_PAGE) ModeSenseBuffer)->PageCode == PageMode)
//             {
//                 // found the mode page.  make sure it's safe to touch it all
//                 // before returning the pointer to caller
//                 if (ModeSenseBuffer + ((PMODE_DISCONNECT_PAGE)ModeSenseBuffer)->PageLength > limit)
//                 {
//                     //  Return NULL since the page is not safe to access in full
//                     result = NULL;
//                 }
//                 else
//                 {
//                     result = ModeSenseBuffer;
//                 }
//                 break;
//             }
//
//             // Advance to the next page which is 4-byte-aligned offset after this page.
//             ModeSenseBuffer += ((PMODE_DISCONNECT_PAGE) ModeSenseBuffer)->PageLength +
//                                 RTL_SIZEOF_THROUGH_FIELD(MODE_DISCONNECT_PAGE, PageLength);
//         }
//     }
//
//     return result;
// } // end ModeSenseFindSpecificPage()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// PerformEjectionControl(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      WDFREQUEST               Request,
//      MEDIA_LOCK_TYPE          LockType,
//      BOOLEAN                  Lock
//     )
// /*++
//
// Routine Description:
//
//     ejection control process
//
// Arguments:
//
//     DeviceExtension - device extension
//     Request - WDF request to be used for communication with the device
//     LockType - the type of lock
//     Lock - if TRUE, lock the device; if FALSE, unlock it
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status;
//     PFILE_OBJECT_CONTEXT    fileObjectContext = NULL;
//     SCSI_REQUEST_BLOCK      srb;
//     PCDB                    cdb = NULL;
//
//     LONG                    newLockCount = 0;
//     LONG                    newProtectedLockCount = 0;
//     LONG                    newInternalLockCount = 0;
//     LONG                    newFileLockCount = 0;
//     BOOLEAN                 countChanged = FALSE;
//     BOOLEAN                 previouslyLocked = FALSE;
//     BOOLEAN                 nowLocked = FALSE;
//
//     PAGED_CODE();
//
//     // Prevent race conditions while working with lock counts
//     status = WdfWaitLockAcquire(DeviceExtension->EjectSynchronizationLock, NULL);
//     if (!NT_SUCCESS(status))
//     {
//         NT_ASSERT(FALSE);
//     }
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                 "PerformEjectionControl: "
//                 "Received request for %s lock type\n",
//                 LockTypeStrings[LockType]
//                 ));
//
//
//     // If this is a "secured" request, retrieve the file object context
//     if (LockType == SecureMediaLock)
//     {
//         WDFFILEOBJECT fileObject = NULL;
//
//         fileObject = WdfRequestGetFileObject(Request);
//
//         if (fileObject == NULL)
//         {
//             status = STATUS_INVALID_HANDLE;
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                             "FileObject does not match to the one in IRP_MJ_CREATE, KMDF returns NULL\n"));
//             goto Exit;
//         }
//
//         fileObjectContext = FileObjectGetContext(fileObject);
//         NT_ASSERT(fileObjectContext != NULL);
//     }
//
//     // Lock counts should never fall below 0
//     NT_ASSERT(DeviceExtension->LockCount >= 0);
//     NT_ASSERT(DeviceExtension->ProtectedLockCount >= 0);
//     NT_ASSERT(DeviceExtension->InternalLockCount >= 0);
//
//     // Get the current lock counts
//     newLockCount = DeviceExtension->LockCount;
//     newProtectedLockCount = DeviceExtension->ProtectedLockCount;
//     newInternalLockCount = DeviceExtension->InternalLockCount;
//     if (fileObjectContext)
//     {
//         // fileObjectContext->LockCount is ULONG and should always >= 0
//         newFileLockCount = fileObjectContext->LockCount;
//     }
//
//     // Determine which lock counts need to be changed and how
//     if (Lock && LockType == SimpleMediaLock)
//     {
//         newLockCount++;
//         countChanged = TRUE;
//     }
//     else if (Lock && LockType == SecureMediaLock)
//     {
//         newFileLockCount++;
//         newProtectedLockCount++;
//         countChanged = TRUE;
//     }
//     else if (Lock && LockType == InternalMediaLock)
//     {
//         newInternalLockCount++;
//         countChanged = TRUE;
//     }
//     else if (!Lock && LockType == SimpleMediaLock)
//     {
//         if (newLockCount != 0)
//         {
//             newLockCount--;
//             countChanged = TRUE;
//         }
//     }
//     else if (!Lock && LockType == SecureMediaLock)
//     {
//         if ( (newFileLockCount == 0) || (newProtectedLockCount == 0) )
//         {
//             status = STATUS_INVALID_DEVICE_STATE;
//             goto Exit;
//         }
//         newFileLockCount--;
//         newProtectedLockCount--;
//         countChanged = TRUE;
//     }
//     else if (!Lock && LockType == InternalMediaLock)
//     {
//         NT_ASSERT(newInternalLockCount != 0);
//         newInternalLockCount--;
//         countChanged = TRUE;
//     }
//
//     if ( (DeviceExtension->LockCount != 0) ||
//          (DeviceExtension->ProtectedLockCount != 0) ||
//          (DeviceExtension->InternalLockCount != 0) )
//     {
//         previouslyLocked = TRUE;
//     }
//     if ( (newLockCount != 0) ||
//          (newProtectedLockCount != 0) ||
//          (newInternalLockCount != 0) )
//     {
//         nowLocked = TRUE;
//     }
//
//     // Only send command down to device when necessary
//     if (previouslyLocked != nowLocked)
//     {
//         // Compose and send the PREVENT ALLOW MEDIA REMOVAL command.
//         RtlZeroMemory(&srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         srb.CdbLength = 6;
//         srb.TimeOutValue = DeviceExtension->TimeOutValue;
//
//         cdb = (PCDB)&srb.Cdb;
//         cdb->MEDIA_REMOVAL.OperationCode = SCSIOP_MEDIUM_REMOVAL;
//         cdb->MEDIA_REMOVAL.Prevent = Lock;
//
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             &srb,
//                                             NULL,
//                                             0,
//                                             FALSE,
//                                             Request);
//     }
//
// Exit:
//
//     // Store the updated lock counts on success
//     if (countChanged && NT_SUCCESS(status))
//     {
//         DeviceExtension->LockCount = newLockCount;
//         DeviceExtension->ProtectedLockCount = newProtectedLockCount;
//         DeviceExtension->InternalLockCount = newInternalLockCount;
//         if (fileObjectContext)
//         {
//             fileObjectContext->LockCount = newFileLockCount;
//         }
//     }
//
//     WdfWaitLockRelease(DeviceExtension->EjectSynchronizationLock);
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                 "PerformEjectionControl: %!STATUS!, "
//                 "Count Changed: %d, Command Sent: %d, "
//                 "Current Counts: Internal: %x  Secure: %x  Simple: %x\n",
//                 status,
//                 countChanged,
//                 previouslyLocked != nowLocked,
//                 DeviceExtension->InternalLockCount,
//                 DeviceExtension->ProtectedLockCount,
//                 DeviceExtension->LockCount
//                 ));
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceUnlockExclusive(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      WDFFILEOBJECT            FileObject,
//      BOOLEAN                  IgnorePreviousMediaChanges
//     )
// /*++
//
// Routine Description:
//
//     to unlock the exclusive lock
//
// Arguments:
//
//     DeviceExtension - device context
//     FileObject - file object that currently holds the lock
//     IgnorePreviousMediaChanges - if TRUE, ignore previously accumulated media changes
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PCDROM_DATA                     cdData = &DeviceExtension->DeviceAdditionalData;
//     PMEDIA_CHANGE_DETECTION_INFO    info = DeviceExtension->MediaChangeDetectionInfo;
//     BOOLEAN                         ANPending = 0;
//     LONG                            requestInUse = 0;
//
//     PAGED_CODE();
//
//     if (!EXCLUSIVE_MODE(cdData))
//     {
//         // Device is not locked for exclusive access.
//         // Can not process unlock request.
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                     "RequestHandleExclusiveAccessUnlockDevice: Device not locked for exclusive access, can't unlock device.\n"));
//         status = STATUS_INVALID_DEVICE_REQUEST;
//     }
//     else if (!EXCLUSIVE_OWNER(cdData, FileObject))
//     {
//         // Request not from the exclusive owner, can't unlock the device.
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                    "RequestHandleExclusiveAccessUnlockDevice: Unable to unlock device, invalid file object\n"));
//
//         status = STATUS_INVALID_HANDLE;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Unless we were explicitly requested not to do so, generate a media removal notification
//         // followed by a media arrival notification similar to volume lock/unlock file system events.
//         if (!IgnorePreviousMediaChanges)
//         {
//             MEDIA_CHANGE_DETECTION_STATE previousMediaState = MediaUnknown;
//
//             // Change the media state to "unavailable", which will cause a removal notification if the media
//             // was previously present. At the same time, store the previous state in previousMediaState.
//             DeviceSetMediaChangeStateEx(DeviceExtension, MediaUnavailable, &previousMediaState);
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                         "DeviceUnlockExclusive: Changing the media state to MediaUnavailable\n"));
//
//             // Restore the previous media state, which will cause a media arrival notification if the media
//             // was originally present.
//             DeviceSetMediaChangeStateEx(DeviceExtension, previousMediaState, NULL);
//         }
//
//         // Set DO_VERIFY_VOLUME so that the file system will remount on it.
//         if (IsVolumeMounted(DeviceExtension->DeviceObject))
//         {
//             SET_FLAG(DeviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME);
//         }
//
//         // Set MMC state to update required
//         cdData->Mmc.WriteAllowed = FALSE;
//         cdData->Mmc.UpdateState = CdromMmcUpdateRequired;
//
//         // Send unlock notification
//         DeviceSendNotification(DeviceExtension,
//                                &GUID_IO_CDROM_EXCLUSIVE_UNLOCK,
//                                0,
//                                NULL);
//
//         InterlockedExchangePointer((PVOID)&cdData->ExclusiveOwner,  NULL);
//
//         if ((info != NULL) && (info->AsynchronousNotificationSupported != FALSE))
//         {
//             ANPending = info->ANSignalPendingDueToExclusiveLock;
//             info->ANSignalPendingDueToExclusiveLock = FALSE;
//
//             if ((ANPending != FALSE) && (info->MediaChangeDetectionDisableCount == 0))
//             {
//                 // if the request is not in use, mark it as such.
//                 requestInUse = InterlockedCompareExchange((PLONG)&info->MediaChangeRequestInUse, 1, 0);
//
//                 if (requestInUse == 0)
//                 {
//                     // The last MCN finished. ok to issue the new one.
//                     RequestSetupMcnSyncIrp(DeviceExtension);
//
//                     // The irp will go into KMDF framework and a request will be created there to represent it.
//                     IoCallDriver(DeviceExtension->DeviceObject, info->MediaChangeSyncIrp);
//                 }
//             }
//         }
//     }
//
//     return status;
// }
//
//
// VOID
// RequestCompletion(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      WDFREQUEST              Request,
//      NTSTATUS                Status,
//      ULONG_PTR               Information
//     )
// {
// #ifdef DBG
//     ULONG                   ioctlCode = 0;
//     WDF_REQUEST_PARAMETERS  requestParameters;
//
//     // Get the Request parameters
//     WDF_REQUEST_PARAMETERS_INIT(&requestParameters);
//     WdfRequestGetParameters(Request, &requestParameters);
//
//     if (requestParameters.Type == WdfRequestTypeDeviceControl)
//     {
//         ioctlCode = requestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode;
//
//         if (requestParameters.Parameters.DeviceIoControl.as_ref().IoControlCode != IOCTL_MCN_SYNC_FAKE_IOCTL)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL,
//                        "Request complete - IOCTL - code: %X; Status: %X; Information: %X\n",
//                        ioctlCode,
//                        Status,
//                        (ULONG)Information));
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_GENERAL,
//                        "Request complete - IOCTL - code: %X; Status: %X; Information: %X\n",
//                        ioctlCode,
//                        Status,
//                        (ULONG)Information));
//         }
//     }
//     else if (requestParameters.Type == WdfRequestTypeRead)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL,
//                    "Request complete - READ - Starting Offset: %X; Length: %X; Transferred Length: %X; Status: %X\n",
//                    (ULONG)requestParameters.Parameters.Read.as_ref().DeviceOffset,
//                    (ULONG)requestParameters.Parameters.Read.as_ref().Length,
//                    (ULONG)Information,
//                    Status));
//     }
//     else if (requestParameters.Type == WdfRequestTypeWrite)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL,
//                    "Request complete - WRITE - Starting Offset: %X; Length: %X; Transferred Length: %X; Status: %X\n",
//                    (ULONG)requestParameters.Parameters.Write.as_ref().DeviceOffset,
//                    (ULONG)requestParameters.Parameters.Write.as_ref().Length,
//                    (ULONG)Information,
//                    Status));
//     }
// #endif
//
//     if (IoIsErrorUserInduced(Status))
//     {
//         PIRP irp = WdfRequestWdmGetIrp(Request);
//         if (irp->Tail.Overlay.Thread)
//         {
//             IoSetHardErrorOrVerifyDevice(irp, DeviceExtension->DeviceObject);
//         }
//     }
//
//     if (!NT_SUCCESS(Status) && DeviceExtension->SurpriseRemoved == TRUE)
//     {
//         // IMAPI expects ERROR_DEV_NOT_EXISTS if recorder has been surprised removed,
//         // or it will retry WRITE commands for up to 3 minutes
//         // CDROM behavior should be consistent for all requests, including SCSI pass-through
//         Status = STATUS_DEVICE_DOES_NOT_EXIST;
//     }
//
//     WdfRequestCompleteWithInformation(Request, Status, Information);
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestDummyCompletionRoutine(
//      WDFREQUEST                     Request,
//      WDFIOTARGET                    Target,
//      PWDF_REQUEST_COMPLETION_PARAMS Params,
//      WDFCONTEXT                     Context
//     )
// /*++
//
// Routine Description:
//
//     This is a dummy competion routine that simply calls WdfRequestComplete. We have to use
//     this dummy competion routine instead of WDF_REQUEST_SEND_OPTION_SEND_AND_FORGET, because
//     the latter causes the framework to not check if the I/O target is closed or not.
//
// Arguments:
//
//     Request - completed request
//     Target - the I/O target that completed the request
//     Params - request parameters
//     Context - not used
//
// Return Value:
//
//     none
//
// --*/
// {
//     UNREFERENCED_PARAMETER(Target);
//     UNREFERENCED_PARAMETER(Params);
//     UNREFERENCED_PARAMETER(Context);
//
//     WdfRequestCompleteWithInformation(Request,
//                                       WdfRequestGetStatus(Request),
//                                       WdfRequestGetInformation(Request));
// }
//
//
// _IRQL_requires_max_(DISPATCH_LEVEL)
// NTSTATUS
// DeviceSendPowerDownProcessRequest(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PFN_WDF_REQUEST_COMPLETION_ROUTINE CompletionRoutine,
//      PVOID Context
//     )
// /*++
//
// Routine Description:
//
//     This function is called during processing power down request.
//     It is used to send either SYNC CACHE command or STOP UNIT command.
//
//     Caller should set proper value in deviceExtension->PowerContext.PowerChangeState.PowerDown
//     to trigger the correct command be sent.
//
// Arguments:
//
//     DeviceExtension -
//
//     CompletionRoutine - Completion routine that needs to be set for the request
//
//     Context - Completion context associated with the completion routine
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status;
//     BOOLEAN                     requestSent = FALSE;
//
//     BOOLEAN                     shouldRetry = TRUE;
//     PCDB                        cdb = (PCDB)DeviceExtension->PowerContext.Srb.Cdb;
//     ULONG                       timeoutValue = DeviceExtension->TimeOutValue;
//     ULONG                       retryCount = 1;
//
//     // reset some fields.
//     DeviceExtension->PowerContext.RetryIntervalIn100ns = 0;
//     status = PowerContextReuseRequest(DeviceExtension);
//     RequestClearSendTime(DeviceExtension->PowerContext.PowerRequest);
//
//     if (!NT_SUCCESS(status))
//     {
//         return status;
//     }
//
//     // set proper timeout value and max retry count.
//     switch(DeviceExtension->PowerContext.PowerChangeState.PowerDown)
//     {
//     case PowerDownDeviceInitial:
//     case PowerDownDeviceQuiesced:
//     case PowerDownDeviceStopped:
//         break;
//
//     case PowerDownDeviceLocked:
//         // Case of issuing SYNC CACHE command. Do not use power irp timeout remaining time in this case
//         // as we want to give best try on SYNC CACHE command.
//         retryCount = MAXIMUM_RETRIES;
//         timeoutValue = DeviceExtension->TimeOutValue;
//         break;
//
//     case PowerDownDeviceFlushed:
//     {
//         // Case of issuing STOP UNIT command
//         // As "Imme" bit is set to '1', this command should be completed in short time.
//         // This command is at low importance, failure of this command has very small impact.
//         ULONG secondsRemaining = 0;
//
// #if (WINVER >= 0x0601)
//         // this API is introduced in Windows7
//         PoQueryWatchdogTime(DeviceExtension->LowerPdo, &secondsRemaining);
// #endif
//
//         if (secondsRemaining == 0)
//         {
//             // not able to retrieve remaining time from PoQueryWatchdogTime API, use default values.
//             retryCount = MAXIMUM_RETRIES;
//             timeoutValue = SCSI_CDROM_TIMEOUT;
//         }
//         else
//         {
//             // plan to leave about 30 seconds to lower level drivers if possible.
//             if (secondsRemaining >= 32)
//             {
//                 retryCount = (secondsRemaining - 30)/SCSI_CDROM_TIMEOUT + 1;
//                 timeoutValue = SCSI_CDROM_TIMEOUT;
//
//                 if (retryCount > MAXIMUM_RETRIES)
//                 {
//                     retryCount = MAXIMUM_RETRIES;
//                 }
//
//                 if (retryCount == 1)
//                 {
//                     timeoutValue = secondsRemaining - 30;
//                 }
//             }
//             else
//             {
//                 // issue the command with minimal timeout value and do not retry on it.
//                 retryCount = 1;
//                 timeoutValue = 2;
//             }
//         }
//     }
//         break;
//     default:
//         NT_ASSERT( FALSE );
//         status = STATUS_NOT_IMPLEMENTED;
//         return status;
//     }
//
//     DeviceExtension->PowerContext.RetryCount = retryCount;
//
//     // issue command.
//     while (shouldRetry)
//     {
//
//         // set SRB fields.
//         DeviceExtension->PowerContext.Srb.SrbFlags = SRB_FLAGS_NO_DATA_TRANSFER |
//                                                      SRB_FLAGS_DISABLE_SYNCH_TRANSFER |
//                                                      SRB_FLAGS_NO_QUEUE_FREEZE |
//                                                      SRB_FLAGS_BYPASS_LOCKED_QUEUE |
//                                                      SRB_FLAGS_D3_PROCESSING;
//
//         DeviceExtension->PowerContext.Srb.Function = SRB_FUNCTION_EXECUTE_SCSI;
//         DeviceExtension->PowerContext.Srb.TimeOutValue = timeoutValue;
//
//         if (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceInitial)
//         {
//             DeviceExtension->PowerContext.Srb.Function = SRB_FUNCTION_LOCK_QUEUE;
//         }
//         else if (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceLocked)
//         {
//             DeviceExtension->PowerContext.Srb.Function = SRB_FUNCTION_QUIESCE_DEVICE;
//         }
//         else if (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceQuiesced)
//         {
//             // Case of issuing SYNC CACHE command.
//             DeviceExtension->PowerContext.Srb.CdbLength = 10;
//             cdb->SYNCHRONIZE_CACHE10.OperationCode = SCSIOP_SYNCHRONIZE_CACHE;
//         }
//         else if (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceFlushed)
//         {
//             // Case of issuing STOP UNIT command.
//             DeviceExtension->PowerContext.Srb.CdbLength = 6;
//             cdb->START_STOP.OperationCode = SCSIOP_START_STOP_UNIT;
//             cdb->START_STOP.Start = 0;
//             cdb->START_STOP.Immediate = 1;
//         }
//         else if (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceStopped)
//         {
//             DeviceExtension->PowerContext.Srb.Function = SRB_FUNCTION_UNLOCK_QUEUE;
//         }
//
//         // Set up completion routine and context if requested
//         if (CompletionRoutine)
//         {
//             WdfRequestSetCompletionRoutine(DeviceExtension->PowerContext.PowerRequest,
//                                            CompletionRoutine,
//                                            Context);
//         }
//
//         status = RequestSend(DeviceExtension,
//                              DeviceExtension->PowerContext.PowerRequest,
//                              DeviceExtension->IoTarget,
//                              CompletionRoutine ? 0 : WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
//                              &requestSent);
//
//         if (requestSent)
//         {
//             if ((CompletionRoutine == NULL) &&
//                 (SRB_STATUS(DeviceExtension->PowerContext.Srb.SrbStatus) != SRB_STATUS_SUCCESS))
//             {
//                 TracePrint!((TRACE_LEVEL_ERROR,
//                             TRACE_FLAG_POWER,
//                             "%p\tError occured when issuing %s command to device. Srb %p, Status %x\n",
//                             DeviceExtension->PowerContext.PowerRequest,
//                             (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceQuiesced) ? "SYNC CACHE" : "STOP UNIT",
//                             &DeviceExtension->PowerContext.Srb,
//                             DeviceExtension->PowerContext.Srb.SrbStatus));
//
//                 NT_ASSERT(!(TEST_FLAG(DeviceExtension->PowerContext.Srb.SrbStatus, SRB_STATUS_QUEUE_FROZEN)));
//
//                 shouldRetry = RequestSenseInfoInterpret(DeviceExtension,
//                                                         DeviceExtension->PowerContext.PowerRequest,
//                                                         &(DeviceExtension->PowerContext.Srb),
//                                                         retryCount - DeviceExtension->PowerContext.RetryCount,
//                                                         &status,
//                                                         &(DeviceExtension->PowerContext.RetryIntervalIn100ns));
//
//                 if (shouldRetry && (DeviceExtension->PowerContext.RetryCount-- == 0))
//                 {
//                     shouldRetry = FALSE;
//                 }
//             }
//             else
//             {
//                 // succeeded, do not need to retry.
//                 shouldRetry = FALSE;
//             }
//
//         }
//         else
//         {
//             // request failed to be sent
//             shouldRetry = FALSE;
//         }
//
//         if (shouldRetry)
//         {
//             LARGE_INTEGER t;
//             t.QuadPart = -DeviceExtension->PowerContext.RetryIntervalIn100ns;
//             KeDelayExecutionThread(KernelMode, FALSE, &t);
//
//             status = PowerContextReuseRequest(DeviceExtension);
//             if (!NT_SUCCESS(status))
//             {
//                 shouldRetry = FALSE;
//             }
//         }
//     }
//
//     if (DeviceExtension->PowerContext.PowerChangeState.PowerDown == PowerDownDeviceQuiesced)
//     {
//         // record SYNC CACHE command completion time stamp.
//         KeQueryTickCount(&DeviceExtension->PowerContext.Step1CompleteTime);
//     }
//
//     return status;
// }
//
// NTSTATUS
// RequestSend(
//             PCDROM_DEVICE_EXTENSION DeviceExtension,
//             WDFREQUEST              Request,
//             WDFIOTARGET             IoTarget,
//             ULONG                   Flags,
//        PBOOLEAN                RequestSent
//     )
// /*++
//
// Routine Description:
//
//     Send the request to the target, wake up the device from Zero Power state if necessary.
//
// Arguments:
//
//     DeviceExtension - device extension
//     Request - the request to be sent
//     IoTarget - target of the above request
//     Flags - flags for the operation
//     RequestSent - optional, if the request was sent
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                 status  = STATUS_SUCCESS;
//     BOOLEAN                  requestSent = FALSE;
//     WDF_REQUEST_SEND_OPTIONS options;
//
//     UNREFERENCED_PARAMETER(DeviceExtension);
//
//     if ((DeviceExtension->ZeroPowerODDInfo != NULL) &&
//         (DeviceExtension->ZeroPowerODDInfo->InZeroPowerState != FALSE))
//     {
//     }
//
//     // Now send down the request
//     if (NT_SUCCESS(status))
//     {
//         WDF_REQUEST_SEND_OPTIONS_INIT(&options, Flags);
//
//         RequestSetSentTime(Request);
//
//         // send request and check status
//
//         // Disable SDV warning about infinitely waiting in caller's context:
//         //   1. Some requests (such as SCSI_PASS_THROUGH, contains buffer from user space) need to be sent down in caller�s context.
//         //      Consequently, these requests wait in caller�s context until they are allowed to be sent down.
//         //   2. Considering the situation that during sleep, a request can be hold by storage port driver. When system resumes, any time out value (if we set using KMDF time out value) might be expires.
//         //      This will cause the waiting request being failed (behavior change). We�d rather not set time out value.
//
//         _Analysis_assume_(options.Timeout != 0);
//         requestSent = WdfRequestSend(Request, IoTarget, &options);
//         _Analysis_assume_(options.Timeout == 0);
//
//         // If WdfRequestSend fails, or if the WDF_REQUEST_SEND_OPTION_SYNCHRONOUS flag is set,
//         // the driver can call WdfRequestGetStatus immediately after calling WdfRequestSend.
//         if ((requestSent == FALSE) ||
//             (Flags & WDF_REQUEST_SEND_OPTION_SYNCHRONOUS))
//         {
//             status = WdfRequestGetStatus(Request);
//
//             if (requestSent == FALSE)
//             {
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL,
//                             "WdfRequestSend failed: %lx\n",
//                             status
//                             ));
//             }
//         }
//         else
//         {
//             status = STATUS_SUCCESS;
//         }
//
//         if (RequestSent != NULL)
//         {
//             *RequestSent = requestSent;
//         }
//     }
//
//     return status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

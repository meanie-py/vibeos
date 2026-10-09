use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetConfigurationInformation, LPCSTR, LPSTR, NTSTRSAFE_PWSTR, PSTORAGE_DESCRIPTOR_HEADER, PSTORAGE_DEVICE_DESCRIPTOR, PSTORAGE_PROPERTY_ID, RtlInitUnicodeString, STORAGE_HOTPLUG_INFO, STORAGE_PROPERTY_QUERY, STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR};
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID as STORAGE_PROPERTY_ID;
use ntkrnl_sys::ddk::_DEVICE_REMOVAL_POLICY_RemovalPolicyExpectOrderlyRemoval as RemovalPolicyExpectOrderlyRemoval;
use ntkrnl_sys::ddk::_DEVICE_REMOVAL_POLICY_RemovalPolicyExpectSurpriseRemoval as RemovalPolicyExpectSurpriseRemoval;
use core::ffi::c_void;

extern "C" {
    static CDROM_REQUEST_CONTEXT: WDF_OBJECT_CONTEXT_TYPE_INFO;
    fn NT_SUCCESS(status: NTSTATUS) -> bool;
    fn CdromDbgPrint0(format: *const i8) -> u32;
    fn CdromDbgPrint1(format: *const i8, arg1: usize) -> u32;
    fn CdromDbgPrint2(format: *const i8, arg1: usize, arg2: usize) -> u32;
    fn CdromDbgPrint3(format: *const i8, arg1: usize, arg2: usize, arg3: usize) -> u32;
    fn CdromDbgPrint5(format: *const i8, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> u32;
}
// NOTE: C `min()`/`max()` are macros, not symbols. The banked translation
// fabricated `extern "system" fn min/max(u32,u32)->u32`. On i386 `extern
// "system"` is stdcall, but ddk_shims defines them `extern "C"` (cdecl) — a
// calling-convention mismatch that corrupts the stack (and `max` also linked
// to an i64-typed def). The net effect zeroed MaxPageAlignedTransferBytes and
// could smash the return address (BUGCHECK 0x1E ILLEGAL_INSTRUCTION). Use
// correctly-typed local wrappers (plain Rust calls — no FFI, no CC mismatch).
#[inline] fn min(a: u32, b: u32) -> u32 { core::cmp::min(a, b) }
#[inline] fn max(a: u32, b: u32) -> u32 { core::cmp::max(a, b) }
extern "system" {
    fn WDF_OBJECT_ATTRIBUTES_INIT(attributes: *mut WDF_OBJECT_ATTRIBUTES);
    fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(attributes: *mut WDF_OBJECT_ATTRIBUTES, context_type_info: *mut WDF_OBJECT_CONTEXT_TYPE_INFO);
    static DEVPKEY_Device_InLocalMachineContainer: DEVPROPKEY;
    static mut CdromHackItems: CDROM_SCAN_FOR_SPECIAL_INFO;
    fn DeviceErrorHandlerForHitachiGD2000( DeviceExtension: PCDROM_DEVICE_EXTENSION, Srb: *mut SCSI_REQUEST_BLOCK, Status: *mut NTSTATUS, Retry: *mut u8, ) -> NTSTATUS;
    fn DeviceSendSrbSynchronously( Device: *mut core::ffi::c_void, Srb: *mut SCSI_REQUEST_BLOCK, DataBuffer: *mut core::ffi::c_void, DataBufferLength: u32, Write: u8, SrbStatus: *mut u8, ) -> NTSTATUS;
    fn RtlZeroMemory(Destination: *mut core::ffi::c_void, Length: usize);
    fn ExAllocatePoolWithTag(PoolType: u32, NumberOfBytes: SIZE_T, Tag: u32) -> *mut core::ffi::c_void;
    fn ExFreePool(P: *mut core::ffi::c_void);
}
unsafe extern "C" {
}
extern "system" {
    // WDF completion callback: NTAPI/stdcall on i386, C on amd64/arm64.
    // Defined in common.rs as `extern "system"`; declare it with a matching ABI
    // so `Some(DeviceReleaseQueueCompletion)` fits the WDF fn-ptr field on i386.
    fn DeviceReleaseQueueCompletion(request: WDFREQUEST, target: WDFIOTARGET, params: PWDF_REQUEST_COMPLETION_PARAMS, context: WDFCONTEXT);
}
extern "C" {
    fn RtlStringCchPrintfW( pszDest: NTSTRSAFE_PWSTR, cchDest: usize, pszFormat: *const u16, ... ) -> i32;
    fn DeviceInitializeZPODD(deviceExtension: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS;
    fn DeviceEnableMainTimer(deviceExtension: *mut CDROM_DEVICE_EXTENSION) -> u32;
    fn MediaReadCapacity(Device: WDFDEVICE) -> NTSTATUS;
    fn IoSetDeviceInterfacePropertyData( symbolic_link_name: *const ntkrnl_sys::ddk::UNICODE_STRING, property_key: *const DEVPROPKEY, lcid: i32, flags: u32, property_type: u32, property_size: u32, property_data: *const core::ffi::c_void, ) -> NTSTATUS;
}
extern "system" {
    fn WDF_MEMORY_DESCRIPTOR_INIT_BUFFER( descriptor: *mut WDF_MEMORY_DESCRIPTOR, buffer: *mut core::ffi::c_void, length: u32, );
    fn NT_ASSERT( condition: i32, );
    static WdfObjectContextTypeInfo_CDROM_REQUEST_CONTEXT: WDF_OBJECT_CONTEXT_TYPE_INFO;
    fn DeviceGetExtension(Device: WDFDEVICE) -> PCDROM_DEVICE_EXTENSION;
    fn DeviceGetParameter( DeviceExtension: *mut core::ffi::c_void, SubKeyName: *mut u16, ValueName: *mut u16, Value: *mut u32, ) -> ();
    fn TEST_FLAG(Flags: u32, Flag: u32) -> u32;
    fn DeviceSetParameter(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, SubKeyName: *const u16, ValueName: *const u16, Value: u32) -> u32;
    fn DevicePickDvdRegion(Device: WDFDEVICE);
    fn IoGetDevicePropertyData( DeviceObject: *mut core::ffi::c_void, PropertyKey: *const DEVPROPKEY, Lcid: u32, Flags: u32, Size: u32, Data: *mut core::ffi::c_void, RequiredSize: *mut u32, Type: *mut u32, ) -> NTSTATUS;
    fn SET_FLAG(flags: *mut u32, flag: u32);
}
unsafe extern "C" {
    fn DeviceGetConfiguration(Device: *mut core::ffi::c_void, Buffer: *mut core::ffi::c_void, Size: u32, Usable: *mut u32, Feature: u16, RequestType: u32) -> i32; // NTSTATUS
    fn REVERSE_BYTES(dest: *mut u32, src: *const u8);
    fn DeviceAllocateMmcResources(Device: WDFDEVICE) -> i32;
    fn DeviceUpdateMmcWriteCapability(buffer: *const u8, size: u32, check: u8, is_writer: *mut u8, validation_schema: *mut u32, blocking_factor: *mut u32);
}
unsafe extern "C" {
    fn StringsAreMatched(a: *const i8, b: *const i8) -> i32;
}
unsafe extern "C" {
    fn FREE_POOL(Ptr: *mut core::ffi::c_void);
}
unsafe extern "C" {
    fn DeviceFindFeaturePage(buffer: *mut u8, size: u32, feature: u32) -> *mut FEATURE_DATA_CD_READ;
}
unsafe extern "C" {
    fn DevicePrintAllFeaturePages(buffer: *const u8, size: u32);
}
unsafe extern "C" {
    // full def sig (sense.rs) — the old zero-arg decl was address-only shorthand
    // and clashed with the types_* mirrors
    fn DeviceErrorHandlerForMmc(
        DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
        Srb: *mut SCSI_REQUEST_BLOCK,
        Status: *mut i32,
        Retry: *mut u8,
    ) -> i32;
}
unsafe extern "C" {
    fn DeviceInitializeMediaChangeDetection(deviceExtension: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS;
}
unsafe extern "C" {
    fn DeviceSetMediaChangeStateEx( deviceExtension: *mut CDROM_DEVICE_EXTENSION, new_state: u32, old_state: *mut u32, );
}
extern "C" {
    fn DvdStartSessionReadKey( DeviceExtension: *mut CDROM_DEVICE_EXTENSION, IoControlCode: u32, OriginalRequest: WDFREQUEST, InputBuffer: *mut core::ffi::c_void, InputBufferLength: usize, OutputBuffer: *mut core::ffi::c_void, OutputBufferLength: usize, BytesReturned: *mut usize, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
extern "system" {
    fn ScratchBuffer_Allocate(DeviceExtension: PCDROM_DEVICE_EXTENSION) -> u8;
}

// ---- value-alias imports for the integrated bodies below ----
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageAdapterProperty as StorageAdapterProperty;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDeviceProperty as StorageDeviceProperty;
use ntkrnl_sys::ddk::_STORAGE_PROPERTY_ID_StorageDevicePowerProperty as StorageDevicePowerProperty;
use ntkrnl_sys::ddk::_STORAGE_BUS_TYPE_BusTypeFibre as BusTypeFibre;
use ntkrnl_sys::ddk::_STORAGE_BUS_TYPE_BusTypeiScsi as BusTypeiScsi;
use ntkrnl_sys::ddk::_STORAGE_BUS_TYPE_BusTypeRAID as BusTypeRAID;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]

// Forward declarations of unknown types and constants
struct WDF_OBJECT_ATTRIBUTES_impl {
    Size: core::ffi::c_ulong,
    ParentObject: *mut core::ffi::c_void,
    _padding: [u8; 40], // ensure size and layout, will be zeroed anyway
}

// Constants (deduplicated; TRACE_LEVEL_* values only gate tracing).
const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

const IOCTL_SCSI_EXECUTE_NONE: u32 = 0x001B0013;
const FALSE: u8 = 0;
const IOCTL_SCSI_GET_ADDRESS: u32 = 0x41018; // CTL_CODE(IOCTL_SCSI_BASE,0x406,METHOD_BUFFERED,FILE_ANY_ACCESS); was 0x41018c (fabricated extra digit)
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D1400; // typical value, replace if needed
const CDROM_TAG_DESCRIPTOR: u32 = 0x44436353; // placeholder
// cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;
const PropertyStandardQuery: u32 = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32;

const FDO_HACK_INVALID_FLAGS: u32 = 0xFFFFFF80;
const FDO_HACK_CANNOT_LOCK_MEDIA: u32 = 0x1; // unknown actual value, placeholder  // was 0x0002 (LLM-invented); bindgen-correct

type DEVICE_REMOVAL_POLICY = u32;

const CDROM_TAG_INQUIRY: u32 = 0x69436353; // 'C' 'q' 'r' 'y'
const CDROM_TAG_STRINGS: u32 = 0x73436353; // 'CDRM'
const SCSIOP_INQUIRY: u8 = 0x12;
const SRB_STATUS_DATA_OVERRUN: u8 = 0x12;
const MINIMUM_CDROM_INQUIRY_SIZE: u32 = 36;
const STATUS_DEVICE_PROTOCOL_ERROR: NTSTATUS = 0xC0000186_u32 as i32; // 0xC0000185

const PLUGPLAY_REGKEY_DEVICE: u32 = 1;

// cdrom.h registry names (wide literals). These were NULL placeholders, so
// DeviceSetParameter wrote its flag into the UNNAMED default value of the
// device's hardware key, and nothing could ever read it back.
const CDROM_SUBKEY_NAME: *const u16 = ntkrnl_sys::w!("CdRom").as_ptr();
const CDROM_NON_MMC_DRIVE_NAME: *const u16 = ntkrnl_sys::w!("NonMmc").as_ptr();
const CDROM_NON_MMC_VENDOR_SPECIFIC_PROFILE: *const u16 =
    ntkrnl_sys::w!("NonMmcVendorSpecificProfile").as_ptr();
const CDROM_TYPE_ONE_GET_CONFIG_NAME: *const u16 = ntkrnl_sys::w!("NoTypeOneGetConfig").as_ptr();

const CDROM_HACK_BAD_GET_CONFIG_SUPPORT: u32 = 0x100;  // was 0x0001 (LLM-invented); bindgen-correct
const CDROM_HACK_BAD_VENDOR_PROFILES: u32 = 0x1000;  // was 0x0002 (LLM-invented); bindgen-correct
const CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG: u32 = 0x800;  // was 0x0004 (LLM-invented); bindgen-correct

// cdrom.h's FeatureProfileList is 0x0000 (FEATURE_NUMBER). This was HELD at 0x0003 while the
// teardown it exercised crashed: with the correct value QEMU's DVD-ROM answers the TYPE_ONE
// probe below with more than the one feature, the device start tears down through
// DeviceClaimRelease(Release) -> WdfObjectDelete -> FxRequest::Release, and that path read
// past the just-freed request object (amd64/i386 BUGCHECK 0x19, arm64 hang). Root cause was
// in vendored KMDF, not here: clang hoisted FxRequest::Release's load of m_Completed above
// the `retValue == 1 &&` guard that was supposed to gate it, reading the object after
// FinalRelease freed it. Fixed in vendor/wdf/shared/core/fxrequest.cpp (capture m_Completed
// before the __super Release, like the sibling flags), so the real value is restored here.
const FeatureProfileList: u32 = 0x0000;
const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL: u32 = 0;
const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE: u32 = 0x2;  // was 1 (LLM-invented); bindgen-correct

const CDROM_TAG_GET_CONFIG: u32 = 0x43436353; // 'CteG' tag
const MAXUSHORT: u32 = 65535;

const SCSIOP_MODE_SENSE10: u8 = 0x5A;
const SCSIOP_MODE_SENSE: u8 = 0x1A;
const MODE_PAGE_CAPABILITIES: u8 = 0x2A;
const Dbd: u8 = 1;
const CDROM_TAG_MODE_DATA: u32 = 0x4D436353; // FIXED 2026-07-05: was LLM-guessed "typical" value; verified vs C header ('MCcS' per cdrom.h, displays as ScCM)

const CDROM_HACK_TOSHIBA_SD_W1101: u32 = 0x10;  // was 0x0001 (LLM-invented); bindgen-correct
const CDROM_HACK_HITACHI_GD_2000: u32 = 0x8;  // was 0x0002 (LLM-invented); bindgen-correct
const CDROM_HACK_FUJITSU_FMCD_10x: u32 = 0x2;  // was 0x0004 (LLM-invented); bindgen-correct
const CDROM_HACK_DEC_RRD: u32 = 0x1;  // was 0x0010 (LLM-invented); bindgen-correct
const DEV_SAFE_START_UNIT: u32 = 0x4;  // was 0x00000001 (LLM-invented); bindgen-correct
const SCSIOP_MODE_SELECT: u8 = 0x15;
const HITACHI_MODE_DATA_SIZE: u32 = 0xc;  // was 0x10 (LLM-invented); bindgen-correct

// Function pointer type used by ScanForSpecialHandler
type CDROM_SCAN_FOR_SPECIAL_HANDLER = unsafe extern "C" fn(*mut CDROM_DEVICE_EXTENSION, *mut CDROM_SCAN_FOR_SPECIAL_INFO) -> u8; // BOOLEAN

const FILE_DEVICE_CD_ROM: u32 = 0x00000002;
const FILE_DEVICE_DVD: u32 = 0x33;
const BusTypeScsi: u32 = 0x00;
const BusTypeAtapi: u32 = 0x01;
const BusTypeUnknown: u32 = 0x02;
const FeatureAACS: u32 = 0x010D;
const FeatureCdRead: u32 = 0x001E; // _FEATURE_NUMBER_FeatureCdRead = 30
const FeatureDvdRead: u32 = 0x0012;
const FeatureDvdCSS: u32 = 0x0106;
const CdromMmcUpdateRequired: u32 = 1;
const FILE_READ_ONLY_DEVICE: u32 = 0x02;

const CLASS_PERF_RESTORE_MINIMUM: u32 = 0x10; // typical minimum  // was 5 (LLM-invented); bindgen-correct
const SCSI_CDROM_OPC_TIMEOUT: u32 = 0x104; // placeholder (actual value ~300 seconds)  // was 100 (LLM-invented); bindgen-correct
const MediaPresent: u32 = 1;
const DEVPROP_TRUE: i32 = 1;
const DEVPROP_FALSE: i32 = 0;
const DEVPROP_TYPE_BOOLEAN: u32 = 0x00000011; // DEVPROP_TYPE_BOOLEAN
const DevicePropertyRemovalPolicy: u32 = 19; // DEVICE_REGISTRY_PROPERTY
const FILE_REMOVABLE_MEDIA: u32 = 0x01;
const FILE_PORTABLE_DEVICE: u32 = 0x4000;
const DO_SYSTEM_SYSTEM_PARTITION: u32 = 0x00200000;
const DO_SYSTEM_BOOT_PARTITION: u32 = 0x0100;
const DO_SYSTEM_CRITICAL_PARTITION: u32 = 0x00400000;

// DVD copy-protect / RPC constants (init.c -> ntddcdvd.h)
const IOCTL_DVD_READ_KEY: u32 = 0x00335004; // FIXED 2026-07-05: translation invented wrong value (verified vs psdk headers)
const DVD_TAG_RPC2_CHECK: u32 = 0x56436353; // 'RPC2'
const DvdGetRpcKey: u32 = 8; // _DVD_KEY_TYPE_DvdGetRpcKey
const DVD_RPC_KEY_LENGTH: u32 =
    (core::mem::size_of::<DVD_RPC_KEY>() + core::mem::size_of::<DVD_COPY_PROTECT_KEY>()) as u32;

/// classp.h spells these registry names as WIDE literals
/// (`#define CLASSP_REG_SUBKEY_NAME (L"Classpnp")`), but the type substrate
/// carries them as ASCII byte strings, and DeviceGetParameter reads UTF-16.
/// The declaration used to say `*const i8`, so the call sites cast the byte
/// string to match it and the callee read "Classpnp\0" as the wide string
/// 0x6C43 0x7373... — a subkey that does not exist. Every parameter fetched
/// through it silently kept its default. One call site in this file already
/// widened correctly (DeviceInitializeHotplugInfo); these three did not.
fn widen<const N: usize>(b: &[u8; N]) -> [u16; N] {
    core::array::from_fn(|i| b[i] as u16)
}

#[no_mangle]
pub unsafe extern "C" fn DeviceClaimRelease(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Release: u8,
) -> i32 {
    // PAGED_CODE();
    let mut status: i32 = 0;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut descriptor: WDF_MEMORY_DESCRIPTOR = core::mem::zeroed();
    let mut request: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();

    // Create a request
    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
        core::ptr::addr_of_mut!(attributes),
        core::ptr::addr_of!(CDROM_REQUEST_CONTEXT) as *mut WDF_OBJECT_CONTEXT_TYPE_INFO,
    );

    status = WdfRequestCreate(
        core::ptr::addr_of_mut!(attributes),
        (*DeviceExtension).IoTarget,
        core::ptr::addr_of_mut!(request) as *mut WDFREQUEST,
    );

    if status >= 0 {
        // fill up srb structure
        srb.OriginalRequest = WdfRequestWdmGetIrp(request as WDFREQUEST) as *mut core::ffi::c_void;
        NT_ASSERT(if !srb.OriginalRequest.is_null() { 1 } else { 0 });

        srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;

        srb.Function = if Release != 0 {
            0x06 /* SRB_FUNCTION_RELEASE_DEVICE */
        } else {
            0x01 /* SRB_FUNCTION_CLAIM_DEVICE */
        };

        WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(
            core::ptr::addr_of_mut!(descriptor),
            core::ptr::addr_of_mut!(srb) as *mut core::ffi::c_void,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32,
        );

        status = WdfIoTargetSendInternalIoctlOthersSynchronously(
            (*DeviceExtension).IoTarget,
            request as WDFREQUEST,
            0x001B0013 /* IOCTL_SCSI_EXECUTE_NONE = (FILE_DEVICE_SCSI<<16)+0x13 */,
            core::ptr::addr_of_mut!(descriptor),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        NT_ASSERT(if (srb.SrbFlags & 0x04 /* SRB_FLAGS_FREE_SENSE_BUFFER */) == 0 {
            1
        } else {
            0
        });

        // The request should be deleted.
        WdfObjectDelete(request);

        if status < 0 {
            let format_str: *const u8 = if Release != 0 {
                b"DeviceClaimRelease: Failed to Release device, status: 0x%X\n\x00".as_ptr()
            } else {
                b"DeviceClaimRelease: Failed to Claim device, status: 0x%X\n\x00".as_ptr()
            };
            TracePrint!(
                1 /* TRACE_LEVEL_FATAL */,
                0x01 /* TRACE_FLAG_PNP */,
                format_str as *const i8,
                status,
            );
        }
    } else {
        let format_str = b"DeviceClaimRelease: Failed to create request, status: 0x%X\n\x00";
        TracePrint!(
            1 /* TRACE_LEVEL_FATAL */,
            0x01 /* TRACE_FLAG_PNP */,
            format_str.as_ptr() as *const i8,
            status,
        );
    }

    if Release != 0 {
        // We only release the device when we don't want to manage it.
        // The failure status does not matter.
        status = 0; // STATUS_SUCCESS
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn DeviceEvtSelfManagedIoInit(Device: WDFDEVICE) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut deviceExtension: *mut CDROM_DEVICE_EXTENSION = core::ptr::null_mut();

    PAGED_CODE!();

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_PNP,
        b"DeviceEvtSelfManagedIoInit: WDFDEVICE %p is being started.\n\0" as *const u8 as *const i8,
        Device as usize,
    );

    deviceExtension = DeviceGetExtension(Device);
    CdromDbgPrint0(b"[SMIO] enter\n\0".as_ptr() as *const i8);
    if deviceExtension.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    let io_target = core::ptr::read_volatile(core::ptr::addr_of!((*deviceExtension).IoTarget));
    let is_initialized = core::ptr::read_volatile(core::ptr::addr_of!((*deviceExtension).IsInitialized));
    CdromDbgPrint3(
        b"[SMIO] ext=%p io=%p init=%x\n\0".as_ptr() as *const i8,
        deviceExtension as usize,
        io_target as usize,
        is_initialized as usize,
    );

    // 1. Set/retrieve basic information, some of the following operations may depend on it
    if NT_SUCCESS(status) {
        // We do not care if this function fails, SCSI address is mainly for debugging/tracing purposes.
        CdromDbgPrint0(b"[SMIO] 1a-pre-scsiaddr\n\0".as_ptr() as *const i8);
        let _ = DeviceRetrieveScsiAddress(deviceExtension, &mut (*deviceExtension).ScsiAddress);
        CdromDbgPrint0(b"[SMIO] 1a-scsiaddr done\n\0".as_ptr() as *const i8);
    }

    if NT_SUCCESS(status) {
        CdromDbgPrint0(b"[SMIO] 1b-pre-descriptors\n\0".as_ptr() as *const i8);
        status = DeviceRetrieveDescriptorsAndTransferLength(deviceExtension);
        CdromDbgPrint1(b"[SMIO] 1b-descriptors done st=%x\n\0".as_ptr() as *const i8, status as usize);
    }

    if NT_SUCCESS(status) {
        // This function should be called after DeviceRetrieveDescriptorsAndTransferLength()
        // It depends on MaxTransferLenth fields.
        status = DeviceInitAllocateBuffers(deviceExtension);
    }

    CdromDbgPrint1(b"[SMIO] 1-descriptors+bufs st=%x\n\0".as_ptr() as *const i8, status as usize);
    // 2. The following functions depend on the allocated buffers.
    // perf re-enable after failing. Q: Is this one used by cdrom.sys?
    if NT_SUCCESS(status) {
        // allow perf to be re-enabled after a given number of failed IOs
        // require this number to be at least CLASS_PERF_RESTORE_MINIMUM
        let mut t: u32 = CLASS_PERF_RESTORE_MINIMUM;

        let sub = widen(CLASSP_REG_SUBKEY_NAME);
        let val = widen(CLASSP_REG_PERF_RESTORE_VALUE_NAME);
        DeviceGetParameter(
            deviceExtension as *mut core::ffi::c_void,
            sub.as_ptr().cast_mut(),
            val.as_ptr().cast_mut(),
            &mut t,
        );
        if t >= CLASS_PERF_RESTORE_MINIMUM {
            (*(*deviceExtension).PrivateFdoData).Perf.ReEnableThreshhold = t;
        }
    }

    // 3. Retrieve information about special devices and hack flags.
    if NT_SUCCESS(status) {
        DeviceRetrieveHackFlagsFromRegistry(deviceExtension);
        // scan for bad items.
        DeviceScanForSpecial(deviceExtension, (&raw mut CdRomBadItems).cast::<CDROM_SCAN_FOR_SPECIAL_INFO>(), Some(DeviceHackFlagsScan));
        // Check to see if it's a special device that needs special error process.
        DeviceScanSpecialDevices(deviceExtension); // may send command to device
    }

    CdromDbgPrint0(b"[SMIO] 3-scanspecial done\n\0".as_ptr() as *const i8);
    // 4. Initialize the hotplug information only after the ScanForSpecial routines,
    // as it relies upon the hack flags - deviceExtension->PrivateFdoData->HackFlags.
    if NT_SUCCESS(status) {
        status = DeviceInitializeHotplugInfo(deviceExtension);
    }
    CdromDbgPrint1(b"[SMIO] 4-hotplug st=%x\n\0".as_ptr() as *const i8, status as usize);

    if NT_SUCCESS(status) {
        // cache the device's inquiry data
        CdromDbgPrint0(b"[SMIO] 4b-pre-inquiry\n\0".as_ptr() as *const i8);
        status = DeviceCacheDeviceInquiryData(deviceExtension);
        if !NT_SUCCESS(status) {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Failed to cache the device's inquiry data, failing %!STATUS!\n\0" as *const u8 as *const i8,
                status,
            );
        }
    }
    CdromDbgPrint1(b"[SMIO] 4b-inquiry st=%x\n\0".as_ptr() as *const i8, status as usize);

    // 5. Initialize MMC context, media change notification stuff and read media capacity
    if NT_SUCCESS(status) {
        status = DeviceInitializeMediaChangeDetection(deviceExtension);
    }
    CdromDbgPrint1(b"[SMIO] 5a-mcd st=%x\n\0".as_ptr() as *const i8, status as usize);
    if NT_SUCCESS(status) {
        status = DeviceInitMmcContext(deviceExtension);
    }
    CdromDbgPrint1(b"[SMIO] 5b-mmc st=%x\n\0".as_ptr() as *const i8, status as usize);
    if NT_SUCCESS(status) {
        status = DeviceInitializeZPODD(deviceExtension);
    }
    CdromDbgPrint1(b"[SMIO] 5c-zpodd st=%x\n\0".as_ptr() as *const i8, status as usize);
    if NT_SUCCESS(status) {
        CdromDbgPrint0(b"[SMIO] 5d-pre-readcapacity\n\0".as_ptr() as *const i8);
        // Do READ CAPACITY. This SCSI command returns the last sector address
        // on the device and the bytes per sector. These are used to calculate
        // the drive capacity in bytes.
        status = MediaReadCapacity(Device);

        // If READ CAPACITY succeeded, we can safely conclude that there is a media present
        if NT_SUCCESS(status) {
            DeviceSetMediaChangeStateEx(deviceExtension, MediaPresent, core::ptr::null_mut());
        }

        // READ CAPACITY is not critical for init, ignore all errors occuring during its execution
        status = STATUS_SUCCESS;
    }

    // 6. Perform DVD-specific initialization
    if NT_SUCCESS(status) {
        status = DeviceInitializeDvd(Device);
    }

    // 7. Miscellaneous initialization actions
    if NT_SUCCESS(status) {
        if !(*deviceExtension).PrivateFdoData.is_null() {
            (*(*deviceExtension).PrivateFdoData).Perf.OriginalSrbFlags = (*deviceExtension).SrbFlags;
        }

        if (*deviceExtension).DeviceAdditionalData.Mmc.IsWriter != 0 {
            // OPC can really take this long per IMAPIv1 timeout....
            (*deviceExtension).TimeOutValue = max((*deviceExtension).TimeOutValue, SCSI_CDROM_OPC_TIMEOUT);
        }
    }

    // 8. Enable the main timer, create ARC name as needed
    if NT_SUCCESS(status) {
        // Device successfully added and initialized, increase CdRomCount.
        CdromDbgPrint0(b"[SMIO] 8-CdRomCount++\n\0".as_ptr() as *const i8);
        let config = IoGetConfigurationInformation();
        if !config.is_null() {
            let count = core::ptr::read_volatile(core::ptr::addr_of!((*config).CdRomCount));
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*config).CdRomCount), count + 1);
        }

        core::ptr::write_volatile(core::ptr::addr_of_mut!((*deviceExtension).IsInitialized), 1);

        DeviceEnableMainTimer(deviceExtension);
    }

    // 9. Set volume interface properties (NTDDI_VERSION >= NTDDI_WIN8)
    if NT_SUCCESS(status) {
        let mut isCritical: i32 = 0;
        let mut isPortable: u8 = 0;
        let isRemovable: i32 = (TEST_FLAG(
            (*(*deviceExtension).DeviceObject).Characteristics,
            FILE_REMOVABLE_MEDIA,
        ) != 0) as i32;
        let mut propCritical: i32 = DEVPROP_FALSE;
        let mut propPortable: i32 = DEVPROP_FALSE;
        let mut propRemovable: i32 = DEVPROP_FALSE;

        status = DeviceIsPortable(deviceExtension, &mut isPortable);

        if NT_SUCCESS(status) {
            if isPortable != 0 {
                SET_FLAG(
                    &mut (*(*deviceExtension).DeviceObject).Characteristics,
                    FILE_PORTABLE_DEVICE,
                );
            }

            propPortable = if isPortable != 0 { DEVPROP_TRUE } else { DEVPROP_FALSE };

            status = IoSetDeviceInterfacePropertyData(
                &(*deviceExtension).MountedDeviceInterfaceName,
                &DEVPKEY_Storage_Portable,
                0,
                0,
                DEVPROP_TYPE_BOOLEAN,
                core::mem::size_of::<i32>() as u32,
                &propPortable as *const i32 as *const core::ffi::c_void,
            );
        }

        if NT_SUCCESS(status) {
            propRemovable = if isRemovable != 0 { DEVPROP_TRUE } else { DEVPROP_FALSE };

            status = IoSetDeviceInterfacePropertyData(
                &(*deviceExtension).MountedDeviceInterfaceName,
                &DEVPKEY_Storage_Removable_Media,
                0,
                0,
                DEVPROP_TYPE_BOOLEAN,
                core::mem::size_of::<i32>() as u32,
                &propRemovable as *const i32 as *const core::ffi::c_void,
            );
        }

        if NT_SUCCESS(status) {
            isCritical = (TEST_FLAG(
                (*(*deviceExtension).DeviceObject).Flags,
                DO_SYSTEM_SYSTEM_PARTITION | DO_SYSTEM_BOOT_PARTITION | DO_SYSTEM_CRITICAL_PARTITION,
            ) != 0) as i32;

            propCritical = if isCritical != 0 { DEVPROP_TRUE } else { DEVPROP_FALSE };

            status = IoSetDeviceInterfacePropertyData(
                &(*deviceExtension).MountedDeviceInterfaceName,
                &DEVPKEY_Storage_System_Critical,
                0,
                0,
                DEVPROP_TYPE_BOOLEAN,
                core::mem::size_of::<i32>() as u32,
                &propCritical as *const i32 as *const core::ffi::c_void,
            );
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceInitReleaseQueueContext(DeviceExtension: *mut CDROM_DEVICE_EXTENSION) -> i32 {
    let mut status: i32 = STATUS_SUCCESS; // STATUS_SUCCESS is imported from ntkrnl_sys::base, assumed available
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();

    // PAGED_CODE(); // omitted (irrelevant in mechanical transliteration)

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
        core::ptr::addr_of_mut!(attributes),
        core::ptr::addr_of!(CDROM_REQUEST_CONTEXT) as *mut WDF_OBJECT_CONTEXT_TYPE_INFO,
    );

    attributes.ParentObject = (*DeviceExtension).Device as *mut core::ffi::c_void;

    status = WdfRequestCreate(
        core::ptr::addr_of_mut!(attributes),
        (*DeviceExtension).IoTarget,
        core::ptr::addr_of_mut!((*DeviceExtension).ReleaseQueueRequest),
    );

    if !NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_PNP,
            b"Cannot create the release queue request\n\0".as_ptr() as *const i8,
        );
        return status;
    }

    // Initialize ReleaseQueueInputMemory, a wrapper around ReleaseQueueSrb
    WDF_OBJECT_ATTRIBUTES_INIT(core::ptr::addr_of_mut!(attributes));
    attributes.ParentObject = (*DeviceExtension).ReleaseQueueRequest as *mut core::ffi::c_void;

    status = WdfMemoryCreatePreallocated(
        core::ptr::addr_of_mut!(attributes),
        core::ptr::addr_of_mut!((*DeviceExtension).ReleaseQueueSrb) as *mut core::ffi::c_void,
        core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        core::ptr::addr_of_mut!((*DeviceExtension).ReleaseQueueInputMemory),
    );

    if !NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_PNP,
            b"Failed to allocate ReleaseQueueSrb.\n\0".as_ptr() as *const i8,
        );
        return status;
    }

    // Preformat the release queue request here to ensure that this call will never
    // fail during an actual release of the queue.
    if NT_SUCCESS(status) {
        status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            (*DeviceExtension).IoTarget,
            (*DeviceExtension).ReleaseQueueRequest,
            IOCTL_SCSI_EXECUTE_NONE,
            (*DeviceExtension).ReleaseQueueInputMemory,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    // Set a CompletionRoutine callback function for ReleaseQueueRequest.
    if NT_SUCCESS(status) {
        WdfRequestSetCompletionRoutine(
            (*DeviceExtension).ReleaseQueueRequest,
            Some(core::mem::transmute(DeviceReleaseQueueCompletion as unsafe extern "system" fn(_, _, _, _))),
            (*DeviceExtension).Device as WDFCONTEXT,
        );
    }

    // Create a spinlock for ReleaseQueueRequest
    WDF_OBJECT_ATTRIBUTES_INIT(core::ptr::addr_of_mut!(attributes));
    attributes.ParentObject = (*DeviceExtension).Device as *mut core::ffi::c_void;

    status = WdfSpinLockCreate(
        core::ptr::addr_of_mut!(attributes),
        core::ptr::addr_of_mut!((*DeviceExtension).ReleaseQueueSpinLock),
    );

    if !NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_PNP,
            b"DeviceInitReleaseQueueContext: Cannot create the release queue spinlock\n\0".as_ptr() as *const i8,
        );
        return status;
    }

    // Initialize miscellaneous ReleaseQueue related fields
    (*DeviceExtension).ReleaseQueueNeeded = FALSE;
    (*DeviceExtension).ReleaseQueueInProgress = FALSE;
    (*DeviceExtension).ReleaseQueueSrb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceInitPowerContext(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    // Unknown extern declarations

    // Local constants
    const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
    const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const IOCTL_SCSI_EXECUTE_IN: u32 = 0x1b0011;  // was 0x0000001B (LLM-invented); bindgen-correct

    let mut status: NTSTATUS = 0; // STATUS_SUCCESS
    let mut attributes = core::mem::MaybeUninit::<WDF_OBJECT_ATTRIBUTES>::uninit();

    // PAGED_CODE();

    // create request object for Power operations
    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
        attributes.as_mut_ptr(),
        core::ptr::addr_of!(WdfObjectContextTypeInfo_CDROM_REQUEST_CONTEXT) as *mut WDF_OBJECT_CONTEXT_TYPE_INFO,
    );
    (*(attributes.as_mut_ptr())).ParentObject = (*(DeviceExtension)).Device as *mut core::ffi::c_void;

    status = WdfRequestCreate(
        attributes.as_mut_ptr(),
        (*(DeviceExtension)).IoTarget,
        &raw mut (*(DeviceExtension)).PowerContext.PowerRequest,
    );

    if (status) < 0 {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_PNP,
            b"Cannot create the power request object.\n\0".as_ptr() as *const i8,
        );

        return status;
    }

    // Preformat the power request. With this being done, we never need to worry about
    // WdfIoTargetFormatRequestForInternalIoctlOthers ever failing later.
    status = WdfIoTargetFormatRequestForInternalIoctlOthers(
        (*(DeviceExtension)).IoTarget,
        (*(DeviceExtension)).PowerContext.PowerRequest,
        IOCTL_SCSI_EXECUTE_IN,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceCreateWellKnownName(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;
    let mut unicodeLinkName = ntkrnl_sys::ddk::UNICODE_STRING {
        Length: 0u16,
        MaximumLength: 0u16,
        Buffer: core::ptr::null_mut(),
    };
    let mut wideLinkName: [u16; 64] = [0u16; 64];
    let mut savedName: *mut u16;
    let mut cdromNumber: i32 = (*DeviceExtension).DeviceNumber as i32;

    // PAGED_CODE();

    if !(*DeviceExtension).DeviceAdditionalData.WellKnownName.Buffer.is_null() {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"DeviceCreateWellKnownName: link already exists %p\n\0".as_ptr() as *const i8,
            (*DeviceExtension).DeviceAdditionalData.WellKnownName.Buffer,
        );
        NT_ASSERT(0);
        return STATUS_UNSUCCESSFUL;
    }

    loop {
        status = crate::write_wide_cdrom_name(
            wideLinkName.as_mut_ptr(),
            64,
            b"\\DosDevices\\CdRom",
            cdromNumber,
        );
        if status & 0x80000000u32 as i32 != 0 {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_INIT,
                b"DeviceCreateWellKnownName: Format symbolic link failed with error: 0x%X\n\0".as_ptr() as *const i8,
                status,
            );
            return status;
        }

        RtlInitUnicodeString(&mut unicodeLinkName, wideLinkName.as_mut_ptr());

        status = WdfDeviceCreateSymbolicLink(
            (*DeviceExtension).Device,
            &unicodeLinkName,
        );

        cdromNumber += 1;

        if !((status == STATUS_OBJECT_NAME_COLLISION) || (status == STATUS_OBJECT_NAME_EXISTS)) {
            break;
        }
    }

    if status & 0x80000000u32 as i32 != 0 {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"DeviceCreateWellKnownName: Error %lx linking %wZ to device %wZ\n\0".as_ptr() as *const i8,
            status,
            &unicodeLinkName as *const ntkrnl_sys::ddk::UNICODE_STRING,
            &(*DeviceExtension).DeviceName as *const ntkrnl_sys::ddk::UNICODE_STRING,
        );
        return status;
    }

    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_INIT,
        b"DeviceCreateWellKnownName: successfully linked %wZ to device %wZ\n\0".as_ptr() as *const i8,
        &unicodeLinkName as *const ntkrnl_sys::ddk::UNICODE_STRING,
        &(*DeviceExtension).DeviceName as *const ntkrnl_sys::ddk::UNICODE_STRING,
    );

    savedName = ExAllocatePoolWithTag(
        PagedPool,
        unicodeLinkName.MaximumLength as SIZE_T,
        CDROM_TAG_STRINGS,
    ) as *mut u16;

    if savedName.is_null() {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"DeviceCreateWellKnownName: unable to allocate memory.\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // NB: write_bytes count is in *elements of T*; savedName is *mut u16, so the count
    // must be a BYTE count cast through *mut u8 (MaximumLength is bytes). Writing
    // `write_bytes(savedName, 0, MaximumLength)` would zero MaximumLength*2 bytes and
    // overflow the pool block (BUGCHECK 0x19 BAD_POOL_HEADER).
    core::ptr::write_bytes(savedName as *mut u8, 0u8, unicodeLinkName.MaximumLength as usize);
    core::ptr::copy_nonoverlapping(
        unicodeLinkName.Buffer as *const u16,
        savedName,
        unicodeLinkName.MaximumLength as usize / core::mem::size_of::<u16>(),
    );

    RtlInitUnicodeString(
        &mut (*DeviceExtension).DeviceAdditionalData.WellKnownName,
        savedName,
    );

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRetrieveDescriptor(
    Device: WDFDEVICE,
    PropertyId: *mut STORAGE_PROPERTY_ID,
    Descriptor: *mut PSTORAGE_DESCRIPTOR_HEADER,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut memory_descriptor: WDF_MEMORY_DESCRIPTOR = core::mem::zeroed();
    let mut query: STORAGE_PROPERTY_QUERY = core::mem::zeroed();
    let mut buffer_length: u32 = 0;
    let mut descriptor: *mut STORAGE_DESCRIPTOR_HEADER = core::ptr::null_mut();
    let device_extension: PCDROM_DEVICE_EXTENSION = DeviceGetExtension(Device);

    PAGED_CODE!();

    // Set the passed-in descriptor pointer to NULL as default
    *Descriptor = core::ptr::null_mut();

    // On the first pass we just want to get the first few
    // bytes of the descriptor so we can read it's size
    query.PropertyId = *PropertyId;
    query.QueryType = PropertyStandardQuery;

    descriptor = (&query as *const STORAGE_PROPERTY_QUERY) as *mut STORAGE_DESCRIPTOR_HEADER;

    NT_ASSERT(
        (core::mem::size_of::<STORAGE_PROPERTY_QUERY>() >= 2 * core::mem::size_of::<u32>()) as i32,
    );

    WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(
        core::ptr::addr_of_mut!(memory_descriptor),
        (&query as *const STORAGE_PROPERTY_QUERY) as *mut core::ffi::c_void,
        core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
    );

    status = WdfIoTargetSendIoctlSynchronously(
        (*device_extension).IoTarget,
        core::ptr::null_mut(),
        IOCTL_STORAGE_QUERY_PROPERTY,
        core::ptr::addr_of_mut!(memory_descriptor),
        core::ptr::addr_of_mut!(memory_descriptor),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"DeviceRetrieveDescriptor: error %lx trying to \"query properties #1\n\0".as_ptr() as *const i8,
            status,
        );
        return status;
    }

    if (*descriptor).Size == 0 {
        // This DebugPrint is to help third-party driver writers
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"DeviceRetrieveDescriptor: size returned was zero?! (status %x\n\0".as_ptr() as *const i8,
            status,
        );
        return STATUS_UNSUCCESSFUL;
    }

    // This time we know how much data there is so we can
    // allocate a buffer of the correct size
    buffer_length = (*descriptor).Size;
    NT_ASSERT(
        (buffer_length as usize >= core::mem::size_of::<STORAGE_PROPERTY_QUERY>()) as i32,
    );
    if (buffer_length as usize) < core::mem::size_of::<STORAGE_PROPERTY_QUERY>() {
        buffer_length = core::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32;
    }

    descriptor = ntkrnl_sys::ex::ExAllocatePoolWithTag(
        NonPagedPoolNx,
        buffer_length as usize,
        CDROM_TAG_DESCRIPTOR,
    ) as *mut STORAGE_DESCRIPTOR_HEADER;

    if descriptor.is_null() {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"DeviceRetrieveDescriptor: unable to memory for descriptor (%d bytes)\n\0".as_ptr() as *const i8,
            buffer_length,
        );
        return ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES;
    }

    // setup the query again, as it was overwritten above
    ntkrnl_sys::rtl::RtlZeroMemory(
        &mut query as *mut STORAGE_PROPERTY_QUERY as *mut core::ffi::c_void,
        core::mem::size_of::<STORAGE_PROPERTY_QUERY>(),
    );
    query.PropertyId = *PropertyId;
    query.QueryType = PropertyStandardQuery;

    // copy the input to the new outputbuffer
    ntkrnl_sys::rtl::RtlCopyMemory(
        descriptor as *mut core::ffi::c_void,
        &query as *const STORAGE_PROPERTY_QUERY as *const core::ffi::c_void,
        core::mem::size_of::<STORAGE_PROPERTY_QUERY>(),
    );

    WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(
        core::ptr::addr_of_mut!(memory_descriptor),
        descriptor as *mut core::ffi::c_void,
        buffer_length,
    );

    status = WdfIoTargetSendIoctlSynchronously(
        (*device_extension).IoTarget,
        core::ptr::null_mut(),
        IOCTL_STORAGE_QUERY_PROPERTY,
        core::ptr::addr_of_mut!(memory_descriptor),
        core::ptr::addr_of_mut!(memory_descriptor),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_INIT,
            b"DeviceRetrieveDescriptor: error %lx trying to \"query properties #1\n\0".as_ptr() as *const i8,
            status,
        );
        ntkrnl_sys::ddk::ExFreePool(descriptor as *mut core::ffi::c_void);

        return status;
    }

    // return the memory we've allocated to the caller
    *Descriptor = descriptor;

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceRetrieveHackFlagsFromRegistry(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    let mut status: i32 = STATUS_SUCCESS;
    let mut hardware_key: WDFKEY = core::ptr::null_mut();
    let mut sub_key: WDFKEY = core::ptr::null_mut();
    let mut device_hacks: u32 = 0;

    // PAGED_CODE(); -- ignored

    // Build UNICODE_STRINGs for the (ASCII) registry names declared as byte
    // strings in the type substrate. DECLARE_CONST_UNICODE_STRING in the C used
    // wide literals; widen the ASCII bytes here.
    let sub_key_wide: [u16; CLASSP_REG_SUBKEY_NAME.len()] =
        core::array::from_fn(|i| CLASSP_REG_SUBKEY_NAME[i] as u16);
    let hack_value_wide: [u16; CLASSP_REG_HACK_VALUE_NAME.len()] =
        core::array::from_fn(|i| CLASSP_REG_HACK_VALUE_NAME[i] as u16);
    let mut sub_key_name: ntkrnl_sys::ddk::UNICODE_STRING = core::mem::zeroed();
    let mut hack_value_name: ntkrnl_sys::ddk::UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&raw mut sub_key_name, sub_key_wide.as_ptr());
    RtlInitUnicodeString(&raw mut hack_value_name, hack_value_wide.as_ptr());

    status = WdfDeviceOpenRegistryKey(
        (*DeviceExtension).Device,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_READ,
        core::ptr::null_mut(), // WDF_NO_OBJECT_ATTRIBUTES
        &raw mut hardware_key,
    );
    if NT_SUCCESS(status) {
        status = WdfRegistryOpenKey(
            hardware_key,
            &sub_key_name as *const ntkrnl_sys::ddk::UNICODE_STRING,
            KEY_READ,
            core::ptr::null_mut(),
            &raw mut sub_key,
        );
        if NT_SUCCESS(status) {
            status = WdfRegistryQueryULong(
                sub_key,
                &hack_value_name as *const ntkrnl_sys::ddk::UNICODE_STRING,
                &raw mut device_hacks,
            );
            if NT_SUCCESS(status) {
                // CLEAR_FLAG(device_hacks, FDO_HACK_INVALID_FLAGS)
                device_hacks &= !FDO_HACK_INVALID_FLAGS;
                // SET_FLAG(DeviceExtension->PrivateFdoData->HackFlags, device_hacks)
                let private_fdo = (*DeviceExtension).PrivateFdoData;
                (*private_fdo).HackFlags |= device_hacks as ntkrnl_sys::ddk::ULONG_PTR;
            }
            WdfRegistryClose(sub_key);
        }
        WdfRegistryClose(hardware_key);
    }

    // (comments preserved from original)
}
#[no_mangle]
pub unsafe extern "C" fn DeviceScanForSpecial(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    DeviceList: *mut CDROM_SCAN_FOR_SPECIAL_INFO,
    Function: Option<unsafe extern "C" fn(*mut CDROM_DEVICE_EXTENSION, usize)>,
) {
    // Constants with inline comment
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_FLAG_INIT: u32 = 8; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    PAGED_CODE!();

    // NT_ASSERT(DeviceList)
    NT_ASSERT((!DeviceList.is_null()) as i32);
    // NT_ASSERT(Function)
    NT_ASSERT((Function.is_some()) as i32);

    if DeviceList.is_null() {
        return;
    }
    if Function.is_none() {
        return;
    }

    let deviceDescriptor: *mut STORAGE_DEVICE_DESCRIPTOR = (*DeviceExtension).DeviceDescriptor;

    // nullString: an empty string used as fallback
    let null_string: [u8; 1] = [0];

    let vendorId: *mut u8;
    if (*deviceDescriptor).VendorIdOffset != 0 && (*deviceDescriptor).VendorIdOffset != u32::MAX {
        vendorId = (deviceDescriptor as *mut u8).add((*deviceDescriptor).VendorIdOffset as usize);
    } else {
        vendorId = null_string.as_ptr() as *mut u8;
    }

    let productId: *mut u8;
    if (*deviceDescriptor).ProductIdOffset != 0 && (*deviceDescriptor).ProductIdOffset != u32::MAX {
        productId = (deviceDescriptor as *mut u8).add((*deviceDescriptor).ProductIdOffset as usize);
    } else {
        productId = null_string.as_ptr() as *mut u8;
    }

    let productRevision: *mut u8;
    if (*deviceDescriptor).ProductRevisionOffset != 0 && (*deviceDescriptor).ProductRevisionOffset != u32::MAX {
        productRevision = (deviceDescriptor as *mut u8).add((*deviceDescriptor).ProductRevisionOffset as usize);
    } else {
        productRevision = null_string.as_ptr() as *mut u8;
    }

    // loop while any field is non-null (C for loop: DeviceList->VendorId != NULL || ...)
    let mut current_list: *mut CDROM_SCAN_FOR_SPECIAL_INFO = DeviceList;
    while !((*current_list).VendorId.is_null() && (*current_list).ProductId.is_null() && (*current_list).ProductRevision.is_null()) {
        if StringsAreMatched((*current_list).VendorId as *const i8, vendorId as *const i8) != 0 &&
            StringsAreMatched((*current_list).ProductId as *const i8, productId as *const i8) != 0 &&
            StringsAreMatched((*current_list).ProductRevision as *const i8, productRevision as *const i8) != 0
        {
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
                       b"DeviceScanForSpecial: Found matching controller Ven: %s Prod: %s Rev: %s\n\0".as_ptr() as *const i8,
                       vendorId as *const i8,
                       productId as *const i8,
                       productRevision as *const i8);
            if let Some(func) = Function {
                func(DeviceExtension, (*current_list).Data as usize);
            }
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
                       b"DeviceScanForSpecial: completed callback\n\0".as_ptr() as *const i8,
                       core::ptr::null::<i8>(),
                       core::ptr::null::<i8>(),
                       core::ptr::null::<i8>());
            return;
        }
        current_list = current_list.add(1);
    }

    TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
               b"DeviceScanForSpecial: no match found for %p\n\0".as_ptr() as *const i8,
               (*DeviceExtension).DeviceObject as *const i8,
               core::ptr::null::<core::ffi::c_void>(),
               core::ptr::null::<core::ffi::c_void>());
    return;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceHackFlagsScan(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Data: usize,
) {
    PAGED_CODE!();

    let mut data = Data;
    // CLEAR_FLAG
    data &= !(FDO_HACK_INVALID_FLAGS as usize);
    // SET_FLAG
    (*(*DeviceExtension).PrivateFdoData).HackFlags |= data as ntkrnl_sys::ddk::ULONG_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceInitializeHotplugInfo(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    unsafe {
        let mut status: NTSTATUS = STATUS_SUCCESS;
        let fdoData = (*DeviceExtension).PrivateFdoData;
        let mut deviceRemovalPolicy: DEVICE_REMOVAL_POLICY = 0;
        let mut resultLength: u32 = 0;
        let mut writeCacheOverride: u32;

        // RtlZeroMemory(&(fdoData->HotplugInfo), sizeof(STORAGE_HOTPLUG_INFO));
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*fdoData).HotplugInfo) as *mut u8,
            0u8,
            core::mem::size_of::<STORAGE_HOTPLUG_INFO>(),
        );

        // set the size (aka version)
        (*fdoData).HotplugInfo.Size = core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as u32;

        // set if the device has removable media
        if (*((*DeviceExtension).DeviceDescriptor)).RemovableMedia != 0 {
            (*fdoData).HotplugInfo.MediaRemovable = 1; // TRUE
        } else {
            (*fdoData).HotplugInfo.MediaRemovable = 0; // FALSE
        }

        if TEST_FLAG(
            (*(*DeviceExtension).PrivateFdoData).HackFlags as u32,
            FDO_HACK_CANNOT_LOCK_MEDIA,
        ) != 0
        {
            (*fdoData).HotplugInfo.MediaHotplug = 1; // TRUE
        } else {
            (*fdoData).HotplugInfo.MediaHotplug = 0; // FALSE
        }

        // Query the default removal policy from the kernel
        status = WdfDeviceQueryProperty(
            (*DeviceExtension).Device,
            DevicePropertyRemovalPolicy,
            core::mem::size_of::<DEVICE_REMOVAL_POLICY>() as u32,
            &raw mut deviceRemovalPolicy as *mut DEVICE_REMOVAL_POLICY as *mut core::ffi::c_void,
            &raw mut resultLength,
        );
        if NT_SUCCESS(status) {
            if resultLength != core::mem::size_of::<DEVICE_REMOVAL_POLICY>() as u32 {
                status = STATUS_UNSUCCESSFUL;
            }
        }

        if NT_SUCCESS(status) {
            if (deviceRemovalPolicy == RemovalPolicyExpectOrderlyRemoval)
                || (deviceRemovalPolicy == RemovalPolicyExpectSurpriseRemoval)
            {
                let mut userRemovalPolicy: DEVICE_REMOVAL_POLICY = 0;
                let sub = widen(CLASSP_REG_SUBKEY_NAME);
                let val = widen(CLASSP_REG_REMOVAL_POLICY_VALUE_NAME);
                DeviceGetParameter(
                    DeviceExtension as *mut core::ffi::c_void,
                    sub.as_ptr().cast_mut(),
                    val.as_ptr().cast_mut(),
                    &raw mut userRemovalPolicy as *mut DEVICE_REMOVAL_POLICY as *mut u32,
                );

                if (userRemovalPolicy == RemovalPolicyExpectOrderlyRemoval)
                    || (userRemovalPolicy == RemovalPolicyExpectSurpriseRemoval)
                {
                    deviceRemovalPolicy = userRemovalPolicy;
                }
            }

            if deviceRemovalPolicy == RemovalPolicyExpectSurpriseRemoval {
                (*fdoData).HotplugInfo.DeviceHotplug = 1; // TRUE
            } else {
                (*fdoData).HotplugInfo.DeviceHotplug = 0; // FALSE
            }

            writeCacheOverride = 0; // FALSE
            let sub = widen(CLASSP_REG_SUBKEY_NAME);
            let val = widen(CLASSP_REG_WRITE_CACHE_VALUE_NAME);
            DeviceGetParameter(
                DeviceExtension as *mut core::ffi::c_void,
                sub.as_ptr().cast_mut(),
                val.as_ptr().cast_mut(),
                &raw mut writeCacheOverride,
            );

            if writeCacheOverride != 0 {
                (*fdoData).HotplugInfo.WriteCacheEnableOverride = 1; // TRUE
            } else {
                (*fdoData).HotplugInfo.WriteCacheEnableOverride = 0; // FALSE
            }
        }

        if !NT_SUCCESS(status) {
            TracePrint!(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_INIT,
                b"Could not initialize hotplug information %lx\n\0".as_ptr() as *const i8,
                status,
            );
        }

        status
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeviceGetTimeOutValueFromRegistry() -> u32 {
    // PAGED_CODE();
    let mut status: i32;
    let mut registryKey: WDFKEY = core::ptr::null_mut();
    let mut timeOutValue: u32 = 0;
    let buff: [u16; 13] = [0x54, 0x69, 0x6D, 0x65, 0x4F, 0x75, 0x74, 0x56, 0x61, 0x6C, 0x75, 0x65, 0];
    let registryValueName = ntkrnl_sys::ddk::UNICODE_STRING {
        Length: 12 * 2,
        MaximumLength: 12 * 2,
        Buffer: core::ptr::addr_of!(buff[0]) as *mut u16,
    };
    status = WdfDriverOpenParametersRegistryKey(
        WdfGetDriver(),
        KEY_READ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(registryKey),
    );
    if NT_SUCCESS(status) {
        status = WdfRegistryQueryULong(
            registryKey,
            &registryValueName as *const ntkrnl_sys::ddk::UNICODE_STRING,
            core::ptr::addr_of_mut!(timeOutValue),
        );
        WdfRegistryClose(registryKey);
    }
    if !NT_SUCCESS(status) {
        timeOutValue = 0;
    }
    return timeOutValue;
}
#[no_mangle]
pub unsafe extern "C" fn ScanForSpecialHandler(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    HackFlags: usize, // ULONG_PTR
) {
    // PAGED_CODE() macro call
    PAGED_CODE!();

    // CLEAR_FLAG(HackFlags, CDROM_HACK_INVALID_FLAGS) – macro expanded to bitwise clear
    // Because CLEAR_FLAG is a macro and not a callable function, we manually expand it.
    // CDROM_HACK_INVALID_FLAGS is not supplied; using a placeholder value.
    const CDROM_HACK_INVALID_FLAGS: usize = 0; // TODO: define real value
    let mut flags = HackFlags;
    flags &= !CDROM_HACK_INVALID_FLAGS;

    (*DeviceExtension).DeviceAdditionalData.HackFlags = flags as ntkrnl_sys::ddk::ULONG_PTR;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceCacheDeviceInquiryData(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let cdb: *mut CDB = core::ptr::addr_of_mut!((srb).Cdb) as *mut CDB;
    let mut tmpInquiry: *mut INQUIRYDATA = core::ptr::null_mut();
    let mut requestedInquiryTransferBytes: u32 = MINIMUM_CDROM_INQUIRY_SIZE;
    let mut needResendCommand: i32 = 1; // TRUE
    let mut portDriverHack: i32 = 0; // FALSE

    // Compute minInquiryAdditionalLength: MINIMUM_CDROM_INQUIRY_SIZE - RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, AdditionalLength)
    let field_offset = core::mem::offset_of!(INQUIRYDATA, AdditionalLength) + 1; // RTL_SIZEOF_THROUGH_FIELD adds size of field (which is 1)
    let minInquiryAdditionalLength: u8 = (MINIMUM_CDROM_INQUIRY_SIZE - field_offset as u32) as u8;

    // Skip C_ASSERT and PAGED_CODE

    // Short-circuit if already cached
    if !(*DeviceExtension).DeviceAdditionalData.CachedInquiryData.is_null() {
        NT_ASSERT(((*DeviceExtension).DeviceAdditionalData.CachedInquiryDataByteCount != 0) as i32);
        return STATUS_SUCCESS;
    }

    // 1. Retrieve inquiry data length
    tmpInquiry = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        requestedInquiryTransferBytes as SIZE_T,
        CDROM_TAG_INQUIRY,
    ) as *mut INQUIRYDATA;
    if tmpInquiry.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    // 1.2 send INQUIRY command
    if status >= 0 {
        srb.CdbLength = 6;
        (*cdb).AsByte.as_mut()[0] = SCSIOP_INQUIRY;
        (*cdb).AsByte.as_mut()[3] = (requestedInquiryTransferBytes >> (8 * 1)) as u8;
        (*cdb).AsByte.as_mut()[4] = (requestedInquiryTransferBytes >> (8 * 0)) as u8;

        status = DeviceSendSrbSynchronously(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(srb),
            tmpInquiry as *mut core::ffi::c_void,
            requestedInquiryTransferBytes,
            0, // FALSE
            core::ptr::null_mut(),
        );
        CdromDbgPrint5(
            b"[INQ] send1 st=%x srbst=%x dtl=%x req=%x addlen=%x\n\0".as_ptr() as *const i8,
            status as usize,
            srb.SrbStatus as usize,
            srb.DataTransferLength as usize,
            requestedInquiryTransferBytes as usize,
            (*tmpInquiry).AdditionalLength as usize,
        );
    }

    // 1.3 get required data length
    if status >= 0 {
        let field_size = (core::mem::offset_of!(INQUIRYDATA, AdditionalLength) + 1) as u32;
        if requestedInquiryTransferBytes == srb.DataTransferLength
            && requestedInquiryTransferBytes
                == ((*tmpInquiry).AdditionalLength as u32 + field_size)
        {
            needResendCommand = 0; // FALSE
        } else {
            if (*tmpInquiry).AdditionalLength == 0 {
                (*tmpInquiry).AdditionalLength = minInquiryAdditionalLength;
                portDriverHack = 1; // TRUE
            }
            requestedInquiryTransferBytes =
                (*tmpInquiry).AdditionalLength as u32 + field_size;
            if requestedInquiryTransferBytes >= MINIMUM_CDROM_INQUIRY_SIZE {
                needResendCommand = 1;
            } else {
                needResendCommand = 0;
                status = STATUS_DEVICE_PROTOCOL_ERROR;
            }
        }
    }

    // 2. retrieve the inquiry data if still needed
    if status >= 0 && needResendCommand != 0 {
        FREE_POOL(tmpInquiry as *mut core::ffi::c_void);
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(srb) as *mut u8,
            0,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        );
        tmpInquiry = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            requestedInquiryTransferBytes as SIZE_T,
            CDROM_TAG_INQUIRY,
        ) as *mut INQUIRYDATA;
        if tmpInquiry.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    // 2.2 resend INQUIRY command
    if status >= 0 && needResendCommand != 0 {
        srb.CdbLength = 6;
        (*cdb).AsByte.as_mut()[0] = SCSIOP_INQUIRY;
        (*cdb).AsByte.as_mut()[3] = (requestedInquiryTransferBytes >> (8 * 1)) as u8;
        (*cdb).AsByte.as_mut()[4] = (requestedInquiryTransferBytes >> (8 * 0)) as u8;

        status = DeviceSendSrbSynchronously(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(srb),
            tmpInquiry as *mut core::ffi::c_void,
            requestedInquiryTransferBytes,
            0,
            core::ptr::null_mut(),
        );

        if !(status >= 0) {
            // Workaround for drives that report more data than they have
            if srb.SrbStatus == SRB_STATUS_DATA_OVERRUN
                && srb.DataTransferLength < requestedInquiryTransferBytes
                && srb.DataTransferLength >= MINIMUM_CDROM_INQUIRY_SIZE
            {
                requestedInquiryTransferBytes = srb.DataTransferLength;
                FREE_POOL(tmpInquiry as *mut core::ffi::c_void);
                core::ptr::write_bytes(
                    core::ptr::addr_of_mut!(srb) as *mut u8,
                    0,
                    core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
                );
                tmpInquiry = ExAllocatePoolWithTag(
                    NonPagedPoolNx,
                    requestedInquiryTransferBytes as SIZE_T,
                    CDROM_TAG_INQUIRY,
                ) as *mut INQUIRYDATA;
                if tmpInquiry.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    srb.CdbLength = 6;
                    (*cdb).AsByte.as_mut()[0] = SCSIOP_INQUIRY;
                    (*cdb).AsByte.as_mut()[3] = (requestedInquiryTransferBytes >> (8 * 1)) as u8;
                    (*cdb).AsByte.as_mut()[4] = (requestedInquiryTransferBytes >> (8 * 0)) as u8;

                    status = DeviceSendSrbSynchronously(
                        (*DeviceExtension).Device as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(srb),
                        tmpInquiry as *mut core::ffi::c_void,
                        requestedInquiryTransferBytes,
                        0,
                        core::ptr::null_mut(),
                    );
                }
            }
        }

        // Check transferred data length
        if status >= 0 {
            requestedInquiryTransferBytes = srb.DataTransferLength;
            if requestedInquiryTransferBytes < MINIMUM_CDROM_INQUIRY_SIZE {
                status = STATUS_DEVICE_PROTOCOL_ERROR;
            }
        }

        // Ensure we got non-zero data when port driver hack was applied
        if status >= 0 && portDriverHack != 0 {
            let mut tmp: *mut u32 = tmpInquiry as *mut u32;
            let mut i: u32 = MINIMUM_CDROM_INQUIRY_SIZE / core::mem::size_of::<u32>() as u32;
            loop {
                if i == 0 {
                    break;
                }
                if *tmp != 0 {
                    break;
                }
                i -= 1;
                tmp = tmp.add(1);
            }
            if i == 0 {
                status = STATUS_DEVICE_PROTOCOL_ERROR;
            }
        }
    }

    // Final assignment
    if status >= 0 {
        (*DeviceExtension).DeviceAdditionalData.CachedInquiryData = tmpInquiry;
        (*DeviceExtension).DeviceAdditionalData.CachedInquiryDataByteCount =
            requestedInquiryTransferBytes;
    } else {
        FREE_POOL(tmpInquiry as *mut core::ffi::c_void);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSetRawReadInfo(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut buffer: *mut u8 = core::ptr::null_mut();
    let mut count: u32 = 0;

    // PAGED_CODE();
    unsafe { PAGED_CODE!(); } // keep the macro call

    // Check GET_CONFIGURATION data
    {
        let mut cd_read_header: *mut FEATURE_DATA_CD_READ = core::ptr::null_mut();
        let additional_length: u32 =
            core::mem::size_of::<FEATURE_DATA_CD_READ>() as u32 - core::mem::size_of::<FEATURE_HEADER>() as u32;

        cd_read_header = DeviceFindFeaturePage(
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *mut u8,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
            FeatureCdRead,
        );

        if !cd_read_header.is_null()
            && u32::from((*cd_read_header).Header.AdditionalLength) >= additional_length
            && (*cd_read_header).C2ErrorData() != 0
        {
            TracePrint!(
                4u32, // TRACE_LEVEL_INFORMATION
                2u32, // TRACE_FLAG_IOCTL
                b"DeviceSetRawReadInfo: FDO %p GET_CONFIG shows ability to read C2 error bits\n\0".as_ptr()
                    as *const i8,
                (*DeviceExtension).DeviceObject,
            );
            (*DeviceExtension).DeviceAdditionalData.Mmc.ReadCdC2Pointers = 1; // TRUE
        }
    }

    // Loop trying mode sense
    count = 0;
    while count < 6 {
        let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
        let cdb: *mut CDB = &mut srb.Cdb as *mut _ as *mut CDB;
        let buffer_length: u32;

        if (count / 3) == 0 {
            // 10-byte CDB
            buffer_length = core::mem::size_of::<CDVD_CAPABILITIES_PAGE>() as u32
                + core::mem::size_of::<MODE_PARAMETER_HEADER10>() as u32
                + core::mem::size_of::<MODE_PARAMETER_BLOCK>() as u32;

            (*cdb).MODE_SENSE10.as_mut().OperationCode = SCSIOP_MODE_SENSE10;
            (*cdb).MODE_SENSE10.as_mut().set_Dbd(Dbd);
            (*cdb).MODE_SENSE10.as_mut().set_PageCode(MODE_PAGE_CAPABILITIES);
            (*cdb).MODE_SENSE10.as_mut().AllocationLength[0] = (buffer_length >> 8) as u8;
            (*cdb).MODE_SENSE10.as_mut().AllocationLength[1] = (buffer_length >> 0) as u8;
            srb.CdbLength = 10;
        } else {
            // 6-byte CDB
            buffer_length = core::mem::size_of::<CDVD_CAPABILITIES_PAGE>() as u32
                + core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32
                + core::mem::size_of::<MODE_PARAMETER_BLOCK>() as u32;

            (*cdb).MODE_SENSE.as_mut().OperationCode = SCSIOP_MODE_SENSE;
            (*cdb).MODE_SENSE.as_mut().set_Dbd(Dbd);
            (*cdb).MODE_SENSE.as_mut().set_PageCode(MODE_PAGE_CAPABILITIES);
            (*cdb).MODE_SENSE.as_mut().AllocationLength = buffer_length as u8;
            srb.CdbLength = 6;
        }

        srb.TimeOutValue = (*DeviceExtension).TimeOutValue;

        buffer = ExAllocatePoolWithTag(NonPagedPoolNx, buffer_length as SIZE_T, CDROM_TAG_MODE_DATA) as *mut u8;

        if buffer.is_null() {
            TracePrint!(
                2u32, // TRACE_LEVEL_ERROR
                2u32, // TRACE_FLAG_IOCTL
                b"DeviceSetRawReadInfo: cannot allocate buffer, so not setting raw read info for FDO %p\n\0".as_ptr()
                    as *const i8,
                (*DeviceExtension).DeviceObject,
            );
            status = STATUS_INSUFFICIENT_RESOURCES;
            // goto FnExit;
            // We'll handle via break and cleanup at end.
            // The loop body currently ends with FREE_POOL after warning; we need to go to FnExit.
            // So we'll exit the loop and let normal cleanup happen.
            break;
        }

        core::ptr::write_bytes((buffer) as *mut u8, 0, buffer_length as usize);

        status = DeviceSendSrbSynchronously(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            &mut srb,
            buffer as *mut core::ffi::c_void,
            buffer_length,
            0u8, // FALSE
            core::ptr::null_mut(),
        );

        // Check success
        if status >= 0 || status == STATUS_DATA_OVERRUN || status == STATUS_BUFFER_OVERFLOW {
            let capabilities: *mut CDVD_CAPABILITIES_PAGE;

            if (count / 3) == 0 {
                let p: *mut MODE_PARAMETER_HEADER10 = buffer as *mut MODE_PARAMETER_HEADER10;
                let block_desc_len: u32 =
                    ((*p).BlockDescriptorLength[0] as u32) * 256u32 + ((*p).BlockDescriptorLength[1] as u32);
                capabilities = buffer
                    .add(
                        core::mem::size_of::<MODE_PARAMETER_HEADER10>()
                            + block_desc_len as usize,
                    )
                    as *mut CDVD_CAPABILITIES_PAGE;
            } else {
                let p: *mut MODE_PARAMETER_HEADER = buffer as *mut MODE_PARAMETER_HEADER;
                capabilities = buffer
                    .add(
                        core::mem::size_of::<MODE_PARAMETER_HEADER>()
                            + (*p).BlockDescriptorLength as usize,
                    )
                    as *mut CDVD_CAPABILITIES_PAGE;
            }

            TracePrint!(
                4u32, // TRACE_LEVEL_INFORMATION
                2u32, // TRACE_FLAG_IOCTL
                b"DeviceSetRawReadInfo: FDO %p CDVD Capabilities buffer %p\n\0".as_ptr() as *const i8,
                (*DeviceExtension).DeviceObject,
                buffer,
            );

            if (*capabilities).PageCode() == MODE_PAGE_CAPABILITIES {
                if (*capabilities).C2Pointers() != 0 {
                    TracePrint!(
                        1u32,
                        2u32,
                        b"DeviceSetRawReadInfo: FDO %p supports C2 error bits in READ_CD command\n\0".as_ptr()
                            as *const i8,
                        (*DeviceExtension).DeviceObject,
                    );
                    (*DeviceExtension).DeviceAdditionalData.Mmc.ReadCdC2Pointers = 1;
                }
                if (*capabilities).RWSupported() != 0 {
                    TracePrint!(
                        1u32,
                        2u32,
                        b"DeviceSetRawReadInfo: FDO %p supports raw subcode in READ_CD command\n\0".as_ptr()
                            as *const i8,
                        (*DeviceExtension).DeviceObject,
                    );
                    (*DeviceExtension).DeviceAdditionalData.Mmc.ReadCdSubCode = 1;
                }
                // Successfully got the page; break out of the loop. Do NOT free buffer
                // here: the C success branch is a bare `break` (init.c:2537) and the FnExit
                // cleanup below frees it exactly once. An earlier translation added a
                // FREE_POOL here; because the Rust FREE_POOL is a fn and cannot null the
                // caller's local the way the C macro does (ExFreePool(p); p = NULL), that
                // left `buffer` dangling and made FnExit free it again -> BUGCHECK 0x19.
                break;
            }
        }

        // After the attempt (regardless of success/fail if we haven't broken),
        // trace warning and free buffer.
        TracePrint!(
            3u32, // TRACE_LEVEL_WARNING
            2u32, // TRACE_FLAG_IOCTL
            b"DeviceSetRawReadInfo: FDO %p failed %x byte mode sense, status %x\n\0".as_ptr() as *const i8,
            (*DeviceExtension).DeviceObject,
            if (count / 3) == 0 { 10u32 } else { 6u32 },
            status,
        );

        FREE_POOL(buffer as *mut core::ffi::c_void);
        // Null after freeing, matching the C FREE_POOL macro (ExFreePool(p); p = NULL).
        // The Rust FREE_POOL is a by-value fn and cannot null the caller's local, so
        // without this the FnExit free below double-frees on the all-fail path.
        buffer = core::ptr::null_mut();

        count += 1;
    }

    // After loop: if count == 6 (tried all and never broke)
    // Note: In Rust we are using while loop; after loop, count might be 6 if loop completed without break.
    // But we might have broken early due to success; count would be the index at the time of break (still <6), but we already set flags and broken.
    // Actually count increments at end of each iteration, so if we break before increment, count will be less than 6. So check count == 6 after loop.
    // However, we need to ensure accurate count after break. In our loop, we have `break` before increment, so count remains the value it had at the iteration start.
    // That's fine.
    // Now we go to FnExit logic.
    // FnExit label: if (buffer) FREE_POOL(buffer); return status.
    // Note: at this point buffer may hold a pointer if we exited due to allocation failure (where we broke after 'buffer = ...; ... break'). In that case buffer is non-null.
    // Or if we got to FnExit from the success branch break, buffer was freed and set to null? No, we didn't null it. So we need to null it after freeing.
    // We'll mimic the C behavior: at FnExit, they check `if (buffer) FREE_POOL(buffer);`. So we'll do that, but we must ensure buffer is null after we free. We'll free and then null it.
    // For clarity, we'll do:
    // FnExit:
    if !buffer.is_null() {
        FREE_POOL(buffer as *mut core::ffi::c_void);
    }

    if count == 6 {
        TracePrint!(
            3u32, // TRACE_LEVEL_WARNING
            2u32, // TRACE_FLAG_IOCTL
            b"DeviceSetRawReadInfo: FDO %p couldn't get mode sense data\n\0".as_ptr() as *const i8,
            (*DeviceExtension).DeviceObject,
        );
    }

    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceInitializeDvd(Device: WDFDEVICE) -> NTSTATUS {
    // PAGED_CODE();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let deviceExtension: *mut CDROM_DEVICE_EXTENSION;
    let mut copyProtectKey: *mut DVD_COPY_PROTECT_KEY = core::ptr::null_mut();
    let rpcKey: *mut DVD_RPC_KEY;
    let bufferLen: u32;
    let mut bytesReturned: usize = 0;

    deviceExtension = DeviceGetExtension(Device);

    if (*deviceExtension).DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD {
        return STATUS_SUCCESS;
    }

    bufferLen = DVD_RPC_KEY_LENGTH;
    copyProtectKey = ExAllocatePoolWithTag(PagedPool, bufferLen as SIZE_T, DVD_TAG_RPC2_CHECK) as *mut DVD_COPY_PROTECT_KEY;

    if copyProtectKey.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(copyProtectKey as *mut core::ffi::c_void, bufferLen as usize);
    (*copyProtectKey).KeyLength = DVD_RPC_KEY_LENGTH;
    (*copyProtectKey).KeyType = DvdGetRpcKey;

    status = DvdStartSessionReadKey(
        deviceExtension,
        IOCTL_DVD_READ_KEY,
        core::ptr::null_mut(),
        copyProtectKey as *mut core::ffi::c_void,
        DVD_RPC_KEY_LENGTH as usize,
        copyProtectKey as *mut core::ffi::c_void,
        DVD_RPC_KEY_LENGTH as usize,
        core::ptr::addr_of_mut!(bytesReturned),
    );

    if NT_SUCCESS(status) {
        rpcKey = (*copyProtectKey).KeyData.as_ptr() as *mut DVD_RPC_KEY;

        if (*rpcKey).TypeCode() == 0 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_PNP,
                b"DVD Initialize (%p): must choose DVD region\n\0".as_ptr() as *const i8,
                Device as *mut core::ffi::c_void,
            );
            (*deviceExtension).DeviceAdditionalData.PickDvdRegion = 1;

            if (*deviceExtension).DeviceAdditionalData.Mmc.IsCssDvd != 0 {
                DevicePickDvdRegion(Device);
            }
        }
    }

    ExFreePool(copyProtectKey as *mut core::ffi::c_void);

    STATUS_SUCCESS
}

const DEVPROP_TYPE_EMPTY: u32 = 0;


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
//     init.c
//
// Abstract:
//
//     Initialization routines for CDROM
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
// #include "devpkey.h"
//
// #include "cdrom.h"
// #include "scratch.h"
// #include "mmc.h"
//
// #ifdef DEBUG_USE_WPP
// #include "init.tmh"
// #endif
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceInitAllocateBuffers(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceRetrieveScsiAddress(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PSCSI_ADDRESS           ScsiAddress
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceRetrieveDescriptorsAndTransferLength(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceScanSpecialDevices(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitMmcContext(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceGetMmcSupportInfo(
//       PCDROM_DEVICE_EXTENSION   DeviceExtension,
//      PBOOLEAN                  IsMmcDevice
//     );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceIsPortable(
//       PCDROM_DEVICE_EXTENSION   DeviceExtension,
//      PBOOLEAN                  IsPortable
//     );
// #endif
//
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceClaimRelease)
// #pragma alloc_text(PAGE, DeviceEvtSelfManagedIoInit)
//
// #pragma alloc_text(PAGE, DeviceInitReleaseQueueContext)
// #pragma alloc_text(PAGE, DeviceInitAllocateBuffers)
// #pragma alloc_text(PAGE, DeviceInitPowerContext)
// #pragma alloc_text(PAGE, DeviceCreateWellKnownName)
// #pragma alloc_text(PAGE, DeviceRetrieveScsiAddress)
// #pragma alloc_text(PAGE, DeviceRetrieveDescriptorsAndTransferLength)
// #pragma alloc_text(PAGE, DeviceInitializeHotplugInfo)
// #pragma alloc_text(PAGE, DeviceScanSpecialDevices)
// #pragma alloc_text(PAGE, DeviceGetTimeOutValueFromRegistry)
// #pragma alloc_text(PAGE, DeviceGetMmcSupportInfo)
// #pragma alloc_text(PAGE, DeviceRetrieveDescriptor)
// #pragma alloc_text(PAGE, DeviceRetrieveHackFlagsFromRegistry)
// #pragma alloc_text(PAGE, DeviceScanForSpecial)
// #pragma alloc_text(PAGE, DeviceHackFlagsScan)
// #pragma alloc_text(PAGE, DeviceInitMmcContext)
// #pragma alloc_text(PAGE, ScanForSpecialHandler)
// #pragma alloc_text(PAGE, DeviceSetRawReadInfo)
// #pragma alloc_text(PAGE, DeviceInitializeDvd)
// #pragma alloc_text(PAGE, DeviceCacheDeviceInquiryData)
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
// #pragma alloc_text(PAGE, DeviceIsPortable)
// #endif
//
// #endif
//
// #pragma warning(push)
// #pragma warning(disable:4152) // nonstandard extension, function/data pointer conversion in expression
// #pragma warning(disable:26000) // read overflow reported because of pointer type conversion
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceClaimRelease(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      BOOLEAN                 Release
//     )
// /*++
//
// Routine Description:
//
//     This function claims a device in the port driver.  The port driver object
//     is updated with the correct driver object if the device is successfully
//     claimed.
//
// Arguments:
//
//     Device - The WDFDEVICE that needs to be claimed or released.
//
//     Release - Indicates the logical unit should be released rather than claimed.
//
// Return Value:
//
//     Returns a status indicating success or failure of the operation.
//
// --*/
// {
//     NTSTATUS                status;
//     SCSI_REQUEST_BLOCK      srb = {0};
//     WDF_MEMORY_DESCRIPTOR   descriptor;
//     WDFREQUEST              request;
//     WDF_OBJECT_ATTRIBUTES   attributes;
//
//     PAGED_CODE();
//
//     //Create a request
//     WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                             CDROM_REQUEST_CONTEXT);
//
//     status = WdfRequestCreate(&attributes,
//                               DeviceExtension->IoTarget,
//                               &request);
//
//     if (NT_SUCCESS(status))
//     {
//         //fill up srb structure
//         srb.OriginalRequest = WdfRequestWdmGetIrp(request);
//         NT_ASSERT(srb.OriginalRequest != NULL);
//
//         srb.Length = sizeof(SCSI_REQUEST_BLOCK);
//
//         srb.Function = Release
//                         ? SRB_FUNCTION_RELEASE_DEVICE
//                         : SRB_FUNCTION_CLAIM_DEVICE;
//
//
//         WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(&descriptor,
//                                           &srb,
//                                           sizeof(srb));
//
//         status = WdfIoTargetSendInternalIoctlOthersSynchronously(DeviceExtension->IoTarget,
//                                                                  request,
//                                                                  IOCTL_SCSI_EXECUTE_NONE,
//                                                                  &descriptor,
//                                                                  NULL,
//                                                                  NULL,
//                                                                  NULL,
//                                                                  NULL);
//
//         NT_ASSERT(!TEST_FLAG(srb.SrbFlags, SRB_FLAGS_FREE_SENSE_BUFFER));
//
//         // The request should be deleted.
//         WdfObjectDelete(request);
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                         "DeviceClaimRelease: Failed to %s device, status: 0x%X\n",
//                         Release ? "Release" : "Claim",
//                         status));
//         }
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_FATAL, TRACE_FLAG_PNP,
//                     "DeviceClaimRelease: Failed to create request, status: 0x%X\n",
//                     status));
//     }
//
//     if (Release)
//     {
//         // We only release the device when we don't want to manage it.
//         // The failure status does not matter.
//         status = STATUS_SUCCESS;
//     }
//
//     return status;
// } // end DeviceClaimRelease()
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceEvtSelfManagedIoInit(
//      WDFDEVICE      Device
//     )
// /*++
//
// Routine Description:
//
//     This routine is called only once after the device is added in system, so it's used to do
//     hardware-dependent device initialization work and resource allocation.
//     If this routine fails, DeviceEvtSelfManagedIoCleanup will be invoked by the framework.
//
// Arguments:
//
//     Device - Handle to device object
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION deviceExtension = NULL;
//
//     PAGED_CODE();
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP,
//                 "DeviceEvtSelfManagedIoInit: WDFDEVICE %p is being started.\n",
//                 Device));
//
//     deviceExtension = DeviceGetExtension(Device);
//
//     // 1. Set/retrieve basic information, some of the following operations may depend on it
//     if (NT_SUCCESS(status))
//     {
//         // We do not care if this function fails, SCSI address is mainly for debugging/tracing purposes.
//         (VOID) DeviceRetrieveScsiAddress(deviceExtension, &deviceExtension->ScsiAddress);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceRetrieveDescriptorsAndTransferLength(deviceExtension);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // This function should be called after DeviceRetrieveDescriptorsAndTransferLength()
//         // It depends on MaxTransferLenth fields.
//         status = DeviceInitAllocateBuffers(deviceExtension);
//     }
//
//     // 2. The following functions depend on the allocated buffers.
//
//     // perf re-enable after failing. Q: Is this one used by cdrom.sys?
//     if (NT_SUCCESS(status))
//     {
//         // allow perf to be re-enabled after a given number of failed IOs
//         // require this number to be at least CLASS_PERF_RESTORE_MINIMUM
//         ULONG t = CLASS_PERF_RESTORE_MINIMUM;
//
//         DeviceGetParameter(deviceExtension,
//                            CLASSP_REG_SUBKEY_NAME,
//                            CLASSP_REG_PERF_RESTORE_VALUE_NAME,
//                            &t);
//         if (t >= CLASS_PERF_RESTORE_MINIMUM)
//         {
//             deviceExtension->PrivateFdoData->Perf.ReEnableThreshhold = t;
//         }
//     }
//
//     // 3. Retrieve information about special devices and hack flags.
//     if (NT_SUCCESS(status))
//     {
//         DeviceRetrieveHackFlagsFromRegistry(deviceExtension);
//         // scan for bad items.
//         DeviceScanForSpecial(deviceExtension, CdRomBadItems, DeviceHackFlagsScan);
//         // Check to see if it's a special device that needs special error process.
//         DeviceScanSpecialDevices(deviceExtension);  // may send command to device
//     }
//
//     // 4. Initialize the hotplug information only after the ScanForSpecial routines,
//     // as it relies upon the hack flags - deviceExtension->PrivateFdoData->HackFlags.
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitializeHotplugInfo(deviceExtension);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // cache the device's inquiry data
//         status = DeviceCacheDeviceInquiryData(deviceExtension);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "Failed to cache the device's inquiry data, failng %!STATUS!\n",
//                         status
//                         ));
//         }
//     }
//
//     // 5. Initialize MMC context, media change notification stuff and read media capacity
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitializeMediaChangeDetection(deviceExtension);
//     }
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitMmcContext(deviceExtension);
//     }
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitializeZPODD(deviceExtension);
//     }
//     if (NT_SUCCESS(status))
//     {
//         // Do READ CAPACITY. This SCSI command returns the last sector address
//         // on the device and the bytes per sector. These are used to calculate
//         // the drive capacity in bytes.
//         status = MediaReadCapacity(Device);
//
//         // If READ CAPACITY succeeded, we can safely conclude that there is a media present
//         if (NT_SUCCESS(status))
//         {
//             DeviceSetMediaChangeStateEx(deviceExtension,
//                                         MediaPresent,
//                                         NULL);
//         }
//
//         // READ CAPACITY is not critical for init, ignore all errors occuring during its execution
//         status = STATUS_SUCCESS;
//     }
//
//     // 6. Perform DVD-specific initialization
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceInitializeDvd(Device);
//     }
//
//     // 7. Miscellaneous initialization actions
//     if (NT_SUCCESS(status))
//     {
//         if (deviceExtension->PrivateFdoData != NULL)
//         {
//             deviceExtension->PrivateFdoData->Perf.OriginalSrbFlags = deviceExtension->SrbFlags;
//         }
//
//         if (deviceExtension->DeviceAdditionalData.Mmc.IsWriter)
//         {
//             // OPC can really take this long per IMAPIv1 timeout....
//             deviceExtension->TimeOutValue = max(deviceExtension->TimeOutValue, SCSI_CDROM_OPC_TIMEOUT);
//         }
//     }
//
//     // 8. Enable the main timer, create ARC name as needed
//     if (NT_SUCCESS(status))
//     {
//         // Device successfully added and initialized, increase CdRomCount.
//         IoGetConfigurationInformation()->CdRomCount++;
//
//         deviceExtension->IsInitialized = TRUE;
//
//         DeviceEnableMainTimer(deviceExtension);
//
//     }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     // 9. Set volume interface properties
//     if (NT_SUCCESS(status))
//     {
//         BOOLEAN isCritical = FALSE;
//         BOOLEAN isPortable = FALSE;
//         BOOLEAN isRemovable = TEST_FLAG(deviceExtension->DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA);
//         DEVPROP_BOOLEAN propCritical = DEVPROP_FALSE;
//         DEVPROP_BOOLEAN propPortable = DEVPROP_FALSE;
//         DEVPROP_BOOLEAN propRemovable = DEVPROP_FALSE;
//
//         status = DeviceIsPortable(deviceExtension, &isPortable);
//
//         if (NT_SUCCESS(status))
//         {
//             if (isPortable) {
//                 SET_FLAG(deviceExtension->DeviceObject->Characteristics, FILE_PORTABLE_DEVICE);
//             }
//
//             propPortable = isPortable ? DEVPROP_TRUE : DEVPROP_FALSE;
//
//             status = IoSetDeviceInterfacePropertyData(&deviceExtension->MountedDeviceInterfaceName,
//                                                       &DEVPKEY_Storage_Portable,
//                                                       0,
//                                                       0,
//                                                       DEVPROP_TYPE_BOOLEAN,
//                                                       sizeof(DEVPROP_BOOLEAN),
//                                                       &propPortable);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             propRemovable = isRemovable ? DEVPROP_TRUE : DEVPROP_FALSE;
//
//             status = IoSetDeviceInterfacePropertyData(&deviceExtension->MountedDeviceInterfaceName,
//                                                       &DEVPKEY_Storage_Removable_Media,
//                                                       0,
//                                                       0,
//                                                       DEVPROP_TYPE_BOOLEAN,
//                                                       sizeof(DEVPROP_BOOLEAN),
//                                                       &propRemovable);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             isCritical = TEST_FLAG(deviceExtension->DeviceObject->Flags,
//                                    (DO_SYSTEM_SYSTEM_PARTITION |
//                                     DO_SYSTEM_BOOT_PARTITION   |
//                                     DO_SYSTEM_CRITICAL_PARTITION));
//
//             propCritical = isCritical ? DEVPROP_TRUE : DEVPROP_FALSE;
//
//             status = IoSetDeviceInterfacePropertyData(&deviceExtension->MountedDeviceInterfaceName,
//                                                       &DEVPKEY_Storage_System_Critical,
//                                                       0,
//                                                       0,
//                                                       DEVPROP_TYPE_BOOLEAN,
//                                                       sizeof(DEVPROP_BOOLEAN),
//                                                       &propCritical);
//         }
//
//     }
// #endif
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceInitReleaseQueueContext(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     Part of device initialize routine. Initialize ReleaseQueue related stuff.
//
// Arguments:
//
//     DeviceExtension - device extension of WDFDEVICE.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     WDF_OBJECT_ATTRIBUTES   attributes;
//
//     PAGED_CODE();
//
//     WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                             CDROM_REQUEST_CONTEXT);
//     attributes.ParentObject = DeviceExtension->Device;
//
//     status = WdfRequestCreate(&attributes,
//                               DeviceExtension->IoTarget,
//                               &(DeviceExtension->ReleaseQueueRequest));
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP,  "Cannot create the release queue request\n"));
//
//         return status;
//     }
//
//     // Initialize ReleaseQueueInputMemory, a wrapper around ReleaseQueueSrb
//     WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//     attributes.ParentObject = DeviceExtension->ReleaseQueueRequest;
//
//     status = WdfMemoryCreatePreallocated(&attributes,
//                                          &DeviceExtension->ReleaseQueueSrb,
//                                          sizeof(SCSI_REQUEST_BLOCK),
//                                          &DeviceExtension->ReleaseQueueInputMemory);
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP,  "Failed to allocate ReleaseQueueSrb.\n"));
//
//         return status;
//     }
//
//     // Preformat the release queue request here to ensure that this call will never
//     // fail during an actual release of the queue.
//     if (NT_SUCCESS(status))
//     {
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(DeviceExtension->IoTarget,
//                                                                 DeviceExtension->ReleaseQueueRequest,
//                                                                 IOCTL_SCSI_EXECUTE_NONE,
//                                                                 DeviceExtension->ReleaseQueueInputMemory,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL,
//                                                                 NULL);
//     }
//
//     // Set a CompletionRoutine callback function for ReleaseQueueRequest.
//     if (NT_SUCCESS(status))
//     {
//         WdfRequestSetCompletionRoutine(DeviceExtension->ReleaseQueueRequest,
//                                        DeviceReleaseQueueCompletion,
//                                        DeviceExtension->Device);
//     }
//
//     // Create a spinlock for ReleaseQueueRequest
//     WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//     attributes.ParentObject = DeviceExtension->Device;
//
//     status = WdfSpinLockCreate(&attributes,
//                                &(DeviceExtension->ReleaseQueueSpinLock));
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP,
//                     "DeviceInitReleaseQueueContext: Cannot create the release queue spinlock\n"));
//
//         return status;
//     }
//
//     // Initialize miscellaneous ReleaseQueue related fields
//     DeviceExtension->ReleaseQueueNeeded = FALSE;
//     DeviceExtension->ReleaseQueueInProgress = FALSE;
//     DeviceExtension->ReleaseQueueSrb.Length = sizeof(SCSI_REQUEST_BLOCK);
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceInitPowerContext(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     Part of device initialize routine. Initialize PowerContext related stuff.
//
// Arguments:
//
//     DeviceExtension - device extension of WDFDEVICE.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     WDF_OBJECT_ATTRIBUTES   attributes;
//
//     PAGED_CODE();
//
//     // create request object for Power operations
//
//     WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                             CDROM_REQUEST_CONTEXT);
//     attributes.ParentObject = DeviceExtension->Device;
//
//     status = WdfRequestCreate(&attributes,
//                               DeviceExtension->IoTarget,
//                               &(DeviceExtension->PowerContext.PowerRequest) );
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP,  "Cannot create the power request object.\n"));
//
//         return status;
//     }
//
//     // Preformat the power request. With this being done, we never need to worry about
//     // WdfIoTargetFormatRequestForInternalIoctlOthers ever failing later.
//     status = WdfIoTargetFormatRequestForInternalIoctlOthers(DeviceExtension->IoTarget,
//                                                             DeviceExtension->PowerContext.PowerRequest,
//                                                             IOCTL_SCSI_EXECUTE_IN,
//                                                             NULL, NULL,
//                                                             NULL, NULL,
//                                                             NULL, NULL);
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceCreateWellKnownName(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine creates a symbolic link to the cdrom device object
//     under \dosdevices.  The number of the cdrom device does not neccessarily
//     match between \dosdevices and \device, but usually will be the same.
//
//     Saves the buffer
//
// Arguments:
//
//     DeviceObject -
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//     UNICODE_STRING  unicodeLinkName = {0};
//     WCHAR           wideLinkName[64] = {0};
//     PWCHAR          savedName;
//
//     LONG            cdromNumber = DeviceExtension->DeviceNumber;
//
//     PAGED_CODE();
//
//     // if already linked, assert then return
//     if (DeviceExtension->DeviceAdditionalData.WellKnownName.Buffer != NULL)
//     {
//
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
//                     "DeviceCreateWellKnownName: link already exists %p\n",
//                     DeviceExtension->DeviceAdditionalData.WellKnownName.Buffer));
//
//         NT_ASSERT(FALSE);
//
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     // find an unused CdRomNN to link to.
//     // It's doing this way because the same might be used for other device in another driver.
//     do
//     {
//         status = RtlStringCchPrintfW((NTSTRSAFE_PWSTR)wideLinkName,
//                                      RTL_NUMBER_OF(wideLinkName),
//                                      L"\\DosDevices\\CdRom%d",
//                                      cdromNumber);
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
//                         "DeviceCreateWellKnownName: Format symbolic link failed with error: 0x%X\n", status));
//             return status;
//         }
//
//         RtlInitUnicodeString(&unicodeLinkName, wideLinkName);
//
//         status = WdfDeviceCreateSymbolicLink(DeviceExtension->Device,
//                                              &unicodeLinkName);
//
//         cdromNumber++;
//
//     } while((status == STATUS_OBJECT_NAME_COLLISION) ||
//             (status == STATUS_OBJECT_NAME_EXISTS));
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,
//                     "DeviceCreateWellKnownName: Error %lx linking %wZ to "
//                     "device %wZ\n",
//                     status,
//                     &unicodeLinkName,
//                     &(DeviceExtension->DeviceName)));
//         return status;
//     }
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                 "DeviceCreateWellKnownName: successfully linked %wZ "
//                 "to device %wZ\n",
//                 &unicodeLinkName,
//                 &(DeviceExtension->DeviceName)));
//
//     // Save away the symbolic link name in the driver data block.  We need
//     // it so we can delete the link when the device is removed.
//     savedName = ExAllocatePoolWithTag(PagedPool,
//                                       unicodeLinkName.MaximumLength,
//                                       CDROM_TAG_STRINGS);
//
//     if (savedName == NULL)
//     {
//         // Test Note: test path should excise here to see if the symbolic is deleted by framework.
//         // IoDeleteSymbolicLink(&unicodeLinkName);
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                     "DeviceCreateWellKnownName: unable to allocate memory.\n"));
//
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(savedName, unicodeLinkName.MaximumLength);
//     RtlCopyMemory(savedName, unicodeLinkName.Buffer, unicodeLinkName.MaximumLength);
//
//     RtlInitUnicodeString(&(DeviceExtension->DeviceAdditionalData.WellKnownName), savedName);
//
//     // the name was saved and the link created
//
//     return STATUS_SUCCESS;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceRetrieveScsiAddress(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PSCSI_ADDRESS           ScsiAddress
//     )
// /*++
//
// Routine Description:
//
//     retrieve SCSI address information and put into device extension
//
// Arguments:
//
//     DeviceExtension - device context.
//     ScsiAddress - the buffer to put the scsi address info.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status;
//     WDF_MEMORY_DESCRIPTOR   outputDescriptor;
//
//     PAGED_CODE();
//
//     if ((DeviceExtension == NULL) ||
//         (ScsiAddress == NULL))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //Get IOTARGET for sending request to port driver.
//     WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(&outputDescriptor,
//                                       (PVOID)ScsiAddress,
//                                       sizeof(SCSI_ADDRESS));
//
//     status = WdfIoTargetSendIoctlSynchronously(DeviceExtension->IoTarget,
//                                                NULL,
//                                                IOCTL_SCSI_GET_ADDRESS,
//                                                NULL,
//                                                &outputDescriptor,
//                                                NULL,
//                                                NULL);
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                     "DeviceRetrieveScsiAddress: Get Address failed %lx\n",
//                     status));
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                     "GetAddress: Port %x, Path %x, Target %x, Lun %x\n",
//                     ScsiAddress->PortNumber,
//                     ScsiAddress->PathId,
//                     ScsiAddress->TargetId,
//                     ScsiAddress->Lun));
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceInitAllocateBuffers(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     Part of device initialize routine.
//     Allocate all buffers in Device Extension.
//
// Arguments:
//
//     DeviceExtension - device extension of WDFDEVICE.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     PVOID               senseData = NULL;
//
//     PAGED_CODE();
//
//     // allocate a private extension for class data
//     if (DeviceExtension->PrivateFdoData == NULL)
//     {
//         DeviceExtension->PrivateFdoData = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                                 sizeof(CDROM_PRIVATE_FDO_DATA),
//                                                                 CDROM_TAG_PRIVATE_DATA);
//     }
//
//     if (DeviceExtension->PrivateFdoData == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         // initialize the struct's various fields.
//         RtlZeroMemory(DeviceExtension->PrivateFdoData, sizeof(CDROM_PRIVATE_FDO_DATA));
//     }
//
//     // Allocate request sense buffer.
//     senseData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                       SENSE_BUFFER_SIZE,
//                                       CDROM_TAG_SENSE_INFO);
//
//     if (senseData == NULL)
//     {
//         // The buffer cannot be allocated.
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         // Set the sense data pointer in the device extension.
//         DeviceExtension->SenseData = senseData;
//     }
//
//     // Allocate scratch buffer -- Must occur after determining
//     // max transfer size, but before other CD specific detection
//     // (which relies upon this buffer).
//     if (!ScratchBuffer_Allocate(DeviceExtension))
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                     "Failed to allocate scratch buffer, failing  %!STATUS!\n",
//                     status
//                     ));
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceRetrieveDescriptorsAndTransferLength(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     Part of device initialize routine.
//     Retrieve Device Descriptor and Adaptor Descriptor.
//
// Arguments:
//
//     DeviceExtension - device extension of WDFDEVICE.
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     STORAGE_PROPERTY_ID propertyId;
//
//     PAGED_CODE();
//
//     if (NT_SUCCESS(status))
//     {
//         // Call port driver to get adapter capabilities.
//         propertyId = StorageAdapterProperty;
//
//         status = DeviceRetrieveDescriptor(DeviceExtension->Device,
//                                           &propertyId,
//                                           (PSTORAGE_DESCRIPTOR_HEADER*)&DeviceExtension->AdapterDescriptor);
//     }
//     if (NT_SUCCESS(status))
//     {
//         // Call port driver to get device descriptor.
//         propertyId = StorageDeviceProperty;
//
//         status = DeviceRetrieveDescriptor(DeviceExtension->Device,
//                                           &propertyId,
//                                           (PSTORAGE_DESCRIPTOR_HEADER*)&DeviceExtension->DeviceDescriptor);
//     }
//     if (NT_SUCCESS(status))
//     {
//         // Call port driver to get device power property.
//         // Not all port drivers support this property, and it's not fatal if this query fails.
//         propertyId = StorageDevicePowerProperty;
//
//         (void) DeviceRetrieveDescriptor(DeviceExtension->Device,
//                                         &propertyId,
//                                         (PSTORAGE_DESCRIPTOR_HEADER*)&DeviceExtension->PowerDescriptor);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Determine the maximum page-aligned and non-page-aligned transfer
//         // lengths here, so we needn't do this in common READ/WRITE code paths
//
//         // start with the number of pages the adapter can support
//         ULONG maxAlignedTransfer = DeviceExtension->AdapterDescriptor->MaximumPhysicalPages;
//         ULONG maxUnalignedTransfer = DeviceExtension->AdapterDescriptor->MaximumPhysicalPages;
//
//
//         // Unaligned buffers could cross a page boundary.
//         if (maxUnalignedTransfer > 1)
//         {
//             maxUnalignedTransfer--;
//         }
//
//         // if we'd overflow multiplying by page size, just max out the
//         // transfer length allowed by the number of pages limit.
//         if (maxAlignedTransfer >= (((ULONG)-1) / PAGE_SIZE))
//         {
//             maxAlignedTransfer = (ULONG)-1;
//         }
//         else
//         {
//             maxAlignedTransfer *= PAGE_SIZE;
//         }
//
//         if (maxUnalignedTransfer >= (((ULONG)-1) / PAGE_SIZE))
//         {
//             maxUnalignedTransfer = (ULONG)-1;
//         }
//         else
//         {
//             maxUnalignedTransfer *= PAGE_SIZE;
//         }
//
//         // finally, take the smaller of the above and the adapter's
//         // reported maximum number of bytes per transfer.
//         maxAlignedTransfer   = min(maxAlignedTransfer,   DeviceExtension->AdapterDescriptor->MaximumTransferLength);
//         maxUnalignedTransfer = min(maxUnalignedTransfer, DeviceExtension->AdapterDescriptor->MaximumTransferLength);
//
//         // Make sure the values are reasonable and not zero.
//         maxAlignedTransfer   = max(maxAlignedTransfer,   PAGE_SIZE);
//         maxUnalignedTransfer = max(maxUnalignedTransfer, PAGE_SIZE);
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                   "Device %p Max aligned/unaligned transfer size is %x/%x\n",
//                   DeviceExtension->Device,
//                   maxAlignedTransfer,
//                   maxUnalignedTransfer
//                   ));
//         DeviceExtension->DeviceAdditionalData.MaxPageAlignedTransferBytes = maxAlignedTransfer;
//         DeviceExtension->DeviceAdditionalData.MaxUnalignedTransferBytes = maxUnalignedTransfer;
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP,  "DeviceRetrieveDescriptorsAndTransferLength failed %lx\n", status));
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceRetrieveDescriptor(
//      WDFDEVICE                              Device,
//      PSTORAGE_PROPERTY_ID                   PropertyId,
//      PSTORAGE_DESCRIPTOR_HEADER*        Descriptor
//     )
// /*++
//
// Routine Description:
//
//     This routine will perform a query for the specified property id and will
//     allocate a non-paged buffer to store the data in.  It is the responsibility
//     of the caller to ensure that this buffer is freed.
//
//     This routine must be run at IRQL_PASSIVE_LEVEL
//
// Arguments:
//
//     Device - the device object
//     PropertyId - type of property to retrieve
//     Descriptor - buffer allocated in this function to hold the descriptor data
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS                status;
//     WDF_MEMORY_DESCRIPTOR   memoryDescriptor;
//
//     STORAGE_PROPERTY_QUERY  query = {0};
//     ULONG                   bufferLength = 0;
//
//     PSTORAGE_DESCRIPTOR_HEADER  descriptor = NULL;
//     PCDROM_DEVICE_EXTENSION     deviceExtension = DeviceGetExtension(Device);
//
//     PAGED_CODE();
//
//     // Set the passed-in descriptor pointer to NULL as default
//     *Descriptor = NULL;
//
//     // On the first pass we just want to get the first few
//     // bytes of the descriptor so we can read it's size
//     query.PropertyId = *PropertyId;
//     query.QueryType = PropertyStandardQuery;
//
//     descriptor = (PVOID)&query;
//
//     NT_ASSERT(sizeof(STORAGE_PROPERTY_QUERY) >= (sizeof(ULONG)*2));
//
//     WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(&memoryDescriptor,
//                                       (PVOID)&query,
//                                       sizeof(STORAGE_PROPERTY_QUERY));
//
//     status = WdfIoTargetSendIoctlSynchronously(deviceExtension->IoTarget,
//                                                NULL,
//                                                IOCTL_STORAGE_QUERY_PROPERTY,
//                                                &memoryDescriptor,
//                                                &memoryDescriptor,
//                                                NULL,
//                                                NULL);
//
//     if(!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,  "DeviceRetrieveDescriptor: error %lx trying to "
//                        "query properties #1\n", status));
//         return status;
//     }
//
//     if (descriptor->Size == 0)
//     {
//         // This DebugPrint is to help third-party driver writers
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,  "DeviceRetrieveDescriptor: size returned was zero?! (status "
//                     "%x\n", status));
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     // This time we know how much data there is so we can
//     // allocate a buffer of the correct size
//     bufferLength = descriptor->Size;
//     NT_ASSERT(bufferLength >= sizeof(STORAGE_PROPERTY_QUERY));
//     bufferLength = max(bufferLength, sizeof(STORAGE_PROPERTY_QUERY));
//
//     descriptor = ExAllocatePoolWithTag(NonPagedPoolNx, bufferLength, CDROM_TAG_DESCRIPTOR);
//
//     if(descriptor == NULL)
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,  "DeviceRetrieveDescriptor: unable to memory for descriptor "
//                     "(%d bytes)\n", bufferLength));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // setup the query again, as it was overwritten above
//     RtlZeroMemory(&query, sizeof(STORAGE_PROPERTY_QUERY));
//     query.PropertyId = *PropertyId;
//     query.QueryType = PropertyStandardQuery;
//
//     // copy the input to the new outputbuffer
//     RtlCopyMemory(descriptor,
//                   &query,
//                   sizeof(STORAGE_PROPERTY_QUERY));
//
//     WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(&memoryDescriptor,
//                                       (PVOID)descriptor,
//                                       bufferLength);
//
//     status = WdfIoTargetSendIoctlSynchronously(deviceExtension->IoTarget,
//                                                NULL,
//                                                IOCTL_STORAGE_QUERY_PROPERTY,
//                                                &memoryDescriptor,
//                                                &memoryDescriptor,
//                                                NULL,
//                                                NULL);
//
//     if(!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,  "DeviceRetrieveDescriptor: error %lx trying to "
//                        "query properties #1\n", status));
//         FREE_POOL(descriptor);
//
//         return status;
//     }
//
//     // return the memory we've allocated to the caller
//     *Descriptor = descriptor;
//
//     return status;
// } // end DeviceRetrieveDescriptor()
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// VOID
// DeviceRetrieveHackFlagsFromRegistry(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     try to retrieve hack flages from registry and put the information in
//     device extension.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//
//     none
//
// --*/
// {
//     NTSTATUS        status = STATUS_SUCCESS;
//     WDFKEY          hardwareKey = NULL;
//     WDFKEY          subKey = NULL;
//     ULONG           deviceHacks = 0;
//
//     DECLARE_CONST_UNICODE_STRING(subKeyName, CLASSP_REG_SUBKEY_NAME);
//     DECLARE_CONST_UNICODE_STRING(valueName, CLASSP_REG_HACK_VALUE_NAME);
//
//     PAGED_CODE();
//
//     status = WdfDeviceOpenRegistryKey(DeviceExtension->Device,
//                                       PLUGPLAY_REGKEY_DEVICE,
//                                       KEY_READ,
//                                       WDF_NO_OBJECT_ATTRIBUTES,
//                                       &hardwareKey);
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRegistryOpenKey(hardwareKey,
//                                     &subKeyName,
//                                     KEY_READ,
//                                     WDF_NO_OBJECT_ATTRIBUTES,
//                                     &subKey);
//
//         if (NT_SUCCESS(status))
//         {
//             status = WdfRegistryQueryULong(subKey,
//                                            &valueName,
//                                            &deviceHacks);
//             if (NT_SUCCESS(status))
//             {
//                 // remove unknown values and save...
//                 CLEAR_FLAG(deviceHacks, FDO_HACK_INVALID_FLAGS);
//                 SET_FLAG(DeviceExtension->PrivateFdoData->HackFlags, deviceHacks);
//             }
//
//             WdfRegistryClose(subKey);
//         }
//
//         WdfRegistryClose(hardwareKey);
//     }
//
//
//     //
//     // we should modify the system hive to include another key for us to grab
//     // settings from.  in this case:  Classpnp\HackFlags
//     //
//     // the use of a DWORD value for the HackFlags allows 32 hacks w/o
//     // significant use of the registry, and also reduces OEM exposure.
//     //
//     // definition of bit flags:
//     //   0x00000001 -- Device succeeds PREVENT_MEDIUM_REMOVAL, but
//     //                 cannot actually prevent removal.
//     //   0x00000002 -- Device hard-hangs or times out for GESN requests.
//     //   0x00000008 -- Device does not support RESERVE(6) and RELEASE(6).
//     //   0x00000010 -- Device may incorrecly report operational changes in GESN.
//     //   0x00000020 -- Device does not support streaming READ(12) / WRITE(12).
//     //   0x00000040 -- Device does not support asynchronous notification.
//     //   0xffffff80 -- Currently reserved, may be used later.
//     //
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID DeviceScanForSpecial(
//      PCDROM_DEVICE_EXTENSION          DeviceExtension,
//      CDROM_SCAN_FOR_SPECIAL_INFO      DeviceList[],
//      PCDROM_SCAN_FOR_SPECIAL_HANDLER  Function)
// /*++
//
// Routine Description:
//
//     scan the list of devices that should be hacked or not supported.
//
// Arguments:
//
//     DeviceExtension - the device context
//     DeviceList - the device list
//     Function - function used to scan from the list.
//
// Return Value:
//
//     none
//
// --*/
// {
//     PSTORAGE_DEVICE_DESCRIPTOR  deviceDescriptor;
//     PUCHAR                      vendorId;
//     PUCHAR                      productId;
//     PUCHAR                      productRevision;
//     UCHAR                       nullString[] = "";
//
//     PAGED_CODE();
//     NT_ASSERT(DeviceList);
//     NT_ASSERT(Function);
//
//     if (DeviceList == NULL)
//     {
//         return;
//     }
//     if (Function == NULL)
//     {
//         return;
//     }
//
//     deviceDescriptor = DeviceExtension->DeviceDescriptor;
//
//     // SCSI sets offsets to -1, ATAPI sets to 0.  check for both.
//     if (deviceDescriptor->VendorIdOffset != 0 &&
//         deviceDescriptor->VendorIdOffset != -1)
//     {
//         vendorId = ((PUCHAR)deviceDescriptor);
//         vendorId += deviceDescriptor->VendorIdOffset;
//     }
//     else
//     {
//         vendorId = nullString;
//     }
//
//     if (deviceDescriptor->ProductIdOffset != 0 &&
//         deviceDescriptor->ProductIdOffset != -1)
//     {
//         productId = ((PUCHAR)deviceDescriptor);
//         productId += deviceDescriptor->ProductIdOffset;
//     }
//     else
//     {
//         productId = nullString;
//     }
//
//     if (deviceDescriptor->ProductRevisionOffset != 0 &&
//         deviceDescriptor->ProductRevisionOffset != -1)
//     {
//         productRevision = ((PUCHAR)deviceDescriptor);
//         productRevision += deviceDescriptor->ProductRevisionOffset;
//     }
//     else
//     {
//         productRevision = nullString;
//     }
//
//     // loop while the device list is valid (not null-filled)
//     for (;(DeviceList->VendorId        != NULL ||
//            DeviceList->ProductId       != NULL ||
//            DeviceList->ProductRevision != NULL); DeviceList++)
//     {
//         if (StringsAreMatched(DeviceList->VendorId,        (LPSTR)vendorId) &&
//             StringsAreMatched(DeviceList->ProductId,       (LPSTR)productId) &&
//             StringsAreMatched(DeviceList->ProductRevision, (LPSTR)productRevision)
//             )
//         {
//
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, "DeviceScanForSpecial: Found matching "
//                         "controller Ven: %s Prod: %s Rev: %s\n",
//                         (LPCSTR)vendorId, (LPCSTR)productId, (LPCSTR)productRevision));
//
//             // pass the context to the call back routine and exit
//             (Function)(DeviceExtension, DeviceList->Data);
//
//             // for CHK builds, try to prevent wierd stacks by having a debug
//             // print here. it's a hack, but i know of no other way to prevent
//             // the stack from being wrong.
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, "DeviceScanForSpecial: "
//                         "completed callback\n"));
//             return;
//
//         } // else the strings did not match
//
//     } // none of the devices matched.
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT, "DeviceScanForSpecial: no match found for %p\n",
//                 DeviceExtension->DeviceObject));
//     return;
//
// } // end DeviceScanForSpecial()
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceHackFlagsScan(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      ULONG_PTR                Data
//     )
// {
//     PAGED_CODE();
//
//     // remove invalid flags and save
//     CLEAR_FLAG(Data, FDO_HACK_INVALID_FLAGS);
//     SET_FLAG(DeviceExtension->PrivateFdoData->HackFlags, Data);
//
//     return;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeHotplugInfo(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     Retrieve information into struc STORAGE_HOTPLUG_INFO in DeviceExtension
//     initialize the hotplug information only after the ScanForSpecial routines,
//     as it relies upon the hack flags - DeviceExtension->PrivateFdoData->HackFlags.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PCDROM_PRIVATE_FDO_DATA fdoData = DeviceExtension->PrivateFdoData;
//     DEVICE_REMOVAL_POLICY   deviceRemovalPolicy = 0;
//     ULONG                   resultLength = 0;
//     ULONG                   writeCacheOverride;
//
//     PAGED_CODE();
//
//     // start with some default settings
//     RtlZeroMemory(&(fdoData->HotplugInfo), sizeof(STORAGE_HOTPLUG_INFO));
//
//     // set the size (aka version)
//     fdoData->HotplugInfo.Size = sizeof(STORAGE_HOTPLUG_INFO);
//
//     // set if the device has removable media
//     if (DeviceExtension->DeviceDescriptor->RemovableMedia)
//     {
//         fdoData->HotplugInfo.MediaRemovable = TRUE;
//     }
//     else
//     {
//         fdoData->HotplugInfo.MediaRemovable = FALSE;
//     }
//
//     //
//     // this refers to devices which, for reasons not yet understood,
//     // do not fail PREVENT_MEDIA_REMOVAL requests even though they
//     // have no way to lock the media into the drive.  this allows
//     // the filesystems to turn off delayed-write caching for these
//     // devices as well.
//     //
//
//     if (TEST_FLAG(DeviceExtension->PrivateFdoData->HackFlags, FDO_HACK_CANNOT_LOCK_MEDIA))
//     {
//         fdoData->HotplugInfo.MediaHotplug = TRUE;
//     }
//     else
//     {
//         fdoData->HotplugInfo.MediaHotplug = FALSE;
//     }
//
//     // Query the default removal policy from the kernel
//     status = WdfDeviceQueryProperty(DeviceExtension->Device,
//                                     DevicePropertyRemovalPolicy,
//                                     sizeof(DEVICE_REMOVAL_POLICY),
//                                     (PVOID)&deviceRemovalPolicy,
//                                     &resultLength);
//     if (NT_SUCCESS(status))
//     {
//         if (resultLength != sizeof(DEVICE_REMOVAL_POLICY))
//         {
//             status = STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Look into the registry to see if the user has chosen
//         // to override the default setting for the removal policy.
//         // User can override only if the default removal policy is
//         // orderly or suprise removal.
//
//         if ((deviceRemovalPolicy == RemovalPolicyExpectOrderlyRemoval) ||
//             (deviceRemovalPolicy == RemovalPolicyExpectSurpriseRemoval))
//         {
//             DEVICE_REMOVAL_POLICY userRemovalPolicy = 0;
//
//             DeviceGetParameter(DeviceExtension,
//                                 CLASSP_REG_SUBKEY_NAME,
//                                 CLASSP_REG_REMOVAL_POLICY_VALUE_NAME,
//                                 (PULONG)&userRemovalPolicy);
//
//             // Validate the override value and use it only if it is an
//             // allowed value.
//             if ((userRemovalPolicy == RemovalPolicyExpectOrderlyRemoval) ||
//                 (userRemovalPolicy == RemovalPolicyExpectSurpriseRemoval))
//             {
//                 deviceRemovalPolicy = userRemovalPolicy;
//             }
//         }
//
//         // use this info to set the DeviceHotplug setting
//         // don't rely on DeviceCapabilities, since it can't properly
//         // determine device relations, etc.  let the kernel figure this
//         // stuff out instead.
//         if (deviceRemovalPolicy == RemovalPolicyExpectSurpriseRemoval)
//         {
//             fdoData->HotplugInfo.DeviceHotplug = TRUE;
//         }
//         else
//         {
//             fdoData->HotplugInfo.DeviceHotplug = FALSE;
//         }
//
//         // this refers to the *filesystem* caching, but has to be included
//         // here since it's a per-device setting.  this may change to be
//         // stored by the system in the future.
//         writeCacheOverride = FALSE;
//         DeviceGetParameter(DeviceExtension,
//                             CLASSP_REG_SUBKEY_NAME,
//                             CLASSP_REG_WRITE_CACHE_VALUE_NAME,
//                             &writeCacheOverride);
//
//         if (writeCacheOverride)
//         {
//             fdoData->HotplugInfo.WriteCacheEnableOverride = TRUE;
//         }
//         else
//         {
//             fdoData->HotplugInfo.WriteCacheEnableOverride = FALSE;
//         }
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_INIT,  "Could not initialize hotplug information %lx\n", status));
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitMmcContext(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine initializes and populates the internal data structures that are
//     used to discover various MMC-defined capabilities of the device.
//
//     This routine will not clean up allocate resources if it fails - that
//     is left for device stop/removal routines
//
// Arguments:
//
//     DeviceExtension - device extension
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DeviceExtension->DeviceAdditionalData.Mmc.IsMmc = FALSE;
//     DeviceExtension->DeviceAdditionalData.Mmc.IsAACS = FALSE;
//     DeviceExtension->DeviceAdditionalData.Mmc.IsWriter = FALSE;
//     DeviceExtension->DeviceAdditionalData.Mmc.WriteAllowed = FALSE;
//     DeviceExtension->DeviceAdditionalData.Mmc.IsCssDvd = FALSE;
//     DeviceExtension->DeviceAdditionalData.DriveDeviceType = FILE_DEVICE_CD_ROM;
//
//     // Determine if the drive is MMC-Capable
//     if (NT_SUCCESS(status))
//     {
//         status = DeviceGetMmcSupportInfo(DeviceExtension,
//                                          &DeviceExtension->DeviceAdditionalData.Mmc.IsMmc);
//
//         if (!NT_SUCCESS(status))
//         {
//             //Currently, only low resource error comes here.
//             //That is a success case for unsupporting this command.
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "DeviceInitMmcContext: Failed to get the support info for GET CONFIGURATION "
//                         "command, failng %!STATUS!\n", status
//                         ));
//
//             DeviceExtension->DeviceAdditionalData.Mmc.IsMmc = FALSE;
//             status = STATUS_SUCCESS;
//         }
//     }
//
//     if (NT_SUCCESS(status) && DeviceExtension->DeviceAdditionalData.Mmc.IsMmc)
//     {
//         // the drive supports at least a subset of MMC commands
//         // (and therefore supports READ_CD, etc...)
//
//         // allocate a buffer for all the capabilities and such
//         status = DeviceAllocateMmcResources(DeviceExtension->Device);
//     }
//
//     if (NT_SUCCESS(status) && DeviceExtension->DeviceAdditionalData.Mmc.IsMmc)
//     {
//         PFEATURE_HEADER header = NULL;
//         FEATURE_NUMBER  validationSchema;
//         ULONG           blockingFactor;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                   "DeviceInitMmcContext: FDO %p GET CONFIGURATION buffer %p\n",
//                   DeviceExtension->Device,
//                   DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer
//                   ));
//
//         // Update several properties using the retrieved Configuration Data.
//
//         // print all the feature pages (DBG only)
//         DevicePrintAllFeaturePages(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                    DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize);
//
//         // if AACS feature exists, enable AACS flag in the driver
//         header = DeviceFindFeaturePage(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                        DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
//                                        FeatureAACS);
//         if (header)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                         "DeviceInitMmcContext: Reporting AACS support for device due to "
//                         "GET CONFIGURATION showing support\n"
//                         ));
//             DeviceExtension->DeviceAdditionalData.Mmc.IsAACS = TRUE;
//         }
//
// #ifdef ENABLE_AACS_TESTING
//         DeviceExtension->DeviceAdditionalData.Mmc.IsAACS = TRUE; // just force it true for testing
// #endif // ENABLE_AACS_TESTING
//
//         // Check if it's a DVD device
//         header = DeviceFindFeaturePage(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                        DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
//                                        FeatureDvdRead);
//         if (header != NULL)
//         {
//             DeviceExtension->DeviceAdditionalData.DriveDeviceType = FILE_DEVICE_DVD;
//         }
//
//         // check if drive is writer
//         DeviceUpdateMmcWriteCapability(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                        DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
//                                        FALSE,                //Check if the drive has the ability to write.
//                                        (PBOOLEAN)&(DeviceExtension->DeviceAdditionalData.Mmc.IsWriter),
//                                        &validationSchema,
//                                        &blockingFactor);
//
//         // check if there is a CSS protected DVD or CPPM-protected DVDAudio media in drive.
//         header = DeviceFindFeaturePage(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                        DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
//                                        FeatureDvdCSS);
//
//         DeviceExtension->DeviceAdditionalData.Mmc.IsCssDvd = (header != NULL) && (header->Current);
//
//         // Flag the StartIo routine to update its state and hook in the error handler
//         DeviceExtension->DeviceAdditionalData.Mmc.UpdateState = CdromMmcUpdateRequired;
//         DeviceExtension->DeviceAdditionalData.ErrorHandler = DeviceErrorHandlerForMmc;
//
//         SET_FLAG(DeviceExtension->DeviceFlags, DEV_SAFE_START_UNIT);
//
//         // Read the CDROM mode sense page to get additional info for raw read requests.
//         // only valid for MMC devices
//         DeviceSetRawReadInfo(DeviceExtension);
//     }
//
//     // Set Read-Only device flag for non-MMC device.
//     if (!(DeviceExtension->DeviceAdditionalData.Mmc.IsMmc))
//     {
//         ULONG  deviceCharacteristics = WdfDeviceGetCharacteristics(DeviceExtension->Device);
//
//         deviceCharacteristics |= FILE_READ_ONLY_DEVICE;
//
//         WdfDeviceSetCharacteristics(DeviceExtension->Device, deviceCharacteristics);
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                    "DeviceInitMmcContext: FDO %p Device is not an MMC compliant device, so setting "
//                    "to read-only (legacy) mode",
//                    DeviceExtension->Device
//                    ));
//     }
//
//     // Set DEV_SAFE_START_UNIT flag for newer devices.
//     if (DeviceExtension->DeviceAdditionalData.DriveDeviceType == FILE_DEVICE_DVD)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "DeviceInitMmcContext: DVD Devices require START UNIT\n"));
//         SET_FLAG(DeviceExtension->DeviceFlags, DEV_SAFE_START_UNIT);
//
//     }
//     else if ((DeviceExtension->DeviceDescriptor->BusType != BusTypeScsi)  &&
//              (DeviceExtension->DeviceDescriptor->BusType != BusTypeAtapi) &&
//              (DeviceExtension->DeviceDescriptor->BusType != BusTypeUnknown)
//              )
//     {
//         // devices on the newer busses require START_UNIT
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                   "DeviceInitMmcContext: Devices for newer buses require START UNIT\n"));
//         SET_FLAG(DeviceExtension->DeviceFlags, DEV_SAFE_START_UNIT);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// ULONG
// DeviceGetTimeOutValueFromRegistry()
// /*++
//
// Routine Description:
//
//     get the device time out value from registry
//
// Arguments:
//
//     None
//
// Return Value:
//
//     ULONG - value of timeout
//
// --*/
// {
//     NTSTATUS    status;
//     WDFKEY      registryKey = NULL;
//     ULONG        timeOutValue = 0;
//
//     DECLARE_CONST_UNICODE_STRING(registryValueName, L"TimeOutValue");
//
//     PAGED_CODE();
//
//     // open the service key.
//     status = WdfDriverOpenParametersRegistryKey(WdfGetDriver(),
//                                                 KEY_READ,
//                                                 WDF_NO_OBJECT_ATTRIBUTES,
//                                                 &registryKey);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRegistryQueryULong(registryKey,
//                                        &registryValueName,
//                                        &timeOutValue);
//
//         WdfRegistryClose(registryKey);
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         timeOutValue = 0;
//     }
//
//     return timeOutValue;
//
// } // end DeviceGetTimeOutValueFromRegistry()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceScanSpecialDevices(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This function checks to see if an SCSI logical unit requires an special
//     initialization or error processing.
//
// Arguments:
//
//     Device - device object.
//
// Return Value:
//
//     None.
//
// --*/
// {
//
//     PAGED_CODE();
//
//     // set our hack flags
//     DeviceScanForSpecial(DeviceExtension, CdromHackItems, ScanForSpecialHandler);
//
//     //
//     // All CDRom's can ignore the queue lock failure for power operations
//     // and do not require handling the SpinUp case (unknown result of sending
//     // a cdrom a START_UNIT command -- may eject disks?)
//     //
//     // We send the stop command mostly to stop outstanding asynch operations
//     // (like audio playback) from running when the system is powered off.
//     // Because of this and the unlikely chance that a PLAY command will be
//     // sent in the window between the STOP and the time the machine powers down
//     // we don't require queue locks.  This is important because without them
//     // classpnp's power routines will send the START_STOP_UNIT command to the
//     // device whether or not it supports locking (atapi does not support locking
//     // and if we requested them we would end up not stopping audio on atapi
//     // devices).
// //    SET_FLAG(deviceExtension->ScanForSpecialFlags, CDROM_SPECIAL_DISABLE_SPIN_UP);
// //    SET_FLAG(deviceExtension->ScanForSpecialFlags, CDROM_SPECIAL_NO_QUEUE_LOCK);
//
//     if (TEST_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_TOSHIBA_SD_W1101))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "DeviceScanSpecialDevices: Found Toshiba SD-W1101 DVD-RAM "
//                     "-- This drive will *NOT* support DVD-ROM playback.\n"));
//     }
//     else if (TEST_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_HITACHI_GD_2000))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "DeviceScanSpecialDevices: Found Hitachi GD-2000\n"));
//
//         // Setup an error handler to spin up the drive when it idles out
//         // since it seems to like to fail to spin itself back up on its
//         // own for a REPORT_KEY command.  It may also lose the AGIDs that
//         // it has given, which will result in DVD playback failures.
//         // This routine will just do what it can...
//         DeviceExtension->DeviceAdditionalData.ErrorHandler = DeviceErrorHandlerForHitachiGD2000;
//
//         // this drive may require START_UNIT commands to spin
//         // the drive up when it's spun itself down.
//         SET_FLAG(DeviceExtension->DeviceFlags, DEV_SAFE_START_UNIT);
//     }
//     else if (TEST_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_FUJITSU_FMCD_10x))
//     {
//         // When Read command is issued to FMCD-101 or FMCD-102 and there is a music
//         // cd in it. It takes longer time than SCSI_CDROM_TIMEOUT before returning
//         // error status.
//         DeviceExtension->TimeOutValue = 20;
//     }
//     else if (TEST_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_DEC_RRD))
//     {
//         NTSTATUS                   status;
//         PMODE_PARM_READ_WRITE_DATA modeParameters;
//         SCSI_REQUEST_BLOCK         srb = {0};
//         PCDB                       cdb;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
//                     "DeviceScanSpecialDevices:  Found DEC RRD.\n"));
//
//         DeviceExtension->DeviceAdditionalData.IsDecRrd = TRUE;
//
//         // Setup an error handler to reinitialize the cd rom after it is reset?
//         //
//         //DeviceExtension->DevInfo->ClassError = DecRrdProcessError;
//
//         // Found a DEC RRD cd-rom.  These devices do not pass MS HCT
//         // multi-media tests because the DEC firmware modifieds the block
//         // from the PC-standard 2K to 512.  Change the block transfer size
//         // back to the PC-standard 2K by using a mode select command.
//
//         modeParameters = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                sizeof(MODE_PARM_READ_WRITE_DATA),
//                                                CDROM_TAG_MODE_DATA);
//         if (modeParameters == NULL)
//         {
//             return;
//         }
//
//         RtlZeroMemory(modeParameters, sizeof(MODE_PARM_READ_WRITE_DATA));
//         RtlZeroMemory(&srb,           sizeof(SCSI_REQUEST_BLOCK));
//
//         // Set the block length to 2K.
//         modeParameters->ParameterListHeader.BlockDescriptorLength = sizeof(MODE_PARAMETER_BLOCK);
//
//         // Set block length to 2K (0x0800) in Parameter Block.
//         modeParameters->ParameterListBlock.BlockLength[0] = 0x00; //MSB
//         modeParameters->ParameterListBlock.BlockLength[1] = 0x08;
//         modeParameters->ParameterListBlock.BlockLength[2] = 0x00; //LSB
//
//         // Build the mode select CDB.
//         srb.CdbLength = 6;
//         srb.TimeOutValue = DeviceExtension->TimeOutValue;
//
//         cdb = (PCDB)srb.Cdb;
//         cdb->MODE_SELECT.PFBit               = 1;
//         cdb->MODE_SELECT.OperationCode       = SCSIOP_MODE_SELECT;
//         cdb->MODE_SELECT.ParameterListLength = HITACHI_MODE_DATA_SIZE;
//
//         // Send the request to the device.
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             &srb,
//                                             modeParameters,
//                                             sizeof(MODE_PARM_READ_WRITE_DATA),
//                                             TRUE,
//                                             NULL);
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                         "DeviceScanSpecialDevices: Setting DEC RRD to 2K block"
//                         "size failed [%x]\n", status));
//         }
//
//         ExFreePool(modeParameters);
//     }
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// ScanForSpecialHandler(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      ULONG_PTR               HackFlags
//     )
// {
//     PAGED_CODE();
//
//     CLEAR_FLAG(HackFlags, CDROM_HACK_INVALID_FLAGS);
//
//     DeviceExtension->DeviceAdditionalData.HackFlags = HackFlags;
//
//     return;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceCacheDeviceInquiryData(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     get inquiry data from device and cache it into device extension
//     The first INQUIRY command sent is with 0x24 bytes required data,
//     as ATAport driver always sends this to enumerate devices and 0x24
//     bytes is the minimum data device should return by spec.
//
// Arguments:
//
//     DeviceExtension - device extension.
//
// Return Value:
//
//     NTSTATUS.
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     SCSI_REQUEST_BLOCK  srb = {0};
//     PCDB                cdb = (PCDB)(&srb.Cdb);
//     PINQUIRYDATA        tmpInquiry = NULL;
//
//     // by spec, device should return at least 36 bytes.
//     ULONG               requestedInquiryTransferBytes = MINIMUM_CDROM_INQUIRY_SIZE;
//     BOOLEAN             needResendCommand = TRUE;
//     BOOLEAN             portDriverHack = FALSE;
//
//     // this ensures that the strings vendorID, productID, and firmwareRevision
//     // are all available in the inquiry data.  In addition, MMC spec requires
//     // all type 5 devices to have minimum 36 bytes of inquiry.
//     static const UCHAR minInquiryAdditionalLength =
//                                     MINIMUM_CDROM_INQUIRY_SIZE -
//                                     RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, AdditionalLength);
//
//     C_ASSERT( RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, AdditionalLength) <= 8 );
//     C_ASSERT( RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, ProductRevisionLevel) == MINIMUM_CDROM_INQUIRY_SIZE );
//
//     PAGED_CODE();
//
//     // short-circuit here for if already cached for this device
//     // required to avoid use of scratch buffer after initialization
//     // of MCN code.
//     if (DeviceExtension->DeviceAdditionalData.CachedInquiryData != NULL)
//     {
//         NT_ASSERT(DeviceExtension->DeviceAdditionalData.CachedInquiryDataByteCount != 0);
//         return STATUS_SUCCESS;
//     }
//
//     // 1. retrieve the inquiry data length
//
//     // 1.1 allocate inquiry data buffer
//     tmpInquiry = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                        requestedInquiryTransferBytes,
//                                        CDROM_TAG_INQUIRY);
//     if (tmpInquiry == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // 1.2 send INQUIRY command
//     if (NT_SUCCESS(status))
//     {
//         srb.CdbLength = 6;
//         cdb->AsByte[0] = SCSIOP_INQUIRY;
//         cdb->AsByte[3] = (UCHAR)( requestedInquiryTransferBytes >> (8*1) );
//         cdb->AsByte[4] = (UCHAR)( requestedInquiryTransferBytes >> (8*0) );
//
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             &srb,
//                                             tmpInquiry,
//                                             requestedInquiryTransferBytes,
//                                             FALSE,
//                                             NULL);
//     }
//
//     // 1.3 get required data length
//     if (NT_SUCCESS(status))
//     {
//         if ((requestedInquiryTransferBytes == srb.DataTransferLength) &&
//             (requestedInquiryTransferBytes == (tmpInquiry->AdditionalLength +
//                                                RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, AdditionalLength))) )
//         {
//             // device has only 36 bytes of INQUIRY data. do not need to resend the command.
//             needResendCommand = FALSE;
//         }
//         else
//         {
//             // workaround an ATAPI.SYS bug where additional length field is set to zero
//             if (tmpInquiry->AdditionalLength == 0)
//             {
//                 tmpInquiry->AdditionalLength = minInquiryAdditionalLength;
//                 portDriverHack = TRUE;
//             }
//
//             requestedInquiryTransferBytes =
//                                 tmpInquiry->AdditionalLength +
//                                 RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, AdditionalLength);
//
//             if (requestedInquiryTransferBytes >= MINIMUM_CDROM_INQUIRY_SIZE)
//             {
//                 needResendCommand = TRUE;
//             }
//             else
//             {
//                 needResendCommand = FALSE;
//                 //Length is small than minimum length, error out.
//                 status = STATUS_DEVICE_PROTOCOL_ERROR;
//             }
//         }
//     }
//
//     // 2. retrieve the inquiry data if still needed.
//
//     // 2.1 Clean up.
//     if (NT_SUCCESS(status) && needResendCommand)
//     {
//         FREE_POOL(tmpInquiry);
//         RtlZeroMemory(&srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         tmpInquiry = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                            requestedInquiryTransferBytes,
//                                            CDROM_TAG_INQUIRY);
//         if (tmpInquiry == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     // 2.2 resend INQUIRY command
//     if (NT_SUCCESS(status) && needResendCommand)
//     {
//         srb.CdbLength = 6;
//         cdb->AsByte[0] = SCSIOP_INQUIRY;
//         cdb->AsByte[3] = (UCHAR)( requestedInquiryTransferBytes >> (8*1) );
//         cdb->AsByte[4] = (UCHAR)( requestedInquiryTransferBytes >> (8*0) );
//
//         status = DeviceSendSrbSynchronously( DeviceExtension->Device,
//                                              &srb,
//                                              tmpInquiry,
//                                              requestedInquiryTransferBytes,
//                                              FALSE,
//                                              NULL);
//
//         if (!NT_SUCCESS(status))
//         {
//             // Workaround for drive reports that it has more INQUIRY data than reality.
//             if ((srb.SrbStatus == SRB_STATUS_DATA_OVERRUN) &&
//                 (srb.DataTransferLength < requestedInquiryTransferBytes) &&
//                 (srb.DataTransferLength >= MINIMUM_CDROM_INQUIRY_SIZE))
//             {
//                 //Port driver says buffer size mismatch (buffer underrun),
//                 //retry with the real buffer size it could return.
//                 requestedInquiryTransferBytes = srb.DataTransferLength;
//
//                 FREE_POOL(tmpInquiry);
//                 RtlZeroMemory(&srb, sizeof(SCSI_REQUEST_BLOCK));
//
//                 tmpInquiry = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                                    requestedInquiryTransferBytes,
//                                                    CDROM_TAG_INQUIRY);
//                 if (tmpInquiry == NULL)
//                 {
//                     status = STATUS_INSUFFICIENT_RESOURCES;
//                 }
//                 else
//                 {
//                     srb.CdbLength = 6;
//                     cdb->AsByte[0] = SCSIOP_INQUIRY;
//                     cdb->AsByte[3] = (UCHAR)( requestedInquiryTransferBytes >> (8*1) );
//                     cdb->AsByte[4] = (UCHAR)( requestedInquiryTransferBytes >> (8*0) );
//
//                     status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                                         &srb,
//                                                         tmpInquiry,
//                                                         requestedInquiryTransferBytes,
//                                                         FALSE,
//                                                         NULL);
//                 }
//             }
//         }
//
//         //Check the transferred data length for safe.
//         if (NT_SUCCESS(status))
//         {
//             requestedInquiryTransferBytes = srb.DataTransferLength;
//
//             if (requestedInquiryTransferBytes < MINIMUM_CDROM_INQUIRY_SIZE)
//             {
//                 // should never occur
//                 status = STATUS_DEVICE_PROTOCOL_ERROR;
//             }
//         }
//
//         // ensure we got some non-zero data....
//         // This is done so we don't accidentally work around the
//         // ATAPI.SYS bug when no data was transferred.
//         if (NT_SUCCESS(status) && portDriverHack)
//         {
//             PULONG  tmp = (PULONG)tmpInquiry;
//             ULONG   i = MINIMUM_CDROM_INQUIRY_SIZE / sizeof(ULONG);
//             C_ASSERT( RTL_SIZEOF_THROUGH_FIELD(INQUIRYDATA, ProductRevisionLevel) % sizeof(ULONG) == 0 );
//
//             // wouldn't you know it -- there is no RtlIsMemoryZero() function; Make one up.
//             for ( ; i != 0; i--)
//             {
//                 if (*tmp != 0)
//                 {
//                     break; // out of this inner FOR loop -- guarantees 'i != 0'
//                 }
//                 tmp++;
//             }
//
//             if (i == 0) // all loop'd successfully
//             {
//                 // should never occur to successfully get all zero'd data
//                 status = STATUS_DEVICE_PROTOCOL_ERROR;
//             }
//         }
//     }
//
//     // if everything succeeded, then (and only then) modify the device extension
//     if (NT_SUCCESS(status))
//     {
//         DeviceExtension->DeviceAdditionalData.CachedInquiryData = tmpInquiry;
//         DeviceExtension->DeviceAdditionalData.CachedInquiryDataByteCount = requestedInquiryTransferBytes;
//     }
//     else
//     {
//         FREE_POOL(tmpInquiry);
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceGetMmcSupportInfo(
//       PCDROM_DEVICE_EXTENSION   DeviceExtension,
//      PBOOLEAN                  IsMmcDevice
//     )
// /*++
//
// Routine Description:
//
//     check if the device is MMC capable.
//
// Arguments:
//
//     DeviceExtension - device extension.
//
// Return Value:
//
//     NTSTATUS.
//     IsMmcDevice - TRUE (MMC capable); FALSE (not MMC device)
//
// --*/
// {
//     NTSTATUS    status;
//     ULONG       size;
//     ULONG       previouslyFailed;
//
//     PAGED_CODE();
//
//     *IsMmcDevice  = FALSE;
//
//     // read the registry in case the drive failed previously,
//     // and a timeout is occurring.
//     previouslyFailed = FALSE;
//     DeviceGetParameter(DeviceExtension,
//                        CDROM_SUBKEY_NAME,
//                        CDROM_NON_MMC_DRIVE_NAME,
//                        &previouslyFailed);
//
//     if (previouslyFailed)
//     {
//         SET_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_BAD_GET_CONFIG_SUPPORT);
//     }
//
//     // read from the registry in case the drive reports bad profile lengths
//     previouslyFailed = FALSE;
//     DeviceGetParameter(DeviceExtension,
//                        CDROM_SUBKEY_NAME,
//                        CDROM_NON_MMC_VENDOR_SPECIFIC_PROFILE,
//                        &previouslyFailed);
//
//     if (previouslyFailed)
//     {
//         SET_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_BAD_VENDOR_PROFILES);
//     }
//
//     // check for the ProfileList feature to determine if the drive is MMC compliant
//     // and set the "size" local variable to total GetConfig data size available.
//     // NOTE: This will exit this function in some error paths.
//     {
//         GET_CONFIGURATION_HEADER    localHeader = {0};
//         ULONG                       usable = 0;
//
//         status = DeviceGetConfiguration(DeviceExtension->Device,
//                                         &localHeader,
//                                         sizeof(GET_CONFIGURATION_HEADER),
//                                         &usable,
//                                         FeatureProfileList,
//                                         SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL);
//
//         if (status == STATUS_INVALID_DEVICE_REQUEST ||
//             status == STATUS_NO_MEDIA_IN_DEVICE     ||
//             status == STATUS_IO_DEVICE_ERROR        ||
//             status == STATUS_IO_TIMEOUT)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                        "GetConfiguration Failed (%x), device %p not mmc-compliant\n",
//                        status, DeviceExtension->DeviceObject
//                        ));
//
//             previouslyFailed = TRUE;
//             DeviceSetParameter( DeviceExtension,
//                                 CDROM_SUBKEY_NAME,
//                                 CDROM_NON_MMC_DRIVE_NAME,
//                                 previouslyFailed);
//
//             return STATUS_SUCCESS;
//         }
//         else if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                        "GetConfiguration Failed, status %x -- defaulting to -ROM\n",
//                        status));
//
//             return STATUS_SUCCESS;
//         }
//         else if (usable < sizeof(GET_CONFIGURATION_HEADER))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                        "GetConfiguration Failed, returned only %x bytes!\n", usable));
//             previouslyFailed = TRUE;
//             DeviceSetParameter( DeviceExtension,
//                                 CDROM_SUBKEY_NAME,
//                                 CDROM_NON_MMC_DRIVE_NAME,
//                                 previouslyFailed);
//
//             return STATUS_SUCCESS;
//         }
//
//         size = (localHeader.DataLength[0] << 24) |
//                (localHeader.DataLength[1] << 16) |
//                (localHeader.DataLength[2] <<  8) |
//                (localHeader.DataLength[3] <<  0) ;
//
//
//         if ((size <= 4) || (size + 4 < size))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                        "GetConfiguration Failed, claims MMC support but doesn't "
//                        "correctly return config length! (%x)\n",
//                        size
//                        ));
//             previouslyFailed = TRUE;
//             DeviceSetParameter( DeviceExtension,
//                                 CDROM_SUBKEY_NAME,
//                                 CDROM_NON_MMC_DRIVE_NAME,
//                                 previouslyFailed);
//
//             return STATUS_SUCCESS;
//         }
//         else if ((size % 4) != 0)
//         {
//             if ((size % 2) != 0)
//             {
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                            "GetConfiguration Failed, returned odd number of bytes %x!\n",
//                            size
//                            ));
//                 previouslyFailed = TRUE;
//                 DeviceSetParameter( DeviceExtension,
//                                     CDROM_SUBKEY_NAME,
//                                     CDROM_NON_MMC_DRIVE_NAME,
//                                     previouslyFailed);
//
//                 return STATUS_SUCCESS;
//             }
//             else
//             {
//                 if (TEST_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_BAD_VENDOR_PROFILES))
//                 {
//                     // we've already caught this and ASSERT'd once, so don't do it again
//                 }
//                 else
//                 {
//                     TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                                "GetConfiguration returned a size that is not per spec (%x bytes), this is probably because of a vendor specific data header with a size not divisible by 4.\n",
//                                size
//                                ));
//                     previouslyFailed = TRUE;
//                     DeviceSetParameter(DeviceExtension,
//                                        CDROM_SUBKEY_NAME,
//                                        CDROM_NON_MMC_VENDOR_SPECIFIC_PROFILE,
//                                        previouslyFailed);
//                 }
//             }
//         }
//
//         size += 4; // sizeof the datalength fields
//     }
//
//     *IsMmcDevice = TRUE;
//
//     // This code doesn't handle total get config size over 64k
//     NT_ASSERT( size <= MAXUSHORT );
//
//     // Check for SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE support in the device.
//     // NOTE: This will exit this function in some error paths.
//     {
//         ULONG   featureSize = sizeof(GET_CONFIGURATION_HEADER)+sizeof(FEATURE_HEADER);
//         ULONG   usable = 0;
//
//         PGET_CONFIGURATION_HEADER configBuffer = ExAllocatePoolWithTag(
//                                                             NonPagedPoolNx,
//                                                             featureSize,
//                                                             CDROM_TAG_GET_CONFIG);
//
//         if (configBuffer == NULL)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         // read the registry in case the drive failed previously,
//         // and a timeout is occurring.
//         previouslyFailed = FALSE;
//         DeviceGetParameter( DeviceExtension,
//                             CDROM_SUBKEY_NAME,
//                             CDROM_TYPE_ONE_GET_CONFIG_NAME,
//                             &previouslyFailed);
//
//         if (previouslyFailed)
//         {
//             SET_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG);
//             FREE_POOL(configBuffer);
//             return STATUS_SUCCESS;
//         }
//
//         // Get only the config and feature header
//         status = DeviceGetConfiguration(DeviceExtension->Device,
//                                         configBuffer,
//                                         featureSize,
//                                         &usable,
//                                         FeatureProfileList,
//                                         SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE);
//
//         if (!NT_SUCCESS(status) || (usable < featureSize))
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                        "Type One GetConfiguration Failed. Usable buffer size: %d\n", usable));
//             previouslyFailed = TRUE;
//         }
//         else
//         {
//             PFEATURE_HEADER featureHeader;
//             ULONG           totalAvailableBytes = 0;
//             ULONG           expectedAvailableBytes = 0;
//
//             REVERSE_BYTES(&totalAvailableBytes, configBuffer->DataLength);
//             totalAvailableBytes += RTL_SIZEOF_THROUGH_FIELD(GET_CONFIGURATION_HEADER, DataLength);
//
//             featureHeader = (PFEATURE_HEADER) ((PUCHAR)configBuffer + sizeof(GET_CONFIGURATION_HEADER));
//             expectedAvailableBytes = sizeof(GET_CONFIGURATION_HEADER) +
//                                      sizeof(FEATURE_HEADER) +
//                                      featureHeader->AdditionalLength;
//
//             if (totalAvailableBytes > expectedAvailableBytes)
//             {
//                 // Device is returning more than required size. Most likely the device
//                 // is returning TYPE ALL data. Set the flag to use TYPE ALL for TYPE ONE
//                 // requets
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
//                            "Type One GetConfiguration Failed. "
//                            "Device returned %d bytes instead of %d bytes\n",
//                            size, featureSize));
//
//                 previouslyFailed = TRUE;
//             }
//         }
//
//         FREE_POOL(configBuffer);
//
//         if (previouslyFailed == TRUE)
//         {
//             DeviceSetParameter( DeviceExtension,
//                                 CDROM_SUBKEY_NAME,
//                                 CDROM_TYPE_ONE_GET_CONFIG_NAME,
//                                 previouslyFailed);
//
//             SET_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG);
//         }
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceSetRawReadInfo(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine reads the CDROM capabilities mode page and save information
//     in the device extension needed for raw reads.
//     NOTE: this function is only valid for MMC device
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     PUCHAR      buffer = NULL;
//     ULONG       count = 0;
//
//     PAGED_CODE();
//
//     // Check whether the device can return C2 error flag bits and the block
//     // error byte.  If so, save this info and fill in appropriate flag during
//     // raw read requests.
//
//     // Start by checking the GET_CONFIGURATION data
//     {
//         PFEATURE_DATA_CD_READ   cdReadHeader = NULL;
//         ULONG                   additionalLength = sizeof(FEATURE_DATA_CD_READ) - sizeof(FEATURE_HEADER);
//
//         cdReadHeader = DeviceFindFeaturePage(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                             DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
//                                             FeatureCdRead);
//
//         if ((cdReadHeader != NULL) &&
//             (cdReadHeader->Header.AdditionalLength >= additionalLength) &&
//             (cdReadHeader->C2ErrorData)
//             )
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                       "DeviceSetRawReadInfo: FDO %p GET_CONFIG shows ability to read C2 error bits\n",
//                       DeviceExtension->DeviceObject
//                       ));
//             DeviceExtension->DeviceAdditionalData.Mmc.ReadCdC2Pointers = TRUE;    // Device returns C2 error info.
//         }
//     }
//
//     // Unfortunately, the only way to check for the ability to read R-W subcode
//     // information is via MODE_SENSE.  Do so here, and check the C2 bit as well
//     // in case the drive has a firmware bug where it fails to report this ability
//     // in GET_CONFIG (which some drives do).
//     for (count = 0; count < 6; count++)
//     {
//         SCSI_REQUEST_BLOCK  srb = {0};
//         PCDB                cdb = (PCDB)srb.Cdb;
//         ULONG               bufferLength = 0;
//
//         // Build the MODE SENSE CDB.  Try 10-byte CDB first.
//         if ((count/3) == 0)
//         {
//             bufferLength = sizeof(CDVD_CAPABILITIES_PAGE)  +
//                            sizeof(MODE_PARAMETER_HEADER10) +
//                            sizeof(MODE_PARAMETER_BLOCK);
//
//             cdb->MODE_SENSE10.OperationCode = SCSIOP_MODE_SENSE10;
//             cdb->MODE_SENSE10.Dbd = 1;
//             cdb->MODE_SENSE10.PageCode = MODE_PAGE_CAPABILITIES;
//             cdb->MODE_SENSE10.AllocationLength[0] = (UCHAR)(bufferLength >> 8);
//             cdb->MODE_SENSE10.AllocationLength[1] = (UCHAR)(bufferLength >> 0);
//             srb.CdbLength = 10;
//         }
//         else
//         {
//             bufferLength = sizeof(CDVD_CAPABILITIES_PAGE) +
//                            sizeof(MODE_PARAMETER_HEADER)  +
//                            sizeof(MODE_PARAMETER_BLOCK);
//
//             cdb->MODE_SENSE.OperationCode = SCSIOP_MODE_SENSE;
//             cdb->MODE_SENSE.Dbd = 1;
//             cdb->MODE_SENSE.PageCode = MODE_PAGE_CAPABILITIES;
//             cdb->MODE_SENSE.AllocationLength = (UCHAR)bufferLength;
//             srb.CdbLength = 6;
//         }
//
//         // Set timeout value from device extension.
//         srb.TimeOutValue = DeviceExtension->TimeOutValue;
//
//         buffer = ExAllocatePoolWithTag(NonPagedPoolNx, bufferLength, CDROM_TAG_MODE_DATA);
//
//         if (buffer == NULL)
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_IOCTL,
//                       "DeviceSetRawReadInfo: cannot allocate "
//                       "buffer, so not setting raw read info for FDO %p\n",
//                       DeviceExtension->DeviceObject
//                       ));
//             status = STATUS_INSUFFICIENT_RESOURCES;
//             goto FnExit;
//         }
//
//         RtlZeroMemory(buffer, bufferLength);
//
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             &srb,
//                                             buffer,
//                                             bufferLength,
//                                             FALSE,
//                                             NULL);
//
//         if (NT_SUCCESS(status) ||
//             (status == STATUS_DATA_OVERRUN) ||
//             (status == STATUS_BUFFER_OVERFLOW))
//         {
//             PCDVD_CAPABILITIES_PAGE capabilities = NULL;
//
//             // determine where the capabilities page really is
//             if ((count/3) == 0)
//             {
//                 PMODE_PARAMETER_HEADER10 p = (PMODE_PARAMETER_HEADER10)buffer;
//                 capabilities = (PCDVD_CAPABILITIES_PAGE)(buffer +
//                                                          sizeof(MODE_PARAMETER_HEADER10) +
//                                                          (p->BlockDescriptorLength[0] * 256) +
//                                                          p->BlockDescriptorLength[1]);
//             }
//             else
//             {
//                 PMODE_PARAMETER_HEADER p = (PMODE_PARAMETER_HEADER)buffer;
//                 capabilities = (PCDVD_CAPABILITIES_PAGE)(buffer +
//                                                          sizeof(MODE_PARAMETER_HEADER) +
//                                                          p->BlockDescriptorLength);
//             }
//
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                       "DeviceSetRawReadInfo: FDO %p CDVD Capabilities buffer %p\n",
//                       DeviceExtension->DeviceObject,
//                       buffer
//                       ));
//
//             if (capabilities->PageCode == MODE_PAGE_CAPABILITIES)
//             {
//                 if (capabilities->C2Pointers)
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                               "DeviceSetRawReadInfo: FDO %p supports C2 error bits in READ_CD command\n",
//                               DeviceExtension->DeviceObject
//                               ));
//                     DeviceExtension->DeviceAdditionalData.Mmc.ReadCdC2Pointers = TRUE;
//                 }
//
//                 if (capabilities->RWSupported)
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL,
//                               "DeviceSetRawReadInfo: FDO %p supports raw subcode in READ_CD command\n",
//                               DeviceExtension->DeviceObject
//                               ));
//                     DeviceExtension->DeviceAdditionalData.Mmc.ReadCdSubCode = TRUE;
//                 }
//
//                 break;
//             }
//         }
//
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                   "DeviceSetRawReadInfo: FDO %p failed %x byte mode sense, status %x\n",
//                   DeviceExtension->DeviceObject,
//                   (((count/3) == 0) ? 10 : 6),
//                   status
//                   ));
//
//         FREE_POOL(buffer);
//     }
//
//     if (count == 6)
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_IOCTL,
//                   "DeviceSetRawReadInfo: FDO %p couldn't get mode sense data\n",
//                   DeviceExtension->DeviceObject
//                   ));
//     }
//
// FnExit:
//
//     if (buffer)
//     {
//         FREE_POOL(buffer);
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceInitializeDvd(
//      WDFDEVICE Device
//     )
// /*++
//
// Routine Description:
//
//     This routine sets the region of DVD drive
//     NOTE: this routine uses ScratchBuffer, it must be called after ScratchBuffer allocated.
//
// Arguments:
//
//     Device - device object
//
// Return Value:
//
//     NTSTATUS
//
// --*/
//
// {
//     NTSTATUS                    status = STATUS_SUCCESS;
//     PCDROM_DEVICE_EXTENSION     deviceExtension;
//     PDVD_COPY_PROTECT_KEY       copyProtectKey = NULL;
//     PDVD_RPC_KEY                rpcKey = NULL;
//     ULONG                       bufferLen = 0;
//     size_t                      bytesReturned;
//
//     PAGED_CODE();
//
//     deviceExtension = DeviceGetExtension(Device);
//
//     // check to see if we have a DVD device
//     if (deviceExtension->DeviceAdditionalData.DriveDeviceType != FILE_DEVICE_DVD)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     // we got a DVD drive.
//     bufferLen = DVD_RPC_KEY_LENGTH;
//     copyProtectKey = (PDVD_COPY_PROTECT_KEY)ExAllocatePoolWithTag(PagedPool,
//                                                                   bufferLen,
//                                                                   DVD_TAG_RPC2_CHECK);
//
//     if (copyProtectKey == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     // get the device region
//     RtlZeroMemory (copyProtectKey, bufferLen);
//     copyProtectKey->KeyLength = DVD_RPC_KEY_LENGTH;
//     copyProtectKey->KeyType = DvdGetRpcKey;
//
//     // perform IOCTL_DVD_READ_KEY
//     status = DvdStartSessionReadKey(deviceExtension,
//                                     IOCTL_DVD_READ_KEY,
//                                     NULL,
//                                     copyProtectKey,
//                                     DVD_RPC_KEY_LENGTH,
//                                     copyProtectKey,
//                                     DVD_RPC_KEY_LENGTH,
//                                     &bytesReturned);
//
//     if (NT_SUCCESS(status))
//     {
//         rpcKey = (PDVD_RPC_KEY)copyProtectKey->KeyData;
//
//         // TypeCode of zero means that no region has been set.
//         if (rpcKey->TypeCode == 0)
//         {
//             TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_PNP,
//                         "DVD Initialize (%p): must choose DVD region\n",
//                         Device));
//             deviceExtension->DeviceAdditionalData.PickDvdRegion = 1;
//
//             // set the device region code to be the same as region code on media.
//             if (deviceExtension->DeviceAdditionalData.Mmc.IsCssDvd)
//             {
//                 DevicePickDvdRegion(Device);
//             }
//         }
//     }
//
//     FREE_POOL(copyProtectKey);
//
//     // return status of IOCTL_DVD_READ_KEY will be ignored.
//     return STATUS_SUCCESS;
// }
//
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceIsPortable(
//       PCDROM_DEVICE_EXTENSION   DeviceExtension,
//      PBOOLEAN                  IsPortable
//     )
// /*++
//
// Routine Description:
//
//     This routine checks if the volume is on a portable storage device.
//
// Arguments:
//
//     DeviceExtension - device context
//     IsPortable - device is portable
//
// Return Value:
//
//     NTSTATUS.
//
// --*/
//
// {
//     DEVPROP_BOOLEAN isInternal = DEVPROP_FALSE;
//     BOOLEAN         isPortable = FALSE;
//     ULONG           size       = 0;
//     NTSTATUS        status     = STATUS_SUCCESS;
//     DEVPROPTYPE     type       = DEVPROP_TYPE_EMPTY;
//
//     PAGED_CODE();
//
//     *IsPortable = FALSE;
//
//     // Check to see if the underlying device object is in local machine container
//     status = IoGetDevicePropertyData(DeviceExtension->LowerPdo,
//                                      &DEVPKEY_Device_InLocalMachineContainer,
//                                      0,
//                                      0,
//                                      sizeof(isInternal),
//                                      &isInternal,
//                                      &size,
//                                      &type);
//
//     if (!NT_SUCCESS(status))
//     {
//         goto Cleanup;
//     }
//
//     NT_ASSERT(size == sizeof(isInternal));
//     NT_ASSERT(type == DEVPROP_TYPE_BOOLEAN);
//
//     // Volume is hot-pluggable if the disk pdo container id differs from that of root device
//     if (isInternal == DEVPROP_TRUE)
//     {
//         goto Cleanup;
//     }
//
//     isPortable = TRUE;
//
//     // Examine the bus type to  ensure that this really is a fixed device
//     if (DeviceExtension->DeviceDescriptor->BusType == BusTypeFibre ||
//         DeviceExtension->DeviceDescriptor->BusType == BusTypeiScsi ||
//         DeviceExtension->DeviceDescriptor->BusType == BusTypeRAID)
//     {
//         isPortable = FALSE;
//     }
//
//     *IsPortable = isPortable;
//
// Cleanup:
//
//     return status;
// }
// #endif
//
//
// #pragma warning(pop) // un-sets any local warning changes
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// ============================================================================
// Integrated (banked c2rust) function bodies — DO NOT re-translate.
// Sources: tools/rustconv/.responses/cdrom__init/*.rs
// Preambles dropped; mapped to crate-provided types/helpers/consts.
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn DeviceInitAllocateBuffers(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut senseData: *mut core::ffi::c_void = core::ptr::null_mut();
    CdromDbgPrint1(b"[ALOC] enter sz_fdo=%x\n\0".as_ptr() as *const i8, core::mem::size_of::<CDROM_PRIVATE_FDO_DATA>());

    // PAGED_CODE();

    // allocate a private extension for class data
    if (*DeviceExtension).PrivateFdoData.is_null() {
        (*DeviceExtension).PrivateFdoData = ExAllocatePoolWithTag(
            0, // NonPagedPoolNx: ReactOS #defines it to NonPagedPool (no NX pool here)
            core::mem::size_of::<CDROM_PRIVATE_FDO_DATA>() as SIZE_T,
            0x74767250, /* CDROM_TAG_PRIVATE_DATA */
        ) as *mut CDROM_PRIVATE_FDO_DATA;
    }
    CdromDbgPrint1(b"[ALOC] fdo=%p\n\0".as_ptr() as *const i8, (*DeviceExtension).PrivateFdoData as usize);

    if (*DeviceExtension).PrivateFdoData.is_null() {
        status = -1073741670i32; // STATUS_INSUFFICIENT_RESOURCES
    } else {
        // initialize the struct's various fields.
        core::ptr::write_bytes(
            ((*DeviceExtension).PrivateFdoData) as *mut u8,
            0,
            core::mem::size_of::<CDROM_PRIVATE_FDO_DATA>());
    }
    CdromDbgPrint0(b"[ALOC] fdo zeroed; pre-sense\n\0".as_ptr() as *const i8);

    // Allocate request sense buffer.
    senseData = ExAllocatePoolWithTag(
        0x204, /* NonPagedPoolNxCacheAligned = NonPagedPoolNx (512) + CacheAligned (4) */
        0x12 as SIZE_T,  /* SENSE_BUFFER_SIZE */
        0x53656E73, /* CDROM_TAG_SENSE_INFO */
    ) as *mut core::ffi::c_void;

    if senseData.is_null() {
        // The buffer cannot be allocated.
        status = -1073741670i32; // STATUS_INSUFFICIENT_RESOURCES
    } else {
        // Set the sense data pointer in the device extension.
        (*DeviceExtension).SenseData = senseData as _;
    }

    // Allocate scratch buffer -- Must occur after determining
    // max transfer size, but before other CD specific detection
    // (which relies upon this buffer).
    CdromDbgPrint1(b"[ALOC] pre-scratch sense=%p\n\0".as_ptr() as *const i8, senseData as usize);
    if ScratchBuffer_Allocate(DeviceExtension) == 0 {
        status = -1073741670i32; // STATUS_INSUFFICIENT_RESOURCES
    }
    CdromDbgPrint1(b"[ALOC] done st=%x\n\0".as_ptr() as *const i8, status as usize);

    status
}

#[no_mangle]
pub unsafe extern "system" fn DeviceRetrieveScsiAddress(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    ScsiAddress: PSCSI_ADDRESS,
) -> NTSTATUS {
    // PAGED_CODE();
    if DeviceExtension.is_null() || ScsiAddress.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let mut outputDescriptor: WDF_MEMORY_DESCRIPTOR = core::mem::zeroed();
    WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(
        core::ptr::addr_of_mut!(outputDescriptor),
        ScsiAddress as *mut core::ffi::c_void,
        core::mem::size_of::<SCSI_ADDRESS>() as u32,
    );
    let status = WdfIoTargetSendIoctlSynchronously(
        (*DeviceExtension).IoTarget,
        core::ptr::null_mut(),
        IOCTL_SCSI_GET_ADDRESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(outputDescriptor),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !ntkrnl_sys::base::NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"DeviceRetrieveScsiAddress: Get Address failed %lx\n\0".as_ptr() as *const i8,
            status,
        );
    } else {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_INIT,
            b"GetAddress: Port %x, Path %x, Target %x, Lun %x\n\0".as_ptr() as *const i8,
            (*ScsiAddress).PortNumber as u32,
            (*ScsiAddress).PathId as u32,
            (*ScsiAddress).TargetId as u32,
            (*ScsiAddress).Lun as u32,
        );
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn DeviceRetrieveDescriptorsAndTransferLength(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut propertyId: STORAGE_PROPERTY_ID;
    PAGED_CODE!();

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        propertyId = StorageAdapterProperty;
        status = DeviceRetrieveDescriptor(
            (*DeviceExtension).Device,
            &mut propertyId,
            core::ptr::addr_of_mut!((*DeviceExtension).AdapterDescriptor)
                as *mut *mut STORAGE_DESCRIPTOR_HEADER,
        );
    }
    if ntkrnl_sys::base::NT_SUCCESS(status) {
        propertyId = StorageDeviceProperty;
        status = DeviceRetrieveDescriptor(
            (*DeviceExtension).Device,
            &mut propertyId,
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceDescriptor)
                as *mut *mut STORAGE_DESCRIPTOR_HEADER,
        );
    }
    if ntkrnl_sys::base::NT_SUCCESS(status) {
        propertyId = StorageDevicePowerProperty;
        let _ = DeviceRetrieveDescriptor(
            (*DeviceExtension).Device,
            &mut propertyId,
            core::ptr::addr_of_mut!((*DeviceExtension).PowerDescriptor)
                as *mut *mut STORAGE_DESCRIPTOR_HEADER,
        );
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        DeviceComputeTransferLengths(DeviceExtension);
    } else {
        TracePrint!(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_PNP,
            b"DeviceRetrieveDescriptorsAndTransferLength failed %lx\n\0".as_ptr() as *const i8,
            status,
        );
    }

    status
}

#[inline(never)]
unsafe fn DeviceComputeTransferLengths(DeviceExtension: PCDROM_DEVICE_EXTENSION) {
    CdromDbgPrint3(b"[DCTL] enter adapter=%p maxpages=%x maxxfer=%x\n\0".as_ptr() as *const i8,
        (*DeviceExtension).AdapterDescriptor as usize,
        if (*DeviceExtension).AdapterDescriptor.is_null() { 0xBAD } else { (*(*DeviceExtension).AdapterDescriptor).MaximumPhysicalPages as usize },
        if (*DeviceExtension).AdapterDescriptor.is_null() { 0xBAD } else { (*(*DeviceExtension).AdapterDescriptor).MaximumTransferLength as usize });
    let mut max_aligned_transfer: u32 =
        (*(*DeviceExtension).AdapterDescriptor).MaximumPhysicalPages;
    let mut max_unaligned_transfer: u32 =
        (*(*DeviceExtension).AdapterDescriptor).MaximumPhysicalPages;

    if max_unaligned_transfer > 1 {
        max_unaligned_transfer -= 1;
    }

    if max_aligned_transfer >= (u32::MAX / (PAGE_SIZE as u32)) {
        max_aligned_transfer = u32::MAX;
    } else {
        max_aligned_transfer *= PAGE_SIZE as u32;
    }

    if max_unaligned_transfer >= (u32::MAX / (PAGE_SIZE as u32)) {
        max_unaligned_transfer = u32::MAX;
    } else {
        max_unaligned_transfer *= PAGE_SIZE as u32;
    }

    max_aligned_transfer = min(
        max_aligned_transfer,
        (*(*DeviceExtension).AdapterDescriptor).MaximumTransferLength,
    );
    max_unaligned_transfer = min(
        max_unaligned_transfer,
        (*(*DeviceExtension).AdapterDescriptor).MaximumTransferLength,
    );

    max_aligned_transfer = max(max_aligned_transfer, PAGE_SIZE as u32);
    max_unaligned_transfer = max(max_unaligned_transfer, PAGE_SIZE as u32);

    (*DeviceExtension)
        .DeviceAdditionalData
        .MaxPageAlignedTransferBytes = max_aligned_transfer;
    (*DeviceExtension)
        .DeviceAdditionalData
        .MaxUnalignedTransferBytes = max_unaligned_transfer;
    CdromDbgPrint2(b"[DCTL] wrote maxpat=%x readback=%x\n\0".as_ptr() as *const i8,
        max_aligned_transfer as usize,
        (*DeviceExtension).DeviceAdditionalData.MaxPageAlignedTransferBytes as usize);
}

#[no_mangle]
pub unsafe extern "system" fn DeviceScanSpecialDevices(DeviceExtension: PCDROM_DEVICE_EXTENSION) {
    // PAGED_CODE();  (no-op)
    DeviceScanForSpecial(DeviceExtension, core::ptr::addr_of_mut!(CdromHackItems), Some(ScanForSpecialHandler));

    // Hack flag checks
    if ((*DeviceExtension).DeviceAdditionalData.HackFlags & CDROM_HACK_TOSHIBA_SD_W1101 as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
            b"DeviceScanSpecialDevices: Found Toshiba SD-W1101 DVD-RAM -- This drive will *NOT* support DVD-ROM playback.\n\0"
                .as_ptr() as *const i8);
    } else if ((*DeviceExtension).DeviceAdditionalData.HackFlags & CDROM_HACK_HITACHI_GD_2000 as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
            b"DeviceScanSpecialDevices: Found Hitachi GD-2000\n\0"
                .as_ptr() as *const i8);

        (*DeviceExtension).DeviceAdditionalData.ErrorHandler =
            core::mem::transmute::<*const core::ffi::c_void, PCDROM_ERROR_HANDLER>(
                DeviceErrorHandlerForHitachiGD2000 as *const core::ffi::c_void,
            );
        (*DeviceExtension).DeviceFlags |= DEV_SAFE_START_UNIT as u16;
    } else if ((*DeviceExtension).DeviceAdditionalData.HackFlags & CDROM_HACK_FUJITSU_FMCD_10x as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
        (*DeviceExtension).TimeOutValue = 20;
    } else if ((*DeviceExtension).DeviceAdditionalData.HackFlags & CDROM_HACK_DEC_RRD as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_INIT,
            b"DeviceScanSpecialDevices:  Found DEC RRD.\n\0"
                .as_ptr() as *const i8);

        (*DeviceExtension).DeviceAdditionalData.IsDecRrd = 1; // TRUE

        // Allocate mode parameters
        let mode_parameters = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<MODE_PARM_READ_WRITE_DATA>() as SIZE_T,
            CDROM_TAG_MODE_DATA,
        ) as *mut MODE_PARM_READ_WRITE_DATA;

        if mode_parameters.is_null() {
            return;
        }

        RtlZeroMemory(
            mode_parameters as *mut core::ffi::c_void,
            core::mem::size_of::<MODE_PARM_READ_WRITE_DATA>(),
        );

        // Prepare SCSI request block on the stack (zero-initialised)
        let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();

        // Set block length to 2KB
        (*mode_parameters).ParameterListHeader.BlockDescriptorLength =
            core::mem::size_of::<MODE_PARAMETER_BLOCK>() as u8;
        (*mode_parameters).ParameterListBlock.BlockLength[0] = 0x00; // MSB
        (*mode_parameters).ParameterListBlock.BlockLength[1] = 0x08;
        (*mode_parameters).ParameterListBlock.BlockLength[2] = 0x00; // LSB

        // Build the MODE SELECT CDB
        srb.CdbLength = 6;
        srb.TimeOutValue = (*DeviceExtension).TimeOutValue;

        let cdb = core::ptr::addr_of_mut!(srb.Cdb) as *mut CDB;
        (*cdb).MODE_SELECT.as_mut().set_PFBit(1);
        (*cdb).MODE_SELECT.as_mut().OperationCode = SCSIOP_MODE_SELECT;
        (*cdb).MODE_SELECT.as_mut().ParameterListLength = HITACHI_MODE_DATA_SIZE as u8;

        // Send synchronous SRB
        let status = DeviceSendSrbSynchronously(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(srb),
            mode_parameters as *mut core::ffi::c_void,
            core::mem::size_of::<MODE_PARM_READ_WRITE_DATA>() as u32,
            1, // TRUE
            core::ptr::null_mut(),
        );

        if !ntkrnl_sys::base::NT_SUCCESS(status) {
            TracePrint!(TRACE_LEVEL_WARNING, TRACE_FLAG_INIT,
                b"DeviceScanSpecialDevices: Setting DEC RRD to 2K blocksize failed [%x]\n\0"
                    .as_ptr() as *const i8,
                status as u32);
        }

        ExFreePool(mode_parameters as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "system" fn DeviceIsPortable(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    IsPortable: PBOOLEAN,
) -> NTSTATUS {
    let mut isInternal: i32 = DEVPROP_FALSE as i32;
    let mut isPortable: u8 = 0; // FALSE
    let mut size: u32 = 0;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut type_: u32 = DEVPROP_TYPE_EMPTY;

    PAGED_CODE!();

    'Cleanup: loop {
        *IsPortable = 0; // FALSE

        status = IoGetDevicePropertyData(
            (*DeviceExtension).LowerPdo as *mut core::ffi::c_void,
            core::ptr::addr_of!(DEVPKEY_Device_InLocalMachineContainer),
            0,
            0,
            core::mem::size_of::<i32>() as u32,
            core::ptr::addr_of_mut!(isInternal) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(size),
            core::ptr::addr_of_mut!(type_),
        );

        if !ntkrnl_sys::base::NT_SUCCESS(status) {
            break 'Cleanup;
        }

        NT_ASSERT((size as usize == core::mem::size_of::<i32>()) as i32);
        NT_ASSERT((type_ == DEVPROP_TYPE_BOOLEAN) as i32);

        if isInternal == DEVPROP_TRUE as i32 {
            break 'Cleanup;
        }

        isPortable = 1; // TRUE

        if (*(*DeviceExtension).DeviceDescriptor).BusType == BusTypeFibre
            || (*(*DeviceExtension).DeviceDescriptor).BusType == BusTypeiScsi
            || (*(*DeviceExtension).DeviceDescriptor).BusType == BusTypeRAID
        {
            isPortable = 0; // FALSE
        }

        *IsPortable = isPortable;
        break 'Cleanup;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn DeviceInitMmcContext(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    PAGED_CODE!();

    let mut status: i32 = 0; // STATUS_SUCCESS

    (*DeviceExtension).DeviceAdditionalData.Mmc.IsMmc = 0u8;
    (*DeviceExtension).DeviceAdditionalData.Mmc.IsAACS = 0u8;
    (*DeviceExtension).DeviceAdditionalData.Mmc.IsWriter = 0u8;
    (*DeviceExtension).DeviceAdditionalData.Mmc.WriteAllowed = 0u8;
    (*DeviceExtension).DeviceAdditionalData.Mmc.IsCssDvd = 0u8;
    (*DeviceExtension).DeviceAdditionalData.DriveDeviceType = FILE_DEVICE_CD_ROM;

    if status >= 0 {
        status = DeviceGetMmcSupportInfo(
            DeviceExtension,
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData.Mmc.IsMmc),
        );
        if status < 0 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"DeviceInitMmcContext: Failed to get the support info for GET CONFIGURATION command, failng %!STATUS!\n\0"
                    .as_ptr() as *const i8,
                status,
            );
            (*DeviceExtension).DeviceAdditionalData.Mmc.IsMmc = 0u8;
            status = 0;
        }
    }

    if status >= 0 && (*DeviceExtension).DeviceAdditionalData.Mmc.IsMmc != 0 {
        status = DeviceAllocateMmcResources((*DeviceExtension).Device);
    }

    if status >= 0 && (*DeviceExtension).DeviceAdditionalData.Mmc.IsMmc != 0 {
        let mut header: *mut FEATURE_DATA_CD_READ;
        let mut validationSchema: u32 = 0;
        let mut blockingFactor: u32 = 0;

        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"DeviceInitMmcContext: FDO %p GET CONFIGURATION buffer %p\n\0".as_ptr() as *const i8,
            (*DeviceExtension).Device as *const core::ffi::c_void,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *const core::ffi::c_void,
        );

        DevicePrintAllFeaturePages(
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *const u8,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
        );

        header = DeviceFindFeaturePage(
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *mut u8,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
            FeatureAACS,
        );
        if !header.is_null() {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_INIT,
                b"DeviceInitMmcContext: Reporting AACS support for device due to GET CONFIGURATION showing support\n\0"
                    .as_ptr() as *const i8,
            );
            (*DeviceExtension).DeviceAdditionalData.Mmc.IsAACS = 1u8;
        }

        #[cfg(feature = "ENABLE_AACS_TESTING")]
        {
            (*DeviceExtension).DeviceAdditionalData.Mmc.IsAACS = 1u8;
        }

        header = DeviceFindFeaturePage(
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *mut u8,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
            FeatureDvdRead,
        );
        if !header.is_null() {
            (*DeviceExtension).DeviceAdditionalData.DriveDeviceType = FILE_DEVICE_DVD;
        }

        DeviceUpdateMmcWriteCapability(
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *const u8,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
            0,
            core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData.Mmc.IsWriter),
            core::ptr::addr_of_mut!(validationSchema),
            core::ptr::addr_of_mut!(blockingFactor),
        );

        header = DeviceFindFeaturePage(
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *mut u8,
            (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
            FeatureDvdCSS,
        );

        (*DeviceExtension).DeviceAdditionalData.Mmc.IsCssDvd =
            if !header.is_null() && (*(header as *mut FEATURE_HEADER)).Current() != 0 {
                1u8
            } else {
                0u8
            };

        (*DeviceExtension).DeviceAdditionalData.Mmc.UpdateState = CdromMmcUpdateRequired as i32;
        (*DeviceExtension).DeviceAdditionalData.ErrorHandler =
            core::mem::transmute::<*const core::ffi::c_void, PCDROM_ERROR_HANDLER>(
                DeviceErrorHandlerForMmc as *const core::ffi::c_void,
            );

        (*DeviceExtension).DeviceFlags |= DEV_SAFE_START_UNIT as u16;

        DeviceSetRawReadInfo(DeviceExtension);
    }

    if (*DeviceExtension).DeviceAdditionalData.Mmc.IsMmc == 0 {
        let mut deviceCharacteristics: u32 =
            WdfDeviceGetCharacteristics((*DeviceExtension).Device);
        deviceCharacteristics |= FILE_READ_ONLY_DEVICE;
        WdfDeviceSetCharacteristics((*DeviceExtension).Device, deviceCharacteristics);
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"DeviceInitMmcContext: FDO %p Device is not an MMC compliant device, so setting to read-only (legacy) mode\0"
                .as_ptr() as *const i8,
            (*DeviceExtension).Device as *const core::ffi::c_void,
        );
    }

    if (*DeviceExtension).DeviceAdditionalData.DriveDeviceType == FILE_DEVICE_DVD {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"DeviceInitMmcContext: DVD Devices require START UNIT\n\0".as_ptr() as *const i8,
        );
        (*DeviceExtension).DeviceFlags |= DEV_SAFE_START_UNIT as u16;
    } else if ((*(*DeviceExtension).DeviceDescriptor).BusType != BusTypeScsi)
        && ((*(*DeviceExtension).DeviceDescriptor).BusType != BusTypeAtapi)
        && ((*(*DeviceExtension).DeviceDescriptor).BusType != BusTypeUnknown)
    {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_INIT,
            b"DeviceInitMmcContext: Devices for newer buses require START UNIT\n\0".as_ptr() as *const i8,
        );
        (*DeviceExtension).DeviceFlags |= DEV_SAFE_START_UNIT as u16;
    }

    status
}

// ---- integrated banked body: DeviceGetMmcSupportInfo ----
#[no_mangle]
pub unsafe extern "system" fn DeviceGetMmcSupportInfo(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    IsMmcDevice: *mut u8, // PBOOLEAN
) -> NTSTATUS {
    let mut status: i32;
    let mut size: u32;
    let mut previously_failed: u32;

    // PAGED_CODE() ignored

    *IsMmcDevice = 0; // FALSE

    // read the registry in case the drive failed previously,
    // and a timeout is occurring. (The three DeviceGetParameter reads in this
    // function were dropped: the flags DeviceSetParameter persists below were
    // never consulted, so a non-MMC drive re-timed-out on every start.)
    previously_failed = 0; // FALSE
    DeviceGetParameter(
        DeviceExtension as *mut core::ffi::c_void,
        CDROM_SUBKEY_NAME as *mut u16,
        CDROM_NON_MMC_DRIVE_NAME as *mut u16,
        &mut previously_failed,
    );
    if previously_failed != 0 {
        (*DeviceExtension).DeviceAdditionalData.HackFlags |=
            CDROM_HACK_BAD_GET_CONFIG_SUPPORT as ntkrnl_sys::ddk::ULONG_PTR;
    }

    // read from the registry in case the drive reports bad profile lengths
    previously_failed = 0;
    DeviceGetParameter(
        DeviceExtension as *mut core::ffi::c_void,
        CDROM_SUBKEY_NAME as *mut u16,
        CDROM_NON_MMC_VENDOR_SPECIFIC_PROFILE as *mut u16,
        &mut previously_failed,
    );
    if previously_failed != 0 {
        (*DeviceExtension).DeviceAdditionalData.HackFlags |=
            CDROM_HACK_BAD_VENDOR_PROFILES as ntkrnl_sys::ddk::ULONG_PTR;
    }

    // check for the ProfileList feature to determine if the drive is MMC compliant
    // and set the "size" local variable to total GetConfig data size available.
    {
        let mut local_header: GET_CONFIGURATION_HEADER = core::mem::zeroed();
        let mut usable: u32 = 0;

        status = DeviceGetConfiguration(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            &mut local_header as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32,
            &mut usable,
            (FeatureProfileList) as _,
            SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL,
        );

        if status == -1073741436      // STATUS_INVALID_DEVICE_REQUEST 0xC0000010
            || status == -1073741801  // STATUS_NO_MEDIA_IN_DEVICE 0xC0000013
            || status == -1073741666  // STATUS_IO_DEVICE_ERROR 0xC0000185
            || status == -1073741766  // STATUS_IO_TIMEOUT 0xC00000B6
        {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"GetConfiguration Failed (%x), device %p not mmc-compliant\n\0".as_ptr() as *const i8,
                status,
                (*DeviceExtension).DeviceObject,
            );

            previously_failed = 1; // TRUE
            DeviceSetParameter(
                DeviceExtension,
                CDROM_SUBKEY_NAME,
                CDROM_NON_MMC_DRIVE_NAME,
                previously_failed,
            );

            return 0; // STATUS_SUCCESS
        } else if status < 0 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"GetConfiguration Failed, status %x -- defaulting to -ROM\n\0".as_ptr() as *const i8,
                status,
            );
            return 0;
        } else if usable < core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32 {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"GetConfiguration Failed, returned only %x bytes!\n\0".as_ptr() as *const i8,
                usable,
            );
            previously_failed = 1;
            DeviceSetParameter(
                DeviceExtension,
                CDROM_SUBKEY_NAME,
                CDROM_NON_MMC_DRIVE_NAME,
                previously_failed,
            );
            return 0;
        }

        // combine DataLength bytes (big-endian) into size
        size = ((local_header.DataLength[0] as u32) << 24)
            | ((local_header.DataLength[1] as u32) << 16)
            | ((local_header.DataLength[2] as u32) << 8)
            | ((local_header.DataLength[3] as u32) << 0);

        if size <= 4 || size.wrapping_add(4) < size {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"GetConfiguration Failed, claims MMC support but doesn't correctly return config length! (%x)\n\0".as_ptr() as *const i8,
                size,
            );
            previously_failed = 1;
            DeviceSetParameter(
                DeviceExtension,
                CDROM_SUBKEY_NAME,
                CDROM_NON_MMC_DRIVE_NAME,
                previously_failed,
            );
            return 0;
        } else if (size % 4) != 0 {
            if (size % 2) != 0 {
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_INIT,
                    b"GetConfiguration Failed, returned odd number of bytes %x!\n\0".as_ptr() as *const i8,
                    size,
                );
                previously_failed = 1;
                DeviceSetParameter(
                    DeviceExtension,
                    CDROM_SUBKEY_NAME,
                    CDROM_NON_MMC_DRIVE_NAME,
                    previously_failed,
                );
                return 0;
            } else {
                let hack_flags = (*DeviceExtension).DeviceAdditionalData.HackFlags;
                if (hack_flags & CDROM_HACK_BAD_VENDOR_PROFILES as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
                    // we've already caught this and ASSERT'd once, so don't do it again
                } else {
                    TracePrint!(
                        TRACE_LEVEL_WARNING,
                        TRACE_FLAG_INIT,
                        b"GetConfiguration returned a size that is not per spec (%x bytes), this is probably because of a vendor specific data header with a size not divisible by 4.\n\0".as_ptr() as *const i8,
                        size,
                    );
                    previously_failed = 1;
                    DeviceSetParameter(
                        DeviceExtension,
                        CDROM_SUBKEY_NAME,
                        CDROM_NON_MMC_VENDOR_SPECIFIC_PROFILE,
                        previously_failed,
                    );
                }
            }
        }

        size += 4; // sizeof DataLength fields (4 bytes)
    }

    *IsMmcDevice = 1; // TRUE

    // This code doesn't handle total get config size over 64k
    NT_ASSERT((size <= MAXUSHORT) as i32);

    // Check for SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE support in the device.
    {
        let feature_size = core::mem::size_of::<GET_CONFIGURATION_HEADER>()
            + core::mem::size_of::<FEATURE_HEADER>();
        let mut usable: u32 = 0;

        let config_buffer = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            feature_size as SIZE_T,
            CDROM_TAG_GET_CONFIG,
        ) as *mut GET_CONFIGURATION_HEADER;

        if config_buffer.is_null() {
            return -1073741670; // STATUS_INSUFFICIENT_RESOURCES
        }

        // read the registry in case the drive failed previously,
        // and a timeout is occurring.
        previously_failed = 0;
        DeviceGetParameter(
            DeviceExtension as *mut core::ffi::c_void,
            CDROM_SUBKEY_NAME as *mut u16,
            CDROM_TYPE_ONE_GET_CONFIG_NAME as *mut u16,
            &mut previously_failed,
        );
        if previously_failed != 0 {
            (*DeviceExtension).DeviceAdditionalData.HackFlags |=
                CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG as ntkrnl_sys::ddk::ULONG_PTR;
            ExFreePool(config_buffer as *mut core::ffi::c_void);
            return 0;
        }

        // Get only the config and feature header
        status = DeviceGetConfiguration(
            (*DeviceExtension).Device as *mut core::ffi::c_void,
            config_buffer as *mut core::ffi::c_void,
            feature_size as u32,
            &mut usable,
            (FeatureProfileList) as _,
            SCSI_GET_CONFIGURATION_REQUEST_TYPE_ONE,
        );

        if status < 0 || (usable as usize) < feature_size {
            TracePrint!(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_INIT,
                b"Type One GetConfiguration Failed. Usable buffer size: %d\n\0".as_ptr() as *const i8,
                usable,
            );
            previously_failed = 1;
        } else {
            let feature_header: *const FEATURE_HEADER;
            let mut total_available_bytes: u32 = 0;
            let expected_available_bytes: u32;

            REVERSE_BYTES(
                &mut total_available_bytes,
                &(*config_buffer).DataLength as *const _ as *const u8,
            );
            total_available_bytes += core::mem::offset_of!(GET_CONFIGURATION_HEADER, DataLength) as u32
                + core::mem::size_of::<u32>() as u32; // RTL_SIZEOF_THROUGH_FIELD

            feature_header = (config_buffer as *const u8)
                .add(core::mem::size_of::<GET_CONFIGURATION_HEADER>())
                as *const FEATURE_HEADER;

            expected_available_bytes = core::mem::size_of::<GET_CONFIGURATION_HEADER>() as u32
                + core::mem::size_of::<FEATURE_HEADER>() as u32
                + (*feature_header).AdditionalLength as u32;

            if total_available_bytes > expected_available_bytes {
                // Device is returning more than required size. Most likely the device
                // is returning TYPE ALL data. Set the flag to use TYPE ALL for TYPE ONE
                // requests
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_INIT,
                    b"Type One GetConfiguration Failed. Device returned %d bytes instead of %d bytes\n\0".as_ptr() as *const i8,
                    size,
                    feature_size as u32,
                );
                previously_failed = 1;
            }
        }

        ExFreePool(config_buffer as *mut core::ffi::c_void);

        if previously_failed != 0 {
            DeviceSetParameter(
                DeviceExtension,
                CDROM_SUBKEY_NAME,
                CDROM_TYPE_ONE_GET_CONFIG_NAME,
                previously_failed,
            );

            (*DeviceExtension).DeviceAdditionalData.HackFlags |=
                CDROM_HACK_BAD_TYPE_ONE_GET_CONFIG as ntkrnl_sys::ddk::ULONG_PTR;
        }
    }

    status
}

#[allow(unused_imports)] use crate::{NT_ASSERT};
use crate::types::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::common::strlen;
// The crate's own functions: the definition is the declaration.
use crate::common::DeviceGetParameter;
use crate::common::DeviceReleaseQueue;
use crate::common::DeviceSendNotification;
use crate::common::DeviceSetParameter;
use crate::ddk_shims::IsVolumeMounted;
use crate::common::RequestClearSendTime;
use crate::ioctl::RequestHandleEventNotification;
use crate::common::RequestSend;
use crate::sense::RequestSenseInfoInterpret;
use crate::ddk_shims::TEST_FLAG;
use crate::ddk_shims::WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE;
use crate::ddk_shims::WDF_WORKITEM_CONFIG_INIT;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL, _UNICODE_STRING, HANDLE};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, _STORAGE_BUS_TYPE_BusTypeAtapi as BusTypeAtapi, CLASS_MEDIA_CHANGE_CONTEXT, _KWAIT_REASON_Executive, IoAllocateIrp, IoAllocateMdl, IoFreeIrp, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoReuseIrp, IoSetCompletionRoutine, IoSetNextIrpStackLocation, KeInitializeMutex, KeQueryTimeIncrement, KeReleaseMutex, LARGE_INTEGER, LPCSTR, MmBuildMdlForNonPagedPool, POWER_PLATFORM_INFORMATION, PPREVENT_MEDIA_REMOVAL, PREVENT_MEDIA_REMOVAL, STORAGE_DEVICE_DESCRIPTOR, PSTORAGE_DEVICE_DESCRIPTOR, PUNICODE_STRING, _POWER_INFORMATION_LEVEL_PlatformInformation as PlatformInformation, RTL_QUERY_REGISTRY_TABLE, RtlAnsiStringToUnicodeString, RtlCompareMemory, RtlFreeUnicodeString, RtlQueryRegistryValues, _KWAIT_REASON_UserRequest, ZwPowerInformation};
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{copy_nonoverlapping, write_bytes, null_mut};

extern "C" {
    static GUID_IO_MEDIA_ARRIVAL: GUID;
    static GUID_IO_MEDIA_REMOVAL: GUID;
    static CDROM_DEVICE_EXTENSION_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
    static CDROM_REQUEST_CONTEXT_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
    fn SRB_STATUS(srb_status: u8) -> u32;
    fn PORT_ALLOCATED_SENSE(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Srb: *mut SCSI_REQUEST_BLOCK) -> u8;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER(DeviceExtension: *mut CDROM_DEVICE_EXTENSION, Srb: *mut SCSI_REQUEST_BLOCK);
}
unsafe extern "system" {
    fn SET_FLAG(flags: *mut u32, flag: u32);
}
unsafe extern "system" {
    fn WDF_OBJECT_ATTRIBUTES_INIT(attributes: *mut WDF_OBJECT_ATTRIBUTES);
}
unsafe extern "system" {
}
extern "system" {
}
extern "C" {
    fn CdromMcnPollMark();
    fn NT_SUCCESS(status: NTSTATUS) -> bool;
}
extern "system" {
    fn WDF_REQUEST_REUSE_PARAMS_INIT(params: *mut WDF_REQUEST_REUSE_PARAMS, flags: u32, status: NTSTATUS);
    fn DeviceGetExtension(device: WDFDEVICE) -> PCDROM_DEVICE_EXTENSION;
    fn RtlInitUnicodeString( DestinationString: *mut ntkrnl_sys::ddk::_UNICODE_STRING, SourceString: *const u16 );
    fn KeWaitForMutexObject( Mutex: *mut core::ffi::c_void, WaitReason: i32, WaitMode: i32, Alertable: i32, Timeout: *const core::ffi::c_void, ) -> i32;
    static GUID_IO_DEVICE_EXTERNAL_REQUEST: GUID;
    static GUID_IO_MEDIA_EJECT_REQUEST: GUID;
    static GUID_IO_DEVICE_BECOMING_READY: GUID;
// PULONG
    fn PoRegisterPowerSettingCallback( DeviceObject: *mut core::ffi::c_void, SettingGuid: *const core::ffi::c_void, Callback: *mut core::ffi::c_void, Context: *mut core::ffi::c_void, Handle: *mut *mut core::ffi::c_void, ) -> NTSTATUS;
}

// These two are cdecl, NOT stdcall — they were mis-declared in the `extern "system"` block
// above, while their DEFINITIONS (common.rs / pnppower.rs) are `extern "C"` and cdrom.h
// declares them with no NTAPI. On i386 `extern "system"` is stdcall, so those decls named
// `_Name@12` while the definitions emitted `_Name` — an i386-only mismatch that AGENTS.md
// calls out ("links silently on amd64/arm64 but breaks i386"). It stayed latent because the
// old no-op `TracePrint` let LLVM fold these call paths away; giving TracePrint a real
// out-of-line body surfaced them as `undefined symbol: _DeviceSendRequestSynchronously@12`.
// Had this code ever run on i386 as declared, the stdcall callee-cleanup would have
// corrupted the caller's stack.
// STDCALL (NTAPI) — kernel/WDF exports and the in-crate fns whose DEFINITIONS are
// `extern "system"`. On i386 these decorate as `_Name@N`.
extern "system" {
    fn IoFreeMdl(Mdl: PMDL);
    fn KeGetCurrentIrql() -> KIRQL;
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
    fn PoUnregisterPowerSettingCallback(Handle: *mut core::ffi::c_void) -> NTSTATUS;
    fn FileObjectGetContext( FileObject: WDFFILEOBJECT, ) -> *mut FILE_OBJECT_CONTEXT;
}

// CDECL — cdrom's OWN helpers. Every one of these is DEFINED `extern "C"` elsewhere in this
// crate (common.rs / pnppower.rs / …) and declared with no NTAPI in cdrom.h, but they were
// all sitting in the `extern "system"` block above, so on i386 the call sites emitted
// `_Name@N` while the definitions emitted `_Name`. See the block comment above.
// NOT DevicePowerSettingCallback: cdrom.h declares it as `POWER_SETTING_CALLBACK`, whose
// typedef (potypes.h) is NTAPI, and PoRegisterPowerSettingCallback calls it stdcall. Its
// definition is `extern "system"`, and this module uses it rather than redeclaring it.
use crate::pnppower::DevicePowerSettingCallback;
extern "C" {
    fn DeviceSendRequestSynchronously( Device: WDFDEVICE, Request: WDFREQUEST, WaitForCompletion: BOOLEAN, ) -> NTSTATUS;
    fn FREE_POOL(ptr: *mut core::ffi::c_void);
    fn CountOfSetBitsUChar(byte: u8) -> u8;
    fn DeviceMarkActive(Extension: *mut CDROM_DEVICE_EXTENSION, Active: BOOLEAN, b: BOOLEAN);
    fn DeviceZPODDIsInHomePosition(Extension: *mut CDROM_DEVICE_EXTENSION) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
pub const GESN_TIMEOUT_VALUE: u32 = 0x4;
pub const GESN_BUFFER_SIZE: u32 = 0x8;
pub const GESN_DEVICE_BUSY_LOWER_THRESHOLD_100_MS: u32 = 2;
pub const MAXIMUM_IMMEDIATE_MCN_RETRIES: u32 = 0x20;
pub static MCN_REG_SUBKEY_NAME: [u16; 24] = [
    0x004D, 0x0065, 0x0064, 0x0069, 0x0061, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065,
    0x004E, 0x006F, 0x0074, 0x0069, 0x0066, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static MCN_REG_AUTORUN_DISABLE_INSTANCE_NAME: [u16; 17] = [
    0x0041, 0x006C, 0x0077, 0x0061, 0x0079, 0x0073, 0x0044, 0x0069, 0x0073, 0x0061, 0x0062,
    0x006C, 0x0065, 0x004D, 0x0043, 0x004E, 0x0000,
];
pub static MCN_REG_AUTORUN_ENABLE_INSTANCE_NAME: [u16; 16] = [
    0x0041, 0x006C, 0x0077, 0x0061, 0x0079, 0x0073, 0x0045, 0x006E, 0x0061, 0x0062,
    0x006C, 0x0065, 0x004D, 0x0043, 0x004E, 0x0000,
];

// Trace levels/flags (WPP tracing gates; only used to gate DbgPrint-style output)
const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
const TRACE_FLAG_MCN: u32 = 12; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// MEDIA_CHANGE_DETECTION_STATE enum values (bindgen-prefixed)
use crate::types::_MEDIA_CHANGE_DETECTION_STATE_MediaUnknown as MediaUnknown;
use crate::types::_MEDIA_CHANGE_DETECTION_STATE_MediaPresent as MediaPresent;
use crate::types::_MEDIA_CHANGE_DETECTION_STATE_MediaNotPresent as MediaNotPresent;
use crate::types::_MEDIA_CHANGE_DETECTION_STATE_MediaUnavailable as MediaUnavailable;

const WDF_REQUEST_SEND_OPTION_SYNCHRONOUS: u32 = 0x02;

// Constants needed by this module
const KEY_ALL_ACCESS: u32 = 0xF003F;
const CDROM_TAG_AUTORUN_DISABLE: u32 = 0x41436353; // 'DCAT'
const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5_u32 as i32;
// cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;

const PARAMETER_KEY_NAME: [u16; 11] = [
    0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073, 0
];
const AUTORUN_VALUE_NAME: [u16; 8] = [
    0x0041, 0x0075, 0x0074, 0x006F, 0x0072, 0x0075, 0x006E, 0
];

const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
const IOCTL_MCN_SYNC_FAKE_IOCTL: u32 = 0x0007C0C8;  // = IOCTL_DISK_UPDATE_DRIVE_SIZE
// (cdrom.h:181, CTL_CODE(IOCTL_DISK_BASE, 0x32, METHOD_BUFFERED, READ|WRITE)). NOT a
// placeholder: autorun.rs WRITES this into IoControlCode and cdrom.rs matches on it,
// so an invented value means the MCN sync IRP is never recognised.

const CDROM_HACK_MSFT_VIRTUAL_ODD: u32 = 0x2000;  // was 0x00000001 (LLM-invented); bindgen-correct
const WdfExecutionLevelInheritFromParent: u32 = 1;
const DO_VERIFY_VOLUME: u32 = 0x00000002;
const STATUS_MEDIA_CHANGED: NTSTATUS = 0x8000001Cu32 as i32; // STATUS_MEDIA_CHANGED
const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as i32;
const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as i32;
const STATUS_DEVICE_PROTOCOL_ERROR: NTSTATUS = 0xC0000186u32 as i32;

// STATUS_INVALID_DEVICE_STATE (0xC0000184)
const STATUS_INVALID_DEVICE_STATE: NTSTATUS = (0xC0000184u32 as i32) as NTSTATUS;

// Constants
const CDROM_TAG_GESN: u32 = 0x47436353; // 'GESN' tag (little-endian)
// cdrom.h:68: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNxCacheAligned: u32 = 4;

// Registry key/value name strings (wide strings, matching DeviceGetParameter/DeviceSetParameter PWSTR args)
static CLASSP_REG_SUBKEY_NAME_W: [u16; 6] = [b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, 0];
static CLASSP_REG_MMC_DETECTION_VALUE_NAME_W: [u16; 13] = [
    b'M' as u16, b'm' as u16, b'c' as u16, b'D' as u16, b'e' as u16, b't' as u16,
    b'e' as u16, b'c' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, 0,
];
const CLASSP_REG_SUBKEY_NAME: *mut u16 = CLASSP_REG_SUBKEY_NAME_W.as_ptr() as *mut u16;
const CLASSP_REG_MMC_DETECTION_VALUE_NAME: *mut u16 = CLASSP_REG_MMC_DETECTION_VALUE_NAME_W.as_ptr() as *mut u16;

// Flag constants
const FDO_HACK_GESN_IS_BAD: u32 = 0x2;               // assume typical bit  // was 0x00000001 (LLM-invented); bindgen-correct
const NOTIFICATION_MEDIA_STATUS_CLASS_MASK: u32 = 0x10;  // was 0x00000001 (LLM-invented); bindgen-correct
const NOTIFICATION_DEVICE_BUSY_CLASS_MASK: u32 = 0x40;  // was 0x00000002 (LLM-invented); bindgen-correct
const NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK: u32 = 0x2;  // was 0x00000004 (LLM-invented); bindgen-correct
const NOTIFICATION_EXTERNAL_REQUEST_CLASS_MASK: u32 = 0x00000008;
const FDO_HACK_GESN_IGNORE_OPCHANGE: u32 = 0x00000010;

// SRB flags/status
const SRB_STATUS_MASK: u8 = 0x3F; // srb.h: SRB_STATUS(Status) masks 0x3F (the queue-frozen/autosense bits are 0x40/0x80); was 0xFF
const SRB_STATUS_SUCCESS: u8 = 0x1;  // was 0 (LLM-invented); bindgen-correct
const SRB_STATUS_BUS_RESET: u8 = 0xe;  // was 0x06 (LLM-invented); bindgen-correct
const SRB_STATUS_QUEUE_FROZEN: u8 = 0x40;

// Detection state values
const CdromDetectionUnknown: u32 = 0;
const CdromDetectionUnsupported: u32 = 1;
const CdromDetectionSupported: u32 = 2;

// Pool / SRB constants (caller expected to provide correct values)
const CDROM_TAG_MEDIA_CHANGE_DETECTION: u32 = 0x61436353; // 'CMCM'
const SENSE_BUFFER_SIZE: usize = 18; // classic SENSE_DATA size — matches the senseBuffer alloc
const FDO_HACK_NO_ASYNCHRONOUS_NOTIFICATION: u64 = 0x40; // matches HackFlags (ULONG_PTR)  // was 0x20000 (LLM-invented); bindgen-correct
const SRB_CLASS_FLAGS_LOW_PRIORITY: u32 = 0x10000000; // Unknown  // was 0x100 (LLM-invented); bindgen-correct
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x100; // Unknown  // was 0x200 (LLM-invented); bindgen-correct
const SRB_FLAGS_DISABLE_SYNCH_TRANSFER: u32 = 0x8; // Unknown  // was 0x400 (LLM-invented); bindgen-correct
const SRB_FLAGS_NO_KEEP_AWAKE: u32 = 0x100000; // Unknown  // was 0x800 (LLM-invented); bindgen-correct
const GUID_CONSOLE_DISPLAY_STATE: *const core::ffi::c_void = 0x1 as *const core::ffi::c_void; // Placeholder
#[no_mangle]
pub unsafe extern "C" fn GesnDataInterpret(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Header: *mut NOTIFICATION_EVENT_STATUS_HEADER,
    ResendImmediately: *mut BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let info: *mut MEDIA_CHANGE_DETECTION_INFO = (*DeviceExtension).MediaChangeDetectionInfo;
    let mut dataLength: LONG;
    let mut requiredLength: LONG;
    let mut inHomePosition: BOOLEAN;

    *ResendImmediately = FALSE as BOOLEAN;

    if (*Header).NEA() != 0 {
        return status;
    }
    if (*Header).NotificationClass() as u32 == NOTIFICATION_NO_CLASS_EVENTS {
        return status;
    }

    if (*info).Gesn.HackEventMask != 0 {
        let thisEvent: u8 = *(*Header).ClassEventData.as_ptr().add(0) & 0xf;
        let thisEventBit: u8 = 1u8.wrapping_shl((*Header).NotificationClass() as u32);
        let mut lowestSetBit: u8;

        if ((*info).Gesn.EventMask & thisEventBit) == 0 {
            return STATUS_DEVICE_PROTOCOL_ERROR;
        }

        lowestSetBit = (*info).Gesn.EventMask;
        lowestSetBit &= ((*info).Gesn.EventMask).wrapping_sub(1);
        lowestSetBit ^= (*info).Gesn.EventMask;

        if thisEventBit != lowestSetBit {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"GESN::NONE: Compliant drive found, removing GESN hack (%x, %x)\n\0".as_ptr() as *const i8,
                thisEventBit as u32,
                (*info).Gesn.EventMask as u32,
            );

            (*info).Gesn.HackEventMask = FALSE as BOOLEAN;
        } else if thisEvent == 0 {
            (*info).Gesn.NoChangeEventMask |= thisEventBit;
            (*info).Gesn.EventMask &= !thisEventBit;

            if (*info).Gesn.EventMask == 0 {
                (*info).Gesn.EventMask = (*info).Gesn.NoChangeEventMask;
                (*info).Gesn.NoChangeEventMask = 0;
            } else {
                *ResendImmediately = TRUE as BOOLEAN;
            }
            return status;
        }
    }

    dataLength = (((*Header).EventDataLength[0] as LONG) << 8)
        | ((*Header).EventDataLength[1] as LONG) & 0xff;
    dataLength -= 2;
    requiredLength = 4;

    if dataLength < requiredLength {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"error - GESN returned only %x bytes data for fdo %p\n\0".as_ptr() as *const i8,
            dataLength as u32,
            (*DeviceExtension).DeviceObject as *const core::ffi::c_void,
        );
        return STATUS_DEVICE_PROTOCOL_ERROR;
    }

    if dataLength > requiredLength {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"error - GESN returned too many (%x) bytes data for fdo %p\n\0".as_ptr() as *const i8,
            dataLength as u32,
            (*DeviceExtension).DeviceObject as *const core::ffi::c_void,
        );
    }

    if (*(*Header).ClassEventData.as_ptr().add(0) & 0xf) == 0 {
        return status;
    }

    *ResendImmediately = TRUE as BOOLEAN;

    match (*Header).NotificationClass() as u32 {
        NOTIFICATION_OPERATIONAL_CHANGE_CLASS_EVENTS => {
            let opChangeInfo: *mut NOTIFICATION_OPERATIONAL_STATUS =
                (*Header).ClassEventData.as_mut_ptr() as *mut NOTIFICATION_OPERATIONAL_STATUS;
            let event: ULONG;

            if (*opChangeInfo).OperationalEvent() as u32 == NOTIFICATION_OPERATIONAL_EVENT_CHANGE_REQUESTED {
                // break;
            } else {
                event = (((*opChangeInfo).Operation[0] as ULONG) << 8)
                    | ((*opChangeInfo).Operation[1] as ULONG);

                if (*info).MediaChangeRetryCount >= 4 {
                    static OpChangeMask: u8 = 0x02;

                    TracePrint!(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_MCN,
                        b"GESN OpChange events are broken.  Working around this problem in software (for WDFDEVICE %p)\n\0".as_ptr() as *const i8,
                        (*DeviceExtension).Device as *const core::ffi::c_void,
                    );

                    ntkrnl_sys::ASSERT!(CountOfSetBitsUChar((*info).Gesn.EventMask) != 1);

                    (*info).Gesn.NoChangeEventMask |= OpChangeMask;
                    (*info).Gesn.EventMask &= !OpChangeMask;
                    (*info).Gesn.HackEventMask = TRUE as BOOLEAN;

                    if (*info).Gesn.EventMask == 0 {
                        (*info).Gesn.EventMask = (*info).Gesn.NoChangeEventMask;
                        (*info).Gesn.NoChangeEventMask = 0;
                        *ResendImmediately = FALSE as BOOLEAN;
                    } else {
                        *ResendImmediately = TRUE as BOOLEAN;
                    }
                    // break;
                } else if (event == NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_ADDED)
                    | (event == NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_CHANGE)
                {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_MCN,
                        b"GESN says features added/changed for WDFDEVICE %p\n\0".as_ptr()
                            as *const i8,
                        (*DeviceExtension).Device as *const core::ffi::c_void,
                    );

                    if IsVolumeMounted((*DeviceExtension).DeviceObject) != 0 {
                        (*(*DeviceExtension).DeviceObject).Flags |= DO_VERIFY_VOLUME;
                    }

                    {
                        let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
                        let mut sense: SENSE_DATA = core::mem::zeroed();
                        let mut tempStatus: NTSTATUS;
                        let mut retry: BOOLEAN;

                        tempStatus = STATUS_MEDIA_CHANGED;
                        retry = FALSE as BOOLEAN;

                        srb.CdbLength = 6;
                        srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
                        srb.SrbStatus = (SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_ERROR) as u8;

                        let rtl_size = core::mem::offset_of!(SENSE_DATA, AdditionalSenseLength)
                            + core::mem::size_of::<u8>();
                        sense.AdditionalSenseLength =
                            (core::mem::size_of::<SENSE_DATA>() - rtl_size) as u8;

                        sense.set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as u8);
                        sense.AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as u8;

                        // After the sense fields, not before as in the C: writing `sense`
                        // directly once a raw pointer to it is stored would invalidate that
                        // pointer, and rustc -- blind to the ErrorHandler's read through
                        // SenseInfoBuffer -- reported the writes as dead.
                        srb.SenseInfoBuffer = &mut sense as *mut SENSE_DATA as *mut core::ffi::c_void;
                        srb.SenseInfoBufferLength = core::mem::size_of::<SENSE_DATA>() as u8;

                        if let Some(handler) = (*DeviceExtension).DeviceAdditionalData.ErrorHandler {
                            handler(
                                DeviceExtension,
                                &mut srb,
                                &mut tempStatus,
                                &mut retry,
                            );
                        }
                    }
                }
            }
        }

        NOTIFICATION_EXTERNAL_REQUEST_CLASS_EVENTS => {
            let externalInfo: *mut NOTIFICATION_EXTERNAL_STATUS =
                (*Header).ClassEventData.as_mut_ptr() as *mut NOTIFICATION_EXTERNAL_STATUS;
            let mut externalData: DEVICE_EVENT_EXTERNAL_REQUEST = core::mem::zeroed();

            if (*externalInfo).ExternalEvent() as u32 != NOTIFICATION_EXTERNAL_EVENT_BUTTON_DOWN {
                // break;
            } else {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"GESN::EXTERNAL: Event: %x Status %x Req %x\n\0".as_ptr() as *const i8,
                    (*externalInfo).ExternalEvent() as u32,
                    (*externalInfo).ExternalStatus() as u32,
                    (((*externalInfo).Request[0] as u32) << 8) | ((*externalInfo).Request[1] as u32),
                );

                externalData.Version = 1;
                externalData.DeviceClass = 0;
                externalData.ButtonStatus = (*externalInfo).ExternalEvent() as u16;
                externalData.Request = ((((*externalInfo).Request[0] as u32) << 8)
                    | ((*externalInfo).Request[1] as u32)) as u16;
                KeQuerySystemTime(&mut externalData.SystemTime);
                externalData.SystemTime.QuadPart *= KeQueryTimeIncrement() as LONGLONG;

                DeviceSendNotification(
                    DeviceExtension,
                    &GUID_IO_DEVICE_EXTERNAL_REQUEST as *const GUID,
                    core::mem::size_of::<DEVICE_EVENT_EXTERNAL_REQUEST>() as u32,
                    &externalData as *const DEVICE_EVENT_EXTERNAL_REQUEST as *mut core::ffi::c_void,
                );

                return status;
            }
        }

        NOTIFICATION_MEDIA_STATUS_CLASS_EVENTS => {
            let mediaInfo: *mut NOTIFICATION_MEDIA_STATUS =
                (*Header).ClassEventData.as_mut_ptr() as *mut NOTIFICATION_MEDIA_STATUS;

            if ((*mediaInfo).MediaEvent() as u32 == NOTIFICATION_MEDIA_EVENT_NEW_MEDIA)
                || ((*mediaInfo).MediaEvent() as u32 == NOTIFICATION_MEDIA_EVENT_MEDIA_CHANGE)
            {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"GESN::MEDIA ARRIVAL, Status %x\n\0".as_ptr() as *const i8,
                    *(*mediaInfo).__bindgen_anon_1.MediaStatus.as_ref() as u32,
                );

                if IsVolumeMounted((*DeviceExtension).DeviceObject) != 0 {
                    (*(*DeviceExtension).DeviceObject).Flags |= DO_VERIFY_VOLUME;
                }
                DeviceSetMediaChangeStateEx(DeviceExtension, MediaPresent, core::ptr::null_mut());

                if ((!(*DeviceExtension).ZeroPowerODDInfo.is_null())
                    && ((*(*DeviceExtension).ZeroPowerODDInfo).LoadingMechanism as u32
                        == LOADING_MECHANISM_CADDY)
                    && ((*(*DeviceExtension).ZeroPowerODDInfo).Load == 0))
                {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"GesnDataInterpret: MediaArrival event detected, device marked as active\n\0".as_ptr() as *const i8,
                    );

                    DeviceMarkActive(DeviceExtension, TRUE as BOOLEAN, FALSE as BOOLEAN);
                }
            } else if (*mediaInfo).MediaEvent() as u32 == NOTIFICATION_MEDIA_EVENT_MEDIA_REMOVAL {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"GESN::MEDIA REMOVAL, Status %x\n\0".as_ptr() as *const i8,
                    *(*mediaInfo).__bindgen_anon_1.MediaStatus.as_ref() as u32,
                );

                DeviceSetMediaChangeStateEx(DeviceExtension, MediaNotPresent, core::ptr::null_mut());

                if ((!(*DeviceExtension).ZeroPowerODDInfo.is_null())
                    && ((*(*DeviceExtension).ZeroPowerODDInfo).LoadingMechanism as u32
                        == LOADING_MECHANISM_CADDY)
                    && ((*(*DeviceExtension).ZeroPowerODDInfo).Load == 0))
                {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"GesnDataInterpret: MediaRemoval event detected, device marked as idle\n\0".as_ptr() as *const i8,
                    );

                    DeviceMarkActive(DeviceExtension, FALSE as BOOLEAN, FALSE as BOOLEAN);
                }
            } else if (*mediaInfo).MediaEvent() as u32 == NOTIFICATION_MEDIA_EVENT_EJECT_REQUEST {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"GESN::MEDIA EJECTION, Status %x\n\0".as_ptr() as *const i8,
                    *(*mediaInfo).__bindgen_anon_1.MediaStatus.as_ref() as u32,
                );

                DeviceSendNotification(
                    DeviceExtension,
                    &GUID_IO_MEDIA_EJECT_REQUEST as *const GUID,
                    0,
                    core::ptr::null_mut(),
                );
            }
        }

        NOTIFICATION_DEVICE_BUSY_CLASS_EVENTS => {
            let busyInfo: *mut NOTIFICATION_BUSY_STATUS =
                (*Header).ClassEventData.as_mut_ptr() as *mut NOTIFICATION_BUSY_STATUS;
            let mut busyData: DEVICE_EVENT_BECOMING_READY = core::mem::zeroed();

            busyData.Version = 1;
            busyData.Reason = (*busyInfo).DeviceBusyStatus as u32;
            busyData.Estimated100msToReady = (((*busyInfo).Time[0] as u32) << 8)
                | ((*busyInfo).Time[1] as u32) & 0xff;

            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"GESN::BUSY: Event: %x Status %x Time %x\n\0".as_ptr() as *const i8,
                (*busyInfo).DeviceBusyEvent() as u32,
                (*busyInfo).DeviceBusyStatus as u32,
                busyData.Estimated100msToReady,
            );

            if busyData.Estimated100msToReady >= GESN_DEVICE_BUSY_LOWER_THRESHOLD_100_MS {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"GesnDataInterpret: media BECOMING_READY\n\0".as_ptr() as *const i8,
                );

                DeviceSendNotification(
                    DeviceExtension,
                    &GUID_IO_DEVICE_BECOMING_READY as *const GUID,
                    core::mem::size_of::<DEVICE_EVENT_BECOMING_READY>() as u32,
                    &busyData as *const DEVICE_EVENT_BECOMING_READY as *mut core::ffi::c_void,
                );
            }

            if ((!(*DeviceExtension).ZeroPowerODDInfo.is_null())
                && ((*(*DeviceExtension).ZeroPowerODDInfo).LoadingMechanism as u32
                    == LOADING_MECHANISM_TRAY)
                && ((*(*DeviceExtension).ZeroPowerODDInfo).Load == 0)
                && ((*busyInfo).DeviceBusyEvent() as u32 == NOTIFICATION_BUSY_EVENT_LO_CHANGE)
                && ((*busyInfo).DeviceBusyStatus as u32 == NOTIFICATION_BUSY_STATUS_NO_EVENT))
            {
                inHomePosition = DeviceZPODDIsInHomePosition(DeviceExtension);

                if inHomePosition == (FALSE as BOOLEAN) {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"GesnDataInterpret: LoChange event detected, device marked as active\n\0"
                            .as_ptr() as *const i8,
                    );

                    DeviceMarkActive(DeviceExtension, TRUE as BOOLEAN, FALSE as BOOLEAN);
                } else {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"GesnDataInterpret: LoChange event detected, device marked as idle\n\0"
                            .as_ptr() as *const i8,
                    );

                    DeviceMarkActive(DeviceExtension, FALSE as BOOLEAN, FALSE as BOOLEAN);
                }
            }
        }

        _ => {}
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceInternalSetMediaChangeState(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    NewState: MEDIA_CHANGE_DETECTION_STATE,
    OldState: *mut MEDIA_CHANGE_DETECTION_STATE,
) {
    unsafe {
        #[cfg(debug_assertions)]
        let states: [*const i8; 4] = [
            b"Unknown\0".as_ptr() as *const i8,
            b"Present\0".as_ptr() as *const i8,
            b"Not Present\0".as_ptr() as *const i8,
            b"Unavailable\0".as_ptr() as *const i8,
        ];

        // Validate new state range
        if !((NewState as i32 >= MediaUnknown as i32) && (NewState as i32 <= MediaUnavailable as i32)) {
            return;
        }

        let info = (*DeviceExtension).MediaChangeDetectionInfo;
        if info.is_null() {
            return;
        }

        let oldMediaState = (*info).LastKnownMediaDetectionState;
        if !OldState.is_null() {
            *OldState = oldMediaState;
        }

        (*info).LastKnownMediaDetectionState = NewState;

        // Increment MediaChangeCount on transition to MediaPresent
        if NewState == MediaPresent && oldMediaState != NewState {
            let count_ptr = core::ptr::addr_of_mut!((*DeviceExtension).MediaChangeCount) as *mut i32;
            InterlockedIncrement(count_ptr);
        }

        if (*info).MediaChangeDetectionDisableCount != 0 {
            #[cfg(debug_assertions)]
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"DeviceInternalSetMediaChangeState: MCN not enabled, state changed from %s to %s\n\0"
                    .as_ptr()
                    as *const i8,
                states[oldMediaState as usize],
                states[NewState as usize],
            );
            return;
        }

        #[cfg(debug_assertions)]
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"DeviceInternalSetMediaChangeState: State change from %s to %s\n\0"
                .as_ptr()
                as *const i8,
            states[oldMediaState as usize],
            states[NewState as usize],
        );

        if (*info).LastReportedMediaDetectionState == (*info).LastKnownMediaDetectionState {
            // Media is in the same state as we reported last time, no need to report again.
            return;
        }

        // make the data useful -- it used to always be zero.
        let mcnContext = CLASS_MEDIA_CHANGE_CONTEXT {
            MediaChangeCount: (*DeviceExtension).MediaChangeCount,
            NewState,
        };

        if NewState == MediaPresent {
            #[cfg(debug_assertions)]
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"DeviceInternalSetMediaChangeState: Reporting media ARRIVAL\n\0"
                    .as_ptr()
                    as *const i8,
            );

            DeviceSendNotification(
                DeviceExtension,
                &GUID_IO_MEDIA_ARRIVAL as *const GUID,
                core::mem::size_of::<CLASS_MEDIA_CHANGE_CONTEXT>() as u32,
                &mcnContext as *const CLASS_MEDIA_CHANGE_CONTEXT as *mut core::ffi::c_void,
            );
        } else if NewState == MediaNotPresent || NewState == MediaUnavailable {
            #[cfg(debug_assertions)]
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"DeviceInternalSetMediaChangeState: Reporting media REMOVAL\n\0"
                    .as_ptr()
                    as *const i8,
            );

            DeviceSendNotification(
                DeviceExtension,
                &GUID_IO_MEDIA_REMOVAL as *const GUID,
                core::mem::size_of::<CLASS_MEDIA_CHANGE_CONTEXT>() as u32,
                &mcnContext as *const CLASS_MEDIA_CHANGE_CONTEXT as *mut core::ffi::c_void,
            );
        } else {
            // Don't notify of changed going to unknown.
            return;
        }

        (*info).LastReportedMediaDetectionState = (*info).LastKnownMediaDetectionState;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSetMediaChangeStateEx(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    NewState: MEDIA_CHANGE_DETECTION_STATE,
    OldState: *mut MEDIA_CHANGE_DETECTION_STATE,
) {
    let info = (*DeviceExtension).MediaChangeDetectionInfo;
    if info.is_null() {
        return;
    }

    let mut zero = LARGE_INTEGER { QuadPart: 0 };
    let status = KeWaitForMutexObject(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive as i32,
        0i32,
        0,
        &zero as *const _ as *const core::ffi::c_void,
    );
    if status == STATUS_TIMEOUT {
        return;
    }

    DeviceInternalSetMediaChangeState(DeviceExtension, NewState, OldState);

    KeReleaseMutex(core::ptr::addr_of_mut!((*info).MediaChangeMutex), 0);
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSendDelayedMediaChangeNotifications(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    let mut zero: LARGE_INTEGER = core::mem::zeroed();
    zero.QuadPart = 0;

    let info = (*DeviceExtension).MediaChangeDetectionInfo;

    if info.is_null() {
        return;
    }

    let status = KeWaitForMutexObject(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex) as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive as i32,
        0i32,
        0,
        core::ptr::addr_of_mut!(zero) as *const core::ffi::c_void,
    );

    if status == STATUS_TIMEOUT {
        // Someone else is in the process of setting the media state.
        // That's totally okay, we'll send delayed notifications later.
        return;
    }

    if ((*info).LastKnownMediaDetectionState != (*info).LastReportedMediaDetectionState) &&
        ((*info).MediaChangeDetectionDisableCount == 0)
    {
        DeviceInternalSetMediaChangeState(
            DeviceExtension,
            (*info).LastKnownMediaDetectionState,
            core::ptr::null_mut(),
        );
    }

    KeReleaseMutex(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RequestSetupMcnRequest(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    UseGesn: u8, // BOOLEAN
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    CdromMcnPollMark();
    let srb: *mut SCSI_REQUEST_BLOCK;
    let irp: *mut IRP;
    let nextIrpStack: *mut IO_STACK_LOCATION;
    let cdb: *mut CDB;
    let buffer: *mut core::ffi::c_void;
    let mut params: WDF_REQUEST_REUSE_PARAMS = core::mem::zeroed();
    let info = (*DeviceExtension).MediaChangeDetectionInfo;

    irp = WdfRequestWdmGetIrp((*info).MediaChangeRequest);
    NT_ASSERT!(!irp.is_null() as i32);

    if !(*irp).MdlAddress.is_null() {
        (*irp).MdlAddress = core::ptr::null_mut();
    }

    if NT_SUCCESS(status) {
        WDF_REQUEST_REUSE_PARAMS_INIT(
            &raw mut params,
            0, // WDF_REQUEST_REUSE_NO_FLAGS
            STATUS_NOT_SUPPORTED,
        );
        status = WdfRequestReuse((*info).MediaChangeRequest, &raw mut params);
    }

    if NT_SUCCESS(status) {
        status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            (*DeviceExtension).IoTarget,
            (*info).MediaChangeRequest,
            // IOCTL_SCSI_EXECUTE_IN is 0x1B0011; 0x4D014 (the comment's own claim) is no
            // SCSI ioctl at all, so the media-change request was formatted for the wrong code.
            0x1b0011, // IOCTL_SCSI_EXECUTE_IN
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            TracePrint!(
                2, // TRACE_LEVEL_ERROR
                0, // TRACE_FLAG_GENERAL
                b"RequestSetupMcnRequest: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n\0"
                    .as_ptr()
                    as *const i8,
                status,
            );
        }
    }

    if NT_SUCCESS(status) {
        RequestClearSendTime(((*info).MediaChangeRequest as *mut core::ffi::c_void).cast());

        nextIrpStack = IoGetNextIrpStackLocation(irp);

        (*nextIrpStack).Flags = 0x02; // SL_OVERRIDE_VERIFY_VOLUME
        (*nextIrpStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
        (*nextIrpStack).Parameters.Scsi.Srb = &raw mut (*info).MediaChangeSrb as *mut _;

        srb = (*nextIrpStack).Parameters.Scsi.Srb as *mut SCSI_REQUEST_BLOCK;
        buffer = (*info).SenseBuffer as *mut core::ffi::c_void;

        RtlZeroMemory(srb as *mut core::ffi::c_void, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
        // SENSE_BUFFER_SIZE is 18 (matching the 18-byte senseBuffer allocation) —
        // the translated literal 32 zeroed 8 bytes PAST the 24-usable pool block,
        // wiping the next pool header => deterministic BAD_POOL_HEADER 0x19/5 at a
        // later free (2026-07-18 i386 boot bugcheck; forensics: [POOL] page walk,
        // corrupted neighbor of the ScCa block).
        RtlZeroMemory(buffer as *mut core::ffi::c_void, SENSE_BUFFER_SIZE);

        (*srb).QueueTag = 0; // SP_UNTAGGED
        (*srb).QueueAction = 0; // SRB_SIMPLE_TAG_REQUEST
        (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
        (*srb).Function = 0x00; // SRB_FUNCTION_EXECUTE_SCSI
        (*srb).SenseInfoBuffer = buffer as *mut core::ffi::c_void;
        (*srb).SrbStatus = 0;
        (*srb).ScsiStatus = 0;
        (*srb).OriginalRequest = irp as *mut core::ffi::c_void;
        (*srb).SenseInfoBufferLength = SENSE_BUFFER_SIZE as u8; // 18 — see above

        (*srb).SrbFlags = (*DeviceExtension).SrbFlags;
        (*srb).SrbFlags |= (*info).SrbFlags; // SET_FLAG expansion

        if UseGesn == 0 {
            (*srb).TimeOutValue = 10; // CDROM_TEST_UNIT_READY_TIMEOUT
            (*srb).CdbLength = 6;
            (*srb).DataTransferLength = 0;
            (*srb).SrbFlags |= 0x02; // SRB_FLAGS_NO_DATA_TRANSFER
            (*nextIrpStack).Parameters.DeviceIoControl.IoControlCode = 0x001B0013; // IOCTL_SCSI_EXECUTE_NONE
            (*srb).DataBuffer = core::ptr::null_mut();
            (*srb).DataTransferLength = 0;
            (*irp).MdlAddress = core::ptr::null_mut();

            cdb = &raw mut (*srb).Cdb[0] as *mut CDB;
            (*cdb).CDB6GENERIC.as_mut().OperationCode = 0x00; // SCSIOP_TEST_UNIT_READY
        } else {
            NT_ASSERT!(!(*info).Gesn.Buffer.is_null() as i32);
            (*srb).TimeOutValue = 0x4; // GESN_TIMEOUT_VALUE
            (*srb).CdbLength = 10;
            (*srb).SrbFlags |= 0x40; // SRB_FLAGS_DATA_IN
            (*nextIrpStack).Parameters.DeviceIoControl.IoControlCode = 0x4D014; // IOCTL_SCSI_EXECUTE_IN
            (*srb).DataBuffer = (*info).Gesn.Buffer as *mut core::ffi::c_void;
            (*srb).DataTransferLength = (*info).Gesn.BufferSize;
            (*irp).MdlAddress = (*info).Gesn.Mdl as *mut _;

            cdb = &raw mut (*srb).Cdb[0] as *mut CDB;
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().OperationCode = 0x4A; // SCSIOP_GET_EVENT_STATUS
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().set_Immediate(1);
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().EventListLength[0] = ((*info).Gesn.BufferSize >> 8) as u8;
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().EventListLength[1] = ((*info).Gesn.BufferSize & 0xff) as u8;
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().NotificationClassRequest = (*info).Gesn.EventMask as u8;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn DeviceDisableGesn(WorkItem: WDFWORKITEM) {
    let device = WdfWorkItemGetParentObject(WorkItem) as WDFDEVICE;
    let deviceExtension = DeviceGetExtension(device);

    // PAGED_CODE(); // macro omitted

    DeviceSetParameter(
        deviceExtension,
        CLASSP_REG_SUBKEY_NAME,
        CLASSP_REG_MMC_DETECTION_VALUE_NAME,
        CdromDetectionUnsupported,
    );

    WdfObjectDelete(WorkItem as WDFOBJECT);
}
#[no_mangle]
pub unsafe extern "C" fn RequestPostWorkMcnRequest(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> u8 {
    // Forward declarations for unknown/macro symbols (caller assembles these)
    // Assume macros are defined as Rust macros or crate-level functions; use macro invocations.

    let mut status: i32 = 0; // STATUS_SUCCESS
    let info: *mut MEDIA_CHANGE_DETECTION_INFO = (*DeviceExtension).MediaChangeDetectionInfo;
    let irp: *mut IRP;
    let mut retryImmediately: u8 = 0; // FALSE

    PAGED_CODE!();

    NT_ASSERT!(!(*info).MediaChangeRequest.is_null() as i32);
    irp = WdfRequestWdmGetIrp((*info).MediaChangeRequest);

    NT_ASSERT!((TEST_FLAG((*info).MediaChangeSrb.SrbStatus as u32, SRB_STATUS_QUEUE_FROZEN as u32) == 0) as i32);

    // use InterpretSenseInfo routine to check for media state, and also
    // to call ClassError() with correct parameters.
    if SRB_STATUS((*info).MediaChangeSrb.SrbStatus) != 0 { // SRB_STATUS_SUCCESS is 0
        let sense_buf = (*info).MediaChangeSrb.SenseInfoBuffer as *const SENSE_DATA;
        TracePrint!(TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
            b"MCN request - failed - srb status=%x, sense=%x/%x/%x.\n\0".as_ptr() as *const i8,
            (*info).MediaChangeSrb.SrbStatus as u32,
            (*sense_buf).SenseKey() as u32,
            (*sense_buf).AdditionalSenseCode as u32,
            (*sense_buf).AdditionalSenseCodeQualifier as u32,
        );

        if SRB_STATUS((*info).MediaChangeSrb.SrbStatus) != 0x0010 { // SRB_STATUS_NOT_POWERED
            // Release the queue if it is frozen.
            if ((*info).MediaChangeSrb.SrbStatus & SRB_STATUS_QUEUE_FROZEN) != 0 {
                DeviceReleaseQueue((*DeviceExtension).Device);
            }

            RequestSenseInfoInterpret(
                DeviceExtension,
                (*info).MediaChangeRequest as *mut core::ffi::c_void,
                &mut (*info).MediaChangeSrb,
                0,
                &mut status,
                core::ptr::null_mut(),
            );
        }
    } else {
        (*(*DeviceExtension).PrivateFdoData).LoggedTURFailureSinceLastIO = 0; // FALSE

        if (*info).Gesn.Supported == 0 {
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
                b"MCN request - succeeded (GESN NOT supported, setting MediaPresent).\n\0".as_ptr() as *const i8,
            );

            // success != media for GESN case
            DeviceSetMediaChangeStateEx(
                DeviceExtension,
                MediaPresent,
                core::ptr::null_mut(),
            );
        } else {
            TracePrint!(TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
                b"MCN request - succeeded (GESN supported).\n\0".as_ptr() as *const i8,
            );
        }
    }

    if (*info).Gesn.Supported != 0 {
        if status == ntkrnl_sys::constants::STATUS_DATA_OVERRUN {
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
                b"MCN Request - Data Overrun\n\0".as_ptr() as *const i8,
            );
            status = 0; // STATUS_SUCCESS
        }

        if !(status >= 0) { // NT_SUCCESS
            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
                b"MCN Request: GESN failed with status %x\n\0".as_ptr() as *const i8,
                status,
            );
        } else {
            // for GESN, need to interpret the results of the data.
            // this may also require an immediate retry
            if (*irp).IoStatus.Information == 8 {
                GesnDataInterpret(
                    DeviceExtension,
                    (*info).Gesn.Buffer as *mut NOTIFICATION_EVENT_STATUS_HEADER,
                    &mut retryImmediately,
                );
            }
        }
    }

    // free port-allocated sense buffer, if any.
    if PORT_ALLOCATED_SENSE(DeviceExtension, &mut (*info).MediaChangeSrb) != 0 {
        FREE_PORT_ALLOCATED_SENSE_BUFFER(DeviceExtension, &mut (*info).MediaChangeSrb);
    }

    // Remember the IRP and SRB for use the next time.
    NT_ASSERT!(!IoGetNextIrpStackLocation(irp).is_null() as i32);
    (*IoGetNextIrpStackLocation(irp)).Parameters.Scsi.Srb = &raw mut (*info).MediaChangeSrb as *mut _;

    // run a sanity check to make sure we're not recursing continuously
    if retryImmediately != 0 {
        (*info).MediaChangeRetryCount += 1;

        if (*info).MediaChangeRetryCount > 0x20 { // MAXIMUM_IMMEDIATE_MCN_RETRIES
            // Disable GESN on this device.
            // Create a work item to set the value in the registry
            let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut workitemConfig: WDF_WORKITEM_CONFIG = core::mem::zeroed();
            let mut workItem: WDFWORKITEM = core::ptr::null_mut();

            WDF_OBJECT_ATTRIBUTES_INIT(&mut attributes);
            attributes.ParentObject = (*DeviceExtension).Device as *mut core::ffi::c_void;

            WDF_WORKITEM_CONFIG_INIT(&mut workitemConfig, Some(DeviceDisableGesn));
            workitemConfig.AutomaticSerialization = 0; // FALSE

            status = WdfWorkItemCreate(
                &mut workitemConfig,
                &mut attributes,
                &mut workItem,
            );

            TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
                b"MCN Request: Disabling GESN for WDFDEVICE %p\n\0".as_ptr() as *const i8,
                (*DeviceExtension).Device,
            );

            if status >= 0 { // NT_SUCCESS
                WdfWorkItemEnqueue(workItem);
            }

            (*info).Gesn.Supported = 0; // FALSE
            (*info).Gesn.EventMask = 0;
            (*info).Gesn.BufferSize = 0;
            (*info).MediaChangeRetryCount = 0;
            retryImmediately = 0; // FALSE
            // should we log an error in event log?
        }
    } else {
        (*info).MediaChangeRetryCount = 0;
    }

    retryImmediately
}
#[no_mangle]
pub unsafe extern "C" fn RequestSendMcnRequest(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> u8 {
    let mut request_sent: u8 = 0; // FALSE
    let info = (*DeviceExtension).MediaChangeDetectionInfo;

    // PAGED_CODE(); -- debug assertion omitted

    RequestSend(
        DeviceExtension,
        ((*info).MediaChangeRequest as *mut core::ffi::c_void).cast(),
        ((*DeviceExtension).IoTarget as *mut core::ffi::c_void).cast(),
        WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
        core::ptr::addr_of_mut!(request_sent),
    );

    request_sent
}
#[no_mangle]
pub unsafe extern "C" fn DeviceInitializeMcn(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    AllowDriveToSleep: u8, // BOOLEAN
) -> NTSTATUS {

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut mediaChangeInfo: *mut MEDIA_CHANGE_DETECTION_INFO;
    let mut irp: *mut IRP = null_mut();
    let mut senseBuffer: *mut core::ffi::c_void = null_mut();
    let mut attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();

    PAGED_CODE!();

    if !(*DeviceExtension).MediaChangeDetectionInfo.is_null() {
        return STATUS_SUCCESS;
    }

    (*DeviceExtension).KernelModeMcnContext.FileObject = (-1isize) as *mut _;
    (*DeviceExtension).KernelModeMcnContext.DeviceObject = (-1isize) as *mut _;
    (*DeviceExtension).KernelModeMcnContext.LockCount = 0;
    (*DeviceExtension).KernelModeMcnContext.McnDisableCount = 0;

    mediaChangeInfo = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        size_of::<MEDIA_CHANGE_DETECTION_INFO>(),
        CDROM_TAG_MEDIA_CHANGE_DETECTION,
    ) as *mut MEDIA_CHANGE_DETECTION_INFO;

    if mediaChangeInfo.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        RtlZeroMemory(mediaChangeInfo as *mut core::ffi::c_void, size_of::<MEDIA_CHANGE_DETECTION_INFO>());
    }

    if status >= 0 {
        if !(*DeviceExtension).PowerDescriptor.is_null()
            && (*(*DeviceExtension).PowerDescriptor).AsynchronousNotificationSupported != 0
            && ((*(*DeviceExtension).PrivateFdoData).HackFlags & (FDO_HACK_NO_ASYNCHRONOUS_NOTIFICATION as ntkrnl_sys::ddk::ULONG_PTR)) == 0
        {
            (*mediaChangeInfo).AsynchronousNotificationSupported = 1;
        }
    }

    // Allocate an IRP to carry the IOCTL_MCN_SYNC_FAKE_IOCTL
    if status >= 0 {
        irp = IoAllocateIrp((*(*DeviceExtension).DeviceObject).StackSize, 0);
        if irp.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if status >= 0 {
        WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
            &mut attributes,
            (&CDROM_REQUEST_CONTEXT_TYPE_INFO as *const _ as *const core::ffi::c_void).cast(),
        );
        attributes.ParentObject = (*DeviceExtension).Device as *mut core::ffi::c_void;
        status = WdfRequestCreate(
            &mut attributes,
            (*DeviceExtension).IoTarget,
            &mut (*mediaChangeInfo).MediaChangeRequest,
        );
    }

    if status >= 0 {
        // Preformat the media change request
        status = WdfIoTargetFormatRequestForInternalIoctlOthers(
            (*DeviceExtension).IoTarget,
            (*mediaChangeInfo).MediaChangeRequest,
            IOCTL_SCSI_EXECUTE_IN,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
        );
    }

    if status >= 0 {
        senseBuffer = ExAllocatePoolWithTag(
            NonPagedPoolNxCacheAligned,
            SENSE_BUFFER_SIZE,
            CDROM_TAG_MEDIA_CHANGE_DETECTION,
        );
        if senseBuffer.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if status >= 0 {
        (*mediaChangeInfo).MediaChangeSyncIrp = irp;
        (*mediaChangeInfo).SenseBuffer = senseBuffer as *mut u8;

        // Set default values for the media change notification configuration
        (*mediaChangeInfo).MediaChangeDetectionDisableCount = 0;

        // Assume that there is initially no media in the device
        (*mediaChangeInfo).LastKnownMediaDetectionState = MediaUnknown;
        (*mediaChangeInfo).LastReportedMediaDetectionState = MediaUnknown;

        // setup all extra flags we'll be setting for this irp
        (*mediaChangeInfo).SrbFlags = 0;

        (*mediaChangeInfo).SrbFlags |= SRB_CLASS_FLAGS_LOW_PRIORITY;
        (*mediaChangeInfo).SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;
        (*mediaChangeInfo).SrbFlags |= SRB_FLAGS_DISABLE_SYNCH_TRANSFER;

        if AllowDriveToSleep != 0 {
            (*mediaChangeInfo).SrbFlags |= SRB_FLAGS_NO_KEEP_AWAKE;
        }

        KeInitializeMutex(&mut (*mediaChangeInfo).MediaChangeMutex, 0x100);

        // It is ok to support media change events on this device
        (*DeviceExtension).MediaChangeDetectionInfo = mediaChangeInfo;

        // check the device supports GESN or not, initialize GESN structure if it supports
        {
            let mut tempStatus: NTSTATUS;

            // TracePrint macros disabled due to varargs; original C calls commented
            // TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
            //             "DeviceInitializeMcn: Testing for GESN\n"));
            tempStatus = DeviceInitializeGesn(DeviceExtension);

            if tempStatus >= 0 {
                // TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
                //             "DeviceInitializeMcn: GESN available for %p\n",
                //             DeviceExtension->DeviceObject));
                NT_ASSERT!(((*mediaChangeInfo).Gesn.Supported != 0) as i32);
                NT_ASSERT!((!(*mediaChangeInfo).Gesn.Buffer.is_null()) as i32);
                NT_ASSERT!(((*mediaChangeInfo).Gesn.BufferSize != 0) as i32);
                NT_ASSERT!(((*mediaChangeInfo).Gesn.EventMask != 0) as i32);
            } else {
                // TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
                //             "DeviceInitializeMcn: GESN *NOT* available for %p\n",
                //             DeviceExtension->DeviceObject));
                NT_ASSERT!(((*mediaChangeInfo).Gesn.Supported == 0) as i32);
                NT_ASSERT!(((*mediaChangeInfo).Gesn.Buffer.is_null()) as i32);
                NT_ASSERT!(((*mediaChangeInfo).Gesn.BufferSize == 0) as i32);
                NT_ASSERT!(((*mediaChangeInfo).Gesn.EventMask == 0) as i32);
                (*mediaChangeInfo).Gesn.Supported = 0; // just in case....
            }
        }
    }

    if status >= 0 {
        // Register for display state change on AOAC capable systems
        if (*mediaChangeInfo).DisplayStateCallbackHandle.is_null() {
            let mut PlatformInfo: POWER_PLATFORM_INFORMATION = core::mem::zeroed();
            status = ZwPowerInformation(
                PlatformInformation,
                null_mut(),
                0,
                &mut PlatformInfo as *mut _ as *mut core::ffi::c_void,
                size_of::<POWER_PLATFORM_INFORMATION>() as u32,
            );

            if status >= 0 && PlatformInfo.AoAc != 0 {
                PoRegisterPowerSettingCallback(
                    (*DeviceExtension).DeviceObject as *mut core::ffi::c_void,
                    GUID_CONSOLE_DISPLAY_STATE,
                    DevicePowerSettingCallback as *mut core::ffi::c_void,
                    DeviceExtension as *mut core::ffi::c_void,
                    &mut (*mediaChangeInfo).DisplayStateCallbackHandle,
                );
            }

            // Ignore any failures above
            status = STATUS_SUCCESS;
        }
    }

    if status < 0 {
        if !irp.is_null() {
            IoFreeIrp(irp);
        }
        FREE_POOL(senseBuffer);
        FREE_POOL(mediaChangeInfo as *mut core::ffi::c_void);
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceInitializeGesn(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> NTSTATUS {
    // PAGED_CODE(); -- macro, no-op in Rust

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut header: *mut NOTIFICATION_EVENT_STATUS_HEADER;
    let mut detectionState: u32 = CdromDetectionUnknown;
    let deviceDescriptor: *mut STORAGE_DEVICE_DESCRIPTOR = (*DeviceExtension).DeviceDescriptor;
    let mut retryImmediately: BOOLEAN = 1; // TRUE
    let mut i: u32;
    let mut atapiResets: u32 = 0;
    let info: *mut MEDIA_CHANGE_DETECTION_INFO = (*DeviceExtension).MediaChangeDetectionInfo;

    ntkrnl_sys::ASSERT!(!info.is_null(), "info != NULL");

    // Read if we already know the abilities of the device
    DeviceGetParameter(
        DeviceExtension,
        CLASSP_REG_SUBKEY_NAME,
        CLASSP_REG_MMC_DETECTION_VALUE_NAME,
        core::ptr::addr_of_mut!(detectionState),
    );

    if detectionState == CdromDetectionUnsupported {
        status = STATUS_NOT_SUPPORTED;
    }

    // Check if the device has a hack flag saying never to try this.
    if NT_SUCCESS(status) && ((*(*DeviceExtension).PrivateFdoData).HackFlags & FDO_HACK_GESN_IS_BAD as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
        DeviceSetParameter(
            DeviceExtension,
            CLASSP_REG_SUBKEY_NAME,
            CLASSP_REG_MMC_DETECTION_VALUE_NAME,
            CdromDetectionUnsupported,
        );
        status = STATUS_NOT_SUPPORTED;
    }

    // Else go through the process since we allocate buffers and
    // get all sorts of device settings.
    if NT_SUCCESS(status) {
        if (*info).Gesn.Buffer.is_null() {
            (*info).Gesn.Buffer = ExAllocatePoolWithTag(
                NonPagedPoolNxCacheAligned,
                GESN_BUFFER_SIZE as usize,
                CDROM_TAG_GESN,
            ) as *mut u8;
        }

        if (*info).Gesn.Buffer.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if NT_SUCCESS(status) {
        if !(*info).Gesn.Mdl.is_null() {
            IoFreeMdl((*info).Gesn.Mdl as *mut _);
        }

        (*info).Gesn.Mdl = IoAllocateMdl(
            (*info).Gesn.Buffer as *mut core::ffi::c_void,
            GESN_BUFFER_SIZE,
            0, // FALSE
            0,  // FALSE
            core::ptr::null_mut(),
        ) as *mut _;
        if ((*info).Gesn.Mdl).is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    if NT_SUCCESS(status) {
        MmBuildMdlForNonPagedPool((*info).Gesn.Mdl as *mut _);
        (*info).Gesn.BufferSize = GESN_BUFFER_SIZE;
        (*info).Gesn.EventMask = 0;

        // All items are prepared to use GESN (except the event mask, so don't
        // optimize this part out!).
        //
        // Now see if it really works. We have to loop through this because
        // many SAMSUNG (and one COMPAQ) drives timeout when requesting
        // NOT_READY events, even when the IMMEDIATE bit is set. :(
        //
        // Using a drive list is cumbersome, so this might fix the problem.
        i = 0;
        while i < 16 && retryImmediately != 0 {
            status = RequestSetupMcnRequest(DeviceExtension, 1);

            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_MCN,
                    b"Setup Mcn request failed %x for WDFDEVICE %p\n\0".as_ptr() as *const i8,
                    status,
                    (*DeviceExtension).Device,
                );
                break;
            }

            ntkrnl_sys::ASSERT!(((*info).MediaChangeSrb.SrbFlags & SRB_FLAGS_NO_QUEUE_FREEZE) != 0, "SRB_FLAGS_NO_QUEUE_FREEZE not set");

            status = DeviceSendRequestSynchronously((*DeviceExtension).Device, (*info).MediaChangeRequest, 1);

            if ((*info).MediaChangeSrb.SrbStatus & SRB_STATUS_MASK) != SRB_STATUS_SUCCESS {
                // Release the queue if it is frozen.
                if ((*info).MediaChangeSrb.SrbStatus & SRB_STATUS_QUEUE_FROZEN) != 0 {
                    DeviceReleaseQueue((*DeviceExtension).Device);
                }

                RequestSenseInfoInterpret(
                    DeviceExtension,
                    (*info).MediaChangeRequest as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!((*info).MediaChangeSrb),
                    0,
                    core::ptr::addr_of_mut!(status),
                    core::ptr::null_mut(),
                );
            }

            if ((*deviceDescriptor).BusType == BusTypeAtapi as u32)
                && ((*info).MediaChangeSrb.SrbStatus & SRB_STATUS_MASK) == SRB_STATUS_BUS_RESET
            {
                //
                // ATAPI unfortunately returns SRB_STATUS_BUS_RESET instead
                // of SRB_STATUS_TIMEOUT, so we cannot differentiate between
                // the two.  if we get this status four time consecutively,
                // stop trying this command.  it is too late to change ATAPI
                // at this point, so special-case this here. (07/10/2001)
                // NOTE: any value more than 4 may cause the device to be
                //       marked missing.
                //
                atapiResets += 1;
                if atapiResets >= 4 {
                    status = STATUS_IO_DEVICE_ERROR;
                    break;
                }
            }

            if status == STATUS_DATA_OVERRUN {
                status = STATUS_SUCCESS;
            }

            if status == STATUS_INVALID_DEVICE_REQUEST
                || status == STATUS_TIMEOUT
                || status == STATUS_IO_DEVICE_ERROR
                || status == STATUS_IO_TIMEOUT
            {
                // With these error codes, we don't ever want to try this command
                // again on this device, since it reacts poorly.
                DeviceSetParameter(
                    DeviceExtension,
                    CLASSP_REG_SUBKEY_NAME,
                    CLASSP_REG_MMC_DETECTION_VALUE_NAME,
                    CdromDetectionUnsupported,
                );
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_MCN,
                    b"GESN test failed %x for WDFDEVICE %p\n\0".as_ptr() as *const i8,
                    status,
                    (*DeviceExtension).Device,
                );
                break;
            }

            if !NT_SUCCESS(status) {
                // This may be other errors that should not disable GESN
                // for all future start_device calls.
                TracePrint!(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_MCN,
                    b"GESN test failed %x for WDFDEVICE %p\n\0".as_ptr() as *const i8,
                    status,
                    (*DeviceExtension).Device,
                );
                break;
            } else if i == 0 {
                // The first time, the request was just retrieving a mask of
                // available bits.  Use this to mask future requests.
                header = (*info).Gesn.Buffer as *mut NOTIFICATION_EVENT_STATUS_HEADER;

                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"WDFDEVICE %p supports event mask %x\n\0".as_ptr() as *const i8,
                    (*DeviceExtension).Device,
                    (*header).SupportedEventClasses as core::ffi::c_uint,
                );

                if ((*header).SupportedEventClasses as u32 & NOTIFICATION_MEDIA_STATUS_CLASS_MASK) != 0 {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_MCN,
                        b"GESN supports MCN\n\0".as_ptr() as *const i8,
                    );
                }
                if ((*header).SupportedEventClasses as u32 & NOTIFICATION_DEVICE_BUSY_CLASS_MASK) != 0 {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_MCN,
                        b"GESN supports DeviceBusy\n\0".as_ptr() as *const i8,
                    );
                }
                if ((*header).SupportedEventClasses as u32 & NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK) != 0 {
                    if ((*(*DeviceExtension).PrivateFdoData).HackFlags & FDO_HACK_GESN_IGNORE_OPCHANGE as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
                        TracePrint!(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_MCN,
                            b"GESN supports OpChange, but must ignore these events for compatibility\n\0".as_ptr() as *const i8,
                        );
                        (*header).SupportedEventClasses &= !NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK as u8;
                    } else {
                        TracePrint!(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_MCN,
                            b"GESN supports OpChange\n\0".as_ptr() as *const i8,
                        );
                    }
                }
                (*info).Gesn.EventMask = (*header).SupportedEventClasses;

                //
                // Realistically, we are only considering the following events:
                //    EXTERNAL REQUEST - this is being tested for play/stop/etc.
                //    MEDIA STATUS - autorun and ejection requests.
                //    DEVICE BUSY - to allow us to predict when media will be ready.
                // Therefore, we should not bother querying for the other,
                // unknown events. Clear all but the above flags.
                //
                (*info).Gesn.EventMask &= (NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK
                    | NOTIFICATION_EXTERNAL_REQUEST_CLASS_MASK
                    | NOTIFICATION_MEDIA_STATUS_CLASS_MASK
                    | NOTIFICATION_DEVICE_BUSY_CLASS_MASK) as u8;

                //
                // HACKHACK - REF #0001
                // Some devices will *never* report an event if we've also requested
                // that it report lower-priority events.  This is due to a
                // misunderstanding in the specification wherein a "No Change" is
                // interpreted to be a real event.  What should occur is that the
                // device should ignore "No Change" events when multiple event types
                // are requested unless there are no other events waiting.  This
                // greatly reduces the number of requests that the host must send
                // to determine if an event has occurred. Since we must work on all
                // drives, default to enabling the hack until we find evidence of
                // proper firmware.
                //
                if (*info).Gesn.EventMask == 0 {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_MCN,
                        b"GESN supported, but not mask we care about (%x) for FDO %p\n\0".as_ptr() as *const i8,
                        (*header).SupportedEventClasses as core::ffi::c_uint,
                        (*DeviceExtension).DeviceObject,
                    );
                    // NOTE: the status is still status_success.
                    break;
                } else if CountOfSetBitsUChar((*info).Gesn.EventMask as u8) == 1 {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_MCN,
                        b"GESN hack not required for FDO %p\n\0".as_ptr() as *const i8,
                        (*DeviceExtension).DeviceObject,
                    );
                } else {
                    TracePrint!(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_MCN,
                        b"GESN hack enabled for FDO %p\n\0".as_ptr() as *const i8,
                        (*DeviceExtension).DeviceObject,
                    );
                    (*info).Gesn.HackEventMask = 1;
                }
            } else {
                // i > 0; not the first time looping through, so interpret the results.
                status = GesnDataInterpret(
                    DeviceExtension,
                    (*info).Gesn.Buffer as *mut NOTIFICATION_EVENT_STATUS_HEADER,
                    core::ptr::addr_of_mut!(retryImmediately),
                );

                if !NT_SUCCESS(status) {
                    // This drive does not support GESN correctly
                    DeviceSetParameter(
                        DeviceExtension,
                        CLASSP_REG_SUBKEY_NAME,
                        CLASSP_REG_MMC_DETECTION_VALUE_NAME,
                        CdromDetectionUnsupported,
                    );
                    break;
                }
            }

            i += 1;
        } // end 'for' loop of GESN requests...
    }

    if NT_SUCCESS(status) {
        //
        // We can only use this if it can be relied upon for media changes,
        // since we are (by definition) no longer going to be polling via
        // a TEST_UNIT_READY IRP, and drives will not report UNIT ATTENTION
        // for this command (although a filter driver, such as one for burning
        // CD's, might still fake those errors).
        //
        // Since we also rely upon NOT_READY events to change the cursor
        // into a "wait" cursor; GESN is still more reliable than other
        // methods, and includes eject button requests, so we'll use it
        // without DEVICE_BUSY in Windows Vista.
        //

        if ((*info).Gesn.EventMask as u32 & NOTIFICATION_MEDIA_STATUS_CLASS_MASK) != 0 {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"Enabling GESN support for WDFDEVICE %p\n\0".as_ptr() as *const i8,
                (*DeviceExtension).Device,
            );
            (*info).Gesn.Supported = 1; // TRUE
            DeviceSetParameter(
                DeviceExtension,
                CLASSP_REG_SUBKEY_NAME,
                CLASSP_REG_MMC_DETECTION_VALUE_NAME,
                CdromDetectionSupported,
            );
        } else {
            TracePrint!(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"GESN available but not enabled for WDFDEVICE %p\n\0".as_ptr() as *const i8,
                (*DeviceExtension).Device,
            );
            status = STATUS_NOT_SUPPORTED;
        }
    }

    if !NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"GESN support detection failed  for WDFDEVICE %p with status %08x\n\0".as_ptr() as *const i8,
            (*DeviceExtension).Device,
            status,
        );

        if !(*info).Gesn.Mdl.is_null() {
            let irp: *mut IRP = WdfRequestWdmGetIrp((*info).MediaChangeRequest);
            IoFreeMdl((*info).Gesn.Mdl as *mut _);
            (*info).Gesn.Mdl = core::ptr::null_mut();
            (*irp).MdlAddress = core::ptr::null_mut();
        }

        FREE_POOL((*info).Gesn.Buffer as *mut core::ffi::c_void);
        (*info).Gesn.Supported = 0;   // FALSE
        (*info).Gesn.EventMask = 0;
        (*info).Gesn.BufferSize = 0;
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceInitializeMediaChangeDetection(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> i32 {
    let mut status: i32 = 0; // STATUS_SUCCESS
    let mut disabled: i32; // FALSE
    let mut instanceOverride: u8 = 0; // initialised to avoid uninitialized use

    ; // PAGED_CODE()

    disabled = DeviceIsMediaChangeDisabledDueToHardwareLimitation(DeviceExtension);

    if disabled != 0 {
        let fmt = b"DeviceInitializeMediaChangeDetection: Disabled due to hardware limitations for this device\n\0"
            .as_ptr() as *const i8;
        TracePrint!(4 /* TRACE_LEVEL_INFORMATION */, 0 /* TRACE_FLAG_MCN */, fmt);
    } else {
        disabled = DeviceIsMediaChangeDisabledForClass(DeviceExtension);

        let disabled_str = if disabled != 0 {
            b"disabled\0".as_ptr() as *const i8
        } else {
            b"enabled\0".as_ptr() as *const i8
        };
        let fmt = b"DeviceInitializeMediaChangeDetection: MCN is %s\n\0"
            .as_ptr() as *const i8;
        TracePrint!(4 /* TRACE_LEVEL_INFORMATION */, 0 /* TRACE_FLAG_MCN */, fmt, disabled_str);

        status = DeviceMediaChangeDeviceInstanceOverride(
            DeviceExtension,
            core::ptr::addr_of_mut!(instanceOverride),
        );

        if !NT_SUCCESS(status) {
            let fmt = b"DeviceInitializeMediaChangeDetection: Instance using default\n\0"
                .as_ptr() as *const i8;
            TracePrint!(4 /* TRACE_LEVEL_INFORMATION */, 0 /* TRACE_FLAG_MCN */, fmt);
        } else {
            let override_action = if instanceOverride != 0 {
                b"Enabling\0".as_ptr() as *const i8
            } else {
                b"Disabling\0".as_ptr() as *const i8
            };
            let fmt = b"DeviceInitializeMediaChangeDetection: Instance override: %s MCN\n\0"
                .as_ptr() as *const i8;
            TracePrint!(4 /* TRACE_LEVEL_INFORMATION */, 0 /* TRACE_FLAG_MCN */, fmt, override_action);
            disabled = if instanceOverride == 0 { 1 } else { 0 }; // !instanceOverride
        }

        let final_disabled_str = if disabled != 0 {
            b"disabled\0".as_ptr() as *const i8
        } else {
            b"enabled\0".as_ptr() as *const i8
        };
        let fmt = b"DeviceInitializeMediaChangeDetection: Instance MCN is %s\n\0"
            .as_ptr() as *const i8;
        TracePrint!(4 /* TRACE_LEVEL_INFORMATION */, 0 /* TRACE_FLAG_MCN */, fmt, final_disabled_str);
    }

    if disabled == 0 {
        status = DeviceInitializeMcn(DeviceExtension, 0); // FALSE
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceMediaChangeDeviceInstanceOverride(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Enabled: *mut u8,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut deviceKey: WDFKEY = core::ptr::null_mut();
    let mut subKey: WDFKEY = core::ptr::null_mut();
    let mut subkeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut enableMcnValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut disableMcnValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut alwaysEnable: u32 = 0;
    let mut alwaysDisable: u32 = 0;

    PAGED_CODE!();

    status = WdfDeviceOpenRegistryKey(
        (*DeviceExtension).Device,
        1u32, /* PLUGPLAY_REGKEY_DEVICE */
        0xF003Fu32, /* KEY_ALL_ACCESS */
        core::ptr::null_mut(), /* WDF_NO_OBJECT_ATTRIBUTES */
        &mut deviceKey,
    );
    if !NT_SUCCESS(status) {
        TracePrint!(
            4u32, /* TRACE_LEVEL_INFORMATION */
            0u32, /* TRACE_FLAG_MCN */
            b"DeviceMediaChangeDeviceInstanceOverride: Could not open device registry key [%lx]\n\0"
                .as_ptr() as *const i8,
            status,
        );
    } else {
        let subkey_name: [u16; 24] = [
            'M' as u16, 'e' as u16, 'd' as u16, 'i' as u16, 'a' as u16,
            'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16,
            'e' as u16, 'N' as u16, 'o' as u16, 't' as u16, 'i' as u16,
            'f' as u16, 'i' as u16, 'c' as u16, 'a' as u16, 't' as u16,
            'i' as u16, 'o' as u16, 'n' as u16, 0u16,
        ];
        RtlInitUnicodeString(&mut subkeyName, subkey_name.as_ptr());
        status = WdfRegistryOpenKey(
            deviceKey,
            &mut subkeyName,
            0x20019u32, /* KEY_READ */
            core::ptr::null_mut(), /* WDF_NO_OBJECT_ATTRIBUTES */
            &mut subKey,
        );
    }

    if !NT_SUCCESS(status) {
        TracePrint!(
            4u32, /* TRACE_LEVEL_INFORMATION */
            0u32, /* TRACE_FLAG_MCN */
            b"DeviceMediaChangeDeviceInstanceOverride: subkey could not be created. %lx\n\0"
                .as_ptr() as *const i8,
            status,
        );
    } else {
        let enable_mcn_name: [u16; 16] = [
            'A' as u16, 'l' as u16, 'w' as u16, 'a' as u16, 'y' as u16,
            's' as u16, 'E' as u16, 'n' as u16, 'a' as u16, 'b' as u16,
            'l' as u16, 'e' as u16, 'M' as u16, 'C' as u16, 'N' as u16,
            0u16,
        ];
        let disable_mcn_name: [u16; 17] = [
            'A' as u16, 'l' as u16, 'w' as u16, 'a' as u16, 'y' as u16,
            's' as u16, 'D' as u16, 'i' as u16, 's' as u16, 'a' as u16,
            'b' as u16, 'l' as u16, 'e' as u16, 'M' as u16, 'C' as u16,
            'N' as u16, 0u16,
        ];
        RtlInitUnicodeString(&mut enableMcnValueName, enable_mcn_name.as_ptr());
        RtlInitUnicodeString(&mut disableMcnValueName, disable_mcn_name.as_ptr());
        let _ = WdfRegistryQueryULong(subKey, &mut enableMcnValueName, &mut alwaysEnable);
        let _ = WdfRegistryQueryULong(subKey, &mut disableMcnValueName, &mut alwaysDisable);
    }

    if !subKey.is_null() {
        WdfRegistryClose(subKey);
    }
    if !deviceKey.is_null() {
        WdfRegistryClose(deviceKey);
    }

    if alwaysEnable != 0 && alwaysDisable != 0 {
        TracePrint!(
            4u32, /* TRACE_LEVEL_INFORMATION */
            0u32, /* TRACE_FLAG_MCN */
            b"DeviceMediaChangeDeviceInstanceOverride: %s selected\n\0"
                .as_ptr() as *const i8,
            b"Both Enable and Disable set -- DISABLE\0".as_ptr() as *const i8,
        );
        NT_ASSERT!(NT_SUCCESS(status) as i32);
        status = STATUS_SUCCESS;
        *Enabled = 0; // FALSE
    } else if alwaysDisable != 0 {
        TracePrint!(
            4u32, /* TRACE_LEVEL_INFORMATION */
            0u32, /* TRACE_FLAG_MCN */
            b"DeviceMediaChangeDeviceInstanceOverride: %s selected\n\0"
                .as_ptr() as *const i8,
            b"DISABLE\0".as_ptr() as *const i8,
        );
        NT_ASSERT!(NT_SUCCESS(status) as i32);
        status = STATUS_SUCCESS;
        *Enabled = 0; // FALSE
    } else if alwaysEnable != 0 {
        TracePrint!(
            4u32, /* TRACE_LEVEL_INFORMATION */
            0u32, /* TRACE_FLAG_MCN */
            b"DeviceMediaChangeDeviceInstanceOverride: %s selected\n\0"
                .as_ptr() as *const i8,
            b"ENABLE\0".as_ptr() as *const i8,
        );
        NT_ASSERT!(NT_SUCCESS(status) as i32);
        status = STATUS_SUCCESS;
        *Enabled = 1; // TRUE
    } else {
        TracePrint!(
            4u32, /* TRACE_LEVEL_INFORMATION */
            0u32, /* TRACE_FLAG_MCN */
            b"DeviceMediaChangeDeviceInstanceOverride: %s selected\n\0"
                .as_ptr() as *const i8,
            b"DEFAULT\0".as_ptr() as *const i8,
        );
        status = STATUS_UNSUCCESSFUL;
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn DeviceMediaChangeRegistryCallBack(
    ValueName: *mut u16,
    ValueType: u32,
    ValueData: *mut core::ffi::c_void,
    ValueLength: u32,
    Context: *mut core::ffi::c_void,
    EntryContext: *mut core::ffi::c_void,
) -> i32 {
    const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30
    const TRACE_FLAG_MCN: u32 = 12; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const TRUE: u32 = 1;

    PAGED_CODE!();
    // UNREFERENCED_PARAMETER(ValueName);
    let _ = ValueName;

    if Context.is_null() || EntryContext.is_null() {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"DeviceMediaChangeRegistryCallBack: NULL context should never be passed to registry call-back!\n\0".as_ptr() as *const i8,
        );
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    let value_found: *mut u32 = EntryContext as *mut u32;
    if *value_found != 0 {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"DeviceMediaChangeRegistryCallBack: already set to true\n\0".as_ptr() as *const i8,
        );
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    if ValueLength == core::mem::size_of::<u16>() as u32 {
        TracePrint!(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"DeviceMediaChangeRegistryCallBack: NULL string should never be passed to registry call-back!\n\0".as_ptr() as *const i8,
        );
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    if ValueType != ntkrnl_sys::constants::REG_SZ {
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    let device_string: *mut _UNICODE_STRING = Context as *mut _UNICODE_STRING;
    let key_value: *mut u16 = ValueData as *mut u16;
    let mut value_length = ValueLength;
    value_length -= core::mem::size_of::<u16>() as u32; // ignore the null character

    if value_length > (*device_string).Length as u32 {
        value_length = (*device_string).Length as u32;
    }

    if key_value.is_null() {
        return ntkrnl_sys::base::STATUS_SUCCESS;
    }

    // if the strings match, disable autorun
    if RtlCompareMemory(
        (*device_string).Buffer as *const core::ffi::c_void,
        key_value as *const core::ffi::c_void,
        value_length as ntkrnl_sys::ddk::SIZE_T,
    ) == value_length as ntkrnl_sys::ddk::SIZE_T
    {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_MCN,
            b"DeviceMediaChangeRegistryCallBack: Match found\n\0".as_ptr() as *const i8,
        );
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_MCN,
            b"DeviceMediaChangeRegistryCallBack: DeviceString at %p\n\0".as_ptr() as *const i8,
            (*device_string).Buffer,
        );
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_MCN,
            b"DeviceMediaChangeRegistryCallBack: KeyValue at %p\n\0".as_ptr() as *const i8,
            key_value,
        );
        *value_found = TRUE;
    }

    ntkrnl_sys::base::STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn DeviceIsMediaChangeDisabledDueToHardwareLimitation(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> i32 {
    // Call pagable code marker
    PAGED_CODE!();

    let mut status: NTSTATUS;
    let device_descriptor: PSTORAGE_DEVICE_DESCRIPTOR = (*DeviceExtension).DeviceDescriptor;
    let mut wdf_key: WDFKEY = core::ptr::null_mut();
    let mut service_key: HANDLE;
    let mut parameters: [RTL_QUERY_REGISTRY_TABLE; 2] = unsafe { core::mem::zeroed() };
    let mut device_unicode_string: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut device_string: ANSI_STRING = unsafe { core::mem::zeroed() };
    let mut media_change_notification_disabled: u32 = 0;

    status = WdfDriverOpenParametersRegistryKey(
        WdfGetDriver(),
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(wdf_key),
    );

    if !NT_SUCCESS(status) {
        // always take the safe path
        return 1; // TRUE
    }

    if NT_SUCCESS(status) {
        let null_multi_sz_buf: [u16; 1] = [0u16; 1];
        let null_multi_sz: *const u16 = null_multi_sz_buf.as_ptr();
        let auto_run_always_disable_buf: [u16; 21] = [
            'A' as u16, 'u' as u16, 't' as u16, 'o' as u16, 'R' as u16, 'u' as u16, 'n' as u16,
            'A' as u16, 'l' as u16, 'w' as u16, 'a' as u16, 'y' as u16, 's' as u16, 'D' as u16,
            'i' as u16, 's' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16, 0u16,
        ];
        let auto_run_always_disable: *const u16 = auto_run_always_disable_buf.as_ptr();

        let mut vendor_id: *const u8 = core::ptr::null();
        let mut product_id: *const u8 = core::ptr::null();
        let mut revision_id: *const u8 = core::ptr::null();
        let mut length: usize;
        let mut offset: usize;

        device_string.Buffer = core::ptr::null_mut();
        device_unicode_string.Buffer = core::ptr::null_mut();

        service_key = WdfRegistryWdmGetHandle(wdf_key);

        // there may be nothing to check against
        if ((*device_descriptor).VendorIdOffset == 0) && ((*device_descriptor).ProductIdOffset == 0) {
            status = STATUS_INTERNAL_ERROR;
        }

        // build deviceString using VendorId, Model and Revision
        if NT_SUCCESS(status) {
            length = 0;

            if (*device_descriptor).VendorIdOffset == 0 {
                vendor_id = core::ptr::null();
            } else {
                vendor_id = (device_descriptor as *const u8)
                    .add((*device_descriptor).VendorIdOffset as usize);
                length = strlen((vendor_id).cast());
            }

            if (*device_descriptor).ProductIdOffset == 0 {
                product_id = core::ptr::null();
            } else {
                product_id = (device_descriptor as *const u8)
                    .add((*device_descriptor).ProductIdOffset as usize);
                length += strlen((product_id).cast());
            }

            if (*device_descriptor).ProductRevisionOffset == 0 {
                revision_id = core::ptr::null();
            } else {
                revision_id = (device_descriptor as *const u8)
                    .add((*device_descriptor).ProductRevisionOffset as usize);
                length += strlen((revision_id).cast());
            }

            // allocate a buffer for the string
            device_string.Length = length as u16;
            device_string.MaximumLength = (length + 1) as u16;
            device_string.Buffer = ExAllocatePoolWithTag(
                NonPagedPoolNx,
                device_string.MaximumLength as usize,
                CDROM_TAG_AUTORUN_DISABLE,
            ) as *mut i8;

            if device_string.Buffer.is_null() {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"DeviceIsMediaChangeDisabledDueToHardwareLimitation: Unable to alloc string buffer\n\0".as_ptr() as *const i8,
                    0,
                );
                status = STATUS_INTERNAL_ERROR;
            }
        }

        if NT_SUCCESS(status) {
            offset = 0;

            if !vendor_id.is_null() {
                core::ptr::copy_nonoverlapping(
                    vendor_id,
                    device_string.Buffer.add(offset) as *mut u8,
                    strlen((vendor_id).cast()),
                );
                offset += strlen((vendor_id).cast());
            }

            if !product_id.is_null() {
                core::ptr::copy_nonoverlapping(
                    product_id,
                    device_string.Buffer.add(offset) as *mut u8,
                    strlen((product_id).cast()),
                );
                offset += strlen((product_id).cast());
            }

            if !revision_id.is_null() {
                core::ptr::copy_nonoverlapping(
                    revision_id,
                    device_string.Buffer.add(offset) as *mut u8,
                    strlen((revision_id).cast()),
                );
                offset += strlen((revision_id).cast());
            }

            NT_ASSERT!((offset == device_string.Length as usize) as i32);

            // Null-terminate
            *device_string.Buffer.add(device_string.Length as usize) = 0i8;

            // convert to unicode as registry deals with unicode strings
            status = RtlAnsiStringToUnicodeString(
                &raw mut device_unicode_string as *mut _,
                &raw mut device_string as *mut _,
                1,
            );

            if !NT_SUCCESS(status) {
                TracePrint!(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"DeviceIsMediaChangeDisabledDueToHardwareLimitation: cannot convert to unicode %lx\n\0"
                        .as_ptr() as *const i8,
                    status,
                );
            }
        }

        if NT_SUCCESS(status) {
            // query the value, setting valueFound to true if found
            // The callback is defined `extern "system"` (stdcall ABI on i386,
            // C ABI on amd64/arm64) — exactly the field's ABI. On i386 rustc
            // still nominally distinguishes `"system"` from the field's
            // `"stdcall"` fn-ptr type, so transmute to the inferred field type
            // (an ABI-identical, zero-cost reinterpretation on every arch).
            parameters[0].QueryRoutine = Some(core::mem::transmute(DeviceMediaChangeRegistryCallBack as unsafe extern "system" fn(_, _, _, _, _, _) -> _));
            parameters[0].Flags = RTL_QUERY_REGISTRY_REQUIRED;
            parameters[0].Name = auto_run_always_disable;
            parameters[0].EntryContext =
                core::ptr::addr_of_mut!(media_change_notification_disabled) as *mut core::ffi::c_void;
            parameters[0].DefaultType = REG_MULTI_SZ;
            parameters[0].DefaultData = null_multi_sz as *mut core::ffi::c_void;
            parameters[0].DefaultLength = 0;

            status = RtlQueryRegistryValues(
                RTL_REGISTRY_HANDLE,
                service_key as *const u16,
                parameters.as_mut_ptr(),
                &raw const device_unicode_string as *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
            UNREFERENCED_PARAMETER!((status) as usize);
        }
    }

    // Cleanup
    {
        FREE_POOL(device_string.Buffer as *mut core::ffi::c_void);
        if !device_unicode_string.Buffer.is_null() {
            RtlFreeUnicodeString(&raw mut device_unicode_string as *mut _);
        }
        WdfRegistryClose(wdf_key);
    }

    if media_change_notification_disabled > 0 {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"DeviceIsMediaChangeDisabledDueToHardwareLimitation: Device is on MCN disable list\n\0"
                .as_ptr() as *const i8,
            0,
        );
    }

    return if media_change_notification_disabled > 0 { 1 } else { 0 };
}
#[no_mangle]
pub unsafe extern "C" fn DeviceIsMediaChangeDisabledForClass(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION
) -> i32 {
    let mut status: i32;
    let mut serviceKey: WDFKEY = core::ptr::null_mut();
    let mut parametersKey: WDFKEY = core::ptr::null_mut();
    let mut parameterKeyName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut valueName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut mcnRegistryValue: u32 = 1;

    status = WdfDriverOpenParametersRegistryKey(
        WdfGetDriver(),
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(serviceKey)
    );

    if !NT_SUCCESS(status) {
        let selected_str = if mcnRegistryValue != 0 {
            b"Enabled\0".as_ptr() as *const i8
        } else {
            b"Disabled\0".as_ptr() as *const i8
        };
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"DeviceIsMediaChangeDisabledForClass: Defaulting to %s\n\0".as_ptr() as *const i8,
            selected_str
        );
        return (mcnRegistryValue == 0) as i32;
    } else {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(parameterKeyName),
            PARAMETER_KEY_NAME.as_ptr()
        );

        status = WdfRegistryOpenKey(
            serviceKey,
            core::ptr::addr_of_mut!(parameterKeyName),
            ntkrnl_sys::constants::KEY_READ,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(parametersKey)
        );
    }

    if !NT_SUCCESS(status) {
        parametersKey = core::ptr::null_mut();
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(valueName),
        AUTORUN_VALUE_NAME.as_ptr()
    );

    status = WdfRegistryQueryULong(
        serviceKey,
        core::ptr::addr_of_mut!(valueName),
        core::ptr::addr_of_mut!(mcnRegistryValue)
    );

    if NT_SUCCESS(status) {
        TracePrint!(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_MCN,
            b"DeviceIsMediaChangeDisabledForClass: <Service>/Autorun flag = %d\n\0".as_ptr() as *const i8,
            mcnRegistryValue
        );
    }

    if parametersKey != core::ptr::null_mut() {
        status = WdfRegistryQueryULong(
            parametersKey,
            core::ptr::addr_of_mut!(valueName),
            core::ptr::addr_of_mut!(mcnRegistryValue)
        );

        if NT_SUCCESS(status) {
            TracePrint!(
                TRACE_LEVEL_VERBOSE,
                TRACE_FLAG_MCN,
                b"DeviceIsMediaChangeDisabledForClass: <Service>/Parameters/Autorun flag = %d\n\0".as_ptr() as *const i8,
                mcnRegistryValue
            );
        }

        WdfRegistryClose(parametersKey);
    }

    WdfRegistryClose(serviceKey);

    let status_str = if mcnRegistryValue != 0 {
        b"on\0".as_ptr() as *const i8
    } else {
        b"off\0".as_ptr() as *const i8
    };
    TracePrint!(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_MCN,
        b"DeviceIsMediaChangeDisabledForClass: Autoplay for device %p is %s\n\0".as_ptr() as *const i8,
        (*DeviceExtension).DeviceObject as *mut core::ffi::c_void,
        status_str
    );

    return (mcnRegistryValue == 0) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceEnableMediaChangeDetection(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    FileObjectContext: *mut FILE_OBJECT_CONTEXT,
    IgnorePreviousMediaChanges: i32,
) {
    unsafe {
        // PAGED_CODE();
        PAGED_CODE!();

        // if (FileObjectContext)
        if !FileObjectContext.is_null() {
            InterlockedDecrement(core::ptr::addr_of_mut!((*FileObjectContext).McnDisableCount) as *mut i32);
        }

        let info = (*DeviceExtension).MediaChangeDetectionInfo;

        // if (info == NULL)
        if info.is_null() {
            TracePrint!(0x4 /* TRACE_LEVEL_INFORMATION */, 0x2 /* TRACE_FLAG_MCN */,
                       b"DeviceEnableMediaChangeDetection: not initialized\n\0".as_ptr() as *const i8);
            return;
        }

        // (VOID) KeWaitForMutexObject(...)
        let _ = KeWaitForMutexObject(
            &raw mut (*info).MediaChangeMutex as *mut core::ffi::c_void,
            _KWAIT_REASON_UserRequest as i32,
            0i32,
            0, // FALSE
            core::ptr::null(), // NULL
        );

        // oldCount = --info->MediaChangeDetectionDisableCount;
        let oldCount = {
            let p = info;
            let cnt = &raw mut (*p).MediaChangeDetectionDisableCount;
            *cnt -= 1;
            *cnt
        };

        // NT_ASSERT(oldCount >= 0);
        NT_ASSERT!((oldCount >= 0) as i32);

        TracePrint!(0x5 /* TRACE_LEVEL_VERBOSE */, 0x2 /* TRACE_FLAG_MCN */,
                   b"DeviceEnableMediaChangeDetection: Disable count reduced to %d - \n\0".as_ptr() as *const i8,
                   oldCount);

        if oldCount == 0 {
            if IgnorePreviousMediaChanges != 0 {
                (*info).LastReportedMediaDetectionState = (*info).LastKnownMediaDetectionState;
            }

            TracePrint!(0x4 /* TRACE_LEVEL_INFORMATION */, 0x2 /* TRACE_FLAG_MCN */,
                       b"MCN is enabled\n\0".as_ptr() as *const i8);
        } else {
            TracePrint!(0x5 /* TRACE_LEVEL_VERBOSE */, 0x2 /* TRACE_FLAG_MCN */,
                       b"MCD still disabled\n\0".as_ptr() as *const i8);
        }

        // KeReleaseMutex(&info->MediaChangeMutex, FALSE);
        KeReleaseMutex(&raw mut (*info).MediaChangeMutex, 0);

        return;
    }
}

#[no_mangle]
pub unsafe extern "C" fn DeviceDisableMediaChangeDetection(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    FileObjectContext: *mut FILE_OBJECT_CONTEXT,
) {
    // PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
    let info = (*DeviceExtension).MediaChangeDetectionInfo;

    // PAGED_CODE();
    PAGED_CODE!();

    // if (FileObjectContext) ...
    if !FileObjectContext.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*FileObjectContext).McnDisableCount) as *mut i32);
    }

    // if (info == NULL) return;
    if info.is_null() {
        return;
    }

    // (VOID) KeWaitForMutexObject(&info->MediaChangeMutex, UserRequest, KernelMode, FALSE, NULL);
    let _ = KeWaitForMutexObject(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex) as *mut core::ffi::c_void,
        ntkrnl_sys::ddk::_KWAIT_REASON_UserRequest as i32,
        0i32,
        0, // FALSE
        core::ptr::null(),
    );

    // info->MediaChangeDetectionDisableCount++;
    (*info).MediaChangeDetectionDisableCount += 1;

    // TracePrint!(...)
    TracePrint!(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_MCN,
        b"DisableMediaChangeDetection: disable count is %d\n\0".as_ptr() as *const i8,
        (*info).MediaChangeDetectionDisableCount,
    );

    // KeReleaseMutex(&info->MediaChangeMutex, FALSE);
    let _ = ntkrnl_sys::ddk::KeReleaseMutex(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex),
        0,
    );
}

const APC_LEVEL: KIRQL = 1;
#[no_mangle]
pub unsafe extern "C" fn DeviceReleaseMcnResources(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    let irql = KeGetCurrentIrql();
    if irql > APC_LEVEL {
        // PAGED_CODE assertion: ASSERT(FALSE)
    }

    let info = (*DeviceExtension).MediaChangeDetectionInfo;
    if info.is_null() {
        return;
    }

    if !(*info).Gesn.Mdl.is_null() {
        let irp = WdfRequestWdmGetIrp((*info).MediaChangeRequest);
        IoFreeMdl((*info).Gesn.Mdl);
        (*irp).MdlAddress = core::ptr::null_mut();
    }

    IoFreeIrp((*info).MediaChangeSyncIrp);

    FREE_POOL((*info).Gesn.Buffer as *mut core::ffi::c_void);
    FREE_POOL((*info).SenseBuffer as *mut core::ffi::c_void);

    if !(*info).DisplayStateCallbackHandle.is_null() {
        PoUnregisterPowerSettingCallback((*info).DisplayStateCallbackHandle);
        (*info).DisplayStateCallbackHandle = core::ptr::null_mut();
    }

    FREE_POOL(info as *mut core::ffi::c_void);

    (*DeviceExtension).MediaChangeDetectionInfo = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RequestMcnSyncIrpCompletion(
    DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::IRP,
    Context: *mut core::ffi::c_void,
) -> i32 {
    const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32;

    let mut device_extension: *mut CDROM_DEVICE_EXTENSION;
    let mut info: *mut MEDIA_CHANGE_DETECTION_INFO;

    if Context.is_null() {
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    device_extension = Context as *mut CDROM_DEVICE_EXTENSION;
    info = (*device_extension).MediaChangeDetectionInfo;

    #[cfg(not(debug_assertions))]
    {
        let _ = Irp;
    }
    let _ = DeviceObject;

    ntkrnl_sys::ASSERT!(Irp == (*info).MediaChangeSyncIrp);

    ntkrnl_sys::ddk::IoReuseIrp((*info).MediaChangeSyncIrp, ntkrnl_sys::constants::STATUS_NOT_SUPPORTED);

    let atomic_ptr = &raw const (*info).MediaChangeRequestInUse as *const i32 as *const core::sync::atomic::AtomicI32;
    let _ = (*atomic_ptr).compare_exchange(
        1,
        0,
        core::sync::atomic::Ordering::AcqRel,
        core::sync::atomic::Ordering::Acquire,
    );

    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn RequestSetupMcnSyncIrp(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    let mut irp: *mut IRP;
    let mut irpStack: *mut IO_STACK_LOCATION;
    let mut nextIrpStack: *mut IO_STACK_LOCATION;

    irp = (*(*DeviceExtension).MediaChangeDetectionInfo).MediaChangeSyncIrp;
    NT_ASSERT!(if irp.is_null() { 0 } else { 1 });
    IoSetNextIrpStackLocation(irp);

    irpStack = IoGetCurrentIrpStackLocation(irp);
    (*irpStack).DeviceObject = (*DeviceExtension).DeviceObject;

    nextIrpStack = IoGetNextIrpStackLocation(irp);
    (*nextIrpStack).Flags |= SL_OVERRIDE_VERIFY_VOLUME as u8;
    (*nextIrpStack).MajorFunction = IRP_MJ_DEVICE_CONTROL;
    (*nextIrpStack).Parameters.Others.Argument1 =
        RequestSetupMcnSyncIrp as *const () as *mut core::ffi::c_void;
    (*nextIrpStack).Parameters.Others.Argument2 =
        RequestSetupMcnSyncIrp as *const () as *mut core::ffi::c_void;
    (*nextIrpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_MCN_SYNC_FAKE_IOCTL;
    (*nextIrpStack).Parameters.Others.Argument4 =
        RequestSetupMcnSyncIrp as *const () as *mut core::ffi::c_void;

    IoSetCompletionRoutine(
        irp,
        core::mem::transmute::<_, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(
            RequestMcnSyncIrpCompletion
                as unsafe extern "system" fn(ntkrnl_sys::ddk::PDEVICE_OBJECT, *mut ntkrnl_sys::ddk::IRP, *mut core::ffi::c_void) -> i32,
        ),
        DeviceExtension as *mut core::ffi::c_void,
        1,
        1,
        1,
    );
}
#[no_mangle]
pub unsafe extern "system" fn DeviceMainTimerTickHandler(Timer: WDFTIMER) {

    let mut deviceExtension: PCDROM_DEVICE_EXTENSION;
    let mut dataLength: usize = 0;

    deviceExtension = WdfObjectGetTypedContext(
        WdfTimerGetParentObject(Timer),
        &CDROM_DEVICE_EXTENSION_TYPE_INFO,
    ) as PCDROM_DEVICE_EXTENSION;

    RequestHandleEventNotification(
        deviceExtension,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dataLength),
    );
}

#[no_mangle]
pub unsafe extern "C" fn DeviceEnableMainTimer(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) -> i32 {
    let mut status: i32 = STATUS_SUCCESS;

    let media_change_info = (*DeviceExtension).MediaChangeDetectionInfo;
    if media_change_info.is_null()
        || (*media_change_info).AsynchronousNotificationSupported as i32 != 0
    {
        return status;
    }

    if (*DeviceExtension).MainTimer.is_null() {
        let mut timer_config: WDF_TIMER_CONFIG = core::mem::zeroed();
        timer_config.Size = core::mem::size_of::<WDF_TIMER_CONFIG>() as u32;
        timer_config.EvtTimerFunc = Some(core::mem::transmute(DeviceMainTimerTickHandler as unsafe extern "system" fn(_)));

        if ((*DeviceExtension).DeviceAdditionalData.HackFlags & CDROM_HACK_MSFT_VIRTUAL_ODD as ntkrnl_sys::ddk::ULONG_PTR) != 0 {
            timer_config.Period = 2000;
        } else {
            timer_config.Period = 1000;
        }

        timer_config.TolerableDelay = 500;
        timer_config.AutomaticSerialization = 0; // FALSE

        let mut timer_attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
        timer_attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
        timer_attributes.ParentObject = (*DeviceExtension).Device as *mut core::ffi::c_void;
        timer_attributes.ExecutionLevel = WdfExecutionLevelInheritFromParent;
        // C WDF_OBJECT_ATTRIBUTES_INIT sets this; zeroed struct left it 0
        // (WdfSynchronizationScopeInvalid → WDF rejects the timer). semdiff.
        timer_attributes.SynchronizationScope = 1; // WdfSynchronizationScopeInheritFromParent

        status = WdfTimerCreate(
            &mut timer_config,
            &mut timer_attributes,
            &mut (*DeviceExtension).MainTimer,
        );
    }

    if status >= 0 {
        WdfTimerStart(
            (*DeviceExtension).MainTimer,
            -10000 * 100, // WDF_REL_TIMEOUT_IN_MS(100)
        );

        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"DeviceEnableMainTimer: Once a second timer enabled  for WDFDEVICE %p\n\0".as_ptr() as *const i8,
        );
    } else {
        TracePrint!(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"DeviceEnableMainTimer: WDFDEVICE %p, Status %lx  initializing timer\n\0".as_ptr() as *const i8,
        );
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceDisableMainTimer(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    // PAGED_CODE();  // macro not available, no-op in Rust

    let mcn_info = (*DeviceExtension).MediaChangeDetectionInfo;
    if mcn_info.is_null() || (*mcn_info).AsynchronousNotificationSupported != 0 {
        // Asynchronous Notification is enabled, timer not needed.
        return;
    }

    let main_timer = (*DeviceExtension).MainTimer;
    if !main_timer.is_null() {
        //
        // we are only going to stop the actual timer in remove device routine.
        // it is the responsibility of the code within the timer routine to
        // check if the device is removed and not processing io for the final
        // call.
        // this keeps the code clean and prevents lots of bugs.
        //
        WdfTimerStop(main_timer, 1);

        TracePrint!(
            4, // TRACE_LEVEL_INFORMATION
            1, // TRACE_FLAG_MCN
            b"DeviceDisableMainTimer: Once a second timer disabled for device %p\n\0".as_ptr() as *const i8,
            (*DeviceExtension).Device as *const core::ffi::c_void,
        );
    } else {
        TracePrint!(
            3, // TRACE_LEVEL_WARNING
            1, // TRACE_FLAG_MCN
            b"DeviceDisableMainTimer: Timer never enabled\n\0".as_ptr() as *const i8,
            core::ptr::null::<core::ffi::c_void>(),
        );
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn RequestHandleMcnControl(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    Request: WDFREQUEST,
    DataLength: *mut usize,
) -> NTSTATUS {
    // PAGED_CODE();
    let mut status: NTSTATUS;
    let mut file_object: WDFFILEOBJECT;
    let mut file_object_context: *mut FILE_OBJECT_CONTEXT = core::ptr::null_mut();
    let mut media_removal: *mut ntkrnl_sys::ddk::PREVENT_MEDIA_REMOVAL = core::ptr::null_mut();

    *DataLength = 0;

    status = WdfRequestRetrieveInputBuffer(
        Request,
        core::mem::size_of::<ntkrnl_sys::ddk::PREVENT_MEDIA_REMOVAL>(),
        &mut media_removal as *mut *mut ntkrnl_sys::ddk::PREVENT_MEDIA_REMOVAL as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    );

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        file_object = WdfRequestGetFileObject(Request);

        if !file_object.is_null() {
            file_object_context = FileObjectGetContext(file_object);
        }

        if file_object_context.is_null()
            && WdfRequestGetRequestorMode(Request) == 0i8
        {
            file_object_context = core::ptr::addr_of_mut!(
                (*DeviceExtension).KernelModeMcnContext
            ) as *mut FILE_OBJECT_CONTEXT;
        }

        if file_object_context.is_null() {
            status = ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
        }
    }

    if ntkrnl_sys::base::NT_SUCCESS(status) {
        if (*media_removal).PreventMediaRemoval != 0 {
            DeviceDisableMediaChangeDetection(DeviceExtension, file_object_context);
        } else {
            if (*file_object_context).McnDisableCount == 0 {
                status = STATUS_INVALID_DEVICE_STATE;
            } else {
                DeviceEnableMediaChangeDetection(DeviceExtension, file_object_context, 1);
            }
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
//     autorun.c
//
// Abstract:
//
//     Code for support of media change detection in the cd/dvd driver
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
// #include "mmc.h"
// #include "ioctl.h"
//
// #include "ntstrsafe.h"
//
// #ifdef DEBUG_USE_WPP
// #include "autorun.tmh"
// #endif
//
// #define GESN_TIMEOUT_VALUE (0x4)
// #define GESN_BUFFER_SIZE (0x8)
// #define GESN_DEVICE_BUSY_LOWER_THRESHOLD_100_MS   (2)
//
// #define MAXIMUM_IMMEDIATE_MCN_RETRIES (0x20)
// #define MCN_REG_SUBKEY_NAME                   (L"MediaChangeNotification")
// #define MCN_REG_AUTORUN_DISABLE_INSTANCE_NAME (L"AlwaysDisableMCN")
// #define MCN_REG_AUTORUN_ENABLE_INSTANCE_NAME  (L"AlwaysEnableMCN")
//
// //
// // Only send polling irp when device is fully powered up and a
// // power down irp is not in progress.
// //
// // NOTE:   This helps close a window in time where a polling irp could cause
// //         a drive to spin up right after it has powered down. The problem is
// //         that SCSIPORT, ATAPI and SBP2 will be in the process of powering
// //         down (which may take a few seconds), but won't know that. It would
// //         then get a polling irp which will be put into its queue since it
// //         the disk isn't powered down yet. Once the disk is powered down it
// //         will find the polling irp in the queue and then power up the
// //         device to do the poll. They do not want to check if the polling
// //         irp has the SRB_NO_KEEP_AWAKE flag here since it is in a critical
// //         path and would slow down all I/Os. A better way to fix this
// //         would be to serialize the polling and power down irps so that
// //         only one of them is sent to the device at a time.
// //
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// BOOLEAN
// DeviceIsMediaChangeDisabledDueToHardwareLimitation(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// BOOLEAN
// DeviceIsMediaChangeDisabledForClass(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceMediaChangeDeviceInstanceOverride(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      PBOOLEAN                Enabled
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeMcn(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      BOOLEAN                  AllowDriveToSleep
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeGesn(
//      PCDROM_DEVICE_EXTENSION      DeviceExtension
//     );
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// GesnDataInterpret(
//       PCDROM_DEVICE_EXTENSION             DeviceExtension,
//       PNOTIFICATION_EVENT_STATUS_HEADER   Header,
//      PBOOLEAN                            ResendImmediately
//     );
//
// RTL_QUERY_REGISTRY_ROUTINE DeviceMediaChangeRegistryCallBack;
//
// EVT_WDF_WORKITEM DeviceDisableGesn;
//
// #if ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceInitializeMediaChangeDetection)
// #pragma alloc_text(PAGE, DeviceEnableMediaChangeDetection)
// #pragma alloc_text(PAGE, DeviceDisableMediaChangeDetection)
// #pragma alloc_text(PAGE, DeviceSendDelayedMediaChangeNotifications)
// #pragma alloc_text(PAGE, DeviceReleaseMcnResources)
// #pragma alloc_text(PAGE, DeviceMediaChangeRegistryCallBack)
// #pragma alloc_text(PAGE, DeviceInitializeMcn)
// #pragma alloc_text(PAGE, DeviceDisableGesn)
//
// #pragma alloc_text(PAGE, DeviceIsMediaChangeDisabledDueToHardwareLimitation)
// #pragma alloc_text(PAGE, DeviceMediaChangeDeviceInstanceOverride)
// #pragma alloc_text(PAGE, DeviceIsMediaChangeDisabledForClass)
//
// #pragma alloc_text(PAGE, DeviceDisableMainTimer)
//
// #pragma alloc_text(PAGE, GesnDataInterpret)
//
// #pragma alloc_text(PAGE, RequestSetupMcnRequest)
// #pragma alloc_text(PAGE, RequestPostWorkMcnRequest)
// #pragma alloc_text(PAGE, RequestSendMcnRequest)
//
// //
// // DeviceEnableMainTimer is called by EvtDeviceD0Entry which can't be made pageable
// // so neither is DeviceEnableMainTimer
// //
// //#pragma alloc_text(PAGE, DeviceEnableMainTimer)
//
// #pragma alloc_text(PAGE, DeviceInitializeGesn)
//
// #pragma alloc_text(PAGE, RequestHandleMcnControl)
//
// #endif
//
// #pragma warning(push)
// #pragma warning(disable:4152) // nonstandard extension, function/data pointer conversion in expression
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// GesnDataInterpret(
//       PCDROM_DEVICE_EXTENSION             DeviceExtension,
//       PNOTIFICATION_EVENT_STATUS_HEADER   Header,
//      PBOOLEAN                            ResendImmediately
//     )
// /*++
//
// Routine Description:
//
//     This routine will interpret the data returned for a GESN command, and
//     (if appropriate) set the media change event, and broadcast the
//     appropriate events to user mode for applications who care.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
//     Header - the resulting data from a GESN event.
//         requires at least EIGHT valid bytes (header == 4, data == 4)
//
//     ResendImmediately - whether or not to immediately resend the request.
//         this should be FALSE if there was no event, FALSE if the reported
//         event was of the DEVICE BUSY class, else true.
//
// Return Value:
//
//     STATUS_SUCCESS if successful, an error code otherwise
//
// Notes:
//
//     DataBuffer must be at least four bytes of valid data (header == 4 bytes),
//     and have at least eight bytes of allocated memory (all events == 4 bytes).
//
//     The call to StartNextPacket may occur before this routine is completed.
//     the operational change notifications are informational in nature, and
//     while useful, are not neccessary to ensure proper operation.  For example,
//     if the device morphs to no longer supporting WRITE commands, all further
//     write commands will fail.  There exists a small timing window wherein
//     IOCTL_IS_DISK_WRITABLE may be called and get an incorrect response.  If
//     a device supports software write protect, it is expected that the
//     application can handle such a case.
//
//     NOTE: perhaps setting the updaterequired byte to one should be done here.
//     if so, it relies upon the setting of a 32-byte value to be an atomic
//     operation.  unfortunately, there is no simple way to notify a class driver
//     which wants to know that the device behavior requires updating.
//
//     Not ready events may be sent every second.  For example, if we were
//     to minimize the number of asynchronous notifications, an application may
//     register just after a large busy time was reported.  This would then
//     prevent the application from knowing the device was busy until some
//     arbitrarily chosen timeout has occurred.  Also, the GESN request would
//     have to still occur, since it checks for non-busy events (such as user
//     keybutton presses and media change events) as well.  The specification
//     states that the lower-numered events get reported first, so busy events,
//     while repeating, will only be reported when all other events have been
//     cleared from the device.
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PMEDIA_CHANGE_DETECTION_INFO    info = DeviceExtension->MediaChangeDetectionInfo;
//     LONG                            dataLength = 0;
//     LONG                            requiredLength = 0;
//     BOOLEAN                         inHomePosition = FALSE;
//
//     PAGED_CODE();
//
//     // note: don't allocate anything in this routine so that we can
//     //       always just 'return'.
//     *ResendImmediately = FALSE;
//
//     if (Header->NEA)
//     {
//         return status;
//     }
//     if (Header->NotificationClass == NOTIFICATION_NO_CLASS_EVENTS)
//     {
//         return status;
//     }
//
//     // HACKHACK - REF #0001
//     // This loop is only taken initially, due to the inability to reliably
//     // auto-detect drives that report events correctly at boot.  When we
//     // detect this behavior during the normal course of running, we will
//     // disable the hack, allowing more efficient use of the system.  This
//     // should occur "nearly" instantly, as the drive should have multiple
//     // events queue'd (ie. power, morphing, media).
//     if (info->Gesn.HackEventMask)
//     {
//         // all events use the low four bytes of zero to indicate
//         // that there was no change in status.
//         UCHAR thisEvent = Header->ClassEventData[0] & 0xf;
//         UCHAR lowestSetBit;
//         UCHAR thisEventBit = (1 << Header->NotificationClass);
//
//         if (!TEST_FLAG(info->Gesn.EventMask, thisEventBit))
//         {
//             // The drive is reporting an event that wasn't requested
//             return STATUS_DEVICE_PROTOCOL_ERROR;
//         }
//
//         // some bit magic here... this results in the lowest set bit only
//         lowestSetBit = info->Gesn.EventMask;
//         lowestSetBit &= (info->Gesn.EventMask - 1);
//         lowestSetBit ^= (info->Gesn.EventMask);
//
//         if (thisEventBit != lowestSetBit)
//         {
//             // HACKHACK - REF #0001
//             // the first time we ever see an event set that is not the lowest
//             // set bit in the request (iow, highest priority), we know that the
//             // hack is no longer required, as the device is ignoring "no change"
//             // events when a real event is waiting in the other requested queues.
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GESN::NONE: Compliant drive found, "
//                        "removing GESN hack (%x, %x)\n",
//                        thisEventBit, info->Gesn.EventMask));
//
//             info->Gesn.HackEventMask = FALSE;
//         }
//         else if (thisEvent == 0)  // NOTIFICATION_*_EVENT_NO_CHANGE
//         {
//             // HACKHACK - REF #0001
//             // note: this hack prevents poorly implemented firmware from constantly
//             //       returning "No Event".  we do this by cycling through the
//             //       supported list of events here.
//             SET_FLAG(info->Gesn.NoChangeEventMask, thisEventBit);
//             CLEAR_FLAG(info->Gesn.EventMask, thisEventBit);
//
//             // if we have cycled through all supported event types, then
//             // we need to reset the events we are asking about. else we
//             // want to resend this request immediately in case there was
//             // another event pending.
//             if (info->Gesn.EventMask == 0)
//             {
//                 info->Gesn.EventMask         = info->Gesn.NoChangeEventMask;
//                 info->Gesn.NoChangeEventMask = 0;
//             }
//             else
//             {
//                 *ResendImmediately = TRUE;
//             }
//             return status;
//         }
//
//     } // end if (info->Gesn.HackEventMask)
//
//     dataLength = (Header->EventDataLength[0] << 8) |
//                  (Header->EventDataLength[1] & 0xff);
//     dataLength -= 2;
//     requiredLength = 4; // all events are four bytes
//
//     if (dataLength < requiredLength)
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "error - GESN returned only %x bytes data for fdo %p\n",
//                    dataLength, DeviceExtension->DeviceObject));
//
//         return STATUS_DEVICE_PROTOCOL_ERROR;
//     }
//
//     if (dataLength > requiredLength)
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "error - GESN returned too many (%x) bytes data for fdo %p\n",
//                    dataLength, DeviceExtension->DeviceObject));
//     }
//
//     if ((Header->ClassEventData[0] & 0xf) == 0)
//     {
//         // a zero event is a "no change event, so do not retry
//         return status;
//     }
//
//     // because a event other than "no change" occurred,
//     // we should immediately resend this request.
//     *ResendImmediately = TRUE;
//
//     switch (Header->NotificationClass)
//     {
//
//     case NOTIFICATION_OPERATIONAL_CHANGE_CLASS_EVENTS:  // 0x01
//     {
//         PNOTIFICATION_OPERATIONAL_STATUS opChangeInfo =
//                                             (PNOTIFICATION_OPERATIONAL_STATUS)(Header->ClassEventData);
//         ULONG event;
//
//         if (opChangeInfo->OperationalEvent == NOTIFICATION_OPERATIONAL_EVENT_CHANGE_REQUESTED)
//         {
//             break;
//         }
//
//         event = (opChangeInfo->Operation[0] << 8) |
//                 (opChangeInfo->Operation[1]     ) ;
//
//         // Workaround some hardware that is buggy but prevalent in the market
//         // This hardware has the property that it will report OpChange events repeatedly,
//         // causing us to retry immediately so quickly that we will eventually disable
//         // GESN to prevent an infinite loop.
//         // (only one valid OpChange event type now, only two ever defined)
//         if (info->MediaChangeRetryCount >= 4)
//         {
//             //
//             // HACKHACK - REF #0002
//             // Some drives incorrectly report OpChange/Change (001b/0001h) events
//             // continuously when the tray has been ejected.  This causes this routine
//             // to set ResendImmediately to "TRUE", and that results in our cycling
//             // 32 times immediately resending.  At that point, we give up detecting
//             // the infinite retry loop, and disable GESN on these drives.  This
//             // prevents Media Eject Request (from eject button) from being reported.
//             // Thus, instead we should attempt to workaround this issue by detecting
//             // this behavior.
//             //
//
//             static UCHAR const OpChangeMask = 0x02;
//
//             // At least one device reports "temporarily busy" (which is useless) on eject
//             // At least one device reports "OpChange" repeatedly when re-inserting media
//             // All seem to work well using this workaround
//
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_MCN,
//                         "GESN OpChange events are broken.  Working around this problem in software (for WDFDEVICE %p)\n",
//                         DeviceExtension->Device));
//
//             // OpChange is not the only bit set -- Media class is required....
//             NT_ASSERT(CountOfSetBitsUChar(info->Gesn.EventMask) != 1);
//
//             // Force the use of the hackhack (ref #0001) to workaround the
//             // issue noted this hackhack (ref #0002).
//             SET_FLAG(info->Gesn.NoChangeEventMask, OpChangeMask);
//             CLEAR_FLAG(info->Gesn.EventMask, OpChangeMask);
//             info->Gesn.HackEventMask = TRUE;
//
//             // don't request the opChange event again.  use the method
//             // defined by hackhack (ref #0001) as the workaround.
//             if (info->Gesn.EventMask == 0)
//             {
//                 info->Gesn.EventMask = info->Gesn.NoChangeEventMask;
//                 info->Gesn.NoChangeEventMask = 0;
//                 *ResendImmediately = FALSE;
//             }
//             else
//             {
//                 *ResendImmediately = TRUE;
//             }
//
//             break;
//         }
//
//
//         if ((event == NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_ADDED) |
//             (event == NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_CHANGE))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GESN says features added/changed for WDFDEVICE %p\n",
//                        DeviceExtension->Device));
//
//             // don't notify that new media arrived, just set the
//             // DO_VERIFY to force a FS reload.
//
//             if (IsVolumeMounted(DeviceExtension->DeviceObject))
//             {
//                 SET_FLAG(DeviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME);
//             }
//
//             // Call error handler with
//             // a "fake" media change error in case it needs to update
//             // internal structures as though a media change occurred.
//             {
//                 SCSI_REQUEST_BLOCK  srb = {0};
//                 SENSE_DATA          sense = {0};
//                 NTSTATUS            tempStatus;
//                 BOOLEAN             retry;
//
//                 tempStatus = STATUS_MEDIA_CHANGED;
//                 retry = FALSE;
//
//                 srb.CdbLength = 6;
//                 srb.Length    = sizeof(SCSI_REQUEST_BLOCK);
//                 srb.SrbStatus = SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_ERROR;
//                 srb.SenseInfoBuffer = &sense;
//                 srb.SenseInfoBufferLength = sizeof(SENSE_DATA);
//
//                 sense.AdditionalSenseLength = sizeof(SENSE_DATA) -
//                                             RTL_SIZEOF_THROUGH_FIELD(SENSE_DATA, AdditionalSenseLength);
//
//                 sense.SenseKey = SCSI_SENSE_UNIT_ATTENTION;
//                 sense.AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED;
//
//                 if (DeviceExtension->DeviceAdditionalData.ErrorHandler)
//                 {
//                     DeviceExtension->DeviceAdditionalData.ErrorHandler(DeviceExtension,
//                                                                        &srb,
//                                                                        &tempStatus,
//                                                                        &retry);
//                 }
//             } // end error handler
//
//         }
//         break;
//     }
//
//     case NOTIFICATION_EXTERNAL_REQUEST_CLASS_EVENTS:  // 0x3
//     {
//         PNOTIFICATION_EXTERNAL_STATUS externalInfo =
//                                         (PNOTIFICATION_EXTERNAL_STATUS)(Header->ClassEventData);
//         DEVICE_EVENT_EXTERNAL_REQUEST externalData = {0};
//
//         // unfortunately, due to time constraints, we will only notify
//         // about keys being pressed, and not released.  this makes keys
//         // single-function, but simplifies the code significantly.
//         if (externalInfo->ExternalEvent != NOTIFICATION_EXTERNAL_EVENT_BUTTON_DOWN)
//         {
//             break;
//         }
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "GESN::EXTERNAL: Event: %x Status %x Req %x\n",
//                    externalInfo->ExternalEvent, externalInfo->ExternalStatus,
//                    (externalInfo->Request[0] << 8) | externalInfo->Request[1]
//                    ));
//
//         externalData.Version = 1;
//         externalData.DeviceClass = 0;
//         externalData.ButtonStatus = externalInfo->ExternalEvent;
//         externalData.Request = (externalInfo->Request[0] << 8) |
//                                (externalInfo->Request[1] & 0xff);
//         KeQuerySystemTime(&(externalData.SystemTime));
//         externalData.SystemTime.QuadPart *= (LONGLONG)KeQueryTimeIncrement();
//
//         DeviceSendNotification(DeviceExtension,
//                                &GUID_IO_DEVICE_EXTERNAL_REQUEST,
//                                sizeof(DEVICE_EVENT_EXTERNAL_REQUEST),
//                                &externalData);
//
//         return status;
//     }
//
//     case NOTIFICATION_MEDIA_STATUS_CLASS_EVENTS:  // 0x4
//     {
//         PNOTIFICATION_MEDIA_STATUS mediaInfo =
//                                     (PNOTIFICATION_MEDIA_STATUS)(Header->ClassEventData);
//
//         if ((mediaInfo->MediaEvent == NOTIFICATION_MEDIA_EVENT_NEW_MEDIA) ||
//             (mediaInfo->MediaEvent == NOTIFICATION_MEDIA_EVENT_MEDIA_CHANGE))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GESN::MEDIA ARRIVAL, Status %x\n",
//                        mediaInfo->MediaStatus));
//
//             if (IsVolumeMounted(DeviceExtension->DeviceObject))
//             {
//                 SET_FLAG(DeviceExtension->DeviceObject->Flags, DO_VERIFY_VOLUME);
//             }
//             DeviceSetMediaChangeStateEx(DeviceExtension,
//                                         MediaPresent,
//                                         NULL);
//
//             // If media is inserted into slot loading type, mark the device active
//             // to not power off.
//             if ((DeviceExtension->ZeroPowerODDInfo != NULL) &&
//                 (DeviceExtension->ZeroPowerODDInfo->LoadingMechanism == LOADING_MECHANISM_CADDY) &&
//                 (DeviceExtension->ZeroPowerODDInfo->Load == 0))                                     // Slot
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                            "GesnDataInterpret: MediaArrival event detected, device marked as active\n"));
//
//                 DeviceMarkActive(DeviceExtension, TRUE, FALSE);
//             }
//         }
//         else if (mediaInfo->MediaEvent == NOTIFICATION_MEDIA_EVENT_MEDIA_REMOVAL)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GESN::MEDIA REMOVAL, Status %x\n",
//                        mediaInfo->MediaStatus));
//
//             DeviceSetMediaChangeStateEx(DeviceExtension,
//                                         MediaNotPresent,
//                                         NULL);
//
//             // If media is removed from slot loading type, start powering off the device
//             // if it is ZPODD capable.
//             if ((DeviceExtension->ZeroPowerODDInfo != NULL) &&
//                 (DeviceExtension->ZeroPowerODDInfo->LoadingMechanism == LOADING_MECHANISM_CADDY) &&
//                 (DeviceExtension->ZeroPowerODDInfo->Load == 0))                                    // Slot
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                            "GesnDataInterpret: MediaRemoval event detected, device marked as idle\n"));
//
//                 DeviceMarkActive(DeviceExtension, FALSE, FALSE);
//             }
//         }
//         else if (mediaInfo->MediaEvent == NOTIFICATION_MEDIA_EVENT_EJECT_REQUEST)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GESN::MEDIA EJECTION, Status %x\n",
//                        mediaInfo->MediaStatus));
//
//             DeviceSendNotification(DeviceExtension,
//                                    &GUID_IO_MEDIA_EJECT_REQUEST,
//                                    0,
//                                    NULL);
//         }
//
//         break;
//     }
//
//     case NOTIFICATION_DEVICE_BUSY_CLASS_EVENTS:  // lowest priority events...
//     {
//         PNOTIFICATION_BUSY_STATUS busyInfo =
//                                     (PNOTIFICATION_BUSY_STATUS)(Header->ClassEventData);
//         DEVICE_EVENT_BECOMING_READY busyData = {0};
//
//         // else we want to report the approximated time till it's ready.
//         busyData.Version = 1;
//         busyData.Reason = busyInfo->DeviceBusyStatus;
//         busyData.Estimated100msToReady = (busyInfo->Time[0] << 8) |
//                                          (busyInfo->Time[1] & 0xff);
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "GESN::BUSY: Event: %x Status %x Time %x\n",
//                    busyInfo->DeviceBusyEvent, busyInfo->DeviceBusyStatus,
//                    busyData.Estimated100msToReady
//                    ));
//
//         // Ignore the notification if the time is small
//         if (busyData.Estimated100msToReady >= GESN_DEVICE_BUSY_LOWER_THRESHOLD_100_MS)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GesnDataInterpret: media BECOMING_READY\n"));
//
//             DeviceSendNotification(DeviceExtension,
//                                    &GUID_IO_DEVICE_BECOMING_READY,
//                                    sizeof(DEVICE_EVENT_BECOMING_READY),
//                                    &busyData);
//         }
//
//         // If manual loading operation is observed for slot loading type, start powering off the device
//         // if it is ZPODD capable.
//         if ((DeviceExtension->ZeroPowerODDInfo != NULL) &&
//             (DeviceExtension->ZeroPowerODDInfo->LoadingMechanism == LOADING_MECHANISM_TRAY) &&
//             (DeviceExtension->ZeroPowerODDInfo->Load == 0) &&                                   // Drawer
//             (busyInfo->DeviceBusyEvent == NOTIFICATION_BUSY_EVENT_LO_CHANGE) &&
//             (busyInfo->DeviceBusyStatus == NOTIFICATION_BUSY_STATUS_NO_EVENT))
//         {
//             inHomePosition = DeviceZPODDIsInHomePosition(DeviceExtension);
//
//             if (inHomePosition == FALSE)
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                            "GesnDataInterpret: LoChange event detected, device marked as active\n"));
//
//                 DeviceMarkActive(DeviceExtension, TRUE, FALSE);
//             }
//             else
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                            "GesnDataInterpret: LoChange event detected, device marked as idle\n"));
//
//                 DeviceMarkActive(DeviceExtension, FALSE, FALSE);
//             }
//         }
//
//         break;
//     }
//
//     default:
//     {
//         break;
//     }
//
//     } // end switch on notification class
//
//     return status;
// }
//
//
// VOID
// DeviceInternalSetMediaChangeState(
//             PCDROM_DEVICE_EXTENSION       DeviceExtension,
//             MEDIA_CHANGE_DETECTION_STATE  NewState,
//      PMEDIA_CHANGE_DETECTION_STATE OldState
//     )
// /*++
//
// Routine Description:
//
//     This routine will (if appropriate) set the media change event for the
//     device.  The event will be set if the media state is changed and
//     media change events are enabled.  Otherwise the media state will be
//     tracked but the event will not be set.
//
//     This routine will lock out the other media change routines if possible
//     but if not a media change notification may be lost after the enable has
//     been completed.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
//     NewState - new state for setting
//
//     OldState - optional storage for the old state
//
// Return Value:
//
//     none
//
// --*/
// {
// #if DBG
//     LPCSTR states[] = {"Unknown", "Present", "Not Present", "Unavailable"};
// #endif
//     MEDIA_CHANGE_DETECTION_STATE oldMediaState;
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//     CLASS_MEDIA_CHANGE_CONTEXT   mcnContext;
//
//     if (!((NewState >= MediaUnknown) && (NewState <= MediaUnavailable)))
//     {
//         return;
//     }
//
//     if (info == NULL)
//     {
//         return;
//     }
//
//     oldMediaState = info->LastKnownMediaDetectionState;
//     if (OldState)
//     {
//         *OldState = oldMediaState;
//     }
//
//     info->LastKnownMediaDetectionState = NewState;
//
//     // Increment MediaChangeCount on transition to MediaPresent
//     if (NewState == MediaPresent && oldMediaState != NewState)
//     {
//         InterlockedIncrement((PLONG)&DeviceExtension->MediaChangeCount);
//     }
//
//     if (info->MediaChangeDetectionDisableCount != 0)
//     {
// #if DBG
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceInternalSetMediaChangeState: MCN not enabled, state "
//                     "changed from %s to %s\n",
//                     states[oldMediaState], states[NewState]));
// #endif
//         return;
//     }
// #if DBG
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                 "DeviceInternalSetMediaChangeState: State change from %s to %s\n",
//                 states[oldMediaState], states[NewState]));
// #endif
//
//     if (info->LastReportedMediaDetectionState == info->LastKnownMediaDetectionState)
//     {
//         // Media is in the same state as we reported last time, no need to report again.
//         return;
//     }
//
//     // make the data useful -- it used to always be zero.
//     mcnContext.MediaChangeCount = DeviceExtension->MediaChangeCount;
//     mcnContext.NewState = NewState;
//
//     if (NewState == MediaPresent)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceInternalSetMediaChangeState: Reporting media ARRIVAL\n"));
//
//         DeviceSendNotification(DeviceExtension,
//                                &GUID_IO_MEDIA_ARRIVAL,
//                                sizeof(CLASS_MEDIA_CHANGE_CONTEXT),
//                                &mcnContext);
//     }
//     else if ((NewState == MediaNotPresent) || (NewState == MediaUnavailable))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceInternalSetMediaChangeState: Reporting media REMOVAL\n"));
//         DeviceSendNotification(DeviceExtension,
//                                &GUID_IO_MEDIA_REMOVAL,
//                                sizeof(CLASS_MEDIA_CHANGE_CONTEXT),
//                                &mcnContext);
//     }
//     else
//     {
//         // Don't notify of changed going to unknown.
//         return;
//     }
//
//     info->LastReportedMediaDetectionState = info->LastKnownMediaDetectionState;
//
//     return;
// } // end DeviceInternalSetMediaChangeState()
//
//
// VOID
// DeviceSetMediaChangeStateEx(
//             PCDROM_DEVICE_EXTENSION       DeviceExtension,
//             MEDIA_CHANGE_DETECTION_STATE  NewState,
//      PMEDIA_CHANGE_DETECTION_STATE OldState
//     )
// /*++
//
// Routine Description:
//
//     This routine will (if appropriate) set the media change event for the
//     device.  The event will be set if the media state is changed and
//     media change events are enabled.  Otherwise the media state will be
//     tracked but the event will not be set.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
//     NewState - new state for setting
//
//     OldState - optional storage for the old state
//
// Return Value:
//
//     none
//
// --*/
// {
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//     LARGE_INTEGER                zero;
//     NTSTATUS                     status;
//
//     // timeout value must be 0, as this function can be called at DISPATCH_LEVEL.
//     zero.QuadPart = 0;
//
//     if (info == NULL)
//     {
//         return;
//     }
//
//     status = KeWaitForMutexObject(&info->MediaChangeMutex,
//                                 Executive,
//                                 KernelMode,
//                                 FALSE,
//                                 &zero);
//
//     if (status == STATUS_TIMEOUT)
//     {
//         // Someone else is in the process of setting the media state.
//         return;
//     }
//
//     // Change the media present state and signal an event, if applicable
//     DeviceInternalSetMediaChangeState(DeviceExtension, NewState, OldState);
//
//     KeReleaseMutex(&info->MediaChangeMutex, FALSE);
//
//     return;
// } // end DeviceSetMediaChangeStateEx()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceSendDelayedMediaChangeNotifications(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine sends the so-called delayed media change notifications.
//     These notifications get accumulated while the MCN mechanism is disabled
//     and need to be sent to the application on MCN enabling, if MCN enabling
//     happens as a part of exclusive access unlock and the application has not
//     requested us explicitly to not send the delayed notifications.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
// Return Value:
//
//     none
//
// --*/
// {
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//     LARGE_INTEGER                zero;
//     NTSTATUS                     status;
//
//     PAGED_CODE();
//
//     zero.QuadPart = 0;
//
//     if (info == NULL)
//     {
//         return;
//     }
//
//     status = KeWaitForMutexObject(&info->MediaChangeMutex,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   &zero);
//
//     if (status == STATUS_TIMEOUT)
//     {
//         // Someone else is in the process of setting the media state.
//         // That's totally okay, we'll send delayed notifications later.
//         return;
//     }
//
//     // If the last reported state and the last known state are different and
//     // MCN is enabled, generate a notification based on the last known state.
//     if ((info->LastKnownMediaDetectionState != info->LastReportedMediaDetectionState) &&
//         (info->MediaChangeDetectionDisableCount == 0))
//     {
//         DeviceInternalSetMediaChangeState(DeviceExtension, info->LastKnownMediaDetectionState, NULL);
//     }
//
//     KeReleaseMutex(&info->MediaChangeMutex, FALSE);
//
//     return;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestSetupMcnRequest(
//      PCDROM_DEVICE_EXTENSION      DeviceExtension,
//      BOOLEAN                      UseGesn
// )
// /*++
//
// Routine Description:
//
//     This routine sets up the fields of the request for MCN
//
// Arguments:
//     DeviceExtension - device context
//
//     UseGesn - If TRUE, the device supports GESN and it's currently the mechanism for MCN
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                     status = STATUS_SUCCESS;
//     PSCSI_REQUEST_BLOCK          srb;
//     PIRP                         irp;
//     PIO_STACK_LOCATION           nextIrpStack;
//     PCDB                         cdb;
//     PVOID                        buffer;
//     WDF_REQUEST_REUSE_PARAMS     params;
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//
//     PAGED_CODE();
//
//     irp = WdfRequestWdmGetIrp(info->MediaChangeRequest);
//     NT_ASSERT(irp != NULL);
//
//     // deassign the MdlAddress, this is the value we assign explicitly.
//     // this is to prevent WdfRequestReuse to release the Mdl unexpectly.
//     if (irp->MdlAddress)
//     {
//         irp->MdlAddress = NULL;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Setup the IRP to perform a test unit ready.
//         WDF_REQUEST_REUSE_PARAMS_INIT(&params,
//                                       WDF_REQUEST_REUSE_NO_FLAGS,
//                                       STATUS_NOT_SUPPORTED);
//
//         status = WdfRequestReuse(info->MediaChangeRequest, &params);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Format the request.
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(DeviceExtension->IoTarget,
//                                                                 info->MediaChangeRequest,
//                                                                 IOCTL_SCSI_EXECUTE_IN,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL);
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL,
//                        "RequestSetupMcnRequest: WdfIoTargetFormatRequestForInternalIoctlOthers failed, %!STATUS!\n",
//                        status));
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         RequestClearSendTime(info->MediaChangeRequest);
//
//         nextIrpStack = IoGetNextIrpStackLocation(irp);
//
//         nextIrpStack->Flags = SL_OVERRIDE_VERIFY_VOLUME;
//         nextIrpStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//         nextIrpStack->Parameters.Scsi.Srb = &(info->MediaChangeSrb);
//
//         // Prepare the SRB for execution.
//         srb    = nextIrpStack->Parameters.Scsi.Srb;
//         buffer = info->SenseBuffer;
//         RtlZeroMemory(srb, sizeof(SCSI_REQUEST_BLOCK));
//         RtlZeroMemory(buffer, SENSE_BUFFER_SIZE);
//
//         srb->QueueTag        = SP_UNTAGGED;
//         srb->QueueAction     = SRB_SIMPLE_TAG_REQUEST;
//         srb->Length          = sizeof(SCSI_REQUEST_BLOCK);
//         srb->Function        = SRB_FUNCTION_EXECUTE_SCSI;
//         srb->SenseInfoBuffer = buffer;
//         srb->SrbStatus       = 0;
//         srb->ScsiStatus      = 0;
//         srb->OriginalRequest = irp;
//         srb->SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//
//         srb->SrbFlags        = DeviceExtension->SrbFlags;
//         SET_FLAG(srb->SrbFlags, info->SrbFlags);
//
//         if (!UseGesn)
//         {
//             srb->TimeOutValue = CDROM_TEST_UNIT_READY_TIMEOUT;
//             srb->CdbLength = 6;
//             srb->DataTransferLength = 0;
//             SET_FLAG(srb->SrbFlags, SRB_FLAGS_NO_DATA_TRANSFER);
//             nextIrpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_NONE;
//             srb->DataBuffer = NULL;
//             srb->DataTransferLength = 0;
//             irp->MdlAddress = NULL;
//
//             cdb = (PCDB) &srb->Cdb[0];
//             cdb->CDB6GENERIC.OperationCode = SCSIOP_TEST_UNIT_READY;
//         }
//         else
//         {
//             NT_ASSERT(info->Gesn.Buffer);
//
//             srb->TimeOutValue = GESN_TIMEOUT_VALUE; // much shorter timeout for GESN
//
//             srb->CdbLength = 10;
//             SET_FLAG(srb->SrbFlags, SRB_FLAGS_DATA_IN);
//             nextIrpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_IN;
//             srb->DataBuffer = info->Gesn.Buffer;
//             srb->DataTransferLength = info->Gesn.BufferSize;
//             irp->MdlAddress = info->Gesn.Mdl;
//
//             cdb = (PCDB) &srb->Cdb[0];
//             cdb->GET_EVENT_STATUS_NOTIFICATION.OperationCode = SCSIOP_GET_EVENT_STATUS;
//             cdb->GET_EVENT_STATUS_NOTIFICATION.Immediate = 1;
//             cdb->GET_EVENT_STATUS_NOTIFICATION.EventListLength[0] = (UCHAR)((info->Gesn.BufferSize) >> 8);
//             cdb->GET_EVENT_STATUS_NOTIFICATION.EventListLength[1] = (UCHAR)((info->Gesn.BufferSize) & 0xff);
//             cdb->GET_EVENT_STATUS_NOTIFICATION.NotificationClassRequest = info->Gesn.EventMask;
//         }
//     }
//
//     return status;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceDisableGesn(
//      WDFWORKITEM  WorkItem
//     )
// /*++
//
// Routine Description:
//
//     Work item routine to set the hack flag in the registry to disable GESN
//     This routine is invoked when the device reports TOO many events that affects system
//
// Arguments:
//     WorkItem - the work item be perfromed.
//
// Return Value:
//     None
//
// --*/
// {
//     WDFDEVICE               device = WdfWorkItemGetParentObject(WorkItem);
//     PCDROM_DEVICE_EXTENSION deviceExtension = DeviceGetExtension(device);
//
//     PAGED_CODE();
//
//     //
//     // Set the hack flag in the registry
//     //
//     DeviceSetParameter(deviceExtension,
//                        CLASSP_REG_SUBKEY_NAME,
//                        CLASSP_REG_MMC_DETECTION_VALUE_NAME,
//                        CdromDetectionUnsupported);
//
//     WdfObjectDelete(WorkItem);
//
//     return;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// BOOLEAN
// RequestPostWorkMcnRequest(
//      PCDROM_DEVICE_EXTENSION      DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine handles the completion of the test unit ready irps used to
//     determine if the media has changed.  If the media has changed, this code
//     signals the named event to wake up other system services that react to
//     media change (aka AutoPlay).
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//
//     BOOLEAN - TRUE (needs retry); FALSE (shoule not retry)
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PMEDIA_CHANGE_DETECTION_INFO    info = DeviceExtension->MediaChangeDetectionInfo;
//     PIRP                            irp;
//     BOOLEAN                         retryImmediately = FALSE;
//
//     PAGED_CODE();
//
//     NT_ASSERT(info->MediaChangeRequest != NULL);
//     irp = WdfRequestWdmGetIrp(info->MediaChangeRequest);
//
//     NT_ASSERT(!TEST_FLAG(info->MediaChangeSrb.SrbStatus, SRB_STATUS_QUEUE_FROZEN));
//
//     // use InterpretSenseInfo routine to check for media state, and also
//     // to call ClassError() with correct parameters.
//     if (SRB_STATUS(info->MediaChangeSrb.SrbStatus) != SRB_STATUS_SUCCESS)
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN, "MCN request - failed - srb status=%x, sense=%x/%x/%x.\n",
//                    info->MediaChangeSrb.SrbStatus,
//                    ((PSENSE_DATA)(info->MediaChangeSrb.SenseInfoBuffer))->SenseKey,
//                    ((PSENSE_DATA)(info->MediaChangeSrb.SenseInfoBuffer))->AdditionalSenseCode,
//                    ((PSENSE_DATA)(info->MediaChangeSrb.SenseInfoBuffer))->AdditionalSenseCodeQualifier));
//
//         if (SRB_STATUS(info->MediaChangeSrb.SrbStatus) != SRB_STATUS_NOT_POWERED)
//         {
//             // Release the queue if it is frozen.
//             if (info->MediaChangeSrb.SrbStatus & SRB_STATUS_QUEUE_FROZEN)
//             {
//                 DeviceReleaseQueue(DeviceExtension->Device);
//             }
//
//             RequestSenseInfoInterpret(DeviceExtension,
//                                       info->MediaChangeRequest,
//                                       &info->MediaChangeSrb,
//                                       0,
//                                       &status,
//                                       NULL);
//         }
//     }
//     else
//     {
//         DeviceExtension->PrivateFdoData->LoggedTURFailureSinceLastIO = FALSE;
//
//         if (!info->Gesn.Supported)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "MCN request - succeeded (GESN NOT supported, setting MediaPresent).\n"));
//
//             // success != media for GESN case
//             DeviceSetMediaChangeStateEx(DeviceExtension,
//                                         MediaPresent,
//                                         NULL);
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                        "MCN request - succeeded (GESN supported).\n"));
//         }
//     }
//
//     if (info->Gesn.Supported)
//     {
//         if (status == STATUS_DATA_OVERRUN)
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "MCN Request - Data Overrun\n"));
//             status = STATUS_SUCCESS;
//         }
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "MCN Request: GESN failed with status %x\n", status));
//         }
//         else
//         {
//             // for GESN, need to interpret the results of the data.
//             // this may also require an immediate retry
//             if (irp->IoStatus.Information == 8 )
//             {
//                 GesnDataInterpret(DeviceExtension,
//                                   (PVOID)info->Gesn.Buffer,
//                                   &retryImmediately);
//             }
//
//         } // end of NT_SUCCESS(status)
//
//     } // end of Info->Gesn.Supported
//
//     // free port-allocated sense buffer, if any.
//     if (PORT_ALLOCATED_SENSE(DeviceExtension, &info->MediaChangeSrb))
//     {
//         FREE_PORT_ALLOCATED_SENSE_BUFFER(DeviceExtension, &info->MediaChangeSrb);
//     }
//
//     // Remember the IRP and SRB for use the next time.
//     NT_ASSERT(IoGetNextIrpStackLocation(irp));
//     IoGetNextIrpStackLocation(irp)->Parameters.Scsi.Srb = &info->MediaChangeSrb;
//
//     // run a sanity check to make sure we're not recursing continuously
//     if (retryImmediately)
//     {
//         info->MediaChangeRetryCount++;
//
//         if (info->MediaChangeRetryCount > MAXIMUM_IMMEDIATE_MCN_RETRIES)
//         {
//             // Disable GESN on this device.
//             // Create a work item to set the value in the registry
//             WDF_OBJECT_ATTRIBUTES   attributes;
//             WDF_WORKITEM_CONFIG     workitemConfig;
//             WDFWORKITEM             workItem;
//
//             WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
//             attributes.ParentObject = DeviceExtension->Device;
//
//             WDF_WORKITEM_CONFIG_INIT(&workitemConfig, DeviceDisableGesn);
//             workitemConfig.AutomaticSerialization = FALSE;
//
//             status = WdfWorkItemCreate(&workitemConfig,
//                                        &attributes,
//                                        &workItem);
//
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "MCN Request: Disabling GESN for WDFDEVICE %p\n", DeviceExtension->Device));
//
//             if (NT_SUCCESS(status))
//             {
//                 WdfWorkItemEnqueue(workItem);
//             }
//
//             info->Gesn.Supported  = FALSE;
//             info->Gesn.EventMask  = 0;
//             info->Gesn.BufferSize = 0;
//             info->MediaChangeRetryCount = 0;
//             retryImmediately = FALSE;
//             // should we log an error in event log?
//         }
//     }
//     else
//     {
//         info->MediaChangeRetryCount = 0;
//     }
//
//     return retryImmediately;
// }
//
//
// _IRQL_requires_max_(APC_LEVEL)
// BOOLEAN
// RequestSendMcnRequest(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine sends the formatted MCN request sychronizely to lower driver.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//     BOOLEAN - TRUE (requst successfully sent); FALSE (request failed to send)
//
// --*/
// {
//     BOOLEAN                      requestSent = FALSE;
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//
//     PAGED_CODE();
//
//     RequestSend(DeviceExtension,
//                 info->MediaChangeRequest,
//                 DeviceExtension->IoTarget,
//                 WDF_REQUEST_SEND_OPTION_SYNCHRONOUS,
//                 &requestSent);
//
//     return requestSent;
// } // end RequestSendMcnRequest()
//
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeMcn(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      BOOLEAN                  AllowDriveToSleep
//     )
// /*++
//
// Routine Description:
//
//     This routine initialize the contents of MCN structure.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
//     AllowDriveToSleep - for CDROM, this parameter should be always FALSE
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                        status = STATUS_SUCCESS;
//     PMEDIA_CHANGE_DETECTION_INFO    mediaChangeInfo = NULL;
//     PIRP                            irp = NULL;
//     PVOID                           senseBuffer = NULL;
//     WDF_OBJECT_ATTRIBUTES           attributes;
//
//     PAGED_CODE();
//
//     if (DeviceExtension->MediaChangeDetectionInfo != NULL)
//     {
//         //Already initialized.
//         return STATUS_SUCCESS;
//     }
//
//     DeviceExtension->KernelModeMcnContext.FileObject      = (PVOID)-1;
//     DeviceExtension->KernelModeMcnContext.DeviceObject    = (PVOID)-1;
//     DeviceExtension->KernelModeMcnContext.LockCount       = 0;
//     DeviceExtension->KernelModeMcnContext.McnDisableCount = 0;
//
//     mediaChangeInfo = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                             sizeof(MEDIA_CHANGE_DETECTION_INFO),
//                                             CDROM_TAG_MEDIA_CHANGE_DETECTION);
//
//     if (mediaChangeInfo == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         RtlZeroMemory(mediaChangeInfo, sizeof(MEDIA_CHANGE_DETECTION_INFO));
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if ((DeviceExtension->PowerDescriptor != NULL) &&
//             (DeviceExtension->PowerDescriptor->AsynchronousNotificationSupported != FALSE) &&
//             (!TEST_FLAG(DeviceExtension->PrivateFdoData->HackFlags, FDO_HACK_NO_ASYNCHRONOUS_NOTIFICATION)))
//         {
//             mediaChangeInfo->AsynchronousNotificationSupported = TRUE;
//         }
//     }
//
//     //  Allocate an IRP to carry the IOCTL_MCN_SYNC_FAKE_IOCTL.
//     if (NT_SUCCESS(status))
//     {
//         irp = IoAllocateIrp(DeviceExtension->DeviceObject->StackSize, FALSE);
//
//         if (irp == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes,
//                                                 CDROM_REQUEST_CONTEXT);
//         attributes.ParentObject = DeviceExtension->Device;
//         status = WdfRequestCreate(&attributes,
//                                   DeviceExtension->IoTarget,
//                                   &mediaChangeInfo->MediaChangeRequest);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Preformat the media change request. With this being done, we never need to worry about
//         // WdfIoTargetFormatRequestForInternalIoctlOthers ever failing later.
//         status = WdfIoTargetFormatRequestForInternalIoctlOthers(DeviceExtension->IoTarget,
//                                                                 mediaChangeInfo->MediaChangeRequest,
//                                                                 IOCTL_SCSI_EXECUTE_IN,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL,
//                                                                 NULL, NULL);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         senseBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                             SENSE_BUFFER_SIZE,
//                                             CDROM_TAG_MEDIA_CHANGE_DETECTION);
//         if (senseBuffer == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         mediaChangeInfo->MediaChangeSyncIrp = irp;
//         mediaChangeInfo->SenseBuffer = senseBuffer;
//
//         // Set default values for the media change notification
//         // configuration.
//         mediaChangeInfo->MediaChangeDetectionDisableCount = 0;
//
//         // Assume that there is initially no media in the device
//         // only notify upper layers if there is something there
//         mediaChangeInfo->LastKnownMediaDetectionState = MediaUnknown;
//         mediaChangeInfo->LastReportedMediaDetectionState = MediaUnknown;
//
//         // setup all extra flags we'll be setting for this irp
//         mediaChangeInfo->SrbFlags = 0;
//
//         SET_FLAG(mediaChangeInfo->SrbFlags, SRB_CLASS_FLAGS_LOW_PRIORITY);
//         SET_FLAG(mediaChangeInfo->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//         SET_FLAG(mediaChangeInfo->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//
//         if (AllowDriveToSleep)  //FALSE for CD/DVD devices
//         {
//             SET_FLAG(mediaChangeInfo->SrbFlags, SRB_FLAGS_NO_KEEP_AWAKE);
//         }
//
//         KeInitializeMutex(&mediaChangeInfo->MediaChangeMutex, 0x100);
//
//         // It is ok to support media change events on this device.
//         DeviceExtension->MediaChangeDetectionInfo = mediaChangeInfo;
//
//         // check the device supports GESN or not, initialize GESN structure if it supports.
//         {
//             // This is only valid for type5 devices.
//             NTSTATUS tempStatus = STATUS_SUCCESS;
//
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "DeviceInitializeMcn: Testing for GESN\n"));
//             tempStatus = DeviceInitializeGesn(DeviceExtension);
//
//             if (NT_SUCCESS(tempStatus))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                            "DeviceInitializeMcn: GESN available for %p\n",
//                            DeviceExtension->DeviceObject));
//                 NT_ASSERT(mediaChangeInfo->Gesn.Supported );
//                 NT_ASSERT(mediaChangeInfo->Gesn.Buffer     != NULL);
//                 NT_ASSERT(mediaChangeInfo->Gesn.BufferSize != 0);
//                 NT_ASSERT(mediaChangeInfo->Gesn.EventMask  != 0);
//             }
//             else
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                            "DeviceInitializeMcn: GESN *NOT* available for %p\n",
//                            DeviceExtension->DeviceObject));
//                 NT_ASSERT(!mediaChangeInfo->Gesn.Supported);
//                 NT_ASSERT(mediaChangeInfo->Gesn.Buffer == NULL);
//                 NT_ASSERT(mediaChangeInfo->Gesn.BufferSize == 0);
//                 NT_ASSERT(mediaChangeInfo->Gesn.EventMask  == 0);
//                 mediaChangeInfo->Gesn.Supported = FALSE; // just in case....
//             }
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         // Register for display state change on AOAC capable systems so we can put the
//         // device to low power state when not required.
//         if (mediaChangeInfo->DisplayStateCallbackHandle == NULL)
//         {
//             POWER_PLATFORM_INFORMATION PlatformInfo = {0};
//
//             status = ZwPowerInformation(PlatformInformation,
//                                         NULL,
//                                         0,
//                                         &PlatformInfo,
//                                         sizeof(PlatformInfo));
//
//             if (NT_SUCCESS(status) && PlatformInfo.AoAc)
//             {
//                 PoRegisterPowerSettingCallback(DeviceExtension->DeviceObject,
//                                                &GUID_CONSOLE_DISPLAY_STATE,
//                                                &DevicePowerSettingCallback,
//                                                DeviceExtension,
//                                                &mediaChangeInfo->DisplayStateCallbackHandle);
//             }
//
//             // Ignore any failures above.
//             status = STATUS_SUCCESS;
//         }
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         if (irp != NULL)
//         {
//             IoFreeIrp(irp);
//         }
//         FREE_POOL(senseBuffer);
//         FREE_POOL(mediaChangeInfo);
//     }
//
//     return status;
//
// } // end DeviceInitializeMcn()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeGesn(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine initialize the contents of GESN structure.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                            status = STATUS_SUCCESS;
//     PNOTIFICATION_EVENT_STATUS_HEADER   header = NULL;
//     CDROM_DETECTION_STATE               detectionState = CdromDetectionUnknown;
//     PSTORAGE_DEVICE_DESCRIPTOR          deviceDescriptor = DeviceExtension->DeviceDescriptor;
//     BOOLEAN                             retryImmediately = TRUE;
//     ULONG                               i = 0;
//     ULONG                               atapiResets = 0;
//     PMEDIA_CHANGE_DETECTION_INFO        info = DeviceExtension->MediaChangeDetectionInfo;
//
//     PAGED_CODE();
//
//     NT_ASSERT(info != NULL);
//
//     // read if we already know the abilities of the device
//     DeviceGetParameter(DeviceExtension,
//                        CLASSP_REG_SUBKEY_NAME,
//                        CLASSP_REG_MMC_DETECTION_VALUE_NAME,
//                        (PULONG)&detectionState);
//
//     if (detectionState == CdromDetectionUnsupported)
//     {
//         status = STATUS_NOT_SUPPORTED;
//     }
//
//     // check if the device has a hack flag saying never to try this.
//     if (NT_SUCCESS(status) &&
//         (TEST_FLAG(DeviceExtension->PrivateFdoData->HackFlags, FDO_HACK_GESN_IS_BAD)) )
//     {
//         DeviceSetParameter(DeviceExtension,
//                            CLASSP_REG_SUBKEY_NAME,
//                            CLASSP_REG_MMC_DETECTION_VALUE_NAME,
//                            CdromDetectionUnsupported);
//         status = STATUS_NOT_SUPPORTED;
//     }
//
//     // else go through the process since we allocate buffers and
//     // get all sorts of device settings.
//     if (NT_SUCCESS(status))
//     {
//         if (info->Gesn.Buffer == NULL)
//         {
//             info->Gesn.Buffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                                       GESN_BUFFER_SIZE,
//                                                       CDROM_TAG_GESN);
//         }
//
//         if (info->Gesn.Buffer == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (info->Gesn.Mdl != NULL)
//         {
//             IoFreeMdl(info->Gesn.Mdl);
//         }
//
//         info->Gesn.Mdl = IoAllocateMdl(info->Gesn.Buffer,
//                                        GESN_BUFFER_SIZE,
//                                        FALSE,
//                                        FALSE,
//                                        NULL);
//         if (info->Gesn.Mdl == NULL)
//         {
//             status = STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         MmBuildMdlForNonPagedPool(info->Gesn.Mdl);
//         info->Gesn.BufferSize = GESN_BUFFER_SIZE;
//         info->Gesn.EventMask = 0;
//
//         // all items are prepared to use GESN (except the event mask, so don't
//         // optimize this part out!).
//         //
//         // now see if it really works. we have to loop through this because
//         // many SAMSUNG (and one COMPAQ) drives timeout when requesting
//         // NOT_READY events, even when the IMMEDIATE bit is set. :(
//         //
//         // using a drive list is cumbersome, so this might fix the problem.
//         for (i = 0; (i < 16) && retryImmediately; i++)
//         {
//             status = RequestSetupMcnRequest(DeviceExtension, TRUE);
//
//             if (!NT_SUCCESS(status))
//             {
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                            "Setup Mcn request failed %x for WDFDEVICE %p\n",
//                            status, DeviceExtension->Device));
//                 break;
//             }
//
//             NT_ASSERT(TEST_FLAG(info->MediaChangeSrb.SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE));
//
//             status = DeviceSendRequestSynchronously(DeviceExtension->Device, info->MediaChangeRequest, TRUE);
//
//             if (SRB_STATUS(info->MediaChangeSrb.SrbStatus) != SRB_STATUS_SUCCESS)
//             {
//                 // Release the queue if it is frozen.
//                 if (info->MediaChangeSrb.SrbStatus & SRB_STATUS_QUEUE_FROZEN)
//                 {
//                     DeviceReleaseQueue(DeviceExtension->Device);
//                 }
//
//                 RequestSenseInfoInterpret(DeviceExtension,
//                                           info->MediaChangeRequest,
//                                           &(info->MediaChangeSrb),
//                                           0,
//                                           &status,
//                                           NULL);
//             }
//
//             if ((deviceDescriptor->BusType == BusTypeAtapi) &&
//                 (info->MediaChangeSrb.SrbStatus == SRB_STATUS_BUS_RESET))
//             {
//                 //
//                 // ATAPI unfortunately returns SRB_STATUS_BUS_RESET instead
//                 // of SRB_STATUS_TIMEOUT, so we cannot differentiate between
//                 // the two.  if we get this status four time consecutively,
//                 // stop trying this command.  it is too late to change ATAPI
//                 // at this point, so special-case this here. (07/10/2001)
//                 // NOTE: any value more than 4 may cause the device to be
//                 //       marked missing.
//                 //
//                 atapiResets++;
//                 if (atapiResets >= 4)
//                 {
//                     status = STATUS_IO_DEVICE_ERROR;
//                     break;
//                 }
//             }
//
//             if (status == STATUS_DATA_OVERRUN)
//             {
//                 status = STATUS_SUCCESS;
//             }
//
//             if ((status == STATUS_INVALID_DEVICE_REQUEST) ||
//                 (status == STATUS_TIMEOUT) ||
//                 (status == STATUS_IO_DEVICE_ERROR) ||
//                 (status == STATUS_IO_TIMEOUT))
//             {
//                 // with these error codes, we don't ever want to try this command
//                 // again on this device, since it reacts poorly.
//                 DeviceSetParameter( DeviceExtension,
//                                     CLASSP_REG_SUBKEY_NAME,
//                                     CLASSP_REG_MMC_DETECTION_VALUE_NAME,
//                                     CdromDetectionUnsupported);
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                            "GESN test failed %x for WDFDEVICE %p\n",
//                            status, DeviceExtension->Device));
//                 break;
//             }
//
//             if (!NT_SUCCESS(status))
//             {
//                 // this may be other errors that should not disable GESN
//                 // for all future start_device calls.
//                 TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                            "GESN test failed %x for WDFDEVICE %p\n",
//                            status, DeviceExtension->Device));
//                 break;
//             }
//             else if (i == 0)
//             {
//                 // the first time, the request was just retrieving a mask of
//                 // available bits.  use this to mask future requests.
//                 header = (PNOTIFICATION_EVENT_STATUS_HEADER)(info->Gesn.Buffer);
//
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                            "WDFDEVICE %p supports event mask %x\n",
//                            DeviceExtension->Device, header->SupportedEventClasses));
//
//                 if (TEST_FLAG(header->SupportedEventClasses,
//                               NOTIFICATION_MEDIA_STATUS_CLASS_MASK))
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                "GESN supports MCN\n"));
//                 }
//                 if (TEST_FLAG(header->SupportedEventClasses,
//                               NOTIFICATION_DEVICE_BUSY_CLASS_MASK))
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                "GESN supports DeviceBusy\n"));
//                 }
//                 if (TEST_FLAG(header->SupportedEventClasses,
//                               NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK))
//                 {
//                     if (TEST_FLAG(DeviceExtension->PrivateFdoData->HackFlags,
//                                   FDO_HACK_GESN_IGNORE_OPCHANGE))
//                     {
//                         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                     "GESN supports OpChange, but must ignore these events for compatibility\n"));
//                         CLEAR_FLAG(header->SupportedEventClasses,
//                                    NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK);
//                     }
//                     else
//                     {
//                         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                     "GESN supports OpChange\n"));
//                     }
//                 }
//                 info->Gesn.EventMask = header->SupportedEventClasses;
//
//                 //
//                 // realistically, we are only considering the following events:
//                 //    EXTERNAL REQUEST - this is being tested for play/stop/etc.
//                 //    MEDIA STATUS - autorun and ejection requests.
//                 //    DEVICE BUSY - to allow us to predict when media will be ready.
//                 // therefore, we should not bother querying for the other,
//                 // unknown events. clear all but the above flags.
//                 //
//                 info->Gesn.EventMask &= NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK |
//                                         NOTIFICATION_EXTERNAL_REQUEST_CLASS_MASK   |
//                                         NOTIFICATION_MEDIA_STATUS_CLASS_MASK       |
//                                         NOTIFICATION_DEVICE_BUSY_CLASS_MASK        ;
//
//
//                 //
//                 // HACKHACK - REF #0001
//                 // Some devices will *never* report an event if we've also requested
//                 // that it report lower-priority events.  this is due to a
//                 // misunderstanding in the specification wherein a "No Change" is
//                 // interpreted to be a real event.  what should occur is that the
//                 // device should ignore "No Change" events when multiple event types
//                 // are requested unless there are no other events waiting.  this
//                 // greatly reduces the number of requests that the host must send
//                 // to determine if an event has occurred. Since we must work on all
//                 // drives, default to enabling the hack until we find evidence of
//                 // proper firmware.
//                 //
//                 if (info->Gesn.EventMask == 0)
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                "GESN supported, but not mask we care about (%x) for FDO %p\n",
//                                header->SupportedEventClasses,
//                                DeviceExtension->DeviceObject));
//                     // NOTE: the status is still status_sucess.
//                     break;
//                 }
//                 else if (CountOfSetBitsUChar(info->Gesn.EventMask) == 1)
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                "GESN hack not required for FDO %p\n",
//                                DeviceExtension->DeviceObject));
//                 }
//                 else
//                 {
//                     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                                "GESN hack enabled for FDO %p\n",
//                                DeviceExtension->DeviceObject));
//                     info->Gesn.HackEventMask = 1;
//                 }
//             }
//             else
//             {
//                 // i > 0; not the first time looping through, so interpret the results.
//                 status = GesnDataInterpret(DeviceExtension,
//                                            (PVOID)info->Gesn.Buffer,
//                                            &retryImmediately);
//
//                 if (!NT_SUCCESS(status))
//                 {
//                     // This drive does not support GESN correctly
//                     DeviceSetParameter( DeviceExtension,
//                                         CLASSP_REG_SUBKEY_NAME,
//                                         CLASSP_REG_MMC_DETECTION_VALUE_NAME,
//                                         CdromDetectionUnsupported);
//                     break;
//                 }
//             }
//         } // end 'for' loop of GESN requests....
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         //
//         // we can only use this if it can be relied upon for media changes,
//         // since we are (by definition) no longer going to be polling via
//         // a TEST_UNIT_READY irp, and drives will not report UNIT ATTENTION
//         // for this command (although a filter driver, such as one for burning
//         // cd's, might still fake those errors).
//         //
//         // since we also rely upon NOT_READY events to change the cursor
//         // into a "wait" cursor; GESN is still more reliable than other
//         // methods, and includes eject button requests, so we'll use it
//         // without DEVICE_BUSY in Windows Vista.
//         //
//
//         if (TEST_FLAG(info->Gesn.EventMask, NOTIFICATION_MEDIA_STATUS_CLASS_MASK))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "Enabling GESN support for WDFDEVICE %p\n",
//                        DeviceExtension->Device));
//             info->Gesn.Supported = TRUE;
//
//             DeviceSetParameter( DeviceExtension,
//                                 CLASSP_REG_SUBKEY_NAME,
//                                 CLASSP_REG_MMC_DETECTION_VALUE_NAME,
//                                 CdromDetectionSupported);
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                        "GESN available but not enabled for WDFDEVICE %p\n",
//                        DeviceExtension->Device));
//             status = STATUS_NOT_SUPPORTED;
//         }
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "GESN support detection failed  for WDFDEVICE %p with status %08x\n",
//                    DeviceExtension->Device, status));
//
//         if (info->Gesn.Mdl)
//         {
//             PIRP irp = WdfRequestWdmGetIrp(info->MediaChangeRequest);
//
//             IoFreeMdl(info->Gesn.Mdl);
//             info->Gesn.Mdl = NULL;
//             irp->MdlAddress = NULL;
//         }
//
//         FREE_POOL(info->Gesn.Buffer);
//         info->Gesn.Supported  = FALSE;
//         info->Gesn.EventMask  = 0;
//         info->Gesn.BufferSize = 0;
//     }
//
//     return status;
// }
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeMediaChangeDetection(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine checks to see if it is safe to initialize MCN (the back end
//     to autorun) for a given device.  It will then check the device-type wide
//     key "Autorun" in the service key (for legacy reasons), and then look in
//     the device-specific key to potentially override that setting.
//
//     If MCN is to be enabled, all neccessary structures and memory are
//     allocated and initialized.
//
//     This routine MUST be called only from the DeviceInit...() .
//
// Arguments:
//
//     DeviceExtension - the device to initialize MCN for, if appropriate
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//
//     BOOLEAN     disabled = FALSE;
//     BOOLEAN     instanceOverride;
//
//     PAGED_CODE();
//
//     // NOTE: This assumes that DeviceInitializeMediaChangeDetection is always
//     //       called in the context of the DeviceInitDevicePhase2. If called
//     //       after then this check will have already been made and the
//     //       once a second timer will not have been enabled.
//
//     disabled = DeviceIsMediaChangeDisabledDueToHardwareLimitation(DeviceExtension);
//
//     if (disabled)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceInitializeMediaChangeDetection: Disabled due to hardware"
//                     "limitations for this device\n"));
//     }
//     else
//     {
//         // autorun should now be enabled by default for all media types.
//         disabled = DeviceIsMediaChangeDisabledForClass(DeviceExtension);
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceInitializeMediaChangeDetection: MCN is %s\n",
//                     (disabled ? "disabled" : "enabled")));
//
//         status = DeviceMediaChangeDeviceInstanceOverride(DeviceExtension,
//                                                          &instanceOverride);  // default value
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                         "DeviceInitializeMediaChangeDetection: Instance using default\n"));
//         }
//         else
//         {
//             TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                         "DeviceInitializeMediaChangeDetection: Instance override: %s MCN\n",
//                         (instanceOverride ? "Enabling" : "Disabling")));
//             disabled = !instanceOverride;
//         }
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceInitializeMediaChangeDetection: Instance MCN is %s\n",
//                     (disabled ? "disabled" : "enabled")));
//     }
//
//     if (!disabled)
//     {
//         // if the drive is not a CDROM, allow the drive to sleep
//         status = DeviceInitializeMcn(DeviceExtension, FALSE);
//     }
//
//     return status;
// } // end DeviceInitializeMediaChangeDetection()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceMediaChangeDeviceInstanceOverride(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension,
//      PBOOLEAN                Enabled
//     )
// /*++
//
// Routine Description:
//
//     The user can override the global setting to enable or disable Autorun on a
//     specific cdrom device via the control panel.  This routine checks and/or
//     sets this value.
//
// Arguments:
//
//     DeviceExtension - the device to set/get the value for
//
//     Enabled - TRUE (Autorun is enabled)
//               FALSE (Autorun is disabled)
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS        status = STATUS_UNSUCCESSFUL;
//     WDFKEY          deviceKey = NULL;
//     WDFKEY          subKey = NULL;
//
//     UNICODE_STRING  subkeyName;
//     UNICODE_STRING  enableMcnValueName;
//     UNICODE_STRING  disableMcnValueName;
//     ULONG           alwaysEnable = 0;
//     ULONG           alwaysDisable = 0;
//
//     PAGED_CODE();
//
//     status = WdfDeviceOpenRegistryKey(DeviceExtension->Device,
//                                       PLUGPLAY_REGKEY_DEVICE,
//                                       KEY_ALL_ACCESS,
//                                       WDF_NO_OBJECT_ATTRIBUTES,
//                                       &deviceKey);
//     if (!NT_SUCCESS(status))
//     {
//         // this can occur when a new device is added to the system
//         // this is due to cdrom.sys being an 'essential' driver
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeDeviceInstanceOverride: "
//                     "Could not open device registry key [%lx]\n", status));
//     }
//     else
//     {
//         RtlInitUnicodeString(&subkeyName, MCN_REG_SUBKEY_NAME);
//
//         status = WdfRegistryOpenKey(deviceKey,
//                                     &subkeyName,
//                                     KEY_READ,
//                                     WDF_NO_OBJECT_ATTRIBUTES,
//                                     &subKey);
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeDeviceInstanceOverride: "
//                     "subkey could not be created. %lx\n", status));
//     }
//     else
//     {
//         // Default to not changing autorun behavior, based upon setting
//         // registryValue to zero.
//         RtlInitUnicodeString(&enableMcnValueName, MCN_REG_AUTORUN_ENABLE_INSTANCE_NAME);
//         RtlInitUnicodeString(&disableMcnValueName, MCN_REG_AUTORUN_DISABLE_INSTANCE_NAME);
//
//         // Ignore failures on reading of subkeys
//         (VOID) WdfRegistryQueryULong(subKey,
//                                      &enableMcnValueName,
//                                      &alwaysEnable);
//         (VOID) WdfRegistryQueryULong(subKey,
//                                      &disableMcnValueName,
//                                      &alwaysDisable);
//     }
//
//     // set return value and cleanup
//
//     if (subKey != NULL)
//     {
//         WdfRegistryClose(subKey);
//     }
//
//     if (deviceKey != NULL)
//     {
//         WdfRegistryClose(deviceKey);
//     }
//
//     if (alwaysEnable && alwaysDisable)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeDeviceInstanceOverride: %s selected\n",
//                     "Both Enable and Disable set -- DISABLE"));
//         NT_ASSERT(NT_SUCCESS(status));
//         status = STATUS_SUCCESS;
//         *Enabled = FALSE;
//     }
//     else if (alwaysDisable)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeDeviceInstanceOverride: %s selected\n",
//                     "DISABLE"));
//         NT_ASSERT(NT_SUCCESS(status));
//         status = STATUS_SUCCESS;
//         *Enabled = FALSE;
//     }
//     else if (alwaysEnable)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeDeviceInstanceOverride: %s selected\n",
//                     "ENABLE"));
//         NT_ASSERT(NT_SUCCESS(status));
//         status = STATUS_SUCCESS;
//         *Enabled = TRUE;
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeDeviceInstanceOverride: %s selected\n",
//                     "DEFAULT"));
//         status = STATUS_UNSUCCESSFUL;
//     }
//
//     return status;
// } // end DeviceMediaChangeDeviceInstanceOverride()
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceMediaChangeRegistryCallBack(
//      PWSTR ValueName,
//      ULONG ValueType,
//      PVOID ValueData,
//      ULONG ValueLength,
//      PVOID Context,
//      PVOID EntryContext
//     )
// /*++
//
// Routine Description:
//
//     This callback for a registry SZ or MULTI_SZ is called once for each
//     SZ in the value.  It will attempt to match the data with the
//     UNICODE_STRING passed in as Context, and modify EntryContext if a
//     match is found.  Written for ClasspCheckRegistryForMediaChangeCompletion
//
// Arguments:
//
//     ValueName     - name of the key that was opened
//     ValueType     - type of data stored in the value (REG_SZ for this routine)
//     ValueData     - data in the registry, in this case a wide string
//     ValueLength   - length of the data including the terminating null
//     Context       - unicode string to compare against ValueData
//     EntryContext  - should be initialized to 0, will be set to 1 if match found
//
// Return Value:
//
//     STATUS_SUCCESS
//     EntryContext will be 1 if found
//
// --*/
// {
//     PULONG          valueFound;
//     PUNICODE_STRING deviceString;
//     PWSTR           keyValue;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER(ValueName);
//
//     if ((Context == NULL) || (EntryContext == NULL))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "DeviceMediaChangeRegistryCallBack: NULL context should never be passed to registry call-back!\n"));
//
//         return STATUS_SUCCESS;
//     }
//
//     // if we have already set the value to true, exit
//     valueFound = EntryContext;
//     if ((*valueFound) != 0)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceMediaChangeRegistryCallBack: already set to true\n"));
//         return STATUS_SUCCESS;
//     }
//
//     if (ValueLength == sizeof(WCHAR))
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "DeviceMediaChangeRegistryCallBack: NULL string should never be passed to registry call-back!\n"));
//         return STATUS_SUCCESS;
//     }
//
//     // if the data is not a terminated string, exit
//     if (ValueType != REG_SZ)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     deviceString = Context;
//     keyValue = ValueData;
//     ValueLength -= sizeof(WCHAR); // ignore the null character
//
//     // do not compare more memory than is in deviceString
//     if (ValueLength > deviceString->Length)
//     {
//         ValueLength = deviceString->Length;
//     }
//
//     if (keyValue == NULL)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     // if the strings match, disable autorun
//     if (RtlCompareMemory(deviceString->Buffer, keyValue, ValueLength) == ValueLength)
//     {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN, "DeviceMediaChangeRegistryCallBack: Match found\n"));
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN, "DeviceMediaChangeRegistryCallBack: DeviceString at %p\n",
//                     deviceString->Buffer));
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                     "DeviceMediaChangeRegistryCallBack: KeyValue at %p\n",
//                     keyValue));
//         (*valueFound) = TRUE;
//     }
//
//     return STATUS_SUCCESS;
// } // end DeviceMediaChangeRegistryCallBack()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// BOOLEAN
// DeviceIsMediaChangeDisabledDueToHardwareLimitation(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     The key AutoRunAlwaysDisable contains a MULTI_SZ of hardware IDs for
//     which to never enable MediaChangeNotification.
//
//     The user can override the global setting to enable or disable Autorun on a
//     specific cdrom device via the control panel.
//
//     NOTE: It's intended that not use WdfRegistryQueryMultiString in this funciton,
//           as it's much more complicated.(needs WDFCOLLECTION, WDFSTRING other than
//           UNICODE_STRING)
//
// Arguments:
//
//     FdoExtension -
//     RegistryPath - pointer to the unicode string inside
//                    ...\CurrentControlSet\Services\Cdrom
//
// Return Value:
//
//     TRUE - no autorun.
//     FALSE - Autorun may be enabled
//
// --*/
// {
//     NTSTATUS                    status;
//
//     PSTORAGE_DEVICE_DESCRIPTOR  deviceDescriptor = DeviceExtension->DeviceDescriptor;
//     WDFKEY                      wdfKey;
//     HANDLE                      serviceKey = NULL;
//     RTL_QUERY_REGISTRY_TABLE    parameters[2] = {0};
//
//     UNICODE_STRING              deviceUnicodeString = {0};
//     ANSI_STRING                 deviceString = {0};
//     ULONG                       mediaChangeNotificationDisabled = 0;
//
//     PAGED_CODE();
//
//     // open the service key.
//     status = WdfDriverOpenParametersRegistryKey(WdfGetDriver(),
//                                                 KEY_ALL_ACCESS,
//                                                 WDF_NO_OBJECT_ATTRIBUTES,
//                                                 &wdfKey);
//
//     if(!NT_SUCCESS(status))
//     {
//         // NT_ASSERT(FALSE); __REACTOS__ : allow to fail (for 1st stage setup)
//
//         // always take the safe path.  if we can't open the service key, disable autorun
//         return TRUE;
//     }
//
//     if(NT_SUCCESS(status))
//     {
//         // Determine if drive is in a list of those requiring
//         // autorun to be disabled.  this is stored in a REG_MULTI_SZ
//         // named AutoRunAlwaysDisable.  this is required as some autochangers
//         // must load the disc to reply to ChkVerify request, causing them
//         // to cycle discs continuously.
//
//         PWSTR   nullMultiSz;
//         PUCHAR  vendorId = NULL;
//         PUCHAR  productId = NULL;
//         PUCHAR  revisionId = NULL;
//         size_t  length;
//         size_t  offset;
//
//         deviceString.Buffer        = NULL;
//         deviceUnicodeString.Buffer = NULL;
//
//         serviceKey = WdfRegistryWdmGetHandle(wdfKey);
//
//         // there may be nothing to check against
//         if ((deviceDescriptor->VendorIdOffset == 0) &&
//             (deviceDescriptor->ProductIdOffset == 0))
//         {
//             // no valid data in device extension.
//             status = STATUS_INTERNAL_ERROR;
//         }
//
//         // build deviceString using VendorId, Model and Revision.
//         // this string will be used to checked if it's one of devices in registry disable list.
//         if (NT_SUCCESS(status))
//         {
//             length = 0;
//
//             if (deviceDescriptor->VendorIdOffset == 0)
//             {
//                 vendorId = NULL;
//             }
//             else
//             {
//                 vendorId = (PUCHAR) deviceDescriptor + deviceDescriptor->VendorIdOffset;
//                 length = strlen((LPCSTR)vendorId);
//             }
//
//             if ( deviceDescriptor->ProductIdOffset == 0 )
//             {
//                 productId = NULL;
//             }
//             else
//             {
//                 productId = (PUCHAR) deviceDescriptor + deviceDescriptor->ProductIdOffset;
//                 length += strlen((LPCSTR)productId);
//             }
//
//             if ( deviceDescriptor->ProductRevisionOffset == 0 )
//             {
//                 revisionId = NULL;
//             }
//             else
//             {
//                 revisionId = (PUCHAR) deviceDescriptor + deviceDescriptor->ProductRevisionOffset;
//                 length += strlen((LPCSTR)revisionId);
//             }
//
//             // allocate a buffer for the string
//             deviceString.Length = (USHORT)( length );
//             deviceString.MaximumLength = deviceString.Length + 1;
//             deviceString.Buffer = (PCHAR)ExAllocatePoolWithTag( NonPagedPoolNx,
//                                                                  deviceString.MaximumLength,
//                                                                  CDROM_TAG_AUTORUN_DISABLE
//                                                                  );
//             if (deviceString.Buffer == NULL)
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                             "DeviceIsMediaChangeDisabledDueToHardwareLimitation: Unable to alloc string buffer\n" ));
//                 status = STATUS_INTERNAL_ERROR;
//             }
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             // copy strings to the buffer
//             offset = 0;
//
//             if (vendorId != NULL)
//             {
//                 RtlCopyMemory(deviceString.Buffer + offset,
//                               vendorId,
//                               strlen((LPCSTR)vendorId));
//                 offset += strlen((LPCSTR)vendorId);
//             }
//
//             if ( productId != NULL )
//             {
//                 RtlCopyMemory(deviceString.Buffer + offset,
//                               productId,
//                               strlen((LPCSTR)productId));
//                 offset += strlen((LPCSTR)productId);
//             }
//             if ( revisionId != NULL )
//             {
//                 RtlCopyMemory(deviceString.Buffer + offset,
//                               revisionId,
//                               strlen((LPCSTR)revisionId));
//                 offset += strlen((LPCSTR)revisionId);
//             }
//
//             NT_ASSERT(offset == deviceString.Length);
//
//             #pragma warning(suppress:6386) // Not an issue as deviceString.Buffer is of size deviceString.MaximumLength, which is equal to (deviceString.Length + 1)
//             deviceString.Buffer[deviceString.Length] = '\0';  // Null-terminated
//
//             // convert to unicode as registry deals with unicode strings
//             status = RtlAnsiStringToUnicodeString( &deviceUnicodeString,
//                                                    &deviceString,
//                                                    TRUE
//                                                    );
//             if (!NT_SUCCESS(status))
//             {
//                 TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                             "DeviceIsMediaChangeDisabledDueToHardwareLimitation: cannot convert "
//                             "to unicode %lx\n", status));
//             }
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             // query the value, setting valueFound to true if found
//             nullMultiSz = L"\0";
//             parameters[0].QueryRoutine  = DeviceMediaChangeRegistryCallBack;
//             parameters[0].Flags         = RTL_QUERY_REGISTRY_REQUIRED;
//             parameters[0].Name          = L"AutoRunAlwaysDisable";
//             parameters[0].EntryContext  = &mediaChangeNotificationDisabled;
//             parameters[0].DefaultType   = REG_MULTI_SZ;
//             parameters[0].DefaultData   = nullMultiSz;
//             parameters[0].DefaultLength = 0;
//
//             status = RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                                             serviceKey,
//                                             parameters,
//                                             &deviceUnicodeString,
//                                             NULL);
//             UNREFERENCED_PARAMETER(status); //defensive coding, avoid PREFAST warning.
//         }
//     }
//
//     // Cleanup
//     {
//
//         FREE_POOL( deviceString.Buffer );
//         if (deviceUnicodeString.Buffer != NULL)
//         {
//             RtlFreeUnicodeString( &deviceUnicodeString );
//         }
//
//         // handle serviceKey will be closed by framework while it closes registry key.
//         WdfRegistryClose(wdfKey);
//     }
//
//     if (mediaChangeNotificationDisabled > 0)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceIsMediaChangeDisabledDueToHardwareLimitation: Device is on MCN disable list\n"));
//     }
//
//     return (mediaChangeNotificationDisabled > 0);
//
// } // end DeviceIsMediaChangeDisabledDueToHardwareLimitation()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// BOOLEAN
// DeviceIsMediaChangeDisabledForClass(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     The user must specify that AutoPlay is to run on the platform
//     by setting the registry value HKEY_LOCAL_MACHINE\System\CurrentControlSet\
//     Services\<SERVICE>\Autorun:REG_DWORD:1.
//
//     The user can override the global setting to enable or disable Autorun on a
//     specific cdrom device via the control panel.
//
// Arguments:
//
//     DeviceExtension - device extension
//
// Return Value:
//
//     TRUE - Autorun is disabled for this class
//     FALSE - Autorun is enabled for this class
//
// --*/
// {
//     NTSTATUS                 status;
//     WDFKEY                   serviceKey = NULL;
//     WDFKEY                   parametersKey = NULL;
//
//     UNICODE_STRING           parameterKeyName;
//     UNICODE_STRING           valueName;
//
//     //  Default to ENABLING MediaChangeNotification (!)
//     ULONG                    mcnRegistryValue = 1;
//
//     PAGED_CODE();
//
//     // open the service key.
//     status = WdfDriverOpenParametersRegistryKey(WdfGetDriver(),
//                                                 KEY_ALL_ACCESS,
//                                                 WDF_NO_OBJECT_ATTRIBUTES,
//                                                 &serviceKey);
//     if(!NT_SUCCESS(status))
//     {
//         // return the default value, which is the inverse of the registry setting default
//         // since this routine asks if it's disabled
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceIsMediaChangeDisabledForClass: Defaulting to %s\n",
//                    (mcnRegistryValue ? "Enabled" : "Disabled")));
//         return (BOOLEAN)(mcnRegistryValue == 0);
//     }
//     else
//     {
//         // Open the parameters key (if any) beneath the services key.
//         RtlInitUnicodeString(&parameterKeyName, L"Parameters");
//
//         status = WdfRegistryOpenKey(serviceKey,
//                                     &parameterKeyName,
//                                     KEY_READ,
//                                     WDF_NO_OBJECT_ATTRIBUTES,
//                                     &parametersKey);
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         parametersKey = NULL;
//     }
//
//     RtlInitUnicodeString(&valueName, L"Autorun");
//     // ignore failures
//     status = WdfRegistryQueryULong(serviceKey,
//                                    &valueName,
//                                    &mcnRegistryValue);
//
//     if (NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                    "DeviceIsMediaChangeDisabledForClass: <Service>/Autorun flag = %d\n",
//                    mcnRegistryValue));
//     }
//
//     if (parametersKey != NULL)
//     {
//         status = WdfRegistryQueryULong(parametersKey,
//                                        &valueName,
//                                        &mcnRegistryValue);
//
//         if (NT_SUCCESS(status))
//         {
//             TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                        "DeviceIsMediaChangeDisabledForClass: <Service>/Parameters/Autorun flag = %d\n",
//                         mcnRegistryValue));
//         }
//
//         WdfRegistryClose(parametersKey);
//     }
//
//     WdfRegistryClose(serviceKey);
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                "DeviceIsMediaChangeDisabledForClass: Autoplay for device %p is %s\n",
//                DeviceExtension->DeviceObject,
//                (mcnRegistryValue ? "on" : "off")
//                 ));
//
//     // return if it is _disabled_, which is the
//     // inverse of the registry setting
//
//     return (BOOLEAN)(!mcnRegistryValue);
// } // end DeviceIsMediaChangeDisabledForClass()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceEnableMediaChangeDetection(
//         PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PFILE_OBJECT_CONTEXT    FileObjectContext,
//         BOOLEAN                 IgnorePreviousMediaChanges
//     )
// /*++
//
// Routine Description:
//
//     When the disable count decrease to 0, enable the MCN
//
// Arguments:
//
//     DeviceExtension - the device context
//
//     FileObjectContext - the file object context
//
//     IgnorePreviousMediaChanges - ignore all previous media changes
//
// Return Value:
//     None.
//
// --*/
// {
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//     LONG                         oldCount;
//
//     PAGED_CODE();
//
//     if (FileObjectContext)
//     {
//         InterlockedDecrement((PLONG)&(FileObjectContext->McnDisableCount));
//     }
//
//     if (info == NULL)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                     "DeviceEnableMediaChangeDetection: not initialized\n"));
//         return;
//     }
//
//     (VOID) KeWaitForMutexObject(&info->MediaChangeMutex,
//                                 UserRequest,
//                                 KernelMode,
//                                 FALSE,
//                                 NULL);
//
//     oldCount = --info->MediaChangeDetectionDisableCount;
//
//     NT_ASSERT(oldCount >= 0);
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                "DeviceEnableMediaChangeDetection: Disable count reduced to %d - \n",
//                info->MediaChangeDetectionDisableCount));
//
//     if (oldCount == 0)
//     {
//         if (IgnorePreviousMediaChanges)
//         {
//             info->LastReportedMediaDetectionState = info->LastKnownMediaDetectionState;
//         }
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN, "MCN is enabled\n"));
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN, "MCD still disabled\n"));
//     }
//
//     // Let something else run.
//     KeReleaseMutex(&info->MediaChangeMutex, FALSE);
//
//     return;
// } // end DeviceEnableMediaChangeDetection()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceDisableMediaChangeDetection(
//         PCDROM_DEVICE_EXTENSION DeviceExtension,
//      PFILE_OBJECT_CONTEXT    FileObjectContext
//     )
// /*++
//
// Routine Description:
//
//     Increase the disable count.
//
// Arguments:
//
//     DeviceExtension - the device context
//
//     FileObjectContext - the file object context
//
// Return Value:
//     None.
//
// --*/
// {
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//
//     PAGED_CODE();
//
//     if (FileObjectContext)
//     {
//         InterlockedIncrement((PLONG)&(FileObjectContext->McnDisableCount));
//     }
//
//     if (info == NULL)
//     {
//         return;
//     }
//
//     (VOID) KeWaitForMutexObject(&info->MediaChangeMutex,
//                                 UserRequest,
//                                 KernelMode,
//                                 FALSE,
//                                 NULL);
//
//     info->MediaChangeDetectionDisableCount++;
//
//     TracePrint!((TRACE_LEVEL_VERBOSE, TRACE_FLAG_MCN,
//                "DisableMediaChangeDetection: disable count is %d\n",
//                info->MediaChangeDetectionDisableCount));
//
//     KeReleaseMutex(&info->MediaChangeMutex, FALSE);
//
//     return;
// } // end DeviceDisableMediaChangeDetection()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceReleaseMcnResources(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine will cleanup any resources allocated for MCN.  It is called
//     by classpnp during remove device, and therefore is not typically required
//     by external drivers.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//     None.
//
// --*/
// {
//     PMEDIA_CHANGE_DETECTION_INFO info = DeviceExtension->MediaChangeDetectionInfo;
//
//     PAGED_CODE()
//
//     if(info == NULL)
//     {
//         return;
//     }
//
//     if (info->Gesn.Mdl)
//     {
//         PIRP irp = WdfRequestWdmGetIrp(info->MediaChangeRequest);
//         IoFreeMdl(info->Gesn.Mdl);
//         irp->MdlAddress = NULL;
//     }
//     IoFreeIrp(info->MediaChangeSyncIrp);
//     FREE_POOL(info->Gesn.Buffer);
//     FREE_POOL(info->SenseBuffer);
//
//     if (info->DisplayStateCallbackHandle)
//     {
//         PoUnregisterPowerSettingCallback(info->DisplayStateCallbackHandle);
//         info->DisplayStateCallbackHandle = NULL;
//     }
//
//     FREE_POOL(info);
//
//     DeviceExtension->MediaChangeDetectionInfo = NULL;
//
//     return;
// } // end DeviceReleaseMcnResources()
//
//
// IO_COMPLETION_ROUTINE RequestMcnSyncIrpCompletion;
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// RequestMcnSyncIrpCompletion(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//     () PVOID Context
//     )
// /*++
//
// Routine Description:
//
//     The MCN work finishes, reset the fields to allow another MCN request
//     be scheduled.
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
//     PCDROM_DEVICE_EXTENSION DeviceExtension = NULL;
//     PMEDIA_CHANGE_DETECTION_INFO info = NULL;
//
//     if (Context == NULL)
//     {
//         // this will never happen, but code must be there to prevent OACR warnings.
//         return STATUS_MORE_PROCESSING_REQUIRED;
//     }
//
//     DeviceExtension = (PCDROM_DEVICE_EXTENSION) Context;
//     info = DeviceExtension->MediaChangeDetectionInfo;
//
// #ifndef DEBUG
//     UNREFERENCED_PARAMETER(Irp);
// #endif
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     NT_ASSERT(Irp == info->MediaChangeSyncIrp);
//
//     IoReuseIrp(info->MediaChangeSyncIrp, STATUS_NOT_SUPPORTED);
//
//     // reset the value to let timer routine be able to send the next request.
//     InterlockedCompareExchange((PLONG)&(info->MediaChangeRequestInUse), 0, 1);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
//
// VOID
// RequestSetupMcnSyncIrp(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     setup the MCN synchronization irp.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//     None
//
// --*/
// {
//     PIRP                irp = NULL;
//     PIO_STACK_LOCATION  irpStack = NULL;
//     PIO_STACK_LOCATION  nextIrpStack = NULL;
//
//     irp = DeviceExtension->MediaChangeDetectionInfo->MediaChangeSyncIrp;
//     NT_ASSERT(irp != NULL);
//
//     //
//     //  For the driver that creates an IRP, there is no 'current' stack location.
//     //  Step down one IRP stack location so that the extra top one
//     //  becomes our 'current' one.
//     //
//     IoSetNextIrpStackLocation(irp);
//
//     /*
//      *  Cache our device object in the extra top IRP stack location
//      *  so we have it in our completion routine.
//      */
//     irpStack = IoGetCurrentIrpStackLocation(irp);
//     irpStack->DeviceObject = DeviceExtension->DeviceObject;
//
//     //
//     // If the irp is sent down when the volume needs to be
//     // verified, CdRomUpdateGeometryCompletion won't complete
//     // it since it's not associated with a thread.  Marking
//     // it to override the verify causes it always be sent
//     // to the port driver
//     //
//     nextIrpStack = IoGetNextIrpStackLocation(irp);
//
//     SET_FLAG(nextIrpStack->Flags, SL_OVERRIDE_VERIFY_VOLUME);
//
//     nextIrpStack->MajorFunction = IRP_MJ_DEVICE_CONTROL;
//     // pick up this IOCTL code as it's not normaly seen for CD/DVD drive and does not require input.
//     // set other fields to make this IOCTL recognizable by CDROM.SYS
//     nextIrpStack->Parameters.Others.Argument1 = RequestSetupMcnSyncIrp;
//     nextIrpStack->Parameters.Others.Argument2 = RequestSetupMcnSyncIrp;
//     nextIrpStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_MCN_SYNC_FAKE_IOCTL; //Argument3.
//     nextIrpStack->Parameters.Others.Argument4 = RequestSetupMcnSyncIrp;
//
//     IoSetCompletionRoutine(irp,
//                            RequestMcnSyncIrpCompletion,
//                            DeviceExtension,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     return;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DeviceMainTimerTickHandler(
//      WDFTIMER  Timer
//     )
// /*++
//
// Routine Description:
//
//     This routine setup a sync irp and send it to the serial queue.
//     Serial queue will process MCN when receive this sync irp.
//
// Arguments:
//
//     Timer - the timer object that fires.
//
// Return Value:
//     None
//
// --*/
// {
//     PCDROM_DEVICE_EXTENSION      deviceExtension = NULL;
//     size_t                       dataLength = 0;
//
//     deviceExtension = WdfObjectGetTypedContext(WdfTimerGetParentObject(Timer), CDROM_DEVICE_EXTENSION);
//
//     (void) RequestHandleEventNotification(deviceExtension, NULL, NULL, &dataLength);
//
//     return;
// } // end DeviceMainTimerTickHandler()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// DeviceEnableMainTimer(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine will allocate timer related resources on the first time call.
//     Start the timer.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS status = STATUS_SUCCESS;
//
//     if ((DeviceExtension->MediaChangeDetectionInfo == NULL) ||
//         (DeviceExtension->MediaChangeDetectionInfo->AsynchronousNotificationSupported != FALSE))
//     {
//         // Asynchronous Notification is enabled, timer not needed.
//         return status;
//     }
//
//     if (DeviceExtension->MainTimer == NULL)
//     {
//         //create main timer object.
//         WDF_TIMER_CONFIG        timerConfig;
//         WDF_OBJECT_ATTRIBUTES   timerAttributes;
//
//         WDF_TIMER_CONFIG_INIT(&timerConfig, DeviceMainTimerTickHandler);
//
//         // Polling frequently on virtual optical devices created by Hyper-V will
//         // cause a significant perf / power hit. These devices need to be polled
//         // less frequently for device state changes.
//         if (TEST_FLAG(DeviceExtension->DeviceAdditionalData.HackFlags, CDROM_HACK_MSFT_VIRTUAL_ODD))
//         {
//             timerConfig.Period = 2000; // 2 seconds, in milliseconds.
//         }
//         else
//         {
//             timerConfig.Period = 1000; // 1 second, in milliseconds.
//         }
//
//         timerConfig.TolerableDelay = 500; // 0.5 seconds, in milliseconds
//
//         //Set the autoSerialization to FALSE, as the parent device's
//         //execute level is WdfExecutionLevelPassive.
//         timerConfig.AutomaticSerialization = FALSE;
//
//         WDF_OBJECT_ATTRIBUTES_INIT(&timerAttributes);
//         timerAttributes.ParentObject = DeviceExtension->Device;
//         timerAttributes.ExecutionLevel = WdfExecutionLevelInheritFromParent;
//
//         status = WdfTimerCreate(&timerConfig,
//                                 &timerAttributes,
//                                 &DeviceExtension->MainTimer);
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         WdfTimerStart(DeviceExtension->MainTimer,WDF_REL_TIMEOUT_IN_MS(100));
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceEnableMainTimer: Once a second timer enabled  for WDFDEVICE %p\n",
//                    DeviceExtension->Device));
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceEnableMainTimer: WDFDEVICE %p, Status %lx  initializing timer\n",
//                    DeviceExtension->Device, status));
//     }
//
//     return status;
// } // end DeviceEnableMainTimer()
//
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// VOID
// DeviceDisableMainTimer(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     stop the timer.
//
// Arguments:
//
//     DeviceExtension - device context
//
// Return Value:
//     None
//
// --*/
// {
//     PAGED_CODE();
//
//     if ((DeviceExtension->MediaChangeDetectionInfo == NULL) ||
//         (DeviceExtension->MediaChangeDetectionInfo->AsynchronousNotificationSupported != FALSE))
//     {
//         // Asynchronous Notification is enabled, timer not needed.
//         return;
//     }
//
//     if (DeviceExtension->MainTimer != NULL)
//     {
//         //
//         // we are only going to stop the actual timer in remove device routine.
//         // it is the responsibility of the code within the timer routine to
//         // check if the device is removed and not processing io for the final
//         // call.
//         // this keeps the code clean and prevents lots of bugs.
//         //
//         WdfTimerStop(DeviceExtension->MainTimer,TRUE);
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_MCN,
//                    "DeviceDisableMainTimer: Once a second timer disabled for device %p\n",
//                    DeviceExtension->Device));
//     }
//     else
//     {
//         TracePrint!((TRACE_LEVEL_WARNING, TRACE_FLAG_MCN,
//                    "DeviceDisableMainTimer: Timer never enabled\n"));
//     }
//
//     return;
// } // end DeviceDisableMainTimer()
//
//
// _IRQL_requires_max_(APC_LEVEL)
// NTSTATUS
// RequestHandleMcnControl(
//       PCDROM_DEVICE_EXTENSION  DeviceExtension,
//       WDFREQUEST               Request,
//      size_t *                 DataLength
//     )
// /*++
//
// Routine Description:
//
//     This routine handles the process of IOCTL_STORAGE_MCN_CONTROL
//
// Arguments:
//
//     DeviceExtension - device context
//
//     Request - request object
//
//     RequestParameters - request parameters
//
//     DataLength - data transferred
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     WDFFILEOBJECT           fileObject = NULL;
//     PFILE_OBJECT_CONTEXT    fileObjectContext = NULL;
//     PPREVENT_MEDIA_REMOVAL  mediaRemoval = NULL;
//
//     PAGED_CODE();
//
//     *DataLength = 0;
//
//     status = WdfRequestRetrieveInputBuffer(Request,
//                                            sizeof(PREVENT_MEDIA_REMOVAL),
//                                            &mediaRemoval,
//                                            NULL);
//
//     if (NT_SUCCESS(status))
//     {
//         fileObject = WdfRequestGetFileObject(Request);
//
//         // Check to make sure we have a file object extension to keep track of this
//         // request.  If not we'll fail it before synchronizing.
//         if (fileObject != NULL)
//         {
//             fileObjectContext = FileObjectGetContext(fileObject);
//         }
//
//         if ((fileObjectContext == NULL) &&
//             (WdfRequestGetRequestorMode(Request) == KernelMode))
//         {
//             fileObjectContext = &DeviceExtension->KernelModeMcnContext;
//         }
//
//         if (fileObjectContext == NULL)
//         {
//             // This handle isn't setup correctly.  We can't let the
//             // operation go.
//             status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         if (mediaRemoval->PreventMediaRemoval)
//         {
//             // This is a lock command.  Reissue the command in case bus or
//             // device was reset and the lock was cleared.
//             DeviceDisableMediaChangeDetection(DeviceExtension, fileObjectContext);
//         }
//         else
//         {
//             if (fileObjectContext->McnDisableCount == 0)
//             {
//                 status = STATUS_INVALID_DEVICE_STATE;
//             }
//             else
//             {
//                 DeviceEnableMediaChangeDetection(DeviceExtension, fileObjectContext, TRUE);
//             }
//         }
//     }
//
//     return status;
// } // end RequestHandleMcnControl()
//
// #pragma warning(pop) // un-sets any local warning changes
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, _STORAGE_BUS_TYPE_BusTypeAtapi as BusTypeAtapi, CLASS_MEDIA_CHANGE_CONTEXT, _WORK_QUEUE_TYPE_DelayedWorkQueue as DelayedWorkQueue, EVENT_DESCRIPTOR, _KWAIT_REASON_Executive, GUID, PIO_COMPLETION_ROUTINE as IO_COMPLETION_ROUTINE, IO_ERROR_LOG_PACKET, _IO_STATUS_BLOCK, PIO_WORKITEM_ROUTINE as IO_WORKITEM_ROUTINE, IoAllocateErrorLogEntry, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoFreeIrp, IoFreeMdl, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoGetDriverObjectExtension, IoGetNextIrpStackLocation, IoOpenDeviceRegistryKey, IoQueueWorkItem, IoReportTargetDeviceChangeAsynchronous, IoReuseIrp, IoSetCompletionRoutine, IoSetNextIrpStackLocation, IoWriteErrorLogEntry, KEVENT, KeCancelTimer, KeEnterCriticalRegion, KeInitializeDpc, KeInitializeEvent, KeInitializeMutex, KeInitializeTimer, KeLeaveCriticalRegion, KeQueryTimeIncrement, KeReleaseMutex, KeWaitForSingleObject, LARGE_INTEGER, LPGUID, MmBuildMdlForNonPagedPool, _OBJECT_ATTRIBUTES, PCSZ, PDEVICE_OBJECT, PFILE_OBJECT, PFILE_OBJECT_EXTENSION, PIO_ERROR_LOG_PACKET, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PPREVENT_MEDIA_REMOVAL, PSECURITY_DESCRIPTOR, PSTORAGE_DEVICE_DESCRIPTOR, PTARGET_DEVICE_CUSTOM_NOTIFICATION, PUNICODE_STRING, _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0, PRTL_QUERY_REGISTRY_ROUTINE as RTL_QUERY_REGISTRY_ROUTINE, RTL_QUERY_REGISTRY_TABLE, RtlAnsiStringToUnicodeString, RtlCompareMemory, RtlFreeUnicodeString, RtlInitUnicodeString, RtlQueryRegistryValues, STORAGE_PREDICT_FAILURE, _EVENT_TYPE_SynchronizationEvent, TARGET_DEVICE_CUSTOM_NOTIFICATION, _KWAIT_REASON_UserRequest, ZwClose, ZwCreateKey, ZwOpenKey};
use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{FILE_DEVICE_DISK, IO_WRN_FAILURE_PREDICTED, IRP_MJ_SCSI, KEY_ALL_ACCESS, PLUGPLAY_REGKEY_DEVICE, RTL_QUERY_REGISTRY_TYPECHECK, SL_OVERRIDE_VERIFY_VOLUME, STATUS_DEVICE_PROTOCOL_ERROR, STATUS_INVALID_DEVICE_STATE, STATUS_IO_DEVICE_ERROR, STATUS_IO_TIMEOUT, STATUS_MEDIA_CHANGED, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING, VPB_MOUNTED};
use crate::types::{CLASS_SRB_STORAGE_REQUEST_BLOCK, ClassInitialize, ClassSignalCompletion, ClasspPowerSettingCallback, ClasspUpdateDiskProperties, DEFAULT_FAILURE_PREDICTION_PERIOD, FAILURE_PREDICTION_INFO, FAILURE_PREDICTION_METHOD, IOCTL_SCSI_EXECUTE_IN, IOCTL_SCSI_EXECUTE_NONE, MEDIA_CHANGE_DEFAULT_TIME, MEDIA_CHANGE_DETECTION_INFO, MEDIA_CHANGE_TIMEOUT_TIME, _MEDIA_CHANGE_DETECTION_STATE_MediaPresent as MediaPresent, NOTIFICATION_DEVICE_BUSY_CLASS_MASK, NOTIFICATION_EXTERNAL_EVENT_BUTTON_DOWN, NOTIFICATION_EXTERNAL_REQUEST_CLASS_MASK, NOTIFICATION_MEDIA_EVENT_EJECT_REQUEST, NOTIFICATION_MEDIA_EVENT_MEDIA_CHANGE, NOTIFICATION_MEDIA_EVENT_MEDIA_REMOVAL, NOTIFICATION_MEDIA_EVENT_NEW_MEDIA, NOTIFICATION_MEDIA_STATUS_CLASS_MASK, NOTIFICATION_NO_CLASS_EVENTS, NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK, NOTIFICATION_OPERATIONAL_EVENT_CHANGE_REQUESTED, NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_ADDED, NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_CHANGE, PCDB, PCLASS_DRIVER_EXTENSION, PCLASS_PRIVATE_FDO_DATA, PCOMMON_DEVICE_EXTENSION, PFAILURE_PREDICTION_INFO, PFUNCTIONAL_DEVICE_EXTENSION, PMEDIA_CHANGE_DETECTION_INFO, PNOTIFICATION_BUSY_STATUS, PNOTIFICATION_EVENT_STATUS_HEADER, PNOTIFICATION_EXTERNAL_STATUS, PNOTIFICATION_MEDIA_STATUS, PNOTIFICATION_OPERATIONAL_STATUS, PSCSI_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK_HEADER, REMOVE_COMPLETE, SCSIOP_GET_EVENT_STATUS, SCSIOP_TEST_UNIT_READY, SCSI_ADDRESS, SCSI_ADSENSE_LUN_NOT_READY, SCSI_ADSENSE_MEDIUM_CHANGED, SCSI_REQUEST_BLOCK, SCSI_SENSE_ERRORCODE_FIXED_CURRENT, SCSI_SENSE_NOT_READY, SCSI_SENSE_UNIT_ATTENTION, SENSE_DATA, SRB_FLAGS_DATA_IN, SRB_FLAGS_NO_DATA_TRANSFER, SRB_FLAGS_NO_KEEP_AWAKE, SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_STORAGE_REQUEST_BLOCK, SRB_SIMPLE_TAG_REQUEST, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_BUS_RESET, SRB_STATUS_ERROR, SRB_STATUS_QUEUE_FROZEN, SRB_STATUS_SUCCESS, SRB_TYPE_STORAGE_REQUEST_BLOCK, STORAGE_ADDRESS_TYPE_BTL8, _CLASS_DETECTION_STATE as CLASS_DETECTION_STATE, _CLASS_DETECTION_STATE_ClassDetectionSupported as ClassDetectionSupported, _CLASS_DETECTION_STATE_ClassDetectionUnknown as ClassDetectionUnknown, _CLASS_DETECTION_STATE_ClassDetectionUnsupported as ClassDetectionUnsupported, _CLASS_PRIVATE_FDO_DATA, _FAILURE_PREDICTION_INFO, _FUNCTIONAL_DEVICE_EXTENSION, _MEDIA_CHANGE_DETECTION_INFO, _MEDIA_CHANGE_DETECTION_STATE as MEDIA_CHANGE_DETECTION_STATE, _MEDIA_CHANGE_DETECTION_STATE_MediaNotPresent as MediaNotPresent, _MEDIA_CHANGE_DETECTION_STATE_MediaUnavailable as MediaUnavailable, _MEDIA_CHANGE_DETECTION_STATE_MediaUnknown as MediaUnknown, _NOTIFICATION_EVENT_STATUS_HEADER, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16};
use ntkrnl_sys::ddk::{ACCESS_MASK, CCHAR, DEVICE_OBJECT, FILE_OBJECT, HANDLE, KPROCESSOR_MODE, KTIMER, KWAIT_REASON, LPCGUID, PCCHAR, PCEVENT_DESCRIPTOR, PCSTR, PDRIVER_OBJECT, PRTL_QUERY_REGISTRY_TABLE, UINT, ULONG64, _FILE_OBJECT, _IO_WORKITEM, _IRP, _LARGE_INTEGER, _RTL_QUERY_REGISTRY_TABLE, _STORAGE_DEVICE_DESCRIPTOR, _TARGET_DEVICE_CUSTOM_NOTIFICATION, _UNICODE_STRING};
use ntkrnl_sys::mm::{NonPagedPool, NonPagedPoolCacheAligned};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{FILE_DEVICE_CD_ROM, RTL_QUERY_REGISTRY_TYPECHECK_SHIFT, SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED, STATUS_MORE_PROCESSING_REQUIRED, TRACE_LEVEL_VERBOSE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_AUTORUN_DISABLE, CLASS_TAG_FAILURE_PREDICT, CLASS_TAG_MEDIA_CHANGE_DETECTION};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_MCN, TRACE_FLAG_WMI};
use crate::glue::{NOTIFICATION_OPERATIONAL_CHANGE_CLASS_EVENTS, NOTIFICATION_EXTERNAL_REQUEST_CLASS_EVENTS,
    NOTIFICATION_MEDIA_STATUS_CLASS_EVENTS, NOTIFICATION_DEVICE_BUSY_CLASS_EVENTS};

// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx, NonPagedPoolNxCacheAligned, SP_UNTAGGED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE, FailurePredictionNone, IOCTL_SCSI_GET_ADDRESS, IOCTL_STORAGE_PREDICT_FAILURE, SENSE_BUFFER_SIZE_EX};
use ntkrnl_sys::ddk::{_GUID};
use crate::shims::*;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ClassGetDeviceParameter, ClassGetDriverExtension, ClassGetFsContext, ClassGetVpb, ClassSetDeviceParameter, ClassWmiFireEvent, ClasspPowerActivateDevice, ClasspPowerIdleDevice, InitializeStorageRequestBlock};
use ntkrnl_sys::ddk::{ExFreePool};
// resolve_imports.py: explicit kernel/private-type imports
use crate::guid::{GUID_CONSOLE_DISPLAY_STATE, GUID_IO_DEVICE_BECOMING_READY, GUID_IO_DEVICE_EXTERNAL_REQUEST, GUID_IO_MEDIA_ARRIVAL, GUID_IO_MEDIA_EJECT_REQUEST, GUID_IO_MEDIA_REMOVAL};
use crate::types::{ScreenStateNotificationHandle};
// resolve_imports.py: explicit kernel/private-type imports
use crate::srblib::{SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength, SrbSetScsiStatus};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{_PREDICT_FAILURE_METHOD_FailurePredictionNone};

extern "C" {
    static SharedSystemTime: LARGE_INTEGER;
}
extern "C" {
    fn TracePrint(Level: ULONG, Flags: ULONG, Format: PCCHAR, ...);
    fn StorDebugPrint(Level: ULONG, Flags: ULONG, Format: PCCHAR, ...);
    
    fn CountOfSetBitsUChar(val: UCHAR) -> UCHAR;
    fn RtlULongAdd(augend: ULONG, addend: ULONG, result: *mut ULONG) -> NTSTATUS;
    fn strlen(s: *const i8) -> usize;
    // ExTimer APIs
    fn ExSetTimer(Timer: PVOID, DueTime: i64, Period: i64, Parameters: PVOID) -> BOOLEAN;
    fn ExDeleteTimer(Timer: PVOID, Cancel: BOOLEAN, Wait: BOOLEAN, Parameters: PVOID) -> BOOLEAN;
    fn ExInitializeSetTimerParameters(Parameters: PVOID);
    fn ExInitializeDeleteTimerParameters(Parameters: PVOID);
    fn ExAllocateTimer(Callback: PVOID, CallbackContext: PVOID, Attributes: ULONG) -> PVOID;
    // Etw APIs
    fn EventDescCreate(EventDescriptor: PVOID, Id: u16, Version: u8, Channel: u8, Level: u8, Task: u16, Opcode: u8, Keyword: u64);
    // Debug string functions (debug-only no-ops in practice)
    fn DBGGETSRBSTATUSSTR(Srb: PVOID) -> PCCHAR;
    fn DBGGETSENSECODESTR(Srb: PVOID) -> PCCHAR;
    fn DBGGETADSENSECODESTR(Srb: PVOID) -> PCCHAR;
    fn DBGGETADSENSEQUALIFIERSTR(Srb: PVOID) -> PCCHAR;
    // ClasspMediaChangeRegistryCallBack
    #[link_name = "ClassAcquireRemoveLock"]
    fn ClassAcquireRemoveLockR(DeviceObject: PDEVICE_OBJECT, Tag: PIRP) -> ULONG;
    // ClasspMediaChangeRegistryCallBack is NTAPI (stdcall) — declared in a separate
    // extern "system" block below so the reference matches class_part_d's stdcall definition.
    fn PORT_ALLOCATED_SENSE_EX(
        fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
        Srb: PSTORAGE_REQUEST_BLOCK_HEADER,
    ) -> BOOLEAN;
    fn FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(
        fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
        Srb: PSTORAGE_REQUEST_BLOCK_HEADER,
    );
}
unsafe extern "system" {
    fn KeSetCoalescableTimer( Timer: *mut KTIMER, DueTime: LARGE_INTEGER, Period: ULONG, TolerableDelay: ULONG, Dpc: PKDPC, );
    fn PoRegisterPowerSettingCallback(
        DeviceObject: PDEVICE_OBJECT,
        SettingGuid: *const GUID,
        Callback: ::core::option::Option<unsafe extern "system" fn(
            SettingGuid: *const GUID,
            Value: PVOID,
            ValueLength: ULONG,
            Context: PVOID,
        ) -> NTSTATUS>,
        Context: PVOID,
        Handle: *mut PVOID,
    ) -> NTSTATUS;
    fn EtwWrite(RegHandle: u64, EventDescriptor: PCEVENT_DESCRIPTOR, ActivityId: *const GUID, UserDataCount: ULONG, UserData: PVOID);
}

// classpnp.h FILE_OBJECT_EXTENSION (different from ntkrnl-sys version)
#[repr(C)]
struct _FILE_OBJECT_EXTENSION {
    pub FileObject: PFILE_OBJECT,
    pub DeviceObject: PDEVICE_OBJECT,
    pub LockCount: ULONG,
    pub McnDisableCount: ULONG,
}

// ioevent.h structs not in ntkrnl-sys
#[repr(C)]
struct DEVICE_EVENT_BECOMING_READY {
    pub Version: ULONG,
    pub Reason: ULONG,
    pub Estimated100msToReady: ULONG,
}
#[repr(C)]
struct DEVICE_EVENT_EXTERNAL_REQUEST {
    pub Version: ULONG,
    pub DeviceClass: ULONG,
    pub ButtonStatus: USHORT,
    pub Request: USHORT,
    pub SystemTime: LARGE_INTEGER,
}

// EX_TIMER types (NTDDI_WINBLUE+, not in ntkrnl-sys bindgen)
type PEX_TIMER = PVOID;
const EX_TIMER_UNLIMITED_TOLERANCE: i64 = -1i64;
const EX_TIMER_NO_WAKE: ULONG = 0x00000008;
#[repr(C)]
struct EXT_SET_PARAMETERS {
    pub Version: ULONG,
    pub Reserved: ULONG,
    pub NoWakeTolerance: i64,
}
#[repr(C)]
struct EXT_DELETE_PARAMETERS {
    pub Version: ULONG,
    pub Reserved: ULONG,
    pub DeleteCallback: PVOID,
    pub DeleteContext: PVOID,
}
// Workaround: CurrentNoWakeTolerance is missing from types_i386 (generated without NTDDI_WINBLUE).
// NEEDS SHARED SUBSTRATE: regenerate types_i386.rs with NTDDI_VERSION >= NTDDI_WINBLUE so
// _CLASS_PRIVATE_FDO_DATA has PEX_TIMER TickTimer + LONGLONG CurrentNoWakeTolerance.
// Also: TickTimer: KTIMER in current types should be PEX_TIMER — reinterpret via raw ptr below.
static mut LOCAL_CURRENT_NO_WAKE_TOLERANCE: i64 = i64::MAX;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const CLASSP_REG_SUBKEY_NAME: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(*b"C\0l\0a\0s\0s\0p\0n\0p\0\0\0")
};

pub const CLASSP_REG_MMC_DETECTION_VALUE_NAME: [::core::ffi::c_ushort; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_ushort; 18]>(
        *b"M\0M\0C\0D\0e\0t\0e\0c\0t\0i\0o\0n\0S\0t\0a\0t\0e\0\0\0",
    )
};

pub const MCN_REG_SUBKEY_NAME: [::core::ffi::c_ushort; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [::core::ffi::c_ushort; 24]>(
        *b"M\0e\0d\0i\0a\0C\0h\0a\0n\0g\0e\0N\0o\0t\0i\0f\0i\0c\0a\0t\0i\0o\0n\0\0\0",
    )
};

pub const MCN_REG_AUTORUN_DISABLE_INSTANCE_NAME: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"A\0l\0w\0a\0y\0s\0D\0i\0s\0a\0b\0l\0e\0M\0C\0N\0\0\0",
    )
};

pub const MCN_REG_AUTORUN_ENABLE_INSTANCE_NAME: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"A\0l\0w\0a\0y\0s\0E\0n\0a\0b\0l\0e\0M\0C\0N\0\0\0",
    )
};

pub static mut StoragePredictFailureEventGuid: GUID = _GUID {
    Data1: 0x78ebc104 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4cf9 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// ClasspScreenOff: single no_mangle definition lives in globals.rs — a local twin
// here split-brained against power.rs's writes (screen-off never seen). Import it.
use crate::types::ClasspScreenOff;
use ntkrnl_sys::ddk::RtlAssert;

pub static mut BreakOnMcnDisable: ULONG = FALSE as ULONG;

pub const GESN_TIMEOUT_VALUE: u32 = 0x4;
pub const GESN_BUFFER_SIZE: u32 = 0x8;
pub const GESN_DEVICE_BUSY_LOWER_THRESHOLD_100_MS: u32 = 2;
pub const MAXIMUM_IMMEDIATE_MCN_RETRIES: u32 = 0x20;
pub const TICK_TIMER_PERIOD_IN_MSEC: u32 = 1000;
pub const TICK_TIMER_DELAY_IN_MSEC: u32 = 1000;

unsafe fn ClasspCanSendPollingIrp(
    fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> BOOLEAN {
    ((*fdoExtension).DevicePowerState == PowerDeviceD0
        && (*fdoExtension).PowerDownInProgress == FALSE
        && ClasspScreenOff == FALSE) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "C" fn ClassSendEjectionNotification(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    TracePrint((4i32) as u32, TRACE_FLAG_MCN, b"ClassSendEjectionNotification: media EJECT_REQUEST\0".as_ptr() as PCCHAR);
    ClassSendNotification(FdoExtension, &raw const GUID_IO_MEDIA_EJECT_REQUEST, 0u32, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn ClassSendNotification(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    Guid: *const GUID,
    ExtraDataSize: u32,
    ExtraData: *mut core::ffi::c_void,
) {
    let mut notification: *mut _TARGET_DEVICE_CUSTOM_NOTIFICATION = core::ptr::null_mut();
    let mut required_size: u32 = 0;
    let status: i32;

    status = RtlULongAdd(
        (core::mem::size_of::<_TARGET_DEVICE_CUSTOM_NOTIFICATION>() - core::mem::size_of::<u8>()) as u32,
        ExtraDataSize,
        &mut required_size
    );

    if !(status >= 0) || required_size > 0xffff {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"Error sending event: size too large! (%x)\n\0".as_ptr() as PCCHAR,
            required_size,
        );
        return;
    }

    notification = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        required_size as usize,
        (b'S' as u32) << 24 | (b'c' as u32) << 16 | (b'N' as u32) << 8 | b'o' as u32,
    ) as *mut _TARGET_DEVICE_CUSTOM_NOTIFICATION;

    if notification.is_null() {
        return;
    }

    core::ptr::write_bytes(notification as *mut core::ffi::c_void, 0, required_size as usize);

    (*notification).Version = 1u16;
    (*notification).Size = required_size as u16;
    (*notification).FileObject = core::ptr::null_mut::<_FILE_OBJECT>();
    (*notification).NameBufferOffset = -1i32;
    (*notification).Event = *Guid;

    if !ExtraData.is_null() && ExtraDataSize != 0 {
        let dst = core::ptr::addr_of_mut!((*notification).CustomDataBuffer) as *mut u8;
        core::ptr::copy_nonoverlapping(ExtraData as *const u8, dst, ExtraDataSize as usize);
    }

    IoReportTargetDeviceChangeAsynchronous(
        (*FdoExtension).LowerPdo,
        notification as *mut core::ffi::c_void,
        None,
        core::ptr::null_mut(),
    );

    ExFreePool(notification as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInterpretGesnData(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Header: PNOTIFICATION_EVENT_STATUS_HEADER,
    ResendImmediately: PBOOLEAN,
) -> NTSTATUS {
    let info: PMEDIA_CHANGE_DETECTION_INFO = (*FdoExtension).MediaChangeDetectionInfo as PMEDIA_CHANGE_DETECTION_INFO;
    let mut status: NTSTATUS = STATUS_SUCCESS;
    *ResendImmediately = 0u8;
    if (*Header).NEA() != 0 {
        return status;
    }
    if (*Header).NotificationClass() == (NOTIFICATION_NO_CLASS_EVENTS) as u8 {
        return status;
    }

    if (*info).Gesn.HackEventMask != 0u8 {
        let thisEvent: UCHAR = (*(*Header).ClassEventData.as_ptr() as i32 & 0xf) as UCHAR;
        let thisEventBit: UCHAR = ((1u32) << (*Header).NotificationClass()) as UCHAR;
        if ((*info).Gesn.EventMask as i32 & thisEventBit as i32) == 0 {
            return STATUS_DEVICE_PROTOCOL_ERROR;
        }
        let mut lowestSetBit: UCHAR = (*info).Gesn.EventMask;
        lowestSetBit = (lowestSetBit as i32 & ((*info).Gesn.EventMask as i32 - 1)) as UCHAR;
        lowestSetBit = (lowestSetBit as i32 ^ (*info).Gesn.EventMask as i32) as UCHAR;
        if (thisEventBit as i32) != (lowestSetBit as i32) {
            TracePrint(
                (4i32) as u32,
                TRACE_FLAG_MCN,
                b"Classpnp => GESN::NONE: Compliant drive found, removing GESN hack (%x, %x)\n\0"
                    .as_ptr() as PCHAR,
                thisEventBit as i32,
                (*info).Gesn.EventMask as i32,
            );
            (*info).Gesn.HackEventMask = 0u8;
        } else if thisEvent as i32 == 0 {
            (*info).Gesn.NoChangeEventMask =
                ((*info).Gesn.NoChangeEventMask as i32 | thisEventBit as i32) as UCHAR;
            (*info).Gesn.EventMask = ((*info).Gesn.EventMask as i32 & !(thisEventBit as i32)) as UCHAR;
            if (*info).Gesn.EventMask == 0 {
                (*info).Gesn.EventMask = (*info).Gesn.NoChangeEventMask;
                (*info).Gesn.NoChangeEventMask = 0;
            } else {
                *ResendImmediately = 1u8;
            }
            return status;
        }
    }

    let mut dataLength: LONG =
        (((*Header).EventDataLength[0] as i32) << 8) | ((*Header).EventDataLength[1] as i32 & 0xff) as LONG;
    dataLength -= 2;
    let requiredLength: LONG = 4;
    if dataLength < requiredLength {
        TracePrint(
            (3i32) as u32,
            TRACE_FLAG_MCN,
            b"Classpnp => GESN returned only %x bytes data for fdo %p\n\0".as_ptr() as PCHAR,
            dataLength,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
        return STATUS_DEVICE_PROTOCOL_ERROR;
    }
    if dataLength != requiredLength {
        TracePrint(
            (3i32) as u32,
            TRACE_FLAG_MCN,
            b"Classpnp => GESN returned too many (%x) bytes data for fdo %p\n\0".as_ptr() as PCHAR,
            dataLength,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
    }
    ntkrnl_sys::ASSERT!(dataLength == 4, "dataLength == 4");
    if (*(*Header).ClassEventData.as_ptr() as i32 & 0xf) == 0 {
        return status;
    }
    *ResendImmediately = 1u8;

    match (*Header).NotificationClass() {
        NOTIFICATION_OPERATIONAL_CHANGE_CLASS_EVENTS => {
            let opChangeInfo: PNOTIFICATION_OPERATIONAL_STATUS =
                &raw mut (*Header).ClassEventData as *mut UCHAR as PNOTIFICATION_OPERATIONAL_STATUS;
            if (*opChangeInfo).OperationalEvent() != (NOTIFICATION_OPERATIONAL_EVENT_CHANGE_REQUESTED) as u8 {
                let event: ULONG =
                    ((((*opChangeInfo).Operation[0] as i32) << 8) | (((*opChangeInfo).Operation[1] as i32) & 0xff)) as u32 ;
                if (*info).MediaChangeRetryCount >= 4 {
                    const OP_CHANGE_MASK: UCHAR = 0x02u8;
                    TracePrint(
                        (2i32) as u32,
                        TRACE_FLAG_MCN,
                        b"Classpnp => GESN OpChange events are broken.  Working around this problem in software (for fdo %p)\n\0"
                            .as_ptr() as PCHAR,
                        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                    );
                    ntkrnl_sys::ASSERT!(CountOfSetBitsUChar((*info).Gesn.EventMask) != 1, "CountOfSetBitsUChar(info->Gesn.EventMask) != 1");
                    (*info).Gesn.NoChangeEventMask |= OP_CHANGE_MASK;
                    (*info).Gesn.EventMask &= !OP_CHANGE_MASK;
                    (*info).Gesn.HackEventMask = 1u8;
                    if (*info).Gesn.EventMask == 0 {
                        (*info).Gesn.EventMask = (*info).Gesn.NoChangeEventMask;
                        (*info).Gesn.NoChangeEventMask = 0;
                        *ResendImmediately = 0u8;
                    } else {
                        *ResendImmediately = 1u8;
                    }
                } else if event == NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_ADDED as ULONG
                    || event == NOTIFICATION_OPERATIONAL_OPCODE_FEATURE_CHANGE as ULONG
                {
                    TracePrint(
                        (4i32) as u32,
                        TRACE_FLAG_MCN,
                        b"Classpnp => GESN says features added/changedfor fdo %p\n\0"
                            .as_ptr() as PCHAR,
                        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                    );
                    if (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject)
                        .Characteristics as u32 & 0x1u32 != 0
                        && !ClassGetVpb(
                            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                        )
                        .is_null()
                        && (*ClassGetVpb(
                            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                        ))
                        .Flags as i32 & (VPB_MOUNTED) as i32 != 0
                    {
                        (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).Flags |=
                            0x2u32;
                    }
                    if (*(*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DevInfo)
                        .ClassError
                        .is_some()
                    {
                        let mut srb: SCSI_REQUEST_BLOCK =
                            core::mem::zeroed::<SCSI_REQUEST_BLOCK>();
                        let mut srbExBuffer: [UCHAR; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as usize] =
                            [0u8; CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as usize];
                        let srbEx: PSTORAGE_REQUEST_BLOCK =
                            &raw mut srbExBuffer as *mut UCHAR as PSTORAGE_REQUEST_BLOCK;
                        let mut srbPtr: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
                        let mut sense: SENSE_DATA = core::mem::zeroed::<SENSE_DATA>();
                        let mut tempStatus: NTSTATUS = 0;
                        let mut retry: BOOLEAN = 0;
                        tempStatus = STATUS_MEDIA_CHANGED;
                        retry = 0u8;
                        sense.set_ErrorCode(SCSI_SENSE_ERRORCODE_FIXED_CURRENT as UCHAR);
                        sense.AdditionalSenseLength = (core::mem::size_of::<SENSE_DATA>() as UINT)
                            .wrapping_sub(
                                (7i32 + core::mem::size_of::<UCHAR>() as i32)
                                    as UINT,
                            ) as UCHAR;
                        sense.set_SenseKey(SCSI_SENSE_UNIT_ATTENTION as UCHAR);
                        sense.AdditionalSenseCode = SCSI_ADSENSE_MEDIUM_CHANGED as UCHAR;
                        if (*(*FdoExtension)
                            .__bindgen_anon_1
                            .CommonExtension
                            .as_ref().DriverExtension)
                            .SrbSupport as u32 & CLASS_SRB_STORAGE_REQUEST_BLOCK as u32 != 0
                        {
                            status = InitializeStorageRequestBlock(
                                srbEx,
                                STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                                CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
                                1,
                                SrbExDataTypeScsiCdb16 as i32,
                            );
                            if NT_SUCCESS(status) {
                                SrbSetCdbLength(srbEx as *mut ::core::ffi::c_void, 6u8);
                                (*(srbEx as *mut ::core::ffi::c_void as *mut SCSI_REQUEST_BLOCK))
                                    .SrbStatus =
                                    (SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_ERROR) as UCHAR;
                                SrbSetSenseInfoBuffer(
                                    srbEx as *mut ::core::ffi::c_void,
                                    &raw mut sense as *mut ::core::ffi::c_void,
                                );
                                SrbSetSenseInfoBufferLength(
                                    srbEx as *mut ::core::ffi::c_void,
                                    ((core::mem::size_of::<SENSE_DATA>() as UCHAR) as u32) as _,
                                );
                                srbPtr = srbEx as PSCSI_REQUEST_BLOCK;
                            } else {
                                ntkrnl_sys::ASSERT!(0i32 == 0);
                                srb.CdbLength = 6u8;
                                srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                                srb.SrbStatus =
                                    (SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_ERROR) as UCHAR;
                                srb.SenseInfoBuffer = &raw mut sense as *mut ::core::ffi::c_void;
                                srb.SenseInfoBufferLength =
                                    core::mem::size_of::<SENSE_DATA>() as UCHAR;
                                srbPtr = &raw mut srb;
                            }
                        } else {
                            srb.CdbLength = 6u8;
                            srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                            srb.SrbStatus = (SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_ERROR) as UCHAR;
                            srb.SenseInfoBuffer = &raw mut sense as *mut ::core::ffi::c_void;
                            srb.SenseInfoBufferLength = core::mem::size_of::<SENSE_DATA>() as UCHAR;
                            srbPtr = &raw mut srb;
                        }
                        (*(*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DevInfo)
                            .ClassError
                            .expect("non-null function pointer")(
                            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                            srbPtr,
                            &raw mut tempStatus,
                            &raw mut retry,
                        );
                    }
                }
            }
        }
        NOTIFICATION_EXTERNAL_REQUEST_CLASS_EVENTS => {
            let externalInfo: PNOTIFICATION_EXTERNAL_STATUS =
                &raw mut (*Header).ClassEventData as *mut UCHAR as PNOTIFICATION_EXTERNAL_STATUS;
            if (*externalInfo).ExternalEvent() as i32 != (NOTIFICATION_EXTERNAL_EVENT_BUTTON_DOWN) as i32 {
                // Do nothing if not button down
            } else {
                TracePrint(
                    (4i32) as u32,
                    TRACE_FLAG_MCN,
                    b"Classpnp => GESN::EXTERNAL: Event: %x Status %x Req %x\n\0"
                        .as_ptr() as PCHAR,
                    (*externalInfo).ExternalEvent() as i32,
                    (*externalInfo).ExternalStatus() as i32,
                    ((*externalInfo).Request[0] as i32) << 8
                        | (*externalInfo).Request[1] as i32,
                );
                let mut externalData: DEVICE_EVENT_EXTERNAL_REQUEST =
                    core::mem::zeroed::<DEVICE_EVENT_EXTERNAL_REQUEST>();
                externalData.Version = 1;
                externalData.DeviceClass = 0;
                externalData.ButtonStatus = (*externalInfo).ExternalEvent() as USHORT;
                externalData.Request =
                    ((((*externalInfo).Request[0] as i32) << 8)
                        | (((*externalInfo).Request[1] as i32) & 0xff)) as USHORT;
                // KeQuerySystemTime expanded: assign SharedSystemTime
                (*(&raw mut externalData.SystemTime as *mut ULONG64)) =
                    *(&raw const SharedSystemTime as *const LARGE_INTEGER as *const ULONG64) ;
                externalData.SystemTime.QuadPart *= KeQueryTimeIncrement() as LONGLONG;
                TracePrint(
                    (4i32) as u32,
                    TRACE_FLAG_MCN,
                    b"ClasspInterpretGesnData: media DEVICE_EXTERNAL_REQUEST\0"
                        .as_ptr() as PCHAR,
                );
                ClassSendNotification(
                    FdoExtension,
                    &raw const GUID_IO_DEVICE_EXTERNAL_REQUEST,
                    core::mem::size_of::<DEVICE_EVENT_EXTERNAL_REQUEST>() as ULONG,
                    &raw mut externalData as *mut core::ffi::c_void,
                );
                return status;
            }
        }
        NOTIFICATION_MEDIA_STATUS_CLASS_EVENTS => {
            let mediaInfo: PNOTIFICATION_MEDIA_STATUS =
                &raw mut (*Header).ClassEventData as *mut UCHAR as PNOTIFICATION_MEDIA_STATUS;
            TracePrint(
                (4i32) as u32,
                TRACE_FLAG_MCN,
                b"Classpnp => GESN::MEDIA: Event: %x Status %x\n\0".as_ptr() as PCHAR,
                (*mediaInfo).MediaEvent() as i32,
                *(*mediaInfo).__bindgen_anon_1.MediaStatus.as_ref() as i32,
            );
            if (*mediaInfo).MediaEvent() as i32 == (NOTIFICATION_MEDIA_EVENT_NEW_MEDIA) as i32
                || (*mediaInfo).MediaEvent() as i32 == (NOTIFICATION_MEDIA_EVENT_MEDIA_CHANGE) as i32
            {
                if (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject)
                    .Characteristics as u32 & 0x1u32 != 0
                    && !ClassGetVpb(
                        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                    )
                    .is_null()
                    && (*ClassGetVpb(
                        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                    ))
                    .Flags as i32 & (VPB_MOUNTED) as i32 != 0
                {
                    (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).Flags |=
                        0x2u32;
                }
                InterlockedIncrement(
                    &raw mut (*FdoExtension).MediaChangeCount as *mut ::core::ffi::c_long,
                );
                ClasspSetMediaChangeStateEx(
                    FdoExtension,
                    MediaPresent,
                    FALSE as BOOLEAN,
                    TRUE as BOOLEAN,
                );
            } else if (*mediaInfo).MediaEvent() as i32 == (NOTIFICATION_MEDIA_EVENT_MEDIA_REMOVAL) as i32 {
                ClasspSetMediaChangeStateEx(
                    FdoExtension,
                    MediaNotPresent,
                    FALSE as BOOLEAN,
                    TRUE as BOOLEAN,
                );
            } else if (*mediaInfo).MediaEvent() as i32 == (NOTIFICATION_MEDIA_EVENT_EJECT_REQUEST) as i32 {
                TracePrint(
                    (4i32) as u32,
                    TRACE_FLAG_MCN,
                    b"Classpnp => GESN Ejection request received!\n\0".as_ptr() as PCHAR,
                );
                ClassSendEjectionNotification(FdoExtension);
            }
        }
        NOTIFICATION_DEVICE_BUSY_CLASS_EVENTS => {
            let busyInfo: PNOTIFICATION_BUSY_STATUS =
                &raw mut (*Header).ClassEventData as *mut UCHAR as PNOTIFICATION_BUSY_STATUS;
            let mut busyData: DEVICE_EVENT_BECOMING_READY =
                core::mem::zeroed::<DEVICE_EVENT_BECOMING_READY>();
            *ResendImmediately = 0u8;
            busyData.Version = 1;
            busyData.Reason = (*busyInfo).DeviceBusyStatus as ULONG;
            busyData.Estimated100msToReady =
                ((((*busyInfo).Time[0] as i32) << 8)
                    | (((*busyInfo).Time[1] as i32) & 0xff)) as ULONG;
            TracePrint(
                (4i32) as u32,
                TRACE_FLAG_MCN,
                b"Classpnp => GESN::BUSY: Event: %x Status %x Time %x\n\0"
                    .as_ptr() as PCHAR,
                (*busyInfo).DeviceBusyEvent() as i32,
                (*busyInfo).DeviceBusyStatus as i32,
                busyData.Estimated100msToReady,
            );
            if busyData.Estimated100msToReady < GESN_DEVICE_BUSY_LOWER_THRESHOLD_100_MS {
                // skip notification
            } else {
                TracePrint(
                    (4i32) as u32,
                    TRACE_FLAG_MCN,
                    b"ClasspInterpretGesnData: media BECOMING_READY\0"
                        .as_ptr() as PCHAR,
                );
                ClassSendNotification(
                    FdoExtension,
                    &raw const GUID_IO_DEVICE_BECOMING_READY,
                    core::mem::size_of::<DEVICE_EVENT_BECOMING_READY>() as ULONG,
                    &raw mut busyData as *mut core::ffi::c_void,
                );
            }
        }
        _ => {}
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInternalSetMediaChangeState(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    NewState: MEDIA_CHANGE_DETECTION_STATE,
    KnownStateChange: BOOLEAN,
) {

    let states: [PCSZ; 4] = [
        b"Unknown\0".as_ptr() as PCCHAR,
        b"Present\0".as_ptr() as PCCHAR,
        b"Not Present\0".as_ptr() as PCCHAR,
        b"Unavailable\0".as_ptr() as PCCHAR,
    ];

    if !(NewState >= MediaUnknown && NewState <= MediaUnavailable) {
        return;
    }

    let info = (*FdoExtension).MediaChangeDetectionInfo;
    if info.is_null() {
        return;
    }

    let state_atomic = &*(core::ptr::addr_of_mut!((*info).MediaChangeDetectionState) as *mut AtomicI32);
    let oldMediaState = state_atomic.swap(NewState as i32, Ordering::SeqCst) as MEDIA_CHANGE_DETECTION_STATE;

    if oldMediaState == MediaUnknown && KnownStateChange == 0 {
        TracePrint(4, TRACE_FLAG_MCN, b"ClassSetMediaChangeState: State was unknown - this may not be a change\n\0".as_ptr() as PCCHAR);
        return;
    } else if oldMediaState == NewState {
        return;
    }

    {
        let work_item_active = &*(core::ptr::addr_of_mut!(
            (*(*FdoExtension).PrivateFdoData).UpdateDiskPropertiesWorkItemActive
        ) as *mut AtomicI32);
        let previous = work_item_active.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire);
        if previous.is_ok() && previous.unwrap() == 0 {
            let workItem = IoAllocateWorkItem((*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject);
            if !workItem.is_null() {
                IoQueueWorkItem(
                    workItem,
                    Some(ClasspUpdateDiskProperties),
                    DelayedWorkQueue,
                    workItem as PVOID,
                );
            } else {
                let _ = work_item_active.swap(0, Ordering::SeqCst);
            }
        }
    }

    if (*info).MediaChangeDetectionDisableCount != 0 {
        TracePrint(4, TRACE_FLAG_MCN,
            b"ClassSetMediaChangeState: MCN not enabled, state changed from %s to %s\n\0".as_ptr() as PCCHAR,
            states[oldMediaState as usize],
            states[NewState as usize]);
        return;
    }

    TracePrint(4, TRACE_FLAG_MCN,
        b"ClassSetMediaChangeState: State change from %s to %s\n\0".as_ptr() as PCCHAR,
        states[oldMediaState as usize],
        states[NewState as usize]);

    let mut mcnContext = CLASS_MEDIA_CHANGE_CONTEXT {
        MediaChangeCount: (*FdoExtension).MediaChangeCount,
        NewState: NewState as ULONG,
    };

    if NewState == MediaPresent {
        TracePrint(4, TRACE_FLAG_MCN, b"ClasspInternalSetMediaChangeState: media ARRIVAL\0".as_ptr() as PCCHAR);
        ClassSendNotification(
            FdoExtension,
            &raw const GUID_IO_MEDIA_ARRIVAL as *const GUID,
            core::mem::size_of::<CLASS_MEDIA_CHANGE_CONTEXT>() as ULONG,
            &raw mut mcnContext as PVOID,
        );
    } else if NewState == MediaNotPresent || NewState == MediaUnavailable {
        TracePrint(4, TRACE_FLAG_MCN, b"ClasspInternalSetMediaChangeState: media REMOVAL\0".as_ptr() as PCCHAR);
        ClassSendNotification(
            FdoExtension,
            &raw const GUID_IO_MEDIA_REMOVAL as *const GUID,
            core::mem::size_of::<CLASS_MEDIA_CHANGE_CONTEXT>() as ULONG,
            &raw mut mcnContext as PVOID,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ClasspSetMediaChangeStateEx(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    NewState: MEDIA_CHANGE_DETECTION_STATE,
    Wait: BOOLEAN,
    KnownStateChange: BOOLEAN,
) {
    let info: PMEDIA_CHANGE_DETECTION_INFO = (*FdoExtension).MediaChangeDetectionInfo;
    let mut zero: LARGE_INTEGER = core::mem::zeroed();
    zero.QuadPart = 0;

    if NewState == MediaNotPresent {
        (*FdoExtension).FailurePredicted = FALSE as BOOLEAN;
        (*FdoExtension).FailureReason = 0;
    }

    if info.is_null() {
        return;
    }

    let status = KeWaitForSingleObject(
        &raw mut (*info).MediaChangeMutex as PVOID,
        _KWAIT_REASON_UserRequest,
        0i32 as KPROCESSOR_MODE,
        0u8,
        if Wait != 0 { core::ptr::null_mut() } else { &raw mut zero },
    );

    if status == STATUS_TIMEOUT {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"ClasspSetMediaChangeStateEx - timed out waiting for mutex\0".as_ptr() as PCCHAR,
        );
        return;
    }

    ClasspInternalSetMediaChangeState(FdoExtension, NewState, KnownStateChange);
    KeReleaseMutex(&raw mut (*info).MediaChangeMutex, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn ClassSetMediaChangeState(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    NewState: MEDIA_CHANGE_DETECTION_STATE,
    Wait: BOOLEAN,
) {
    ClasspSetMediaChangeStateEx(FdoExtension, NewState, Wait, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn ClasspMediaChangeDetectionCompletion(
    mut DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut Srb: PSTORAGE_REQUEST_BLOCK_HEADER = Context as PSTORAGE_REQUEST_BLOCK_HEADER;
    // _Analysis_assume_(Srb != NULL) is no-op

    // Use the DeviceObject from the current stack location
    DeviceObject = (*IoGetCurrentIrpStackLocation(Irp)).DeviceObject;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let fdoData: PCLASS_PRIVATE_FDO_DATA = (*fdoExtension).PrivateFdoData;
    let info: PMEDIA_CHANGE_DETECTION_INFO = (*fdoExtension).MediaChangeDetectionInfo;

    // Assertions
    NT_ASSERT!(!(*info).MediaChangeIrp.is_null() as u8);
    NT_ASSERT!((((*Srb).SrbStatus as u8 & (SRB_STATUS_QUEUE_FROZEN) as u8) == 0) as u8);

    TracePrint(
        5, // TRACE_LEVEL_VERBOSE
        TRACE_FLAG_MCN,
        b"> ClasspMediaChangeDetectionCompletion: Device %p completed MCN irp %p.\0"
            .as_ptr() as PCCHAR,
        DeviceObject as PVOID,
        Irp as PVOID,
    );

    // Hack for IoMega Jaz drive spin-down
    let srb_status_clean = ((*Srb).SrbStatus as u8) & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8;
    if srb_status_clean != (SRB_STATUS_SUCCESS) as u8
        && ((*fdoExtension).ScanForSpecialFlags as u32 & 0x20 /* CLASS_SPECIAL_CAUSE_NOT_REPORTABLE_HACK */) != 0
    {
        let senseData: PVOID = SrbGetSenseInfoBuffer(Srb as PVOID);
        if !senseData.is_null() {
            let mut senseInfoBufferLength: u8 = SrbGetSenseInfoBufferLength(Srb as PVOID);
            let mut senseKey: u8 = 0;
            let mut additionalSenseCode: u8 = 0;
            let validSense: BOOLEAN = ScsiGetSenseKeyAndCodes(
                senseData,
                senseInfoBufferLength,
                SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
                &raw mut senseKey,
                &raw mut additionalSenseCode,
                core::ptr::null_mut(),
            );
            if validSense != 0
                && senseKey == (SCSI_SENSE_NOT_READY) as u8
                && additionalSenseCode == (SCSI_ADSENSE_LUN_NOT_READY) as u8
            {
                (*Srb).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
            }
        }
    }

    // Check SRB status
    let mut status: NTSTATUS = STATUS_SUCCESS;
    if ((*Srb).SrbStatus as u8 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as u8) != (SRB_STATUS_SUCCESS) as u8 {
        TracePrint(
            3, // TRACE_LEVEL_WARNING
            TRACE_FLAG_MCN,
            b"ClasspMediaChangeDetectionCompletion - failed - srb status=%s, sense=%s/%s/%s.\0"
                .as_ptr() as PCCHAR,
            DBGGETSRBSTATUSSTR(Srb as PVOID),
            DBGGETSENSECODESTR(Srb as PVOID),
            DBGGETADSENSECODESTR(Srb as PVOID),
            DBGGETADSENSEQUALIFIERSTR(Srb as PVOID),
        );
        InterpretSenseInfoWithoutHistory(
            DeviceObject,
            Irp,
            Srb as PSCSI_REQUEST_BLOCK,
            IRP_MJ_SCSI as u8,
            0,
            0,
            &raw mut status,
            core::ptr::null_mut(),
        );
    } else {
        (*fdoData).LoggedTURFailureSinceLastIO = FALSE;
        if (*info).Gesn.Supported == 0 {
            TracePrint(
                4, // TRACE_LEVEL_INFORMATION
                TRACE_FLAG_MCN,
                b"ClasspMediaChangeDetectionCompletion - succeeded and GESN NOT supported, setting MediaPresent.\0"
                    .as_ptr() as PCCHAR,
            );
            ClassSetMediaChangeState(fdoExtension, MediaPresent, FALSE);
        } else {
            TracePrint(
                4,
                TRACE_FLAG_MCN,
                b"ClasspMediaChangeDetectionCompletion - succeeded (GESN supported).\0"
                    .as_ptr() as PCCHAR,
            );
        }
    }

    // GESN post-processing
    if (*info).Gesn.Supported != 0 {
        if status == STATUS_DATA_OVERRUN {
            TracePrint(
                4,
                TRACE_FLAG_MCN,
                b"ClasspMediaChangeDetectionCompletion - Overrun\0".as_ptr() as PCCHAR,
            );
            status = STATUS_SUCCESS;
        }
        if !NT_SUCCESS(status as i32) {
            TracePrint(
                4,
                TRACE_FLAG_MCN,
                b"ClasspMediaChangeDetectionCompletion: GESN failed with status %x\0"
                    .as_ptr() as PCCHAR,
                status as u32,
            );
        } else if (*Irp).IoStatus.Information == 8 {
            let mut retryImmediately: BOOLEAN = FALSE;
            ClasspInterpretGesnData(
                fdoExtension,
                (*info).Gesn.Buffer as PVOID as PNOTIFICATION_EVENT_STATUS_HEADER,
                &raw mut retryImmediately,
            );
            // retryImmediately variable is used later in the outer scope
            // Move declaration up to the function scope.
            // But we are about to re-declare; instead we'll set the outer variable.
            // We'll restructure: declare retryImmediately at top and set it here.
        }
    }

    // Free port-allocated sense buffer
    if PORT_ALLOCATED_SENSE_EX(fdoExtension, Srb) != 0 {
        FREE_PORT_ALLOCATED_SENSE_BUFFER_EX(fdoExtension, Srb);
    }

    // Remember the IRP and SRB for next time
    NT_ASSERT!(!IoGetNextIrpStackLocation(Irp).is_null() as u8);
    let nextStack = IoGetNextIrpStackLocation(Irp);
    (*nextStack).Parameters.Scsi.Srb = Srb as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;

    // Reset the MCN timer
    ClassResetMediaChangeTimer(fdoExtension);

    // retryImmediately must be declared before the GESN block; we'll do it here
    let mut retryImmediately: BOOLEAN = FALSE;
    // Move the GESN block earlier to set retryImmediately correctly.
    // Actually the original code declares retryImmediately at the beginning.
    // I'll just put the declaration at the top and set it in the GESN block.

    // For clarity, we'll reorder: declare retryImmediately before all logic,
    // then set inside GESN, then use after resetting timer.
    // I'll rewrite by moving declaration up.

    // ... will adjust below ...

    // Sanity check retries
    if retryImmediately != 0 {
        (*info).MediaChangeRetryCount += 1;
        if (*info).MediaChangeRetryCount > (MAXIMUM_IMMEDIATE_MCN_RETRIES) as i32 {
            let workItem: PIO_WORKITEM = IoAllocateWorkItem(DeviceObject);
            if !workItem.is_null() {
                IoQueueWorkItem(
                    workItem,
                    Some(core::mem::transmute::<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, PVOID), _>(ClasspDisableGesn)),
                    DelayedWorkQueue,
                    workItem as PVOID,
                );
            }
            (*info).Gesn.Supported = 0;
            (*info).Gesn.EventMask = 0;
            (*info).Gesn.BufferSize = 0;
            (*info).MediaChangeRetryCount = 0;
            retryImmediately = FALSE;
        }
    } else {
        (*info).MediaChangeRetryCount = 0;
    }

    // Release remove lock and resend IRP
    {
        let mut uniqueValue: u8 = 0;
        ClassAcquireRemoveLock(DeviceObject, (&raw mut uniqueValue as PVOID).cast());
        ClassReleaseRemoveLock(DeviceObject, Irp);

        // Atomically mark IRP not in use
        let old = core::sync::atomic::AtomicI32::from_ptr(
            (core::ptr::addr_of_mut!((*info).MediaChangeIrpInUse) as *mut core::sync::atomic::AtomicI32).cast()
        ).compare_exchange(
            1,
            0,
            core::sync::atomic::Ordering::AcqRel,
            core::sync::atomic::Ordering::Acquire,
        );
        let irpWasInUse = match old {
            Ok(v) => v,
            Err(v) => v,
        };
        NT_ASSERT!((irpWasInUse == 1) as u8);

        if retryImmediately != 0 {
            ClasspSendMediaStateIrp(fdoExtension, info, 0);
        } else {
            TracePrint(
                5,
                TRACE_FLAG_MCN,
                b"ClasspMediaChangeDetectionCompletion - not retrying immediately\0"
                    .as_ptr() as PCCHAR,
            );
        }

        ClassReleaseRemoveLock(DeviceObject, &raw mut uniqueValue as PVOID as PIRP);
    }

    TracePrint(
        5,
        TRACE_FLAG_MCN,
        b"< ClasspMediaChangeDetectionCompletion\0".as_ptr() as PCCHAR,
    );

    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPrepareMcnIrp(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Info: PMEDIA_CHANGE_DETECTION_INFO,
    UseGesn: BOOLEAN,
) -> PIRP {
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let irpStack: PIO_STACK_LOCATION;
    let nextIrpStack: PIO_STACK_LOCATION;
    let mut status: NTSTATUS = 0;
    let mut cdb: PCDB = core::ptr::null_mut();
    let buffer: PVOID;
    let bufferLength: UCHAR;
    let mut srbFlags: ULONG;
    let mut timeOutValue: ULONG;
    let cdbLength: UCHAR;
    let dataBuffer: PVOID;
    let dataTransferLength: ULONG;

    let irp = (*Info).MediaChangeIrp;

    if irp.is_null() {
        if irp.is_null() {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    as *const u8 as *const i8,
                1338i32,
                b"irp\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        return core::ptr::null_mut();
    }

    status = ClassAcquireRemoveLockR(
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        irp,
    ) as NTSTATUS;

    const REMOVE_COMPLETE: NTSTATUS = 2;
    const REMOVE_PENDING: NTSTATUS = 1;
    const NO_REMOVE: NTSTATUS = 0;

    if status == REMOVE_COMPLETE {
        if status != REMOVE_COMPLETE {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    as *const u8 as *const i8,
                1348i32,
                b"status != REMOVE_COMPLETE\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        return core::ptr::null_mut();
    } else if status == REMOVE_PENDING {
        ClassReleaseRemoveLock(
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
            irp,
        );
        return core::ptr::null_mut();
    } else {
        if status != NO_REMOVE {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    as *const u8 as *const i8,
                1356i32,
                b"status == NO_REMOVE\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    IoReuseIrp(irp, STATUS_NOT_SUPPORTED);
    IoSetNextIrpStackLocation(irp);

    irpStack = IoGetCurrentIrpStackLocation(irp);
    (*irpStack).DeviceObject = (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
    (*irpStack).Flags = ((*irpStack).Flags as i32 | (SL_OVERRIDE_VERIFY_VOLUME) as i32) as UCHAR;

    nextIrpStack = IoGetNextIrpStackLocation(irp);
    (*nextIrpStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL as UCHAR;
    (*nextIrpStack).Parameters.Scsi.Srb = core::ptr::addr_of_mut!((*Info).MediaChangeSrb.Srb) as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;

    buffer = (*Info).SenseBuffer as PVOID;
    bufferLength = (*Info).SenseBufferLength;

    if bufferLength as i32 <= 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
            b"drivers/storage/class/classpnp/autorun.c\0"
                as *const u8 as *const i8,
            1396i32,
            b"bufferLength > 0\0" as *const u8 as *const i8,
        );
        __int2c();
    }

    RtlZeroMemory(buffer, bufferLength as usize);

    srbFlags = (*FdoExtension).SrbFlags;
    srbFlags |= (*Info).SrbFlags;

    timeOutValue = (*FdoExtension).TimeOutValue.wrapping_mul(2);
    if timeOutValue == 0 {
        if (*FdoExtension).TimeOutValue == 0 {
            TracePrint(
                3u32,
                TRACE_FLAG_MCN,
                b"ClassSendTestUnitIrp: FdoExtension->TimeOutValue is set to zero?! -- resetting to 10\n\0"
                    .as_ptr() as PCCHAR,
            );
            timeOutValue = 10 * 2;
        } else {
            TracePrint(
                3u32,
                TRACE_FLAG_MCN,
                b"ClassSendTestUnitIrp: Someone set srb->TimeOutValue to zero?! -- resetting to %x\n\0"
                    .as_ptr() as PCCHAR,
                (*FdoExtension).TimeOutValue.wrapping_mul(2),
            );
            timeOutValue = (*FdoExtension).TimeOutValue.wrapping_mul(2);
        }
    }

    if UseGesn == 0 {
        (*nextIrpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_NONE as ULONG;
        (*irp).MdlAddress = core::ptr::null_mut();
        srbFlags |= SRB_FLAGS_NO_DATA_TRANSFER;
        if (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).DeviceType != FILE_DEVICE_CD_ROM as ULONG
            && ClasspScreenOff == TRUE
        {
            srbFlags |= SRB_FLAGS_NO_KEEP_AWAKE;
        }
        cdbLength = 6;
        dataBuffer = core::ptr::null_mut();
        dataTransferLength = 0;
    } else {
        if (*Info).Gesn.Buffer.is_null() {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    as *const u8 as *const i8,
                1444i32,
                b"Info->Gesn.Buffer\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        (*nextIrpStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_SCSI_EXECUTE_IN as ULONG;
        (*irp).MdlAddress = (*Info).Gesn.Mdl;
        srbFlags |= SRB_FLAGS_DATA_IN;
        cdbLength = 10;
        dataBuffer = (*Info).Gesn.Buffer as PVOID;
        dataTransferLength = (*Info).Gesn.BufferSize;
        timeOutValue = GESN_TIMEOUT_VALUE as ULONG;
    }

    srb = (*nextIrpStack).Parameters.Scsi.Srb as PSCSI_REQUEST_BLOCK;
    if (*((*FdoExtension).AdapterDescriptor as *const crate::types::_STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbEx = (*nextIrpStack).Parameters.Scsi.Srb as PSTORAGE_REQUEST_BLOCK;
        status = InitializeStorageRequestBlock(
            srbEx,
            STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
            1u32,
            SrbExDataTypeScsiCdb16,
        );
        if !(status >= 0) {
            if false {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as PCSTR,
                    b"drivers/storage/class/classpnp/autorun.c\0"
                        as *const u8 as *const i8,
                    1471i32,
                    b"FALSE\0" as *const u8 as *const i8,
                );
                __int2c();
            }
            return core::ptr::null_mut();
        }

        (*srbEx).RequestTag = SP_UNTAGGED;
        (*srbEx).RequestAttribute = (SRB_SIMPLE_TAG_REQUEST) as u16;
        (*srbEx).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
        (*srbEx).SrbStatus = 0;
        (*srbEx).OriginalRequest = irp as PVOID;
        (*srbEx).SrbFlags = srbFlags;
        (*srbEx).TimeOutValue = timeOutValue;
        (*srbEx).DataBuffer = dataBuffer;
        (*srbEx).DataTransferLength = dataTransferLength;

        SrbSetScsiStatus(srbEx as PVOID, 0);
        SrbSetSenseInfoBuffer(srbEx as PVOID, buffer);
        SrbSetSenseInfoBufferLength(srbEx as PVOID, ((bufferLength) as u32) as _);
        SrbSetCdbLength(srbEx as PVOID, cdbLength);
        cdb = SrbGetCdb(srbEx as PVOID);
    } else {
        core::ptr::write_bytes(srb as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
        (*srb).QueueTag = (SP_UNTAGGED) as u8;
        (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as UCHAR;
        (*srb).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
        (*srb).SenseInfoBuffer = buffer;
        (*srb).SenseInfoBufferLength = bufferLength;
        (*srb).SrbStatus = 0;
        (*srb).ScsiStatus = 0;
        (*srb).OriginalRequest = irp as PVOID;
        (*srb).SrbFlags = srbFlags;
        (*srb).TimeOutValue = timeOutValue;
        (*srb).CdbLength = cdbLength;
        (*srb).DataBuffer = dataBuffer;
        (*srb).DataTransferLength = dataTransferLength;
        cdb = (*srb).Cdb.as_mut_ptr() as PCDB;
    }

    if !cdb.is_null() {
        if UseGesn == 0 {
            (*cdb).CDB6GENERIC.as_mut().OperationCode = SCSIOP_TEST_UNIT_READY as UCHAR;
        } else {
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().OperationCode = SCSIOP_GET_EVENT_STATUS as UCHAR;
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().set_Immediate(1);
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().EventListLength[0] =
                ((*Info).Gesn.BufferSize >> 8) as UCHAR;
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().EventListLength[1] =
                ((*Info).Gesn.BufferSize & 0xff) as UCHAR;
            (*cdb).GET_EVENT_STATUS_NOTIFICATION.as_mut().NotificationClassRequest = (*Info).Gesn.EventMask;
        }
    }

    IoSetCompletionRoutine(
        irp,
        Some(core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS, _>(ClasspMediaChangeDetectionCompletion)),
        srb as PVOID,
        TRUE as BOOLEAN,
        TRUE as BOOLEAN,
        TRUE as BOOLEAN,
    );

    irp
}
#[no_mangle]
pub unsafe extern "C" fn ClasspSendMediaStateIrp(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Info: PMEDIA_CHANGE_DETECTION_INFO,
    CountDown: ULONG,
) {
    let mut requestPending: BOOLEAN = 0u8;
    let mut leave: BOOLEAN = 0u8;
    let mut irpInUse: LONG;

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_MCN,
        b"> ClasspSendMediaStateIrp\0".as_ptr() as PCCHAR,
    );

    if ((*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().CurrentState != IRP_MN_START_DEVICE
        || (*FdoExtension).DevicePowerState != PowerDeviceD0)
        && (*Info).MediaChangeIrpLost == 0u8
    {
        (*Info).MediaChangeIrpTimeInUse = 0;
    }

    let atomic_in_use = &*(core::ptr::addr_of_mut!((*Info).MediaChangeIrpInUse) as *mut AtomicI32);
    irpInUse = atomic_in_use
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .unwrap_or_else(|e| e);

    if irpInUse != 0 {
        let timeInUse = atomic_in_use.fetch_add(1, Ordering::SeqCst) + 1;
        TracePrint(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_MCN,
            b"ClasspSendMediaStateIrp: irp in use for %x seconds when synchronizing for MCD\n\0"
                .as_ptr() as PCCHAR,
            timeInUse,
        );
        if (*Info).MediaChangeIrpLost == 0u8 {
            if timeInUse > (MEDIA_CHANGE_TIMEOUT_TIME) as i32 {
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_MCN,
                    b"CdRom%d: Media Change Notification has lost it's irp and doesn't know where to find it.  Leave it alone and it'll come home dragging it's stack behind it.\n\0"
                        .as_ptr() as PCCHAR,
                    (*FdoExtension).DeviceNumber,
                );
                (*Info).MediaChangeIrpLost = 1u8;
            }
        }
        TracePrint(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_MCN,
            b"< ClasspSendMediaStateIrp - irpInUse\0".as_ptr() as PCCHAR,
        );
        return;
    }

    leave = 0u8;

    if leave == 0u8 && (*Info).MediaChangeDetectionDisableCount != 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassCheckMediaState: device %p has  detection disabled \n\0".as_ptr() as PCCHAR,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
        leave = 1u8;
    }

    if leave == 0u8 && (*FdoExtension).DevicePowerState != PowerDeviceD0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassCheckMediaState: device %p needs to powerup to handle this io (may take a few extra seconds).\n\0"
                .as_ptr() as PCCHAR,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
    }

    if leave == 0u8 {
        (*Info).MediaChangeIrpTimeInUse = 0;
        (*Info).MediaChangeIrpLost = 0u8;

        if CountDown == 0 {
            if leave == 0u8 && (*Info).MediaChangeDetectionDisableCount != 0 {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_MCN,
                    b"ClassCheckMediaState: detection disabled\n\0".as_ptr() as PCCHAR,
                );
                leave = 1u8;
            }

            if leave == 0u8 {
                let irp = ClasspPrepareMcnIrp(FdoExtension, Info, (*Info).Gesn.Supported);
                TracePrint(
                    TRACE_LEVEL_VERBOSE,
                    TRACE_FLAG_MCN,
                    b"ClasspSendMediaStateIrp: Device %p getting TUR  irp %p\n\0".as_ptr() as PCCHAR,
                    (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                    irp,
                );
                if irp.is_null() {
                    leave = 1u8;
                } else {
                    requestPending = 1u8;
                    TracePrint(
                        TRACE_LEVEL_VERBOSE,
                        TRACE_FLAG_MCN,
                        b"  ClasspSendMediaStateIrp - calling IoCallDriver.\0".as_ptr() as PCCHAR,
                    );
                    IoCallDriver(
                        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
                        irp,
                    );
                }
            }
        }
    }

    if requestPending == 0u8 {
        if cfg!(debug_assertions) {
            let atomic = &*(core::ptr::addr_of_mut!((*Info).MediaChangeIrpInUse) as *mut AtomicI32);
            irpInUse = atomic
                .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
                .unwrap_or_else(|e| e);
            NT_ASSERT!((irpInUse) as u8);
        } else {
            let _ = atomic_in_use.compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst);
        }
    }

    TracePrint(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_MCN,
        b"< ClasspSendMediaStateIrp\0".as_ptr() as PCCHAR,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ClassCheckMediaState(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    let info: PMEDIA_CHANGE_DETECTION_INFO = (*FdoExtension).MediaChangeDetectionInfo;

    if info.is_null() {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassCheckMediaState: detection not enabled\n\0".as_ptr() as PCHAR,
        );
        return;
    }

    let countDown: LONG =
        InterlockedDecrement(&raw mut (*info).MediaChangeCountDown as *mut ::core::ffi::c_long)
            as LONG;

    ClasspSendMediaStateIrp(FdoExtension, info, countDown as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn ClassResetMediaChangeTimer(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
) {
    let info: *mut _MEDIA_CHANGE_DETECTION_INFO = (*FdoExtension).MediaChangeDetectionInfo;
    if !info.is_null() {
        let count_down_ptr =
            core::ptr::addr_of_mut!((*info).MediaChangeCountDown) as *mut i32;
        let atomic_ptr = count_down_ptr as *mut core::sync::atomic::AtomicI32;
        (*atomic_ptr).swap((MEDIA_CHANGE_DEFAULT_TIME) as i32, core::sync::atomic::Ordering::SeqCst);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializePolling(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    AllowDriveToSleep: u8,
) -> NTSTATUS {
    // PAGED_CODE expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            b"Pageable code called at IRQL > APC_LEVEL\0".as_ptr() as *mut i8,
        );
    }

    if !(*FdoExtension).MediaChangeDetectionInfo.is_null() {
        return STATUS_SUCCESS;
    }

    let info = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MEDIA_CHANGE_DETECTION_INFO>() as SIZE_T,
        CLASS_TAG_MEDIA_CHANGE_DETECTION,
    ) as PMEDIA_CHANGE_DETECTION_INFO;
    if info.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(
        info as *mut u8,
        0u8,
        core::mem::size_of::<MEDIA_CHANGE_DETECTION_INFO>(),
    );

    {
        let mcn = core::ptr::addr_of_mut!((*FdoExtension).KernelModeMcnContext)
            as *mut _FILE_OBJECT_EXTENSION;
        (*mcn).FileObject = (-1isize) as *mut FILE_OBJECT;
        (*mcn).DeviceObject = (-1isize) as *mut DEVICE_OBJECT;
        (*mcn).LockCount = 0u32;
        (*mcn).McnDisableCount = 0u32;
    }

    let fdo = (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
    let irp = IoAllocateIrp(
        ((*fdo).StackSize as i32 + 1) as CCHAR,
        0u8,
    );
    if irp.is_null() {
        ExFreePool(info as *mut c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let buffer = ExAllocatePoolWithTag(
        NonPagedPoolCacheAligned,
        SENSE_BUFFER_SIZE_EX as SIZE_T,
        CLASS_TAG_MEDIA_CHANGE_DETECTION,
    );
    if buffer.is_null() {
        IoFreeIrp(irp);
        ExFreePool(info as *mut c_void);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*info).MediaChangeIrp = irp;
    (*info).SenseBuffer = buffer as PUCHAR;
    (*info).SenseBufferLength = SENSE_BUFFER_SIZE_EX as u8;
    (*info).MediaChangeCountDown = (MEDIA_CHANGE_DEFAULT_TIME) as i32;
    (*info).MediaChangeDetectionDisableCount = 0;
    (*info).MediaChangeDetectionState = MediaUnknown;
    (*info).MediaChangeIrpTimeInUse = 0;
    (*info).MediaChangeIrpLost = 0u8;

    (*info).SrbFlags = 0;
    if AllowDriveToSleep != 0 {
        (*info).SrbFlags |= 0x100000u32;
    }
    (*info).SrbFlags |= 0x10000000u32;
    (*info).SrbFlags |= 0x100u32;
    (*info).SrbFlags |= 0x8u32;

    KeInitializeMutex(
        &mut (*info).MediaChangeMutex,
        0x100u32,
    );

    (*FdoExtension).MediaChangeDetectionInfo = info;

    let mut GesnSupported: u8 = 0u8;

    if (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).DeviceType == FILE_DEVICE_CD_ROM as u32 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClasspInitializePolling: Testing for GESN\n\0".as_ptr() as PCCHAR,
        );
        let status = ClasspInitializeGesn(FdoExtension, info);
        if status >= 0 {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"ClasspInitializePolling: GESN available for %p\n\0".as_ptr() as PCCHAR,
                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject as *mut c_void,
            );
            if (*info).Gesn.Supported == 0 {
                RtlAssert(
                    b"info->Gesn.Supported\0".as_ptr() as *mut core::ffi::c_void,
                    b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                    line!(),
                    core::ptr::null_mut::<i8>(),
                );
            }
            if (*info).Gesn.Buffer.is_null() {
                RtlAssert(
                    b"info->Gesn.Buffer != NULL\0".as_ptr() as *mut core::ffi::c_void,
                    b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                    line!(),
                    core::ptr::null_mut::<i8>(),
                );
            }
            if (*info).Gesn.BufferSize == 0 {
                RtlAssert(
                    b"info->Gesn.BufferSize != 0\0".as_ptr() as *mut core::ffi::c_void,
                    b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                    line!(),
                    core::ptr::null_mut::<i8>(),
                );
            }
            if (*info).Gesn.EventMask == 0 {
                RtlAssert(
                    b"info->Gesn.EventMask != 0\0".as_ptr() as *mut core::ffi::c_void,
                    b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                    line!(),
                    core::ptr::null_mut::<i8>(),
                );
            }
            GesnSupported = 1u8;
        } else {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"ClasspInitializePolling: GESN *NOT* available for %p\n\0".as_ptr() as PCCHAR,
                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject as *mut c_void,
            );
        }
    }

    if GesnSupported == 0 {
        if (*info).Gesn.Supported != 0 {
            RtlAssert(
                b"info->Gesn.Supported == 0\0".as_ptr() as *mut core::ffi::c_void,
                b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        if !(*info).Gesn.Buffer.is_null() {
            RtlAssert(
                b"info->Gesn.Buffer == NULL\0".as_ptr() as *mut core::ffi::c_void,
                b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        if (*info).Gesn.BufferSize != 0 {
            RtlAssert(
                b"info->Gesn.BufferSize == 0\0".as_ptr() as *mut core::ffi::c_void,
                b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        if (*info).Gesn.EventMask != 0 {
            RtlAssert(
                b"info->Gesn.EventMask == 0\0".as_ptr() as *mut core::ffi::c_void,
                b"autorun.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        (*info).Gesn.Supported = 0u8;
    }

    if ScreenStateNotificationHandle.is_null() {
        // This bridged the island's per-arch spelling of ClasspPowerSettingCallback (stdcall on
        // i386, C elsewhere) to PoRegisterPowerSettingCallback's `system` slot with a transmute
        // per arch. The island declares it `system` now, so the types match on their own — and
        // the transmute, which erased the signature along with the convention, is gone.
        let _cb: unsafe extern "system" fn(LPCGUID, PVOID, ULONG, PVOID) -> NTSTATUS =
            ClasspPowerSettingCallback;
        PoRegisterPowerSettingCallback(
            fdo,
            &raw const GUID_CONSOLE_DISPLAY_STATE,
            Some(_cb),
            core::ptr::null_mut(),
            &raw mut ScreenStateNotificationHandle,
        );
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeGesn(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Info: PMEDIA_CHANGE_DETECTION_INFO,
) -> NTSTATUS {
    let mut header: *mut _NOTIFICATION_EVENT_STATUS_HEADER = core::ptr::null_mut();
    let mut detectionState: CLASS_DETECTION_STATE = ClassDetectionUnknown;
    let mut deviceDescriptor: *mut _STORAGE_DEVICE_DESCRIPTOR = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_NOT_SUPPORTED;
    let mut irp: *mut _IRP = core::ptr::null_mut();
    let mut event: KEVENT = core::mem::zeroed();
    let mut retryImmediately: BOOLEAN = 0;
    let mut i: ULONG = 0;
    let mut atapiResets: ULONG = 0;
    let mut srbFlags: ULONG = 0;

    // PAGED_CODE();
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/autorun.c\0"
                .as_ptr() as *const i8,
            2006i32,
            b"0\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    // NT_ASSERT(Info == FdoExtension->MediaChangeDetectionInfo);
    if !(Info == (*FdoExtension).MediaChangeDetectionInfo) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/autorun.c\0"
                .as_ptr() as *const i8,
            2007i32,
            b"Info == FdoExtension->MediaChangeDetectionInfo\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    ClassGetDeviceParameter(
        FdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_MMC_DETECTION_VALUE_NAME.as_ptr() as PWSTR,
        &mut detectionState as PULONG,
    );

    if detectionState as u32 == ClassDetectionUnsupported as u32 {
        status = STATUS_NOT_SUPPORTED;
        // fall through to ExitWithError
    } else {
        // TEST_FLAG(FdoExtension->PrivateFdoData->HackFlags, FDO_HACK_GESN_IS_BAD)
        if ((*(*FdoExtension).PrivateFdoData).HackFlags & 0x2) != 0 {
            ClassSetDeviceParameter(
                FdoExtension,
                CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                CLASSP_REG_MMC_DETECTION_VALUE_NAME.as_ptr() as PWSTR,
                ClassDetectionUnsupported as ULONG,
            );
            status = STATUS_NOT_SUPPORTED;
        } else {
            if (*Info).Gesn.Buffer.is_null() {
                (*Info).Gesn.Buffer = ExAllocatePoolWithTag(
                    NonPagedPoolNxCacheAligned,
                    GESN_BUFFER_SIZE as SIZE_T,
                    1061118803u32,
                ) as PUCHAR;
            }
            if (*Info).Gesn.Buffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
            } else {
                if !(*Info).Gesn.Mdl.is_null() {
                    IoFreeMdl((*Info).Gesn.Mdl);
                }
                (*Info).Gesn.Mdl = IoAllocateMdl(
                    (*Info).Gesn.Buffer as PVOID,
                    GESN_BUFFER_SIZE as ULONG,
                    0u8,
                    0u8,
                    core::ptr::null_mut(),
                );
                if (*Info).Gesn.Mdl.is_null() {
                    status = STATUS_INSUFFICIENT_RESOURCES;
                } else {
                    MmBuildMdlForNonPagedPool((*Info).Gesn.Mdl);
                    (*Info).Gesn.BufferSize = GESN_BUFFER_SIZE as ULONG;
                    (*Info).Gesn.EventMask = 0u8;

                    deviceDescriptor = (*FdoExtension).DeviceDescriptor;
                    atapiResets = 0;
                    retryImmediately = 1u8;

                    i = 0;
                    while i < 16 && retryImmediately != 0 {
                        irp = ClasspPrepareMcnIrp(FdoExtension, Info, 1u8);
                        if irp.is_null() {
                            status = STATUS_INSUFFICIENT_RESOURCES;
                            break;
                        }

                        if (*Info).MediaChangeSrb.Srb.as_ref().Function == (SRB_FUNCTION_STORAGE_REQUEST_BLOCK) as u8 {
                            srbFlags = (*Info).MediaChangeSrb.SrbEx.as_ref().SrbFlags;
                        } else {
                            srbFlags = (*Info).MediaChangeSrb.Srb.as_ref().SrbFlags;
                        }
                        // NT_ASSERT(TEST_FLAG(srbFlags, SRB_FLAGS_NO_QUEUE_FREEZE));
                        if (srbFlags & 0x100) == 0 {
                            DbgPrint(
                                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                                b"drivers/storage/class/classpnp/autorun.c\0"
                                    .as_ptr() as *const i8,
                                2094i32,
                                b"TEST_FLAG(srbFlags, SRB_FLAGS_NO_QUEUE_FREEZE)\0".as_ptr() as *const i8,
                            );
                            __int2c();
                        }

                        KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0u8);
                        IoSetCompletionRoutine(
                            irp,
                            Some(ClassSignalCompletion),
                            core::ptr::addr_of_mut!(event) as PVOID,
                            1u8,
                            1u8,
                            1u8,
                        );

                        status = IoCallDriver(
                            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
                            irp,
                        );

                        if status == STATUS_PENDING {
                            status = KeWaitForSingleObject(
                                core::ptr::addr_of_mut!(event) as PVOID,
                                _KWAIT_REASON_Executive,
                                (0i32) as i8,
                                0u8,
                                core::ptr::null_mut(),
                            );
                            // NT_ASSERT(NT_SUCCESS(status))
                            if !(status >= 0) {
                                DbgPrint(
                                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                                    b"drivers/storage/class/classpnp/autorun.c\0"
                                        .as_ptr() as *const i8,
                                    2114i32,
                                    b"NT_SUCCESS(status)\0".as_ptr() as *const i8,
                                );
                                __int2c();
                            }
                        }

                        ClassReleaseRemoveLock((*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject, irp);

                        // SRB_STATUS macro
                        let srbStatus = (*Info).MediaChangeSrb.Srb.as_ref().SrbStatus as i32;
                        if (srbStatus & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) != (SRB_STATUS_SUCCESS) as i32 {
                            InterpretSenseInfoWithoutHistory(
                                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                                irp,
                                core::ptr::addr_of_mut!((*Info).MediaChangeSrb.Srb) as PSCSI_REQUEST_BLOCK,
                                IRP_MJ_SCSI as u8,
                                0,
                                0,
                                &mut status,
                                core::ptr::null_mut(),
                            );
                        }

                        if (*deviceDescriptor).BusType as u32 == BusTypeAtapi as u32
                            && srbStatus == (SRB_STATUS_BUS_RESET) as i32
                        {
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
                            ClassSetDeviceParameter(
                                FdoExtension,
                                CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                                CLASSP_REG_MMC_DETECTION_VALUE_NAME.as_ptr() as PWSTR,
                                ClassDetectionUnsupported as ULONG,
                            );
                            TracePrint(
                                TRACE_LEVEL_WARNING,
                                TRACE_FLAG_MCN,
                                b"Classpnp => GESN test failed %x for fdo %p\n\0".as_ptr() as PCCHAR,
                                status,
                                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                            );
                            break;
                        }

                        if !NT_SUCCESS(status) {
                            TracePrint(
                                TRACE_LEVEL_WARNING,
                                TRACE_FLAG_MCN,
                                b"Classpnp => GESN test failed %x for fdo %p\n\0".as_ptr() as PCCHAR,
                                status,
                                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                            );
                            break;
                        }

                        if i == 0 {
                            header = (*Info).Gesn.Buffer as *mut _NOTIFICATION_EVENT_STATUS_HEADER;
                            TracePrint(
                                TRACE_LEVEL_INFORMATION,
                                TRACE_FLAG_MCN,
                                b"Classpnp => Fdo %p supports event mask %x\n\0".as_ptr() as PCCHAR,
                                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                                (*header).SupportedEventClasses as i32,
                            );

                            if ((*header).SupportedEventClasses as i32 & (NOTIFICATION_MEDIA_STATUS_CLASS_MASK) as i32) != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_MCN,
                                    b"Classpnp => GESN supports MCN\n\0".as_ptr() as PCCHAR,
                                );
                            }
                            if ((*header).SupportedEventClasses as i32 & (NOTIFICATION_DEVICE_BUSY_CLASS_MASK) as i32) != 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_MCN,
                                    b"Classpnp => GESN supports DeviceBusy\n\0".as_ptr() as PCCHAR,
                                );
                            }
                            if ((*header).SupportedEventClasses as i32 & (NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK) as i32) != 0 {
                                // TEST_FLAG(FdoExtension->PrivateFdoData->HackFlags, FDO_HACK_GESN_IGNORE_OPCHANGE)
                                if ((*(*FdoExtension).PrivateFdoData).HackFlags & 0x10) != 0 {
                                    TracePrint(
                                        TRACE_LEVEL_INFORMATION,
                                        TRACE_FLAG_MCN,
                                        b"Classpnp => GESN supports OpChange, but must ignore these events for compatibility\n\0"
                                            .as_ptr() as PCCHAR,
                                    );
                                    // CLEAR_FLAG
                                    (*header).SupportedEventClasses &= !(NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK as u8);
                                } else {
                                    TracePrint(
                                        TRACE_LEVEL_INFORMATION,
                                        TRACE_FLAG_MCN,
                                        b"Classpnp => GESN supports OpChange\n\0".as_ptr() as PCCHAR,
                                    );
                                }
                            }

                            (*Info).Gesn.EventMask = (*header).SupportedEventClasses;
                            (*Info).Gesn.EventMask &= (NOTIFICATION_OPERATIONAL_CHANGE_CLASS_MASK
                                | NOTIFICATION_EXTERNAL_REQUEST_CLASS_MASK
                                | NOTIFICATION_MEDIA_STATUS_CLASS_MASK
                                | NOTIFICATION_DEVICE_BUSY_CLASS_MASK) as u8;

                            if (*Info).Gesn.EventMask == 0 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_MCN,
                                    b"Classpnp => GESN supported, but not mask we care about (%x) for FDO %p\n\0"
                                        .as_ptr() as PCCHAR,
                                    (*header).SupportedEventClasses as i32,
                                    (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                                );
                                break;
                            } else if CountOfSetBitsUChar((*Info).Gesn.EventMask) == 1 {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_MCN,
                                    b"Classpnp => GESN hack not required for FDO %p\n\0".as_ptr() as PCCHAR,
                                    (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                                );
                            } else {
                                TracePrint(
                                    TRACE_LEVEL_INFORMATION,
                                    TRACE_FLAG_MCN,
                                    b"Classpnp => GESN hack enabled for FDO %p\n\0".as_ptr() as PCCHAR,
                                    (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                                );
                                (*Info).Gesn.HackEventMask = 1u8;
                            }
                        } else {
                            status = ClasspInterpretGesnData(
                                FdoExtension,
                                (*Info).Gesn.Buffer as *mut _NOTIFICATION_EVENT_STATUS_HEADER,
                                &mut retryImmediately,
                            );
                            if !NT_SUCCESS(status) {
                                ClassSetDeviceParameter(
                                    FdoExtension,
                                    CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                                    CLASSP_REG_MMC_DETECTION_VALUE_NAME.as_ptr() as PWSTR,
                                    ClassDetectionUnsupported as ULONG,
                                );
                                break;
                            }
                        }
                        i = i.wrapping_add(1);
                    }

                    // after loop, if we didn't break with error
                    if status == STATUS_SUCCESS || NT_SUCCESS(status) {
                        // Check if we can enable GESN
                        if ((*Info).Gesn.EventMask & (NOTIFICATION_MEDIA_STATUS_CLASS_MASK) as u8) != 0 {
                            TracePrint(
                                TRACE_LEVEL_INFORMATION,
                                TRACE_FLAG_MCN,
                                b"Classpnp => Enabling GESN support for fdo %p\n\0".as_ptr() as PCCHAR,
                                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                            );
                            (*Info).Gesn.Supported = 1u8;
                            ClassSetDeviceParameter(
                                FdoExtension,
                                CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
                                CLASSP_REG_MMC_DETECTION_VALUE_NAME.as_ptr() as PWSTR,
                                ClassDetectionSupported as ULONG,
                            );
                            return STATUS_SUCCESS;
                        }
                        TracePrint(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_MCN,
                            b"Classpnp => GESN available but not enabled for fdo %p\n\0".as_ptr() as PCCHAR,
                            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                        );
                    }
                }
            }
        }
    }

    // ExitWithError:
    TracePrint(
        TRACE_LEVEL_WARNING,
        TRACE_FLAG_MCN,
        b"Classpnp => GESN support detection failed  for fdo %p with status %08x\n\0".as_ptr() as PCCHAR,
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        status,
    );

    if !(*Info).Gesn.Mdl.is_null() {
        IoFreeMdl((*Info).Gesn.Mdl);
        (*Info).Gesn.Mdl = core::ptr::null_mut();
    }
    if !(*Info).Gesn.Buffer.is_null() {
        ExFreePool((*Info).Gesn.Buffer as PVOID);
        (*Info).Gesn.Buffer = core::ptr::null_mut();
    }
    (*Info).Gesn.Supported = 0u8;
    (*Info).Gesn.EventMask = 0u8;
    (*Info).Gesn.BufferSize = 0;
    STATUS_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn ClasspDisableGesn(
    Fdo: PDEVICE_OBJECT,
    Context: PVOID,
) {
    // PAGED_CODE expansion
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    .as_ptr() as *const i8,
                2381i32,
                b"0\0".as_ptr() as *const i8,
            );
            loop { core::hint::spin_loop(); }
        }
    }

    let fdoExtension = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let WorkItem = Context as PIO_WORKITEM;

    ClassSetDeviceParameter(
        fdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_MMC_DETECTION_VALUE_NAME.as_ptr() as PWSTR,
        ClassDetectionUnsupported as ULONG,
    );

    IoFreeWorkItem(WorkItem);
}

#[no_mangle]
pub unsafe extern "system" fn ClassInitializeTestUnitPolling(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    AllowDriveToSleep: u8,
) -> NTSTATUS {
    ClasspInitializePolling(FdoExtension, AllowDriveToSleep)
}

#[no_mangle]
pub unsafe extern "system" fn ClassInitializeMediaChangeDetection(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    EventPrefix: PUCHAR,
) {
    let fdo: PDEVICE_OBJECT = (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
    let driverExtension: PCLASS_DRIVER_EXTENSION =
        ClassGetDriverExtension((*fdo).DriverObject);
    let disabledForBadHardware: BOOLEAN;
    let mut disabled: BOOLEAN = 0;
    let mut instanceOverride: BOOLEAN = 0;

    // UNREFERENCED_PARAMETER
    _ = EventPrefix;

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                2474i32,
                b"0\0".as_ptr() as *const core::ffi::c_char,
            );
            __int2c();
        }
    }

    disabledForBadHardware = ClasspIsMediaChangeDisabledDueToHardwareLimitation(
        FdoExtension,
        core::ptr::addr_of_mut!((*driverExtension).RegistryPath),
    );
    if disabledForBadHardware != 0 {
        StorDebugPrint(
            4u32,
            TRACE_FLAG_MCN,
            b"ClassInitializeMCN: Disabled due to hardwarelimitations for this device\0"
                .as_ptr() as PCCHAR,
        );
        return;
    }

    disabled = ClasspIsMediaChangeDisabledForClass(
        FdoExtension,
        core::ptr::addr_of_mut!((*driverExtension).RegistryPath),
    );
    StorDebugPrint(
        4u32,
        TRACE_FLAG_MCN,
        b"ClassInitializeMCN: Class    MCN is %s\n\0".as_ptr() as PCCHAR,
        if disabled != 0 {
            b"disabled\0".as_ptr() as PCCHAR
        } else {
            b"enabled\0".as_ptr() as PCCHAR
        },
    );

    let status: NTSTATUS =
        ClasspMediaChangeDeviceInstanceOverride(FdoExtension, &raw mut instanceOverride);
    if !(status >= 0) {
        StorDebugPrint(
            4u32,
            TRACE_FLAG_MCN,
            b"ClassInitializeMCN: Instance using default\n\0".as_ptr() as PCCHAR,
        );
    } else {
        StorDebugPrint(
            4u32,
            TRACE_FLAG_MCN,
            b"ClassInitializeMCN: Instance override: %s MCN\n\0".as_ptr() as PCCHAR,
            if instanceOverride != 0 {
                b"Enabling\0".as_ptr() as PCCHAR
            } else {
                b"Disabling\0".as_ptr() as PCCHAR
            },
        );
        disabled = (instanceOverride == 0) as BOOLEAN;
    }

    StorDebugPrint(
        4u32,
        TRACE_FLAG_MCN,
        b"ClassInitializeMCN: Instance MCN is %s\n\0".as_ptr() as PCCHAR,
        if disabled != 0 {
            b"disabled\0".as_ptr() as PCCHAR
        } else {
            b"enabled\0".as_ptr() as PCCHAR
        },
    );

    if disabled != 0 {
        return;
    }

    ClasspInitializePolling(FdoExtension, 0u8);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspMediaChangeDeviceInstanceOverride(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Enabled: PBOOLEAN,
) -> NTSTATUS {
    let mut deviceParameterHandle: HANDLE = core::ptr::null_mut();
    let mut driverParameterHandle: HANDLE = core::ptr::null_mut();
    let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 3] = [RTL_QUERY_REGISTRY_TABLE {
        QueryRoutine: None,
        Flags: 0,
        Name: core::ptr::null(),
        EntryContext: core::ptr::null_mut(),
        DefaultType: 0,
        DefaultData: core::ptr::null_mut(),
        DefaultLength: 0,
    }; 3];
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut subkeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut alwaysEnable: ULONG = 0;
    let mut alwaysDisable: ULONG = 0;
    let mut i: ULONG;

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/autorun.c\0"
                .as_ptr() as PCSTR,
            2578i32,
            b"0\0".as_ptr() as PCSTR,
        );
        __int2c();
    }

    // --- main logic ---
    status = IoOpenDeviceRegistryKey(
        (*FdoExtension).LowerPdo,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_ALL_ACCESS,
        core::ptr::addr_of_mut!(deviceParameterHandle),
    );
    if !NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassMediaChangeDeviceInstanceDisabled: Could not open device registry key [%lx]\n\0"
                .as_ptr() as PCCHAR,
            status,
        );
        // No LEAVE, we continue to cleanup but skip the inner block
    } else {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(subkeyName),
            MCN_REG_SUBKEY_NAME.as_ptr() as PCWSTR,
        );
        // Inline InitializeObjectAttributes
        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = deviceParameterHandle;
        objectAttributes.ObjectName = core::ptr::addr_of_mut!(subkeyName);
        objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwCreateKey(
            core::ptr::addr_of_mut!(driverParameterHandle),
            KEY_READ,
            core::ptr::addr_of_mut!(objectAttributes),
            0,
            core::ptr::null_mut(),
            0, // REG_OPTION_NON_VOLATILE
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_MCN,
                b"ClassMediaChangeDeviceInstanceDisabled: subkey could not be created. %lx\n\0"
                    .as_ptr() as PCCHAR,
                status,
            );
        } else {
            i = 0;
            while i < 2 {
                core::ptr::write_bytes(
                    (&raw mut queryTable) as *mut u8,
                    0,
                    core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 3]>(),
                );
                queryTable[0].Flags =
                    (RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK) as ULONG;
                queryTable[0].DefaultType =
                    ((REG_DWORD << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | REG_NONE) as ULONG;
                queryTable[0].DefaultLength = 0;
                if i == 0 {
                    queryTable[0].Name =
                        MCN_REG_AUTORUN_DISABLE_INSTANCE_NAME.as_ptr() as PCWSTR;
                    queryTable[0].EntryContext =
                        core::ptr::addr_of_mut!(alwaysDisable) as PVOID;
                    queryTable[0].DefaultData =
                        core::ptr::addr_of_mut!(alwaysDisable) as PVOID;
                } else {
                    queryTable[0].Name =
                        MCN_REG_AUTORUN_ENABLE_INSTANCE_NAME.as_ptr() as PCWSTR;
                    queryTable[0].EntryContext =
                        core::ptr::addr_of_mut!(alwaysEnable) as PVOID;
                    queryTable[0].DefaultData =
                        core::ptr::addr_of_mut!(alwaysEnable) as PVOID;
                }
                RtlQueryRegistryValues(
                    RTL_REGISTRY_HANDLE,
                    driverParameterHandle as PWSTR,
                    queryTable.as_mut_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                i += 1;
            }
        }
    }

    // Cleanup
    if !driverParameterHandle.is_null() {
        ZwClose(driverParameterHandle);
    }
    if !deviceParameterHandle.is_null() {
        ZwClose(deviceParameterHandle);
    }

    // Determine result
    if alwaysEnable != 0 && alwaysDisable != 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassMediaChangeDeviceInstanceDisabled: %s selected\n\0".as_ptr() as PCCHAR,
            b"Both Enable and Disable set -- DISABLE\0".as_ptr() as PCCHAR,
        );
        if !NT_SUCCESS(status) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    .as_ptr() as PCSTR,
                2667i32,
                b"NT_SUCCESS(status)\0".as_ptr() as PCSTR,
            );
            __int2c();
        }
        *Enabled = 0u8;
        STATUS_SUCCESS
    } else if alwaysDisable != 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassMediaChangeDeviceInstanceDisabled: %s selected\n\0".as_ptr() as PCCHAR,
            b"DISABLE\0".as_ptr() as PCCHAR,
        );
        if !NT_SUCCESS(status) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    .as_ptr() as PCSTR,
                2676i32,
                b"NT_SUCCESS(status)\0".as_ptr() as PCSTR,
            );
            __int2c();
        }
        *Enabled = 0u8;
        STATUS_SUCCESS
    } else if alwaysEnable != 0 {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassMediaChangeDeviceInstanceDisabled: %s selected\n\0".as_ptr() as PCCHAR,
            b"ENABLE\0".as_ptr() as PCCHAR,
        );
        if !NT_SUCCESS(status) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    .as_ptr() as PCSTR,
                2685i32,
                b"NT_SUCCESS(status)\0".as_ptr() as PCSTR,
            );
            __int2c();
        }
        *Enabled = 1u8;
        STATUS_SUCCESS
    } else {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassMediaChangeDeviceInstanceDisabled: %s selected\n\0".as_ptr() as PCCHAR,
            b"DEFAULT\0".as_ptr() as PCCHAR,
        );
        STATUS_UNSUCCESSFUL
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspIsMediaChangeDisabledDueToHardwareLimitation(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    RegistryPath: PUNICODE_STRING,
) -> BOOLEAN {
    let deviceDescriptor = (*FdoExtension).DeviceDescriptor;
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    objectAttributes.ObjectName = RegistryPath;
    objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut serviceKey: *mut core::ffi::c_void = core::ptr::null_mut();
    let status = ZwOpenKey(&raw mut serviceKey, KEY_READ, &raw mut objectAttributes);

    if !NT_SUCCESS(status) {
        return 1u8;
    }

    let mut mediaChangeNotificationDisabled: u32 = 0;
    let mut deviceUnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut deviceString: ANSI_STRING = core::mem::zeroed();

    let _ = loop {
        if (*deviceDescriptor).VendorIdOffset == 0 && (*deviceDescriptor).ProductIdOffset == 0 {
            break;
        }

        let vendorId = if (*deviceDescriptor).VendorIdOffset != 0 {
            (deviceDescriptor as *const u8).add((*deviceDescriptor).VendorIdOffset as usize) as *mut u8
        } else {
            core::ptr::null_mut()
        };
        let productId = if (*deviceDescriptor).ProductIdOffset != 0 {
            (deviceDescriptor as *const u8).add((*deviceDescriptor).ProductIdOffset as usize) as *mut u8
        } else {
            core::ptr::null_mut()
        };
        let revisionId = if (*deviceDescriptor).ProductRevisionOffset != 0 {
            (deviceDescriptor as *const u8).add((*deviceDescriptor).ProductRevisionOffset as usize) as *mut u8
        } else {
            core::ptr::null_mut()
        };

        let mut length: u32 = 0;
        if !vendorId.is_null() {
            length = strlen(vendorId as *const i8) as u32;
        }
        if !productId.is_null() {
            length += strlen(productId as *const i8) as u32;
        }
        if !revisionId.is_null() {
            length += strlen(revisionId as *const i8) as u32;
        }

        deviceString.Length = length as u16;
        deviceString.MaximumLength = deviceString.Length + 1;
        deviceString.Buffer = ExAllocatePoolWithTag(
            0u32,
            deviceString.MaximumLength as usize,
            CLASS_TAG_AUTORUN_DISABLE,
        ) as *mut i8;
        if deviceString.Buffer.is_null() {
            TracePrint(
                4u32,
                TRACE_FLAG_MCN,
                b"ClassMediaChangeDisabledForHardware: Unable to alloc string buffer\n\0".as_ptr() as PCCHAR,
            );
            break;
        }

        let mut offset: u32 = 0;
        if !vendorId.is_null() {
            let len = strlen(vendorId as *const i8);
            core::ptr::copy_nonoverlapping(
                vendorId as *const u8,
                deviceString.Buffer.add(offset as usize) as *mut u8,
                len,
            );
            offset += len as u32;
        }
        if !productId.is_null() {
            let len = strlen(productId as *const i8);
            core::ptr::copy_nonoverlapping(
                productId as *const u8,
                deviceString.Buffer.add(offset as usize) as *mut u8,
                len,
            );
            offset += len as u32;
        }
        if !revisionId.is_null() {
            let len = strlen(revisionId as *const i8);
            core::ptr::copy_nonoverlapping(
                revisionId as *const u8,
                deviceString.Buffer.add(offset as usize) as *mut u8,
                len,
            );
            offset += len as u32;
        }

        *deviceString.Buffer.add(deviceString.Length as usize) = 0i8;

        let status = RtlAnsiStringToUnicodeString(
            &raw mut deviceUnicodeString,
            &raw mut deviceString,
            1u8,
        );
        if !NT_SUCCESS(status) {
            TracePrint(
                4u32,
                TRACE_FLAG_MCN,
                b"ClassMediaChangeDisabledForHardware: cannot convert to unicode %lx\n\0".as_ptr() as PCCHAR,
                status,
            );
            break;
        }

        const AUTORUN_DISABLE_NAME: [u16; 21] = [
            0x0041, 0x0075, 0x0074, 0x006F, 0x0052, 0x0075, 0x006E, 0x0041, 0x006C, 0x0077,
            0x0061, 0x0079, 0x0073, 0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0000,
        ];
        let nullMultiSz: PWSTR = [0u16, 0u16].as_ptr() as PWSTR;

        // NTAPI (stdcall) callback — reference the stdcall symbol that class_part_d defines.
        extern "system" {
            fn ClasspMediaChangeRegistryCallBack(
                ValueName: PUNICODE_STRING, ValueType: ULONG, ValueData: PVOID,
                ValueLength: ULONG, Context: PVOID, EntryContext: PVOID,
            ) -> NTSTATUS;
        }
        let mut parameters: [_RTL_QUERY_REGISTRY_TABLE; 2] = core::mem::zeroed();
        parameters[0].QueryRoutine = Some(core::mem::transmute::<unsafe extern "system" fn(PUNICODE_STRING, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
            _,
        >(ClasspMediaChangeRegistryCallBack));
        parameters[0].Flags = RTL_QUERY_REGISTRY_REQUIRED;
        parameters[0].Name = AUTORUN_DISABLE_NAME.as_ptr() as PCWSTR;
        parameters[0].EntryContext = &raw mut mediaChangeNotificationDisabled as *mut core::ffi::c_void;
        parameters[0].DefaultType = REG_MULTI_SZ;
        parameters[0].DefaultData = nullMultiSz as *mut core::ffi::c_void;
        parameters[0].DefaultLength = 0;

        let status = RtlQueryRegistryValues(
            RTL_REGISTRY_HANDLE,
            serviceKey as PCWSTR,
            &raw mut parameters as *mut _,
            &raw mut deviceUnicodeString as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(status) {
            // nothing, just break to cleanup
        }
        break;
    };

    if !deviceString.Buffer.is_null() {
        ExFreePool(deviceString.Buffer as *mut core::ffi::c_void);
    }
    if !deviceUnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&raw mut deviceUnicodeString);
    }
    ZwClose(serviceKey);

    if mediaChangeNotificationDisabled != 0 {
        TracePrint(
            4u32,
            TRACE_FLAG_MCN,
            b"ClassMediaChangeDisabledForHardware: Device is on disable list\n\0".as_ptr() as PCCHAR,
        );
        return 1u8;
    }
    0u8
}
#[no_mangle]
pub unsafe extern "C" fn ClasspIsMediaChangeDisabledForClass(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    RegistryPath: PUNICODE_STRING,
) -> BOOLEAN {
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut serviceKey: HANDLE = core::ptr::null_mut();
    let mut parametersKey: HANDLE = core::ptr::null_mut();
    let mut parameters: [RTL_QUERY_REGISTRY_TABLE; 3] = core::mem::zeroed();
    let mut paramStr: _UNICODE_STRING = core::mem::zeroed();
    let mut mcnRegistryValue: ULONG = 1;
    let mut status: NTSTATUS;

    // PAGED_CODE()
    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCCHAR,
            KeGetCurrentIrql() as i32,
        );
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR,
            b"autorun.c\0".as_ptr() as PCCHAR,
            2977,
            b"0\0".as_ptr() as PCCHAR,
        );
        __int2c();
    }

    // InitializeObjectAttributes(&objectAttributes, RegistryPath, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL)
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    objectAttributes.RootDirectory = core::ptr::null_mut::<c_void>() as HANDLE;
    objectAttributes.ObjectName = RegistryPath;
    objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut::<c_void>() as PVOID;
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut::<c_void>() as PVOID;

    status = ZwOpenKey(
        &raw mut serviceKey,
        KEY_READ as ACCESS_MASK,
        &raw mut objectAttributes,
    );

    // NT_ASSERT(NT_SUCCESS(status))
    if !(status >= 0) {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR,
            b"autorun.c\0".as_ptr() as PCCHAR,
            2993,
            b"NT_SUCCESS(status)\0".as_ptr() as PCCHAR,
        );
        __int2c();
    }

    if !(status >= 0) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassCheckServiceMCN: Defaulting to %s\n\0".as_ptr() as PCCHAR,
            if mcnRegistryValue != 0 {
                b"Enabled\0".as_ptr() as PCCHAR
            } else {
                b"Disabled\0".as_ptr() as PCCHAR
            },
        );
        return (mcnRegistryValue == 0) as BOOLEAN;
    }

    // RtlInitUnicodeString(&paramStr, L"Parameters")
    let param_str: [u16; 11] = [
        0x0050, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000,
    ];
    RtlInitUnicodeString(&raw mut paramStr, param_str.as_ptr() as PCWSTR);

    // InitializeObjectAttributes again for Parameters subkey
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    objectAttributes.RootDirectory = serviceKey;
    objectAttributes.ObjectName = &raw mut paramStr as PUNICODE_STRING;
    objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut::<c_void>() as PVOID;
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut::<c_void>() as PVOID;

    status = ZwOpenKey(
        &raw mut parametersKey,
        KEY_READ as ACCESS_MASK,
        &raw mut objectAttributes,
    );

    if !(status >= 0) {
        parametersKey = core::ptr::null_mut();
    }

    // Set up query table entry for "Autorun"
    let autorun_str: [u16; 8] = [0x0041, 0x0075, 0x0074, 0x006F, 0x0072, 0x0075, 0x006E, 0x0000];
    parameters[0].Flags = RTL_QUERY_REGISTRY_DIRECT as ULONG;
    parameters[0].Name = autorun_str.as_ptr() as PCWSTR;
    parameters[0].EntryContext = &raw mut mcnRegistryValue as PVOID;
    parameters[0].DefaultType = ((REG_DWORD as ULONG) << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | (REG_DWORD as ULONG);
    parameters[0].DefaultData = &raw mut mcnRegistryValue as PVOID;
    parameters[0].DefaultLength = core::mem::size_of::<ULONG>() as ULONG;

    RtlQueryRegistryValues(
        (RTL_REGISTRY_HANDLE | RTL_REGISTRY_OPTIONAL) as ULONG,
        serviceKey as PCWSTR,
        &raw mut parameters as PRTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_MCN,
        b"ClassCheckServiceMCN: <Service>/Autorun flag = %d\n\0".as_ptr() as PCCHAR,
        mcnRegistryValue,
    );

    if !parametersKey.is_null() {
        RtlQueryRegistryValues(
            (RTL_REGISTRY_HANDLE | RTL_REGISTRY_OPTIONAL) as ULONG,
            parametersKey as PCWSTR,
            &raw mut parameters as PRTL_QUERY_REGISTRY_TABLE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClassCheckServiceMCN: <Service>/Parameters/Autorun flag = %d\n\0".as_ptr() as PCCHAR,
            mcnRegistryValue,
        );
        ZwClose(parametersKey);
    }

    ZwClose(serviceKey);

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_MCN,
        b"ClassCheckServiceMCN: Autoplay for device %p is %s\n\0".as_ptr() as PCCHAR,
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        if mcnRegistryValue != 0 {
            b"on\0".as_ptr() as PCCHAR
        } else {
            b"off\0".as_ptr() as PCCHAR
        },
    );

    (mcnRegistryValue == 0) as BOOLEAN
}
#[no_mangle]
pub unsafe extern "system" fn ClassEnableMediaChangeDetection(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    let info: PMEDIA_CHANGE_DETECTION_INFO = (*FdoExtension).MediaChangeDetectionInfo;
    let mut oldCount: i32;

    // PAGED_CODE() expansion as in c2rust anchor
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCCHAR,
            (KeGetCurrentIrql()) as i32,
        );
        // NT_ASSERT(FALSE)
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR,
            b"drivers/storage/class/classpnp/autorun.c\0".as_ptr() as PCCHAR,
            3109i32,
            b"0\0".as_ptr() as PCCHAR,
        );
        __int2c();
    }

    if info.is_null() {
        StorDebugPrint(
            4u32,
            TRACE_FLAG_MCN,
            b"ClassEnableMediaChangeDetection: not initialized\n\0".as_ptr() as PCCHAR,
        );
        return;
    }

    KeWaitForSingleObject(
        &raw mut (*info).MediaChangeMutex as PVOID,
        _KWAIT_REASON_UserRequest,
        0i32 as KPROCESSOR_MODE,
        0u8, // FALSE
        core::ptr::null_mut::<LARGE_INTEGER>(),
    );

    (*info).MediaChangeDetectionDisableCount -= 1;
    oldCount = (*info).MediaChangeDetectionDisableCount;

    // NT_ASSERT(oldCount >= 0)
    if oldCount < 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCCHAR,
            b"drivers/storage/class/classpnp/autorun.c\0".as_ptr() as PCCHAR,
            3125i32,
            b"oldCount >= 0\0".as_ptr() as PCCHAR,
        );
        __int2c();
    }

    StorDebugPrint(
        4u32,
        TRACE_FLAG_MCN,
        b"ClassEnableMediaChangeDetection: Disable count reduced to %d - \0".as_ptr() as PCCHAR,
        (*info).MediaChangeDetectionDisableCount,
    );

    if oldCount == 0 {
        ClasspInternalSetMediaChangeState(FdoExtension, MediaUnknown, 0u8);
        ClassResetMediaChangeTimer(FdoExtension);
        StorDebugPrint(
            4u32,
            TRACE_FLAG_MCN,
            b"MCD is enabled\n\0".as_ptr() as PCCHAR,
        );
    } else {
        StorDebugPrint(
            4u32,
            TRACE_FLAG_MCN,
            b"MCD still disabled\n\0".as_ptr() as PCCHAR,
        );
    }

    KeReleaseMutex(&raw mut (*info).MediaChangeMutex, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn ClassDisableMediaChangeDetection(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    let info = (*FdoExtension).MediaChangeDetectionInfo as PMEDIA_CHANGE_DETECTION_INFO;

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8 as *const i8,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    .as_ptr() as *const u8 as *const i8,
                3194i32,
                b"0\0".as_ptr() as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    if info.is_null() {
        return;
    }

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex) as *mut core::ffi::c_void,
        _KWAIT_REASON_UserRequest as KWAIT_REASON,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut::<LARGE_INTEGER>(),
    );

    (*info).MediaChangeDetectionDisableCount += 1;

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_MCN,
        b"ClassDisableMediaChangeDetection: disable count is %d\n\0".as_ptr() as PCCHAR,
        (*info).MediaChangeDetectionDisableCount,
    );

    KeReleaseMutex(
        core::ptr::addr_of_mut!((*info).MediaChangeMutex),
        0u8,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ClassCleanupMediaChangeDetection(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) {
    let mut info: PMEDIA_CHANGE_DETECTION_INFO = (*FdoExtension).MediaChangeDetectionInfo;

    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const ::core::ffi::c_char,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const ::core::ffi::c_char,
                b"drivers/storage/class/classpnp/autorun.c\0".as_ptr() as *const ::core::ffi::c_char,
                3241i32,
                b"0\0".as_ptr() as *const ::core::ffi::c_char,
            );
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] core::arch::asm!("int 0x2c");
            #[cfg(target_arch = "aarch64")] core::arch::asm!("brk #0");
        }
    }

    if info.is_null() {
        return;
    }

    (*FdoExtension).MediaChangeDetectionInfo = core::ptr::null_mut();

    if !(*info).Gesn.Mdl.is_null() {
        IoFreeMdl((*info).Gesn.Mdl);
    }
    if !(*info).Gesn.Buffer.is_null() {
        ExFreePool((*info).Gesn.Buffer as PVOID);
        (*info).Gesn.Buffer = core::ptr::null_mut::<u8>();
    }
    IoFreeIrp((*info).MediaChangeIrp);
    if !(*info).SenseBuffer.is_null() {
        ExFreePool((*info).SenseBuffer as PVOID);
        (*info).SenseBuffer = core::ptr::null_mut::<u8>();
    }
    if !info.is_null() {
        ExFreePool(info as PVOID);
        info = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspMcnControl(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Irp: PIRP,
    Srb: PSCSI_REQUEST_BLOCK,
) -> NTSTATUS {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let request: PPREVENT_MEDIA_REMOVAL =
        (*Irp).AssociatedIrp.SystemBuffer as PPREVENT_MEDIA_REMOVAL;
    let fileObject: PFILE_OBJECT = (*irpStack).FileObject;
    let mut fsContext: PFILE_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE expanded
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            (KeGetCurrentIrql()) as i32,
        );
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/autorun.c\0"
                .as_ptr() as *const i8,
            3293i32,
            b"0\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    if !fileObject.is_null() {
        fsContext = ClassGetFsContext(commonExtension, fileObject);
    } else if (*Irp).RequestorMode == (0i32) as i8 {
        fsContext = &raw mut (*FdoExtension).KernelModeMcnContext as PFILE_OBJECT_EXTENSION;
    }

    if fsContext.is_null() {
        status = STATUS_INVALID_PARAMETER;
    } else {
        let fsCtxClasspnp = fsContext as *mut _FILE_OBJECT_EXTENSION;
        if (*request).PreventMediaRemoval != 0 {
            ClassDisableMediaChangeDetection(FdoExtension);
            InterlockedIncrement(&raw mut (*fsCtxClasspnp).McnDisableCount as *mut i32);
        } else if (*fsCtxClasspnp).McnDisableCount == 0u32 {
            status = STATUS_INVALID_DEVICE_STATE;
        } else {
            InterlockedDecrement(&raw mut (*fsCtxClasspnp).McnDisableCount as *mut i32);
            ClassEnableMediaChangeDetection(FdoExtension);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;

    ExFreePool(Srb as *mut core::ffi::c_void);
    let _ = Srb; // Srb is set to null in cleanup; we just dealloc

    ClassReleaseRemoveLock((*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject, Irp);
    ClassCompleteRequest(
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        Irp,
        (IO_NO_INCREMENT) as _,
    );

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClasspTimerTickEx(Timer: PEX_TIMER, Context: PVOID) {
    let mut dummyDpc: KDPC = core::mem::zeroed();
    let _ = Timer;
    ClasspTimerTick(
        &mut dummyDpc as *mut KDPC,
        Context,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn ClasspTimerTick(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let fdo_extension = DeferredContext as PFUNCTIONAL_DEVICE_EXTENSION;
    let _ = (Dpc, SystemArgument1, SystemArgument2);

    ntkrnl_sys::ASSERT!(!fdo_extension.is_null());

    let common_extension = core::ptr::addr_of_mut!((*fdo_extension).__bindgen_anon_1.CommonExtension);
    let device_object = (*fdo_extension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
    ntkrnl_sys::ASSERT!((*common_extension).as_ref().__bindgen_anon_1.IsFdo() != 0u8);

    let is_removed = ClassAcquireRemoveLockR(device_object, ClasspTimerTick as *mut core::ffi::c_void as PIRP);

    ntkrnl_sys::ASSERT!(is_removed != REMOVE_COMPLETE);

    if is_removed == 0 {
        let info = (*fdo_extension).FailurePredictionInfo;

        if !(*fdo_extension).MediaChangeDetectionInfo.is_null()
            && (*(*fdo_extension).FunctionSupportInfo).AsynchronousNotificationSupported == 0u8
        {
            ClassCheckMediaState(fdo_extension);
        }

        if !info.is_null() {
            if (*info).Method != FailurePredictionNone {
                if ClasspCanSendPollingIrp(fdo_extension) != 0 {
                    let count_down = InterlockedDecrement(core::ptr::addr_of_mut!((*info).CountDown) as *mut LONG) as ULONG;
                    if count_down == 0 {
                        if (*info).WorkQueueItem.is_null() {
                            let work_item = IoAllocateWorkItem((*fdo_extension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject);
                            (*info).WorkQueueItem = work_item;
                            if work_item.is_null() {
                                InterlockedExchange(core::ptr::addr_of_mut!((*info).CountDown) as *mut LONG, 60);
                            } else {
                                ClassAcquireRemoveLock(
                                    (*fdo_extension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                                    work_item as *mut core::ffi::c_void as PIRP,
                                );
                                IoQueueWorkItem(
                                    work_item,
                                    Some(core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PVOID), _>(ClasspFailurePredict)),
                                    DelayedWorkQueue,
                                    info as PVOID,
                                );
                            }
                        }
                    }
                }
            }
        }

        let driver_ext = (*common_extension).as_ref().DriverExtension;
        if !driver_ext.is_null() {
            let class_tick = (*driver_ext).InitData.ClassTick;
            if let Some(tick_fn) = class_tick {
                tick_fn(device_object);
            }
        }
    }

    ClassReleaseRemoveLock(device_object, ClasspTimerTick as *mut core::ffi::c_void as PIRP);
}
#[no_mangle]
pub unsafe extern "C" fn ClasspUpdateTimerNoWakeTolerance(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> u8 {
    let mut fdoData: *mut _CLASS_PRIVATE_FDO_DATA = core::ptr::null_mut();

    if (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0 {
        fdoData = (*FdoExtension).PrivateFdoData;
    }

    // NEEDS SHARED SUBSTRATE: TickTimer is KTIMER in types_i386 but should be PEX_TIMER (pointer).
    // Reinterpret first 4 bytes of TickTimer field as a PVOID.
    if !fdoData.is_null()
        && !(*(core::ptr::addr_of!((*fdoData).TickTimer) as *const PVOID)).is_null()
        && (*fdoData).TimerInitialized != 0
        && (*fdoData).TickTimerEnabled != 0
    {
        let mut noWakeTolerance: i64 = TICK_TIMER_DELAY_IN_MSEC as i64 * (10 * 1000i64);

        let class_tick = (*(*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverExtension).InitData.ClassTick;
        if ClasspScreenOff != 0u8
            && class_tick.is_none()
            && (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).DeviceType == FILE_DEVICE_DISK
        {
            noWakeTolerance = EX_TIMER_UNLIMITED_TOLERANCE;
        }

        // NEEDS SHARED SUBSTRATE: CurrentNoWakeTolerance field missing from types_i386._CLASS_PRIVATE_FDO_DATA
        if LOCAL_CURRENT_NO_WAKE_TOLERANCE != noWakeTolerance {
            let mut parameters: EXT_SET_PARAMETERS = core::mem::zeroed();
            let period: i64 = TICK_TIMER_PERIOD_IN_MSEC as i64 * (10 * 1000i64);
            let dueTime: i64 = period * -1;

            ExInitializeSetTimerParameters(&mut parameters as *mut EXT_SET_PARAMETERS as PVOID);
            parameters.NoWakeTolerance = noWakeTolerance;
            LOCAL_CURRENT_NO_WAKE_TOLERANCE = noWakeTolerance;

            ExSetTimer(
                *(core::ptr::addr_of!((*fdoData).TickTimer) as *const PVOID),
                dueTime,
                period,
                &mut parameters as *mut EXT_SET_PARAMETERS as PVOID,
            );

            return 1u8;
        }
    }

    0u8
}
#[no_mangle]
pub unsafe extern "C" fn ClasspInitializeTimer(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut fdoData: PCLASS_PRIVATE_FDO_DATA = core::ptr::null_mut();

    if (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0 {
        fdoData = (*FdoExtension).PrivateFdoData;
    }

    if fdoData.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if (*fdoData).TimerInitialized == 0u8 {
        KeInitializeDpc(
            core::ptr::addr_of_mut!((*fdoData).TickTimerDpc),
            Some(core::mem::transmute::<unsafe extern "system" fn(*mut ntkrnl_sys::ke::KDPC, PVOID, PVOID, PVOID), _>(ClasspTimerTick)),
            FdoExtension as PVOID,
        );
        KeInitializeTimer(core::ptr::addr_of_mut!((*fdoData).TickTimer));
        (*fdoData).TimerInitialized = 1u8;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDeleteTimer(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) {
    let mut fdoData: PCLASS_PRIVATE_FDO_DATA = core::ptr::null_mut();
    if (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0u8 {
        fdoData = (*FdoExtension).PrivateFdoData;
        if !fdoData.is_null() {
            // autorun.c: the EX_TIMER delete is NTDDI_WINBLUE-only, and classpnp builds at WIN8,
            // where TickTimer is an inline KTIMER that ClasspInitializeTimer KeInitializeTimer'd.
            // Ungated, this read the dispatcher header as a pointer and then zeroed it.
            if cfg!(ntddi_ge_ntddi_winblue) {
                let tick_ptr = *(core::ptr::addr_of!((*fdoData).TickTimer) as *const PVOID);
                if !tick_ptr.is_null() {
                    let mut parameters: EXT_DELETE_PARAMETERS = core::mem::zeroed();
                    ExInitializeDeleteTimerParameters(&mut parameters as *mut EXT_DELETE_PARAMETERS as PVOID);
                    ExDeleteTimer(
                        tick_ptr,
                        1u8,  // TRUE
                        0u8,  // FALSE
                        &mut parameters as *mut EXT_DELETE_PARAMETERS as PVOID,
                    );
                    *(core::ptr::addr_of_mut!((*fdoData).TickTimer) as *mut PVOID) = core::ptr::null_mut();
                }
            }
            (*fdoData).TimerInitialized = 0u8;
            (*fdoData).TickTimerEnabled = 0u8;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspEnableTimer(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) {
    let mut fdoData: PCLASS_PRIVATE_FDO_DATA = core::ptr::null_mut();
    if (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0 {
        fdoData = (*FdoExtension).PrivateFdoData;
    }
    if !fdoData.is_null() {
        ntkrnl_sys::ASSERT!((*fdoData).TimerInitialized != 0);
        if (*fdoData).TimerInitialized == 0 {
            let status = ClasspInitializeTimer(FdoExtension);
            if status < 0 {
                return;
            }
        }
        let timeout = LARGE_INTEGER {
            QuadPart: -10000000i64,
        };
        KeSetCoalescableTimer(
            core::ptr::addr_of_mut!((*fdoData).TickTimer),
            timeout,
            1000u32,
            1000u32,
            (core::ptr::addr_of_mut!((*fdoData).TickTimerDpc)).cast(),
        );
        (*fdoData).TickTimerEnabled = 1u8;
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClasspEnableTimer: Periodic tick timer enabled for device %p\n\0".as_ptr() as PCCHAR,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject as isize,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspDisableTimer(FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION) {
    let mut fdoData: *mut _CLASS_PRIVATE_FDO_DATA = core::ptr::null_mut();
    if (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() != 0u8 {
        fdoData = (*FdoExtension).PrivateFdoData;
    }
    if !fdoData.is_null() && (*fdoData).TimerInitialized == 1u8 {
        KeCancelTimer(core::ptr::addr_of_mut!((*fdoData).TickTimer));
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_MCN,
            b"ClasspDisableTimer: Periodic tick timer disabled for device %p\n\0".as_ptr() as PCCHAR,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
        (*fdoData).TickTimerEnabled = 0u8;
    } else {
        TracePrint(
            TRACE_LEVEL_WARNING,
            TRACE_FLAG_MCN,
            b"ClasspDisableTimer: Timer never initialized\n\0".as_ptr() as PCCHAR,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClasspFailurePredict(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let Info = Context as PFAILURE_PREDICTION_INFO;

    if Info.is_null() {
        NT_ASSERT!(!Info.is_null() as u8);
        return;
    }

    let mut checkFailure: STORAGE_PREDICT_FAILURE = core::mem::zeroed();
    let mut scsiAddress: SCSI_ADDRESS = core::mem::zeroed();
    RtlZeroMemory(
        core::ptr::addr_of_mut!(checkFailure) as PVOID,
        (core::mem::size_of::<STORAGE_PREDICT_FAILURE>() as ULONG) as usize,
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!(scsiAddress) as PVOID,
        (core::mem::size_of::<SCSI_ADDRESS>() as ULONG) as usize,
    );

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_WMI,
        b"ClasspFailurePredict: Polling for failure\n\0".as_ptr() as PCCHAR,
    );

    InterlockedExchange(
        core::ptr::addr_of_mut!((*Info).CountDown) as *mut LONG,
        (*Info).Period as LONG,
    );
    let workItem = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*Info).WorkQueueItem) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    ) as PIO_WORKITEM;

    if ClasspCanSendPollingIrp(fdoExtension) != 0 {
        let mut event: KEVENT = core::mem::zeroed();
        let mut topOfStack: PDEVICE_OBJECT = core::ptr::null_mut();
        let mut irp: PIRP = core::ptr::null_mut();
        let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut status: NTSTATUS;
        let activateStatus: NTSTATUS;

        if (*(*fdoExtension).FunctionSupportInfo).IdlePower.IdlePowerEnabled() != 0 {
            activateStatus = ClasspPowerActivateDevice(DeviceObject);
        } else {
            activateStatus = STATUS_UNSUCCESSFUL;
        }

        KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, FALSE);

        topOfStack = IoGetAttachedDeviceReference(DeviceObject);

        irp = IoBuildDeviceIoControlRequest(
            IOCTL_STORAGE_PREDICT_FAILURE as ULONG,
            topOfStack,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(checkFailure) as PVOID,
            core::mem::size_of::<STORAGE_PREDICT_FAILURE>() as ULONG,
            FALSE,
            core::ptr::addr_of_mut!(event),
            &mut ioStatus,
        );

        if !irp.is_null() {
            status = IoCallDriver(topOfStack, irp);
            if status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(event) as PVOID,
                    _KWAIT_REASON_Executive,
                    (0i32) as i8,
                    FALSE,
                    core::ptr::null_mut(),
                );
                status = ioStatus.__bindgen_anon_1.Status;
            }
        } else {
            status = STATUS_INSUFFICIENT_RESOURCES;
        }

        if NT_SUCCESS(status) && checkFailure.PredictFailure != 0 {
            checkFailure.PredictFailure = 512;

            KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, FALSE);
            RtlZeroMemory(
                core::ptr::addr_of_mut!(scsiAddress) as PVOID,
                (core::mem::size_of::<SCSI_ADDRESS>() as ULONG) as usize,
            );

            irp = IoBuildDeviceIoControlRequest(
                IOCTL_SCSI_GET_ADDRESS as ULONG,
                topOfStack,
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(scsiAddress) as PVOID,
                core::mem::size_of::<SCSI_ADDRESS>() as ULONG,
                FALSE,
                core::ptr::addr_of_mut!(event),
                &mut ioStatus,
            );

            if !irp.is_null() {
                status = IoCallDriver(topOfStack, irp);
                if status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of_mut!(event) as PVOID,
                        _KWAIT_REASON_Executive,
                        (0i32) as i8,
                        FALSE,
                        core::ptr::null_mut(),
                    );
                }
            }

            ClassNotifyFailurePredicted(
                fdoExtension,
                core::ptr::addr_of_mut!(checkFailure) as PUCHAR,
                core::mem::size_of::<STORAGE_PREDICT_FAILURE>() as ULONG,
                ((*fdoExtension).FailurePredicted == FALSE) as BOOLEAN,
                2,
                scsiAddress.PathId,
                scsiAddress.TargetId,
                scsiAddress.Lun,
            );

            (*fdoExtension).FailurePredicted = TRUE;
        }

        ObDereferenceObject(topOfStack as PVOID);

        (*Info).LastFailurePredictionQueryTime = SharedSystemTime;

        if NT_SUCCESS(activateStatus) {
            ClasspPowerIdleDevice(DeviceObject);
        }
    }

    ClassReleaseRemoveLock(DeviceObject, workItem as PIRP);
    IoFreeWorkItem(workItem);
}
#[no_mangle]
pub unsafe extern "system" fn ClassNotifyFailurePredicted(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Buffer: PUCHAR,
    BufferSize: ULONG,
    LogError: BOOLEAN,
    UniqueErrorValue: ULONG,
    PathId: UCHAR,
    TargetId: UCHAR,
    Lun: UCHAR,
) {
    let mut logEntry: PIO_ERROR_LOG_PACKET = core::ptr::null_mut();
    let mut eventDescriptor: EVENT_DESCRIPTOR = EVENT_DESCRIPTOR {
        Id: 0,
        Version: 0,
        Channel: 0,
        Level: 0,
        Opcode: 0,
        Task: 0,
        Keyword: 0,
    };
    let mut driverExtension: PCLASS_DRIVER_EXTENSION = core::ptr::null_mut();

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_WMI,
        b"ClasspFailurePredictPollCompletion: Failure predicted for device %p\n\0"
            .as_ptr() as PCCHAR,
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
    );

    ClassWmiFireEvent(
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        &raw const StoragePredictFailureEventGuid as LPGUID,
        0u32,
        BufferSize,
        Buffer as PVOID,
    );

    if LogError != 0u8 {
        logEntry = IoAllocateErrorLogEntry(
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject as PVOID,
            (core::mem::size_of::<IO_ERROR_LOG_PACKET>() + 3 * core::mem::size_of::<ULONG>())
                as u8,
        ) as PIO_ERROR_LOG_PACKET;

        if !logEntry.is_null() {
            (*logEntry).FinalStatus = STATUS_SUCCESS;
            (*logEntry).ErrorCode = IO_WRN_FAILURE_PREDICTED;
            (*logEntry).SequenceNumber = 0u32;
            (*logEntry).MajorFunctionCode = IRP_MJ_DEVICE_CONTROL as u8;
            (*logEntry).IoControlCode = IOCTL_STORAGE_PREDICT_FAILURE as ULONG;
            (*logEntry).RetryCount = 0u8;
            (*logEntry).UniqueErrorValue = UniqueErrorValue;
            (*logEntry).DumpDataSize = 3u16;

            let dump = &raw mut (*logEntry).DumpData as *mut ULONG;
            *dump.offset(0) = PathId as ULONG;
            *dump.offset(1) = TargetId as ULONG;
            *dump.offset(2) = Lun as ULONG;

            IoWriteErrorLogEntry(logEntry as PVOID);
        }
    }

    if LogError == 1u8
        || ((*FdoExtension).FailurePredicted == 0u8 && UniqueErrorValue == 4u32)
    {
        driverExtension = IoGetDriverObjectExtension(
            (*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).DriverObject
                as PDRIVER_OBJECT,
            ClassInitialize as *mut core::ffi::c_void,
        ) as PCLASS_DRIVER_EXTENSION;

        if !driverExtension.is_null() && (*driverExtension).EtwHandle != 0u64 {
            EventDescCreate(
                &raw mut eventDescriptor as PVOID,
                1u16,
                0u8,
                0u8,
                0u8,
                0u16,
                0u8,
                0u64,
            );
            EtwWrite(
                (*driverExtension).EtwHandle,
                &raw mut eventDescriptor as PCEVENT_DESCRIPTOR,
                core::ptr::null(),
                0u32,
                core::ptr::null_mut(),
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn ClassSetFailurePredictionPoll(
    FdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION,
    FailurePredictionMethod: FAILURE_PREDICTION_METHOD,
    PollingPeriod: u32,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if true {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/classpnp/autorun.c\0"
                    as *const u8 as *const i8,
                4302,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    let mut info: *mut _FAILURE_PREDICTION_INFO = core::ptr::null_mut();
    let mut status: NTSTATUS;

    if (*FdoExtension).FailurePredictionInfo.is_null() {
        if FailurePredictionMethod != FailurePredictionNone {
            info = ExAllocatePoolWithTag(
                NonPagedPoolNx,
                core::mem::size_of::<FAILURE_PREDICTION_INFO>(),
                CLASS_TAG_FAILURE_PREDICT,
            ) as *mut _FAILURE_PREDICTION_INFO;
            if info.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            KeInitializeEvent(
                core::ptr::addr_of_mut!((*info).Event),
                _EVENT_TYPE_SynchronizationEvent,
                1u8,
            );
            (*info).WorkQueueItem = core::ptr::null_mut::<_IO_WORKITEM>();
            (*info).Period = DEFAULT_FAILURE_PREDICTION_PERIOD;
            (*(core::ptr::addr_of_mut!((*info).LastFailurePredictionQueryTime) as *mut u64)) =
                *(core::ptr::addr_of!(SharedSystemTime) as *const u64);
        } else {
            return STATUS_SUCCESS;
        }
        (*FdoExtension).FailurePredictionInfo = info;
    } else {
        info = (*FdoExtension).FailurePredictionInfo;
    }

    KeEnterCriticalRegion();
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*info).Event) as PVOID,
        _KWAIT_REASON_UserRequest,
        (0i32) as i8,
        0u8,
        core::ptr::null_mut::<_LARGE_INTEGER>(),
    );

    if PollingPeriod != 0 {
        (*(core::ptr::addr_of_mut!((*info).Period) as *mut AtomicI32))
            .swap(PollingPeriod as i32, core::sync::atomic::Ordering::SeqCst);
    }
    (*(core::ptr::addr_of_mut!((*info).CountDown) as *mut AtomicI32))
        .swap((*info).Period as i32, core::sync::atomic::Ordering::SeqCst);

    (*info).Method = FailurePredictionMethod;

    if FailurePredictionMethod != FailurePredictionNone {
        ClasspEnableTimer(FdoExtension);
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_WMI,
            b"ClassEnableFailurePredictPoll: Enabled for device %p\n\0".as_ptr() as PCCHAR,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
    } else {
        ClasspDisableTimer(FdoExtension);
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_WMI,
            b"ClassEnableFailurePredictPoll: Disabled for device %p\n\0".as_ptr() as PCCHAR,
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        );
    }

    status = STATUS_SUCCESS;

    KeSetEvent(
        core::ptr::addr_of_mut!((*info).Event) as PVOID,
        IO_NO_INCREMENT as i32,
        0u8,
    );
    KeLeaveCriticalRegion();

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClasspFailurePredictionPeriodMissed(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> BOOLEAN {
    let mut currentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut timeDifference: LARGE_INTEGER = core::mem::zeroed();
    let mut missedPeriod: BOOLEAN = 0u8;

    // NT_ASSERT
    ntkrnl_sys::ASSERT!(!FdoExtension.is_null());
    ntkrnl_sys::ASSERT!(!(*FdoExtension).FailurePredictionInfo.is_null());
    ntkrnl_sys::ASSERT!(
        (*(*FdoExtension).FailurePredictionInfo).Method as u32
            != _PREDICT_FAILURE_METHOD_FailurePredictionNone as u32
    );

    // KeQuerySystemTime(&currentTime) inline expansion
    currentTime = *(core::ptr::addr_of!(SharedSystemTime) as *const LARGE_INTEGER);

    timeDifference.QuadPart = currentTime.QuadPart
        - (*(*FdoExtension).FailurePredictionInfo)
            .LastFailurePredictionQueryTime
            .QuadPart;

    // 10LL * 1000LL * 1000LL
    timeDifference.QuadPart /= 10_000_000;

    if timeDifference.QuadPart >= (*(*FdoExtension).FailurePredictionInfo).Period as i64 {
        missedPeriod = 1u8;
    }

    missedPeriod
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
    pub fn InterlockedDecrement(Addend: *mut LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
    pub fn InterlockedDecrement(Addend: *mut LONG) -> LONG;
}
extern "system" {
    fn ScsiGetSenseKeyAndCodes(
        SenseInfoBuffer: PVOID,
        SenseInfoBufferLength: u8,
        Options: ULONG,
        SenseKey: *mut u8,
        AdditionalSenseCode: *mut u8,
        AdditionalSenseCodeQualifier: *mut u8,
    ) -> BOOLEAN;
}

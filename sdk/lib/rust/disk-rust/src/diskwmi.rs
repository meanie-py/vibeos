use crate::types::*;
use crate::glue::*;
use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ushort};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DISK_GEOMETRY, _KWAIT_REASON_Executive, GUID, IDEREGS, _IO_STATUS_BLOCK, IoAllocateIrp, IoAllocateMdl, IoAllocateWorkItem, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoFreeIrp, IoFreeMdl, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending, IoQueueWorkItem, IoSetCompletionRoutine, IoSetNextIrpStackLocation, IoWMIRegistrationControl, KEVENT, KeInitializeEvent, KeInitializeSpinLock, KeWaitForSingleObject, LARGE_INTEGER, MmBuildMdlForNonPagedPool, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP, PSENDCMDINPARAMS, PSENDCMDOUTPARAMS, PSTORAGE_PREDICT_FAILURE, PUNICODE_STRING, SENDCMDINPARAMS, SENDCMDOUTPARAMS, STORAGE_PREDICT_FAILURE, _EVENT_TYPE_SynchronizationEvent, _WORK_QUEUE_TYPE_DelayedWorkQueue};
use core::ffi::c_void;
// ExInterlockedPopEntryList / ExInterlockedPushEntryList: not in i386 bindgen — declare for all arches.
extern "system" {
    fn ExInterlockedPopEntryList(ListHead: *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY, Lock: ntkrnl_sys::ddk::PKSPIN_LOCK) -> *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY;
    fn ExInterlockedPushEntryList(ListHead: *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY, ListEntry: *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY, Lock: ntkrnl_sys::ddk::PKSPIN_LOCK) -> *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY;
}

// variadic — cdecl on all arches (never stdcall)
extern "C" {
    fn StorDebugPrint(level: u32, flags: u32, msg: *const i8, ...);
}
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_FAILURE_PREDICT_DATA as STORAGE_FAILURE_PREDICT_DATA, _STORAGE_FAILURE_PREDICT_STATUS as STORAGE_FAILURE_PREDICT_STATUS};
use ntkrnl_sys::constants::{STATUS_INVALID_DEVICE_REQUEST};
use ntkrnl_sys::ddk::{CCHAR, DEVICE_OBJECT, IO_STACK_LOCATION, IRP, MDL, _GUID, _IO_WORKITEM, _SINGLE_LIST_ENTRY};
// resolve_imports.py: explicit kernel/private-type imports
// bitfield accessors are methods on MODE_INFO_EXCEPTIONS / its inner type (not standalone imports)
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PIO_COMPLETION_ROUTINE, PIO_WORKITEM_ROUTINE};
// moved from extern "system": types_i386 declares these cdecl (no NTAPI in disk.h);
// the defs are extern "C" — CC must agree or LLVM folds callers to ud2.
extern "C" {
    fn DiskSetInfoExceptionInformation(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, ModeInfo: *mut MODE_INFO_EXCEPTIONS) -> NTSTATUS;
    fn DiskReadDriveCapacity(Fdo: PDEVICE_OBJECT) -> NTSTATUS;
    fn DiskGetInfoExceptionInformation( FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, modeInfo: *mut MODE_INFO_EXCEPTIONS, ) -> NTSTATUS;
}

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
    fn __int2c();
    fn TracePrint(level: i32, flag: u32, format: *const i8, ...);
}
extern "system" {
    fn KeGetCurrentIrql() -> u8;
    fn ClassModeSenseEx( DeviceObject: *mut DEVICE_OBJECT, DataBuffer: *mut i8, DataBufferLength: u32, PageMode: u8, PageControl: u8, ) -> u32;
    fn ClassFindModePage( DataBuffer: *mut i8, DataBufferLength: u32, PageMode: u8, Reposition: i32, ) -> *mut core::ffi::c_void;
    fn ExFreePool(P: *mut core::ffi::c_void);
}
extern "C" {
    // Real stdcall NTAPI (resolves to the ntoskrnl export; called at line ~854).
    fn KeFlushIoBuffers(Mdl: *mut core::ffi::c_void, ReadOperation: i32, DmaOperation: i32) -> i32;
}
// These are all C macros / FORCEINLINEs shimmed as cdecl functions in
// ddk_shims.rs (extern "C"). The block MUST be extern "C": on i386 an
// extern "system" decl decorates them stdcall (`_Name@N`) and fails to link
// against the cdecl defs (`_Name`) — REVERSE_BYTES/GetSrbScsiData surfaced as
// `undefined _REVERSE_BYTES@8`/`_GetSrbScsiData@24`. No-op on amd64/arm64
// where system==C. (verify_ffi FFI-ABI-mismatch class; field manual §ABI.)
extern "C" {
    fn REVERSE_BYTES(Dst: *mut u8, Src: *const u8);
    fn FREE_POOL(ptr: *mut core::ffi::c_void);
    fn NT_ASSERT(expression: i32);
    fn TEST_FLAG(value: u8, flag: u8) -> bool;
    fn SET_FLAG(var: &mut u32, flag: u32);
    fn CLEAR_FLAG(var: &mut u32, flag: u32);
    fn GetSrbScsiData( srb: *mut STORAGE_REQUEST_BLOCK, data_out: *mut u8, data_length: *mut u32, scsi_status: *mut u8, sense_buffer: *mut *mut core::ffi::c_void, sense_buffer_length: *mut u8, ) -> *mut CDB;
    fn SetSrbScsiData( srbEx: PSTORAGE_REQUEST_BLOCK, cdbLength8: UCHAR, cdbLength32: ULONG, cdb10: UCHAR, senseBuffer: PVOID, senseBufferLength: UCHAR, );
    fn DiskIsValidSmartSelfTest(Subcommand: u8) -> BOOLEAN;
}
extern "system" {
    fn IoGetIoPriorityHint(Irp: PIRP) -> u32;
    fn ClassAcquireRemoveLockEx( DeviceObject: PDEVICE_OBJECT, Irp: *mut core::ffi::c_void, File: *const i8, Line: u32, ) -> u32;
    fn ClassReleaseRemoveLock(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
    fn ClassWmiCompleteRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Status: NTSTATUS, Information: ULONG, PriorityBoost: CCHAR) -> NTSTATUS;
    fn ClassModeSelect( DeviceObject: *mut DEVICE_OBJECT, ModeData: *const i8, ModeDataSize: u32, PSBit: u8, ) -> NTSTATUS;
    fn ClassReleaseQueue(dev: PDEVICE_OBJECT);
    fn ClassInterpretSenseInfo( dev: PDEVICE_OBJECT, srb: *mut SCSI_REQUEST_BLOCK, majorFunction: UCHAR, retryCount: ULONG, remainingRetries: ULONG, status: *mut NTSTATUS, retryInterval: *mut ULONG, ) -> BOOLEAN;
    fn ClassNotifyFailurePredicted( FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION, Buffer: *mut u8, Length: u32, FirstFailure: i32, ReasonCode: u32, PathId: u8, TargetId: u8, Lun: u8, );
    fn ClassSetFailurePredictionPoll(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION, method: FAILURE_PREDICTION_METHOD, PollTimeInSeconds: ULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut DiskReregHead: SINGLE_LIST_ENTRY = SINGLE_LIST_ENTRY {
    Next: (::core::ptr::null::<_SINGLE_LIST_ENTRY>() as *mut _SINGLE_LIST_ENTRY).cast(),
};

pub static mut DiskReregSpinlock: KSPIN_LOCK = 0;

pub static mut DiskReregWorkItems: LONG = 0;

#[no_mangle] // referenced cross-file via the types_* extern decl — needs the C symbol name
pub static mut DiskWmiFdoGuidList: [GUIDREGINFO; 7] = [
    GUIDREGINFO {
        Guid: _GUID {
            Data1: 0x25007f51 as ::core::ffi::c_int as ::core::ffi::c_uint,
            Data2: 0x57c2 as ::core::ffi::c_ushort,
            Data3: 0x11d1 as ::core::ffi::c_ushort,
            Data4: [
                0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            ],
        },
        InstanceCount: 1 as ULONG,
        Flags: 0 as ULONG,
    },
    GUIDREGINFO {
        Guid: _GUID {
            Data1: 0x78ebc102 as ::core::ffi::c_int as ::core::ffi::c_uint,
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
        },
        InstanceCount: 1 as ULONG,
        Flags: WMIREG_FLAG_EXPENSIVE as ULONG,
    },
    GUIDREGINFO {
        Guid: _GUID {
            Data1: 0x78ebc103 as ::core::ffi::c_int as ::core::ffi::c_uint,
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
        },
        InstanceCount: 1 as ULONG,
        Flags: WMIREG_FLAG_EXPENSIVE as ULONG,
    },
    GUIDREGINFO {
        Guid: _GUID {
            Data1: 0x78ebc105 as ::core::ffi::c_int as ::core::ffi::c_uint,
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
        },
        InstanceCount: 1 as ULONG,
        Flags: WMIREG_FLAG_EXPENSIVE as ULONG,
    },
    GUIDREGINFO {
        Guid: _GUID {
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
        },
        InstanceCount: 1 as ULONG,
        Flags: WMIREG_FLAG_EVENT_ONLY_GUID as ULONG,
    },
    GUIDREGINFO {
        Guid: _GUID {
            Data1: 0xdae10783 as ::core::ffi::c_uint,
            Data2: 0xcc31 as ::core::ffi::c_ushort,
            Data3: 0x4d2a as ::core::ffi::c_ushort,
            Data4: [
                0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            ],
        },
        InstanceCount: 1 as ULONG,
        Flags: WMIREG_FLAG_EXPENSIVE as ULONG,
    },
    GUIDREGINFO {
        Guid: _GUID {
            Data1: 0x1101d829 as ::core::ffi::c_int as ::core::ffi::c_uint,
            Data2: 0x167b as ::core::ffi::c_ushort,
            Data3: 0x4ebf as ::core::ffi::c_ushort,
            Data4: [
                0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
                0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            ],
        },
        InstanceCount: 1 as ULONG,
        Flags: 0 as ULONG,
    },
];

pub static mut DiskPredictFailureEventGuid: GUID = _GUID {
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

pub const DiskGeometryGuid: usize = 0;
pub const SmartStatusGuid: usize = 1;
pub const SmartDataGuid: usize = 2;
pub const SmartPerformFunction: usize = 3;
pub const AllowDisallowPerformanceHit: usize = 1;
pub const EnableDisableHardwareFailurePrediction: usize = 2;
pub const EnableDisableFailurePredictionPolling: usize = 3;
pub const GetFailurePredictionCapability: usize = 4;
pub const EnableOfflineDiags: usize = 5;
pub const SmartEventGuid: usize = 4;
pub const SmartThresholdsGuid: usize = 5;
pub const ScsiInfoExceptionsGuid: usize = 6;
pub const WMIREG_FLAG_EXPENSIVE: u32 = 0x00000002;
pub const WMIREG_FLAG_EVENT_ONLY_GUID: u32 = 0x00000004;

const APC_LEVEL: u8 = 1;
const SMART_CMD: u8 = 0xB0;
const ID_CMD: u8 = 0xEC;
const ENABLE_SMART: u8 = 0xD8;
const DISABLE_SMART: u8 = 0xD9;
const RETURN_SMART_STATUS: u8 = 0xDA;
const ENABLE_DISABLE_AUTOSAVE: u8 = 0xD2;
const SAVE_ATTRIBUTE_VALUES: u8 = 0xD3;
const EXECUTE_OFFLINE_DIAGS: u8 = 0xD4;
const READ_ATTRIBUTES: u8 = 0xD0;
const READ_THRESHOLDS: u8 = 0xD1;
const SMART_READ_LOG: u8 = 0xD5;
const SMART_WRITE_LOG: u8 = 0xD6;
// ddk/scsi.h: ((FILE_DEVICE_SCSI << 16) + 0x05xx). SRB_IO_CONTROL.ControlCode
// values the miniport dispatches on. All but WRITE_SMART_LOG were invented
// (0x0500xx), so no IDE/ATA miniport could recognise a single SMART request.
const IOCTL_SCSI_MINIPORT_IDENTIFY: u32 = 0x1B0501;
const IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS: u32 = 0x1B0502;
const IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS: u32 = 0x1B0503;
const IOCTL_SCSI_MINIPORT_ENABLE_SMART: u32 = 0x1B0504;
const IOCTL_SCSI_MINIPORT_DISABLE_SMART: u32 = 0x1B0505;
const IOCTL_SCSI_MINIPORT_RETURN_STATUS: u32 = 0x1B0506;
const IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE: u32 = 0x1B0507;
const IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES: u32 = 0x1B0508;
const IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS: u32 = 0x1B0509;
const IOCTL_SCSI_MINIPORT_READ_SMART_LOG: u32 = 0x1B050B;
const IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG: u32 = 0x1B050C;
const SRB_TYPE_STORAGE_REQUEST_BLOCK: u8 = 1; // srb.h:664
// ddk/srb.h:108 == 0x28. This constant is WRITTEN into srbEx.Function here and TESTED
// against a second, differently-wrong copy below -- the test could never match.
const SRB_FUNCTION_STORAGE_REQUEST_BLOCK: u8 = 0x28;
// 0x53524258 ('SRBX'), ddk/srb.h:616 -- storport validates this field, and NEITHER of
// the two values this crate invented ('SRBS' here, 'SRBT' in disk.rs) matches it.
const SRB_SIGNATURE: u32 = 0x53524258;
const STORAGE_REQUEST_BLOCK_VERSION_1: u32 = 1;
const SRB_FUNCTION_IO_CONTROL: u32 = 2;
const SRB_FLAGS_DATA_IN: u32 = 0x40;
const SRB_FLAGS_NO_QUEUE_FREEZE: u32 = 0x100;
const SRB_FLAGS_NO_KEEP_AWAKE: u32 = 0x100000;
const SRB_SIMPLE_TAG_REQUEST: u16 = 0x20;
// ddk/srb.h:55 `#define SP_UNTAGGED ((UCHAR) ~0)` == 0xFF. QueueTag is a UCHAR, so
// neither 0 nor 0xFFFFFFFF can ever compare equal to an untagged request.
const SP_UNTAGGED: u32 = 0xFF;
const STOR_ADDRESS_TYPE_BTL8: u16 = 1; // scsi.h:3571 (0 is STOR_ADDRESS_TYPE_UNKNOWN)
const STOR_ADDR_BTL8_ADDRESS_LENGTH: u32 = 4; // scsi.h:3574
const SMART_CYL_LOW: u8 = 0x4F;
const SMART_CYL_HI: u8 = 0xC2;
const IRP_MJ_SCSI: u32 = 0x0F;

const IOCTL_STORAGE_PREDICT_FAILURE: u32 = 0x002D1100;

const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
const TRACE_FLAG_WMI: u32 = 6; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185_u32 as i32;

#[repr(C)]
pub struct MODE_PARAMETER_HEADER {
    pub ModeDataLength: u8,
    pub MediumType: u8,
    pub DeviceSpecificParameter: u8,
    pub BlockDescriptorLength: u8,
    pub Reserved: [u8; 4],
}

const NonPagedPool: u32 = 0;
const DISK_TAG_SMART: u32 = 0x61446353; // 'aDcS' (disk.h)


type KIRQL = u8;
type PCSTR = *const i8;


// disk.h:75: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)


// Forward declarations for unknown functions

// Constants required by the function
const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const MODE_PAGE_FAULT_REPORTING: UCHAR = 0x1C;
const STATUS_WMI_READ_ONLY: NTSTATUS = 0xC00002C6u32 as i32;
const STATUS_WMI_GUID_NOT_FOUND: NTSTATUS = 0xC0000295u32 as i32;
const IO_NO_INCREMENT: CCHAR = 0;


const SMART_LOG_SECTOR_SIZE: usize = 512;
const FailurePredictionSense: u32 = 3; // classpnp.h:237 -- and diskwmi.rs already carried a
                                       // function-local copy holding the correct 3, so this
                                       // file compared the same capability two ways
const FailurePredictionNone: u32 = 0;
const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xC000009Au32 as i32 ;
const STATUS_NOT_SUPPORTED: i32 = 0xC00000BBu32 as i32 ;
const STATUS_SUCCESS: i32 = 0;



const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32 ;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC0000023u32 as i32 ;
const STATUS_WMI_ITEMID_NOT_FOUND: i32 = 0xC0000297u32 as i32;


const ReadLogSectors: u32 = 6;
const WriteLogSectors: u32 = 7;
const ExecuteSelfTest: u32 = 8;

const FailurePredictionSmart: i32 = 2; // classpnp.h:237
const FailurePredictionIoctl: i32 = 1; // classpnp.h:237

unsafe fn DiskEnableSmart(FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) -> NTSTATUS {
    const SRB_ARRAY_SIZE: usize = core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<SENDCMDINPARAMS>();
    let mut srbControl: [u8; SRB_ARRAY_SIZE] = [0u8; SRB_ARRAY_SIZE];
    let mut bufferSize: u32 = SRB_ARRAY_SIZE as u32;
    DiskPerformSmartCommand(
        FdoExtension,
        IOCTL_SCSI_MINIPORT_ENABLE_SMART,
        SMART_CMD,
        ENABLE_SMART,
        0u8,
        0u8,
        core::ptr::addr_of_mut!(srbControl) as *mut u8 as PSRB_IO_CONTROL,
        core::ptr::addr_of_mut!(bufferSize),
    )
}

unsafe fn DiskDisableSmart(FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION) -> NTSTATUS {
    const SRB_CTL_SIZE: usize = core::mem::size_of::<SRB_IO_CONTROL>();
    const SENDCMD_SIZE: usize = core::mem::size_of::<SENDCMDINPARAMS>();
    let mut srbControl: [u8; SRB_CTL_SIZE + SENDCMD_SIZE] = [0u8; SRB_CTL_SIZE + SENDCMD_SIZE];
    let mut bufferSize: u32 = (SRB_CTL_SIZE + SENDCMD_SIZE) as u32;
    DiskPerformSmartCommand(
        FdoExtension,
        IOCTL_SCSI_MINIPORT_DISABLE_SMART, // was 0
        SMART_CMD,
        DISABLE_SMART,
        0u8,
        0u8,
        &raw mut srbControl as *mut u8 as *mut SRB_IO_CONTROL,
        &raw mut bufferSize,
    )
}
#[no_mangle]
pub unsafe extern "system" fn DiskEnableSmartAttributeAutosave(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
) -> i32 {
    let mut srbControl =
        [0u8; core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<SENDCMDINPARAMS>()];
    let mut bufferSize = core::mem::size_of_val(&srbControl) as u32;
    DiskPerformSmartCommand(
        FdoExtension,
        IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE,
        SMART_CMD,
        ENABLE_DISABLE_AUTOSAVE,
        0xf1u8,
        0u8,
        srbControl.as_mut_ptr() as *mut SRB_IO_CONTROL,
        &raw mut bufferSize,
    )
}
#[no_mangle]
pub unsafe extern "system" fn DiskDisableSmartAttributeAutosave(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut srbControl = [0u8; core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<SENDCMDINPARAMS>()];
    let mut bufferSize: u32 = srbControl.len() as u32;

    DiskPerformSmartCommand(
        FdoExtension,
        IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE,
        SMART_CMD,
        ENABLE_DISABLE_AUTOSAVE,
        0x00u8,
        0u8,
        srbControl.as_mut_ptr() as *mut SRB_IO_CONTROL,
        &mut bufferSize,
    )
}

unsafe fn DiskExecuteSmartDiagnostics(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Subcommand: u8,
) -> NTSTATUS {
    let mut srbControl: [u8; 61] = [0; 61];
    let mut bufferSize: u32 = srbControl.len() as u32;
    return DiskPerformSmartCommand(
        FdoExtension,
        IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS as u32,
        SMART_CMD,
        EXECUTE_OFFLINE_DIAGS,
        0u8,
        Subcommand,
        &raw mut srbControl as *mut u8 as PSRB_IO_CONTROL,
        &mut bufferSize,
    );
}
#[no_mangle]
pub unsafe extern "system" fn DiskReadSmartLog(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    SectorCount: u8,
    LogAddress: u8,
    Buffer: *mut u8,
) -> i32 {
    let mut srbControl: *mut SRB_IO_CONTROL = core::ptr::null_mut();
    let mut status: i32;
    let mut sendCmdOutParams: *mut SENDCMDOUTPARAMS;
    let logSize: u32;
    let mut bufferSize: u32;

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
                b"drivers/storage/class/disk/diskwmi.c\0".as_ptr()
                    as *const u8,
                391 as i32,
                b"0\0".as_ptr() as *const u8,
            );
            __int2c();
        }
    }

    logSize = (SectorCount as usize * SMART_LOG_SECTOR_SIZE) as u32;
    bufferSize = (core::mem::size_of::<SRB_IO_CONTROL>() as usize).wrapping_add(
        if core::mem::size_of::<SENDCMDINPARAMS>() as usize
            > (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
                .wrapping_sub(1)
                .wrapping_add(logSize as usize)
        {
            core::mem::size_of::<SENDCMDINPARAMS>() as usize
        } else {
            (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
                .wrapping_sub(1)
                .wrapping_add(logSize as usize)
        },
    ) as u32;

    srbControl = ExAllocatePoolWithTag(NonPagedPool, (bufferSize as u64) as usize, DISK_TAG_SMART) as *mut SRB_IO_CONTROL;
    if !srbControl.is_null() {
        status = DiskPerformSmartCommand(
            FdoExtension,
            IOCTL_SCSI_MINIPORT_READ_SMART_LOG, // was 0x2c ("typical value")
            SMART_CMD,
            SMART_READ_LOG,
            SectorCount,
            LogAddress,
            srbControl,
            &mut bufferSize,
        );
        if status >= 0 {
            sendCmdOutParams = (srbControl as *mut u8)
                .add(core::mem::size_of::<SRB_IO_CONTROL>()) as *mut SENDCMDOUTPARAMS;
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of_mut!((*sendCmdOutParams).bBuffer) as *const u8,
                Buffer,
                logSize as usize,
            );
        }
        if !srbControl.is_null() {
            ExFreePool(srbControl as *mut core::ffi::c_void);
            srbControl = core::ptr::null_mut();
        }
    } else {
        status = 0xC000009Au32 as i32 ; // STATUS_INSUFFICIENT_RESOURCES
    }
    status
}
#[no_mangle]
pub unsafe extern "system" fn DiskWriteSmartLog(
    FdoExtension: *mut core::ffi::c_void, // PFUNCTIONAL_DEVICE_EXTENSION
    SectorCount: u8,
    LogAddress: u8,
    Buffer: *mut u8,
) -> NTSTATUS {
    const SMART_LOG_SECTOR_SIZE: u32 = 512;
    const DISK_TAG_SMART: u32 = 0x61446353; // 'aDcS' (disk.h)
    const NonPagedPool: u32 = 0;
    const IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG: u32 = 0x1B050C; // scsi.h ((FILE_DEVICE_SCSI<<16)+0x050c)
    const SMART_CMD: u8 = 0xB0;
    const SMART_WRITE_LOG: u8 = 0xD6;
    const APC_LEVEL: u32 = 1;

    let mut srbControl: *mut SRB_IO_CONTROL = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut sendCmdInParams: *mut SENDCMDINPARAMS = core::ptr::null_mut();
    let mut logSize: u32 = 0;
    let mut bufferSize: u32 = 0;

    if KeGetCurrentIrql() as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0".as_ptr() as *const i8,
                441i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    logSize = (SectorCount as i32 * SMART_LOG_SECTOR_SIZE as i32) as u32;
    bufferSize = (core::mem::size_of::<SRB_IO_CONTROL>()
        .wrapping_add(core::mem::size_of::<SENDCMDINPARAMS>())
        .wrapping_sub(1)
        .wrapping_add(logSize as usize)) as u32;

    srbControl = ExAllocatePoolWithTag(NonPagedPool, bufferSize as usize, DISK_TAG_SMART) as *mut SRB_IO_CONTROL;

    if !srbControl.is_null() {
        sendCmdInParams = (srbControl as *mut u8)
            .add(core::mem::size_of::<SRB_IO_CONTROL>()) as *mut SENDCMDINPARAMS;

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*sendCmdInParams).bBuffer) as *mut u8 as *mut core::ffi::c_void,
            Buffer as *const core::ffi::c_void,
            logSize as usize,
        );

        status = DiskPerformSmartCommand(
            (FdoExtension).cast(),
            IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG,
            SMART_CMD,
            SMART_WRITE_LOG,
            SectorCount,
            LogAddress,
            srbControl,
            core::ptr::addr_of_mut!(bufferSize),
        );

        // FREE_POOL(srbControl)
        if !srbControl.is_null() {
            ExFreePool(srbControl as *mut core::ffi::c_void);
            srbControl = core::ptr::null_mut();
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DiskPerformSmartCommand(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    SrbControlCode: u32,
    Command: u8,
    Feature: u8,
    SectorCount: u8,
    SectorNumber: u8,
    SrbControl: PSRB_IO_CONTROL,
    BufferSize: *mut u32,
) -> i32 {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8).cast(),
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8,
                536i32,
                b"0\0" as *const u8,
            );
            __int2c();
        }
    }

    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let mut buffer: *mut u8;
    let mut cmdInParameters: PSENDCMDINPARAMS;
    let mut availableBufferSize: u32;

    // Point to the buffer after SRB_IO_CONTROL header
    buffer = (SrbControl as *mut u8).offset(core::mem::size_of::<SRB_IO_CONTROL>() as isize);
    cmdInParameters = buffer as PSENDCMDINPARAMS;

    if *BufferSize as usize >= core::mem::size_of::<SRB_IO_CONTROL>() {
        availableBufferSize =
            (*BufferSize as usize).wrapping_sub(core::mem::size_of::<SRB_IO_CONTROL>()) as u32;
    } else {
        return STATUS_BUFFER_TOO_SMALL;
    }

    // Debug block from C #if DBG
    if cfg!(debug_assertions) {
        let mut controlCode: u32 = 0;
        let mut lengthNeeded: u32 = core::mem::size_of::<SENDCMDINPARAMS>() as u32;

        if Command == SMART_CMD {
            match Feature {
                ENABLE_SMART => {
                    controlCode = IOCTL_SCSI_MINIPORT_ENABLE_SMART;
                }
                DISABLE_SMART => {
                    controlCode = IOCTL_SCSI_MINIPORT_DISABLE_SMART;
                }
                RETURN_SMART_STATUS => {
                    controlCode = IOCTL_SCSI_MINIPORT_RETURN_STATUS;
                    let alt = (core::mem::size_of::<SENDCMDOUTPARAMS>() - 1)
                        + core::mem::size_of::<IDEREGS>();
                    lengthNeeded = if (lengthNeeded as usize) > alt {
                        lengthNeeded as usize
                    } else {
                        alt
                    } as u32;
                }
                ENABLE_DISABLE_AUTOSAVE => {
                    controlCode = IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE;
                }
                SAVE_ATTRIBUTE_VALUES => {
                    controlCode = IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES;
                }
                EXECUTE_OFFLINE_DIAGS => {
                    controlCode = IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS;
                }
                READ_ATTRIBUTES => {
                    controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS;
                    let alt = (core::mem::size_of::<SENDCMDOUTPARAMS>() - 1) + 512;
                    lengthNeeded = if (lengthNeeded as usize) > alt {
                        lengthNeeded as usize
                    } else {
                        alt
                    } as u32;
                }
                READ_THRESHOLDS => {
                    controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS;
                    let alt = (core::mem::size_of::<SENDCMDOUTPARAMS>() - 1) + 512;
                    lengthNeeded = if (lengthNeeded as usize) > alt {
                        lengthNeeded as usize
                    } else {
                        alt
                    } as u32;
                }
                SMART_READ_LOG => {
                    controlCode = IOCTL_SCSI_MINIPORT_READ_SMART_LOG;
                    let alt = (core::mem::size_of::<SENDCMDOUTPARAMS>() - 1)
                        + (SectorCount as usize * SMART_LOG_SECTOR_SIZE);
                    lengthNeeded = if (lengthNeeded as usize) > alt {
                        lengthNeeded as usize
                    } else {
                        alt
                    } as u32;
                }
                SMART_WRITE_LOG => {
                    controlCode = IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG;
                    lengthNeeded = (lengthNeeded as usize)
                        .wrapping_sub(1)
                        .wrapping_add(SectorCount as usize * SMART_LOG_SECTOR_SIZE)
                        as u32;
                }
                _ => {}
            }
        } else if Command == ID_CMD {
            controlCode = IOCTL_SCSI_MINIPORT_IDENTIFY;
            let alt = (core::mem::size_of::<SENDCMDOUTPARAMS>() - 1) + 512;
            lengthNeeded = if (lengthNeeded as usize) > alt {
                lengthNeeded as usize
            } else {
                alt
            } as u32;
        } else {
            if 0 == 0 {
                DbgPrint(
                    (b"Assertion failed at %s(%d): %s\n\0" as *const u8).cast(),
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        as *const u8,
                    642i32,
                    b"FALSE\0" as *const u8,
                );
                __int2c();
            }
        }

        if controlCode != SrbControlCode {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8).cast(),
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8,
                645i32,
                b"controlCode == SrbControlCode\0" as *const u8,
            );
            __int2c();
        }

        if availableBufferSize < lengthNeeded {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8).cast(),
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8,
                646i32,
                b"availableBufferSize >= lengthNeeded\0" as *const u8,
            );
            __int2c();
        }
    }

    // Build SrbControl header
    (*SrbControl).HeaderLength = core::mem::size_of::<SRB_IO_CONTROL>() as u32;
    core::ptr::copy_nonoverlapping(
        b"SCSIDISK\0".as_ptr() as *const u8,
        core::ptr::addr_of_mut!((*SrbControl).Signature) as *mut u8,
        8,
    );
    (*SrbControl).Timeout = (*FdoExtension).TimeOutValue;
    (*SrbControl).Length = availableBufferSize;
    (*SrbControl).ControlCode = SrbControlCode;

    // Build input parameters
    (*cmdInParameters).cBufferSize = core::mem::size_of::<SENDCMDINPARAMS>() as u32;
    (*cmdInParameters).bDriveNumber = (*diskData).ScsiAddress.TargetId;
    (*cmdInParameters).irDriveRegs.bFeaturesReg = Feature;
    (*cmdInParameters).irDriveRegs.bSectorCountReg = SectorCount;
    (*cmdInParameters).irDriveRegs.bSectorNumberReg = SectorNumber;
    (*cmdInParameters).irDriveRegs.bCylLowReg = SMART_CYL_LOW;
    (*cmdInParameters).irDriveRegs.bCylHighReg = SMART_CYL_HI;
    (*cmdInParameters).irDriveRegs.bCommandReg = Command;

    let mut event: KEVENT = core::mem::zeroed();
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut startingOffset: LARGE_INTEGER = core::mem::zeroed();
    KeInitializeEvent(&raw mut event, _EVENT_TYPE_NotificationEvent as u32, 0);
    startingOffset.QuadPart = 1;

    let length = (*SrbControl).HeaderLength + (*SrbControl).Length;
    let irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_SCSI,
        (*commonExtension).LowerDeviceObject,
        SrbControl as *mut core::ffi::c_void,
        length,
        &raw mut startingOffset as *mut LARGE_INTEGER,
        &raw mut event as *mut KEVENT,
        &raw mut ioStatus as *mut _IO_STATUS_BLOCK,
    );

    if irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let irpStack = IoGetNextIrpStackLocation(irp);
    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;
    (*irpStack).MinorFunction = 1;

    if (*(*FdoExtension).AdapterDescriptor).SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        let mut srbExBuffer: [u8; 144] = [0u8; 144];
        let srbEx = srbExBuffer.as_mut_ptr() as *mut STORAGE_REQUEST_BLOCK;
        (*irpStack).Parameters.Others.Argument1 = srbEx as *mut core::ffi::c_void;

        (*srbEx).Length = core::mem::offset_of!(STORAGE_REQUEST_BLOCK, Signature) as u16;
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
        (*srbEx).Signature = SRB_SIGNATURE;
        (*srbEx).Version = STORAGE_REQUEST_BLOCK_VERSION_1;
        (*srbEx).SrbLength = core::mem::size_of::<[u8; 144]>() as u32;
        (*srbEx).SrbFunction = SRB_FUNCTION_IO_CONTROL;
        (*srbEx).RequestPriority = IoGetIoPriorityHint(irp) as u16;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        (*srbEx).SrbFlags = (*FdoExtension).SrbFlags;
        (*srbEx).SrbFlags |= SRB_FLAGS_DATA_IN;
        (*srbEx).SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;
        (*srbEx).SrbFlags |= SRB_FLAGS_NO_KEEP_AWAKE;
        (*srbEx).RequestAttribute = SRB_SIMPLE_TAG_REQUEST;
        (*srbEx).RequestTag = SP_UNTAGGED;
        (*srbEx).OriginalRequest = irp as *mut core::ffi::c_void;
        (*srbEx).TimeOutValue = (*SrbControl).Timeout;
        (*srbEx).DataBuffer = SrbControl as *mut core::ffi::c_void;
        (*srbEx).DataTransferLength = length;

        let storAddrBtl8 =
            (srbEx as *mut u8).offset((*srbEx).AddressOffset as isize) as *mut STOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8;
        (*storAddrBtl8).AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
        (*storAddrBtl8).Path = (*diskData).ScsiAddress.PathId;
        (*storAddrBtl8).Target = (*diskData).ScsiAddress.TargetId;
        srb.Lun = (*diskData).ScsiAddress.Lun;
        (*storAddrBtl8).Lun = srb.Lun;
    } else {
        (*irpStack).Parameters.Others.Argument1 =
            &raw mut srb as *mut SCSI_REQUEST_BLOCK as *mut core::ffi::c_void;
        srb.PathId = (*diskData).ScsiAddress.PathId;
        srb.TargetId = (*diskData).ScsiAddress.TargetId;
        srb.Lun = (*diskData).ScsiAddress.Lun;
        srb.Function = SRB_FUNCTION_IO_CONTROL as u8;
        srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
        srb.SrbFlags = (*FdoExtension).SrbFlags;
        srb.SrbFlags |= SRB_FLAGS_DATA_IN;
        srb.SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;
        srb.SrbFlags |= SRB_FLAGS_NO_KEEP_AWAKE;
        srb.QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
        srb.QueueTag = (SP_UNTAGGED) as u8;
        srb.OriginalRequest = irp as *mut core::ffi::c_void;
        srb.TimeOutValue = (*SrbControl).Timeout;
        srb.DataBuffer = SrbControl as *mut core::ffi::c_void;
        srb.DataTransferLength = length;
    }

    KeFlushIoBuffers(((*irp).MdlAddress).cast(), 0, 1);

    let mut status = IoCallDriver((*commonExtension).LowerDeviceObject, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut event as *mut KEVENT as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i32 as i32 as i8,
            0,
            core::ptr::null_mut(),
        );
        status = ioStatus.__bindgen_anon_1.Status;
    }

    status
}



#[no_mangle]
pub unsafe extern "system" fn DiskGetIdentifyInfo(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    SupportSmart: *mut BOOLEAN,
) -> NTSTATUS {
    let mut outBuffer: [u8; 556] = [0u8; 556];
    let mut outBufferSize: u32 = core::mem::size_of::<[u8; 556]>() as u32;
    let mut status: NTSTATUS = 0;

    // PAGED_CODE expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8 as *const i8,
                822 as i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    status = DiskPerformSmartCommand(
        FdoExtension,
        IOCTL_SCSI_MINIPORT_IDENTIFY,
        ID_CMD,
        0u8,
        0u8,
        0u8,
        (&raw mut outBuffer as *mut u8).cast(),
        &raw mut outBufferSize,
    );

    if status >= 0 {
        let identifyData: *mut u16 = (&raw mut outBuffer as *mut u8)
            .add(core::mem::size_of::<SRB_IO_CONTROL>() + core::mem::size_of::<SENDCMDOUTPARAMS>() - 1)
            as *mut u16;
        let commandSetSupported: u16 = *identifyData.add(82);
        *SupportSmart = ((commandSetSupported as i32 != 0xffff)
            && (commandSetSupported as i32 != 0)
            && ((commandSetSupported as i32 & 1) == 1)) as u8;
    } else {
        *SupportSmart = 0u8;
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_IOCTL,
        b"DiskGetIdentifyInfo: SMART %s supported for device %p, status %lx\n\0" as *const u8 as *const i8,
        if *SupportSmart != 0u8 {
            b"is\0" as *const u8 as *const i8
        } else {
            b"is not\0" as *const u8 as *const i8
        },
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject as *mut core::ffi::c_void,
        status,
    );

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn DiskSendFailurePredictIoctl(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    checkFailure: *mut STORAGE_PREDICT_FAILURE,
) -> NTSTATUS {
    let mut event: KEVENT = core::mem::zeroed();
    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut irp: PIRP = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;

    // PAGED_CODE expansion
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL as i32) {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8).cast(),
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8,
                865i32,
                b"0\0" as *const u8,
            );
            __int2c();
        }
    }

    KeInitializeEvent(&raw mut event, _EVENT_TYPE_SynchronizationEvent as u32, 0u8);

    deviceObject = IoGetAttachedDeviceReference((*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_STORAGE_PREDICT_FAILURE as u32,
        deviceObject,
        core::ptr::null_mut(),
        0u32,
        checkFailure as *mut core::ffi::c_void,
        core::mem::size_of::<STORAGE_PREDICT_FAILURE>() as u32,
        0u8,
        &raw mut event,
        &raw mut ioStatus,
    );

    if !irp.is_null() {
        status = IoCallDriver(deviceObject, irp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &raw mut event as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive as u32,
                (0i32 as u32) as i8,
                0u8,
                core::ptr::null_mut(),
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    ObDereferenceObject(deviceObject as *mut core::ffi::c_void);

    status
}


#[no_mangle]
 pub unsafe extern "system" fn DiskGetModePage(
    Fdo: *mut DEVICE_OBJECT,
    PageMode: u8,
    PageControl: u8,
    ModeData: *mut MODE_PARAMETER_HEADER,
    ModeDataSize: *mut u32,
    PageData: *mut *mut core::ffi::c_void,
) -> i32 {
    let mut size: u32 = 0;
    let mut pageData: *mut core::ffi::c_void = core::ptr::null_mut();

    // PAGED_CODE();
    if KeGetCurrentIrql() > APC_LEVEL {}

    if ModeData.is_null()
        || ModeDataSize.is_null()
        || *ModeDataSize < (core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32)
        || PageData.is_null()
    {
        return STATUS_INVALID_PARAMETER;
    }

    RtlZeroMemory(
        ModeData as *mut core::ffi::c_void,
        *ModeDataSize as usize,
    );

    size = ClassModeSenseEx(
        Fdo,
        ModeData as *mut i8,
        *ModeDataSize,
        PageMode,
        PageControl,
    );

    if size < (core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32) {
        size = ClassModeSenseEx(
            Fdo,
            ModeData as *mut i8,
            *ModeDataSize,
            PageMode,
            PageControl,
        );

        if size < (core::mem::size_of::<MODE_PARAMETER_HEADER>() as u32) {
            TracePrint(
                (TRACE_LEVEL_ERROR) as i32,
                TRACE_FLAG_WMI,
                b"DiskGetModePage: Mode Sense for Page Mode %d with Page Control %d failed\n\0"
                    .as_ptr() as *const i8,
                PageMode as i32,
                PageControl as i32,
            );
            *ModeDataSize = 0;
            return STATUS_IO_DEVICE_ERROR;
        }
    }

    if size > ((*ModeData).ModeDataLength as u32 + 1) {
        size = (*ModeData).ModeDataLength as u32 + 1;
    }

    *ModeDataSize = size;

    pageData = ClassFindModePage(
        ModeData as *mut i8,
        size,
        PageMode,
        1, // TRUE
    );

    if !pageData.is_null() {
        *PageData = pageData;
        return STATUS_SUCCESS;
    } else {
        *PageData = core::ptr::null_mut();
        return STATUS_NOT_SUPPORTED;
    }
}
#[no_mangle]
pub unsafe extern "system" fn DiskEnableInfoExceptions(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    Enable: u8,
) -> NTSTATUS {
    PAGED_CODE!();

    let diskData: *mut DISK_DATA =
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as *mut DISK_DATA;
    let mut status: NTSTATUS = STATUS_NOT_SUPPORTED;
    let mut modeDataSize: u32 = MODE_DATA_SIZE;

    let modeData: *mut MODE_PARAMETER_HEADER = ExAllocatePoolWithTag(
        (NonPagedPoolNxCacheAligned) as u32,
        (modeDataSize) as usize,
        0x78456944u32,
    ) as *mut MODE_PARAMETER_HEADER;

    if modeData.is_null() {
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_WMI,
            b"DiskEnableInfoExceptions: Unable to allocate mode data buffer\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut pageData: *mut MODE_INFO_EXCEPTIONS = core::ptr::null_mut();
    status = DiskGetModePage(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        MODE_PAGE_FAULT_REPORTING,
        1,
        modeData,
        &mut modeDataSize as *mut u32,
        &mut pageData as *mut *mut MODE_INFO_EXCEPTIONS as *mut *mut core::ffi::c_void,
    );

    if !NT_SUCCESS(status) || pageData.is_null() {
        TracePrint(
            (TRACE_LEVEL_INFORMATION) as i32,
            TRACE_FLAG_WMI,
            b"DiskEnableInfoExceptions: does NOT support SMART for device %p\n\0".as_ptr() as *const i8,
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );
        FREE_POOL(modeData as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_WMI,
        b"DiskEnableInfoExceptions: DOES support SMART for device %p\n\0".as_ptr() as *const i8,
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
    );

    if (*pageData).__bindgen_anon_1.__bindgen_anon_1.as_ref().Dexcpt() == 0 {
        TracePrint(
            (TRACE_LEVEL_INFORMATION) as i32,
            TRACE_FLAG_WMI,
            b"DiskEnableInfoExceptions: does NOT support DEXCPT bit for device %p\n\0".as_ptr() as *const i8,
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );
        FREE_POOL(modeData as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    let mut changeablePageData: MODE_INFO_EXCEPTIONS = core::mem::zeroed();
    RtlCopyMemory(
        &mut changeablePageData as *mut MODE_INFO_EXCEPTIONS as *mut core::ffi::c_void,
        pageData as *const MODE_INFO_EXCEPTIONS as *const core::ffi::c_void,
        core::mem::size_of::<MODE_INFO_EXCEPTIONS>(),
    );

    status = DiskGetModePage(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        MODE_PAGE_FAULT_REPORTING,
        0,
        modeData,
        &mut modeDataSize as *mut u32,
        &mut pageData as *mut *mut MODE_INFO_EXCEPTIONS as *mut *mut core::ffi::c_void,
    );

    if !NT_SUCCESS(status) || pageData.is_null() {
        NT_ASSERT(((NT_SUCCESS(status) as i32) != 0 && !pageData.is_null()) as i32);
        FREE_POOL(modeData as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    if (*pageData).ReportMethod() == 0 && changeablePageData.ReportMethod() == 0 {
        TracePrint(
            (TRACE_LEVEL_INFORMATION) as i32,
            TRACE_FLAG_WMI,
            b"DiskEnableInfoExceptions: MRIE field is 0 and is not changeable for device %p\n\0".as_ptr() as *const i8,
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        );
        FREE_POOL(modeData as *mut core::ffi::c_void);
        return STATUS_NOT_SUPPORTED;
    }

    if changeablePageData.__bindgen_anon_1.__bindgen_anon_1.as_ref().Perf() != 0 {
        (*pageData).__bindgen_anon_1.__bindgen_anon_1.as_mut().set_Perf(if (*diskData).AllowFPPerfHit != 0 { 0 } else { 1 });
    }

    if changeablePageData.ReportMethod() != 0 {
        (*pageData).set_ReportMethod(4);
    }

    (*pageData).__bindgen_anon_1.__bindgen_anon_1.as_mut().set_Dexcpt(if Enable == 0 { 1 } else { 0 });

    status = ClassModeSelect(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        modeData as *const i8,
        modeDataSize,
        (*pageData).PSBit() as u8,
    );

    if NT_SUCCESS(status) {
        if Enable != 0 {
            (*diskData).FailurePredictionCapability = FailurePredictionSense;
            (*diskData).FailurePredictionEnabled = 1;
        } else {
            (*diskData).FailurePredictionCapability = FailurePredictionNone;
            (*diskData).FailurePredictionEnabled = 0;
        }
    }

    FREE_POOL(modeData as *mut core::ffi::c_void);
    status
}



#[no_mangle]
pub unsafe extern "C" fn DiskEnableDisableFailurePrediction(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Enable: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let commonExtension: *mut COMMON_DEVICE_EXTENSION =
        &raw mut (*FdoExtension).__bindgen_anon_1 as *mut _ as *mut COMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;

    // PAGED_CODE();
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8 as *const i8,
                1152i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    match (*diskData).FailurePredictionCapability as u32 {
        2 => {
            // FailurePredictionSmart
            if Enable != 0 {
                status = DiskEnableSmart(FdoExtension);
            } else {
                status = DiskDisableSmart(FdoExtension);
            }
            if status >= 0 {
                (*diskData).FailurePredictionEnabled = Enable;
            }
        }
        3 | 1 => {
            // FailurePredictionSense, FailurePredictionIoctl
            status = STATUS_SUCCESS;
        }
        _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DiskEnableDisableFailurePredictPolling(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    Enable: BOOLEAN,
    PollTimeInSeconds: ULONG,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0".as_ptr() as *const i8,
                1222i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        } else {
        }
    }
    if Enable != 0 {
        status = DiskEnableDisableFailurePrediction(FdoExtension, Enable);
    } else {
        status = STATUS_SUCCESS;
    }
    if NT_SUCCESS(status) {
        status = ClassSetFailurePredictionPoll(
            FdoExtension,
            if Enable != 0 {
                (*diskData).FailurePredictionCapability
            } else {
                FailurePredictionNone as FAILURE_PREDICTION_METHOD
            },
            PollTimeInSeconds,
        );
    }
    return status;
}

#[no_mangle]
pub unsafe extern "C" fn DiskReadFailurePredictStatus(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    DiskSmartStatus: PSTORAGE_FAILURE_PREDICT_STATUS,
) -> NTSTATUS {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let mut status: NTSTATUS = 0;
    // PAGED_CODE assert block
    if (KeGetCurrentIrql() as i32) > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8 as *const i8,
                1277i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }
    (*DiskSmartStatus).PredictFailure = 0u8; // FALSE
    match (*diskData).FailurePredictionCapability {
        2 => {
            let mut outBuffer: [u8; 61] = [0u8; 61];
            let mut outBufferSize: u32 = 61u32;
            let mut cmdOutParameters: PSENDCMDOUTPARAMS = core::ptr::null_mut();
            status = DiskPerformSmartCommand(
                FdoExtension,
                IOCTL_SCSI_MINIPORT_RETURN_STATUS,
                SMART_CMD,
                RETURN_SMART_STATUS,
                0u8,
                0u8,
                &raw mut outBuffer as *mut u8 as PSRB_IO_CONTROL,
                &raw mut outBufferSize,
            );
            if status >= 0 {
                cmdOutParameters = (&raw mut outBuffer as *mut u8)
                    .offset(core::mem::size_of::<SRB_IO_CONTROL>() as isize)
                    as PSENDCMDOUTPARAMS;
                (*DiskSmartStatus).Reason = 0u32;
                (*DiskSmartStatus).PredictFailure = ((*(&raw mut (*cmdOutParameters).bBuffer as *mut u8).offset(3isize) as u8 == 0xf4u8) &&
                                                      (*(&raw mut (*cmdOutParameters).bBuffer as *mut u8).offset(4isize) as u8 == 0x2cu8)) as u8;
            }
        }
        3 => {
            (*DiskSmartStatus).Reason = (*FdoExtension).FailureReason;
            (*DiskSmartStatus).PredictFailure = (*FdoExtension).FailurePredicted;
            status = STATUS_SUCCESS;
        }
        1 | 0 | _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DiskReadFailurePredictData(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    DiskSmartData: *mut STORAGE_FAILURE_PREDICT_DATA,
) -> NTSTATUS {
    // PAGED_CODE
    const APC_LEVEL: i32 = 1;
    if (KeGetCurrentIrql() as i32) > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    .as_ptr() as *const i8,
                1354 as i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    let commonExtension = FdoExtension as *mut COMMON_DEVICE_EXTENSION;
    let diskData = ((*commonExtension).DriverData) as *mut DISK_DATA;
    let mut status: NTSTATUS = 0;

    const FailurePredictionSmart: u32 = 2;
    const FailurePredictionSense: u32 = 3;

    match (*diskData).FailurePredictionCapability as u32  {
        FailurePredictionSmart => {
            let mut outBuffer: *mut u8 = core::ptr::null_mut();
            let mut outBufferSize: u32 = 0;
            let mut cmdOutParameters: *mut SENDCMDOUTPARAMS = core::ptr::null_mut();
            const READ_ATTRIBUTE_BUFFER_SIZE: usize = 512;
            outBufferSize = (core::mem::size_of::<SRB_IO_CONTROL>() as usize).wrapping_add(
                if core::mem::size_of::<SENDCMDINPARAMS>() as usize
                    > (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
                        .wrapping_sub(1)
                        .wrapping_add(READ_ATTRIBUTE_BUFFER_SIZE)
                {
                    core::mem::size_of::<SENDCMDINPARAMS>()
                } else {
                    (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
                        .wrapping_sub(1)
                        .wrapping_add(READ_ATTRIBUTE_BUFFER_SIZE)
                },
            ) as u32;

            outBuffer = ExAllocatePoolWithTag(
                0, // NonPagedPool (unknown constant)
                outBufferSize as usize,
                // disk.h DISK_TAG_SMART ('aDcS'); 0 is PoolHitTag's default and breaks into
                // the pool tracker's DbgBreakPoint.
                DISK_TAG_SMART,
            ) as *mut u8;

            if !outBuffer.is_null() {
                status = DiskPerformSmartCommand(
                    FdoExtension,
                    // DiskReadSmartData(): this ControlCode was 0
                    IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS,
                    SMART_CMD,
                    READ_ATTRIBUTES,
                    0u8,
                    0u8,
                    outBuffer as *mut SRB_IO_CONTROL,
                    &raw mut outBufferSize,
                );
                if status >= 0 {
                    cmdOutParameters = outBuffer
                        .offset(core::mem::size_of::<SRB_IO_CONTROL>() as isize)
                        as *mut SENDCMDOUTPARAMS;
                    (*DiskSmartData).Length = READ_ATTRIBUTE_BUFFER_SIZE as u32;
                    ntkrnl_sys::rtl::RtlCopyMemory(
                        core::ptr::addr_of_mut!((*DiskSmartData).VendorSpecific) as *mut ::core::ffi::c_void,
                        core::ptr::addr_of!((*cmdOutParameters).bBuffer) as *const ::core::ffi::c_void,
                        if READ_ATTRIBUTE_BUFFER_SIZE < core::mem::size_of::<[u8; 512]>() {
                            READ_ATTRIBUTE_BUFFER_SIZE
                        } else {
                            core::mem::size_of::<[u8; 512]>()
                        } as usize,
                    );
                }
                ExFreePool(outBuffer as *mut ::core::ffi::c_void);
                outBuffer = core::ptr::null_mut();
            } else {
                status = ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES;
            }
        }
        FailurePredictionSense => {
            (*DiskSmartData).Length = core::mem::size_of::<u32>() as u32;
            *((&raw mut (*DiskSmartData).VendorSpecific) as *mut u8 as *mut u32) =
                (*FdoExtension).FailureReason as u32;
            status = ntkrnl_sys::base::STATUS_SUCCESS;
        }
        _ => {
            status = 0xC0000010u32 as NTSTATUS;
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn DiskReadFailurePredictThresholds(
    mut FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    mut DiskSmartThresholds: *mut STORAGE_FAILURE_PREDICT_THRESHOLDS,
) -> NTSTATUS {
    let mut commonExtension = FdoExtension as PCOMMON_DEVICE_EXTENSION;
    let mut diskData = (*commonExtension).DriverData as PDISK_DATA;
    let mut status: NTSTATUS = 0;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0" as *const u8 as *const i8,
                1444i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    match (*diskData).FailurePredictionCapability as u32 {
        2 => {
            // FailurePredictionSmart
            let mut outBuffer: *mut u8 = core::ptr::null_mut();
            let mut cmdOutParameters: PSENDCMDOUTPARAMS = core::ptr::null_mut();
            let mut outBufferSize: u32 = 0;

            outBufferSize = (core::mem::size_of::<SRB_IO_CONTROL>() as usize).wrapping_add(
                if core::mem::size_of::<SENDCMDINPARAMS>() as usize
                    > (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
                        .wrapping_sub(1usize)
                        .wrapping_add(512usize)
                {
                    core::mem::size_of::<SENDCMDINPARAMS>() as usize
                } else {
                    (core::mem::size_of::<SENDCMDOUTPARAMS>() as usize)
                        .wrapping_sub(1usize)
                        .wrapping_add(512usize)
                },
            ) as u32;

            outBuffer = ExAllocatePoolWithTag(
                NonPagedPool,
                outBufferSize as SIZE_T,
                DISK_TAG_SMART,
            ) as *mut u8;

            if !outBuffer.is_null() {
                status = DiskPerformSmartCommand(
                    FdoExtension,
                    IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS,
                    SMART_CMD,
                    READ_THRESHOLDS,
                    0u8,
                    0u8,
                    outBuffer as PSRB_IO_CONTROL,
                    &raw mut outBufferSize,
                );

                if status >= 0 {
                    cmdOutParameters = outBuffer
                        .add(core::mem::size_of::<SRB_IO_CONTROL>() as usize)
                        as PSENDCMDOUTPARAMS;

                    core::ptr::copy_nonoverlapping(
                        &raw mut (*cmdOutParameters).bBuffer as *mut u8 as *const u8,
                        &raw mut (*DiskSmartThresholds).VendorSpecific as *mut u8,
                        if 512usize < core::mem::size_of::<[u8; 512]>() as usize {
                            512usize
                        } else {
                            core::mem::size_of::<[u8; 512]>() as usize
                        },
                    );
                }

                if !outBuffer.is_null() {
                    ExFreePool(outBuffer as *mut core::ffi::c_void);
                    outBuffer = core::ptr::null_mut();
                }
            } else {
                status = STATUS_INSUFFICIENT_RESOURCES;
            }
        }
        3 | 1 | 0 | _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    return status;
}


#[no_mangle]
pub unsafe extern "system" fn DiskReregWorker(
    mut DevObject: PDEVICE_OBJECT,
    Context: *mut core::ffi::c_void,
) {
    let mut reregRequest: *mut DISKREREGREQUEST = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut irp: PIRP = core::ptr::null_mut();

    // PAGED_CODE(); expanded
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0" as *const u8 as *const i8,
                1509i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    // NT_ASSERT(Context != NULL)
    if Context.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
            b"drivers/storage/class/disk/diskwmi.c\0" as *const u8 as *const i8,
            1512i32,
            b"Context != NULL\0" as *const u8 as *const i8,
        );
        __int2c();
    }

    loop {
        reregRequest = ExInterlockedPopEntryList(
            &raw mut DiskReregHead as *mut _,
            // `as PKSPIN_LOCK`, not `as *mut u64`: the substrate spells a KSPIN_LOCK
            // ULONG-wide per arch (u32 on i386), while this crate's static is the
            // ULONG_PTR spelling. Same object and same width; cast to the type the
            // declaration actually names rather than to a fixed 64 bits.
            &raw mut DiskReregSpinlock as ntkrnl_sys::ddk::PKSPIN_LOCK,
        ) as *mut DISKREREGREQUEST;

        if !reregRequest.is_null() {
            deviceObject = (*reregRequest).DeviceObject;
            irp = (*reregRequest).Irp;

            status = IoWMIRegistrationControl(deviceObject, WMIREG_ACTION_UPDATE_GUIDS as u32);

            ClassReleaseRemoveLock(deviceObject, irp);
            IoFreeMdl((*irp).MdlAddress as *mut MDL);
            IoFreeIrp(irp);

            // FREE_POOL macro
            if !reregRequest.is_null() {
                ExFreePool(reregRequest as *mut core::ffi::c_void);
                reregRequest = core::ptr::null_mut();
            }
        } else {
            // NT_ASSERTMSG("Disk Re-registration request list should not be empty", FALSE);
            if 0 == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0" as *const u8 as *const i8,
                    1542i32,
                    b"Disk Re-registration request list should not be empty\0" as *const u8 as *const i8,
                );
                __int2c();
            }
            status = STATUS_INTERNAL_ERROR;
        }

        // if (!NT_SUCCESS(status))
        if !(status >= 0) {
            StorDebugPrint(
                2, // TRACE_LEVEL_ERROR (assuming 2)
                TRACE_FLAG_GENERAL,
                b"DiskReregWorker: Reregistration failed %x\n\0" as *const u8 as *const i8,
                status,
            );
        }

        // while (InterlockedDecrement(&DiskReregWorkItems))
        if InterlockedDecrement(&raw mut DiskReregWorkItems as *mut i32) == 0 {
            break;
        }
    }

    IoFreeWorkItem(Context as PIO_WORKITEM);
}





#[no_mangle]
pub unsafe extern "C" fn DiskInitializeReregistration() -> NTSTATUS {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/disk/diskwmi.c\0".as_ptr() as *const i8,
                1564i32,
                b"0\0".as_ptr() as PCSTR,
            );
            __int2c();
        }
    }
    KeInitializeSpinLock((&raw mut DiskReregSpinlock).cast());
    0i32
}

#[no_mangle]
pub unsafe extern "system" fn DiskPostReregisterRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut reregRequest: PDISKREREGREQUEST = core::ptr::null_mut::<DISKREREGREQUEST>();
    let mut workItem: PIO_WORKITEM = core::ptr::null_mut::<_IO_WORKITEM>();
    let mut status: NTSTATUS = 0;
    workItem = IoAllocateWorkItem(DeviceObject);
    if workItem.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    reregRequest = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        core::mem::size_of::<DISKREREGREQUEST>() as SIZE_T,
        DISK_TAG_SMART as u32,
    ) as PDISKREREGREQUEST;
    if !reregRequest.is_null() {
        (*reregRequest).DeviceObject = DeviceObject;
        (*reregRequest).Irp = Irp;
        ExInterlockedPushEntryList(
            &raw mut DiskReregHead as *mut _,
            &raw mut (*reregRequest).Next as *mut _,
            &raw mut DiskReregSpinlock as ntkrnl_sys::ddk::PKSPIN_LOCK,
        );
        if InterlockedIncrement(&raw mut DiskReregWorkItems) == 1 {
            IoQueueWorkItem(
                workItem,
                core::mem::transmute::<Option<unsafe extern "system" fn(PDEVICE_OBJECT, PVOID)>, PIO_WORKITEM_ROUTINE>(Some(DiskReregWorker as unsafe extern "system" fn(PDEVICE_OBJECT, PVOID))),
                _WORK_QUEUE_TYPE_DelayedWorkQueue,
                workItem as PVOID,
            );
        } else {
            IoFreeWorkItem(workItem);
        }
        status = STATUS_SUCCESS;
    } else {
        IoFreeWorkItem(workItem);
        TracePrint(
            (TRACE_LEVEL_ERROR) as i32,
            TRACE_FLAG_GENERAL as u32,
            b"DiskPostReregisterRequest: could not allocate reregRequest for %p\n\0".as_ptr()
                as *const i8,
            DeviceObject,
        );
        status = STATUS_INSUFFICIENT_RESOURCES;
    }
    return status;
}






#[no_mangle]
pub unsafe extern "system" fn DiskInfoExceptionComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    // Forward declarations for unknown externs used in this function

    // Constants not provided
    const SRB_STATUS_AUTOSENSE_VALID: i32 = 0x80;
    const SRB_STATUS_QUEUE_FROZEN: i32 = 0x40;
    // 0x01, ddk/srb.h:115 -- NOT 0. This was fabricated as 0 under the "Constants not
    // provided" comment above, and it is used as `if srbStatus != SRB_STATUS_SUCCESS`,
    // so the error path ran on every SUCCESSFUL request and SRB_STATUS_PENDING (0) was
    // read as success. The eleven other files that define this name all say 0x1.
    const SRB_STATUS_SUCCESS: u32 = 0x01;
    const SRB_STATUS_DATA_OVERRUN: u32 = 0x12;
    // ddk/srb.h:108 == 0x28 (see the write site above).
    const SRB_FUNCTION_STORAGE_REQUEST_BLOCK: i32 = 0x28;
    const SRB_FUNCTION_EXECUTE_SCSI: u32 = 0x00;
    const SRB_SIMPLE_TAG_REQUEST: u16 = 0x20;
    // ddk/srb.h:55 -- (UCHAR)~0 is 0xFF, not a 32-bit ~0.
    const SP_UNTAGGED: u32 = 0xFF;
    const MAXIMUM_RETRIES: u32 = 3;
    const STATUS_VERIFY_REQUIRED: NTSTATUS = 0x80000016u32 as i32;
    const STATUS_IO_DEVICE_ERROR: NTSTATUS = 0xC0000185u32 as i32 ;
    const IRP_MJ_SCSI: u8 = 0x0F;
    const MODE_PAGE_FAULT_REPORTING: u8 = 0x1C;
    const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
    const TRACE_LEVEL_WARNING: u32 = 3; // storswtr.h:26-30
    const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const TRACE_FLAG_WMI: u32 = 6; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let nextIrpStack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    let mut srb: *mut SCSI_REQUEST_BLOCK = Context as *mut SCSI_REQUEST_BLOCK;
    let mut status: NTSTATUS = 0;
    let mut retry: BOOLEAN = 0;
    let mut retryInterval: ULONG = 0;
    let srbStatus: ULONG;
    let mut freeLockAndIrp: BOOLEAN = 1;
    let mut originalSenseInfoBuffer: PVOID = (*irpStack).Parameters.Others.Argument3;
    let mut srbEx: PSTORAGE_REQUEST_BLOCK = core::ptr::null_mut::<STORAGE_REQUEST_BLOCK>();
    let mut dataBuffer: PVOID = core::ptr::null_mut();
    let mut dataLength: ULONG = 0;
    let mut senseBuffer: PVOID = core::ptr::null_mut();
    let mut cdbLength8: UCHAR = 0;
    let mut cdbLength32: ULONG = 0;
    let mut senseBufferLength: UCHAR = 0;

    srbStatus = ((*srb).SrbStatus as i32 & !(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as ULONG;

    if (*srb).Function as i32 == SRB_FUNCTION_STORAGE_REQUEST_BLOCK {
        srbEx = srb as PSTORAGE_REQUEST_BLOCK;
        dataBuffer = (*srbEx).DataBuffer;
        dataLength = (*srbEx).DataTransferLength;
        if (*srbEx).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI as u32
            && (*srbEx).NumSrbExData > 0
        {
            GetSrbScsiData(
                srbEx,
                &raw mut cdbLength8,
                &raw mut cdbLength32,
                core::ptr::null_mut::<u8>(),
                &raw mut senseBuffer,
                &raw mut senseBufferLength,
            );
        }
    } else {
        dataBuffer = (*srb).DataBuffer;
        dataLength = (*srb).DataTransferLength;
        senseBuffer = (*srb).SenseInfoBuffer;
    }

    // Check SRB status for success of completing request.
    // SRB_STATUS_DATA_OVERRUN also indicates success.
    if srbStatus != SRB_STATUS_SUCCESS && srbStatus != SRB_STATUS_DATA_OVERRUN {
        TracePrint(
            (TRACE_LEVEL_INFORMATION) as i32,
            TRACE_FLAG_GENERAL,
            b"DiskInfoExceptionComplete: IRP %p, SRB %p\n\0".as_ptr() as *const i8,
            Irp,
            srb,
        );

        if TEST_FLAG((*srb).SrbStatus, SRB_STATUS_QUEUE_FROZEN as u8) {
            ClassReleaseQueue(DeviceObject);
        }

        retry = ClassInterpretSenseInfo(
            DeviceObject,
            srb,
            (*irpStack).MajorFunction,
            0,
            (MAXIMUM_RETRIES)
                .wrapping_sub((*irpStack).Parameters.Others.Argument4 as usize as ULONG),
            &raw mut status,
            &raw mut retryInterval,
        );

        if TEST_FLAG((*irpStack).Flags, 0x2) && status == STATUS_VERIFY_REQUIRED {
            status = STATUS_IO_DEVICE_ERROR;
            retry = 1;
        }

        retry = (retry != 0 && !(*irpStack).Parameters.Others.Argument4.is_null()) as BOOLEAN;

        (*irpStack).Parameters.Others.Argument4 =
            ((*irpStack).Parameters.Others.Argument4 as usize).wrapping_sub(1) as PVOID;

        if retry != 0 {
            TracePrint(
                (TRACE_LEVEL_INFORMATION) as i32,
                TRACE_FLAG_GENERAL,
                b"DiskInfoExceptionComplete: Retry request %p\n\0".as_ptr() as *const i8,
                Irp,
            );

            NT_ASSERT((dataBuffer == ((*(*Irp).MdlAddress).StartVa as *mut u8).add((*(*Irp).MdlAddress).ByteOffset as usize) as PVOID) as i32);

            if (*srb).Function as i32 == SRB_FUNCTION_STORAGE_REQUEST_BLOCK {
                (*srbEx).DataTransferLength = (*(*Irp).MdlAddress).ByteCount;
                (*srbEx).SrbStatus = 0;
                if (*srbEx).SrbFunction == SRB_FUNCTION_EXECUTE_SCSI as u32
                    && (*srbEx).NumSrbExData > 0
                {
                    SetSrbScsiData(srbEx, cdbLength8, cdbLength32, 0 as UCHAR, senseBuffer, senseBufferLength);
                }

                SET_FLAG(&mut (*srbEx).SrbFlags, 0x4u32);
                SET_FLAG(&mut (*srbEx).SrbFlags, 0x8u32);
                CLEAR_FLAG(&mut (*srbEx).SrbFlags, 0x2u32);

                (*srbEx).RequestAttribute = (SRB_SIMPLE_TAG_REQUEST as u8) as u16;
                (*srbEx).RequestTag = (SP_UNTAGGED as u8) as u32;
            } else {
                (*srb).DataTransferLength = (*(*Irp).MdlAddress).ByteCount;
                (*srb).ScsiStatus = 0;
                (*srb).SrbStatus = (*srb).ScsiStatus;

                SET_FLAG(&mut (*srb).SrbFlags, 0x4u32);
                SET_FLAG(&mut (*srb).SrbFlags, 0x8u32);
                CLEAR_FLAG(&mut (*srb).SrbFlags, 0x2u32);

                (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
                (*srb).QueueTag = (SP_UNTAGGED) as u8;
            }

            (*nextIrpStack).MajorFunction = IRP_MJ_SCSI;
            (*nextIrpStack).Parameters.Scsi.Srb = srb as *mut _;

            IoSetCompletionRoutine(
                Irp,
                core::mem::transmute::<Option<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS>, PIO_COMPLETION_ROUTINE>(Some(DiskInfoExceptionComplete as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS)),
                srb as PVOID,
                1,
                1,
                1,
            );

            IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
            return 0xC0000016u32 as i32 ; // STATUS_MORE_PROCESSING_REQUIRED
        }
    } else {
        let mut pageData: *mut MODE_INFO_EXCEPTIONS = core::ptr::null_mut();
        let mut modeData: *mut MODE_PARAMETER_HEADER;
        let mut modeDataLength: ULONG;

        modeData = dataBuffer as *mut MODE_PARAMETER_HEADER;
        modeDataLength = dataLength;

        pageData = ClassFindModePage(
            modeData as *mut i8,
            modeDataLength,
            MODE_PAGE_FAULT_REPORTING,
            1,
        ) as *mut MODE_INFO_EXCEPTIONS;

        if !pageData.is_null() {
            TracePrint(
                (TRACE_LEVEL_INFORMATION) as i32,
                TRACE_FLAG_WMI,
                b"DiskInfoExceptionComplete: %p supports SMART\n\0".as_ptr() as *const i8,
                DeviceObject,
            );

            (*diskData).ScsiInfoExceptionsSupported = 1;

            if (*pageData).__bindgen_anon_1.__bindgen_anon_1.as_ref().Dexcpt() == 0
                && (*pageData).ReportMethod() >= 2
                && (*pageData).ReportMethod() <= 6
            {
                (*diskData).FailurePredictionCapability = FailurePredictionSense;
                (*diskData).FailurePredictionEnabled = 1;
                status = DiskPostReregisterRequest(DeviceObject, Irp);

                if status >= 0 {
                    freeLockAndIrp = 0;
                }
            } else {
                TracePrint(
                    (TRACE_LEVEL_WARNING) as i32,
                    TRACE_FLAG_WMI,
                    b"DiskInfoExceptionComplete: %p is not enabled for SMART\n\0".as_ptr() as *const i8,
                    DeviceObject,
                );
            }
        } else {
            TracePrint(
                (TRACE_LEVEL_WARNING) as i32,
                TRACE_FLAG_WMI,
                b"DiskInfoExceptionComplete: %p does not supports SMART\n\0".as_ptr() as *const i8,
                DeviceObject,
            );
        }

        status = 0; // STATUS_SUCCESS
    }

    // Free resources
    if senseBuffer != originalSenseInfoBuffer {
        if !originalSenseInfoBuffer.is_null() {
            FREE_POOL(originalSenseInfoBuffer);
            originalSenseInfoBuffer = core::ptr::null_mut(); // not really needed but anchor does it
        }
    }
    if !senseBuffer.is_null() {
        FREE_POOL(senseBuffer);
        senseBuffer = core::ptr::null_mut();
    }
    if !dataBuffer.is_null() {
        FREE_POOL(dataBuffer);
        dataBuffer = core::ptr::null_mut();
    }
    if !srb.is_null() {
        FREE_POOL(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
    }

    if freeLockAndIrp != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;

        if (*Irp).PendingReturned != 0 {
            IoMarkIrpPending(Irp);
        }

        ClassReleaseRemoveLock(DeviceObject, Irp);
        IoFreeMdl((*Irp).MdlAddress);
        IoFreeIrp(Irp);
    }

    0xC0000016u32 as i32  // STATUS_MORE_PROCESSING_REQUIRED
}





#[no_mangle]
pub unsafe extern "system" fn DiskInfoExceptionCheck(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut modeData: *mut u8 = core::ptr::null_mut();
    let mut srb: *mut SCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut cdb: *mut CDB = core::ptr::null_mut();
    let mut irp: *mut IRP = core::ptr::null_mut();
    let mut irpStack: *mut IO_STACK_LOCATION = core::ptr::null_mut();
    let mut senseInfoBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut senseInfoBufferLength: u8 = 0;
    let mut isRemoved: u32 = 0;
    let mut srbSize: u32 = 0;
    let mut srbEx: *mut STORAGE_REQUEST_BLOCK = core::ptr::null_mut();
    let mut storAddrBtl8: *mut STOR_ADDR_BTL8 = core::ptr::null_mut();
    let mut srbExDataCdb16: *mut SRBEX_DATA_SCSI_CDB16 = core::ptr::null_mut();

    modeData = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned as u32,
        MODE_DATA_SIZE as usize,
        DISK_TAG_INFO_EXCEPTION,
    ) as *mut u8;
    if modeData.is_null() {
        StorDebugPrint(
            3,
            TRACE_FLAG_WMI,
            b"DiskInfoExceptionCheck: Can't allocate mode data buffer\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*(*FdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
    } else {
        srbSize = SCSI_REQUEST_BLOCK_SIZE;
    }

    srb = ExAllocatePoolWithTag(NonPagedPool as u32, srbSize as usize, DISK_TAG_SRB)
        as *mut SCSI_REQUEST_BLOCK;
    if srb.is_null() {
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
        StorDebugPrint(
            3,
            TRACE_FLAG_WMI,
            b"DiskInfoExceptionCheck: Can't allocate srb buffer\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(srb as *mut u8, 0, srbSize as usize);

    senseInfoBuffer = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned as u32,
        SENSE_BUFFER_SIZE_EX as usize,
        927163219u32,
    );
    if senseInfoBuffer.is_null() {
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
        StorDebugPrint(
            3,
            TRACE_FLAG_WMI,
            b"DiskInfoExceptionCheck: Can't allocate request sense buffer\n\0".as_ptr()
                as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    senseInfoBufferLength = SENSE_BUFFER_SIZE_EX as u8;

    irp = IoAllocateIrp(
        ((*(*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject).StackSize as i32
            + 1) as i8,
        0 as u8,
    );
    if irp.is_null() {
        ExFreePool(senseInfoBuffer);
        senseInfoBuffer = core::ptr::null_mut();
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
        StorDebugPrint(
            3,
            TRACE_FLAG_WMI,
            b"DiskInfoExceptionCheck: Can't allocate Irp\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    isRemoved = ClassAcquireRemoveLockEx(
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        (irp).cast(),
        b"drivers/storage/class/disk/diskwmi.c\0"
            .as_ptr() as *const i8,
        2003u32,
    );
    if isRemoved != 0 {
        ClassReleaseRemoveLock(
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
            irp,
        );
        IoFreeIrp(irp);
        ExFreePool(senseInfoBuffer);
        senseInfoBuffer = core::ptr::null_mut();
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
        StorDebugPrint(
            3,
            TRACE_FLAG_WMI,
            b"DiskInfoExceptionCheck: RemoveLock says isRemoved\n\0".as_ptr() as *const i8,
        );
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    IoSetNextIrpStackLocation(irp);
    irpStack = IoGetCurrentIrpStackLocation(irp);
    (*irpStack).DeviceObject = (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject;
    (*irpStack).Parameters.Others.Argument4 = MAXIMUM_RETRIES as *mut core::ffi::c_void;
    (*irpStack).Parameters.Others.Argument3 = senseInfoBuffer;
    irpStack = IoGetNextIrpStackLocation(irp);
    (*irpStack).MajorFunction = IRP_MJ_SCSI as u8;
    (*irpStack).Parameters.Scsi.Srb = srb as *mut _;
    IoSetCompletionRoutine(
        irp,
        core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP, *mut core::ffi::c_void) -> i32>, PIO_COMPLETION_ROUTINE>(
            Some(DiskInfoExceptionComplete as unsafe extern "system" fn(
                *mut ntkrnl_sys::ddk::_DEVICE_OBJECT,
                *mut ntkrnl_sys::ddk::_IRP,
                *mut core::ffi::c_void,
            ) -> i32)),
        srb as *mut core::ffi::c_void,
        1 as u8,
        1 as u8,
        1 as u8,
    );

    (*irp).MdlAddress = IoAllocateMdl(
        modeData as *mut core::ffi::c_void,
        MODE_DATA_SIZE as u32,
        0 as u8,
        0 as u8,
        irp,
    );
    if (*irp).MdlAddress.is_null() {
        ClassReleaseRemoveLock(
            (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
            irp,
        );
        ExFreePool(srb as *mut core::ffi::c_void);
        srb = core::ptr::null_mut();
        ExFreePool(modeData as *mut core::ffi::c_void);
        modeData = core::ptr::null_mut();
        ExFreePool(senseInfoBuffer);
        senseInfoBuffer = core::ptr::null_mut();
        IoFreeIrp(irp);
        StorDebugPrint(
            3,
            TRACE_FLAG_WMI,
            b"DiskINfoExceptionCheck: Can't allocate MDL\n\0".as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    MmBuildMdlForNonPagedPool((*irp).MdlAddress);

    if (*(*FdoExtension).AdapterDescriptor).SrbType as i32 == (SRB_TYPE_STORAGE_REQUEST_BLOCK) as i32 {
        srbEx = srb as *mut STORAGE_REQUEST_BLOCK;
        (*srbEx).Length = (core::mem::offset_of!(STORAGE_REQUEST_BLOCK, Signature) as u8) as u16;
        (*srbEx).Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK as u8;
        (*srbEx).Signature = SRB_SIGNATURE;
        (*srbEx).Version = (STORAGE_REQUEST_BLOCK_VERSION_1 as u8) as u32;
        (*srbEx).SrbLength = (srbSize as u8) as u32;
        (*srbEx).SrbFunction = (SRB_FUNCTION_EXECUTE_SCSI as u8) as u32;
        (*srbEx).RequestPriority = (IoGetIoPriorityHint(irp) as u8) as u16;
        (*srbEx).AddressOffset = core::mem::size_of::<STORAGE_REQUEST_BLOCK>() as u32;
        (*srbEx).NumSrbExData = 1u32;
        (*srbEx).TimeOutValue = (*FdoExtension).TimeOutValue;
        (*srbEx).DataTransferLength = MODE_DATA_SIZE as u32;
        (*srbEx).DataBuffer = modeData as *mut core::ffi::c_void;
        (*srbEx).SrbFlags = (*FdoExtension).SrbFlags;
        (*srbEx).SrbFlags |= 0x40u32;
        (*srbEx).SrbFlags |= 0x8u32;
        (*srbEx).SrbFlags |= 0x100u32;
        (*srbEx).RequestAttribute = (SRB_SIMPLE_TAG_REQUEST as u8) as u16;
        (*srbEx).RequestTag = (SP_UNTAGGED as u8) as u32;
        (*srbEx).OriginalRequest = (irp).cast();

        storAddrBtl8 =
            (srbEx as *mut u8).offset((*srbEx).AddressOffset as isize) as *mut STOR_ADDR_BTL8;
        (*storAddrBtl8).Type = STOR_ADDRESS_TYPE_BTL8 as u16;
        (*storAddrBtl8).AddressLength = (STOR_ADDR_BTL8_ADDRESS_LENGTH as u16) as u32;

        (*srbEx).SrbExDataOffset[0] = (core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
            + core::mem::size_of::<STOR_ADDR_BTL8>()) as u32;

        if (*srbEx).SrbExDataOffset[0] as usize + core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>()
            <= (*srbEx).SrbLength as usize
        {
            srbExDataCdb16 = (srbEx as *mut u8)
                .offset((*srbEx).SrbExDataOffset[0] as isize)
                as *mut SRBEX_DATA_SCSI_CDB16;
            (*srbExDataCdb16).Type = SrbExDataTypeScsiCdb16;
            (*srbExDataCdb16).Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
            (*srbExDataCdb16).CdbLength = 6u8;
            (*srbExDataCdb16).SenseInfoBufferLength = senseInfoBufferLength;
            (*srbExDataCdb16).SenseInfoBuffer = senseInfoBuffer;
            cdb = &raw mut (*srbExDataCdb16).Cdb as *mut u8 as *mut CDB;
        } else {
            if 0i32 == 0  {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        .as_ptr() as *const i8,
                    2144i32,
                    b"FALSE\0".as_ptr() as *const i8,
                );
                __int2c();
            }
            ClassReleaseRemoveLock(
                (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                irp,
            );
            ExFreePool(srb as *mut core::ffi::c_void);
            srb = core::ptr::null_mut();
            ExFreePool(modeData as *mut core::ffi::c_void);
            modeData = core::ptr::null_mut();
            ExFreePool(senseInfoBuffer);
            senseInfoBuffer = core::ptr::null_mut();
            IoFreeIrp(irp);
            StorDebugPrint(
                3,
                TRACE_FLAG_WMI,
                b"DiskINfoExceptionCheck: Insufficient extended SRB size\n\0".as_ptr()
                    as *const i8,
            );
            return STATUS_INTERNAL_ERROR;
        }
    } else {
        (*srb).Length = SCSI_REQUEST_BLOCK_SIZE as u16;
        (*srb).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
        (*srb).SenseInfoBufferLength = senseInfoBufferLength;
        (*srb).SenseInfoBuffer = senseInfoBuffer;
        (*srb).TimeOutValue = (*FdoExtension).TimeOutValue;
        (*srb).DataTransferLength = MODE_DATA_SIZE as u32;
        (*srb).DataBuffer = modeData as *mut core::ffi::c_void;
        (*srb).SrbFlags = (*FdoExtension).SrbFlags;
        (*srb).SrbFlags |= 0x40u32;
        (*srb).SrbFlags |= 0x8u32;
        (*srb).SrbFlags |= 0x100u32;
        (*srb).QueueAction = SRB_SIMPLE_TAG_REQUEST as u8;
        (*srb).QueueTag = (SP_UNTAGGED) as u8;
        (*srb).OriginalRequest = (irp).cast();
        (*srb).CdbLength = 6u8;
        cdb = &raw mut (*srb).Cdb as *mut u8 as *mut CDB;
    }

    (*cdb).MODE_SENSE.as_mut().OperationCode = SCSIOP_MODE_SENSE as u8;
    (*cdb)
        .MODE_SENSE
        .as_mut()
        .set_PageCode(MODE_PAGE_FAULT_REPORTING as u8);
    (*cdb).MODE_SENSE.as_mut().AllocationLength = MODE_DATA_SIZE as u8;

    IoMarkIrpPending(irp);
    IoCallDriver(
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
        irp,
    );

    STATUS_PENDING
}





#[no_mangle]
pub unsafe extern "C" fn DiskDetectFailurePrediction(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    FailurePredictCapability: *mut u32,
    ScsiAddressAvailable: i32,
) -> i32 {
    let commonExtension = FdoExtension as *mut COMMON_DEVICE_EXTENSION;
    let diskData = (*commonExtension).DriverData as *mut DISK_DATA;
    let mut supportFP: i32 = 0;
    let mut status: i32 = 0;
    let mut checkFailure: STORAGE_PREDICT_FAILURE = STORAGE_PREDICT_FAILURE {
        PredictFailure: 0,
        VendorSpecific: [0; 512],
    };
    let mut diskSmartStatus: STORAGE_FAILURE_PREDICT_STATUS = STORAGE_FAILURE_PREDICT_STATUS {
        Reason: 0,
        PredictFailure: 0,
    };

    // PAGED_CODE()
    if KeGetCurrentIrql() > 1u8 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8 as *const i8,
                2278i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    *FailurePredictCapability = FailurePredictionNone;

    if ScsiAddressAvailable != 0 {
        DiskGetIdentifyInfo(FdoExtension, (&raw mut supportFP).cast());
        if supportFP != 0 {
            status = DiskEnableSmart(FdoExtension);
            if status >= 0 {
                *FailurePredictCapability = (FailurePredictionSmart) as u32;
                (*diskData).FailurePredictionEnabled = 1;
                status = DiskReadFailurePredictStatus(FdoExtension, &raw mut diskSmartStatus);
                StorDebugPrint(
                    (4i32) as u32,
                    TRACE_FLAG_WMI,
                    b"Disk: Device %p %s IDE SMART\n\0" as *const u8 as *const i8,
                    (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                    if status >= 0 {
                        b"does\0" as *const u8 as *const i8
                    } else {
                        b"does not\0" as *const u8 as *const i8
                    },
                );
                if !(status >= 0) {
                    *FailurePredictCapability = FailurePredictionNone;
                    (*diskData).FailurePredictionEnabled = 0;
                }
            }
            return status;
        }
    }

    status = DiskSendFailurePredictIoctl(FdoExtension, &raw mut checkFailure);
    StorDebugPrint(
        (4i32) as u32,
        TRACE_FLAG_WMI,
        b"Disk: Device %p %s IOCTL_STORAGE_FAILURE_PREDICT\n\0" as *const u8 as *const i8,
        (*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
        if status >= 0 {
            b"does\0" as *const u8 as *const i8
        } else {
            b"does not\0" as *const u8 as *const i8
        },
    );

    if status >= 0 {
        *FailurePredictCapability = (FailurePredictionIoctl) as u32;
        (*diskData).FailurePredictionEnabled = 1;
        if checkFailure.PredictFailure != 0 {
            checkFailure.PredictFailure = 512;
            ClassNotifyFailurePredicted(
                FdoExtension,
                &raw mut checkFailure as *mut u8,
                core::mem::size_of::<STORAGE_PREDICT_FAILURE>() as u32,
                (if (*FdoExtension).FailurePredicted == 0 { 1 } else { 0 }) as i32,
                0x11u32,
                (*diskData).ScsiAddress.PathId,
                (*diskData).ScsiAddress.TargetId,
                (*diskData).ScsiAddress.Lun,
            );
            (*FdoExtension).FailurePredicted = 1;
        }
        return status;
    }

    *FailurePredictCapability = FailurePredictionNone;
    DiskInfoExceptionCheck(FdoExtension);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn DiskWmiFunctionControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    GuidIndex: u32,
    Function: CLASSENABLEDISABLEFUNCTION,
    Enable: u8,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    // PAGED_CODE
    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    as *const u8 as *const i8,
                2416i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    if Function == 0u32 && Enable != 0 {
        if GuidIndex == SmartStatusGuid as u32
            || GuidIndex == SmartDataGuid as u32
            || GuidIndex == SmartThresholdsGuid as u32
            || GuidIndex == SmartPerformFunction as u32
        {
            status = DiskEnableDisableFailurePrediction(fdoExtension, 1u8);
            StorDebugPrint(
                (4i32) as u32,
                TRACE_FLAG_WMI,
                b"Disk: DeviceObject %p, Irp %p Enable -> %lx\n\0" as *const u8 as *const i8,
                DeviceObject,
                Irp,
                status,
            );
        } else {
            StorDebugPrint(
                (4i32) as u32,
                TRACE_FLAG_WMI,
                b"Disk: DeviceObject %p, Irp %p, GuidIndex %d %s for Collection\n\0" as *const u8
                    as *const i8,
                DeviceObject,
                Irp,
                GuidIndex,
                if Enable != 0 {
                    b"Enabled\0" as *const u8 as *const i8
                } else {
                    b"Disabled\0" as *const u8 as *const i8
                },
            );
        }
    } else if Function == 1u32 {
        StorDebugPrint(
            (4i32) as u32,
            TRACE_FLAG_WMI,
            b"Disk: DeviceObject %p, Irp %p, GuidIndex %d %s for Event Generation\n\0" as *const u8
                as *const i8,
            DeviceObject,
            Irp,
            GuidIndex,
            if Enable != 0 {
                b"Enabled\0" as *const u8 as *const i8
            } else {
                b"Disabled\0" as *const u8 as *const i8
            },
        );
        if GuidIndex == SmartEventGuid as u32 && Enable != 0 {
            status = DiskEnableDisableFailurePredictPolling(fdoExtension, Enable, 0u32);
            StorDebugPrint(
                (4i32) as u32,
                TRACE_FLAG_WMI,
                b"Disk: DeviceObject %p, Irp %p %s -> %lx\n\0" as *const u8 as *const i8,
                DeviceObject,
                Irp,
                if Enable != 0 {
                    b"DiskEnableSmartPolling\0" as *const u8 as *const i8
                } else {
                    b"DiskDisableSmartPolling\0" as *const u8 as *const i8
                },
                status,
            );
        }
    } else {
        StorDebugPrint(
            (4i32) as u32,
            TRACE_FLAG_WMI,
            b"Disk: DeviceObject %p, Irp %p, GuidIndex %d %s for function %d\n\0" as *const u8
                as *const i8,
            DeviceObject,
            Irp,
            GuidIndex,
            if Enable != 0 {
                b"Enabled\0" as *const u8 as *const i8
            } else {
                b"Disabled\0" as *const u8 as *const i8
            },
            Function as u32,
        );
    }

    status = ClassWmiCompleteRequest(
        DeviceObject,
        Irp,
        status,
        0u32,
        IO_NO_INCREMENT as i8,
    );
    return status;
}





#[no_mangle]
pub unsafe extern "system" fn DiskFdoQueryWmiRegInfo(
    DeviceObject: PDEVICE_OBJECT,
    RegFlags: *mut ULONG,
    InstanceName: PUNICODE_STRING,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData = (*commonExtension).DriverData as PDISK_DATA;

    // PAGED_CODE macro expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr()).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr()).cast(),
                b"drivers/storage/class/disk/diskwmi.c\0".as_ptr() as *const i8,
                2521i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // Initial SET_FLAG on SmartThresholdsGuid (5) and ScsiInfoExceptionsGuid (6)
    DiskWmiFdoGuidList[5usize].Flags |= 0x10000u32;
    DiskWmiFdoGuidList[6usize].Flags |= 0x10000u32;

    let current_block: u64;
    match (*diskData).FailurePredictionCapability as u32 {
        2 => {
            // CLEAR_FLAG on SmartThresholdsGuid
            DiskWmiFdoGuidList[5usize].Flags &= !(0x10000i32 as u32);
            current_block = 13183875560443969876;
        }
        1 => {
            current_block = 13183875560443969876;
        }
        3 => {
            // FailurePredictionSense
            DiskWmiFdoGuidList[1usize].Flags &= !(0x10000i32 as u32); // SmartStatusGuid
            DiskWmiFdoGuidList[4usize].Flags &= !(0x10000i32 as u32); // SmartEventGuid
            DiskWmiFdoGuidList[3usize].Flags &= !(0x10000i32 as u32); // SmartPerformFunction
            DiskWmiFdoGuidList[6usize].Flags &= !(0x10000i32 as u32); // ScsiInfoExceptionsGuid
            DiskWmiFdoGuidList[2usize].Flags |= 0x10000u32; // SmartDataGuid
            current_block = 4808432441040389987;
        }
        _ => {
            // default
            DiskWmiFdoGuidList[1usize].Flags |= 0x10000u32;
            DiskWmiFdoGuidList[2usize].Flags |= 0x10000u32;
            DiskWmiFdoGuidList[4usize].Flags |= 0x10000u32;
            DiskWmiFdoGuidList[3usize].Flags |= 0x10000u32;
            current_block = 4808432441040389987;
        }
    }

    match current_block {
        13183875560443969876 => {
            // Common fallthrough for FailurePredictionSmart and FailurePredictionIoctl
            DiskWmiFdoGuidList[1usize].Flags &= !(0x10000i32 as u32); // SmartStatusGuid
            DiskWmiFdoGuidList[2usize].Flags &= !(0x10000i32 as u32); // SmartDataGuid
            DiskWmiFdoGuidList[4usize].Flags &= !(0x10000i32 as u32); // SmartEventGuid
            DiskWmiFdoGuidList[3usize].Flags &= !(0x10000i32 as u32); // SmartPerformFunction
        }
        _ => {}
    }

    *RegFlags = WMIREG_FLAG_INSTANCE_PDO as ULONG;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn DiskFdoQueryWmiRegInfoEx(
    DeviceObject: PDEVICE_OBJECT,
    RegFlags: *mut ULONG,
    InstanceName: PUNICODE_STRING,
    MofName: PUNICODE_STRING,
) -> NTSTATUS {
    let status = DiskFdoQueryWmiRegInfo(DeviceObject, RegFlags, InstanceName);
    return status;
}
#[no_mangle]
pub unsafe extern "system" fn DiskFdoQueryWmiDataBlock(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    GuidIndex: ULONG,
    BufferAvail: ULONG,
    Buffer: PUCHAR,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData = (*commonExtension).DriverData as PDISK_DATA;
    let mut sizeNeeded: ULONG = 0;

    {
        const APC_LEVEL: i32 = 1;
        if KeGetCurrentIrql() > (APC_LEVEL as i32) as u8 {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
                KeGetCurrentIrql() as i32,
            );
            if 0 as ::core::ffi::c_int == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        .as_ptr() as *const i8,
                    2681 as i32,
                    b"0\0".as_ptr() as *const i8,
                );
                __int2c();
            }
        }
    }

    {
        const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
        const TRACE_FLAG_WMI: u32 = 6; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
        TracePrint(
            (TRACE_LEVEL_INFORMATION) as i32,
            TRACE_FLAG_WMI,
            b"Disk: DiskQueryWmiDataBlock, Device %p, Irp %p, GuiIndex %d\n      BufferAvail %lx Buffer %p\n\0"
                .as_ptr() as *const i8,
            DeviceObject,
            Irp,
            GuidIndex,
            BufferAvail,
            Buffer,
        );
    }

    match GuidIndex {
        0 => {
            sizeNeeded = core::mem::size_of::<DISK_GEOMETRY>() as ULONG;
            if BufferAvail >= sizeNeeded {
                let mut current_block_78: u64;
                if (*DeviceObject).Characteristics as u32 & FILE_REMOVABLE_MEDIA != 0 {
                    status = DiskReadDriveCapacity(
                        (*(*commonExtension).PartitionZeroExtension).LowerPdo,
                    );
                    (*diskData).ReadyStatus = status;
                    if !NT_SUCCESS(status) {
                        current_block_78 = 3229571381435211107;
                    } else {
                        current_block_78 = 11050875288958768710;
                    }
                } else {
                    current_block_78 = 11050875288958768710;
                }
                match current_block_78 {
                    3229571381435211107 => {}
                    _ => {
                        core::ptr::copy_nonoverlapping(
                            &(*fdoExtension).DiskGeometry as *const DISK_GEOMETRY as *const u8,
                            Buffer,
                            core::mem::size_of::<DISK_GEOMETRY>(),
                        );
                        status = STATUS_SUCCESS;
                    }
                }
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        1 => {
            if !((*diskData).FailurePredictionCapability as u32
                != FailurePredictionNone as u32)
            {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        .as_ptr() as *const i8,
                    2729 as i32,
                    b"diskData->FailurePredictionCapability != FailurePredictionNone\0"
                        .as_ptr() as *const i8,
                );
                __int2c();
            }
            sizeNeeded = core::mem::size_of::<STORAGE_FAILURE_PREDICT_STATUS>() as ULONG;
            if BufferAvail >= sizeNeeded {
                let mut checkFailure: STORAGE_PREDICT_FAILURE = STORAGE_PREDICT_FAILURE {
                    PredictFailure: 0,
                    VendorSpecific: [0u8; 512],
                };
                let diskSmartStatus: PSTORAGE_FAILURE_PREDICT_STATUS =
                    Buffer as PSTORAGE_FAILURE_PREDICT_STATUS;
                status = DiskSendFailurePredictIoctl(fdoExtension, &raw mut checkFailure);
                if NT_SUCCESS(status) {
                    if (*diskData).FailurePredictionCapability as u32
                        == FailurePredictionSense as u32
                    {
                        (*diskSmartStatus).Reason =
                            *(&raw mut checkFailure.VendorSpecific as *mut UCHAR as *const ULONG) as ULONG;
                    } else {
                        (*diskSmartStatus).Reason = 0 as ULONG;
                    }
                    (*diskSmartStatus).PredictFailure =
                        (checkFailure.PredictFailure != 0) as BOOLEAN;
                }
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        2 => {
            if !((*diskData).FailurePredictionCapability as u32 == FailurePredictionSmart as u32
                || (*diskData).FailurePredictionCapability as u32
                    == FailurePredictionIoctl as u32)
            {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        .as_ptr() as *const i8,
                    2766 as i32,
                    b"(diskData->FailurePredictionCapability == FailurePredictionSmart) || (diskData->FailurePredictionCapability == FailurePredictionIoctl)\0"
                        .as_ptr() as *const i8,
                );
                __int2c();
            }
            sizeNeeded = core::mem::size_of::<STORAGE_FAILURE_PREDICT_DATA>() as ULONG;
            if BufferAvail >= sizeNeeded {
                let checkFailure: PSTORAGE_PREDICT_FAILURE =
                    Buffer as PSTORAGE_PREDICT_FAILURE;
                let diskSmartData: PSTORAGE_FAILURE_PREDICT_DATA =
                    Buffer as PSTORAGE_FAILURE_PREDICT_DATA;
                status = DiskSendFailurePredictIoctl(fdoExtension, checkFailure);
                if NT_SUCCESS(status) {
                    (*diskSmartData).Length = 512 as ULONG;
                }
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        5 => {
            if !((*diskData).FailurePredictionCapability as u32
                == FailurePredictionSmart as u32)
            {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        .as_ptr() as *const i8,
                    2794 as i32,
                    b"(diskData->FailurePredictionCapability == FailurePredictionSmart)\0"
                        .as_ptr() as *const i8,
                );
                __int2c();
            }
            sizeNeeded = core::mem::size_of::<STORAGE_FAILURE_PREDICT_THRESHOLDS>() as ULONG;
            if BufferAvail >= sizeNeeded {
                let diskSmartThresholds: PSTORAGE_FAILURE_PREDICT_THRESHOLDS =
                    Buffer as PSTORAGE_FAILURE_PREDICT_THRESHOLDS;
                status = DiskReadFailurePredictThresholds(fdoExtension, diskSmartThresholds);
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        3 => {
            sizeNeeded = 0 as ULONG;
            status = STATUS_SUCCESS;
        }
        6 => {
            if !((*diskData).FailurePredictionCapability as u32
                == FailurePredictionSense as u32)
            {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        .as_ptr() as *const i8,
                    2822 as i32,
                    b"(diskData->FailurePredictionCapability == FailurePredictionSense)\0"
                        .as_ptr() as *const i8,
                );
                __int2c();
            }
            sizeNeeded = core::mem::size_of::<STORAGE_SCSI_INFO_EXCEPTIONS>() as ULONG;
            if BufferAvail >= sizeNeeded {
                let infoExceptions: PSTORAGE_SCSI_INFO_EXCEPTIONS =
                    Buffer as PSTORAGE_SCSI_INFO_EXCEPTIONS;
                let mut modeInfo: MODE_INFO_EXCEPTIONS = core::mem::zeroed();
                status = DiskGetInfoExceptionInformation(fdoExtension, &raw mut modeInfo);
                if NT_SUCCESS(status) {
                    (*infoExceptions).PageSavable = modeInfo.PSBit() as BOOLEAN;
                    (*infoExceptions).Flags = modeInfo.__bindgen_anon_1.bindgen_union_field;
                    (*infoExceptions).MRIE = modeInfo.ReportMethod();
                    (*infoExceptions).Padding = 0 as UCHAR;
                    {
                        let val1: *mut u8 = (&raw mut (*infoExceptions).IntervalTimer) as *mut u8;
                        let val2: *const u8 = (&raw const modeInfo.IntervalTimer) as *const u8;
                        *val1.add(3) = *val2.add(0);
                        *val1.add(2) = *val2.add(1);
                        *val1.add(1) = *val2.add(2);
                        *val1.add(0) = *val2.add(3);
                    }
                    {
                        let val1: *mut u8 = (&raw mut (*infoExceptions).ReportCount) as *mut u8;
                        let val2: *const u8 = (&raw const modeInfo.ReportCount) as *const u8;
                        *val1.add(3) = *val2.add(0);
                        *val1.add(2) = *val2.add(1);
                        *val1.add(1) = *val2.add(2);
                        *val1.add(0) = *val2.add(3);
                    }
                }
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        _ => {
            sizeNeeded = 0 as ULONG;
            status = STATUS_WMI_GUID_NOT_FOUND;
        }
    }

    {
        const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
        const TRACE_FLAG_WMI: u32 = 6; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
        TracePrint(
            (TRACE_LEVEL_INFORMATION) as i32,
            TRACE_FLAG_WMI,
            b"Disk: DiskQueryWmiDataBlock Device %p, Irp %p returns %lx\n\0".as_ptr()
                as *const i8,
            DeviceObject,
            Irp,
            status,
        );
    }

    status = ClassWmiCompleteRequest(
        DeviceObject,
        Irp,
        status,
        sizeNeeded,
        IO_NO_INCREMENT as i8,
    );

    status
}








#[no_mangle]
pub unsafe extern "system" fn DiskFdoSetWmiDataBlock(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    GuidIndex: ULONG,
    BufferSize: ULONG,
    Buffer: PUCHAR,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;

    // Constants for unknown values (customary definitions)
    const FILE_REMOVABLE_MEDIA: u32 = 0x00000001;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;

    // PAGED_CODE();
    let current_irql = KeGetCurrentIrql() as i32;
    if current_irql > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            current_irql,
        );
        if 0 as i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0".as_ptr() as *const i8,
                2910i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_WMI,
        b"Disk: DiskSetWmiDataBlock, Device %p, Irp %p, GuiIndex %d\n      BufferSize %#x Buffer %p\n\0"
            .as_ptr() as *const i8,
        DeviceObject,
        Irp,
        GuidIndex,
        BufferSize,
        Buffer,
    );

    if GuidIndex == ScsiInfoExceptionsGuid as ULONG {
        let mut infoExceptions: PSTORAGE_SCSI_INFO_EXCEPTIONS = core::ptr::null_mut();
        let mut modeInfo: MODE_INFO_EXCEPTIONS = core::mem::zeroed();

        if BufferSize as usize >= core::mem::size_of::<STORAGE_SCSI_INFO_EXCEPTIONS>() {
            infoExceptions = Buffer as PSTORAGE_SCSI_INFO_EXCEPTIONS;

            modeInfo.set_PageCode(MODE_PAGE_FAULT_REPORTING as UCHAR);
            modeInfo.PageLength = (core::mem::size_of::<MODE_INFO_EXCEPTIONS>() - 2) as UCHAR;
            modeInfo.set_PSBit(0);
            *modeInfo.__bindgen_anon_1.Flags.as_mut() = (*infoExceptions).Flags as UCHAR;
            modeInfo.set_ReportMethod((*infoExceptions).MRIE as UCHAR);

            REVERSE_BYTES(
                &mut modeInfo.IntervalTimer[0] as *mut u8,
                (&(*infoExceptions).IntervalTimer as *const u32) as *const u8,
            );
            REVERSE_BYTES(
                &mut modeInfo.ReportCount[0] as *mut u8,
                (&(*infoExceptions).ReportCount as *const u32) as *const u8,
            );

            if modeInfo.__bindgen_anon_1.__bindgen_anon_1.as_ref().Perf() as i32 == 1 {
                (*diskData).AllowFPPerfHit = 0; // FALSE
            } else {
                (*diskData).AllowFPPerfHit = 1; // TRUE
            }

            status = DiskSetInfoExceptionInformation(fdoExtension, &raw mut modeInfo);
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    } else if GuidIndex <= SmartThresholdsGuid as ULONG {
        status = STATUS_WMI_READ_ONLY;
    } else {
        status = STATUS_WMI_GUID_NOT_FOUND;
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_WMI,
        b"Disk: DiskSetWmiDataBlock Device %p, Irp %p returns %lx\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
        status,
    );

    status = ClassWmiCompleteRequest(DeviceObject, Irp, status, 0, IO_NO_INCREMENT as CCHAR);
    status
}





#[no_mangle]
pub unsafe extern "system" fn DiskFdoSetWmiDataItem(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    GuidIndex: u32,
    DataItemId: u32,
    BufferSize: u32,
    Buffer: PUCHAR,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0i32 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/diskwmi.c\0"
                    .as_ptr() as *const i8,
                3016i32,
                b"0\0".as_ptr() as *const i8,
            );
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("int 0x2C", options(nostack));
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("brk #0xF001", options(nostack));
        }
    }

    StorDebugPrint(
        (4i32) as u32,
        TRACE_FLAG_WMI,
        b"Disk: DiskSetWmiDataItem, Device %p, Irp %p, GuiIndex %d, DataId %d\n      BufferSize %#x Buffer %p\n\0"
            .as_ptr() as *const i8,
        DeviceObject,
        Irp,
        GuidIndex,
        DataItemId,
        BufferSize,
        Buffer,
    );

    if GuidIndex <= (SmartThresholdsGuid) as u32 {
        status = STATUS_WMI_READ_ONLY;
    } else {
        status = STATUS_WMI_GUID_NOT_FOUND;
    }

    StorDebugPrint(
        (4i32) as u32,
        TRACE_FLAG_WMI,
        b"Disk: DiskSetWmiDataItem Device %p, Irp %p returns %lx\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
        status,
    );

    status = ClassWmiCompleteRequest(
        DeviceObject,
        Irp,
        status,
        0u32,
        IO_NO_INCREMENT,
    );

    status
}
#[no_mangle]
pub unsafe extern "system" fn DiskFdoExecuteWmiMethod(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    GuidIndex: u32,
    MethodId: u32,
    InBufferSize: u32,
    OutBufferSize: u32,
    mut Buffer: *mut u8,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let mut sizeNeeded: u32 = 0;
    let mut status: i32;

    // PAGED_CODE
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        __int2c();
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_WMI,
        b"Disk: DiskExecuteWmiMethod, DeviceObject %p, Irp %p, Guid Id %d, MethodId %d\n      InBufferSize %#x, OutBufferSize %#x, Buffer %p\n\0" as *const u8 as *const i8,
        DeviceObject,
        Irp,
        GuidIndex,
        MethodId,
        InBufferSize,
        OutBufferSize,
        Buffer,
    );

    match GuidIndex {
        __mv if __mv == (SmartPerformFunction) as u32 => {
            // NT_ASSERT
            if !((*diskData).FailurePredictionCapability == (FailurePredictionSmart) as u32
                || (*diskData).FailurePredictionCapability == (FailurePredictionIoctl) as u32
                || (*diskData).FailurePredictionCapability == FailurePredictionSense)
            {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/diskwmi.c\0"
                        as *const u8 as *const i8,
                    3112i32,
                    b"(diskData->FailurePredictionCapability == FailurePredictionSmart) || (diskData->FailurePredictionCapability == FailurePredictionIoctl) || (diskData->FailurePredictionCapability == FailurePredictionSense)\0"
                        as *const u8 as *const i8,
                );
                __int2c();
            }

            match MethodId {
                __mv if __mv == (AllowDisallowPerformanceHit) as u32 => {
                    let mut allowPerfHit: u8 = 0;
                    sizeNeeded = 0;
                    if InBufferSize as usize >= core::mem::size_of::<u8>() {
                        status = STATUS_SUCCESS;
                        allowPerfHit = *Buffer;
                        if (*diskData).AllowFPPerfHit as i32 != allowPerfHit as i32 {
                            (*diskData).AllowFPPerfHit = allowPerfHit;
                            if (*diskData).FailurePredictionCapability == FailurePredictionSense {
                                let mut modeInfo: MODE_INFO_EXCEPTIONS = core::mem::zeroed();
                                status = DiskGetInfoExceptionInformation(
                                    fdoExtension,
                                    &raw mut modeInfo,
                                );
                                if status >= 0 {
                                    modeInfo.__bindgen_anon_1.__bindgen_anon_1.as_mut().set_Perf(
                                        if allowPerfHit != 0 { 0u8 } else { 1u8 },
                                    );
                                    status = DiskSetInfoExceptionInformation(
                                        fdoExtension,
                                        &raw mut modeInfo,
                                    );
                                }
                            } else {
                                status = STATUS_INVALID_DEVICE_REQUEST;
                            }
                        }
                        TracePrint(
                            (TRACE_LEVEL_INFORMATION) as i32,
                            TRACE_FLAG_WMI,
                            b"DiskFdoWmiExecuteMethod: AllowPerformanceHit %x for device %p --> %lx\n\0"
                                as *const u8 as *const i8,
                            allowPerfHit as i32,
                            (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                            status,
                        );
                    } else {
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
                __mv if __mv == (EnableDisableHardwareFailurePrediction) as u32 => {
                    let mut enable: u8 = 0;
                    sizeNeeded = 0;
                    if InBufferSize as usize >= core::mem::size_of::<u8>() {
                        status = STATUS_SUCCESS;
                        enable = *Buffer;
                        if enable == 0 {
                            DiskEnableDisableFailurePredictPolling(
                                fdoExtension,
                                enable,
                                0u32,
                            );
                        }
                        status = DiskEnableDisableFailurePrediction(fdoExtension, enable);
                        TracePrint(
                            (TRACE_LEVEL_INFORMATION) as i32,
                            TRACE_FLAG_WMI,
                            b"DiskFdoWmiExecuteMethod: EnableDisableHardwareFailurePrediction: %x for device %p --> %lx\n\0"
                                as *const u8 as *const i8,
                            enable as i32,
                            (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                            status,
                        );
                    } else {
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
                __mv if __mv == (EnableDisableFailurePredictionPolling) as u32 => {
                    let mut enable: u8 = 0;
                    let mut period: u32 = 0;
                    sizeNeeded = 0;
                    if InBufferSize as usize >= core::mem::size_of::<u32>() + core::mem::size_of::<u8>() {
                        period = *(Buffer as *mut u32);
                        Buffer = Buffer.offset(core::mem::size_of::<u32>() as isize);
                        enable = *Buffer;
                        status = DiskEnableDisableFailurePredictPolling(fdoExtension, enable, period);
                        TracePrint(
                            (TRACE_LEVEL_INFORMATION) as i32,
                            TRACE_FLAG_WMI,
                            b"DiskFdoWmiExecuteMethod: EnableDisableFailurePredictionPolling: %x %x for device %p --> %lx\n\0"
                                as *const u8 as *const i8,
                            enable as i32,
                            period,
                            (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                            status,
                        );
                    } else {
                        status = STATUS_INVALID_PARAMETER;
                    }
                }
                __mv if __mv == (GetFailurePredictionCapability) as u32 => {
                    sizeNeeded = core::mem::size_of::<u32>() as u32;
                    if OutBufferSize >= sizeNeeded {
                        status = STATUS_SUCCESS;
                        *(Buffer as *mut u32) = (*diskData).FailurePredictionCapability as u32;
                        TracePrint(
                            (TRACE_LEVEL_INFORMATION) as i32,
                            TRACE_FLAG_WMI,
                            b"DiskFdoWmiExecuteMethod: GetFailurePredictionCapability: %x for device %p --> %lx\n\0"
                                as *const u8 as *const i8,
                            *(Buffer as *mut u32),
                            (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                            status,
                        );
                    } else {
                        status = STATUS_BUFFER_TOO_SMALL;
                    }
                }
                __mv if __mv == (EnableOfflineDiags) as u32 => {
                    sizeNeeded = core::mem::size_of::<u8>() as u32;
                    if OutBufferSize >= sizeNeeded {
                        if (*diskData).FailurePredictionCapability == (FailurePredictionSmart) as u32 {
                            status = DiskExecuteSmartDiagnostics(fdoExtension, 0u8);
                        } else {
                            status = STATUS_INVALID_DEVICE_REQUEST;
                        }
                        *Buffer = (status >= 0) as u8;
                        TracePrint(
                            (TRACE_LEVEL_INFORMATION) as i32,
                            TRACE_FLAG_WMI,
                            b"DiskFdoWmiExecuteMethod: EnableOfflineDiags for device %p --> %lx\n\0"
                                as *const u8 as *const i8,
                            (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
                            status,
                        );
                    } else {
                        status = STATUS_BUFFER_TOO_SMALL;
                    }
                }
                ReadLogSectors => {
                    if (*diskData).FailurePredictionCapability == (FailurePredictionSmart) as u32 {
                        if InBufferSize as usize >= core::mem::size_of::<READ_LOG_SECTORS_IN>() {
                            let inParams: *mut READ_LOG_SECTORS_IN = Buffer as *mut READ_LOG_SECTORS_IN;
                            let mut outParams: *mut READ_LOG_SECTORS_OUT;
                            let readSize = ((*inParams).SectorCount as u32) * (SMART_LOG_SECTOR_SIZE) as u32;
                            sizeNeeded = core::mem::offset_of!(READ_LOG_SECTORS_OUT, LogSectors) as u32 + readSize;
                            if OutBufferSize >= sizeNeeded {
                                outParams = Buffer as *mut READ_LOG_SECTORS_OUT;
                                status = DiskReadSmartLog(
                                    fdoExtension,
                                    (*inParams).SectorCount,
                                    (*inParams).LogAddress,
                                    &raw mut (*outParams).LogSectors as *mut u8,
                                );
                                if status >= 0 {
                                    (*outParams).Length = readSize;
                                } else {
                                    (*outParams).Length = 0u32;
                                    status = STATUS_SUCCESS;
                                }
                            } else {
                                status = STATUS_BUFFER_TOO_SMALL;
                            }
                        } else {
                            status = STATUS_INVALID_PARAMETER;
                        }
                    } else {
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    }
                }
                WriteLogSectors => {
                    if (*diskData).FailurePredictionCapability == (FailurePredictionSmart) as u32 {
                        if InBufferSize as i32 >= core::mem::offset_of!(WRITE_LOG_SECTORS_IN, LogSectors) as i32 {
                            let inParams: *mut WRITE_LOG_SECTORS_IN = Buffer as *mut WRITE_LOG_SECTORS_IN;
                            let mut outParams: *mut WRITE_LOG_SECTORS_OUT;
                            let writeSize = ((*inParams).SectorCount as u32) * (SMART_LOG_SECTOR_SIZE) as u32;
                            if InBufferSize as u32 >= core::mem::offset_of!(WRITE_LOG_SECTORS_IN, LogSectors) as u32 + writeSize {
                                sizeNeeded = core::mem::size_of::<WRITE_LOG_SECTORS_OUT>() as u32;
                                if OutBufferSize >= sizeNeeded {
                                    outParams = Buffer as *mut WRITE_LOG_SECTORS_OUT;
                                    status = DiskWriteSmartLog(
                                        fdoExtension as *mut core::ffi::c_void ,
                                        (*inParams).SectorCount,
                                        (*inParams).LogAddress,
                                        &raw mut (*inParams).LogSectors as *mut u8,
                                    );
                                    if status >= 0 {
                                        (*outParams).Success = 1u8;
                                    } else {
                                        (*outParams).Success = 0u8;
                                        status = STATUS_SUCCESS;
                                    }
                                } else {
                                    status = STATUS_BUFFER_TOO_SMALL;
                                }
                            } else {
                                status = STATUS_INVALID_PARAMETER;
                            }
                        } else {
                            status = STATUS_INVALID_PARAMETER;
                        }
                    } else {
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    }
                }
                ExecuteSelfTest => {
                    if (*diskData).FailurePredictionCapability == (FailurePredictionSmart) as u32 {
                        if InBufferSize as usize >= core::mem::size_of::<EXECUTE_SELF_TEST_IN>() {
                            sizeNeeded = core::mem::size_of::<EXECUTE_SELF_TEST_OUT>() as u32;
                            if OutBufferSize >= sizeNeeded {
                                let inParam: *mut EXECUTE_SELF_TEST_IN = Buffer as *mut EXECUTE_SELF_TEST_IN;
                                let outParam: *mut EXECUTE_SELF_TEST_OUT = Buffer as *mut EXECUTE_SELF_TEST_OUT;
                                if DiskIsValidSmartSelfTest((*inParam).Subcommand) != 0 {
                                    status = DiskExecuteSmartDiagnostics(fdoExtension, (*inParam).Subcommand);
                                    if status >= 0 {
                                        (*outParam).ReturnCode = 0u32;
                                    } else {
                                        (*outParam).ReturnCode = 2u32;
                                        status = STATUS_SUCCESS;
                                    }
                                } else {
                                    (*outParam).ReturnCode = 1u32;
                                    status = STATUS_SUCCESS;
                                }
                            } else {
                                status = STATUS_BUFFER_TOO_SMALL;
                            }
                        } else {
                            status = STATUS_INVALID_PARAMETER;
                        }
                    } else {
                        status = STATUS_INVALID_DEVICE_REQUEST;
                    }
                }
                _ => {
                    sizeNeeded = 0;
                    status = STATUS_WMI_ITEMID_NOT_FOUND;
                }
            }
        }
        __mv if __mv == (DiskGeometryGuid) as u32 || __mv == (SmartStatusGuid) as u32 || __mv == (SmartDataGuid) as u32 || __mv == (SmartEventGuid) as u32 || __mv == (SmartThresholdsGuid) as u32 || __mv == (ScsiInfoExceptionsGuid) as u32 => {
            sizeNeeded = 0;
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
        _ => {
            sizeNeeded = 0;
            status = STATUS_WMI_GUID_NOT_FOUND;
        }
    }

    TracePrint(
        (TRACE_LEVEL_INFORMATION) as i32,
        TRACE_FLAG_WMI,
        b"Disk: DiskExecuteMethod Device %p, Irp %p returns %lx\n\0" as *const u8 as *const i8,
        DeviceObject,
        Irp,
        status,
    );

    status = ClassWmiCompleteRequest(
        DeviceObject,
        Irp,
        status,
        sizeNeeded,
        IO_NO_INCREMENT,
    );

    status
}






// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 2010
//
// Module Name:
//
//     diskwmi.c
//
// Abstract:
//
//     SCSI disk class driver - WMI support routines
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
// Revision History:
//
// --*/
//
// #include "disk.h"
//
// #ifdef DEBUG_USE_WPP
// #include "diskwmi.tmh"
// #endif
//
// NTSTATUS
// DiskSendFailurePredictIoctl(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PSTORAGE_PREDICT_FAILURE checkFailure
//     );
//
// NTSTATUS
// DiskGetIdentifyInfo(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PBOOLEAN SupportSmart
//     );
//
// NTSTATUS
// DiskDetectFailurePrediction(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PFAILURE_PREDICTION_METHOD FailurePredictCapability,
//     BOOLEAN ScsiAddressAvailable
//     );
//
// NTSTATUS
// DiskReadFailurePredictThresholds(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PSTORAGE_FAILURE_PREDICT_THRESHOLDS DiskSmartThresholds
//     );
//
// NTSTATUS
// DiskReadSmartLog(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN UCHAR SectorCount,
//     IN UCHAR LogAddress,
//     OUT PUCHAR Buffer
//     );
//
// NTSTATUS
// DiskWriteSmartLog(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN UCHAR SectorCount,
//     IN UCHAR LogAddress,
//     IN PUCHAR Buffer
//     );
//
// IO_WORKITEM_ROUTINE DiskReregWorker;
//
// IO_COMPLETION_ROUTINE DiskInfoExceptionComplete;
//
// //
// // WMI reregistration globals
// //
// // Since it will take too long to do a mode sense on some drive, we
// // need a good way to effect the mode sense for the info exceptions
// // mode page so that we can determine if SMART is supported and enabled
// // for the drive. So the strategy is to do an asynchronous mode sense
// // when the device starts and then look at the info exceptions mode
// // page within the completion routine. Now within the completion
// // routine we cannot call IoWMIRegistrationControl since we are at DPC
// // level, so we create a stack of device objects that will be processed
// // by a single work item that is fired off only when the stack
// // transitions from empty to non empty.
// //
// SINGLE_LIST_ENTRY DiskReregHead;
// KSPIN_LOCK DiskReregSpinlock;
// LONG DiskReregWorkItems;
//
// GUIDREGINFO DiskWmiFdoGuidList[] =
// {
//     {
//         WMI_DISK_GEOMETRY_GUID,
//         1,
//         0
//     },
//
//     {
//         WMI_STORAGE_FAILURE_PREDICT_STATUS_GUID,
//         1,
//         WMIREG_FLAG_EXPENSIVE
//     },
//
//     {
//         WMI_STORAGE_FAILURE_PREDICT_DATA_GUID,
//         1,
//         WMIREG_FLAG_EXPENSIVE
//     },
//
//     {
//         WMI_STORAGE_FAILURE_PREDICT_FUNCTION_GUID,
//         1,
//         WMIREG_FLAG_EXPENSIVE
//     },
//
//     {
//         WMI_STORAGE_PREDICT_FAILURE_EVENT_GUID,
//         1,
//         WMIREG_FLAG_EVENT_ONLY_GUID
//     },
//
//     {
//         WMI_STORAGE_FAILURE_PREDICT_THRESHOLDS_GUID,
//         1,
//         WMIREG_FLAG_EXPENSIVE
//     },
//
//     {
//         WMI_STORAGE_SCSI_INFO_EXCEPTIONS_GUID,
//         1,
//         0
//     }
// };
//
//
// GUID DiskPredictFailureEventGuid = WMI_STORAGE_PREDICT_FAILURE_EVENT_GUID;
//
// #define DiskGeometryGuid           0
// #define SmartStatusGuid            1
// #define SmartDataGuid              2
// #define SmartPerformFunction       3
//     #define AllowDisallowPerformanceHit                 1
//     #define EnableDisableHardwareFailurePrediction      2
//     #define EnableDisableFailurePredictionPolling       3
//     #define GetFailurePredictionCapability              4
//     #define EnableOfflineDiags                          5
//
// #define SmartEventGuid             4
// #define SmartThresholdsGuid        5
// #define ScsiInfoExceptionsGuid     6
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DiskWmiFunctionControl)
// #pragma alloc_text(PAGE, DiskFdoQueryWmiRegInfo)
// #pragma alloc_text(PAGE, DiskFdoQueryWmiDataBlock)
// #pragma alloc_text(PAGE, DiskFdoSetWmiDataBlock)
// #pragma alloc_text(PAGE, DiskFdoSetWmiDataItem)
// #pragma alloc_text(PAGE, DiskFdoExecuteWmiMethod)
//
// #pragma alloc_text(PAGE, DiskDetectFailurePrediction)
// #pragma alloc_text(PAGE, DiskEnableDisableFailurePrediction)
// #pragma alloc_text(PAGE, DiskEnableDisableFailurePredictPolling)
// #pragma alloc_text(PAGE, DiskReadFailurePredictStatus)
// #pragma alloc_text(PAGE, DiskReadFailurePredictData)
// #pragma alloc_text(PAGE, DiskReadFailurePredictThresholds)
// #pragma alloc_text(PAGE, DiskGetIdentifyInfo)
// #pragma alloc_text(PAGE, DiskReadSmartLog)
// #pragma alloc_text(PAGE, DiskWriteSmartLog)
// #pragma alloc_text(PAGE, DiskPerformSmartCommand)
// #pragma alloc_text(PAGE, DiskSendFailurePredictIoctl)
// #pragma alloc_text(PAGE, DiskReregWorker)
// #pragma alloc_text(PAGE, DiskInitializeReregistration)
//
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
// #pragma alloc_text(PAGE, DiskGetModePage)
// #pragma alloc_text(PAGE, DiskEnableInfoExceptions)
// #endif // (NTDDI_VERSION >= NTDDI_WINBLUE)
//
// #endif
//
//
// //
// // Note:
// // Some port drivers assume that the SENDCMDINPARAMS structure will always be atleast
// // sizeof(SENDCMDINPARAMS). So do not adjust for the [pBuffer] if it isn't being used
// //
//
// //
// // SMART/IDE specific routines
// //
//
// //
// // Read SMART data attributes.
// // SrbControl should be : sizeof(SRB_IO_CONTROL) + MAX[ sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + READ_ATTRIBUTE_BUFFER_SIZE ]
// // Attribute data returned at &SendCmdOutParams->bBuffer[0]
// //
// #define DiskReadSmartData(FdoExtension,                                 \
//                           SrbControl,                                   \
//                           BufferSize)                                   \
//     DiskPerformSmartCommand(FdoExtension,                               \
//                             IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS,     \
//                             SMART_CMD,                                  \
//                             READ_ATTRIBUTES,                            \
//                             0,                                          \
//                             0,                                          \
//                             (SrbControl),                               \
//                             (BufferSize))
//
//
// //
// // Read SMART data thresholds.
// // SrbControl should be : sizeof(SRB_IO_CONTROL) + MAX[ sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + READ_THRESHOLD_BUFFER_SIZE ]
// // Attribute data returned at &SendCmdOutParams->bBuffer[0]
// //
// #define DiskReadSmartThresholds(FdoExtension,                           \
//                                 SrbControl,                             \
//                                 BufferSize)                             \
//     DiskPerformSmartCommand(FdoExtension,                               \
//                             IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS,  \
//                             SMART_CMD,                                  \
//                             READ_THRESHOLDS,                            \
//                             0,                                          \
//                             0,                                          \
//                             (SrbControl),                               \
//                             (BufferSize))
//
//
// //
// // Read SMART status
// // SrbControl should be : sizeof(SRB_IO_CONTROL) + MAX[ sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + sizeof(IDEREGS) ]
// // Failure predicted if SendCmdOutParams->bBuffer[3] == 0xf4 and SendCmdOutParams->bBuffer[4] == 0x2c
// //
// #define DiskReadSmartStatus(FdoExtension,                               \
//                             SrbControl,                                 \
//                             BufferSize)                                 \
//     DiskPerformSmartCommand(FdoExtension,                               \
//                             IOCTL_SCSI_MINIPORT_RETURN_STATUS,          \
//                             SMART_CMD,                                  \
//                             RETURN_SMART_STATUS,                        \
//                             0,                                          \
//                             0,                                          \
//                             (SrbControl),                               \
//                             (BufferSize))
//
//
// //
// // Read disks IDENTIFY data
// // SrbControl should be : sizeof(SRB_IO_CONTROL) + MAX[ sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + IDENTIFY_BUFFER_SIZE ]
// // Identify data returned at &SendCmdOutParams->bBuffer[0]
// //
// #define DiskGetIdentifyData(FdoExtension,                               \
//                             SrbControl,                                 \
//                             BufferSize)                                 \
//     DiskPerformSmartCommand(FdoExtension,                               \
//                             IOCTL_SCSI_MINIPORT_IDENTIFY,               \
//                             ID_CMD,                                     \
//                             0,                                          \
//                             0,                                          \
//                             0,                                          \
//                             (SrbControl),                               \
//                             (BufferSize))
//
//
// //
// // Enable SMART
// //
// static NTSTATUS
// DiskEnableSmart(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     UCHAR srbControl[sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS)] = {0};
//     ULONG bufferSize = sizeof(srbControl);
//
//     return DiskPerformSmartCommand(FdoExtension,
//                                    IOCTL_SCSI_MINIPORT_ENABLE_SMART,
//                                    SMART_CMD,
//                                    ENABLE_SMART,
//                                    0,
//                                    0,
//                                    (PSRB_IO_CONTROL)srbControl,
//                                    &bufferSize);
// }
//
//
// //
// // Disable SMART
// //
// static NTSTATUS
// DiskDisableSmart(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     UCHAR srbControl[sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS)] = {0};
//     ULONG bufferSize = sizeof(srbControl);
//
//     return DiskPerformSmartCommand(FdoExtension,
//                                    IOCTL_SCSI_MINIPORT_DISABLE_SMART,
//                                    SMART_CMD,
//                                    DISABLE_SMART,
//                                    0,
//                                    0,
//                                    (PSRB_IO_CONTROL)srbControl,
//                                    &bufferSize);
// }
//
// #ifndef __REACTOS__ // functions are not used
// //
// // Enable Attribute Autosave
// //
// _inline NTSTATUS
// DiskEnableSmartAttributeAutosave(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     UCHAR srbControl[sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS)] = {0};
//     ULONG bufferSize = sizeof(srbControl);
//
//     return DiskPerformSmartCommand(FdoExtension,
//                                    IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE,
//                                    SMART_CMD,
//                                    ENABLE_DISABLE_AUTOSAVE,
//                                    0xf1,
//                                    0,
//                                    (PSRB_IO_CONTROL)srbControl,
//                                    &bufferSize);
// }
//
//
// //
// // Disable Attribute Autosave
// //
// _inline NTSTATUS
// DiskDisableSmartAttributeAutosave(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     UCHAR srbControl[sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS)] = {0};
//     ULONG bufferSize = sizeof(srbControl);
//
//     return DiskPerformSmartCommand(FdoExtension,
//                                    IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE,
//                                    SMART_CMD,
//                                    ENABLE_DISABLE_AUTOSAVE,
//                                    0x00,
//                                    0,
//                                    (PSRB_IO_CONTROL)srbControl,
//                                    &bufferSize);
// }
// #endif
//
// //
// // Initialize execution of SMART online diagnostics
// //
// static NTSTATUS
// DiskExecuteSmartDiagnostics(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     UCHAR Subcommand
//     )
// {
//     UCHAR srbControl[sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS)] = {0};
//     ULONG bufferSize = sizeof(srbControl);
//
//     return DiskPerformSmartCommand(FdoExtension,
//                                    IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS,
//                                    SMART_CMD,
//                                    EXECUTE_OFFLINE_DIAGS,
//                                    0,
//                                    Subcommand,
//                                    (PSRB_IO_CONTROL)srbControl,
//                                    &bufferSize);
// }
//
//
// NTSTATUS
// DiskReadSmartLog(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN UCHAR SectorCount,
//     IN UCHAR LogAddress,
//     OUT PUCHAR Buffer
//     )
// {
//     PSRB_IO_CONTROL srbControl;
//     NTSTATUS status;
//     PSENDCMDOUTPARAMS sendCmdOutParams;
//     ULONG logSize, bufferSize;
//
//     PAGED_CODE();
//
//     logSize = SectorCount * SMART_LOG_SECTOR_SIZE;
//     bufferSize = sizeof(SRB_IO_CONTROL) +  max( sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + logSize );
//
//     srbControl = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                        bufferSize,
//                                        DISK_TAG_SMART);
//
//     if (srbControl != NULL)
//     {
//         status = DiskPerformSmartCommand(FdoExtension,
//                                          IOCTL_SCSI_MINIPORT_READ_SMART_LOG,
//                                          SMART_CMD,
//                                          SMART_READ_LOG,
//                                          SectorCount,
//                                          LogAddress,
//                                          srbControl,
//                                          &bufferSize);
//
//         if (NT_SUCCESS(status))
//         {
//             sendCmdOutParams = (PSENDCMDOUTPARAMS)((PUCHAR)srbControl +
//                                                    sizeof(SRB_IO_CONTROL));
//             RtlCopyMemory(Buffer,
//                           &sendCmdOutParams->bBuffer[0],
//                           logSize);
//         }
//
//         FREE_POOL(srbControl);
//     } else {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     return(status);
// }
//
//
// NTSTATUS
// DiskWriteSmartLog(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN UCHAR SectorCount,
//     IN UCHAR LogAddress,
//     IN PUCHAR Buffer
//     )
// {
//     PSRB_IO_CONTROL srbControl;
//     NTSTATUS status;
//     PSENDCMDINPARAMS sendCmdInParams;
//     ULONG logSize, bufferSize;
//
//     PAGED_CODE();
//
//     logSize = SectorCount * SMART_LOG_SECTOR_SIZE;
//     bufferSize = sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDINPARAMS) - 1 +
//                  logSize;
//
//     srbControl = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                        bufferSize,
//                                        DISK_TAG_SMART);
//
//     if (srbControl != NULL)
//     {
//         sendCmdInParams = (PSENDCMDINPARAMS)((PUCHAR)srbControl +
//                                                sizeof(SRB_IO_CONTROL));
//         RtlCopyMemory(&sendCmdInParams->bBuffer[0],
//                       Buffer,
//                       logSize);
//         status = DiskPerformSmartCommand(FdoExtension,
//                                          IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG,
//                                          SMART_CMD,
//                                          SMART_WRITE_LOG,
//                                          SectorCount,
//                                          LogAddress,
//                                          srbControl,
//                                          &bufferSize);
//
//         FREE_POOL(srbControl);
//     } else {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     return(status);
// }
//
//
// NTSTATUS
// DiskPerformSmartCommand(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     IN ULONG SrbControlCode,
//     IN UCHAR Command,
//     IN UCHAR Feature,
//     IN UCHAR SectorCount,
//     IN UCHAR SectorNumber,
//     IN OUT PSRB_IO_CONTROL SrbControl,
//     OUT PULONG BufferSize
//     )
// /*++
//
// Routine Description:
//
//     This routine will perform some SMART command
//
// Arguments:
//
//     FdoExtension is the FDO device extension
//
//     SrbControlCode is the SRB control code to use for the request
//
//     Command is the SMART command to be executed. It may be SMART_CMD or
//         ID_CMD.
//
//     Feature is the value to place in the IDE feature register.
//
//     SectorCount is the value to place in the IDE SectorCount register
//
//     SrbControl is the buffer used to build the SRB_IO_CONTROL and pass
//         any input parameters. It also returns the output parameters.
//
//     *BufferSize on entry has total size of SrbControl and on return has
//         the size used in SrbControl.
//
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = (PCOMMON_DEVICE_EXTENSION)FdoExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PUCHAR buffer;
//     PSENDCMDINPARAMS cmdInParameters;
//     NTSTATUS status;
//     ULONG availableBufferSize;
//     KEVENT event;
//     PIRP irp;
//     _IO_STATUS_BLOCK ioStatus = { 0 };
//     SCSI_REQUEST_BLOCK srb = {0};
//     LARGE_INTEGER startingOffset;
//     ULONG length;
//     PIO_STACK_LOCATION irpStack;
//     UCHAR srbExBuffer[CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE] = {0};
//     PSTORAGE_REQUEST_BLOCK srbEx = (PSTORAGE_REQUEST_BLOCK)srbExBuffer;
//     PSTOR_ADDR_BTL8 storAddrBtl8;
//
//     PAGED_CODE();
//
//     //
//     // Point to the 'buffer' portion of the SRB_CONTROL and compute how
//     // much room we have left in the srb control. Abort if the buffer
//     // isn't at least the size of SRB_IO_CONTROL.
//     //
//
//     buffer = (PUCHAR)SrbControl + sizeof(SRB_IO_CONTROL);
//
//     cmdInParameters = (PSENDCMDINPARAMS)buffer;
//
//     if (*BufferSize >= sizeof(SRB_IO_CONTROL)) {
//         availableBufferSize = *BufferSize - sizeof(SRB_IO_CONTROL);
//     } else {
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
// #if DBG
//
//     //
//     // Ensure control codes and buffer lengths passed are correct
//     //
//     {
//         ULONG controlCode = 0;
//         ULONG lengthNeeded = sizeof(SENDCMDINPARAMS);
//
//         if (Command == SMART_CMD)
//         {
//             switch (Feature)
//             {
//                 case ENABLE_SMART:
//                 {
//                     controlCode = IOCTL_SCSI_MINIPORT_ENABLE_SMART;
//                     break;
//                 }
//
//                 case DISABLE_SMART:
//                 {
//                     controlCode = IOCTL_SCSI_MINIPORT_DISABLE_SMART;
//                     break;
//                 }
//
//                 case RETURN_SMART_STATUS:
//                 {
//                     controlCode = IOCTL_SCSI_MINIPORT_RETURN_STATUS;
//                     lengthNeeded = max( lengthNeeded, sizeof(SENDCMDOUTPARAMS) - 1 + sizeof(IDEREGS) );
//                     break;
//                 }
//
//                 case ENABLE_DISABLE_AUTOSAVE:
//                 {
//                     controlCode = IOCTL_SCSI_MINIPORT_ENABLE_DISABLE_AUTOSAVE;
//                     break;
//                 }
//
//                 case SAVE_ATTRIBUTE_VALUES:
//                 {
//                     controlCode = IOCTL_SCSI_MINIPORT_SAVE_ATTRIBUTE_VALUES;
//                     break;
//                 }
//
//
//                 case EXECUTE_OFFLINE_DIAGS:
//                 {
//                     controlCode = IOCTL_SCSI_MINIPORT_EXECUTE_OFFLINE_DIAGS;
//                     break;
//                 }
//
//                 case READ_ATTRIBUTES:
//                 {
//                     controlCode  = IOCTL_SCSI_MINIPORT_READ_SMART_ATTRIBS;
//                     lengthNeeded = max( lengthNeeded, sizeof(SENDCMDOUTPARAMS) - 1 + READ_ATTRIBUTE_BUFFER_SIZE );
//                     break;
//                 }
//
//                 case READ_THRESHOLDS:
//                 {
//                     controlCode  = IOCTL_SCSI_MINIPORT_READ_SMART_THRESHOLDS;
//                     lengthNeeded = max( lengthNeeded, sizeof(SENDCMDOUTPARAMS) - 1 + READ_THRESHOLD_BUFFER_SIZE );
//                     break;
//                 }
//
//                 case SMART_READ_LOG:
//                 {
//                     controlCode  = IOCTL_SCSI_MINIPORT_READ_SMART_LOG;
//                     lengthNeeded = max( lengthNeeded, sizeof(SENDCMDOUTPARAMS) - 1 + (SectorCount * SMART_LOG_SECTOR_SIZE) );
//                     break;
//                 }
//
//                 case SMART_WRITE_LOG:
//                 {
//                     controlCode  = IOCTL_SCSI_MINIPORT_WRITE_SMART_LOG;
//                     lengthNeeded = lengthNeeded - 1 + (SectorCount * SMART_LOG_SECTOR_SIZE);
//                     break;
//                 }
//
//             }
//
//         } else if (Command == ID_CMD) {
//
//             controlCode  = IOCTL_SCSI_MINIPORT_IDENTIFY;
//             lengthNeeded = max( lengthNeeded, sizeof(SENDCMDOUTPARAMS) - 1 + IDENTIFY_BUFFER_SIZE );
//
//         } else {
//
//             NT_ASSERT(FALSE);
//         }
//
//         NT_ASSERT(controlCode == SrbControlCode);
//         NT_ASSERT(availableBufferSize >= lengthNeeded);
//     }
//
// #endif
//
//     //
//     // Build SrbControl and input to SMART command
//     //
//     SrbControl->HeaderLength = sizeof(SRB_IO_CONTROL);
//     RtlMoveMemory (SrbControl->Signature, "SCSIDISK", 8);
//     SrbControl->Timeout      = FdoExtension->TimeOutValue;
//     SrbControl->Length       = availableBufferSize;
//     SrbControl->ControlCode  = SrbControlCode;
//
//     cmdInParameters->cBufferSize  = sizeof(SENDCMDINPARAMS);
//     cmdInParameters->bDriveNumber = diskData->ScsiAddress.TargetId;
//     cmdInParameters->irDriveRegs.bFeaturesReg     = Feature;
//     cmdInParameters->irDriveRegs.bSectorCountReg  = SectorCount;
//     cmdInParameters->irDriveRegs.bSectorNumberReg = SectorNumber;
//     cmdInParameters->irDriveRegs.bCylLowReg       = SMART_CYL_LOW;
//     cmdInParameters->irDriveRegs.bCylHighReg      = SMART_CYL_HI;
//     cmdInParameters->irDriveRegs.bCommandReg      = Command;
//
//     //
//     // Create and send irp
//     //
//     KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//     startingOffset.QuadPart = (LONGLONG) 1;
//
//     length = SrbControl->HeaderLength + SrbControl->Length;
//
//     irp = IoBuildSynchronousFsdRequest(
//                 IRP_MJ_SCSI,
//                 commonExtension->LowerDeviceObject,
//                 SrbControl,
//                 length,
//                 &startingOffset,
//                 &event,
//                 &ioStatus);
//
//     if (irp == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     irpStack = IoGetNextIrpStackLocation(irp);
//
//     //
//     // Set major and minor codes.
//     //
//
//     irpStack->MajorFunction = IRP_MJ_SCSI;
//     irpStack->MinorFunction = 1;
//
//     //
//     // Fill in SRB fields.
//     //
//
//     if (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         irpStack->Parameters.Others.Argument1 = srbEx;
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = sizeof(srbExBuffer);
//         srbEx->SrbFunction = SRB_FUNCTION_IO_CONTROL;
//         srbEx->RequestPriority = IoGetIoPriorityHint(irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//
//         srbEx->SrbFlags = FdoExtension->SrbFlags;
//         SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_DATA_IN);
//         SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//         SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_NO_KEEP_AWAKE);
//
//         srbEx->RequestAttribute = SRB_SIMPLE_TAG_REQUEST;
//         srbEx->RequestTag = SP_UNTAGGED;
//
//         srbEx->OriginalRequest = irp;
//
//         //
//         // Set timeout to requested value.
//         //
//
//         srbEx->TimeOutValue = SrbControl->Timeout;
//
//         //
//         // Set the data buffer.
//         //
//
//         srbEx->DataBuffer = SrbControl;
//         srbEx->DataTransferLength = length;
//
//         //
//         // Set up address fields
//         //
//
//         storAddrBtl8 = (PSTOR_ADDR_BTL8) ((PUCHAR)srbEx + srbEx->AddressOffset);
//         storAddrBtl8->Type = STOR_ADDRESS_TYPE_BTL8;
//         storAddrBtl8->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//         storAddrBtl8->Path = diskData->ScsiAddress.PathId;
//         storAddrBtl8->Target = diskData->ScsiAddress.TargetId;
//         storAddrBtl8->Lun = srb.Lun = diskData->ScsiAddress.Lun;
//
//     } else {
//         irpStack->Parameters.Others.Argument1 = &srb;
//
//         srb.PathId = diskData->ScsiAddress.PathId;
//         srb.TargetId = diskData->ScsiAddress.TargetId;
//         srb.Lun = diskData->ScsiAddress.Lun;
//
//         srb.Function = SRB_FUNCTION_IO_CONTROL;
//         srb.Length = sizeof(SCSI_REQUEST_BLOCK);
//
//         srb.SrbFlags = FdoExtension->SrbFlags;
//         SET_FLAG(srb.SrbFlags, SRB_FLAGS_DATA_IN);
//         SET_FLAG(srb.SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//         SET_FLAG(srb.SrbFlags, SRB_FLAGS_NO_KEEP_AWAKE);
//
//         srb.QueueAction = SRB_SIMPLE_TAG_REQUEST;
//         srb.QueueTag = SP_UNTAGGED;
//
//         srb.OriginalRequest = irp;
//
//         //
//         // Set timeout to requested value.
//         //
//
//         srb.TimeOutValue = SrbControl->Timeout;
//
//         //
//         // Set the data buffer.
//         //
//
//         srb.DataBuffer = SrbControl;
//         srb.DataTransferLength = length;
//     }
//
//     //
//     // Flush the data buffer for output. This will insure that the data is
//     // written back to memory.  Since the data-in flag is the the port driver
//     // will flush the data again for input which will ensure the data is not
//     // in the cache.
//     //
//
//     KeFlushIoBuffers(irp->MdlAddress, FALSE, TRUE);
//
//     //
//     // Call port driver to handle this request.
//     //
//
//     status = IoCallDriver(commonExtension->LowerDeviceObject, irp);
//
//     if (status == STATUS_PENDING) {
//         KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//         status = ioStatus.Status;
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// DiskGetIdentifyInfo(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PBOOLEAN SupportSmart
//     )
// {
//     UCHAR outBuffer[sizeof(SRB_IO_CONTROL) + max( sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + IDENTIFY_BUFFER_SIZE )] = {0};
//     ULONG outBufferSize = sizeof(outBuffer);
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     status = DiskGetIdentifyData(FdoExtension,
//                                  (PSRB_IO_CONTROL)outBuffer,
//                                  &outBufferSize);
//
//     if (NT_SUCCESS(status))
//     {
//         PUSHORT identifyData = (PUSHORT)&(outBuffer[sizeof(SRB_IO_CONTROL) + sizeof(SENDCMDOUTPARAMS) - 1]);
//         USHORT commandSetSupported = identifyData[82];
//
//         *SupportSmart = ((commandSetSupported != 0xffff) &&
//                          (commandSetSupported != 0) &&
//                          ((commandSetSupported & 1) == 1));
//     } else {
//         *SupportSmart = FALSE;
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_IOCTL, "DiskGetIdentifyInfo: SMART %s supported for device %p, status %lx\n",
//                    *SupportSmart ? "is" : "is not",
//                    FdoExtension->DeviceObject,
//                    status));
//
//     return status;
// }
//
//
// //
// // FP Ioctl specific routines
// //
//
// NTSTATUS
// DiskSendFailurePredictIoctl(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PSTORAGE_PREDICT_FAILURE checkFailure
//     )
// {
//     KEVENT event;
//     PDEVICE_OBJECT deviceObject;
//     _IO_STATUS_BLOCK ioStatus = { 0 };
//     PIRP irp;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     KeInitializeEvent(&event, SynchronizationEvent, FALSE);
//
//     deviceObject = IoGetAttachedDeviceReference(FdoExtension->DeviceObject);
//
//     irp = IoBuildDeviceIoControlRequest(
//                     IOCTL_STORAGE_PREDICT_FAILURE,
//                     deviceObject,
//                     NULL,
//                     0,
//                     checkFailure,
//                     sizeof(STORAGE_PREDICT_FAILURE),
//                     FALSE,
//                     &event,
//                     &ioStatus);
//
//     if (irp != NULL)
//     {
//         status = IoCallDriver(deviceObject, irp);
//         if (status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//             status = ioStatus.Status;
//         }
//
//     } else {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     ObDereferenceObject(deviceObject);
//
//     return status;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WINBLUE)
//
// NTSTATUS
// DiskGetModePage(
//      PDEVICE_OBJECT Fdo,
//      UCHAR PageMode,
//      UCHAR PageControl,
//      PMODE_PARAMETER_HEADER ModeData,
//      PULONG ModeDataSize,
//      PVOID* PageData
//     )
// {
//     ULONG size = 0;
//     PVOID pageData = NULL;
//
//     PAGED_CODE();
//
//     if (ModeData == NULL ||
//         ModeDataSize == NULL ||
//         *ModeDataSize < sizeof(MODE_PARAMETER_HEADER) ||
//         PageData == NULL) {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     RtlZeroMemory (ModeData, *ModeDataSize);
//
//     size = ClassModeSenseEx(Fdo,
//                             (PCHAR) ModeData,
//                             *ModeDataSize,
//                             PageMode,
//                             PageControl);
//
//     if (size < sizeof(MODE_PARAMETER_HEADER)) {
//
//         //
//         // Retry the request in case of a check condition.
//         //
//         size = ClassModeSenseEx(Fdo,
//                             (PCHAR) ModeData,
//                             *ModeDataSize,
//                             PageMode,
//                             PageControl);
//
//         if (size < sizeof(MODE_PARAMETER_HEADER)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_WMI, "DiskGetModePage: Mode Sense for Page Mode %d with Page Control %d failed\n",
//                 PageMode, PageControl));
//             *ModeDataSize = 0;
//             return STATUS_IO_DEVICE_ERROR;
//         }
//     }
//
//     //
//     // If the length is greater than length indicated by the mode data reset
//     // the data to the mode data.
//     //
//     if (size > (ULONG) (ModeData->ModeDataLength + 1)) {
//         size = ModeData->ModeDataLength + 1;
//     }
//
//     *ModeDataSize = size;
//
//     //
//     // Find the mode page
//     //
//     pageData = ClassFindModePage((PCHAR) ModeData,
//                                  size,
//                                  PageMode,
//                                  TRUE);
//
//     if (pageData) {
//         *PageData = pageData;
//         return STATUS_SUCCESS;
//     } else {
//         *PageData = NULL;
//         return STATUS_NOT_SUPPORTED;
//     }
// }
//
// NTSTATUS
// DiskEnableInfoExceptions(
//      PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//      BOOLEAN Enable
//     )
// {
//     PDISK_DATA diskData = (PDISK_DATA)(FdoExtension->CommonExtension.DriverData);
//     NTSTATUS status = STATUS_NOT_SUPPORTED;
//     PMODE_PARAMETER_HEADER modeData;
//     PMODE_INFO_EXCEPTIONS pageData;
//     MODE_INFO_EXCEPTIONS changeablePageData;
//     ULONG modeDataSize;
//
//     PAGED_CODE();
//
//     modeDataSize = MODE_DATA_SIZE;
//
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                          modeDataSize,
//                                          DISK_TAG_INFO_EXCEPTION);
//
//     if (modeData == NULL) {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_WMI, "DiskEnableInfoExceptions: Unable to allocate mode "
//                        "data buffer\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // First see which data is actually changeable.
//     //
//     status = DiskGetModePage(FdoExtension->DeviceObject,
//                              MODE_PAGE_FAULT_REPORTING,
//                              1, // Page Control = 1 indicates we want changeable values.
//                              modeData,
//                              &modeDataSize,
//                              (PVOID*)&pageData);
//
//     if (!NT_SUCCESS(status) || pageData == NULL) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskEnableInfoExceptions: does NOT support SMART for device %p\n",
//                   FdoExtension->DeviceObject));
//         FREE_POOL(modeData);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskEnableInfoExceptions: DOES support SMART for device %p\n",
//                 FdoExtension->DeviceObject));
//
//     //
//     // At the very least, the DEXCPT bit must be changeable.
//     // If it's not, bail out now.
//     //
//     if (pageData->Dexcpt == 0) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskEnableInfoExceptions: does NOT support DEXCPT bit for device %p\n",
//                   FdoExtension->DeviceObject));
//         FREE_POOL(modeData);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // Cache away which values are changeable.
//     //
//     RtlCopyMemory(&changeablePageData, pageData, sizeof(MODE_INFO_EXCEPTIONS));
//
//     //
//     // Now get the current values.
//     //
//     status = DiskGetModePage(FdoExtension->DeviceObject,
//                              MODE_PAGE_FAULT_REPORTING,
//                              0, // Page Control = 0 indicates we want current values.
//                              modeData,
//                              &modeDataSize,
//                              (PVOID*)&pageData);
//
//     if (!NT_SUCCESS(status) || pageData == NULL) {
//         //
//         // At this point we know the device supports this mode page so
//         // assert if something goes wrong here.
//         //
//         NT_ASSERT(NT_SUCCESS(status) && pageData);
//         FREE_POOL(modeData);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // If the device is currently configured to not report any informational
//     // exceptions and we cannot change the value of that field, there's
//     // nothing to be done.
//     //
//     if (pageData->ReportMethod == 0 && changeablePageData.ReportMethod == 0) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskEnableInfoExceptions: MRIE field is 0 and is not changeable for device %p\n",
//                   FdoExtension->DeviceObject));
//         FREE_POOL(modeData);
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // If the PERF bit is changeable, set it now.
//     //
//     if (changeablePageData.Perf) {
//         pageData->Perf = diskData->AllowFPPerfHit ? 0 : 1;
//     }
//
//     //
//     // If the MRIE field is changeable, set it to 4 so that informational
//     // exceptions get reported with the "Recovered Error" sense key.
//     //
//     if (changeablePageData.ReportMethod) {
//         pageData->ReportMethod = 4;
//     }
//
//     //
//     // Finally, set the DEXCPT bit appropriately to enable/disable
//     // informational exception reporting and send the Mode Select.
//     //
//     pageData->Dexcpt = !Enable;
//
//     status = ClassModeSelect(FdoExtension->DeviceObject,
//                                 (PCHAR)modeData,
//                                 modeDataSize,
//                                 pageData->PSBit);
//
//     //
//     // Update the failure prediction state.  Note that for this particular
//     // mode FailurePredictionNone is used when it's not enabled.
//     //
//     if (NT_SUCCESS(status)) {
//         if (Enable) {
//             diskData->FailurePredictionCapability = FailurePredictionSense;
//             diskData->FailurePredictionEnabled = TRUE;
//         } else {
//             diskData->FailurePredictionCapability = FailurePredictionNone;
//             diskData->FailurePredictionEnabled = FALSE;
//         }
//     }
//
//     FREE_POOL(modeData);
//
//     return status;
// }
// #endif
//
//
// //
// // FP type independent routines
// //
//
// NTSTATUS
// DiskEnableDisableFailurePrediction(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     BOOLEAN Enable
//     )
// /*++
//
// Routine Description:
//
//     Enable or disable failure prediction at the hardware level
//
// Arguments:
//
//     FdoExtension
//
//     Enable
//
// Return Value:
//
//     NT Status
//
// --*/
// {
//     NTSTATUS status;
//     PCOMMON_DEVICE_EXTENSION commonExtension = &(FdoExtension->CommonExtension);
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//
//     PAGED_CODE();
//
//     switch(diskData->FailurePredictionCapability)
//     {
//         case FailurePredictionSmart:
//         {
//             if (Enable)
//             {
//                 status = DiskEnableSmart(FdoExtension);
//             } else {
//                 status = DiskDisableSmart(FdoExtension);
//             }
//
//             if (NT_SUCCESS(status)) {
//                 diskData->FailurePredictionEnabled = Enable;
//             }
//
//             break;
//         }
//
//         case  FailurePredictionSense:
//         case  FailurePredictionIoctl:
//         {
//             //
//             // We assume that the drive is already setup properly for
//             // failure prediction
//             //
//             status = STATUS_SUCCESS;
//             break;
//         }
//
//         default:
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//     return status;
// }
//
//
// NTSTATUS
// DiskEnableDisableFailurePredictPolling(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     BOOLEAN Enable,
//     ULONG PollTimeInSeconds
//     )
// /*++
//
// Routine Description:
//
//     Enable or disable polling for hardware failure detection
//
// Arguments:
//
//     FdoExtension
//
//     Enable
//
//     PollTimeInSeconds - if 0 then no change to current polling timer
//
// Return Value:
//
//     NT Status
//
// --*/
// {
//     NTSTATUS status;
//     PCOMMON_DEVICE_EXTENSION commonExtension = (PCOMMON_DEVICE_EXTENSION)FdoExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//
//     PAGED_CODE();
//
//     if (Enable)
//     {
//         status = DiskEnableDisableFailurePrediction(FdoExtension,
//                                            Enable);
//     } else {
//         status = STATUS_SUCCESS;
//     }
//
//     if (NT_SUCCESS(status))
//     {
//         status = ClassSetFailurePredictionPoll(FdoExtension,
//                         Enable ? diskData->FailurePredictionCapability :
//                                  FailurePredictionNone,
//                                      PollTimeInSeconds);
//
//         //
//         // Even if this failed we do not want to disable FP on the
//         // hardware. FP is only ever disabled on the hardware by
//         // specific command of the user.
//         //
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// DiskReadFailurePredictStatus(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PSTORAGE_FAILURE_PREDICT_STATUS DiskSmartStatus
//     )
// /*++
//
// Routine Description:
//
//     Obtains current failure prediction status
//
// Arguments:
//
//     FdoExtension
//
//     DiskSmartStatus
//
// Return Value:
//
//     NT Status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = (PCOMMON_DEVICE_EXTENSION)FdoExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     DiskSmartStatus->PredictFailure = FALSE;
//
//     switch(diskData->FailurePredictionCapability)
//     {
//         case FailurePredictionSmart:
//         {
//             UCHAR outBuffer[sizeof(SRB_IO_CONTROL) + max( sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + sizeof(IDEREGS) )] = {0};
//             ULONG outBufferSize = sizeof(outBuffer);
//             PSENDCMDOUTPARAMS cmdOutParameters;
//
//             status = DiskReadSmartStatus(FdoExtension,
//                                      (PSRB_IO_CONTROL)outBuffer,
//                                      &outBufferSize);
//
//             if (NT_SUCCESS(status))
//             {
//                 cmdOutParameters = (PSENDCMDOUTPARAMS)(outBuffer +
//                                                sizeof(SRB_IO_CONTROL));
//
//                 DiskSmartStatus->Reason = 0; // Unknown;
//                 DiskSmartStatus->PredictFailure = ((cmdOutParameters->bBuffer[3] == 0xf4) &&
//                                                    (cmdOutParameters->bBuffer[4] == 0x2c));
//             }
//             break;
//         }
//
//         case FailurePredictionSense:
//         {
//             DiskSmartStatus->Reason = FdoExtension->FailureReason;
//             DiskSmartStatus->PredictFailure = FdoExtension->FailurePredicted;
//             status = STATUS_SUCCESS;
//             break;
//         }
//
//         case FailurePredictionIoctl:
//         case FailurePredictionNone:
//         default:
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// DiskReadFailurePredictData(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PSTORAGE_FAILURE_PREDICT_DATA DiskSmartData
//     )
// /*++
//
// Routine Description:
//
//     Obtains current failure prediction data. Not available for
//     FAILURE_PREDICT_SENSE types.
//
// Arguments:
//
//     FdoExtension
//
//     DiskSmartData
//
// Return Value:
//
//     NT Status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = (PCOMMON_DEVICE_EXTENSION)FdoExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     switch(diskData->FailurePredictionCapability)
//     {
//         case FailurePredictionSmart:
//         {
//             PUCHAR outBuffer;
//             ULONG outBufferSize;
//             PSENDCMDOUTPARAMS cmdOutParameters;
//
//             outBufferSize = sizeof(SRB_IO_CONTROL) + max( sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + READ_ATTRIBUTE_BUFFER_SIZE );
//
//             outBuffer = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                               outBufferSize,
//                                               DISK_TAG_SMART);
//
//             if (outBuffer != NULL)
//             {
//                 status = DiskReadSmartData(FdoExtension,
//                                            (PSRB_IO_CONTROL)outBuffer,
//                                            &outBufferSize);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     cmdOutParameters = (PSENDCMDOUTPARAMS)(outBuffer +
//                                                     sizeof(SRB_IO_CONTROL));
//
//                     DiskSmartData->Length = READ_ATTRIBUTE_BUFFER_SIZE;
//                     RtlCopyMemory(DiskSmartData->VendorSpecific,
//                                   cmdOutParameters->bBuffer,
//                                   min(READ_ATTRIBUTE_BUFFER_SIZE, sizeof(DiskSmartData->VendorSpecific)));
//                 }
//                 FREE_POOL(outBuffer);
//             } else {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             break;
//         }
//
//         case FailurePredictionSense:
//         {
//             DiskSmartData->Length = sizeof(ULONG);
//             *((PULONG)DiskSmartData->VendorSpecific) = FdoExtension->FailureReason;
//
//             status = STATUS_SUCCESS;
//             break;
//         }
//
//         case FailurePredictionIoctl:
//         case FailurePredictionNone:
//         default:
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//         }
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// DiskReadFailurePredictThresholds(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PSTORAGE_FAILURE_PREDICT_THRESHOLDS DiskSmartThresholds
//     )
// /*++
//
// Routine Description:
//
//     Obtains current failure prediction thresholds. Not available for
//     FAILURE_PREDICT_SENSE types.
//
// Arguments:
//
//     FdoExtension
//
//     DiskSmartData
//
// Return Value:
//
//     NT Status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = (PCOMMON_DEVICE_EXTENSION)FdoExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     switch(diskData->FailurePredictionCapability)
//     {
//         case FailurePredictionSmart:
//         {
//             PUCHAR outBuffer;
//             PSENDCMDOUTPARAMS cmdOutParameters;
//             ULONG outBufferSize;
//
//             outBufferSize = sizeof(SRB_IO_CONTROL) + max( sizeof(SENDCMDINPARAMS), sizeof(SENDCMDOUTPARAMS) - 1 + READ_THRESHOLD_BUFFER_SIZE );
//
//             outBuffer = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                               outBufferSize,
//                                               DISK_TAG_SMART);
//
//             if (outBuffer != NULL)
//             {
//                 status = DiskReadSmartThresholds(FdoExtension,
//                                                 (PSRB_IO_CONTROL)outBuffer,
//                                                 &outBufferSize);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     cmdOutParameters = (PSENDCMDOUTPARAMS)(outBuffer +
//                                            sizeof(SRB_IO_CONTROL));
//
//                     RtlCopyMemory(DiskSmartThresholds->VendorSpecific,
//                                   cmdOutParameters->bBuffer,
//                                   min(READ_THRESHOLD_BUFFER_SIZE, sizeof(DiskSmartThresholds->VendorSpecific)));
//                 }
//                 FREE_POOL(outBuffer);
//             } else {
//                 status = STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             break;
//         }
//
//         case FailurePredictionSense:
//         case FailurePredictionIoctl:
//         case FailurePredictionNone:
//         default:
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//         }
//     }
//
//     return status;
// }
//
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskReregWorker(
//     IN PDEVICE_OBJECT DevObject,
//     IN PVOID Context
//     )
// {
//     PDISKREREGREQUEST reregRequest;
//     NTSTATUS status;
//     PDEVICE_OBJECT deviceObject;
//     PIRP irp;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(DevObject);
//
//     NT_ASSERT(Context != NULL);
//     _Analysis_assume_(Context != NULL);
//
//     do
//     {
//         reregRequest = (PDISKREREGREQUEST)ExInterlockedPopEntryList(
//                              &DiskReregHead,
//                              &DiskReregSpinlock);
//
//         if (reregRequest != NULL)
//         {
//             deviceObject = reregRequest->DeviceObject;
//             irp = reregRequest->Irp;
//
//             status = IoWMIRegistrationControl(deviceObject,
//                                               WMIREG_ACTION_UPDATE_GUIDS);
//
//             //
//             // Release remove lock and free irp, now that we are done
//             // processing this
//             //
//             ClassReleaseRemoveLock(deviceObject, irp);
//
//             IoFreeMdl(irp->MdlAddress);
//             IoFreeIrp(irp);
//
//             FREE_POOL(reregRequest);
//
//         } else {
//
//             NT_ASSERTMSG("Disk Re-registration request list should not be empty", FALSE);
//
//             status = STATUS_INTERNAL_ERROR;
//         }
//
//         if (!NT_SUCCESS(status))
//         {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskReregWorker: Reregistration failed %x\n",
//                         status));
//         }
//
//     } while (InterlockedDecrement(&DiskReregWorkItems));
//
//     IoFreeWorkItem((PIO_WORKITEM)Context);
// }
//
//
// NTSTATUS
// DiskInitializeReregistration(
//     VOID
//     )
// {
//     PAGED_CODE();
//
//     //
//     // Initialize the spinlock used to manage the
//     // list of disks reregistering their guids
//     //
//     KeInitializeSpinLock(&DiskReregSpinlock);
//
//     return(STATUS_SUCCESS);
// }
//
//
// NTSTATUS
// DiskPostReregisterRequest(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp
//     )
// {
//     PDISKREREGREQUEST reregRequest;
//     PIO_WORKITEM workItem;
//     NTSTATUS status;
//
//     workItem = IoAllocateWorkItem(DeviceObject);
//
//     if (!workItem) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     reregRequest = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                          sizeof(DISKREREGREQUEST),
//                                          DISK_TAG_SMART);
//     if (reregRequest != NULL)
//     {
//         //
//         // add the disk that needs reregistration to the stack of disks
//         // to reregister. If the list is transitioning from empty to
//         // non empty then also kick off the work item so that the
//         // reregistration worker can do the reregister.
//         //
//         reregRequest->DeviceObject = DeviceObject;
//         reregRequest->Irp = Irp;
//         ExInterlockedPushEntryList(
//                                    &DiskReregHead,
//                                    &reregRequest->Next,
//                                    &DiskReregSpinlock);
//
//         if (InterlockedIncrement(&DiskReregWorkItems) == 1)
//         {
//             //
//             // There is no worker routine running, queue this one.
//             // When the work item runs, it will process the reregistration
//             // list.
//             //
//
//             IoQueueWorkItem(workItem,
//                             DiskReregWorker,
//                             DelayedWorkQueue,
//                             workItem);
//         } else {
//
//             //
//             // There is a worker routine already running, so we
//             // can free this unused work item.
//             //
//
//             IoFreeWorkItem(workItem);
//         }
//
//         status = STATUS_SUCCESS;
//
//     } else {
//
//         IoFreeWorkItem(workItem);
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskPostReregisterRequest: could not allocate reregRequest for %p\n",
//                     DeviceObject));
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     return(status);
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskInfoExceptionComplete(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context
//     )
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//     PIO_STACK_LOCATION nextIrpStack = IoGetNextIrpStackLocation(Irp);
//     PSCSI_REQUEST_BLOCK srb = Context;
//     NTSTATUS status;
//     BOOLEAN retry;
//     ULONG retryInterval;
//     ULONG srbStatus;
//     BOOLEAN freeLockAndIrp = TRUE;
//     PVOID originalSenseInfoBuffer = irpStack->Parameters.Others.Argument3;
//     PSTORAGE_REQUEST_BLOCK srbEx = NULL;
//     PVOID dataBuffer = NULL;
//     ULONG dataLength = 0;
//     PVOID senseBuffer = NULL;
//     UCHAR cdbLength8 = 0;
//     ULONG cdbLength32 = 0;
//     UCHAR senseBufferLength = 0;
//
//     srbStatus = SRB_STATUS(srb->SrbStatus);
//
//     if (srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) {
//         srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//         dataBuffer = srbEx->DataBuffer;
//         dataLength = srbEx->DataTransferLength;
//         if ((srbEx->SrbFunction == SRB_FUNCTION_EXECUTE_SCSI) &&
//             (srbEx->NumSrbExData > 0)) {
//             (void)GetSrbScsiData(srbEx, &cdbLength8, &cdbLength32, NULL, &senseBuffer, &senseBufferLength);
//         }
//     } else {
//         dataBuffer = srb->DataBuffer;
//         dataLength = srb->DataTransferLength;
//         senseBuffer = srb->SenseInfoBuffer;
//     }
//
//     //
//     // Check SRB status for success of completing request.
//     // SRB_STATUS_DATA_OVERRUN also indicates success.
//     //
//     if ((srbStatus != SRB_STATUS_SUCCESS) &&
//         (srbStatus != SRB_STATUS_DATA_OVERRUN))
//     {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskInfoExceptionComplete: IRP %p, SRB %p\n", Irp, srb));
//
//         if (TEST_FLAG(srb->SrbStatus, SRB_STATUS_QUEUE_FROZEN))
//         {
//             ClassReleaseQueue(DeviceObject);
//         }
//
//         retry = ClassInterpretSenseInfo(
//                     DeviceObject,
//                     srb,
//                     irpStack->MajorFunction,
//                      0,
//                     MAXIMUM_RETRIES -
//                         ((ULONG)(ULONG_PTR)irpStack->Parameters.Others.Argument4),
//                     &status,
//                     &retryInterval);
//
//         //
//         // If the status is verified required and the this request
//         // should bypass verify required then retry the request.
//         //
//
//         if (TEST_FLAG(irpStack->Flags, SL_OVERRIDE_VERIFY_VOLUME) &&
//             status == STATUS_VERIFY_REQUIRED)
//         {
//             status = STATUS_IO_DEVICE_ERROR;
//             retry = TRUE;
//         }
//
//         retry = retry && irpStack->Parameters.Others.Argument4;
//
//         irpStack->Parameters.Others.Argument4 = (PVOID)((ULONG_PTR)irpStack->Parameters.Others.Argument4 - 1);
//
//         if (retry)
//         {
//             //
//             // Retry request.
//             //
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskInfoExceptionComplete: Retry request %p\n", Irp));
//
//             NT_ASSERT(dataBuffer == MmGetMdlVirtualAddress(Irp->MdlAddress));
//
//             if (srb->Function == SRB_FUNCTION_STORAGE_REQUEST_BLOCK) {
//
//                 //
//                 // Reset byte count of transfer in SRB Extension.
//                 //
//                 srbEx->DataTransferLength = Irp->MdlAddress->ByteCount;
//
//                 //
//                 // Zero SRB statuses.
//                 //
//
//                 srbEx->SrbStatus = 0;
//                 if ((srbEx->SrbFunction == SRB_FUNCTION_EXECUTE_SCSI) &&
//                     (srbEx->NumSrbExData > 0)) {
//                     SetSrbScsiData(srbEx, cdbLength8, cdbLength32, 0, senseBuffer, senseBufferLength);
//                 }
//
//                 //
//                 // Set the no disconnect flag, disable synchronous data transfers and
//                 // disable tagged queuing. This fixes some errors.
//                 //
//
//                 SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_DISABLE_DISCONNECT);
//                 SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//                 CLEAR_FLAG(srbEx->SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//
//                 srbEx->RequestAttribute = SRB_SIMPLE_TAG_REQUEST;
//                 srbEx->RequestTag = SP_UNTAGGED;
//
//             } else {
//
//                 //
//                 // Reset byte count of transfer in SRB Extension.
//                 //
//                 srb->DataTransferLength = Irp->MdlAddress->ByteCount;
//
//                 //
//                 // Zero SRB statuses.
//                 //
//
//                 srb->SrbStatus = srb->ScsiStatus = 0;
//
//                 //
//                 // Set the no disconnect flag, disable synchronous data transfers and
//                 // disable tagged queuing. This fixes some errors.
//                 //
//
//                 SET_FLAG(srb->SrbFlags, SRB_FLAGS_DISABLE_DISCONNECT);
//                 SET_FLAG(srb->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//                 CLEAR_FLAG(srb->SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//
//                 srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//                 srb->QueueTag = SP_UNTAGGED;
//             }
//
//             //
//             // Set up major SCSI function.
//             //
//
//             nextIrpStack->MajorFunction = IRP_MJ_SCSI;
//
//             //
//             // Save SRB address in next stack for port driver.
//             //
//
//             nextIrpStack->Parameters.Scsi.Srb = srb;
//
//
//             IoSetCompletionRoutine(Irp,
//                                    DiskInfoExceptionComplete,
//                                    srb,
//                                    TRUE, TRUE, TRUE);
//
//             (VOID)IoCallDriver(commonExtension->LowerDeviceObject, Irp);
//
//             return STATUS_MORE_PROCESSING_REQUIRED;
//         }
//
//     } else {
//
//         //
//         // Get the results from the mode sense
//         //
//         PMODE_INFO_EXCEPTIONS pageData;
//         PMODE_PARAMETER_HEADER modeData;
//         ULONG modeDataLength;
//
//         modeData = dataBuffer;
//         modeDataLength = dataLength;
//
//         pageData = ClassFindModePage((PCHAR)modeData,
//                                      modeDataLength,
//                                      MODE_PAGE_FAULT_REPORTING,
//                                      TRUE);
//         if (pageData != NULL)
//         {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskInfoExceptionComplete: %p supports SMART\n",
//                         DeviceObject));
//
//             diskData->ScsiInfoExceptionsSupported = TRUE;
//
//             //
//             // The DEXCPT bit must be 0 and the MRIE field must be valid.
//             //
//             if (pageData->Dexcpt == 0 &&
//                 pageData->ReportMethod >= 2 &&
//                 pageData->ReportMethod <= 6)
//             {
//                 diskData->FailurePredictionCapability = FailurePredictionSense;
//                 diskData->FailurePredictionEnabled = TRUE;
//                 status = DiskPostReregisterRequest(DeviceObject, Irp);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     //
//                     // Make sure we won't free the remove lock and the irp
//                     // since we need to keep these until after the work
//                     // item has completed running
//                     //
//                     freeLockAndIrp = FALSE;
//                 }
//             } else {
//                 TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionComplete: %p is not enabled for SMART\n",
//                         DeviceObject));
//
//             }
//
//         } else {
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionComplete: %p does not supports SMART\n",
//                         DeviceObject));
//
//         }
//
//         //
//         // Set status for successful request
//         //
//
//         status = STATUS_SUCCESS;
//
//     } // end if (SRB_STATUS(srb->SrbStatus) == SRB_STATUS_SUCCESS)
//
//     //
//     // Free the srb
//     //
//     if (senseBuffer != originalSenseInfoBuffer)
//     {
//         //
//         // Free the original sense info buffer in case the port driver has overwritten it
//         //
//         FREE_POOL(originalSenseInfoBuffer);
//     }
//
//     FREE_POOL(senseBuffer);
//     FREE_POOL(dataBuffer);
//     FREE_POOL(srb);
//
//     if (freeLockAndIrp)
//     {
//         //
//         // Set status in completing IRP.
//         //
//
//         Irp->IoStatus.Status = status;
//
//         //
//         // If pending has be returned for this irp then mark the current stack as
//         // pending.
//         //
//
//         if (Irp->PendingReturned) {
//             IoMarkIrpPending(Irp);
//         }
//
//         ClassReleaseRemoveLock(DeviceObject, Irp);
//         IoFreeMdl(Irp->MdlAddress);
//         IoFreeIrp(Irp);
//     }
//
//     return(STATUS_MORE_PROCESSING_REQUIRED);
// }
//
//
// NTSTATUS
// DiskInfoExceptionCheck(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// {
//     PUCHAR modeData;
//     PSCSI_REQUEST_BLOCK srb;
//     PCDB cdb;
//     PIRP irp;
//     PIO_STACK_LOCATION irpStack;
//     PVOID senseInfoBuffer = NULL;
//     UCHAR senseInfoBufferLength = 0;
//     ULONG isRemoved;
//     ULONG srbSize;
//     PSTORAGE_REQUEST_BLOCK srbEx = NULL;
//     PSTOR_ADDR_BTL8 storAddrBtl8 = NULL;
//     PSRBEX_DATA_SCSI_CDB16 srbExDataCdb16 = NULL;
//
//     modeData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                      MODE_DATA_SIZE,
//                                      DISK_TAG_INFO_EXCEPTION);
//     if (modeData == NULL)
//     {
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionCheck: Can't allocate mode data "
//                         "buffer\n"));
//         return(STATUS_INSUFFICIENT_RESOURCES);
//     }
//
//     if (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//         srbSize = CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE;
//     } else {
//         srbSize = SCSI_REQUEST_BLOCK_SIZE;
//     }
//     srb = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                 srbSize,
//                                 DISK_TAG_SRB);
//     if (srb == NULL)
//     {
//         FREE_POOL(modeData);
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionCheck: Can't allocate srb "
//                         "buffer\n"));
//         return(STATUS_INSUFFICIENT_RESOURCES);
//     }
//     RtlZeroMemory(srb, srbSize);
//
//     //
//     // Sense buffer is in aligned nonpaged pool.
//     //
//
//     senseInfoBuffer = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                             SENSE_BUFFER_SIZE_EX,
//                                             '7CcS');
//
//     if (senseInfoBuffer == NULL)
//     {
//         FREE_POOL(srb);
//         FREE_POOL(modeData);
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionCheck: Can't allocate request sense "
//                         "buffer\n"));
//         return(STATUS_INSUFFICIENT_RESOURCES);
//     }
//
//     senseInfoBufferLength = SENSE_BUFFER_SIZE_EX;
//
//     //
//     // Build device I/O control request with METHOD_NEITHER data transfer.
//     // We'll queue a completion routine to cleanup the MDL's and such ourself.
//     //
//
//     irp = IoAllocateIrp(
//             (CCHAR) (FdoExtension->CommonExtension.LowerDeviceObject->StackSize + 1),
//             FALSE);
//
//     if (irp == NULL)
//     {
//         FREE_POOL(senseInfoBuffer);
//         FREE_POOL(srb);
//         FREE_POOL(modeData);
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionCheck: Can't allocate Irp\n"));
//         return(STATUS_INSUFFICIENT_RESOURCES);
//     }
//
//     isRemoved = ClassAcquireRemoveLock(FdoExtension->DeviceObject, irp);
//
//     if (isRemoved)
//     {
//         ClassReleaseRemoveLock(FdoExtension->DeviceObject, irp);
//         IoFreeIrp(irp);
//         FREE_POOL(senseInfoBuffer);
//         FREE_POOL(srb);
//         FREE_POOL(modeData);
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskInfoExceptionCheck: RemoveLock says isRemoved\n"));
//         return(STATUS_DEVICE_DOES_NOT_EXIST);
//     }
//
//     //
//     // Get next stack location.
//     //
//
//     IoSetNextIrpStackLocation(irp);
//     irpStack = IoGetCurrentIrpStackLocation(irp);
//     irpStack->DeviceObject = FdoExtension->DeviceObject;
//
//     //
//     // Save retry count in current Irp stack.
//     //
//     irpStack->Parameters.Others.Argument4 = (PVOID)MAXIMUM_RETRIES;
//
//     //
//     // Save allocated sense info buffer in case the port driver overwrites it
//     //
//     irpStack->Parameters.Others.Argument3 = senseInfoBuffer;
//
//     irpStack = IoGetNextIrpStackLocation(irp);
//
//     //
//     // Set up SRB for execute scsi request. Save SRB address in next stack
//     // for the port driver.
//     //
//
//     irpStack->MajorFunction = IRP_MJ_SCSI;
//     irpStack->Parameters.Scsi.Srb = srb;
//
//     IoSetCompletionRoutine(irp,
//                            DiskInfoExceptionComplete,
//                            srb,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     irp->MdlAddress = IoAllocateMdl( modeData,
//                                      MODE_DATA_SIZE,
//                                      FALSE,
//                                      FALSE,
//                                      irp );
//     if (irp->MdlAddress == NULL)
//     {
//         ClassReleaseRemoveLock(FdoExtension->DeviceObject, irp);
//         FREE_POOL(srb);
//         FREE_POOL(modeData);
//         FREE_POOL(senseInfoBuffer);
//         IoFreeIrp( irp );
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskINfoExceptionCheck: Can't allocate MDL\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     MmBuildMdlForNonPagedPool(irp->MdlAddress);
//
//     //
//     // Build the MODE SENSE CDB.
//     //
//
//     if (FdoExtension->AdapterDescriptor->SrbType == SRB_TYPE_STORAGE_REQUEST_BLOCK) {
//
//         //
//         // Set up STORAGE_REQUEST_BLOCK fields
//         //
//
//         srbEx = (PSTORAGE_REQUEST_BLOCK)srb;
//         srbEx->Length = FIELD_OFFSET(STORAGE_REQUEST_BLOCK, Signature);
//         srbEx->Function = SRB_FUNCTION_STORAGE_REQUEST_BLOCK;
//         srbEx->Signature = SRB_SIGNATURE;
//         srbEx->Version = STORAGE_REQUEST_BLOCK_VERSION_1;
//         srbEx->SrbLength = srbSize;
//         srbEx->SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
//         srbEx->RequestPriority = IoGetIoPriorityHint(irp);
//         srbEx->AddressOffset = sizeof(STORAGE_REQUEST_BLOCK);
//         srbEx->NumSrbExData = 1;
//
//         // Set timeout value from device extension.
//         srbEx->TimeOutValue = FdoExtension->TimeOutValue;
//
//         // Set the transfer length.
//         srbEx->DataTransferLength = MODE_DATA_SIZE;
//         srbEx->DataBuffer = modeData;
//
//         srbEx->SrbFlags = FdoExtension->SrbFlags;
//
//         SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_DATA_IN);
//
//         //
//         // Disable synchronous transfer for these requests.
//         //
//         SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//
//         //
//         // Don't freeze the queue on an error
//         //
//         SET_FLAG(srbEx->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//
//         srbEx->RequestAttribute = SRB_SIMPLE_TAG_REQUEST;
//         srbEx->RequestTag = SP_UNTAGGED;
//
//         // Set up IRP Address.
//         srbEx->OriginalRequest = irp;
//
//         //
//         // Set up address fields
//         //
//
//         storAddrBtl8 = (PSTOR_ADDR_BTL8) ((PUCHAR)srbEx + srbEx->AddressOffset);
//         storAddrBtl8->Type = STOR_ADDRESS_TYPE_BTL8;
//         storAddrBtl8->AddressLength = STOR_ADDR_BTL8_ADDRESS_LENGTH;
//
//         //
//         // Set up SCSI SRB extended data fields
//         //
//
//         srbEx->SrbExDataOffset[0] = sizeof(STORAGE_REQUEST_BLOCK) +
//             sizeof(STOR_ADDR_BTL8);
//         if ((srbEx->SrbExDataOffset[0] + sizeof(SRBEX_DATA_SCSI_CDB16)) <= srbEx->SrbLength) {
//             srbExDataCdb16 = (PSRBEX_DATA_SCSI_CDB16)((PUCHAR)srbEx + srbEx->SrbExDataOffset[0]);
//             srbExDataCdb16->Type = SrbExDataTypeScsiCdb16;
//             srbExDataCdb16->Length = SRBEX_DATA_SCSI_CDB16_LENGTH;
//             srbExDataCdb16->CdbLength = 6;
//
//             // Enable auto request sense.
//             srbExDataCdb16->SenseInfoBufferLength = senseInfoBufferLength;
//             srbExDataCdb16->SenseInfoBuffer = senseInfoBuffer;
//
//             cdb = (PCDB)srbExDataCdb16->Cdb;
//         } else {
//             // Should not happen
//             NT_ASSERT(FALSE);
//
//             ClassReleaseRemoveLock(FdoExtension->DeviceObject, irp);
//             FREE_POOL(srb);
//             FREE_POOL(modeData);
//             FREE_POOL(senseInfoBuffer);
//             IoFreeIrp( irp );
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "DiskINfoExceptionCheck: Insufficient extended SRB size\n"));
//             return STATUS_INTERNAL_ERROR;
//         }
//
//     } else {
//
//         //
//         // Write length to SRB.
//         //
//         srb->Length = SCSI_REQUEST_BLOCK_SIZE;
//
//         //
//         // Set SCSI bus address.
//         //
//
//         srb->Function = SRB_FUNCTION_EXECUTE_SCSI;
//
//         //
//         // Enable auto request sense.
//         //
//
//         srb->SenseInfoBufferLength = senseInfoBufferLength;
//         srb->SenseInfoBuffer = senseInfoBuffer;
//
//         //
//         // Set timeout value from device extension.
//         //
//         srb->TimeOutValue = FdoExtension->TimeOutValue;
//
//         //
//         // Set the transfer length.
//         //
//         srb->DataTransferLength = MODE_DATA_SIZE;
//         srb->DataBuffer = modeData;
//
//         srb->SrbFlags = FdoExtension->SrbFlags;
//
//         SET_FLAG(srb->SrbFlags, SRB_FLAGS_DATA_IN);
//
//         //
//         // Disable synchronous transfer for these requests.
//         //
//         SET_FLAG(srb->SrbFlags, SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
//
//         //
//         // Don't freeze the queue on an error
//         //
//         SET_FLAG(srb->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//
//         srb->QueueAction = SRB_SIMPLE_TAG_REQUEST;
//         srb->QueueTag = SP_UNTAGGED;
//
//         //
//         // Set up IRP Address.
//         //
//         srb->OriginalRequest = irp;
//
//         srb->CdbLength = 6;
//         cdb = (PCDB)srb->Cdb;
//
//     }
//
//     cdb->MODE_SENSE.OperationCode = SCSIOP_MODE_SENSE;
//     cdb->MODE_SENSE.PageCode = MODE_PAGE_FAULT_REPORTING;
//     cdb->MODE_SENSE.AllocationLength = MODE_DATA_SIZE;
//
//     //
//     // Call the port driver with the request and wait for it to complete.
//     //
//
//     IoMarkIrpPending(irp);
//     IoCallDriver(FdoExtension->CommonExtension.LowerDeviceObject,
//                           irp);
//
//     return(STATUS_PENDING);
// }
//
//
// NTSTATUS
// DiskDetectFailurePrediction(
//     PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     PFAILURE_PREDICTION_METHOD FailurePredictCapability,
//     BOOLEAN ScsiAddressAvailable
//     )
// /*++
//
// Routine Description:
//
//     Detect if device has any failure prediction capabilities. First we
//     check for IDE SMART capability. This is done by sending the drive an
//     IDENTIFY command and checking if the SMART command set bit is set.
//
//     Next we check if SCSI SMART (aka Information Exception Control Page,
//     X3T10/94-190 Rev 4). This is done by querying for the Information
//     Exception mode page.
//
//     Lastly we check if the device has IOCTL failure prediction. This mechanism
//     a filter driver implements IOCTL_STORAGE_PREDICT_FAILURE and will respond
//     with the information in the IOCTL. We do this by sending the ioctl and
//     if the status returned is STATUS_SUCCESS we assume that it is supported.
//
// Arguments:
//
//     FdoExtension
//
//     *FailurePredictCapability
//
//     ScsiAddressAvailable TRUE if there is a valid SCSI_ADDRESS available
//                          for this device, FALSE otherwise.
//                          If FALSE we do not allow SMART IOCTLs (FailurePredictionSmart capability)
//                          which require a valid SCSI_ADDRESS. The other capabilities
//                          <FailurePredictionIoctl, FailurePredictionSense) do not requere
//                          SCSI_ADDRESS so we'll still try to initialize them.
//
// Return Value:
//
//     NT Status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = (PCOMMON_DEVICE_EXTENSION)FdoExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     BOOLEAN supportFP;
//     NTSTATUS status;
//     STORAGE_PREDICT_FAILURE checkFailure;
//     STORAGE_FAILURE_PREDICT_STATUS diskSmartStatus;
//
//     PAGED_CODE();
//
//     //
//     // Assume no failure predict mechanisms
//     //
//     *FailurePredictCapability = FailurePredictionNone;
//
//     //
//     // See if this is an IDE drive that supports SMART. If so enable SMART
//     // and then ensure that it suports the SMART READ STATUS command
//     //
//
//     if (ScsiAddressAvailable)
//     {
//         DiskGetIdentifyInfo(FdoExtension, &supportFP);
//
//         if (supportFP)
//         {
//             status = DiskEnableSmart(FdoExtension);
//             if (NT_SUCCESS(status))
//             {
//                 *FailurePredictCapability = FailurePredictionSmart;
//                 diskData->FailurePredictionEnabled = TRUE;
//
//                 status = DiskReadFailurePredictStatus(FdoExtension,
//                                                   &diskSmartStatus);
//
//                 TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: Device %p %s IDE SMART\n",
//                            FdoExtension->DeviceObject,
//                            NT_SUCCESS(status) ? "does" : "does not"));
//
//                 if (!NT_SUCCESS(status))
//                 {
//                     *FailurePredictCapability = FailurePredictionNone;
//                     diskData->FailurePredictionEnabled = FALSE;
//                 }
//             }
//             return(status);
//         }
//     }
//     //
//     // See if there is a a filter driver to intercept
//     // IOCTL_STORAGE_PREDICT_FAILURE
//     //
//     status = DiskSendFailurePredictIoctl(FdoExtension,
//                                          &checkFailure);
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: Device %p %s IOCTL_STORAGE_FAILURE_PREDICT\n",
//                        FdoExtension->DeviceObject,
//                        NT_SUCCESS(status) ? "does" : "does not"));
//
//     if (NT_SUCCESS(status))
//     {
//         *FailurePredictCapability = FailurePredictionIoctl;
//         diskData->FailurePredictionEnabled = TRUE;
//         if (checkFailure.PredictFailure)
//         {
//             checkFailure.PredictFailure = 512;
//             ClassNotifyFailurePredicted(FdoExtension,
//                                         (PUCHAR)&checkFailure,
//                                         sizeof(checkFailure),
//                                         (BOOLEAN)(FdoExtension->FailurePredicted == FALSE),
//                                         0x11,
//                                         diskData->ScsiAddress.PathId,
//                                         diskData->ScsiAddress.TargetId,
//                                         diskData->ScsiAddress.Lun);
//
//             FdoExtension->FailurePredicted = TRUE;
//         }
//         return(status);
//     }
//
//     //
//     // Finally we assume it will not be a scsi smart drive. but
//     // we'll also send off an asynchronous mode sense so that if
//     // it is SMART we'll reregister the device object
//     //
//
//     *FailurePredictCapability = FailurePredictionNone;
//
//     DiskInfoExceptionCheck(FdoExtension);
//
//     return(STATUS_SUCCESS);
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskWmiFunctionControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN CLASSENABLEDISABLEFUNCTION Function,
//     IN BOOLEAN Enable
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to enabled or disable event
//     generation or data block collection. A device should only expect a
//     single enable when the first event or data consumer enables events or
//     data collection and a single disable when the last event or data
//     consumer disables events or data collection. Data blocks will only
//     receive collection enable/disable if they were registered as requiring
//     it.
//
//
//     When NT boots, failure prediction is not automatically enabled, although
//     it may have been persistantly enabled on a previous boot. Polling is also
//     not automatically enabled. When the first data block that accesses SMART
//     such as SmartStatusGuid, SmartDataGuid, SmartPerformFunction, or
//     SmartEventGuid is accessed then SMART is automatically enabled in the
//     hardware. Polling is enabled when SmartEventGuid is enabled and disabled
//     when it is disabled. Hardware SMART is only disabled when the DisableSmart
//     method is called. Polling is also disabled when this is called regardless
//     of the status of the other guids or events.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     GuidIndex is the index into the list of guids provided when the
//         device registered
//
//     Function specifies which functionality is being enabled or disabled
//
//     Enable is TRUE then the function is being enabled else disabled
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS status = STATUS_SUCCESS;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//
//     PAGED_CODE();
//
//     if ((Function == DataBlockCollection) && Enable)
//     {
//         if ((GuidIndex == SmartStatusGuid) ||
//             (GuidIndex == SmartDataGuid) ||
//             (GuidIndex == SmartThresholdsGuid) ||
//             (GuidIndex == SmartPerformFunction))
//         {
//             status = DiskEnableDisableFailurePrediction(fdoExtension,
//                                                         TRUE);
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DeviceObject %p, Irp %p Enable -> %lx\n",
//                        DeviceObject,
//                        Irp,
//                        status));
//
//         } else {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DeviceObject %p, Irp %p, GuidIndex %d %s for Collection\n",
//                       DeviceObject, Irp,
//                       GuidIndex,
//                       Enable ? "Enabled" : "Disabled"));
//         }
//     } else if (Function == EventGeneration) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DeviceObject %p, Irp %p, GuidIndex %d %s for Event Generation\n",
//                   DeviceObject, Irp,
//                   GuidIndex,
//                   Enable ? "Enabled" : "Disabled"));
//
//
//         if ((GuidIndex == SmartEventGuid) && Enable)
//         {
//             status = DiskEnableDisableFailurePredictPolling(fdoExtension,
//                                                    Enable,
//                                                    0);
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DeviceObject %p, Irp %p %s -> %lx\n",
//                        DeviceObject,
//                        Irp,
//                        Enable ? "DiskEnableSmartPolling" : "DiskDisableSmartPolling",
//                        status));
//         }
//
// #if DBG
//     } else {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DeviceObject %p, Irp %p, GuidIndex %d %s for function %d\n",
//                   DeviceObject, Irp,
//                   GuidIndex,
//                   Enable ? "Enabled" : "Disabled",
//                   Function));
// #endif
//     }
//
//     status = ClassWmiCompleteRequest(DeviceObject,
//                                      Irp,
//                                      status,
//                                      0,
//                                      IO_NO_INCREMENT);
//     return status;
// }
//
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoQueryWmiRegInfo(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT ULONG *RegFlags,
//     OUT PUNICODE_STRING InstanceName
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to retrieve the list of
//     guids or data blocks that the driver wants to register with WMI. This
//     routine may not pend or block. Driver should NOT call
//     ClassWmiCompleteRequest.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     *RegFlags returns with a set of flags that describe the guids being
//         registered for this device. If the device wants enable and disable
//         collection callbacks before receiving queries for the registered
//         guids then it should return the WMIREG_FLAG_EXPENSIVE flag. Also the
//         returned flags may specify WMIREG_FLAG_INSTANCE_PDO in which case
//         the instance name is determined from the PDO associated with the
//         device object. Note that the PDO must have an associated devnode. If
//         WMIREG_FLAG_INSTANCE_PDO is not set then Name must return a unique
//         name for the device.
//
//     InstanceName returns with the instance name for the guids if
//         WMIREG_FLAG_INSTANCE_PDO is not set in the returned *RegFlags. The
//         caller will call ExFreePool with the buffer returned.
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(InstanceName);
//
//     SET_FLAG(DiskWmiFdoGuidList[SmartThresholdsGuid].Flags,  WMIREG_FLAG_REMOVE_GUID);
//     SET_FLAG(DiskWmiFdoGuidList[ScsiInfoExceptionsGuid].Flags,  WMIREG_FLAG_REMOVE_GUID);
//
//     switch (diskData->FailurePredictionCapability)
//     {
//         case FailurePredictionSmart:
//         {
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartThresholdsGuid].Flags,  WMIREG_FLAG_REMOVE_GUID);
//             //
//             // Fall Through
//             //
//         }
//         case FailurePredictionIoctl:
//         {
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartStatusGuid].Flags,      WMIREG_FLAG_REMOVE_GUID);
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartDataGuid].Flags,        WMIREG_FLAG_REMOVE_GUID);
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartEventGuid].Flags,       WMIREG_FLAG_REMOVE_GUID);
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartPerformFunction].Flags, WMIREG_FLAG_REMOVE_GUID);
//
//             break;
//         }
//
//         case FailurePredictionSense:
//         {
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartStatusGuid].Flags,      WMIREG_FLAG_REMOVE_GUID);
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartEventGuid].Flags,       WMIREG_FLAG_REMOVE_GUID);
//             CLEAR_FLAG(DiskWmiFdoGuidList[SmartPerformFunction].Flags, WMIREG_FLAG_REMOVE_GUID);
//             CLEAR_FLAG(DiskWmiFdoGuidList[ScsiInfoExceptionsGuid].Flags,  WMIREG_FLAG_REMOVE_GUID);
//             SET_FLAG  (DiskWmiFdoGuidList[SmartDataGuid].Flags,        WMIREG_FLAG_REMOVE_GUID);
//             break;
//         }
//
//
//         default:
//         {
//             SET_FLAG  (DiskWmiFdoGuidList[SmartStatusGuid].Flags,      WMIREG_FLAG_REMOVE_GUID);
//             SET_FLAG  (DiskWmiFdoGuidList[SmartDataGuid].Flags,        WMIREG_FLAG_REMOVE_GUID);
//             SET_FLAG  (DiskWmiFdoGuidList[SmartEventGuid].Flags,       WMIREG_FLAG_REMOVE_GUID);
//             SET_FLAG  (DiskWmiFdoGuidList[SmartPerformFunction].Flags, WMIREG_FLAG_REMOVE_GUID);
//             break;
//         }
//     }
//
//     //
//     // Use devnode for FDOs
//     *RegFlags = WMIREG_FLAG_INSTANCE_PDO;
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoQueryWmiRegInfoEx(
//     IN PDEVICE_OBJECT DeviceObject,
//     OUT ULONG *RegFlags,
//     OUT PUNICODE_STRING InstanceName,
//     OUT PUNICODE_STRING MofName
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to retrieve the list of
//     guids or data blocks that the driver wants to register with WMI. This
//     routine may not pend or block. Driver should NOT call
//     ClassWmiCompleteRequest.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     *RegFlags returns with a set of flags that describe the guids being
//         registered for this device. If the device wants enable and disable
//         collection callbacks before receiving queries for the registered
//         guids then it should return the WMIREG_FLAG_EXPENSIVE flag. Also the
//         returned flags may specify WMIREG_FLAG_INSTANCE_PDO in which case
//         the instance name is determined from the PDO associated with the
//         device object. Note that the PDO must have an associated devnode. If
//         WMIREG_FLAG_INSTANCE_PDO is not set then Name must return a unique
//         name for the device.
//
//     InstanceName returns with the instance name for the guids if
//         WMIREG_FLAG_INSTANCE_PDO is not set in the returned *RegFlags. The
//         caller will call ExFreePool with the buffer returned.
//
//     MofName returns initialized with the mof resource name for the
//         binary mof resource attached to the driver's image file. If the
//         driver does not have a mof resource then it should leave this
//         parameter untouched.
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS status;
//
//     UNREFERENCED_PARAMETER(MofName);
//
//     status = DiskFdoQueryWmiRegInfo(DeviceObject,
//                                     RegFlags,
//                                     InstanceName);
//
//     //
//     // Leave MofName alone since disk doesn't have one
//     //
//     return(status);
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoQueryWmiDataBlock(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN ULONG BufferAvail,
//     OUT PUCHAR Buffer
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to query for the contents of
//     a data block. When the driver has finished filling the data block it
//     must call ClassWmiCompleteRequest to complete the irp. The driver can
//     return STATUS_PENDING if the irp cannot be completed immediately.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     Irp is the Irp that makes this request
//
//     GuidIndex is the index into the list of guids provided when the
//         device registered
//
//     BufferAvail on has the maximum size available to write the data
//         block.
//
//     Buffer on return is filled with the returned data block
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS status;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     ULONG sizeNeeded;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskQueryWmiDataBlock, Device %p, Irp %p, GuiIndex %d\n"
//              "      BufferAvail %lx Buffer %p\n",
//              DeviceObject, Irp,
//              GuidIndex, BufferAvail, Buffer));
//
//     switch (GuidIndex)
//     {
//         case DiskGeometryGuid:
//         {
//             sizeNeeded = sizeof(DISK_GEOMETRY);
//             if (BufferAvail >= sizeNeeded)
//             {
//                 if (DeviceObject->Characteristics & FILE_REMOVABLE_MEDIA)
//                 {
//                     //
//                     // Issue ReadCapacity to update device extension
//                     // with information for current media.
//                     status = DiskReadDriveCapacity(commonExtension->PartitionZeroExtension->DeviceObject);
//
//                     //
//                     // Note whether the drive is ready.
//                     diskData->ReadyStatus = status;
//
//                     if (!NT_SUCCESS(status))
//                     {
//                         break;
//                     }
//                 }
//
//                 //
//                 // Copy drive geometry information from device extension.
//                 RtlMoveMemory(Buffer,
//                               &(fdoExtension->DiskGeometry),
//                               sizeof(DISK_GEOMETRY));
//
//                 status = STATUS_SUCCESS;
//             } else {
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//             break;
//         }
//
//         case SmartStatusGuid:
//         {
//             PSTORAGE_FAILURE_PREDICT_STATUS diskSmartStatus;
//
//             NT_ASSERT(diskData->FailurePredictionCapability != FailurePredictionNone);
//
//             sizeNeeded = sizeof(STORAGE_FAILURE_PREDICT_STATUS);
//             if (BufferAvail >= sizeNeeded)
//             {
//                 STORAGE_PREDICT_FAILURE checkFailure;
//
//                 diskSmartStatus = (PSTORAGE_FAILURE_PREDICT_STATUS)Buffer;
//
//                 status = DiskSendFailurePredictIoctl(fdoExtension,
//                                                      &checkFailure);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     if (diskData->FailurePredictionCapability ==
//                                                       FailurePredictionSense)
//                     {
//                         diskSmartStatus->Reason =  *((PULONG)checkFailure.VendorSpecific);
//                     } else {
//                         diskSmartStatus->Reason =  0; // unknown
//                     }
//
//                     diskSmartStatus->PredictFailure = (checkFailure.PredictFailure != 0);
//                 }
//             } else {
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//             break;
//         }
//
//         case SmartDataGuid:
//         {
//             PSTORAGE_FAILURE_PREDICT_DATA diskSmartData;
//
//             NT_ASSERT((diskData->FailurePredictionCapability ==
//                                                   FailurePredictionSmart) ||
//                    (diskData->FailurePredictionCapability ==
//                                                   FailurePredictionIoctl));
//
//             sizeNeeded = sizeof(STORAGE_FAILURE_PREDICT_DATA);
//             if (BufferAvail >= sizeNeeded)
//             {
//                 PSTORAGE_PREDICT_FAILURE checkFailure = (PSTORAGE_PREDICT_FAILURE)Buffer;
//
//                 diskSmartData = (PSTORAGE_FAILURE_PREDICT_DATA)Buffer;
//
//                 status = DiskSendFailurePredictIoctl(fdoExtension,
//                                                      checkFailure);
//
//                 if (NT_SUCCESS(status))
//                 {
//                     diskSmartData->Length = 512;
//                 }
//             } else {
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//
//             break;
//         }
//
//         case SmartThresholdsGuid:
//         {
//             PSTORAGE_FAILURE_PREDICT_THRESHOLDS diskSmartThresholds;
//
//             NT_ASSERT((diskData->FailurePredictionCapability ==
//                                                   FailurePredictionSmart));
//
//             sizeNeeded = sizeof(STORAGE_FAILURE_PREDICT_THRESHOLDS);
//             if (BufferAvail >= sizeNeeded)
//             {
//                 diskSmartThresholds = (PSTORAGE_FAILURE_PREDICT_THRESHOLDS)Buffer;
//                 status = DiskReadFailurePredictThresholds(fdoExtension,
//                                                           diskSmartThresholds);
//             } else {
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//
//             break;
//         }
//
//         case SmartPerformFunction:
//         {
//             sizeNeeded = 0;
//             status = STATUS_SUCCESS;
//             break;
//         }
//
//         case ScsiInfoExceptionsGuid:
//         {
//             PSTORAGE_SCSI_INFO_EXCEPTIONS infoExceptions;
//             MODE_INFO_EXCEPTIONS modeInfo;
//
//             NT_ASSERT((diskData->FailurePredictionCapability ==
//                                                   FailurePredictionSense));
//
//             sizeNeeded = sizeof(STORAGE_SCSI_INFO_EXCEPTIONS);
//             if (BufferAvail >= sizeNeeded)
//             {
//                 infoExceptions = (PSTORAGE_SCSI_INFO_EXCEPTIONS)Buffer;
//                 status = DiskGetInfoExceptionInformation(fdoExtension,
//                                                          &modeInfo);
//                 if (NT_SUCCESS(status))
//                 {
//                     infoExceptions->PageSavable = modeInfo.PSBit;
//                     infoExceptions->Flags = modeInfo.Flags;
//                     infoExceptions->MRIE = modeInfo.ReportMethod;
//                     infoExceptions->Padding = 0;
//                     REVERSE_BYTES(&infoExceptions->IntervalTimer,
//                                   &modeInfo.IntervalTimer);
//                     REVERSE_BYTES(&infoExceptions->ReportCount,
//                                   &modeInfo.ReportCount)
//                 }
//             } else {
//                 status = STATUS_BUFFER_TOO_SMALL;
//             }
//
//             break;
//         }
//
//         default:
//         {
//             sizeNeeded = 0;
//             status = STATUS_WMI_GUID_NOT_FOUND;
//         }
//     }
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskQueryWmiDataBlock Device %p, Irp %p returns %lx\n",
//              DeviceObject, Irp, status));
//
//     status = ClassWmiCompleteRequest(DeviceObject,
//                                      Irp,
//                                      status,
//                                      sizeNeeded,
//                                      IO_NO_INCREMENT);
//
//     return status;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoSetWmiDataBlock(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN ULONG BufferSize,
//     IN PUCHAR Buffer
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to query for the contents of
//     a data block. When the driver has finished filling the data block it
//     must call ClassWmiCompleteRequest to complete the irp. The driver can
//     return STATUS_PENDING if the irp cannot be completed immediately.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     Irp is the Irp that makes this request
//
//     GuidIndex is the index into the list of guids provided when the
//         device registered
//
//     BufferSize has the size of the data block passed
//
//     Buffer has the new values for the data block
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS status;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskSetWmiDataBlock, Device %p, Irp %p, GuiIndex %d\n"
//              "      BufferSize %#x Buffer %p\n",
//              DeviceObject, Irp,
//              GuidIndex, BufferSize, Buffer));
//
//     if (GuidIndex == ScsiInfoExceptionsGuid)
//     {
//         PSTORAGE_SCSI_INFO_EXCEPTIONS infoExceptions;
//         MODE_INFO_EXCEPTIONS modeInfo = {0};
//
//         if (BufferSize >= sizeof(STORAGE_SCSI_INFO_EXCEPTIONS))
//         {
//             infoExceptions = (PSTORAGE_SCSI_INFO_EXCEPTIONS)Buffer;
//
//             modeInfo.PageCode = MODE_PAGE_FAULT_REPORTING;
//             modeInfo.PageLength = sizeof(MODE_INFO_EXCEPTIONS) - 2;
//
//             modeInfo.PSBit = 0;
//             modeInfo.Flags = infoExceptions->Flags;
//
//             modeInfo.ReportMethod = infoExceptions->MRIE;
//
//             REVERSE_BYTES(&modeInfo.IntervalTimer[0],
//                           &infoExceptions->IntervalTimer);
//
//             REVERSE_BYTES(&modeInfo.ReportCount[0],
//                           &infoExceptions->ReportCount);
//
//             if (modeInfo.Perf == 1)
//             {
//                 diskData->AllowFPPerfHit = FALSE;
//             } else {
//                 diskData->AllowFPPerfHit = TRUE;
//             }
//
//             status = DiskSetInfoExceptionInformation(fdoExtension,
//                                                      &modeInfo);
//         } else {
//             status = STATUS_INVALID_PARAMETER;
//         }
//
//     } else if (GuidIndex <= SmartThresholdsGuid)
//     {
//         status = STATUS_WMI_READ_ONLY;
//     } else {
//         status = STATUS_WMI_GUID_NOT_FOUND;
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskSetWmiDataBlock Device %p, Irp %p returns %lx\n",
//              DeviceObject, Irp, status));
//
//     status = ClassWmiCompleteRequest(DeviceObject,
//                                      Irp,
//                                      status,
//                                      0,
//                                      IO_NO_INCREMENT);
//
//     return status;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoSetWmiDataItem(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN ULONG DataItemId,
//     IN ULONG BufferSize,
//     IN PUCHAR Buffer
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to query for the contents of
//     a data block. When the driver has finished filling the data block it
//     must call ClassWmiCompleteRequest to complete the irp. The driver can
//     return STATUS_PENDING if the irp cannot be completed immediately.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     Irp is the Irp that makes this request
//
//     GuidIndex is the index into the list of guids provided when the
//         device registered
//
//     DataItemId has the id of the data item being set
//
//     BufferSize has the size of the data item passed
//
//     Buffer has the new values for the data item
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskSetWmiDataItem, Device %p, Irp %p, GuiIndex %d, DataId %d\n"
//              "      BufferSize %#x Buffer %p\n",
//              DeviceObject, Irp,
//              GuidIndex, DataItemId, BufferSize, Buffer));
//
//     if (GuidIndex <= SmartThresholdsGuid)
//     {
//         status = STATUS_WMI_READ_ONLY;
//     } else {
//         status = STATUS_WMI_GUID_NOT_FOUND;
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskSetWmiDataItem Device %p, Irp %p returns %lx\n",
//              DeviceObject, Irp, status));
//
//     status = ClassWmiCompleteRequest(DeviceObject,
//                                      Irp,
//                                      status,
//                                      0,
//                                      IO_NO_INCREMENT);
//
//     return status;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskFdoExecuteWmiMethod(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN ULONG MethodId,
//     IN ULONG InBufferSize,
//     IN ULONG OutBufferSize,
//     IN PUCHAR Buffer
//     )
// /*++
//
// Routine Description:
//
//     This routine is a callback into the driver to execute a method. When the
//     driver has finished filling the data block it must call
//     ClassWmiCompleteRequest to complete the irp. The driver can
//     return STATUS_PENDING if the irp cannot be completed immediately.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     Irp is the Irp that makes this request
//
//     GuidIndex is the index into the list of guids provided when the
//         device registered
//
//     MethodId has the id of the method being called
//
//     InBufferSize has the size of the data block passed in as the input to
//         the method.
//
//     OutBufferSize on entry has the maximum size available to write the
//         returned data block.
//
//     Buffer is filled with the returned data block
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA)(commonExtension->DriverData);
//     ULONG sizeNeeded = 0;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskExecuteWmiMethod, DeviceObject %p, Irp %p, Guid Id %d, MethodId %d\n"
//              "      InBufferSize %#x, OutBufferSize %#x, Buffer %p\n",
//              DeviceObject, Irp,
//              GuidIndex, MethodId, InBufferSize, OutBufferSize, Buffer));
//
//     switch(GuidIndex)
//     {
//         case SmartPerformFunction:
//         {
//
//             NT_ASSERT((diskData->FailurePredictionCapability ==
//                                                   FailurePredictionSmart) ||
//                    (diskData->FailurePredictionCapability ==
//                                                   FailurePredictionIoctl) ||
//                    (diskData->FailurePredictionCapability ==
//                                                   FailurePredictionSense));
//
//
//             switch(MethodId)
//             {
//                 //
//                 // void AllowPerformanceHit([in] boolean Allow)
//                 //
//                 case AllowDisallowPerformanceHit:
//                 {
//                     BOOLEAN allowPerfHit;
//
//                     sizeNeeded = 0;
//                     if (InBufferSize >= sizeof(BOOLEAN))
//                     {
//                         status = STATUS_SUCCESS;
//
//                         allowPerfHit = *((PBOOLEAN)Buffer);
//                         if (diskData->AllowFPPerfHit != allowPerfHit)
//                         {
//                             diskData->AllowFPPerfHit = allowPerfHit;
//                             if (diskData->FailurePredictionCapability ==
//                                 FailurePredictionSense)
//                             {
//                                 MODE_INFO_EXCEPTIONS modeInfo;
//
//                                 status = DiskGetInfoExceptionInformation(fdoExtension,
//                                                                          &modeInfo);
//                                 if (NT_SUCCESS(status))
//                                 {
//                                     modeInfo.Perf = allowPerfHit ? 0 : 1;
//                                     status = DiskSetInfoExceptionInformation(fdoExtension,
//                                                                              &modeInfo);
//                                 }
//                             }
//                             else
//                             {
//                                 status = STATUS_INVALID_DEVICE_REQUEST;
//                             }
//                         }
//
//                         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskFdoWmiExecuteMethod: AllowPerformanceHit %x for device %p --> %lx\n",
//                                     allowPerfHit,
//                                     fdoExtension->DeviceObject,
//                                     status));
//                     } else {
//                         status = STATUS_INVALID_PARAMETER;
//                     }
//                     break;
//                 }
//
//                 //
//                 // void EnableDisableHardwareFailurePrediction([in] boolean Enable)
//                 //
//                 case EnableDisableHardwareFailurePrediction:
//                 {
//                     BOOLEAN enable;
//
//                     sizeNeeded = 0;
//                     if (InBufferSize >= sizeof(BOOLEAN))
//                     {
//                         status = STATUS_SUCCESS;
//                         enable = *((PBOOLEAN)Buffer);
//                         if (!enable)
//                         {
//                             //
//                             // If we are disabling we need to also disable
//                             // polling
//                             //
//                             DiskEnableDisableFailurePredictPolling(
//                                                                fdoExtension,
//                                                                enable,
//                                                                0);
//                         }
//
//                         status = DiskEnableDisableFailurePrediction(
//                                                            fdoExtension,
//                                                            enable);
//
//                         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskFdoWmiExecuteMethod: EnableDisableHardwareFailurePrediction: %x for device %p --> %lx\n",
//                                     enable,
//                                     fdoExtension->DeviceObject,
//                                     status));
//                     } else {
//                         status = STATUS_INVALID_PARAMETER;
//                     }
//                     break;
//                 }
//
//                 //
//                 // void EnableDisableFailurePredictionPolling(
//                 //                               [in] uint32 Period,
//                 //                               [in] boolean Enable)
//                 //
//                 case EnableDisableFailurePredictionPolling:
//                 {
//                     BOOLEAN enable;
//                     ULONG period;
//
//                     sizeNeeded = 0;
//                     if (InBufferSize >= (sizeof(ULONG) + sizeof(BOOLEAN)))
//                     {
//                         period = *((PULONG)Buffer);
//                         Buffer += sizeof(ULONG);
//                         enable = *((PBOOLEAN)Buffer);
//
//                            status = DiskEnableDisableFailurePredictPolling(
//                                                                fdoExtension,
//                                                                enable,
//                                                                period);
//
//                            TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskFdoWmiExecuteMethod: EnableDisableFailurePredictionPolling: %x %x for device %p --> %lx\n",
//                                     enable,
//                                     period,
//                                     fdoExtension->DeviceObject,
//                                     status));
//                     } else {
//                         status = STATUS_INVALID_PARAMETER;
//                     }
//                     break;
//                 }
//
//                 //
//                 // void GetFailurePredictionCapability([out] uint32 Capability)
//                 //
//                 case GetFailurePredictionCapability:
//                 {
//                     sizeNeeded = sizeof(ULONG);
//                     if (OutBufferSize >= sizeNeeded)
//                     {
//                         status = STATUS_SUCCESS;
//                         *((PFAILURE_PREDICTION_METHOD)Buffer) = diskData->FailurePredictionCapability;
//                         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskFdoWmiExecuteMethod: GetFailurePredictionCapability: %x for device %p --> %lx\n",
//                                     *((PFAILURE_PREDICTION_METHOD)Buffer),
//                                     fdoExtension->DeviceObject,
//                                     status));
//                     } else {
//                         status = STATUS_BUFFER_TOO_SMALL;
//                     }
//                     break;
//                 }
//
//                 //
//                 // void EnableOfflineDiags([out] boolean Success);
//                 //
//                 case EnableOfflineDiags:
//                 {
//                     sizeNeeded = sizeof(BOOLEAN);
//                     if (OutBufferSize >= sizeNeeded)
//                     {
//                         if (diskData->FailurePredictionCapability ==
//                                   FailurePredictionSmart)
//                         {
//                             //
//                             // Initiate or resume offline diagnostics.
//                             // This may cause a loss of performance
//                             // to the disk, but mayincrease the amount
//                             // of disk checking.
//                             //
//                             status = DiskExecuteSmartDiagnostics(fdoExtension,
//                                                                 0);
//
//                         } else {
//                             status = STATUS_INVALID_DEVICE_REQUEST;
//                         }
//
//                         *((PBOOLEAN)Buffer) = NT_SUCCESS(status);
//
//                         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "DiskFdoWmiExecuteMethod: EnableOfflineDiags for device %p --> %lx\n",
//                                     fdoExtension->DeviceObject,
//                                     status));
//                     } else {
//                         status = STATUS_BUFFER_TOO_SMALL;
//                     }
//                     break;
//                 }
//
//                 //
//                 //    void ReadLogSectors([in] uint8 LogAddress,
//                 //        [in] uint8 SectorCount,
//                 //        [out] uint32 Length,
//                 //        [out, WmiSizeIs("Length")] uint8 LogSectors[]
//                 //       );
//                 //
//                 case ReadLogSectors:
//                 {
//                     if (diskData->FailurePredictionCapability ==
//                                   FailurePredictionSmart)
//                     {
//                         if (InBufferSize >= sizeof(READ_LOG_SECTORS_IN))
//                         {
//                             PREAD_LOG_SECTORS_IN inParams;
//                             PREAD_LOG_SECTORS_OUT outParams;
//                             ULONG readSize;
//
//                             inParams = (PREAD_LOG_SECTORS_IN)Buffer;
//                             readSize = inParams->SectorCount * SMART_LOG_SECTOR_SIZE;
//                             sizeNeeded = FIELD_OFFSET(READ_LOG_SECTORS_OUT,
//                                                   LogSectors) + readSize;
//
//                             if (OutBufferSize >= sizeNeeded)
//                             {
//                                 outParams = (PREAD_LOG_SECTORS_OUT)Buffer;
//                                 status = DiskReadSmartLog(fdoExtension,
//                                                         inParams->SectorCount,
//                                                         inParams->LogAddress,
//                                                         outParams->LogSectors);
//
//                                 if (NT_SUCCESS(status))
//                                 {
//                                     outParams->Length = readSize;
//                                 } else {
//                                     //
//                                     // SMART command failure is
//                                     // indicated by successful
//                                     // execution, but no data returned
//                                     //
//                                     outParams->Length = 0;
//                                     status = STATUS_SUCCESS;
//                                 }
//                             } else {
//                                 status = STATUS_BUFFER_TOO_SMALL;
//                             }
//
//                         } else {
//                             status = STATUS_INVALID_PARAMETER;
//                         }
//                     } else {
//                         status = STATUS_INVALID_DEVICE_REQUEST;
//                     }
//                     break;
//                 }
//
//                 //    void WriteLogSectors([in] uint8 LogAddress,
//                 //        [in] uint8 SectorCount,
//                 //        [in] uint32 Length,
//                 //        [in, WmiSizeIs("Length")] uint8 LogSectors[],
//                 //        [out] boolean Success
//                 //       );
//                 case WriteLogSectors:
//                 {
//                     if (diskData->FailurePredictionCapability ==
//                                   FailurePredictionSmart)
//                     {
//                         if ((LONG)InBufferSize >= FIELD_OFFSET(WRITE_LOG_SECTORS_IN,
//                                                         LogSectors))
//                         {
//                             PWRITE_LOG_SECTORS_IN inParams;
//                             PWRITE_LOG_SECTORS_OUT outParams;
//                             ULONG writeSize;
//
//                             inParams = (PWRITE_LOG_SECTORS_IN)Buffer;
//                             writeSize = inParams->SectorCount * SMART_LOG_SECTOR_SIZE;
//                             if (InBufferSize >= (FIELD_OFFSET(WRITE_LOG_SECTORS_IN,
//                                                              LogSectors) +
//                                                  writeSize))
//                             {
//                                 sizeNeeded = sizeof(WRITE_LOG_SECTORS_OUT);
//
//                                 if (OutBufferSize >= sizeNeeded)
//                                 {
//                                     outParams = (PWRITE_LOG_SECTORS_OUT)Buffer;
//                                     status = DiskWriteSmartLog(fdoExtension,
//                                                         inParams->SectorCount,
//                                                         inParams->LogAddress,
//                                                         inParams->LogSectors);
//
//                                     if (NT_SUCCESS(status))
//                                     {
//                                         outParams->Success = TRUE;
//                                     } else {
//                                         outParams->Success = FALSE;
//                                         status = STATUS_SUCCESS;
//                                     }
//                                 } else {
//                                     status = STATUS_BUFFER_TOO_SMALL;
//                                 }
//                             } else {
//                                 status = STATUS_INVALID_PARAMETER;
//                             }
//                         } else {
//                             status = STATUS_INVALID_PARAMETER;
//                         }
//                     } else {
//                         status = STATUS_INVALID_DEVICE_REQUEST;
//                     }
//                     break;
//                 }
//
//                 //    void ExecuteSelfTest([in] uint8 Subcommand,
//                 //         [out,
//                 //          Values{"0", "1", "2"},
//                 //          ValueMap{"Successful Completion",
//                 //                   "Captive Mode Required",
//                 //                   "Unsuccessful Completion"}
//                 //         ]
//                 //         uint32 ReturnCode);
//                 case ExecuteSelfTest:
//                 {
//                     if (diskData->FailurePredictionCapability ==
//                               FailurePredictionSmart)
//                     {
//                         if (InBufferSize >= sizeof(EXECUTE_SELF_TEST_IN))
//                         {
//                             sizeNeeded = sizeof(EXECUTE_SELF_TEST_OUT);
//                             if (OutBufferSize >= sizeNeeded)
//                             {
//                                 PEXECUTE_SELF_TEST_IN inParam;
//                                 PEXECUTE_SELF_TEST_OUT outParam;
//
//                                 inParam = (PEXECUTE_SELF_TEST_IN)Buffer;
//                                 outParam = (PEXECUTE_SELF_TEST_OUT)Buffer;
//
//                                 if (DiskIsValidSmartSelfTest(inParam->Subcommand))
//                                 {
//                                    status = DiskExecuteSmartDiagnostics(fdoExtension,
//                                                             inParam->Subcommand);
//                                    if (NT_SUCCESS(status))
//                                    {
//                                        //
//                                        // Return self test executed
//                                        // without a problem
//                                        //
//                                        outParam->ReturnCode = 0;
//                                    } else {
//                                        //
//                                        // Return Self test execution
//                                        // failed status
//                                        //
//                                        outParam->ReturnCode = 2;
//                                        status = STATUS_SUCCESS;
//                                    }
//                                 } else {
//                                     //
//                                     // If self test subcommand requires
//                                     // captive mode then return that
//                                     // status
//                                     //
//                                     outParam->ReturnCode = 1;
//                                     status = STATUS_SUCCESS;
//                                 }
//
//                             } else {
//                                 status = STATUS_BUFFER_TOO_SMALL;
//                             }
//
//                         } else {
//                             status = STATUS_INVALID_PARAMETER;
//                         }
//                     } else {
//                         status = STATUS_INVALID_DEVICE_REQUEST;
//                     }
//
//                     break;
//                 }
//
//                 default :
//                 {
//                     sizeNeeded = 0;
//                     status = STATUS_WMI_ITEMID_NOT_FOUND;
//                     break;
//                 }
//             }
//
//             break;
//         }
//
//         case DiskGeometryGuid:
//         case SmartStatusGuid:
//         case SmartDataGuid:
//         case SmartEventGuid:
//         case SmartThresholdsGuid:
//         case ScsiInfoExceptionsGuid:
//         {
//             sizeNeeded = 0;
//             status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//         }
//
//         default:
//         {
//             sizeNeeded = 0;
//             status = STATUS_WMI_GUID_NOT_FOUND;
//         }
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_WMI, "Disk: DiskExecuteMethod Device %p, Irp %p returns %lx\n",
//              DeviceObject, Irp, status));
//
//     status = ClassWmiCompleteRequest(DeviceObject,
//                                      Irp,
//                                      status,
//                                      sizeNeeded,
//                                      IO_NO_INCREMENT);
//
//     return status;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
